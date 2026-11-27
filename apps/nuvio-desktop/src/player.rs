#[cfg(target_os = "linux")]
use crate::player_overlay_linux::{
    LinuxOverlayHandle, LinuxOverlayShared, OVERLAY_REPLY_TAG, schedule_x11_child_cleanup,
};
use crate::skip::SkipChapter;
use libloading::{Library, Symbol};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_double, c_int, c_void};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
#[cfg(target_os = "windows")]
use tauri::Manager;
use tauri::{AppHandle, Emitter, WebviewWindow};

const MPV_FORMAT_NONE: c_int = 0;
const MPV_FORMAT_STRING: c_int = 1;
const MPV_FORMAT_FLAG: c_int = 3;
const MPV_FORMAT_INT64: c_int = 4;
const MPV_FORMAT_DOUBLE: c_int = 5;
const MPV_FORMAT_NODE: c_int = 6;
const MPV_FORMAT_NODE_ARRAY: c_int = 7;

const MPV_EVENT_NONE: c_int = 0;
const MPV_EVENT_SHUTDOWN: c_int = 1;
const MPV_EVENT_COMMAND_REPLY: c_int = 5;
const MPV_EVENT_START_FILE: c_int = 6;
const MPV_EVENT_END_FILE: c_int = 7;
const MPV_EVENT_PLAYBACK_RESTART: c_int = 21;
const MPV_EVENT_PROPERTY_CHANGE: c_int = 22;

#[repr(C)]
struct MpvEvent {
    event_id: c_int,
    error: c_int,
    reply_userdata: u64,
    data: *mut c_void,
}

#[repr(C)]
struct MpvEventProperty {
    name: *const c_char,
    format: c_int,
    data: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
union MpvNodeValue {
    string: *const c_char,
    flag: c_int,
    int64: i64,
    double_: c_double,
    list: *mut MpvNodeList,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct MpvNode {
    u: MpvNodeValue,
    format: c_int,
}

#[repr(C)]
struct MpvNodeList {
    num: c_int,
    values: *mut MpvNode,
    keys: *mut *mut c_char,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayTrack {
    pub index: usize,
    pub id: String,
    pub label: String,
    pub language: String,
    pub selected: bool,
    pub forced: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct MediaTrack {
    pub external: bool,
    pub id: i64,
    pub track_type: String,
    pub title: String,
    pub lang: String,
    pub codec: String,
    pub channels: String,
    pub selected: bool,
    pub forced: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerMediaInfo {
    pub width: i64,
    pub height: i64,
    pub video_codec: String,
    pub hwdec: String,
    pub audio_codec: String,
    pub audio_channels: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerStatePayload {
    pub active: bool,
    pub paused: bool,
    pub buffering: bool,
    pub loading: bool,
    pub ended: bool,
    pub position_secs: f64,
    pub duration_secs: f64,
    pub buffered_secs: f64,
    pub volume: f64,
    pub speed: f64,
    pub audio_tracks: Vec<MediaTrack>,
    pub subtitle_tracks: Vec<MediaTrack>,
    pub reference_id: i64,
    pub reference_text: String,
    pub reference_start_secs: f64,
    pub chapters: Vec<SkipChapter>,
    pub media_info: PlayerMediaInfo,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerUiEventPayload {
    pub event_type: String,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerMetrics {
    pub hwdec_current: String,
    pub video_codec: String,
    pub pixel_format: String,
    pub audio_codec: String,
    pub cache_secs: f64,
    pub width: i64,
    pub height: i64,
    pub fps: f64,
    pub dropped_frames: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleStylePayload {
    pub font_size_sp: f64,
    pub outline_enabled: bool,
    pub bold: bool,
    pub bottom_offset: i32,
    pub text_color: String,
    pub outline_color: String,
    pub background_color: String,
    pub shadow_enabled: bool,
    pub shadow_opacity: f64,
    pub shadow_size: f64,
    pub font_name: Option<String>,
}

type MpvCreateFn = unsafe extern "C" fn() -> *mut c_void;
type MpvLoadConfigFn = unsafe extern "C" fn(*mut c_void, *const c_char) -> c_int;
type MpvInitializeFn = unsafe extern "C" fn(*mut c_void) -> c_int;
type MpvSetOptionFn =
    unsafe extern "C" fn(*mut c_void, *const c_char, c_int, *mut c_void) -> c_int;
type MpvSetOptionStringFn =
    unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char) -> c_int;
type MpvCommandFn = unsafe extern "C" fn(*mut c_void, *const *const c_char) -> c_int;
type MpvCommandAsyncFn = unsafe extern "C" fn(*mut c_void, u64, *const *const c_char) -> c_int;
type MpvSetPropertyStringFn =
    unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char) -> c_int;
type MpvGetPropertyFn =
    unsafe extern "C" fn(*mut c_void, *const c_char, c_int, *mut c_void) -> c_int;
type MpvObservePropertyFn =
    unsafe extern "C" fn(*mut c_void, u64, *const c_char, c_int) -> c_int;
type MpvWaitEventFn = unsafe extern "C" fn(*mut c_void, c_double) -> *const MpvEvent;
type MpvWakeupFn = unsafe extern "C" fn(*mut c_void);
type MpvFreeFn = unsafe extern "C" fn(*mut c_void);
type MpvTerminateDestroyFn = unsafe extern "C" fn(*mut c_void);

#[derive(Clone)]
struct MpvApi {
    _lib: Arc<Library>,
    create: MpvCreateFn,
    initialize: MpvInitializeFn,
    load_config: MpvLoadConfigFn,
    set_option: MpvSetOptionFn,
    set_option_string: MpvSetOptionStringFn,
    command: MpvCommandFn,
    command_async: MpvCommandAsyncFn,
    set_property_string: MpvSetPropertyStringFn,
    get_property: MpvGetPropertyFn,
    observe_property: MpvObservePropertyFn,
    wait_event: MpvWaitEventFn,
    wakeup: MpvWakeupFn,
    free: MpvFreeFn,
    terminate_destroy: MpvTerminateDestroyFn,
}

include!(concat!(env!("OUT_DIR"), "/embedded_libmpv.rs"));

impl MpvApi {
    fn load() -> Result<Self, String> {
        let mut candidates: Vec<std::ffi::OsString> = Vec::new();
        if cfg!(target_os = "windows") {
            if let Some(env_path) = std::env::var_os("NUVIO_LIBMPV_PATH")
                && !env_path.is_empty()
            {
                candidates.push(env_path);
            }
            if let Ok(exe_path) = std::env::current_exe()
                && let Some(exe_dir) = exe_path.parent()
            {
                candidates.push(exe_dir.join("libmpv-2.dll").into_os_string());
                candidates.push(exe_dir.join("mpv-2.dll").into_os_string());
                candidates.push(exe_dir.join("runtime").join("libmpv-2.dll").into_os_string());
                candidates.push(exe_dir.join("native").join("libmpv-2.dll").into_os_string());
            }
            if let Some(gz_bytes) = EMBEDDED_LIBMPV_GZ
                && let Ok(extracted) = ensure_embedded_windows_libmpv(gz_bytes)
            {
                candidates.push(extracted.into_os_string());
            }
            candidates.push("libmpv-2.dll".into());
            candidates.push("mpv-2.dll".into());
            candidates.push(r"C:\Program Files (x86)\Nuvio\app\native\libmpv-2.dll".into());
            candidates.push(r"C:\Program Files\Nuvio\app\native\libmpv-2.dll".into());
            candidates.push(r"C:\msys64\ucrt64\bin\libmpv-2.dll".into());
        } else if cfg!(target_os = "macos") {
            let _ = EMBEDDED_LIBMPV_GZ;
            candidates.push("libmpv.2.dylib".into());
            candidates.push("libmpv.dylib".into());
        } else {
            let _ = EMBEDDED_LIBMPV_GZ;
            candidates.push("libmpv.so.2".into());
            candidates.push("libmpv.so".into());
        }

        let mut last_err = String::new();
        crate::crash_log::note("player: a carregar libmpv");
        for name in &candidates {
            match unsafe { Library::new(name) } {
                Ok(lib) => unsafe {
                    let create: Symbol<MpvCreateFn> =
                        lib.get(b"mpv_create\0").map_err(|e| e.to_string())?;
                    let initialize: Symbol<MpvInitializeFn> =
                        lib.get(b"mpv_initialize\0").map_err(|e| e.to_string())?;
                    let set_option: Symbol<MpvSetOptionFn> =
                        lib.get(b"mpv_set_option\0").map_err(|e| e.to_string())?;
                    let set_option_string: Symbol<MpvSetOptionStringFn> = lib
                        .get(b"mpv_set_option_string\0")
                        .map_err(|e| e.to_string())?;
                    let command: Symbol<MpvCommandFn> =
                        lib.get(b"mpv_command\0").map_err(|e| e.to_string())?;
                    let command_async: Symbol<MpvCommandAsyncFn> =
                        lib.get(b"mpv_command_async\0").map_err(|e| e.to_string())?;
                    let set_property_string: Symbol<MpvSetPropertyStringFn> = lib
                        .get(b"mpv_set_property_string\0")
                        .map_err(|e| e.to_string())?;
                    let get_property: Symbol<MpvGetPropertyFn> =
                        lib.get(b"mpv_get_property\0").map_err(|e| e.to_string())?;
                    let observe_property: Symbol<MpvObservePropertyFn> = lib
                        .get(b"mpv_observe_property\0")
                        .map_err(|e| e.to_string())?;
                    let wait_event: Symbol<MpvWaitEventFn> =
                        lib.get(b"mpv_wait_event\0").map_err(|e| e.to_string())?;
                    let wakeup: Symbol<MpvWakeupFn> =
                        lib.get(b"mpv_wakeup\0").map_err(|e| e.to_string())?;
                    let free: Symbol<MpvFreeFn> =
                        lib.get(b"mpv_free\0").map_err(|e| e.to_string())?;
                    let terminate_destroy: Symbol<MpvTerminateDestroyFn> = lib
                        .get(b"mpv_terminate_destroy\0")
                        .map_err(|e| e.to_string())?;

                    crate::crash_log::note(&format!(
                        "player: libmpv carregada de {}",
                        name.to_string_lossy()
                    ));
                    return Ok(Self {
                        create: *create,
                        initialize: *initialize,
                        load_config: *lib
                            .get::<MpvLoadConfigFn>(b"mpv_load_config_file\0")
                            .map_err(|e| e.to_string())?,
                        set_option: *set_option,
                        set_option_string: *set_option_string,
                        command: *command,
                        command_async: *command_async,
                        set_property_string: *set_property_string,
                        get_property: *get_property,
                        observe_property: *observe_property,
                        wait_event: *wait_event,
                        wakeup: *wakeup,
                        free: *free,
                        terminate_destroy: *terminate_destroy,
                        _lib: Arc::new(lib),
                    });
                },
                Err(e) => last_err = e.to_string(),
            }
        }
        crate::crash_log::note(&format!("player: falha ao carregar libmpv: {last_err}"));
        Err(format!("Failed to load libmpv: {last_err}"))
    }
}

fn ensure_embedded_windows_libmpv(gz_bytes: &[u8]) -> Result<std::path::PathBuf, String> {
    let base_dir = std::env::var_os("LOCALAPPDATA")
        .filter(|v| !v.is_empty())
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let runtime_dir = base_dir
        .join("NuvioDesktop")
        .join("runtime")
        .join(env!("CARGO_PKG_VERSION"));
    decompress_embedded_libmpv_to(gz_bytes, &runtime_dir)
}

fn decompress_embedded_libmpv_to(
    gz_bytes: &[u8],
    runtime_dir: &std::path::Path,
) -> Result<std::path::PathBuf, String> {
    use flate2::read::GzDecoder;
    use std::io::Read;

    std::fs::create_dir_all(runtime_dir).map_err(|e| e.to_string())?;
    let dll_path = runtime_dir.join("libmpv-2.dll");
    let marker_path = runtime_dir.join(".gz-len");
    let expected_marker = gz_bytes.len().to_string();

    if let Ok(meta) = std::fs::metadata(&dll_path)
        && meta.len() > 0
        && std::fs::read_to_string(&marker_path).ok().as_deref() == Some(expected_marker.as_str())
    {
        return Ok(dll_path);
    }

    let tmp_path = runtime_dir.join(format!("libmpv-2.{}.tmp", std::process::id()));
    let mut decoder = GzDecoder::new(gz_bytes);
    let mut out = Vec::new();
    decoder.read_to_end(&mut out).map_err(|e| e.to_string())?;
    std::fs::write(&tmp_path, &out).map_err(|e| e.to_string())?;
    if let Err(err) = std::fs::rename(&tmp_path, &dll_path) {
        let _ = std::fs::remove_file(&tmp_path);
        if !dll_path.exists() {
            return Err(err.to_string());
        }
    }
    let _ = std::fs::write(&marker_path, expected_marker);
    Ok(dll_path)
}

#[cfg(test)]
mod embedded_libmpv_tests {
    use super::decompress_embedded_libmpv_to;
    use flate2::{Compression, write::GzEncoder};
    use std::io::Write;

    #[test]
    fn extracts_gzipped_libmpv_once_and_reuses_cached_file() {
        let mut enc = GzEncoder::new(Vec::new(), Compression::fast());
        enc.write_all(b"MZ_FAKE_DLL_BYTES").unwrap();
        let gz = enc.finish().unwrap();

        let dir = std::env::temp_dir().join(format!("nuvio-mpv-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);

        let first = decompress_embedded_libmpv_to(&gz, &dir).unwrap();
        assert_eq!(std::fs::read(&first).unwrap(), b"MZ_FAKE_DLL_BYTES");

        let second = decompress_embedded_libmpv_to(&gz, &dir).unwrap();
        assert_eq!(first, second);

        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(target_os = "linux")]
mod platform_host {
    use libloading::{Library, Symbol};
    use std::os::raw::{c_int, c_uint, c_ulong, c_void};

    type XOpenDisplayFn = unsafe extern "C" fn(*const i8) -> *mut c_void;
    type XCreateSimpleWindowFn = unsafe extern "C" fn(
        *mut c_void,
        c_ulong,
        c_int,
        c_int,
        c_uint,
        c_uint,
        c_uint,
        c_ulong,
        c_ulong,
    ) -> c_ulong;
    type XMapWindowFn = unsafe extern "C" fn(*mut c_void, c_ulong) -> c_int;
    type XUnmapWindowFn = unsafe extern "C" fn(*mut c_void, c_ulong) -> c_int;
    type XRaiseWindowFn = unsafe extern "C" fn(*mut c_void, c_ulong) -> c_int;
    type XMoveResizeWindowFn =
        unsafe extern "C" fn(*mut c_void, c_ulong, c_int, c_int, c_uint, c_uint) -> c_int;
    type XDestroyWindowFn = unsafe extern "C" fn(*mut c_void, c_ulong) -> c_int;
    type XCloseDisplayFn = unsafe extern "C" fn(*mut c_void) -> c_int;
    type XSyncFn = unsafe extern "C" fn(*mut c_void, c_int) -> c_int;
    type XFlushFn = unsafe extern "C" fn(*mut c_void) -> c_int;

    pub struct ChildSurface {
        _x11: Library,
        display: *mut c_void,
        parent_xid: u64,
        child_xid: c_ulong,
        unmap_win: XUnmapWindowFn,
        raise_window: XRaiseWindowFn,
        move_resize: XMoveResizeWindowFn,
        destroy: XDestroyWindowFn,
        close_display: XCloseDisplayFn,
        flush: XFlushFn,
    }

    unsafe impl Send for ChildSurface {}

    impl ChildSurface {
        pub fn create(parent_xid: u64, x: i32, y: i32, w: u32, h: u32) -> Result<Self, String> {
            unsafe {
                let x11 = Library::new("libX11.so.6").map_err(|e| e.to_string())?;
                let open_display: Symbol<XOpenDisplayFn> =
                    x11.get(b"XOpenDisplay\0").map_err(|e| e.to_string())?;
                let create_simple: Symbol<XCreateSimpleWindowFn> = x11
                    .get(b"XCreateSimpleWindow\0")
                    .map_err(|e| e.to_string())?;
                let map_win: Symbol<XMapWindowFn> =
                    x11.get(b"XMapWindow\0").map_err(|e| e.to_string())?;
                let unmap_win: Symbol<XUnmapWindowFn> =
                    x11.get(b"XUnmapWindow\0").map_err(|e| e.to_string())?;
                let raise_window: Symbol<XRaiseWindowFn> =
                    x11.get(b"XRaiseWindow\0").map_err(|e| e.to_string())?;
                let move_resize: Symbol<XMoveResizeWindowFn> =
                    x11.get(b"XMoveResizeWindow\0").map_err(|e| e.to_string())?;
                let destroy: Symbol<XDestroyWindowFn> =
                    x11.get(b"XDestroyWindow\0").map_err(|e| e.to_string())?;
                let close_display: Symbol<XCloseDisplayFn> =
                    x11.get(b"XCloseDisplay\0").map_err(|e| e.to_string())?;
                let sync: Symbol<XSyncFn> = x11.get(b"XSync\0").map_err(|e| e.to_string())?;
                let flush: Symbol<XFlushFn> = x11.get(b"XFlush\0").map_err(|e| e.to_string())?;

                let display = open_display(std::ptr::null());
                if display.is_null() {
                    return Err("XOpenDisplay returned null (Wayland without XWayland?)".into());
                }
                let child_xid = create_simple(
                    display,
                    parent_xid as c_ulong,
                    x,
                    y,
                    w.max(16),
                    h.max(16),
                    0,
                    0,
                    0,
                );
                if child_xid == 0 {
                    close_display(display);
                    return Err("XCreateSimpleWindow failed".into());
                }
                map_win(display, child_xid);
                raise_window(display, child_xid);
                sync(display, 0);

                Ok(Self {
                    display,
                    parent_xid,
                    child_xid,
                    unmap_win: *unmap_win,
                    raise_window: *raise_window,
                    move_resize: *move_resize,
                    destroy: *destroy,
                    close_display: *close_display,
                    flush: *flush,
                    _x11: x11,
                })
            }
        }

        pub fn wid_string(&self) -> String {
            self.child_xid.to_string()
        }

        pub fn raw_id(&self) -> u64 {
            self.child_xid
        }

        pub fn parent_id(&self) -> u64 {
            self.parent_xid
        }

        pub fn hide_and_detach_for_cleanup(&mut self) -> (*mut c_void, u64) {
            if self.display.is_null() || self.child_xid == 0 {
                return (std::ptr::null_mut(), 0);
            }
            unsafe {
                (self.unmap_win)(self.display, self.child_xid);
                (self.flush)(self.display);
            }
            let dpy = self.display;
            let xid = self.child_xid;
            self.display = std::ptr::null_mut();
            self.child_xid = 0;
            (dpy, xid)
        }

        pub fn set_bounds(&self, x: i32, y: i32, w: u32, h: u32) {
            if self.display.is_null() || self.child_xid == 0 {
                return;
            }
            unsafe {
                (self.move_resize)(self.display, self.child_xid, x, y, w.max(16), h.max(16));
                (self.raise_window)(self.display, self.child_xid);
                (self.flush)(self.display);
            }
        }
    }

    impl Drop for ChildSurface {
        fn drop(&mut self) {
            if self.display.is_null() {
                return;
            }
            unsafe {
                if self.child_xid != 0 {
                    (self.destroy)(self.display, self.child_xid);
                    (self.flush)(self.display);
                }
                (self.close_display)(self.display);
            }
        }
    }
}

#[cfg(target_os = "windows")]
mod platform_host {
    use libloading::{Library, Symbol};
    use std::os::raw::{c_int, c_void};
    use std::sync::Arc;
    use tauri::{AppHandle, WebviewWindow};

    const WS_CHILD: u32 = 0x40000000;
    const WS_VISIBLE: u32 = 0x10000000;
    const WS_CLIPCHILDREN: u32 = 0x02000000;
    const HWND_TOP: isize = 0;
    const SWP_NOACTIVATE: u32 = 0x0010;
    const SWP_SHOWWINDOW: u32 = 0x0040;
    const SWP_NOCOPYBITS: u32 = 0x0100;
    const SWP_ASYNCWINDOWPOS: u32 = 0x4000;

    type CreateWindowExWFn = unsafe extern "system" fn(
        u32,
        *const u16,
        *const u16,
        u32,
        c_int,
        c_int,
        c_int,
        c_int,
        isize,
        isize,
        isize,
        *mut c_void,
    ) -> isize;
    type SetWindowPosFn =
        unsafe extern "system" fn(isize, isize, c_int, c_int, c_int, c_int, u32) -> c_int;
    type DestroyWindowFn = unsafe extern "system" fn(isize) -> c_int;
    type GetWindowThreadProcessIdFn = unsafe extern "system" fn(isize, *mut u32) -> u32;
    type GetCurrentThreadIdFn = unsafe extern "system" fn() -> u32;

    fn current_thread_id() -> u32 {
        unsafe {
            if let Ok(k32) = Library::new("kernel32.dll")
                && let Ok(get_tid) = k32.get::<GetCurrentThreadIdFn>(b"GetCurrentThreadId\0")
            {
                return get_tid();
            }
        }
        0
    }

    pub struct ChildSurface {
        _user32: Arc<Library>,
        app: AppHandle,
        owner_tid: u32,
        hwnd: isize,
        set_window_pos: SetWindowPosFn,
        destroy_window: DestroyWindowFn,
    }

    unsafe impl Send for ChildSurface {}

    impl ChildSurface {
        pub fn create(
            app: &AppHandle,
            window: &WebviewWindow,
            parent_hwnd: isize,
            x: i32,
            y: i32,
            w: u32,
            h: u32,
        ) -> Result<Self, String> {
            let user32 =
                Arc::new(unsafe { Library::new("user32.dll") }.map_err(|e| e.to_string())?);
            let owner_tid = unsafe {
                let get_win_tid: Symbol<GetWindowThreadProcessIdFn> = user32
                    .get(b"GetWindowThreadProcessId\0")
                    .map_err(|e| e.to_string())?;
                get_win_tid(parent_hwnd, std::ptr::null_mut())
            };
            let app_clone = app.clone();
            if owner_tid != 0 && current_thread_id() != owner_tid {
                let (tx, rx) = std::sync::mpsc::sync_channel(1);
                let user32_c = Arc::clone(&user32);
                window
                    .run_on_main_thread(move || {
                        let res = Self::create_on_owner_thread(
                            user32_c,
                            app_clone,
                            parent_hwnd,
                            x,
                            y,
                            w,
                            h,
                        );
                        let _ = tx.send(res);
                    })
                    .map_err(|e| e.to_string())?;
                return rx
                    .recv()
                    .map_err(|_| "Main thread dropped ChildSurface creation".to_string())?;
            }
            Self::create_on_owner_thread(user32, app_clone, parent_hwnd, x, y, w, h)
        }

        fn create_on_owner_thread(
            user32: Arc<Library>,
            app: AppHandle,
            parent_hwnd: isize,
            x: i32,
            y: i32,
            w: u32,
            h: u32,
        ) -> Result<Self, String> {
            unsafe {
                let create_win: Symbol<CreateWindowExWFn> = user32
                    .get(b"CreateWindowExW\0")
                    .map_err(|e| e.to_string())?;
                let set_window_pos: Symbol<SetWindowPosFn> = user32
                    .get(b"SetWindowPos\0")
                    .map_err(|e| e.to_string())?;
                let destroy_window: Symbol<DestroyWindowFn> = user32
                    .get(b"DestroyWindow\0")
                    .map_err(|e| e.to_string())?;

                let class_name: Vec<u16> = "STATIC\0".encode_utf16().collect();
                let title: Vec<u16> = "NuvioVideo\0".encode_utf16().collect();
                let cw = w.max(16) as c_int;
                let ch = h.max(16) as c_int;
                // Omit WS_CLIPSIBLINGS so DWM never clips the video HWND into a white/black box against sibling WebView windows.
                let hwnd = create_win(
                    0,
                    class_name.as_ptr(),
                    title.as_ptr(),
                    WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN,
                    x,
                    y,
                    cw,
                    ch,
                    parent_hwnd,
                    0,
                    0,
                    std::ptr::null_mut(),
                );
                if hwnd == 0 {
                    return Err("CreateWindowExW failed".into());
                }
                (set_window_pos)(
                    hwnd,
                    HWND_TOP,
                    x,
                    y,
                    cw,
                    ch,
                    SWP_SHOWWINDOW | SWP_NOACTIVATE | SWP_NOCOPYBITS,
                );
                Ok(Self {
                    hwnd,
                    app,
                    owner_tid: current_thread_id(),
                    set_window_pos: *set_window_pos,
                    destroy_window: *destroy_window,
                    _user32: user32,
                })
            }
        }

        pub fn wid_string(&self) -> String {
            self.hwnd.to_string()
        }

        pub fn raw_id(&self) -> u64 {
            self.hwnd as u64
        }

        pub fn set_bounds(&self, x: i32, y: i32, w: u32, h: u32) {
            unsafe {
                (self.set_window_pos)(
                    self.hwnd,
                    HWND_TOP,
                    x,
                    y,
                    w.max(16) as c_int,
                    h.max(16) as c_int,
                    SWP_SHOWWINDOW | SWP_NOACTIVATE | SWP_NOCOPYBITS | SWP_ASYNCWINDOWPOS,
                );
            }
        }
    }

    impl Drop for ChildSurface {
        fn drop(&mut self) {
            if self.owner_tid == 0 || current_thread_id() == self.owner_tid {
                unsafe {
                    (self.destroy_window)(self.hwnd);
                }
            } else {
                let hwnd = self.hwnd;
                let destroy = self.destroy_window;
                let user32 = Arc::clone(&self._user32);
                let _ = self.app.run_on_main_thread(move || {
                    let _keep_user32 = user32;
                    unsafe {
                        (destroy)(hwnd);
                    }
                });
            }
        }
    }
}

#[cfg(all(not(target_os = "linux"), not(target_os = "windows")))]
mod platform_host {
    pub struct ChildSurface;
    impl ChildSurface {
        pub fn create(_parent: u64, _x: i32, _y: i32, _w: u32, _h: u32) -> Result<Self, String> {
            Err("Embedded child surface not implemented for this OS".into())
        }
        pub fn wid_string(&self) -> String {
            String::new()
        }
        pub fn raw_id(&self) -> u64 {
            0
        }
        pub fn set_bounds(&self, _x: i32, _y: i32, _w: u32, _h: u32) {}
    }
}

struct ActiveSession {
    api: MpvApi,
    handle: *mut c_void,
    surface: platform_host::ChildSurface,
    stop_flag: Arc<AtomicBool>,
    ended_flag: Arc<AtomicBool>,
    #[cfg(target_os = "linux")]
    overlay_shared: Arc<LinuxOverlayShared>,
    #[cfg(target_os = "linux")]
    overlay_handle: Option<LinuxOverlayHandle>,
    #[cfg(target_os = "windows")]
    win_overlay: Option<WebviewWindow>,
    event_thread: Option<std::thread::JoinHandle<()>>,
}

/// Returns unused glibc heap pages back to the kernel after heavy video/JS teardown.
pub fn trim_process_heap() {
    #[cfg(target_os = "linux")]
    unsafe {
        static TRIM_FN: std::sync::OnceLock<Option<unsafe extern "C" fn(usize) -> c_int>> =
            std::sync::OnceLock::new();
        let fn_opt = TRIM_FN.get_or_init(|| {
            let lib = Box::leak(Box::new(Library::new("libc.so.6").ok()?));
            let sym = lib
                .get::<unsafe extern "C" fn(usize) -> c_int>(b"malloc_trim\0")
                .ok()?;
            Some(*sym)
        });
        if let Some(trim) = fn_opt {
            trim(0);
        }
    }
}

unsafe impl Send for ActiveSession {}

pub struct EmbeddedPlayer {
    session: Arc<Mutex<Option<ActiveSession>>>,
    party_locked: Arc<AtomicBool>,
    #[cfg(target_os = "windows")]
    last_controls_json: Arc<Mutex<Option<String>>>,
}

#[derive(Debug, Clone, Copy)]
pub struct ViewportRect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl EmbeddedPlayer {
    pub fn new() -> Self {
        Self {
            session: Arc::new(Mutex::new(None)),
            party_locked: Arc::new(AtomicBool::new(false)),
            #[cfg(target_os = "windows")]
            last_controls_json: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_party_locked(&self, locked: bool) {
        self.party_locked.store(locked, Ordering::Relaxed);
    }

    pub fn handle_ui_action(&self, ev_type: &str, val: f64) {
        if ev_type == "controlsReady" {
            #[cfg(target_os = "windows")]
            if let Some(json) = self
                .last_controls_json
                .lock()
                .ok()
                .and_then(|g| g.clone())
                && let Ok(guard) = self.session.lock()
                && let Some(sess) = guard.as_ref()
                && let Some(ref win_ov) = sess.win_overlay
            {
                let script = format!("window.playerControls&&window.playerControls({json})");
                let _ = win_ov.eval(&script);
            }
            return;
        }
        let blocked = self.party_locked.load(Ordering::Relaxed)
            && matches!(
                ev_type,
                "setPlaybackState"
                    | "setPlaybackStateQuiet"
                    | "toggle"
                    | "keyboardToggle"
                    | "scrubFinish"
                    | "seekBack"
                    | "seekForward"
                    | "setPlaybackSpeed"
            );
        if blocked {
            return;
        }
        if let Ok(guard) = self.session.lock()
            && let Some(sess) = guard.as_ref()
        {
            handle_low_latency_mpv_action(
                &sess.api,
                sess.handle,
                &sess.ended_flag,
                ev_type,
                val,
            );
        }
    }

    pub fn sync_windows_overlay_bounds(&self, window: &WebviewWindow) {
        #[cfg(target_os = "windows")]
        {
            let guard = self.session.lock().expect("player mutex");
            if let Some(sess) = guard.as_ref()
                && let Some(ref win_ov) = sess.win_overlay
            {
                if window.is_minimized().unwrap_or(false) {
                    let _ = win_ov.hide();
                } else if let (Ok(pos), Ok(size)) = (window.inner_position(), window.inner_size()) {
                    let _ = win_ov.set_position(pos);
                    let _ = win_ov.set_size(size);
                    let _ = win_ov.show();
                }
            }
        }
        #[cfg(not(target_os = "windows"))]
        let _ = window;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn play(
        &self,
        app: AppHandle,
        window: &WebviewWindow,
        url: &str,
        headers: Option<HashMap<String, String>>,
        rect: ViewportRect,
        start_secs: Option<f64>,
        controls_url: String,
        initial_controls_json: Option<String>,
        options: &crate::video_options::VideoOptions,
    ) -> Result<bool, String> {
        self.stop();

        let raw_handle = window.window_handle().map_err(|e| e.to_string())?;
        let surface = match raw_handle.as_raw() {
            #[cfg(target_os = "linux")]
            raw @ (RawWindowHandle::Xlib(_) | RawWindowHandle::Xcb(_)) => {
                create_surface_for_linux(raw, rect)?
            }
            #[cfg(target_os = "windows")]
            RawWindowHandle::Win32(h) => platform_host::ChildSurface::create(
                &app,
                window,
                h.hwnd.get(),
                rect.x,
                rect.y,
                rect.w,
                rect.h,
            )?,
            other => {
                return Err(format!(
                    "Window handle {:?} is not X11/Win32 (set GDK_BACKEND=x11 on Wayland)",
                    other
                ));
            }
        };

        let wid = surface.wid_string();
        #[cfg(not(target_os = "windows"))]
        let host_id = surface.raw_id();
        #[cfg(target_os = "linux")]
        let parent_id = surface.parent_id();
        let api = MpvApi::load()?;

        let handle = create_and_init_mpv(&api, &wid, headers.as_ref(), start_secs, options)?;
        mpv_cmd(&api, handle, &["loadfile", url, "replace"])?;

        let stop_flag = Arc::new(AtomicBool::new(false));
        let first_frame_shown = Arc::new(AtomicBool::new(false));
        let ended_flag = Arc::new(AtomicBool::new(false));

        #[cfg(target_os = "linux")]
        let overlay_shared = LinuxOverlayShared::new();
        #[cfg(target_os = "linux")]
        let overlay_handle = {
            let app_cb = app.clone();
            let api_cb = api.clone();
            let handle_addr = handle as usize;
            let ended_cb = Arc::clone(&ended_flag);
            let shared_cb = Arc::clone(&overlay_shared);
            let party_locked_cb = Arc::clone(&self.party_locked);
            LinuxOverlayHandle::attach(
                (host_id, parent_id),
                (controls_url.clone(), initial_controls_json.clone()),
                handle,
                api.command_async,
                api.command,
                Arc::clone(&overlay_shared),
                Box::new(move |ev_type, val| {
                    let blocked = party_locked_cb.load(Ordering::Relaxed)
                        && matches!(
                            ev_type,
                            "setPlaybackState"
                                | "setPlaybackStateQuiet"
                                | "toggle"
                                | "keyboardToggle"
                                | "scrubFinish"
                                | "seekBack"
                                | "seekForward"
                                | "setPlaybackSpeed"
                        );
                    if !blocked && shared_cb.mpv_alive.load(Ordering::Relaxed) {
                        handle_low_latency_mpv_action(
                            &api_cb,
                            handle_addr as *mut c_void,
                            &ended_cb,
                            ev_type,
                            val,
                        );
                    }
                    let _ = app_cb.emit(
                        "player:ui_event",
                        PlayerUiEventPayload {
                            event_type: ev_type.to_string(),
                            value: val,
                        },
                    );
                }),
            )
            .ok()
        };

        #[cfg(target_os = "linux")]
        let has_overlay = overlay_handle.is_some();

        #[cfg(target_os = "windows")]
        let win_overlay = match create_windows_overlay(&app, window) {
            Ok(ov) => Some(ov),
            Err(err) => {
                tracing::error!("Windows player overlay unavailable: {err}");
                eprintln!("Windows player overlay unavailable: {err}");
                None
            }
        };
        #[cfg(target_os = "windows")]
        {
            *self.last_controls_json.lock().expect("controls mutex") =
                initial_controls_json.clone();
            if let (Some(win_ov), Some(init_json)) = (&win_overlay, &initial_controls_json) {
                let script = format!("window.playerControls&&window.playerControls({init_json})");
                let _ = win_ov.eval(&script);
            }
        }
        #[cfg(not(target_os = "linux"))]
        let _ = &controls_url;
        #[cfg(not(target_os = "linux"))]
        let _ = &initial_controls_json;
        #[cfg(target_os = "windows")]
        let has_overlay = win_overlay.is_some();

        #[cfg(all(not(target_os = "linux"), not(target_os = "windows")))]
        let has_overlay = {
            let _ = (host_id, controls_url);
            false
        };

        let stop_clone = Arc::clone(&stop_flag);
        let first_clone = first_frame_shown;
        let ended_clone = Arc::clone(&ended_flag);
        #[cfg(target_os = "linux")]
        let shared_loop = Arc::clone(&overlay_shared);
        let session_arc = Arc::clone(&self.session);

        *self.session.lock().expect("player mutex") = Some(ActiveSession {
            api: api.clone(),
            handle,
            surface,
            stop_flag,
            ended_flag,
            #[cfg(target_os = "linux")]
            overlay_shared,
            #[cfg(target_os = "linux")]
            overlay_handle,
            #[cfg(target_os = "windows")]
            win_overlay,
            event_thread: None,
        });

        observe_player_properties(&api, handle);

        let api_evt = api.clone();
        let handle_addr_evt = handle as usize;
        let evt_jh = std::thread::spawn(move || {
            const CONTINUOUS_EMIT_INTERVAL: std::time::Duration =
                std::time::Duration::from_millis(200);
            let mut cached = CachedPlayerState::default();
            let mut last_emit = std::time::Instant::now()
                .checked_sub(CONTINUOUS_EMIT_INTERVAL)
                .unwrap_or_else(std::time::Instant::now);
            let mut pending_continuous = false;

            while !stop_clone.load(Ordering::Relaxed) {
                let h_ptr = handle_addr_evt as *mut c_void;
                let wait_timeout = if pending_continuous {
                    let elapsed = last_emit.elapsed();
                    if elapsed >= CONTINUOUS_EMIT_INTERVAL {
                        0.0
                    } else {
                        (CONTINUOUS_EMIT_INTERVAL - elapsed)
                            .as_secs_f64()
                            .max(0.005)
                    }
                } else {
                    -1.0
                };

                let mut dirty_discrete = false;
                let mut dirty_continuous = false;
                let mut tracks_dirty = false;
                let mut chapters_dirty = false;
                let mut first_wait = true;

                loop {
                    if stop_clone.load(Ordering::Relaxed) {
                        break;
                    }
                    let timeout = if first_wait {
                        first_wait = false;
                        wait_timeout
                    } else {
                        0.0
                    };
                    let ev_ptr = unsafe { (api_evt.wait_event)(h_ptr, timeout) };
                    if ev_ptr.is_null() {
                        break;
                    }
                    let ev = unsafe { &*ev_ptr };
                    if ev.event_id == MPV_EVENT_NONE {
                        break;
                    }
                    match ev.event_id {
                        MPV_EVENT_START_FILE => {
                            first_clone.store(false, Ordering::Relaxed);
                            ended_clone.store(false, Ordering::Relaxed);
                            dirty_discrete = true;
                        }
                        MPV_EVENT_PLAYBACK_RESTART => {
                            first_clone.store(true, Ordering::Relaxed);
                            dirty_discrete = true;
                        }
                        MPV_EVENT_END_FILE => {
                            ended_clone.store(true, Ordering::Relaxed);
                            dirty_discrete = true;
                        }
                        MPV_EVENT_COMMAND_REPLY => {
                            #[cfg(target_os = "linux")]
                            if (ev.reply_userdata >> 32) == OVERLAY_REPLY_TAG {
                                shared_loop.mark_overlay_done(ev.reply_userdata & 0xffff_ffff);
                            }
                        }
                        MPV_EVENT_PROPERTY_CHANGE => {
                            if !ev.data.is_null() {
                                let prop = unsafe { &*(ev.data as *const MpvEventProperty) };
                                let change = cached.apply_property_event(ev.reply_userdata, prop);
                                tracks_dirty |= change.tracks_dirty;
                                chapters_dirty |= change.chapters_dirty;
                                dirty_discrete |= change.discrete;
                                dirty_continuous |= change.continuous;
                            }
                        }
                        MPV_EVENT_SHUTDOWN => {
                            stop_clone.store(true, Ordering::Relaxed);
                            break;
                        }
                        _ => {}
                    }
                }

                if stop_clone.load(Ordering::Relaxed) {
                    break;
                }

                if tracks_dirty {
                    cached.refresh_tracks(&api_evt, h_ptr);
                    dirty_discrete = true;
                }
                if chapters_dirty {
                    cached.refresh_chapters(&api_evt, h_ptr);
                    dirty_discrete = true;
                }

                let should_emit = if dirty_discrete {
                    true
                } else if dirty_continuous || pending_continuous {
                    if last_emit.elapsed() >= CONTINUOUS_EMIT_INTERVAL {
                        true
                    } else {
                        pending_continuous = true;
                        false
                    }
                } else {
                    false
                };

                if should_emit {
                    last_emit = std::time::Instant::now();
                    pending_continuous = false;
                    let ff = first_clone.load(Ordering::Relaxed);
                    let ended = ended_clone.load(Ordering::Relaxed);
                    let (payload, update_js) = cached.build_payload_and_js(ff, ended);

                    {
                        let guard = session_arc.lock().expect("player mutex");
                        let Some(sess) = guard.as_ref() else {
                            break;
                        };

                        #[cfg(target_os = "linux")]
                        {
                            sess.overlay_shared
                                .loading
                                .store(payload.loading, Ordering::Relaxed);
                            sess.overlay_shared
                                .paused
                                .store(payload.paused, Ordering::Relaxed);
                            if let Some(ref ov) = sess.overlay_handle {
                                ov.push_update_js(update_js);
                            }
                        }

                        #[cfg(target_os = "windows")]
                        if let Some(ref win_ov) = sess.win_overlay {
                            let _ = win_ov.eval(&update_js);
                        }
                    }

                    let _ = app.emit("player:state", payload);
                }
            }
        });
        if let Some(ref mut sess) = *self.session.lock().expect("player mutex") {
            sess.event_thread = Some(evt_jh);
        }

        Ok(has_overlay)
    }

    pub fn push_controls_json(&self, controls_json: String) {
        #[cfg(target_os = "windows")]
        {
            *self.last_controls_json.lock().expect("controls mutex") = Some(controls_json.clone());
        }
        let guard = self.session.lock().expect("player mutex");
        let Some(sess) = guard.as_ref() else {
            return;
        };
        #[cfg(target_os = "linux")]
        if let Some(ref ov) = sess.overlay_handle {
            ov.update_controls_json(controls_json);
        }
        #[cfg(target_os = "windows")]
        if let Some(ref win_ov) = sess.win_overlay {
            let script = format!("window.playerControls&&window.playerControls({controls_json})");
            let _ = win_ov.eval(&script);
        }
        #[cfg(all(not(target_os = "linux"), not(target_os = "windows")))]
        let _ = (sess, controls_json);
    }

    pub fn focus_overlay(&self) {
        let guard = self.session.lock().expect("player mutex");
        let Some(sess) = guard.as_ref() else {
            return;
        };
        #[cfg(target_os = "linux")]
        if let Some(ref ov) = sess.overlay_handle {
            ov.focus();
        }
        #[cfg(target_os = "windows")]
        if let Some(ref win_ov) = sess.win_overlay {
            let _ = win_ov.set_focus();
        }
        #[cfg(all(not(target_os = "linux"), not(target_os = "windows")))]
        let _ = sess;
    }

    pub fn set_bounds(&self, window: &WebviewWindow, x: i32, y: i32, w: u32, h: u32) {
        let guard = self.session.lock().expect("player mutex");
        if let Some(sess) = guard.as_ref() {
            sess.surface.set_bounds(x, y, w, h);
            #[cfg(target_os = "windows")]
            if let Some(ref win_ov) = sess.win_overlay {
                if let (Ok(pos), Ok(size)) = (window.inner_position(), window.inner_size()) {
                    let _ = win_ov.set_position(pos);
                    let _ = win_ov.set_size(size);
                }
            }
            #[cfg(not(target_os = "windows"))]
            let _ = window;
        }
    }

    pub fn set_paused(&self, paused: bool) -> Result<(), String> {
        let guard = self.session.lock().expect("player mutex");
        let sess = guard.as_ref().ok_or("No active player")?;
        if !paused
            && (sess.ended_flag.load(Ordering::Relaxed)
                || get_prop_flag(&sess.api, sess.handle, "eof-reached"))
        {
            let _ = mpv_cmd(&sess.api, sess.handle, &["seek", "0", "absolute"]);
            sess.ended_flag.store(false, Ordering::Relaxed);
        }
        mpv_set_prop(
            &sess.api,
            sess.handle,
            "pause",
            if paused { "yes" } else { "no" },
        )
    }

    pub fn seek_absolute(&self, secs: f64) -> Result<(), String> {
        let guard = self.session.lock().expect("player mutex");
        let sess = guard.as_ref().ok_or("No active player")?;
        let target = secs.max(0.0).to_string();
        mpv_cmd(&sess.api, sess.handle, &["seek", &target, "absolute+exact"])
    }

    pub fn set_volume(&self, vol: f64) -> Result<(), String> {
        let guard = self.session.lock().expect("player mutex");
        let sess = guard.as_ref().ok_or("No active player")?;
        let v = vol.clamp(0.0, 200.0).to_string();
        mpv_set_prop(&sess.api, sess.handle, "volume", &v)
    }

    pub fn set_speed(&self, speed: f64) -> Result<(), String> {
        let guard = self.session.lock().expect("player mutex");
        let sess = guard.as_ref().ok_or("No active player")?;
        let s = speed.clamp(0.25, 4.0).to_string();
        mpv_set_prop(&sess.api, sess.handle, "speed", &s)
    }

    pub fn set_resize_mode(&self, mode: usize) -> Result<(), String> {
        let guard = self.session.lock().expect("player mutex");
        let sess = guard.as_ref().ok_or("No active player")?;
        apply_resize_mode(&sess.api, sess.handle, mode)
    }

    pub fn set_audio_track(&self, aid: &str) -> Result<(), String> {
        let guard = self.session.lock().expect("player mutex");
        let sess = guard.as_ref().ok_or("No active player")?;
        mpv_set_prop(&sess.api, sess.handle, "aid", aid)
    }

    pub fn set_subtitle_track(&self, sid: &str) -> Result<(), String> {
        let guard = self.session.lock().expect("player mutex");
        let sess = guard.as_ref().ok_or("No active player")?;
        mpv_set_prop(&sess.api, sess.handle, "sid", sid)
    }

    pub fn set_reference_track(&self, sid: Option<i64>) -> Result<(), String> {
        let guard = self.session.lock().expect("player mutex");
        let sess = guard.as_ref().ok_or("No active player")?;
        if sid.is_some() {
            mpv_set_prop(&sess.api, sess.handle, "secondary-sub-visibility", "no")?;
            mpv_set_prop(&sess.api, sess.handle, "secondary-sub-delay", "0")?;
        }
        let value = sid.map(|id| id.to_string()).unwrap_or_else(|| "no".into());
        mpv_set_prop(&sess.api, sess.handle, "secondary-sid", &value)
    }

    pub fn add_subtitle_url(&self, url: &str) -> Result<(), String> {
        let guard = self.session.lock().expect("player mutex");
        let sess = guard.as_ref().ok_or("No active player")?;
        let _ = mpv_set_prop(&sess.api, sess.handle, "sub-visibility", "yes");
        mpv_cmd(&sess.api, sess.handle, &["sub-add", url, "select"])
    }

    pub fn clear_external_subtitles(&self) -> Result<(), String> {
        let guard = self.session.lock().expect("player mutex");
        let sess = guard.as_ref().ok_or("No active player")?;
        let _ = mpv_cmd(&sess.api, sess.handle, &["sub-remove"]);
        Ok(())
    }

    pub fn set_subtitle_visible(&self, visible: bool) -> Result<(), String> {
        let guard = self.session.lock().expect("player mutex");
        let sess = guard.as_ref().ok_or("No active player")?;
        mpv_set_prop(
            &sess.api,
            sess.handle,
            "sub-visibility",
            if visible { "yes" } else { "no" },
        )
    }

    pub fn set_subtitle_delay_ms(&self, delay_ms: i64) -> Result<(), String> {
        let guard = self.session.lock().expect("player mutex");
        let sess = guard.as_ref().ok_or("No active player")?;
        let secs = (delay_ms as f64) / 1000.0;
        mpv_set_prop(&sess.api, sess.handle, "sub-delay", &format!("{secs:.3}"))
    }

    pub fn apply_subtitle_style(&self, style: &SubtitleStylePayload) -> Result<(), String> {
        let guard = self.session.lock().expect("player mutex");
        let sess = guard.as_ref().ok_or("No active player")?;
        let a = &sess.api;
        let h = sess.handle;

        let _ = mpv_set_prop(a, h, "sub-ass-override", "no");
        let _ = mpv_set_prop(a, h, "sub-color", &style.text_color);
        let _ = mpv_set_prop(a, h, "sub-border-color", &style.outline_color);
        let _ = mpv_set_prop(a, h, "sub-back-color", &style.background_color);
        let transparent_bg = style
            .background_color
            .to_ascii_lowercase()
            .starts_with("#00");
        let border_style = if transparent_bg {
            "outline-and-shadow"
        } else {
            "background-box"
        };
        let _ = mpv_set_prop(a, h, "sub-border-style", border_style);
        let border_size = if style.outline_enabled { "2.2" } else { "0" };
        let _ = mpv_set_prop(a, h, "sub-border-size", border_size);
        let _ = mpv_set_prop(a, h, "sub-bold", if style.bold { "yes" } else { "no" });
        let mapped_size = (style.font_size_sp * 2.1).clamp(16.0, 96.0);
        let _ = mpv_set_prop(a, h, "sub-font-size", &format!("{mapped_size:.1}"));
        let pos = (100 - style.bottom_offset.clamp(0, 120) / 3).clamp(40, 100);
        let _ = mpv_set_prop(a, h, "sub-pos", &pos.to_string());
        if style.shadow_enabled {
            let alpha = ((style.shadow_opacity / 100.0).clamp(0.0, 1.0) * 255.0).round() as u8;
            let _ = mpv_set_prop(a, h, "sub-shadow-color", &format!("#{alpha:02X}000000"));
            let _ = mpv_set_prop(
                a,
                h,
                "sub-shadow-offset",
                &format!("{:.1}", style.shadow_size.clamp(0.0, 8.0)),
            );
        } else {
            let _ = mpv_set_prop(a, h, "sub-shadow-offset", "0");
        }
        if let Some(ref fname) = style.font_name
            && !fname.trim().is_empty()
        {
            let _ = mpv_set_prop(a, h, "sub-font", fname.trim());
        }
        Ok(())
    }

    /// Runtime properties of the active mpv session, for the performance diagnostics.
    pub fn metrics(&self) -> Option<PlayerMetrics> {
        let guard = self.session.lock().ok()?;
        let sess = guard.as_ref()?;
        let (api, handle) = (&sess.api, sess.handle);
        Some(PlayerMetrics {
            hwdec_current: get_prop_string(api, handle, "hwdec-current"),
            video_codec: get_prop_string(api, handle, "video-codec"),
            pixel_format: get_prop_string(api, handle, "video-format"),
            audio_codec: get_prop_string(api, handle, "audio-codec-name"),
            cache_secs: get_prop_double(api, handle, "demuxer-cache-time"),
            width: get_prop_i64(api, handle, "width"),
            height: get_prop_i64(api, handle, "height"),
            fps: get_prop_double(api, handle, "estimated-vf-fps"),
            dropped_frames: get_prop_i64(api, handle, "vo-drop-frame-count"),
        })
    }

    pub fn stop(&self) {
        let taken = self.session.lock().expect("player mutex").take();
        if let Some(mut sess) = taken {
            sess.stop_flag.store(true, Ordering::SeqCst);
            #[cfg(target_os = "linux")]
            {
                sess.overlay_shared.mpv_alive.store(false, Ordering::SeqCst);
                let (dpy, xid) = sess.surface.hide_and_detach_for_cleanup();
                if let Some(ov) = sess.overlay_handle.take() {
                    ov.destroy();
                }
                unsafe {
                    (sess.api.wakeup)(sess.handle);
                }
                if let Some(jh) = sess.event_thread.take() {
                    let _ = jh.join();
                }
                unsafe {
                    (sess.api.terminate_destroy)(sess.handle);
                }
                schedule_x11_child_cleanup(dpy, xid);
            }
            #[cfg(not(target_os = "linux"))]
            {
                #[cfg(target_os = "windows")]
                {
                    *self.last_controls_json.lock().expect("controls mutex") = None;
                    if let Some(win_ov) = sess.win_overlay.take() {
                        let _ = win_ov.eval("window.playerControls&&window.playerControls({closeModalsToken:Date.now()})");
                        let _ = win_ov.hide();
                    }
                }
                unsafe {
                    (sess.api.wakeup)(sess.handle);
                }
                if let Some(jh) = sess.event_thread.take() {
                    let _ = jh.join();
                }
                unsafe {
                    (sess.api.terminate_destroy)(sess.handle);
                }
                drop(sess.surface);
            }
            trim_process_heap();
        }
    }
}

fn handle_low_latency_mpv_action(
    api: &MpvApi,
    handle: *mut c_void,
    ended_flag: &AtomicBool,
    ev_type: &str,
    val: f64,
) {
    match ev_type {
        "setPlaybackState" | "setPlaybackStateQuiet" => {
            let should_play = val >= 0.5;
            if should_play
                && (ended_flag.load(Ordering::Relaxed) || get_prop_flag(api, handle, "eof-reached"))
            {
                let _ = mpv_cmd(api, handle, &["seek", "0", "absolute"]);
                ended_flag.store(false, Ordering::Relaxed);
            }
            let _ = mpv_set_prop(api, handle, "pause", if should_play { "no" } else { "yes" });
        }
        "toggle" | "keyboardToggle" => {
            let paused = get_prop_flag(api, handle, "pause");
            if paused
                && (ended_flag.load(Ordering::Relaxed) || get_prop_flag(api, handle, "eof-reached"))
            {
                let _ = mpv_cmd(api, handle, &["seek", "0", "absolute"]);
                ended_flag.store(false, Ordering::Relaxed);
            }
            let _ = mpv_set_prop(api, handle, "pause", if paused { "no" } else { "yes" });
        }
        "scrubFinish" => {
            let secs = (val / 1000.0).max(0.0);
            let _ = mpv_cmd(&api, handle, &["seek", &format!("{secs:.3}"), "absolute+exact"]);
        }
        "seekBack" | "keyboardSeekBack" => {
            let _ = mpv_cmd(api, handle, &["seek", "-10", "relative"]);
        }
        "seekForward" | "keyboardSeekForward" => {
            let _ = mpv_cmd(api, handle, &["seek", "10", "relative"]);
        }
        "setPlaybackSpeed" => {
            let spd = val.clamp(0.25, 4.0);
            let _ = mpv_set_prop(api, handle, "speed", &format!("{spd:.2}"));
        }
        "volumeChange" | "volumeChangeTemporary" => {
            let vol = (val * 100.0).clamp(0.0, 200.0);
            let _ = mpv_set_prop(api, handle, "volume", &format!("{vol:.1}"));
        }
        "keyboardVolumeUp" => {
            let vol = (get_prop_double(api, handle, "volume") + 10.0).clamp(0.0, 200.0);
            let _ = mpv_set_prop(api, handle, "volume", &format!("{vol:.1}"));
        }
        "keyboardVolumeDown" => {
            let vol = (get_prop_double(api, handle, "volume") - 10.0).clamp(0.0, 200.0);
            let _ = mpv_set_prop(api, handle, "volume", &format!("{vol:.1}"));
        }
        "selectAudioTrack" => {
            let id = val.round() as i64;
            if id > 0 {
                let _ = mpv_set_prop(api, handle, "aid", &id.to_string());
            }
        }
        "subtitleVisibility" => {
            let _ = mpv_set_prop(
                api,
                handle,
                "sub-visibility",
                if val >= 0.5 { "yes" } else { "no" },
            );
        }
        _ => {}
    }
}

fn apply_resize_mode(api: &MpvApi, handle: *mut c_void, mode: usize) -> Result<(), String> {
    match mode % 4 {
        0 => {
            let _ = mpv_set_prop(api, handle, "keepaspect", "yes");
            mpv_set_prop(api, handle, "panscan", "0")
        }
        1 | 2 => {
            let _ = mpv_set_prop(api, handle, "keepaspect", "yes");
            mpv_set_prop(api, handle, "panscan", "1")
        }
        _ => {
            let _ = mpv_set_prop(api, handle, "keepaspect", "no");
            mpv_set_prop(api, handle, "panscan", "0")
        }
    }
}

#[cfg(target_os = "windows")]
fn create_windows_overlay(
    app: &AppHandle,
    parent: &WebviewWindow,
) -> Result<WebviewWindow, String> {
    use std::path::PathBuf;
    use tauri::{WebviewUrl, WebviewWindowBuilder};
    let pos = parent.inner_position().map_err(|e| e.to_string())?;
    let size = parent.inner_size().map_err(|e| e.to_string())?;
    if let Some(existing) = app.get_webview_window("player-overlay") {
        let _ = existing.set_position(pos);
        let _ = existing.set_size(size);
        let _ = existing.show();
        let _ = existing.set_focus();
        return Ok(existing);
    }

    let init_script = r#"
      window.__NUVIO_POST__ = function(msg) {
        try {
          if (window.__TAURI_INTERNALS__ && typeof window.__TAURI_INTERNALS__.invoke === 'function') {
            window.__TAURI_INTERNALS__.invoke('player_ui_message', {
              eventType: String((msg && msg.type) || ''),
              value: Number(msg && msg.value) || 0
            }).catch(function(){});
          }
        } catch (_) {}
      };
      document.addEventListener('DOMContentLoaded', function() {
        var s = document.createElement('style');
        s.textContent =
          '@font-face{font-family:"Nuvio JetBrains Sans";src:url("/assets/jetbrains_sans_regular.ttf") format("truetype");font-weight:400;font-style:normal;font-display:block;}' +
          '@font-face{font-family:"Nuvio JetBrains Sans";src:url("/assets/jetbrains_sans_semibold.ttf") format("truetype");font-weight:600;font-style:normal;font-display:block;}' +
          '@font-face{font-family:"Nuvio JetBrains Sans";src:url("/assets/jetbrains_sans_bold.ttf") format("truetype");font-weight:700;font-style:normal;font-display:block;}';
        (document.head || document.documentElement).appendChild(s);
      });
    "#;

    let ov = WebviewWindowBuilder::new(
        app,
        "player-overlay",
        WebviewUrl::App(PathBuf::from("player-ui/controls.html")),
    )
    .title("Nuvio Player Controls")
    .transparent(true)
    .decorations(false)
    .shadow(false)
    .skip_taskbar(true)
    .parent(parent)
    .map_err(|e| e.to_string())?
    .initialization_script(init_script)
    .build()
    .map_err(|e| {
        crate::crash_log::note(&format!("player: falha ao criar overlay Windows: {e}"));
        e.to_string()
    })?;
    crate::crash_log::note("player: overlay Windows criado");
    let _ = ov.set_position(pos);
    let _ = ov.set_size(size);
    let _ = ov.set_focus();
    Ok(ov)
}

#[cfg(target_os = "linux")]
fn create_surface_for_linux(
    raw: RawWindowHandle,
    rect: ViewportRect,
) -> Result<platform_host::ChildSurface, String> {
    match raw {
        RawWindowHandle::Xlib(handle) => {
            platform_host::ChildSurface::create(handle.window, rect.x, rect.y, rect.w, rect.h)
        }
        RawWindowHandle::Xcb(handle) => platform_host::ChildSurface::create(
            u64::from(handle.window.get()),
            rect.x,
            rect.y,
            rect.w,
            rect.h,
        ),
        other => Err(format!(
            "Unsupported Linux window handle {other:?}; ensure GDK_BACKEND=x11"
        )),
    }
}

fn ensure_c_lc_numeric() {
    #[cfg(target_os = "linux")]
    unsafe {
        if let Ok(libc) = Library::new("libc.so.6")
            && let Ok(setlocale) = libc
                .get::<unsafe extern "C" fn(c_int, *const c_char) -> *const c_char>(b"setlocale\0")
        {
            // GTK initializes LC_NUMERIC from user locale (e.g. pt_BR.UTF-8), causing libmpv's mpv_create() to return NULL.
            const LC_NUMERIC: c_int = 1;
            setlocale(LC_NUMERIC, c"C".as_ptr().cast());
        }
    }
}

fn create_and_init_mpv(
    api: &MpvApi,
    wid: &str,
    headers: Option<&HashMap<String, String>>,
    start_secs: Option<f64>,
    options: &crate::video_options::VideoOptions,
) -> Result<*mut c_void, String> {
    ensure_c_lc_numeric();

    #[cfg(target_os = "linux")]
    let attempts: &[(&str, &str, &str)] = &[
        ("gpu", "x11egl", "auto"),
        ("gpu", "x11vk", "nvdec-copy"),
        ("gpu", "x11", "auto"),
    ];
    #[cfg(target_os = "windows")]
    let attempts: &[(&str, &str, &str)] = &[
        ("gpu-next,gpu", "auto", "auto"),
        ("gpu", "d3d11", "d3d11va"),
        ("gpu", "win", "auto"),
    ];
    #[cfg(all(not(target_os = "linux"), not(target_os = "windows")))]
    let attempts: &[(&str, &str, &str)] = &[("gpu", "auto", "auto")];

    let mut last_err = 0;
    for (vo, gpu_ctx, hwdec) in attempts {
        let handle = unsafe { (api.create)() };
        if handle.is_null() {
            return Err("mpv_create returned null".into());
        }
        let _ = mpv_set_opt(api, handle, "wid", wid);
        let _ = mpv_set_opt(api, handle, "osc", "no");
        let _ = mpv_set_opt(api, handle, "osd-level", "0");
        let _ = mpv_set_opt(api, handle, "input-default-bindings", "no");
        let _ = mpv_set_opt(api, handle, "input-vo-keyboard", "no");
        let _ = mpv_set_opt(api, handle, "input-cursor", "no");
        let _ = mpv_set_opt(api, handle, "cursor-autohide", "no");
        let _ = mpv_set_opt(api, handle, "keep-open", "yes");
        let _ = mpv_set_opt(api, handle, "idle", "yes");
        let _ = mpv_set_opt(api, handle, "vo", vo);
        let _ = mpv_set_opt(api, handle, "force-window", "immediate");
        let _ = mpv_set_opt(api, handle, "gpu-context", gpu_ctx);
        let _ = mpv_set_opt(api, handle, "hwdec", hwdec);
        let _ = mpv_set_opt(api, handle, "gpu-hwdec-interop", "auto");
        let _ = mpv_set_opt(api, handle, "vd-lavc-software-fallback", "yes");
        let _ = mpv_set_opt(api, handle, "force-seekable", "yes");
        let _ = mpv_set_opt(api, handle, "volume-max", "200");
        // Bound the demuxer cache: keeps >=20 s lookahead without ballooning RSS.
        let _ = mpv_set_opt(api, handle, "demuxer-max-bytes", "48MiB");
        let _ = mpv_set_opt(api, handle, "demuxer-max-back-bytes", "16MiB");
        // Keep the network cache in memory only and bounded: a long-running
        // stream must not grow its cache indefinitely.
        let _ = mpv_set_opt(api, handle, "cache", "auto");
        let _ = mpv_set_opt(api, handle, "cache-on-disk", "no");
        let _ = mpv_set_opt(api, handle, "demuxer-readahead-secs", "20");
        let _ = mpv_set_opt(api, handle, "audio-buffer", "0.5");
        let _ = mpv_set_opt(api, handle, "sub-ass-override", "no");
        let _ = mpv_set_opt(api, handle, "audio-channels", "auto");

        if let Some(secs) = start_secs
            && secs > 2.0
        {
            let _ = mpv_set_opt(api, handle, "start", &format!("{secs:.1}"));
        }

        if let Some(hdrs) = headers {
            let fields = format_http_header_fields(hdrs);
            if !fields.is_empty() {
                let _ = mpv_set_opt_string_array(api, handle, "http-header-fields", &fields);
            }
        }

        for &(key, value) in &options.options {
            if let Err(error) = mpv_set_opt(api, handle, key, value) {
                unsafe { (api.terminate_destroy)(handle) };
                return Err(error);
            }
        }
        if let Some(path) = &options.config {
            let filename =
                CString::new(path.to_string_lossy().as_bytes()).expect("filesystem path");
            if unsafe { (api.load_config)(handle, filename.as_ptr()) } < 0 {
                unsafe { (api.terminate_destroy)(handle) };
                return Err("Falha ao carregar configuração mpv".into());
            }
        }
        let rc = unsafe { (api.initialize)(handle) };
        if rc >= 0 {
            return Ok(handle);
        }
        last_err = rc;
        unsafe { (api.terminate_destroy)(handle) };
    }
    Err(format!("mpv_initialize failed (error {last_err})"))
}

pub fn format_http_header_fields(headers: &HashMap<String, String>) -> Vec<String> {
    let mut fields: Vec<String> = headers
        .iter()
        .filter_map(|(k, v)| {
            let clean_k = k.replace(['\r', '\n'], "");
            let clean_k = clean_k.trim();
            if clean_k.is_empty() || clean_k.eq_ignore_ascii_case("range") {
                return None;
            }
            let clean_v = v.replace(['\r', '\n'], "");
            let clean_v = clean_v.trim();
            Some(format!("{clean_k}: {clean_v}"))
        })
        .collect();
    fields.sort();
    fields
}

pub fn format_external_mpv_header_fields(headers: &HashMap<String, String>) -> Option<String> {
    let fields = format_http_header_fields(headers);
    if fields.is_empty() {
        return None;
    }
    let escaped: Vec<String> = fields
        .into_iter()
        .map(|entry| entry.replace('\\', "\\\\").replace(',', "\\,"))
        .collect();
    Some(escaped.join(","))
}

fn mpv_set_opt(api: &MpvApi, handle: *mut c_void, k: &str, v: &str) -> Result<(), String> {
    let ck = CString::new(k).map_err(|e| e.to_string())?;
    let cv = CString::new(v).map_err(|e| e.to_string())?;
    let rc = unsafe { (api.set_option_string)(handle, ck.as_ptr(), cv.as_ptr()) };
    if rc < 0 {
        Err(format!("mpv_set_option_string({k}) failed: {rc}"))
    } else {
        Ok(())
    }
}

fn mpv_set_opt_string_array(
    api: &MpvApi,
    handle: *mut c_void,
    k: &str,
    items: &[String],
) -> Result<(), String> {
    let ck = CString::new(k).map_err(|e| e.to_string())?;
    let c_items: Vec<CString> = items
        .iter()
        .map(|s| CString::new(s.as_str()).map_err(|e| e.to_string()))
        .collect::<Result<_, _>>()?;
    let mut nodes: Vec<MpvNode> = c_items
        .iter()
        .map(|cs| MpvNode {
            u: MpvNodeValue {
                string: cs.as_ptr(),
            },
            format: MPV_FORMAT_STRING,
        })
        .collect();
    let mut node_list = MpvNodeList {
        num: nodes.len() as c_int,
        values: nodes.as_mut_ptr(),
        keys: std::ptr::null_mut(),
    };
    let mut root = MpvNode {
        u: MpvNodeValue {
            list: &mut node_list,
        },
        format: MPV_FORMAT_NODE_ARRAY,
    };
    let rc = unsafe {
        (api.set_option)(
            handle,
            ck.as_ptr(),
            MPV_FORMAT_NODE,
            (&mut root as *mut MpvNode).cast(),
        )
    };
    if rc < 0 {
        Err(format!("mpv_set_option({k}) failed: {rc}"))
    } else {
        Ok(())
    }
}

fn mpv_set_prop(api: &MpvApi, handle: *mut c_void, k: &str, v: &str) -> Result<(), String> {
    let ck = CString::new(k).map_err(|e| e.to_string())?;
    let cv = CString::new(v).map_err(|e| e.to_string())?;
    let rc = unsafe { (api.set_property_string)(handle, ck.as_ptr(), cv.as_ptr()) };
    if rc < 0 {
        Err(format!("mpv_set_property_string({k}) failed: {rc}"))
    } else {
        Ok(())
    }
}

fn mpv_cmd(api: &MpvApi, handle: *mut c_void, args: &[&str]) -> Result<(), String> {
    let c_args: Vec<CString> = args
        .iter()
        .map(|s| CString::new(*s).map_err(|e| e.to_string()))
        .collect::<Result<_, _>>()?;
    let mut ptrs: Vec<*const c_char> = c_args.iter().map(|c| c.as_ptr()).collect();
    ptrs.push(std::ptr::null());
    let rc = unsafe { (api.command)(handle, ptrs.as_ptr()) };
    if rc < 0 {
        Err(format!("mpv_command({args:?}) failed: {rc}"))
    } else {
        Ok(())
    }
}

fn get_prop_double(api: &MpvApi, handle: *mut c_void, name: &str) -> f64 {
    let Ok(cname) = CString::new(name) else {
        return 0.0;
    };
    let mut val: c_double = 0.0;
    let rc = unsafe {
        (api.get_property)(
            handle,
            cname.as_ptr(),
            MPV_FORMAT_DOUBLE,
            (&mut val as *mut c_double).cast(),
        )
    };
    if rc >= 0 && val.is_finite() { val } else { 0.0 }
}

fn get_prop_flag(api: &MpvApi, handle: *mut c_void, name: &str) -> bool {
    let Ok(cname) = CString::new(name) else {
        return false;
    };
    let mut val: c_int = 0;
    let rc = unsafe {
        (api.get_property)(
            handle,
            cname.as_ptr(),
            MPV_FORMAT_FLAG,
            (&mut val as *mut c_int).cast(),
        )
    };
    rc >= 0 && val != 0
}

fn get_prop_i64(api: &MpvApi, handle: *mut c_void, name: &str) -> i64 {
    let Ok(cname) = CString::new(name) else {
        return 0;
    };
    let mut val: i64 = 0;
    let rc = unsafe {
        (api.get_property)(
            handle,
            cname.as_ptr(),
            MPV_FORMAT_INT64,
            (&mut val as *mut i64).cast(),
        )
    };
    if rc >= 0 { val } else { 0 }
}

fn get_prop_string(api: &MpvApi, handle: *mut c_void, name: &str) -> String {
    let Ok(cname) = CString::new(name) else {
        return String::new();
    };
    let mut ptr: *mut c_char = std::ptr::null_mut();
    let rc = unsafe {
        (api.get_property)(
            handle,
            cname.as_ptr(),
            MPV_FORMAT_STRING,
            (&mut ptr as *mut *mut c_char).cast(),
        )
    };
    if rc >= 0 && !ptr.is_null() {
        let out = unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() };
        unsafe { (api.free)(ptr.cast()) };
        out
    } else {
        String::new()
    }
}

fn format_track_label(title: &str, lang: &str, codec: &str, ch: &str, idx: usize) -> String {
    let base = if !title.trim().is_empty() {
        title.trim().to_string()
    } else if !lang.trim().is_empty() {
        lang.trim().to_string()
    } else {
        format!("Faixa {}", idx + 1)
    };
    let extra: Vec<&str> = [ch.trim(), codec.trim()]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect();
    if extra.is_empty() {
        base
    } else {
        format!("{base} ({})", extra.join(", "))
    }
}

const PROP_TRACK_LIST: u64 = 1;
const PROP_AID: u64 = 2;
const PROP_SID: u64 = 3;
const PROP_CHAPTER_COUNT: u64 = 4;
const PROP_DURATION: u64 = 5;
const PROP_TIME_POS: u64 = 6;
const PROP_DEMUXER_CACHE_TIME: u64 = 7;
const PROP_PAUSE: u64 = 8;
const PROP_EOF_REACHED: u64 = 9;
const PROP_PAUSED_FOR_CACHE: u64 = 10;
const PROP_CORE_IDLE: u64 = 11;
const PROP_VOLUME: u64 = 12;
const PROP_SPEED: u64 = 13;
const PROP_VIDEO_W: u64 = 14;
const PROP_VIDEO_H: u64 = 15;
const PROP_VIDEO_FORMAT: u64 = 16;
const PROP_HWDEC_CURRENT: u64 = 17;
const PROP_AUDIO_CODEC: u64 = 18;
const PROP_AUDIO_CHANNELS: u64 = 19;
const PROP_SECONDARY_SID: u64 = 20;
const PROP_SECONDARY_SUB_TEXT: u64 = 21;
const PROP_SECONDARY_SUB_START: u64 = 22;

fn observe_player_properties(api: &MpvApi, handle: *mut c_void) {
    let observed: &[(u64, &str, c_int)] = &[
        (PROP_TRACK_LIST, "track-list", MPV_FORMAT_NONE),
        (PROP_AID, "aid", MPV_FORMAT_NONE),
        (PROP_SID, "sid", MPV_FORMAT_NONE),
        (PROP_CHAPTER_COUNT, "chapter-list/count", MPV_FORMAT_INT64),
        (PROP_DURATION, "duration", MPV_FORMAT_DOUBLE),
        (PROP_TIME_POS, "time-pos", MPV_FORMAT_DOUBLE),
        (
            PROP_DEMUXER_CACHE_TIME,
            "demuxer-cache-time",
            MPV_FORMAT_DOUBLE,
        ),
        (PROP_PAUSE, "pause", MPV_FORMAT_FLAG),
        (PROP_EOF_REACHED, "eof-reached", MPV_FORMAT_FLAG),
        (PROP_PAUSED_FOR_CACHE, "paused-for-cache", MPV_FORMAT_FLAG),
        (PROP_CORE_IDLE, "core-idle", MPV_FORMAT_FLAG),
        (PROP_VOLUME, "volume", MPV_FORMAT_DOUBLE),
        (PROP_SPEED, "speed", MPV_FORMAT_DOUBLE),
        (PROP_VIDEO_W, "video-params/w", MPV_FORMAT_INT64),
        (PROP_VIDEO_H, "video-params/h", MPV_FORMAT_INT64),
        (PROP_VIDEO_FORMAT, "video-format", MPV_FORMAT_STRING),
        (PROP_HWDEC_CURRENT, "hwdec-current", MPV_FORMAT_STRING),
        (PROP_AUDIO_CODEC, "audio-codec-name", MPV_FORMAT_STRING),
        (
            PROP_AUDIO_CHANNELS,
            "audio-params/channels",
            MPV_FORMAT_STRING,
        ),
        (PROP_SECONDARY_SID, "secondary-sid", MPV_FORMAT_INT64),
        (
            PROP_SECONDARY_SUB_TEXT,
            "secondary-sub-text",
            MPV_FORMAT_STRING,
        ),
        (
            PROP_SECONDARY_SUB_START,
            "secondary-sub-start",
            MPV_FORMAT_DOUBLE,
        ),
    ];

    for &(tag, name, fmt) in observed {
        if let Ok(cname) = CString::new(name) {
            unsafe {
                (api.observe_property)(handle, tag, cname.as_ptr(), fmt);
            }
        }
    }
}

#[derive(Default)]
struct PropertyChangeFlags {
    tracks_dirty: bool,
    chapters_dirty: bool,
    discrete: bool,
    continuous: bool,
}

fn prop_event_double(prop: &MpvEventProperty) -> f64 {
    if prop.format == MPV_FORMAT_DOUBLE && !prop.data.is_null() {
        let val = unsafe { *(prop.data as *const c_double) };
        if val.is_finite() { val } else { 0.0 }
    } else {
        0.0
    }
}

fn prop_event_flag(prop: &MpvEventProperty) -> bool {
    if prop.format == MPV_FORMAT_FLAG && !prop.data.is_null() {
        unsafe { *(prop.data as *const c_int) != 0 }
    } else {
        false
    }
}

fn prop_event_i64(prop: &MpvEventProperty) -> i64 {
    if prop.format == MPV_FORMAT_INT64 && !prop.data.is_null() {
        unsafe { *(prop.data as *const i64) }
    } else {
        0
    }
}

fn prop_event_string(prop: &MpvEventProperty) -> String {
    if prop.format == MPV_FORMAT_STRING && !prop.data.is_null() {
        let ptr = unsafe { *(prop.data as *const *const c_char) };
        if !ptr.is_null() {
            return unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() };
        }
    }
    String::new()
}

struct CachedPlayerState {
    audio_tracks: Vec<MediaTrack>,
    subtitle_tracks: Vec<MediaTrack>,
    overlay_audio: Vec<OverlayTrack>,
    overlay_subs: Vec<OverlayTrack>,
    chapters: Vec<SkipChapter>,
    duration_secs: f64,
    position_secs: f64,
    demux_cache: f64,
    paused: bool,
    eof_reached: bool,
    paused_for_cache: bool,
    core_idle: bool,
    volume: f64,
    speed: f64,
    media_info: PlayerMediaInfo,
    reference_id: i64,
    reference_text: String,
    reference_start_secs: f64,
}

impl Default for CachedPlayerState {
    fn default() -> Self {
        Self {
            audio_tracks: Vec::new(),
            subtitle_tracks: Vec::new(),
            overlay_audio: Vec::new(),
            overlay_subs: Vec::new(),
            chapters: Vec::new(),
            duration_secs: 0.0,
            position_secs: 0.0,
            demux_cache: 0.0,
            paused: false,
            eof_reached: false,
            paused_for_cache: false,
            core_idle: false,
            volume: 100.0,
            speed: 1.0,
            media_info: PlayerMediaInfo {
                width: 0,
                height: 0,
                video_codec: String::new(),
                hwdec: String::new(),
                audio_codec: String::new(),
                audio_channels: String::new(),
            },
            reference_id: 0,
            reference_text: String::new(),
            reference_start_secs: 0.0,
        }
    }
}

impl CachedPlayerState {
    fn apply_property_event(
        &mut self,
        userdata: u64,
        prop: &MpvEventProperty,
    ) -> PropertyChangeFlags {
        let mut flags = PropertyChangeFlags::default();
        match userdata {
            PROP_TRACK_LIST | PROP_AID | PROP_SID => {
                flags.tracks_dirty = true;
            }
            PROP_CHAPTER_COUNT => {
                flags.chapters_dirty = true;
            }
            PROP_DURATION => {
                self.duration_secs = prop_event_double(prop);
                flags.discrete = true;
            }
            PROP_TIME_POS => {
                self.position_secs = prop_event_double(prop);
                flags.continuous = true;
            }
            PROP_DEMUXER_CACHE_TIME => {
                self.demux_cache = prop_event_double(prop);
                flags.continuous = true;
            }
            PROP_PAUSE => {
                self.paused = prop_event_flag(prop);
                flags.discrete = true;
            }
            PROP_EOF_REACHED => {
                self.eof_reached = prop_event_flag(prop);
                flags.discrete = true;
            }
            PROP_PAUSED_FOR_CACHE => {
                self.paused_for_cache = prop_event_flag(prop);
                flags.discrete = true;
            }
            PROP_CORE_IDLE => {
                self.core_idle = prop_event_flag(prop);
                flags.discrete = true;
            }
            PROP_VOLUME => {
                self.volume = prop_event_double(prop).clamp(0.0, 200.0);
                flags.discrete = true;
            }
            PROP_SPEED => {
                self.speed = prop_event_double(prop);
                flags.discrete = true;
            }
            PROP_VIDEO_W => {
                self.media_info.width = prop_event_i64(prop);
                flags.discrete = true;
            }
            PROP_VIDEO_H => {
                self.media_info.height = prop_event_i64(prop);
                flags.discrete = true;
            }
            PROP_VIDEO_FORMAT => {
                self.media_info.video_codec = prop_event_string(prop);
                flags.discrete = true;
            }
            PROP_HWDEC_CURRENT => {
                self.media_info.hwdec = prop_event_string(prop);
                flags.discrete = true;
            }
            PROP_AUDIO_CODEC => {
                self.media_info.audio_codec = prop_event_string(prop);
                flags.discrete = true;
            }
            PROP_AUDIO_CHANNELS => {
                self.media_info.audio_channels = prop_event_string(prop);
                flags.discrete = true;
            }
            PROP_SECONDARY_SID => {
                self.reference_id = prop_event_i64(prop);
                flags.discrete = true;
            }
            PROP_SECONDARY_SUB_TEXT => {
                self.reference_text = prop_event_string(prop);
                flags.discrete = true;
            }
            PROP_SECONDARY_SUB_START => {
                self.reference_start_secs = prop_event_double(prop);
                flags.discrete = true;
            }
            _ => {}
        }
        flags
    }

    fn refresh_tracks(&mut self, api: &MpvApi, handle: *mut c_void) {
        let count = get_prop_i64(api, handle, "track-list/count").clamp(0, 64);
        self.audio_tracks.clear();
        self.subtitle_tracks.clear();
        self.overlay_audio.clear();
        self.overlay_subs.clear();

        for i in 0..count {
            let t_type = get_prop_string(api, handle, &format!("track-list/{i}/type"));
            if t_type != "audio" && t_type != "sub" {
                continue;
            }
            let id = get_prop_i64(api, handle, &format!("track-list/{i}/id"));
            let title = get_prop_string(api, handle, &format!("track-list/{i}/title"));
            let lang = get_prop_string(api, handle, &format!("track-list/{i}/lang"));
            let codec = get_prop_string(api, handle, &format!("track-list/{i}/codec"));
            let channels = get_prop_string(api, handle, &format!("track-list/{i}/demux-channels"));
            let selected = get_prop_flag(api, handle, &format!("track-list/{i}/selected"));
            let forced = get_prop_flag(api, handle, &format!("track-list/{i}/forced"));

            let track = MediaTrack {
                external: get_prop_flag(api, handle, &format!("track-list/{i}/external")),
                id,
                track_type: t_type.clone(),
                title: title.clone(),
                lang: lang.clone(),
                codec: codec.clone(),
                channels: channels.clone(),
                selected,
                forced,
            };
            if t_type == "audio" {
                let idx = self.audio_tracks.len();
                self.overlay_audio.push(OverlayTrack {
                    index: idx,
                    id: id.to_string(),
                    label: format_track_label(&title, &lang, &codec, &channels, idx),
                    language: lang,
                    selected,
                    forced,
                });
                self.audio_tracks.push(track);
            } else {
                let idx = self.subtitle_tracks.len();
                self.overlay_subs.push(OverlayTrack {
                    index: idx,
                    id: id.to_string(),
                    label: format_track_label(&title, &lang, &codec, "", idx),
                    language: lang,
                    selected,
                    forced,
                });
                self.subtitle_tracks.push(track);
            }
        }
    }

    fn refresh_chapters(&mut self, api: &MpvApi, handle: *mut c_void) {
        let chap_count = get_prop_i64(api, handle, "chapter-list/count").clamp(0, 128);
        self.chapters.clear();
        for i in 0..chap_count {
            let start_s = get_prop_double(api, handle, &format!("chapter-list/{i}/time"));
            let title = get_prop_string(api, handle, &format!("chapter-list/{i}/title"));
            self.chapters.push(SkipChapter {
                start_ms: (start_s * 1000.0).round() as i64,
                end_ms: None,
                title,
            });
        }
    }

    fn build_payload_and_js(
        &self,
        first_frame_shown: bool,
        ended_flag: bool,
    ) -> (PlayerStatePayload, String) {
        let buffered_secs = self.demux_cache.max(self.position_secs).max(0.0);
        let ended = ended_flag || self.eof_reached;
        let loading = !first_frame_shown
            || self.paused_for_cache
            || (self.core_idle && !self.paused && !ended && self.duration_secs <= 0.0);
        let volume_level = (self.volume / 100.0).clamp(0.0, 2.0);

        let audio_json = serde_json::to_string(&self.overlay_audio).unwrap_or_else(|_| "[]".into());
        let sub_json = serde_json::to_string(&self.overlay_subs).unwrap_or_else(|_| "[]".into());
        let update_js = format!(
            "window.playerUpdate&&window.playerUpdate({{duration:{:.3},position:{:.3},bufferedPosition:{buffered_secs:.3},volumeLevel:{volume_level:.3},paused:{},loading:{loading},audioTracks:{audio_json},subtitleTracks:{sub_json}}})",
            self.duration_secs, self.position_secs, self.paused,
        );

        let payload = PlayerStatePayload {
            active: true,
            paused: self.paused,
            buffering: self.paused_for_cache,
            loading,
            ended,
            position_secs: self.position_secs,
            duration_secs: self.duration_secs,
            buffered_secs,
            volume: self.volume,
            speed: self.speed,
            audio_tracks: self.audio_tracks.clone(),
            subtitle_tracks: self.subtitle_tracks.clone(),
            reference_id: self.reference_id,
            reference_text: self.reference_text.clone(),
            reference_start_secs: self.reference_start_secs,
            chapters: self.chapters.clone(),
            media_info: self.media_info.clone(),
        };

        (payload, update_js)
    }
}

#[cfg(test)]
mod player_tests {
    use super::{
        CachedPlayerState, MpvNode, MpvNodeList, format_external_mpv_header_fields,
        format_http_header_fields,
    };
    use std::collections::HashMap;

    #[test]
    fn filters_range_and_preserves_commas_and_backslashes_for_node_array() {
        let mut headers = HashMap::new();
        headers.insert("Range".to_string(), "bytes=0-1024".to_string());
        headers.insert("range".to_string(), "bytes=512-".to_string());
        headers.insert("User-Agent".to_string(), "Mozilla/5.0 (Win64, x64)\\v1".to_string());
        headers.insert("Cookie".to_string(), "a=1,2; b=x\\y\r\n".to_string());
        headers.insert("   ".to_string(), "ignored".to_string());

        let fields = format_http_header_fields(&headers);
        assert_eq!(
            fields,
            vec![
                "Cookie: a=1,2; b=x\\y".to_string(),
                "User-Agent: Mozilla/5.0 (Win64, x64)\\v1".to_string(),
            ]
        );
    }

    #[test]
    fn external_header_fields_strip_range_and_escape_cli_delimiters() {
        let mut headers = HashMap::new();
        headers.insert("RANGE".to_string(), "bytes=0-1".to_string());
        assert_eq!(format_external_mpv_header_fields(&headers), None);

        headers.insert("Referer".to_string(), "https://ex.com/a,b\\c".to_string());
        assert_eq!(
            format_external_mpv_header_fields(&headers).as_deref(),
            Some("Referer: https://ex.com/a\\,b\\\\c")
        );
    }

    #[test]
    fn mpv_node_layout_matches_c_abi() {
        assert_eq!(std::mem::size_of::<MpvNode>(), 16);
        assert_eq!(std::mem::align_of::<MpvNode>(), 8);
        assert!(std::mem::size_of::<MpvNodeList>() >= 24);
    }

    #[test]
    fn cached_player_state_computes_loading_ended_and_buffer() {
        let mut state = CachedPlayerState {
            duration_secs: 120.0,
            position_secs: 30.0,
            demux_cache: 45.0,
            volume: 150.0,
            ..Default::default()
        };

        let (payload, js) = state.build_payload_and_js(true, false);
        assert!(!payload.loading);
        assert!(!payload.ended);
        assert_eq!(payload.buffered_secs, 45.0);
        assert!(js.contains("volumeLevel:1.500"));
        assert!(js.contains("position:30.000"));

        state.paused_for_cache = true;
        let (buffering_payload, _) = state.build_payload_and_js(true, false);
        assert!(buffering_payload.loading);
        assert!(buffering_payload.buffering);
    }
}

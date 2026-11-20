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

const MPV_FORMAT_STRING: c_int = 1;
const MPV_FORMAT_FLAG: c_int = 3;
const MPV_FORMAT_INT64: c_int = 4;
const MPV_FORMAT_DOUBLE: c_int = 5;

const MPV_EVENT_NONE: c_int = 0;
const MPV_EVENT_SHUTDOWN: c_int = 1;
const MPV_EVENT_COMMAND_REPLY: c_int = 5;
const MPV_EVENT_START_FILE: c_int = 6;
const MPV_EVENT_END_FILE: c_int = 7;
const MPV_EVENT_PLAYBACK_RESTART: c_int = 21;

#[repr(C)]
struct MpvEvent {
    event_id: c_int,
    error: c_int,
    reply_userdata: u64,
    data: *mut c_void,
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
    pub chapters: Vec<SkipChapter>,
    pub media_info: PlayerMediaInfo,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerUiEventPayload {
    pub event_type: String,
    pub value: f64,
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
type MpvInitializeFn = unsafe extern "C" fn(*mut c_void) -> c_int;
type MpvSetOptionStringFn =
    unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char) -> c_int;
type MpvCommandFn = unsafe extern "C" fn(*mut c_void, *const *const c_char) -> c_int;
type MpvCommandAsyncFn = unsafe extern "C" fn(*mut c_void, u64, *const *const c_char) -> c_int;
type MpvSetPropertyStringFn =
    unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char) -> c_int;
type MpvGetPropertyFn =
    unsafe extern "C" fn(*mut c_void, *const c_char, c_int, *mut c_void) -> c_int;
type MpvWaitEventFn = unsafe extern "C" fn(*mut c_void, c_double) -> *const MpvEvent;
type MpvWakeupFn = unsafe extern "C" fn(*mut c_void);
type MpvFreeFn = unsafe extern "C" fn(*mut c_void);
type MpvTerminateDestroyFn = unsafe extern "C" fn(*mut c_void);

#[derive(Clone)]
struct MpvApi {
    _lib: Arc<Library>,
    create: MpvCreateFn,
    initialize: MpvInitializeFn,
    set_option_string: MpvSetOptionStringFn,
    command: MpvCommandFn,
    command_async: MpvCommandAsyncFn,
    set_property_string: MpvSetPropertyStringFn,
    get_property: MpvGetPropertyFn,
    wait_event: MpvWaitEventFn,
    wakeup: MpvWakeupFn,
    free: MpvFreeFn,
    terminate_destroy: MpvTerminateDestroyFn,
}

impl MpvApi {
    fn load() -> Result<Self, String> {
        let candidates: &[&str] = if cfg!(target_os = "windows") {
            &["libmpv-2.dll", "mpv-2.dll"]
        } else if cfg!(target_os = "macos") {
            &["libmpv.2.dylib", "libmpv.dylib"]
        } else {
            &["libmpv.so.2", "libmpv.so"]
        };

        let mut last_err = String::new();
        for &name in candidates {
            match unsafe { Library::new(name) } {
                Ok(lib) => unsafe {
                    let create: Symbol<MpvCreateFn> =
                        lib.get(b"mpv_create\0").map_err(|e| e.to_string())?;
                    let initialize: Symbol<MpvInitializeFn> =
                        lib.get(b"mpv_initialize\0").map_err(|e| e.to_string())?;
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
                    let wait_event: Symbol<MpvWaitEventFn> =
                        lib.get(b"mpv_wait_event\0").map_err(|e| e.to_string())?;
                    let wakeup: Symbol<MpvWakeupFn> =
                        lib.get(b"mpv_wakeup\0").map_err(|e| e.to_string())?;
                    let free: Symbol<MpvFreeFn> =
                        lib.get(b"mpv_free\0").map_err(|e| e.to_string())?;
                    let terminate_destroy: Symbol<MpvTerminateDestroyFn> = lib
                        .get(b"mpv_terminate_destroy\0")
                        .map_err(|e| e.to_string())?;

                    return Ok(Self {
                        create: *create,
                        initialize: *initialize,
                        set_option_string: *set_option_string,
                        command: *command,
                        command_async: *command_async,
                        set_property_string: *set_property_string,
                        get_property: *get_property,
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
        Err(format!("Failed to load libmpv: {last_err}"))
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
                let create_simple: Symbol<XCreateSimpleWindowFn> =
                    x11.get(b"XCreateSimpleWindow\0").map_err(|e| e.to_string())?;
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

    const WS_CHILD: u32 = 0x40000000;
    const WS_VISIBLE: u32 = 0x10000000;
    const WS_CLIPSIBLINGS: u32 = 0x04000000;

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
    type MoveWindowFn = unsafe extern "system" fn(isize, c_int, c_int, c_int, c_int, c_int) -> c_int;
    type DestroyWindowFn = unsafe extern "system" fn(isize) -> c_int;

    pub struct ChildSurface {
        _user32: Library,
        hwnd: isize,
        move_window: MoveWindowFn,
        destroy_window: DestroyWindowFn,
    }

    unsafe impl Send for ChildSurface {}

    impl ChildSurface {
        pub fn create(parent_hwnd: isize, x: i32, y: i32, w: u32, h: u32) -> Result<Self, String> {
            unsafe {
                let user32 = Library::new("user32.dll").map_err(|e| e.to_string())?;
                let create_win: Symbol<CreateWindowExWFn> =
                    user32.get(b"CreateWindowExW\0").map_err(|e| e.to_string())?;
                let move_window: Symbol<MoveWindowFn> =
                    user32.get(b"MoveWindow\0").map_err(|e| e.to_string())?;
                let destroy_window: Symbol<DestroyWindowFn> =
                    user32.get(b"DestroyWindow\0").map_err(|e| e.to_string())?;

                let class_name: Vec<u16> = "STATIC\0".encode_utf16().collect();
                let title: Vec<u16> = "NuvioVideo\0".encode_utf16().collect();
                let hwnd = create_win(
                    0,
                    class_name.as_ptr(),
                    title.as_ptr(),
                    WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS,
                    x,
                    y,
                    w.max(16) as c_int,
                    h.max(16) as c_int,
                    parent_hwnd,
                    0,
                    0,
                    std::ptr::null_mut(),
                );
                if hwnd == 0 {
                    return Err("CreateWindowExW failed".into());
                }
                Ok(Self {
                    hwnd,
                    move_window: *move_window,
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
                (self.move_window)(
                    self.hwnd,
                    x,
                    y,
                    w.max(16) as c_int,
                    h.max(16) as c_int,
                    1,
                );
            }
        }
    }

    impl Drop for ChildSurface {
        fn drop(&mut self) {
            unsafe {
                (self.destroy_window)(self.hwnd);
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
    first_frame_shown: Arc<AtomicBool>,
    ended_flag: Arc<AtomicBool>,
    #[cfg(target_os = "linux")]
    overlay_shared: Arc<LinuxOverlayShared>,
    #[cfg(target_os = "linux")]
    overlay_handle: Option<LinuxOverlayHandle>,
    #[cfg(target_os = "windows")]
    win_overlay: Option<WebviewWindow>,
}

unsafe impl Send for ActiveSession {}

pub struct EmbeddedPlayer {
    session: Arc<Mutex<Option<ActiveSession>>>,
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
        }
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
    ) -> Result<bool, String> {
        self.stop();

        let raw_handle = window.window_handle().map_err(|e| e.to_string())?;
        let surface = match raw_handle.as_raw() {
            #[cfg(target_os = "linux")]
            raw @ (RawWindowHandle::Xlib(_) | RawWindowHandle::Xcb(_)) => {
                create_surface_for_linux(raw, rect)?
            }
            #[cfg(target_os = "windows")]
            RawWindowHandle::Win32(h) => {
                platform_host::ChildSurface::create(h.hwnd.get(), rect.x, rect.y, rect.w, rect.h)?
            }
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

        let handle = create_and_init_mpv(&api, &wid, headers.as_ref(), start_secs)?;
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
            LinuxOverlayHandle::attach(
                (host_id, parent_id),
                (controls_url.clone(), initial_controls_json.clone()),
                handle,
                api.command_async,
                api.command,
                Arc::clone(&overlay_shared),
                Box::new(move |ev_type, val| {
                    if shared_cb.mpv_alive.load(Ordering::Relaxed) {
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
        let win_overlay = create_windows_overlay(&app, window, &controls_url).ok();
        #[cfg(target_os = "windows")]
        if let (Some(win_ov), Some(init_json)) = (&win_overlay, &initial_controls_json) {
            let script = format!("window.playerControls&&window.playerControls({init_json})");
            let _ = win_ov.eval(&script);
        }
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
        let first_clone = Arc::clone(&first_frame_shown);
        let ended_clone = Arc::clone(&ended_flag);
        #[cfg(target_os = "linux")]
        let shared_loop = Arc::clone(&overlay_shared);
        let session_arc = Arc::clone(&self.session);

        *self.session.lock().expect("player mutex") = Some(ActiveSession {
            api: api.clone(),
            handle,
            surface,
            stop_flag,
            first_frame_shown,
            ended_flag,
            #[cfg(target_os = "linux")]
            overlay_shared,
            #[cfg(target_os = "linux")]
            overlay_handle,
            #[cfg(target_os = "windows")]
            win_overlay,
        });

        let api_evt = api.clone();
        let handle_addr_evt = handle as usize;
        std::thread::spawn(move || {
            let mut last_tick = std::time::Instant::now()
                .checked_sub(std::time::Duration::from_millis(250))
                .unwrap_or_else(std::time::Instant::now);

            while !stop_clone.load(Ordering::Relaxed) {
                let h_ptr = handle_addr_evt as *mut c_void;
                loop {
                    if stop_clone.load(Ordering::Relaxed) {
                        break;
                    }
                    let ev_ptr = unsafe { (api_evt.wait_event)(h_ptr, 0.03) };
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
                        }
                        MPV_EVENT_PLAYBACK_RESTART => {
                            first_clone.store(true, Ordering::Relaxed);
                        }
                        MPV_EVENT_END_FILE => {
                            ended_clone.store(true, Ordering::Relaxed);
                        }
                        MPV_EVENT_COMMAND_REPLY => {
                            #[cfg(target_os = "linux")]
                            if (ev.reply_userdata >> 32) == OVERLAY_REPLY_TAG {
                                shared_loop.mark_overlay_done(ev.reply_userdata & 0xffff_ffff);
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

                if last_tick.elapsed() >= std::time::Duration::from_millis(200) {
                    last_tick = std::time::Instant::now();
                    let guard = session_arc.lock().expect("player mutex");
                    let Some(sess) = guard.as_ref() else {
                        break;
                    };
                    let ff = sess.first_frame_shown.load(Ordering::Relaxed);
                    let ended = sess.ended_flag.load(Ordering::Relaxed);
                    let (payload, update_js) =
                        read_player_state(&sess.api, sess.handle, ff, ended);

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

                    let _ = app.emit("player:state", payload);
                }
            }
        });

        Ok(has_overlay)
    }

    pub fn push_controls_json(&self, controls_json: String) {
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
        mpv_cmd(&sess.api, sess.handle, &["seek", &target, "absolute"])
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
        let transparent_bg = style.background_color.to_ascii_lowercase().starts_with("#00");
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
                    (sess.api.terminate_destroy)(sess.handle);
                }
                schedule_x11_child_cleanup(dpy, xid);
            }
            #[cfg(not(target_os = "linux"))]
            {
                #[cfg(target_os = "windows")]
                if let Some(win_ov) = sess.win_overlay.take() {
                    let _ = win_ov.hide();
                    let _ = win_ov.close();
                }
                unsafe {
                    (sess.api.wakeup)(sess.handle);
                    (sess.api.terminate_destroy)(sess.handle);
                }
                drop(sess.surface);
            }
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
            let _ = mpv_cmd(api, handle, &["seek", &format!("{secs:.3}"), "absolute"]);
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
    controls_url: &str,
) -> Result<WebviewWindow, String> {
    use tauri::{WebviewUrl, WebviewWindowBuilder};
    if let Some(existing) = app.get_webview_window("player-overlay") {
        let _ = existing.show();
        return Ok(existing);
    }
    let pos = parent.inner_position().map_err(|e| e.to_string())?;
    let size = parent.inner_size().map_err(|e| e.to_string())?;
    let parsed_url = controls_url.parse().map_err(|e: url::ParseError| e.to_string())?;

    let init_script = r#"
      window.chrome = window.chrome || {};
      window.chrome.webview = {
        postMessage: function(msg) {
          if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke) {
            window.__TAURI_INTERNALS__.invoke('player_ui_message', { eventType: msg.type || '', value: Number(msg.value) || 0 });
          }
        }
      };
    "#;

    let ov = WebviewWindowBuilder::new(app, "player-overlay", WebviewUrl::External(parsed_url))
        .title("Nuvio Player Controls")
        .transparent(true)
        .decorations(false)
        .shadow(false)
        .skip_taskbar(true)
        .parent(parent)
        .map_err(|e| e.to_string())?
        .initialization_script(init_script)
        .build()
        .map_err(|e| e.to_string())?;
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
            && let Ok(setlocale) = libc.get::<unsafe extern "C" fn(c_int, *const c_char) -> *const c_char>(b"setlocale\0")
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
) -> Result<*mut c_void, String> {
    ensure_c_lc_numeric();

    #[cfg(target_os = "linux")]
    let attempts: &[(&str, &str)] = &[("x11egl", "auto"), ("x11vk", "nvdec-copy"), ("x11", "auto")];
    #[cfg(not(target_os = "linux"))]
    let attempts: &[(&str, &str)] = &[("auto", "auto")];

    let mut last_err = 0;
    for (gpu_ctx, hwdec) in attempts {
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
        let _ = mpv_set_opt(api, handle, "vo", "gpu");
        let _ = mpv_set_opt(api, handle, "force-window", "immediate");
        let _ = mpv_set_opt(api, handle, "gpu-context", gpu_ctx);
        let _ = mpv_set_opt(api, handle, "hwdec", hwdec);
        let _ = mpv_set_opt(api, handle, "gpu-hwdec-interop", "auto");
        let _ = mpv_set_opt(api, handle, "vd-lavc-software-fallback", "yes");
        let _ = mpv_set_opt(api, handle, "force-seekable", "yes");
        let _ = mpv_set_opt(api, handle, "volume-max", "200");
        let _ = mpv_set_opt(api, handle, "sub-ass-override", "no");
        let _ = mpv_set_opt(api, handle, "audio-channels", "auto");

        if let Some(secs) = start_secs
            && secs > 2.0
        {
            let _ = mpv_set_opt(api, handle, "start", &format!("{secs:.1}"));
        }

        if let Some(hdrs) = headers
            && !hdrs.is_empty()
        {
            let joined: Vec<String> = hdrs
                .iter()
                .map(|(k, v)| {
                    let esc_v = v.replace('\\', "\\\\").replace(',', "\\,");
                    format!("{k}: {esc_v}")
                })
                .collect();
            let _ = mpv_set_opt(api, handle, "http-header-fields", &joined.join(","));
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

fn read_player_state(
    api: &MpvApi,
    handle: *mut c_void,
    first_frame_shown: bool,
    ended_flag: bool,
) -> (PlayerStatePayload, String) {
    let count = get_prop_i64(api, handle, "track-list/count").clamp(0, 64);
    let mut audio_tracks = Vec::new();
    let mut subtitle_tracks = Vec::new();
    let mut overlay_audio = Vec::new();
    let mut overlay_subs = Vec::new();

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
            let idx = audio_tracks.len();
            overlay_audio.push(OverlayTrack {
                index: idx,
                id: id.to_string(),
                label: format_track_label(&title, &lang, &codec, &channels, idx),
                language: lang,
                selected,
                forced,
            });
            audio_tracks.push(track);
        } else {
            let idx = subtitle_tracks.len();
            overlay_subs.push(OverlayTrack {
                index: idx,
                id: id.to_string(),
                label: format_track_label(&title, &lang, &codec, "", idx),
                language: lang,
                selected,
                forced,
            });
            subtitle_tracks.push(track);
        }
    }

    let chap_count = get_prop_i64(api, handle, "chapter-list/count").clamp(0, 128);
    let mut chapters = Vec::new();
    for i in 0..chap_count {
        let start_s = get_prop_double(api, handle, &format!("chapter-list/{i}/time"));
        let title = get_prop_string(api, handle, &format!("chapter-list/{i}/title"));
        chapters.push(SkipChapter {
            start_ms: (start_s * 1000.0).round() as i64,
            end_ms: None,
            title,
        });
    }

    let duration_secs = get_prop_double(api, handle, "duration");
    let position_secs = get_prop_double(api, handle, "time-pos");
    let demux_cache = get_prop_double(api, handle, "demuxer-cache-time");
    let buffered_secs = demux_cache.max(position_secs).max(0.0);
    let paused = get_prop_flag(api, handle, "pause");
    let eof_reached = get_prop_flag(api, handle, "eof-reached");
    let paused_for_cache = get_prop_flag(api, handle, "paused-for-cache");
    let core_idle = get_prop_flag(api, handle, "core-idle");
    let ended = ended_flag || eof_reached;
    let loading = !first_frame_shown
        || paused_for_cache
        || (core_idle && !paused && !ended && duration_secs <= 0.0);

    let volume = get_prop_double(api, handle, "volume").clamp(0.0, 200.0);
    let volume_level = (volume / 100.0).clamp(0.0, 2.0);
    let speed = get_prop_double(api, handle, "speed");

    let media_info = PlayerMediaInfo {
        width: get_prop_i64(api, handle, "video-params/w"),
        height: get_prop_i64(api, handle, "video-params/h"),
        video_codec: get_prop_string(api, handle, "video-format"),
        hwdec: get_prop_string(api, handle, "hwdec-current"),
        audio_codec: get_prop_string(api, handle, "audio-codec-name"),
        audio_channels: get_prop_string(api, handle, "audio-params/channels"),
    };

    let audio_json = serde_json::to_string(&overlay_audio).unwrap_or_else(|_| "[]".into());
    let sub_json = serde_json::to_string(&overlay_subs).unwrap_or_else(|_| "[]".into());
    let update_js = format!(
        "window.playerUpdate&&window.playerUpdate({{duration:{duration_secs:.3},position:{position_secs:.3},bufferedPosition:{buffered_secs:.3},volumeLevel:{volume_level:.3},paused:{paused},loading:{loading},audioTracks:{audio_json},subtitleTracks:{sub_json}}})"
    );

    let payload = PlayerStatePayload {
        active: true,
        paused,
        buffering: paused_for_cache,
        loading,
        ended,
        position_secs,
        duration_secs,
        buffered_secs,
        volume,
        speed,
        audio_tracks,
        subtitle_tracks,
        chapters,
        media_info,
    };

    (payload, update_js)
}

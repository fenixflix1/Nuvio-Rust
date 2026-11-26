#![cfg(target_os = "linux")]

use libloading::Library;
use std::collections::HashSet;
use std::ffi::{CStr, CString, c_char, c_double, c_int, c_uint, c_ulong, c_void};
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

pub const OVERLAY_REPLY_TAG: u64 = 0x6f76_6c79;

const CAIRO_FORMAT_ARGB32: c_int = 0;
const GDK_FRAME_CLOCK_PHASE_UPDATE: c_uint = 1 << 2;
const GDK_EVENT_MASK_OVERLAY: c_int = (1 << 2)
    | (1 << 8)
    | (1 << 9)
    | (1 << 10)
    | (1 << 11)
    | (1 << 12)
    | (1 << 13)
    | (1 << 14)
    | (1 << 21)
    | (1 << 23);

#[repr(C)]
struct GdkRgba {
    red: c_double,
    green: c_double,
    blue: c_double,
    alpha: c_double,
}

#[repr(C)]
struct GdkRectangle {
    x: c_int,
    y: c_int,
    width: c_int,
    height: c_int,
}

#[repr(C)]
struct XWindowAttributes {
    x: c_int,
    y: c_int,
    width: c_int,
    height: c_int,
    _rest: [u8; 160],
}

#[repr(C)]
struct GdkEventButton {
    _type: c_int,
    _window: *mut c_void,
    _send_event: i8,
    _time: u32,
    _x: f64,
    _y: f64,
    _axes: *mut f64,
    _state: c_uint,
    button: c_uint,
    _device: *mut c_void,
    _x_root: f64,
    _y_root: f64,
}

pub type MpvCmdAsyncFn = unsafe extern "C" fn(*mut c_void, u64, *const *const c_char) -> c_int;
pub type MpvCmdSyncFn = unsafe extern "C" fn(*mut c_void, *const *const c_char) -> c_int;

struct GtkApis {
    _libs: Vec<Library>,
    g_idle_add: unsafe extern "C" fn(unsafe extern "C" fn(*mut c_void) -> c_int, *mut c_void) -> c_uint,
    g_timeout_add:
        unsafe extern "C" fn(c_uint, unsafe extern "C" fn(*mut c_void) -> c_int, *mut c_void) -> c_uint,
    g_source_remove: unsafe extern "C" fn(c_uint) -> c_int,
    g_main_context_find_source_by_id: unsafe extern "C" fn(*mut c_void, c_uint) -> *mut c_void,
    g_source_is_destroyed: unsafe extern "C" fn(*mut c_void) -> c_int,
    g_free: unsafe extern "C" fn(*mut c_void),
    g_error_free: unsafe extern "C" fn(*mut c_void),
    g_get_monotonic_time: unsafe extern "C" fn() -> i64,
    g_signal_connect_data: unsafe extern "C" fn(
        *mut c_void,
        *const c_char,
        *const c_void,
        *mut c_void,
        *const c_void,
        c_int,
    ) -> c_ulong,
    g_object_unref: unsafe extern "C" fn(*mut c_void),
    g_cancellable_new: unsafe extern "C" fn() -> *mut c_void,
    g_cancellable_cancel: unsafe extern "C" fn(*mut c_void),

    gtk_window_new: unsafe extern "C" fn(c_int) -> *mut c_void,
    gtk_window_set_decorated: unsafe extern "C" fn(*mut c_void, c_int),
    gtk_window_set_accept_focus: unsafe extern "C" fn(*mut c_void, c_int),
    gtk_window_resize: unsafe extern "C" fn(*mut c_void, c_int, c_int),
    gtk_widget_set_app_paintable: unsafe extern "C" fn(*mut c_void, c_int),
    gtk_widget_set_can_focus: unsafe extern "C" fn(*mut c_void, c_int),
    gtk_widget_grab_focus: unsafe extern "C" fn(*mut c_void),
    gtk_widget_get_screen: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    gtk_widget_set_visual: unsafe extern "C" fn(*mut c_void, *mut c_void),
    gtk_widget_add_events: unsafe extern "C" fn(*mut c_void, c_int),
    gtk_widget_realize: unsafe extern "C" fn(*mut c_void),
    gtk_widget_get_window: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    gtk_widget_get_display: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    gtk_widget_show_all: unsafe extern "C" fn(*mut c_void),
    gtk_widget_hide: unsafe extern "C" fn(*mut c_void),
    gtk_widget_size_allocate: unsafe extern "C" fn(*mut c_void, *mut GdkRectangle),
    gtk_widget_destroy: unsafe extern "C" fn(*mut c_void),
    gtk_container_add: unsafe extern "C" fn(*mut c_void, *mut c_void),

    gdk_display_get_default: unsafe extern "C" fn() -> *mut c_void,
    gdk_display_sync: unsafe extern "C" fn(*mut c_void),
    gdk_screen_get_rgba_visual: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    gdk_window_get_display: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    gdk_window_destroy: unsafe extern "C" fn(*mut c_void),
    gdk_window_get_scale_factor: unsafe extern "C" fn(*mut c_void) -> c_int,
    gdk_window_reparent: unsafe extern "C" fn(*mut c_void, *mut c_void, c_int, c_int),
    gdk_window_raise: unsafe extern "C" fn(*mut c_void),
    gdk_window_move_resize: unsafe extern "C" fn(*mut c_void, c_int, c_int, c_int, c_int),
    gdk_window_set_event_compression: unsafe extern "C" fn(*mut c_void, c_int),
    gdk_window_get_frame_clock: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    gdk_frame_clock_request_phase: unsafe extern "C" fn(*mut c_void, c_uint),
    gdk_cursor_new_from_name: unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_void,
    gdk_window_set_cursor: unsafe extern "C" fn(*mut c_void, *mut c_void),
    gdk_x11_window_foreign_new_for_display: unsafe extern "C" fn(*mut c_void, c_ulong) -> *mut c_void,
    gdk_x11_window_get_xid: unsafe extern "C" fn(*mut c_void) -> c_ulong,
    gdk_x11_display_get_xdisplay: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    gdk_x11_display_error_trap_push: unsafe extern "C" fn(*mut c_void),
    gdk_x11_display_error_trap_pop_ignored: unsafe extern "C" fn(*mut c_void),
    gdk_x11_window_set_frame_sync_enabled: unsafe extern "C" fn(*mut c_void, c_int),

    webkit_web_context_new_ephemeral: unsafe extern "C" fn() -> *mut c_void,
    webkit_web_view_new_with_context: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    webkit_web_view_get_user_content_manager: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    webkit_web_view_terminate_web_process: unsafe extern "C" fn(*mut c_void),
    webkit_user_content_manager_register_script_message_handler:
        unsafe extern "C" fn(*mut c_void, *const c_char) -> c_int,
    webkit_web_view_set_background_color: unsafe extern "C" fn(*mut c_void, *const GdkRgba),
    webkit_web_view_get_settings: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    webkit_settings_set_allow_file_access_from_file_urls: unsafe extern "C" fn(*mut c_void, c_int),
    webkit_settings_set_allow_universal_access_from_file_urls:
        unsafe extern "C" fn(*mut c_void, c_int),
    webkit_settings_set_javascript_can_access_clipboard: unsafe extern "C" fn(*mut c_void, c_int),
    webkit_settings_set_hardware_acceleration_policy: unsafe extern "C" fn(*mut c_void, c_int),
    webkit_web_view_load_uri: unsafe extern "C" fn(*mut c_void, *const c_char),
    webkit_web_view_evaluate_javascript: unsafe extern "C" fn(
        *mut c_void,
        *const c_char,
        isize,
        *const c_char,
        *const c_char,
        *mut c_void,
        *const c_void,
        *mut c_void,
    ),
    webkit_web_view_get_snapshot: unsafe extern "C" fn(
        *mut c_void,
        c_int,
        c_int,
        *mut c_void,
        unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void),
        *mut c_void,
    ),
    webkit_web_view_get_snapshot_finish:
        unsafe extern "C" fn(*mut c_void, *mut c_void, *mut *mut c_void) -> *mut c_void,
    webkit_javascript_result_get_js_value: unsafe extern "C" fn(*mut c_void) -> *mut c_void,

    jsc_value_is_object: unsafe extern "C" fn(*mut c_void) -> c_int,
    jsc_value_is_number: unsafe extern "C" fn(*mut c_void) -> c_int,
    jsc_value_object_get_property: unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_void,
    jsc_value_to_string: unsafe extern "C" fn(*mut c_void) -> *mut c_char,
    jsc_value_to_double: unsafe extern "C" fn(*mut c_void) -> c_double,

    cairo_image_surface_get_format: unsafe extern "C" fn(*mut c_void) -> c_int,
    cairo_image_surface_get_width: unsafe extern "C" fn(*mut c_void) -> c_int,
    cairo_image_surface_get_height: unsafe extern "C" fn(*mut c_void) -> c_int,
    cairo_image_surface_get_stride: unsafe extern "C" fn(*mut c_void) -> c_int,
    cairo_image_surface_get_data: unsafe extern "C" fn(*mut c_void) -> *const u8,
    cairo_surface_flush: unsafe extern "C" fn(*mut c_void),
    cairo_surface_destroy: unsafe extern "C" fn(*mut c_void),

    x_composite_query_extension:
        unsafe extern "C" fn(*mut c_void, *mut c_int, *mut c_int) -> c_int,
    x_composite_redirect_window: unsafe extern "C" fn(*mut c_void, c_ulong, c_int),
    x_get_window_attributes:
        unsafe extern "C" fn(*mut c_void, c_ulong, *mut XWindowAttributes) -> c_int,
    x_query_tree: unsafe extern "C" fn(
        *mut c_void,
        c_ulong,
        *mut c_ulong,
        *mut c_ulong,
        *mut *mut c_ulong,
        *mut c_uint,
    ) -> c_int,
    x_raise_window: unsafe extern "C" fn(*mut c_void, c_ulong) -> c_int,
    x_default_root_window: unsafe extern "C" fn(*mut c_void) -> c_ulong,
    x_translate_coordinates: unsafe extern "C" fn(
        *mut c_void,
        c_ulong,
        c_ulong,
        c_int,
        c_int,
        *mut c_int,
        *mut c_int,
        *mut c_ulong,
    ) -> c_int,
    x_query_pointer: unsafe extern "C" fn(
        *mut c_void,
        c_ulong,
        *mut c_ulong,
        *mut c_ulong,
        *mut c_int,
        *mut c_int,
        *mut c_int,
        *mut c_int,
        *mut c_uint,
    ) -> c_int,
    x_get_input_focus: unsafe extern "C" fn(*mut c_void, *mut c_ulong, *mut c_int) -> c_int,
    x_set_input_focus: unsafe extern "C" fn(*mut c_void, c_ulong, c_int, c_ulong) -> c_int,
    x_destroy_window: unsafe extern "C" fn(*mut c_void, c_ulong) -> c_int,
    x_close_display: unsafe extern "C" fn(*mut c_void) -> c_int,
    x_sync: unsafe extern "C" fn(*mut c_void, c_int) -> c_int,
    x_free: unsafe extern "C" fn(*mut c_void) -> c_int,
    x_flush: unsafe extern "C" fn(*mut c_void) -> c_int,
}

unsafe impl Send for GtkApis {}
unsafe impl Sync for GtkApis {}

fn load_sym<T: Copy>(lib: &Library, name: &[u8]) -> Result<T, String> {
    unsafe {
        let sym = lib
            .get::<T>(name)
            .map_err(|e| format!("Symbol {} missing: {e}", String::from_utf8_lossy(name)))?;
        Ok(*sym)
    }
}

fn gtk_apis() -> Result<&'static GtkApis, String> {
    static APIS: OnceLock<Result<GtkApis, String>> = OnceLock::new();
    let res = APIS.get_or_init(|| unsafe {
        let load = |candidates: &[&str]| -> Result<Library, String> {
            for c in candidates {
                if let Ok(lib) = Library::new(*c) {
                    return Ok(lib);
                }
            }
            Err(format!("Failed to load any of {candidates:?}"))
        };
        let glib = load(&["libglib-2.0.so.0", "libglib-2.0.so"])?;
        let gobj = load(&["libgobject-2.0.so.0", "libgobject-2.0.so"])?;
        let gio = load(&["libgio-2.0.so.0", "libgio-2.0.so"])?;
        let gtk = load(&["libgtk-3.so.0", "libgtk-3.so"])?;
        let gdk = load(&["libgdk-3.so.0", "libgdk-3.so"])?;
        let wk = load(&["libwebkit2gtk-4.1.so.0", "libwebkit2gtk-4.1.so"])?;
        let jsc = load(&["libjavascriptcoregtk-4.1.so.0", "libjavascriptcoregtk-4.1.so"])?;
        let cairo = load(&["libcairo.so.2", "libcairo.so"])?;
        let xcomp = load(&["libXcomposite.so.1", "libXcomposite.so"])?;
        let x11 = load(&["libX11.so.6", "libX11.so"])?;

        Ok(GtkApis {
            g_idle_add: load_sym(&glib, b"g_idle_add\0")?,
            g_timeout_add: load_sym(&glib, b"g_timeout_add\0")?,
            g_source_remove: load_sym(&glib, b"g_source_remove\0")?,
            g_main_context_find_source_by_id: load_sym(&glib, b"g_main_context_find_source_by_id\0")?,
            g_source_is_destroyed: load_sym(&glib, b"g_source_is_destroyed\0")?,
            g_free: load_sym(&glib, b"g_free\0")?,
            g_error_free: load_sym(&glib, b"g_error_free\0")?,
            g_get_monotonic_time: load_sym(&glib, b"g_get_monotonic_time\0")?,
            g_signal_connect_data: load_sym(&gobj, b"g_signal_connect_data\0")?,
            g_object_unref: load_sym(&gobj, b"g_object_unref\0")?,
            g_cancellable_new: load_sym(&gio, b"g_cancellable_new\0")?,
            g_cancellable_cancel: load_sym(&gio, b"g_cancellable_cancel\0")?,
            gtk_window_new: load_sym(&gtk, b"gtk_window_new\0")?,
            gtk_window_set_decorated: load_sym(&gtk, b"gtk_window_set_decorated\0")?,
            gtk_window_set_accept_focus: load_sym(&gtk, b"gtk_window_set_accept_focus\0")?,
            gtk_window_resize: load_sym(&gtk, b"gtk_window_resize\0")?,
            gtk_widget_set_app_paintable: load_sym(&gtk, b"gtk_widget_set_app_paintable\0")?,
            gtk_widget_set_can_focus: load_sym(&gtk, b"gtk_widget_set_can_focus\0")?,
            gtk_widget_grab_focus: load_sym(&gtk, b"gtk_widget_grab_focus\0")?,
            gtk_widget_get_screen: load_sym(&gtk, b"gtk_widget_get_screen\0")?,
            gtk_widget_set_visual: load_sym(&gtk, b"gtk_widget_set_visual\0")?,
            gtk_widget_add_events: load_sym(&gtk, b"gtk_widget_add_events\0")?,
            gtk_widget_realize: load_sym(&gtk, b"gtk_widget_realize\0")?,
            gtk_widget_get_window: load_sym(&gtk, b"gtk_widget_get_window\0")?,
            gtk_widget_get_display: load_sym(&gtk, b"gtk_widget_get_display\0")?,
            gtk_widget_show_all: load_sym(&gtk, b"gtk_widget_show_all\0")?,
            gtk_widget_hide: load_sym(&gtk, b"gtk_widget_hide\0")?,
            gtk_widget_size_allocate: load_sym(&gtk, b"gtk_widget_size_allocate\0")?,
            gtk_widget_destroy: load_sym(&gtk, b"gtk_widget_destroy\0")?,
            gtk_container_add: load_sym(&gtk, b"gtk_container_add\0")?,
            gdk_display_get_default: load_sym(&gdk, b"gdk_display_get_default\0")?,
            gdk_display_sync: load_sym(&gdk, b"gdk_display_sync\0")?,
            gdk_screen_get_rgba_visual: load_sym(&gdk, b"gdk_screen_get_rgba_visual\0")?,
            gdk_window_get_display: load_sym(&gdk, b"gdk_window_get_display\0")?,
            gdk_window_destroy: load_sym(&gdk, b"gdk_window_destroy\0")?,
            gdk_window_get_scale_factor: load_sym(&gdk, b"gdk_window_get_scale_factor\0")?,
            gdk_window_reparent: load_sym(&gdk, b"gdk_window_reparent\0")?,
            gdk_window_raise: load_sym(&gdk, b"gdk_window_raise\0")?,
            gdk_window_move_resize: load_sym(&gdk, b"gdk_window_move_resize\0")?,
            gdk_window_set_event_compression: load_sym(
                &gdk,
                b"gdk_window_set_event_compression\0",
            )?,
            gdk_window_get_frame_clock: load_sym(&gdk, b"gdk_window_get_frame_clock\0")?,
            gdk_frame_clock_request_phase: load_sym(&gdk, b"gdk_frame_clock_request_phase\0")?,
            gdk_cursor_new_from_name: load_sym(&gdk, b"gdk_cursor_new_from_name\0")?,
            gdk_window_set_cursor: load_sym(&gdk, b"gdk_window_set_cursor\0")?,
            gdk_x11_window_foreign_new_for_display: load_sym(
                &gdk,
                b"gdk_x11_window_foreign_new_for_display\0",
            )?,
            gdk_x11_window_get_xid: load_sym(&gdk, b"gdk_x11_window_get_xid\0")?,
            gdk_x11_display_get_xdisplay: load_sym(&gdk, b"gdk_x11_display_get_xdisplay\0")?,
            gdk_x11_display_error_trap_push: load_sym(
                &gdk,
                b"gdk_x11_display_error_trap_push\0",
            )?,
            gdk_x11_display_error_trap_pop_ignored: load_sym(
                &gdk,
                b"gdk_x11_display_error_trap_pop_ignored\0",
            )?,
            gdk_x11_window_set_frame_sync_enabled: load_sym(
                &gdk,
                b"gdk_x11_window_set_frame_sync_enabled\0",
            )?,
            webkit_web_context_new_ephemeral: load_sym(
                &wk,
                b"webkit_web_context_new_ephemeral\0",
            )?,
            webkit_web_view_new_with_context: load_sym(
                &wk,
                b"webkit_web_view_new_with_context\0",
            )?,
            webkit_web_view_get_user_content_manager: load_sym(
                &wk,
                b"webkit_web_view_get_user_content_manager\0",
            )?,
            webkit_web_view_terminate_web_process: load_sym(
                &wk,
                b"webkit_web_view_terminate_web_process\0",
            )?,
            webkit_user_content_manager_register_script_message_handler: load_sym(
                &wk,
                b"webkit_user_content_manager_register_script_message_handler\0",
            )?,
            webkit_web_view_set_background_color: load_sym(
                &wk,
                b"webkit_web_view_set_background_color\0",
            )?,
            webkit_web_view_get_settings: load_sym(&wk, b"webkit_web_view_get_settings\0")?,
            webkit_settings_set_allow_file_access_from_file_urls: load_sym(
                &wk,
                b"webkit_settings_set_allow_file_access_from_file_urls\0",
            )?,
            webkit_settings_set_allow_universal_access_from_file_urls: load_sym(
                &wk,
                b"webkit_settings_set_allow_universal_access_from_file_urls\0",
            )?,
            webkit_settings_set_javascript_can_access_clipboard: load_sym(
                &wk,
                b"webkit_settings_set_javascript_can_access_clipboard\0",
            )?,
            webkit_settings_set_hardware_acceleration_policy: load_sym(
                &wk,
                b"webkit_settings_set_hardware_acceleration_policy\0",
            )?,
            webkit_web_view_load_uri: load_sym(&wk, b"webkit_web_view_load_uri\0")?,
            webkit_web_view_evaluate_javascript: load_sym(
                &wk,
                b"webkit_web_view_evaluate_javascript\0",
            )?,
            webkit_web_view_get_snapshot: load_sym(&wk, b"webkit_web_view_get_snapshot\0")?,
            webkit_web_view_get_snapshot_finish: load_sym(
                &wk,
                b"webkit_web_view_get_snapshot_finish\0",
            )?,
            webkit_javascript_result_get_js_value: load_sym(
                &wk,
                b"webkit_javascript_result_get_js_value\0",
            )?,
            jsc_value_is_object: load_sym(&jsc, b"jsc_value_is_object\0")?,
            jsc_value_is_number: load_sym(&jsc, b"jsc_value_is_number\0")?,
            jsc_value_object_get_property: load_sym(&jsc, b"jsc_value_object_get_property\0")?,
            jsc_value_to_string: load_sym(&jsc, b"jsc_value_to_string\0")?,
            jsc_value_to_double: load_sym(&jsc, b"jsc_value_to_double\0")?,
            cairo_image_surface_get_format: load_sym(&cairo, b"cairo_image_surface_get_format\0")?,
            cairo_image_surface_get_width: load_sym(&cairo, b"cairo_image_surface_get_width\0")?,
            cairo_image_surface_get_height: load_sym(&cairo, b"cairo_image_surface_get_height\0")?,
            cairo_image_surface_get_stride: load_sym(&cairo, b"cairo_image_surface_get_stride\0")?,
            cairo_image_surface_get_data: load_sym(&cairo, b"cairo_image_surface_get_data\0")?,
            cairo_surface_flush: load_sym(&cairo, b"cairo_surface_flush\0")?,
            cairo_surface_destroy: load_sym(&cairo, b"cairo_surface_destroy\0")?,
            x_composite_query_extension: load_sym(&xcomp, b"XCompositeQueryExtension\0")?,
            x_composite_redirect_window: load_sym(&xcomp, b"XCompositeRedirectWindow\0")?,
            x_get_window_attributes: load_sym(&x11, b"XGetWindowAttributes\0")?,
            x_query_tree: load_sym(&x11, b"XQueryTree\0")?,
            x_raise_window: load_sym(&x11, b"XRaiseWindow\0")?,
            x_default_root_window: load_sym(&x11, b"XDefaultRootWindow\0")?,
            x_translate_coordinates: load_sym(&x11, b"XTranslateCoordinates\0")?,
            x_query_pointer: load_sym(&x11, b"XQueryPointer\0")?,
            x_get_input_focus: load_sym(&x11, b"XGetInputFocus\0")?,
            x_set_input_focus: load_sym(&x11, b"XSetInputFocus\0")?,
            x_destroy_window: load_sym(&x11, b"XDestroyWindow\0")?,
            x_close_display: load_sym(&x11, b"XCloseDisplay\0")?,
            x_sync: load_sym(&x11, b"XSync\0")?,
            x_free: load_sym(&x11, b"XFree\0")?,
            x_flush: load_sym(&x11, b"XFlush\0")?,
            _libs: vec![glib, gobj, gio, gtk, gdk, wk, jsc, cairo, xcomp, x11],
        })
    });
    match res {
        Ok(a) => Ok(a),
        Err(e) => Err(e.clone()),
    }
}

fn live_overlays() -> &'static Mutex<HashSet<usize>> {
    static LIVE: OnceLock<Mutex<HashSet<usize>>> = OnceLock::new();
    LIVE.get_or_init(|| Mutex::new(HashSet::new()))
}

fn is_alive(ptr: usize) -> bool {
    live_overlays().lock().expect("live lock").contains(&ptr)
}

pub struct LinuxOverlayShared {
    pub seq_issued: AtomicU64,
    pub seq_done: AtomicU64,
    pub loading: AtomicBool,
    pub paused: AtomicBool,
    pub mpv_alive: AtomicBool,
}

impl LinuxOverlayShared {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            seq_issued: AtomicU64::new(0),
            seq_done: AtomicU64::new(0),
            loading: AtomicBool::new(true),
            paused: AtomicBool::new(false),
            mpv_alive: AtomicBool::new(true),
        })
    }

    pub fn mark_overlay_done(&self, seq: u64) {
        let mut cur = self.seq_done.load(Ordering::Relaxed);
        while seq > cur {
            match self.seq_done.compare_exchange_weak(
                cur,
                seq,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => cur = actual,
            }
        }
    }
}

type EventCallback = Box<dyn Fn(&str, f64) + Send + Sync>;

struct OverlayImpl {
    apis: &'static GtkApis,
    host_xid: c_ulong,
    restore_xid: c_ulong,
    controls_url: String,
    mpv_handle: *mut c_void,
    mpv_cmd_async: MpvCmdAsyncFn,
    mpv_cmd_sync: MpvCmdSyncFn,
    shared: Arc<LinuxOverlayShared>,
    on_event: EventCallback,

    // GTK-thread only fields
    gtk_window: *mut c_void,
    webview: *mut c_void,
    host_gdk: *mut c_void,
    overlay_xid: c_ulong,
    composite_timer: c_uint,
    snap_cancel: *mut c_void,
    snap_surf: *mut c_void,
    snap_surf_prev: *mut c_void,
    cursor_visible: *mut c_void,
    cursor_hidden: *mut c_void,

    snap_gen: AtomicU32,
    snap_in_flight: AtomicBool,
    snap_wait_ticks: AtomicU32,
    snap_cooldown: AtomicU32,
    snap_still: AtomicU32,
    snap_skip: AtomicU32,
    overlay_active: AtomicBool,
    overlay_pushed: AtomicBool,
    fade_ticks: AtomicU32,
    toast_ticks: AtomicU32,
    skip_shown: AtomicBool,
    next_ep_shown: AtomicBool,
    overlay_dirty_reported: AtomicBool,
    overlay_async_at: AtomicI64,

    pending_controls_json: Mutex<Option<String>>,
    last_update_js: Mutex<String>,
    pending_js_queue: Mutex<Vec<String>>,
    last_root_x: c_int,
    last_root_y: c_int,
    last_ptr_x: c_int,
    last_ptr_y: c_int,
}

unsafe impl Send for OverlayImpl {}
unsafe impl Sync for OverlayImpl {}

pub struct LinuxOverlayHandle {
    raw_ptr: usize,
}

impl LinuxOverlayHandle {
    pub fn attach(
        (host_xid, restore_xid): (u64, u64),
        (controls_url, initial_controls_json): (String, Option<String>),
        mpv_handle: *mut c_void,
        mpv_cmd_async: MpvCmdAsyncFn,
        mpv_cmd_sync: MpvCmdSyncFn,
        shared: Arc<LinuxOverlayShared>,
        on_event: EventCallback,
    ) -> Result<Self, String> {
        let apis = gtk_apis()?;
        let boxed = Box::new(OverlayImpl {
            apis,
            host_xid: host_xid as c_ulong,
            restore_xid: restore_xid as c_ulong,
            controls_url,
            mpv_handle,
            mpv_cmd_async,
            mpv_cmd_sync,
            shared,
            on_event,
            gtk_window: null_mut(),
            webview: null_mut(),
            host_gdk: null_mut(),
            overlay_xid: 0,
            composite_timer: 0,
            snap_cancel: null_mut(),
            snap_surf: null_mut(),
            snap_surf_prev: null_mut(),
            cursor_visible: null_mut(),
            cursor_hidden: null_mut(),
            snap_gen: AtomicU32::new(1),
            snap_in_flight: AtomicBool::new(false),
            snap_wait_ticks: AtomicU32::new(0),
            snap_cooldown: AtomicU32::new(0),
            snap_still: AtomicU32::new(0),
            snap_skip: AtomicU32::new(0),
            overlay_active: AtomicBool::new(true),
            overlay_pushed: AtomicBool::new(false),
            fade_ticks: AtomicU32::new(0),
            toast_ticks: AtomicU32::new(0),
            skip_shown: AtomicBool::new(false),
            next_ep_shown: AtomicBool::new(false),
            overlay_dirty_reported: AtomicBool::new(false),
            overlay_async_at: AtomicI64::new(0),
            pending_controls_json: Mutex::new(initial_controls_json),
            last_update_js: Mutex::new(String::new()),
            pending_js_queue: Mutex::new(Vec::new()),
            last_root_x: i32::MIN,
            last_root_y: i32::MIN,
            last_ptr_x: -1,
            last_ptr_y: -1,
        });

        let raw_ptr = Box::into_raw(boxed) as usize;
        live_overlays().lock().expect("live lock").insert(raw_ptr);

        unsafe {
            (apis.g_idle_add)(idle_create_overlay, raw_ptr as *mut c_void);
        }

        Ok(Self { raw_ptr })
    }

    pub fn update_controls_json(&self, json: String) {
        if let Ok(apis) = gtk_apis()
            && is_alive(self.raw_ptr)
        {
            let ov = unsafe { &*(self.raw_ptr as *const OverlayImpl) };
            *ov.pending_controls_json.lock().expect("ctrl json") = Some(json);
            unsafe {
                (apis.g_idle_add)(idle_apply_controls, self.raw_ptr as *mut c_void);
            }
        }
    }

    pub fn push_update_js(&self, update_js: String) {
        if let Ok(apis) = gtk_apis()
            && is_alive(self.raw_ptr)
        {
            let ov = unsafe { &*(self.raw_ptr as *const OverlayImpl) };
            let mut last = ov.last_update_js.lock().expect("last js");
            if *last == update_js {
                return;
            }
            *last = update_js.clone();
            drop(last);
            ov.pending_js_queue.lock().expect("js queue").push(update_js);
            unsafe {
                (apis.g_idle_add)(idle_flush_js, self.raw_ptr as *mut c_void);
            }
        }
    }

    pub fn focus(&self) {
        if let Ok(apis) = gtk_apis()
            && is_alive(self.raw_ptr)
        {
            unsafe {
                (apis.g_idle_add)(idle_focus_overlay, self.raw_ptr as *mut c_void);
            }
        }
    }

    pub fn destroy(self) {
        if let Ok(apis) = gtk_apis() {
            live_overlays()
                .lock()
                .expect("live lock")
                .remove(&self.raw_ptr);
            let (tx, rx) = std::sync::mpsc::sync_channel::<()>(1);
            let pkg = Box::into_raw(Box::new((self.raw_ptr, Some(tx))));
            unsafe {
                (apis.g_idle_add)(idle_destroy_overlay, pkg.cast());
            }
            let _ = rx.recv_timeout(std::time::Duration::from_millis(1500));
        }
    }
}

pub fn schedule_x11_child_cleanup(display: *mut c_void, child_xid: u64) {
    if display.is_null() || child_xid == 0 {
        return;
    }
    if let Ok(apis) = gtk_apis() {
        struct PendingX11Cleanup {
            apis: &'static GtkApis,
            display: *mut c_void,
            child_xid: c_ulong,
        }
        unsafe extern "C" fn finish_x11_cleanup(data: *mut c_void) -> c_int {
            let pkg = unsafe { Box::from_raw(data as *mut PendingX11Cleanup) };
            unsafe {
                let gdpy = (pkg.apis.gdk_display_get_default)();
                if !gdpy.is_null() {
                    (pkg.apis.gdk_x11_display_error_trap_push)(gdpy);
                }
                (pkg.apis.x_destroy_window)(pkg.display, pkg.child_xid);
                (pkg.apis.x_sync)(pkg.display, 0);
                (pkg.apis.x_close_display)(pkg.display);
                if !gdpy.is_null() {
                    (pkg.apis.gdk_display_sync)(gdpy);
                    (pkg.apis.gdk_x11_display_error_trap_pop_ignored)(gdpy);
                }
            }
            0
        }
        let pkg = Box::into_raw(Box::new(PendingX11Cleanup {
            apis,
            display,
            child_xid: child_xid as c_ulong,
        }));
        unsafe {
            (apis.g_timeout_add)(320, finish_x11_cleanup, pkg.cast());
        }
    }
}

unsafe fn eval_on_gtk(ov: &OverlayImpl, script: &str) {
    if ov.webview.is_null() {
        return;
    }
    if let Ok(c_str) = CString::new(script) {
        unsafe {
            (ov.apis.webkit_web_view_evaluate_javascript)(
                ov.webview,
                c_str.as_ptr(),
                -1,
                null(),
                null(),
                null_mut(),
                null(),
                null_mut(),
            );
        }
    }
}

unsafe fn wake_snapshots(ov: &OverlayImpl) {
    ov.snap_still.store(0, Ordering::Relaxed);
    ov.snap_skip.store(0, Ordering::Relaxed);
}

unsafe fn release_snapshots(ov: &mut OverlayImpl) {
    if !ov.snap_surf.is_null() {
        unsafe { (ov.apis.cairo_surface_destroy)(ov.snap_surf) };
        ov.snap_surf = null_mut();
    }
    if !ov.snap_surf_prev.is_null() {
        unsafe { (ov.apis.cairo_surface_destroy)(ov.snap_surf_prev) };
        ov.snap_surf_prev = null_mut();
    }
}

unsafe fn set_cursor_hidden(ov: &OverlayImpl, hidden: bool) {
    if ov.gtk_window.is_null() {
        return;
    }
    let gw = unsafe { (ov.apis.gtk_widget_get_window)(ov.gtk_window) };
    if gw.is_null() {
        return;
    }
    let cursor = if hidden {
        ov.cursor_hidden
    } else {
        ov.cursor_visible
    };
    if !cursor.is_null() {
        unsafe { (ov.apis.gdk_window_set_cursor)(gw, cursor) };
    }
}

unsafe fn json_flag(json: &str, key: &str, fallback: bool) -> bool {
    let needle = format!("\"{key}\":");
    let Some(pos) = json.find(&needle) else {
        return fallback;
    };
    let rest = json[pos + needle.len()..].trim_start();
    rest.starts_with("true")
}

unsafe extern "C" fn idle_create_overlay(data: *mut c_void) -> c_int {
    let raw = data as usize;
    if !is_alive(raw) {
        return 0;
    }
    let ov = unsafe { &mut *(raw as *mut OverlayImpl) };
    let apis = ov.apis;

    unsafe {
        let win = (apis.gtk_window_new)(0);
        (apis.gtk_window_set_decorated)(win, 0);
        (apis.gtk_window_set_accept_focus)(win, 1);
        (apis.gtk_widget_set_app_paintable)(win, 1);
        let screen = (apis.gtk_widget_get_screen)(win);
        let rgba_vis = (apis.gdk_screen_get_rgba_visual)(screen);
        if !rgba_vis.is_null() {
            (apis.gtk_widget_set_visual)(win, rgba_vis);
        }

        let wk_ctx = (apis.webkit_web_context_new_ephemeral)();
        let wv = (apis.webkit_web_view_new_with_context)(wk_ctx);
        if !wk_ctx.is_null() {
            (apis.g_object_unref)(wk_ctx);
        }
        let ucm = (apis.webkit_web_view_get_user_content_manager)(wv);
        let h_player = b"player\0";
        let h_dirty = b"overlayDirty\0";
        (apis.webkit_user_content_manager_register_script_message_handler)(
            ucm,
            h_player.as_ptr().cast(),
        );
        (apis.webkit_user_content_manager_register_script_message_handler)(
            ucm,
            h_dirty.as_ptr().cast(),
        );

        let sig_player = b"script-message-received::player\0";
        let sig_dirty = b"script-message-received::overlayDirty\0";
        (apis.g_signal_connect_data)(
            ucm,
            sig_player.as_ptr().cast(),
            on_player_message as *const c_void,
            data,
            null(),
            0,
        );
        (apis.g_signal_connect_data)(
            ucm,
            sig_dirty.as_ptr().cast(),
            on_overlay_dirty as *const c_void,
            data,
            null(),
            0,
        );
        (apis.gtk_widget_set_can_focus)(wv, 1);
        let transparent = GdkRgba {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 0.0,
        };
        (apis.webkit_web_view_set_background_color)(wv, &transparent);

        let settings = (apis.webkit_web_view_get_settings)(wv);
        if !settings.is_null() {
            (apis.webkit_settings_set_allow_file_access_from_file_urls)(settings, 1);
            (apis.webkit_settings_set_allow_universal_access_from_file_urls)(settings, 1);
            (apis.webkit_settings_set_javascript_can_access_clipboard)(settings, 1);
            // Force software path strictly on this small overlay WebView so snapshot alpha stays
            // exact without disabling GPU DMABUF acceleration in the main catalog window.
            (apis.webkit_settings_set_hardware_acceleration_policy)(settings, 2);
        }

        let sig_load = b"load-changed\0";
        let sig_ctx = b"context-menu\0";
        let sig_btn = b"button-press-event\0";
        let sig_destroy = b"destroy\0";
        (apis.g_signal_connect_data)(
            wv,
            sig_load.as_ptr().cast(),
            on_load_changed as *const c_void,
            data,
            null(),
            0,
        );
        (apis.g_signal_connect_data)(
            wv,
            sig_ctx.as_ptr().cast(),
            on_context_menu as *const c_void,
            null_mut(),
            null(),
            0,
        );
        (apis.g_signal_connect_data)(
            wv,
            sig_btn.as_ptr().cast(),
            on_button_press as *const c_void,
            data,
            null(),
            0,
        );
        (apis.g_signal_connect_data)(
            win,
            sig_btn.as_ptr().cast(),
            on_button_press as *const c_void,
            data,
            null(),
            0,
        );
        (apis.g_signal_connect_data)(
            win,
            sig_destroy.as_ptr().cast(),
            on_overlay_destroyed as *const c_void,
            data,
            null(),
            0,
        );

        (apis.gtk_container_add)(win, wv);
        (apis.gtk_widget_add_events)(win, GDK_EVENT_MASK_OVERLAY);
        (apis.gtk_widget_add_events)(wv, GDK_EVENT_MASK_OVERLAY);
        (apis.gtk_widget_realize)(win);

        let gdk_win = (apis.gtk_widget_get_window)(win);
        if gdk_win.is_null() {
            return 0;
        }
        let dpy = (apis.gdk_x11_display_get_xdisplay)((apis.gdk_window_get_display)(gdk_win));
        let gtk_xid = (apis.gdk_x11_window_get_xid)(gdk_win);
        (apis.gdk_x11_window_set_frame_sync_enabled)(gdk_win, 0);

        let mut attrs = std::mem::zeroed::<XWindowAttributes>();
        if (apis.x_get_window_attributes)(dpy, ov.host_xid, &mut attrs) != 0
            && attrs.width > 0
            && attrs.height > 0
        {
            let scale = (apis.gdk_window_get_scale_factor)(gdk_win).max(1);
            (apis.gtk_window_resize)(win, attrs.width / scale, attrs.height / scale);
        }

        let gdk_display = (apis.gdk_window_get_display)(gdk_win);
        let host_gdk = (apis.gdk_x11_window_foreign_new_for_display)(gdk_display, ov.host_xid);
        if !host_gdk.is_null() {
            (apis.gdk_window_reparent)(gdk_win, host_gdk, 0, 0);
            ov.host_gdk = host_gdk;
        }
        (apis.gtk_widget_show_all)(win);
        (apis.gdk_window_raise)(gdk_win);

        let mut ce = 0;
        let mut cerr = 0;
        if (apis.x_composite_query_extension)(dpy, &mut ce, &mut cerr) != 0 {
            (apis.x_composite_redirect_window)(dpy, gtk_xid, 1);
        }
        (apis.gdk_window_set_event_compression)(gdk_win, 0);
        let wv_win = (apis.gtk_widget_get_window)(wv);
        if !wv_win.is_null() && wv_win != gdk_win {
            (apis.gdk_window_set_event_compression)(wv_win, 0);
        }

        let name_def = b"default\0";
        let name_none = b"none\0";
        ov.cursor_visible =
            (apis.gdk_cursor_new_from_name)(gdk_display, name_def.as_ptr().cast());
        ov.cursor_hidden = (apis.gdk_cursor_new_from_name)(gdk_display, name_none.as_ptr().cast());
        if !ov.cursor_visible.is_null() {
            (apis.gdk_window_set_cursor)(gdk_win, ov.cursor_visible);
        }

        (apis.x_flush)(dpy);
        if let Ok(c_url) = CString::new(ov.controls_url.as_str()) {
            (apis.webkit_web_view_load_uri)(wv, c_url.as_ptr());
        }

        ov.gtk_window = win;
        ov.webview = wv;
        ov.overlay_xid = gtk_xid;
        ov.composite_timer = (apis.g_timeout_add)(33, composite_tick, data);

        focus_overlay_gtk(ov);
    }
    0
}

unsafe extern "C" fn idle_apply_controls(data: *mut c_void) -> c_int {
    let raw = data as usize;
    if !is_alive(raw) {
        return 0;
    }
    let ov = unsafe { &*(raw as *const OverlayImpl) };
    let maybe_json = ov.pending_controls_json.lock().expect("ctrl").clone();
    let Some(json) = maybe_json else {
        return 0;
    };

    unsafe {
        let visible = json_flag(&json, "controlsVisible", true);
        set_cursor_hidden(ov, !visible);
        ov.overlay_active.store(visible, Ordering::Relaxed);
        if !visible {
            ov.fade_ticks.store(18, Ordering::Relaxed);
        }
        ov.skip_shown
            .store(json_flag(&json, "skipPromptVisible", false), Ordering::Relaxed);
        ov.next_ep_shown
            .store(json_flag(&json, "nextEpisodeVisible", false), Ordering::Relaxed);
        wake_snapshots(ov);
        let script = format!("window.playerControls&&window.playerControls({json})");
        eval_on_gtk(ov, &script);
    }
    0
}

unsafe extern "C" fn idle_flush_js(data: *mut c_void) -> c_int {
    let raw = data as usize;
    if !is_alive(raw) {
        return 0;
    }
    let ov = unsafe { &*(raw as *const OverlayImpl) };
    let scripts: Vec<String> = {
        let mut q = ov.pending_js_queue.lock().expect("js queue");
        std::mem::take(&mut *q)
    };
    for s in scripts {
        unsafe { eval_on_gtk(ov, &s) };
    }
    0
}

unsafe fn focus_overlay_gtk(ov: &OverlayImpl) {
    if ov.overlay_xid == 0 || ov.gtk_window.is_null() {
        return;
    }
    unsafe {
        (ov.apis.gtk_window_set_accept_focus)(ov.gtk_window, 1);
        if !ov.webview.is_null() {
            (ov.apis.gtk_widget_set_can_focus)(ov.webview, 1);
            (ov.apis.gtk_widget_grab_focus)(ov.webview);
        }
        let gw = (ov.apis.gtk_widget_get_window)(ov.gtk_window);
        if !gw.is_null() {
            let gdk_display = (ov.apis.gdk_window_get_display)(gw);
            let dpy = (ov.apis.gdk_x11_display_get_xdisplay)(gdk_display);
            (ov.apis.gdk_x11_display_error_trap_push)(gdk_display);
            (ov.apis.x_set_input_focus)(dpy, ov.overlay_xid, 1, 0);
            (ov.apis.x_flush)(dpy);
            (ov.apis.gdk_x11_display_error_trap_pop_ignored)(gdk_display);
        }
    }
}

unsafe extern "C" fn idle_focus_overlay(data: *mut c_void) -> c_int {
    let raw = data as usize;
    if !is_alive(raw) {
        return 0;
    }
    let ov = unsafe { &*(raw as *const OverlayImpl) };
    unsafe { focus_overlay_gtk(ov) };
    0
}

unsafe extern "C" fn idle_destroy_overlay(data: *mut c_void) -> c_int {
    let mut pkg = unsafe { Box::from_raw(data as *mut (usize, Option<std::sync::mpsc::SyncSender<()>>)) };
    let raw = pkg.0;
    let mut ov = unsafe { Box::from_raw(raw as *mut OverlayImpl) };
    let apis = ov.apis;
    unsafe {
        if ov.composite_timer != 0 {
            let src = (apis.g_main_context_find_source_by_id)(null_mut(), ov.composite_timer);
            if !src.is_null() && (apis.g_source_is_destroyed)(src) == 0 {
                (apis.g_source_remove)(ov.composite_timer);
            }
            ov.composite_timer = 0;
        }
        if !ov.snap_cancel.is_null() {
            (apis.g_cancellable_cancel)(ov.snap_cancel);
            (apis.g_object_unref)(ov.snap_cancel);
            ov.snap_cancel = null_mut();
        }
        release_snapshots(&mut ov);
        if !ov.gtk_window.is_null() {
            let gw = (apis.gtk_widget_get_window)(ov.gtk_window);
            if !gw.is_null() && ov.overlay_xid != 0 {
                let gdpy = (apis.gdk_window_get_display)(gw);
                let dpy = (apis.gdk_x11_display_get_xdisplay)(gdpy);
                let mut cur: c_ulong = 0;
                let mut revert: c_int = 0;
                (apis.gdk_x11_display_error_trap_push)(gdpy);
                (apis.x_get_input_focus)(dpy, &mut cur, &mut revert);
                if cur == ov.overlay_xid || cur == ov.host_xid {
                    let target = if ov.restore_xid != 0 {
                        ov.restore_xid
                    } else {
                        ov.host_xid
                    };
                    (apis.x_set_input_focus)(dpy, target, 1, 0);
                    (apis.x_flush)(dpy);
                }
                (apis.gdk_x11_display_error_trap_pop_ignored)(gdpy);
            }

            let destroy_dpy = (apis.gtk_widget_get_display)(ov.gtk_window);
            if !destroy_dpy.is_null() {
                (apis.gdk_x11_display_error_trap_push)(destroy_dpy);
            }
            if !ov.webview.is_null() {
                (apis.webkit_web_view_terminate_web_process)(ov.webview);
            }
            (apis.gtk_widget_hide)(ov.gtk_window);
            (apis.gtk_widget_destroy)(ov.gtk_window);
            // Drop GDK's foreign wrapper of the host surface deterministically so GDK never
            // queries the pointer (XIQueryPointer) on an X window that is about to vanish.
            if !ov.host_gdk.is_null() {
                (apis.gdk_window_destroy)(ov.host_gdk);
                ov.host_gdk = null_mut();
            }
            if !destroy_dpy.is_null() {
                (apis.gdk_display_sync)(destroy_dpy);
                (apis.gdk_x11_display_error_trap_pop_ignored)(destroy_dpy);
            }
            ov.gtk_window = null_mut();
            ov.webview = null_mut();
            ov.overlay_xid = 0;
        }
        if !ov.cursor_visible.is_null() {
            (apis.g_object_unref)(ov.cursor_visible);
            ov.cursor_visible = null_mut();
        }
        if !ov.cursor_hidden.is_null() {
            (apis.g_object_unref)(ov.cursor_hidden);
            ov.cursor_hidden = null_mut();
        }
    }
    if let Some(tx) = pkg.1.take() {
        let _ = tx.send(());
    }
    0
}

unsafe extern "C" fn on_load_changed(_wv: *mut c_void, event: c_int, data: *mut c_void) {
    // WEBKIT_LOAD_FINISHED == 3
    if event == 3 {
        let _ = unsafe { idle_apply_controls(data) };
    }
}

unsafe extern "C" fn on_context_menu(
    _wv: *mut c_void,
    _menu: *mut c_void,
    _ev: *mut c_void,
    _hit: *mut c_void,
    _data: *mut c_void,
) -> c_int {
    1
}

unsafe extern "C" fn on_button_press(
    _widget: *mut c_void,
    ev: *const GdkEventButton,
    data: *mut c_void,
) -> c_int {
    let raw = data as usize;
    if !is_alive(raw) || ev.is_null() {
        return 0;
    }
    let ov = unsafe { &*(raw as *const OverlayImpl) };
    let btn = unsafe { (*ev).button };
    if btn == 1 {
        unsafe { focus_overlay_gtk(ov) };
    }
    let action = match btn {
        8 => "seekBack",
        9 => "seekForward",
        _ => return 0,
    };
    (ov.on_event)(action, 0.0);
    1
}

unsafe extern "C" fn on_overlay_destroyed(_w: *mut c_void, data: *mut c_void) {
    let raw = data as usize;
    if !is_alive(raw) {
        return;
    }
    let ov = unsafe { &mut *(raw as *mut OverlayImpl) };
    ov.gtk_window = null_mut();
    ov.webview = null_mut();
    ov.overlay_xid = 0;
}

unsafe extern "C" fn on_overlay_dirty(_ucm: *mut c_void, _js: *mut c_void, data: *mut c_void) {
    let raw = data as usize;
    if !is_alive(raw) {
        return;
    }
    let ov = unsafe { &*(raw as *const OverlayImpl) };
    ov.overlay_dirty_reported.store(true, Ordering::Relaxed);
    unsafe { wake_snapshots(ov) };
}

unsafe extern "C" fn on_player_message(_ucm: *mut c_void, js: *mut c_void, data: *mut c_void) {
    let raw = data as usize;
    if !is_alive(raw) || js.is_null() {
        return;
    }
    let ov = unsafe { &*(raw as *const OverlayImpl) };
    let apis = ov.apis;
    unsafe {
        let msg = (apis.webkit_javascript_result_get_js_value)(js);
        if msg.is_null() || (apis.jsc_value_is_object)(msg) == 0 {
            return;
        }
        let k_type = b"type\0";
        let k_val = b"value\0";
        let type_v = (apis.jsc_value_object_get_property)(msg, k_type.as_ptr().cast());
        let val_v = (apis.jsc_value_object_get_property)(msg, k_val.as_ptr().cast());
        let c_type = if !type_v.is_null() {
            (apis.jsc_value_to_string)(type_v)
        } else {
            null_mut()
        };
        let value = if !val_v.is_null() && (apis.jsc_value_is_number)(val_v) != 0 {
            (apis.jsc_value_to_double)(val_v)
        } else {
            0.0
        };

        if !c_type.is_null() {
            let ty = CStr::from_ptr(c_type).to_string_lossy().into_owned();
            wake_snapshots(ov);
            ov.last_update_js.lock().expect("last js").clear();

            if ty == "hideChrome" {
                ov.overlay_active.store(false, Ordering::Relaxed);
                ov.fade_ticks.store(18, Ordering::Relaxed);
                set_cursor_hidden(ov, true);
            } else if ty.starts_with("keyboard") {
                ov.toast_ticks.store(55, Ordering::Relaxed);
            } else {
                ov.overlay_active.store(true, Ordering::Relaxed);
                ov.fade_ticks.store(0, Ordering::Relaxed);
                set_cursor_hidden(ov, false);
            }

            if ty == "controlsReady" {
                let _ = idle_apply_controls(data);
            } else if ty == "focusOverlay" {
                focus_overlay_gtk(ov);
            }
            (ov.on_event)(&ty, value);
            (apis.g_free)(c_type.cast());
        }
        if !type_v.is_null() {
            (apis.g_object_unref)(type_v);
        }
        if !val_v.is_null() {
            (apis.g_object_unref)(val_v);
        }
    }
}

struct SnapCtx {
    raw: usize,
    snap_gen: u32,
}

unsafe extern "C" fn on_overlay_snapshot(
    src: *mut c_void,
    res: *mut c_void,
    user_data: *mut c_void,
) {
    let ctx = unsafe { Box::from_raw(user_data as *mut SnapCtx) };
    let mut err: *mut c_void = null_mut();
    let Ok(apis) = gtk_apis() else {
        return;
    };
    let surf = unsafe { (apis.webkit_web_view_get_snapshot_finish)(src, res, &mut err) };
    if !err.is_null() {
        unsafe { (apis.g_error_free)(err) };
    }
    if !is_alive(ctx.raw) {
        if !surf.is_null() {
            unsafe { (apis.cairo_surface_destroy)(surf) };
        }
        return;
    }
    let ov = unsafe { &mut *(ctx.raw as *mut OverlayImpl) };
    if ctx.snap_gen != ov.snap_gen.load(Ordering::Relaxed)
        || !ov.shared.mpv_alive.load(Ordering::Relaxed)
    {
        if !surf.is_null() {
            unsafe { (apis.cairo_surface_destroy)(surf) };
        }
        return;
    }
    ov.snap_in_flight.store(false, Ordering::Relaxed);
    if surf.is_null() {
        return;
    }

    unsafe {
        if (apis.cairo_image_surface_get_format)(surf) != CAIRO_FORMAT_ARGB32 {
            (apis.cairo_surface_destroy)(surf);
            return;
        }
        let w = (apis.cairo_image_surface_get_width)(surf);
        let h = (apis.cairo_image_surface_get_height)(surf);
        let stride = (apis.cairo_image_surface_get_stride)(surf);
        if w <= 0 || h <= 0 || stride <= 0 {
            (apis.cairo_surface_destroy)(surf);
            return;
        }
        (apis.cairo_surface_flush)(surf);

        if !ov.snap_surf.is_null()
            && ov.overlay_pushed.load(Ordering::Relaxed)
            && (apis.cairo_image_surface_get_width)(ov.snap_surf) == w
            && (apis.cairo_image_surface_get_height)(ov.snap_surf) == h
            && (apis.cairo_image_surface_get_stride)(ov.snap_surf) == stride
        {
            let byte_len = (stride as usize) * (h as usize);
            let old_slice = std::slice::from_raw_parts(
                (apis.cairo_image_surface_get_data)(ov.snap_surf),
                byte_len,
            );
            let new_slice =
                std::slice::from_raw_parts((apis.cairo_image_surface_get_data)(surf), byte_len);
            if old_slice == new_slice {
                (apis.cairo_surface_destroy)(surf);
                let still = ov.snap_still.fetch_add(1, Ordering::Relaxed).saturating_add(1);
                if still >= 3 {
                    let cap = if ov.overlay_dirty_reported.load(Ordering::Relaxed) {
                        30
                    } else {
                        15
                    };
                    ov.snap_skip
                        .store((still * 2).min(cap), Ordering::Relaxed);
                }
                return;
            }
        }

        ov.snap_still.store(0, Ordering::Relaxed);
        ov.snap_skip.store(0, Ordering::Relaxed);

        let issued = ov.shared.seq_issued.load(Ordering::Relaxed);
        let done = ov.shared.seq_done.load(Ordering::Relaxed);
        if issued > done
            && (apis.g_get_monotonic_time)() - ov.overlay_async_at.load(Ordering::Relaxed) < 500_000
        {
            (apis.cairo_surface_destroy)(surf);
            return;
        }

        let ptr_addr = (apis.cairo_image_surface_get_data)(surf) as usize;
        let c_ov = CString::new("overlay-add").unwrap();
        let c_zero = CString::new("0").unwrap();
        let c_addr = CString::new(format!("&{ptr_addr}")).unwrap();
        let c_fmt = CString::new("bgra").unwrap();
        let c_w = CString::new(w.to_string()).unwrap();
        let c_h = CString::new(h.to_string()).unwrap();
        let c_s = CString::new(stride.to_string()).unwrap();
        let cmd: [*const c_char; 11] = [
            c_ov.as_ptr(),
            c_zero.as_ptr(),
            c_zero.as_ptr(),
            c_zero.as_ptr(),
            c_addr.as_ptr(),
            c_zero.as_ptr(),
            c_fmt.as_ptr(),
            c_w.as_ptr(),
            c_h.as_ptr(),
            c_s.as_ptr(),
            null(),
        ];

        let next_seq = ov.shared.seq_issued.fetch_add(1, Ordering::Relaxed) + 1;
        ov.overlay_async_at
            .store((apis.g_get_monotonic_time)(), Ordering::Relaxed);
        let tag = (OVERLAY_REPLY_TAG << 32) | (next_seq & 0xffff_ffff);
        if (ov.mpv_cmd_async)(ov.mpv_handle, tag, cmd.as_ptr()) < 0 {
            ov.shared.mark_overlay_done(next_seq);
        }
        ov.overlay_pushed.store(true, Ordering::Relaxed);
        if !ov.snap_surf_prev.is_null() {
            (apis.cairo_surface_destroy)(ov.snap_surf_prev);
        }
        ov.snap_surf_prev = ov.snap_surf;
        ov.snap_surf = surf;
    }
}

unsafe extern "C" fn composite_tick(data: *mut c_void) -> c_int {
    let raw = data as usize;
    if !is_alive(raw) {
        return 0;
    }
    let ov = unsafe { &mut *(raw as *mut OverlayImpl) };
    let apis = ov.apis;
    if ov.overlay_xid == 0
        || ov.gtk_window.is_null()
        || ov.mpv_handle.is_null()
        || !ov.shared.mpv_alive.load(Ordering::Relaxed)
    {
        return 1;
    }

    unsafe {
        let gw = (apis.gtk_widget_get_window)(ov.gtk_window);
        if gw.is_null() {
            return 1;
        }
        let gdpy = (apis.gdk_window_get_display)(gw);
        let dpy = (apis.gdk_x11_display_get_xdisplay)(gdpy);
        (apis.gdk_x11_display_error_trap_push)(gdpy);

        // Keep overlay topmost in X stacking order for input without spamming XRaiseWindow.
        let mut r0 = 0;
        let mut p0 = 0;
        let mut kids: *mut c_ulong = null_mut();
        let mut nkids = 0;
        let mut on_top = false;
        if (apis.x_query_tree)(dpy, ov.host_xid, &mut r0, &mut p0, &mut kids, &mut nkids) != 0
            && !kids.is_null()
        {
            if nkids > 0 {
                let slice = std::slice::from_raw_parts(kids, nkids as usize);
                on_top = slice.last().copied() == Some(ov.overlay_xid);
            }
            (apis.x_free)(kids.cast());
        }
        if !on_top {
            (apis.x_raise_window)(dpy, ov.overlay_xid);
        }

        // Sync size AND screen position (handles ancestor window move on XWayland/X11).
        let mut host_wa = std::mem::zeroed::<XWindowAttributes>();
        let mut ov_wa = std::mem::zeroed::<XWindowAttributes>();
        let got_attrs = (apis.x_get_window_attributes)(dpy, ov.host_xid, &mut host_wa) != 0
            && (apis.x_get_window_attributes)(dpy, ov.overlay_xid, &mut ov_wa) != 0;

        let root_win = (apis.x_default_root_window)(dpy);
        let mut root_x = ov.last_root_x;
        let mut root_y = ov.last_root_y;
        let mut child_ret: c_ulong = 0;
        let got_coords = (apis.x_translate_coordinates)(
            dpy,
            ov.host_xid,
            root_win,
            0,
            0,
            &mut root_x,
            &mut root_y,
            &mut child_ret,
        ) != 0;

        // X11 XQueryPointer wake: ensures mouse movement over the player always wakes hidden
        // controls even if XWayland stops delivering MotionNotify after a window move.
        let mut q_root: c_ulong = 0;
        let mut q_child: c_ulong = 0;
        let mut q_rx: c_int = 0;
        let mut q_ry: c_int = 0;
        let mut q_wx: c_int = -1;
        let mut q_wy: c_int = -1;
        let mut q_mask: c_uint = 0;
        let got_ptr = (apis.x_query_pointer)(
            dpy,
            ov.host_xid,
            &mut q_root,
            &mut q_child,
            &mut q_rx,
            &mut q_ry,
            &mut q_wx,
            &mut q_wy,
            &mut q_mask,
        ) != 0;
        (apis.gdk_x11_display_error_trap_pop_ignored)(gdpy);

        let host_moved = got_coords && (root_x != ov.last_root_x || root_y != ov.last_root_y);
        if host_moved {
            ov.last_root_x = root_x;
            ov.last_root_y = root_y;
        }

        if got_attrs
            && host_wa.width > 0
            && host_wa.height > 0
            && (ov_wa.width != host_wa.width || ov_wa.height != host_wa.height || host_moved)
        {
            let scale = (apis.gdk_window_get_scale_factor)(gw).max(1);
            let lw = host_wa.width / scale;
            let lh = host_wa.height / scale;
            (apis.gdk_window_move_resize)(gw, 0, 0, lw, lh);
            (apis.gtk_window_resize)(ov.gtk_window, lw, lh);
            let mut alloc = GdkRectangle {
                x: 0,
                y: 0,
                width: lw,
                height: lh,
            };
            (apis.gtk_widget_size_allocate)(ov.gtk_window, &mut alloc);
            if ov_wa.width != host_wa.width || ov_wa.height != host_wa.height {
                if ov.overlay_pushed.load(Ordering::Relaxed) {
                    let c_rm = CString::new("overlay-remove").unwrap();
                    let c_z = CString::new("0").unwrap();
                    let cmd: [*const c_char; 3] = [c_rm.as_ptr(), c_z.as_ptr(), null()];
                    (ov.mpv_cmd_sync)(ov.mpv_handle, cmd.as_ptr());
                    ov.overlay_pushed.store(false, Ordering::Relaxed);
                }
                wake_snapshots(ov);
            }
        }

        if got_ptr
            && got_attrs
            && q_wx >= 0
            && q_wy >= 0
            && q_wx < host_wa.width
            && q_wy < host_wa.height
            && (q_wx != ov.last_ptr_x || q_wy != ov.last_ptr_y)
        {
            let had_prev = ov.last_ptr_x >= 0;
            ov.last_ptr_x = q_wx;
            ov.last_ptr_y = q_wy;
            if had_prev && !ov.overlay_active.load(Ordering::Relaxed) {
                ov.overlay_active.store(true, Ordering::Relaxed);
                ov.fade_ticks.store(0, Ordering::Relaxed);
                set_cursor_hidden(ov, false);
                wake_snapshots(ov);
                (ov.on_event)("cursorActivity", 0.0);
            }
        }

        let active = ov.shared.loading.load(Ordering::Relaxed)
            || ov.shared.paused.load(Ordering::Relaxed)
            || ov.overlay_active.load(Ordering::Relaxed)
            || ov.fade_ticks.load(Ordering::Relaxed) > 0
            || ov.toast_ticks.load(Ordering::Relaxed) > 0
            || ov.skip_shown.load(Ordering::Relaxed)
            || ov.next_ep_shown.load(Ordering::Relaxed);

        if !active {
            if ov.overlay_pushed.load(Ordering::Relaxed) {
                let c_rm = CString::new("overlay-remove").unwrap();
                let c_z = CString::new("0").unwrap();
                let cmd: [*const c_char; 3] = [c_rm.as_ptr(), c_z.as_ptr(), null()];
                (ov.mpv_cmd_sync)(ov.mpv_handle, cmd.as_ptr());
                ov.overlay_pushed.store(false, Ordering::Relaxed);
            }
            return 1;
        }

        if !ov.snap_in_flight.load(Ordering::Relaxed) && !ov.webview.is_null() {
            let cooldown = ov.snap_cooldown.load(Ordering::Relaxed);
            let skip = ov.snap_skip.load(Ordering::Relaxed);
            if cooldown > 0 {
                ov.snap_cooldown.fetch_sub(1, Ordering::Relaxed);
            } else if skip > 0 {
                ov.snap_skip.fetch_sub(1, Ordering::Relaxed);
            } else {
                ov.snap_in_flight.store(true, Ordering::Relaxed);
                ov.snap_wait_ticks.store(0, Ordering::Relaxed);
                if ov.snap_cancel.is_null() {
                    ov.snap_cancel = (apis.g_cancellable_new)();
                }
                let ctx = Box::into_raw(Box::new(SnapCtx {
                    raw,
                    snap_gen: ov.snap_gen.load(Ordering::Relaxed),
                }));
                (apis.webkit_web_view_get_snapshot)(
                    ov.webview,
                    0,
                    1,
                    ov.snap_cancel,
                    on_overlay_snapshot,
                    ctx.cast(),
                );
            }
        } else if ov.snap_in_flight.load(Ordering::Relaxed)
            && ov.snap_wait_ticks.fetch_add(1, Ordering::Relaxed) > 30
        {
            ov.snap_gen.fetch_add(1, Ordering::Relaxed);
            if !ov.snap_cancel.is_null() {
                (apis.g_cancellable_cancel)(ov.snap_cancel);
                (apis.g_object_unref)(ov.snap_cancel);
                ov.snap_cancel = null_mut();
            }
            ov.snap_in_flight.store(false, Ordering::Relaxed);
            ov.snap_wait_ticks.store(0, Ordering::Relaxed);
            ov.snap_cooldown.store(15, Ordering::Relaxed);
        }

        let fc = (apis.gdk_window_get_frame_clock)(gw);
        if !fc.is_null() {
            (apis.gdk_frame_clock_request_phase)(fc, GDK_FRAME_CLOCK_PHASE_UPDATE);
        }

        let f = ov.fade_ticks.load(Ordering::Relaxed);
        if !ov.overlay_active.load(Ordering::Relaxed) && f > 0 {
            ov.fade_ticks.store(f - 1, Ordering::Relaxed);
        }
        let t = ov.toast_ticks.load(Ordering::Relaxed);
        if t > 0 {
            ov.toast_ticks.store(t - 1, Ordering::Relaxed);
        }
    }
    1
}

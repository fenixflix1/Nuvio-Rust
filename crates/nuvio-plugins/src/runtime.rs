use crate::{DomBridge, PluginError, Result};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use nuvio_storage::sha256_hex;
use reqwest::redirect::Policy;
use rquickjs::{CatchResultExt, Context, Function, Runtime};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::{Notify, Semaphore};

pub const MAX_CONCURRENT_PLUGINS: usize = 28;
const TMDB_CACHE_TTL: Duration = Duration::from_secs(900);
const PLUGIN_EXEC_DEADLINE: Duration = Duration::from_secs(25);

static PLUGIN_SEMAPHORE: LazyLock<Semaphore> =
    LazyLock::new(|| Semaphore::new(MAX_CONCURRENT_PLUGINS));

static PLUGIN_HTTP_RT: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(6)
        .enable_all()
        .build()
        .expect("Failed to initialize plugin HTTP runtime")
});

static CLIENT_FOLLOW: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to build pooled HTTP client")
});

static CLIENT_MANUAL: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(10))
        .redirect(Policy::none())
        .build()
        .expect("Failed to build manual-redirect HTTP client")
});

#[derive(Clone)]
struct CachedTmdbEntry {
    expires_at: Instant,
    payload_template: serde_json::Value,
}

struct AsyncInFlightSlot {
    result: Mutex<Option<String>>,
    notify: Notify,
}

static TMDB_RESPONSE_CACHE: LazyLock<Mutex<HashMap<String, CachedTmdbEntry>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static TMDB_INFLIGHT: LazyLock<Mutex<HashMap<String, Arc<AsyncInFlightSlot>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static IMDB_TO_TMDB_CACHE: LazyLock<Mutex<HashMap<String, Option<String>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static TMDB_UPSTREAM_CALLS: AtomicU64 = AtomicU64::new(0);
static TMDB_SHARED_HITS: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, Default, serde::Serialize)]
pub struct TmdbCacheStats {
    pub upstream_http_calls: u64,
    pub cache_or_coalesced_hits: u64,
}

pub fn tmdb_cache_stats() -> TmdbCacheStats {
    TmdbCacheStats {
        upstream_http_calls: TMDB_UPSTREAM_CALLS.load(Ordering::Relaxed),
        cache_or_coalesced_hits: TMDB_SHARED_HITS.load(Ordering::Relaxed),
    }
}

pub fn reset_tmdb_cache_stats() {
    TMDB_UPSTREAM_CALLS.store(0, Ordering::Relaxed);
    TMDB_SHARED_HITS.store(0, Ordering::Relaxed);
}

pub fn clear_tmdb_cache() {
    TMDB_RESPONSE_CACHE.lock().expect("tmdb cache").clear();
    reset_tmdb_cache_stats();
}

#[derive(Debug, Clone, Copy)]
pub struct PluginExecOptions {
    pub use_shared_tmdb_cache: bool,
}

impl Default for PluginExecOptions {
    fn default() -> Self {
        Self {
            use_shared_tmdb_cache: true,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginCallArgs {
    pub tmdb_id: String,
    pub media_type: String,
    pub season: Option<i32>,
    pub episode: Option<i32>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginStreamResult {
    pub name: Option<String>,
    pub title: Option<String>,
    pub url: String,
    pub quality: Option<String>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

enum HostEvent {
    FetchDone { id: u64, payload_json: String },
    TimerFired { id: u64 },
}

struct HostInner {
    scraper_id: String,
    settings_json: String,
    tmdb_api_key: String,
    call_args_json: String,
    captured_result: Option<String>,
    dom: DomBridge,
    exec_opts: PluginExecOptions,
    next_async_id: u64,
    in_flight_fetches: HashSet<u64>,
    active_timers: HashSet<u64>,
    event_tx: std::sync::mpsc::Sender<HostEvent>,
}

const JS_POLYFILL: &str = r##"
var global = globalThis;
var console = {
  log: function() { __native_log("info", Array.prototype.slice.call(arguments).join(" ")); },
  info: function() { __native_log("info", Array.prototype.slice.call(arguments).join(" ")); },
  warn: function() { __native_log("warn", Array.prototype.slice.call(arguments).join(" ")); },
  error: function() { __native_log("error", Array.prototype.slice.call(arguments).join(" ")); },
  debug: function() { __native_log("debug", Array.prototype.slice.call(arguments).join(" ")); }
};
var localStorage = (function() {
  var store = Object.create(null);
  return {
    getItem: function(k) { return Object.prototype.hasOwnProperty.call(store, String(k)) ? store[String(k)] : null; },
    setItem: function(k, v) { store[String(k)] = String(v); },
    removeItem: function(k) { delete store[String(k)]; },
    clear: function() { store = Object.create(null); }
  };
})();
function atob(str) { return __native_atob(String(str)); }
function btoa(str) { return __native_btoa(String(str)); }
function TextEncoder() {}
TextEncoder.prototype.encode = function(str) {
  var s = String(str == null ? "" : str);
  var out = [];
  for (var i = 0; i < s.length; i++) {
    var c = s.charCodeAt(i);
    if (c < 0x80) out.push(c);
    else if (c < 0x800) { out.push(0xc0 | (c >> 6), 0x80 | (c & 0x3f)); }
    else if (c < 0xd800 || c >= 0xe000) { out.push(0xe0 | (c >> 12), 0x80 | ((c >> 6) & 0x3f), 0x80 | (c & 0x3f)); }
    else {
      i++;
      c = 0x10000 + (((c & 0x3ff) << 10) | (s.charCodeAt(i) & 0x3ff));
      out.push(0xf0 | (c >> 18), 0x80 | ((c >> 12) & 0x3f), 0x80 | ((c >> 6) & 0x3f), 0x80 | (c & 0x3f));
    }
  }
  return new Uint8Array(out);
};
function TextDecoder() {}
TextDecoder.prototype.decode = function(buf) {
  if (!buf) return "";
  var bytes = new Uint8Array(buf.buffer || buf);
  var out = "";
  for (var i = 0; i < bytes.length; i++) out += String.fromCharCode(bytes[i]);
  try { return decodeURIComponent(escape(out)); } catch (_) { return out; }
};
function URLSearchParams(init) {
  this._pairs = [];
  if (typeof init === "string") {
    var q = init.charAt(0) === "?" ? init.slice(1) : init;
    if (q) {
      var parts = q.split("&");
      for (var i = 0; i < parts.length; i++) {
        if (!parts[i]) continue;
        var eq = parts[i].indexOf("=");
        var k = decodeURIComponent((eq >= 0 ? parts[i].slice(0, eq) : parts[i]).replace(/\+/g, " "));
        var v = decodeURIComponent((eq >= 0 ? parts[i].slice(eq + 1) : "").replace(/\+/g, " "));
        this._pairs.push([k, v]);
      }
    }
  } else if (init && typeof init === "object") {
    var keys = Object.keys(init);
    for (var j = 0; j < keys.length; j++) this._pairs.push([keys[j], String(init[keys[j]])]);
  }
}
URLSearchParams.prototype.get = function(k) {
  for (var i = 0; i < this._pairs.length; i++) if (this._pairs[i][0] === String(k)) return this._pairs[i][1];
  return null;
};
URLSearchParams.prototype.has = function(k) { return this.get(k) !== null; };
URLSearchParams.prototype.set = function(k, v) {
  k = String(k); v = String(v);
  var updated = false, next = [];
  for (var i = 0; i < this._pairs.length; i++) {
    if (this._pairs[i][0] === k) {
      if (!updated) { next.push([k, v]); updated = true; }
    } else next.push(this._pairs[i]);
  }
  if (!updated) next.push([k, v]);
  this._pairs = next;
};
URLSearchParams.prototype.append = function(k, v) { this._pairs.push([String(k), String(v)]); };
URLSearchParams.prototype.delete = function(k) {
  this._pairs = this._pairs.filter(function(p) { return p[0] !== String(k); });
};
URLSearchParams.prototype.toString = function() {
  return this._pairs.map(function(p) { return encodeURIComponent(p[0]) + "=" + encodeURIComponent(p[1]); }).join("&");
};
function URL(input, base) {
  var raw = String(input || "").trim();
  if (!/^[a-zA-Z][a-zA-Z0-9+.-]*:/.test(raw)) {
    if (!base) throw new TypeError("Invalid URL: " + raw);
    var b = new URL(base);
    if (raw.indexOf("//") === 0) raw = b.protocol + raw;
    else if (raw.charAt(0) === "/") raw = b.origin + raw;
    else if (raw.charAt(0) === "?") raw = b.origin + b.pathname + raw;
    else if (raw.charAt(0) === "#") raw = b.origin + b.pathname + b.search + raw;
    else {
      var dir = b.pathname.replace(/[^/]*$/, "");
      raw = b.origin + dir + raw;
    }
  }
  var m = raw.match(/^([a-zA-Z][a-zA-Z0-9+.-]*:)\/\/([^/?#]*)([^?#]*)(\?[^#]*)?(#.*)?$/);
  if (!m) throw new TypeError("Invalid URL: " + raw);
  this.protocol = m[1].toLowerCase();
  this.host = m[2];
  var colon = this.host.lastIndexOf(":");
  this.hostname = colon >= 0 ? this.host.slice(0, colon) : this.host;
  this.port = colon >= 0 ? this.host.slice(colon + 1) : "";
  this.origin = this.protocol + "//" + this.host;
  this.pathname = m[3] || "/";
  this.search = m[4] || "";
  this.hash = m[5] || "";
  this.searchParams = new URLSearchParams(this.search);
  this.href = this.origin + this.pathname + this.search + this.hash;
}
URL.prototype.toString = function() {
  var q = this.searchParams ? this.searchParams.toString() : "";
  this.search = q ? "?" + q : "";
  this.href = this.origin + this.pathname + this.search + this.hash;
  return this.href;
};
function Headers(init) {
  this._map = Object.create(null);
  if (init && typeof init === "object") {
    var src = init._map || init;
    var ks = Object.keys(src);
    for (var i = 0; i < ks.length; i++) {
      var lk = String(ks[i]).toLowerCase();
      var val = String(src[ks[i]]);
      this._map[lk] = val;
      this[lk] = val;
    }
  }
}
Headers.prototype.get = function(k) {
  var lk = String(k || "").toLowerCase();
  return Object.prototype.hasOwnProperty.call(this._map, lk) ? this._map[lk] : null;
};
Headers.prototype.set = function(k, v) {
  var lk = String(k || "").toLowerCase();
  this._map[lk] = String(v);
  this[lk] = String(v);
};
Headers.prototype.has = function(k) {
  return Object.prototype.hasOwnProperty.call(this._map, String(k || "").toLowerCase());
};
function AbortController() {
  var sig = { aborted: false, _listeners: [], addEventListener: function(t, fn) { if (t === "abort" && fn) sig._listeners.push(fn); }, removeEventListener: function(){} };
  this.signal = sig;
}
AbortController.prototype.abort = function() {
  if (this.signal.aborted) return;
  this.signal.aborted = true;
  if (this.signal._onAbort) this.signal._onAbort();
  var ls = this.signal._listeners;
  for (var i = 0; i < ls.length; i++) { try { ls[i](); } catch (_) {} }
};

var __fetchCallbacks = Object.create(null);
var __timerCallbacks = Object.create(null);
function __onFetchDone(id, raw) {
  var cb = __fetchCallbacks[id];
  if (!cb) return;
  delete __fetchCallbacks[id];
  var res = JSON.parse(raw);
  cb.resolve({
    ok: !!res.ok,
    status: res.status || 0,
    statusText: res.statusText || "",
    url: res.url || cb.url,
    headers: new Headers(res.headers || {}),
    text: function() { return Promise.resolve(res.body || ""); },
    json: function() { return Promise.resolve(JSON.parse(res.body || "null")); }
  });
}
function __onTimerDone(id) {
  var fn = __timerCallbacks[id];
  if (!fn) return;
  delete __timerCallbacks[id];
  fn();
}
function setTimeout(fn, ms) {
  var id = __native_timer(Math.max(0, Number(ms) || 0));
  __timerCallbacks[id] = fn;
  return id;
}
function clearTimeout(id) {
  var n = Number(id) || 0;
  delete __timerCallbacks[n];
  __native_clear_timer(n);
}
function fetch(url, opts) {
  opts = opts || {};
  if (opts.signal && opts.signal.aborted) {
    return Promise.reject(new Error("The operation was aborted"));
  }
  var method = opts.method || "GET";
  var hdrObj = (opts.headers && opts.headers._map) ? opts.headers._map : (opts.headers || {});
  var headers = JSON.stringify(hdrObj);
  var body = opts.body ? String(opts.body) : "";
  var redirect = opts.redirect === "manual" ? "manual" : "follow";
  return new Promise(function(resolve, reject) {
    var id = __native_fetch_async(String(url), String(method), headers, body, redirect);
    __fetchCallbacks[id] = { resolve: resolve, reject: reject, url: String(url) };
    if (opts.signal && typeof opts.signal === "object") {
      opts.signal._onAbort = function() {
        if (__fetchCallbacks[id]) {
          delete __fetchCallbacks[id];
          __native_cancel_fetch(id);
          reject(new Error("The operation was aborted"));
        }
      };
    }
  });
}
var cheerio = {
  load: function(html) {
    var docId = __cheerio_load(String(html || ""));
    function wrap(ids) {
      return {
        length: ids.length,
        text: function() { return ids.map(function(id) { return __cheerio_text(id); }).join(""); },
        html: function() { return ids.length ? __cheerio_html(ids[0]) : ""; },
        attr: function(name) {
          if (!ids.length) return undefined;
          var v = __cheerio_attr(ids[0], String(name));
          return v === "__UNDEFINED__" ? undefined : v;
        },
        first: function() { return wrap(ids.length ? [ids[0]] : []); },
        eq: function(i) { return wrap(i >= 0 && i < ids.length ? [ids[i]] : []); },
        next: function() {
          if (!ids.length) return wrap([]);
          var nid = __cheerio_next(docId, ids[0]);
          return wrap(nid === "__NONE__" ? [] : [nid]);
        },
        prev: function() {
          if (!ids.length) return wrap([]);
          var pid = __cheerio_prev(docId, ids[0]);
          return wrap(pid === "__NONE__" ? [] : [pid]);
        },
        each: function(cb) {
          for (var i = 0; i < ids.length; i++) { cb.call(wrap([ids[i]]), i, wrap([ids[i]])); }
          return this;
        },
        map: function(cb) {
          var out = [];
          for (var i = 0; i < ids.length; i++) { out.push(cb.call(wrap([ids[i]]), i, wrap([ids[i]]))); }
          return { get: function() { return out; } };
        }
      };
    }
    return function(selector) {
      if (typeof selector === "object" && selector !== null && typeof selector.text === "function") {
        return selector;
      }
      var raw = __cheerio_select(docId, String(selector || ""));
      var ids = JSON.parse(raw);
      return wrap(ids);
    };
  }
};
var module = { exports: {} };
var exports = module.exports;
"##;

/// High-speed QuickJS plugin runtime backed by `rquickjs` + async Tokio event loop.
pub struct PluginRuntime;

impl PluginRuntime {
    pub async fn execute_plugin(
        code: String,
        scraper_id: String,
        settings_json: String,
        tmdb_api_key: String,
        args: PluginCallArgs,
    ) -> Result<Vec<PluginStreamResult>> {
        Self::execute_plugin_with_options(
            code,
            scraper_id,
            settings_json,
            tmdb_api_key,
            args,
            PluginExecOptions::default(),
        )
        .await
    }

    pub async fn execute_plugin_with_options(
        code: String,
        scraper_id: String,
        settings_json: String,
        tmdb_api_key: String,
        args: PluginCallArgs,
        exec_opts: PluginExecOptions,
    ) -> Result<Vec<PluginStreamResult>> {
        let _permit = PLUGIN_SEMAPHORE
            .acquire()
            .await
            .map_err(|e| PluginError::Js(e.to_string()))?;

        tokio::task::spawn_blocking(move || {
            Self::execute_sync_with_options(
                &code,
                &scraper_id,
                &settings_json,
                &tmdb_api_key,
                &args,
                exec_opts,
            )
        })
        .await
        .map_err(|e| PluginError::Js(format!("Plugin worker join error: {e}")))?
    }

    pub fn execute_sync(
        code: &str,
        scraper_id: &str,
        settings_json: &str,
        tmdb_api_key: &str,
        args: &PluginCallArgs,
    ) -> Result<Vec<PluginStreamResult>> {
        Self::execute_sync_with_options(
            code,
            scraper_id,
            settings_json,
            tmdb_api_key,
            args,
            PluginExecOptions::default(),
        )
    }

    pub fn execute_sync_with_options(
        code: &str,
        scraper_id: &str,
        settings_json: &str,
        tmdb_api_key: &str,
        args: &PluginCallArgs,
        exec_opts: PluginExecOptions,
    ) -> Result<Vec<PluginStreamResult>> {
        let rt = Runtime::new().map_err(|e| PluginError::Js(e.to_string()))?;
        rt.set_memory_limit(128 * 1024 * 1024);
        rt.set_max_stack_size(2 * 1024 * 1024);

        let deadline = Instant::now() + PLUGIN_EXEC_DEADLINE;
        rt.set_interrupt_handler(Some(Box::new(move || Instant::now() >= deadline)));

        let ctx = Context::full(&rt).map_err(|e| PluginError::Js(e.to_string()))?;
        let (tx, rx) = std::sync::mpsc::channel::<HostEvent>();

        let host = Rc::new(RefCell::new(HostInner {
            scraper_id: scraper_id.to_string(),
            settings_json: settings_json.to_string(),
            tmdb_api_key: tmdb_api_key.to_string(),
            call_args_json: serde_json::to_string(args)?,
            captured_result: None,
            dom: DomBridge::new(),
            exec_opts,
            next_async_id: 1,
            in_flight_fetches: HashSet::new(),
            active_timers: HashSet::new(),
            event_tx: tx,
        }));

        ctx.with(|qctx| -> Result<()> {
            register_host_functions(&qctx, Rc::clone(&host))?;
            eval_qjs(&qctx, JS_POLYFILL)?;
            eval_qjs(&qctx, code)?;

            let call_script = r#"
(function() {
  var fn = (typeof getStreams === "function")
    ? getStreams
    : (module && module.exports && typeof module.exports.getStreams === "function")
      ? module.exports.getStreams
      : null;
  if (!fn) {
    __capture_result("[]");
    return;
  }
  var a = JSON.parse(__get_call_args());
  var res = fn(a.tmdbId, a.mediaType, a.season, a.episode);
  if (res && typeof res.then === "function") {
    res.then(function(list) { __capture_result(JSON.stringify(list || [])); })
       .catch(function() { __capture_result("[]"); });
  } else {
    __capture_result(JSON.stringify(res || []));
  }
})();
"#;
            eval_qjs(&qctx, call_script)?;
            Ok(())
        })?;

        drain_pending_jobs(&rt);

        loop {
            let (done, pending_fetches, pending_timers) = {
                let b = host.borrow();
                (
                    b.captured_result.is_some(),
                    b.in_flight_fetches.len(),
                    b.active_timers.len(),
                )
            };
            if done || (pending_fetches == 0 && pending_timers == 0) {
                break;
            }
            let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                break;
            };
            let ev = match rx.recv_timeout(remaining) {
                Ok(e) => e,
                Err(_) => break,
            };

            let mut batch = vec![ev];
            while let Ok(more) = rx.try_recv() {
                batch.push(more);
            }

            let _ = ctx.with(|qctx| -> Result<()> {
                for item in batch {
                    match item {
                        HostEvent::FetchDone { id, payload_json } => {
                            let active = host.borrow_mut().in_flight_fetches.remove(&id);
                            if active {
                                let literal = serde_json::to_string(&payload_json)?;
                                let script = format!("__onFetchDone({id}, {literal});");
                                let _ = eval_qjs(&qctx, &script);
                            }
                        }
                        HostEvent::TimerFired { id } => {
                            let active = host.borrow_mut().active_timers.remove(&id);
                            if active {
                                let script = format!("__onTimerDone({id});");
                                let _ = eval_qjs(&qctx, &script);
                            }
                        }
                    }
                }
                Ok(())
            });
            drain_pending_jobs(&rt);
        }

        let raw = host
            .borrow_mut()
            .captured_result
            .take()
            .unwrap_or_else(|| "[]".to_string());
        let streams: Vec<PluginStreamResult> = serde_json::from_str(&raw).unwrap_or_default();
        Ok(streams)
    }
}

fn drain_pending_jobs(rt: &Runtime) {
    while rt.is_job_pending() {
        match rt.execute_pending_job() {
            Ok(true) => continue,
            _ => break,
        }
    }
}

fn eval_qjs(ctx: &rquickjs::Ctx<'_>, code: &str) -> Result<()> {
    ctx.eval::<(), _>(code)
        .catch(ctx)
        .map_err(|e| PluginError::Js(format!("{e:?}")))
}

fn register_host_functions<'js>(
    ctx: &rquickjs::Ctx<'js>,
    host: Rc<RefCell<HostInner>>,
) -> Result<()> {
    let g = ctx.globals();
    let map_err = |e: rquickjs::Error| PluginError::Js(e.to_string());

    {
        let h = Rc::clone(&host);
        let f = Function::new(ctx.clone(), move || h.borrow().scraper_id.clone()).map_err(map_err)?;
        g.set("__get_scraper_id", f).map_err(map_err)?;
    }
    {
        let h = Rc::clone(&host);
        let f = Function::new(ctx.clone(), move || h.borrow().settings_json.clone()).map_err(map_err)?;
        g.set("__get_scraper_settings", f).map_err(map_err)?;
    }
    {
        let h = Rc::clone(&host);
        let f = Function::new(ctx.clone(), move || h.borrow().tmdb_api_key.clone()).map_err(map_err)?;
        g.set("__get_tmdb_api_key", f).map_err(map_err)?;
    }
    {
        let h = Rc::clone(&host);
        let f = Function::new(ctx.clone(), move || h.borrow().call_args_json.clone()).map_err(map_err)?;
        g.set("__get_call_args", f).map_err(map_err)?;
    }
    {
        let h = Rc::clone(&host);
        let f = Function::new(ctx.clone(), move |raw: String| {
            h.borrow_mut().captured_result = Some(raw);
        })
        .map_err(map_err)?;
        g.set("__capture_result", f).map_err(map_err)?;
    }
    {
        let f = Function::new(ctx.clone(), |level: String, msg: String| {
            tracing::debug!(target: "nuvio_plugin", level = %level, "{msg}");
        })
        .map_err(map_err)?;
        g.set("__native_log", f).map_err(map_err)?;
    }
    {
        let f = Function::new(ctx.clone(), |input: String| sha256_hex(&input)).map_err(map_err)?;
        g.set("__crypto_sha256_hex", f).map_err(map_err)?;
    }
    {
        let f = Function::new(ctx.clone(), |input: String| {
            let bytes = BASE64.decode(input.trim()).unwrap_or_default();
            String::from_utf8_lossy(&bytes).into_owned()
        })
        .map_err(map_err)?;
        g.set("__native_atob", f).map_err(map_err)?;
    }
    {
        let f = Function::new(ctx.clone(), |input: String| BASE64.encode(input.as_bytes()))
            .map_err(map_err)?;
        g.set("__native_btoa", f).map_err(map_err)?;
    }
    {
        let h = Rc::clone(&host);
        let f = Function::new(ctx.clone(), move |html: String| {
            h.borrow_mut().dom.load_document(&html)
        })
        .map_err(map_err)?;
        g.set("__cheerio_load", f).map_err(map_err)?;
    }
    {
        let h = Rc::clone(&host);
        let f = Function::new(ctx.clone(), move |doc_id: String, sel: String| {
            let ids = h.borrow_mut().dom.select(&doc_id, &sel);
            serde_json::to_string(&ids).unwrap_or_else(|_| "[]".to_string())
        })
        .map_err(map_err)?;
        g.set("__cheerio_select", f).map_err(map_err)?;
    }
    {
        let h = Rc::clone(&host);
        let f = Function::new(ctx.clone(), move |el_id: String| h.borrow().dom.text(&el_id))
            .map_err(map_err)?;
        g.set("__cheerio_text", f).map_err(map_err)?;
    }
    {
        let h = Rc::clone(&host);
        let f = Function::new(ctx.clone(), move |el_id: String| h.borrow().dom.html(&el_id))
            .map_err(map_err)?;
        g.set("__cheerio_html", f).map_err(map_err)?;
    }
    {
        let h = Rc::clone(&host);
        let f = Function::new(ctx.clone(), move |el_id: String, attr: String| {
            h.borrow()
                .dom
                .attr(&el_id, &attr)
                .unwrap_or_else(|| "__UNDEFINED__".to_string())
        })
        .map_err(map_err)?;
        g.set("__cheerio_attr", f).map_err(map_err)?;
    }
    {
        let h = Rc::clone(&host);
        let f = Function::new(ctx.clone(), move |doc_id: String, el_id: String| {
            h.borrow_mut()
                .dom
                .next_sibling(&doc_id, &el_id)
                .unwrap_or_else(|| "__NONE__".to_string())
        })
        .map_err(map_err)?;
        g.set("__cheerio_next", f).map_err(map_err)?;
    }
    {
        let h = Rc::clone(&host);
        let f = Function::new(ctx.clone(), move |doc_id: String, el_id: String| {
            h.borrow_mut()
                .dom
                .prev_sibling(&doc_id, &el_id)
                .unwrap_or_else(|| "__NONE__".to_string())
        })
        .map_err(map_err)?;
        g.set("__cheerio_prev", f).map_err(map_err)?;
    }
    {
        let h = Rc::clone(&host);
        let f = Function::new(
            ctx.clone(),
            move |url: String, method: String, headers_json: String, body: String, redirect: String| -> f64 {
                let (id, tx, tmdb_key, use_cache) = {
                    let mut b = h.borrow_mut();
                    let id = b.next_async_id;
                    b.next_async_id += 1;
                    b.in_flight_fetches.insert(id);
                    (
                        id,
                        b.event_tx.clone(),
                        b.tmdb_api_key.clone(),
                        b.exec_opts.use_shared_tmdb_cache,
                    )
                };
                PLUGIN_HTTP_RT.spawn(async move {
                    let payload_json = execute_http_async(
                        &url,
                        &method,
                        &headers_json,
                        &body,
                        &redirect,
                        &tmdb_key,
                        use_cache,
                    )
                    .await;
                    let _ = tx.send(HostEvent::FetchDone { id, payload_json });
                });
                id as f64
            },
        )
        .map_err(map_err)?;
        g.set("__native_fetch_async", f).map_err(map_err)?;
    }
    {
        let h = Rc::clone(&host);
        let f = Function::new(ctx.clone(), move |id: f64| {
            h.borrow_mut().in_flight_fetches.remove(&(id as u64));
        })
        .map_err(map_err)?;
        g.set("__native_cancel_fetch", f).map_err(map_err)?;
    }
    {
        let h = Rc::clone(&host);
        let f = Function::new(ctx.clone(), move |ms: f64| -> f64 {
            let ms_u = ms.max(0.0) as u64;
            let (id, tx) = {
                let mut b = h.borrow_mut();
                let id = b.next_async_id;
                b.next_async_id += 1;
                b.active_timers.insert(id);
                (id, b.event_tx.clone())
            };
            PLUGIN_HTTP_RT.spawn(async move {
                if ms_u > 0 {
                    tokio::time::sleep(Duration::from_millis(ms_u)).await;
                }
                let _ = tx.send(HostEvent::TimerFired { id });
            });
            id as f64
        })
        .map_err(map_err)?;
        g.set("__native_timer", f).map_err(map_err)?;
    }
    {
        let h = Rc::clone(&host);
        let f = Function::new(ctx.clone(), move |id: f64| {
            h.borrow_mut().active_timers.remove(&(id as u64));
        })
        .map_err(map_err)?;
        g.set("__native_clear_timer", f).map_err(map_err)?;
    }

    Ok(())
}

/// Resolves an IMDb ID (`tt123456`) to a TMDB numeric ID once and caches the mapping in RAM.
pub async fn resolve_imdb_to_tmdb(
    imdb_id: &str,
    media_type: &str,
    tmdb_api_key: &str,
) -> Option<String> {
    let base_imdb = imdb_id.split(':').next().unwrap_or(imdb_id).trim();
    if !base_imdb.starts_with("tt") || tmdb_api_key.trim().is_empty() {
        return None;
    }
    let cache_key = format!("{media_type}:{base_imdb}");
    if let Some(cached) = IMDB_TO_TMDB_CACHE.lock().expect("imdb map").get(&cache_key) {
        return cached.clone();
    }

    let url = format!(
        "https://api.themoviedb.org/3/find/{base_imdb}?external_source=imdb_id&api_key={}",
        tmdb_api_key.trim()
    );
    let resp = CLIENT_FOLLOW.get(&url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let val: serde_json::Value = resp.json().await.ok()?;
    let primary_field = if media_type == "movie" {
        "movie_results"
    } else {
        "tv_results"
    };
    let fallback_field = if media_type == "movie" {
        "tv_results"
    } else {
        "movie_results"
    };

    let found = val
        .get(primary_field)
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .or_else(|| {
            val.get(fallback_field)
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
        })
        .and_then(|item| item.get("id"))
        .and_then(|id| id.as_i64())
        .map(|n| n.to_string());

    IMDB_TO_TMDB_CACHE
        .lock()
        .expect("imdb map")
        .insert(cache_key, found.clone());
    found
}

/// Pre-warms TMDB metadata once in the shared single-flight cache before spawning concurrent plugins.
pub async fn prefetch_tmdb_for_plugins(
    tmdb_id: &str,
    media_type: &str,
    season: Option<i32>,
    tmdb_api_key: &str,
) {
    let clean_id = tmdb_id.trim();
    let key = tmdb_api_key.trim();
    if clean_id.is_empty() || key.is_empty() {
        return;
    }
    let endpoint = if media_type == "movie" { "movie" } else { "tv" };
    let mut urls = vec![
        format!("https://api.themoviedb.org/3/{endpoint}/{clean_id}?api_key={key}&language=pt-BR"),
        format!("https://api.themoviedb.org/3/{endpoint}/{clean_id}?api_key={key}"),
        format!("https://api.themoviedb.org/3/{endpoint}/{clean_id}/external_ids?api_key={key}"),
    ];
    if endpoint == "tv"
        && let Some(s) = season
    {
        urls.push(format!(
            "https://api.themoviedb.org/3/tv/{clean_id}/season/{s}?api_key={key}&language=pt-BR"
        ));
    }

    let mut handles = Vec::with_capacity(urls.len());
    for url in urls {
        let k = key.to_string();
        handles.push(PLUGIN_HTTP_RT.spawn(async move {
            let _ = execute_http_async(&url, "GET", "{}", "", "follow", &k, true).await;
        }));
    }
    for h in handles {
        let _ = h.await;
    }
}

fn canonical_tmdb_cache_key(raw_url: &str, fallback_api_key: &str) -> Option<(String, String)> {
    let mut parsed = url::Url::parse(raw_url).ok()?;
    if parsed.scheme() != "https" && parsed.scheme() != "http" {
        return None;
    }
    if parsed.host_str()? != "api.themoviedb.org" {
        return None;
    }

    let mut has_non_empty_api_key = false;
    let mut non_key_params: Vec<(String, String)> = Vec::new();
    for (k, v) in parsed.query_pairs() {
        if k == "api_key" {
            if !v.trim().is_empty() {
                has_non_empty_api_key = true;
            }
        } else {
            non_key_params.push((k.into_owned(), v.into_owned()));
        }
    }
    non_key_params.sort();

    if !has_non_empty_api_key && !fallback_api_key.trim().is_empty() {
        parsed
            .query_pairs_mut()
            .append_pair("api_key", fallback_api_key.trim());
    }

    let query_sig: String = non_key_params
        .into_iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("&");

    let key = format!("{}?{}", parsed.path(), query_sig);
    Some((key, parsed.to_string()))
}

async fn perform_raw_http_async(
    url: &str,
    method: &str,
    headers_json: &str,
    body: &str,
    redirect: &str,
) -> std::result::Result<serde_json::Value, String> {
    let client = if redirect == "manual" {
        &*CLIENT_MANUAL
    } else {
        &*CLIENT_FOLLOW
    };
    let parsed_method =
        reqwest::Method::from_bytes(method.trim().to_ascii_uppercase().as_bytes())
            .unwrap_or(reqwest::Method::GET);
    let mut req = client.request(parsed_method, url);
    if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(headers_json) {
        for (k, v) in map {
            req = req.header(k, v);
        }
    }
    if !body.is_empty() {
        req = req.body(body.to_string());
    }
    let resp = req.send().await.map_err(|e| e.to_string())?;
    let status = resp.status().as_u16();
    let ok = resp.status().is_success();
    let final_url = resp.url().to_string();
    let mut resp_headers = HashMap::new();
    for (k, v) in resp.headers() {
        if let Ok(vs) = v.to_str() {
            resp_headers.insert(k.as_str().to_ascii_lowercase(), vs.to_string());
        }
    }
    let text = resp.text().await.map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "ok": ok,
        "status": status,
        "statusText": "",
        "url": final_url,
        "headers": resp_headers,
        "body": text,
    }))
}

async fn execute_http_async(
    url: &str,
    method: &str,
    headers_json: &str,
    body: &str,
    redirect: &str,
    fallback_tmdb_key: &str,
    use_shared_tmdb_cache: bool,
) -> String {
    let is_get = method.trim().eq_ignore_ascii_case("GET");
    let tmdb_target = if is_get {
        canonical_tmdb_cache_key(url, fallback_tmdb_key)
    } else {
        None
    };

    if let Some((cache_key, fetch_url)) = tmdb_target {
        if !use_shared_tmdb_cache {
            TMDB_UPSTREAM_CALLS.fetch_add(1, Ordering::Relaxed);
            return match perform_raw_http_async(&fetch_url, method, headers_json, body, redirect)
                .await
            {
                Ok(v) => v.to_string(),
                Err(err) => error_json(&fetch_url, &err),
            };
        }

        {
            let mut cache = TMDB_RESPONSE_CACHE.lock().expect("tmdb cache");
            if let Some(entry) = cache.get(&cache_key) {
                if Instant::now() < entry.expires_at {
                    TMDB_SHARED_HITS.fetch_add(1, Ordering::Relaxed);
                    let mut cloned = entry.payload_template.clone();
                    if let Some(obj) = cloned.as_object_mut() {
                        obj.insert("url".to_string(), serde_json::Value::String(url.to_string()));
                    }
                    return cloned.to_string();
                }
                cache.remove(&cache_key);
            }
        }

        let (slot, is_leader) = {
            let mut inflight = TMDB_INFLIGHT.lock().expect("tmdb inflight");
            if let Some(existing) = inflight.get(&cache_key) {
                (Arc::clone(existing), false)
            } else {
                let created = Arc::new(AsyncInFlightSlot {
                    result: Mutex::new(None),
                    notify: Notify::new(),
                });
                inflight.insert(cache_key.clone(), Arc::clone(&created));
                (created, true)
            }
        };

        if is_leader {
            TMDB_UPSTREAM_CALLS.fetch_add(1, Ordering::Relaxed);
            let res_val =
                perform_raw_http_async(&fetch_url, method, headers_json, body, redirect).await;
            let serialized = match &res_val {
                Ok(v) => v.to_string(),
                Err(err) => error_json(&fetch_url, err),
            };
            if let Ok(v) = res_val
                && v.get("ok").and_then(|o| o.as_bool()) == Some(true)
            {
                TMDB_RESPONSE_CACHE.lock().expect("tmdb cache").insert(
                    cache_key.clone(),
                    CachedTmdbEntry {
                        expires_at: Instant::now() + TMDB_CACHE_TTL,
                        payload_template: v,
                    },
                );
            }
            {
                *slot.result.lock().expect("slot") = Some(serialized.clone());
                slot.notify.notify_waiters();
            }
            TMDB_INFLIGHT.lock().expect("tmdb inflight").remove(&cache_key);
            return serialized;
        }

        TMDB_SHARED_HITS.fetch_add(1, Ordering::Relaxed);
        loop {
            let notified = slot.notify.notified();
            if let Some(payload) = slot.result.lock().expect("slot").clone() {
                return payload;
            }
            notified.await;
        }
    }

    match perform_raw_http_async(url, method, headers_json, body, redirect).await {
        Ok(v) => v.to_string(),
        Err(err) => error_json(url, &err),
    }
}

fn error_json(url: &str, err: &str) -> String {
    serde_json::json!({
        "ok": false,
        "status": 0,
        "statusText": err,
        "url": url,
        "headers": {},
        "body": "",
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn executes_js_plugin_with_cheerio_dom_bridge() {
        let code = r#"
            function getStreams(tmdbId, mediaType, season, episode) {
                var u = new URL('/watch?v=' + tmdbId, 'https://cdn.example');
                var $ = cheerio.load('<div class="links"><a href="https://cdn.example/video.m3u8">1080p</a></div>');
                var url = $('div.links a').attr('href');
                var quality = $('div.links a').text();
                return [{
                    name: 'RustPlugin ' + u.searchParams.get('v'),
                    title: mediaType + ' ' + quality,
                    url: url,
                    quality: quality
                }];
            }
        "#;
        let args = PluginCallArgs {
            tmdb_id: "550".into(),
            media_type: "movie".into(),
            season: None,
            episode: None,
        };
        let streams = PluginRuntime::execute_sync(code, "test_scraper", "{}", "", &args).unwrap();
        assert_eq!(streams.len(), 1);
        assert_eq!(streams[0].url, "https://cdn.example/video.m3u8");
        assert_eq!(streams[0].quality.as_deref(), Some("1080p"));
    }

    #[test]
    fn canonicalizes_tmdb_urls_ignoring_api_key() {
        let (k1, u1) = canonical_tmdb_cache_key(
            "https://api.themoviedb.org/3/tv/1429?api_key=AAA&language=pt-BR",
            "FALLBACK",
        )
        .unwrap();
        let (k2, _) = canonical_tmdb_cache_key(
            "https://api.themoviedb.org/3/tv/1429?language=pt-BR&api_key=BBB",
            "FALLBACK",
        )
        .unwrap();
        assert_eq!(k1, k2);
        assert!(u1.contains("api_key=AAA"));

        let (_, u3) = canonical_tmdb_cache_key(
            "https://api.themoviedb.org/3/tv/1429?language=pt-BR",
            "INJECTED",
        )
        .unwrap();
        assert!(u3.contains("api_key=INJECTED"));
    }
}

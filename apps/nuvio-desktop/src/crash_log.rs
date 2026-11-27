//! Minimal crash log so a crashed player/window leaves evidence behind.
//!
//! Windows builds run without a console (`windows_subsystem = "windows"`), so a
//! panic is otherwise invisible. Every panic (main thread or player thread) is
//! appended to `%LOCALAPPDATA%\Nuvio\crash.log` with a UTC timestamp, the
//! location, the thread and a captured backtrace. `note()` writes lighter
//! breadcrumb lines (libmpv resolution, overlay creation) that show what the
//! app was doing right before a native crash.
//!
//! On Linux/macOS the same file lives in the app config directory, so the
//! diagnostics can be exercised during development.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

const LOG_FILE: &str = "crash.log";
const PREVIOUS_LOG_FILE: &str = "crash.prev.log";
/// Rotate instead of growing forever when a panic repeats in a loop.
const MAX_BYTES: u64 = 2 * 1024 * 1024;

static LOG_PATH: OnceLock<PathBuf> = OnceLock::new();

thread_local! {
    /// A panic inside the hook itself (e.g. while resolving the path) must not
    /// recurse and turn into an abort.
    static IN_HOOK: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Absolute path of the crash log (`%LOCALAPPDATA%\Nuvio\crash.log` on Windows).
pub fn path() -> PathBuf {
    LOG_PATH.get_or_init(|| log_dir().join(LOG_FILE)).clone()
}

fn log_dir() -> PathBuf {
    if cfg!(target_os = "windows") {
        env_path(&["LOCALAPPDATA", "APPDATA"])
            .unwrap_or_else(std::env::temp_dir)
            .join("Nuvio")
    } else if cfg!(target_os = "macos") {
        home_dir().join("Library/Application Support/Nuvio")
    } else {
        env_path(&["XDG_CONFIG_HOME"])
            .unwrap_or_else(|| home_dir().join(".config"))
            .join("nuvio")
    }
}

fn env_path(names: &[&str]) -> Option<PathBuf> {
    names.iter().find_map(|name| {
        std::env::var_os(name)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
    })
}

fn home_dir() -> PathBuf {
    env_path(&["HOME"]).unwrap_or_else(std::env::temp_dir)
}

/// Installs the panic hook and writes the session header. Call once, first
/// thing in `main`, so panics during startup are recorded too.
pub fn install() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        write_panic(info);
        previous(info);
    }));
    note(&format!(
        "sessao iniciada - Nuvio {} ({} {}), pid {} - log em {}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH,
        std::process::id(),
        path().display()
    ));
}

/// Appends a breadcrumb line. Used around risky native calls (libmpv, overlay).
pub fn note(context: &str) {
    let thread = std::thread::current();
    let name = thread.name().unwrap_or("sem-nome");
    let entry = format!("[{}] [{name}] {context}\n", timestamp());
    append(&entry);
}

fn write_panic(info: &std::panic::PanicHookInfo<'_>) {
    if IN_HOOK.with(|busy| busy.replace(true)) {
        return;
    }
    let thread = std::thread::current();
    let name = thread.name().unwrap_or("sem-nome");
    let location = info
        .location()
        .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
        .unwrap_or_else(|| "desconhecido".to_string());
    let backtrace = std::backtrace::Backtrace::force_capture();
    let entry = format!(
        "===== {} =====\n\
         tipo: PANIC\n\
         versao: {}\n\
         so: {} / {}\n\
         thread: {name}\n\
         local: {location}\n\
         mensagem: {}\n\
         backtrace:\n{backtrace}\n\n",
        timestamp(),
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH,
        panic_message(info),
    );
    append(&entry);
    IN_HOOK.with(|busy| busy.set(false));
}

fn panic_message(info: &std::panic::PanicHookInfo<'_>) -> String {
    let payload = info.payload();
    if let Some(text) = payload.downcast_ref::<&str>() {
        (*text).to_string()
    } else if let Some(text) = payload.downcast_ref::<String>() {
        text.clone()
    } else {
        "<payload nao textual>".to_string()
    }
}

fn append(entry: &str) {
    let path = path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if fs::metadata(&path).map(|meta| meta.len() > MAX_BYTES).unwrap_or(false) {
        // Windows rename fails when the destination exists, so drop it first.
        let previous = path.with_file_name(PREVIOUS_LOG_FILE);
        let _ = fs::remove_file(&previous);
        let _ = fs::rename(&path, previous);
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = file.write_all(entry.as_bytes());
        let _ = file.flush();
    }
}

/// `YYYY-MM-DD HH:MM:SS UTC`, computed without a date crate.
fn timestamp() -> String {
    let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        return "data-desconhecida".to_string();
    };
    let secs = now.as_secs();
    let (year, month, day) = civil_from_days((secs / 86_400) as i64);
    let rem = secs % 86_400;
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02} UTC",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

/// Howard Hinnant's `civil_from_days`: days since 1970-01-01 to a calendar date.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if month <= 2 { year + 1 } else { year }, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_known_epoch_dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(20_000), (2024, 10, 4));
    }

    /// Exercises the real hook: breadcrumb, a caught panic, and the file it all
    /// lands in. Uses a throwaway config dir so the user's crash.log is never
    /// touched; the hook chains to the previous one, so the test harness still
    /// reports failures normally.
    #[test]
    #[cfg(not(target_os = "windows"))]
    fn writes_breadcrumbs_and_panics_to_the_crash_log() {
        let dir = std::env::temp_dir().join(format!("nuvio-crash-test-{}", std::process::id()));
        unsafe { std::env::set_var("XDG_CONFIG_HOME", &dir) };
        install();
        let log = path();
        assert_eq!(log, dir.join("nuvio").join(LOG_FILE), "caminho inesperado");

        note("teste de breadcrumb");
        let panicked = std::panic::catch_unwind(|| panic!("teste de panico do crash_log"));
        assert!(panicked.is_err());

        let contents = fs::read_to_string(&log).expect("crash.log criado");
        assert!(contents.contains("teste de breadcrumb"), "{contents}");
        assert!(contents.contains("tipo: PANIC"), "{contents}");
        assert!(contents.contains("teste de panico do crash_log"), "{contents}");
        assert!(contents.contains("crash_log.rs:"), "{contents}");
        assert!(contents.contains("UTC"), "{contents}");
        let _ = fs::remove_dir_all(&dir);
    }
}

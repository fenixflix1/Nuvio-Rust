use serde::Serialize;
use std::collections::VecDeque;
use std::sync::{LazyLock, Mutex};

use crate::player::PlayerMetrics;

/// One point of the session's memory curve, sampled in the background.
#[derive(Debug, Clone, Copy)]
pub struct Sample {
    pub at_ms: i64,
    pub app_mb: f64,
    pub webkit_mb: f64,
    pub total_mb: f64,
}

const HISTORY_LEN: usize = 360; // one hour at 10 s per sample
const SAMPLE_EVERY: std::time::Duration = std::time::Duration::from_secs(10);

static HISTORY: LazyLock<Mutex<VecDeque<Sample>>> =
    LazyLock::new(|| Mutex::new(VecDeque::with_capacity(HISTORY_LEN)));

fn push_sample(sample: Sample) {
    let mut history = HISTORY.lock().expect("history lock");
    history.push_back(sample);
    while history.len() > HISTORY_LEN {
        history.pop_front();
    }
}

fn history_snapshot() -> Vec<Sample> {
    HISTORY
        .lock()
        .expect("history lock")
        .iter()
        .copied()
        .collect()
}

/// Records the process tree's memory every 10 s for the whole session.
///
/// The leak report is otherwise impossible to judge: a single reading cannot
/// tell a steady state from a slow climb, and the app has to be left running for
/// hours to tell them apart.
pub fn spawn_monitor() {
    std::thread::spawn(|| {
        loop {
            let metrics = collect(0);
            push_sample(Sample {
                at_ms: now_ms(),
                app_mb: metrics
                    .processes
                    .iter()
                    .find(|p| p.pid == metrics.app_pid)
                    .map(|p| p.rss_mb)
                    .unwrap_or(0.0),
                webkit_mb: metrics
                    .processes
                    .iter()
                    .filter(|p| p.name.starts_with("WebKitWebProcess"))
                    .map(|p| p.rss_mb)
                    .sum(),
                total_mb: metrics.total_rss_mb,
            });
            std::thread::sleep(SAMPLE_EVERY);
        }
    });
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GrowthReport {
    pub first_sample_age_minutes: f64,
    pub samples: usize,
    pub total_growth_mb: f64,
    pub app_growth_mb: f64,
    pub webkit_growth_mb: f64,
    /// Total RSS of the most recent samples, oldest first, for a compact sparkline.
    pub recent_mb: Vec<f64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessMetrics {
    pub pid: u32,
    pub name: String,
    pub rss_mb: f64,
    pub peak_mb: f64,
    pub cpu_percent: f64,
    pub threads: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeMetrics {
    pub debug_build: bool,
    pub app_pid: u32,
    pub total_rss_mb: f64,
    pub total_cpu_percent: f64,
    pub processes: Vec<ProcessMetrics>,
    pub player: Option<PlayerMetrics>,
    pub growth: GrowthReport,
    pub note: String,
}

/// Samples the app process and its children (the GTK WebKit processes included).
pub fn collect(window_ms: u64) -> RuntimeMetrics {
    #[cfg(target_os = "linux")]
    {
        collect_linux(window_ms)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = window_ms;
        RuntimeMetrics {
            debug_build: cfg!(debug_assertions),
            app_pid: std::process::id(),
            total_rss_mb: 0.0,
            total_cpu_percent: 0.0,
            processes: Vec::new(),
            player: None,
            growth: growth_report(),
            note: "A medição por processo está disponível apenas no Linux.".into(),
        }
    }
}

#[cfg(target_os = "linux")]
const USER_HZ: f64 = 100.0;

#[cfg(target_os = "linux")]
fn read_stat(pid: u32) -> Option<(u32, u64, u64, u32)> {
    let raw = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let rest = raw.rsplit_once(')')?.1;
    let fields: Vec<&str> = rest.split_whitespace().collect();
    let ppid = fields.get(1)?.parse().ok()?;
    let utime: u64 = fields.get(11)?.parse().ok()?;
    let stime: u64 = fields.get(12)?.parse().ok()?;
    let threads: u32 = fields.get(17)?.parse().ok()?;
    Some((ppid, utime, stime, threads))
}

#[cfg(target_os = "linux")]
fn read_status(pid: u32) -> (String, f64, f64) {
    let mut name = String::new();
    let mut rss = 0.0;
    let mut peak = 0.0;
    if let Ok(raw) = std::fs::read_to_string(format!("/proc/{pid}/status")) {
        for line in raw.lines() {
            let Some((key, value)) = line.split_once(':') else {
                continue;
            };
            match key {
                "Name" => name = value.trim().to_string(),
                // VmRSS / VmHWM are reported in kB.
                "VmRSS" => rss = kilobytes(value),
                "VmHWM" => peak = kilobytes(value),
                _ => {}
            }
        }
    }
    (name, rss, peak)
}

#[cfg(target_os = "linux")]
fn kilobytes(value: &str) -> f64 {
    let kb = value
        .split_whitespace()
        .next()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0);
    round1(kb / 1024.0)
}

fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

#[cfg(target_os = "linux")]
fn collect_linux(window_ms: u64) -> RuntimeMetrics {
    use std::collections::{HashMap, HashSet};

    let root = std::process::id();
    let mut parents: HashMap<u32, u32> = HashMap::new();
    if let Ok(entries) = std::fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let Some(pid) = name.to_str().and_then(|n| n.parse::<u32>().ok()) else {
                continue;
            };
            if let Some((ppid, _, _, _)) = read_stat(pid) {
                parents.insert(pid, ppid);
            }
        }
    }

    let mut pids = vec![root];
    let mut seen: HashSet<u32> = HashSet::from([root]);
    let mut index = 0;
    while index < pids.len() {
        let parent = pids[index];
        index += 1;
        for (&pid, &ppid) in &parents {
            if ppid == parent && seen.insert(pid) {
                pids.push(pid);
            }
        }
    }

    let before: Vec<_> = pids
        .iter()
        .map(|&pid| (pid, read_stat(pid), read_status(pid)))
        .collect();
    std::thread::sleep(std::time::Duration::from_millis(window_ms.clamp(50, 3000)));

    let mut processes = Vec::with_capacity(before.len());
    for (pid, stat_before, (name, rss, peak)) in before {
        let stat_after = read_stat(pid);
        let cpu_percent = match (stat_before, stat_after) {
            (Some((_, u0, s0, _)), Some((_, u1, s1, _))) => {
                let ticks = u1.saturating_sub(u0) + s1.saturating_sub(s0);
                round1(
                    ticks as f64 / USER_HZ / (window_ms.max(1) as f64 / 1000.0) * 100.0,
                )
            }
            _ => 0.0,
        };
        let threads = stat_after.map_or(0, |stat| stat.3);
        processes.push(ProcessMetrics {
            pid,
            name,
            rss_mb: rss,
            peak_mb: peak,
            cpu_percent,
            threads,
        });
    }
    processes.sort_by(|a, b| b.rss_mb.total_cmp(&a.rss_mb));

    let total_rss_mb = round1(processes.iter().map(|p| p.rss_mb).sum());
    let total_cpu_percent = round1(processes.iter().map(|p| p.cpu_percent).sum());

    RuntimeMetrics {
        debug_build: cfg!(debug_assertions),
        app_pid: root,
        total_rss_mb,
        total_cpu_percent,
        processes,
        player: None,
        growth: growth_report(),
        note: if cfg!(debug_assertions) {
            "Build de depuração: CPU e memória ficam bem acima da versão release.".into()
        } else {
            String::new()
        },
    }
}

fn growth_report() -> GrowthReport {
    let samples = history_snapshot();
    let (Some(first), Some(last)) = (samples.first(), samples.last()) else {
        return GrowthReport {
            first_sample_age_minutes: 0.0,
            samples: 0,
            total_growth_mb: 0.0,
            app_growth_mb: 0.0,
            webkit_growth_mb: 0.0,
            recent_mb: Vec::new(),
        };
    };
    GrowthReport {
        first_sample_age_minutes: round1((last.at_ms - first.at_ms) as f64 / 60_000.0),
        samples: samples.len(),
        total_growth_mb: round1(last.total_mb - first.total_mb),
        app_growth_mb: round1(last.app_mb - first.app_mb),
        webkit_growth_mb: round1(last.webkit_mb - first.webkit_mb),
        recent_mb: samples
            .iter()
            .rev()
            .take(60)
            .rev()
            .map(|sample| sample.total_mb)
            .collect(),
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;

    #[test]
    fn samples_own_process_tree() {
        let metrics = collect(120);
        assert_eq!(metrics.app_pid, std::process::id());
        assert!(metrics.total_rss_mb > 0.0);
        assert!(metrics.processes.iter().any(|p| p.pid == metrics.app_pid));
        assert!(metrics.processes.iter().all(|p| p.rss_mb >= 0.0));
    }
}

use nuvio_plugins::{
    PluginCallArgs, PluginExecOptions, PluginRuntime, clear_tmdb_cache, prefetch_tmdb_for_plugins,
    tmdb_cache_stats,
};
use nuvio_storage::{AppPaths, KvStore};
use serde::Deserialize;
use std::time::Instant;

#[derive(Debug, Clone, Deserialize)]
struct StoredScraper {
    id: String,
    name: String,
    supported_types: Vec<String>,
    enabled: bool,
    code: String,
}

#[derive(Debug, Deserialize)]
struct StoredPlugins {
    scrapers: Vec<StoredScraper>,
}

#[derive(Debug, Deserialize)]
struct StoredSettings {
    #[serde(default)]
    tmdb_api_key: String,
}

async fn run_suite(
    label: &str,
    scrapers: &[StoredScraper],
    tmdb_key: &str,
    args: &PluginCallArgs,
    shared_tmdb: bool,
    prefetch: bool,
) -> (u128, u64, u64) {
    clear_tmdb_cache();
    let t_total = Instant::now();

    if prefetch && !tmdb_key.is_empty() {
        prefetch_tmdb_for_plugins(&args.tmdb_id, &args.media_type, args.season, tmdb_key).await;
    }

    let mut handles = Vec::with_capacity(scrapers.len());
    for s in scrapers {
        let code = s.code.clone();
        let id = s.id.clone();
        let name = s.name.clone();
        let key = tmdb_key.to_string();
        let a = args.clone();
        handles.push(tokio::spawn(async move {
            let t0 = Instant::now();
            let res = PluginRuntime::execute_plugin_with_options(
                code,
                id.clone(),
                "{}".to_string(),
                key,
                a,
                PluginExecOptions {
                    use_shared_tmdb_cache: shared_tmdb,
                },
            )
            .await;
            let ms = t0.elapsed().as_millis();
            (id, name, ms, res)
        }));
    }

    let mut rows = Vec::new();
    for h in handles {
        if let Ok(item) = h.await {
            rows.push(item);
        }
    }
    rows.sort_by_key(|r| r.2);
    let wall_ms = t_total.elapsed().as_millis();
    let stats = tmdb_cache_stats();

    println!("\n=== {label} (wall={wall_ms}ms, tmdb_http={}, tmdb_shared_hits={}) ===",
        stats.upstream_http_calls, stats.cache_or_coalesced_hits);
    for (id, name, ms, res) in &rows {
        match res {
            Ok(streams) => println!("  {id:20} | {name:24} | {ms:5} ms | ok ({} streams)", streams.len()),
            Err(e) => println!("  {id:20} | {name:24} | {ms:5} ms | ERR: {e}"),
        }
    }
    (wall_ms, stats.upstream_http_calls, stats.cache_or_coalesced_hits)
}

#[tokio::main]
async fn main() {
    let Ok(paths) = AppPaths::resolve() else {
        eprintln!("AppPaths unavailable");
        return;
    };
    let Ok(kv) = KvStore::open(paths.db_path()) else {
        eprintln!("KV DB not found");
        return;
    };
    let Some(raw_plugins) = kv.namespace("plugins").get_string("state_json").ok().flatten() else {
        eprintln!("No plugins installed in KV");
        return;
    };
    let Ok(ps) = serde_json::from_str::<StoredPlugins>(&raw_plugins) else {
        eprintln!("Failed to parse plugins.state_json");
        return;
    };
    let tmdb_key = kv
        .namespace("desktop_settings")
        .get_string("config_json")
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str::<StoredSettings>(&s).ok())
        .map(|s| s.tmdb_api_key)
        .unwrap_or_default();

    let tv_scrapers: Vec<StoredScraper> = ps
        .scrapers
        .iter()
        .filter(|s| {
            s.enabled
                && s.supported_types
                    .iter()
                    .any(|t| t == "tv" || t == "series")
        })
        .cloned()
        .collect();

    let tv_args = PluginCallArgs {
        tmdb_id: "1429".to_string(),
        media_type: "tv".to_string(),
        season: Some(1),
        episode: Some(1),
    };

    println!("Benchmark TV/Anime TMDB=1429 S1E1 ({} scrapers, key_present={})",
        tv_scrapers.len(), !tmdb_key.is_empty());
    let orig = run_suite(
        "Modo Nuvio Original (sem cache TMDB compartilhado)",
        &tv_scrapers,
        &tmdb_key,
        &tv_args,
        false,
        false,
    )
    .await;
    let opt = run_suite(
        "Modo Busca Unica Compartilhada (prefetch + single-flight)",
        &tv_scrapers,
        &tmdb_key,
        &tv_args,
        true,
        true,
    )
    .await;

    println!("\nRESUMO_TV|orig_ms={}|orig_tmdb_http={}|shared_ms={}|shared_tmdb_http={}|shared_hits={}",
        orig.0, orig.1, opt.0, opt.1, opt.2);
}

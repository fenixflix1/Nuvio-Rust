use nuvio_core::p2p::ffi::NativeNuvioEngineLibrary;
use nuvio_core::p2p::{build_magnet_uri, find_bundled_engine_library, platform_library_name};
use nuvio_core::storage::{AppPaths, CacheStore, KvStore, sha256_hex};
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let paths = AppPaths::resolve().expect("Failed to resolve platform app paths");
    println!("OS: {}", std::env::consts::OS);
    println!("Config dir: {}", paths.config_dir.display());
    println!("Cache dir:  {}", paths.cache_dir.display());
    println!("SQLite DB:  {}", paths.db_path().display());
    println!("Redb cache: {}", paths.cache_db_path().display());

    let kv = KvStore::open(paths.db_path()).expect("Failed to open SQLite KV store");
    let ns = kv.namespace("diagnostics");
    ns.put_string("last_runtime", Some("rust")).unwrap();
    println!(
        "KV check (diagnostics.last_runtime): {:?}",
        ns.get_string("last_runtime").unwrap()
    );

    let cache = CacheStore::open(paths.cache_db_path()).expect("Failed to open Redb cache");
    cache.set("probe", b"ok", Some(60)).unwrap();
    println!(
        "Redb check (probe): {:?}",
        cache.get("probe").unwrap().map(|b| String::from_utf8_lossy(&b).into_owned())
    );

    println!("PIN SHA-256('1234'): {}", sha256_hex("1234"));

    let magnet = build_magnet_uri("0123456789abcdef0123456789abcdef01234567", &[]).unwrap();
    println!("Sample magnet URI len: {} bytes", magnet.len());

    let repo_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if let Some(lib_path) = find_bundled_engine_library(&repo_root) {
        println!("Found bundled engine ({}): {}", platform_library_name(), lib_path.display());
        match unsafe { NativeNuvioEngineLibrary::load(&lib_path) } {
            Ok(lib) => {
                println!(
                    "  nuvio_engine_api_version: {:?}",
                    lib.api_version()
                );
                println!(
                    "  nuvio_engine_protocol_backend_version: {:?}",
                    lib.protocol_backend_version()
                );
            }
            Err(e) => println!("  Load failed: {e}"),
        }
    } else {
        println!(
            "Bundled engine ({}) not found under {}",
            platform_library_name(),
            repo_root.display()
        );
    }
}

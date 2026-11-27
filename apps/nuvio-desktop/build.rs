use flate2::{Compression, write::GzEncoder};
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-env-changed=NUVIO_EMBED_LIBMPV_PATH");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR missing"));
    let generated_rs = out_dir.join("embedded_libmpv.rs");
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    let mut embedded = false;
    if target_os == "windows"
        && let Ok(dll_path) = env::var("NUVIO_EMBED_LIBMPV_PATH")
        && !dll_path.trim().is_empty()
    {
        let src_path = PathBuf::from(dll_path.trim());
        println!("cargo:rerun-if-changed={}", src_path.display());
        let raw = fs::read(&src_path).unwrap_or_else(|e| {
            panic!("Failed to read NUVIO_EMBED_LIBMPV_PATH ({}): {e}", src_path.display())
        });
        let gz_path = out_dir.join("libmpv-2.dll.gz");
        let gz_file = File::create(&gz_path).expect("create libmpv-2.dll.gz");
        let mut encoder = GzEncoder::new(gz_file, Compression::default());
        encoder.write_all(&raw).expect("compress libmpv-2.dll");
        encoder.finish().expect("finish gzip libmpv-2.dll");
        embedded = true;
    }

    let content = if embedded {
        "pub static EMBEDDED_LIBMPV_GZ: Option<&[u8]> = Some(include_bytes!(concat!(env!(\"OUT_DIR\"), \"/libmpv-2.dll.gz\")));\n"
    } else {
        "pub static EMBEDDED_LIBMPV_GZ: Option<&[u8]> = None;\n"
    };
    fs::write(&generated_rs, content).expect("write embedded_libmpv.rs");

    tauri_build::build()
}

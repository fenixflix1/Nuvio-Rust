# Nuvio Desktop — Rust Core Workspace

Cross-platform (Linux & Windows) Rust rewrite of the non-UI Nuvio core modules.

## Workspace Crates

| Crate | Replaces (Kotlin / Native) | Key Dependencies |
| --- | --- | --- |
| `nuvio-storage` | `DesktopStorage`, `ProfilePinCrypto`, `PluginCrypto` | `directories`, `rusqlite` (bundled), `redb`, `sha2`, `hmac`, `pbkdf2`, `aes-gcm`, `cbc`, `ecb` |
| `nuvio-supabase` | `SupabaseProvider` (`supabase-kt`: Auth, PostgREST, Functions, Storage, Realtime) | `reqwest`, `tokio`, `tokio-tungstenite` |
| `nuvio-integrations` | TMDB, Trakt, Simkl, MDBList, Debrid (`Torbox`, `Premiumize`, `RealDebrid`), Stremio Addons, Sentry | `reqwest`, `serde_json`, `sentry` |
| `nuvio-plugins` | `PluginRuntime` (`quickjs-kt`), `DomBridge` (`ksoup`), `FetchBridge` | `boa_engine`, `scraper`, `reqwest` |
| `nuvio-p2p` | `P2pStreaming`, `DesktopEngineLibrary` (`libnuvio_engine.so` / `nuvio_engine.dll`) | `libloading` (+ optional pure-Rust `librqbit` feature) |
| `nuvio-core` | Unified facade + `nuvio-cli` diagnostic binary | All crates above |
| `nuvio-desktop` | Desktop UI app (Linux WebKitGTK / Windows WebView2) | `tauri` v2, `nuvio-core`, external `mpv` |

## Build & Test

Run the Tauri v2 Desktop App (from repo root or inside `rust/`):
```bash
cd rust
cargo run -p nuvio-desktop
```

Run workspace tests and CLI diagnostics:
```bash
cd rust
cargo test --workspace
cargo run -p nuvio-core --bin nuvio-cli
```

## Windows Target (from Linux or Windows)

On Windows:
```powershell
cd rust
cargo test --workspace
```

Cross-compile from Linux (`x86_64-pc-windows-gnu`, requires `mingw-w64` + `rustup target add x86_64-pc-windows-gnu`):
```bash
cd rust
cargo build --workspace --target x86_64-pc-windows-gnu
```

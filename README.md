# Nuvio Desktop (Rust Edition)

> **Aviso honesto:** este projeto nasceu quase como uma piada / experimento feito com IA — *"e se a gente reescrevesse o Nuvio Desktop inteiro em Rust para ver até onde ia em peso, velocidade e consumo de memória?"*  
> A brincadeira deu certo demais e virou um cliente funcional completo em **Rust + Tauri v2 + libmpv + QuickJS**.

---

## Qual é a ideia do projeto?

Recriar a experiência visual e funcional do **Nuvio Desktop** (originalmente em Kotlin / Compose Multiplatform + JVM) em uma base **100% Rust no backend**, mantendo compatibilidade com:

- **Conta e sincronização Nuvio (Supabase)**: login, Device Link, perfis, biblioteca, marcados como assistido e histórico (*Continuar Assistindo*).
- **Addons Stremio**: catálogos, busca progressiva, metadados e streams em paralelo sem travar esperando o addon mais lento.
- **Plugins JS em QuickJS (`rquickjs`)**: suporte aos repositórios e scrapers JS existentes com bridge DOM estilo Cheerio (`scraper`).
- **Player embutido (`libmpv`)**: controles integrados na janela, faixas de áudio e legendas (embutidas e addons), sincronização/atraso de legendas, proporção de tela e **Pular Abertura / Créditos** (capítulos Matroska, AniSkip + ARM e IntroDB).

---

## O sistema de busca única para Plugins (TMDB Cache + Single-Flight)

No fluxo tradicional, quando você abre um filme/episódio com vários scrapers JS ativos (como pacotes com dezenas de provedores), **cada provedor faz suas próprias chamadas repetidas à API do TMDB**, desperdiçando cota e tempo de rede.

Aqui foi implementado um sistema de **busca única compartilhada** (`nuvio-plugins` + `nuvio-desktop`):

1. **Pré-resolução e pré-aquecimento único**: antes de disparar os scrapers, o host converte IMDb → TMDB uma única vez (`resolve_imdb_to_tmdb`) e pré-aquece os metadados (`prefetch_tmdb_for_plugins`).
2. **Cache compartilhado em memória (`TMDB_RESPONSE_CACHE`, TTL 900s)**: todas as chamadas `api.themoviedb.org` feitas dentro do runtime `rquickjs` são interceptadas e normalizadas (ignorando a `api_key` na chave do cache).
3. **Single-Flight (`TMDB_INFLIGHT`)**: se múltiplos scrapers pedirem a mesma URL do TMDB no mesmo instante em paralelo, apenas **uma** requisição HTTP sai para a rede e todas as outras aguardam e reutilizam o resultado.

### Resultado medido (28 provedores do Kamirama, `TMDB=1429` S1E1, `--release`)

| Métrica | Modo Tradicional (TMDB por scraper) | Busca Única Compartilhada |
| --- | ---: | ---: |
| Requisições HTTP enviadas ao TMDB | **125** | **7** (**-94%**, 122 cache hits) |
| Ex. `pomfy` | 5.138 ms | **1.169 ms** |
| Ex. `animefire` | 6.858 ms | **2.439 ms** |
| Ex. `animy` | 6.396 ms | **2.156 ms** |
| Ex. `animesfhd` | 8.013 ms | **4.148 ms** |

---

## Comparativo real: Rust (`--release`) vs Original (Kotlin/AppImage)

Medido na mesma máquina Linux (`DISPLAY=:0`), considerando a árvore completa de processos de cada app ao abrir:

| Métrica | Nuvio Rust (`--release`) | Nuvio Original (Kotlin/AppImage) | Diferença |
| --- | ---: | ---: | --- |
| Abertura até a primeira janela | **189,4 ms** | 4.200,0 ms | **~22x mais rápido** |
| RAM total (`t = 8s`) | **449,1 MiB** | 649,0 MiB | **-199,9 MiB (-31%)** |
| RAM do processo principal (`t = 8s`) | **190,3 MiB** | 472,4 MiB | **2,5x menos RAM** |
| CPU acumulado até `8s` | **3,65 s** | 24,08 s | **6,6x menos CPU** |
| Binário compilado / AppImage | **29,5 MiB** (binário) / **9,6 MiB** (AppImage) | 164,0 MiB | Bem menor em disco |

---

## Arquitetura do Workspace

| Crate / App | Responsabilidade |
| --- | --- |
| `crates/nuvio-storage` | Caminhos nativos, SQLite (`rusqlite`), cache `redb` e criptografia |
| `crates/nuvio-supabase` | Autenticação, Device Link, PostgREST, perfis e sync em nuvem |
| `crates/nuvio-integrations` | TMDB, Trakt, Simkl, MDBList, Debrid e cliente de Addons Stremio |
| `crates/nuvio-plugins` | Runtime QuickJS (`rquickjs`), bridge DOM (`scraper`) e cache TMDB |
| `crates/nuvio-p2p` | Integração opcional P2P via `libloading` |
| `crates/nuvio-core` | Fachada unificada + binário CLI de diagnóstico (`nuvio-cli`) |
| `apps/nuvio-desktop` | App Tauri v2 (Linux WebKitGTK + X11 / Windows WebView2) + `libmpv` |

---

## Executáveis prontos e Como Compilar

Os artefatos gerados ficam em `dist/`:

- **Linux (`dist/NuvioDesktop-Rust-x86_64.AppImage`)**:
  ```bash
  chmod +x dist/NuvioDesktop-Rust-x86_64.AppImage
  ./dist/NuvioDesktop-Rust-x86_64.AppImage
  ```
  > Requer `webkit2gtk-4.1` e `libmpv` (`mpv`) instalados no sistema Linux.

- **Windows (`dist/NuvioDesktop-Rust.exe`)**:
  > Compilado de forma cruzada para `x86_64-pc-windows-msvc` (ainda não testado em uma máquina Windows física). Para o player embutido funcionar no Windows, coloque a `libmpv-2.dll` (ou `mpv-2.dll`) na mesma pasta do `.exe` ou no `PATH` (requer também o Microsoft Edge WebView2 Runtime, já incluso no Windows 10/11).

### Compilando do código-fonte

```bash
# Testes e lint do workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# Rodar em desenvolvimento
cargo run -p nuvio-desktop

# Build release nativo
cargo build --release -p nuvio-desktop
```

Este repositório também inclui [`.github/workflows/release.yml`](.github/workflows/release.yml) para compilar automaticamente o `.AppImage` (Linux) e o `.exe` (Windows) via GitHub Actions.

# Nuvio para LG webOS (`.ipk`)

Este módulo cria o pacote da **TV LG (webOS)** de forma **100% isolada** do aplicativo de PC (`apps/nuvio-desktop`).

- **Zero impacto no Windows e no Linux:** nenhum arquivo Rust (`src/`), Tauri, `libmpv` ou script original da UI de desktop é modificado. O `build-ipk.mjs` apenas copia a UI para `apps/nuvio-webos/dist/` e injeta a camada webOS somente nessa cópia.

## Estrutura
- `app/appinfo.json` e `app/assets/` (`80x80` e `130x130`): metadados oficiais do pacote `com.nuvio.tv`.
- `service/service.js`: serviço Node.js do webOS (`com.nuvio.tv.service`) para requisições HTTP sem CORS (addons Stremio, TMDB, MDBList, nuvio.tv) e persistência local na TV.
- `bridge/webos-bridge.js`: implementa `window.__TAURI__` usando HTML5 `<video>` hardware da TV + serviço webOS.
- `bridge/webos-tv.js` e `webos-tv.css`: navegação por controle remoto (setas, OK/Enter, Voltar `461`, Play/Pause/FF/RW) e foco visível para TV.

## Testar no computador antes de mandar para a TV
```bash
cd apps/nuvio-webos
npm run dev
```
Abra `http://127.0.0.1:8787` no navegador para validar layout, addons e controles sem precisar de TV física.

## Gerar o `.ipk` e instalar na TV LG
1. Instale a CLI oficial da LG webOS uma vez:
   ```bash
   npm install -g @webos-tools/cli
   ```
2. Na TV LG, instale **Developer Mode** pela LG Content Store, entre com uma conta de desenvolvedor (`developer.lge.com`), ligue **Dev Mode Status** e **Key Server**.
3. Cadastre a TV e gere/instale o `.ipk`:
   ```bash
   cd apps/nuvio-webos
   npm run build
   ares-setup-device
   ares-install --device sua-tv dist/com.nuvio.tv_1.0.0_all.ipk
   ares-launch --device sua-tv com.nuvio.tv
   ```

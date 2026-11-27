#!/usr/bin/env node
import fs from 'fs';
import path from 'path';
import { execFileSync, spawnSync } from 'child_process';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const webosRoot = path.resolve(__dirname, '..');
const repoRoot = path.resolve(webosRoot, '..', '..');
const desktopUiDir = path.join(repoRoot, 'apps', 'nuvio-desktop', 'ui');
const distDir = path.join(webosRoot, 'dist');
const distAppDir = path.join(distDir, 'app');
const distServiceDir = path.join(distDir, 'service');

function resetDir(dir) {
  fs.rmSync(dir, { recursive: true, force: true });
  fs.mkdirSync(dir, { recursive: true });
}

function copyRecursive(src, dest) {
  const stat = fs.statSync(src);
  if (stat.isDirectory()) {
    fs.mkdirSync(dest, { recursive: true });
    for (const entry of fs.readdirSync(src)) {
      copyRecursive(path.join(src, entry), path.join(dest, entry));
    }
    return;
  }
  fs.mkdirSync(path.dirname(dest), { recursive: true });
  fs.copyFileSync(src, dest);
}

console.log('[nuvio-webos] Preparando stage isolado em dist/ ...');
resetDir(distAppDir);
resetDir(distServiceDir);

// 1. Copia a UI web sem modificar nenhum arquivo original de Windows/Linux
copyRecursive(desktopUiDir, distAppDir);

// 2. Copia appinfo.json, ícones webOS e bridge/CSS da TV
fs.copyFileSync(path.join(webosRoot, 'app', 'appinfo.json'), path.join(distAppDir, 'appinfo.json'));
copyRecursive(path.join(webosRoot, 'app', 'assets'), path.join(distAppDir, 'assets'));
fs.copyFileSync(path.join(webosRoot, 'bridge', 'webos-tv.css'), path.join(distAppDir, 'webos-tv.css'));
fs.copyFileSync(path.join(webosRoot, 'bridge', 'webos-qr.js'), path.join(distAppDir, 'webos-qr.js'));
fs.copyFileSync(path.join(webosRoot, 'bridge', 'webos-bridge.js'), path.join(distAppDir, 'webos-bridge.js'));
fs.copyFileSync(path.join(webosRoot, 'bridge', 'webos-tv.js'), path.join(distAppDir, 'webos-tv.js'));

// 3. Injeta CSS + QR + Bridge + Navegação D-Pad apenas na cópia dist/app/index.html
const indexPath = path.join(distAppDir, 'index.html');
let html = fs.readFileSync(indexPath, 'utf8');
html = html.replace(
  '<link rel="stylesheet" href="styles.css" />',
  '<link rel="stylesheet" href="styles.css" />\n  <link rel="stylesheet" href="webos-tv.css" />'
);
html = html.replace(
  '<script src="subtitle-sync.js"></script>',
  '<script src="webos-qr.js"></script>\n  <script src="webos-bridge.js"></script>\n  <script src="webos-tv.js"></script>\n  <script src="subtitle-sync.js"></script>'
);
fs.writeFileSync(indexPath, html, 'utf8');

// 4. Copia o serviço Node.js webOS (Luna Service)
copyRecursive(path.join(webosRoot, 'service'), distServiceDir);

console.log('[nuvio-webos] Stage concluído:', distAppDir);

if (process.argv.includes('--stage-only')) {
  process.exit(0);
}

const which = spawnSync('which', ['ares-package'], { encoding: 'utf8' });
if (which.status === 0 && which.stdout.trim()) {
  console.log('[nuvio-webos] Empacotando .ipk com ares-package...');
  execFileSync(which.stdout.trim(), [distAppDir, distServiceDir, '-o', distDir], { stdio: 'inherit' });
  const ipks = fs.readdirSync(distDir).filter((f) => f.endsWith('.ipk'));
  console.log('[nuvio-webos] Pacote gerado:', ipks.join(', '));
} else {
  console.log('[nuvio-webos] ares-package não encontrado. Instale @webos-tools/cli (`npm i -g @webos-tools/cli`) e rode novamente para gerar o .ipk.');
}

const invoke = (cmd, args = {}) => window.__TAURI__.core.invoke(cmd, args);
const listen = (event, cb) => window.__TAURI__.event.listen(event, cb);

const THEMES = [
  { id: 'white', label: 'Original', color: '#F5F5F5' },
  { id: 'gold', label: 'Gold', color: '#E8A91C' },
  { id: 'jade', label: 'Jade', color: '#22D37C' },
  { id: 'rose_gold', label: 'Rose Gold', color: '#EC70A9' },
  { id: 'arctic_blue', label: 'Arctic Blue', color: '#3185F5' },
  { id: 'graphite', label: 'Graphite', color: '#AAB2BE' },
  { id: 'crimson', label: 'Crimson', color: '#E53935' },
  { id: 'ocean', label: 'Ocean', color: '#1E88E5' },
  { id: 'violet', label: 'Violet', color: '#8E24AA' },
  { id: 'emerald', label: 'Emerald', color: '#43A047' },
  { id: 'amber', label: 'Amber', color: '#FB8C00' },
  { id: 'rose', label: 'Rose', color: '#D81B60' },
  { id: 'custom', label: 'Customizado', color: 'conic-gradient(#E8A91C,#22D37C,#3185F5,#EC70A9,#E8A91C)' },
];

const ADDON_PUZZLE_SVG = '<svg viewBox="0 0 24 24" width="18" height="18"><path fill="currentColor" d="M20.5 11H19V7a2 2 0 0 0-2-2h-4V3.5a2.5 2.5 0 0 0-5 0V5H4a2 2 0 0 0-1.99 2v3.8h1.5a2.7 2.7 0 0 1 0 5.4H2V20a2 2 0 0 0 2 2h3.8v-1.5a2.7 2.7 0 0 1 5.4 0V22H17a2 2 0 0 0 2-2v-4h1.5a2.5 2.5 0 0 0 0-5"/></svg>';
const SVG_UP = '<svg viewBox="0 0 24 24" width="18" height="18"><path fill="currentColor" d="M4 12l1.41 1.41L11 7.83V20h2V7.83l5.58 5.59L20 12l-8-8z"/></svg>';
const SVG_DOWN = '<svg viewBox="0 0 24 24" width="18" height="18"><path fill="currentColor" d="M20 12l-1.41-1.41L13 16.17V4h-2v12.17l-5.58-5.59L4 12l8 8z"/></svg>';
const SVG_REFRESH = '<svg viewBox="0 0 24 24" width="18" height="18"><path fill="currentColor" d="M17.65 6.35A7.958 7.958 0 0 0 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08A5.99 5.99 0 0 1 12 18c-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4z"/></svg>';
const SVG_CONFIG = '<svg viewBox="0 0 24 24" width="18" height="18"><path fill="currentColor" d="M19.14 12.94c.04-.3.06-.61.06-.94c0-.32-.02-.64-.07-.94l2.03-1.58a.49.49 0 0 0 .12-.61l-1.92-3.32a.488.488 0 0 0-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54a.484.484 0 0 0-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.09.63-.09.94s.02.64.07.94l-2.03 1.58a.49.49 0 0 0-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6s3.6 1.62 3.6 3.6s-1.62 3.6-3.6 3.6"/></svg>';
const SVG_DELETE = '<svg viewBox="0 0 24 24" width="18" height="18"><path fill="currentColor" d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6zM19 4h-3.5l-1-1h-5l-1 1H5v2h14z"/></svg>';

const DEFAULT_SUBTITLE_STYLE = {
  fontSizeSp: 20,
  outlineEnabled: true,
  bold: false,
  bottomOffset: 24,
  textColor: '#FFFFFFFF',
  outlineColor: '#FF000000',
  backgroundColor: '#00000000',
  shadowEnabled: true,
  shadowOpacity: 75,
  shadowSize: 2,
  fontName: null,
};

const SUBTITLE_SWATCHES = [
  '#FFFFFFFF', '#FFFFD54F', '#FF81D4FA', '#FFA5D6A7', '#FFEF9A9A', '#FF111111', '#00000000',
];

const RESIZE_MODES = ['Ajustar', 'Preencher', 'Zoom', 'Esticar'];

let isSignUp = false;
let devicePollTimer = null;
let heroTimer = null;
let heroItems = [];
let heroIdx = 0;
let authState = null;
let appSettings = null;
let previousTab = 'home';
let homeSectionsMap = new Map();
let cwItemsMap = new Map();
let cwCardsCache = [];
let activeCatalog = null;
let activeMetaContext = null;

let libraryRawItems = [];
let librarySet = new Set();
let watchedSet = new Set();
let libraryListMode = false;

let streamGroups = [];
let activeStreamFilter = 'all';
let activeStreamCtx = null;
let activeScrapers = [];
let streamUnlistenAddon = null;
let streamUnlistenDone = null;
let streamUnlistenScrapers = null;
let homeUnlistenSection = null;
let homeUnlistenDone = null;
let cwUnlistenItem = null;

let hoverPreviewTimer = null;
let hoverPreviewItem = null;
let switchingProfile = false;

const mainViewportEl = document.getElementById('main-viewport');
mainViewportEl.addEventListener(
  'scroll',
  () => {
    if (hoverPreviewItem || !hoverPopupEl.classList.contains('hidden')) {
      hideHoverPreview();
    }

    if (activeTabName() === 'details') {
      updateDetailScrollDim();
    }

    if (
      activeTabName() === 'search'
      && mainViewportEl.scrollTop + mainViewportEl.clientHeight >= mainViewportEl.scrollHeight - 260
    ) {
      maybeInfiniteDiscover();
    }
  },
  { passive: true },
);

function esc(v) {
  return String(v ?? '')
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;');
}

function optimizePosterUrl(url, targetWidth = 342) {
  if (!url || typeof url !== 'string') return '';
  if (url.includes('image.tmdb.org/t/p/')) {
    return url.replace(/\/t\/p\/(w\d+|original)\//, `/t/p/w${targetWidth}/`);
  }
  return url;
}

function showToast(message) {
  if (!message) return;
  const host = document.getElementById('toast-host');
  const el = document.createElement('div');
  el.className = 'nuvio-toast';
  el.textContent = message;
  host.appendChild(el);
  setTimeout(() => {
    el.style.opacity = '0';
    el.style.transition = 'opacity 0.25s';
    setTimeout(() => el.remove(), 260);
  }, 2600);
}

function confirmDialog(title, message) {
  return new Promise((resolve) => {
    const modal = document.getElementById('nuvio-confirm-modal');
    document.getElementById('confirm-modal-title').textContent = title || 'Confirmar';
    document.getElementById('confirm-modal-msg').textContent = message || '';
    modal.classList.remove('hidden');

    const yesBtn = document.getElementById('btn-confirm-yes');
    const noBtn = document.getElementById('btn-confirm-no');

    const finish = (ans) => {
      modal.classList.add('hidden');
      yesBtn.onclick = null;
      noBtn.onclick = null;
      resolve(ans);
    };
    yesBtn.onclick = () => finish(true);
    noBtn.onclick = () => finish(false);
  });
}

function contrastOnHex(hex) {
  const clean = String(hex || '#FFFFFF').replace('#', '');
  if (clean.length < 6) return '#111111';
  const r = parseInt(clean.slice(0, 2), 16) || 0;
  const g = parseInt(clean.slice(2, 4), 16) || 0;
  const b = parseInt(clean.slice(4, 6), 16) || 0;
  const lum = (0.299 * r + 0.587 * g + 0.114 * b) / 255;
  return lum > 0.58 ? '#111111' : '#FFFFFF';
}

const ratingProviders = {
  imdb: 'IMDb',
  tmdb: 'TMDB',
  tomatoes: 'Rotten Tomatoes',
  metacritic: 'Metacritic',
  trakt: 'Trakt',
  letterboxd: 'Letterboxd',
  audience: 'Nota do público',
  mal: 'MyAnimeList',
};
document.getElementById('rating-providers').innerHTML = Object.entries(ratingProviders)
  .map(([key, label]) => `
    <label class="toggle-row">
      <span>${label}</span>
      <span class="nuvio-switch"><input type="checkbox" data-provider="${key}" /><span class="nuvio-switch-track"></span></span>
    </label>`)
  .join('');

function discordActivity() {
  const prefs = appSettings?.services;
  if (!prefs?.discord_enabled) return null;
  const type = prefs.discord_style === 'playing' ? 0 : 3;
  const view = document.querySelector('.view.active')?.id;
  if (view === 'view-player') {
    const ctx = activePlayerCtx;
    if (!ctx?.latestState?.active || ctx.latestState.ended) return null;
    const episode = ctx.episode != null && prefs.episode_name
      ? [`T${ctx.season ?? 1} E${ctx.episode}`, ctx.episodeTitle].filter(Boolean).join(' · ') : '';
    const status = ctx.latestState.paused ? 'Pausado' : 'Assistindo';
    const assets = {};
    if (/^https?:\/\//i.test(ctx.poster || '')) {
      assets.large_image = prefs.portrait
        ? `https://wsrv.nl/?${new URLSearchParams({url: ctx.poster, w: 512, h: 512, fit: 'contain', cbg: 'black'})}` : ctx.poster;
    }
    if (prefs.episode_image && ctx.episode != null && /^https?:\/\//i.test(ctx.episodeThumbnail || '')) {
      assets.small_image = ctx.episodeThumbnail;
    }
    const timestamps = !ctx.latestState.paused && ctx.discordStart
      ? { start: ctx.discordStart } : undefined;
    return { type, timestamps, details: (ctx.title || 'Nuvio').slice(0, 120),
      state: (episode ? (ctx.latestState.paused ? `Pausado · ${episode}` : episode) : status).slice(0, 120), assets };
  }
  if (view !== 'view-app' || prefs.discord_scope === 'playback') return null;
  const tab = activeTabName();
  const section = {home: 'Início', search: 'Busca', discover: 'Catálogos', library: 'Biblioteca',
    settings: 'Ajustes', streams: 'Fontes', catalog: 'Catálogo', details: activeMetaContext?.name || 'Detalhes'};
  return { type, details: `Navegando · ${section[tab] || 'Nuvio'}`.slice(0, 120) };
}

let sentPresence = 'null';
let sendingPresence = false;
let discordConnected = false;
let discordStatusText = '';
function setDiscordStatus(text, connected = false) {
  discordConnected = Boolean(connected);
  if (text === discordStatusText) return;
  discordStatusText = text;
  const el = document.getElementById('discord-presence-status');
  if (el) el.textContent = text;
}

function renderDiscordReport(report, { showDetails = false } = {}) {
  if (!report) return;
  console.info('[discord]', report);
  const el = document.getElementById('discord-diagnostic');
  if (!el) return;
  const lines = [];
  const sockets = Array.isArray(report.sockets) ? report.sockets : [];
  lines.push(sockets.length ? `Canais discord-ipc encontrados: ${sockets.length}` : 'Canais discord-ipc encontrados: nenhum');
  sockets.forEach((path) => lines.push(`  ${path}`));
  if (report.endpoint) lines.push(`Canal usado: ${report.endpoint}`);
  if (report.handshake) lines.push(`Handshake: ${report.handshake}`);
  if (report.activity) lines.push(`Atividade: ${report.activity}`);
  if (Array.isArray(report.details) && report.details.length) {
    lines.push('', 'Passos:');
    report.details.forEach((line) => lines.push(`  ${line}`));
  }
  el.textContent = lines.join('\n');
  if (showDetails) el.hidden = false;
}

async function syncDiscord(force = false) {
  if (sendingPresence || !appSettings) return;
  sendingPresence = true;
  try {
    if (!appSettings.services?.discord_enabled) {
      if (sentPresence !== 'null') {
        sentPresence = 'null';
        await invoke('discord_presence', { activity: null }).catch(() => {});
      }
      setDiscordStatus('Desativada.', false);
      return;
    }
    let activity = discordActivity();
    let key = JSON.stringify(activity);
    let report = null;
    while (key !== sentPresence || (force && activity)) {
      report = await invoke('discord_presence', { activity });
      sentPresence = key;
      force = false;
      if (!report?.ok) break;
      activity = discordActivity();
      key = JSON.stringify(activity);
    }
    if (report && !report.ok) {
      sentPresence = 'null';
      renderDiscordReport(report, { showDetails: true });
      setDiscordStatus(report.message || 'Não foi possível publicar a presença no Discord.', false);
      return;
    }
    if (report) renderDiscordReport(report);
    setDiscordStatus(
      sentPresence === 'null'
        ? 'Conectado ao Discord — sem atividade no momento.'
        : `Conectado ao Discord — ${report?.activity || 'exibindo sua atividade'}.`,
      true,
    );
  } catch (error) {
    sentPresence = 'null';
    setDiscordStatus(`Falha ao chamar o conector do Discord: ${String(error)}`, false);
  } finally {
    sendingPresence = false;
  }
}
// Refresh connected sessions every 30s and retry disconnected sessions faster (every 8s) so opening Discord connects quickly.
setInterval(() => {
  if (appSettings?.services?.discord_enabled && !discordConnected) syncDiscord(true);
}, 8000);
setInterval(() => syncDiscord(true), 30000);

function renderRuntimeMetrics(metrics) {
  const el = document.getElementById('runtime-metrics');
  if (!el || !metrics) return;
  console.info('[metrics]', metrics);
  const lines = [];
  lines.push(metrics.debugBuild
    ? 'Build: depuração (CPU e memória bem acima da release)'
    : 'Build: release');
  (metrics.processes || []).forEach((proc) => {
    const label = proc.pid === metrics.appPid ? `${proc.name} (app)` : proc.name;
    lines.push(`${label} · pid ${proc.pid} · ${proc.rssMb.toFixed(1)} MB (pico ${proc.peakMb.toFixed(1)} MB) · ${proc.cpuPercent.toFixed(1)}% CPU · ${proc.threads} threads`);
  });
  lines.push('', `Total: ${metrics.totalRssMb.toFixed(1)} MB · ${metrics.totalCpuPercent.toFixed(1)}% CPU`);
  const player = metrics.player;
  if (player) {
    const size = player.width && player.height ? `${player.width}x${player.height}` : 'resolução desconhecida';
    const fps = player.fps ? ` @ ${player.fps.toFixed(2)} fps` : '';
    lines.push(`Player: hwdec=${player.hwdecCurrent || 'desconhecido'} · ${player.videoCodec || '?'} ${player.pixelFormat || ''}`.trim() + ` · ${size}${fps} · cache ${Number(player.cacheSecs || 0).toFixed(1)} s · frames perdidos ${player.droppedFrames || 0}`);
  } else {
    lines.push('Player: nenhuma reprodução ativa.');
  }
  const growth = metrics.growth;
  if (growth && growth.samples > 1) {
    const sign = (value) => `${value >= 0 ? '+' : ''}${value.toFixed(1)}`;
    lines.push('', `Sessão monitorada: ${growth.firstSampleAgeMinutes.toFixed(0)} min (${growth.samples} amostras)`);
    lines.push(`Variação: total ${sign(growth.totalGrowthMb)} MB · app ${sign(growth.appGrowthMb)} MB · WebKit ${sign(growth.webkitGrowthMb)} MB`);
    if (growth.recentMb?.length > 3) {
      const max = Math.max(...growth.recentMb);
      const points = growth.recentMb.map((v) => Math.round((v / max) * 8));
      lines.push(`Curva (min→agora): ${points.map((n) => '▁▂▃▄▅▆▇█'[Math.min(7, n)]).join('')}`);
    }
    if (growth.totalGrowthMb > 150) {
      lines.push('Atenção: a memória subiu mais de 150 MB nesta sessão — envie esta medição junto do relato.');
    }
  } else {
    lines.push('', 'Monitor de memória: coletando a primeira amostra (a cada 10 s).');
  }
  if (metrics.note) lines.push('', metrics.note);
  el.textContent = lines.join('\n');
  el.hidden = false;
}

document.getElementById('btn-runtime-metrics')?.addEventListener('click', async (event) => {
  const button = event.currentTarget;
  button.disabled = true;
  button.textContent = 'Medindo…';
  try {
    renderRuntimeMetrics(await invoke('runtime_metrics'));
  } catch (error) {
    const el = document.getElementById('runtime-metrics');
    if (el) {
      el.textContent = `Falha ao medir: ${String(error)}`;
      el.hidden = false;
    }
  } finally {
    button.disabled = false;
    button.textContent = 'Medir agora';
  }
});

document.getElementById('btn-discord-connect')?.addEventListener('click', async (event) => {
  const button = event.currentTarget;
  button.disabled = true;
  button.textContent = 'Conectando…';
  try {
    const toggle = document.querySelector('[data-service="discord_enabled"]');
    if (toggle && !toggle.checked) {
      toggle.checked = true;
      const services = readServicesFromForm();
      await persistSettingsPatch({ services });
    }
    sentPresence = 'null';
    await syncDiscord(true);
    if (discordConnected) {
      showToast('Conectado ao Discord!');
    } else {
      showToast('Abra o Discord (ou arRPC) no PC para conectar.');
    }
  } finally {
    button.disabled = false;
    button.textContent = 'Conectar agora';
  }
});

function showView(id) {
  if (id === "view-auth" || id === "view-profiles") clearConnectedSession();
  document.querySelectorAll('.view').forEach((v) => v.classList.remove('active'));
  document.getElementById(id).classList.add('active');
  syncDiscord();
}

function activeTabName() {
  return document.querySelector('.tab-panel.active')?.id?.replace('tab-', '') || 'home';
}

function setTab(tab) {
  const cur = activeTabName();
  if (cur && cur !== 'details' && cur !== 'streams' && cur !== 'catalog' && cur !== 'insights') {
    previousTab = cur;
  }
  hideHoverPreview();
  closeContextMenu();
  document.querySelectorAll('.sidebar-item, .topnav-item').forEach((b) => {
    b.classList.toggle('active', b.dataset.tab === tab);
  });
  document.querySelectorAll('.tab-panel').forEach((p) => {
    p.classList.toggle('active', p.id === `tab-${tab}`);
  });
  if (tab !== 'streams' && !playerActive && partyLive.active) {
    reportPartyPlayerStatus('browsing', '');
  }
  syncDiscord();
  mainViewportEl.scrollTo({ top: 0 });
  if (tab === 'library') { refreshLibrary(); renderFollowingSection(); }
  if (tab === 'search') {
    renderRecentSearches();
    if (!discoverItems.length && discoverCatalogs.length) syncDiscoverSelection();
  }
  if (tab === 'insights' && typeof loadProfileInsights === 'function') loadProfileInsights();
}

function handleBackNavigation() {
  if (document.getElementById('view-party')?.classList.contains('active')) {
    showView('view-app');
    return;
  }
  if (!document.getElementById('poster-context-overlay').classList.contains('hidden')) {
    closeContextMenu();
    return;
  }
  if (!document.getElementById('nuvio-confirm-modal').classList.contains('hidden')) {
    document.getElementById('btn-confirm-no').click();
    return;
  }
  if (!document.getElementById('custom-server-modal').classList.contains('hidden')) {
    document.getElementById('custom-server-modal').classList.add('hidden');
    return;
  }
  if (playerActive) {
    closeEmbeddedPlayer();
    return;
  }
  const cur = activeTabName();
  if (cur === 'streams') {
    setTab(activeMetaContext ? 'details' : (previousTab || 'home'));
  } else if (cur === 'person') {
    setTab(activeMetaContext ? 'details' : (previousTab || 'home'));
  } else if (cur === 'insights') {
    setTab(previousTab || 'settings');
  } else if (cur === 'details' || cur === 'catalog') {
    setTab(previousTab || 'home');
  }
}

window.addEventListener('mouseup', (e) => {
  if (e.button === 3) {
    e.preventDefault();
    handleBackNavigation();
  }
});

window.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') {
    handleBackNavigation();
  }
});

function formatClock(secs) {
  if (!Number.isFinite(secs) || secs < 0) return '00:00';
  const total = Math.floor(secs);
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  const mm = String(m).padStart(2, '0');
  const ss = String(s).padStart(2, '0');
  return h > 0 ? `${h}:${mm}:${ss}` : `${mm}:${ss}`;
}

function libKey(id, type) {
  return `${type || 'movie'}:${id || ''}`;
}

function epWatchKey(contentId, season, episode) {
  if (season == null || episode == null) return String(contentId || '');
  return `${contentId}:${season}:${episode}`;
}

// --- AUTH SCREEN ---
function syncAuthModeUi() {
  document.getElementById('auth-heading').textContent = isSignUp ? 'Criar conta' : 'Bem-vindo de volta';
  document.getElementById('auth-subtitle').textContent = isSignUp
    ? 'Cadastre-se para sincronizar seus dados em todos os dispositivos'
    : 'Entre para acessar sua biblioteca e seu histórico';
  document.getElementById('btn-auth-submit').textContent = isSignUp ? 'Criar conta' : 'Entrar';
  document.getElementById('auth-toggle-prompt').textContent = isSignUp
    ? 'Já possui uma conta?'
    : 'Não tem uma conta?';
  document.getElementById('btn-auth-toggle-mode').textContent = isSignUp ? 'Entrar' : 'Criar conta';
  document.getElementById('device-link-area').classList.toggle('hidden', isSignUp);
}

document.getElementById('btn-auth-toggle-mode').addEventListener('click', () => {
  isSignUp = !isSignUp;
  cancelDeviceLink();
  document.getElementById('auth-error').classList.add('hidden');
  syncAuthModeUi();
});

document.getElementById('btn-toggle-pass').addEventListener('click', () => {
  const input = document.getElementById('auth-password');
  input.type = input.type === 'password' ? 'text' : 'password';
});

document.getElementById('btn-auth-submit').addEventListener('click', async () => {
  const email = document.getElementById('auth-email').value.trim();
  const password = document.getElementById('auth-password').value;
  const errEl = document.getElementById('auth-error');
  const btn = document.getElementById('btn-auth-submit');
  errEl.classList.add('hidden');
  if (!email || password.length < 6) {
    errEl.textContent = 'Informe um e-mail válido e senha (mínimo 6 caracteres).';
    errEl.classList.remove('hidden');
    return;
  }
  btn.disabled = true;
  try {
    const res = await invoke(isSignUp ? 'sign_up_email' : 'sign_in_email', { email, password });
    await handleAuthenticated(res);
  } catch (e) {
    errEl.textContent = String(e);
    errEl.classList.remove('hidden');
  } finally {
    btn.disabled = false;
  }
});

document.getElementById('btn-auth-guest').addEventListener('click', async () => {
  const res = await invoke('continue_as_guest');
  await enterMainApp(res);
});

function cancelDeviceLink() {
  if (devicePollTimer) {
    clearInterval(devicePollTimer);
    devicePollTimer = null;
  }
  document.getElementById('device-link-waiting').classList.add('hidden');
  document.getElementById('device-link-area').classList.toggle('hidden', isSignUp);
}

document.getElementById('btn-device-link').addEventListener('click', async () => {
  const errEl = document.getElementById('auth-error');
  errEl.classList.add('hidden');
  try {
    const info = await invoke('start_device_link');
    document.getElementById('device-link-area').classList.add('hidden');
    document.getElementById('device-link-waiting').classList.remove('hidden');
    document.getElementById('device-link-code').textContent = info.user_code;
    const urlEl = document.getElementById('device-link-url');
    urlEl.href = info.verification_uri_complete;
    urlEl.textContent = info.verification_uri_complete.replace(/^https?:\/\//, '').split('?')[0];
    document.getElementById('device-link-status').textContent = 'Aguardando aprovação…';

    const intervalMs = Math.max(2, Math.min(10, info.poll_interval_seconds || 5)) * 1000;
    devicePollTimer = setInterval(async () => {
      try {
        const poll = await invoke('poll_device_link', {
          deviceCode: info.device_code,
          nonce: info.nonce,
        });
        if (poll.status === 'approved' && poll.auth) {
          cancelDeviceLink();
          await handleAuthenticated(poll.auth);
        } else if (poll.status !== 'pending') {
          document.getElementById('device-link-status').textContent = `Status: ${poll.status}`;
          clearInterval(devicePollTimer);
          devicePollTimer = null;
        }
      } catch (err) {
        document.getElementById('device-link-status').textContent = String(err);
      }
    }, intervalMs);
  } catch (e) {
    errEl.textContent = String(e);
    errEl.classList.remove('hidden');
  }
});

document.getElementById('btn-device-link-cancel').addEventListener('click', cancelDeviceLink);

document.getElementById('btn-server-menu').addEventListener('click', (e) => {
  e.stopPropagation();
  document.getElementById('server-menu-dropdown').classList.toggle('hidden');
});
document.addEventListener('click', () => {
  document.getElementById('server-menu-dropdown').classList.add('hidden');
});
document.getElementById('btn-use-official-server').addEventListener('click', async () => {
  const srv = await invoke('configure_server', { serverUrl: null });
  document.getElementById('auth-server-pill').textContent = srv.backend_url.replace(/^https?:\/\//, '');
  showToast('Conectado ao servidor oficial.');
});
document.getElementById('btn-open-custom-server').addEventListener('click', () => {
  document.getElementById('custom-server-modal').classList.remove('hidden');
});
document.getElementById('btn-custom-server-cancel').addEventListener('click', () => {
  document.getElementById('custom-server-modal').classList.add('hidden');
});
document.getElementById('btn-custom-server-connect').addEventListener('click', async () => {
  const url = document.getElementById('custom-server-input').value.trim();
  const err = document.getElementById('custom-server-error');
  err.classList.add('hidden');
  try {
    const srv = await invoke('configure_server', { serverUrl: url });
    document.getElementById('auth-server-pill').textContent = srv.backend_url.replace(/^https?:\/\//, '');
    document.getElementById('custom-server-modal').classList.add('hidden');
    showToast('Servidor customizado configurado.');
  } catch (e) {
    err.textContent = String(e);
    err.classList.remove('hidden');
  }
});

// --- PROFILES ---
async function handleAuthenticated(status) {
  authState = status;
  if (status.profiles && status.profiles.length > 0) {
    renderProfilePicker(status.profiles);
    showView('view-profiles');
  } else {
    await enterMainApp(status);
  }
}

function addonCatalogSignature(addons) {
  return (addons || [])
    .filter((a) => a.enabled)
    .flatMap((a) => (a.manifest?.catalogs || [])
      .filter((c) => !(c.extra || []).some((e) => e.name === 'search' && e.is_required))
      .map((c) => `${a.manifest_url}|${c.type}|${c.id}`))
    .slice(0, 18)
    .join(';');
}

async function switchToProfile(idx, fromPicker) {
  if (switchingProfile) return;
  switchingProfile = true;
  if (libraryCollections) libraryCollections.open = false;
  clearAccountCollections();
  document.getElementById('profile-switcher-stack')?.classList.add('hidden');
  authState.active_profile_index = idx;
  if (fromPicker) {
    showView('view-app');
  }
  updateProfileUi();
  updateAccountSettingsUi();
  document.getElementById('home-catalogs-container').innerHTML = '';
  document.getElementById('home-cw-container').classList.add('hidden');
  if (typeof clearHomeCollections === 'function') clearHomeCollections();
  try {
    await invoke('select_profile', { profileIndex: idx });
    const homeTask = refreshHomeProgressive(false);
    const simklTask = simklLoadProfile();
    await Promise.all([
      homeTask,
      simklTask,
      refreshWatchedKeys(),
      refreshLibraryState(),
      refreshFollowedPeople(),
      refreshAddons(),
      refreshPluginsState(),
      refreshContinueWatching(),
    ]);
  } finally {
    switchingProfile = false;
  }
}

function renderProfilePicker(profiles) {
  const grid = document.getElementById('profiles-grid');
  grid.innerHTML = profiles.map((p) => `
    <button class="profile-card" data-profile-index="${p.profile_index}">
      <div class="profile-avatar-lg" style="background:${esc(p.avatar_color_hex || '#1E88E5')}">
        ${p.avatar_url ? `<img src="${esc(p.avatar_url)}" alt="" onerror="this.remove()" />` : esc((p.name || 'P').charAt(0).toUpperCase())}
      </div>
      <span>${esc(p.name || `Perfil ${p.profile_index}`)}</span>
    </button>
  `).join('');

  grid.querySelectorAll('[data-profile-index]').forEach((btn) => {
    btn.addEventListener('click', () => {
      switchToProfile(Number(btn.dataset.profileIndex), true);
    });
  });
}

document.getElementById('btn-profiles-signout').addEventListener('click', async () => {
  ++simklGeneration;
  simklAccount = {};
  await invoke('sign_out');
  showView('view-auth');
});

function homeSnapshotKey() {
  return `nuvio_home_snapshot_p${authState?.active_profile_index || 1}`;
}

function readHomeSnapshot() {
  try {
    const parsed = JSON.parse(localStorage.getItem(homeSnapshotKey()) || 'null');
    return Array.isArray(parsed) && parsed.length ? parsed : null;
  } catch {
    return null;
  }
}

function writeHomeSnapshot() {
  try {
    const list = Array.from(homeSectionsMap.values()).sort((a, b) => a.order - b.order);
    if (!list.length) return;
    localStorage.setItem(homeSnapshotKey(), JSON.stringify(list));
  } catch {
    // Quota/storage errors are non-fatal
  }
}

function renderHomeSectionsChunked(shelvesContainer, sections, gen, onHeroReady) {
  if (!sections || !sections.length) return;
  const sorted = [...sections].sort((a, b) => a.order - b.order);

  const applySection = (sec) => {
    if (gen !== homeRunGen) return;
    document.getElementById('home-loading-note')?.remove();
    const prev = homeSectionsMap.get(sec.order);
    const sameContent = prev && catalogSectionDigest(prev) === catalogSectionDigest(sec);
    homeSectionsMap.set(sec.order, sec);

    if (heroItems.length === 0) {
      const candidates = sec.items.filter((i) => i.banner || i.poster).slice(0, 8);
      if (candidates.length) {
        heroItems = candidates;
        renderHero();
        if (onHeroReady) onHeroReady();
      }
    }

    if (!sameContent || !shelvesContainer.querySelector(`.shelf-section[data-order="${sec.order}"]`)) {
      appendShelfOrdered(shelvesContainer, sec);
    }
  };

  // Paint hero + top 4 visible shelves immediately in frame 0, then remaining in chunks
  const immediateCount = Math.min(4, sorted.length);
  for (let i = 0; i < immediateCount; i += 1) {
    applySection(sorted[i]);
  }
  if (sorted.length <= immediateCount) return;

  let idx = immediateCount;
  const pump = () => {
    if (gen !== homeRunGen) return;
    const end = Math.min(idx + 4, sorted.length);
    for (; idx < end; idx += 1) {
      applySection(sorted[idx]);
    }
    if (idx < sorted.length) {
      requestAnimationFrame(pump);
    }
  };
  requestAnimationFrame(pump);
}

async function enterMainApp(status) {
  if (libraryCollections) libraryCollections.open = false;
  clearAccountCollections();
  authState = status;
  showView('view-app');
  updateProfileUi();
  updateAccountSettingsUi();
  logUiTiming(`boot: vista principal aberta em +${Math.round(performance.now())} ms`);

  // Renderiza o último snapshot local na hora enquanto busca/revalida em paralelo.
  const cachedAddonsTask = invoke('list_addons').catch(() => []);
  const homeTask = refreshHomeProgressive(false);
  const cachedAddons = await cachedAddonsTask;
  const preSyncSig = addonCatalogSignature(cachedAddons);
  if (cachedAddons.length) renderAddonsList(cachedAddons);

  const profileSyncTask = (status.signed_in && status.active_profile_index != null)
    ? invoke('select_profile', { profileIndex: status.active_profile_index })
      .then((synced) => {
        renderAddonsList(synced);
        if (addonCatalogSignature(synced) !== preSyncSig) {
          return refreshHomeProgressive(true);
        }
      })
      .catch((error) => {
        console.error('Falha ao sincronizar os addons do perfil', error);
      })
    : refreshAddons();

  const simklTask = simklLoadProfile().then(() => {
    if (simklUses('library') || simklUses('progress')) {
      return simklRefreshViews();
    }
  });

  await Promise.all([
    homeTask,
    profileSyncTask,
    simklTask,
    refreshWatchedKeys(),
    refreshLibraryState().then(() => {
      preloadLibraryCalendar();
    }),
    refreshContinueWatching(),
  ]);
  setTimeout(() => {
    refreshFollowedPeople();
    refreshPluginsState();
  }, 60);
  if (appSettings?.services?.discord_enabled) {
    syncDiscord(true);
    setTimeout(() => syncDiscord(true), 3000);
    setTimeout(() => syncDiscord(true), 8000);
  }
}

function updateProfileUi() {
  const profiles = authState?.profiles || [];
  const active = profiles.find((p) => p.profile_index === authState?.active_profile_index) || profiles[0];
  const name = active?.name || 'Local';
  const color = active?.avatar_color_hex || 'var(--accent)';
  const avatarInner = active?.avatar_url
    ? `<img src="${esc(active.avatar_url)}" alt="" onerror="this.remove()" />`
    : esc(name.charAt(0).toUpperCase());

  const sbAvatar = document.getElementById('sidebar-avatar');
  sbAvatar.style.background = color;
  sbAvatar.innerHTML = avatarInner;
  document.getElementById('sidebar-profile-name').textContent = name;

  const tnAvatar = document.getElementById('topnav-avatar');
  tnAvatar.style.background = color;
  tnAvatar.innerHTML = avatarInner;
  document.getElementById('topnav-profile-name').textContent = name;

  const stack = document.getElementById('profile-switcher-stack');
  const activeIdx = authState?.active_profile_index;
  stack.innerHTML = profiles.length
    ? '<div class="switcher-title">Trocar perfil</div>' + profiles.map((p) => {
      const isActive = p.profile_index === activeIdx;
      const label = p.name || `Perfil ${p.profile_index}`;
      const avatar = p.avatar_url
        ? `<img src="${esc(p.avatar_url)}" alt="" onerror="this.remove()" />`
        : esc(label.charAt(0).toUpperCase());
      return `
        <button class="switcher-row${isActive ? ' active' : ''}" data-switch-idx="${p.profile_index}" type="button">
          <span class="mini-avatar" style="background:${esc(p.avatar_color_hex || '#1E88E5')}">${avatar}</span>
          <span class="switcher-name">${esc(label)}</span>
          ${isActive ? '<span class="switcher-check" aria-hidden="true">✓</span>' : ''}
        </button>
      `;
    }).join('')
    : '<div class="switcher-title">Nenhum perfil encontrado</div>';
  stack.querySelectorAll('[data-switch-idx]').forEach((btn) => {
    btn.addEventListener('click', () => {
      stack.classList.add('hidden');
      switchToProfile(Number(btn.dataset.switchIdx), false);
    });
  });
}

function positionProfileSwitcher(anchor) {
  const stack = document.getElementById('profile-switcher-stack');
  const trigger = anchor || document.getElementById('btn-sidebar-profile');
  if (!stack || !trigger) return;
  const rect = trigger.getBoundingClientRect();
  const margin = 10;
  stack.classList.remove('hidden');
  const width = stack.offsetWidth;
  const height = stack.offsetHeight;
  let left;
  let top;
  if (trigger.id === 'btn-topnav-profile') {
    left = rect.right - width;
    top = rect.bottom + 8;
  } else {
    left = rect.right + margin;
    top = rect.top;
  }
  if (left + width > window.innerWidth - 12) left = Math.max(12, rect.left - width - margin);
  left = Math.max(12, left);
  if (top + height > window.innerHeight - 12) top = Math.max(12, window.innerHeight - height - 12);
  stack.style.left = `${Math.round(left)}px`;
  stack.style.top = `${Math.round(top)}px`;
}

function setProfileSwitcherOpen(open, anchor) {
  const stack = document.getElementById('profile-switcher-stack');
  if (!stack) return;
  if (open) positionProfileSwitcher(anchor);
  else stack.classList.add('hidden');
}

document.getElementById('btn-sidebar-profile').addEventListener('click', (event) => {
  event.stopPropagation();
  const stack = document.getElementById('profile-switcher-stack');
  setProfileSwitcherOpen(stack.classList.contains('hidden'), event.currentTarget);
});

document.addEventListener('click', (event) => {
  const stack = document.getElementById('profile-switcher-stack');
  if (!stack || stack.classList.contains('hidden')) return;
  if (stack.contains(event.target) || event.target.closest?.('#btn-sidebar-profile, #btn-topnav-profile')) return;
  stack.classList.add('hidden');
});

document.addEventListener('keydown', (event) => {
  if (event.key === 'Escape') document.getElementById('profile-switcher-stack')?.classList.add('hidden');
});

window.addEventListener('resize', () => {
  document.getElementById('profile-switcher-stack')?.classList.add('hidden');
});

document.getElementById('btn-topnav-profile').addEventListener('click', (event) => {
  event.stopPropagation();
  if (!authState?.profiles?.length) return;
  // Quick switch popup (like NuvioMobile-Enhanced's profile tab); the full
  // picker stays available from Ajustes → Trocar perfil.
  setProfileSwitcherOpen(true, event.currentTarget);
});

document.querySelectorAll('.sidebar-item, .topnav-item').forEach((btn) => {
  btn.addEventListener('click', () => setTab(btn.dataset.tab));
});

// --- WATCHED & LIBRARY STATE ---
let mdblistLibraryCache = { lists: [], items: [], error: null };
let mdblistProgressCache = { watchedKeys: [], cards: [], error: null };
let mdblistProgressTask = null;
let mdblistScrobbleQueue = Promise.resolve();

function activeLibrarySource() {
  const src = appSettings?.library_source;
  if (src === 'mdblist') return 'mdblist';
  if (src === 'simkl') return simklAccount?.connected ? 'simkl' : 'default';
  if (src === 'default') return 'default';
  if (simklAccount?.connected && simklAccount?.sources?.library === 'simkl') return 'simkl';
  return 'default';
}

function activeProgressSource() {
  const src = appSettings?.progress_source;
  if (src === 'mdblist') return 'mdblist';
  if (src === 'simkl') return simklAccount?.connected ? 'simkl' : 'default';
  if (src === 'default') return 'default';
  if (simklAccount?.connected && simklAccount?.sources?.progress === 'simkl') return 'simkl';
  return 'default';
}

function mdblistMatchesCard(card, id) {
  if (!card || !id) return false;
  if (card.content_id === id) return true;
  const ids = card.media_ids || {};
  if (/^tt\d+$/.test(id)) return String(ids.imdb || '') === id;
  const parts = /^(imdb|tmdb|tvdb|trakt):(.+)$/.exec(id);
  return parts ? String(ids[parts[1]] || '') === parts[2] : false;
}

function clearMdblistState() {
  mdblistLibraryCache = { lists: [], items: [], error: null };
  mdblistProgressCache = { watchedKeys: [], cards: [], error: null };
  mdblistProgressTask = null;
}

function refreshMdblistProgress(force = false) {
  if (mdblistProgressTask && !force) return mdblistProgressTask;
  const task = invoke('mdblist_progress_sync', { force: Boolean(force) })
    .then((snap) => {
      mdblistProgressCache = {
        watchedKeys: Array.isArray(snap?.watched_keys) ? snap.watched_keys : [],
        cards: Array.isArray(snap?.continue_watching) ? snap.continue_watching : [],
        error: null,
      };
      return mdblistProgressCache;
    })
    .catch((e) => {
      mdblistProgressCache = { watchedKeys: [], cards: [], error: String(e) };
      throw e;
    })
    .finally(() => {
      if (mdblistProgressTask === task) mdblistProgressTask = null;
    });
  mdblistProgressTask = task;
  return task;
}

function mdblistSendScrobble(ctx, st, stop = false) {
  if (activeProgressSource() !== 'mdblist' || !ctx?.contentId || !st || st.loading || !(st.duration_secs > 10)) return;
  if (ctx.mediaType !== 'movie' && (ctx.season == null || ctx.episode == null)) return;
  const action = stop || st.ended ? 'stop' : st.paused ? 'pause' : 'start';
  const percent = Math.min(100, Math.max(0, (100 * st.position_secs) / st.duration_secs));
  if (ctx.mdblistLastAction === 'stop') return mdblistScrobbleQueue;
  if (ctx.mdblistLastAction === action && (action !== 'start' || Date.now() - (ctx.mdblistAt || 0) < 60000)) {
    return;
  }
  ctx.mdblistAt = Date.now();
  ctx.mdblistLastAction = action;
  mdblistScrobbleQueue = mdblistScrobbleQueue.then(async () => {
    if (activeProgressSource() !== 'mdblist') return;
    await invoke('mdblist_scrobble', {
      action,
      mediaType: ctx.mediaType === 'movie' ? 'movie' : 'series',
      id: ctx.contentId,
      season: ctx.season ?? null,
      episode: ctx.episode ?? null,
      progress: percent,
    });
    if (action === 'stop') {
      await refreshMdblistProgress(true).catch(() => {});
      await Promise.all([refreshWatchedKeys(), refreshContinueWatching()]);
      refreshPosterWatchedBadges();
    }
  }).catch((e) => {
    if (ctx.mdblistLastAction === action) ctx.mdblistLastAction = null;
    if (!ctx.mdblistErrorShown) {
      ctx.mdblistErrorShown = true;
      showToast('Falha ao enviar reprodução para o MDBList. ' + String(e));
    }
  });
  return mdblistScrobbleQueue;
}

async function refreshWatchedKeys() {
  try {
    const source = activeProgressSource();
    if (source === 'mdblist') {
      await refreshMdblistProgress(false);
      watchedSet = new Set(mdblistProgressCache.watchedKeys || []);
      return;
    }
    const keys = source === 'simkl' ? simklWatchedKeys() : await invoke('list_watched_keys');
    watchedSet = new Set(keys || []);
  } catch {
    // Non-fatal
  }
}

async function refreshLibraryState(force = false) {
  try {
    const source = activeLibrarySource();
    if (source === 'mdblist') {
      const snap = await invoke('mdblist_library_sync', { force: Boolean(force) });
      mdblistLibraryCache = {
        lists: Array.isArray(snap?.lists) ? snap.lists : [],
        items: Array.isArray(snap?.items) ? snap.items : [],
        error: null,
      };
      libraryRawItems = mdblistLibraryCache.items;
      librarySet = new Set(libraryRawItems.map((it) => libKey(it.content_id || it.id, it.content_type || it.type)));
      for (const item of libraryRawItems) {
        for (const [k, v] of Object.entries(item.media_ids || {})) {
          if (v && k !== 'slug') {
            librarySet.add(libKey(k === 'imdb' ? String(v) : `${k}:${v}`, item.content_type));
          }
        }
      }
      return;
    }

    mdblistLibraryCache = { lists: [], items: [], error: null };
    const items = source === 'simkl' ? simklLibrary() : await invoke('pull_account_library');
    libraryRawItems = Array.isArray(items) ? items : [];
    librarySet = new Set(libraryRawItems.map((it) => libKey(it.content_id || it.id, it.content_type || it.type)));
    if (source === 'simkl') {
      for (const item of libraryRawItems) {
        for (const [k,id] of Object.entries(item.media.ids || {})) {
          if (id && k !== 'slug') librarySet.add(libKey(k === 'imdb' ? id : k + ':' + id, item.content_type));
        }
      }
    }
  } catch (e) {
    mdblistLibraryCache = { lists: [], items: [], error: String(e) };
    libraryRawItems = [];
    librarySet = new Set();
  }
}

async function toggleItemLibrary(meta) {
  const payload = {
    content_id: meta.id,
    content_type: meta.type || 'movie',
    name: meta.name || meta.title || 'Sem título',
    poster: meta.poster || null,
    background: meta.banner || meta.background || null,
    logo: meta.logo || null,
    description: meta.description || null,
    release_info: meta.release_info || meta.releaseInfo || null,
    imdb_rating: meta.imdb_rating || meta.imdbRating || null,
    added_at: Date.now(),
  };
  let saved;
  const source = activeLibrarySource();
  if (source === 'mdblist') {
    saved = !librarySet.has(libKey(meta.id, meta.type));
    const listSelect = document.getElementById('library-mdblist-list');
    const selectedList = listSelect && !listSelect.classList.contains('hidden') ? listSelect.value : 'watchlist';
    const targetList = selectedList === 'all' ? 'watchlist' : selectedList;
    try {
      await invoke('mdblist_library_mutate', {
        listKey: targetList,
        action: saved ? 'add' : 'remove',
        mediaType: meta.type || 'movie',
        id: meta.id,
      });
    } catch (e) {
      showToast(String(e));
      return !saved;
    }
    await refreshLibraryState(true);
    showToast(saved ? 'Adicionado ao MDBList' : 'Removido do MDBList');
    return saved;
  }
  if (source === 'simkl') {
    saved = !librarySet.has(libKey(meta.id,meta.type));
    if (!saved && !(await confirmDialog('Remover do Simkl?', 'O Simkl também removerá o histórico deste título ao retirá-lo da sua lista.'))) return true;
    try { await simklMutate(saved ? 'library_add' : 'remove', meta.id, meta.type, saved ? {to:'plantowatch'} : {}); }
    catch(e) {showToast(String(e)); return !saved;}
  } else {
    saved = await invoke('toggle_library_item', { item: payload });
  }
  await refreshLibraryState();
  showToast(saved ? 'Adicionado à Biblioteca' : 'Removido da Biblioteca');
  return saved;
}

async function toggleItemWatched(contentId, contentType, title, season = null, episode = null) {
  const progressSource = activeProgressSource();
  if (progressSource === 'mdblist') {
    const key = epWatchKey(contentId, season, episode);
    const watched = !watchedSet.has(key);
    try {
      await invoke('mdblist_watched_mutate', {
        remove: !watched,
        mediaType: contentType === 'movie' ? 'movie' : 'series',
        id: contentId,
        season: season ?? null,
        episode: episode ?? null,
      });
      await refreshMdblistProgress(true);
      await refreshWatchedKeys();
      showToast(watched ? 'Marcado como assistido no MDBList' : 'Desmarcado como assistido no MDBList');
      return watched;
    } catch (e) {
      showToast(String(e));
      return !watched;
    }
  }
  if (progressSource === 'simkl') {
    const key = epWatchKey(contentId,season,episode);
    const watched = !watchedSet.has(key);
    const fields = season != null && episode != null
      ? {seasons:[{number:season, episodes:[{number:episode}]}], ...(simklFindEntry(contentId)?.bucket === 'anime' ? {use_tvdb_anime_seasons:true} : {})}
      : watched && contentType === 'series' ? {status:'completed'} : {};
    try {
      await simklMutate(watched ? 'history_add' : 'history_remove', contentId,contentType,fields);
      await refreshWatchedKeys();
      return watched;
    } catch(e) {showToast(String(e)); return !watched;}
  }
  const watched = await invoke('toggle_watched_item', {
    contentId,
    contentType: contentType || 'movie',
    title: title || contentId,
    season,
    episode,
  });
  const key = epWatchKey(contentId, season, episode);
  if (watched) watchedSet.add(key);
  else watchedSet.delete(key);
  showToast(watched ? 'Marcado como assistido' : 'Desmarcado como assistido');
  return watched;
}

// --- HOVER PREVIEW & RIGHT-CLICK CONTEXT MENU ---
function hideHoverPreview() {
  clearTimeout(hoverPreviewTimer);
  hoverPreviewTimer = null;
  hoverPreviewItem = null;
  document.getElementById('hover-preview-popup')?.classList.add('hidden');
}

function hoverTypeLabel(mediaType) {
  const t = String(mediaType || '').toLowerCase();
  if (t === 'series' || t === 'tv') return 'Series';
  if (t === 'anime') return 'Anime';
  if (t === 'movie') return 'Filme';
  return t ? t.charAt(0).toUpperCase() + t.slice(1) : '';
}

function renderHoverPreviewContent(popup, item, meta) {
  const m = meta || {};
  const inLib = librarySet.has(libKey(item.id, item.type));
  const isWatched = watchedSet.has(item.id);
  const horizontalArtwork = m.background || m.banner || m.landscapePoster || item.banner || '';
  const artwork = horizontalArtwork || item.poster || m.poster || '';
  const isPosterFallback = !horizontalArtwork && Boolean(artwork);
  const logo = m.logo || item.logo || '';
  const title = m.name || item.name || '';
  const release = m.releaseInfo || item.release_info || '';
  const imdb = m.imdbRating || item.imdb_rating || '';
  const genres = Array.isArray(m.genres) && m.genres.length
    ? m.genres
    : (Array.isArray(item.genres) ? item.genres : []);
  const desc = m.description || item.description || '';
  const metaParts = [
    hoverTypeLabel(m.type || item.type),
    release,
    imdb ? `IMDb ${imdb}` : '',
  ].filter(Boolean);

  const icoCheck = '<svg viewBox="0 0 24 24" width="16" height="16"><path fill="currentColor" d="m9 16.2-3.5-3.5 1.4-1.4L9 13.4l7.1-7.1 1.4 1.4z"/></svg>';
  const icoPlus = '<svg viewBox="0 0 24 24" width="17" height="17"><path fill="currentColor" d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6z"/></svg>';
  const icoArrow = '<svg viewBox="0 0 24 24" width="15" height="15"><path fill="currentColor" d="M12 4l-1.41 1.41L16.17 11H4v2h12.17l-5.58 5.59L12 20l8-8z"/></svg>';

  popup.innerHTML = `
    <div class="hover-preview-media">
      ${artwork ? `<img class="hover-preview-bg ${isPosterFallback ? 'is-poster-fallback' : ''}" src="${esc(artwork)}" alt="" />` : ''}
      <div class="hover-preview-media-scrim"></div>
      ${isWatched ? `<span class="hover-preview-watched-pill" title="Assistido">${icoCheck}</span>` : ''}
      ${logo ? `<img class="hover-preview-logo" src="${esc(logo)}" alt="${esc(title)}" />` : `<div class="hover-preview-media-title">${esc(title)}</div>`}
    </div>
    <div class="hover-preview-body">
      ${metaParts.length ? `<div class="hover-preview-meta-line">${esc(metaParts.join(' • '))}</div>` : ''}
      ${genres.length ? `<div class="hover-preview-genres">${esc(genres.slice(0, 3).join(' • '))}</div>` : ''}
      ${desc ? `<div class="hover-preview-desc">${esc(desc)}</div>` : ''}
      <div class="hover-preview-actions">
        <button class="btn-accent hover-preview-primary-btn" id="hp-open-btn" type="button"><span>Ver detalhes</span>${icoArrow}</button>
        <button class="hover-preview-icon-btn ${isWatched ? 'is-active' : ''}" id="hp-watch-btn" type="button" title="${isWatched ? 'Desmarcar assistido' : 'Marcar como assistido'}" aria-label="Marcar como assistido">${icoCheck}</button>
        <button class="hover-preview-icon-btn ${inLib ? 'is-active' : ''}" id="hp-lib-btn" type="button" title="${inLib ? 'Remover da biblioteca' : 'Adicionar à biblioteca'}" aria-label="Biblioteca">${inLib ? icoCheck : icoPlus}</button>
      </div>
    </div>
  `;

  document.getElementById('hp-open-btn').onclick = () => {
    hideHoverPreview();
    openDetailsPage(item);
  };
  document.getElementById('hp-watch-btn').onclick = async (e) => {
    const btn = e.currentTarget;
    const watched = await toggleItemWatched(item.id, item.type, title);
    btn.classList.toggle('is-active', watched);
    btn.title = watched ? 'Desmarcar assistido' : 'Marcar como assistido';
    const mediaEl = popup.querySelector('.hover-preview-media');
    const existingPill = mediaEl?.querySelector('.hover-preview-watched-pill');
    if (watched && mediaEl && !existingPill) {
      mediaEl.insertAdjacentHTML('beforeend', `<span class="hover-preview-watched-pill" title="Assistido">${icoCheck}</span>`);
    } else if (!watched && existingPill) {
      existingPill.remove();
    }
  };
  document.getElementById('hp-lib-btn').onclick = async (e) => {
    const btn = e.currentTarget;
    const saved = await toggleItemLibrary(item);
    btn.classList.toggle('is-active', saved);
    btn.innerHTML = saved ? icoCheck : icoPlus;
    btn.title = saved ? 'Remover da biblioteca' : 'Adicionar à biblioteca';
  };
}

function showHoverPreviewForCard(cardEl, item) {
  if (!appSettings?.hover_preview_enabled) return;
  const popup = document.getElementById('hover-preview-popup');
  const rect = cardEl.getBoundingClientRect();
  const pw = 400;
  let left = rect.left + rect.width / 2 - pw / 2;
  left = Math.max(18, Math.min(window.innerWidth - pw - 18, left));
  let top = rect.top - 18;
  if (top + 350 > window.innerHeight - 16) {
    top = Math.max(16, window.innerHeight - 366);
  }

  popup.style.left = `${left}px`;
  popup.style.top = `${top}px`;

  const cachedMeta = getCachedMetaDetails(item.type, item.id);
  renderHoverPreviewContent(popup, item, cachedMeta);
  popup.classList.remove('hidden');

  if (!cachedMeta && item.id && item.type) {
    fetchMetaDetailsCached(item.type, item.id)
      .then((meta) => {
        if (meta && hoverPreviewItem === item && !popup.classList.contains('hidden')) {
          renderHoverPreviewContent(popup, item, meta);
        }
      })
      .catch(() => {});
  }
}

const hoverPopupEl = document.getElementById('hover-preview-popup');
hoverPopupEl.addEventListener('mouseleave', hideHoverPreview);

function closeContextMenu() {
  document.getElementById('poster-context-overlay')?.classList.add('hidden');
}

document.getElementById('poster-context-overlay').addEventListener('click', (e) => {
  if (e.target.id === 'poster-context-overlay') closeContextMenu();
});

function openPosterContextMenu(item, extraActions = []) {
  hideHoverPreview();
  const overlay = document.getElementById('poster-context-overlay');
  const card = document.getElementById('poster-context-card');
  const inLib = librarySet.has(libKey(item.id, item.type));
  const isWatched = watchedSet.has(item.id);
  const artUrl = item.poster || item.banner || '';
  const isLandscape = !item.poster && !!item.banner;

  const icoInfo = '<svg viewBox="0 0 24 24" width="18" height="18"><path fill="currentColor" d="M11 7h2v2h-2zm0 4h2v6h-2zm1-9C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2"/></svg>';
  const icoPlay = '<svg viewBox="0 0 24 24" width="18" height="18"><path fill="currentColor" d="M8 5v14l11-7z"/></svg>';
  const icoLib = inLib
    ? '<svg viewBox="0 0 24 24" width="18" height="18"><path fill="currentColor" d="m9 16.2-3.5-3.5 1.4-1.4L9 13.4l7.1-7.1 1.4 1.4z"/></svg>'
    : '<svg viewBox="0 0 24 24" width="18" height="18"><path fill="currentColor" d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6z"/></svg>';
  const icoWatch = isWatched
    ? '<svg viewBox="0 0 24 24" width="18" height="18"><path fill="currentColor" d="M12 4.5C7 4.5 2.73 7.61 1 12c1.73 4.39 6 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6-7.5-11-7.5M12 17a5 5 0 1 1 0-10 5 5 0 0 1 0 10"/></svg>'
    : '<svg viewBox="0 0 24 24" width="18" height="18"><path fill="currentColor" d="M12 6.5c3.79 0 7.17 2.13 8.82 5.5-1.65 3.37-5.03 5.5-8.82 5.5S4.83 15.37 3.18 12C4.83 8.63 8.21 6.5 12 6.5m0-2C7 4.5 2.73 7.61 1 12c1.73 4.39 6 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6-7.5-11-7.5"/></svg>';
  const icoAct = '<svg viewBox="0 0 24 24" width="18" height="18"><path fill="currentColor" d="M19 6.41 17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/></svg>';

  card.innerHTML = `
    ${artUrl ? `<div class="context-zoom-poster-wrap ${isLandscape ? 'landscape' : ''}"><img src="${esc(artUrl)}" alt="" draggable="false" /></div>` : ''}
    <div class="context-zoom-meta">
      <div class="context-zoom-title">${esc(item.name || item.title)}</div>
      ${item.release_info || item.type ? `<div class="context-zoom-sub">${esc(item.release_info || item.type)}</div>` : ''}
    </div>
    <div class="context-zoom-menu">
      <button class="context-action-item" data-ctx-act="details">${icoInfo}<span>Abrir Detalhes</span></button>
      <button class="context-action-item" data-ctx-act="streams">${icoPlay}<span>Buscar Streams</span></button>
      <button class="context-action-item" data-ctx-act="library">${icoLib}<span>${inLib ? 'Remover da Biblioteca' : 'Salvar na Biblioteca'}</span></button>
      <button class="context-action-item" data-ctx-act="watched">${icoWatch}<span>${isWatched ? 'Desmarcar como Assistido' : 'Marcar como Assistido'}</span></button>
      ${extraActions.map((act, i) => `<button class="context-action-item ${act.danger ? 'danger' : ''}" data-extra-idx="${i}">${icoAct}<span>${esc(act.label)}</span></button>`).join('')}
    </div>
  `;

  overlay.classList.remove('hidden');

  card.querySelector('[data-ctx-act="details"]').onclick = () => {
    closeContextMenu();
    openDetailsPage(item);
  };
  card.querySelector('[data-ctx-act="streams"]').onclick = () => {
    closeContextMenu();
    openStreamsScreen({
      mediaType: item.type || 'movie',
      videoId: item.video_id || item.id,
      contentId: item.id,
      title: item.name || item.title,
      logo: item.logo || '',
      background: item.banner || item.poster || '',
      poster: item.poster || '',
      episode: item.episode || null,
      resumePositionSecs: item.resumePositionSecs || 0,
    });
  };
  card.querySelector('[data-ctx-act="library"]').onclick = async () => {
    closeContextMenu();
    await toggleItemLibrary(item);
    if (activeTabName() === 'library') refreshLibrary();
  };
  card.querySelector('[data-ctx-act="watched"]').onclick = async () => {
    closeContextMenu();
    await toggleItemWatched(item.id, item.type, item.name || item.title, null, null);
    refreshPosterWatchedBadges();
  };
  card.querySelectorAll('[data-extra-idx]').forEach((btn) => {
    btn.onclick = () => {
      closeContextMenu();
      extraActions[Number(btn.dataset.extraIdx)]?.onClick?.();
    };
  });
}

function refreshPosterWatchedBadges() {
  document.querySelectorAll('.poster-card').forEach((card) => {
    const wrap = card.querySelector('.poster-img-wrap');
    if (!wrap) return;
    const isWatched = watchedSet.has(card.dataset.metaId);
    const existing = wrap.querySelector('.watched-badge');
    if (isWatched && !existing) {
      wrap.insertAdjacentHTML('beforeend', '<span class="watched-badge" title="Assistido">&#10003;</span>');
    } else if (!isWatched && existing) {
      existing.remove();
    }
  });
}

// --- CONTINUE WATCHING ---
async function refreshContinueWatching() {
  if (cwUnlistenItem) { cwUnlistenItem(); cwUnlistenItem = null; }
  cwItemsMap.clear();
  const wrap = document.getElementById('home-cw-container');
  wrap.classList.add('hidden');
  wrap.innerHTML = '';

  const progressSource = activeProgressSource();
  if (progressSource === 'mdblist') {
    try {
      await refreshMdblistProgress(false);
      (mdblistProgressCache.cards || []).forEach((card, index) => cwItemsMap.set(index, card));
      renderContinueWatchingRow();
      return;
    } catch (error) {
      console.warn('[continue-watching] falha ao carregar MDBList:', error);
    }
  } else if (progressSource === 'simkl') {
    const cards = simklProgressCards();
    // Simkl escolhido como fonte, mas sem snapshot sincronizado (conta
    // desconectada/token expirado): cai para o progresso do Nuvio.
    if (cards.length || simklAccount.snapshot) {
      cards.forEach((card, index) => cwItemsMap.set(index, card));
      renderContinueWatchingRow();
      return;
    }
  }
  cwUnlistenItem = await listen('home:continue-watching-item', (event) => {
    const { order, card } = event.payload;
    cwItemsMap.set(order, card);
    renderContinueWatchingRow();
  });

  try {
    const ack = await invoke('start_continue_watching');
    if (ack && !ack.total) console.info('[continue-watching] nenhum item em progresso retornado');
  } catch (error) {
    console.warn('[continue-watching] falha ao carregar o progresso:', error);
  }
}

function renderContinueWatchingRow() {
  const wrap = document.getElementById('home-cw-container');
  cwCardsCache = Array.from(cwItemsMap.entries())
    .sort((a, b) => a[0] - b[0])
    .map(([, card]) => card);

  if (!cwCardsCache.length) {
    wrap.classList.add('hidden');
    wrap.innerHTML = '';
    return;
  }

  const style = appSettings?.cw_style || 'card';
  const styleClass = style === 'wide' ? 'cw-wide' : (style === 'poster' ? 'cw-poster' : '');

  wrap.classList.remove('hidden');
  wrap.innerHTML = `
    <div class="shelf-header">
      <div class="shelf-title-btn" style="cursor:default;">Continuar Assistindo</div>
    </div>
    <div class="shelf-row">
      ${cwCardsCache.map((c, idx) => {
        const subParts = [];
        if (c.season != null && c.episode != null) {
          subParts.push(`T${c.season} E${c.episode}`);
        } else {
          subParts.push(c.content_type === 'series' ? 'Série' : 'Filme');
        }
        if (c.episode_title) subParts.push(c.episode_title);
        // Arte horizontal sempre que existir: o poster vertical dentro de um cartão
        // 16:9 fica cortado e esticado.
        const horizontalArt = (c.artwork && c.artwork !== c.poster ? c.artwork : '') || c.background || '';
        const rawImg = style === 'poster'
          ? (c.poster || c.artwork || '')
          : (horizontalArt || c.poster || c.artwork || '');
        const imgUrl = optimizePosterUrl(rawImg, style === 'poster' ? 342 : 500);
        // A arte do cartão é o episódio que estás a retomar (não é spoiler) e
        // deve aparecer nítida mesmo com "Desfocar miniaturas" ligado.
        return `
          <button class="cw-card ${styleClass}" data-cw-idx="${idx}">
            ${imgUrl ? `<img src="${esc(imgUrl)}" alt="${esc(c.title)}" loading="lazy" decoding="async" draggable="false" />` : ''}
            <div class="cw-scrim"></div>
            ${c.remaining_minutes ? `<span class="cw-badge-top">Faltam ${c.remaining_minutes}m</span>` : ''}
            <div class="cw-meta">
              <div class="cw-title">${esc(c.title)}</div>
              <div class="cw-sub">${esc(subParts.join(' • '))}</div>
            </div>
            <div class="cw-progress-track">
              <div class="cw-progress-fill" style="width:${c.progress_percent}%"></div>
            </div>
          </button>
        `;
      }).join('')}
    </div>
  `;

  wireHorizontalShelfDrag(wrap);
  wrap.querySelectorAll('[data-cw-idx]').forEach((btn) => {
    const idx = Number(btn.dataset.cwIdx);
    const c = cwCardsCache[idx];
    const openCw = () => openStreamsScreen({
      mediaType: c.content_type,
      videoId: c.video_id || c.content_id,
      contentId: c.content_id,
      title: c.title,
      logo: c.logo || '',
      background: c.background || c.artwork || '',
      poster: c.poster || '',
      resumePositionSecs: Math.max(0, (c.position_ms || 0) / 1000),
      episode: c.season != null && c.episode != null ? {
        season: c.season,
        episode: c.episode,
        title: c.episode_title || '',
        thumbnail: c.artwork || '',
      } : null,
    });

    btn.addEventListener('click', () => {
      const shelfRow = btn.closest('.shelf-row');
      if (shelfRow?.dataset.suppressClick) return;
      openCw();
    });

    btn.addEventListener('contextmenu', (e) => {
      e.preventDefault();
      const progressKey = c.season != null && c.episode != null
        ? `${c.content_id}_s${c.season}e${c.episode}`
        : c.content_id;
      openPosterContextMenu({
        id: c.content_id,
        video_id: c.video_id,
        type: c.content_type,
        name: c.title,
        poster: c.poster || '',
        banner: c.background || c.artwork || '',
        logo: c.logo || '',
        resumePositionSecs: Math.max(0, (c.position_ms || 0) / 1000),
        episode: c.season != null ? { season: c.season, episode: c.episode, title: c.episode_title || '' } : null,
      }, [
        {
          label: 'Remover de Continuar Assistindo',
          danger: true,
          onClick: async () => {
            const progressSource = activeProgressSource();
            if (progressSource === 'mdblist') {
              try {
                await invoke('mdblist_scrobble', {
                  action: 'clear',
                  mediaType: c.content_type === 'movie' ? 'movie' : 'series',
                  id: c.content_id,
                  season: c.season ?? null,
                  episode: c.episode ?? null,
                  progress: null,
                });
                await refreshMdblistProgress(true);
                await refreshContinueWatching();
              } catch (e) { showToast(String(e)); }
              return;
            }
            if (progressSource === 'simkl') {
              try {await simklRequest('remove_progress', c.simkl_session); await simklSync(); await refreshContinueWatching();}
              catch(e) {showToast(String(e));}
              return;
            }
            await invoke('remove_watch_progress', { progressKey });
            cwItemsMap.forEach((val, key) => {
              if (val.content_id === c.content_id) cwItemsMap.delete(key);
            });
            renderContinueWatchingRow();
            showToast('Removido de Continuar Assistindo');
          },
        },
      ]);
    });
  });
}

// --- HOME PROGRESSIVE LOADING ---
// Tempos do arranque vão para o crash.log via comando `frontend_log`.
function logUiTiming(message) {
  invoke('frontend_log', { message }).catch(() => {});
}

let homeRunGen = 0;

function catalogSectionDigest(sec) {
  return `${sec.key}|${sec.title}|${(sec.items || []).map((i) => `${i.id}:${i.poster || ''}`).join(',')}`;
}

async function refreshHomeProgressive(force = false) {
  const gen = ++homeRunGen;
  const homeStartedAt = performance.now();
  const sinceHome = () => Math.round(performance.now() - homeStartedAt);
  logUiTiming(`home: inicio (pagina aberta ha ${Math.round(homeStartedAt)} ms)`);
  if (homeUnlistenSection) { homeUnlistenSection(); homeUnlistenSection = null; }
  if (homeUnlistenDone) { homeUnlistenDone(); homeUnlistenDone = null; }

  const heroContainer = document.getElementById('home-hero-container');
  const shelvesContainer = document.getElementById('home-catalogs-container');
  const hadSections = homeSectionsMap.size > 0 && !force;
  let heroLogged = false;
  const logHeroOnce = () => {
    if (!heroLogged) {
      heroLogged = true;
      logUiTiming(`home: hero em ${sinceHome()} ms`);
    }
  };

  if (!hadSections) {
    homeSectionsMap.clear();
    heroItems = [];
    heroIdx = 0;
    const localSnap = !force ? readHomeSnapshot() : null;
    if (localSnap) {
      renderHomeSectionsChunked(shelvesContainer, localSnap, gen, logHeroOnce);
    } else {
      shelvesContainer.innerHTML = '<div class="muted" id="home-loading-note">Carregando catálogos…</div>';
    }
  }
  if (typeof refreshHomeCollections === 'function') refreshHomeCollections();

  homeUnlistenSection = await listen('home:section', (event) => {
    if (gen !== homeRunGen) return;
    const sec = event.payload;
    document.getElementById('home-loading-note')?.remove();
    const prev = homeSectionsMap.get(sec.order);
    const sameContent = prev && catalogSectionDigest(prev) === catalogSectionDigest(sec);
    homeSectionsMap.set(sec.order, sec);

    if (heroItems.length === 0) {
      const candidates = sec.items.filter((i) => i.banner || i.poster).slice(0, 8);
      if (candidates.length) {
        heroItems = candidates;
        renderHero();
        logHeroOnce();
      }
    }

    if (!sameContent || !shelvesContainer.querySelector(`.shelf-section[data-order="${sec.order}"]`)) {
      appendShelfOrdered(shelvesContainer, sec);
    }
  });

  homeUnlistenDone = await listen('home:done', () => {
    if (gen !== homeRunGen) return;
    document.getElementById('home-loading-note')?.remove();
    logUiTiming(`home: pronto em ${sinceHome()} ms (${homeSectionsMap.size} secoes)`);
    if (homeSectionsMap.size === 0) {
      renderEmptyHome(heroContainer, shelvesContainer);
    } else {
      writeHomeSnapshot();
    }
  });

  const ack = await invoke('start_home_catalogs', { force: Boolean(force) });
  if (gen !== homeRunGen) return;
  if (ack.cached_sections?.length) {
    renderHomeSectionsChunked(shelvesContainer, ack.cached_sections, gen, logHeroOnce);
    writeHomeSnapshot();
  }
  logUiTiming(`home: pedido aceite em ${sinceHome()} ms (${ack.total} catalogos)`);
  if (ack.total === 0) {
    renderEmptyHome(heroContainer, shelvesContainer);
  }
}

function renderEmptyHome(heroContainer, shelvesContainer) {
  heroContainer.innerHTML = `
    <div class="hero-frame" style="min-height:320px;height:340px;">
      <div class="hero-scrim-left"></div>
      <div class="hero-scrim-bottom"></div>
      <div class="hero-content">
        <img src="assets/app_logo_wordmark.png" alt="Nuvio" class="hero-logo" style="max-height:48px;" />
        <h2 class="hero-title" style="font-size:28px;">Nenhum catálogo carregado ainda</h2>
        <p class="hero-desc">Entre na sua conta Nuvio ou instale um addon Stremio em Ajustes → Addons para preencher sua tela inicial.</p>
        <button class="btn-accent" id="btn-empty-go-addons">Abrir Addons</button>
      </div>
    </div>`;
  document.getElementById('btn-empty-go-addons')?.addEventListener('click', () => {
    setTab('settings');
    selectSettingsSection('addons');
  });
  shelvesContainer.innerHTML = '';
}

function appendShelfOrdered(shelvesContainer, sec, sectionsMap = homeSectionsMap) {
  const wrapper = document.createElement('div');
  wrapper.className = 'shelf-section';
  wrapper.dataset.order = String(sec.order);
  const previewItems = sec.searchQuery ? sec.items.slice(0, 20) : sec.items;
  wrapper.innerHTML = `
    <div class="shelf-header">
      <button class="shelf-title-btn" data-open-order="${sec.order}">
        <span>${esc(sec.title)}</span>
        <span class="muted" style="font-weight:400;font-size:13px;">• ${esc(sec.addon_name)}</span>
      </button>
      <button class="view-all-pill" data-open-order="${sec.order}" title="Ver catálogo completo">
        <svg viewBox="0 0 24 24" width="18" height="18"><path fill="currentColor" d="M8.59 16.59L13.17 12L8.59 7.41L10 6l6 6l-6 6z"/></svg>
      </button>
    </div>
    <div class="shelf-row">
      ${previewItems.map((item) => renderPosterCard(item)).join('')}
    </div>
  `;

  const existingWrapper = shelvesContainer.querySelector(`.shelf-section[data-order="${sec.order}"]`);
  if (existingWrapper) {
    existingWrapper.replaceWith(wrapper);
  } else {
    const existing = Array.from(shelvesContainer.children);
    const nextSibling = existing.find((el) => Number(el.dataset.order) > sec.order);
    if (nextSibling) {
      shelvesContainer.insertBefore(wrapper, nextSibling);
    } else {
      shelvesContainer.appendChild(wrapper);
    }
  }

  wrapper.querySelectorAll('[data-open-order]').forEach((btn) => {
    btn.addEventListener('click', () => {
      const target = sectionsMap.get(Number(btn.dataset.openOrder));
      if (target) openFullCatalog(target);
    });
  });

  wireHorizontalShelfDrag(wrapper);
  wirePosterInteractions(wrapper);
}

function wireHorizontalShelfDrag(container) {
  container.querySelectorAll('.shelf-row').forEach((row) => {
    if (row.dataset.dragWired) return;
    row.dataset.dragWired = '1';

    let tracking = false;
    let locked = null; // 'x' | 'y' | null
    let pointerId = -1;
    let startX = 0;
    let startY = 0;
    let lastX = 0;
    let lastTs = 0;
    let velocity = 0;
    let flingRaf = 0;
    let moveRaf = 0;
    let pendingDx = 0;

    const stopFling = () => {
      if (flingRaf) {
        cancelAnimationFrame(flingRaf);
        flingRaf = 0;
      }
      row.classList.remove('is-scrolling');
    };

    const startFling = () => {
      stopFling();
      if (Math.abs(velocity) < 0.4) return;
      row.classList.add('is-scrolling');
      let prev = performance.now();
      const maxScroll = Math.max(0, row.scrollWidth - row.clientWidth);
      const step = (now) => {
        const dt = Math.min(32, Math.max(4, now - prev));
        prev = now;
        row.scrollLeft += velocity * (dt / 16.67);
        velocity *= Math.pow(0.92, dt / 16.67);
        if (Math.abs(velocity) < 0.35 || row.scrollLeft <= 0 || row.scrollLeft >= maxScroll) {
          stopFling();
          return;
        }
        flingRaf = requestAnimationFrame(step);
      };
      flingRaf = requestAnimationFrame(step);
    };

    row.addEventListener('dragstart', (e) => e.preventDefault());

    row.addEventListener('pointerdown', (e) => {
      if (e.button !== 0) return;
      stopFling();
      if (moveRaf) {
        cancelAnimationFrame(moveRaf);
        moveRaf = 0;
      }
      pendingDx = 0;
      tracking = true;
      locked = null;
      pointerId = e.pointerId;
      startX = e.clientX;
      startY = e.clientY;
      lastX = e.clientX;
      lastTs = performance.now();
      velocity = 0;
    });

    row.addEventListener('pointermove', (e) => {
      if (!tracking || e.pointerId !== pointerId) return;
      const dxTotal = e.clientX - startX;
      const dyTotal = e.clientY - startY;

      if (!locked) {
        if (Math.abs(dyTotal) > 6 && Math.abs(dyTotal) > Math.abs(dxTotal)) {
          locked = 'y';
          tracking = false;
          return;
        }
        if (Math.abs(dxTotal) > 6 && Math.abs(dxTotal) >= Math.abs(dyTotal)) {
          locked = 'x';
          row.classList.add('dragging');
          try { row.setPointerCapture(pointerId); } catch {}
        } else {
          return;
        }
      }

      const now = performance.now();
      const stepDx = e.clientX - lastX;
      const dt = Math.max(1, now - lastTs);
      velocity = (-stepDx / dt) * 16.67 * 0.75 + velocity * 0.25;
      lastX = e.clientX;
      lastTs = now;

      pendingDx += stepDx;
      if (!moveRaf) {
        moveRaf = requestAnimationFrame(() => {
          row.scrollLeft -= pendingDx;
          pendingDx = 0;
          moveRaf = 0;
        });
      }
    });

    const endDrag = (e) => {
      if (!tracking && locked !== 'x') return;
      if (e.pointerId !== pointerId) return;
      const wasHoriz = locked === 'x';
      tracking = false;
      locked = null;
      row.classList.remove('dragging');
      if (moveRaf) {
        cancelAnimationFrame(moveRaf);
        moveRaf = 0;
        if (pendingDx !== 0) {
          row.scrollLeft -= pendingDx;
          pendingDx = 0;
        }
      }
      try {
        if (row.hasPointerCapture(pointerId)) row.releasePointerCapture(pointerId);
      } catch {}
      if (wasHoriz) {
        row.dataset.suppressClick = '1';
        setTimeout(() => delete row.dataset.suppressClick, 60);
        startFling();
      }
    };

    row.addEventListener('pointerup', endDrag);
    row.addEventListener('pointercancel', endDrag);

    row.addEventListener(
      'wheel',
      (e) => {
        if (Math.abs(e.deltaX) > 0) {
          stopFling();
          row.scrollLeft += e.deltaX;
        } else if (e.shiftKey && Math.abs(e.deltaY) > 0) {
          stopFling();
          e.preventDefault();
          row.scrollLeft += e.deltaY;
        }
      },
      { passive: false },
    );
  });
}

async function openFullCatalog(sec) {
  const catalogRef = {
    manifestUrl: sec.manifest_url,
    mediaType: sec.media_type,
    catalogId: sec.catalog_id,
    title: sec.title,
    addonName: sec.addon_name,
    genre: sec.genre || null,
    searchQuery: sec.searchQuery || null,
    items: [...(sec.items || [])],
  };
  activeCatalog = catalogRef;
  document.getElementById('catalog-page-title').textContent = sec.title;
  document.getElementById('catalog-page-sub').textContent = sec.addon_name;
  document.getElementById('catalog-full-status').textContent = '';
  setTab('catalog');
  renderFullCatalogGrid();
  document.getElementById('btn-catalog-load-more').classList.toggle('hidden', activeCatalog.items.length < 10);

  if (!sec.sources && !sec.searchQuery && sec.manifest_url && (sec.items?.length || 0) >= 20) {
    invoke('load_catalog_page', {
      manifestUrl: sec.manifest_url,
      mediaType: sec.media_type,
      catalogId: sec.catalog_id,
      skip: 0,
      genre: sec.genre || null,
      search: null,
    }).then((fullPage) => {
      if (activeCatalog !== catalogRef || !Array.isArray(fullPage) || fullPage.length <= catalogRef.items.length) return;
      const seen = new Set(catalogRef.items.map((x) => `${x.type}:${x.id}`));
      let added = false;
      for (const item of fullPage) {
        const k = `${item.type}:${item.id}`;
        if (!seen.has(k)) {
          seen.add(k);
          catalogRef.items.push(item);
          added = true;
        }
      }
      if (added) {
        renderFullCatalogGrid();
        document.getElementById('btn-catalog-load-more').classList.toggle('hidden', catalogRef.items.length < 10);
      }
    }).catch(() => {});
  }
}

function renderFullCatalogGrid() {
  const grid = document.getElementById('catalog-full-grid');
  grid.innerHTML = activeCatalog.items.map((it) => renderPosterCard(it)).join('');
  wirePosterInteractions(grid);
}

document.getElementById('btn-catalog-back').addEventListener('click', () => setTab(previousTab || 'home'));

document.getElementById('btn-catalog-load-more').addEventListener('click', async () => {
  if (!activeCatalog) return;
  if (activeCatalog.sources) {
    if (typeof loadMoreCollectionFolder === 'function') await loadMoreCollectionFolder();
    return;
  }
  const btn = document.getElementById('btn-catalog-load-more');
  const status = document.getElementById('catalog-full-status');
  btn.disabled = true;
  status.textContent = 'Carregando mais títulos…';
  try {
    const more = await invoke('load_catalog_page', {
      manifestUrl: activeCatalog.manifestUrl,
      mediaType: activeCatalog.mediaType,
      catalogId: activeCatalog.catalogId,
      skip: activeCatalog.items.length,
      genre: activeCatalog.genre,
      search: activeCatalog.searchQuery || null,
    });
    if (!more.length) {
      btn.classList.add('hidden');
      status.textContent = 'Fim do catálogo.';
    } else {
      const seen = new Set(activeCatalog.items.map((x) => `${x.type}:${x.id}`));
      for (const item of more) {
        if (!seen.has(`${item.type}:${item.id}`)) activeCatalog.items.push(item);
      }
      renderFullCatalogGrid();
      status.textContent = '';
    }
  } catch (e) {
    status.textContent = String(e);
  } finally {
    btn.disabled = false;
  }
});

function renderPosterCard(item) {
  const watched = watchedSet.has(item.id);
  const posterSrc = optimizePosterUrl(item.poster);
  return `
    <button class="poster-card" data-meta-id="${esc(item.id)}" data-meta-type="${esc(item.type)}" data-meta-name="${esc(item.name)}" data-meta-poster="${esc(item.poster || '')}" data-meta-banner="${esc(item.banner || '')}" data-meta-logo="${esc(item.logo || '')}" data-meta-desc="${esc(item.description || '')}" data-meta-release="${esc(item.release_info || '')}" data-meta-imdb="${esc(item.imdb_rating || '')}">
      <div class="poster-img-wrap">
        ${item.poster ? `<img src="${esc(posterSrc)}" alt="${esc(item.name)}" loading="lazy" decoding="async" draggable="false" />` : ''}
        ${watched ? '<span class="watched-badge" title="Assistido">&#10003;</span>' : ''}
      </div>
      <div class="poster-name">${esc(item.name)}</div>
      ${item.release_info ? `<div class="poster-sub">${esc(item.release_info)}</div>` : ''}
    </button>
  `;
}

const heroBackdropCache = new Map();
const heroBackdropPending = new Set();
const metaDetailsCache = new Map();
const metaDetailsPending = new Map();

function getCachedMetaDetails(mediaType, id) {
  if (!id) return null;
  return metaDetailsCache.get(`${mediaType || 'movie'}:${id}`) || null;
}

function cacheMetaDetails(mediaType, id, meta) {
  if (!id || !meta || typeof meta !== 'object') return;
  const key = `${mediaType || 'movie'}:${id}`;
  if (metaDetailsCache.has(key)) metaDetailsCache.delete(key);
  metaDetailsCache.set(key, meta);
  while (metaDetailsCache.size > 50) {
    const oldest = metaDetailsCache.keys().next().value;
    if (!oldest) break;
    metaDetailsCache.delete(oldest);
  }
}

function fetchMetaDetailsCached(mediaType, id) {
  const key = `${mediaType || 'movie'}:${id}`;
  const cached = metaDetailsCache.get(key);
  if (cached) return Promise.resolve(cached);
  if (metaDetailsPending.has(key)) return metaDetailsPending.get(key);
  const req = invoke('fetch_meta_details', { mediaType, id })
    .then((meta) => {
      cacheMetaDetails(mediaType, id, meta);
      return meta;
    })
    .finally(() => metaDetailsPending.delete(key));
  metaDetailsPending.set(key, req);
  return req;
}

function applyHeroBackdrop(itemId, url) {
  const cur = heroItems[heroIdx % (heroItems.length || 1)];
  if (!cur || cur.id !== itemId || !url) return;
  const frame = document.querySelector('#home-hero-container .hero-frame');
  if (!frame) return;
  let img = frame.querySelector('.hero-bg');
  if (!img) {
    img = document.createElement('img');
    img.className = 'hero-bg';
    img.alt = '';
    frame.insertBefore(img, frame.firstChild);
  }
  if (img.getAttribute('src') !== url) img.src = url;
}

function prefetchHeroBackdrops() {
  const fetchOne = (item) => {
    if (!item?.id || heroBackdropCache.has(item.id) || heroBackdropPending.has(item.id)) return;
    heroBackdropPending.add(item.id);
    fetchMetaDetailsCached(item.type, item.id)
      .then((meta) => {
        const url = meta?.background || meta?.banner || '';
        heroBackdropCache.set(item.id, url);
        applyHeroBackdrop(item.id, url);
      })
      .catch(() => heroBackdropCache.set(item.id, ''))
      .finally(() => heroBackdropPending.delete(item.id));
  };
  if (heroItems[0]) fetchOne(heroItems[0]);
  if (heroItems.length > 1) {
    setTimeout(() => {
      heroItems.slice(1).forEach(fetchOne);
    }, 350);
  }
}

function renderHero() {
  if (heroTimer) clearInterval(heroTimer);
  const container = document.getElementById('home-hero-container');
  if (!heroItems.length || appSettings?.show_hero === false) {
    container.innerHTML = '';
    return;
  }
  const draw = () => {
    const item = heroItems[heroIdx % heroItems.length];
    // Same horizontal backdrop the details screen uses; never stretch a vertical poster.
    const cachedBackdrop = heroBackdropCache.get(item.id);
    const bg = cachedBackdrop || item.banner || (cachedBackdrop === '' ? item.poster : '') || '';
    const genres = Array.isArray(item.genres) ? item.genres : [];
    container.innerHTML = `
      <div class="hero-frame">
        ${bg ? `<img class="hero-bg" src="${esc(bg)}" alt="" />` : ''}
        <div class="hero-scrim-top"></div>
        <div class="hero-scrim-left"></div>
        <div class="hero-scrim-bottom"></div>
        <div class="hero-circle-top-right">
          <button class="detail-action-circle-btn" id="btn-hero-open-circle" title="Ver detalhes">
            <svg viewBox="0 0 24 24" width="20" height="20"><path fill="currentColor" d="M8.59 16.59L13.17 12L8.59 7.41L10 6l6 6l-6 6z"/></svg>
          </button>
        </div>
        <div class="hero-content">
          ${item.logo ? `<img class="hero-logo" src="${esc(item.logo)}" alt="${esc(item.name)}" />` : `<h1 class="hero-title">${esc(item.name)}</h1>`}
          <div class="hero-meta-row">
            ${item.imdb_rating ? `<span class="imdb-chip"><img src="assets/rating_imdb.png" alt="IMDb" /> ${esc(item.imdb_rating)}</span>` : ''}
            ${item.release_info ? `<span>${esc(item.release_info)}</span>` : ''}
            ${genres.length ? `<span>${esc(genres.slice(0, 3).join(' • '))}</span>` : ''}
          </div>
          ${item.description ? `<p class="hero-desc">${esc(item.description)}</p>` : ''}
          <div class="hero-actions">
            <button class="btn-accent" id="btn-hero-details">Ver Detalhes</button>
          </div>
        </div>
        <div class="hero-dots">
          ${heroItems.map((_, idx) => `<button class="hero-dot ${idx === heroIdx ? 'active' : ''}" data-hero-dot="${idx}"></button>`).join('')}
        </div>
      </div>
    `;
    populateHeroRatings(container.querySelector('.hero-meta-row'), item);
    document.getElementById('btn-hero-details').addEventListener('click', () => openDetailsPage(item));
    document.getElementById('btn-hero-open-circle').addEventListener('click', () => openDetailsPage(item));
    container.querySelectorAll('[data-hero-dot]').forEach((dot) => {
      dot.addEventListener('click', () => {
        heroIdx = Number(dot.dataset.heroDot);
        draw();
      });
    });
  };
  draw();
  prefetchHeroBackdrops();
  heroTimer = setInterval(() => {
    heroIdx = (heroIdx + 1) % heroItems.length;
    draw();
  }, 8000);
}

function cardToItem(card) {
  return {
    id: card.dataset.metaId,
    type: card.dataset.metaType,
    name: card.dataset.metaName,
    poster: card.dataset.metaPoster,
    banner: card.dataset.metaBanner,
    logo: card.dataset.metaLogo,
    description: card.dataset.metaDesc,
    release_info: card.dataset.metaRelease,
    imdb_rating: card.dataset.metaImdb,
  };
}

let posterDelegationReady = false;
let hoverActiveCard = null;
function wirePosterInteractions(_root) {
  if (posterDelegationReady) return;
  posterDelegationReady = true;

  mainViewportEl.addEventListener('click', (e) => {
    if (e.target.closest('[data-unfollow-key]')) return;
    const castCard = e.target.closest('.cast-portrait-card[data-person-name], .following-person-card[data-person-name]');
    if (castCard) {
      hideHoverPreview();
      openPersonPage({
        id: castCard.dataset.personId,
        name: castCard.dataset.personName || '',
        photo: castCard.dataset.personPhoto || '',
      });
      return;
    }
    const card = e.target.closest('.poster-card');
    if (!card) return;
    const shelfRow = card.closest('.shelf-row');
    if (shelfRow?.dataset.suppressClick) return;
    hideHoverPreview();
    openDetailsPage(cardToItem(card));
  });

  mainViewportEl.addEventListener('keydown', (e) => {
    if (e.key !== 'Enter' && e.key !== ' ') return;
    if (e.target.closest('[data-unfollow-key]')) return;
    const castCard = e.target.closest('.cast-portrait-card[data-person-name], .following-person-card[data-person-name]');
    if (!castCard) return;
    e.preventDefault();
    openPersonPage({
      id: castCard.dataset.personId,
      name: castCard.dataset.personName || '',
      photo: castCard.dataset.personPhoto || '',
    });
  });

  mainViewportEl.addEventListener('contextmenu', (e) => {
    const card = e.target.closest('.poster-card:not(.following-person-card)');
    if (!card) return;
    e.preventDefault();
    openPosterContextMenu(cardToItem(card));
  });

  let hoverPrefetchTimer = null;
  mainViewportEl.addEventListener('pointerover', (e) => {
    const card = e.target.closest('.poster-card:not(.following-person-card)');
    if (card?.dataset.metaId && card?.dataset.metaType) {
      clearTimeout(hoverPrefetchTimer);
      const { metaType, metaId } = card.dataset;
      hoverPrefetchTimer = setTimeout(() => {
        fetchMetaDetailsCached(metaType, metaId).catch(() => {});
      }, 180);
    }
    if (!appSettings?.hover_preview_enabled) return;
    if (!card || card === hoverActiveCard) return;
    const shelfRow = card.closest('.shelf-row');
    if (shelfRow && (shelfRow.classList.contains('dragging') || shelfRow.classList.contains('is-scrolling'))) return;
    hoverActiveCard = card;
    clearTimeout(hoverPreviewTimer);
    const item = cardToItem(card);
    hoverPreviewItem = item;
    hoverPreviewTimer = setTimeout(() => {
      if (hoverPreviewItem === item && hoverActiveCard === card) {
        showHoverPreviewForCard(card, item);
      }
    }, 680);
  });

  mainViewportEl.addEventListener('pointerout', (e) => {
    const card = e.target.closest('.poster-card');
    if (!card || (e.relatedTarget && card.contains(e.relatedTarget))) return;
    if (hoverActiveCard === card) hoverActiveCard = null;
    clearTimeout(hoverPreviewTimer);
    clearTimeout(hoverPrefetchTimer);
    setTimeout(() => {
      if (!hoverPopupEl.matches(':hover') && !hoverActiveCard) hideHoverPreview();
    }, 120);
  });
}

// --- FULL-SCREEN PERSON (ACTOR) SCREEN ---
let activePersonContext = null;
const _followedHydrating = new Set();

function normalizeFollowedPerson(raw) {
  if (typeof raw === 'string') {
    const trimmed = raw.trim();
    if (!trimmed) return null;
    return /^\d+$/.test(trimmed)
      ? { id: trimmed, name: '', photo: '', role: '' }
      : { id: '', name: trimmed, photo: '', role: '' };
  }
  if (!raw || typeof raw !== 'object') return null;
  const id = String(raw.id || '').trim();
  const name = String(raw.name || '').trim();
  if (!id && !name) return null;
  return {
    id,
    name,
    photo: String(raw.photo || '').trim(),
    role: String(raw.role || '').trim(),
  };
}

function sameFollowedPerson(a, b) {
  if (!a || !b) return false;
  if (a.id && b.id && String(a.id) === String(b.id)) return true;
  if (a.name && b.name && a.name.toLowerCase() === b.name.toLowerCase()) return true;
  return false;
}

function dedupeFollowedList(items) {
  const out = [];
  for (const raw of items || []) {
    const entry = normalizeFollowedPerson(raw);
    if (!entry) continue;
    const idx = out.findIndex((existing) => sameFollowedPerson(existing, entry));
    if (idx === -1) {
      out.push(entry);
    } else {
      out[idx] = {
        id: out[idx].id || entry.id,
        name: out[idx].name || entry.name,
        photo: out[idx].photo || entry.photo,
        role: out[idx].role || entry.role,
      };
    }
  }
  return out;
}

const _fpStored = (() => {
  try {
    return JSON.parse(localStorage.getItem('nuvio_followed_people') || '[]');
  } catch {
    return [];
  }
})();
let followedPeopleList = dedupeFollowedList(_fpStored);

function saveFollowedPeopleLocal() {
  try {
    localStorage.setItem('nuvio_followed_people', JSON.stringify(followedPeopleList));
  } catch {}
}

function persistFollowedPeople() {
  saveFollowedPeopleLocal();
  invoke('save_followed_people', { people: followedPeopleList }).catch(() => {});
}

function findFollowedIndex(person) {
  const target = normalizeFollowedPerson(person);
  if (!target) return -1;
  return followedPeopleList.findIndex((item) => sameFollowedPerson(item, target));
}

function isPersonFollowed(person) {
  return findFollowedIndex(person) !== -1;
}

function toggleFollowPerson(person) {
  const entry = normalizeFollowedPerson(person);
  if (!entry) return false;
  const idx = findFollowedIndex(entry);
  if (idx !== -1) {
    followedPeopleList.splice(idx, 1);
    persistFollowedPeople();
    return false;
  }
  followedPeopleList.unshift(entry);
  persistFollowedPeople();
  return true;
}

function syncFollowedPersonMeta(person) {
  const entry = normalizeFollowedPerson(person);
  if (!entry) return;
  const idx = findFollowedIndex(entry);
  if (idx === -1) return;
  const cur = followedPeopleList[idx];
  const next = {
    id: entry.id || cur.id,
    name: entry.name || cur.name,
    photo: entry.photo || cur.photo,
    role: entry.role || cur.role,
  };
  if (
    next.id !== cur.id ||
    next.name !== cur.name ||
    next.photo !== cur.photo ||
    next.role !== cur.role
  ) {
    followedPeopleList[idx] = next;
    persistFollowedPeople();
  }
}

async function refreshFollowedPeople() {
  let dbList = [];
  try {
    dbList = await invoke('list_followed_people');
  } catch {}
  const localList = (() => {
    try {
      return JSON.parse(localStorage.getItem('nuvio_followed_people') || '[]');
    } catch {
      return [];
    }
  })();
  const merged = dedupeFollowedList([...(Array.isArray(dbList) ? dbList : []), ...localList]);
  followedPeopleList = merged;
  saveFollowedPeopleLocal();
  if (merged.length && JSON.stringify(merged) !== JSON.stringify(dbList || [])) {
    invoke('save_followed_people', { people: merged }).catch(() => {});
  }
  if (activeTabName() === 'library') renderFollowingSection();
}

async function hydrateMissingFollowedPhotos() {
  const pending = followedPeopleList.filter((p) => (!p.photo || !p.name) && !_followedHydrating.has(p.id || p.name));
  if (!pending.length) return;
  let updated = false;
  for (const p of pending) {
    const hydKey = p.id || p.name;
    _followedHydrating.add(hydKey);
    try {
      const data = await invoke('person_details', {
        personId: Number(p.id) || null,
        name: p.name || null,
      });
      if (data && (data.name || data.profile || data.id)) {
        syncFollowedPersonMeta({
          id: String(data.id || p.id || ''),
          name: data.name || p.name || '',
          photo: data.profile || p.photo || '',
          role: personRoleLabel(data),
        });
        updated = true;
      }
    } catch {
      // TMDB key absent or network failure; leave placeholder intact
    }
  }
  if (updated && activeTabName() === 'library') {
    renderFollowingSection();
  }
}

const notifiedPeople = new Set();
let personUi = { tab: 'filmography', type: 'all', sort: 'date', view: 'grid' };

const PP_ICONS = {
  back: '<svg viewBox="0 0 24 24" width="20" height="20"><path fill="currentColor" d="M15.41 7.41L14 6l-6 6l6 6l1.41-1.41L10.83 12z"/></svg>',
  plus: '<svg viewBox="0 0 24 24" width="18" height="18"><path fill="currentColor" d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6z"/></svg>',
  check: '<svg viewBox="0 0 24 24" width="18" height="18"><path fill="currentColor" d="M9 16.17L4.83 12l-1.42 1.41L9 19L21 7l-1.41-1.41z"/></svg>',
  bell: '<svg viewBox="0 0 24 24" width="19" height="19"><path fill="currentColor" d="M12 22a2 2 0 0 0 2-2h-4a2 2 0 0 0 2 2m6-6V11c0-3.07-1.64-5.64-4.5-6.32V4a1.5 1.5 0 0 0-3 0v.68C7.63 5.36 6 7.92 6 11v5l-2 2v1h16v-1z"/></svg>',
  share: '<svg viewBox="0 0 24 24" width="19" height="19"><path fill="currentColor" d="M18 16.08c-.76 0-1.44.3-1.96.77L8.91 12.7c.05-.23.09-.46.09-.7s-.04-.47-.09-.7l7.05-4.11c.54.5 1.25.81 2.04.81a3 3 0 1 0-3-3c0 .24.04.47.09.7L8.04 9.81A3 3 0 0 0 6 9a3 3 0 1 0 0 6c.79 0 1.5-.31 2.04-.81l7.12 4.16c-.05.21-.08.43-.08.65a2.92 2.92 0 1 0 2.92-2.92"/></svg>',
  cake: '<svg viewBox="0 0 24 24" width="18" height="18"><path fill="currentColor" d="M19 4h-1V2h-2v2H8V2H6v2H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V6a2 2 0 0 0-2-2m0 16H5V10h14zm0-12H5V6h14z"/></svg>',
  pin: '<svg viewBox="0 0 24 24" width="18" height="18"><path fill="currentColor" d="M12 2a7 7 0 0 0-7 7c0 5.25 7 13 7 13s7-7.75 7-13a7 7 0 0 0-7-7m0 9.5A2.5 2.5 0 1 1 14.5 9 2.5 2.5 0 0 1 12 11.5"/></svg>',
  film: '<svg viewBox="0 0 24 24" width="18" height="18"><path fill="currentColor" d="M18 4l2 4h-3l-2-4h-2l2 4h-3l-2-4H8l2 4H7L5 4H4a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V4z"/></svg>',
  grid: '<svg viewBox="0 0 24 24" width="17" height="17"><path fill="currentColor" d="M3 3h8v8H3zm10 0h8v8h-8zM3 13h8v8H3zm10 0h8v8h-8z"/></svg>',
  list: '<svg viewBox="0 0 24 24" width="17" height="17"><path fill="currentColor" d="M3 5h18v2H3zm0 6h18v2H3zm0 6h18v2H3z"/></svg>',
};

function personAgeLabel(birthday, deathday) {
  if (!birthday) return '';
  const born = new Date(birthday);
  if (Number.isNaN(born.getTime())) return '';
  const end = deathday ? new Date(deathday) : new Date();
  if (Number.isNaN(end.getTime())) return '';
  let age = end.getFullYear() - born.getFullYear();
  const month = end.getMonth() - born.getMonth();
  if (month < 0 || (month === 0 && end.getDate() < born.getDate())) age -= 1;
  return age > 0 ? `${age} anos` : '';
}

function personBornLabel(birthday, deathday) {
  if (!birthday) return '';
  const born = new Date(`${birthday}T12:00:00`);
  if (Number.isNaN(born.getTime())) return birthday;
  const date = born.toLocaleDateString('pt-BR', { day: '2-digit', month: 'short', year: 'numeric' });
  const age = personAgeLabel(birthday, deathday);
  return age ? `${date} (${age})` : date;
}

function personRoleLabel(data) {
  if (data.knownFor === 'Acting') return data.gender === 1 ? 'Atriz' : 'Ator';
  if (data.knownFor === 'Directing') return 'Direção';
  if (data.knownFor === 'Writing') return 'Roteiro';
  if (data.knownFor === 'Production') return 'Produção';
  return data.knownFor || 'Elenco';
}

function ppPosterSrc(url) {
  return typeof optimizePosterUrl === 'function' ? optimizePosterUrl(url) : url;
}

function renderPersonCreditCard(item) {
  const rating = Number(item.rating) > 0 ? `★ ${Number(item.rating).toFixed(1)}` : '';
  const kind = item.type === 'movie' ? 'Filme' : 'Série';
  const sub = [item.year, kind].filter(Boolean).join(' • ');
  return `
    <button class="poster-card pp-card" data-meta-id="${esc(item.id)}" data-meta-type="${esc(item.type)}" data-meta-name="${esc(item.name)}" data-meta-poster="${esc(item.poster || '')}" data-meta-banner="${esc(item.banner || '')}" data-meta-desc="" data-meta-release="${esc(item.year || '')}" data-meta-imdb="">
      <div class="poster-img-wrap">
        ${item.poster ? `<img src="${esc(ppPosterSrc(item.poster))}" alt="${esc(item.name)}" loading="lazy" decoding="async" draggable="false" />` : ''}
      </div>
      <div class="pp-card-info">
        <div class="pp-card-title"><span class="pp-card-name">${esc(item.name)}</span>${rating ? `<span class="pp-rating">${rating}</span>` : ''}</div>
        <div class="pp-card-sub">${esc(sub)}</div>
        ${item.character ? `<div class="pp-card-char">como ${esc(item.character)}</div>` : ''}
      </div>
    </button>
  `;
}

function personVisibleCredits(credits) {
  let list = credits.slice();
  if (personUi.type !== 'all') list = list.filter((c) => c.type === personUi.type);
  if (personUi.sort === 'date') {
    list.sort((a, b) => (Number(b.year) || 0) - (Number(a.year) || 0));
  } else if (personUi.sort === 'date-asc') {
    list.sort((a, b) => (Number(a.year) || 9999) - (Number(b.year) || 9999));
  } else {
    list.sort((a, b) => (Number(b.rating) || 0) - (Number(a.rating) || 0));
  }
  return list;
}

function renderPersonCredits() {
  const grid = document.getElementById('pp-credits-grid');
  if (!grid) return;
  const credits = Array.isArray(activePersonContext?.full?.credits) ? activePersonContext.full.credits : [];
  const list = personVisibleCredits(credits);
  grid.className = `pp-grid ${personUi.view === 'list' ? 'list' : ''}`;
  grid.innerHTML = list.length
    ? list.map(renderPersonCreditCard).join('')
    : `<div class="muted">${credits.length ? 'Nenhum título nesse filtro.' : 'Carregando filmografia…'}</div>`;
  const count = document.getElementById('pp-credits-count');
  if (count) count.textContent = credits.length ? `${list.length} títulos` : '';
}

async function openPersonPage(person) {
  const id = String(person?.id || '').trim();
  const personName = String(person?.name || '').trim();
  if (!id && !personName) {
    showToast('Elenco sem identificação no TMDB.');
    return;
  }
  activePersonContext = { ...person, id };
  personUi = { tab: 'filmography', type: 'all', sort: 'date', view: 'grid' };
  setTab('person');
  renderPersonScreen(person, null);
  if (activeTabName() !== 'person') return;
  try {
    const data = await invoke('person_details', { personId: Number(id) || null, name: personName || null });
    if (activePersonContext?.id === id && activePersonContext?.name === person.name && activeTabName() === 'person') {
      activePersonContext.full = data;
      renderPersonScreen(person, data);
    }
  } catch (error) {
    showToast(String(error));
  }
}

function renderPersonScreen(preview, data) {
  const d = data || {};
  const resolvedId = String(d.id || preview.id || '').trim();
  const personKey = String(resolvedId || preview.name);
  const name = d.name || preview.name || 'Elenco';
  const photo = d.profile || preview.photo || '';
  const bigPhoto = d.profileLarge || photo;
  const credits = Array.isArray(d.credits) ? d.credits : [];
  const backdrop = credits.filter((c) => c.banner)[0]?.banner || '';
  const role = personRoleLabel(d);
  if (data && (resolvedId || name)) {
    syncFollowedPersonMeta({ id: resolvedId, name, photo, role });
  }
  const birthYear = d.birthday ? Number(d.birthday.slice(0, 4)) : 0;
  const yearFloor = birthYear > 1900 ? birthYear : 1920;
  const years = credits.map((c) => Number(c.year)).filter((y) => y >= yearFloor);
  const sinceYear = years.length ? Math.min(...years) : null;
  const biography = String(d.biography || '').trim();
  const following = isPersonFollowed({ id: resolvedId, name });
  const notified = notifiedPeople.has(personKey);

  const infoRows = [
    d.birthday && [PP_ICONS.cake, 'Nascimento', personBornLabel(d.birthday, d.deathday)],
    d.placeOfBirth && [PP_ICONS.pin, 'Local', d.placeOfBirth],
    credits.length && [PP_ICONS.film, 'Créditos', `${credits.length} títulos${sinceYear ? ` • desde ${sinceYear}` : ''}`],
  ].filter(Boolean);

  const heroWrap = document.getElementById('person-hero-wrapper');
  const bodyWrap = document.getElementById('person-body-wrapper');
  if (!heroWrap) return;
  if (bodyWrap) bodyWrap.innerHTML = '';

  heroWrap.innerHTML = `
    <div class="pp-page">
      ${backdrop ? `<img class="pp-bg-banner" src="${esc(backdrop)}" alt="" />` : ''}
      ${bigPhoto ? `<img class="pp-bg-photo" src="${esc(bigPhoto)}" alt="" />` : ''}
      <div class="pp-bg-scrim"></div>
      <button class="circle-back-btn pp-back" id="btn-person-back" title="Voltar" type="button">${PP_ICONS.back}</button>
      <div class="pp-layout">
        <aside class="pp-side">
          <div class="pp-id">
            <div class="pp-avatar">${photo ? `<img src="${esc(photo)}" alt="${esc(name)}" draggable="false" />` : esc(name.charAt(0).toUpperCase())}</div>
            <div class="pp-id-text">
              <h1 class="pp-name">${esc(name)}</h1>
              <div class="pp-role">${esc(role)}</div>
            </div>
          </div>
          <div class="pp-actions">
            <button class="pp-follow ${following ? 'active' : ''}" id="btn-person-follow" type="button">${following ? PP_ICONS.check : PP_ICONS.plus}<span>${following ? 'Seguindo' : 'Seguir'}</span></button>
            <button class="pp-circle ${notified ? 'active' : ''}" id="btn-person-bell" type="button" title="Notificações">${PP_ICONS.bell}</button>
            <button class="pp-circle" id="btn-person-homepage" type="button" title="Abrir no TMDB">${PP_ICONS.share}</button>
          </div>
          ${infoRows.length ? `<div class="pp-card-box pp-info">
            ${infoRows.map(([icon, label, value]) => `<div class="pp-info-row"><span class="pp-info-icon">${icon}</span><span class="pp-info-label">${esc(label)}</span><strong>${esc(value)}</strong></div>`).join('')}
          </div>` : ''}
          ${biography ? `<div class="pp-card-box pp-bio-box">
            <h3>Biografia</h3>
            <p class="pp-bio clamped" id="person-bio">${esc(biography)}</p>
            <button class="pp-more" id="btn-person-bio" type="button">Ler mais ⌄</button>
          </div>` : ''}
        </aside>
        <main class="pp-main">
          <div class="pp-tabs" role="tablist">
            <button class="pp-tab ${personUi.tab === 'overview' ? 'active' : ''}" data-person-tab="overview" type="button">Visão geral</button>
            <button class="pp-tab ${personUi.tab === 'filmography' ? 'active' : ''}" data-person-tab="filmography" type="button">Filmografia</button>
          </div>
          <section class="pp-panel" id="pp-panel-overview" ${personUi.tab === 'overview' ? '' : 'hidden'}>
            <h2 class="pp-title">Mais conhecidos</h2>
            <div class="pp-grid" id="pp-known-grid"></div>
          </section>
          <section class="pp-panel" id="pp-panel-filmography" ${personUi.tab === 'filmography' ? '' : 'hidden'}>
            <div class="pp-head">
              <h2 class="pp-title">Filmografia <span class="pp-count" id="pp-credits-count"></span></h2>
              <div class="pp-controls">
                <select class="pp-select" id="pp-filter-type" aria-label="Filtrar por tipo">
                  <option value="all" ${personUi.type === 'all' ? 'selected' : ''}>Todos</option>
                  <option value="movie" ${personUi.type === 'movie' ? 'selected' : ''}>Filmes</option>
                  <option value="series" ${personUi.type === 'series' ? 'selected' : ''}>Séries</option>
                </select>
                <select class="pp-select" id="pp-sort" aria-label="Ordenar">
                  <option value="date" ${personUi.sort === 'date' ? 'selected' : ''}>Mais recentes</option>
                  <option value="date-asc" ${personUi.sort === 'date-asc' ? 'selected' : ''}>Mais antigos</option>
                  <option value="rating" ${personUi.sort === 'rating' ? 'selected' : ''}>Melhor nota</option>
                </select>
                <div class="pp-view-toggle">
                  <button class="${personUi.view === 'grid' ? 'active' : ''}" data-person-view="grid" type="button" title="Grade">${PP_ICONS.grid}</button>
                  <button class="${personUi.view === 'list' ? 'active' : ''}" data-person-view="list" type="button" title="Lista">${PP_ICONS.list}</button>
                </div>
              </div>
            </div>
            <div class="pp-grid" id="pp-credits-grid"></div>
          </section>
        </main>
      </div>
    </div>
  `;

  const known = credits.slice().sort((a, b) => (Number(b.rating) || 0) - (Number(a.rating) || 0)).slice(0, 12);
  const knownGrid = document.getElementById('pp-known-grid');
  if (knownGrid) knownGrid.innerHTML = known.map(renderPersonCreditCard).join('');
  renderPersonCredits();

  document.getElementById('btn-person-back').addEventListener('click', () => {
    setTab(activeMetaContext ? 'details' : (previousTab || 'home'));
  });
  document.getElementById('btn-person-follow').addEventListener('click', () => {
    const nowFollowing = toggleFollowPerson({ id: resolvedId, name, photo, role });
    showToast(nowFollowing ? `Seguindo ${name}.` : `Deixou de seguir ${name}.`);
    renderPersonScreen(preview, data);
  });
  document.getElementById('btn-person-bell').addEventListener('click', () => {
    const was = notifiedPeople.has(personKey);
    if (was) notifiedPeople.delete(personKey); else notifiedPeople.add(personKey);
    showToast(was ? 'Notificações desativadas.' : `Você será avisado sobre ${name}.`);
    renderPersonScreen(preview, data);
  });
  document.getElementById('btn-person-homepage').addEventListener('click', () => {
    const url = d.homepage || (d.id ? `https://www.themoviedb.org/person/${d.id}` : '');
    if (url) invoke('open_external_url', { url }).catch((e) => showToast(String(e)));
  });
  document.getElementById('btn-person-bio')?.addEventListener('click', (e) => {
    const p = document.getElementById('person-bio');
    const clamped = p.classList.toggle('clamped');
    e.currentTarget.textContent = clamped ? 'Ler mais ⌄' : 'Ler menos ⌃';
  });
  heroWrap.querySelectorAll('[data-person-tab]').forEach((tab) => {
    tab.addEventListener('click', () => {
      personUi.tab = tab.dataset.personTab;
      heroWrap.querySelectorAll('[data-person-tab]').forEach((o) => o.classList.toggle('active', o === tab));
      document.getElementById('pp-panel-overview').hidden = personUi.tab !== 'overview';
      document.getElementById('pp-panel-filmography').hidden = personUi.tab !== 'filmography';
    });
  });
  document.getElementById('pp-filter-type').addEventListener('change', (e) => {
    personUi.type = e.target.value;
    renderPersonCredits();
  });
  document.getElementById('pp-sort').addEventListener('change', (e) => {
    personUi.sort = e.target.value;
    renderPersonCredits();
  });
  heroWrap.querySelectorAll('[data-person-view]').forEach((btn) => {
    btn.addEventListener('click', () => {
      personUi.view = btn.dataset.personView;
      heroWrap.querySelectorAll('[data-person-view]').forEach((o) => o.classList.toggle('active', o === btn));
      renderPersonCredits();
    });
  });
}

// --- FULL-SCREEN META DETAILS SCREEN (MetaDetailsScreen.kt) ---
function formatDetailStatus(status, isSeries) {
  const raw = String(status || '').trim();
  if (!raw) return '';
  switch (raw.toLowerCase()) {
    case 'released': return 'Lançado';
    case 'ended': return isSeries ? 'Finalizada' : 'Encerrado';
    case 'returning series':
    case 'continuing': return 'Em exibição';
    case 'current': return 'Atual';
    case 'canceled':
    case 'cancelled': return isSeries ? 'Cancelada' : 'Cancelado';
    case 'planned': return 'Planejado';
    case 'in production': return 'Em produção';
    case 'post production': return 'Pós-produção';
    case 'rumored': return 'Rumor';
    case 'pilot': return 'Piloto';
    default: return raw;
  }
}

function formatDetailRuntime(rawRuntime) {
  const raw = String(rawRuntime || '').trim();
  if (!raw) return '';
  let mins = 0;
  const hMatch = raw.match(/(\d+)\s*h/i);
  const mMatch = raw.match(/(\d+)\s*m/i);
  if (hMatch || mMatch) {
    mins = (hMatch ? Number(hMatch[1]) * 60 : 0) + (mMatch ? Number(mMatch[1]) : 0);
  } else {
    const colon = raw.match(/^(\d+)\s*:\s*(\d{1,2})$/);
    if (colon) mins = Number(colon[1]) * 60 + Number(colon[2]);
    else if (/^\d+$/.test(raw)) mins = Number(raw);
  }
  if (!Number.isFinite(mins) || mins <= 0) return raw;
  const h = Math.floor(mins / 60);
  const m = mins % 60;
  if (h > 0 && m > 0) return `${h}h ${m}m`;
  if (h > 0) return `${h}h`;
  return `${m}m`;
}

function formatDetailReleaseDate(raw) {
  const text = String(raw || '').trim();
  if (!text) return '';
  if (/^\d{4}$/.test(text)) return text;
  const iso = text.match(/^(\d{4})-(\d{2})-(\d{2})/);
  if (!iso) return text;
  const d = new Date(Number(iso[1]), Number(iso[2]) - 1, Number(iso[3]));
  if (Number.isNaN(d.getTime())) return text;
  return d.toLocaleDateString('pt-BR', { day: 'numeric', month: 'short', year: 'numeric' });
}

function normalizePersonRole(role) {
  const r = String(role || '').trim();
  if (!r) return '';
  const low = r.toLowerCase();
  if (low === 'director') return 'Diretor';
  if (low === 'creator') return 'Criador';
  if (low === 'writer' || low === 'screenplay') return 'Roteirista';
  return r;
}

function extractDetailPeople(m) {
  const dirs = [];
  const dirSeen = new Map();
  const actors = [];
  const actorSeen = new Map();

  const pushEntry = (target, seen, entry, fallbackRole = '') => {
    if (!entry) return;
    const isObj = typeof entry === 'object';
    const name = String(isObj ? (entry.name || '') : entry).trim();
    if (!name) return;
    const role = normalizePersonRole(isObj ? (entry.character || entry.role || fallbackRole) : fallbackRole);
    let photo = isObj ? (entry.photo || entry.profile || entry.image || entry.profile_path || '') : '';
    if (photo && photo.startsWith('/')) photo = `https://image.tmdb.org/t/p/w185${photo}`;
    const id = isObj ? String(entry.id || entry.tmdbId || '').trim() : '';
    const key = `${name.toLowerCase()}|${role.toLowerCase()}`;
    if (seen.has(key)) {
      const prev = seen.get(key);
      if (!prev.photo && photo) prev.photo = photo;
      if (!prev.id && id) prev.id = id;
      return;
    }
    const item = { id, name, character: role, photo };
    seen.set(key, item);
    target.push(item);
  };

  if (Array.isArray(m.app_extras?.creators)) {
    m.app_extras.creators.forEach((c) => pushEntry(dirs, dirSeen, c, 'Criador'));
  }
  if (Array.isArray(m.app_extras?.directors)) {
    m.app_extras.directors.forEach((d) => pushEntry(dirs, dirSeen, d, 'Diretor'));
  }
  if (Array.isArray(m.director)) {
    m.director.forEach((d) => pushEntry(dirs, dirSeen, d, 'Diretor'));
  } else if (typeof m.director === 'string' && m.director.trim()) {
    m.director.split(',').forEach((d) => pushEntry(dirs, dirSeen, d.trim(), 'Diretor'));
  }

  const castSource = m.castMembers || m.credits_cast || m.app_extras?.cast || m.cast;
  const rawCast = Array.isArray(castSource) ? castSource : [];
  rawCast.forEach((c) => {
    const isObj = typeof c === 'object';
    const name = String(isObj ? (c?.name || '') : (c || '')).trim();
    const role = normalizePersonRole(isObj ? (c?.character || c?.role || '') : '');
    const key = `${name.toLowerCase()}|${role.toLowerCase()}`;
    if (dirSeen.has(key)) {
      pushEntry(dirs, dirSeen, c, role);
    } else {
      pushEntry(actors, actorSeen, c, '');
    }
  });

  return [...dirs, ...actors];
}

function extractProductionCompanies(m, isSeries) {
  const parseList = (val) => {
    if (Array.isArray(val)) {
      return val.map((c) => (typeof c === 'string'
        ? { name: c.trim(), logo: '', id: '' }
        : { name: String(c?.name || '').trim(), logo: c?.logo || c?.logoUrl || '', id: c?.id || c?.tmdbId || '' }))
        .filter((c) => c.name);
    }
    if (typeof val === 'string' && val.trim()) {
      return val.split(',').map((s) => ({ name: s.trim(), logo: '', id: '' })).filter((c) => c.name);
    }
    return [];
  };
  const prod = parseList(m.productionCompanies).length
    ? parseList(m.productionCompanies)
    : (parseList(m.app_extras?.productionCompanies).length ? parseList(m.app_extras?.productionCompanies) : parseList(m.production));
  const net = parseList(m.networkCompanies).length
    ? parseList(m.networkCompanies)
    : (parseList(m.app_extras?.networks).length ? parseList(m.app_extras?.networks) : parseList(m.networks));
  const chosen = isSeries ? (net.length ? net : prod) : (prod.length ? prod : net);
  if (!isSeries) {
    const withLogo = chosen.filter((c) => c.logo);
    return (withLogo.length ? withLogo : chosen).slice(0, 6);
  }
  return chosen.slice(0, 6);
}

function localizeTrailerCategory(rawType) {
  const t = String(rawType || '').trim();
  if (!t) return 'Trailer';
  switch (t.toLowerCase()) {
    case 'trailer': return 'Trailer';
    case 'teaser': return 'Teaser';
    case 'clip': return 'Clipe';
    case 'featurette': return 'Featurette';
    case 'behind the scenes': return 'Bastidores';
    case 'bloopers': return 'Erros';
    default: return t;
  }
}

function extractYoutubeKey(item) {
  const direct = String(item?.key || item?.source || '').trim();
  if (direct && /^[A-Za-z0-9_-]{6,20}$/.test(direct)) return direct;
  const url = String(item?.url || '').trim();
  const match = url.match(/(?:youtube\.com\/watch\?v=|youtu\.be\/|youtube\.com\/embed\/)([A-Za-z0-9_-]{6,20})/i);
  return match ? match[1] : '';
}

function extractDetailTrailers(m, fallbackTitle, fallbackRelease) {
  const raw = [
    ...(Array.isArray(m.trailers) ? m.trailers : []),
    ...(Array.isArray(m.trailerStreams) ? m.trailerStreams : []),
  ];
  const seen = new Set();
  const out = [];
  raw.forEach((tr) => {
    if (!tr || typeof tr !== 'object') return;
    const key = extractYoutubeKey(tr);
    const url = tr.url || (key ? `https://www.youtube.com/watch?v=${key}` : '');
    if (!key && !url) return;
    const dedupe = key || url;
    if (seen.has(dedupe)) return;
    seen.add(dedupe);
    const category = localizeTrailerCategory(tr.type);
    const titleText = String(tr.name || tr.title || '').trim() || `${category} - ${fallbackTitle}`;
    const rawYear = String(tr.publishedAt || tr.published_at || tr.year || fallbackRelease || '').trim();
    const yearMatch = rawYear.match(/\b(\d{4})\b/);
    out.push({
      key,
      url,
      category,
      name: titleText,
      year: yearMatch ? yearMatch[1] : '',
      thumb: key ? `https://img.youtube.com/vi/${key}/hqdefault.jpg` : '',
    });
  });
  return out;
}

function renderDetailsLoading() {
  const backdropEl = document.getElementById('detail-screen-backdrop');
  if (backdropEl) backdropEl.innerHTML = '';
  const heroWrap = document.getElementById('detail-hero-wrapper');
  if (heroWrap) {
    heroWrap.innerHTML = `
      <div class="detail-loading-stage">
        <div class="detail-top-bar">
          <button class="circle-back-btn" id="btn-detail-back" title="Voltar">
            <svg viewBox="0 0 24 24" width="20" height="20"><path fill="currentColor" d="M15.41 7.41L14 6l-6 6l6 6l1.41-1.41L10.83 12z"/></svg>
          </button>
        </div>
        <div class="detail-loading-spinner" aria-label="Carregando detalhes"></div>
      </div>
    `;
    document.getElementById('btn-detail-back')?.addEventListener('click', () => setTab(previousTab || 'home'));
  }
  const bodyWrap = document.getElementById('detail-body-wrapper');
  if (bodyWrap) bodyWrap.innerHTML = '';
}

async function openDetailsPage(preview) {
  activeMetaContext = { ...preview, full: null };
  setTab('details');

  const cached = getCachedMetaDetails(preview.type, preview.id);
  if (cached) {
    activeMetaContext.full = cached;
    renderDetailsScreen(preview, cached);
    return;
  }

  renderDetailsLoading();

  try {
    const meta = await fetchMetaDetailsCached(preview.type, preview.id);
    if (activeMetaContext?.id === preview.id) {
      activeMetaContext.full = meta;
      renderDetailsScreen(preview, meta);
    }
  } catch {
    if (activeMetaContext?.id === preview.id) {
      renderDetailsScreen(preview, null);
    }
  }
}

function findCwResumeForContent(contentId) {
  const src = activeProgressSource();
  if (src === 'simkl') return cwCardsCache.find((c) => simklProgressMatches(c, contentId)) || null;
  if (src === 'mdblist') return cwCardsCache.find((c) => mdblistMatchesCard(c, contentId)) || null;
  return cwCardsCache.find((c) => c.content_id === contentId) || null;
}

function updateDetailScrollDim() {
  const dim = document.getElementById('detail-scroll-dim');
  const hero = document.querySelector('#tab-details .detail-hero');
  if (!dim || !hero) return;
  const heroH = hero.offsetHeight || 660;
  const progress = Math.min(1, Math.max(0, (mainViewportEl?.scrollTop || 0) / (heroH * 0.75)));
  const mode = appSettings?.detail_bg_mode || 'cinematic';
  const maxAlpha = mode === 'normal' ? 0.86 : 0.36;
  dim.style.opacity = (progress * maxAlpha).toFixed(3);
}

function renderDetailsScreen(preview, meta) {
  const m = meta || {};
  const title = m.name || preview.name || 'Sem título';
  const bg = m.background || preview.banner || m.poster || '';
  const backdropEl = document.getElementById('detail-screen-backdrop');
  if (backdropEl) {
    backdropEl.innerHTML = `
      ${bg ? `<img class="detail-screen-bg" src="${esc(bg)}" alt="" />` : ''}
      <div class="detail-scrim-side"></div>
      <div class="detail-scroll-dim" id="detail-scroll-dim"></div>
    `;
  }
  const logo = m.logo || preview.logo || '';
  const desc = m.description || preview.description || '';
  const release = m.releaseInfo || preview.release_info || '';
  const imdb = m.imdbRating || preview.imdb_rating || '';
  const runtime = formatDetailRuntime(m.runtime || '');
  const resolvedTmdbId = m.moviedb_id || m.tmdb_id || preview.tmdbId || null;
  const ageRating = m.ageRating || m.app_extras?.certificationLocal || m.certification || m.app_extras?.certification || '';
  const genres = Array.isArray(m.genres) ? m.genres : [];
  const videos = Array.isArray(m.videos) ? m.videos : [];
  const isSeriesLike = preview.type === 'series' || videos.some((v) => v.season != null || v.episode != null);
  const statusLabel = formatDetailStatus(m.status, isSeriesLike);
  const cast = extractDetailPeople(m);
  const companies = extractProductionCompanies(m, isSeriesLike);
  const trailers = extractDetailTrailers(m, title, release);

  const inLib = librarySet.has(libKey(preview.id, preview.type));
  const titleWatched = watchedSet.has(preview.id);
  const cwResume = findCwResumeForContent(preview.id);

  let defaultPlayId = preview.id.startsWith('tmdb:') && m.imdb_id ? m.imdb_id : preview.id;
  let defaultPlayLabel = cwResume ? 'Continuar' : 'Assistir';
  let defaultEpisodeInfo = null;
  let resumeSecs = cwResume ? Math.max(0, (cwResume.position_ms || 0) / 1000) : 0;

  if (videos.length > 0) {
    let chosenEp = null;
    if (cwResume?.video_id) {
      chosenEp = videos.find((v) => v.id === cwResume.video_id)
        || videos.find((v) => v.season === cwResume.season && (v.episode ?? v.number) === cwResume.episode);
    }
    if (!chosenEp) {
      chosenEp = videos.find((v) => (v.season ?? 1) > 0) || videos[0];
      if (!cwResume || cwResume.video_id !== chosenEp.id) resumeSecs = 0;
    }
    defaultPlayId = chosenEp.id || preview.id;
    const s = chosenEp.season ?? 1;
    const e = chosenEp.episode ?? chosenEp.number ?? 1;
    defaultPlayLabel = `${cwResume?.video_id === chosenEp.id ? 'Continuar' : 'Assistir'} T${s} E${e}`;
    defaultEpisodeInfo = chosenEp;
  }

  const heroWrap = document.getElementById('detail-hero-wrapper');
  heroWrap.innerHTML = `
    <div class="detail-hero">
      <div class="detail-top-bar">
        <button class="circle-back-btn" id="btn-detail-back" title="Voltar">
          <svg viewBox="0 0 24 24" width="20" height="20"><path fill="currentColor" d="M15.41 7.41L14 6l-6 6l6 6l1.41-1.41L10.83 12z"/></svg>
        </button>
      </div>
      <div class="detail-hero-content">
        ${logo ? `<img class="detail-logo" src="${esc(logo)}" alt="${esc(title)}" />` : `<h1 class="detail-title">${esc(title)}</h1>`}
        <div class="detail-meta-chips">
          ${imdb ? `<span class="imdb-chip"><img src="assets/rating_imdb.png" alt="IMDb" /> ${esc(imdb)}</span>` : ''}
          ${ratingBadges(m.externalRatings)}
          ${release ? `<span>${esc(formatDetailReleaseDate(release))}</span>` : ''}
          ${runtime ? `<span>• ${esc(runtime)}</span>` : ''}
          ${ageRating ? `<span class="meta-badge">${esc(ageRating)}</span>` : ''}
          ${statusLabel ? `<span class="meta-badge">${esc(statusLabel)}</span>` : ''}
        </div>
        ${genres.length ? `<div class="detail-genres">${esc(genres.slice(0, 4).join(' • '))}</div>` : ''}
        ${desc ? `
          <p class="detail-synopsis clamped" id="detail-synopsis">${esc(desc)}</p>
          <button class="link-btn" id="btn-toggle-synopsis" style="margin-top:6px;font-size:12px;">Ver mais</button>
        ` : ''}
        <div class="detail-actions-row">
          <button class="detail-play-btn" id="btn-detail-play">
            <svg viewBox="0 0 24 24" width="20" height="20"><path fill="currentColor" d="M8 5v14l11-7z"/></svg>
            <span>${esc(defaultPlayLabel)}</span>
          </button>
          <button class="detail-circle-action ${inLib ? 'active' : ''}" id="btn-detail-library" title="${inLib ? 'Remover da Biblioteca' : 'Salvar na Biblioteca'}">${inLib ? '&#10003;' : '+'}</button>
          <button class="detail-circle-action ${titleWatched ? 'active' : ''}" id="btn-detail-watched" title="${titleWatched ? 'Desmarcar como assistido' : 'Marcar como assistido'}">
            <svg viewBox="0 0 24 24" width="18" height="18"><path fill="currentColor" d="M12 4.5C7 4.5 2.73 7.61 1 12c1.73 4.39 6 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6-7.5-11-7.5M12 17a5 5 0 1 1 0-10 5 5 0 0 1 0 10m0-8a3 3 0 1 0 0 6 3 3 0 0 0 0-6"/></svg>
          </button>
          ${videos.length > 1 ? `
            <button class="detail-circle-action" id="btn-detail-shuffle" title="Episódio aleatório">
              <svg viewBox="0 0 24 24" width="18" height="18"><path fill="currentColor" d="M10.59 9.17 5.41 4 4 5.41l5.17 5.17zM14.5 4l2.04 2.04L4 18.59 5.41 20 17.96 7.46 20 9.5V4zm.33 9.41-1.41 1.41 3.13 3.13L14.5 20H20v-5.5l-2.04 2.04z"/></svg>
            </button>
          ` : ''}
          ${trailers.length ? `
            <button class="detail-circle-action" id="btn-detail-trailer" title="Assistir Trailer">
              <svg viewBox="0 0 24 24" width="18" height="18"><path fill="currentColor" d="M18 4l2 4h-3l-2-4h-2l2 4h-3l-2-4H8l2 4H7L5 4H4a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V4z"/></svg>
            </button>
          ` : ''}
        </div>
      </div>
    </div>
  `;

  document.getElementById('btn-detail-back').addEventListener('click', () => setTab(previousTab || 'home'));
  document.getElementById('btn-toggle-synopsis')?.addEventListener('click', (e) => {
    const p = document.getElementById('detail-synopsis');
    const clamped = p.classList.toggle('clamped');
    e.target.textContent = clamped ? 'Ver mais' : 'Ver menos';
  });
  document.getElementById('btn-detail-shuffle')?.addEventListener('click', () => {
    const pool = videos.filter((v) => (v.season ?? 1) > 0);
    const list = pool.length ? pool : videos;
    const picked = list[Math.floor(Math.random() * list.length)];
    if (!picked) return;
    openStreamsScreen({
      mediaType: preview.type,
      videoId: picked.id || preview.id,
      contentId: preview.id,
      tmdbId: resolvedTmdbId ? String(resolvedTmdbId) : null,
      title,
      logo,
      background: bg,
      poster: m.poster || preview.poster || '',
      episode: picked,
      videos,
      resumePositionSecs: 0,
    });
  });
  document.getElementById('btn-detail-trailer')?.addEventListener('click', () => {
    const tr = trailers[0];
    if (tr?.url) invoke('open_external_url', { url: tr.url }).catch((e) => showToast(String(e)));
  });
  document.getElementById('btn-detail-library').addEventListener('click', async (e) => {
    const btn = e.currentTarget;
    const saved = await toggleItemLibrary({
      id: preview.id,
      type: preview.type,
      name: title,
      poster: m.poster || preview.poster || '',
      banner: bg,
      logo,
      description: desc,
      release_info: release,
      imdb_rating: imdb,
    });
    btn.classList.toggle('active', saved);
    btn.innerHTML = saved ? '&#10003;' : '+';
  });
  document.getElementById('btn-detail-watched').addEventListener('click', async (e) => {
    const btn = e.currentTarget;
    const watched = await toggleItemWatched(preview.id, preview.type, title, null, null);
    btn.classList.toggle('active', watched);
    refreshPosterWatchedBadges();
  });
  document.getElementById('btn-detail-play').addEventListener('click', () => {
    openStreamsScreen({
      mediaType: preview.type,
      videoId: defaultPlayId,
      contentId: preview.id,
      tmdbId: resolvedTmdbId ? String(resolvedTmdbId) : null,
      title,
      logo,
      background: bg,
      poster: m.poster || preview.poster || '',
      episode: defaultEpisodeInfo,
      videos,
      resumePositionSecs: resumeSecs,
    });
  });

  const bodyWrap = document.getElementById('detail-body-wrapper');
  let bodyHtml = '';
  const seasonInfo = new Map((m.tmdbSeasons || []).map(s => [s.number, s]));
  const seasons = [...new Set([...videos.map(v => v.season ?? 1),
    ...(appSettings?.services?.tmdb_episodes ? seasonInfo.keys() : [])])].sort((a,b) => a-b);
  if (seasons.length > 0) {
    const defaultSeason = seasons.find((s) => s > 0) ?? seasons[0];
    bodyHtml += `
      <div>
        <div class="row-gap" style="justify-content:space-between;margin-bottom:10px;">
          <div class="season-pills" id="detail-season-pills" style="margin-bottom:0;">
            ${seasons.map((s) => `<button class="season-pill ${s === defaultSeason ? 'active' : ''}" data-season="${s}">${seasonInfo.get(s)?.poster ? `<img class="season-thumb" src="${esc(seasonInfo.get(s).poster)}" loading="lazy" alt="" />` : ''}${s === 0 ? 'Especiais' : `Temporada ${s}`}</button>`).join('')}
          </div>
          <button class="btn-ghost" id="btn-season-mark-watched" style="font-size:12px;">&#10003; Marcar temporada vista</button>
        </div>
        <div class="episodes-grid shelf-row" id="detail-episodes-grid"></div>
      </div>
    `;
  }

  if (companies.length > 0) {
    const prodTitle = isSeriesLike ? 'Emissoras' : 'Produção';
    bodyHtml += `
      <section class="production-section">
        <h3 class="detail-section-title">${prodTitle}</h3>
        <div class="production-chips-row">
          ${companies.map((c) => `
            <div class="production-chip" title="${esc(c.name)}">
              ${c.logo ? `<img src="${esc(c.logo)}" alt="${esc(c.name)}" loading="lazy" draggable="false" />` : `<span>${esc(c.name)}</span>`}
            </div>
          `).join('')}
        </div>
      </section>
    `;
  }

  if (cast.length > 0) {
    bodyHtml += `
      <section>
        <h3 class="detail-section-title">Elenco</h3>
        <div class="cast-row shelf-row">
          ${cast.slice(0, 24).map((c) => {
            const actorName = c.name || '';
            const character = c.character || '';
            const photo = c.photo || '';
            const personId = c.id || '';
            const personAttrs = actorName
              ? ` data-person-id="${esc(personId)}" data-person-name="${esc(actorName)}" data-person-photo="${esc(photo)}" role="button" tabindex="0"`
              : '';
            return `
              <div class="cast-portrait-card"${personAttrs}>
                <div class="cast-avatar-circle">
                  ${photo ? `<img src="${esc(photo)}" alt="${esc(actorName)}" loading="lazy" draggable="false" />` : esc(actorName.charAt(0).toUpperCase())}
                </div>
                <div class="cast-actor-name">${esc(actorName)}</div>
                ${character ? `<div class="cast-char-name">${esc(character)}</div>` : ''}
              </div>
            `;
          }).join('')}
        </div>
      </section>
    `;
  }

  let trailerGroups = null;
  let defaultTrailerCategory = '';
  if (trailers.length > 0) {
    trailerGroups = new Map();
    trailers.forEach((tr) => {
      if (!trailerGroups.has(tr.category)) trailerGroups.set(tr.category, []);
      trailerGroups.get(tr.category).push(tr);
    });
    const categories = Array.from(trailerGroups.keys());
    defaultTrailerCategory = categories.find((c) => c.toLowerCase() === 'trailer') || categories[0];
    bodyHtml += `
      <section class="detail-trailers-section">
        <div class="trailer-header-row">
          <h3 class="detail-section-title">Trailers</h3>
          <div class="trailer-cat-wrap">
            <select id="detail-trailer-category" class="trailer-cat-select" aria-label="Categoria de trailer">
              ${categories.map((cat) => `<option value="${esc(cat)}"${cat === defaultTrailerCategory ? ' selected' : ''}>${esc(cat)}</option>`).join('')}
            </select>
            <svg class="trailer-cat-chevron" viewBox="0 0 24 24" width="16" height="16"><path fill="currentColor" d="M7.41 8.59 12 13.17l4.59-4.58L18 10l-6 6-6-6z"/></svg>
          </div>
        </div>
        <div class="shelf-row detail-trailers-row" id="detail-trailers-row"></div>
      </section>
    `;
  }

  const infoRows = [
    statusLabel && ['Status', statusLabel],
    release && ['Informações de lançamento', formatDetailReleaseDate(release)],
    runtime && ['Duração', runtime],
    ageRating && ['Classificação', ageRating],
    m.country && ['País de origem', m.country],
    m.language && ['Idioma original', String(m.language).toUpperCase()],
  ].filter(Boolean);

  if (infoRows.length > 0) {
    const infoTitle = isSeriesLike ? 'Detalhes da série' : 'Detalhes do filme';
    bodyHtml += `
      <section>
        <h3 class="detail-section-title">${infoTitle}</h3>
        <div class="detail-info-table">
          ${infoRows.map(([k, v]) => `
            <div class="detail-info-row">
              <span class="detail-info-label">${esc(k)}</span>
              <span class="detail-info-value">${esc(v)}</span>
            </div>
          `).join('')}
        </div>
      </section>
    `;
  }

  if (!isSeriesLike) {
    bodyHtml += metadataLandscapeShelf(m.collectionName || 'Coleção', m.tmdbCollection);
  }
  if (typeof simklUses !== 'function' || !simklUses('similar')) {
    bodyHtml += metadataLandscapeShelf('Mais como este', m.tmdbSimilar);
  }
  bodyWrap.innerHTML = bodyHtml;
  updateDetailScrollDim();
  simklRenderDetails(preview, bodyWrap);

  const missingPhotoCards = Array.from(bodyWrap.querySelectorAll('.cast-portrait-card'))
    .filter((card) => !card.dataset.personPhoto && card.dataset.personName);
  if (missingPhotoCards.length > 0 && appSettings?.tmdb_api_key) {
    const names = [...new Set(missingPhotoCards.map((c) => c.dataset.personName))].slice(0, 18);
    invoke('resolve_people_photos', { names }).then((resolved) => {
      if (!resolved || !bodyWrap.isConnected || activeMetaContext?.id !== preview.id) return;
      missingPhotoCards.forEach((card) => {
        const entry = resolved[card.dataset.personName];
        if (!entry) return;
        if (entry.id && !card.dataset.personId) card.dataset.personId = String(entry.id);
        if (entry.photo) {
          card.dataset.personPhoto = entry.photo;
          const circle = card.querySelector('.cast-avatar-circle');
          if (circle && !circle.querySelector('img')) {
            circle.innerHTML = `<img src="${esc(entry.photo)}" alt="${esc(card.dataset.personName)}" loading="lazy" draggable="false" />`;
          }
        }
      });
    }).catch(() => {});
  }

  if (trailerGroups && defaultTrailerCategory) {
    const trailersRow = document.getElementById('detail-trailers-row');
    const trailerSelect = document.getElementById('detail-trailer-category');
    const renderTrailersForCategory = (cat) => {
      const items = trailerGroups.get(cat) || [];
      trailersRow.innerHTML = items.map((tr, idx) => `
        <button class="trailer-card" data-trailer-idx="${idx}" type="button">
          <div class="trailer-thumb-wrap">
            ${tr.thumb ? `<img src="${esc(tr.thumb)}" alt="${esc(tr.name)}" loading="lazy" draggable="false" />` : ''}
            <div class="trailer-thumb-scrim"></div>
          </div>
          <div class="trailer-card-title">${esc(tr.name)}</div>
          ${tr.year ? `<div class="trailer-card-year">${esc(tr.year)}</div>` : ''}
        </button>
      `).join('');
      trailersRow.querySelectorAll('.trailer-card').forEach((btn) => {
        btn.addEventListener('click', () => {
          if (trailersRow.dataset.suppressClick) return;
          const tr = items[Number(btn.dataset.trailerIdx)];
          if (tr?.url) invoke('open_external_url', { url: tr.url }).catch((e) => showToast(String(e)));
        });
      });
    };
    renderTrailersForCategory(defaultTrailerCategory);
    trailerSelect?.addEventListener('change', (e) => renderTrailersForCategory(e.target.value));
  }

  if (seasons.length > 0) {
    const requestedSeasons = new Set();
    let currentSeason = seasons.find((s) => s > 0) ?? seasons[0];
    const nowTs = Date.now();

    const renderSeasonEpisodes = (seasonNum) => {
      currentSeason = seasonNum;
      const eps = videos.filter((v) => (v.season ?? 1) === seasonNum);
      const grid = document.getElementById('detail-episodes-grid');
      if (seasonInfo.has(seasonNum) && appSettings?.services?.tmdb_episodes && !requestedSeasons.has(seasonNum)) {
        requestedSeasons.add(seasonNum);
        invoke('tmdb_season_episodes', { series: Number(resolvedTmdbId), season: seasonNum })
          .then(rows => {
            if (!grid.isConnected || !rows.length) return;
            mergeSeasonEpisodes(videos, rows, preview, m);
            if (!defaultEpisodeInfo) {
              defaultEpisodeInfo = videos.find(v => v.season > 0) || videos[0];
              defaultPlayId = defaultEpisodeInfo.id;
              document.querySelector('#btn-detail-play span').textContent =
                `Assistir T${defaultEpisodeInfo.season} E${defaultEpisodeInfo.episode}`;
            }
            if (currentSeason === seasonNum) renderSeasonEpisodes(seasonNum);
          }).catch(() => requestedSeasons.delete(seasonNum));
      }
      grid.innerHTML = eps.map((ep, idx) => {
        const epNum = ep.episode ?? ep.number ?? idx + 1;
        const thumb = ep.thumbnail || bg;
        const epWatched = watchedSet.has(epWatchKey(preview.id, seasonNum, epNum));
        const releasedTs = ep.released ? Date.parse(ep.released) : NaN;
        const notReleased = Number.isFinite(releasedTs) && releasedTs > nowTs;
        const blurred = appSettings?.blur_unwatched_episodes && !epWatched;
        return `
          <button class="episode-card" data-ep-idx="${idx}">
            <div class="episode-thumb-wrap">
              ${thumb ? `<img class="${blurred ? 'blurred-img' : ''}" src="${esc(thumb)}" alt="" loading="lazy" decoding="async" draggable="false" />` : ''}
              <span class="episode-ep-tag">T${seasonNum} E${epNum}</span>
              ${notReleased ? '<span class="episode-unavail-badge">Em breve</span>' : ''}
              ${epWatched ? '<span class="watched-badge" title="Assistido">&#10003;</span>' : ''}
            </div>
            <div class="episode-card-body">
              <div class="episode-card-title">${esc(ep.title || ep.name || `Episódio ${epNum}`)}</div>
              ${ep.overview || ep.description ? `<div class="episode-card-desc">${esc(ep.overview || ep.description)}</div>` : ''}
            </div>
          </button>
        `;
      }).join('');

      grid.querySelectorAll('[data-ep-idx]').forEach((card) => {
        const ep = eps[Number(card.dataset.epIdx)];
        const epNum = ep.episode ?? ep.number ?? 1;
        card.addEventListener('click', () => {
          if (grid.dataset.suppressClick) return;
          openStreamsScreen({
            mediaType: preview.type,
            videoId: ep.id || preview.id,
            contentId: preview.id,
            tmdbId: resolvedTmdbId ? String(resolvedTmdbId) : null,
            title,
            logo,
            background: bg,
            poster: m.poster || preview.poster || '',
            episode: ep,
            videos,
            resumePositionSecs: 0,
          });
        });
        card.addEventListener('contextmenu', async (e) => {
          e.preventDefault();
          await toggleItemWatched(preview.id, preview.type, title, seasonNum, epNum);
          renderSeasonEpisodes(seasonNum);
        });
      });
    };

    document.getElementById('detail-season-pills').querySelectorAll('[data-season]').forEach((pill) => {
      pill.addEventListener('click', () => {
        document.getElementById('detail-season-pills').querySelectorAll('.season-pill').forEach((p) => p.classList.remove('active'));
        pill.classList.add('active');
        renderSeasonEpisodes(Number(pill.dataset.season));
      });
    });

    document.getElementById('btn-season-mark-watched').onclick = async () => {
      const eps = videos.filter((v) => (v.season ?? 1) === currentSeason);
      const progressSource = activeProgressSource();
      if (progressSource === 'mdblist') {
        const unwatched = eps.filter((ep) => !watchedSet.has(epWatchKey(preview.id, currentSeason, ep.episode ?? ep.number ?? 1)));
        if (!unwatched.length) return;
        try {
          for (const ep of unwatched) {
            await invoke('mdblist_watched_mutate', {
              remove: false,
              mediaType: 'series',
              id: preview.id,
              season: currentSeason,
              episode: ep.episode ?? ep.number ?? 1,
            });
          }
          await refreshMdblistProgress(true);
          await refreshWatchedKeys();
          renderSeasonEpisodes(currentSeason);
        } catch (e) { showToast(String(e)); }
        return;
      }
      if (progressSource === 'simkl') {
        const episodes = eps.map(ep => ({number:ep.episode ?? ep.number ?? 1}))
          .filter(ep => !watchedSet.has(epWatchKey(preview.id,currentSeason,ep.number)));
        if (!episodes.length) return;
        try {
          await simklMutate('history_add', preview.id,preview.type, {
            seasons:[{number:currentSeason, episodes}],
            ...(simklFindEntry(preview.id)?.bucket === 'anime' ? {use_tvdb_anime_seasons:true} : {}),
          });
          await refreshWatchedKeys();
          renderSeasonEpisodes(currentSeason);
        } catch(e) {showToast(String(e));}
        return;
      }
      for (const ep of eps) {
        const epNum = ep.episode ?? ep.number ?? 1;
        if (!watchedSet.has(epWatchKey(preview.id, currentSeason, epNum))) {
          await toggleItemWatched(preview.id, preview.type, title, currentSeason, epNum);
        }
      }
      renderSeasonEpisodes(currentSeason);
    };

    renderSeasonEpisodes(currentSeason);
  }
  wireHorizontalShelfDrag(bodyWrap);
}

// --- STREAMS PARSER & SCREEN (StreamsTabletLayout.kt) ---
function parseStreamBadges(stream) {
  const txt = `${stream.name || ''} ${stream.title || ''} ${stream.description || ''}`;
  const pills = [];
  const res = txt.match(/\b(2160p|4K|UHD|1080p|FHD|720p|480p)\b/i);
  if (res) pills.push(res[1].toUpperCase());
  if (/\b(DV|Dolby[\s.]?Vision)\b/i.test(txt)) pills.push('DV');
  if (/\b(HDR10\+?|HDR)\b/i.test(txt)) pills.push('HDR');
  if (/\b(HEVC|H\.?265|x265)\b/i.test(txt)) pills.push('HEVC');
  if (/\b(Atmos|TrueHD|DDP|DTS-HD|AAC)\b/i.test(txt)) pills.push('AUDIO');
  if (/\b(Dublado|PT-?BR|Dual)\b/i.test(txt)) pills.push('PT-BR');
  const sizeMatch = txt.match(/(\d+(?:[.,]\d+)?\s*(?:GB|MB))/i);
  const formattedSize = sizeMatch ? sizeMatch[1].toUpperCase() : '';
  return { pills: [...new Set(pills)], formattedSize };
}

function bindHorizontalDragScroll(el) {
  if (!el || el.dataset.dragBound === '1') return;
  el.dataset.dragBound = '1';
  let down = false;
  let moved = false;
  let captured = false;
  let startX = 0;
  let startLeft = 0;

  el.addEventListener('pointerdown', (e) => {
    if (e.button !== 0) return;
    down = true;
    moved = false;
    captured = false;
    startX = e.clientX;
    startLeft = el.scrollLeft;
  });
  el.addEventListener('pointermove', (e) => {
    if (!down) return;
    const dx = e.clientX - startX;
    if (!moved && Math.abs(dx) > 5) {
      moved = true;
      el.classList.add('dragging');
      if (!captured && el.setPointerCapture) {
        try { el.setPointerCapture(e.pointerId); captured = true; } catch {}
      }
    }
    if (moved) el.scrollLeft = startLeft - dx;
  });
  const endDrag = (e) => {
    if (!down) return;
    down = false;
    el.classList.remove('dragging');
    if (captured && el.releasePointerCapture) {
      try { el.releasePointerCapture(e.pointerId); } catch {}
      captured = false;
    }
  };
  el.addEventListener('pointerup', endDrag);
  el.addEventListener('pointercancel', endDrag);
  el.addEventListener('click', (e) => {
    if (moved) {
      e.preventDefault();
      e.stopPropagation();
      moved = false;
    }
  }, true);
  el.addEventListener('wheel', (e) => {
    if (el.scrollWidth <= el.clientWidth) return;
    if (Math.abs(e.deltaY) > Math.abs(e.deltaX)) {
      el.scrollLeft += e.deltaY;
      e.preventDefault();
    }
  }, { passive: false });
}

function closeStreamActionsPopup() {
  document.getElementById('stream-actions-popup')?.classList.add('hidden');
}

document.addEventListener('pointerdown', (e) => {
  const popup = document.getElementById('stream-actions-popup');
  if (popup && !popup.classList.contains('hidden') && !popup.contains(e.target)) {
    closeStreamActionsPopup();
  }
});

document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') closeStreamActionsPopup();
});

async function openStreamsScreen(ctx) {
  if (streamUnlistenAddon) { streamUnlistenAddon(); streamUnlistenAddon = null; }
  if (streamUnlistenDone) { streamUnlistenDone(); streamUnlistenDone = null; }
  if (streamUnlistenScrapers) { streamUnlistenScrapers(); streamUnlistenScrapers = null; }
  closeStreamActionsPopup();

  activeStreamCtx = ctx;
  streamGroups = [];
  activeStreamFilter = 'all';
  activeScrapers = [];

  setTab('streams');
  reportPartyPlayerStatus('choosing', partyStreamMediaKey());
  const streamsBgMode = appSettings?.streams_bg_mode || 'cover';
  const artwork = streamsBgMode === 'default'
    ? ''
    : (ctx.background || ctx.poster || ctx.episode?.thumbnail || '');
  document.getElementById('streams-bg-image').style.backgroundImage = artwork ? `url("${artwork}")` : 'none';
  document.getElementById('streams-screen-root').classList.toggle('streams-bg-plain', !artwork);

  const epLabel = ctx.episode
    ? `T${ctx.episode.season ?? 1} E${ctx.episode.episode ?? ctx.episode.number ?? 1} - ${ctx.episode.title || ctx.episode.name || 'Episódio'}`
    : '';

  document.getElementById('streams-meta-summary').innerHTML = `
    ${ctx.logo ? `<img src="${esc(ctx.logo)}" style="max-width:100%;max-height:120px;object-fit:contain;" alt="" />` : `<h2 style="margin:0;font-size:32px;font-weight:900;text-shadow:0 2px 6px rgba(0,0,0,0.8);">${esc(ctx.title)}</h2>`}
    ${epLabel ? `<div style="font-size:16px;font-weight:700;color:rgba(255,255,255,0.9);text-shadow:0 1px 4px rgba(0,0,0,0.9);">${esc(epLabel)}</div>` : ''}
  `;

  const bannerEl = document.getElementById('streams-resume-banner');
  if (ctx.resumePositionSecs && ctx.resumePositionSecs > 5) {
    bannerEl.textContent = `Continuando de ${formatClock(ctx.resumePositionSecs)}`;
    bannerEl.classList.remove('hidden');
  } else {
    bannerEl.classList.add('hidden');
  }

  document.getElementById('streams-filter-row').innerHTML = '';
  document.getElementById('streams-active-scrapers').classList.add('hidden');
  document.getElementById('streams-cards-list').innerHTML = '';
  document.getElementById('streams-status-bar').textContent = 'Buscando streams…';

  streamUnlistenScrapers = await listen('streams:active_scrapers', (event) => {
    activeScrapers = Array.isArray(event.payload) ? event.payload : [];
    renderActiveScrapersRow();
  });

  streamUnlistenAddon = await listen('streams:addon', (event) => {
    streamGroups.push(event.payload);
    streamGroups.sort((a, b) => (a.order ?? 0) - (b.order ?? 0));
    renderStreamsView(ctx, true);
  });

  streamUnlistenDone = await listen('streams:done', () => {
    activeScrapers = [];
    renderActiveScrapersRow();
    renderStreamsView(ctx, false);
  });

  renderStreamsView(ctx, true);

  const ack = await invoke('start_stream_search', {
    mediaType: ctx.mediaType,
    id: ctx.videoId,
    tmdbId: ctx.tmdbId || null,
    season: ctx.episode?.season ?? null,
    episode: ctx.episode?.episode ?? ctx.episode?.number ?? null,
  });
  if (ack.total === 0) {
    document.getElementById('streams-status-bar').textContent = 'Nenhum addon ou plugin ativo suporta streams para este ID.';
  }
}

function renderActiveScrapersRow() {
  const el = document.getElementById('streams-active-scrapers');
  if (!activeScrapers.length) {
    el.classList.add('hidden');
    el.innerHTML = '';
    return;
  }
  const prevScroll = document.getElementById('active-scrapers-carousel')?.scrollLeft || 0;
  el.classList.remove('hidden');
  el.innerHTML = `
    <div class="active-scrapers-title">Scrapers ativos (${activeScrapers.length})</div>
    <div class="active-scrapers-row" id="active-scrapers-carousel">
      ${activeScrapers.map((name) => `<span class="scraper-shimmer-chip">${esc(name)}</span>`).join('')}
    </div>
  `;
  const row = document.getElementById('active-scrapers-carousel');
  if (row) {
    row.scrollLeft = prevScroll;
    bindHorizontalDragScroll(row);
  }
}

document.getElementById('btn-streams-back').addEventListener('click', () => {
  setTab(activeMetaContext ? 'details' : (previousTab || 'home'));
});

function renderStreamsView(ctx, stillSearching) {
  const filterRow = document.getElementById('streams-filter-row');
  const statusEl = document.getElementById('streams-status-bar');
  const listEl = document.getElementById('streams-cards-list');
  const prevFilterScroll = filterRow.scrollLeft;

  const totalCount = streamGroups.reduce((acc, g) => acc + g.streams.length, 0);
  statusEl.textContent = stillSearching
    ? (totalCount > 0 ? `${totalCount} stream(s) carregadas…` : 'Buscando nos addons…')
    : (totalCount > 0 ? '' : 'Nenhuma stream encontrada nos addons ativos.');

  filterRow.innerHTML = [
    `<button class="streams-refresh-btn ${stillSearching ? 'spin-icon' : ''}" id="btn-streams-refresh" title="Atualizar streams">${SVG_REFRESH}</button>`,
    `<button class="streams-provider-pill ${activeStreamFilter === 'all' ? 'active' : ''}" data-filter="all">Todos</button>`,
    ...streamGroups.map((g) => `<button class="streams-provider-pill ${activeStreamFilter === g.addon_id ? 'active' : ''}" data-filter="${esc(g.addon_id)}">${esc(g.addon_name)}</button>`),
  ].join('');
  filterRow.scrollLeft = prevFilterScroll;
  bindHorizontalDragScroll(filterRow);

  document.getElementById('btn-streams-refresh')?.addEventListener('click', () => openStreamsScreen(ctx));
  filterRow.querySelectorAll('[data-filter]').forEach((btn) => {
    btn.addEventListener('click', () => {
      activeStreamFilter = btn.dataset.filter;
      renderStreamsView(ctx, stillSearching);
    });
  });

  const showLogo = appSettings?.show_addon_logo !== false;
  const showSize = appSettings?.show_file_size_badge !== false;
  const badgesTop = appSettings?.stream_badges_top !== false;

  const visibleGroups = streamGroups.filter(
    (g) => activeStreamFilter === 'all' || g.addon_id === activeStreamFilter,
  );
  const flatStreams = [];
  listEl.innerHTML = visibleGroups.map((g) => {
    const cardsHtml = g.streams.map((s) => {
      const idx = flatStreams.length;
      flatStreams.push({ stream: s, group: g });
      const streamTitle = (s.name || '').trim() || (s.title || '').split('\n')[0] || g.addon_name;
      const desc = (s.description || (s.name ? s.title : '') || s.url || '').trim();
      const playable = s.url || (s.infoHash ? `magnet:?xt=urn:btih:${s.infoHash}` : '');
      const { pills, formattedSize } = parseStreamBadges(s);
      const badgesHtml = showSize && (pills.length || formattedSize)
        ? `<div class="stream-quality-pills">
            ${pills.map((p) => `<span class="stream-q-pill">${esc(p)}</span>`).join('')}
            ${formattedSize ? `<span class="stream-q-pill size">${esc(formattedSize)}</span>` : ''}
          </div>`
        : '';

      const iconHtml = g.addon_logo
        ? `<img src="${esc(g.addon_logo)}" alt="" decoding="async" onerror="this.style.display='none';this.nextElementSibling.style.display='flex';" /><span style="display:none;align-items:center;justify-content:center;">${ADDON_PUZZLE_SVG}</span>`
        : ADDON_PUZZLE_SVG;

      return `
        <button class="stream-card-btn" data-stream-idx="${idx}" ${playable ? '' : 'disabled'}>
          <div class="stream-card-main">
            ${badgesTop ? badgesHtml : ''}
            <div class="stream-quality-name">${esc(streamTitle)}</div>
            ${desc ? `<div class="stream-desc-col">${esc(desc)}</div>` : ''}
            ${!badgesTop ? badgesHtml : ''}
          </div>
          ${showLogo ? `
            <div class="stream-card-provider">
              <div class="stream-provider-icon-wrap">${iconHtml}</div>
              <span class="stream-provider-label">${esc(g.addon_name)}</span>
            </div>
          ` : ''}
        </button>
      `;
    }).join('');
    return `
      <div class="stream-group-block">
        <div class="stream-group-header">${esc(g.addon_name)}</div>
        <div class="stream-group-cards">${cardsHtml}</div>
      </div>
    `;
  }).join('');

  const playEntry = (s, g, url, headers, epSubtitle, forceExternal) => {
    if (forceExternal) {
      launchConnectedExternal(url, headers, s, ctx).catch((err) => showToast(String(err)));
      return;
    }
    startInAppPlayback({
      fileIdx: s.fileIdx,
      url,
      headers,
      title: ctx.title,
      subtitle: epSubtitle,
      mediaType: ctx.mediaType,
      contentId: ctx.contentId || ctx.videoId,
      videoId: ctx.videoId,
      tmdbId: ctx.tmdbId || null,
      season: ctx.season ?? ctx.episode?.season ?? null,
      episode: typeof ctx.episode === 'number'
        ? ctx.episode
        : (ctx.episode?.episode ?? ctx.episode?.number ?? null),
      episodeThumbnail: ctx.episode?.thumbnail || '',
      episodeTitle: ctx.episode?.title || ctx.episode?.name || '',
      episodeOverview: ctx.episode?.overview || ctx.episode?.description || '',
      background: ctx.background || '',
      poster: ctx.poster || '',
      logo: ctx.logo || '',
      streamTitle: s.name || s.title || g.addon_name,
      providerName: g.addon_name,
      providerLogo: g.addon_logo || '',
      addonId: g.addon_id,
      startSecs: ctx.resumePositionSecs || 0,
      videos: ctx.videos || [],
    });
  };

  listEl.querySelectorAll('[data-stream-idx]').forEach((btn) => {
    const entry = flatStreams[Number(btn.dataset.streamIdx)];
    const s = entry.stream;
    const g = entry.group;
    const url = s.url || `magnet:?xt=urn:btih:${s.infoHash}`;
    const headers = s.behaviorHints?.proxyHeaders?.request || null;
    const epSubtitle = ctx.episode
      ? `T${ctx.episode.season ?? 1} E${ctx.episode.episode ?? ctx.episode.number ?? 1} • ${ctx.episode.title || ctx.episode.name || ''}`
      : (s.name || '');

    btn.addEventListener('click', () => {
      playEntry(s, g, url, headers, epSubtitle, Boolean(appSettings?.external_player_enabled));
    });

    btn.addEventListener('contextmenu', (e) => {
      e.preventDefault();
      if (!url) return;
      const popup = document.getElementById('stream-actions-popup');
      if (!popup) return;
      const icoCopy = '<svg viewBox="0 0 24 24" width="16" height="16"><path fill="currentColor" d="M16 1H4a2 2 0 0 0-2 2v14h2V3h12zm3 4H8a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h11a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2m0 16H8V7h11z"/></svg>';
      const icoExt = '<svg viewBox="0 0 24 24" width="16" height="16"><path fill="currentColor" d="M14 3v2h3.59l-9.83 9.83l1.41 1.41L19 6.41V10h2V3m-2 16H5V5h7V3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7h-2z"/></svg>';
      const icoInt = '<svg viewBox="0 0 24 24" width="16" height="16"><path fill="currentColor" d="M8 5v14l11-7z"/></svg>';
      popup.innerHTML = `
        <button type="button" class="stream-action-row" data-sa="copy">${icoCopy}<span>Copiar link</span></button>
        <button type="button" class="stream-action-row" data-sa="internal">${icoInt}<span>Abrir em player interno</span></button>
        <button type="button" class="stream-action-row" data-sa="external">${icoExt}<span>Abrir em player externo</span></button>
      `;
      popup.classList.remove('hidden');
      const pw = popup.offsetWidth || 244;
      const ph = popup.offsetHeight || 126;
      const left = Math.max(8, Math.min(e.clientX, window.innerWidth - pw - 8));
      const top = Math.max(8, Math.min(e.clientY, window.innerHeight - ph - 8));
      popup.style.left = `${left}px`;
      popup.style.top = `${top}px`;

      popup.querySelector('[data-sa="copy"]').onclick = async () => {
        closeStreamActionsPopup();
        await navigator.clipboard?.writeText(url);
        showToast('Link da stream copiado');
      };
      popup.querySelector('[data-sa="internal"]').onclick = () => {
        closeStreamActionsPopup();
        playEntry(s, g, url, headers, epSubtitle, false);
      };
      popup.querySelector('[data-sa="external"]').onclick = () => {
        closeStreamActionsPopup();
        playEntry(s, g, url, headers, epSubtitle, true);
      };
    });
  });
}

// --- EMBEDDED IN-APP LIBMPV PLAYER + OFFICIAL OVERLAY CONTROLLER ---
let playerStateUnlisten = null;
let playerUiEventUnlisten = null;
let playerCurrentPaused = false;
let playerSeeking = false;
let playerActive = false;
let partyLive = { active: false, members: [], playback: null, serverOffsetMs: 0, chatMessages: [], chatUnread: 0, chatReady: false };
let lastPartyReport = 0;
let lastPartyReportPos = -1;
let lastPartyReportPaused = null;
let lastPartySyncCheck = 0;
let partySyncGuardUntil = 0;
let partyLastAppliedSeq = null;
let lastPartyWaiting = false;
let lastPartyStatusSent = '';
let partyNotice = { text: '', token: 0, pillUntil: 0 };

function decodePartyBase64Url(encoded) {
  const pad = '='.repeat((4 - (encoded.length % 4)) % 4);
  const b64 = (encoded + pad).replace(/-/g, '+').replace(/_/g, '/');
  const bin = atob(b64);
  const bytes = Uint8Array.from(bin, (c) => c.charCodeAt(0));
  return new TextDecoder().decode(bytes);
}

function newPartyClientId() {
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
  let out = '';
  for (let i = 0; i < 20; i += 1) {
    out += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return out;
}

function formatPartyChatRows(messages, members) {
  const byId = new Map((members || []).map((m) => [m.id, m]));
  const gapMs = 5 * 60 * 1000;
  return (messages || []).map((msg, idx, arr) => {
    const prev = idx > 0 ? arr[idx - 1] : null;
    const next = idx + 1 < arr.length ? arr[idx + 1] : null;
    const showTime = !prev || ((Number(msg.sentAt) || 0) - (Number(prev.sentAt) || 0)) > gapMs;
    const nextShowsTime = Boolean(next) && ((Number(next.sentAt) || 0) - (Number(msg.sentAt) || 0)) > gapMs;
    const author = msg.memberId || msg.name || '';
    const prevAuthor = prev ? (prev.memberId || prev.name || '') : '';
    const nextAuthor = next ? (next.memberId || next.name || '') : '';
    const member = byId.get(msg.memberId);
    const displayName = msg.name || member?.name || 'Convidado';
    const ts = Number(msg.sentAt) || 0;
    const timeStr = showTime && ts > 0
      ? new Date(ts).toLocaleTimeString('pt-BR', { hour: '2-digit', minute: '2-digit' })
      : '';
    return {
      key: msg.key || String(msg.id || idx),
      memberId: msg.memberId || '',
      name: displayName,
      initial: displayName.charAt(0).toUpperCase(),
      imageUrl: msg.avatarUrl || member?.avatar || '',
      color: msg.avatarColor || member?.color || '#1E88E5',
      body: msg.body || '',
      time: timeStr,
      mine: Boolean(msg.mine),
      sent: Boolean(msg.sent),
      failed: Boolean(msg.failed),
      startsGroup: showTime || prevAuthor !== author,
      endsGroup: !next || nextShowsTime || nextAuthor !== author,
      showTime,
    };
  });
}

function activePartyProfile() {
  const profiles = authState?.profiles || [];
  return profiles.find((p) => p.profile_index === authState?.active_profile_index) || profiles[0];
}

function partyIdentity() {
  return {
    displayName: activePartyProfile()?.name || 'Você',
    avatarUrl: activePartyProfile()?.avatar_url || null,
    avatarColor: activePartyProfile()?.avatar_color_hex || null,
  };
}

function partyMediaKey() {
  const c = activePlayerCtx;
  if (!c) return '';
  return [String(c.mediaType || '').toLowerCase(), c.contentId || c.videoId || '', c.season ?? '', c.episode ?? ''].join('|');
}

function isPartyMediaCompatible(expected, candidate) {
  if (!expected || !candidate || expected === candidate) return true;
  const e = String(expected).split('|');
  const c = String(candidate).split('|');
  if (e.length >= 4 && c.length >= 4) {
    const eType = e[0] === 'tv' ? 'series' : e[0];
    const cType = c[0] === 'tv' ? 'series' : c[0];
    if (eType !== cType || e[2] !== c[2] || e[3] !== c[3]) return false;
    const eTmdb = e[1].startsWith('tmdb:');
    const cTmdb = c[1].startsWith('tmdb:');
    const eImdb = e[1].startsWith('tt');
    const cImdb = c[1].startsWith('tt');
    if ((eTmdb && cImdb) || (eImdb && cTmdb)) return true;
  }
  return false;
}

function partyStreamMediaKey() {
  if (playerActive && activePlayerCtx) return partyMediaKey();
  const s = activeStreamCtx;
  if (!s) return '';
  const ep = typeof s.episode === 'number' ? s.episode : (s.episode?.episode ?? s.episode?.number ?? '');
  const sea = s.season ?? s.episode?.season ?? '';
  return [String(s.mediaType || '').toLowerCase(), s.contentId || s.videoId || '', sea ?? '', ep ?? ''].join('|');
}

function reportPartyPlayerStatus(status, mediaKey = null) {
  if (!partyLive.active) {
    lastPartyStatusSent = '';
    return;
  }
  const key = mediaKey !== null ? mediaKey : (playerActive ? partyMediaKey() : partyStreamMediaKey());
  const sig = `${partyLive.code || ''}|${status}|${key}`;
  if (sig === lastPartyStatusSent) return;
  lastPartyStatusSent = sig;
  invoke('party_set_status', { status, mediaKey: key || null }).catch(() => {});
}

function partyMediaFromCtx() {
  const c = activePlayerCtx;
  if (!c) return null;
  return {
    type: c.mediaType || 'movie',
    meta_id: c.contentId || c.videoId || '',
    meta_type: c.mediaType || 'movie',
    video_id: c.videoId || null,
    season: c.season ?? null,
    episode: c.episode ?? null,
    title: c.title || c.name || c.streamTitle || 'Assistindo junto',
    episode_title: c.episodeTitle || null,
    poster: c.poster || null,
    background: c.background || null,
  };
}

function watchPartyState() {
  const profile = activePartyProfile();
  const name = profile?.name || 'Você';
  const members = partyLive.members || [];
  const max = Number(partyLive.maxMembers) || 3;
  const active = Boolean(partyLive.active);
  const isHost = Boolean(partyLive.isHost);
  const pillActive = active && partyNotice.pillUntil > Date.now() && partyNotice.text;
  const waitingNames = (partyLive.waitingFor || []).filter(Boolean);
  const waitingMsg = waitingNames.length
    ? `Aguardando ${waitingNames.join(', ')} escolher/carregar o player…`
    : '';
  const activeToastMsg = partyLive.error || (partyNotice.token && partyNotice.pillUntil > Date.now() ? partyNotice.text : '');
  const activeToastToken = partyLive.error ? (partyLive.errorToken || 0) : (partyNotice.token || 0);
  const memberStatusLabel = (m) => {
    if (m.status === 'choosing') return 'Escolhendo player';
    if (m.status === 'loading') return 'Carregando';
    if (m.status === 'buffering') return 'Em buffer';
    return m.isHost ? 'Anfitrião' : 'Pronto';
  };
  return {
    available: true,
    active,
    isHost,
    canEnd: isHost,
    hostOnly: Boolean(partyLive.hostOnly),
    connected: Boolean(partyLive.connected),
    code: partyLive.code || '',
    maxMembers: max,
    seatsLabel: active ? `${members.length} de ${max}` : '',
    nowWatching: partyLive.title || '',
    pillPinned: active,
    pillMessage: active ? (waitingMsg || (pillActive ? partyNotice.text : `Assistindo junto · ${partyLive.code || ''}`)) : '',
    message: activeToastMsg,
    messageToken: activeToastToken,
    members: members.map((m) => {
      const isSelf = Boolean(partyLive.memberId && m.id === partyLive.memberId);
      const rawName = m.name || 'Convidado';
      return {
        id: m.id,
        name: isSelf ? `${rawName} (você)` : rawName,
        initial: rawName.charAt(0).toUpperCase(),
        color: m.color || '#1E88E5',
        imageUrl: m.avatar || '',
        presence: partyLive.connected ? 'online' : 'offline',
        status: memberStatusLabel(m),
        isHost: Boolean(m.isHost),
        canRemove: Boolean(isHost && !isSelf),
        removeTitle: `Remover ${rawName}`,
      };
    }),
    chat: {
      unread: Number(partyLive.chatUnread) || 0,
      ready: active,
      messages: formatPartyChatRows(partyLive.chatMessages || [], members),
    },
    profile: {
      name,
      initial: name.charAt(0).toUpperCase(),
      color: profile?.avatar_color_hex || '#1E88E5',
      imageUrl: profile?.avatar_url || '',
    },
    labels: {
      title: 'Watch party',
      close: 'Fechar',
      startTitle: 'Assista junto',
      startDescription: 'Crie uma sala e convide quem quiser assistir com você, com reprodução sincronizada.',
      startAction: 'Criar sala',
      joinPlaceholder: 'Código da sala',
      joinAction: 'Entrar',
      experimental: 'Recurso experimental',
      codeLabel: 'Código da sala',
      reconnecting: 'Reconectando…',
      copyInvite: 'Copiar código',
      openSeat: 'Convidar participante',
      host: 'Anfitrião',
      hostOnlyTitle: 'Somente o anfitrião controla',
      hostOnlyDescription: 'Impede que convidados pausem ou adiantem o vídeo.',
      hostOnlyNote: 'Somente o anfitrião pode controlar a reprodução.',
      shareTitle: 'Compartilhar reprodução',
      shareDescription: 'Mostra na sala o que você está assistindo.',
      catchUp: 'Sincronizar com o anfitrião',
      watchWithParty: 'Assistir com a sala',
      leave: 'Sair da sala',
      end: 'Encerrar sala',
      endTitle: 'Encerrar Watch party?',
      endMessage: 'Todos os participantes serão desconectados.',
      cancel: 'Cancelar',
      remove: 'Remover',
      removeMessage: 'Remover este participante da sala?',
      tabParty: 'Sala',
      tabChat: 'Chat',
      chatPlaceholder: 'Mensagem…',
      chatSend: 'Enviar',
      chatEmptyTitle: 'Sem mensagens',
      chatEmptyMessage: 'Converse com quem está na sala.',
      chatNotSent: 'Não enviada. Toque para reenviar.',
      chatNewMessages: 'Novas mensagens',
    },
    startNote: '',
  };
}

function renderPartySettings() {
  const status = document.getElementById('party-status');
  const pageActive = document.getElementById('party-page-active');
  const codeRow = document.getElementById('party-code-row');
  const code = document.getElementById('party-code');
  const membersRow = document.getElementById('party-members-row');
  const members = document.getElementById('party-members');
  const activeRow = document.getElementById('party-active-row');
  const endBtn = document.getElementById('btn-party-end');
  if (!status) return;
  const view = partyLive;
  if (pageActive) pageActive.hidden = !view.active && !view.error;
  if (view.active) {
    const count = (view.members || []).length;
    status.textContent = view.isHost
      ? `Você é o anfitrião · ${view.title || 'sala ativa'}${view.connected ? '' : ' · reconectando…'}`
      : `Na sala${view.title ? ` · ${view.title}` : ''}${view.connected ? '' : ' · reconectando…'}`;
    codeRow.hidden = false;
    code.textContent = view.code || '';
    membersRow.hidden = false;
    members.textContent = (view.members || [])
      .map((m) => `${m.name}${m.isHost ? ' (anfitrião)' : ''}`)
      .join(' · ') || `${count} participante(s)`;
    activeRow.hidden = false;
    endBtn.hidden = !view.isHost;
    if (!view.isHost && view.hostOnly) status.textContent += ' · somente o anfitrião controla';
  } else {
    status.textContent = view.error || 'Nenhuma sala ativa.';
    codeRow.hidden = true;
    membersRow.hidden = true;
    activeRow.hidden = true;
  }
}

function formatPartyClock(secs) {
  const total = Math.max(0, Math.floor(Number(secs) || 0));
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  const pad = (n) => String(n).padStart(2, '0');
  return h > 0 ? `${h}:${pad(m)}:${pad(s)}` : `${m}:${pad(s)}`;
}

function partyActionNoticeText(prev, next, view) {
  if (!prev || !next || !next.memberId || next.memberId === view.memberId) return '';
  if ((prev.mediaKey || '') !== (next.mediaKey || '')) return '';
  const actor = (view.members || []).find((m) => m.id === next.memberId)?.name || 'Alguém';
  if (Boolean(prev.paused) !== Boolean(next.paused)) {
    return next.paused ? `${actor} pausou` : `${actor} retomou`;
  }
  const prevElapsed = prev.paused ? 0 : Math.max(0, (Number(next.atMs) || 0) - (Number(prev.atMs) || 0)) * (Number(prev.rate) || 1);
  const prevExpected = (Number(prev.positionMs) || 0) + prevElapsed;
  if (Math.abs((Number(next.positionMs) || 0) - prevExpected) > 2000) {
    return `${actor} foi para ${formatPartyClock((Number(next.positionMs) || 0) / 1000)}`;
  }
  return '';
}

function applyPartyState(view) {
  if (!view) return;
  const previousError = partyLive.error || '';
  if (partyLive.active && view.active && partyLive.code === view.code) {
    const prevMembers = partyLive.members || [];
    const nextMembers = view.members || [];
    const prevIds = new Set(prevMembers.map((m) => m.id));
    const nextIds = new Set(nextMembers.map((m) => m.id));
    const joined = nextMembers.find((m) => !prevIds.has(m.id) && m.id !== view.memberId);
    const left = prevMembers.find((m) => !nextIds.has(m.id) && m.id !== view.memberId);
    if (joined) {
      partyNotice = { text: `${joined.name || 'Alguém'} entrou na sala.`, token: Date.now(), pillUntil: Date.now() + 4000 };
    } else if (left) {
      partyNotice = { text: `${left.name || 'Alguém'} saiu da sala.`, token: Date.now(), pillUntil: Date.now() + 4000 };
    } else {
      const actionMsg = partyActionNoticeText(partyLive.playback, view.playback, view);
      if (actionMsg) {
        partyNotice = { text: actionMsg, token: Date.now(), pillUntil: Date.now() + 3000 };
      }
    }
  } else if (!view.active) {
    partyNotice = { text: '', token: 0, pillUntil: 0 };
    partyLastAppliedSeq = null;
    lastPartyReportPos = -1;
    lastPartyReportPaused = null;
    lastPartyWaiting = false;
    lastPartyStatusSent = '';
  }
  partyLive = {
    ...view,
    errorToken: view.error && view.error !== previousError ? Date.now() : partyLive.errorToken,
  };
  renderPartySettings();
  if (playerActive) pushPlayerControls();
  if (!view.active && playerActive) {
    invoke('player_set_speed', { speed: 1 }).catch(() => {});
    return;
  }
  syncPartyPlayback(view, true);
}

// Follow party playback (PartySync.kt): pause + seek on >0.35s paused drift, seek on >1.2s playing drift, rate nudge below that.
function syncPartyPlayback(view, fromRemote = false) {
  if (!view?.active) return;
  if (!playerActive || !activePlayerCtx) return;
  const st = activePlayerCtx.latestState;
  if (!st || !st.active || st.loading) return;
  const playback = view.playback;
  if (!playback || !playback.atMs) return;
  const isWaiting = Boolean(playback.waiting) || Boolean(view.waitingFor && view.waitingFor.length > 0);
  const waitingChanged = isWaiting !== lastPartyWaiting;
  const isOriginator = Boolean(playback.memberId && view.memberId && playback.memberId === view.memberId);
  if (isOriginator && !isWaiting && !waitingChanged && (partyLastAppliedSeq !== null || view.isHost)) return;
  if (!isWaiting && !waitingChanged && view.hostOnly && playback.memberId && view.hostMemberId && playback.memberId !== view.hostMemberId) return;

  const key = partyMediaKey();
  if (playback.mediaKey && key && !isPartyMediaCompatible(playback.mediaKey, key)) {
    if (fromRemote) notifyPlayerToast('O anfitrião está em outro episódio.');
    return;
  }

  lastPartyWaiting = isWaiting;
  const playbackIdentity = `${playback.seq ?? ''}:${playback.memberId ?? ''}`;
  const isNewEvent = playbackIdentity !== partyLastAppliedSeq;
  if (fromRemote && isNewEvent) partyLastAppliedSeq = playbackIdentity;

  const rate = Number(playback.rate) || 1;
  const serverNow = Date.now() + (Number(view.serverOffsetMs) || 0);
  const effectivePaused = isWaiting || Boolean(playback.paused);
  const elapsed = effectivePaused ? 0 : Math.max(0, serverNow - Number(playback.atMs)) * rate;
  const target = Math.max(0, (Number(playback.positionMs) + elapsed) / 1000);
  const current = Number(st.position_secs) || 0;
  const drift = target - current;

  if (effectivePaused) {
    if (!st.paused) {
      partySyncGuardUntil = Date.now() + 900;
      invoke('player_set_paused', { paused: true }).catch(() => {});
    }
    if (Math.abs(drift) > 0.35) {
      partySyncGuardUntil = Date.now() + 900;
      invoke('player_seek', { secs: target }).catch(() => {});
    }
    if ((Number(st.speed) || 1) !== rate) {
      invoke('player_set_speed', { speed: rate }).catch(() => {});
    }
    return;
  }

  if (st.paused) {
    partySyncGuardUntil = Date.now() + 900;
    invoke('player_set_paused', { paused: false }).catch(() => {});
  }
  if (Math.abs(drift) > 1.2) {
    partySyncGuardUntil = Date.now() + 900;
    invoke('player_seek', { secs: target }).catch(() => {});
  } else if (Math.abs(drift) > 0.2) {
    invoke('player_set_speed', { speed: drift > 0 ? rate * 1.05 : rate * 0.95 }).catch(() => {});
  } else if (Math.abs(drift) < 0.08 && Math.abs((Number(st.speed) || 1) - rate) > 0.01) {
    invoke('player_set_speed', { speed: rate }).catch(() => {});
  }
}

function partyReportTick(state, force = false) {
  if (!partyLive.active || !state || !state.active || state.loading) return;
  if (partyLive.waitingFor?.length || partyLive.playback?.waiting) return;
  const canControl = Boolean(!partyLive.hostOnly || partyLive.isHost);
  if (!canControl) return;
  if (!force && Date.now() < partySyncGuardUntil) return;
  const now = Date.now();
  const posSecs = Number(state.position_secs) || 0;
  const paused = Boolean(state.paused);
  const rate = Number(state.speed) || 1;
  const pausedChanged = lastPartyReportPaused !== null && lastPartyReportPaused !== paused;
  const dtSecs = lastPartyReport > 0 ? Math.max(0, (now - lastPartyReport) / 1000) : 0;
  const expectedPos = lastPartyReportPos < 0
    ? posSecs
    : (lastPartyReportPaused ? lastPartyReportPos : lastPartyReportPos + dtSecs * rate);
  const jumped = lastPartyReportPos >= 0 && Math.abs(posSecs - expectedPos) > 1.5;
  if (!force && lastPartyReportPos >= 0 && !pausedChanged && !jumped && now - lastPartyReport < 8000) return;
  lastPartyReport = now;
  lastPartyReportPos = posSecs;
  lastPartyReportPaused = paused;
  invoke('party_report_playback', {
    positionMs: Math.round(posSecs * 1000),
    paused,
    rate,
    mediaKey: partyMediaKey(),
  }).catch(() => {});
}

async function partyCreate() {
  const media = partyMediaFromCtx();
  if (!media) {
    showToast('Abra um vídeo antes de criar a sala.');
    return;
  }
  const status = document.getElementById('party-status');
  if (status) status.textContent = 'Criando sala…';
  try {
    lastPartyStatusSent = '';
    applyPartyState(await invoke('party_create', { ...partyIdentity(), media }));
    if (playerActive && activePlayerCtx?.latestState?.active) {
      const st = activePlayerCtx.latestState;
      reportPartyPlayerStatus(st.loading ? (activePlayerCtx.hadFirstReady ? 'buffering' : 'loading') : 'ready', partyMediaKey());
      if (!st.loading) partyReportTick(st, true);
    }
  } catch (error) {
    const msg = String(error);
    if (status) status.textContent = msg;
    partyLive = { ...partyLive, error: msg, errorToken: Date.now() };
    if (playerActive) pushPlayerControls();
  }
}

async function partyJoinByCode(rawCode) {
  const code = String(rawCode || '').trim().toUpperCase();
  if (code.length < 4) {
    showToast('Informe o código da sala.');
    return;
  }
  const status = document.getElementById('party-status');
  if (status) status.textContent = 'Entrando…';
  try {
    const view = await invoke('party_join', { code, ...partyIdentity() });
    lastPartyStatusSent = '';
    applyPartyState(view);
    if (playerActive && activePlayerCtx?.latestState?.active) {
      const st = activePlayerCtx.latestState;
      reportPartyPlayerStatus(st.loading ? (activePlayerCtx.hadFirstReady ? 'buffering' : 'loading') : 'ready', partyMediaKey());
    } else if (!playerActive && view?.media?.meta_id) {
      const m = view.media;
      const mediaType = m.meta_type || m.type || 'movie';
      const meta = await invoke('fetch_meta_details', { mediaType, id: m.meta_id }).catch(() => null);
      const videos = Array.isArray(meta?.videos) ? meta.videos : [];
      const epObj = (m.season != null && m.episode != null)
        ? (videos.find((v) => Number(v.season) === Number(m.season) && Number(v.episode ?? v.number) === Number(m.episode))
          || { season: m.season, episode: m.episode, title: m.episode_title || '' })
        : null;
      openStreamsScreen({
        mediaType,
        contentId: m.meta_id,
        videoId: m.video_id || m.meta_id,
        tmdbId: meta?.tmdb_id || null,
        season: m.season ?? null,
        episode: epObj,
        title: meta?.name || m.title || view.title || 'Assistindo junto',
        poster: meta?.poster || m.poster || '',
        background: meta?.background || meta?.banner || m.background || '',
        logo: meta?.logo || '',
        videos,
        resumePositionSecs: 0,
      });
    } else if (activeTabName() === 'streams') {
      reportPartyPlayerStatus('choosing', partyStreamMediaKey());
    }
    return true;
  } catch (error) {
    const msg = String(error);
    if (status) status.textContent = msg;
    partyLive = { ...partyLive, error: msg, errorToken: Date.now() };
    if (playerActive) pushPlayerControls();
    return false;
  }
}

listen('party:state', (event) => applyPartyState(event.payload)).catch(() => {});
async function partyRefreshStatus() {
  try { applyPartyState(await invoke('party_status')); } catch { /* offline */ }
}

document.getElementById('btn-party-leave')?.addEventListener('click', async () => {
  try { await invoke('party_leave'); } catch (error) { showToast(String(error)); }
  partyRefreshStatus();
});
document.getElementById('btn-party-end')?.addEventListener('click', async () => {
  try { await invoke('party_end'); } catch (error) { showToast(String(error)); }
  partyRefreshStatus();
});
invoke('party_status').then(applyPartyState).catch(() => {});

/* Sessão em grupo: página de entrada por código (Ajustes → Sessão em grupo) */
function partyPageInputs() {
  return Array.from(document.querySelectorAll('#party-code-boxes .party-code-box'));
}

function partyPageCode() {
  return partyPageInputs().map((el) => el.value.trim()).join('').toUpperCase();
}

function syncPartyPageEnter() {
  const btn = document.getElementById('btn-party-page-enter');
  if (btn) btn.disabled = partyPageCode().length < 6;
}

function openGroupSessionPage() {
  const avatar = document.getElementById('party-page-avatar-self');
  if (avatar) {
    const active = (authState?.profiles || []).find((p) => p.profile_index === authState?.active_profile_index)
      || (authState?.profiles || [])[0];
    avatar.textContent = String(active?.name || 'N').trim().charAt(0).toUpperCase() || 'N';
    avatar.style.background = active?.avatar_color_hex || '';
  }
  partyPageInputs().forEach((el) => { el.value = ''; });
  const err = document.getElementById('party-page-error');
  if (err) err.textContent = '';
  syncPartyPageEnter();
  renderPartySettings();
  showView('view-party');
  setTimeout(() => partyPageInputs()[0]?.focus(), 40);
}

async function partyPageEnterClick() {
  const code = partyPageCode();
  if (code.length < 6) return;
  const btn = document.getElementById('btn-party-page-enter');
  const err = document.getElementById('party-page-error');
  btn.disabled = true;
  btn.textContent = 'Entrando…';
  err.textContent = '';
  const ok = await partyJoinByCode(code);
  btn.textContent = 'Entrar';
  if (ok) {
    showView('view-app');
    showToast('Você entrou na sessão em grupo.');
    return;
  }
  btn.disabled = false;
  err.textContent = partyLive.error || 'Não foi possível entrar. Confira o código e tente novamente.';
}

partyPageInputs().forEach((input, index, all) => {
  input.addEventListener('input', () => {
    const raw = input.value.replace(/[^a-zA-Z0-9]/g, '').toUpperCase();
    if (!raw) {
      input.value = '';
      syncPartyPageEnter();
      return;
    }
    const chars = raw.split('');
    input.value = chars[0];
    let cursor = index + 1;
    for (const ch of chars.slice(1)) {
      if (cursor >= all.length) break;
      all[cursor].value = ch;
      cursor += 1;
    }
    if (cursor < all.length) all[cursor].focus();
    else all[all.length - 1].blur();
    syncPartyPageEnter();
  });
  input.addEventListener('keydown', (event) => {
    if (event.key === 'Backspace' && !input.value && index > 0) {
      event.preventDefault();
      all[index - 1].value = '';
      all[index - 1].focus();
      syncPartyPageEnter();
    } else if (event.key === 'ArrowLeft' && index > 0) {
      event.preventDefault();
      all[index - 1].focus();
    } else if (event.key === 'ArrowRight' && index < all.length - 1) {
      event.preventDefault();
      all[index + 1].focus();
    } else if (event.key === 'Enter') {
      event.preventDefault();
      partyPageEnterClick();
    }
  });
  input.addEventListener('paste', (event) => {
    const text = String(event.clipboardData?.getData('text') || '').replace(/[^a-zA-Z0-9]/g, '').toUpperCase();
    if (!text) return;
    event.preventDefault();
    all.forEach((el, i) => { el.value = text[i] || ''; });
    syncPartyPageEnter();
    all[Math.min(text.length, all.length - 1)].focus();
  });
});

document.getElementById('btn-party-page-enter')?.addEventListener('click', partyPageEnterClick);
document.getElementById('btn-party-page-back')?.addEventListener('click', () => showView('view-app'));
document.getElementById('btn-open-group-session')?.addEventListener('click', openGroupSessionPage);
document.getElementById('btn-open-connected-services')?.addEventListener('click', () => selectSettingsSection('cloud'));
document.getElementById('btn-cloud-back')?.addEventListener('click', () => selectSettingsSection('connected'));

let activePlayerCtx = null;

function getPlayerStageBounds() {
  const stage = document.getElementById('player-video-stage');
  const r = stage.getBoundingClientRect();
  return {
    x: Math.round(r.left),
    y: Math.round(r.top),
    w: Math.max(16, Math.round(r.width)),
    h: Math.max(16, Math.round(r.height)),
  };
}

function normalizeLangCode(v) {
  return String(v || '')
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .trim()
    .toLowerCase()
    .replace(/[_\s]+/g, '-')
    .replace(/-+/g, '-')
    .replace(/^-|-$/g, '');
}

const LANG_FAMILY_ALIASES = {
  pt: ['pt', 'por', 'pob', 'ptb', 'pt-br', 'ptbr', 'pt-pt', 'portugues', 'portuguese'],
  en: ['en', 'eng', 'en-us', 'en-gb', 'english', 'ingles'],
  ja: ['ja', 'jp', 'jpn', 'japanese', 'japones'],
  es: ['es', 'spa', 'es-la', 'es-419', 'es-es', 'spanish', 'espanhol', 'espanol', 'latino', 'castellano'],
  fr: ['fr', 'fre', 'fra', 'french', 'frances'],
  de: ['de', 'ger', 'deu', 'german', 'alemao'],
  it: ['it', 'ita', 'italian', 'italiano'],
  ko: ['ko', 'kor', 'korean', 'coreano'],
  zh: ['zh', 'zho', 'chi', 'zh-cn', 'zh-tw', 'chinese', 'chines'],
  ru: ['ru', 'rus', 'russian', 'russo'],
};

const LANG_CODE_TO_FAMILY = new Map(
  Object.entries(LANG_FAMILY_ALIASES).flatMap(([family, codes]) => codes.map((c) => [c, family]))
);

const PT_BR_TOKENS = new Set(['pt-br', 'ptbr', 'pob', 'ptb', 'brasil', 'brazil', 'brazilian', 'dublado', 'portugues-br', 'portuguese-br']);
const PT_PT_TOKENS = new Set(['pt-pt', 'portugal', 'europeu', 'european', 'iberico']);

function parseLangPreference(rawToken) {
  const norm = normalizeLangCode(rawToken);
  if (!norm || norm === 'und' || norm === 'unknown' || norm === '__unknown__') return null;
  if (PT_BR_TOKENS.has(norm) || norm.includes('brasil') || norm.includes('brazil') || norm.includes('dublado')) {
    return { token: norm, family: 'pt', variant: 'br' };
  }
  if (PT_PT_TOKENS.has(norm)) {
    return { token: norm, family: 'pt', variant: 'pt' };
  }
  const family = LANG_CODE_TO_FAMILY.get(norm) || LANG_CODE_TO_FAMILY.get(norm.split('-')[0]) || norm.split('-')[0];
  const variant = norm.includes('-') ? norm.split('-')[1] : '';
  return { token: norm, family, variant };
}

function titleMatchesTokens(title, tokens) {
  const clean = String(title || '')
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .toLowerCase();
  if (!clean) return false;
  return tokens.some((tok) => {
    const escaped = tok.replace(/[.*+?^${}()|[\]\\]/g, '\\$&').replace(/-/g, '[-_\\s]*');
    return new RegExp(`(?:^|[^a-z0-9])${escaped}(?:$|[^a-z0-9])`, 'i').test(clean);
  });
}

function scoreSingleTrackPreference(track, pref) {
  if (!pref) return 0;
  const trackLang = parseLangPreference(track?.lang);
  const title = track?.title || '';
  const isUnknownLang = !trackLang;

  if (pref.family === 'pt') {
    const hasBrTitle = titleMatchesTokens(title, ['pt-br', 'ptbr', 'pob', 'ptb', 'brasil', 'brazil', 'brazilian', 'dublado']);
    const hasPtPtTitle = titleMatchesTokens(title, ['pt-pt', 'portugal', 'europeu', 'european', 'iberico']);
    const hasPtTitle = hasBrTitle || hasPtPtTitle || titleMatchesTokens(title, ['portugues', 'portuguese', 'pt']);

    if (trackLang?.family === 'pt') {
      let score = 70;
      if (trackLang.variant === 'br') score += 30;
      else if (trackLang.variant === 'pt') score -= 15;
      if (hasBrTitle) score += pref.variant === 'pt' ? 5 : 25;
      if (hasPtPtTitle) score += pref.variant === 'pt' ? 30 : -15;
      return score;
    }
    if (isUnknownLang && hasPtTitle) {
      let score = 60;
      if (hasBrTitle) score += pref.variant === 'pt' ? 0 : 25;
      if (hasPtPtTitle) score += pref.variant === 'pt' ? 25 : -10;
      return score;
    }
    return 0;
  }

  if (trackLang?.family === pref.family) {
    let score = 75;
    if (pref.variant && trackLang.variant === pref.variant) score += 25;
    else if (pref.token === trackLang.token) score += 20;
    return score;
  }

  const familyTokens = LANG_FAMILY_ALIASES[pref.family] || [pref.token];
  const titleHints = familyTokens.filter((t) => t.length > 2);
  if (isUnknownLang && titleHints.length && titleMatchesTokens(title, titleHints)) {
    return 60;
  }
  return 0;
}

function findBestPreferredTrack(tracks, prefRaw) {
  if (!Array.isArray(tracks) || !tracks.length) return null;
  const prefs = String(prefRaw || '')
    .split(/[,;]+/)
    .map(parseLangPreference)
    .filter(Boolean);
  if (!prefs.length) return null;

  let best = null;
  for (const track of tracks) {
    let trackScore = 0;
    for (let i = 0; i < prefs.length; i += 1) {
      const s = scoreSingleTrackPreference(track, prefs[i]);
      if (s > 0) {
        trackScore = Math.max(trackScore, s + Math.max(0, 5 - i));
        break;
      }
    }
    if (trackScore <= 0) continue;
    if (!best || trackScore > best.score || (trackScore === best.score && track.selected && !best.track.selected)) {
      best = { track, score: trackScore };
    }
  }
  return best;
}

function languageLabelPt(code) {
  const norm = normalizeLangCode(code);
  if (!norm || norm === 'und' || norm === '__unknown__') return 'Desconhecido';
  try {
    return new Intl.DisplayNames(['pt-BR'], { type: 'language' }).of(norm) || norm.toUpperCase();
  } catch {
    return norm.toUpperCase();
  }
}

function buildSubtitleModalData() {
  const builtIn = activePlayerCtx?.latestState?.subtitle_tracks || [];
  const addons = activePlayerCtx?.addonSubtitles || [];
  const groups = new Map();
  const options = [];

  for (const t of builtIn) {
    const langKey = normalizeLangCode(t.lang) || '__unknown__';
    const optId = `builtin:${t.id}`;
    const isSel = activePlayerCtx.activeSubOptionId === optId || (!activePlayerCtx.activeSubOptionId && t.selected);
    options.push({
      id: optId,
      kind: 'builtIn',
      languageKey: langKey,
      sourceLabel: 'Interna',
      title: t.title || languageLabelPt(langKey),
      metadata: t.forced ? 'Forçada' : (t.codec || '').toUpperCase(),
      isSelected: isSel,
      isAutoSynced: false,
      index: Number(t.id),
    });
    const cur = groups.get(langKey) || { key: langKey, label: languageLabelPt(langKey), count: 0 };
    cur.count += 1;
    groups.set(langKey, cur);
  }

  addons.forEach((sub, idx) => {
    const langKey = normalizeLangCode(sub.lang) || '__unknown__';
    const optId = `addon:${sub.id || idx}`;
    const isSel = activePlayerCtx.activeSubOptionId === optId;
    options.push({
      id: optId,
      kind: 'addon',
      languageKey: langKey,
      sourceLabel: sub.addon_name || 'Addon',
      title: sub.addon_name || 'Legenda Online',
      metadata: sub.id || '',
      isSelected: isSel,
      isAutoSynced: isSel && activePlayerCtx.autoSyncedApplied,
      index: idx,
    });
    const cur = groups.get(langKey) || { key: langKey, label: languageLabelPt(langKey), count: 0 };
    cur.count += 1;
    groups.set(langKey, cur);
  });

  const selectedOpt = options.find((o) => o.isSelected);
  const langKeySel = selectedOpt ? selectedOpt.languageKey : '__off__';

  const subtitleLanguageItems = [
    { key: '__off__', label: 'Desativada', count: 0, isSelected: langKeySel === '__off__' },
    ...Array.from(groups.values()).map((g) => ({ ...g, isSelected: g.key === langKeySel })),
  ];

  return { subtitleLanguageItems, subtitleOptionItems: options, selectedLangKey: langKeySel, selectedOptionId: selectedOpt?.id || '' };
}

function buildSourceItemsForOverlay() {
  const all = [];
  const filters = [{ id: '', label: 'Todos', isLoading: false, hasError: false }];
  for (const g of streamGroups) {
    filters.push({ id: g.addon_id, label: g.addon_name, isLoading: false, hasError: false });
    for (const s of g.streams) {
      const url = s.url || (s.infoHash ? `magnet:?xt=urn:btih:${s.infoHash}` : '');
      const { pills, formattedSize } = parseStreamBadges(s);
      const badgeStr = [formattedSize, ...pills].filter(Boolean).join(' • ');
      all.push({
        index: all.length,
        label: (s.name || '').trim() || g.addon_name,
        subtitle: (s.description || s.title || '').trim(),
        addonName: g.addon_name,
        addonLogo: g.addon_logo || '',
        showAddonLogo: appSettings?.show_addon_logo !== false,
        filterId: g.addon_id,
        isCurrent: url === activePlayerCtx?.url,
        isEnabled: Boolean(url),
        badgePlacement: appSettings?.stream_badges_top !== false ? 'TOP' : 'BOTTOM',
        formattedSize: badgeStr,
        badges: [],
        _raw: { stream: s, group: g, url },
      });
    }
  }
  return { sourceFilters: filters, sourceItems: all };
}

function buildEpisodeItemsForOverlay() {
  const videos = activePlayerCtx?.videos || [];
  if (!videos.length) return { episodeSeasons: [], episodeItems: [] };
  const seasonNums = [...new Set(videos.map((v) => v.season ?? 1))].sort((a, b) => a - b);
  const curSeason = activePlayerCtx.season ?? (seasonNums.find((s) => s > 0) ?? seasonNums[0]);
  const episodeSeasons = seasonNums.map((s) => ({
    season: s,
    label: s === 0 ? 'Especiais' : `Temp ${s}`,
    isSelected: s === curSeason,
  }));
  const episodeItems = videos.map((v, idx) => {
    const s = v.season ?? 1;
    const e = v.episode ?? v.number ?? idx + 1;
    return {
      index: idx,
      season: s,
      episode: e,
      code: `T${s} E${e}`,
      title: v.title || v.name || `Episódio ${e}`,
      overview: v.overview || v.description || '',
      thumbnail: v.thumbnail || activePlayerCtx.background || '',
      released: v.released ? String(v.released).slice(0, 10) : '',
      isWatched: watchedSet.has(epWatchKey(activePlayerCtx.contentId, s, e)),
      isCurrent: activePlayerCtx.videoId === v.id || (activePlayerCtx.season === s && activePlayerCtx.episode === e),
    };
  });
  return { episodeSeasons, episodeItems };
}

function computeNextEpisode(videos, curSeason, curEpisode) {
  if (!Array.isArray(videos) || !videos.length || curSeason == null || curEpisode == null) return null;
  const sorted = [...videos].sort((a, b) => ((a.season ?? 1) - (b.season ?? 1)) || ((a.episode ?? a.number ?? 0) - (b.episode ?? b.number ?? 0)));
  const idx = sorted.findIndex((v) => (v.season ?? 1) === curSeason && (v.episode ?? v.number) === curEpisode);
  return idx >= 0 && idx + 1 < sorted.length ? sorted[idx + 1] : null;
}

function buildPlayerControlsObject(extra = {}) {
  if (!playerActive || !activePlayerCtx) return null;
  const ctx = activePlayerCtx;
  const st = ctx.latestState || {};
  const subData = buildSubtitleModalData();
  const srcData = buildSourceItemsForOverlay();
  const epData = buildEpisodeItemsForOverlay();
  const cs = getComputedStyle(document.body);
  const accent = cs.getPropertyValue('--accent').trim() || '#F5F5F5';
  const accentVar = cs.getPropertyValue('--accent-variant').trim() || accent;
  const onAccent = cs.getPropertyValue('--on-accent').trim() || '#111111';
  const focus = cs.getPropertyValue('--focus-ring').trim() || '#FFFFFF';
  const isLoadingNow = st.loading !== undefined ? Boolean(st.loading) : true;

  const nextEp = ctx.nextEpisodeObj;
  const nextEpNum = nextEp ? (nextEp.episode ?? nextEp.number ?? 1) : null;
  const epStr = ctx.season != null && ctx.episode != null ? `T${ctx.season} E${ctx.episode}` : '';

  return {
    title: ctx.title || '',
    seekPreview: ctx.seekPreview,
    episodeText: ctx.subtitle || '',
    streamTitle: ctx.streamTitle || '',
    providerName: ctx.providerName || '',
    showOpeningOverlay: isLoadingNow,
    openingArtwork: ctx.background || ctx.poster || '',
    openingLogo: ctx.logo || '',
    openingTitle: ctx.title || '',
    openingMessage: isLoadingNow ? 'Carregando…' : '',
    pauseOverlayEnabled: appSettings?.pause_overlay_enabled !== false,
    pauseOverlayWatchingLabel: 'Você está assistindo',
    pauseOverlayLogo: ctx.logo || '',
    pauseOverlayEpisodeInfo: epStr,
    pauseOverlayEpisodeTitle: ctx.episodeTitle || '',
    pauseOverlayDescription: ctx.episodeOverview || '',
    resizeModeLabel: RESIZE_MODES[ctx.resizeMode % 4],
    playbackSpeedLabel: `${Number(st.speed || 1).toFixed(2).replace(/\.?0+$/, '')}x`,
    holdToSpeedEnabled: appSettings?.hold_to_speed_enabled !== false,
    holdToSpeedValue: appSettings?.hold_to_speed_value || 2.0,
    isFullscreen: ctx.isFullscreen || false,
    subtitlesLabel: 'Legendas',
    audioLabel: 'Áudio',
    sourcesLabel: 'Fontes',
    episodesLabel: 'Episódios',
    externalPlayerLabel: 'Externo',
    playLabel: 'Reproduzir',
    pauseLabel: 'Pausar',
    closeLabel: 'Fechar player',
    submitIntroLabel: 'Enviar Abertura',
    videoSettingsLabel: 'Ajustes de vídeo',
    sourcesPanelTitle: 'Fontes',
    episodesPanelTitle: 'Episódios',
    streamsPanelTitle: 'Streams',
    allFilterLabel: 'Todos',
    reloadLabel: 'Recarregar',
    backLabel: 'Voltar',
    panelCloseLabel: 'Fechar',
    cancelLabel: 'Cancelar',
    playingLabel: 'Reproduzindo',
    noStreamsLabel: 'Nenhuma stream encontrada',
    noEpisodesLabel: 'Nenhum episódio disponível',
    submitIntroPanelTitle: 'Enviar Tempos',
    submitIntroSegmentTypeLabel: 'TIPO DE SEGMENTO',
    submitIntroSegmentIntroLabel: 'Abertura',
    submitIntroSegmentRecapLabel: 'Recap',
    submitIntroSegmentOutroLabel: 'Encerramento',
    submitIntroStartTimeLabel: 'INÍCIO (MM:SS)',
    submitIntroEndTimeLabel: 'FIM (MM:SS)',
    submitIntroCaptureLabel: 'Capturar',
    submitIntroSubmitLabel: 'Enviar',
    audioTracksPanelTitle: 'Faixas de Áudio',
    noAudioTracksLabel: 'Nenhuma faixa de áudio',
    subtitlesPanelTitle: 'Legendas',
    subtitleLanguagesLabel: 'Idiomas',
    noneLabel: 'Desativada',
    fetchSubtitlesLabel: 'Buscar legendas nos addons',
    subtitleDelayLabel: 'Atraso de Legenda',
    resetLabel: 'Zerar',
    autoSyncLabel: 'Auto Sincronizar',
    reloadSmallLabel: 'Recarregar',
    captureLineLabel: 'Capturar fala',
    selectAddonSubtitleFirstLabel: 'Selecione uma legenda de addon primeiro',
    loadingSubtitleLinesLabel: 'Carregando falas…',
    fontSizeLabel: 'Tamanho da Fonte',
    outlineLabel: 'Contorno',
    boldLabel: 'Negrito',
    bottomOffsetLabel: 'Altura inferior',
    colorLabel: 'Cor',
    textOpacityLabel: 'Opacidade texto',
    outlineColorLabel: 'Cor contorno',
    backgroundColorLabel: 'Cor de fundo',
    backgroundOpacityLabel: 'Opacidade fundo',
    shadowLabel: 'Sombra',
    shadowOpacityLabel: 'Opacidade sombra',
    shadowSizeLabel: 'Tamanho sombra',
    resetDefaultsLabel: 'Restaurar Padrão',
    onLabel: 'Ligado',
    offLabel: 'Desligado',
    autoSyncedLabel: 'Sincronizada',
    themeAccentColor: accent,
    themeAccentStrongColor: accentVar,
    themeOnAccentColor: onAccent,
    themeFocusColor: focus,
    themeSelectedSurfaceColor: 'rgba(255, 255, 255, .14)',
    themeSelectedSurfaceHoverColor: 'rgba(255, 255, 255, .20)',
    themeSelectedRingColor: 'rgba(255, 255, 255, .35)',
    themeTimelineFillColor: accent,
    themeBufferingColor: accent,
    themeControlForegroundColor: '#FFFFFF',
    skipPromptVisible: Boolean(ctx.activeSkipInterval && !ctx.dismissedSkipKeys.has(ctx.activeSkipInterval.key)),
    skipPromptLabel: ctx.activeSkipInterval?.label || 'Pular abertura',
    skipPromptStartMs: Math.round((ctx.activeSkipInterval?.startTime || 0) * 1000),
    skipPromptEndMs: Math.round((ctx.activeSkipInterval?.endTime || 0) * 1000),
    skipPromptDismissed: Boolean(ctx.activeSkipInterval && ctx.dismissedSkipKeys.has(ctx.activeSkipInterval.key)),
    nextEpisodeVisible: Boolean(appSettings?.next_episode_enabled !== false && nextEp && ctx.showNextEpisode),
    nextEpisodeHeaderLabel: 'Próximo episódio',
    nextEpisodeTitle: nextEp ? `T${nextEp.season ?? 1} E${nextEpNum} • ${nextEp.title || nextEp.name || ''}` : '',
    nextEpisodeThumbnail: nextEp?.thumbnail || ctx.background || '',
    nextEpisodeThumbnailBlurred: Boolean(appSettings?.blur_unwatched_episodes),
    nextEpisodeStatus: ctx.nextEpisodeStatus || '',
    nextEpisodeActionLabel: 'Assistir',
    nextEpisodePlayable: Boolean(nextEp),
    subtitleLanguageItems: subData.subtitleLanguageItems,
    subtitleOptionItems: subData.subtitleOptionItems,
    selectedSubtitleLanguageKey: subData.selectedLangKey,
    selectedSubtitleOptionId: subData.selectedOptionId,
    isLoadingAddonSubtitles: ctx.loadingAddonSubtitles,
    subtitleDelayMs: ctx.subtitleDelayMs,
    hasSelectedAddonSubtitle: ctx.selectedAddonIndex >= 0,
    subtitleAutoSyncCapturedPositionMs: ctx.capturedPosMs,
    subtitleAutoSyncCues: ctx.capturedCues,
    subtitleAutoSyncIsLoading: ctx.loadingCues,
    subtitleAutoSyncErrorMessage: ctx.cuesError || '',
    subtitleStyle: ctx.subtitleStyle,
    subtitleColorSwatches: SUBTITLE_SWATCHES,
    subtitleOutlineColorSwatches: SUBTITLE_SWATCHES,
    sourceIsLoading: ctx.loadingSources,
    sourceFilters: srcData.sourceFilters,
    sourceItems: srcData.sourceItems,
    episodeSeasons: epData.episodeSeasons,
    episodeItems: epData.episodeItems,
    episodeStreamsVisible: ctx.episodeStreamsVisible,
    episodeStreamsIsLoading: ctx.loadingEpisodeStreams,
    selectedEpisodeLabel: ctx.selectedEpisodeLabel || '',
    episodeStreamFilters: ctx.episodeStreamFilters || [],
    episodeStreamItems: ctx.episodeStreamItems || [],
    blurUnwatchedEpisodes: Boolean(appSettings?.blur_unwatched_episodes),
    chapters: st.chapters || [],
    streamInfoSections: ctx.streamInfoSections || [],
    isSubmitIntroSubmitting: ctx.submittingIntro || false,
    submitIntroStatusMessage: ctx.submitIntroStatus || '',
    notificationMessage: ctx.notificationMsg || '',
    notificationToken: ctx.notificationToken || 0,
    closeModalsToken: ctx.closeModalsToken || 0,
    submitIntroSuccessToken: ctx.submitIntroSuccessToken || 0,
    controlsVisible: ctx.controlsVisible !== false,
    party: watchPartyState(),
    ...extra,
  };
}

function pushPlayerControls(extra = {}) {
  const controlsObj = buildPlayerControlsObject(extra);
  if (!controlsObj) return;
  invoke('player_push_controls', { controlsJson: JSON.stringify(controlsObj) });
}

function notifyPlayerToast(msg) {
  if (!activePlayerCtx) return;
  activePlayerCtx.notificationMsg = msg;
  activePlayerCtx.notificationToken += 1;
  pushPlayerControls();
}

async function resolveSkipForActivePlayer() {
  const ctx = activePlayerCtx;
  if (!ctx || ctx.skipFetchInFlight) return;
  const chaps = ctx.latestState?.chapters || [];
  if (ctx.skipResolvedCount === 1 && chaps.length === 0) return;
  if (ctx.skipResolvedWithChapters && chaps.length > 0) return;
  ctx.skipFetchInFlight = true;
  if (chaps.length > 0) ctx.skipResolvedWithChapters = true;
  ctx.skipResolvedCount += 1;
  try {
    const intervals = await invoke('player_resolve_skip_intervals', {
      videoId: ctx.videoId,
      tmdbId: ctx.tmdbId || null,
      mediaType: ctx.mediaType || 'movie',
      season: ctx.season ?? null,
      episode: ctx.episode ?? null,
      chapters: chaps,
    });
    if (activePlayerCtx === ctx) {
      ctx.skipIntervals = Array.isArray(intervals) ? intervals : [];
      const pos = ctx.latestState?.position_secs || 0;
      const match = ctx.skipIntervals.find((iv) => pos >= iv.startTime && pos < iv.endTime - 0.5) || null;
      if ((match?.key || '') !== (ctx.activeSkipInterval?.key || '')) {
        ctx.activeSkipInterval = match;
        pushPlayerControls();
      }
    }
  } catch {
    // Non-fatal
  } finally {
    ctx.skipFetchInFlight = false;
  }
}

async function fetchAddonSubtitlesForPlayer() {
  const ctx = activePlayerCtx;
  if (!ctx) return;
  ctx.loadingAddonSubtitles = true;
  pushPlayerControls();
  try {
    const subs = await invoke('player_fetch_addon_subtitles', {
      mediaType: ctx.mediaType || 'movie',
      id: ctx.videoId,
      streamUrl: ctx.url,
    });
    if (activePlayerCtx === ctx) {
      ctx.addonSubtitles = Array.isArray(subs) ? subs : [];
      ctx.loadingAddonSubtitles = false;
      pushPlayerControls();
    }
  } catch {
    if (activePlayerCtx === ctx) {
      ctx.loadingAddonSubtitles = false;
      pushPlayerControls();
    }
  }
}

async function syncActivePlayerWatchProgress(force = false) {
  const ctx = activePlayerCtx;
  const st = ctx?.latestState;
  if (!ctx?.contentId || !st || st.duration_secs <= 10) return;
  if (activeProgressSource() !== 'default') return;
  const now = Date.now();
  if (!force && now - (ctx.lastProgressSyncMs || 0) < 25000) return;
  ctx.lastProgressSyncMs = now;

  const progressKey = ctx.season != null && ctx.episode != null
    ? `${ctx.contentId}_s${ctx.season}e${ctx.episode}`
    : ctx.contentId;

  const posMs = Math.round(st.position_secs * 1000);
  const durMs = Math.round(st.duration_secs * 1000);

  await invoke('upsert_watch_progress', {
    entry: {
      content_id: ctx.contentId,
      content_type: ctx.mediaType || 'movie',
      video_id: ctx.videoId,
      season: ctx.season ?? null,
      episode: ctx.episode ?? null,
      position: posMs,
      duration: durMs,
      last_watched: now,
      progress_key: progressKey,
    },
    syncRemote: true,
  }).catch(() => {});

  if (posMs / durMs >= 0.92 && !ctx.autoMarkedWatched) {
    ctx.autoMarkedWatched = true;
    const wk = epWatchKey(ctx.contentId, ctx.season, ctx.episode);
    if (!watchedSet.has(wk)) {
      toggleItemWatched(ctx.contentId, ctx.mediaType, ctx.title, ctx.season, ctx.episode).catch(() => {});
    }
  }
}

async function handlePlayerUiEvent({ event_type: type, value }) {
  const ctx = activePlayerCtx;
  if (!ctx) return;

  if (type === 'controlsReady') {
    pushPlayerControls();
    return;
  }
  // Chrome visibility mirrors PlayerScreenRuntimeUi.handlePlayerControlsEvent (Kotlin host):
  // the page auto-hides itself, only the host can bring it back on pointer activity.
  if (type === 'cursorActivity' || type === 'keepChromeVisible') {
    if (!ctx.controlsVisible) {
      ctx.controlsVisible = true;
      pushPlayerControls();
    }
    return;
  }
  if (type === 'hideChrome') {
    ctx.controlsVisible = false;
    return;
  }
  if (type === 'toggleChrome') {
    ctx.controlsVisible = !ctx.controlsVisible;
    return;
  }
  if (type === 'setPlaybackState' || type === 'setPlaybackStateQuiet' || type === 'toggle' || type === 'keyboardToggle' || type === 'scrubFinish' || type === 'seekBack' || type === 'seekForward' || type === 'setPlaybackSpeed') {
    if (partyLive.active && partyLive.waitingFor?.length) {
      invoke('player_set_paused', { paused: true }).catch(() => {});
      notifyPlayerToast(`Aguardando ${partyLive.waitingFor.join(', ')} escolher/carregar o player.`);
      return;
    }
    if (partyLive.active && partyLive.hostOnly && !partyLive.isHost) {
      notifyPlayerToast('Somente o anfitrião pode controlar a reprodução.');
      syncPartyPlayback(partyLive, false);
      return;
    }
    if (type === 'setPlaybackState') {
      ctx.controlsVisible = true;
    }
    const st = ctx.latestState;
    if (partyLive.active && st) {
      let nextPaused = Boolean(st.paused);
      let nextPos = Number(st.position_secs) || 0;
      let nextRate = Number(st.speed) || 1;
      if (type === 'setPlaybackState' || type === 'setPlaybackStateQuiet') nextPaused = value < 0.5;
      else if (type === 'toggle' || type === 'keyboardToggle') nextPaused = !nextPaused;
      else if (type === 'scrubFinish') nextPos = Math.max(0, value / 1000);
      else if (type === 'seekBack') nextPos = Math.max(0, nextPos - 10);
      else if (type === 'seekForward') nextPos = Math.max(0, nextPos + 10);
      else if (type === 'setPlaybackSpeed') nextRate = Math.min(4, Math.max(0.25, value));
      partyReportTick({ ...st, paused: nextPaused, position_secs: nextPos, speed: nextRate }, true);
    }
    return;
  }
  if (type === 'back') {
    await closeEmbeddedPlayer();
    return;
  }
  if (type === 'partyStart') {
    await partyCreate();
    pushPlayerControls();
    return;
  }
  if (type.startsWith('partyJoinCode:')) {
    await partyJoinByCode(type.slice('partyJoinCode:'.length));
    pushPlayerControls();
    return;
  }
  if (type === 'partyCopyInvite') {
    if (partyLive.code) {
      navigator.clipboard?.writeText(partyLive.code).catch(() => {});
      notifyPlayerToast(`Código copiado: ${partyLive.code}`);
    }
    return;
  }
  if (type === 'partyLeave') {
    try { await invoke('party_leave'); } catch (error) { showToast(String(error)); }
    await partyRefreshStatus();
    pushPlayerControls();
    return;
  }
  if (type === 'partyEnd') {
    try { await invoke('party_end'); } catch (error) { showToast(String(error)); }
    await partyRefreshStatus();
    pushPlayerControls();
    return;
  }
  if (type === 'partyCatchUp' || type === 'partyWatch') {
    partyLastAppliedSeq = null;
    syncPartyPlayback(partyLive, true);
    pushPlayerControls();
    return;
  }
  if (type === 'partyHostOnly') {
    try {
      applyPartyState(await invoke('party_set_host_only', { hostOnly: value >= 0.5 }));
      pushPlayerControls();
    } catch (error) { showToast(String(error)); }
    return;
  }
  if (type === 'partyChatVisible') {
    try { applyPartyState(await invoke('party_set_chat_open', { open: value >= 0.5 })); } catch {}
    return;
  }
  if (type.startsWith('partyChatSend:')) {
    const encoded = type.slice('partyChatSend:'.length);
    let body = '';
    try { body = decodePartyBase64Url(encoded); } catch { body = ''; }
    if (body.trim()) {
      try {
        applyPartyState(await invoke('party_send_message', { body, clientId: newPartyClientId() }));
      } catch (error) { showToast(String(error)); }
    }
    return;
  }
  if (type.startsWith('partyChatRetry:')) {
    const key = type.slice('partyChatRetry:'.length);
    try { applyPartyState(await invoke('party_retry_message', { key })); } catch (error) { showToast(String(error)); }
    return;
  }
  if (type.startsWith('partyRemoveMember:')) {
    const memberId = type.slice('partyRemoveMember:'.length);
    try { applyPartyState(await invoke('party_remove_member', { memberId })); } catch (error) { showToast(String(error)); }
    return;
  }
  if (type === 'toggleFullscreen') {
    ctx.isFullscreen = await invoke('window_toggle_fullscreen').catch(() => ctx.isFullscreen);
    pushPlayerControls();
    return;
  }
  if (type === 'resize') {
    ctx.resizeMode = (ctx.resizeMode + 1) % 4;
    await invoke('player_set_resize_mode', { mode: ctx.resizeMode });
    notifyPlayerToast(`Proporção: ${RESIZE_MODES[ctx.resizeMode]}`);
    return;
  }
  if (type === 'externalPlayer') {
    await invoke('launch_mpv', { url: ctx.url, headers: ctx.headers || null, anime: playbackIsAnime(ctx) }).catch((e) => showToast(String(e)));
    return;
  }
  if (type === 'selectAudioTrack') {
    ctx.userSelectedAudio = true;
    if (value >= 0) await invoke('player_set_audio', { aid: String(Math.round(value)) });
    return;
  }
  if (type === 'selectBuiltInSubtitleTrack') {
    cancelSubtitleSync(ctx, true);
    ctx.userSelectedSub = true;
    const tid = Math.round(value);
    ctx.selectedAddonIndex = -1;
    ctx.activeAddonSub = null;
    if (tid < 0) {
      ctx.activeSubOptionId = '';
      await invoke('player_set_subtitle', { sid: 'no' });
      await invoke('player_set_subtitle_visible', { visible: false });
    } else {
      ctx.activeSubOptionId = `builtin:${tid}`;
      await invoke('player_set_subtitle', { sid: String(tid) });
      await invoke('player_set_subtitle_visible', { visible: true });
    }
    pushPlayerControls();
    return;
  }
  if (type === 'selectAddonSubtitle') {
    const idx = Math.round(value);
    const sub = ctx.addonSubtitles[idx];
    if (!sub) return;
    cancelSubtitleSync(ctx, true);
    ctx.userSelectedSub = true;
    ctx.selectedAddonIndex = idx;
    ctx.activeAddonSub = sub;
    ctx.activeSubOptionId = `addon:${sub.id || idx}`;
    ctx.autoSyncedApplied = false;
    await invoke('player_clear_external_subtitles');
    await invoke('player_add_subtitle_url', { url: sub.url });
    if (appSettings.playback.subtitle_sync && activePlayerCtx === ctx && ctx.activeAddonSub === sub) beginSubtitleSync(ctx, sub);
    pushPlayerControls();
    return;
  }
  if (type === 'fetchAddonSubtitles') {
    await fetchAddonSubtitlesForPlayer();
    return;
  }
  if (type === 'subtitleDelayDelta') {
    cancelSubtitleSync(ctx);
    ctx.subtitleDelayMs += Math.round(value);
    await invoke('player_set_subtitle_delay', { delayMs: ctx.subtitleDelayMs });
    pushPlayerControls();
    return;
  }
  if (type === 'subtitleDelayReset') {
    cancelSubtitleSync(ctx);
    ctx.subtitleDelayMs = 0;
    await invoke('player_set_subtitle_delay', { delayMs: 0 });
    pushPlayerControls();
    return;
  }
  if (type === 'subtitleAutoSyncReload' || type === 'subtitleAutoSyncCapture') {
    if (!ctx.activeAddonSub) return;
    ctx.loadingCues = true;
    ctx.cuesError = '';
    pushPlayerControls();
    try {
      let cues = ctx.cuesCache.get(ctx.activeAddonSub.url);
      if (!cues) {
        cues = await invoke('player_load_subtitle_cues', { url: ctx.activeAddonSub.url });
        ctx.cuesCache.set(ctx.activeAddonSub.url, cues);
      }
      const posMs = Math.round((ctx.latestState?.position_secs || 0) * 1000);
      ctx.capturedPosMs = posMs;
      const nearby = cues.filter((c) => Math.abs(c.timeMs - posMs) <= 120000);
      ctx.capturedCues = (nearby.length ? nearby : cues).slice(0, 40);
      ctx.allCuesForLookup = cues;
    } catch (e) {
      ctx.cuesError = String(e);
    } finally {
      ctx.loadingCues = false;
      pushPlayerControls();
    }
    return;
  }
  if (type === 'subtitleAutoSyncCue') {
    const cueIdx = Math.round(value);
    const cue = (ctx.allCuesForLookup || ctx.capturedCues || []).find((c) => c.index === cueIdx);
    if (!cue || ctx.capturedPosMs < 0) return;
    cancelSubtitleSync(ctx);
    ctx.subtitleDelayMs = Math.round(ctx.capturedPosMs - cue.timeMs);
    ctx.autoSyncedApplied = true;
    await invoke('player_set_subtitle_delay', { delayMs: ctx.subtitleDelayMs });
    notifyPlayerToast(`Sincronizado (${ctx.subtitleDelayMs} ms)`);
    return;
  }
  if (type.startsWith('subtitle') && await applySubtitleStyleEvent(type, value)) {
    return;
  }
  if (type === 'selectChapter') {
    const chap = ctx.latestState?.chapters?.[Math.round(value)];
    if (chap) await invoke('player_seek', { secs: chap.startMs / 1000 });
    return;
  }
  if (type === 'skipInterval') {
    const iv = ctx.activeSkipInterval;
    if (iv) {
      ctx.dismissedSkipKeys.add(iv.key);
      await invoke('player_seek', { secs: iv.endTime });
      ctx.activeSkipInterval = null;
      pushPlayerControls();
    }
    return;
  }
  if (type === 'selectSource') {
    const built = buildSourceItemsForOverlay();
    const sel = built.sourceItems[Math.round(value)]?._raw;
    if (sel?.url) {
      const keepSecs = ctx.latestState?.position_secs || 0;
      await startInAppPlayback({
        ...ctx,
        url: sel.url,
        fileIdx: sel.stream.fileIdx,
        headers: sel.stream.behaviorHints?.proxyHeaders?.request || null,
        streamTitle: sel.stream.name || sel.group.addon_name,
        providerName: sel.group.addon_name,
        providerLogo: sel.group.addon_logo || '',
        addonId: sel.group.addon_id,
        startSecs: keepSecs,
      });
    }
    return;
  }
  if (type === 'reloadSources') {
    if (activeStreamCtx) {
      ctx.loadingSources = true;
      pushPlayerControls();
      streamGroups = [];
      await invoke('start_stream_search', {
        mediaType: ctx.mediaType,
        id: ctx.videoId,
        tmdbId: ctx.tmdbId || null,
        season: ctx.season ?? null,
        episode: ctx.episode ?? null,
      });
      setTimeout(() => {
        if (activePlayerCtx === ctx) {
          ctx.loadingSources = false;
          pushPlayerControls();
        }
      }, 2200);
    }
    return;
  }
  if (type === 'backToEpisodes') {
    ctx.episodeStreamsVisible = false;
    pushPlayerControls();
    return;
  }
  if (type === 'selectEpisode' || type === 'reloadEpisodeStreams') {
    const ep = type === 'selectEpisode'
      ? (ctx.videos || [])[Math.round(value)]
      : ctx.pickedDrawerEpisode;
    if (!ep) return;
    ctx.pickedDrawerEpisode = ep;
    await loadPlayerDrawerEpisodeStreams(ep);
    return;
  }
  if (type === 'selectEpisodeStream') {
    const sel = ctx.episodeDrawerRaw?.[Math.round(value)];
    const ep = ctx.pickedDrawerEpisode;
    if (sel?.url && ep) {
      const s = ep.season ?? 1;
      const e = ep.episode ?? ep.number ?? 1;
      await startInAppPlayback({
        ...ctx,
        url: sel.url,
        fileIdx: sel.stream.fileIdx,
        headers: sel.stream.behaviorHints?.proxyHeaders?.request || null,
        videoId: ep.id || ctx.contentId,
        season: s,
        episode: e,
        episodeThumbnail: ep.thumbnail || '',
        episodeTitle: ep.title || ep.name || '',
        episodeOverview: ep.overview || ep.description || '',
        subtitle: `T${s} E${e} • ${ep.title || ep.name || ''}`,
        streamTitle: sel.stream.name || sel.group.addon_name,
        providerName: sel.group.addon_name,
        providerLogo: sel.group.addon_logo || '',
        addonId: sel.group.addon_id,
        startSecs: 0,
      });
    }
    return;
  }
  if (type === 'playNextEpisode' && ctx.nextEpisodeObj) {
    ctx.pickedDrawerEpisode = ctx.nextEpisodeObj;
    await loadPlayerDrawerEpisodeStreams(ctx.nextEpisodeObj, true);
    return;
  }
  if (type === 'submitIntroSegment') {
    ctx.submitDraft.segmentType = value === 1 ? 'recap' : (value === 2 ? 'outro' : 'intro');
    return;
  }
  if (type === 'submitIntroStart') {
    ctx.submitDraft.startSec = value;
    return;
  }
  if (type === 'submitIntroEnd') {
    ctx.submitDraft.endSec = value;
    return;
  }
  if (type === 'submitIntroCommit') {
    ctx.submittingIntro = true;
    ctx.submitIntroStatus = 'Enviando segmento…';
    pushPlayerControls();
    try {
      await invoke('player_submit_intro', {
        videoId: ctx.videoId,
        season: ctx.season ?? null,
        episode: ctx.episode ?? null,
        startSec: ctx.submitDraft.startSec,
        endSec: ctx.submitDraft.endSec,
        segmentType: ctx.submitDraft.segmentType,
      });
      ctx.submitIntroStatus = 'Enviado com sucesso!';
      ctx.submitIntroSuccessToken += 1;
      ctx.closeModalsToken += 1;
      notifyPlayerToast('Segmento enviado!');
      ctx.skipResolvedCount = 0;
      resolveSkipForActivePlayer();
    } catch (e) {
      ctx.submitIntroStatus = String(e);
    } finally {
      ctx.submittingIntro = false;
      pushPlayerControls();
    }
    return;
  }
  if (type === 'streamInfo') {
    const mi = ctx.latestState?.media_info;
    ctx.streamInfoSections = [
      {
        title: 'Fonte',
        name: ctx.providerName || 'Stream',
        logo: ctx.providerLogo || '',
        subtitle: ctx.streamTitle || '',
      },
      mi ? {
        title: 'Mídia',
        items: [
          { label: 'Resolução', value: mi.width ? `${mi.width}x${mi.height}` : '-' },
          { label: 'Vídeo', value: (mi.video_codec || '-').toUpperCase() },
          { label: 'Decodificador', value: mi.hwdec || 'Software' },
          { label: 'Áudio', value: `${(mi.audio_codec || '-').toUpperCase()} (${mi.audio_channels || '-'})` },
        ],
      } : null,
    ].filter(Boolean);
    pushPlayerControls();
  }
}

async function applySubtitleStyleEvent(type, value) {
  const ctx = activePlayerCtx;
  const st = ctx.subtitleStyle;
  let changed = true;
  if (type === 'subtitleFontSizeDelta') st.fontSizeSp = Math.max(10, Math.min(42, st.fontSizeSp + Math.round(value)));
  else if (type === 'subtitleOutlineToggle') st.outlineEnabled = !st.outlineEnabled;
  else if (type === 'subtitleBoldToggle') st.bold = !st.bold;
  else if (type === 'subtitleBottomOffsetDelta') st.bottomOffset = Math.max(0, Math.min(120, st.bottomOffset + Math.round(value)));
  else if (type === 'subtitleShadowToggle') st.shadowEnabled = !st.shadowEnabled;
  else if (type === 'subtitleShadowOpacityDelta') st.shadowOpacity = Math.max(0, Math.min(100, st.shadowOpacity + Math.round(value)));
  else if (type === 'subtitleShadowSizeDelta') st.shadowSize = Math.max(0, Math.min(8, st.shadowSize + Math.round(value)));
  else if (type === 'subtitleTextOpacity') st.textColor = withAlphaPercent(st.textColor, value);
  else if (type === 'subtitleBackgroundOpacity') st.backgroundColor = withAlphaPercent(st.backgroundColor, value);
  else if (type === 'subtitleTextColor') st.textColor = SUBTITLE_SWATCHES[Math.round(value)] || st.textColor;
  else if (type === 'subtitleOutlineColor') st.outlineColor = SUBTITLE_SWATCHES[Math.round(value)] || st.outlineColor;
  else if (type === 'subtitleBackgroundColor') st.backgroundColor = SUBTITLE_SWATCHES[Math.round(value)] || st.backgroundColor;
  else if (type === 'subtitleStyleReset') Object.assign(st, { ...DEFAULT_SUBTITLE_STYLE });
  else changed = false;

  if (!changed) return false;
  await invoke('player_apply_subtitle_style', { style: st }).catch(() => {});
  pushPlayerControls();
  return true;
}

function withAlphaPercent(argb, pct) {
  const clean = String(argb || '#FFFFFFFF').replace('#', '').padStart(8, 'F');
  const alpha = Math.round((Math.max(0, Math.min(100, pct)) / 100) * 255).toString(16).padStart(2, '0').toUpperCase();
  return `#${alpha}${clean.slice(-6)}`;
}

async function loadPlayerDrawerEpisodeStreams(ep, autoPlayIfMatched = false) {
  const ctx = activePlayerCtx;
  if (!ctx) return;
  const s = ep.season ?? 1;
  const e = ep.episode ?? ep.number ?? 1;
  ctx.episodeStreamsVisible = true;
  ctx.loadingEpisodeStreams = true;
  ctx.selectedEpisodeLabel = `T${s} E${e} • ${ep.title || ep.name || ''}`;
  ctx.episodeStreamFilters = [];
  ctx.episodeStreamItems = [];
  ctx.episodeDrawerRaw = [];
  pushPlayerControls();

  const tempGroups = [];
  const u1 = await listen('streams:addon', (ev) => { tempGroups.push(ev.payload); });
  const u2 = await listen('streams:done', async () => {
    u1(); u2();
    if (activePlayerCtx !== ctx) return;
    tempGroups.sort((a, b) => (a.order ?? 0) - (b.order ?? 0));
    const items = [];
    const raw = [];
    const filters = [{ id: '', label: 'Todos', isLoading: false, hasError: false }];
    for (const g of tempGroups) {
      filters.push({ id: g.addon_id, label: g.addon_name, isLoading: false, hasError: false });
      for (const st of g.streams) {
        const url = st.url || (st.infoHash ? `magnet:?xt=urn:btih:${st.infoHash}` : '');
        const { pills, formattedSize } = parseStreamBadges(st);
        items.push({
          index: items.length,
          label: (st.name || '').trim() || g.addon_name,
          subtitle: (st.description || st.title || '').trim(),
          addonName: g.addon_name,
          addonLogo: g.addon_logo || '',
          showAddonLogo: appSettings?.show_addon_logo !== false,
          filterId: g.addon_id,
          isEnabled: Boolean(url),
          badgePlacement: appSettings?.stream_badges_top !== false ? 'TOP' : 'BOTTOM',
          formattedSize: [formattedSize, ...pills].filter(Boolean).join(' • '),
          badges: [],
        });
        raw.push({ stream: st, group: g, url });
      }
    }

    if (autoPlayIfMatched) {
      const preferred = raw.find((r) => r.url && r.group.addon_id === ctx.addonId) || raw.find((r) => r.url);
      if (preferred) {
        await startInAppPlayback({
          ...ctx,
          url: preferred.url,
          fileIdx: preferred.stream.fileIdx,
          headers: preferred.stream.behaviorHints?.proxyHeaders?.request || null,
          videoId: ep.id || ctx.contentId,
          season: s,
          episode: e,
          episodeThumbnail: ep.thumbnail || '',
          episodeTitle: ep.title || ep.name || '',
          episodeOverview: ep.overview || ep.description || '',
          subtitle: `T${s} E${e} • ${ep.title || ep.name || ''}`,
          streamTitle: preferred.stream.name || preferred.group.addon_name,
          providerName: preferred.group.addon_name,
          providerLogo: preferred.group.addon_logo || '',
          addonId: preferred.group.addon_id,
          startSecs: 0,
        });
        return;
      }
    }

    ctx.loadingEpisodeStreams = false;
    ctx.episodeStreamFilters = filters;
    ctx.episodeStreamItems = items;
    ctx.episodeDrawerRaw = raw;
    pushPlayerControls();
  });

  await invoke('start_stream_search', {
    mediaType: ctx.mediaType,
    id: ep.id || ctx.contentId,
    tmdbId: ctx.tmdbId || null,
    season: s,
    episode: e,
  });
}

function playbackIsAnime(ctx) {
  const id = ctx.contentId || ctx.videoId || '';
  if (ctx.mediaType === 'anime' || /^(kitsu|mal|anilist):/i.test(id)) return true;
  if (id !== activeMetaContext?.id) return false;
  const meta = activeMetaContext.full || activeMetaContext;
  const genres = Array.isArray(meta.genres) ? meta.genres : [];
  return genres.some(g => /^anime$/i.test(g)) ||
    (genres.some(g => /^(animation|animação)$/i.test(g)) && /japan|japão|日本|\bJP\b/i.test(meta.country || ''));
}

async function startInAppPlayback(req) {
  try { req = await resolveConnectedMedia(req); }
  catch (e) { showToast(String(e)); return; }
  if (activePlayerCtx) {
    await simklSendScrobble(activePlayerCtx, activePlayerCtx.latestState, true);
    await mdblistSendScrobble(activePlayerCtx, activePlayerCtx.latestState, true);
  }
  const progressSrc = activeProgressSource();
  const simklResume = !req.startSecs && (
    progressSrc === 'simkl'
      ? simklProgressCards().find(card =>
          simklProgressMatches(card, req.contentId) && (card.episode ?? null) === (req.episode ?? null) &&
            (card.season ?? null) === (req.season ?? null))
      : progressSrc === 'mdblist'
        ? (mdblistProgressCache.cards || []).find(card =>
            mdblistMatchesCard(card, req.contentId) && (card.episode ?? null) === (req.episode ?? null) &&
              (card.season ?? null) === (req.season ?? null))
        : null
  );
  await invoke('player_stop').catch(() => {});
  playerActive = true;
  playerCurrentPaused = false;

  const subStyleCopy = { ...DEFAULT_SUBTITLE_STYLE, ...(appSettings?.subtitle_style || {}) };
  activePlayerCtx = {
    ...req,
    simklResumePercent: simklResume?.progress_percent || 0,
    seekPreview: null,
    seekPreviewAfter: 0,
    resizeMode: 0,
    isFullscreen: false,
    addonSubtitles: [],
    loadingAddonSubtitles: false,
    activeAddonSub: null,
    selectedAddonIndex: -1,
    activeSubOptionId: '',
    subtitleDelayMs: 0,
    autoSyncedApplied: false,
    cuesCache: new Map(),
    capturedPosMs: -1,
    capturedCues: [],
    loadingCues: false,
    cuesError: '',
    subtitleStyle: subStyleCopy,
    skipIntervals: [],
    skipResolvedCount: 0,
    skipResolvedWithChapters: false,
    skipFetchInFlight: false,
    autoSkippedKeys: new Set(),
    dismissedSkipKeys: new Set(),
    activeSkipInterval: null,
    nextEpisodeObj: computeNextEpisode(req.videos, req.season, req.episode),
    showNextEpisode: false,
    episodeStreamsVisible: false,
    loadingEpisodeStreams: false,
    loadingSources: false,
    notificationToken: 0,
    closeModalsToken: 0,
    submitIntroSuccessToken: 0,
    submitDraft: { segmentType: 'intro', startSec: 0, endSec: 0 },
    prefAudioScore: 0,
    prefAudioSig: '',
    userSelectedAudio: false,
    prefSubScore: 0,
    prefSubSig: '',
    userSelectedSub: false,
    controlsVisible: true,
    lastProgressSyncMs: Date.now(),
    discordWatchedMs: 0,
    discordTickAt: 0,
    discordStart: null,
    latestState: null,
  };

  showView('view-player');
  reportPartyPlayerStatus('loading', partyMediaKey());
  const playerShell = document.getElementById('player-shell');
  playerShell.classList.remove('overlay-active', 'fallback-active');
  if (appSettings?.external_player_enabled) {
    playerShell.classList.add('fallback-active');
  }
  document.getElementById('player-title').textContent = req.title || 'Reproduzindo';
  document.getElementById('player-sub').textContent = req.subtitle || '';
  document.getElementById('player-stage-status').textContent = '';

  if (playerStateUnlisten) { playerStateUnlisten(); playerStateUnlisten = null; }
  if (playerUiEventUnlisten) { playerUiEventUnlisten(); playerUiEventUnlisten = null; }

  playerUiEventUnlisten = await listen('player:ui_event', (ev) => handlePlayerUiEvent(ev.payload));

  playerStateUnlisten = await listen('player:state', (event) => {
    const st = event.payload;
    if (!activePlayerCtx) return;
    const previousActive = activePlayerCtx.latestState?.active;
    const previousEnded = activePlayerCtx.latestState?.ended;
    const wasPaused = activePlayerCtx.latestState?.paused;
    const wasLoading = activePlayerCtx.latestState?.loading;
    activePlayerCtx.latestState = st;
    if (st.active) {
      if (!st.loading) activePlayerCtx.hadFirstReady = true;
      if (partyLive.active) {
        const memberStatus = st.loading ? (activePlayerCtx.hadFirstReady ? 'buffering' : 'loading') : 'ready';
        reportPartyPlayerStatus(memberStatus, partyMediaKey());
      }
    }
    if (wasLoading && !st.loading && partyLive.active) {
      partyLastAppliedSeq = null;
      syncPartyPlayback(partyLive, true);
    }
    partyReportTick(st, wasPaused !== undefined && Boolean(wasPaused) !== Boolean(st.paused));
    if (partyLive.active && Date.now() - lastPartySyncCheck >= 450) {
      lastPartySyncCheck = Date.now();
      syncPartyPlayback(partyLive, false);
    }
    if (st.active && !st.loading) loadSeekPreview(activePlayerCtx, st.duration_secs);
    if (!st.loading && st.active && st.duration_secs > 10 && activePlayerCtx.simklResumePercent) {
      const target = st.duration_secs * activePlayerCtx.simklResumePercent / 100;
      activePlayerCtx.simklResumePercent = 0;
      invoke('player_seek',{secs:target}).catch(e => showToast(String(e)));
    } else if (st.active || st.ended) {
      simklSendScrobble(activePlayerCtx,st);
      mdblistSendScrobble(activePlayerCtx, st);
    }
    observeSubtitleSync(activePlayerCtx, st);
    const nowMs = Date.now();
    const isWatching = Boolean(st.active && !st.paused && !st.loading && !st.ended);
    const wasWatching = Boolean(activePlayerCtx.discordTickAt);
    if (isWatching) {
      if (wasWatching) {
        activePlayerCtx.discordWatchedMs += Math.min(2000, Math.max(0, nowMs - activePlayerCtx.discordTickAt));
      }
      activePlayerCtx.discordTickAt = nowMs;
      if (!activePlayerCtx.discordStart || !wasWatching) {
        activePlayerCtx.discordStart = Math.floor((nowMs - activePlayerCtx.discordWatchedMs) / 1000);
      }
    } else if (wasWatching) {
      activePlayerCtx.discordWatchedMs += Math.min(2000, Math.max(0, nowMs - activePlayerCtx.discordTickAt));
      activePlayerCtx.discordTickAt = 0;
    }
    if (wasWatching !== isWatching || wasPaused !== st.paused || previousActive !== st.active || previousEnded !== st.ended) syncDiscord();
    updateFallbackPlayerUi(st);
    if (wasLoading !== undefined && Boolean(wasLoading) !== Boolean(st.loading)) {
      pushPlayerControls();
    }

    // Preferred audio & subtitle auto-selection independently once each track list populates
    if (!activePlayerCtx.userSelectedAudio && Array.isArray(st.audio_tracks) && st.audio_tracks.length) {
      const audSig = st.audio_tracks.map((t) => `${t.id}:${t.lang}:${t.title}`).join('|');
      if (audSig !== activePlayerCtx.prefAudioSig) {
        activePlayerCtx.prefAudioSig = audSig;
        const audMatch = findBestPreferredTrack(
          st.audio_tracks,
          appSettings?.preferred_audio_lang || 'por'
        );
        if (audMatch && audMatch.score > activePlayerCtx.prefAudioScore) {
          activePlayerCtx.prefAudioScore = audMatch.score;
          if (!audMatch.track.selected) {
            invoke('player_set_audio', { aid: String(audMatch.track.id) }).catch(() => {});
          }
        }
      }
    }
    if (!activePlayerCtx.userSelectedSub && Array.isArray(st.subtitle_tracks) && st.subtitle_tracks.length) {
      const subSig = st.subtitle_tracks.map((t) => `${t.id}:${t.lang}:${t.title}`).join('|');
      if (subSig !== activePlayerCtx.prefSubSig) {
        activePlayerCtx.prefSubSig = subSig;
        const subMatch = findBestPreferredTrack(
          st.subtitle_tracks,
          appSettings?.preferred_sub_lang || 'pt-br'
        );
        if (subMatch && subMatch.score > activePlayerCtx.prefSubScore) {
          activePlayerCtx.prefSubScore = subMatch.score;
          if (!subMatch.track.selected) {
            invoke('player_set_subtitle', { sid: String(subMatch.track.id) }).catch(() => {});
            activePlayerCtx.activeSubOptionId = `builtin:${subMatch.track.id}`;
          }
        }
      }
    }

    if (st.chapters?.length && !activePlayerCtx.skipResolvedWithChapters) {
      resolveSkipForActivePlayer();
    }

    // Active skip interval & Auto-skip logic
    if (appSettings?.skip_intro_enabled !== false && activePlayerCtx.skipIntervals.length) {
      const pos = st.position_secs || 0;
      const match = activePlayerCtx.skipIntervals.find((iv) => pos >= iv.startTime && pos < iv.endTime - 0.5) || null;
      if (match && !activePlayerCtx.autoSkippedKeys.has(match.key)) {
        const cat = match.skipType;
        const autoIntro = (cat === 'op' || cat === 'intro' || cat === 'mixed-op') && appSettings?.auto_skip_intro;
        const autoRecap = cat === 'recap' && appSettings?.auto_skip_recap;
        const autoOutro = (cat === 'ed' || cat === 'outro' || cat === 'credits' || cat === 'mixed-ed') && appSettings?.auto_skip_outro;
        if (autoIntro || autoRecap || autoOutro) {
          activePlayerCtx.autoSkippedKeys.add(match.key);
          invoke('player_seek', { secs: match.endTime });
          notifyPlayerToast(`Pulado: ${match.label}`);
          return;
        }
      }
      if ((match?.key || '') !== (activePlayerCtx.activeSkipInterval?.key || '')) {
        activePlayerCtx.activeSkipInterval = match;
        pushPlayerControls();
      }
    }

    // Next episode threshold
    const rem = (st.duration_secs || 0) - (st.position_secs || 0);
    const shouldShowNext = Boolean(activePlayerCtx.nextEpisodeObj && st.duration_secs > 60 && rem <= 42);
    if (shouldShowNext !== activePlayerCtx.showNextEpisode) {
      activePlayerCtx.showNextEpisode = shouldShowNext;
      pushPlayerControls();
    }

    if (wasPaused !== undefined && wasPaused !== st.paused) {
      syncActivePlayerWatchProgress(true);
    } else {
      syncActivePlayerWatchProgress(false);
    }
  });

  requestAnimationFrame(async () => {
    const b = getPlayerStageBounds();
    const initialObj = buildPlayerControlsObject();
    try {
      const mode = await invoke('player_play', {
        anime: playbackIsAnime(req),
        url: req.url,
        headers: req.headers || null,
        x: b.x,
        y: b.y,
        w: b.w,
        h: b.h,
        startSecs: req.startSecs || null,
        initialControlsJson: initialObj ? JSON.stringify(initialObj) : null,
      });
function getWebviewVideoEl() {
  return document.getElementById('webview-html5-video');
}

function teardownWebviewVideo() {
  const video = getWebviewVideoEl();
  if (video) {
    try { video.pause(); } catch {}
    video.removeAttribute('src');
    video.load();
    video.remove();
  }
}

function mountWebviewHtml5Player(req) {
  teardownWebviewVideo();
  const stage = document.getElementById('player-video-stage');
  const statusEl = document.getElementById('player-stage-status');
  if (statusEl) statusEl.textContent = 'Carregando vídeo…';
  const video = document.createElement('video');
  video.id = 'webview-html5-video';
  video.className = 'webview-html5-video';
  video.playsInline = true;
  video.autoplay = true;
  video.preload = 'auto';
  video.src = req.url;

  const emitWebviewState = () => {
    if (!activePlayerCtx || activePlayerCtx.mode !== 'webview:html5') return;
    const st = {
      active: true,
      paused: video.paused,
      buffering: video.readyState < 3 && !video.paused && !video.ended,
      loading: video.readyState < 2 && !video.error,
      ended: video.ended,
      position_secs: Number(video.currentTime) || 0,
      duration_secs: Number.isFinite(video.duration) ? video.duration : 0,
      buffered_secs: 0,
      volume: Math.round((video.muted ? 0 : video.volume) * 100),
      speed: video.playbackRate || 1,
      audio_tracks: [],
      subtitle_tracks: [],
      chapters: [],
    };
    activePlayerCtx.latestState = st;
    updateFallbackPlayerUi(st);
    syncActivePlayerWatchProgress(false);
  };

  video.addEventListener('loadedmetadata', () => {
    if (statusEl) statusEl.textContent = '';
    if (req.startSecs && req.startSecs > 0 && Number.isFinite(video.duration)) {
      video.currentTime = Math.min(req.startSecs, Math.max(0, video.duration - 2));
    }
    emitWebviewState();
  });
  video.addEventListener('playing', () => {
    if (statusEl) statusEl.textContent = '';
    emitWebviewState();
  });
  video.addEventListener('pause', () => {
    emitWebviewState();
    syncActivePlayerWatchProgress(true);
  });
  video.addEventListener('timeupdate', emitWebviewState);
  video.addEventListener('waiting', emitWebviewState);
  video.addEventListener('ended', () => {
    emitWebviewState();
    syncActivePlayerWatchProgress(true);
  });
  video.addEventListener('error', () => {
    if (statusEl) statusEl.textContent = 'Formato não suportado pelo WebView. Tente outro stream.';
  });
  video.addEventListener('click', () => {
    if (video.paused) video.play().catch(() => {});
    else video.pause();
  });

  stage.prepend(video);
  video.play().catch(() => {});
}

      if (mode === 'embedded:overlay') {
        playerShell.classList.remove('fallback-active');
        playerShell.classList.add('overlay-active');
        invoke('player_set_bounds', getPlayerStageBounds());
        pushPlayerControls();
      } else if (mode === 'webview:html5') {
        if (activePlayerCtx) activePlayerCtx.mode = 'webview:html5';
        playerShell.classList.add('fallback-active');
        mountWebviewHtml5Player(req);
      } else if (mode.startsWith('external:')) {
        playerShell.classList.add('fallback-active');
        document.getElementById('player-stage-status').textContent = `Aberto no player externo (${mode})`;
      } else {
        playerShell.classList.add('fallback-active');
        document.getElementById('player-stage-status').textContent = '';
      }
      resolveSkipForActivePlayer();
      fetchAddonSubtitlesForPlayer();
    } catch (e) {
      playerShell.classList.add('fallback-active');
      document.getElementById('player-stage-status').textContent = String(e);
    }
  });
}

function updateFallbackPlayerUi(st) {
  playerCurrentPaused = !!st.paused;
  document.getElementById('btn-player-playpause').textContent = st.paused ? 'Reproduzir' : 'Pausar';
  document.getElementById('player-buffer-badge').textContent = st.buffering ? 'Carregando buffer…' : '';
  document.getElementById('player-time-cur').textContent = formatClock(st.position_secs);
  document.getElementById('player-time-dur').textContent = formatClock(st.duration_secs);

  const seekEl = document.getElementById('player-seek');
  if (!playerSeeking && st.duration_secs > 0) {
    seekEl.max = st.duration_secs;
    seekEl.value = st.position_secs;
  }

  const audioSel = document.getElementById('player-audio-select');
  const audioKey = st.audio_tracks.map((t) => `${t.id}:${t.selected}`).join(',');
  if (audioSel.dataset.key !== audioKey) {
    audioSel.dataset.key = audioKey;
    audioSel.innerHTML = st.audio_tracks.length
      ? st.audio_tracks.map((t) => `<option value="${t.id}" ${t.selected ? 'selected' : ''}>Áudio: ${esc(t.lang || t.title || `#${t.id}`)}</option>`).join('')
      : '<option value="auto">Áudio: Auto</option>';
  }

  const subSel = document.getElementById('player-sub-select');
  const ctx = activePlayerCtx;
  const addons = ctx?.addonSubtitles || [];
  const activeAddon = ctx?.selectedAddonIndex ?? -1;
  const subKey = JSON.stringify([st.subtitle_tracks.map(t => [t.id, t.selected]), addons.map(a => a.url), activeAddon]);
  if (subSel.dataset.key !== subKey) {
    subSel.dataset.key = subKey;
    const none = activeAddon < 0 && !st.subtitle_tracks.some(t => t.selected);
    subSel.innerHTML = `<option value="-1" ${none ? 'selected' : ''}>Legendas: Desativadas</option>` +
      st.subtitle_tracks.map(t => `<option value="${t.id}" ${activeAddon < 0 && t.selected ? 'selected' : ''}>Leg: ${esc(t.lang || t.title || `#${t.id}`)}</option>`).join('') +
      addons.map((s, i) => `<option value="addon:${i}" ${activeAddon === i ? 'selected' : ''}>${esc(s.addon_name || 'Addon')} · ${esc(languageLabelPt(s.lang))}</option>`).join('');
  }
}

async function closeEmbeddedPlayer() {
  fallbackSeekPreview.hide();
  if (!playerActive) return;
  if (activePlayerCtx) {
    await simklSendScrobble(activePlayerCtx,activePlayerCtx.latestState,true);
    await mdblistSendScrobble(activePlayerCtx, activePlayerCtx.latestState, true);
  }
  await syncActivePlayerWatchProgress(true);
  teardownWebviewVideo();
  playerActive = false;
  activePlayerCtx = null;
  document.getElementById('player-shell').classList.remove('overlay-active', 'fallback-active');
  if (playerStateUnlisten) { playerStateUnlisten(); playerStateUnlisten = null; }
  if (playerUiEventUnlisten) { playerUiEventUnlisten(); playerUiEventUnlisten = null; }
  await invoke('player_stop');
  showView('view-app');
  if (partyLive.active) {
    const inStreams = activeTabName() === 'streams';
    reportPartyPlayerStatus(inStreams ? 'choosing' : 'browsing', inStreams ? partyStreamMediaKey() : '');
  }
  refreshContinueWatching();
}

window.addEventListener('resize', () => {
  if (!playerActive) return;
  invoke('player_set_bounds', getPlayerStageBounds());
});

function blockFallbackControlIfPartyLocked() {
  if (partyLive.active && partyLive.waitingFor?.length) {
    invoke('player_set_paused', { paused: true }).catch(() => {});
    showToast(`Aguardando ${partyLive.waitingFor.join(', ')} escolher/carregar o player.`);
    return true;
  }
  if (partyLive.active && partyLive.hostOnly && !partyLive.isHost) {
    showToast('Somente o anfitrião pode controlar a reprodução.');
    return true;
  }
  return false;
}

document.getElementById('btn-player-back').addEventListener('click', closeEmbeddedPlayer);
document.getElementById('btn-player-playpause').addEventListener('click', () => {
  if (blockFallbackControlIfPartyLocked()) return;
  const v = getWebviewVideoEl();
  if (v) {
    if (v.paused) v.play().catch(() => {});
    else v.pause();
    return;
  }
  invoke('player_set_paused', { paused: !playerCurrentPaused });
});
document.getElementById('btn-player-back10').addEventListener('click', () => {
  if (blockFallbackControlIfPartyLocked()) return;
  const cur = Number(document.getElementById('player-seek').value) || 0;
  const next = Math.max(0, cur - 10);
  const v = getWebviewVideoEl();
  if (v) { v.currentTime = next; return; }
  invoke('player_seek', { secs: next });
});
document.getElementById('btn-player-fwd10').addEventListener('click', () => {
  if (blockFallbackControlIfPartyLocked()) return;
  const cur = Number(document.getElementById('player-seek').value) || 0;
  const next = cur + 10;
  const v = getWebviewVideoEl();
  if (v) { v.currentTime = next; return; }
  invoke('player_seek', { secs: next });
});

const seekInput = document.getElementById('player-seek');
const fallbackSeekPreview = bindSeekPreview(seekInput, () => activePlayerCtx?.seekPreview, () => Number(seekInput.max) * 1000);
seekInput.addEventListener('keyup', ev => {
  if (['ArrowLeft','ArrowRight','Home','End','PageUp','PageDown'].includes(ev.key)) fallbackSeekPreview.show('keyboard', Number(seekInput.value) * 1000);
});
seekInput.addEventListener('input', () => { playerSeeking = true; });
seekInput.addEventListener('change', () => {
  playerSeeking = false;
  if (blockFallbackControlIfPartyLocked()) return;
  const secs = Number(seekInput.value);
  const v = getWebviewVideoEl();
  if (v) { v.currentTime = secs; return; }
  invoke('player_seek', { secs });
});
document.getElementById('player-volume').addEventListener('input', (e) => {
  const vol = Number(e.target.value);
  const v = getWebviewVideoEl();
  if (v) { v.volume = Math.max(0, Math.min(1, vol / 100)); return; }
  invoke('player_set_volume', { volume: vol });
});
document.getElementById('player-speed-select').addEventListener('change', (e) => {
  const speed = Number(e.target.value);
  const v = getWebviewVideoEl();
  if (v) { v.playbackRate = speed; return; }
  invoke('player_set_speed', { speed });
});
document.getElementById('player-audio-select').addEventListener('change', (e) => {
  if (activePlayerCtx) activePlayerCtx.userSelectedAudio = true;
  invoke('player_set_audio', { aid: e.target.value });
});
document.getElementById('player-sub-select').addEventListener('change', (e) => {
  const option = e.target.value;
  const addon = option.startsWith('addon:');
  handlePlayerUiEvent({ event_type: addon ? 'selectAddonSubtitle' : 'selectBuiltInSubtitleTrack',
    value: Number(addon ? option.slice(6) : option) }).catch(error => notifyPlayerToast(String(error)));
});
document.getElementById('btn-player-fullscreen').addEventListener('click', () => {
  invoke('window_toggle_fullscreen');
});

// --- SEARCH & DISCOVER (100% Addons + Recent Searches + Dropdowns) ---
let searchSeq = 0;
let recentSearchOpen = false;
let searchDebounceTimer = null;
let searchListeners;
let activeSearchQuery = '';
const searchSectionsMap = new Map();

let discoverCatalogs = [];
let discoverSelectedType = null;
let discoverSelectedKey = null;
let discoverSelectedGenre = null;
let discoverItems = [];
let discoverSeen = new Set();
let discoverLoading = false;
let discoverRequest = 0;
let discoverHasMore = false;

const searchInputEl = document.getElementById('search-input');
const searchClearBtn = document.getElementById('btn-search-clear');

function recentSearchStorageKey() {
  return `nuvio_recent_searches_${authState?.active_profile_index || 1}`;
}

function loadRecentSearches() {
  try {
    const arr = JSON.parse(localStorage.getItem(recentSearchStorageKey()) || '[]');
    return Array.isArray(arr) ? arr : [];
  } catch {
    return [];
  }
}

function saveRecentSearch(q) {
  const clean = q.trim();
  if (clean.length < 2) return;
  const list = [clean, ...loadRecentSearches().filter((x) => x.toLowerCase() !== clean.toLowerCase())].slice(0, 10);
  localStorage.setItem(recentSearchStorageKey(), JSON.stringify(list));
  renderRecentSearches();
}

function renderRecentSearches() {
  const wrap = document.getElementById('recent-searches-wrap');
  if (!wrap) return;
  const hasQuery = searchInputEl.value.trim().length > 0;
  const list = loadRecentSearches();
  // Só aparece com a barra de pesquisa ativa e sem texto escrito.
  if (!recentSearchOpen || hasQuery || !list.length) {
    wrap.classList.add('hidden');
    wrap.innerHTML = '';
    return;
  }
  wrap.classList.remove('hidden');
  wrap.innerHTML = `
    <div class="recent-searches-title">Buscas recentes</div>
    ${list.map((q) => `
      <div class="recent-search-row" data-recent-q="${esc(q)}">
        <span class="recent-search-text">${esc(q)}</span>
        <button type="button" data-del-recent="${esc(q)}" title="Remover" aria-label="Remover busca">&times;</button>
      </div>
    `).join('')}
  `;
  wrap.querySelectorAll('[data-recent-q]').forEach((row) => {
    row.addEventListener('click', (e) => {
      const del = e.target.closest('[data-del-recent]');
      if (del) {
        const next = loadRecentSearches().filter((x) => x !== del.dataset.delRecent);
        localStorage.setItem(recentSearchStorageKey(), JSON.stringify(next));
        renderRecentSearches();
        return;
      }
      recentSearchOpen = false;
      searchInputEl.value = row.dataset.recentQ;
      runSearch();
    });
  });
}

searchInputEl.addEventListener('focus', () => {
  recentSearchOpen = true;
  renderRecentSearches();
});
searchInputEl.addEventListener('blur', () => {
  // Pequeno atraso para o clique numa busca recente contar antes de fechar.
  setTimeout(() => {
    recentSearchOpen = false;
    renderRecentSearches();
  }, 120);
});
searchInputEl.addEventListener('keydown', (ev) => {
  if (ev.key !== 'Escape') return;
  recentSearchOpen = false;
  renderRecentSearches();
});
document.getElementById('recent-searches-wrap')?.addEventListener('mousedown', (ev) => {
  // Mantém o foco na barra para o painel não fechar ao clicar numa linha.
  ev.preventDefault();
});

function syncSearchModeUi() {
  const hasQuery = searchInputEl.value.trim().length > 0;
  searchClearBtn.classList.toggle('hidden', searchInputEl.value.length === 0);
  document.getElementById('search-results').classList.toggle('hidden', !hasQuery);
  document.getElementById('discover-section')?.classList.toggle('hidden', hasQuery);
  renderRecentSearches();
  if (!hasQuery) {
    document.getElementById('search-status').textContent = '';
  }
}

document.getElementById('btn-search-fullscreen')?.addEventListener('click', () => {
  invoke('window_toggle_fullscreen').catch(() => {});
});

document.getElementById('search-form').addEventListener('submit', (event) => {
  event.preventDefault();
  clearTimeout(searchDebounceTimer);
  runSearch();
});

function cancelAddonSearch() {
  const seq = ++searchSeq;
  activeSearchQuery = '';
  invoke('start_addon_search', { query: '', seq }).catch(console.error);
}

searchClearBtn.addEventListener('click', () => {
  clearTimeout(searchDebounceTimer);
  cancelAddonSearch();
  searchInputEl.value = '';
  searchSectionsMap.clear();
  document.getElementById('search-results').innerHTML = '';
  syncSearchModeUi();
  searchInputEl.focus();
});

searchInputEl.addEventListener('input', () => {
  syncSearchModeUi();
  clearTimeout(searchDebounceTimer);
  if (activeSearchQuery) cancelAddonSearch();
  searchSectionsMap.clear();
  document.getElementById('search-results').replaceChildren();
  if (searchInputEl.value.trim()) searchDebounceTimer = setTimeout(runSearch, 350);
});

async function runSearch() {
  const query = searchInputEl.value.trim();
  const status = document.getElementById('search-status');
  const resultsEl = document.getElementById('search-results');
  syncSearchModeUi();
  if (!query || query === activeSearchQuery) return;

  activeSearchQuery = query;
  saveRecentSearch(query);
  const seq = ++searchSeq;
  searchSectionsMap.clear();
  resultsEl.innerHTML = '';
  status.textContent = 'Buscando nos addons…';

  try {
    searchListeners ||= Promise.all([
      listen('search:section', (event) => {
        const payload = event.payload;
        if (payload.seq !== searchSeq) return;
        const sec = { ...payload.section, searchQuery: activeSearchQuery };
        searchSectionsMap.set(sec.order, sec);
        status.textContent = '';
        appendShelfOrdered(resultsEl, sec, searchSectionsMap);
      }),
      listen('search:done', (event) => {
        const payload = event.payload;
        if (payload.seq !== searchSeq) return;
        status.textContent = searchSectionsMap.size ? '' : payload.total === 0
          ? 'Nenhum addon ativo possui catálogo com suporte a busca.'
          : 'Nenhum resultado encontrado nos addons instalados.';
      }),
    ]);
    await searchListeners;
    if (seq !== searchSeq) return;
    const ack = await invoke('start_addon_search', { query, seq });
    if (ack.total === 0 && seq === searchSeq) {
      status.textContent = 'Nenhum addon ativo possui catálogo com suporte a busca.';
    }
  } catch (e) {
    if (seq === searchSeq) {
      activeSearchQuery = '';
      status.textContent = String(e);
    }
  }
}

function catalogSupportsDiscover(cat) {
  const extra = cat.extra || [];
  if (extra.some((p) => p.name === 'search' && p.is_required)) return false;
  return extra.every((p) => {
    if (p.name === 'genre') return !p.is_required || (p.options && p.options.length > 0);
    if (p.name === 'skip' || p.name === 'search') return true;
    return !p.is_required;
  });
}

function discoverTypeLabel(t) {
  switch (String(t || '').toLowerCase()) {
    case 'movie': return 'Filmes';
    case 'series': return 'Séries';
    case 'anime': return 'Animes';
    case 'channel': return 'Canais';
    case 'tv': return 'TV';
    default: return t ? t.charAt(0).toUpperCase() + t.slice(1) : '';
  }
}

function initDiscoverFromAddons(addons) {
  discoverCatalogs = [];
  for (const addon of (addons || [])) {
    if (!addon.enabled) continue;
    for (const cat of (addon.manifest?.catalogs || [])) {
      if (!catalogSupportsDiscover(cat)) continue;
      const genreProp = (cat.extra || []).find((p) => p.name === 'genre');
      discoverCatalogs.push({
        key: `${addon.manifest.id}:${cat.type}:${cat.id}`,
        addonName: addon.manifest.name,
        manifestUrl: addon.manifest_url,
        type: cat.type,
        catalogId: cat.id,
        catalogName: cat.name,
        genreOptions: genreProp?.options || [],
        genreRequired: !!genreProp?.is_required,
      });
    }
  }

  const types = [...new Set(discoverCatalogs.map((c) => c.type))];
  if (!discoverSelectedType || !types.includes(discoverSelectedType)) {
    discoverSelectedType = types[0] || null;
  }
  if (activeTabName() === 'search') syncDiscoverSelection();
}

function syncDiscoverSelection() {
  const typeSelectEl = document.getElementById('discover-type-select');
  const catSelectEl = document.getElementById('discover-catalog-select');
  const genreSelectEl = document.getElementById('discover-genre-select');
  const ctxLabelEl = document.getElementById('discover-context-label');
  const gridEl = document.getElementById('discover-grid');
  const statusEl = document.getElementById('discover-status');
  const loadMoreBtn = document.getElementById('btn-discover-load-more');

  const types = [...new Set(discoverCatalogs.map((c) => c.type))];
  if (!types.length) {
    ++discoverRequest;
    discoverLoading = false;
    discoverHasMore = false;
    if (typeSelectEl) {
      typeSelectEl.innerHTML = '<option value="">Tipo</option>';
      typeSelectEl.disabled = true;
    }
    if (catSelectEl) {
      catSelectEl.innerHTML = '<option value="">Catálogo</option>';
      catSelectEl.disabled = true;
    }
    if (genreSelectEl) {
      genreSelectEl.innerHTML = '<option value="">Todos os gêneros</option>';
      genreSelectEl.disabled = true;
    }
    ctxLabelEl.textContent = '';
    gridEl.innerHTML = '';
    loadMoreBtn.classList.add('hidden');
    statusEl.textContent = 'Instale ou ative addons com catálogos para explorar.';
    return;
  }

  if (typeSelectEl) {
    typeSelectEl.disabled = false;
    typeSelectEl.innerHTML = types.map((t) =>
      `<option value="${esc(t)}" ${t === discoverSelectedType ? 'selected' : ''}>${esc(discoverTypeLabel(t))}</option>`,
    ).join('');
    typeSelectEl.onchange = () => {
      discoverSelectedType = typeSelectEl.value;
      discoverSelectedKey = null;
      discoverSelectedGenre = null;
      syncDiscoverSelection();
    };
  }

  const filteredCats = discoverCatalogs.filter((c) => c.type === discoverSelectedType);
  if (!discoverSelectedKey || !filteredCats.some((c) => c.key === discoverSelectedKey)) {
    discoverSelectedKey = filteredCats[0]?.key || null;
  }
  const activeCat = filteredCats.find((c) => c.key === discoverSelectedKey) || null;

  if (catSelectEl) {
    catSelectEl.disabled = filteredCats.length === 0;
    catSelectEl.innerHTML = filteredCats.map((c) =>
      `<option value="${esc(c.key)}" ${c.key === discoverSelectedKey ? 'selected' : ''}>${esc(c.catalogName)}</option>`,
    ).join('');
    catSelectEl.onchange = () => {
      discoverSelectedKey = catSelectEl.value;
      discoverSelectedGenre = null;
      syncDiscoverSelection();
    };
  }

  if (genreSelectEl) {
    if (activeCat?.genreOptions?.length) {
      if (activeCat.genreRequired && (!discoverSelectedGenre || !activeCat.genreOptions.includes(discoverSelectedGenre))) {
        discoverSelectedGenre = activeCat.genreOptions[0];
      } else if (discoverSelectedGenre && !activeCat.genreOptions.includes(discoverSelectedGenre)) {
        discoverSelectedGenre = null;
      }
      const opts = activeCat.genreRequired ? activeCat.genreOptions : ['__all__', ...activeCat.genreOptions];
      genreSelectEl.innerHTML = opts.map((g) => {
        const val = g === '__all__' ? '' : g;
        const label = g === '__all__' ? 'Todos os gêneros' : g;
        const isSel = (val || null) === discoverSelectedGenre;
        return `<option value="${esc(val)}" ${isSel ? 'selected' : ''}>${esc(label)}</option>`;
      }).join('');
      genreSelectEl.disabled = opts.length <= 1 && !activeCat.genreRequired;
      genreSelectEl.onchange = () => {
        discoverSelectedGenre = genreSelectEl.value || null;
        syncDiscoverSelection();
      };
    } else {
      discoverSelectedGenre = null;
      genreSelectEl.innerHTML = '<option value="">Todos os gêneros</option>';
      genreSelectEl.disabled = true;
    }
  }

  ctxLabelEl.textContent = activeCat
    ? `${activeCat.addonName} • ${discoverTypeLabel(activeCat.type)}${discoverSelectedGenre ? ` • ${discoverSelectedGenre}` : ''}`
    : '';

  loadDiscoverPage(true);
}

const DISCOVER_MEM_CACHE_TTL_MS = 10 * 60 * 1000;
const discoverPageMemCache = new Map();
const discoverPageInFlight = new Map();

function fetchCatalogPageCached(params) {
  const cacheKey = `${params.manifestUrl}|${params.mediaType}|${params.catalogId}|${params.skip || 0}|${params.genre || ''}`;
  const hit = discoverPageMemCache.get(cacheKey);
  if (hit && Date.now() - hit.at < DISCOVER_MEM_CACHE_TTL_MS) {
    return Promise.resolve(hit.items);
  }
  if (discoverPageInFlight.has(cacheKey)) {
    return discoverPageInFlight.get(cacheKey);
  }
  const req = invoke('load_catalog_page', params)
    .then((items) => {
      if (Array.isArray(items) && items.length) {
        if (discoverPageMemCache.has(cacheKey)) discoverPageMemCache.delete(cacheKey);
        discoverPageMemCache.set(cacheKey, { at: Date.now(), items });
        while (discoverPageMemCache.size > 60) {
          const oldest = discoverPageMemCache.keys().next().value;
          if (!oldest) break;
          discoverPageMemCache.delete(oldest);
        }
      }
      return items;
    })
    .finally(() => discoverPageInFlight.delete(cacheKey));
  discoverPageInFlight.set(cacheKey, req);
  return req;
}

function maybeInfiniteDiscover() {
  if (!discoverLoading && discoverHasMore) {
    loadDiscoverPage(false);
  }
}

async function loadDiscoverPage(reset) {
  const activeCat = discoverCatalogs.find((c) => c.key === discoverSelectedKey);
  if (!activeCat || (!reset && discoverLoading)) return;
  const request = ++discoverRequest;
  discoverLoading = true;

  const gridEl = document.getElementById('discover-grid');
  const statusEl = document.getElementById('discover-status');
  const loadBtn = document.getElementById('btn-discover-load-more');
  const skipVal = reset ? 0 : discoverItems.length;
  const reqParams = {
    manifestUrl: activeCat.manifestUrl,
    mediaType: activeCat.type,
    catalogId: activeCat.catalogId,
    skip: skipVal,
    genre: discoverSelectedGenre,
    search: null,
  };
  const cacheKey = `${reqParams.manifestUrl}|${reqParams.mediaType}|${reqParams.catalogId}|${reqParams.skip}|${reqParams.genre || ''}`;
  const hasWarmCache = discoverPageMemCache.has(cacheKey);

  if (reset) {
    discoverItems = [];
    discoverSeen.clear();
    gridEl.innerHTML = '';
    if (!hasWarmCache) statusEl.textContent = 'Carregando catálogo…';
  }
  loadBtn.disabled = true;

  try {
    const page = await fetchCatalogPageCached(reqParams);
    if (request !== discoverRequest) return;
    const added = [];
    for (const item of page) {
      const k = `${item.type}:${item.id}`;
      if (!discoverSeen.has(k)) {
        discoverSeen.add(k);
        discoverItems.push(item);
        added.push(item);
      }
    }
    if (added.length) {
      const frag = document.createElement('div');
      frag.innerHTML = added.map((it) => renderPosterCard(it)).join('');
      wirePosterInteractions(frag);
      while (frag.firstElementChild) gridEl.appendChild(frag.firstElementChild);
    }
    discoverHasMore = page.length >= 10 && added.length > 0;
    loadBtn.classList.toggle('hidden', !discoverHasMore);
    statusEl.textContent = discoverItems.length ? `${discoverItems.length} item(ns)` : 'Nenhum item neste catálogo.';
    if (discoverHasMore && reset) {
      setTimeout(() => {
        if (request === discoverRequest) {
          fetchCatalogPageCached({ ...reqParams, skip: discoverItems.length }).catch(() => {});
        }
      }, 300);
    }
  } catch (e) {
    if (request !== discoverRequest) return;
    statusEl.textContent = String(e);
    discoverHasMore = false;
    loadBtn.classList.add('hidden');
  } finally {
    if (request === discoverRequest) {
      discoverLoading = false;
      loadBtn.disabled = false;
    }
  }
}

document.getElementById('btn-discover-load-more').addEventListener('click', () => loadDiscoverPage(false));

// --- LIBRARY (LibraryScreen.kt filters & layout toggle) ---
let librarySubTab = 'saved';

function setLibrarySubTab(nextTab) {
  librarySubTab = nextTab === 'following' || nextTab === 'calendar' ? nextTab : 'saved';
  document.querySelectorAll('[data-library-tab]').forEach((btn) => {
    const active = btn.dataset.libraryTab === librarySubTab;
    btn.classList.toggle('active', active);
    btn.setAttribute('aria-selected', active ? 'true' : 'false');
  });
  document.getElementById('btn-library-calendar')?.classList.toggle('is-active', librarySubTab === 'calendar');
  const savedPanel = document.getElementById('library-panel-saved');
  const followingPanel = document.getElementById('library-panel-following');
  const calendarPanel = document.getElementById('library-panel-calendar');
  if (savedPanel) savedPanel.hidden = librarySubTab !== 'saved';
  if (followingPanel) followingPanel.hidden = librarySubTab !== 'following';
  if (calendarPanel) calendarPanel.hidden = librarySubTab !== 'calendar';
  if (librarySubTab === 'following') {
    renderFollowingSection();
  } else if (librarySubTab === 'calendar') {
    loadLibraryCalendar();
  } else {
    renderFilteredLibrary();
  }
}

document.querySelectorAll('[data-library-tab]').forEach((btn) => {
  btn.addEventListener('click', () => setLibrarySubTab(btn.dataset.libraryTab));
});

// --- LIBRARY CALENDAR (Desktop-first release planner) ---
const LIBRARY_CALENDAR_CACHE_KEY = 'nuvio-library-calendar-v6';
const LIBRARY_CALENDAR_TTL_MS = 3 * 60 * 60 * 1000;
const LIBRARY_CALENDAR_MAX_SERIES = 120;
const LIBRARY_CALENDAR_LOOKBACK_DAYS = 7;
const LIBRARY_CALENDAR_WINDOW_DAYS = 210;
const LIBRARY_CALENDAR_MONTHS = ['Janeiro', 'Fevereiro', 'Março', 'Abril', 'Maio', 'Junho', 'Julho', 'Agosto', 'Setembro', 'Outubro', 'Novembro', 'Dezembro'];
const LIBRARY_CALENDAR_MONTHS_SHORT = ['jan', 'fev', 'mar', 'abr', 'mai', 'jun', 'jul', 'ago', 'set', 'out', 'nov', 'dez'];
const LIBRARY_CALENDAR_WEEKDAYS = ['Seg', 'Ter', 'Qua', 'Qui', 'Sex', 'Sáb', 'Dom'];
const LIBRARY_CALENDAR_WEEKDAYS_SHORT = ['dom', 'seg', 'ter', 'qua', 'qui', 'sex', 'sáb'];
let libraryCalendarLoading = false;
let libraryCalendarPromise = null;
let libraryCalendarPromiseSig = '';
const libraryCalendarState = { rows: [], month: null, selected: null, view: 'month', scope: 'all', query: '' };
const CALENDAR_CHEVRON_LEFT = '<svg viewBox="0 0 24 24" width="18" height="18"><path fill="currentColor" d="M15.41 16.59 14 18l-6-6 6-6 1.41 1.41L10.83 12z"/></svg>';
const CALENDAR_CHEVRON_RIGHT = '<svg viewBox="0 0 24 24" width="18" height="18"><path fill="currentColor" d="M8.59 16.59 10 18l6-6-6-6-1.41 1.41L13.17 12z"/></svg>';
const CALENDAR_REFRESH_SVG = '<svg viewBox="0 0 24 24" width="16" height="16"><path fill="currentColor" d="M17.65 6.35A7.958 7.958 0 0 0 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08A5.99 5.99 0 0 1 12 18c-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4z"/></svg>';

function readLibraryCalendarCache() {
  try {
    return JSON.parse(localStorage.getItem(LIBRARY_CALENDAR_CACHE_KEY) || 'null');
  } catch {
    return null;
  }
}

function librarySeriesForCalendar() {
  const seen = new Set();
  const out = [];
  for (const it of libraryRawItems) {
    const type = String(it.content_type || it.type || '').toLowerCase();
    if (type !== 'series' && type !== 'tv' && type !== 'show') continue;
    const id = String(it.content_id || it.id || '').trim();
    if (!id || seen.has(id)) continue;
    seen.add(id);
    out.push({ id, name: it.name || it.title || id, poster: it.poster || '' });
  }
  return out;
}

function calendarParseDay(iso) {
  const date = new Date(`${String(iso || '').slice(0, 10)}T00:00:00`);
  return Number.isNaN(date.getTime()) ? null : date;
}

function calendarDayKey(date) {
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`;
}

function calendarStartOfMonth(date) {
  return new Date(date.getFullYear(), date.getMonth(), 1);
}

function calendarMonthTitle(date) {
  return `${LIBRARY_CALENDAR_MONTHS[date.getMonth()]} ${date.getFullYear()}`;
}

function calendarLongDate(date) {
  const weekday = date.toLocaleDateString('pt-BR', { weekday: 'long' });
  return `${weekday.charAt(0).toUpperCase()}${weekday.slice(1)}, ${date.getDate()} de ${LIBRARY_CALENDAR_MONTHS[date.getMonth()].toLowerCase()}`;
}

function calendarRelativeLabel(date) {
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  const diff = Math.round((date - today) / 86400000);
  if (diff < 0) return 'Já disponível';
  if (diff === 0) return 'Hoje';
  if (diff === 1) return 'Amanhã';
  return `Em ${diff} dias`;
}

function calendarShortDayLabel(iso) {
  const d = calendarParseDay(iso);
  if (!d) return '';
  return `${LIBRARY_CALENDAR_WEEKDAYS_SHORT[d.getDay()]}, ${d.getDate()} ${LIBRARY_CALENDAR_MONTHS_SHORT[d.getMonth()]}`;
}

function calendarScheduleNote(row) {
  const parts = [];
  if (row.streamingDate && row.tvDate && row.streamingDate !== row.tvDate) {
    parts.push(`Streaming: ${calendarShortDayLabel(row.streamingDate)}`);
    parts.push(`TV: ${calendarShortDayLabel(row.tvDate)}`);
  } else if (row.streamingDate && row.platform) {
    parts.push(`${row.platform} · ${calendarShortDayLabel(row.streamingDate)}`);
  } else if (row.tvDate && row.network) {
    parts.push(`TV (${row.network}) · ${calendarShortDayLabel(row.tvDate)}`);
  }
  return parts.join(' • ');
}

function isCalendarEpisodeWatched(row) {
  if (!row.id || !row.season || !row.episode) return false;
  return watchedSet.has(`${row.id}:${row.season}:${row.episode}`)
    || (row.imdbId ? watchedSet.has(`${row.imdbId}:${row.season}:${row.episode}`) : false);
}

function calendarReleaseLabel(row) {
  const code = row.season > 0 ? `T${row.season}E${row.episode}` : `Ep. ${row.episode}`;
  return row.episodeTitle ? `${code} · ${row.episodeTitle}` : code;
}

function collectHintedCalendarTmdbIds(series, stored) {
  const ids = {};
  for (const item of series) {
    const inline = /^tmdb:(\d+)$/.exec(item.id);
    if (inline) {
      ids[item.id] = Number(inline[1]);
      continue;
    }
    const cached = Number(stored?.[item.id]);
    if (cached > 0) ids[item.id] = cached;
  }
  return ids;
}

function renderLibraryCalendarStateCard(wrap, message, showRetry = true) {
  if (!wrap) return;
  wrap.innerHTML = `
    <div class="library-calendar-shell is-desktop-cal cal-state-card">
      <div class="cal-state-body">
        <h3 class="calendar-month">Calendário de estreias</h3>
        <p class="cal-state-msg">${esc(message)}</p>
        ${showRetry ? '<button type="button" class="calendar-today-btn" data-calendar-refresh>Atualizar calendário</button>' : ''}
      </div>
    </div>`;
  wrap.querySelector('[data-calendar-refresh]')?.addEventListener('click', () => {
    loadLibraryCalendar(true);
  });
}

function renderLibraryCalendarSkeleton(wrap, seriesCount) {
  if (!wrap) return;
  const now = new Date();
  const skeletonCells = Array.from({ length: 35 }, () => '<span class="calendar-cell cal-skeleton-cell"></span>').join('');
  wrap.innerHTML = `
    <div class="library-calendar-shell is-desktop-cal is-loading">
      <div class="cal-desktop-toolbar">
        <div class="cal-toolbar-left">
          <h3 class="calendar-month">${esc(calendarMonthTitle(calendarStartOfMonth(now)))}</h3>
          <div class="cal-stats-row">
            <span class="cal-stat-pill is-highlight">Carregando ${seriesCount} ${seriesCount === 1 ? 'série' : 'séries'} em paralelo…</span>
          </div>
        </div>
      </div>
      <div class="cal-month-split">
        <section class="calendar-main">
          <div class="calendar-weekdays">
            ${LIBRARY_CALENDAR_WEEKDAYS.map((label) => `<span class="calendar-weekday">${label}</span>`).join('')}
          </div>
          <div class="calendar-grid">${skeletonCells}</div>
        </section>
        <aside class="calendar-agenda">
          <div class="calendar-agenda-head">
            <div>
              <h3 class="calendar-agenda-title">Sincronizando datas oficiais…</h3>
              <span class="calendar-agenda-date">Buscando TMDB e TVMaze simultaneamente</span>
            </div>
          </div>
          <div class="cal-skeleton-list">
            <div class="cal-skeleton-row"></div>
            <div class="cal-skeleton-row"></div>
            <div class="cal-skeleton-row"></div>
          </div>
        </aside>
      </div>
    </div>`;
}

function buildLibraryCalendarRows(items, tmdbIds) {
  const byId = new Map();
  const byTmdb = new Map();
  for (const item of librarySeriesForCalendar()) {
    byId.set(item.id, item);
    const id = Number(tmdbIds?.[item.id]);
    if (id > 0 && !byTmdb.has(id)) byTmdb.set(id, item);
  }
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  const start = new Date(today.getTime() - LIBRARY_CALENDAR_LOOKBACK_DAYS * 86400000);
  const end = new Date(today.getTime() + LIBRARY_CALENDAR_WINDOW_DAYS * 86400000);
  const seen = new Set();
  const dated = [];
  const undated = [];
  for (const it of Array.isArray(items) ? items : []) {
    const series = (it.id ? byId.get(String(it.id)) : null) || byTmdb.get(Number(it.tmdb_id));
    const date = calendarParseDay(it.released);
    // Episódios já listados mas ainda sem data vão para o fim da agenda.
    if (!date && !it.undated) continue;
    const row = {
      date,
      day: date ? calendarDayKey(date) : '',
      poster: it.poster || series?.poster || '',
      backdrop: it.backdrop || '',
      name: series?.name || it.name || '',
      id: series?.id || it.id || '',
      imdbId: it.imdb_id || '',
      tmdbId: Number(it.tmdb_id) || 0,
      season: Number(it.season) || 0,
      episode: Number(it.episode) || 0,
      episodeTitle: it.title || '',
      overview: it.overview || '',
      streamingDate: it.streaming_date || '',
      tvDate: it.tv_date || '',
      airstamp: it.airstamp || '',
      platform: it.platform || '',
      network: it.network || '',
      scheduleDays: Array.isArray(it.schedule_days) ? it.schedule_days : [],
      undated: !date,
    };
    const key = `${row.id || row.tmdbId}:${row.day}:${row.season}:${row.episode}`;
    if (seen.has(key)) continue;
    seen.add(key);
    if (!date) undated.push(row);
    else if (date >= start && date <= end) dated.push(row);
  }
  dated.sort((a, b) => a.date - b.date || a.name.localeCompare(b.name));
  undated.sort((a, b) => a.name.localeCompare(b.name) || a.episode - b.episode);
  return [...dated, ...undated];
}

function renderLibraryCalendar(items, tmdbIds) {
  const wrap = document.getElementById('library-calendar');
  if (!wrap) return;
  libraryCalendarState.rows = buildLibraryCalendarRows(items, tmdbIds);
  if (!libraryCalendarState.rows.length) {
    renderLibraryCalendarStateCard(wrap, 'Sem estreias anunciadas no período atual para as séries da biblioteca.');
    return;
  }
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  const todayKey = calendarDayKey(today);
  const firstDated = libraryCalendarState.rows.find((row) => !row.undated);
  libraryCalendarState.selected = libraryCalendarState.rows.some((row) => row.day === todayKey)
    ? todayKey
    : (firstDated?.day || todayKey);
  libraryCalendarState.month = calendarStartOfMonth(
    calendarParseDay(libraryCalendarState.selected) || today,
  );
  drawLibraryCalendar();
}

function calendarEpisodeCode(row) {
  return row.season > 0 ? `T${row.season}E${row.episode}` : `Ep. ${row.episode}`;
}

function calendarWeekStart(date) {
  const d = new Date(date.getFullYear(), date.getMonth(), date.getDate());
  const offset = (d.getDay() + 6) % 7;
  d.setDate(d.getDate() - offset);
  return d;
}

function calendarWeekRangeLabel(weekStart) {
  const weekEnd = new Date(weekStart.getFullYear(), weekStart.getMonth(), weekStart.getDate() + 6);
  const startLabel = `${weekStart.getDate()} ${LIBRARY_CALENDAR_MONTHS_SHORT[weekStart.getMonth()]}`;
  const endLabel = `${weekEnd.getDate()} ${LIBRARY_CALENDAR_MONTHS_SHORT[weekEnd.getMonth()]} ${weekEnd.getFullYear()}`;
  return `${startLabel} – ${endLabel}`;
}

function drawLibraryCalendar() {
  const wrap = document.getElementById('library-calendar');
  const allRows = libraryCalendarState.rows;
  if (!wrap || !allRows.length) return;

  const q = libraryCalendarState.query.trim().toLowerCase();
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  const todayKey = calendarDayKey(today);
  const tomorrowKey = calendarDayKey(new Date(today.getTime() + 86400000));
  const weekStartOfToday = calendarWeekStart(today);
  const weekStartKey = calendarDayKey(weekStartOfToday);
  const weekEndKey = calendarDayKey(new Date(weekStartOfToday.getTime() + 6 * 86400000));
  const next7EndKey = calendarDayKey(new Date(today.getTime() + 7 * 86400000));
  const scope = libraryCalendarState.scope || 'all';

  let rows = allRows;
  if (scope === 'week') {
    rows = rows.filter((r) => r.day && r.day >= weekStartKey && r.day <= weekEndKey);
  } else if (scope === 'next7') {
    rows = rows.filter((r) => r.day && r.day >= todayKey && r.day <= next7EndKey);
  }
  if (q) {
    rows = rows.filter((r) =>
      r.name.toLowerCase().includes(q)
      || r.episodeTitle.toLowerCase().includes(q)
      || r.platform.toLowerCase().includes(q)
      || r.network.toLowerCase().includes(q)
    );
  }

  const month = libraryCalendarState.month || calendarStartOfMonth(today);
  const monthKey = `${month.getFullYear()}-${String(month.getMonth() + 1).padStart(2, '0')}`;
  const selected = libraryCalendarState.selected || todayKey;
  const view = libraryCalendarState.view || 'month';

  const byDayAll = new Map();
  for (const row of rows) {
    if (!row.day) continue;
    if (!byDayAll.has(row.day)) byDayAll.set(row.day, []);
    byDayAll.get(row.day).push(row);
  }

  let monthCount = 0;
  let todayCount = 0;
  let weekCount = 0;
  for (const row of rows) {
    if (!row.day) continue;
    if (row.day.startsWith(monthKey)) monthCount += 1;
    if (row.day === todayKey) todayCount += 1;
    if (row.day >= weekStartKey && row.day <= weekEndKey) weekCount += 1;
  }

  const datedRows = rows.filter((row) => row.date);
  const undatedRows = rows.filter((row) => row.undated);
  const firstMonth = calendarStartOfMonth(datedRows.length ? datedRows[0].date : today);
  const lastMonth = calendarStartOfMonth(datedRows.length ? datedRows[datedRows.length - 1].date : today);
  const canPrev = view === 'week' ? true : month.getTime() > firstMonth.getTime();
  const canNext = view === 'week' ? true : month.getTime() < lastMonth.getTime();

  const calendarItemHtml = (row, compact = false) => {
    const note = calendarScheduleNote(row);
    const serviceBadge = row.platform || row.network;
    const watched = isCalendarEpisodeWatched(row);
    return `
    <button class="calendar-item${compact ? ' is-compact' : ''}${watched ? ' is-watched' : ''}" type="button" data-calendar-id="${esc(row.id)}" data-calendar-name="${esc(row.name)}" data-calendar-poster="${esc(row.poster)}">
      <span class="calendar-item-art">
        ${row.backdrop || row.poster ? `<img src="${esc(row.backdrop || row.poster)}" alt="" loading="lazy" decoding="async" />` : ''}
      </span>
      <span class="calendar-item-body">
        <span class="calendar-item-top">
          <span class="calendar-item-name">${esc(row.name)}</span>
          ${serviceBadge ? `<span class="cal-platform-pill">${esc(serviceBadge)}</span>` : ''}
          ${watched ? '<span class="cal-watched-pill" title="Episódio assistido">&#10003;</span>' : ''}
        </span>
        <span class="calendar-item-ep">${esc(calendarReleaseLabel(row))}</span>
        ${note ? `<span class="calendar-item-schedule">${esc(note)}</span>` : ''}
        ${!compact && row.overview ? `<span class="calendar-item-overview">${esc(row.overview)}</span>` : ''}
      </span>
      <span class="calendar-item-chevron">${CALENDAR_CHEVRON_RIGHT}</span>
    </button>`;
  };

  const selectedDateObj = calendarParseDay(selected) || today;
  const activeWeekStart = calendarWeekStart(selectedDateObj);

  const headingTitle = view === 'week'
    ? calendarWeekRangeLabel(activeWeekStart)
    : calendarMonthTitle(month);

  const statsHtml = [
    `<span class="cal-stat-pill">${monthCount} ${monthCount === 1 ? 'episódio' : 'episódios'} em ${LIBRARY_CALENDAR_MONTHS[month.getMonth()].toLowerCase()}</span>`,
    todayCount ? `<span class="cal-stat-pill is-highlight">${todayCount} hoje</span>` : '',
    weekCount ? `<span class="cal-stat-pill">${weekCount} esta semana</span>` : '',
  ].filter(Boolean).join('');

  let mainBodyHtml = '';

  if (view === 'month') {
    const firstOfMonth = new Date(month.getFullYear(), month.getMonth(), 1);
    const daysInMonth = new Date(month.getFullYear(), month.getMonth() + 1, 0).getDate();
    const lead = (firstOfMonth.getDay() + 6) % 7;
    const cells = [];
    for (let i = 0; i < lead; i += 1) cells.push('<span class="calendar-cell is-empty"></span>');
    for (let day = 1; day <= daysInMonth; day += 1) {
      const key = `${monthKey}-${String(day).padStart(2, '0')}`;
      const list = byDayAll.get(key) || [];
      const cls = ['calendar-cell'];
      if (list.length) cls.push('has-release');
      if (key === todayKey) cls.push('is-today');
      if (key === selected) cls.push('is-selected');
      const chipsHtml = list.slice(0, 3).map((row) => `
        <span class="cal-day-chip" title="${esc(`${row.name} — ${calendarReleaseLabel(row)}`)}">
          ${row.poster ? `<img src="${esc(row.poster)}" alt="" loading="lazy" decoding="async" />` : '<span class="cal-day-chip-dot"></span>'}
          <span class="cal-day-chip-name">${esc(row.name)}</span>
          <span class="cal-day-chip-ep">${esc(calendarEpisodeCode(row))}</span>
        </span>
      `).join('');
      const overflowHtml = list.length > 3
        ? `<span class="cal-day-more">+${list.length - 3} mais</span>`
        : '';
      cells.push(`
        <button type="button" class="${cls.join(' ')}" data-calendar-day="${key}">
          <span class="calendar-cell-head">
            <span class="calendar-cell-num">${day}</span>
            ${key === todayKey ? '<span class="cal-cell-today-tag">Hoje</span>' : ''}
          </span>
          <span class="cal-cell-events">${chipsHtml}${overflowHtml}</span>
        </button>`);
    }
    while (cells.length % 7 !== 0) cells.push('<span class="calendar-cell is-empty"></span>');

    const selectedDayRows = byDayAll.get(selected) || [];
    const upcomingAfterSelected = rows.filter((r) => r.day && r.day > selected).slice(0, 24);
    const upcomingGroups = [];
    for (const row of upcomingAfterSelected) {
      const last = upcomingGroups[upcomingGroups.length - 1];
      if (last && last.day === row.day) last.rows.push(row);
      else upcomingGroups.push({ day: row.day, rows: [row] });
    }

    mainBodyHtml = `
      <div class="cal-month-split">
        <section class="calendar-main">
          <div class="calendar-weekdays">
            ${LIBRARY_CALENDAR_WEEKDAYS.map((label) => `<span class="calendar-weekday">${label}</span>`).join('')}
          </div>
          <div class="calendar-grid">${cells.join('')}</div>
        </section>
        <aside class="calendar-agenda">
          <div class="calendar-agenda-head">
            <div>
              <h3 class="calendar-agenda-title">${esc(calendarLongDate(selectedDateObj))}</h3>
              <span class="calendar-agenda-date">${selectedDayRows.length ? `${selectedDayRows.length} ${selectedDayRows.length === 1 ? 'estreia neste dia' : 'estreias neste dia'}` : 'Nenhum episódio neste dia'}</span>
            </div>
            ${selected <= tomorrowKey ? `<span class="calendar-day-now">${esc(calendarRelativeLabel(selectedDateObj))}</span>` : ''}
          </div>
          <div class="calendar-list">
            ${selectedDayRows.length
              ? `<div class="calendar-day-items">${selectedDayRows.map((r) => calendarItemHtml(r)).join('')}</div>`
              : '<div class="calendar-empty">Selecione um dia destacado ou veja os próximos lançamentos abaixo.</div>'}
            ${upcomingGroups.length ? `
              <div class="cal-upcoming-divider">Próximos dias</div>
              ${upcomingGroups.map((group) => {
                const date = calendarParseDay(group.day);
                const near = group.day <= tomorrowKey;
                return `
                <section class="calendar-day-group">
                  <header class="calendar-day-head">
                    <h4 class="calendar-day-title">${esc(calendarLongDate(date))}</h4>
                    ${near ? `<span class="calendar-day-now">${esc(calendarRelativeLabel(date))}</span>` : ''}
                  </header>
                  <div class="calendar-day-items">${group.rows.map((r) => calendarItemHtml(r, true)).join('')}</div>
                </section>`;
              }).join('')}
            ` : ''}
            ${undatedRows.length ? `
              <details class="cal-undated-drawer">
                <summary>Data a anunciar (${undatedRows.length})</summary>
                <div class="calendar-day-items">${undatedRows.map((r) => calendarItemHtml(r, true)).join('')}</div>
              </details>` : ''}
          </div>
        </aside>
      </div>`;
  } else if (view === 'week') {
    const cols = [];
    for (let i = 0; i < 7; i += 1) {
      const d = new Date(activeWeekStart.getFullYear(), activeWeekStart.getMonth(), activeWeekStart.getDate() + i);
      const key = calendarDayKey(d);
      const dayRows = byDayAll.get(key) || [];
      const isToday = key === todayKey;
      const isSelected = key === selected;
      cols.push(`
        <div class="cal-week-col${isToday ? ' is-today' : ''}${isSelected ? ' is-selected' : ''}" data-calendar-day="${key}">
          <div class="cal-week-col-head">
            <span class="cal-week-col-dow">${LIBRARY_CALENDAR_WEEKDAYS[i]}</span>
            <span class="cal-week-col-date">${d.getDate()} ${LIBRARY_CALENDAR_MONTHS_SHORT[d.getMonth()]}</span>
            ${isToday ? '<span class="calendar-day-now">Hoje</span>' : ''}
          </div>
          <div class="cal-week-col-list">
            ${dayRows.length ? dayRows.map((row) => `
              <button type="button" class="cal-week-card calendar-item" data-calendar-id="${esc(row.id)}" data-calendar-name="${esc(row.name)}" data-calendar-poster="${esc(row.poster)}">
                ${row.backdrop || row.poster ? `<div class="cal-week-card-art"><img src="${esc(row.backdrop || row.poster)}" alt="" loading="lazy" decoding="async" /></div>` : ''}
                <div class="cal-week-card-body">
                  <span class="cal-week-card-code">${esc(calendarEpisodeCode(row))}</span>
                  <strong class="cal-week-card-name">${esc(row.name)}</strong>
                  ${row.episodeTitle ? `<span class="cal-week-card-ep">${esc(row.episodeTitle)}</span>` : ''}
                </div>
              </button>
            `).join('') : '<div class="cal-week-empty">—</div>'}
          </div>
        </div>`);
    }
    mainBodyHtml = `<div class="cal-week-board">${cols.join('')}</div>`;
  } else {
    const groups = [];
    for (const row of datedRows) {
      const last = groups[groups.length - 1];
      if (last && last.day === row.day) last.rows.push(row);
      else groups.push({ day: row.day, rows: [row] });
    }
    mainBodyHtml = `
      <div class="cal-timeline-list">
        ${groups.map((group) => {
          const date = calendarParseDay(group.day);
          const near = group.day <= tomorrowKey;
          return `
          <section class="cal-timeline-group">
            <header class="calendar-day-head">
              <h4 class="calendar-day-title">${esc(calendarLongDate(date))}</h4>
              ${near ? `<span class="calendar-day-now">${esc(calendarRelativeLabel(date))}</span>` : ''}
            </header>
            <div class="cal-timeline-grid">${group.rows.map((r) => calendarItemHtml(r)).join('')}</div>
          </section>`;
        }).join('')}
        ${undatedRows.length ? `
          <section class="cal-timeline-group">
            <header class="calendar-day-head"><h4 class="calendar-day-title">Data a anunciar</h4></header>
            <div class="cal-timeline-grid">${undatedRows.map((r) => calendarItemHtml(r, true)).join('')}</div>
          </section>` : ''}
        ${!groups.length && !undatedRows.length ? '<div class="calendar-empty">Nenhum episódio encontrado para o filtro atual.</div>' : ''}
      </div>`;
  }

  wrap.innerHTML = `
    <div class="library-calendar-shell is-desktop-cal">
      <div class="cal-desktop-toolbar">
        <div class="cal-toolbar-left">
          <h3 class="calendar-month">${esc(headingTitle)}</h3>
          <div class="cal-stats-row">${statsHtml}</div>
          <div class="cal-scope-pills" role="group" aria-label="Período do calendário">
            <button type="button" class="cal-scope-btn${scope === 'all' ? ' active' : ''}" data-calendar-scope="all">Todos</button>
            <button type="button" class="cal-scope-btn${scope === 'week' ? ' active' : ''}" data-calendar-scope="week">Esta semana</button>
            <button type="button" class="cal-scope-btn${scope === 'next7' ? ' active' : ''}" data-calendar-scope="next7">Próx. 7 dias</button>
          </div>
        </div>
        <div class="cal-toolbar-right">
          <input type="search" class="cal-search-input" id="cal-series-filter" placeholder="Filtrar série ou plataforma…" value="${esc(libraryCalendarState.query)}" />
          <div class="cal-view-switch" role="tablist" aria-label="Modo de visualização do calendário">
            <button type="button" class="cal-view-btn${view === 'month' ? ' active' : ''}" data-calendar-view="month">Mês</button>
            <button type="button" class="cal-view-btn${view === 'week' ? ' active' : ''}" data-calendar-view="week">Semana</button>
            <button type="button" class="cal-view-btn${view === 'list' ? ' active' : ''}" data-calendar-view="list">Lista</button>
          </div>
          <div class="calendar-nav">
            <button type="button" class="calendar-today-btn" data-calendar-today>Hoje</button>
            <button type="button" class="calendar-nav-btn" data-calendar-nav="-1" aria-label="Anterior" ${canPrev ? '' : 'disabled'}>${CALENDAR_CHEVRON_LEFT}</button>
            <button type="button" class="calendar-nav-btn" data-calendar-nav="1" aria-label="Próximo" ${canNext ? '' : 'disabled'}>${CALENDAR_CHEVRON_RIGHT}</button>
            <button type="button" class="calendar-nav-btn" data-calendar-refresh title="Atualizar calendário" aria-label="Atualizar calendário">${CALENDAR_REFRESH_SVG}</button>
          </div>
        </div>
      </div>
      ${mainBodyHtml}
    </div>`;

  wrap.querySelectorAll('[data-calendar-view]').forEach((btn) => {
    btn.addEventListener('click', () => {
      libraryCalendarState.view = btn.dataset.calendarView || 'month';
      drawLibraryCalendar();
    });
  });

  wrap.querySelectorAll('[data-calendar-scope]').forEach((btn) => {
    btn.addEventListener('click', () => {
      libraryCalendarState.scope = btn.dataset.calendarScope || 'all';
      drawLibraryCalendar();
    });
  });

  const filterInput = wrap.querySelector('#cal-series-filter');
  if (filterInput) {
    filterInput.addEventListener('input', (e) => {
      const caret = e.target.selectionStart;
      libraryCalendarState.query = e.target.value;
      drawLibraryCalendar();
      const restored = document.getElementById('cal-series-filter');
      if (restored) {
        restored.focus();
        if (caret != null) restored.setSelectionRange(caret, caret);
      }
    });
  }

  wrap.querySelector('[data-calendar-refresh]')?.addEventListener('click', () => {
    loadLibraryCalendar(true);
  });

  wrap.querySelectorAll('[data-calendar-nav]').forEach((btn) => {
    btn.addEventListener('click', () => {
      const delta = Number(btn.dataset.calendarNav) || 0;
      if (!delta) return;
      if (libraryCalendarState.view === 'week') {
        const cur = calendarParseDay(libraryCalendarState.selected) || today;
        const next = new Date(cur.getFullYear(), cur.getMonth(), cur.getDate() + delta * 7);
        libraryCalendarState.selected = calendarDayKey(next);
        libraryCalendarState.month = calendarStartOfMonth(next);
      } else {
        libraryCalendarState.month = new Date(month.getFullYear(), month.getMonth() + delta, 1);
      }
      drawLibraryCalendar();
    });
  });

  wrap.querySelector('[data-calendar-today]')?.addEventListener('click', () => {
    libraryCalendarState.selected = todayKey;
    libraryCalendarState.month = calendarStartOfMonth(today);
    drawLibraryCalendar();
  });

  wrap.querySelectorAll('[data-calendar-day]').forEach((el) => {
    el.addEventListener('click', (e) => {
      if (e.target.closest('.calendar-item')) return;
      libraryCalendarState.selected = el.dataset.calendarDay;
      drawLibraryCalendar();
    });
  });

  wrap.querySelectorAll('.calendar-item').forEach((el) => {
    el.addEventListener('click', () => {
      const id = el.dataset.calendarId;
      if (!id) return;
      openDetailsPage({ id, type: 'series', name: el.dataset.calendarName || '', poster: el.dataset.calendarPoster || '' });
    });
  });
}

async function loadLibraryCalendar(force = false, opts = {}) {
  const background = Boolean(opts?.background);
  const wrap = document.getElementById('library-calendar');
  const status = document.getElementById('library-calendar-status');
  const series = librarySeriesForCalendar();
  if (!series.length) {
    if (!background) {
      renderLibraryCalendarStateCard(wrap, 'Ainda não há séries na biblioteca para o calendário.', false);
      if (status) status.textContent = '';
    }
    return;
  }

  const limited = series.slice(0, LIBRARY_CALENDAR_MAX_SERIES);
  const signature = limited.map((s) => s.id).join(',');
  const cached = readLibraryCalendarCache();
  const cacheFresh = !force
    && cached?.signature === signature
    && Array.isArray(cached.items)
    && Date.now() - Number(cached.at || 0) < LIBRARY_CALENDAR_TTL_MS
    && (cached.items.length > 0 || cached.emptyOk === true);

  if (cacheFresh) {
    if (!background || librarySubTab === 'calendar') {
      renderLibraryCalendar(cached.items, cached.tmdbIds);
      if (status) status.textContent = '';
    }
    return;
  }

  if (!background && wrap && !wrap.querySelector('.cal-desktop-toolbar')) {
    renderLibraryCalendarSkeleton(wrap, limited.length);
    if (status) status.textContent = '';
  }

  // Se o pré-carregamento em paralelo com a biblioteca já estiver rodando, aguarda a mesma requisição.
  if (libraryCalendarPromise && libraryCalendarPromiseSig === signature && !force) {
    await libraryCalendarPromise;
    if (!background || librarySubTab === 'calendar') {
      const updated = readLibraryCalendarCache();
      if (updated?.signature === signature && Array.isArray(updated.items)) {
        renderLibraryCalendar(updated.items, updated.tmdbIds);
      }
      if (status) status.textContent = '';
    }
    return;
  }

  libraryCalendarLoading = true;
  libraryCalendarPromiseSig = signature;
  const runPromise = (async () => {
    const hintedIds = collectHintedCalendarTmdbIds(limited, cached?.tmdbIds);
    const today = new Date();
    today.setHours(0, 0, 0, 0);
    const lookback = new Date(today.getTime() - LIBRARY_CALENDAR_LOOKBACK_DAYS * 86400000);
    const seriesPayload = limited.map((s) => ({
      id: s.id,
      name: s.name,
      tmdbId: Number(hintedIds?.[s.id]) || null,
      imdbId: /^tt\d+$/i.test(s.id) ? s.id : null,
      poster: s.poster || null,
    }));

    let failed = false;
    const items = (await invoke('calendar_episodes', {
      series: seriesPayload,
      from: calendarDayKey(lookback),
      force,
    }).catch((err) => {
      console.error('Falha ao carregar calendário:', err);
      failed = true;
      return [];
    })) || [];

    const tmdbIds = { ...hintedIds };
    for (const it of items) {
      const tid = Number(it?.tmdb_id);
      if (it?.id && tid > 0) tmdbIds[String(it.id)] = tid;
    }

    if (!failed) {
      try {
        localStorage.setItem(LIBRARY_CALENDAR_CACHE_KEY, JSON.stringify({
          signature,
          at: Date.now(),
          tmdbIds,
          items,
          emptyOk: items.length === 0,
        }));
      } catch {}
    }

    if (!background || librarySubTab === 'calendar') {
      if (failed && !items.length) {
        renderLibraryCalendarStateCard(wrap, 'Não foi possível carregar o calendário agora. Tente atualizar.');
      } else {
        renderLibraryCalendar(items, tmdbIds);
      }
      if (status) status.textContent = '';
    }
  })().finally(() => {
    if (libraryCalendarPromise === runPromise) {
      libraryCalendarPromise = null;
      libraryCalendarLoading = false;
    }
  });

  libraryCalendarPromise = runPromise;
  await runPromise;
}

function preloadLibraryCalendar() {
  if (!librarySeriesForCalendar().length) return;
  loadLibraryCalendar(false, { background: true }).catch(() => {});
}

function toggleLibraryCalendar() {
  setLibrarySubTab(librarySubTab === 'calendar' ? 'saved' : 'calendar');
}

document.getElementById('btn-library-calendar')?.addEventListener('click', toggleLibraryCalendar);

document.getElementById('btn-library-layout')?.addEventListener('click', () => {
  libraryListMode = !libraryListMode;
  renderFilteredLibrary();
  renderFollowingSection();
});

['library-filter-q', 'library-filter-type', 'library-filter-watched', 'library-sort', 'library-mdblist-list'].forEach((id) => {
  const el = document.getElementById(id);
  if (!el) return;
  el.addEventListener('input', renderFilteredLibrary);
  el.addEventListener('change', renderFilteredLibrary);
});

['following-filter-q', 'following-filter-role', 'following-sort'].forEach((id) => {
  const el = document.getElementById(id);
  if (!el) return;
  el.addEventListener('input', renderFollowingSection);
  el.addEventListener('change', renderFollowingSection);
});

document.getElementById('btn-refresh-library')?.addEventListener('click', refreshLibrary);

const libraryCollections = document.getElementById('library-collections');
const libraryCollectionsList = document.getElementById('library-collections-list');
const libraryCollectionsStatus = document.getElementById('library-collections-status');
let collectionsRequest = 0;
let collectionCatalogRequest = 0;

libraryCollections?.addEventListener('toggle', () => {
  if (libraryCollections.open) refreshAccountCollections();
  else clearAccountCollections();
});

function clearAccountCollections() {
  collectionsRequest++;
  libraryCollectionsList?.replaceChildren();
  if (libraryCollectionsStatus) libraryCollectionsStatus.textContent = '';
}

async function refreshAccountCollections() {
  clearAccountCollections();
  const request = collectionsRequest;
  libraryCollectionsStatus.textContent = 'Carregando coleções…';
  try {
    const [collections, addons] = await Promise.all([
      invoke('pull_account_collections'),
      invoke('list_addons'),
    ]);
    if (request !== collectionsRequest) return;

    const enabledAddons = addons.filter((addon) => addon.enabled);
    for (const collection of collections) {
      const section = document.createElement('section');
      const heading = document.createElement('h3');
      heading.textContent = collection.title || 'Coleção';
      section.append(heading);
      for (const folder of collection.folders || []) {
        const row = document.createElement('div');
        row.className = 'library-controls-row';
        const label = document.createElement('span');
        label.textContent = folder.title || 'Pasta';
        row.append(label);
        const sources = folder.sources?.length ? folder.sources : (folder.catalogSources || []);
        for (const source of sources) {
          if (source.provider && source.provider.toLowerCase() !== 'addon') continue;
          const addon = enabledAddons.find((candidate) => candidate.manifest.id === source.addonId);
          const catalog = addon?.manifest.catalogs?.find(
            (candidate) => candidate.id === source.catalogId && candidate.type === source.type
          );
          if (!catalog) continue;

          const genre = source.genre?.trim();
          const button = document.createElement('button');
          button.className = 'btn-ghost';
          button.textContent = [catalog.name || catalog.id, genre].filter(
            (part) => part && part.toLowerCase() !== 'none'
          ).join(' · ');
          button.addEventListener('click', async () => {
            const catalogRequest = ++collectionCatalogRequest;
            button.disabled = true;
            libraryCollectionsStatus.textContent = 'Carregando catálogo…';
            try {
              const params = {
                manifestUrl: addon.manifest_url,
                mediaType: catalog.type,
                catalogId: catalog.id,
                genre: genre && genre.toLowerCase() !== 'none' ? genre : null,
              };
              const items = await invoke('load_catalog_page', params);
              if (request !== collectionsRequest || catalogRequest !== collectionCatalogRequest ||
                  !libraryCollections.open || activeTabName() !== 'library') return;
              libraryCollectionsStatus.textContent = items.length ? '' : 'O catálogo não retornou títulos.';
              if (items.length) openFullCatalog({
                title: folder.title || catalog.name,
                addon_name: addon.manifest.name,
                manifest_url: params.manifestUrl,
                media_type: params.mediaType,
                catalog_id: params.catalogId,
                genre: params.genre,
                items,
              });
            } catch {
              if (request === collectionsRequest && catalogRequest === collectionCatalogRequest) {
                libraryCollectionsStatus.textContent = 'Não foi possível abrir o catálogo.';
              }
            } finally {
              button.disabled = false;
            }
          });
          row.append(button);
        }
        if (row.childElementCount === 1) {
          const unavailable = document.createElement('span');
          unavailable.className = 'muted';
          unavailable.textContent = 'Sem catálogo de addon ativo para esta pasta.';
          row.append(unavailable);
        }
        section.append(row);
      }
      libraryCollectionsList.append(section);
    }
    libraryCollectionsStatus.textContent = collections.length ? '' : 'Nenhuma coleção sincronizada neste perfil.';
  } catch {
    if (request === collectionsRequest) {
      libraryCollectionsStatus.textContent = 'Não foi possível carregar as coleções deste perfil.';
    }
  }
}

function renderFollowingSection() {
  const grid = document.getElementById('library-following-grid');
  const heading = document.getElementById('library-following-heading');
  const status = document.getElementById('library-status');
  const followingChip = document.querySelector('[data-library-tab="following"]');
  if (followingChip) {
    followingChip.textContent = followedPeopleList.length ? `Seguindo (${followedPeopleList.length})` : 'Seguindo';
  }
  if (!grid) return;

  const q = (document.getElementById('following-filter-q')?.value || '').trim().toLowerCase();
  const roleF = document.getElementById('following-filter-role')?.value || 'all';
  const sortF = document.getElementById('following-sort')?.value || 'recent';

  let list = [...followedPeopleList];
  if (q) {
    list = list.filter((p) => (p.name || p.id || '').toLowerCase().includes(q) || (p.role || '').toLowerCase().includes(q));
  }
  if (roleF === 'actor') {
    list = list.filter((p) => !p.role || /ator|atriz|acting|elenco/i.test(p.role));
  } else if (roleF === 'crew') {
    list = list.filter((p) => p.role && !/ator|atriz|acting|elenco/i.test(p.role));
  }
  if (sortF === 'name') {
    list.sort((a, b) => (a.name || a.id || '').localeCompare(b.name || b.id || ''));
  }

  if (heading) heading.style.display = list.length ? '' : 'none';
  if (librarySubTab === 'following' && status) {
    status.textContent = `${list.length} pessoa(s) seguida(s)`;
  }

  const cardsHtml = list.map((p) => {
    const displayName = p.name || (p.id ? `Ator #${p.id}` : 'Elenco');
    const roleLabel = p.role || 'Elenco';
    const keyAttr = esc(p.id || p.name);
    return `
      <div class="poster-card following-person-card" data-person-id="${esc(p.id)}" data-person-name="${esc(p.name)}" data-person-photo="${esc(p.photo)}" role="button" tabindex="0" title="${esc(displayName)}">
        <div class="poster-img-wrap">
          ${p.photo ? `<img class="following-person-img" src="${esc(p.photo)}" alt="${esc(displayName)}" loading="lazy" decoding="async" draggable="false" />` : `<div class="following-person-placeholder">${esc(displayName.charAt(0).toUpperCase())}</div>`}
          <button class="following-unfollow-btn" data-unfollow-key="${keyAttr}" type="button" title="Deixar de seguir ${esc(displayName)}" aria-label="Deixar de seguir ${esc(displayName)}">×</button>
        </div>
        <div class="poster-name">${esc(displayName)}</div>
        <div class="poster-sub">${esc(roleLabel)}</div>
      </div>
    `;
  }).join('');

  if (!cardsHtml) {
    grid.className = '';
    grid.innerHTML = '<div class="muted">Você ainda não segue ninguém. Abra um título, clique em um ator e toque em Seguir.</div>';
  } else if (libraryListMode) {
    grid.className = 'poster-grid following-poster-grid';
    grid.innerHTML = cardsHtml;
  } else {
    grid.className = 'shelf-row';
    grid.innerHTML = cardsHtml;
    wireHorizontalShelfDrag(grid.parentElement);
  }

  grid.querySelectorAll('[data-unfollow-key]').forEach((btn) => {
    btn.addEventListener('click', (e) => {
      e.stopPropagation();
      const card = btn.closest('.following-person-card');
      const name = card?.dataset.personName || 'Ator';
      toggleFollowPerson({ id: card?.dataset.personId || '', name: card?.dataset.personName || '' });
      showToast(`Deixou de seguir ${name}.`);
      renderFollowingSection();
    });
  });

  wirePosterInteractions(grid);
  hydrateMissingFollowedPhotos();
}

async function refreshLibrary() {
  if (libraryCollections?.open) refreshAccountCollections();
  const status = document.getElementById('library-status');
  if (librarySubTab === 'saved' && status) {
    status.textContent = 'Sincronizando biblioteca…';
  }
  preloadLibraryCalendar();
  await refreshLibraryState(true);
  renderFilteredLibrary();
  preloadLibraryCalendar();
}

function libraryItemToCard(it) {
  return renderPosterCard({
    id: it.content_id || it.id || '',
    type: it.content_type || it.type || 'movie',
    name: it.name || 'Item',
    poster: it.poster || '',
    banner: it.background || it.poster || '',
    logo: it.logo || '',
    description: it.description || '',
    release_info: it.release_info || '',
    imdb_rating: it.imdb_rating || '',
  });
}

function renderFilteredLibrary() {
  const status = document.getElementById('library-status');
  const grid = document.getElementById('library-grid');
  if (!grid) return;

  const source = activeLibrarySource();
  const mdbListSelect = document.getElementById('library-mdblist-list');
  if (mdbListSelect) {
    const lists = mdblistLibraryCache.lists || [];
    if (source === 'mdblist' && lists.length) {
      const prev = mdbListSelect.value;
      const opts = [{ key: 'all', name: 'Todas as listas' }, ...lists];
      const nextSig = opts.map((o) => `${o.key}:${o.name}`).join('|');
      if (mdbListSelect.dataset.sig !== nextSig) {
        mdbListSelect.dataset.sig = nextSig;
        mdbListSelect.innerHTML = opts.map((o) => `<option value="${esc(o.key)}">${esc(o.name)}</option>`).join('');
        mdbListSelect.value = opts.some((o) => o.key === prev) ? prev : 'watchlist';
      }
      mdbListSelect.classList.remove('hidden');
    } else {
      mdbListSelect.classList.add('hidden');
    }
  }

  const q = (document.getElementById('library-filter-q')?.value || '').trim().toLowerCase();
  const typeF = document.getElementById('library-filter-type')?.value || 'all';
  const watchF = document.getElementById('library-filter-watched')?.value || 'all';
  const sortF = document.getElementById('library-sort')?.value || 'recent';
  const listF = source === 'mdblist' && mdbListSelect && !mdbListSelect.classList.contains('hidden')
    ? mdbListSelect.value
    : 'all';

  let items = [...libraryRawItems];
  if (listF && listF !== 'all') {
    items = items.filter((it) => it.list_key === listF);
  } else if (source === 'mdblist') {
    const seen = new Set();
    items = items.filter((it) => {
      const k = libKey(it.content_id || it.id, it.content_type || it.type);
      if (seen.has(k)) return false;
      seen.add(k);
      return true;
    });
  }
  if (q) {
    items = items.filter((it) => (it.name || '').toLowerCase().includes(q));
  }
  if (typeF !== 'all') {
    items = items.filter((it) => (it.content_type || it.type || 'movie') === typeF);
  }
  if (watchF === 'watched') {
    items = items.filter((it) => watchedSet.has(it.content_id || it.id));
  } else if (watchF === 'unwatched') {
    items = items.filter((it) => !watchedSet.has(it.content_id || it.id));
  }

  if (sortF === 'oldest') {
    items.sort((a, b) => (Number(a.added_at) || 0) - (Number(b.added_at) || 0));
    if (items.every((it) => !it.added_at)) items.reverse();
  } else if (sortF === 'title') {
    items.sort((a, b) => (a.name || '').localeCompare(b.name || ''));
  } else if (sortF === 'title_desc') {
    items.sort((a, b) => (b.name || '').localeCompare(a.name || ''));
  } else if (sortF === 'year') {
    items.sort((a, b) => (parseInt(b.release_info, 10) || 0) - (parseInt(a.release_info, 10) || 0));
  } else {
    if (items.some((it) => Number(it.added_at) > 0)) {
      items.sort((a, b) => (Number(b.added_at) || 0) - (Number(a.added_at) || 0));
    }
  }

  if (librarySubTab === 'saved' && status) {
    status.textContent = `${items.length} item(ns) na biblioteca`;
  }

  if (!items.length) {
    grid.className = 'library-shelves';
    const emptyMsg = source === 'mdblist' && mdblistLibraryCache.error
      ? esc(mdblistLibraryCache.error)
      : 'Nenhum título encontrado na biblioteca.';
    grid.innerHTML = `<div class="muted">${emptyMsg}</div>`;
    return;
  }

  const buckets = [
    { key: 'movie', title: 'Movie', list: [] },
    { key: 'series', title: 'Series', list: [] },
    { key: 'other', title: 'Outros', list: [] },
  ];
  for (const it of items) {
    const t = String(it.content_type || it.type || 'movie').toLowerCase();
    if (t === 'movie') buckets[0].list.push(it);
    else if (t === 'series' || t === 'tv' || t === 'show') buckets[1].list.push(it);
    else buckets[2].list.push(it);
  }

  const containerClass = libraryListMode ? 'poster-grid' : 'shelf-row';
  grid.className = 'library-shelves';
  grid.innerHTML = buckets
    .filter((b) => b.list.length > 0)
    .map((b) => `
      <section class="library-shelf-section">
        <h3 class="library-shelf-title">${esc(b.title)}</h3>
        <div class="${containerClass}">${b.list.map(libraryItemToCard).join('')}</div>
      </section>
    `).join('');

  if (!libraryListMode) wireHorizontalShelfDrag(grid);
  wirePosterInteractions(grid);
}

// --- SETTINGS ---
function selectSettingsSection(id) {
  if (!id) return;
  if (id === "connected" || id === "cloud") refreshConnectedAccounts();
  if (id === "system") refreshDiagnostics();
  const sidebarKey = id === 'cloud' ? 'connected' : id;
  document.querySelectorAll('.settings-cat-btn').forEach((b) => {
    b.classList.toggle('active', b.dataset.settingsSec === sidebarKey);
  });
  document.querySelectorAll('.settings-section').forEach((s) => {
    s.classList.toggle('active', s.id === `sec-${id}`);
  });
  if (id === 'simkl' && typeof window.refreshMdblistAccountCard === 'function') {
    window.refreshMdblistAccountCard(true);
  }
}

document.querySelectorAll('.settings-cat-btn').forEach((btn) => {
  btn.addEventListener('click', () => selectSettingsSection(btn.dataset.settingsSec));
});

document.getElementById('settings-search-input').addEventListener('input', (e) => {
  const q = e.target.value.trim().toLowerCase();
  document.querySelectorAll('.settings-cat-btn').forEach((btn) => {
    btn.classList.toggle('hidden', Boolean(q) && !btn.textContent.toLowerCase().includes(q));
  });
});

function updateAccountSettingsUi() {
  document.getElementById('set-account-status').textContent = authState?.signed_in ? 'Conectado' : 'Convidado (Local)';
  document.getElementById('set-account-email').textContent = authState?.email || '-';
  const active = (authState?.profiles || []).find((p) => p.profile_index === authState?.active_profile_index);
  document.getElementById('set-account-profile').textContent = active?.name || 'Local';
}

document.getElementById('btn-switch-profile-screen').addEventListener('click', () => {
  if (authState?.profiles?.length) {
    renderProfilePicker(authState.profiles);
    showView('view-profiles');
  }
});

document.getElementById('btn-sign-out').addEventListener('click', async () => {
  ++simklGeneration;
  simklAccount = {};
  await invoke('sign_out');
  showView('view-auth');
});

function applySettingsToUi(s, patch = null) {
  appSettings = s;
  if (!patch || patch.connected || patch.seekr) applyConnectedPreferences(s);
  if (patch?.services) applyServiceForm(s);
  const libSrcSelect = document.getElementById('library-source-select');
  if (libSrcSelect) {
    libSrcSelect.value = activeLibrarySource();
  }
  const progSrcSelect = document.getElementById('progress-source-select');
  if (progSrcSelect) {
    progSrcSelect.value = activeProgressSource();
  }
  if (typeof window.refreshMdblistAccountCard === 'function') {
    window.refreshMdblistAccountCard(false);
  }
  const theme = s.theme || 'white';
  document.body.dataset.theme = theme;
  if (theme === 'custom' && Array.isArray(s.custom_theme_colors)) {
    const c0 = s.custom_theme_colors[0] || '#FFFFFF';
    const c1 = s.custom_theme_colors[1] || '#B8BEC5';
    document.body.style.setProperty('--accent', c0);
    document.body.style.setProperty('--accent-variant', c1);
    document.body.style.setProperty('--focus-ring', c0);
    document.body.style.setProperty('--on-accent', contrastOnHex(c0));
  } else {
    document.body.style.removeProperty('--accent');
    document.body.style.removeProperty('--accent-variant');
    document.body.style.removeProperty('--focus-ring');
    document.body.style.removeProperty('--on-accent');
  }

  document.body.classList.toggle('amoled', !!s.amoled_black);
  document.getElementById('toggle-amoled').checked = !!s.amoled_black;

  const colors = s.custom_theme_colors || ['#FFFFFF', '#B8BEC5', '#6E7178'];
  document.getElementById('custom-color-0').value = colors[0] || '#FFFFFF';
  document.getElementById('custom-color-1').value = colors[1] || '#B8BEC5';
  document.getElementById('custom-color-2').value = colors[2] || '#6E7178';
  document.getElementById('custom-theme-editor').classList.toggle('hidden', theme !== 'custom');

  // Navigation bar vs sidebar
  const topNavMode = s.nav_style === 'top_bar';
  const shellEl = document.getElementById('desktop-shell');
  const topnavEl = document.getElementById('desktop-topnav');
  const sidebarEl = document.getElementById('desktop-sidebar');
  shellEl.classList.toggle('top-nav-mode', topNavMode);
  topnavEl.dataset.mode = s.nav_mode || 'adaptive';
  topnavEl.classList.toggle('hidden', !topNavMode);
  sidebarEl.classList.toggle('hidden', topNavMode);
  sidebarEl.classList.toggle('nav-expanded', s.nav_mode === 'expanded');
  sidebarEl.classList.toggle('nav-compact', s.nav_mode === 'compact');

  document.getElementById('set-nav-style').value = s.nav_style || 'sidebar';
  document.getElementById('set-nav-mode').value = s.nav_mode || 'adaptive';
  document.getElementById('set-show-hero').checked = s.show_hero !== false;
  document.getElementById('set-hover-preview').checked = s.hover_preview_enabled !== false;
  document.getElementById('set-blur-episodes').checked = !!s.blur_unwatched_episodes;
  document.getElementById('set-show-addon-logo').checked = s.show_addon_logo !== false;
  document.getElementById('set-show-size-badge').checked = s.show_file_size_badge !== false;
  document.getElementById('set-cw-style').value = s.cw_style || 'card';
  document.getElementById('set-detail-bg-mode').value = s.detail_bg_mode || 'cinematic';
  document.getElementById('set-streams-bg-mode').value = s.streams_bg_mode || 'cover';
  const dnsProv = document.getElementById('set-dns-provider');
  const dnsCustom = document.getElementById('set-dns-custom');
  const dnsCustomRow = document.getElementById('dns-custom-row');
  if (dnsProv) dnsProv.value = s.dns_provider || 'cloudflare';
  if (dnsCustom) dnsCustom.value = s.dns_custom || '';
  if (dnsCustomRow) dnsCustomRow.hidden = (dnsProv?.value || 'cloudflare') !== 'custom';

  if (!patch || patch.playback) applyPlaybackForm(s);
  if (!patch || patch.services) applyServiceForm(s);
  syncDiscord();
  renderThemeSwatches(theme);
}

function applyPlaybackForm(s) {
  document.getElementById('set-external-player').checked = !!s.external_player_enabled;
  document.getElementById('set-pause-overlay').checked = s.pause_overlay_enabled !== false;
  document.getElementById('set-next-ep').checked = s.next_episode_enabled !== false;
  document.getElementById('set-hold-speed').checked = s.hold_to_speed_enabled !== false;
  document.getElementById('set-hold-speed-val').value = String(s.hold_to_speed_value || 2);
  document.getElementById('set-pref-audio').value = s.preferred_audio_lang || 'por';
  document.getElementById('set-pref-sub').value = s.preferred_sub_lang || 'pt-br';
  document.getElementById('set-mpv-path').value = s.mpv_path || '';
  document.querySelectorAll('[data-playback]').forEach(el => {
    const value = s.playback[el.dataset.playback];
    if (el.type === 'checkbox') el.checked = value;
    else el.value = value;
  });

  document.getElementById('set-skip-enabled').checked = s.skip_intro_enabled !== false;
  document.getElementById('set-autoskip-intro').checked = !!s.auto_skip_intro;
  document.getElementById('set-autoskip-recap').checked = !!s.auto_skip_recap;
  document.getElementById('set-autoskip-outro').checked = !!s.auto_skip_outro;
  document.getElementById('set-aniskip').checked = s.aniskip_enabled !== false;
  document.getElementById('set-introdb-url').value = s.introdb_url || '';
  document.getElementById('set-introdb-key').value = s.introdb_api_key || '';

  const subSt = s.subtitle_style || DEFAULT_SUBTITLE_STYLE;
  document.getElementById('set-sub-size').value = subSt.fontSizeSp ?? 20;
  document.getElementById('set-sub-bold').checked = !!subSt.bold;
  document.getElementById('set-sub-outline').checked = subSt.outlineEnabled !== false;
  document.getElementById('set-sub-shadow').checked = subSt.shadowEnabled !== false;
}

function applyServiceForm(s) {
  document.getElementById('set-tmdb-key').value = s.tmdb_api_key || '';
  document.querySelectorAll('[data-service]').forEach(el => {
    const value = s.services[el.dataset.service];
    if (el.type === 'checkbox') el.checked = value;
    else el.value = value;
  });
  document.querySelectorAll('[data-provider]').forEach(el => {
    el.checked = s.services.providers.includes(el.dataset.provider);
  });
}

let pendingSettingsSave = Promise.resolve();
function persistSettingsPatch(patch) {
  const saved = pendingSettingsSave.then(async () => {
    const settings = await invoke('load_settings');
    Object.assign(settings, patch);
    await invoke('save_settings', { settings });
    applySettingsToUi(settings, patch);
    return settings;
  });
  pendingSettingsSave = saved.catch(() => {});
  return saved;
}

document.getElementById('toggle-amoled').addEventListener('change', (e) => {
  persistSettingsPatch({ amoled_black: e.target.checked });
});

function renderThemeSwatches(currentTheme) {
  const wrap = document.getElementById('theme-swatches');
  wrap.innerHTML = THEMES.map((t) => `
    <button type="button" class="nv-row nv-theme-row ${t.id === currentTheme ? 'active' : ''}" data-theme-id="${t.id}">
      <span class="nv-theme-dot" style="background:${t.color};"></span>
      <span class="nv-copy"><span class="nv-title">${t.label}</span></span>
      <span class="nv-theme-check">${t.id === currentTheme ? '✓' : ''}</span>
    </button>
  `).join('');
  wrap.querySelectorAll('[data-theme-id]').forEach((btn) => {
    btn.addEventListener('click', () => persistSettingsPatch({ theme: btn.dataset.themeId }));
  });
}

document.getElementById('btn-apply-custom-theme').addEventListener('click', () => {
  persistSettingsPatch({
    theme: 'custom',
    custom_theme_colors: [
      document.getElementById('custom-color-0').value,
      document.getElementById('custom-color-1').value,
      document.getElementById('custom-color-2').value,
    ],
  });
  showToast('Tema customizado aplicado.');
});

[
  ['set-nav-style', 'nav_style', 'value'],
  ['set-nav-mode', 'nav_mode', 'value'],
  ['set-show-hero', 'show_hero', 'checked'],
  ['set-hover-preview', 'hover_preview_enabled', 'checked'],
  ['set-blur-episodes', 'blur_unwatched_episodes', 'checked'],
  ['set-show-addon-logo', 'show_addon_logo', 'checked'],
  ['set-show-size-badge', 'show_file_size_badge', 'checked'],
  ['set-cw-style', 'cw_style', 'value'],
  ['set-detail-bg-mode', 'detail_bg_mode', 'value'],
  ['set-streams-bg-mode', 'streams_bg_mode', 'value'],
].forEach(([elId, key, prop]) => {
  document.getElementById(elId).addEventListener('change', async (e) => {
    await persistSettingsPatch({ [key]: e.target[prop] });
    if (key === 'show_hero') renderHero();
    if (key === 'cw_style' || key === 'blur_unwatched_episodes') renderContinueWatchingRow();
  });
});

document.getElementById('btn-save-settings').addEventListener('click', async () => {
  const s = await invoke('load_settings');
  await persistSettingsPatch({
    external_player_enabled: document.getElementById('set-external-player').checked,
    pause_overlay_enabled: document.getElementById('set-pause-overlay').checked,
    next_episode_enabled: document.getElementById('set-next-ep').checked,
    hold_to_speed_enabled: document.getElementById('set-hold-speed').checked,
    hold_to_speed_value: Number(document.getElementById('set-hold-speed-val').value) || 2.0,
    preferred_audio_lang: document.getElementById('set-pref-audio').value.trim(),
    preferred_sub_lang: document.getElementById('set-pref-sub').value.trim(),
    playback: Object.fromEntries(Array.from(document.querySelectorAll('[data-playback]'), el =>
      [el.dataset.playback, el.type === 'checkbox' ? el.checked : el.value])),
    mpv_path: document.getElementById('set-mpv-path').value.trim(),
    skip_intro_enabled: document.getElementById('set-skip-enabled').checked,
    auto_skip_intro: document.getElementById('set-autoskip-intro').checked,
    auto_skip_recap: document.getElementById('set-autoskip-recap').checked,
    auto_skip_outro: document.getElementById('set-autoskip-outro').checked,
    aniskip_enabled: document.getElementById('set-aniskip').checked,
    introdb_url: document.getElementById('set-introdb-url').value.trim(),
    introdb_api_key: document.getElementById('set-introdb-key').value.trim(),
    subtitle_style: {
      ...(s.subtitle_style || DEFAULT_SUBTITLE_STYLE),
      fontSizeSp: Number(document.getElementById('set-sub-size').value) || 20,
      bold: document.getElementById('set-sub-bold').checked,
      outlineEnabled: document.getElementById('set-sub-outline').checked,
      shadowEnabled: document.getElementById('set-sub-shadow').checked,
    },
  });
  document.getElementById('settings-status').textContent = 'Salvo!';
  showToast('Ajustes de reprodução salvos.');
});

function readServicesFromForm() {
  const services = { ...(appSettings?.services || {}) };
  document.querySelectorAll('[data-service]').forEach((el) => {
    services[el.dataset.service] = el.type === 'checkbox' ? el.checked : el.value.trim();
  });
  services.providers = Array.from(document.querySelectorAll('[data-provider]:checked'), (el) => el.dataset.provider);
  return services;
}

const DISCORD_SERVICE_KEYS = new Set([
  'discord_enabled',
  'discord_style',
  'discord_scope',
  'portrait',
  'episode_image',
  'episode_name',
]);

document.querySelectorAll('[data-service]').forEach((el) => {
  if (!DISCORD_SERVICE_KEYS.has(el.dataset.service)) return;
  el.addEventListener('change', async () => {
    // O mesmo ajuste aparece em mais de uma tela (ex.: Discord Rich Presence em Avançado e em Discord).
    // Espelha o valor antes de ler o formulário, senão readServicesFromForm pega o campo duplicado antigo.
    document.querySelectorAll(`[data-service="${el.dataset.service}"]`).forEach((peer) => {
      if (peer === el) return;
      if (peer.type === 'checkbox') peer.checked = el.checked;
      else peer.value = el.value;
    });
    const services = readServicesFromForm();
    try {
      await persistSettingsPatch({ services });
      sentPresence = 'null';
      await syncDiscord(true);
      if (el.dataset.service === 'discord_enabled') {
        showToast(services.discord_enabled ? 'Discord Rich Presence ativado.' : 'Discord Rich Presence desativado.');
      }
    } catch {
      showToast('Não foi possível salvar a opção do Discord.');
    }
  });
});

// Integrações (TMDB/MDBList) salvam sozinhas ao mudar, para não perder a opção ao fechar o app.
async function persistServiceForm() {
  const services = readServicesFromForm();
  const tmdbKey = document.getElementById('set-tmdb-key')?.value.trim() || '';
  const status = document.getElementById('services-status');
  try {
    await persistSettingsPatch({ tmdb_api_key: tmdbKey, services });
    if (status) status.textContent = 'Salvo!';
    heroRatings.clear();
    renderHero();
  } catch {
    showToast('Não foi possível salvar as integrações.');
  }
}

document.querySelectorAll('[data-service]').forEach((el) => {
  if (DISCORD_SERVICE_KEYS.has(el.dataset.service)) return;
  el.addEventListener('change', persistServiceForm);
});
document.querySelectorAll('[data-provider]').forEach((el) => {
  el.addEventListener('change', persistServiceForm);
});
document.getElementById('set-tmdb-key')?.addEventListener('change', persistServiceForm);


document.getElementById('set-dns-provider')?.addEventListener('change', async (e) => {
  const provider = e.target.value || 'cloudflare';
  const customRow = document.getElementById('dns-custom-row');
  if (customRow) customRow.hidden = provider !== 'custom';
  await persistSettingsPatch({
    dns_provider: provider,
    dns_custom: document.getElementById('set-dns-custom')?.value.trim() || '',
  });
  showToast(`DNS alterado para ${provider.toUpperCase()}.`);
});

document.getElementById('btn-save-dns')?.addEventListener('click', async () => {
  const status = document.getElementById('dns-status');
  const provider = document.getElementById('set-dns-provider')?.value || 'custom';
  const custom = document.getElementById('set-dns-custom')?.value.trim() || '';
  await persistSettingsPatch({ dns_provider: provider, dns_custom: custom });
  if (status) status.textContent = 'DNS salvo!';
});

document.getElementById('btn-clear-cw-cache')?.addEventListener('click', async () => {
  await invoke('clear_continue_watching_cache');
  const cs = document.getElementById('cache-status');
  if (cs) cs.textContent = 'Cache limpo.';
  refreshContinueWatching();
});

// --- ADDONS MANAGER ---
function toConfigureUrl(manifestUrl) {
  return String(manifestUrl || '').replace(/\/manifest\.json(?:\?.*)?$/i, '/configure');
}

function renderAddonSummary(manifest) {
  const resources = (manifest.resources || []).map((r) => r.name).join(', ');
  const types = (manifest.types || []).map((t) => t.charAt(0).toUpperCase() + t.slice(1)).join(' / ');
  const parts = [types, resources].filter(Boolean);
  if (manifest.behavior_hints?.p2p) parts.push('P2P');
  return parts.join(' • ');
}

function renderAddonsList(list) {
  initDiscoverFromAddons(list);
  document.getElementById('addons-stat-total').textContent = String(list.length);
  document.getElementById('addons-stat-active').textContent = String(list.filter((a) => a.enabled).length);
  document.getElementById('addons-stat-catalogs').textContent = String(
    list.filter((a) => a.enabled).reduce((acc, a) => acc + (a.manifest.catalogs?.length || 0), 0),
  );

  const root = document.getElementById('addons-list');
  const statusEl = document.getElementById('addon-status');

  root.innerHTML = list.map((a, index) => {
    const m = a.manifest;
    const hints = m.behavior_hints || {};
    const configurable = !!(hints.configurable || hints.configuration_required);
    const configUrl = toConfigureUrl(a.manifest_url);
    const logoUrl = m.logo_url || '';
    const iconHtml = logoUrl
      ? `<img src="${esc(logoUrl)}" alt="" decoding="async" draggable="false" onerror="this.style.display='none';this.parentElement.classList.add('is-fallback');this.nextElementSibling.style.display='flex';" /><span style="display:none;align-items:center;justify-content:center;">${ADDON_PUZZLE_SVG}</span>`
      : ADDON_PUZZLE_SVG;

    return `
      <div class="nv-row nv-addon-row ${a.enabled ? '' : 'is-disabled'}">
        <span class="addon-icon-badge ${logoUrl ? '' : 'is-fallback'}" title="${esc(a.manifest_url)}">${iconHtml}</span>
        <span class="nv-copy">
          <span class="nv-addon-title">
            <strong class="nv-addon-name">${esc(m.name)}</strong>
            <span class="addon-version">v${esc(m.version)}</span>
          </span>
          ${m.description ? `<span class="nv-desc">${esc(m.description)}</span>` : ''}
          <span class="addon-badges-row">
            <span class="addon-info-badge ${a.enabled ? 'ok' : ''}">${a.enabled ? 'Ativo' : 'Desativado'}</span>
            <span class="addon-info-badge">${(m.resources || []).length} recurso(s)</span>
            <span class="addon-info-badge">${(m.catalogs || []).length} catálogo(s)</span>
            ${configurable ? '<span class="addon-info-badge">Configurável</span>' : ''}
            ${hints.p2p ? '<span class="addon-info-badge">P2P</span>' : ''}
          </span>
          <span class="nv-addon-footer">
            <span class="addon-meta-summary">${esc(renderAddonSummary(m))}</span>
            <span class="addon-actions-row">
              <button class="addon-icon-btn" data-move-addon="${esc(a.manifest_url)}" data-delta="-1" title="Mover para cima" ${index === 0 ? 'disabled' : ''}>${SVG_UP}</button>
              <button class="addon-icon-btn" data-move-addon="${esc(a.manifest_url)}" data-delta="1" title="Mover para baixo" ${index === list.length - 1 ? 'disabled' : ''}>${SVG_DOWN}</button>
              <button class="addon-icon-btn primary" data-refresh-addon="${esc(a.manifest_url)}" title="Atualizar manifesto">${SVG_REFRESH}</button>
              ${configurable ? `<button class="addon-icon-btn" data-config-addon="${esc(configUrl)}" title="Configurar addon">${SVG_CONFIG}</button>` : ''}
              <button class="addon-icon-btn danger" data-del-addon="${esc(a.manifest_url)}" data-addon-name="${esc(m.name)}" title="Remover addon">${SVG_DELETE}</button>
            </span>
          </span>
        </span>
        <label class="nuvio-switch" title="${a.enabled ? 'Desativar addon' : 'Ativar addon'}">
          <input type="checkbox" data-toggle-addon="${esc(a.manifest_url)}" ${a.enabled ? 'checked' : ''} />
          <span class="nuvio-switch-track"></span>
        </label>
      </div>
    `;
  }).join('') || '<div class="nv-note">Nenhum addon instalado.</div>';

  root.querySelectorAll('[data-toggle-addon]').forEach((input) => {
    input.addEventListener('change', async () => {
      try {
        const updated = await invoke('toggle_addon', { manifestUrl: input.dataset.toggleAddon, enabled: input.checked });
        renderAddonsList(updated);
        refreshHomeProgressive(true);
      } catch (e) {
        statusEl.textContent = String(e);
      }
    });
  });

  root.querySelectorAll('[data-move-addon]').forEach((btn) => {
    btn.addEventListener('click', async () => {
      try {
        const updated = await invoke('move_addon', { manifestUrl: btn.dataset.moveAddon, delta: Number(btn.dataset.delta) });
        renderAddonsList(updated);
        refreshHomeProgressive(true);
      } catch (e) {
        statusEl.textContent = String(e);
      }
    });
  });

  root.querySelectorAll('[data-refresh-addon]').forEach((btn) => {
    btn.addEventListener('click', async () => {
      statusEl.textContent = 'Atualizando manifesto…';
      try {
        const updated = await invoke('refresh_addon', { manifestUrl: btn.dataset.refreshAddon });
        statusEl.textContent = 'Addon atualizado.';
        renderAddonsList(updated);
        refreshHomeProgressive(true);
      } catch (e) {
        statusEl.textContent = String(e);
      }
    });
  });

  root.querySelectorAll('[data-config-addon]').forEach((btn) => {
    btn.addEventListener('click', () => {
      invoke('open_external_url', { url: btn.dataset.configAddon }).catch((e) => { statusEl.textContent = String(e); });
    });
  });

  root.querySelectorAll('[data-del-addon]').forEach((btn) => {
    btn.addEventListener('click', async () => {
      const ok = await confirmDialog('Remover Addon', `Deseja remover "${btn.dataset.addonName}"?`);
      if (!ok) return;
      const updated = await invoke('remove_addon', { manifestUrl: btn.dataset.delAddon });
      renderAddonsList(updated);
      refreshHomeProgressive(true);
      showToast('Addon removido.');
    });
  });
}

async function refreshAddons() {
  const list = await invoke('list_addons');
  renderAddonsList(list);
}

document.getElementById('btn-add-addon').addEventListener('click', async () => {
  const url = document.getElementById('addon-url-input').value.trim();
  const status = document.getElementById('addon-status');
  if (!url) return;
  status.textContent = 'Instalando addon…';
  try {
    const installed = await invoke('add_addon', { manifestUrl: url });
    status.textContent = `Instalado: ${installed.manifest.name}`;
    document.getElementById('addon-url-input').value = '';
    await Promise.all([refreshAddons(), refreshHomeProgressive(true)]);
  } catch (e) {
    status.textContent = String(e);
  }
});

// --- PLUGINS REPOSITORY MANAGER ---
function renderPluginsUi(ps) {
  document.getElementById('plugins-stat-repos').textContent = String(ps.repositories?.length || 0);
  document.getElementById('plugins-stat-scrapers').textContent = String(ps.scrapers?.length || 0);
  document.getElementById('plugins-stat-enabled').textContent = String((ps.scrapers || []).filter((s) => s.enabled).length);
  document.getElementById('toggle-plugins-enabled').checked = ps.plugins_enabled !== false;

  const reposEl = document.getElementById('plugin-repos-list');
  reposEl.innerHTML = (ps.repositories || []).map((r) => `
    <div class="nv-row nv-addon-row">
      <span class="nv-copy">
        <span class="nv-addon-title">
          <strong class="nv-addon-name">${esc(r.name)}</strong>
          <span class="addon-version">v${esc(r.version)}</span>
        </span>
        <span class="nv-desc">${esc(r.manifest_url)}</span>
        ${r.description ? `<span class="nv-desc">${esc(r.description)}</span>` : ''}
        <span class="addon-meta-summary">${r.scraper_count} scraper(s)</span>
      </span>
      <span class="addon-actions-row">
        <button class="addon-icon-btn" data-refresh-repo="${esc(r.manifest_url)}" title="Atualizar repositório">${SVG_REFRESH}</button>
        <button class="addon-icon-btn danger" data-del-repo="${esc(r.manifest_url)}" data-repo-name="${esc(r.name)}" title="Remover repositório">${SVG_DELETE}</button>
      </span>
    </div>
  `).join('') || '<div class="nv-note">Nenhum repositório de plugins adicionado.</div>';

  reposEl.querySelectorAll('[data-refresh-repo]').forEach((b) => {
    b.onclick = async () => {
      document.getElementById('plugin-repo-status').textContent = 'Atualizando repositório…';
      try {
        renderPluginsUi(await invoke('refresh_plugin_repo', { manifestUrl: b.dataset.refreshRepo }));
        document.getElementById('plugin-repo-status').textContent = 'Repositório atualizado.';
      } catch (e) {
        document.getElementById('plugin-repo-status').textContent = String(e);
      }
    };
  });

  reposEl.querySelectorAll('[data-del-repo]').forEach((b) => {
    b.onclick = async () => {
      if (!(await confirmDialog('Remover Repositório', `Remover "${b.dataset.repoName}"?`))) return;
      renderPluginsUi(await invoke('remove_plugin_repo', { manifestUrl: b.dataset.delRepo }));
    };
  });

  const scrapersEl = document.getElementById('plugin-scrapers-list');
  scrapersEl.innerHTML = (ps.scrapers || []).map((s) => `
    <div class="nv-row nv-addon-row ${s.enabled ? '' : 'is-disabled'}">
      <span class="nv-copy">
        <span class="nv-addon-title">
          <strong class="nv-addon-name">${esc(s.name)}</strong>
          <span class="addon-version">v${esc(s.version)}</span>
        </span>
        <span class="nv-desc">${esc((s.supported_types || []).join(', ') || 'movie, tv')}</span>
        ${s.description ? `<span class="nv-desc">${esc(s.description)}</span>` : ''}
      </span>
      <span class="nv-addon-side">
        ${s.has_settings ? `<button class="addon-icon-btn" data-config-scraper="${esc(s.id)}" data-scraper-name="${esc(s.name)}" title="Configurações do provedor">${SVG_CONFIG}</button>` : ''}
        <button class="btn-ghost nv-mini-btn" data-test-scraper="${esc(s.id)}">Testar</button>
        <label class="nuvio-switch">
          <input type="checkbox" data-toggle-scraper="${esc(s.id)}" ${s.enabled ? 'checked' : ''} />
          <span class="nuvio-switch-track"></span>
        </label>
      </span>
    </div>
  `).join('') || '<div class="nv-note">Nenhum scraper instalado.</div>';

  scrapersEl.querySelectorAll('[data-toggle-scraper]').forEach((input) => {
    input.onchange = async () => {
      renderPluginsUi(await invoke('toggle_plugin_scraper', { scraperId: input.dataset.toggleScraper, enabled: input.checked }));
    };
  });
  scrapersEl.querySelectorAll('[data-test-scraper]').forEach((btn) => {
    btn.onclick = async () => {
      const st = document.getElementById('plugin-repo-status');
      st.textContent = `Testando scraper ${btn.dataset.testScraper}…`;
      try {
        const res = await invoke('test_plugin_scraper', { scraperId: btn.dataset.testScraper });
        st.textContent = `${res.length} stream(s) retornada(s) no teste.`;
      } catch (e) {
        st.textContent = String(e);
      }
    };
  });
  scrapersEl.querySelectorAll('[data-config-scraper]').forEach((btn) => {
    btn.onclick = () => openPluginSettingsModal(btn.dataset.configScraper, btn.dataset.scraperName || btn.dataset.configScraper);
  });
}

async function openPluginSettingsModal(scraperId, scraperName) {
  const modal = document.getElementById('plugin-settings-modal');
  const titleEl = document.getElementById('plugin-settings-title');
  const bodyEl = document.getElementById('plugin-settings-body');
  const cancelBtn = document.getElementById('btn-plugin-settings-cancel');
  const saveBtn = document.getElementById('btn-plugin-settings-save');
  if (!modal || !bodyEl) return;

  titleEl.textContent = `Configurações · ${scraperName}`;
  bodyEl.innerHTML = '<div class="nv-note">Carregando opções do provedor…</div>';
  saveBtn.disabled = true;
  modal.classList.remove('hidden');

  const closeModal = () => modal.classList.add('hidden');
  cancelBtn.onclick = closeModal;
  modal.onclick = (e) => {
    if (e.target === modal) closeModal();
  };

  try {
    const res = await invoke('get_plugin_settings_layout', { scraperId });
    const layout = Array.isArray(res?.layout) ? res.layout : [];
    const draft = { ...(res?.values && typeof res.values === 'object' ? res.values : {}) };

    if (!layout.length) {
      bodyEl.innerHTML = '<div class="nv-note">Este provedor não expôs opções configuráveis.</div>';
      return;
    }

    bodyEl.innerHTML = '';
    for (const field of layout) {
      if (!field || typeof field !== 'object') continue;
      const type = String(field.type || 'info');
      const key = String(field.key || '');
      const label = String(field.label || '');
      const desc = field.description ? String(field.description) : '';

      if (type === 'header') {
        const h = document.createElement('h4');
        h.className = 'plugin-setting-header';
        h.textContent = label;
        bodyEl.appendChild(h);
      } else if (type === 'info') {
        const p = document.createElement('p');
        p.className = 'plugin-setting-desc';
        p.textContent = label;
        bodyEl.appendChild(p);
      } else if (type === 'text' && key) {
        const wrap = document.createElement('div');
        wrap.className = 'plugin-setting-field';
        const lbl = document.createElement('label');
        lbl.textContent = label;
        const input = document.createElement('input');
        input.type = field.isPassword ? 'password' : 'text';
        input.placeholder = String(field.placeholder || '');
        input.value = String(draft[key] ?? field.defaultValue ?? '');
        input.addEventListener('input', () => {
          draft[key] = input.value;
        });
        wrap.append(lbl, input);
        if (desc) {
          const p = document.createElement('p');
          p.className = 'plugin-setting-desc';
          p.textContent = desc;
          wrap.appendChild(p);
        }
        bodyEl.appendChild(wrap);
      } else if (type === 'select' && key) {
        const options = Array.isArray(field.options) ? field.options : [];
        const current = String(draft[key] ?? field.defaultValue ?? '');
        if (!(key in draft)) draft[key] = current;

        const wrap = document.createElement('div');
        wrap.className = 'plugin-setting-field';
        const lbl = document.createElement('label');
        lbl.textContent = label;
        const select = document.createElement('select');
        for (const opt of options) {
          const optEl = document.createElement('option');
          optEl.value = String(opt?.value ?? '');
          optEl.textContent = String(opt?.label ?? opt?.value ?? '');
          optEl.selected = optEl.value === current;
          select.appendChild(optEl);
        }
        select.addEventListener('change', () => {
          draft[key] = select.value;
        });
        wrap.append(lbl, select);
        if (desc) {
          const p = document.createElement('p');
          p.className = 'plugin-setting-desc';
          p.textContent = desc;
          wrap.appendChild(p);
        }
        bodyEl.appendChild(wrap);
      } else if (type === 'toggle' && key) {
        const current = key in draft ? Boolean(draft[key]) : Boolean(field.defaultValue);
        if (!(key in draft)) draft[key] = current;

        const row = document.createElement('label');
        row.className = 'toggle-row plugin-setting-toggle';
        const copy = document.createElement('span');
        copy.className = 'plugin-setting-toggle-copy';
        const titleSpan = document.createElement('span');
        titleSpan.textContent = label;
        copy.appendChild(titleSpan);
        if (desc) {
          const descSpan = document.createElement('span');
          descSpan.className = 'plugin-setting-desc';
          descSpan.textContent = desc;
          copy.appendChild(descSpan);
        }

        const sw = document.createElement('span');
        sw.className = 'nuvio-switch';
        const cb = document.createElement('input');
        cb.type = 'checkbox';
        cb.checked = current;
        cb.addEventListener('change', () => {
          draft[key] = cb.checked;
        });
        const track = document.createElement('span');
        track.className = 'nuvio-switch-track';
        sw.append(cb, track);

        row.append(copy, sw);
        bodyEl.appendChild(row);
      }
    }

    saveBtn.disabled = false;
    saveBtn.onclick = async () => {
      saveBtn.disabled = true;
      try {
        await invoke('save_plugin_scraper_settings', { scraperId, settings: draft });
        showToast('Configurações do provedor salvas.');
        closeModal();
      } catch (e) {
        showToast(String(e));
      } finally {
        saveBtn.disabled = false;
      }
    };
  } catch (e) {
    bodyEl.innerHTML = `<div class="auth-error">${esc(String(e))}</div>`;
  }
}

async function refreshPluginsState() {
  try {
    renderPluginsUi(await invoke('list_plugins_state'));
  } catch {
    // Ignore
  }
}

document.getElementById('toggle-plugins-enabled').addEventListener('change', async (e) => {
  renderPluginsUi(await invoke('set_plugins_enabled', { enabled: e.target.checked, groupByRepo: null }));
});

document.getElementById('btn-add-plugin-repo').addEventListener('click', async () => {
  const url = document.getElementById('plugin-repo-url-input').value.trim();
  const status = document.getElementById('plugin-repo-status');
  if (!url) return;
  status.textContent = 'Adicionando repositório…';
  try {
    const updated = await invoke('add_plugin_repo', { manifestUrl: url });
    document.getElementById('plugin-repo-url-input').value = '';
    status.textContent = 'Repositório instalado!';
    renderPluginsUi(updated);
  } catch (e) {
    status.textContent = String(e);
  }
});

document.getElementById('btn-run-plugin')?.addEventListener('click', async () => {
  const status = document.getElementById('plugin-status');
  const out = document.getElementById('plugin-results');
  status.textContent = 'Executando script…';
  try {
    const streams = await invoke('run_js_plugin', {
      code: document.getElementById('plugin-code').value,
      tmdbId: document.getElementById('plugin-tmdb-id').value,
      mediaType: document.getElementById('plugin-media-type').value,
      season: null,
      episode: null,
    });
    status.textContent = `${streams.length} stream(s)`;
    out.innerHTML = streams.map((s) => `
      <div class="nv-row">
        <span class="nv-copy">
          <span class="nv-addon-name">${esc(s.name || 'Stream')}</span>
          <span class="nv-desc">${esc(s.title || s.url)}</span>
        </span>
        <button class="btn-accent" data-plugin-play="${esc(s.url)}" data-plugin-title="${esc(s.title || s.name || '')}">Assistir</button>
      </div>
    `).join('') || '<div class="nv-note">Nenhum stream retornado.</div>';
    out.querySelectorAll('[data-plugin-play]').forEach((b) => {
      b.addEventListener('click', () => startInAppPlayback({
        url: b.dataset.pluginPlay,
        title: b.dataset.pluginTitle || 'Plugin Stream',
        subtitle: '',
        contentId: 'plugin-test',
        videoId: 'plugin-test',
      }));
    });
  } catch (e) {
    status.textContent = String(e);
  }
});

async function refreshDiagnostics() {
  const box = document.getElementById('diag-box');
  if (!box) return;
  const d = await invoke('get_diagnostics');
  box.textContent =
    `OS: ${d.os} (${d.arch})\nConfig: ${d.config_dir}\nCache: ${d.cache_dir}\nSQLite: ${d.sqlite_ok} | Redb: ${d.redb_ok}\nmpv: ${d.mpv_available}\nEngine (${d.engine_library_name}): found=${d.engine_found} api=${d.engine_api_version ?? '-'} libtorrent=${d.engine_backend_version ?? '-'}`;
}

// --- NUVIO CUSTOM CHIP DROPDOWNS ---
const NV_DROPDOWN_CHEVRON = '<svg viewBox="0 0 24 24" class="nv-chip-trigger-chevron" aria-hidden="true"><path fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" d="M6 9l6 6 6-6"/></svg>';
const NV_DROPDOWN_CHECK = '<svg viewBox="0 0 24 24" class="nv-dropdown-check" aria-hidden="true"><path fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" d="M20 6L9 17l-5-5"/></svg>';
let openNuvioDropdownState = null;

function closeNuvioDropdown() {
  if (!openNuvioDropdownState) return;
  openNuvioDropdownState.wrap.classList.remove('is-open');
  openNuvioDropdownState.trigger.setAttribute('aria-expanded', 'false');
  openNuvioDropdownState.menu.classList.add('hidden');
  openNuvioDropdownState = null;
}

function positionNuvioDropdownMenu(trigger, menu) {
  const rect = trigger.getBoundingClientRect();
  const margin = 12;
  menu.style.left = `${margin}px`;
  menu.style.top = `${Math.round(rect.bottom + 8)}px`;
  const menuRect = menu.getBoundingClientRect();
  const maxLeft = Math.max(margin, window.innerWidth - menuRect.width - margin);
  const left = Math.min(Math.max(margin, rect.left), maxLeft);
  let top = rect.bottom + 8;
  if (top + menuRect.height > window.innerHeight - margin && rect.top - menuRect.height - 8 >= margin) {
    top = rect.top - menuRect.height - 8;
  }
  menu.style.left = `${Math.round(left)}px`;
  menu.style.top = `${Math.round(top)}px`;
}

function upgradeNuvioChipSelect(selectEl) {
  if (!selectEl || selectEl.dataset.nvDropdownUpgraded) return;
  selectEl.dataset.nvDropdownUpgraded = '1';
  selectEl.classList.add('is-upgraded');

  const wrap = document.createElement('div');
  wrap.className = 'nv-chip-dropdown';
  const trigger = document.createElement('button');
  trigger.type = 'button';
  trigger.className = 'nv-chip-trigger';
  trigger.setAttribute('aria-haspopup', 'listbox');
  trigger.setAttribute('aria-expanded', 'false');
  const labelSpan = document.createElement('span');
  labelSpan.className = 'nv-chip-trigger-label';
  trigger.append(labelSpan);
  trigger.insertAdjacentHTML('beforeend', NV_DROPDOWN_CHEVRON);

  const menu = document.createElement('div');
  menu.className = 'nv-dropdown-menu hidden';
  menu.setAttribute('role', 'listbox');

  selectEl.insertAdjacentElement('afterend', wrap);
  wrap.append(trigger);
  document.body.append(menu);

  const sync = () => {
    const opts = Array.from(selectEl.options || []);
    const selected = opts.find((o) => o.value === selectEl.value) || opts[0];
    labelSpan.textContent = selected ? selected.textContent : '';
    trigger.disabled = !!selectEl.disabled || opts.length === 0;
    wrap.classList.toggle('hidden', selectEl.classList.contains('hidden'));
    menu.innerHTML = opts.map((opt) => {
      const isSel = opt.value === selectEl.value;
      return `
        <button type="button" class="nv-dropdown-option${isSel ? ' is-selected' : ''}" role="option" aria-selected="${isSel ? 'true' : 'false'}" data-nv-opt="${esc(opt.value)}">
          <span class="nv-dropdown-option-label">${esc(opt.textContent)}</span>
          ${isSel ? NV_DROPDOWN_CHECK : ''}
        </button>`;
    }).join('');
    menu.querySelectorAll('[data-nv-opt]').forEach((btn) => {
      btn.addEventListener('click', (ev) => {
        ev.stopPropagation();
        const nextVal = btn.dataset.nvOpt;
        const changed = selectEl.value !== nextVal;
        selectEl.value = nextVal;
        closeNuvioDropdown();
        if (changed) selectEl.dispatchEvent(new Event('change', { bubbles: true }));
      });
    });
  };

  trigger.addEventListener('click', (ev) => {
    ev.stopPropagation();
    if (trigger.disabled) return;
    if (openNuvioDropdownState?.selectEl === selectEl) {
      closeNuvioDropdown();
      return;
    }
    closeNuvioDropdown();
    sync();
    menu.classList.remove('hidden');
    wrap.classList.add('is-open');
    trigger.setAttribute('aria-expanded', 'true');
    positionNuvioDropdownMenu(trigger, menu);
    openNuvioDropdownState = { selectEl, wrap, trigger, menu };
  });

  selectEl.addEventListener('change', sync);
  const observer = new MutationObserver(sync);
  observer.observe(selectEl, { childList: true, attributes: true, attributeFilter: ['disabled', 'class'] });
  sync();
}

function initNuvioChipDropdowns() {
  document.querySelectorAll('select.nuvio-chip-select').forEach(upgradeNuvioChipSelect);
  document.addEventListener('click', (ev) => {
    if (openNuvioDropdownState && !openNuvioDropdownState.wrap.contains(ev.target) && !openNuvioDropdownState.menu.contains(ev.target)) {
      closeNuvioDropdown();
    }
  });
  document.addEventListener('keydown', (ev) => {
    if (ev.key === 'Escape') closeNuvioDropdown();
  });
  window.addEventListener('resize', closeNuvioDropdown);
  document.getElementById('main-viewport')?.addEventListener('scroll', closeNuvioDropdown, { passive: true });
}

// --- APP UPDATER (GITHUB RELEASES) ---
let activeUpdateInfo = null;
let updateCheckInFlight = false;
let updateProgressUnlisten = null;
let updateBootChecked = false;

function formatBytesMb(bytes) {
  if (!bytes || bytes <= 0) return '0 MB';
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function showAppUpdateModal(info) {
  activeUpdateInfo = info;
  const modal = document.getElementById('app-update-modal');
  const verEl = document.getElementById('app-update-version');
  const notesEl = document.getElementById('app-update-notes');
  const progressWrap = document.getElementById('app-update-progress-wrap');
  const progressFill = document.getElementById('app-update-progress-fill');
  const progressLabel = document.getElementById('app-update-progress-label');
  const errEl = document.getElementById('app-update-error');
  const installBtn = document.getElementById('btn-update-install');
  const laterBtn = document.getElementById('btn-update-later');
  const releaseBtn = document.getElementById('btn-update-release');
  if (!modal) return;

  verEl.textContent = `v${info.currentVersion} → v${info.latestVersion}`;
  if (info.releaseNotes) {
    notesEl.textContent = info.releaseNotes;
    notesEl.classList.remove('hidden');
  } else {
    notesEl.textContent = '';
    notesEl.classList.add('hidden');
  }

  progressWrap.classList.add('hidden');
  progressFill.style.width = '0%';
  progressLabel.textContent = 'Preparando download…';
  errEl.textContent = '';
  errEl.classList.add('hidden');

  const canInstall = Boolean(info.canSelfInstall && info.assetUrl);
  installBtn.classList.toggle('hidden', !canInstall);
  installBtn.disabled = false;
  installBtn.textContent = 'Atualizar agora';
  laterBtn.disabled = false;
  releaseBtn.disabled = false;

  modal.classList.remove('hidden');
}

function hideAppUpdateModal(dismissCurrent = false) {
  if (dismissCurrent && activeUpdateInfo?.latestVersion) {
    try { sessionStorage.setItem('nuvio_dismissed_update', activeUpdateInfo.latestVersion); } catch {}
  }
  document.getElementById('app-update-modal')?.classList.add('hidden');
}

async function checkForAppUpdates({ silent = true } = {}) {
  if (updateCheckInFlight) return;
  updateCheckInFlight = true;
  const statusEl = document.getElementById('app-update-status');
  const checkBtn = document.getElementById('btn-check-updates');
  if (!silent && statusEl) statusEl.textContent = 'Verificando atualizações…';
  if (!silent && checkBtn) checkBtn.disabled = true;

  try {
    const info = await invoke('check_app_update');
    if (statusEl) {
      statusEl.textContent = info.updateAvailable
        ? `Nova versão disponível: v${info.latestVersion} (atual: v${info.currentVersion}).`
        : `Você já está na versão mais recente (v${info.currentVersion}).`;
    }
    if (info.updateAvailable) {
      let dismissed = null;
      try { dismissed = sessionStorage.getItem('nuvio_dismissed_update'); } catch {}
      if (!silent || dismissed !== info.latestVersion) {
        showAppUpdateModal(info);
      }
    } else if (!silent) {
      showToast(`Você já está na versão mais recente (v${info.currentVersion}).`);
    }
  } catch (e) {
    if (statusEl && !silent) statusEl.textContent = String(e);
    if (!silent) showToast(String(e));
  } finally {
    updateCheckInFlight = false;
    if (checkBtn) checkBtn.disabled = false;
  }
}

document.getElementById('btn-check-updates')?.addEventListener('click', () => {
  checkForAppUpdates({ silent: false });
});

document.getElementById('btn-update-later')?.addEventListener('click', () => {
  hideAppUpdateModal(true);
});

document.getElementById('btn-update-release')?.addEventListener('click', () => {
  if (activeUpdateInfo?.releaseUrl) {
    invoke('open_external_url', { url: activeUpdateInfo.releaseUrl }).catch((e) => showToast(String(e)));
  }
});

document.getElementById('btn-update-install')?.addEventListener('click', async () => {
  if (!activeUpdateInfo?.assetUrl) return;
  const progressWrap = document.getElementById('app-update-progress-wrap');
  const progressFill = document.getElementById('app-update-progress-fill');
  const progressLabel = document.getElementById('app-update-progress-label');
  const errEl = document.getElementById('app-update-error');
  const installBtn = document.getElementById('btn-update-install');
  const laterBtn = document.getElementById('btn-update-later');

  errEl.classList.add('hidden');
  progressWrap.classList.remove('hidden');
  installBtn.disabled = true;
  laterBtn.disabled = true;
  installBtn.textContent = 'Baixando…';

  if (updateProgressUnlisten) {
    updateProgressUnlisten();
    updateProgressUnlisten = null;
  }
  updateProgressUnlisten = await listen('app:update_progress', (ev) => {
    const p = ev.payload || {};
    const pct = Math.max(0, Math.min(100, Math.round(p.percent || 0)));
    progressFill.style.width = `${pct}%`;
    progressLabel.textContent = p.totalBytes > 0
      ? `Baixando atualização: ${pct}% (${formatBytesMb(p.downloadedBytes)} / ${formatBytesMb(p.totalBytes)})`
      : `Baixando atualização: ${formatBytesMb(p.downloadedBytes)}`;
  });

  try {
    await invoke('install_app_update', {
      assetUrl: activeUpdateInfo.assetUrl,
      assetSize: activeUpdateInfo.assetSize ?? null,
    });
  } catch (e) {
    errEl.textContent = String(e);
    errEl.classList.remove('hidden');
    installBtn.disabled = false;
    laterBtn.disabled = false;
    installBtn.textContent = 'Tentar novamente';
  } finally {
    if (updateProgressUnlisten) {
      updateProgressUnlisten();
      updateProgressUnlisten = null;
    }
  }
});

// --- BOOT ---
(async function boot() {
  initNuvioChipDropdowns();
  let status = null;
  try {
    const [settings, authRes] = await Promise.all([
      invoke('load_settings'),
      invoke('get_auth_status'),
    ]);
    applySettingsToUi(settings);
    status = authRes;
    authState = status;
    if (status.server?.backend_url) {
      document.getElementById('auth-server-pill').textContent = status.server.backend_url.replace(/^https?:\/\//, '');
    }
  } catch (e) {
    console.error('boot failed', e);
  }

  if (status && (status.signed_in || status.is_guest)) {
    await enterMainApp(status);
  } else {
    showView('view-auth');
  }

  if (!updateBootChecked) {
    updateBootChecked = true;
    setTimeout(() => checkForAppUpdates({ silent: true }), 2500);
  }
})();

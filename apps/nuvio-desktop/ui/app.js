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

    if (
      document.getElementById('tab-search')?.classList.contains('active')
      && !searchInputEl.value.trim()
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

function showView(id) {
  document.querySelectorAll('.view').forEach((v) => v.classList.remove('active'));
  document.getElementById(id).classList.add('active');
}

function activeTabName() {
  return document.querySelector('.tab-panel.active')?.id?.replace('tab-', '') || 'home';
}

function setTab(tab) {
  const cur = activeTabName();
  if (cur && cur !== 'details' && cur !== 'streams' && cur !== 'catalog') {
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
  mainViewportEl.scrollTo({ top: 0 });
  if (tab === 'library') refreshLibrary();
  if (tab === 'search') renderRecentSearches();
}

function handleBackNavigation() {
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

async function switchToProfile(idx, fromPicker) {
  if (switchingProfile) return;
  switchingProfile = true;
  document.getElementById('profile-switcher-stack')?.classList.add('hidden');
  authState.active_profile_index = idx;
  if (fromPicker) {
    showView('view-app');
  }
  updateProfileUi();
  updateAccountSettingsUi();
  document.getElementById('home-catalogs-container').innerHTML = '';
  document.getElementById('home-cw-container').classList.add('hidden');
  try {
    await invoke('select_profile', { profileIndex: idx });
    await Promise.all([
      refreshWatchedKeys(),
      refreshLibraryState(),
      refreshAddons(),
      refreshPluginsState(),
      refreshContinueWatching(),
      refreshHomeProgressive(),
      fromPicker ? refreshDiagnostics() : Promise.resolve(),
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
        ${p.avatar_url ? `<img src="${esc(p.avatar_url)}" alt="" />` : esc((p.name || 'P').charAt(0).toUpperCase())}
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
  await invoke('sign_out');
  showView('view-auth');
});

async function enterMainApp(status) {
  authState = status;
  showView('view-app');
  updateProfileUi();
  updateAccountSettingsUi();
  await Promise.all([
    refreshWatchedKeys(),
    refreshLibraryState(),
    refreshAddons(),
    refreshPluginsState(),
    refreshContinueWatching(),
    refreshHomeProgressive(),
    refreshDiagnostics(),
  ]);
}

function updateProfileUi() {
  const profiles = authState?.profiles || [];
  const active = profiles.find((p) => p.profile_index === authState?.active_profile_index) || profiles[0];
  const name = active?.name || 'Local';
  const color = active?.avatar_color_hex || 'var(--accent)';
  const avatarInner = active?.avatar_url
    ? `<img src="${esc(active.avatar_url)}" alt="" />`
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
  stack.innerHTML = profiles.map((p) => `
    <button class="switcher-row" data-switch-idx="${p.profile_index}">
      <div class="mini-avatar" style="background:${esc(p.avatar_color_hex || '#1E88E5')}">
        ${esc((p.name || 'P').charAt(0).toUpperCase())}
      </div>
      <span>${esc(p.name)}</span>
    </button>
  `).join('');
  stack.querySelectorAll('[data-switch-idx]').forEach((btn) => {
    btn.addEventListener('click', () => {
      switchToProfile(Number(btn.dataset.switchIdx), false);
    });
  });
}

document.getElementById('btn-sidebar-profile').addEventListener('click', () => {
  document.getElementById('profile-switcher-stack').classList.toggle('hidden');
});

document.getElementById('btn-topnav-profile').addEventListener('click', () => {
  if (authState?.profiles?.length) {
    renderProfilePicker(authState.profiles);
    showView('view-profiles');
  }
});

document.querySelectorAll('.sidebar-item, .topnav-item').forEach((btn) => {
  btn.addEventListener('click', () => setTab(btn.dataset.tab));
});

// --- WATCHED & LIBRARY STATE ---
async function refreshWatchedKeys() {
  try {
    const keys = await invoke('list_watched_keys');
    watchedSet = new Set(keys || []);
  } catch {
    // Non-fatal
  }
}

async function refreshLibraryState() {
  try {
    const items = await invoke('pull_account_library');
    libraryRawItems = Array.isArray(items) ? items : [];
    librarySet = new Set(libraryRawItems.map((it) => libKey(it.content_id || it.id, it.content_type || it.type)));
  } catch {
    libraryRawItems = [];
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
  const saved = await invoke('toggle_library_item', { item: payload });
  await refreshLibraryState();
  showToast(saved ? 'Adicionado à Biblioteca' : 'Removido da Biblioteca');
  return saved;
}

async function toggleItemWatched(contentId, contentType, title, season = null, episode = null) {
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

function showHoverPreviewForCard(cardEl, item) {
  if (!appSettings?.hover_preview_enabled) return;
  const popup = document.getElementById('hover-preview-popup');
  const rect = cardEl.getBoundingClientRect();
  const pw = 400;
  let left = rect.left + rect.width / 2 - pw / 2;
  left = Math.max(18, Math.min(window.innerWidth - pw - 18, left));
  let top = rect.top - 18;
  if (top + 320 > window.innerHeight - 16) {
    top = Math.max(16, window.innerHeight - 336);
  }
  const inLib = librarySet.has(libKey(item.id, item.type));
  const artwork = item.banner || item.poster || '';

  popup.style.left = `${left}px`;
  popup.style.top = `${top}px`;
  popup.innerHTML = `
    <div class="hover-preview-media">
      ${artwork ? `<img src="${esc(artwork)}" alt="" />` : ''}
      <div class="hover-preview-media-scrim"></div>
    </div>
    <div class="hover-preview-body">
      ${item.logo ? `<img class="hover-preview-logo" src="${esc(item.logo)}" alt="${esc(item.name)}" />` : `<strong style="font-size:15px;">${esc(item.name)}</strong>`}
      <div class="hero-meta-row" style="margin-bottom:2px;font-size:12px;">
        ${item.imdb_rating ? `<span class="imdb-chip"><img src="assets/rating_imdb.png" alt="IMDb" /> ${esc(item.imdb_rating)}</span>` : ''}
        ${item.release_info ? `<span>${esc(item.release_info)}</span>` : ''}
      </div>
      ${item.description ? `<div class="hover-preview-desc">${esc(item.description)}</div>` : ''}
      <div class="hover-preview-actions">
        <button class="btn-accent" id="hp-open-btn" style="padding:7px 14px;font-size:12px;">Ver Detalhes</button>
        <button class="btn-ghost" id="hp-lib-btn" style="padding:7px 12px;font-size:12px;">${inLib ? '✓ Na Biblioteca' : '+ Biblioteca'}</button>
      </div>
    </div>
  `;
  popup.classList.remove('hidden');

  document.getElementById('hp-open-btn').onclick = () => {
    hideHoverPreview();
    openDetailsPage(item);
  };
  document.getElementById('hp-lib-btn').onclick = async (e) => {
    const saved = await toggleItemLibrary(item);
    e.currentTarget.textContent = saved ? '✓ Na Biblioteca' : '+ Biblioteca';
  };
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

  cwUnlistenItem = await listen('home:continue-watching-item', (event) => {
    const { order, card } = event.payload;
    cwItemsMap.set(order, card);
    renderContinueWatchingRow();
  });

  try {
    await invoke('start_continue_watching');
  } catch {
    // Non-fatal
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
        const imgUrl = style === 'poster' ? (c.poster || c.artwork || '') : (c.artwork || c.poster || '');
        const blurred = appSettings?.blur_unwatched_episodes && c.season != null ? 'blurred-img' : '';
        return `
          <button class="cw-card ${styleClass}" data-cw-idx="${idx}">
            ${imgUrl ? `<img class="${blurred}" src="${esc(imgUrl)}" alt="${esc(c.title)}" loading="lazy" decoding="async" draggable="false" />` : ''}
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
async function refreshHomeProgressive() {
  if (homeUnlistenSection) { homeUnlistenSection(); homeUnlistenSection = null; }
  if (homeUnlistenDone) { homeUnlistenDone(); homeUnlistenDone = null; }

  const heroContainer = document.getElementById('home-hero-container');
  const shelvesContainer = document.getElementById('home-catalogs-container');
  homeSectionsMap.clear();
  heroItems = [];
  heroIdx = 0;
  shelvesContainer.innerHTML = '<div class="muted" id="home-loading-note">Carregando catálogos…</div>';

  homeUnlistenSection = await listen('home:section', (event) => {
    const sec = event.payload;
    document.getElementById('home-loading-note')?.remove();
    homeSectionsMap.set(sec.order, sec);

    if (heroItems.length === 0) {
      const candidates = sec.items.filter((i) => i.banner || i.poster).slice(0, 8);
      if (candidates.length) {
        heroItems = candidates;
        renderHero();
      }
    }

    appendShelfOrdered(shelvesContainer, sec);
  });

  homeUnlistenDone = await listen('home:done', () => {
    document.getElementById('home-loading-note')?.remove();
    if (homeSectionsMap.size === 0) {
      renderEmptyHome(heroContainer, shelvesContainer);
    }
  });

  const ack = await invoke('start_home_catalogs');
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
      ${sec.items.map((item) => renderPosterCard(item)).join('')}
    </div>
  `;

  const existing = Array.from(shelvesContainer.children);
  const nextSibling = existing.find((el) => Number(el.dataset.order) > sec.order);
  if (nextSibling) {
    shelvesContainer.insertBefore(wrapper, nextSibling);
  } else {
    shelvesContainer.appendChild(wrapper);
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

    const stopFling = () => {
      if (flingRaf) {
        cancelAnimationFrame(flingRaf);
        flingRaf = 0;
      }
    };

    const startFling = () => {
      stopFling();
      if (Math.abs(velocity) < 0.4) return;
      let prev = performance.now();
      const step = (now) => {
        const dt = Math.min(32, Math.max(4, now - prev));
        prev = now;
        row.scrollLeft += velocity * (dt / 16.67);
        velocity *= Math.pow(0.92, dt / 16.67);
        if (Math.abs(velocity) < 0.35 || row.scrollLeft <= 0 || row.scrollLeft >= row.scrollWidth - row.clientWidth) {
          flingRaf = 0;
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
      row.scrollLeft -= stepDx;
      velocity = (-stepDx / dt) * 16.67 * 0.75 + velocity * 0.25;
      lastX = e.clientX;
      lastTs = now;
    });

    const endDrag = (e) => {
      if (!tracking && locked !== 'x') return;
      if (e.pointerId !== pointerId) return;
      const wasHoriz = locked === 'x';
      tracking = false;
      locked = null;
      row.classList.remove('dragging');
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
        if (e.shiftKey && Math.abs(e.deltaY) > 0) {
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
  activeCatalog = {
    manifestUrl: sec.manifest_url,
    mediaType: sec.media_type,
    catalogId: sec.catalog_id,
    title: sec.title,
    addonName: sec.addon_name,
    searchQuery: sec.searchQuery || null,
    items: [...(sec.items || [])],
  };
  document.getElementById('catalog-page-title').textContent = sec.title;
  document.getElementById('catalog-page-sub').textContent = sec.addon_name;
  setTab('catalog');
  renderFullCatalogGrid();
  document.getElementById('btn-catalog-load-more').classList.toggle('hidden', activeCatalog.items.length < 10);
}

function renderFullCatalogGrid() {
  const grid = document.getElementById('catalog-full-grid');
  grid.innerHTML = activeCatalog.items.map((it) => renderPosterCard(it)).join('');
  wirePosterInteractions(grid);
}

document.getElementById('btn-catalog-back').addEventListener('click', () => setTab(previousTab || 'home'));

document.getElementById('btn-catalog-load-more').addEventListener('click', async () => {
  if (!activeCatalog) return;
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
      genre: null,
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
  return `
    <button class="poster-card" data-meta-id="${esc(item.id)}" data-meta-type="${esc(item.type)}" data-meta-name="${esc(item.name)}" data-meta-poster="${esc(item.poster || '')}" data-meta-banner="${esc(item.banner || '')}" data-meta-logo="${esc(item.logo || '')}" data-meta-desc="${esc(item.description || '')}" data-meta-release="${esc(item.release_info || '')}" data-meta-imdb="${esc(item.imdb_rating || '')}">
      <div class="poster-img-wrap">
        ${item.poster ? `<img src="${esc(item.poster)}" alt="${esc(item.name)}" loading="lazy" decoding="async" draggable="false" />` : ''}
        ${watched ? '<span class="watched-badge" title="Assistido">&#10003;</span>' : ''}
      </div>
      <div class="poster-name">${esc(item.name)}</div>
      <div class="poster-sub">${esc(item.release_info || item.type || '')}</div>
    </button>
  `;
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
    const bg = item.banner || item.poster || '';
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
    const card = e.target.closest('.poster-card');
    if (!card) return;
    const shelfRow = card.closest('.shelf-row');
    if (shelfRow?.dataset.suppressClick) return;
    hideHoverPreview();
    openDetailsPage(cardToItem(card));
  });

  mainViewportEl.addEventListener('contextmenu', (e) => {
    const card = e.target.closest('.poster-card');
    if (!card) return;
    e.preventDefault();
    openPosterContextMenu(cardToItem(card));
  });

  mainViewportEl.addEventListener('pointerover', (e) => {
    if (!appSettings?.hover_preview_enabled) return;
    const card = e.target.closest('.poster-card');
    if (!card || card === hoverActiveCard) return;
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
    setTimeout(() => {
      if (!hoverPopupEl.matches(':hover') && !hoverActiveCard) hideHoverPreview();
    }, 120);
  });
}

// --- FULL-SCREEN META DETAILS SCREEN (MetaDetailsScreen.kt) ---
async function openDetailsPage(preview) {
  activeMetaContext = { ...preview, full: null };
  setTab('details');
  renderDetailsScreen(preview, null);

  try {
    const meta = await invoke('fetch_meta_details', { mediaType: preview.type, id: preview.id });
    if (activeMetaContext?.id === preview.id) {
      activeMetaContext.full = meta;
      renderDetailsScreen(preview, meta);
    }
  } catch {
    // Keep preview details when addon lacks meta resource
  }
}

function findCwResumeForContent(contentId) {
  return cwCardsCache.find((c) => c.content_id === contentId) || null;
}

function renderDetailsScreen(preview, meta) {
  const m = meta || {};
  const title = m.name || preview.name || 'Sem título';
  const bg = m.background || preview.banner || m.poster || preview.poster || '';
  const logo = m.logo || preview.logo || '';
  const desc = m.description || preview.description || '';
  const release = m.releaseInfo || preview.release_info || '';
  const imdb = m.imdbRating || preview.imdb_rating || '';
  const runtime = m.runtime || '';
  const resolvedTmdbId = m.moviedb_id || m.tmdb_id || preview.tmdbId || null;
  const ageRating = m.ageRating || m.certification || '';
  const genres = Array.isArray(m.genres) ? m.genres : [];
  const videos = Array.isArray(m.videos) ? m.videos : [];
  const cast = Array.isArray(m.castMembers || m.credits_cast || m.cast)
    ? (m.castMembers || m.credits_cast || m.cast)
    : [];

  const inLib = librarySet.has(libKey(preview.id, preview.type));
  const titleWatched = watchedSet.has(preview.id);
  const cwResume = findCwResumeForContent(preview.id);

  let defaultPlayId = preview.id;
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
      ${bg ? `<img class="detail-hero-bg" src="${esc(bg)}" alt="" />` : ''}
      <div class="detail-scrim-side"></div>
      <div class="detail-scrim-bottom"></div>
      <div class="detail-top-bar">
        <button class="circle-back-btn" id="btn-detail-back" title="Voltar">
          <svg viewBox="0 0 24 24" width="20" height="20"><path fill="currentColor" d="M15.41 7.41L14 6l-6 6l6 6l1.41-1.41L10.83 12z"/></svg>
        </button>
      </div>
      <div class="detail-hero-content">
        ${logo ? `<img class="detail-logo" src="${esc(logo)}" alt="${esc(title)}" />` : `<h1 class="detail-title">${esc(title)}</h1>`}
        <div class="detail-meta-chips">
          ${imdb ? `<span class="imdb-chip"><img src="assets/rating_imdb.png" alt="IMDb" /> ${esc(imdb)}</span>` : ''}
          ${release ? `<span>${esc(release)}</span>` : ''}
          ${runtime ? `<span>• ${esc(runtime)}</span>` : ''}
          ${ageRating ? `<span class="meta-badge">${esc(ageRating)}</span>` : ''}
          ${m.status ? `<span class="meta-badge">${esc(m.status)}</span>` : ''}
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
          ${(m.trailers?.length || m.trailerStreams?.length) ? `
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
    const tr = m.trailers?.[0] || m.trailerStreams?.[0];
    const url = tr?.url || (tr?.source ? `https://www.youtube.com/watch?v=${tr.source}` : '');
    if (url) invoke('open_external_url', { url }).catch((e) => showToast(String(e)));
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

  if (videos.length > 0) {
    const seasons = [...new Set(videos.map((v) => v.season ?? 1))].sort((a, b) => a - b);
    const defaultSeason = seasons.find((s) => s > 0) ?? seasons[0];
    bodyHtml += `
      <div>
        <div class="row-gap" style="justify-content:space-between;margin-bottom:10px;">
          <div class="season-pills" id="detail-season-pills" style="margin-bottom:0;">
            ${seasons.map((s) => `<button class="season-pill ${s === defaultSeason ? 'active' : ''}" data-season="${s}">${s === 0 ? 'Especiais' : `Temporada ${s}`}</button>`).join('')}
          </div>
          <button class="btn-ghost" id="btn-season-mark-watched" style="font-size:12px;">&#10003; Marcar temporada vista</button>
        </div>
        <div class="episodes-grid shelf-row" id="detail-episodes-grid"></div>
      </div>
    `;
  }

  if (cast.length > 0) {
    bodyHtml += `
      <div>
        <h3 style="margin:0 0 12px;">Elenco</h3>
        <div class="cast-row shelf-row">
          ${cast.slice(0, 18).map((c) => {
            const actorName = typeof c === 'string' ? c : (c.name || '');
            const character = typeof c === 'string' ? '' : (c.character || c.role || '');
            const photo = typeof c === 'string' ? '' : (c.photo || c.profile || c.image || '');
            return `
              <div class="cast-portrait-card">
                <div class="cast-avatar-circle">
                  ${photo ? `<img src="${esc(photo)}" alt="${esc(actorName)}" loading="lazy" draggable="false" />` : esc(actorName.charAt(0).toUpperCase())}
                </div>
                <div class="cast-actor-name">${esc(actorName)}</div>
                ${character ? `<div class="cast-char-name">${esc(character)}</div>` : ''}
              </div>
            `;
          }).join('')}
        </div>
      </div>
    `;
  }

  const extras = [
    m.director && ['Direção', Array.isArray(m.director) ? m.director.join(', ') : m.director],
    m.writer && ['Roteiro', Array.isArray(m.writer) ? m.writer.join(', ') : m.writer],
    m.country && ['País', m.country],
    m.language && ['Idioma', m.language],
    m.imdb_id && ['IMDb ID', m.imdb_id],
  ].filter(Boolean);

  if (extras.length > 0) {
    bodyHtml += `
      <div>
        <h3 style="margin:0 0 12px;">Mais Informações</h3>
        <div class="settings-card">
          ${extras.map(([k, v]) => `<div class="kv-line"><span>${esc(k)}</span><strong>${esc(v)}</strong></div>`).join('')}
        </div>
      </div>
    `;
  }

  bodyWrap.innerHTML = bodyHtml;

  if (videos.length > 0) {
    const seasons = [...new Set(videos.map((v) => v.season ?? 1))].sort((a, b) => a - b);
    let currentSeason = seasons.find((s) => s > 0) ?? seasons[0];
    const nowTs = Date.now();

    const renderSeasonEpisodes = (seasonNum) => {
      currentSeason = seasonNum;
      const eps = videos.filter((v) => (v.season ?? 1) === seasonNum);
      const grid = document.getElementById('detail-episodes-grid');
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

async function openStreamsScreen(ctx) {
  if (streamUnlistenAddon) { streamUnlistenAddon(); streamUnlistenAddon = null; }
  if (streamUnlistenDone) { streamUnlistenDone(); streamUnlistenDone = null; }
  if (streamUnlistenScrapers) { streamUnlistenScrapers(); streamUnlistenScrapers = null; }

  activeStreamCtx = ctx;
  streamGroups = [];
  activeStreamFilter = 'all';
  activeScrapers = [];

  setTab('streams');
  const artwork = ctx.episode?.thumbnail || ctx.background || ctx.poster || '';
  document.getElementById('streams-bg-image').style.backgroundImage = artwork ? `url("${artwork}")` : 'none';

  const epLabel = ctx.episode
    ? `T${ctx.episode.season ?? 1} E${ctx.episode.episode ?? ctx.episode.number ?? 1} • ${ctx.episode.title || ctx.episode.name || ''}`
    : '';

  document.getElementById('streams-meta-summary').innerHTML = `
    ${ctx.logo ? `<img src="${esc(ctx.logo)}" style="max-width:300px;max-height:84px;object-fit:contain;" alt="" />` : `<h2 style="margin:0;font-size:28px;">${esc(ctx.title)}</h2>`}
    ${ctx.episode?.thumbnail ? `<img class="streams-ep-thumb" src="${esc(ctx.episode.thumbnail)}" alt="" />` : ''}
    ${epLabel ? `<div style="font-size:16px;font-weight:600;">${esc(epLabel)}</div>` : ''}
    ${ctx.episode?.overview || ctx.episode?.description ? `<p class="muted">${esc(ctx.episode.overview || ctx.episode.description)}</p>` : ''}
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
  el.classList.remove('hidden');
  el.innerHTML = `
    <div class="active-scrapers-title">Scrapers ativos</div>
    <div class="active-scrapers-row">
      ${activeScrapers.map((name) => `<span class="scraper-shimmer-chip">${esc(name)}</span>`).join('')}
    </div>
  `;
}

document.getElementById('btn-streams-back').addEventListener('click', () => {
  setTab(activeMetaContext ? 'details' : (previousTab || 'home'));
});

function renderStreamsView(ctx, stillSearching) {
  const filterRow = document.getElementById('streams-filter-row');
  const statusEl = document.getElementById('streams-status-bar');
  const listEl = document.getElementById('streams-cards-list');

  const totalCount = streamGroups.reduce((acc, g) => acc + g.streams.length, 0);
  statusEl.textContent = stillSearching
    ? (totalCount > 0 ? `${totalCount} stream(s) carregadas…` : 'Buscando nos addons…')
    : (totalCount > 0 ? '' : 'Nenhuma stream encontrada nos addons ativos.');

  filterRow.innerHTML = [
    `<button class="streams-refresh-btn ${stillSearching ? 'spin-icon' : ''}" id="btn-streams-refresh" title="Atualizar streams">${SVG_REFRESH}</button>`,
    `<button class="streams-provider-pill ${activeStreamFilter === 'all' ? 'active' : ''}" data-filter="all">Todos</button>`,
    ...streamGroups.map((g) => `<button class="streams-provider-pill ${activeStreamFilter === g.addon_id ? 'active' : ''}" data-filter="${esc(g.addon_id)}">${esc(g.addon_name)}</button>`),
  ].join('');

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
      if (appSettings?.external_player_enabled) {
        invoke('launch_mpv', { url, headers }).catch((err) => showToast(String(err)));
        return;
      }
      startInAppPlayback({
        url,
        headers,
        title: ctx.title,
        subtitle: epSubtitle,
        mediaType: ctx.mediaType,
        contentId: ctx.contentId || ctx.videoId,
        videoId: ctx.videoId,
        tmdbId: ctx.tmdbId || null,
        season: ctx.episode?.season ?? null,
        episode: ctx.episode?.episode ?? ctx.episode?.number ?? null,
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
    });

    btn.addEventListener('contextmenu', (e) => {
      e.preventDefault();
      if (!url) return;
      openPosterContextMenu({
        id: ctx.contentId || ctx.videoId,
        video_id: ctx.videoId,
        type: ctx.mediaType,
        name: s.name || g.addon_name,
        poster: ctx.poster || '',
        banner: ctx.background || '',
      }, [
        {
          label: 'Abrir no mpv externo',
          onClick: () => invoke('launch_mpv', { url, headers }).catch((err) => showToast(String(err))),
        },
        {
          label: 'Copiar URL do stream',
          onClick: async () => {
            await navigator.clipboard?.writeText(url);
            showToast('Link copiado!');
          },
        },
      ]);
    });
  });
}

// --- EMBEDDED IN-APP LIBMPV PLAYER + OFFICIAL OVERLAY CONTROLLER ---
let playerStateUnlisten = null;
let playerUiEventUnlisten = null;
let playerCurrentPaused = false;
let playerSeeking = false;
let playerActive = false;
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
  return String(v || '').trim().toLowerCase().replace(/_/g, '-');
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
  if (!ctx || !st || st.duration_secs <= 10) return;
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
  if (type === 'setPlaybackState') {
    ctx.controlsVisible = true;
  }
  if (type === 'back') {
    await closeEmbeddedPlayer();
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
    await invoke('launch_mpv', { url: ctx.url, headers: ctx.headers || null }).catch((e) => showToast(String(e)));
    return;
  }
  if (type === 'selectAudioTrack') {
    if (value >= 0) await invoke('player_set_audio', { aid: String(Math.round(value)) });
    return;
  }
  if (type === 'selectBuiltInSubtitleTrack') {
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
    ctx.selectedAddonIndex = idx;
    ctx.activeAddonSub = sub;
    ctx.activeSubOptionId = `addon:${sub.id || idx}`;
    ctx.autoSyncedApplied = false;
    await invoke('player_clear_external_subtitles');
    await invoke('player_add_subtitle_url', { url: sub.url });
    pushPlayerControls();
    return;
  }
  if (type === 'fetchAddonSubtitles') {
    await fetchAddonSubtitlesForPlayer();
    return;
  }
  if (type === 'subtitleDelayDelta') {
    ctx.subtitleDelayMs += Math.round(value);
    await invoke('player_set_subtitle_delay', { delayMs: ctx.subtitleDelayMs });
    pushPlayerControls();
    return;
  }
  if (type === 'subtitleDelayReset') {
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
        headers: sel.stream.behaviorHints?.proxyHeaders?.request || null,
        videoId: ep.id || ctx.contentId,
        season: s,
        episode: e,
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
          headers: preferred.stream.behaviorHints?.proxyHeaders?.request || null,
          videoId: ep.id || ctx.contentId,
          season: s,
          episode: e,
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

async function startInAppPlayback(req) {
  await invoke('player_stop').catch(() => {});
  showView('view-player');
  playerActive = true;
  playerCurrentPaused = false;

  const subStyleCopy = { ...DEFAULT_SUBTITLE_STYLE, ...(appSettings?.subtitle_style || {}) };
  activePlayerCtx = {
    ...req,
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
    prefTracksApplied: false,
    controlsVisible: true,
    lastProgressSyncMs: Date.now(),
    latestState: null,
  };

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
    const wasPaused = activePlayerCtx.latestState?.paused;
    const wasLoading = activePlayerCtx.latestState?.loading;
    activePlayerCtx.latestState = st;
    updateFallbackPlayerUi(st);
    if (wasLoading !== undefined && Boolean(wasLoading) !== Boolean(st.loading)) {
      pushPlayerControls();
    }

    // Preferred audio & subtitle auto-selection once tracks populate
    if (!activePlayerCtx.prefTracksApplied && (st.audio_tracks.length || st.subtitle_tracks.length)) {
      activePlayerCtx.prefTracksApplied = true;
      const prefAud = normalizeLangCode(appSettings?.preferred_audio_lang);
      const prefSub = normalizeLangCode(appSettings?.preferred_sub_lang);
      const audMatch = prefAud && st.audio_tracks.find((t) => normalizeLangCode(t.lang).startsWith(prefAud.slice(0, 2)));
      if (audMatch && !audMatch.selected) invoke('player_set_audio', { aid: String(audMatch.id) });
      const subMatch = prefSub && st.subtitle_tracks.find((t) => normalizeLangCode(t.lang).startsWith(prefSub.slice(0, 2)));
      if (subMatch && !subMatch.selected) {
        invoke('player_set_subtitle', { sid: String(subMatch.id) });
        activePlayerCtx.activeSubOptionId = `builtin:${subMatch.id}`;
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
        url: req.url,
        headers: req.headers || null,
        x: b.x,
        y: b.y,
        w: b.w,
        h: b.h,
        startSecs: req.startSecs || null,
        initialControlsJson: initialObj ? JSON.stringify(initialObj) : null,
      });
      if (mode === 'embedded:overlay') {
        playerShell.classList.remove('fallback-active');
        playerShell.classList.add('overlay-active');
        invoke('player_set_bounds', getPlayerStageBounds());
        pushPlayerControls();
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
  const subKey = st.subtitle_tracks.map((t) => `${t.id}:${t.selected}`).join(',');
  if (subSel.dataset.key !== subKey) {
    subSel.dataset.key = subKey;
    const noneSelected = !st.subtitle_tracks.some((t) => t.selected);
    subSel.innerHTML = `<option value="no" ${noneSelected ? 'selected' : ''}>Legendas: Desativadas</option>` +
      st.subtitle_tracks.map((t) => `<option value="${t.id}" ${t.selected ? 'selected' : ''}>Leg: ${esc(t.lang || t.title || `#${t.id}`)}</option>`).join('');
  }
}

async function closeEmbeddedPlayer() {
  if (!playerActive) return;
  await syncActivePlayerWatchProgress(true);
  playerActive = false;
  activePlayerCtx = null;
  document.getElementById('player-shell').classList.remove('overlay-active', 'fallback-active');
  if (playerStateUnlisten) { playerStateUnlisten(); playerStateUnlisten = null; }
  if (playerUiEventUnlisten) { playerUiEventUnlisten(); playerUiEventUnlisten = null; }
  await invoke('player_stop');
  showView('view-app');
  refreshContinueWatching();
}

window.addEventListener('resize', () => {
  if (!playerActive) return;
  invoke('player_set_bounds', getPlayerStageBounds());
});

document.getElementById('btn-player-back').addEventListener('click', closeEmbeddedPlayer);
document.getElementById('btn-player-playpause').addEventListener('click', () => {
  invoke('player_set_paused', { paused: !playerCurrentPaused });
});
document.getElementById('btn-player-back10').addEventListener('click', () => {
  const cur = Number(document.getElementById('player-seek').value) || 0;
  invoke('player_seek', { secs: Math.max(0, cur - 10) });
});
document.getElementById('btn-player-fwd10').addEventListener('click', () => {
  const cur = Number(document.getElementById('player-seek').value) || 0;
  invoke('player_seek', { secs: cur + 10 });
});

const seekInput = document.getElementById('player-seek');
seekInput.addEventListener('input', () => { playerSeeking = true; });
seekInput.addEventListener('change', () => {
  playerSeeking = false;
  invoke('player_seek', { secs: Number(seekInput.value) });
});
document.getElementById('player-volume').addEventListener('input', (e) => {
  invoke('player_set_volume', { volume: Number(e.target.value) });
});
document.getElementById('player-speed-select').addEventListener('change', (e) => {
  invoke('player_set_speed', { speed: Number(e.target.value) });
});
document.getElementById('player-audio-select').addEventListener('change', (e) => {
  invoke('player_set_audio', { aid: e.target.value });
});
document.getElementById('player-sub-select').addEventListener('change', (e) => {
  invoke('player_set_subtitle', { sid: e.target.value });
});
document.getElementById('btn-player-fullscreen').addEventListener('click', () => {
  invoke('window_toggle_fullscreen');
});

// --- SEARCH & DISCOVER (100% Addons + Recent Searches + Dropdowns) ---
let searchSeq = 0;
let searchDebounceTimer = null;
let searchUnlistenSection = null;
let searchUnlistenDone = null;
const searchSectionsMap = new Map();

let discoverCatalogs = [];
let discoverSelectedType = null;
let discoverSelectedKey = null;
let discoverSelectedGenre = null;
let discoverItems = [];
let discoverSeen = new Set();
let discoverLoading = false;
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
  const hasQuery = searchInputEl.value.trim().length > 0;
  const list = loadRecentSearches();
  if (hasQuery || !list.length) {
    wrap.classList.add('hidden');
    wrap.innerHTML = '';
    return;
  }
  wrap.classList.remove('hidden');
  wrap.innerHTML = `
    <span class="muted" style="font-size:12px;">Recentes:</span>
    ${list.map((q) => `
      <span class="recent-search-chip" data-recent-q="${esc(q)}">
        <span>${esc(q)}</span>
        <button data-del-recent="${esc(q)}" title="Remover">&times;</button>
      </span>
    `).join('')}
  `;
  wrap.querySelectorAll('[data-recent-q]').forEach((chip) => {
    chip.addEventListener('click', (e) => {
      if (e.target.dataset.delRecent) {
        const next = loadRecentSearches().filter((x) => x !== e.target.dataset.delRecent);
        localStorage.setItem(recentSearchStorageKey(), JSON.stringify(next));
        renderRecentSearches();
        return;
      }
      searchInputEl.value = chip.dataset.recentQ;
      runSearch();
    });
  });
}

function syncSearchModeUi() {
  const hasQuery = searchInputEl.value.trim().length > 0;
  searchClearBtn.classList.toggle('hidden', searchInputEl.value.length === 0);
  document.getElementById('discover-section').classList.toggle('hidden', hasQuery);
  document.getElementById('search-results').classList.toggle('hidden', !hasQuery);
  renderRecentSearches();
  if (!hasQuery) {
    document.getElementById('search-status').textContent = '';
  }
}

document.getElementById('btn-search').addEventListener('click', () => {
  clearTimeout(searchDebounceTimer);
  runSearch();
});

searchClearBtn.addEventListener('click', () => {
  clearTimeout(searchDebounceTimer);
  searchSeq += 1;
  searchInputEl.value = '';
  searchSectionsMap.clear();
  document.getElementById('search-results').innerHTML = '';
  syncSearchModeUi();
  searchInputEl.focus();
});

searchInputEl.addEventListener('keydown', (e) => {
  if (e.key === 'Enter') {
    clearTimeout(searchDebounceTimer);
    runSearch();
  }
});

searchInputEl.addEventListener('input', () => {
  syncSearchModeUi();
  clearTimeout(searchDebounceTimer);
  const q = searchInputEl.value.trim();
  if (!q) {
    searchSeq += 1;
    searchSectionsMap.clear();
    document.getElementById('search-results').innerHTML = '';
    return;
  }
  searchDebounceTimer = setTimeout(runSearch, 350);
});

async function runSearch() {
  const query = searchInputEl.value.trim();
  const status = document.getElementById('search-status');
  const resultsEl = document.getElementById('search-results');
  syncSearchModeUi();
  if (!query) return;

  saveRecentSearch(query);
  const seq = ++searchSeq;
  searchSectionsMap.clear();
  resultsEl.innerHTML = '';
  status.textContent = 'Buscando nos addons…';

  if (searchUnlistenSection) { searchUnlistenSection(); searchUnlistenSection = null; }
  if (searchUnlistenDone) { searchUnlistenDone(); searchUnlistenDone = null; }

  searchUnlistenSection = await listen('search:section', (event) => {
    const payload = event.payload;
    if (payload.seq !== searchSeq) return;
    const sec = { ...payload.section, searchQuery: query };
    searchSectionsMap.set(sec.order, sec);
    status.textContent = '';
    appendShelfOrdered(resultsEl, sec, searchSectionsMap);
  });

  searchUnlistenDone = await listen('search:done', (event) => {
    const payload = event.payload;
    if (payload.seq !== searchSeq) return;
    if (searchSectionsMap.size === 0) {
      status.textContent = payload.total === 0
        ? 'Nenhum addon ativo possui catálogo com suporte a busca.'
        : 'Nenhum resultado encontrado nos addons instalados.';
    } else {
      status.textContent = '';
    }
  });

  try {
    const ack = await invoke('start_addon_search', { query, seq });
    if (ack.total === 0 && seq === searchSeq) {
      status.textContent = 'Nenhum addon ativo possui catálogo com suporte a busca.';
    }
  } catch (e) {
    if (seq === searchSeq) status.textContent = String(e);
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
  syncDiscoverSelection();
}

function syncDiscoverSelection() {
  const typePillsEl = document.getElementById('discover-type-pills');
  const catSelectEl = document.getElementById('discover-catalog-select');
  const genreSelectEl = document.getElementById('discover-genre-select');
  const ctxLabelEl = document.getElementById('discover-context-label');
  const gridEl = document.getElementById('discover-grid');
  const statusEl = document.getElementById('discover-status');
  const loadMoreBtn = document.getElementById('btn-discover-load-more');

  const types = [...new Set(discoverCatalogs.map((c) => c.type))];
  if (!types.length) {
    typePillsEl.innerHTML = '';
    catSelectEl.innerHTML = '';
    genreSelectEl.classList.add('hidden');
    ctxLabelEl.textContent = '';
    gridEl.innerHTML = '';
    loadMoreBtn.classList.add('hidden');
    statusEl.textContent = 'Instale ou ative addons com catálogos para explorar.';
    return;
  }

  typePillsEl.innerHTML = types.map((t) => {
    const label = t === 'movie' ? 'Filmes' : (t === 'series' ? 'Séries' : t);
    return `<button class="season-pill ${t === discoverSelectedType ? 'active' : ''}" data-disc-type="${esc(t)}">${esc(label)}</button>`;
  }).join('');

  typePillsEl.querySelectorAll('[data-disc-type]').forEach((btn) => {
    btn.addEventListener('click', () => {
      discoverSelectedType = btn.dataset.discType;
      discoverSelectedKey = null;
      discoverSelectedGenre = null;
      syncDiscoverSelection();
    });
  });

  const filteredCats = discoverCatalogs.filter((c) => c.type === discoverSelectedType);
  if (!discoverSelectedKey || !filteredCats.some((c) => c.key === discoverSelectedKey)) {
    discoverSelectedKey = filteredCats[0]?.key || null;
  }
  const activeCat = filteredCats.find((c) => c.key === discoverSelectedKey) || null;

  catSelectEl.innerHTML = filteredCats.map((c) =>
    `<option value="${esc(c.key)}" ${c.key === discoverSelectedKey ? 'selected' : ''}>${esc(c.catalogName)} (${esc(c.addonName)})</option>`,
  ).join('');
  catSelectEl.onchange = () => {
    discoverSelectedKey = catSelectEl.value;
    discoverSelectedGenre = null;
    syncDiscoverSelection();
  };

  if (activeCat?.genreOptions?.length) {
    if (activeCat.genreRequired && (!discoverSelectedGenre || !activeCat.genreOptions.includes(discoverSelectedGenre))) {
      discoverSelectedGenre = activeCat.genreOptions[0];
    } else if (discoverSelectedGenre && !activeCat.genreOptions.includes(discoverSelectedGenre)) {
      discoverSelectedGenre = null;
    }
    const opts = activeCat.genreRequired ? activeCat.genreOptions : ['__all__', ...activeCat.genreOptions];
    genreSelectEl.innerHTML = opts.map((g) => {
      const val = g === '__all__' ? '' : g;
      const label = g === '__all__' ? 'Gênero: Todos' : g;
      const isSel = (val || null) === discoverSelectedGenre;
      return `<option value="${esc(val)}" ${isSel ? 'selected' : ''}>${esc(label)}</option>`;
    }).join('');
    genreSelectEl.classList.remove('hidden');
    genreSelectEl.onchange = () => {
      discoverSelectedGenre = genreSelectEl.value || null;
      syncDiscoverSelection();
    };
  } else {
    discoverSelectedGenre = null;
    genreSelectEl.classList.add('hidden');
  }

  ctxLabelEl.textContent = activeCat
    ? `${activeCat.catalogName} • ${activeCat.addonName}${discoverSelectedGenre ? ` • ${discoverSelectedGenre}` : ''}`
    : '';

  loadDiscoverPage(true);
}

function maybeInfiniteDiscover() {
  if (!discoverLoading && discoverHasMore) {
    loadDiscoverPage(false);
  }
}

async function loadDiscoverPage(reset) {
  const activeCat = discoverCatalogs.find((c) => c.key === discoverSelectedKey);
  if (!activeCat || discoverLoading) return;
  discoverLoading = true;

  const gridEl = document.getElementById('discover-grid');
  const statusEl = document.getElementById('discover-status');
  const loadBtn = document.getElementById('btn-discover-load-more');

  if (reset) {
    discoverItems = [];
    discoverSeen.clear();
    gridEl.innerHTML = '';
    statusEl.textContent = 'Carregando catálogo…';
  }
  loadBtn.disabled = true;

  try {
    const page = await invoke('load_catalog_page', {
      manifestUrl: activeCat.manifestUrl,
      mediaType: activeCat.type,
      catalogId: activeCat.catalogId,
      skip: reset ? 0 : discoverItems.length,
      genre: discoverSelectedGenre,
      search: null,
    });
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
  } catch (e) {
    statusEl.textContent = String(e);
    discoverHasMore = false;
    loadBtn.classList.add('hidden');
  } finally {
    discoverLoading = false;
    loadBtn.disabled = false;
  }
}

document.getElementById('btn-discover-load-more').addEventListener('click', () => loadDiscoverPage(false));

// --- LIBRARY (LibraryScreen.kt filters & layout toggle) ---
document.getElementById('btn-library-layout').addEventListener('click', () => {
  libraryListMode = !libraryListMode;
  document.getElementById('library-grid').classList.toggle('list-layout', libraryListMode);
});

['library-filter-q', 'library-filter-type', 'library-filter-watched', 'library-sort'].forEach((id) => {
  document.getElementById(id).addEventListener('input', renderFilteredLibrary);
  document.getElementById(id).addEventListener('change', renderFilteredLibrary);
});

document.getElementById('btn-refresh-library').addEventListener('click', refreshLibrary);

async function refreshLibrary() {
  document.getElementById('library-status').textContent = 'Sincronizando biblioteca…';
  await refreshLibraryState();
  renderFilteredLibrary();
}

function renderFilteredLibrary() {
  const status = document.getElementById('library-status');
  const grid = document.getElementById('library-grid');
  const q = document.getElementById('library-filter-q').value.trim().toLowerCase();
  const typeF = document.getElementById('library-filter-type').value;
  const watchF = document.getElementById('library-filter-watched').value;
  const sortF = document.getElementById('library-sort').value;

  let items = [...libraryRawItems];
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

  if (sortF === 'title') {
    items.sort((a, b) => (a.name || '').localeCompare(b.name || ''));
  } else if (sortF === 'year') {
    items.sort((a, b) => (parseInt(b.release_info, 10) || 0) - (parseInt(a.release_info, 10) || 0));
  }

  status.textContent = `${items.length} item(ns) na biblioteca`;
  grid.classList.toggle('list-layout', libraryListMode);
  grid.innerHTML = items.map((it) => renderPosterCard({
    id: it.content_id || it.id || '',
    type: it.content_type || it.type || 'movie',
    name: it.name || 'Item',
    poster: it.poster || '',
    banner: it.background || it.poster || '',
    logo: it.logo || '',
    description: it.description || '',
    release_info: it.release_info || '',
    imdb_rating: it.imdb_rating || '',
  })).join('') || '<div class="muted">Nenhum título encontrado na biblioteca.</div>';
  wirePosterInteractions(grid);
}

// --- SETTINGS ---
function selectSettingsSection(id) {
  document.querySelectorAll('.settings-cat-btn').forEach((b) => {
    b.classList.toggle('active', b.dataset.settingsSec === id);
  });
  document.querySelectorAll('.settings-section').forEach((s) => {
    s.classList.toggle('active', s.id === `sec-${id}`);
  });
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
  document.getElementById('set-account-server').textContent = authState?.server?.backend_url || 'api.nuvio.tv';
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
  await invoke('sign_out');
  showView('view-auth');
});

function applySettingsToUi(s) {
  appSettings = s;
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

  document.getElementById('set-external-player').checked = !!s.external_player_enabled;
  document.getElementById('set-pause-overlay').checked = s.pause_overlay_enabled !== false;
  document.getElementById('set-next-ep').checked = s.next_episode_enabled !== false;
  document.getElementById('set-hold-speed').checked = s.hold_to_speed_enabled !== false;
  document.getElementById('set-hold-speed-val').value = String(s.hold_to_speed_value || 2);
  document.getElementById('set-pref-audio').value = s.preferred_audio_lang || 'por';
  document.getElementById('set-pref-sub').value = s.preferred_sub_lang || 'pt-br';
  document.getElementById('set-mpv-path').value = s.mpv_path || '';

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

  document.getElementById('set-tmdb-key').value = s.tmdb_api_key || '';
  renderThemeSwatches(theme);
}

async function persistSettingsPatch(patch) {
  const s = await invoke('load_settings');
  Object.assign(s, patch);
  await invoke('save_settings', { settings: s });
  applySettingsToUi(s);
  return s;
}

document.getElementById('toggle-amoled').addEventListener('change', (e) => {
  persistSettingsPatch({ amoled_black: e.target.checked });
});

function renderThemeSwatches(currentTheme) {
  const wrap = document.getElementById('theme-swatches');
  wrap.innerHTML = THEMES.map((t) => `
    <button class="swatch-btn ${t.id === currentTheme ? 'active' : ''}" data-theme-id="${t.id}">
      <span class="swatch-dot" style="background:${t.color};"></span>
      <span>${t.label}</span>
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

document.getElementById('btn-save-services').addEventListener('click', async () => {
  await persistSettingsPatch({ tmdb_api_key: document.getElementById('set-tmdb-key').value.trim() });
  document.getElementById('services-status').textContent = 'Salvo!';
});

document.getElementById('btn-clear-cw-cache').addEventListener('click', async () => {
  await invoke('clear_continue_watching_cache');
  document.getElementById('cache-status').textContent = 'Cache limpo.';
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
      <div class="addon-card ${a.enabled ? '' : 'disabled'}">
        <div class="addon-card-top">
          <div class="addon-card-ident" title="${esc(a.manifest_url)}">
            <div class="addon-icon-badge ${logoUrl ? '' : 'is-fallback'}">${iconHtml}</div>
            <div class="addon-title-wrap">
              <div class="addon-title-row">
                <strong>${esc(m.name)}</strong>
                <span class="addon-version">v${esc(m.version)}</span>
              </div>
            </div>
          </div>
          <label class="nuvio-switch" title="${a.enabled ? 'Desativar addon' : 'Ativar addon'}">
            <input type="checkbox" data-toggle-addon="${esc(a.manifest_url)}" ${a.enabled ? 'checked' : ''} />
            <span class="nuvio-switch-track"></span>
          </label>
        </div>

        <div class="addon-actions-row">
          <button class="addon-icon-btn" data-move-addon="${esc(a.manifest_url)}" data-delta="-1" title="Mover para cima" ${index === 0 ? 'disabled' : ''}>${SVG_UP}</button>
          <button class="addon-icon-btn" data-move-addon="${esc(a.manifest_url)}" data-delta="1" title="Mover para baixo" ${index === list.length - 1 ? 'disabled' : ''}>${SVG_DOWN}</button>
          <button class="addon-icon-btn primary" data-refresh-addon="${esc(a.manifest_url)}" title="Atualizar manifesto">${SVG_REFRESH}</button>
          ${configurable ? `<button class="addon-icon-btn" data-config-addon="${esc(configUrl)}" title="Configurar addon">${SVG_CONFIG}</button>` : ''}
          <button class="addon-icon-btn danger" data-del-addon="${esc(a.manifest_url)}" data-addon-name="${esc(m.name)}" title="Remover addon">${SVG_DELETE}</button>
        </div>

        <div class="addon-divider"></div>

        <div class="addon-badges-row">
          <span class="addon-info-badge ${a.enabled ? 'ok' : ''}">${a.enabled ? 'Ativo' : 'Desativado'}</span>
          <span class="addon-info-badge">${(m.resources || []).length} recurso(s)</span>
          <span class="addon-info-badge">${(m.catalogs || []).length} catálogo(s)</span>
          ${configurable ? '<span class="addon-info-badge">Configurável</span>' : ''}
          ${hints.p2p ? '<span class="addon-info-badge">P2P</span>' : ''}
        </div>

        ${m.description ? `<div class="addon-desc">${esc(m.description)}</div>` : ''}
        <div class="addon-meta-summary">${esc(renderAddonSummary(m))}</div>
      </div>
    `;
  }).join('') || '<div class="settings-card"><div class="muted">Nenhum addon instalado.</div></div>';

  root.querySelectorAll('[data-toggle-addon]').forEach((input) => {
    input.addEventListener('change', async () => {
      try {
        const updated = await invoke('toggle_addon', { manifestUrl: input.dataset.toggleAddon, enabled: input.checked });
        renderAddonsList(updated);
        refreshHomeProgressive();
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
        refreshHomeProgressive();
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
        refreshHomeProgressive();
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
      refreshHomeProgressive();
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
    await Promise.all([refreshAddons(), refreshHomeProgressive()]);
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
    <div class="addon-card">
      <div class="addon-card-top">
        <div class="addon-title-wrap">
          <div class="addon-title-row">
            <strong>${esc(r.name)}</strong>
            <span class="addon-version">v${esc(r.version)}</span>
          </div>
          <div class="addon-url-sub">${esc(r.manifest_url)}</div>
        </div>
        <div class="row-gap">
          <button class="addon-icon-btn" data-refresh-repo="${esc(r.manifest_url)}" title="Atualizar repositório">${SVG_REFRESH}</button>
          <button class="addon-icon-btn danger" data-del-repo="${esc(r.manifest_url)}" data-repo-name="${esc(r.name)}" title="Remover repositório">${SVG_DELETE}</button>
        </div>
      </div>
      ${r.description ? `<div class="addon-desc">${esc(r.description)}</div>` : ''}
      <div class="addon-meta-summary">${r.scraper_count} scraper(s)</div>
    </div>
  `).join('') || '<div class="muted">Nenhum repositório de plugins adicionado.</div>';

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
    <div class="addon-card ${s.enabled ? '' : 'disabled'}">
      <div class="addon-card-top">
        <div class="addon-title-wrap">
          <div class="addon-title-row">
            <strong>${esc(s.name)}</strong>
            <span class="addon-version">v${esc(s.version)}</span>
          </div>
          <div class="addon-url-sub">${esc((s.supported_types || []).join(', ') || 'movie, tv')}</div>
        </div>
        <div class="row-gap">
          <button class="btn-ghost" data-test-scraper="${esc(s.id)}" style="font-size:12px;">Testar</button>
          <label class="nuvio-switch">
            <input type="checkbox" data-toggle-scraper="${esc(s.id)}" ${s.enabled ? 'checked' : ''} />
            <span class="nuvio-switch-track"></span>
          </label>
        </div>
      </div>
      ${s.description ? `<div class="addon-desc">${esc(s.description)}</div>` : ''}
    </div>
  `).join('') || '<div class="muted">Nenhum scraper instalado.</div>';

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

document.getElementById('btn-run-plugin').addEventListener('click', async () => {
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
      <div class="addon-row">
        <div><strong>${esc(s.name || 'Stream')}</strong> <span class="muted">${esc(s.title || s.url)}</span></div>
        <button class="btn-accent" data-plugin-play="${esc(s.url)}" data-plugin-title="${esc(s.title || s.name || '')}">Assistir</button>
      </div>
    `).join('');
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
  const d = await invoke('get_diagnostics');
  document.getElementById('diag-box').textContent =
    `OS: ${d.os} (${d.arch})\nConfig: ${d.config_dir}\nCache: ${d.cache_dir}\nSQLite: ${d.sqlite_ok} | Redb: ${d.redb_ok}\nmpv: ${d.mpv_available}\nEngine (${d.engine_library_name}): found=${d.engine_found} api=${d.engine_api_version ?? '-'} libtorrent=${d.engine_backend_version ?? '-'}`;
}

// --- BOOT ---
(async function boot() {
  let status = null;
  try {
    const settings = await invoke('load_settings');
    applySettingsToUi(settings);
    status = await invoke('get_auth_status');
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
})();

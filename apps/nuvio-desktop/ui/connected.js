const connectedNames = {torbox: 'Torbox', premiumize: 'Premiumize'};
const authAttempts = {};
let cloudRequest = 0;
let cloudPrefsSave = Promise.resolve();

for (const [provider, name] of Object.entries(connectedNames)) {
  const card = document.createElement('div');
  card.className = 'nv-account';
  card.dataset.account = provider;
  const icon = provider === 'torbox'
    ? '<img src="assets/torbox_logo.png" alt="" />'
    : '<span class="nv-link-icon">🔗</span>';
  card.innerHTML = `<div class="nv-row"><span class="service-icon" aria-hidden="true">${icon}</span><span class="nv-copy"><span class="nv-title"></span><span class="nv-desc"></span><span class="nv-account-extra"></span></span><span class="nv-account-side"><span role="status" class="nv-value">Não definido</span><span class="nv-account-actions"><button class="btn-accent nv-mini-btn" data-action="login">Conectar</button><button class="btn-ghost nv-mini-btn" data-action="disconnect" hidden>Desconectar</button></span></span></div><details class="nv-account-manual"><summary>Colar token / chave de API</summary><div class="nv-manual-body"><label class="field-col">Token de API<input type="password" autocomplete="off" placeholder="Cole seu token aqui"></label><button class="btn-ghost nv-mini-btn" data-action="key">Conectar com token</button></div></details>`;
  card.querySelector('.nv-title').textContent = name;
  card.querySelector('.nv-desc').textContent = provider === 'torbox'
    ? 'Vincule sua conta Torbox no navegador ou cole seu token abaixo.' : 'Vincule sua conta Premiumize no navegador ou cole sua chave abaixo.';
  if (provider === 'premiumize') {
    const label = document.createElement('label');
    label.className = 'field-col';
    label.textContent = 'Client ID do aplicativo Premiumize (login no navegador)';
    const input = document.createElement('input');
    input.dataset.connectedPref = 'connected.premiumize_app_id';
    label.append(input);
    card.querySelector('.nv-account-extra').append(label);
  }
  document.getElementById('connected-provider-cards').append(card);
  const status = card.querySelector('[role="status"]');
  card.addEventListener('click', async ev => {
    const btn = ev.target.closest('[data-action]');
    if (!btn) return;
    const attempt = Symbol();
    authAttempts[provider] = attempt;
    btn.disabled = true;
    try {
      const appId = card.querySelector('[data-connected-pref]')?.value.trim() || '';
      if (btn.dataset.action === 'disconnect') await invoke('connected_disconnect', {provider});
      if (btn.dataset.action === 'key') {
        const input = card.querySelector('[type="password"]');
        await invoke('connected_use_key', {provider, key:input.value});
        input.value = '';
      }
      if (btn.dataset.action === 'login') {
        const auth = await invoke('connected_start', {provider, appId});
        if (authAttempts[provider] !== attempt) return;
        status.textContent = 'Autorize no navegador com o código ' + auth.code;
        await invoke('open_external_url', {url:auth.url});
        const expiry = Date.now() + auth.expires * 1000;
        let wait = auth.interval;
        let approved = false;
        while (Date.now() < expiry) {
          await new Promise(done => setTimeout(done, wait * 1000));
          if (authAttempts[provider] !== attempt) return;
          const result = await invoke('connected_poll', {provider, appId});
          if (result === 'connected') { approved = true; break; }
          if (result === 'slow') wait += 5;
        }
        if (!approved) throw Error('O código expirou. Conecte novamente.');
      }
      await refreshConnectedAccounts();
      if (document.getElementById('cloud-library')?.open) loadCloudFiles();
    } catch (e) { if (authAttempts[provider] === attempt) status.textContent = String(e); }
    finally { btn.disabled = false; }
  });
}

async function refreshConnectedAccounts() {
  try {
    const accounts = await invoke('connected_accounts');
    document.querySelectorAll('[data-account]').forEach(card => {
      const linked = accounts[card.dataset.account];
      card.querySelector('[role="status"]').textContent = linked ? 'Conectado' : 'Não definido';
      card.querySelector('[data-action="login"]').hidden = linked;
      card.querySelector('[data-action="disconnect"]').hidden = !linked;
    });
  } catch (e) { showToast(String(e)); }
}

function applyConnectedPreferences(settings) {
  document.querySelectorAll('[data-connected-pref]').forEach(el => {
    const [group, key] = el.dataset.connectedPref.split('.');
    const value = settings[group]?.[key];
    if (el.type === 'checkbox') el.checked = Boolean(value);
    else if (el !== document.activeElement) el.value = value ?? '';
  });
  const library = document.getElementById('cloud-library');
  if (library) {
    library.hidden = !settings.connected?.library;
    if (library.hidden) { library.open = false; document.getElementById('cloud-files')?.replaceChildren(); cloudRequest++; }
  }
}

document.querySelectorAll('[data-connected-pref]').forEach(el => {
  el.addEventListener('change', () => {
    const [group,key] = el.dataset.connectedPref.split('.');
    const value = el.type === 'checkbox' ? el.checked : ['count','size'].includes(key) ? Number(el.value) : el.value.trim();
    cloudPrefsSave = cloudPrefsSave.then(() => persistSettingsPatch({[group]: {...appSettings[group], [key]:value}})).catch(e => showToast(String(e)));
  });
});

function clearConnectedSession() {
  cloudRequest++;
  for (const p of Object.keys(connectedNames)) authAttempts[p] = null;
  const library = document.getElementById('cloud-library');
  if (library) library.open = false;
  document.getElementById('cloud-files')?.replaceChildren();
  document.querySelectorAll('[data-account] input[type="password"]').forEach(el => { el.value=''; });
}

async function loadCloudFiles() {
  const serial = ++cloudRequest;
  const list = document.getElementById('cloud-files');
  if (!list) return;
  list.textContent = 'Carregando arquivos…';
  try {
    const accounts = await invoke('connected_accounts');
    const active = Object.keys(connectedNames).filter(p => accounts[p]);
    const results = await Promise.allSettled(active.map(provider => invoke('connected_files',{provider})));
    if (serial !== cloudRequest) return;
    list.replaceChildren();
    if (!active.length) list.textContent = 'Conecte uma conta em Ajustes → Integrações.';
    results.forEach((result,i) => {
      const name = connectedNames[active[i]];
      const title = document.createElement('strong');
      title.textContent = name;
      list.append(title);
      if (result.status === 'rejected' || !result.value.length) {
        const message = document.createElement('span');
        message.textContent = result.reason || 'Nenhum vídeo encontrado.';
        list.append(message);
        return;
      }
      result.value.forEach(file => {
        const btn = document.createElement('button');
        btn.className = 'btn-ghost';
        btn.textContent = file.name;
        btn.addEventListener('click', async () => {
          btn.disabled = true;
          try {
            const url = await invoke('connected_play', {file});
            if (serial !== cloudRequest) return;
            if (appSettings?.external_player_enabled) await invoke('launch_mpv',{url,headers:null,anime:false});
            else await startInAppPlayback({url,title:file.name,providerName:name,mediaType:'video',videos:[]});
          } catch(e) { showToast(String(e)); } finally { btn.disabled=false; }
        });
        list.append(btn);
      });
    });
  } catch (e) { if(serial === cloudRequest) list.textContent = String(e); }
}
document.getElementById('refresh-cloud')?.addEventListener('click', loadCloudFiles);
document.getElementById('cloud-library')?.addEventListener('toggle', ev => {
  if (ev.target.open) loadCloudFiles();
});

async function resolveConnectedMedia(req) {
  if (!appSettings?.connected?.resolve || !req.url?.startsWith('magnet:?')) return req;
  const url = await invoke('connected_resolve',{request:{magnet:req.url,fileIdx:req.fileIdx ?? null,season:req.season ?? null,episode:req.episode ?? null}});
  return {...req, url, headers:null};
}

async function launchConnectedExternal(url, headers, stream, ctx) {
  const req = await resolveConnectedMedia({url, headers, fileIdx:stream.fileIdx, season:ctx.episode?.season, episode:ctx.episode?.episode ?? ctx.episode?.number});
  return invoke('launch_mpv',{url:req.url, headers:req.headers, anime:playbackIsAnime(ctx)});
}

async function loadSeekPreview(ctx, duration) {
  if (ctx.seekPreviewAfter > Date.now() || !appSettings?.seekr?.enabled || !appSettings.seekr.key || !(duration > 0)) return;
  const id = String(ctx.tmdbId || ctx.contentId || '').replace(/^tmdb:/,'').split(':')[0];
  if (!/^(tt)?\d+$/.test(id)) return;
  ctx.seekPreviewAfter = Infinity;
  try {
    const track = await invoke('seekr_track',{id, durationMs:Math.round(duration*1000), season:ctx.season ?? null, episode:ctx.episode ?? null});
    if (activePlayerCtx !== ctx) return;
    ctx.seekPreview = track;
    ctx.seekPreviewAfter = Date.now() + 5 * 60 * 60 * 1000;
    pushPlayerControls();
  } catch (e) {
    if (activePlayerCtx !== ctx) return;
    notifyPlayerToast('Prévias Seekr indisponíveis: ' + e);
    ctx.seekPreviewAfter = Infinity;
  }
}

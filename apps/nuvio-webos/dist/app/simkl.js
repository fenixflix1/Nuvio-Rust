let simklAccount = {};
let simklGeneration = 0;
let simklSyncTask;
let simklScrobbleQueue = Promise.resolve();
const simklDefaultSources = { library:'default', progress:'default', similar:'default', ratings:'default', anime:'tvdb' };
function simklProgressMatches(card, id) {
  return card.content_id === id || Object.entries(simklIds(id) || {})
    .some(([source, value]) => String(card.media_ids?.[source]) === String(value));
}

const simklField = name => document.getElementById('simkl-' + name);
const simklUses = key => {
  if (key === 'library') {
    return typeof activeLibrarySource === 'function'
      ? activeLibrarySource() === 'simkl'
      : (simklAccount.connected && simklAccount.sources?.library === 'simkl');
  }
  if (key === 'progress') {
    return typeof activeProgressSource === 'function'
      ? activeProgressSource() === 'simkl'
      : (simklAccount.connected && simklAccount.sources?.progress === 'simkl');
  }
  return simklAccount.connected && simklAccount.sources?.[key] === 'simkl';
};
const simklRequest = (action, data = null) => invoke('simkl_action', {profile:simklAccount.profile, action, data});

function simklRenderAccount() {
  const connected = !!simklAccount.connected;
  simklField('account').textContent = connected ? ('Conectado' + (simklAccount.username ? ' como ' + simklAccount.username : ' ao Simkl')) : 'Conta desconectada';
  simklField('client-id').value = simklAccount.clientId || '';
  for (const id of ['client-field','connect']) simklField(id).classList.toggle('hidden', connected);
  for (const id of ['sync','disconnect']) simklField(id).classList.toggle('hidden', !connected);
  document.querySelectorAll('[data-simkl-source]').forEach(select => {
    if (select.dataset.simklSource === 'library') {
      select.value = typeof activeLibrarySource === 'function'
        ? activeLibrarySource()
        : (appSettings?.library_source || 'default');
      select.disabled = false;
      return;
    }
    if (select.dataset.simklSource === 'progress') {
      select.value = typeof activeProgressSource === 'function'
        ? activeProgressSource()
        : (appSettings?.progress_source || 'default');
      select.disabled = false;
      return;
    }
    select.value = simklAccount.sources?.[select.dataset.simklSource] || simklDefaultSources[select.dataset.simklSource];
    select.disabled = !connected;
  });
}

function simklSync() {
  if (simklSyncTask) return simklSyncTask;
  const generation = simklGeneration;
  const task = simklRequest('sync').then(snapshot => {
    if (generation === simklGeneration) simklAccount.snapshot = snapshot;
  }).finally(() => { if (simklSyncTask === task) simklSyncTask = null; });
  simklSyncTask = task;
  return task;
}

async function simklLoadProfile(sync = true) {
  const generation = ++simklGeneration;
  simklAccount = {};
  simklSyncTask = null;
  simklRenderAccount();
  simklField('status').textContent = '';
  simklField('connect').disabled = false;
  try {
    const account = await invoke('simkl_status');
    if (generation !== simklGeneration) return;
    simklAccount = account;
    simklRenderAccount();
    if (sync && account.connected) await simklSync();
  } catch (e) {
    if (generation === simklGeneration) simklField('status').textContent = String(e);
  }
}

async function simklRefreshViews() {
  await Promise.all([refreshWatchedKeys(),refreshLibraryState(),refreshContinueWatching()]);
  renderFilteredLibrary();
  refreshPosterWatchedBadges();
}

simklField('open').onclick = () => invoke('open_external_url', {url:'https://simkl.com'});
simklField('connect').onclick = async () => {
  const generation = simklGeneration;
  try {
    simklField('connect').disabled = true;
    const pin = await simklRequest('connect',{clientId:simklField('client-id').value});
    if (generation !== simklGeneration) return;
    const expires = Date.now() + (pin.expires || 900) * 1000;
    simklField('status').textContent = 'Digite o código ' + pin.code + ' em ' + (pin.url || 'https://simkl.com/pin');
    const url = new URL(pin.url || 'https://simkl.com/pin');
    if (url.protocol === 'https:' && (url.hostname === 'simkl.com' || url.hostname.endsWith('.simkl.com'))) {
      await invoke('open_external_url', {url:url.href}).catch(() => {});
    }
    while (Date.now() < expires && generation === simklGeneration) {
      await new Promise(resolve => setTimeout(resolve, Math.max(1,pin.interval || 5)*1000));
      if (generation !== simklGeneration) return;
      if (await simklRequest('poll',pin.code)) {
        if (generation === simklGeneration) { await simklLoadProfile(); await simklRefreshViews(); }
        return;
      }
    }
    if (generation === simklGeneration) simklField('status').textContent = 'Código expirado. Conecte novamente.';
  } catch(e) { if (generation === simklGeneration) simklField('status').textContent = String(e); }
  finally {if (generation === simklGeneration) simklField('connect').disabled = false;}
};
simklField('disconnect').onclick = async () => {
  try {
    await simklRequest('disconnect');
    await simklLoadProfile(false);
    await simklRefreshViews();
  } catch(e) {showToast(String(e));}
};
simklField('sync').onclick = async () => {
  const button = simklField('sync');
  const generation = simklGeneration;
  button.disabled = true;
  try {
    await simklSync();
    if (generation !== simklGeneration) return;
    await simklRefreshViews();
    simklField('status').textContent = 'Sincronização concluída.';
  } catch(e) {if (generation === simklGeneration) simklField('status').textContent = String(e);}
  finally {button.disabled = false;}
};
document.querySelectorAll('[data-simkl-source]').forEach(select => {
  select.onchange = async () => {
    if (select.dataset.simklSource === 'library') {
      const nextSource = select.value;
      if (nextSource === 'simkl' && !simklAccount.connected) {
        showToast('Conecte sua conta Simkl acima para usar como fonte da biblioteca.');
        simklRenderAccount();
        return;
      }
      if (nextSource === 'mdblist' && !String(appSettings?.services?.mdblist_key || '').trim()) {
        showToast('Configure a chave de API do MDBList em Ajustes → Integrações.');
        simklRenderAccount();
        return;
      }
      try {
        await persistSettingsPatch({ library_source: nextSource });
        if (simklAccount.connected) {
          const sources = {...simklDefaultSources, ...simklAccount.sources, library: nextSource === 'simkl' ? 'simkl' : 'default'};
          await simklRequest('sources', sources);
          simklAccount.sources = sources;
        }
        await refreshLibraryState(true);
        renderFilteredLibrary();
        refreshPosterLibraryBadges();
      } catch(e) { simklRenderAccount(); showToast(String(e)); }
      return;
    }
    if (select.dataset.simklSource === 'progress') {
      const nextSource = select.value;
      if (nextSource === 'simkl' && !simklAccount.connected) {
        showToast('Conecte sua conta Simkl acima para usar como progresso de reprodução.');
        simklRenderAccount();
        return;
      }
      if (nextSource === 'mdblist' && !String(appSettings?.services?.mdblist_key || '').trim()) {
        showToast('Conecte sua conta MDBList acima antes de selecioná-la como fonte.');
        simklRenderAccount();
        return;
      }
      try {
        await persistSettingsPatch({ progress_source: nextSource });
        if (simklAccount.connected) {
          const sources = {...simklDefaultSources, ...simklAccount.sources, progress: nextSource === 'simkl' ? 'simkl' : 'default'};
          await simklRequest('sources', sources);
          simklAccount.sources = sources;
        }
        if (nextSource === 'mdblist' && typeof refreshMdblistProgress === 'function') {
          await refreshMdblistProgress(true);
        }
        await Promise.all([refreshWatchedKeys(), refreshContinueWatching()]);
        refreshPosterWatchedBadges();
      } catch(e) { simklRenderAccount(); showToast(String(e)); }
      return;
    }
    const sources = {...simklDefaultSources, ...simklAccount.sources, [select.dataset.simklSource]:select.value};
    try {
      await simklRequest('sources',sources);
      simklAccount.sources = sources;
      await simklRefreshViews();
    } catch(e) {simklRenderAccount(); showToast(String(e));}
  };
});

function simklMediaId(media, anime) {
  const ids = media?.ids || {};
  const preferred = anime ? simklAccount.sources?.anime || 'tvdb' : 'imdb';
  for (const source of [preferred,'imdb','tmdb','tvdb','mal','kitsu','anilist','anidb','simkl']) {
    if (ids[source]) return source === 'imdb' ? String(ids[source]) : source + ':' + ids[source];
  }
  return '';
}

function simklIds(id) {
  if (/^tt\d+$/.test(id)) return {imdb:id};
  const parts = /^(tmdb|tvdb|simkl|mal|kitsu|anidb|anilist):(\d+)$/.exec(id);
  return parts ? {[parts[1]]:Number(parts[2])} : null;
}

function simklPoster(media) {
  return media?.poster ? 'https://simkl.in/posters/' + encodeURIComponent(media.poster) + '_m.webp' : '';
}

function simklLibrary() {
  const items = [];
  for (const bucket of ['movies','shows','anime']) {
    for (const entry of simklAccount.snapshot?.items?.[bucket] || []) {
      const media = entry.movie || entry.show;
      const id = simklMediaId(media,bucket === 'anime');
      if (!id) continue;
      items.push({content_id:id, content_type:bucket === 'movies' || entry.anime_type === 'movie' ? 'movie':'series',
        name:media.title || id, poster:simklPoster(media), added_at:Date.parse(entry.added_to_watchlist_at) || 0,
        release_info:String(media.year || ''), entry, media, bucket});
    }
  }
  return items;
}

function simklWatchedKeys() {
  const keys = [];
  for (const {content_id:id, content_type:type, entry, media} of simklLibrary()) {
    const aliases = [id, ...Object.entries(media.ids || {}).filter(([k,v]) => v && k !== 'slug')
      .map(([k,v]) => k === 'imdb' ? String(v) : k + ':' + v)];
    if (entry.status === 'completed' || (type === 'movie' && entry.last_watched_at)) keys.push(...aliases);
    for (const season of entry.seasons || []) {
      for (const ep of season.episodes || []) {
        if (!ep.watched_at) continue;
        for (const alias of aliases) {
          const tvStyle = alias.startsWith('tt') || alias.startsWith('tvdb:');
          keys.push(epWatchKey(alias,tvStyle ? ep.tvdb?.season ?? season.number : season.number,
            tvStyle ? ep.tvdb?.episode ?? ep.number : ep.number));
        }
      }
    }
  }
  return keys;
}

function simklProgressCards() {
  const cards = [];
  for (const row of simklAccount.snapshot?.playback || []) {
    const media = row.movie || row.show || row.anime;
    const id = simklMediaId(media, !!row.anime);
    const percent = Number(row.progress);
    if (!id || !(percent > 0 && percent < 100)) continue;
    const tvStyle = id.startsWith('tt') || id.startsWith('tvdb:') || id.startsWith('tmdb:');
    const season = tvStyle ? row.episode?.tvdb_season ?? row.episode?.season : row.episode?.season;
    const episode = tvStyle ? row.episode?.tvdb_number ?? row.episode?.number : row.episode?.number;
    if (row.show && episode == null) continue;
    const animeId = /^(mal|kitsu|anidb|anilist):/.test(id);
    const videoId = episode == null ? id : animeId ? id + ':' + episode : id + ':' + (season ?? 1) + ':' + episode;
    cards.push({content_id:id, media_ids:media.ids, video_id:videoId,
      content_type:episode != null || row.show ? 'series':'movie', title:media.title || id,
      poster:simklPoster(media), season:episode != null ? season ?? 1:null, episode:episode ?? null,
      episode_title:row.episode?.title, progress_percent:percent, simkl_session:row.id,
      position_ms:0, last_watched:Date.parse(row.paused_at || row.watched_at) || 0});
  }
  return cards.sort((a,b) => b.last_watched - a.last_watched);
}

function simklFindEntry(id) {
  const target = simklIds(id);
  return simklLibrary().find(it => it.content_id === id || Object.entries(target || {})
    .some(([key,val]) => String(it.media.ids?.[key]) === String(val)));
}

async function simklMutate(action, id, type, fields = {}, bucket = null) {
  const ids = simklIds(id);
  if (!ids) throw new Error('O título precisa de um ID externo reconhecido pelo Simkl.');
  if (simklSyncTask) await simklSyncTask;
  const group = bucket || (type === 'movie' ? 'movies':'shows');
  await simklRequest(action, {[group]:[{ids,...fields}]});
  await simklSync();
}

function simklSendScrobble(ctx, st, stop = false) {
  if (!simklUses('progress') || !st || st.loading || !(st.duration_secs > 10)) return;
  const ids = simklIds(ctx.contentId);
  if (!ids || (ctx.mediaType !== 'movie' && ctx.episode == null)) return;
  const action = stop || st.ended ? 'stop' : st.paused ? 'pause':'start';
  const percent = Math.min(100,Math.max(0,100*st.position_secs/st.duration_secs));
  if (ctx.simklLastAction === 'stop') return simklScrobbleQueue;
  if ((ctx.simklLastAction === action &&
      (action !== 'start' || Date.now() - ctx.simklAt < 60000))) return;
  ctx.simklAt = Date.now();
  ctx.simklLastAction = action;
  let body = {movie:{ids}};
  if (ctx.episode != null) {
    const anime = /^(mal|kitsu|anilist|anidb):(\d+):(\d+)$/.exec(ctx.videoId || '');
    body = anime
      ? {anime:{ids:{[anime[1]]:Number(anime[2])}}, episode:{number:Number(anime[3])}}
      : {show:{ids}, episode:{season:ctx.season ?? 1,number:ctx.episode}};
  }
  const generation = simklGeneration;
  simklScrobbleQueue = simklScrobbleQueue.then(async () => {
    if (generation === simklGeneration && simklUses('progress')) {
      await simklRequest(action,{...body,progress:percent});
      if (action === 'stop') await simklSync();
    }
  }).catch(e => {
    if (ctx.simklLastAction === action) ctx.simklLastAction = null;
    if (generation === simklGeneration && !ctx.simklErrorShown) {
      ctx.simklErrorShown = true;
      showToast('Falha ao enviar a reprodução para o Simkl. ' + String(e));
    }
  });
  return simklScrobbleQueue;
}

function simklRenderDetails(preview, container) {
  if (simklUses('ratings')) {
    const field = document.createElement('label');
    field.className = 'toggle-row';
    field.textContent = 'Minha nota no Simkl';
    const select = document.createElement('select');
    select.add(new Option('Sem nota',''));
    for (let rating=1; rating<=10; rating++) select.add(new Option(rating+'/10',rating));
    const entry = simklFindEntry(preview.id);
    select.value = entry?.entry.user_rating || '';
    select.onchange = async () => {
      select.disabled = true;
      try {
        const rating = Number(select.value);
        const bucket = entry?.bucket === 'anime' ? (rating ? 'anime' : 'shows') : null;
        await simklMutate(rating?'rate':'unrate',preview.id,preview.type,rating?{rating}:{},bucket);
      } catch(e) {select.value = simklFindEntry(preview.id)?.entry.user_rating || ''; showToast(String(e));}
      finally {select.disabled=false;}
    };
    field.append(select);
    container.prepend(field);
  }
  if (!simklUses('similar')) return;
  const ids = simklIds(preview.id);
  if (!ids) return;
  const section = document.createElement('section');
  container.append(section);
  const [source,id] = Object.entries(ids)[0];
  simklRequest('similar',{source,id:String(id),type:preview.type}).then(items => {
    if (!section.isConnected) return;
    const seen = new Set();
    const cards = [];
    for (const item of items) {
      const id = simklMediaId(item,item.type === 'anime');
      if (!id || seen.has(id)) continue;
      seen.add(id);
      cards.push(renderPosterCard({id,type:item.type === 'movie' || item.anime_type === 'movie'?'movie':'series',
        name:item.en_title || item.title || id,poster:simklPoster(item),release_info:String(item.year || '')}));
      if (cards.length === 20) break;
    }
    if (cards.length) {
      section.innerHTML = '<h3>Títulos semelhantes no Simkl</h3><div class="shelf-row">'+cards.join('')+'</div>';
      wirePosterInteractions(section);
      wireHorizontalShelfDrag(section);
    }
  }).catch(e => {if (section.isConnected) section.textContent = 'Falha ao carregar títulos semelhantes. ' + String(e);});
}

let mdblistCardUsername = '';

async function refreshMdblistAccountCard(verify = false) {
  const accountEl = document.getElementById('mdblist-account');
  const connectBox = document.getElementById('mdblist-connect-box');
  const keyInput = document.getElementById('mdblist-card-key');
  const connectBtn = document.getElementById('btn-mdblist-connect');
  const syncBtn = document.getElementById('btn-mdblist-sync');
  const disconnectBtn = document.getElementById('btn-mdblist-disconnect');
  const statusEl = document.getElementById('mdblist-card-status');
  if (!accountEl || !connectBtn) return;

  const key = String(appSettings?.services?.mdblist_key || '').trim();
  if (keyInput && document.activeElement !== keyInput) keyInput.value = key;

  if (!key) {
    mdblistCardUsername = '';
    accountEl.textContent = '';
    accountEl.classList.add('hidden');
    connectBtn.classList.remove('hidden');
    syncBtn?.classList.add('hidden');
    disconnectBtn?.classList.add('hidden');
    return;
  }

  connectBox?.classList.add('hidden');
  connectBtn.classList.add('hidden');
  syncBtn?.classList.remove('hidden');
  disconnectBtn?.classList.remove('hidden');
  accountEl.classList.remove('hidden');
  accountEl.textContent = mdblistCardUsername ? `Conectado como ${mdblistCardUsername}` : 'Conectado ao MDBList';

  if (verify) {
    try {
      const info = await invoke('mdblist_account_status');
      if (info?.connected) {
        mdblistCardUsername = info.username || '';
        accountEl.textContent = mdblistCardUsername ? `Conectado como ${mdblistCardUsername}` : 'Conectado ao MDBList';
        if (statusEl && statusEl.textContent.startsWith('Falha')) statusEl.textContent = '';
      }
    } catch (e) {
      if (statusEl) statusEl.textContent = String(e);
    }
  }
}
window.refreshMdblistAccountCard = refreshMdblistAccountCard;

async function saveMdblistCardKey() {
  const keyInput = document.getElementById('mdblist-card-key');
  const saveBtn = document.getElementById('btn-mdblist-save-key');
  const statusEl = document.getElementById('mdblist-card-status');
  const key = String(keyInput?.value || '').trim();
  if (!key) {
    if (statusEl) statusEl.textContent = 'Cole sua chave de API do MDBList.';
    keyInput?.focus();
    return;
  }
  const prevKey = String(appSettings?.services?.mdblist_key || '');
  if (saveBtn) saveBtn.disabled = true;
  if (statusEl) statusEl.textContent = 'Validando chave do MDBList...';
  try {
    const nextServices = { ...(appSettings?.services || {}), mdblist_key: key };
    await persistSettingsPatch({ services: nextServices });
    const info = await invoke('mdblist_account_status');
    mdblistCardUsername = info?.username || '';
    document.getElementById('mdblist-connect-box')?.classList.add('hidden');
    if (statusEl) statusEl.textContent = 'MDBList conectado com sucesso.';
    await refreshMdblistAccountCard(false);
    if (typeof activeLibrarySource === 'function' && activeLibrarySource() === 'mdblist') {
      await refreshLibraryState(true);
      renderFilteredLibrary();
      refreshPosterLibraryBadges();
    }
    if (typeof activeProgressSource === 'function' && activeProgressSource() === 'mdblist') {
      if (typeof refreshMdblistProgress === 'function') await refreshMdblistProgress(true);
      await Promise.all([refreshWatchedKeys(), refreshContinueWatching()]);
      refreshPosterWatchedBadges();
    }
  } catch (e) {
    const revertServices = { ...(appSettings?.services || {}), mdblist_key: prevKey };
    await persistSettingsPatch({ services: revertServices }).catch(() => {});
    if (statusEl) statusEl.textContent = String(e);
    await refreshMdblistAccountCard(false);
  } finally {
    if (saveBtn) saveBtn.disabled = false;
  }
}

document.getElementById('btn-mdblist-open')?.addEventListener('click', () => {
  invoke('open_external_url', { url: 'https://mdblist.com' });
});

document.getElementById('btn-mdblist-connect')?.addEventListener('click', () => {
  const box = document.getElementById('mdblist-connect-box');
  if (!box) return;
  box.classList.toggle('hidden');
  if (!box.classList.contains('hidden')) {
    document.getElementById('mdblist-card-key')?.focus();
  }
});

document.getElementById('btn-mdblist-save-key')?.addEventListener('click', saveMdblistCardKey);
document.getElementById('mdblist-card-key')?.addEventListener('keydown', (e) => {
  if (e.key === 'Enter') {
    e.preventDefault();
    saveMdblistCardKey();
  }
});

document.getElementById('btn-mdblist-sync')?.addEventListener('click', async () => {
  const btn = document.getElementById('btn-mdblist-sync');
  const statusEl = document.getElementById('mdblist-card-status');
  if (btn) btn.disabled = true;
  if (statusEl) statusEl.textContent = 'Sincronizando MDBList...';
  try {
    const tasks = [refreshMdblistAccountCard(true)];
    if (typeof activeLibrarySource === 'function' && activeLibrarySource() === 'mdblist') {
      tasks.push(refreshLibraryState(true));
    }
    if (typeof refreshMdblistProgress === 'function') {
      tasks.push(refreshMdblistProgress(true));
    }
    await Promise.all(tasks);
    await Promise.all([refreshWatchedKeys(), refreshContinueWatching()]);
    renderFilteredLibrary();
    refreshPosterLibraryBadges();
    refreshPosterWatchedBadges();
    if (statusEl) statusEl.textContent = 'Sincronização MDBList concluída.';
  } catch (e) {
    if (statusEl) statusEl.textContent = String(e);
  } finally {
    if (btn) btn.disabled = false;
  }
});

document.getElementById('btn-mdblist-disconnect')?.addEventListener('click', async () => {
  const statusEl = document.getElementById('mdblist-card-status');
  try {
    const patch = {
      services: { ...(appSettings?.services || {}), mdblist_key: '' },
    };
    if (typeof activeLibrarySource === 'function' && activeLibrarySource() === 'mdblist') {
      patch.library_source = 'default';
    }
    if (typeof activeProgressSource === 'function' && activeProgressSource() === 'mdblist') {
      patch.progress_source = 'default';
    }
    await persistSettingsPatch(patch);
    mdblistCardUsername = '';
    if (typeof clearMdblistState === 'function') clearMdblistState();
    await refreshMdblistAccountCard(false);
    await Promise.all([refreshWatchedKeys(), refreshLibraryState(), refreshContinueWatching()]);
    renderFilteredLibrary();
    refreshPosterLibraryBadges();
    refreshPosterWatchedBadges();
    if (statusEl) statusEl.textContent = 'MDBList desconectado.';
  } catch (e) {
    showToast(String(e));
  }
});

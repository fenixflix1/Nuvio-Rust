(function () {
  'use strict';

  window.NUVIO_WEBOS = true;

  var SERVICE_ID = 'luna://com.nuvio.tv.service';
  var LOCAL_PROXY_BASE = '/__webos';
  var STORAGE_PREFIX = 'nuvio_tv_';
  var listeners = Object.create(null);
  var memStore = Object.create(null);

  function hasLunaBus() {
    return typeof window.webOS !== 'undefined' && window.webOS.service && typeof window.webOS.service.request === 'function';
  }

  function hasDevProxy() {
    return !hasLunaBus() && (location.protocol === 'http:' || location.protocol === 'https:') && (location.hostname === '127.0.0.1' || location.hostname === 'localhost');
  }

  function lunaRequest(method, parameters) {
    return new Promise(function (resolve, reject) {
      if (hasLunaBus()) {
        window.webOS.service.request(SERVICE_ID, {
          method: method,
          parameters: parameters || {},
          onSuccess: function (res) { resolve(res); },
          onFailure: function (err) { reject(new Error((err && (err.errorText || err.message)) || 'Luna service error')); }
        });
        return;
      }
      if (hasDevProxy()) {
        fetch(LOCAL_PROXY_BASE + '/' + method, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(parameters || {})
        })
          .then(function (r) { return r.json(); })
          .then(resolve)
          .catch(reject);
        return;
      }
      reject(new Error('No webOS service transport available'));
    });
  }

  async function netFetch(url, opts) {
    opts = opts || {};
    var method = String(opts.method || 'GET').toUpperCase();
    var headers = opts.headers || {};
    var body = opts.body != null ? (typeof opts.body === 'string' ? opts.body : JSON.stringify(opts.body)) : undefined;

    if (hasLunaBus() || hasDevProxy()) {
      var res = await lunaRequest('httpFetch', {
        url: url,
        method: method,
        headers: headers,
        body: body,
        timeoutMs: opts.timeoutMs || 20000
      });
      if (!res || res.returnValue === false && !res.status) {
        throw new Error((res && res.errorText) || 'Falha na requisição de rede');
      }
      return {
        ok: Boolean(res.ok),
        status: Number(res.status || 0),
        headers: res.headers || {},
        text: function () { return Promise.resolve(String(res.body || '')); },
        json: function () { return Promise.resolve(JSON.parse(res.body || 'null')); }
      };
    }

    var direct = await fetch(url, { method: method, headers: headers, body: body });
    var txt = await direct.text();
    return {
      ok: direct.ok,
      status: direct.status,
      headers: {},
      text: function () { return Promise.resolve(txt); },
      json: function () { return Promise.resolve(JSON.parse(txt || 'null')); }
    };
  }

  function storageGet(key, fallback) {
    try {
      var raw = localStorage.getItem(STORAGE_PREFIX + key);
      if (raw != null) return JSON.parse(raw);
    } catch (_) {}
    return key in memStore ? memStore[key] : fallback;
  }

  function storageSet(key, val) {
    memStore[key] = val;
    try {
      localStorage.setItem(STORAGE_PREFIX + key, JSON.stringify(val));
    } catch (_) {}
    if (hasLunaBus() || hasDevProxy()) {
      lunaRequest('kvSet', { key: key, value: val }).catch(function () {});
    }
  }

  var kvHydrated = false;
  async function ensureKvHydrated() {
    if (kvHydrated) return;
    kvHydrated = true;
    if (!hasLunaBus() && !hasDevProxy()) return;
    var keys = ['settings', 'auth', 'addons', 'library', 'watched', 'progress', 'followed_people', 'home_cache'];
    await Promise.all(keys.map(function (k) {
      return lunaRequest('kvGet', { key: k }).then(function (res) {
        if (res && res.returnValue && res.value !== null && res.value !== undefined) {
          memStore[k] = res.value;
          try {
            if (localStorage.getItem(STORAGE_PREFIX + k) == null) {
              localStorage.setItem(STORAGE_PREFIX + k, JSON.stringify(res.value));
            }
          } catch (_) {}
        }
      }).catch(function () {});
    }));
  }

  function emit(eventName, payload) {
    var list = listeners[eventName];
    if (!list || !list.length) return;
    list.slice().forEach(function (cb) {
      try { cb({ event: eventName, payload: payload }); } catch (e) { console.error('[webos event error]', eventName, e); }
    });
  }

  function listen(eventName, cb) {
    if (!listeners[eventName]) listeners[eventName] = [];
    listeners[eventName].push(cb);
    return Promise.resolve(function unlisten() {
      var arr = listeners[eventName];
      if (!arr) return;
      var idx = arr.indexOf(cb);
      if (idx >= 0) arr.splice(idx, 1);
    });
  }

  function defaultSettings() {
    return {
      playback: { auto_play_next: true, seek_step_secs: 10 },
      services: {
        providers: ['imdb', 'tmdb', 'tomatoes'],
        tmdb_enabled: true,
        tmdb_artwork: true,
        tmdb_credits: true,
        tmdb_details: true,
        mdblist_enabled: false,
        mdblist_hero: false,
        mdblist_key: '',
        discord_enabled: false
      },
      connected: { library: true, resolve: true, providers: {} },
      seekr: {},
      tmdb_api_key: '',
      theme: 'white',
      amoled_black: false,
      mpv_path: '',
      custom_theme_colors: ['#FFFFFF', '#B8BEC5', '#6E7178'],
      nav_style: 'sidebar',
      nav_mode: 'adaptive',
      show_hero: true,
      cw_style: 'card',
      blur_unwatched_episodes: false,
      hover_preview_enabled: false,
      detail_bg_mode: 'cinematic',
      streams_bg_mode: 'cover',
      dns_provider: 'cloudflare',
      dns_custom: '',
      stream_badges_top: true,
      show_addon_logo: true,
      show_file_size_badge: true,
      preferred_audio_lang: 'por',
      preferred_sub_lang: 'pt-br',
      skip_intro_enabled: true,
      auto_skip_intro: false,
      auto_skip_recap: false,
      auto_skip_outro: false,
      aniskip_enabled: true,
      introdb_url: '',
      introdb_api_key: '',
      hold_to_speed_enabled: false,
      hold_to_speed_value: 2.0,
      external_player_enabled: false,
      library_source: 'default',
      progress_source: 'default',
      subtitle_style: {
        fontSizeSp: 20,
        outlineEnabled: true,
        bold: false,
        bottomOffset: 24,
        textColor: '#FFFFFFFF',
        outlineColor: '#FF000000',
        backgroundColor: '#00000000',
        shadowEnabled: true,
        shadowOpacity: 75,
        shadowSize: 2
      }
    };
  }

  function loadSettings() {
    var def = defaultSettings();
    var saved = storageGet('settings', null);
    if (!saved || typeof saved !== 'object') return def;
    return Object.assign({}, def, saved, {
      playback: Object.assign({}, def.playback, saved.playback || {}),
      services: Object.assign({}, def.services, saved.services || {}),
      connected: Object.assign({}, def.connected, saved.connected || {}),
      subtitle_style: Object.assign({}, def.subtitle_style, saved.subtitle_style || {})
    });
  }

  function defaultAuthState() {
    return {
      signed_in: false,
      is_guest: false,
      email: null,
      user_id: null,
      server: {
        backend_url: 'https://api.nuvio.tv',
        publishable_key: '',
        capabilities: { password: false, device_link: false },
        is_custom: false,
        discovery_url: null
      },
      profiles: [
        {
          profile_index: 1,
          name: 'TV',
          avatar_color_hex: '#FFFFFF',
          avatar_url: null,
          is_primary: true,
          is_kids_profile: false,
          uses_primary_plugins: false
        }
      ],
      active_profile_index: 1
    };
  }

  function loadAuth() {
    var saved = storageGet('auth', null);
    return saved && typeof saved === 'object' ? Object.assign({}, defaultAuthState(), saved) : defaultAuthState();
  }

  async function completeSupabaseSession(srv, sess) {
    storageSet('session', sess);
    var user = sess.user || null;
    if (!user && sess.access_token) {
      try {
        var ur = await netFetch(srv.backend_url + '/auth/v1/user', {
          headers: { apikey: srv.publishable_key, Authorization: 'Bearer ' + sess.access_token }
        });
        if (ur.ok) user = await ur.json();
      } catch (_) {}
    }
    var profiles = [];
    try {
      var pr = await netFetch(srv.backend_url + '/rest/v1/profiles?select=profile_index,name,avatar_color_hex,avatar_url,is_primary,is_kids_profile,uses_primary_plugins&order=profile_index.asc', {
        headers: { apikey: srv.publishable_key, Authorization: 'Bearer ' + sess.access_token }
      });
      if (pr.ok) profiles = await pr.json();
    } catch (_) {}
    if (!Array.isArray(profiles) || !profiles.length) profiles = defaultAuthState().profiles;
    var activeIdx = profiles[0].profile_index || 1;
    var auth = Object.assign({}, defaultAuthState(), {
      signed_in: true,
      is_guest: false,
      email: user ? user.email : null,
      user_id: user ? user.id : null,
      server: srv,
      profiles: profiles,
      active_profile_index: activeIdx
    });
    storageSet('auth', auth);
    // Puxa addons do perfil da nuvem para a TV
    try {
      var ar = await netFetch(srv.backend_url + '/rest/v1/addons?select=url,name,enabled,sort_order&profile_id=eq.' + activeIdx + '&order=sort_order.asc', {
        headers: { apikey: srv.publishable_key, Authorization: 'Bearer ' + sess.access_token }
      });
      if (ar.ok) {
        var rows = await ar.json();
        if (Array.isArray(rows) && rows.length) {
          var fetched = await Promise.all(rows.map(async function (row) {
            var clean = ensureManifestSuffix(row.url);
            try {
              var mr = await netFetch(clean, { timeoutMs: 8000 });
              if (!mr.ok) return null;
              var manifest = await mr.json();
              manifest.transport_url = clean;
              return { manifest_url: clean, manifest: manifest, enabled: row.enabled !== false };
            } catch (_) { return null; }
          }));
          var valid = fetched.filter(Boolean);
          if (valid.length) storageSet('addons_p' + activeIdx, valid);
        }
      }
    } catch (_) {}
    return auth;
  }

  function activeProfileSuffix() {
    var a = loadAuth();
    return 'p' + String(a.active_profile_index || 1);
  }

  function ensureManifestSuffix(url) {
    var trimmed = String(url || '').trim().replace(/\/+$/, '');
    if (!trimmed) return '';
    return /\/manifest\.json$/i.test(trimmed) ? trimmed : trimmed + '/manifest.json';
  }

  function addonBaseUrl(manifestUrl) {
    return String(manifestUrl || '').replace(/\/manifest\.json(?:\?.*)?$/i, '');
  }

  function addonSupportsResource(manifest, resName, type, id) {
    if (!manifest || !Array.isArray(manifest.resources)) return false;
    var ok = false;
    for (var i = 0; i < manifest.resources.length; i++) {
      var r = manifest.resources[i];
      if (typeof r === 'string' && r === resName) { ok = true; break; }
      if (r && typeof r === 'object' && r.name === resName) {
        var typesOk = !r.types || !type || r.types.indexOf(type) >= 0;
        var prefixes = r.idPrefixes || manifest.idPrefixes;
        var prefixOk = !prefixes || !id || prefixes.some(function (p) { return String(id).indexOf(p) === 0; });
        if (typesOk && prefixOk) { ok = true; break; }
      }
    }
    if (!ok) return false;
    if (type && Array.isArray(manifest.types) && manifest.types.length && manifest.types.indexOf(type) < 0) return false;
    return true;
  }

  function catalogExtraString(extraObj) {
    if (!extraObj || typeof extraObj !== 'object') return '';
    var parts = [];
    Object.keys(extraObj).forEach(function (k) {
      var v = extraObj[k];
      if (v !== undefined && v !== null && String(v) !== '') {
        parts.push(encodeURIComponent(k) + '=' + encodeURIComponent(String(v)));
      }
    });
    return parts.join('&');
  }

  async function fetchStremioCatalog(manifest, manifestUrl, type, catalogId, extraObj) {
    var base = addonBaseUrl(manifestUrl);
    var extra = catalogExtraString(extraObj);
    var path = '/catalog/' + encodeURIComponent(type) + '/' + encodeURIComponent(catalogId) + (extra ? '/' + extra : '') + '.json';
    var res = await netFetch(base + path, { timeoutMs: 12000 });
    if (!res.ok) throw new Error('HTTP ' + res.status);
    var data = await res.json();
    return Array.isArray(data && data.metas) ? data.metas : [];
  }

  function normalizeTmdbItem(row, mediaType) {
    var tmdbId = row.id;
    return {
      id: 'tmdb:' + tmdbId,
      type: mediaType,
      name: row.title || row.name || '',
      poster: row.poster_path ? 'https://image.tmdb.org/t/p/w500' + row.poster_path : '',
      background: row.backdrop_path ? 'https://image.tmdb.org/t/p/w1280' + row.backdrop_path : '',
      description: row.overview || '',
      releaseInfo: String(row.release_date || row.first_air_date || '').slice(0, 4),
      imdbRating: row.vote_average ? Number(row.vote_average).toFixed(1) : ''
    };
  }

  async function fetchTmdbJson(path, apiKey) {
    var sep = path.indexOf('?') >= 0 ? '&' : '?';
    var url = 'https://api.themoviedb.org/3' + path + sep + 'api_key=' + encodeURIComponent(apiKey);
    var res = await netFetch(url, { timeoutMs: 12000 });
    if (!res.ok) throw new Error('TMDB HTTP ' + res.status);
    return res.json();
  }

  function watchedKey(contentId, season, episode) {
    return season != null && episode != null ? contentId + '_s' + season + 'e' + episode : String(contentId);
  }

  // --- HTML5 WEBOS PLAYER ---
  var playerState = {
    active: false,
    paused: false,
    buffering: false,
    ended: false,
    loading: false,
    position_secs: 0,
    duration_secs: 0,
    volume: 100,
    speed: 1.0,
    audio_tracks: [],
    subtitle_tracks: [],
    chapters: []
  };
  var playerVideoEl = null;
  var playerTimer = null;
  var externalSubTracks = [];

  function syncPlayerTracksFromVideo() {
    if (!playerVideoEl) return;
    var subs = [];
    var tt = playerVideoEl.textTracks;
    for (var i = 0; i < (tt ? tt.length : 0); i++) {
      subs.push({
        id: String(i),
        lang: tt[i].language || 'und',
        title: tt[i].label || ('Legenda #' + (i + 1)),
        selected: tt[i].mode === 'showing',
        forced: false,
        codec: 'VTT'
      });
    }
    var aud = [];
    var at = playerVideoEl.audioTracks;
    for (var j = 0; j < (at ? at.length : 0); j++) {
      aud.push({
        id: String(j),
        lang: at[j].language || 'und',
        title: at[j].label || ('Áudio #' + (j + 1)),
        selected: Boolean(at[j].enabled)
      });
    }
    playerState.subtitle_tracks = subs;
    playerState.audio_tracks = aud;
  }

  function emitPlayerState() {
    if (!playerVideoEl) return;
    playerState.position_secs = Number(playerVideoEl.currentTime || 0);
    playerState.duration_secs = Number.isFinite(playerVideoEl.duration) ? Number(playerVideoEl.duration || 0) : 0;
    playerState.paused = Boolean(playerVideoEl.paused);
    playerState.ended = Boolean(playerVideoEl.ended);
    syncPlayerTracksFromVideo();
    emit('player:state', Object.assign({}, playerState));
  }

  function clearExternalSubBlobs() {
    externalSubTracks.forEach(function (item) {
      if (item.trackEl && item.trackEl.parentNode) item.trackEl.parentNode.removeChild(item.trackEl);
      if (item.blobUrl) URL.revokeObjectURL(item.blobUrl);
    });
    externalSubTracks = [];
  }

  function stopWebosPlayer() {
    if (playerTimer) { clearInterval(playerTimer); playerTimer = null; }
    clearExternalSubBlobs();
    if (playerVideoEl) {
      try {
        playerVideoEl.pause();
        playerVideoEl.removeAttribute('src');
        playerVideoEl.load();
      } catch (_) {}
      if (playerVideoEl.parentNode) playerVideoEl.parentNode.removeChild(playerVideoEl);
      playerVideoEl = null;
    }
    playerState.active = false;
    playerState.ended = false;
    playerState.loading = false;
  }

  function srtToVtt(raw) {
    var text = String(raw || '').replace(/^\uFEFF/, '').trim();
    if (/^WEBVTT/i.test(text)) return text;
    var converted = text.replace(/(\d{2}:\d{2}:\d{2}),(\d{3})/g, '$1.$2');
    return 'WEBVTT\n\n' + converted;
  }

  function startWebosPlayer(args) {
    stopWebosPlayer();
    var stage = document.getElementById('player-video-stage');
    if (!stage) throw new Error('Palco do player não encontrado.');
    var v = document.createElement('video');
    v.id = 'webos-player-video';
    v.className = 'webos-video';
    v.setAttribute('playsinline', '');
    v.autoplay = true;
    v.crossOrigin = 'anonymous';
    v.volume = Math.min(1, Math.max(0, (playerState.volume || 100) / 100));
    v.playbackRate = Number(playerState.speed || 1);
    stage.prepend(v);
    playerVideoEl = v;

    playerState.active = true;
    playerState.loading = true;
    playerState.buffering = true;
    playerState.ended = false;

    v.addEventListener('loadedmetadata', function () {
      playerState.loading = false;
      playerState.buffering = false;
      if (args.startSecs && args.startSecs > 1 && Number.isFinite(v.duration) && args.startSecs < v.duration - 2) {
        v.currentTime = Number(args.startSecs);
      }
      emitPlayerState();
    });
    v.addEventListener('canplay', function () {
      playerState.loading = false;
      playerState.buffering = false;
      emitPlayerState();
    });
    v.addEventListener('waiting', function () {
      playerState.buffering = true;
      emitPlayerState();
    });
    v.addEventListener('playing', function () {
      playerState.loading = false;
      playerState.buffering = false;
      emitPlayerState();
    });
    v.addEventListener('pause', emitPlayerState);
    v.addEventListener('ended', emitPlayerState);
    v.addEventListener('error', function () {
      var mediaErr = v.error ? ('Código ' + v.error.code) : 'Erro de vídeo';
      var st = document.getElementById('player-stage-status');
      if (st) st.textContent = 'Não foi possível reproduzir este stream no player web (' + mediaErr + ').';
      playerState.loading = false;
      playerState.buffering = false;
      emitPlayerState();
    });

    v.src = args.url;
    v.play().catch(function () {});
    playerTimer = setInterval(emitPlayerState, 250);
    return 'embedded:fallback';
  }

  // --- COMMAND HANDLERS ---
  var handlers = {
    load_settings: function () { return loadSettings(); },
    save_settings: function (args) { storageSet('settings', args.settings || {}); return null; },
    frontend_log: function () { return null; },
    get_diagnostics: function () {
      return {
        os: 'webOS',
        arch: 'tv',
        config_dir: '/media/internal/.nuvio-tv',
        cache_dir: 'localStorage',
        sqlite_ok: true,
        redb_ok: true,
        engine_library_name: 'HTML5 webOS Media',
        engine_found: true,
        mpv_available: false
      };
    },
    runtime_metrics: function () { return { rss_bytes: 0, cpu_percent: 0 }; },
    open_external_url: function (args) {
      if (args && args.url) {
        if (window.NuvioWebosQr && typeof window.NuvioWebosQr.showModal === 'function') {
          window.NuvioWebosQr.showModal(args.url, 'Autorizar no celular');
        } else {
          window.open(args.url, '_blank');
        }
      }
      return null;
    },
    window_toggle_fullscreen: function () { return null; },

    get_auth_status: async function () {
      var auth = loadAuth();
      if (!auth.server || !auth.server.publishable_key) {
        try { auth.server = await handlers.configure_server({ serverUrl: null }); } catch (_) {}
      }
      return auth;
    },
    continue_as_guest: function () {
      var auth = loadAuth();
      auth.is_guest = true;
      storageSet('auth', auth);
      return auth;
    },
    select_profile: function (args) {
      var auth = loadAuth();
      auth.active_profile_index = Number(args.profileIndex || 1);
      storageSet('auth', auth);
      return auth;
    },
    sign_out: function () {
      storageSet('session', null);
      var auth = defaultAuthState();
      storageSet('auth', auth);
      return null;
    },
    configure_server: async function (args) {
      var auth = loadAuth();
      var url = String((args && args.serverUrl) || 'https://api.nuvio.tv').trim().replace(/\/+$/, '');
      var r = await netFetch(url + '/.well-known/nuvio', { timeoutMs: 10000 });
      if (!r.ok) throw new Error('Falha ao conectar em ' + url + ' (HTTP ' + r.status + ')');
      var doc = await r.json();
      auth.server = {
        backend_url: String(doc.backend_url || url).replace(/\/+$/, ''),
        publishable_key: doc.publishable_key || '',
        capabilities: doc.capabilities || { password: true, device_link: true },
        is_custom: url !== 'https://api.nuvio.tv',
        discovery_url: url + '/.well-known/nuvio'
      };
      storageSet('auth', auth);
      return auth.server;
    },
    sign_in_email: async function (args) {
      var srv = await handlers.get_auth_status().then(function (a) { return a.server; });
      if (!srv || !srv.backend_url || !srv.publishable_key) throw new Error('Servidor de login não configurado.');
      var r = await netFetch(srv.backend_url + '/auth/v1/token?grant_type=password', {
        method: 'POST',
        headers: { apikey: srv.publishable_key, 'Content-Type': 'application/json' },
        body: { email: String(args.email || '').trim(), password: String(args.password || '') }
      });
      var data = await r.json();
      if (!r.ok) throw new Error((data && (data.msg || data.error_description || data.message)) || ('HTTP ' + r.status));
      return completeSupabaseSession(srv, data);
    },
    sign_up_email: async function (args) {
      var srv = await handlers.get_auth_status().then(function (a) { return a.server; });
      if (!srv || !srv.backend_url || !srv.publishable_key) throw new Error('Servidor de login não configurado.');
      var r = await netFetch(srv.backend_url + '/auth/v1/signup', {
        method: 'POST',
        headers: { apikey: srv.publishable_key, 'Content-Type': 'application/json' },
        body: { email: String(args.email || '').trim(), password: String(args.password || '') }
      });
      var data = await r.json();
      if (!r.ok) throw new Error((data && (data.msg || data.error_description || data.message)) || ('HTTP ' + r.status));
      if (!data.access_token) throw new Error('Verifique seu e-mail para confirmar a conta.');
      return completeSupabaseSession(srv, data);
    },
    start_device_link: async function () {
      var authStateNow = await handlers.get_auth_status();
      var srv = authStateNow.server;
      if (!srv || !srv.backend_url || !srv.publishable_key) throw new Error('Servidor Nuvio indisponível no momento.');
      var nonceBytes = new Uint8Array(16);
      if (window.crypto && window.crypto.getRandomValues) window.crypto.getRandomValues(nonceBytes);
      else for (var i = 0; i < 16; i++) nonceBytes[i] = Math.floor(Math.random() * 256);
      var nonce = Array.from(nonceBytes).map(function (b) { return b.toString(16).padStart(2, '0'); }).join('');
      var redirectBase = srv.is_custom ? srv.backend_url : 'https://nuvio.tv/link';
      var r = await netFetch(srv.backend_url + '/rest/v1/rpc/start_device_login_session', {
        method: 'POST',
        headers: { apikey: srv.publishable_key, 'Content-Type': 'application/json' },
        body: {
          p_device_nonce: nonce,
          p_redirect_base_url: redirectBase,
          p_device_name: 'Nuvio TV (LG webOS)',
          p_device_type: 'mobile'
        }
      });
      var rows = await r.json();
      if (!r.ok || !Array.isArray(rows) || !rows[0]) {
        throw new Error((rows && rows.message) || 'Falha ao gerar código QR de login.');
      }
      var started = rows[0];
      return {
        device_code: started.device_code,
        user_code: started.user_code,
        verification_uri_complete: started.verification_uri_complete,
        poll_interval_seconds: started.poll_interval_seconds || 5,
        nonce: nonce
      };
    },
    poll_device_link: async function (args) {
      var srv = loadAuth().server;
      if (!srv || !srv.backend_url || !srv.publishable_key) throw new Error('Servidor não configurado.');
      var r = await netFetch(srv.backend_url + '/rest/v1/rpc/poll_tv_login_session', {
        method: 'POST',
        headers: { apikey: srv.publishable_key, 'Content-Type': 'application/json' },
        body: { p_device_code: args.deviceCode, p_device_nonce: args.nonce }
      });
      var rows = await r.json();
      if (!r.ok) throw new Error((rows && rows.message) || ('HTTP ' + r.status));
      var status = (Array.isArray(rows) && rows[0] && rows[0].status ? String(rows[0].status) : 'pending').trim().toLowerCase();
      if (status !== 'approved') return { status: status, auth: null };
      var ex = await netFetch(srv.backend_url + '/functions/v1/tv-logins-exchange', {
        method: 'POST',
        headers: { apikey: srv.publishable_key, 'Content-Type': 'application/json' },
        body: { code: args.deviceCode, device_nonce: args.nonce }
      });
      var sess = await ex.json();
      if (!ex.ok || !sess.access_token) throw new Error((sess && sess.message) || 'Falha ao concluir login');
      var auth = await completeSupabaseSession(srv, sess);
      return { status: 'approved', auth: auth };
    },

    list_addons: function () {
      return storageGet('addons_' + activeProfileSuffix(), []);
    },
    add_addon: async function (args) {
      var clean = ensureManifestSuffix(args && args.manifestUrl);
      if (!clean) throw new Error('URL do manifest vazia.');
      var res = await netFetch(clean, { timeoutMs: 15000 });
      if (!res.ok) throw new Error('Falha ao carregar addon: HTTP ' + res.status);
      var manifest = await res.json();
      if (!manifest || !manifest.id || !manifest.name) throw new Error('Manifest de addon inválido.');
      manifest.transport_url = clean;
      var list = storageGet('addons_' + activeProfileSuffix(), []).filter(function (a) { return a.manifest_url !== clean; });
      var entry = { manifest_url: clean, manifest: manifest, enabled: true };
      list.push(entry);
      storageSet('addons_' + activeProfileSuffix(), list);
      return entry;
    },
    remove_addon: function (args) {
      var list = storageGet('addons_' + activeProfileSuffix(), []).filter(function (a) { return a.manifest_url !== args.manifestUrl; });
      storageSet('addons_' + activeProfileSuffix(), list);
      return list;
    },
    toggle_addon: function (args) {
      var list = storageGet('addons_' + activeProfileSuffix(), []);
      list.forEach(function (a) { if (a.manifest_url === args.manifestUrl) a.enabled = Boolean(args.enabled); });
      storageSet('addons_' + activeProfileSuffix(), list);
      return list;
    },
    move_addon: function (args) {
      var list = storageGet('addons_' + activeProfileSuffix(), []);
      var idx = list.findIndex(function (a) { return a.manifest_url === args.manifestUrl; });
      if (idx >= 0) {
        var target = Math.max(0, Math.min(list.length - 1, idx + Number(args.delta || 0)));
        if (target !== idx) {
          var item = list.splice(idx, 1)[0];
          list.splice(target, 0, item);
          storageSet('addons_' + activeProfileSuffix(), list);
        }
      }
      return list;
    },
    refresh_addon: async function (args) {
      var clean = ensureManifestSuffix(args && args.manifestUrl);
      var res = await netFetch(clean, { timeoutMs: 15000 });
      if (!res.ok) throw new Error('HTTP ' + res.status);
      var manifest = await res.json();
      manifest.transport_url = clean;
      var list = storageGet('addons_' + activeProfileSuffix(), []);
      list.forEach(function (a) {
        if (a.manifest_url === clean || a.manifest_url === args.manifestUrl) {
          a.manifest_url = clean;
          a.manifest = manifest;
        }
      });
      storageSet('addons_' + activeProfileSuffix(), list);
      return list;
    },

    start_home_catalogs: function (args) {
      var addons = storageGet('addons_' + activeProfileSuffix(), []).filter(function (a) { return a.enabled; });
      var requests = [];
      addons.forEach(function (a) {
        var cats = (a.manifest && Array.isArray(a.manifest.catalogs)) ? a.manifest.catalogs : [];
        cats.forEach(function (c) {
          var hasRequired = Array.isArray(c.extra) && c.extra.some(function (e) { return e && e.isRequired; });
          if (hasRequired) return;
          var typeLabel = c.type === 'movie' ? 'Filmes' : (c.type === 'series' ? 'Séries' : c.type);
          requests.push({
            order: requests.length,
            addon: a,
            media_type: c.type,
            catalog_id: c.id,
            title: (c.name || c.id) + ' - ' + typeLabel
          });
        });
      });

      var cached = !args.force ? storageGet('home_cache_' + activeProfileSuffix(), []) : [];
      if (!requests.length) {
        setTimeout(function () { emit('home:done', { total: 0 }); }, 10);
        return { total: 0, cached_sections: [], cache_fresh: true };
      }

      Promise.all(requests.map(function (r) {
        return fetchStremioCatalog(r.addon.manifest, r.addon.manifest_url, r.media_type, r.catalog_id, null)
          .then(function (items) {
            if (!items.length) return null;
            var sec = {
              order: r.order,
              key: (r.addon.manifest.id || r.addon.manifest_url) + ':' + r.media_type + ':' + r.catalog_id,
              title: r.title,
              addon_name: r.addon.manifest.name,
              manifest_url: r.addon.manifest_url,
              media_type: r.media_type,
              catalog_id: r.catalog_id,
              items: items.slice(0, 20)
            };
            emit('home:section', sec);
            return sec;
          })
          .catch(function () { return null; });
      })).then(function (sections) {
        var valid = sections.filter(Boolean).sort(function (a, b) { return a.order - b.order; });
        if (valid.length) storageSet('home_cache_' + activeProfileSuffix(), valid);
        emit('home:done', { total: valid.length });
      });

      return { total: requests.length, cached_sections: cached || [], cache_fresh: false };
    },

    load_catalog_page: async function (args) {
      var addons = storageGet('addons_' + activeProfileSuffix(), []);
      var found = addons.find(function (a) { return a.manifest_url === args.manifestUrl; });
      if (!found) return [];
      var extra = {};
      if (args.skip > 0) extra.skip = Number(args.skip);
      if (args.genre) extra.genre = String(args.genre);
      return fetchStremioCatalog(found.manifest, found.manifest_url, args.mediaType, args.catalogId, extra);
    },

    start_addon_search: function (args) {
      var q = String((args && args.query) || '').trim();
      var seq = Number((args && args.seq) || 0);
      if (!q) return { total: 0 };
      var addons = storageGet('addons_' + activeProfileSuffix(), []).filter(function (a) { return a.enabled; });
      var targets = [];
      addons.forEach(function (a) {
        ((a.manifest && a.manifest.catalogs) || []).forEach(function (c) {
          var canSearch = Array.isArray(c.extra) && c.extra.some(function (e) { return e && e.name === 'search'; });
          if (canSearch) targets.push({ addon: a, cat: c, order: targets.length });
        });
      });

      Promise.all(targets.map(function (t) {
        return fetchStremioCatalog(t.addon.manifest, t.addon.manifest_url, t.cat.type, t.cat.id, { search: q })
          .then(function (items) {
            if (!items.length) return;
            var typeLabel = t.cat.type === 'movie' ? 'Filmes' : (t.cat.type === 'series' ? 'Séries' : t.cat.type);
            emit('search:section', {
              seq: seq,
              section: {
                order: t.order,
                key: t.addon.manifest.id + ':search:' + t.cat.type + ':' + t.cat.id,
                title: (t.cat.name || t.cat.id) + ' - ' + typeLabel,
                addon_name: t.addon.manifest.name,
                manifest_url: t.addon.manifest_url,
                media_type: t.cat.type,
                catalog_id: t.cat.id,
                items: items
              }
            });
          })
          .catch(function () {});
      })).then(function () {
        emit('search:done', { seq: seq, total: targets.length });
      });

      return { total: targets.length };
    },

    fetch_meta_details: async function (args) {
      var mediaType = args.mediaType || 'movie';
      var id = String(args.id || '');
      var addons = storageGet('addons_' + activeProfileSuffix(), []).filter(function (a) {
        return a.enabled && addonSupportsResource(a.manifest, 'meta', mediaType, id);
      });
      var meta = null;
      for (var i = 0; i < addons.length; i++) {
        try {
          var url = addonBaseUrl(addons[i].manifest_url) + '/meta/' + encodeURIComponent(mediaType) + '/' + encodeURIComponent(id) + '.json';
          var res = await netFetch(url, { timeoutMs: 12000 });
          if (res.ok) {
            var json = await res.json();
            if (json && json.meta) { meta = json.meta; break; }
          }
        } catch (_) {}
      }

      var tmdbKey = String(loadSettings().tmdb_api_key || '').trim();
      var tmdbNum = id.indexOf('tmdb:') === 0 ? id.slice(5).split(':')[0] : null;
      if (!tmdbNum && id.indexOf('tt') === 0 && tmdbKey) {
        try {
          var found = await fetchTmdbJson('/find/' + encodeURIComponent(id.split(':')[0]) + '?external_source=imdb_id&language=pt-BR', tmdbKey);
          var hit = (mediaType === 'movie' ? (found.movie_results || found.tv_results) : (found.tv_results || found.movie_results)) || [];
          if (hit[0] && hit[0].id) tmdbNum = String(hit[0].id);
        } catch (_) {}
      }

      if (tmdbNum && tmdbKey) {
        try {
          var ep = mediaType === 'movie' ? 'movie' : 'tv';
          var det = await fetchTmdbJson('/' + ep + '/' + encodeURIComponent(tmdbNum) + '?language=pt-BR&append_to_response=credits,videos,external_ids', tmdbKey);
          if (!meta) meta = { id: id, type: mediaType, name: det.title || det.name || id };
          if (!meta.description && det.overview) meta.description = det.overview;
          if (!meta.poster && det.poster_path) meta.poster = 'https://image.tmdb.org/t/p/w500' + det.poster_path;
          if (!meta.background && det.backdrop_path) meta.background = 'https://image.tmdb.org/t/p/w1280' + det.backdrop_path;
          if (!meta.genres && Array.isArray(det.genres)) meta.genres = det.genres.map(function (g) { return g.name; });
          if (!meta.imdbRating && det.vote_average) meta.imdbRating = Number(det.vote_average).toFixed(1);
          meta.tmdb_id = Number(tmdbNum);
          if (det.credits && Array.isArray(det.credits.cast) && (!meta.cast || !meta.cast.length)) {
            meta.cast = det.credits.cast.slice(0, 18).map(function (c) { return c.name; });
            meta.castMembers = det.credits.cast.slice(0, 18).map(function (c) {
              return {
                tmdb_id: c.id,
                name: c.name,
                character: c.character || '',
                photo: c.profile_path ? 'https://image.tmdb.org/t/p/w185' + c.profile_path : ''
              };
            });
          }
          if (mediaType !== 'movie' && (!meta.videos || !meta.videos.length) && Array.isArray(det.seasons)) {
            meta.videos = [];
          }
        } catch (_) {}
      }
      if (!meta) throw new Error('Detalhes indisponíveis para ' + id);
      return meta;
    },

    tmdb_season_episodes: async function (args) {
      var tmdbKey = String(loadSettings().tmdb_api_key || '').trim();
      if (!tmdbKey || !args.series || args.season == null) return [];
      var data = await fetchTmdbJson('/tv/' + args.series + '/season/' + args.season + '?language=pt-BR', tmdbKey);
      return (data && Array.isArray(data.episodes)) ? data.episodes.map(function (e) {
        return {
          season: e.season_number,
          episode: e.episode_number,
          title: e.name,
          overview: e.overview,
          released: e.air_date,
          thumbnail: e.still_path ? 'https://image.tmdb.org/t/p/w500' + e.still_path : ''
        };
      }) : [];
    },

    resolve_people_photos: async function () { return {}; },
    person_details: async function (args) {
      var tmdbKey = String(loadSettings().tmdb_api_key || '').trim();
      if (!tmdbKey || !args.personId) return null;
      return fetchTmdbJson('/person/' + args.personId + '?language=pt-BR&append_to_response=combined_credits', tmdbKey);
    },
    hero_ratings: async function () { return null; },
    tmdb_next_episodes: async function () { return []; },

    pull_account_library: function () {
      return storageGet('library_' + activeProfileSuffix(), []);
    },
    toggle_library_item: function (args) {
      var item = args.item || {};
      var key = 'library_' + activeProfileSuffix();
      var list = storageGet(key, []);
      var idx = list.findIndex(function (it) { return it.content_id === item.content_id && it.content_type === item.content_type; });
      if (idx >= 0) {
        list.splice(idx, 1);
        storageSet(key, list);
        return false;
      }
      list.unshift(Object.assign({}, item, { added_at: Date.now() }));
      storageSet(key, list);
      return true;
    },

    list_watched_keys: function () {
      return storageGet('watched_' + activeProfileSuffix(), []);
    },
    toggle_watched_item: function (args) {
      var k = watchedKey(args.contentId, args.season, args.episode);
      var sk = 'watched_' + activeProfileSuffix();
      var arr = storageGet(sk, []);
      var idx = arr.indexOf(k);
      if (idx >= 0) { arr.splice(idx, 1); storageSet(sk, arr); return false; }
      arr.push(k);
      storageSet(sk, arr);
      return true;
    },

    start_continue_watching: function () {
      var map = storageGet('progress_' + activeProfileSuffix(), {});
      var entries = Object.values(map).filter(function (e) { return e && e.duration_ms > 0; });
      entries.sort(function (a, b) { return (b.last_watched || 0) - (a.last_watched || 0); });
      entries.forEach(function (e, idx) {
        var pct = Math.max(1, Math.min(99, Math.round((e.position_ms || 0) * 100 / Math.max(1, e.duration_ms))));
        emit('home:continue-watching-item', {
          order: idx,
          card: {
            content_id: e.content_id,
            video_id: e.video_id,
            content_type: e.content_type,
            title: e.title || e.content_id,
            episode_title: e.episode_title || '',
            poster: e.poster || '',
            background: e.backdrop || '',
            artwork: e.backdrop || e.poster || '',
            season: e.season != null ? e.season : null,
            episode: e.episode != null ? e.episode : null,
            progress_percent: pct,
            position_ms: e.position_ms || 0,
            duration_ms: e.duration_ms || 0,
            last_watched: e.last_watched || Date.now()
          }
        });
      });
      return { total: entries.length };
    },
    upsert_watch_progress: function (args) {
      var entry = args.entry || {};
      var pkey = entry.progress_key || entry.video_id || entry.content_id;
      if (!pkey) return null;
      var sk = 'progress_' + activeProfileSuffix();
      var map = storageGet(sk, {});
      map[pkey] = Object.assign({}, entry, { progress_key: pkey });
      storageSet(sk, map);
      return null;
    },
    remove_watch_progress: function (args) {
      var sk = 'progress_' + activeProfileSuffix();
      var map = storageGet(sk, {});
      delete map[args.progressKey];
      storageSet(sk, map);
      return null;
    },
    clear_continue_watching_cache: function () {
      storageSet('progress_' + activeProfileSuffix(), {});
      return null;
    },

    list_followed_people: function () { return storageGet('followed_people_' + activeProfileSuffix(), []); },
    save_followed_people: function (args) { storageSet('followed_people_' + activeProfileSuffix(), args.people || []); return null; },
    pull_account_collections: function () { return []; },
    profile_insights: function () { return {}; },

    start_stream_search: function (args) {
      var addons = storageGet('addons_' + activeProfileSuffix(), []).filter(function (a) {
        return a.enabled && addonSupportsResource(a.manifest, 'stream', args.mediaType, args.id);
      });
      emit('streams:active_scrapers', []);
      Promise.all(addons.map(function (a, order) {
        var url = addonBaseUrl(a.manifest_url) + '/stream/' + encodeURIComponent(args.mediaType) + '/' + encodeURIComponent(args.id) + '.json';
        return netFetch(url, { timeoutMs: 15000 })
          .then(function (r) { return r.ok ? r.json() : null; })
          .then(function (data) {
            var streams = data && Array.isArray(data.streams) ? data.streams : [];
            if (streams.length) {
              emit('streams:addon', {
                order: order,
                addon_id: a.manifest.id || a.manifest_url,
                addon_name: a.manifest.name,
                addon_logo: a.manifest.logo || '',
                streams: streams
              });
            }
          })
          .catch(function () {});
      })).then(function () {
        emit('streams:done', {});
      });
      return { total: addons.length };
    },

    player_play: function (args) { return startWebosPlayer(args || {}); },
    player_stop: function () { stopWebosPlayer(); return null; },
    player_seek: function (args) { if (playerVideoEl) { playerVideoEl.currentTime = Math.max(0, Number(args.secs || 0)); emitPlayerState(); } return null; },
    player_set_paused: function (args) {
      if (!playerVideoEl) return null;
      if (args.paused) playerVideoEl.pause(); else playerVideoEl.play().catch(function () {});
      emitPlayerState();
      return null;
    },
    player_set_volume: function (args) {
      playerState.volume = Math.max(0, Math.min(100, Number(args.volume || 100)));
      if (playerVideoEl) playerVideoEl.volume = playerState.volume / 100;
      return null;
    },
    player_set_speed: function (args) {
      playerState.speed = Number(args.speed || 1);
      if (playerVideoEl) playerVideoEl.playbackRate = playerState.speed;
      return null;
    },
    player_set_audio: function (args) {
      if (playerVideoEl && playerVideoEl.audioTracks) {
        for (var i = 0; i < playerVideoEl.audioTracks.length; i++) {
          playerVideoEl.audioTracks[i].enabled = String(i) === String(args.aid);
        }
        emitPlayerState();
      }
      return null;
    },
    player_set_subtitle: function (args) {
      if (playerVideoEl && playerVideoEl.textTracks) {
        for (var i = 0; i < playerVideoEl.textTracks.length; i++) {
          playerVideoEl.textTracks[i].mode = String(i) === String(args.sid) ? 'showing' : 'disabled';
        }
        emitPlayerState();
      }
      return null;
    },
    player_set_subtitle_visible: function (args) {
      if (playerVideoEl && playerVideoEl.textTracks && !args.visible) {
        for (var i = 0; i < playerVideoEl.textTracks.length; i++) {
          playerVideoEl.textTracks[i].mode = 'disabled';
        }
        emitPlayerState();
      }
      return null;
    },
    player_add_subtitle_url: async function (args) {
      if (!playerVideoEl || !args || !args.url) return null;
      var res = await netFetch(args.url, { timeoutMs: 12000 });
      if (!res.ok) throw new Error('Falha ao baixar legenda');
      var vtt = srtToVtt(await res.text());
      var blobUrl = URL.createObjectURL(new Blob([vtt], { type: 'text/vtt' }));
      for (var i = 0; i < playerVideoEl.textTracks.length; i++) playerVideoEl.textTracks[i].mode = 'disabled';
      var tr = document.createElement('track');
      tr.kind = 'subtitles';
      tr.label = args.title || 'Externa';
      tr.srclang = args.lang || 'pt';
      tr.src = blobUrl;
      tr.default = true;
      playerVideoEl.appendChild(tr);
      externalSubTracks.push({ trackEl: tr, blobUrl: blobUrl });
      if (tr.track) tr.track.mode = 'showing';
      emitPlayerState();
      return null;
    },
    player_clear_external_subtitles: function () { clearExternalSubBlobs(); emitPlayerState(); return null; },
    player_fetch_addon_subtitles: async function (args) {
      var addons = storageGet('addons_' + activeProfileSuffix(), []).filter(function (a) {
        return a.enabled && addonSupportsResource(a.manifest, 'subtitles', args.mediaType, args.videoId);
      });
      var out = [];
      await Promise.all(addons.map(function (a) {
        var u = addonBaseUrl(a.manifest_url) + '/subtitles/' + encodeURIComponent(args.mediaType) + '/' + encodeURIComponent(args.videoId) + '.json';
        return netFetch(u, { timeoutMs: 10000 })
          .then(function (r) { return r.ok ? r.json() : null; })
          .then(function (d) {
            if (d && Array.isArray(d.subtitles)) {
              d.subtitles.forEach(function (s) {
                out.push(Object.assign({}, s, { addon_name: a.manifest.name }));
              });
            }
          }).catch(function () {});
      }));
      return out;
    },
    player_load_subtitle_cues: function () { return []; },
    player_set_resize_mode: function (args) {
      if (playerVideoEl) {
        var modes = ['contain', 'cover', 'fill'];
        playerVideoEl.style.objectFit = modes[Number(args.mode || 0) % modes.length] || 'contain';
      }
      return null;
    },
    player_set_bounds: function () { return null; },
    player_push_controls: function () { return null; },
    player_focus_overlay: function () { return null; },
    player_ui_message: function () { return null; },
    player_resolve_skip_intervals: function () { return []; },
    player_apply_subtitle_style: function () { return null; },
    player_set_subtitle_delay: function () { return null; },
    player_set_reference_track: function () { return null; },
    player_submit_intro: function () { return null; },

    mdblist_account_status: async function () {
      var key = String(loadSettings().services.mdblist_key || '').trim();
      if (!key) return { connected: false, username: null };
      var r = await netFetch('https://api.mdblist.com/user?apikey=' + encodeURIComponent(key));
      if (!r.ok) throw new Error('HTTP ' + r.status);
      var j = await r.json();
      return { connected: true, username: j.username || j.name || null };
    },
    mdblist_library_sync: async function () { return { configured: false, lists: [], items: [] }; },
    mdblist_library_mutate: async function () { return null; },
    mdblist_progress_sync: async function () { return { watched_keys: [], continue_watching: [] }; },
    mdblist_watched_mutate: async function () { return null; },
    mdblist_scrobble: async function () { return null; },

    simkl_status: function () { return { connected: false, sources: {} }; },
    simkl_action: function () { return {}; },

    list_plugins_state: function () { return { plugins_enabled: false, group_by_repo: false, repositories: [], scrapers: [] }; },
    set_plugins_enabled: function () { return { plugins_enabled: false, group_by_repo: false, repositories: [], scrapers: [] }; },

    connected_accounts: function () { return { accounts: [] }; },
    discord_presence: function () { return { active: false }; },
    launch_mpv: function () { throw new Error('Player externo não disponível na TV.'); },
    party_status: function () { return { active: false }; }
  };

  async function invoke(cmd, args) {
    await ensureKvHydrated();
    var fn = handlers[cmd];
    if (fn) return await fn(args || {});
    console.warn('[nuvio-webos] comando não implementado:', cmd);
    return null;
  }

  window.__TAURI__ = {
    core: { invoke: invoke },
    event: { listen: listen, emit: emit }
  };
})();

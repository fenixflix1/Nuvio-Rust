// PC Widescreen Active Library & Taste Insights dashboard for the active profile.
const insightsUi = {
  subTab: 'overview',
  typeFilter: 'all',
  genreFilter: '',
  sort: 'recent',
  loading: false,
  data: null,
};
const insightsMetaCache = new Map();

function normalizeGenreList(raw) {
  if (!Array.isArray(raw)) return [];
  return raw
    .map((g) => String(g || '').trim())
    .filter(Boolean);
}

function parseReleaseYear(item) {
  const raw = String(item.release_info || item.releaseInfo || '').trim();
  const m = raw.match(/\b(19\d{2}|20\d{2})\b/);
  return m ? Number(m[1]) : null;
}

function decadeLabel(year) {
  if (!year || year < 1920 || year > 2100) return '';
  if (year < 1990) return 'Clássicos (<90)';
  return `${Math.floor(year / 10) * 10}s`;
}

function resolveItemTitle(entry, libraryById, cwById) {
  if (entry.title && String(entry.title).trim()) return String(entry.title).trim();
  const id = entry.content_id || entry.id;
  const cached = insightsMetaCache.get(id);
  if (cached?.name) return cached.name;
  const cw = cwById.get(id);
  if (cw?.title) return cw.title;
  const lib = libraryById.get(id);
  if (lib?.name || lib?.title) return lib.name || lib.title;
  return id || 'Sem título';
}

function isLiveTvOrChannel(contentType, contentId, title) {
  const t = String(contentType || '').trim().toLowerCase();
  const id = String(contentId || '').trim().toLowerCase();
  const name = String(title || '').trim().toLowerCase();
  if (t === 'channel' || t === 'live' || t === 'iptv' || t === 'radio') return true;
  if (id.startsWith('tv:') || id.startsWith('cs:channel:') || id.startsWith('channel:') || id.startsWith('event:') || id.startsWith('iptv:')) {
    return true;
  }
  if (t === 'tv' && !id.startsWith('tt') && !id.startsWith('tmdb:') && !id.startsWith('kitsu:')) {
    return true;
  }
  return [id, name].some((v) => (
    v.startsWith('http://') || v.startsWith('https://') || v.startsWith('rtmp://') || v.startsWith('rtsp://') || v.endsWith('.m3u') || v.endsWith('.m3u8')
  ));
}

function normalizeInsightKind(rawType) {
  const t = String(rawType || '').trim().toLowerCase();
  if (t === 'movie' || t === 'film') return 'movie';
  if (t === 'series' || t === 'tv' || t === 'show' || t === 'tvshow' || t === 'anime') return 'series';
  return '';
}

function formatWatchTimeLabel(totalMs) {
  const mins = Math.max(0, Math.round((Number(totalMs) || 0) / 60000));
  if (mins < 60) return `${mins} min`;
  const totalHours = Math.floor(mins / 60);
  if (totalHours < 24) return `${totalHours} h`;
  const days = Math.floor(totalHours / 24);
  const remHours = totalHours % 24;
  return remHours > 0 ? `${days} d ${remHours} h` : `${days} d`;
}

// Mobile parity: runtime strings such as "2h 15m", "135 min" or "135".
function parseRuntimeMinutes(value) {
  const raw = String(value ?? '').trim();
  if (!raw) return 0;
  const hours = raw.match(/(\d+)\s*h/i);
  const mins = raw.match(/(\d+)\s*m/i);
  if (hours || mins) return (hours ? Number(hours[1]) * 60 : 0) + (mins ? Number(mins[1]) : 0);
  const digits = raw.match(/^(\d+)$/);
  return digits ? Number(digits[1]) : 0;
}

// NuvioMobile-Enhanced fallbacks (ProfileInsightsFallbackMovieMinutes/EpisodeMinutes).
const FALLBACK_MOVIE_MINUTES = 115;
const FALLBACK_EPISODE_MINUTES = 42;

function profileFallbackDurationMs(kind, isEpisode) {
  if (kind === 'movie' && !isEpisode) return FALLBACK_MOVIE_MINUTES * 60000;
  if (kind === 'series' && isEpisode) return FALLBACK_EPISODE_MINUTES * 60000;
  return 0;
}

function progressPercentOf(entry) {
  const pct = Number(entry.progress_percent ?? entry.progressPercent);
  return Number.isFinite(pct) ? Math.min(100, Math.max(0, pct)) : null;
}

// WatchProgressEntry.progressFraction: explicit percent first, then position/duration.
function progressFractionOf(entry) {
  const pct = progressPercentOf(entry);
  if (pct !== null) return Math.min(1, pct / 100);
  const dur = Math.max(0, Number(entry.duration) || 0);
  const pos = Math.max(0, Number(entry.position) || 0);
  return dur > 0 ? Math.min(1, pos / dur) : 0;
}

// WatchProgressEntry.isEffectivelyCompleted: 90% by explicit percent or position/duration.
function isProgressCompleted(entry) {
  const pct = progressPercentOf(entry);
  if (pct !== null && pct >= 90) return true;
  const dur = Math.max(0, Number(entry.duration) || 0);
  const pos = Math.max(0, Number(entry.position) || 0);
  return dur > 0 && pos / dur >= 0.90;
}

// Mobile parity: effective duration = real duration, else the 115/42 fallback.
function trackedProgressMs(entry, kind, isEpisode) {
  const dur = Math.max(0, Number(entry.duration) || 0);
  const effective = dur > 0 ? dur : profileFallbackDurationMs(kind, isEpisode);
  const pos = Math.max(0, Number(entry.position) || 0);
  if (effective <= 0) return pos;
  if (isProgressCompleted(entry)) return effective;
  if (pos > 0) return Math.min(pos, effective);
  const pct = progressPercentOf(entry);
  if (pct === null || pct <= 0) return 0;
  return Math.min(effective, Math.round((effective * pct) / 100));
}

function computeProfileInsights(raw) {
  const activeProfile = (authState?.profiles || []).find((p) => p.profile_index === authState?.active_profile_index)
    || (authState?.profiles || [])[0]
    || { name: 'Local', avatar_color_hex: '#1E88E5' };

  const simklLib = typeof simklUses === 'function' && simklUses('library') && typeof simklLibrary === 'function'
    ? simklLibrary()
    : null;
  const library = (Array.isArray(simklLib) && simklLib.length ? simklLib : (raw?.library || []))
    .filter((it) => !isLiveTvOrChannel(it.content_type || it.type, it.content_id || it.id, it.name || it.title))
    .map((it) => ({
      id: it.content_id || it.id || '',
      type: String(it.content_type || it.type || 'movie').toLowerCase() === 'movie' ? 'movie' : 'series',
      name: it.name || it.title || it.content_id || 'Sem título',
      poster: it.poster || '',
      background: it.background || it.banner || '',
      logo: it.logo || '',
      description: it.description || '',
      release_info: it.release_info || it.releaseInfo || '',
      imdb_rating: Number(it.imdb_rating ?? it.imdbRating) || 0,
      genres: normalizeGenreList(it.genres),
      added_at: Number(it.added_at) || Date.parse(it.created_at || '') || 0,
    }));

  const libraryById = new Map(library.map((it) => [it.id, it]));
  const cwById = new Map((cwCardsCache || []).map((c) => [c.content_id, c]));

  const watched = (Array.isArray(raw?.watched_items) ? raw.watched_items : [])
    .filter((w) => !isLiveTvOrChannel(w.content_type, w.content_id, w.title));
  const progress = (Array.isArray(raw?.watch_progress) ? raw.watch_progress : [])
    .filter((p) => !isLiveTvOrChannel(p.content_type, p.content_id, p.title));
  const followed = Array.isArray(raw?.followed_people) ? raw.followed_people : (followedPeopleList || []);

  const movieCount = library.filter((it) => it.type === 'movie').length;
  const seriesCount = library.length - movieCount;

  // Match NuvioMobile-Enhanced stats: deduplicated continue watching (<90%), watched movies, completed & ongoing series,
  // and watch time from real durations merged with watched refs (fallback 115/42 when the length is unknown).
  const seenProgressKeys = new Set();
  const continueGroupKeys = new Set();
  const continueEntries = [];
  const progressDurationByKey = new Map();
  const watchedDurationByKey = new Map();
  const engagedSeriesEpisodes = new Map();
  const completedProgressMovies = new Set();
  const activityKey = (kind, cid, season, episode) => `${kind}:${cid}:${season ?? -1}:${episode ?? -1}`;

  for (const p of progress) {
    const cid = String(p.content_id || '').trim();
    const kind = normalizeInsightKind(p.content_type);
    if (!cid || !kind) continue;
    const dedupeKey = `${p.content_type || ''}|${cid}|${p.video_id || ''}|${p.season ?? ''}|${p.episode ?? ''}`;
    if (seenProgressKeys.has(dedupeKey)) continue;
    seenProgressKeys.add(dedupeKey);

    const fraction = progressFractionOf(p);
    const effectivelyDone = isProgressCompleted(p);
    // Mobile parity: resume list keeps entries from 2% up, one per series.
    if (!effectivelyDone && fraction >= 0.02) {
      const groupKey = kind === 'series' ? `series:${cid}` : `movie:${cid}:${p.video_id || cid}`;
      if (!continueGroupKeys.has(groupKey)) {
        continueGroupKeys.add(groupKey);
        continueEntries.push(p);
      }
    }

    const sea = p.season ?? null;
    const ep = p.episode ?? null;
    if (kind === 'movie' || (kind === 'series' && sea != null && ep != null)) {
      const ak = activityKey(kind, cid, sea, ep);
      const tracked = trackedProgressMs(p, kind, sea != null && ep != null);
      if (tracked > 0 && tracked > (progressDurationByKey.get(ak) || 0)) {
        progressDurationByKey.set(ak, tracked);
      }
      if (kind === 'movie' && effectivelyDone) completedProgressMovies.add(cid);
      if (kind === 'series' && sea != null && ep != null && tracked > 0) {
        if (!engagedSeriesEpisodes.has(cid)) engagedSeriesEpisodes.set(cid, new Set());
        engagedSeriesEpisodes.get(cid).add(`${sea}:${ep}`);
      }
    }
  }

  const watchedMovieIds = new Set();
  const completedSeriesIds = new Set();
  const distinctEpisodes = new Set();
  const watchedActivityKeys = new Set();

  for (const w of watched) {
    const cid = String(w.content_id || '').trim();
    const rawType = String(w.content_type || '').trim().toLowerCase();
    const kind = normalizeInsightKind(rawType);
    if (!cid || !kind) continue;
    const sea = w.season ?? null;
    const ep = w.episode ?? null;
    // Mobile parity: one watched ref per activity, duration from the cached runtime otherwise the fallback.
    const ak0 = activityKey(kind, cid, sea, ep);
    if (watchedActivityKeys.has(ak0)) continue;
    watchedActivityKeys.add(ak0);
    if (kind === 'movie' && sea == null && ep == null) {
      watchedMovieIds.add(cid);
      const runtimeMin = parseRuntimeMinutes(insightsMetaCache.get(cid)?.runtime) || FALLBACK_MOVIE_MINUTES;
      const ms = Math.round(runtimeMin * 60000);
      if (ms > (watchedDurationByKey.get(ak0) || 0)) watchedDurationByKey.set(ak0, ms);
    } else if (kind === 'series') {
      if (sea != null && ep != null) {
        distinctEpisodes.add(`${cid}:${sea}:${ep}`);
        if (!engagedSeriesEpisodes.has(cid)) engagedSeriesEpisodes.set(cid, new Set());
        engagedSeriesEpisodes.get(cid).add(`${sea}:${ep}`);
        const ms = FALLBACK_EPISODE_MINUTES * 60000;
        if (ms > (watchedDurationByKey.get(ak0) || 0)) watchedDurationByKey.set(ak0, ms);
      } else if (rawType !== 'tv') {
        completedSeriesIds.add(cid);
      }
    }
  }

  const ongoingSeriesCount = [...engagedSeriesEpisodes.keys()].filter((id) => !completedSeriesIds.has(id)).length;
  // Same merge as profileWatchTimeSplit: one duration per activity key, the larger wins.
  const durationByKey = new Map(progressDurationByKey);
  for (const [key, ms] of watchedDurationByKey) {
    if (ms > (durationByKey.get(key) || 0)) durationByKey.set(key, ms);
  }
  const totalWatchTimeMs = [...durationByKey.values()].reduce((acc, v) => acc + v, 0);

  // Movie vs Series balance matches buildProfileTypeBalance: unique watched movies vs engaged series
  // (completed titles or series with at least 2 episodes watched). Library items are not counted here.
  const engagedSeriesIds = new Set([
    ...completedSeriesIds,
    ...[...engagedSeriesEpisodes.entries()].filter(([, eps]) => eps.size >= 2).map(([id]) => id),
  ]);
  const balanceMovies = new Set([...watchedMovieIds, ...completedProgressMovies]).size;
  const balanceSeries = engagedSeriesIds.size;
  const balanceSum = balanceMovies + balanceSeries;
  const moviePct = balanceSum ? Math.round((balanceMovies / balanceSum) * 100) : 50;
  const seriesPct = balanceSum ? 100 - moviePct : 50;

  const watchedMovies = watchedMovieIds.size;
  const watchedEpisodes = distinctEpisodes.size;

  const ratedItems = library.filter((it) => it.imdb_rating > 0);
  const avgRating = ratedItems.length
    ? ratedItems.reduce((acc, it) => acc + it.imdb_rating, 0) / ratedItems.length
    : 0;

  // Genre histogram weighted by library items
  const genreMap = new Map();
  for (const it of library) {
    for (const g of it.genres) {
      genreMap.set(g, (genreMap.get(g) || 0) + 1);
    }
  }
  const topGenres = [...genreMap.entries()]
    .sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]))
    .slice(0, 8)
    .map(([name, count]) => ({
      name,
      count,
      pct: library.length ? Math.round((count / library.length) * 100) : 0,
    }));

  // Decade distribution
  const decadeBuckets = ['Clássicos (<90)', '1990s', '2000s', '2010s', '2020s'];
  const decadeCounts = new Map(decadeBuckets.map((k) => [k, 0]));
  for (const it of library) {
    const label = decadeLabel(parseReleaseYear(it));
    if (label) decadeCounts.set(label, (decadeCounts.get(label) || 0) + 1);
  }
  const decades = decadeBuckets.map((label) => ({ label, count: decadeCounts.get(label) || 0 }));
  const dominantDecade = [...decades].sort((a, b) => b.count - a.count)[0];

  // Rating tier distribution
  const ratingTiers = [
    { label: '8.0+ Excelente', count: ratedItems.filter((it) => it.imdb_rating >= 8).length },
    { label: '7.0–7.9 Ótimo', count: ratedItems.filter((it) => it.imdb_rating >= 7 && it.imdb_rating < 8).length },
    { label: '6.0–6.9 Regular', count: ratedItems.filter((it) => it.imdb_rating >= 6 && it.imdb_rating < 7).length },
    { label: '<6.0 Abaixo de 6', count: ratedItems.filter((it) => it.imdb_rating < 6).length },
  ];

  // 6-month activity timeline from watched_at, last_watched, added_at
  const now = new Date();
  const months = [];
  for (let i = 5; i >= 0; i -= 1) {
    const d = new Date(now.getFullYear(), now.getMonth() - i, 1);
    months.push({
      key: `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`,
      label: d.toLocaleDateString('pt-BR', { month: 'short' }).replace('.', ''),
      count: 0,
    });
  }
  const monthIdx = new Map(months.map((m, idx) => [m.key, idx]));
  const addTimestamp = (ts) => {
    const n = Number(ts) || 0;
    if (n <= 0) return;
    const ms = n < 1e11 ? n * 1000 : n;
    const d = new Date(ms);
    const k = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`;
    const idx = monthIdx.get(k);
    if (idx !== undefined) months[idx].count += 1;
  };
  library.forEach((it) => addTimestamp(it.added_at));
  watched.forEach((w) => addTimestamp(w.watched_at));
  progress.forEach((p) => addTimestamp(p.last_watched));

  // Recent 7-day activity count + DNA chips matching AKRusso/NuvioMobile-Enhanced
  const recentCutoffMs = Date.now() - 7 * 24 * 60 * 60 * 1000;
  const recentActivitySet = new Set();
  for (const w of watched) {
    const ts = Number(w.watched_at) || 0;
    const ms = ts > 0 && ts < 1e11 ? ts * 1000 : ts;
    if (ms >= recentCutoffMs && w.content_id) {
      recentActivitySet.add(`${w.content_id}:${w.season ?? -1}:${w.episode ?? -1}`);
    }
  }
  for (const p of progress) {
    const ts = Number(p.last_watched) || 0;
    const ms = ts > 0 && ts < 1e11 ? ts * 1000 : ts;
    if (ms >= recentCutoffMs && p.content_id) {
      recentActivitySet.add(`${p.content_id}:${p.season ?? -1}:${p.episode ?? -1}`);
    }
  }
  const recentActivityCount = recentActivitySet.size;

  const dnaChips = [];
  if (balanceSum === 0) dnaChips.push('Aprendendo gosto');
  else if (moviePct >= 62) dnaChips.push('Foco em Filmes');
  else if (seriesPct >= 62) dnaChips.push('Foco em Séries');
  else dnaChips.push('Equilibrado');
  if (continueEntries.length >= 3) dnaChips.push('Maratonista');
  if (recentActivityCount >= 5) dnaChips.push('Alta Atividade');
  if (library.length >= 12) dnaChips.push('Colecionador');
  if (watchedMovieIds.size + completedSeriesIds.size >= 10) dnaChips.push('Completacionista');

  const formatProgressRow = (p) => {
    const dur = Math.max(1, Number(p.duration) || 0);
    const pos = Math.max(0, Number(p.position) || 0);
    const cw = cwById.get(p.content_id);
    const lib = libraryById.get(p.content_id);
    const cached = insightsMetaCache.get(p.content_id);
    return {
      id: p.content_id,
      type: p.content_type || cw?.content_type || cached?.type || 'movie',
      title: resolveItemTitle(p, libraryById, cwById),
      subtitle: p.season != null && p.episode != null
        ? `T${p.season} E${p.episode}`
        : (lib?.release_info || cached?.release_info || 'Filme'),
      poster: cw?.poster || lib?.poster || cached?.poster || '',
      background: cw?.background || lib?.background || cached?.background || '',
      pct: Math.min(99, Math.max(1, Math.round((pos / dur) * 100))),
      remMin: dur > pos ? Math.max(1, Math.round((dur - pos) / 60000)) : null,
    };
  };

  const smartResumeEntry = [...continueEntries]
    .filter((p) => {
      const frac = progressFractionOf(p);
      return frac >= 0.05 && frac <= 0.92;
    })
    .sort((a, b) => (Number(b.last_watched) || 0) - (Number(a.last_watched) || 0))[0] || null;
  const smartResume = smartResumeEntry ? formatProgressRow(smartResumeEntry) : null;

  // DNA Taste Archetype headline
  const leadGenres = topGenres.slice(0, 2).map((g) => g.name).join(' & ');
  let formatArchetype = 'Equilíbrio Filmes & Séries';
  if (moviePct >= 65) formatArchetype = 'Perfil Cinéfilo';
  else if (seriesPct >= 65 || watchedEpisodes >= 20) formatArchetype = 'Maratonista de Séries';
  const tasteHeadline = leadGenres ? `${leadGenres} · ${formatArchetype}` : formatArchetype;

  const activeProgressRows = [...continueEntries]
    .filter((p) => p !== smartResumeEntry)
    .sort((a, b) => (Number(b.last_watched) || 0) - (Number(a.last_watched) || 0))
    .slice(0, 5)
    .map(formatProgressRow);

  const topRated = [...ratedItems]
    .sort((a, b) => b.imdb_rating - a.imdb_rating || b.added_at - a.added_at)
    .slice(0, 5);

  return {
    activeProfile,
    tasteHeadline,
    library,
    movieCount,
    seriesCount,
    balanceMovies,
    balanceSeries,
    moviePct,
    seriesPct,
    watchedMovies,
    watchedEpisodes,
    completedSeriesCount: completedSeriesIds.size,
    ongoingSeriesCount,
    continueCount: continueEntries.length,
    watchTimeLabel: formatWatchTimeLabel(totalWatchTimeMs),
    watchTimeHours: Math.round(totalWatchTimeMs / 3600000),
    recentActivityCount,
    dnaChips: dnaChips.slice(0, 4),
    smartResume,
    avgRating: avgRating ? avgRating.toFixed(1) : '—',
    ratedCount: ratedItems.length,
    followed,
    topGenres,
    decades,
    dominantDecade: dominantDecade?.count ? dominantDecade.label : 'Variada',
    ratingTiers,
    months,
    activeProgressRows,
    topRated,
  };
}

function openItemFromInsights(item) {
  if (!item?.id) return;
  openDetailsPage({
    id: item.id,
    type: item.type || 'movie',
    name: item.name || item.title || item.id,
    poster: item.poster || '',
    banner: item.background || '',
    logo: item.logo || '',
    description: item.description || '',
    release_info: item.release_info || '',
    imdb_rating: item.imdb_rating || null,
  });
}

function renderInsightsDashboard() {
  const shell = document.getElementById('insights-shell');
  if (!shell) return;
  if (insightsUi.loading && !insightsUi.data) {
    shell.innerHTML = `<div class="insights-loading">Analisando biblioteca ativa e histórico de gosto…</div>`;
    return;
  }
  const m = insightsUi.data || computeProfileInsights({});
  const profName = m.activeProfile?.name || 'Perfil';
  const profColor = m.activeProfile?.avatar_color_hex || '#1E88E5';
  const profBg = m.activeProfile?.profile_background_url || m.library.find((x) => x.background)?.background || '';

  const filteredLibrary = m.library.filter((it) => {
    if (insightsUi.typeFilter !== 'all' && it.type !== insightsUi.typeFilter) return false;
    if (insightsUi.genreFilter && !it.genres.includes(insightsUi.genreFilter)) return false;
    return true;
  }).sort((a, b) => {
    if (insightsUi.sort === 'rating') return b.imdb_rating - a.imdb_rating || a.name.localeCompare(b.name);
    if (insightsUi.sort === 'title') return a.name.localeCompare(b.name);
    return b.added_at - a.added_at;
  });

  const maxGenre = Math.max(1, ...(m.topGenres.map((g) => g.count)));
  const maxDecade = Math.max(1, ...(m.decades.map((d) => d.count)));
  const maxMonth = Math.max(1, ...(m.months.map((x) => x.count)));
  const maxTier = Math.max(1, ...(m.ratingTiers.map((t) => t.count)));

  shell.innerHTML = `
    <div class="insights-topbar">
      <div class="row-gap">
        <button class="circle-back-btn" id="btn-insights-back" title="Voltar" type="button">
          <svg viewBox="0 0 24 24" width="20" height="20"><path fill="currentColor" d="M15.41 7.41L14 6l-6 6l6 6l1.41-1.41L10.83 12z"/></svg>
        </button>
        <div>
          <h2 class="insights-page-title">Insights do Perfil</h2>
          <p class="muted" style="margin:2px 0 0;font-size:13px;">Biblioteca ativa e análise de gosto para desktop</p>
        </div>
      </div>
      <div class="insights-topbar-actions">
        <div class="insights-subtabs" role="tablist">
          <button class="library-source-chip ${insightsUi.subTab === 'overview' ? 'active' : ''}" data-insights-sub="overview" type="button">Visão Geral</button>
          <button class="library-source-chip ${insightsUi.subTab === 'taste' ? 'active' : ''}" data-insights-sub="taste" type="button">Perfil de Gosto</button>
        </div>
        <button class="btn-ghost nv-mini-btn" id="btn-insights-refresh" type="button">${insightsUi.loading ? 'Atualizando…' : 'Atualizar'}</button>
        <button class="btn-accent nv-mini-btn" id="btn-insights-library" type="button">Abrir Biblioteca</button>
      </div>
    </div>

    <div class="insights-hero">
      ${profBg ? `<div class="insights-hero-bg" style="background-image:url('${esc(profBg)}')"></div>` : ''}
      <div class="insights-hero-content">
        <div class="insights-profile-id">
          <div class="insights-avatar" style="background:${esc(profColor)}">
            <span>${esc(profName.charAt(0).toUpperCase())}</span>
            ${m.activeProfile?.avatar_url
              ? `<img src="${esc(m.activeProfile.avatar_url)}" alt="" onerror="this.remove()" />`
              : ''}
          </div>
          <div>
            <div class="insights-dna-pill">DNA DE GOSTO · ${esc(profName)}</div>
            <h1 class="insights-dna-headline">${esc(m.tasteHeadline)}</h1>
            <div class="insights-hero-tags">
              ${m.dnaChips.map((chip) => `<span class="insights-dna-chip">${esc(chip)}</span>`).join('')}
              <span>${m.recentActivityCount} atividades (7d)</span>
              <span>Época: ${esc(m.dominantDecade)}</span>
              <span>Nota média: ${esc(m.avgRating)}${m.ratedCount ? ' ★' : ''}</span>
            </div>
          </div>
        </div>
        <div class="insights-mix-box">
          <div class="insights-mix-head">
            <span title="Base: ${m.balanceMovies} filmes e ${m.balanceSeries} séries com atividade">Filmes <strong>${m.moviePct}%</strong></span>
            <span title="Base: ${m.balanceMovies} filmes e ${m.balanceSeries} séries com atividade">Séries <strong>${m.seriesPct}%</strong></span>
          </div>
          <div class="insights-mix-bar">
            <div class="insights-mix-movies" style="width:${m.moviePct}%"></div>
            <div class="insights-mix-series" style="width:${m.seriesPct}%"></div>
          </div>
        </div>
      </div>
    </div>

    ${insightsUi.subTab === 'overview' ? `
      <div class="insights-kpi-grid">
        <div class="insights-kpi-card">
          <span class="insights-kpi-label">Assistidos &amp; Concluídas</span>
          <strong class="insights-kpi-val">${m.watchedMovies + m.completedSeriesCount}</strong>
          <span class="insights-kpi-sub">${m.watchedMovies} filmes · ${m.completedSeriesCount} concluídas · ${m.watchedEpisodes} episódios</span>
        </div>
        <div class="insights-kpi-card">
          <span class="insights-kpi-label">Tempo Assistido</span>
          <strong class="insights-kpi-val">${esc(m.watchTimeLabel)}</strong>
          <span class="insights-kpi-sub">${m.watchTimeHours} h totais · ${m.ongoingSeriesCount} séries em andamento</span>
        </div>
        <div class="insights-kpi-card">
          <span class="insights-kpi-label">Assistindo Agora</span>
          <strong class="insights-kpi-val">${m.continueCount}</strong>
          <span class="insights-kpi-sub">${m.library.length} salvos na biblioteca (${m.movieCount}F / ${m.seriesCount}S)</span>
        </div>
        <div class="insights-kpi-card">
          <span class="insights-kpi-label">Curadoria &amp; Seguindo</span>
          <strong class="insights-kpi-val">${m.avgRating}${m.ratedCount ? ' ★' : ''}</strong>
          <span class="insights-kpi-sub">${m.ratedCount} avaliados · ${m.followed.length} pessoas seguidas</span>
        </div>
      </div>

      <div class="insights-bento-2col">
        <div class="insights-card">
          <div class="insights-card-head">
            <h3>Em progresso &amp; ritmo recente</h3>
            <span class="muted">Últimos 6 meses</span>
          </div>
          <div class="insights-sparkline">
            ${m.months.map((mo) => {
              const h = mo.count ? Math.max(12, Math.round((mo.count / maxMonth) * 72)) : 6;
              return `
                <div class="insights-spark-col" title="${esc(mo.label)}: ${mo.count} registros">
                  <span class="insights-spark-num">${mo.count || ''}</span>
                  <div class="insights-spark-bar-wrap"><div class="insights-spark-bar ${mo.count ? 'filled' : ''}" style="height:${h}px"></div></div>
                  <span class="insights-spark-lbl">${esc(mo.label)}</span>
                </div>
              `;
            }).join('')}
          </div>
          <div class="insights-subhead">Continuando agora</div>
          ${m.smartResume ? `
            <button class="insights-smart-resume" id="btn-insights-smart-resume" type="button">
              <div class="insights-mini-poster">${m.smartResume.poster ? `<img src="${esc(m.smartResume.poster)}" alt="" />` : ''}</div>
              <div class="insights-progress-copy">
                <div class="insights-smart-kicker">RETOMAR AGORA</div>
                <div class="insights-progress-title">${esc(m.smartResume.title)}</div>
                <div class="insights-progress-meta">${esc(m.smartResume.subtitle)}${m.smartResume.remMin ? ` · faltam ${m.smartResume.remMin} min` : ''}</div>
                <div class="insights-progress-track"><div style="width:${m.smartResume.pct}%"></div></div>
              </div>
              <span class="insights-smart-pct">${m.smartResume.pct}%</span>
            </button>
          ` : ''}
          ${m.activeProgressRows.length ? `
            <div class="insights-progress-list">
              ${m.activeProgressRows.map((row, idx) => `
                <button class="insights-progress-row" data-insights-progress="${idx}" type="button">
                  <div class="insights-mini-poster">${row.poster ? `<img src="${esc(row.poster)}" alt="" />` : ''}</div>
                  <div class="insights-progress-copy">
                    <div class="insights-progress-title">${esc(row.title)}</div>
                    <div class="insights-progress-meta">${esc(row.subtitle)}${row.remMin ? ` · faltam ${row.remMin} min` : ''}</div>
                    <div class="insights-progress-track"><div style="width:${row.pct}%"></div></div>
                  </div>
                  <span class="insights-pct">${row.pct}%</span>
                </button>
              `).join('')}
            </div>
          ` : `<p class="muted" style="margin:8px 0 0;">Nenhum título em andamento no momento.</p>`}
        </div>

        <div class="insights-card">
          <div class="insights-card-head">
            <h3>Destaques mais bem avaliados</h3>
            <span class="muted">Na sua biblioteca</span>
          </div>
          ${m.topRated.length ? `
            <div class="insights-top-list">
              ${m.topRated.map((item, idx) => `
                <button class="insights-top-row" data-insights-top="${idx}" type="button">
                  <span class="insights-rank">#${idx + 1}</span>
                  <div class="insights-mini-poster">${item.poster ? `<img src="${esc(item.poster)}" alt="" />` : ''}</div>
                  <div class="insights-top-copy">
                    <div class="insights-top-title">${esc(item.name)}</div>
                    <div class="insights-top-meta">${esc(item.genres.slice(0, 3).join(' · ') || (item.type === 'movie' ? 'Filme' : 'Série'))}${item.release_info ? ` · ${esc(item.release_info)}` : ''}</div>
                  </div>
                  <span class="pp-rating">★ ${item.imdb_rating.toFixed(1)}</span>
                </button>
              `).join('')}
            </div>
          ` : `<p class="muted">Adicione títulos com nota IMDb à biblioteca para ver os destaques.</p>`}
          <div class="insights-subhead" style="margin-top:18px;">Pessoas seguidas (${m.followed.length})</div>
          ${m.followed.length ? `
            <div class="insights-people-chips">
              ${m.followed.map((p, idx) => `
                <button class="insights-person-chip" data-insights-person="${idx}" type="button">
                  <span class="insights-person-dot">${p.photo ? `<img src="${esc(p.photo)}" alt="" />` : esc((p.name || '?').charAt(0).toUpperCase())}</span>
                  <span>${esc(p.name)}</span>
                </button>
              `).join('')}
            </div>
          ` : `<p class="muted" style="margin:4px 0 0;">Siga atores e diretores na tela de detalhes.</p>`}
        </div>
      </div>
    ` : `
      <div class="insights-bento-2col">
        <div class="insights-card">
          <div class="insights-card-head">
            <h3>Gêneros dominantes</h3>
            <span class="muted">Clique para filtrar abaixo</span>
          </div>
          ${m.topGenres.length ? `
            <div class="insights-bars">
              ${m.topGenres.map((g) => {
                const w = Math.max(8, Math.round((g.count / maxGenre) * 100));
                const active = insightsUi.genreFilter === g.name;
                return `
                  <button class="insights-bar-row ${active ? 'active' : ''}" data-insights-genre="${esc(g.name)}" type="button">
                    <span class="insights-bar-name">${esc(g.name)}</span>
                    <span class="insights-bar-track"><span class="insights-bar-fill" style="width:${w}%"></span></span>
                    <span class="insights-bar-val">${g.count} (${g.pct}%)</span>
                  </button>
                `;
              }).join('')}
            </div>
          ` : `<p class="muted">Sem gêneros suficientes nos itens salvos.</p>`}
        </div>

        <div class="insights-card">
          <div class="insights-card-head">
            <h3>Décadas &amp; notas</h3>
            <span class="muted">${m.ratedCount} títulos avaliados</span>
          </div>
          <div class="insights-subhead" style="margin-top:0;">Por década de lançamento</div>
          <div class="insights-decades">
            ${m.decades.map((d) => {
              const h = d.count ? Math.max(10, Math.round((d.count / maxDecade) * 64)) : 4;
              return `
                <div class="insights-decade-col">
                  <span class="insights-spark-num">${d.count || 0}</span>
                  <div class="insights-spark-bar-wrap"><div class="insights-spark-bar ${d.count ? 'filled' : ''}" style="height:${h}px"></div></div>
                  <span class="insights-spark-lbl">${esc(d.label)}</span>
                </div>
              `;
            }).join('')}
          </div>
          <div class="insights-subhead">Distribuição por nota IMDb</div>
          <div class="insights-tiers">
            ${m.ratingTiers.map((t) => {
              const w = t.count ? Math.max(6, Math.round((t.count / maxTier) * 100)) : 0;
              return `
                <div class="insights-tier-row">
                  <span>${esc(t.label)}</span>
                  <span class="insights-bar-track"><span class="insights-bar-fill" style="width:${w}%"></span></span>
                  <span class="insights-bar-val">${t.count}</span>
                </div>
              `;
            }).join('')}
          </div>
        </div>
      </div>
    `}

    <div class="insights-card insights-library-explorer">
      <div class="insights-explorer-head">
        <div>
          <h3 style="margin:0;">Biblioteca Ativa do Perfil</h3>
          <span class="muted" style="font-size:13px;">${filteredLibrary.length} de ${m.library.length} títulos</span>
        </div>
        <div class="row-gap" style="flex-wrap:wrap;">
          ${insightsUi.genreFilter ? `<button class="library-source-chip active" id="btn-insights-clear-genre" type="button">Gênero: ${esc(insightsUi.genreFilter)} ×</button>` : ''}
          <button class="library-source-chip ${insightsUi.typeFilter === 'all' ? 'active' : ''}" data-insights-type="all" type="button">Todos</button>
          <button class="library-source-chip ${insightsUi.typeFilter === 'movie' ? 'active' : ''}" data-insights-type="movie" type="button">Filmes</button>
          <button class="library-source-chip ${insightsUi.typeFilter === 'series' ? 'active' : ''}" data-insights-type="series" type="button">Séries</button>
          <select id="insights-sort-select" class="nuvio-chip-select">
            <option value="recent" ${insightsUi.sort === 'recent' ? 'selected' : ''}>Adicionados recentemente</option>
            <option value="rating" ${insightsUi.sort === 'rating' ? 'selected' : ''}>Maior nota</option>
            <option value="title" ${insightsUi.sort === 'title' ? 'selected' : ''}>Nome (A–Z)</option>
          </select>
        </div>
      </div>
      ${filteredLibrary.length ? `
        <div class="poster-grid insights-grid">
          ${filteredLibrary.map((it, idx) => `
            <button class="pp-card" data-insights-lib="${idx}" type="button">
              <div class="poster-img-wrap">
                ${it.poster ? `<img src="${esc(it.poster)}" loading="lazy" alt="" />` : ''}
              </div>
              <div class="pp-card-info">
                <div class="pp-card-title">
                  <span class="pp-card-name">${esc(it.name)}</span>
                  ${it.imdb_rating ? `<span class="pp-rating">★ ${it.imdb_rating.toFixed(1)}</span>` : ''}
                </div>
                <div class="pp-card-sub">${esc(it.genres.slice(0, 2).join(' · ') || (it.type === 'movie' ? 'Filme' : 'Série'))}${it.release_info ? ` · ${esc(it.release_info)}` : ''}</div>
              </div>
            </button>
          `).join('')}
        </div>
      ` : `<p class="muted" style="margin:12px 0 0;">Nenhum título encontrado para esse filtro.</p>`}
    </div>
  `;

  document.getElementById('btn-insights-back')?.addEventListener('click', () => handleBackNavigation());
  document.getElementById('btn-insights-refresh')?.addEventListener('click', () => loadProfileInsights(true));
  document.getElementById('btn-insights-library')?.addEventListener('click', () => setTab('library'));
  document.getElementById('btn-insights-clear-genre')?.addEventListener('click', () => {
    insightsUi.genreFilter = '';
    renderInsightsDashboard();
  });

  shell.querySelectorAll('[data-insights-sub]').forEach((btn) => {
    btn.addEventListener('click', () => {
      insightsUi.subTab = btn.dataset.insightsSub;
      renderInsightsDashboard();
    });
  });

  shell.querySelectorAll('[data-insights-type]').forEach((btn) => {
    btn.addEventListener('click', () => {
      insightsUi.typeFilter = btn.dataset.insightsType;
      renderInsightsDashboard();
    });
  });

  shell.querySelectorAll('[data-insights-genre]').forEach((btn) => {
    btn.addEventListener('click', () => {
      const g = btn.dataset.insightsGenre;
      insightsUi.genreFilter = insightsUi.genreFilter === g ? '' : g;
      renderInsightsDashboard();
    });
  });

  document.getElementById('insights-sort-select')?.addEventListener('change', (e) => {
    insightsUi.sort = e.target.value;
    renderInsightsDashboard();
  });

  document.getElementById('btn-insights-smart-resume')?.addEventListener('click', () => openItemFromInsights(m.smartResume));
  shell.querySelectorAll('[data-insights-progress]').forEach((btn) => {
    btn.addEventListener('click', () => openItemFromInsights(m.activeProgressRows[Number(btn.dataset.insightsProgress)]));
  });
  shell.querySelectorAll('[data-insights-top]').forEach((btn) => {
    btn.addEventListener('click', () => openItemFromInsights(m.topRated[Number(btn.dataset.insightsTop)]));
  });
  shell.querySelectorAll('[data-insights-person]').forEach((btn) => {
    btn.addEventListener('click', () => {
      const person = m.followed[Number(btn.dataset.insightsPerson)];
      if (person && typeof openPersonPage === 'function') openPersonPage(person);
    });
  });
  shell.querySelectorAll('[data-insights-lib]').forEach((btn) => {
    btn.addEventListener('click', () => openItemFromInsights(filteredLibrary[Number(btn.dataset.insightsLib)]));
  });
}

async function enrichUnresolvedProgressRows(raw) {
  const rows = insightsUi.data?.activeProgressRows || [];
  const unresolved = rows.filter((r) => r.id && r.title === r.id && !insightsMetaCache.has(r.id)).slice(0, 5);
  if (!unresolved.length) return;
  let updated = false;
  await Promise.all(unresolved.map(async (r) => {
    try {
      const meta = await invoke('fetch_meta_details', { mediaType: r.type || 'movie', id: r.id });
      if (meta && (meta.name || meta.title)) {
        insightsMetaCache.set(r.id, {
          name: meta.name || meta.title,
          type: meta.type || r.type,
          poster: meta.poster || '',
          background: meta.banner || meta.background || '',
          release_info: meta.release_info || meta.releaseInfo || '',
        });
        updated = true;
      }
    } catch {
      insightsMetaCache.set(r.id, null);
    }
  }));
  if (updated && activeTabName() === 'insights') {
    insightsUi.data = computeProfileInsights(raw);
    renderInsightsDashboard();
  }
}

async function loadProfileInsights(force = false) {
  if (insightsUi.loading && !force) return;
  insightsUi.loading = true;
  renderInsightsDashboard();
  let lastRaw = null;
  try {
    const raw = await invoke('profile_insights');
    lastRaw = raw || {};
    insightsUi.data = computeProfileInsights(lastRaw);
  } catch {
    lastRaw = {
      library: libraryRawItems || [],
      watched_items: [],
      watch_progress: [],
      followed_people: followedPeopleList || [],
    };
    insightsUi.data = computeProfileInsights(lastRaw);
  } finally {
    insightsUi.loading = false;
    if (activeTabName() === 'insights') {
      renderInsightsDashboard();
      if (lastRaw) enrichUnresolvedProgressRows(lastRaw);
    }
  }
}

document.getElementById('btn-open-profile-insights')?.addEventListener('click', () => setTab('insights'));
document.getElementById('btn-library-insights')?.addEventListener('click', () => setTab('insights'));

function ratingBadges(ratings) {
  if (!Array.isArray(ratings)) return '';
  return ratings.filter(r => ratingProviders[r.source] && Number.isFinite(r.value))
    .map(r => {
      const value = r.value.toLocaleString('pt-BR', { maximumFractionDigits: 1 });
      return `<span class="meta-badge">${esc(ratingProviders[r.source])} ${value}${r.maximum === 100 ? '%' : '/' + r.maximum}</span>`;
    }).join('');
}

const heroRatings = new Map();

async function populateHeroRatings(element, item) {
  const prefs = appSettings?.services;
  if (!prefs?.mdblist_enabled || !prefs.mdblist_hero || !element) return;
  const key = JSON.stringify([item.type, item.id]);
  if (!heroRatings.has(key)) {
    if (heroRatings.size >= 100) heroRatings.delete(heroRatings.keys().next().value);
    heroRatings.set(key, invoke('hero_ratings', { kind: item.type, id: item.id })
      .catch(() => { heroRatings.delete(key); return []; }));
  }
  const badges = ratingBadges(await heroRatings.get(key));
  if (badges && element.isConnected) element.insertAdjacentHTML('beforeend', badges);
}

function metadataShelf(label, items) {
  if (!items?.length) return '';
  return `<section><h3>${esc(label)}</h3><div class="shelf-row">${items.map(renderPosterCard).join('')}</div></section>`;
}

function metadataLandscapeShelf(label, items) {
  if (!items?.length) return '';
  return `<section class="detail-rail-section"><h3 class="detail-section-title">${esc(label)}</h3><div class="shelf-row detail-landscape-row">${items.map((item) => {
    const watched = watchedSet.has(item.id);
    const imgSrc = optimizePosterUrl(item.banner || item.poster || '');
    return `
      <button class="poster-card detail-landscape-card" data-meta-id="${esc(item.id)}" data-meta-type="${esc(item.type)}" data-meta-name="${esc(item.name)}" data-meta-poster="${esc(item.poster || '')}" data-meta-banner="${esc(item.banner || '')}" data-meta-logo="${esc(item.logo || '')}" data-meta-desc="${esc(item.description || '')}" data-meta-release="${esc(item.release_info || '')}" data-meta-imdb="${esc(item.imdb_rating || '')}">
        <div class="poster-img-wrap detail-landscape-wrap">
          ${imgSrc ? `<img src="${esc(imgSrc)}" alt="${esc(item.name)}" loading="lazy" decoding="async" draggable="false" />` : ''}
          ${watched ? '<span class="watched-badge" title="Assistido">&#10003;</span>' : ''}
        </div>
        <div class="poster-name">${esc(item.name)}</div>
      </button>
    `;
  }).join('')}</div></section>`;
}

function mergeSeasonEpisodes(videos, rows, preview, meta) {
  for (const row of rows) {
    const ep = videos.find(v => (v.season ?? 1) === row.season && (v.episode ?? v.number) === row.episode);
    if (ep) {
      for (const [key, value] of Object.entries(row)) {
        if (value != null && value !== '') ep[key] = value;
      }
    } else {
      videos.push({ ...row, id: `${meta.imdb_id || preview.id}:${row.season}:${row.episode}` });
    }
  }
}

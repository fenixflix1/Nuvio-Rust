// Home "Coleções": custom groups of catalogs shown as folders, with covers and hover GIFs.
let homeCollections = [];
let homeCollectionsRequest = 0;
let collectionFolderRequest = 0;
let collectionsAddons = [];
let accountCollections = [];
let localCollections = [];
const LOCAL_COLLECTIONS_KEY = 'nuvio.collections.local';

function folderSources(folder) {
  return Array.isArray(folder.sources) && folder.sources.length
    ? folder.sources
    : Array.isArray(folder.catalogSources) ? folder.catalogSources : [];
}

function resolveCollectionSources(folder, addons) {
  const resolved = [];
  for (const source of folderSources(folder)) {
    const provider = String(source.provider || 'addon').toLowerCase();
    if (provider !== 'addon') continue;
    const addon = (addons || []).find((candidate) => candidate.enabled && candidate.manifest?.id === source.addonId);
    const catalog = addon?.manifest?.catalogs?.find(
      (candidate) => candidate.id === source.catalogId && candidate.type === source.type
    );
    if (!addon || !catalog) continue;
    const genre = source.genre && String(source.genre).toLowerCase() !== 'none' ? String(source.genre).trim() : null;
    const key = `${addon.manifest.id}:${catalog.type}:${catalog.id}:${genre || ''}`;
    if (resolved.some((item) => item.key === key)) continue;
    resolved.push({
      key,
      addonName: addon.manifest.name,
      manifestUrl: addon.manifest_url,
      mediaType: catalog.type,
      catalogId: catalog.id,
      catalogName: catalog.name || catalog.id,
      genre,
      skip: 0,
      hasMore: false,
    });
  }
  return resolved;
}

function collectionTileShape(folder) {
  const shape = String(folder.tileShape || 'LANDSCAPE').toUpperCase();
  return shape === 'PORTRAIT' || shape === 'SQUARE' ? shape.toLowerCase() : 'landscape';
}

function collectionGifMedia(url) {
  const clean = String(url || '').trim();
  if (!clean) return null;
  const videoUrl = /\.gifv($|\?)/i.test(clean)
    ? clean.replace(/\.gifv($|\?)/i, '.mp4$1')
    : /\.(mp4|webm|m4v)($|\?)/i.test(clean) ? clean : '';
  if (videoUrl) {
    const video = document.createElement('video');
    video.muted = true;
    video.loop = true;
    video.playsInline = true;
    video.preload = 'auto';
    video.src = videoUrl;
    return video;
  }
  const image = document.createElement('img');
  image.src = clean;
  image.alt = '';
  image.loading = 'eager';
  image.decoding = 'async';
  return image;
}

function createCollectionTile(collection, folder, addons) {
  const tile = document.createElement('button');
  tile.type = 'button';
  tile.className = 'collection-folder';
  tile.dataset.shape = collectionTileShape(folder);
  tile.title = folder.title || collection.title || '';

  const coverUrl = folder.coverImageUrl || folder.heroBackdropUrl || '';
  // The provided custom covers already contain the logo; only overlay a logo
  // when the folder has no cover of its own.
  const logoUrl = coverUrl ? '' : (folder.titleLogoUrl || '');
  const showTitle = !folder.hideTitle && !logoUrl;
  const gifUrl = folder.focusGifEnabled === false ? '' : String(folder.focusGifUrl || '').trim();

  const placeholder = () => {
    const fallback = document.createElement('span');
    fallback.className = 'collection-folder-placeholder';
    fallback.textContent = folder.coverEmoji || folder.title || collection.title || 'Coleção';
    tile.prepend(fallback);
  };

  if (coverUrl) {
    const cover = document.createElement('img');
    cover.className = 'collection-folder-media';
    cover.src = coverUrl;
    cover.alt = '';
    cover.loading = 'lazy';
    cover.decoding = 'async';
    cover.draggable = false;
    cover.addEventListener('error', () => {
      cover.remove();
      placeholder();
    }, { once: true });
    tile.append(cover);
  } else {
    placeholder();
  }

  const shade = document.createElement('span');
  shade.className = 'collection-folder-shade';
  tile.append(shade);

  if (logoUrl) {
    const logo = document.createElement('img');
    logo.className = 'collection-folder-logo';
    logo.src = logoUrl;
    logo.alt = folder.title || '';
    logo.loading = 'lazy';
    logo.decoding = 'async';
    logo.draggable = false;
    logo.addEventListener('error', () => {
      logo.remove();
      if (folder.hideTitle) return;
      const text = document.createElement('span');
      text.className = 'collection-folder-title';
      text.textContent = folder.title || '';
      tile.append(text);
    }, { once: true });
    tile.append(logo);
  } else if (showTitle) {
    const title = document.createElement('span');
    title.className = 'collection-folder-title';
    title.textContent = folder.title || '';
    tile.append(title);
  }

  if (gifUrl) {
    let gif = null;
    const start = () => {
      if (!gif) {
        gif = collectionGifMedia(gifUrl);
        if (!gif) return;
        gif.classList.add('collection-folder-gif');
        tile.insertBefore(gif, shade);
      }
      if (gif.tagName === 'VIDEO') gif.play().catch(() => {});
    };
    const stop = () => {
      if (gif?.tagName === 'VIDEO') gif.pause();
    };
    tile.addEventListener('pointerenter', start);
    tile.addEventListener('focus', start);
    tile.addEventListener('pointerleave', stop);
    tile.addEventListener('blur', stop);
  }

  tile.addEventListener('click', () => {
    if (tile.closest('.shelf-row')?.dataset.suppressClick) return;
    openCollectionFolder(collection, folder, addons || collectionsAddons);
  });
  return tile;
}

function renderHomeCollections() {
  const container = document.getElementById('home-collections-container');
  if (!container) return;
  container.replaceChildren();
  const collections = homeCollections.filter(
    (collection) => collection && Array.isArray(collection.folders) && collection.folders.length
  );
  if (!collections.length) {
    container.classList.add('hidden');
    return;
  }
  container.classList.remove('hidden');

  const ordered = [...collections].sort(
    (a, b) => Number(Boolean(b.pinToTop)) - Number(Boolean(a.pinToTop))
  );
  for (const collection of ordered) {
    const section = document.createElement('section');
    section.className = 'collection-section';
    const header = document.createElement('div');
    header.className = 'shelf-header';
    const heading = document.createElement('div');
    heading.className = 'shelf-title-btn collection-title';
    heading.textContent = collection.title || 'Coleção';
    header.append(heading);
    section.append(header);

    const row = document.createElement('div');
    row.className = 'shelf-row collection-row';
    if (collection.focusGlowEnabled !== false) row.classList.add('has-glow');
    for (const folder of collection.folders) {
      if (!folder) continue;
      row.append(createCollectionTile(collection, folder, collectionsAddons));
    }
    section.append(row);
    container.append(section);
    wireHorizontalShelfDrag(section);
  }
}

function loadLocalCollections() {
  try {
    const raw = localStorage.getItem(LOCAL_COLLECTIONS_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    const list = Array.isArray(parsed) ? parsed : parsed && typeof parsed === 'object' ? [parsed] : [];
    return list.filter((item) => item && typeof item === 'object');
  } catch {
    return [];
  }
}

function mergeCollections(account, local) {
  const merged = Array.isArray(account) ? [...account] : [];
  const seenIds = new Set(merged.map((item) => item?.id).filter(Boolean));
  const seenTitles = new Set(merged.map((item) => String(item?.title || '').trim().toLowerCase()).filter(Boolean));
  for (const collection of local) {
    if (collection?.id && seenIds.has(collection.id)) continue;
    const title = String(collection?.title || '').trim().toLowerCase();
    if (title && seenTitles.has(title)) continue;
    merged.push(collection);
    if (collection?.id) seenIds.add(collection.id);
    if (title) seenTitles.add(title);
  }
  return merged;
}

function syncCollectionsEditor() {
  const area = document.getElementById('collections-json');
  if (area && document.activeElement !== area) {
    try {
      area.value = localStorage.getItem(LOCAL_COLLECTIONS_KEY) || '';
    } catch {}
  }
  const count = document.getElementById('collections-account-count');
  if (count) {
    count.textContent = `${accountCollections.length} da conta • ${localCollections.length} local(is)`;
  }
}

async function refreshHomeCollections() {
  const container = document.getElementById('home-collections-container');
  if (!container) return;
  const request = ++homeCollectionsRequest;
  const [accountResult, addonsResult] = await Promise.allSettled([
    invoke('pull_account_collections'),
    invoke('list_addons'),
  ]);
  if (request !== homeCollectionsRequest) return;
  accountCollections = accountResult.status === 'fulfilled' && Array.isArray(accountResult.value)
    ? accountResult.value : [];
  collectionsAddons = addonsResult.status === 'fulfilled' && Array.isArray(addonsResult.value)
    ? addonsResult.value : [];
  localCollections = loadLocalCollections();
  homeCollections = mergeCollections(accountCollections, localCollections);
  renderHomeCollections();
  syncCollectionsEditor();
}

function clearHomeCollections() {
  ++homeCollectionsRequest;
  ++collectionFolderRequest;
  accountCollections = [];
  homeCollections = [];
  const container = document.getElementById('home-collections-container');
  if (container) {
    container.replaceChildren();
    container.classList.add('hidden');
  }
}

async function openCollectionFolder(collection, folder, addons) {
  const sources = resolveCollectionSources(folder, addons);
  if (!sources.length) {
    showToast('Nenhum catálogo de addon ativo nesta pasta.');
    return;
  }
  const request = ++collectionFolderRequest;
  activeCatalog = {
    title: folder.title || collection.title || 'Coleção',
    addonName: collection.title || 'Coleção',
    sources,
    items: [],
    media_type: sources[0].mediaType,
    manifestUrl: sources[0].manifestUrl,
    catalogId: sources[0].catalogId,
    genre: sources[0].genre,
  };
  document.getElementById('catalog-page-title').textContent = activeCatalog.title;
  document.getElementById('catalog-page-sub').textContent = sources.length === 1
    ? sources[0].addonName
    : `${sources.length} catálogos • ${collection.title || 'Coleção'}`;
  document.getElementById('catalog-full-status').textContent = 'Carregando catálogos…';
  document.getElementById('catalog-full-grid').innerHTML = '';
  document.getElementById('btn-catalog-load-more').classList.add('hidden');
  setTab('catalog');

  const pages = await Promise.allSettled(sources.map((source) => invoke('load_catalog_page', {
    manifestUrl: source.manifestUrl,
    mediaType: source.mediaType,
    catalogId: source.catalogId,
    skip: 0,
    genre: source.genre,
    search: null,
  })));
  if (request !== collectionFolderRequest) return;

  const seen = new Set();
  pages.forEach((result, index) => {
    const source = sources[index];
    const page = result.status === 'fulfilled' && Array.isArray(result.value) ? result.value : [];
    source.skip = page.length;
    source.hasMore = page.length >= 10;
    for (const item of page) {
      const key = `${item.type}:${item.id}`;
      if (seen.has(key)) continue;
      seen.add(key);
      activeCatalog.items.push(item);
    }
  });
  renderFullCatalogGrid();
  document.getElementById('catalog-full-status').textContent = activeCatalog.items.length
    ? '' : 'Nenhum título encontrado nas fontes desta pasta.';
  document.getElementById('btn-catalog-load-more').classList.toggle(
    'hidden', !sources.some((source) => source.hasMore)
  );
}

async function loadMoreCollectionFolder() {
  const catalog = activeCatalog;
  if (!catalog?.sources) return;
  const btn = document.getElementById('btn-catalog-load-more');
  const status = document.getElementById('catalog-full-status');
  const request = collectionFolderRequest;
  btn.disabled = true;
  status.textContent = 'Carregando mais títulos…';
  try {
    const pending = catalog.sources.filter((source) => source.hasMore);
    const pages = await Promise.allSettled(pending.map((source) => invoke('load_catalog_page', {
      manifestUrl: source.manifestUrl,
      mediaType: source.mediaType,
      catalogId: source.catalogId,
      skip: source.skip,
      genre: source.genre,
      search: null,
    })));
    if (request !== collectionFolderRequest || catalog !== activeCatalog) return;

    const seen = new Set(catalog.items.map((item) => `${item.type}:${item.id}`));
    pages.forEach((result, index) => {
      const source = pending[index];
      const page = result.status === 'fulfilled' && Array.isArray(result.value) ? result.value : [];
      source.skip += page.length;
      source.hasMore = page.length >= 10;
      for (const item of page) {
        const key = `${item.type}:${item.id}`;
        if (seen.has(key)) continue;
        seen.add(key);
        catalog.items.push(item);
      }
    });
    renderFullCatalogGrid();
    status.textContent = '';
    btn.classList.toggle('hidden', !catalog.sources.some((source) => source.hasMore));
  } catch (error) {
    status.textContent = String(error);
  } finally {
    btn.disabled = false;
  }
}

// Collections settings: local JSON import for groupings created outside the account.
function collectionsSettingsStatus(text) {
  const status = document.getElementById('collections-settings-status');
  if (status) status.textContent = text;
}

document.querySelector('[data-settings-sec="collections"]')?.addEventListener('click', () => {
  syncCollectionsEditor();
  collectionsSettingsStatus('');
});

document.getElementById('btn-save-collections-json')?.addEventListener('click', () => {
  const area = document.getElementById('collections-json');
  try {
    const text = area?.value.trim() || '';
    let valid = [];
    if (text) {
      const parsed = JSON.parse(text);
      const list = Array.isArray(parsed) ? parsed : parsed && typeof parsed === 'object' ? [parsed] : [];
      valid = list.filter((item) => item && typeof item === 'object' &&
        Array.isArray(item.folders) && item.folders.length > 0);
      if (!valid.length) throw new Error('nenhuma coleção com pastas foi encontrada.');
      localStorage.setItem(LOCAL_COLLECTIONS_KEY, JSON.stringify(valid, null, 2));
    } else {
      localStorage.removeItem(LOCAL_COLLECTIONS_KEY);
    }
    localCollections = valid;
    if (area) area.value = valid.length ? JSON.stringify(valid, null, 2) : '';
    collectionsSettingsStatus(valid.length ? `Salvo! ${valid.length} coleção(ões) local(is).` : 'Coleções locais removidas.');
    refreshHomeCollections();
  } catch (error) {
    collectionsSettingsStatus('JSON inválido: ' + (error?.message || error));
  }
});

document.getElementById('btn-clear-local-collections')?.addEventListener('click', () => {
  try { localStorage.removeItem(LOCAL_COLLECTIONS_KEY); } catch {}
  localCollections = [];
  const area = document.getElementById('collections-json');
  if (area) area.value = '';
  collectionsSettingsStatus('Coleções locais removidas.');
  refreshHomeCollections();
});

document.getElementById('btn-reload-account-collections')?.addEventListener('click', async () => {
  collectionsSettingsStatus('Recarregando coleções…');
  await refreshHomeCollections();
  collectionsSettingsStatus('Coleções atualizadas.');
});

syncCollectionsEditor();

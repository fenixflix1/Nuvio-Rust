// Shared by the native overlay and the fallback controls.
function bindSeekPreview(input, readTrack, durationMs) {
  const strip = document.createElement('div');
  strip.setAttribute('aria-hidden', 'true');
  strip.style.cssText = 'display:none;position:fixed;pointer-events:none;z-index:2147483647;gap:3px;align-items:center;padding:4px;background:#111;border-radius:7px;overflow:hidden';
  document.body.append(strip);
  let currentUrl = '', cues = [], pending, retryAt = 0;
  let dragging = false, timer, requested = 0;
  function prepare(url) {
    if (url === currentUrl && (pending || cues.length || Date.now() < retryAt)) return pending;
    currentUrl = url;
    cues = [];
    pending = (async () => {
      try {
        const response = await fetch(url, { credentials: 'omit' });
        if (!response.ok) throw Error('Prévias indisponíveis');
        const parsed = parseSeekrVtt(await response.text(), url);
        if (!parsed.length) throw Error('Miniaturas indisponíveis');
        if (currentUrl === url) cues = parsed;
      } catch {
        if (currentUrl === url) retryAt = Date.now() + 10000;
      } finally {
        if (currentUrl === url) pending = null;
      }
    })();
    return pending;
  }
  function hide() {
    ++requested;
    clearTimeout(timer);
    strip.style.display = 'none';
    strip.replaceChildren();
  }
  async function show(mode, ms, clientX) {
    const serial = ++requested;
    const track = readTrack();
    if (!track?.[mode] || !track.url) { hide(); return; }
    await prepare(track.url);
    if (serial !== requested || !cues.length || readTrack()?.url !== track.url) return;
    const target = Math.max(0, ms / track.scale);
    let index = cues.findIndex(cue => cue.end > target);
    if (index < 0) index = cues.length - 1;
    const count = Math.min(cues.length, Math.max(1, Math.min(9, track.count)));
    const first = Math.max(0, Math.min(index - Math.floor(count / 2), cues.length - count));
    const signature = [track.url, first, index, count, track.size, innerWidth].join('|');
    if (strip.dataset.signature !== signature || !strip.children.length) {
      strip.replaceChildren();
      strip.dataset.signature = signature;
      for (let i = first; i < first + count; i++) {
        const c = cues[i];
        const width = Math.min(124 * track.size / 100, (innerWidth - 30) / count - 3);
        const factor = width / c.w;
        const frame = document.createElement('div');
        frame.style.cssText = 'position:relative;overflow:hidden;flex:none;border-radius:4px';
        frame.style.width = width + 'px';
        frame.style.height = factor * c.h + 'px';
        frame.style.opacity = i === index ? '1' : '.64';
        const img = document.createElement('img');
        img.alt = '';
        img.src = c.url;
        img.style.cssText = 'position:absolute;width:auto;height:auto;max-width:none;max-height:none;transform-origin:top left';
        img.style.transform = 'scale(' + factor + ')';
        img.style.left = -c.x * factor + 'px';
        img.style.top = -c.y * factor + 'px';
        frame.append(img);
        strip.append(frame);
      }
    }
    const rect = input.getBoundingClientRect();
    const x = clientX ?? (rect.left + rect.width * Math.min(1, ms / durationMs()));
    strip.style.display = 'flex';
    strip.style.left = Math.max(4, Math.min(innerWidth - strip.offsetWidth - 4, x - strip.offsetWidth / 2)) + 'px';
    strip.style.bottom = Math.max(5, innerHeight - rect.top + 8) + 'px';
    clearTimeout(timer);
    if (mode === 'keyboard') timer = setTimeout(hide, 1200);
  }
  function pointer(ev) {
    const rect = input.getBoundingClientRect();
    const fraction = Math.max(0, Math.min(1, (ev.clientX - rect.left) / rect.width));
    show(dragging ? 'drag' : 'hover', durationMs() * fraction, ev.clientX);
  }
  input.addEventListener('pointerdown', ev => { dragging = true; pointer(ev); });
  input.addEventListener('pointermove', pointer);
  input.addEventListener('pointerleave', () => { if (!dragging) hide(); });
  window.addEventListener('pointerup', () => { if (dragging) hide(); dragging = false; });
  window.addEventListener('pointercancel', () => { dragging = false; hide(); });
  return { show, hide };
}

function parseSeekrVtt(text, base) {
  const cues = [];
  const timestamp = value => value.split(':').reduce((s, part) => s * 60 + Number(part), 0) * 1000;
  for (const block of text.trim().split(/\r?\n\s*\r?\n/)) {
    const m = block.match(/([\d:.]+)\s+-->\s+([\d:.]+)[^\n]*\n([^\s#]+)#xywh=(\d+),(\d+),(\d+),(\d+)/);
    if (!m) continue;
    let url;
    try { url = new URL(m[3], base); } catch { continue; }
    if (url.protocol !== 'https:' || url.hostname !== 'sprites.seekr.tv') continue;
    const end = timestamp(m[2]);
    if (Number.isFinite(end) && +m[6] > 0 && +m[7] > 0) cues.push({end, url:url.href, x:+m[4], y:+m[5], w:+m[6], h:+m[7]});
  }
  return cues;
}

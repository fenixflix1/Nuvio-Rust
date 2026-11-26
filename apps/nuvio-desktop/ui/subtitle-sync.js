// An addon timeline is aligned only against observations from an embedded text track.
function subtitleFingerprint(text) {
  return text.normalize('NFKD').toLowerCase().replace(/[^\p{L}\p{N}]+/gu, '');
}

function subtitleIndex(cues) {
  const text = new Map();
  const times = [];
  for (const cue of cues) {
    const key = subtitleFingerprint(cue.text);
    if (key.length >= 8) text.set(key, text.has(key) ? null : cue.timeMs);
    times.push(cue.timeMs);
  }
  times.sort((a, b) => a - b);
  return { text, times };
}

function medianDelay(values) {
  values.sort((a, b) => a - b);
  return values[Math.floor(values.length / 2)];
}

function textDelay(samples, index, minCount = 4) {
  const offsets = [];
  for (const cue of samples) {
    const ref = index.text.get(subtitleFingerprint(cue.text));
    if (ref != null) offsets.push(cue.timeMs - ref);
  }
  if (offsets.length < minCount) return null;
  const median = medianDelay(offsets);
  return offsets.filter(n => Math.abs(n - median) <= 300).length >= Math.ceil(offsets.length * 0.9)
    ? median : null;
}

function lowerBound(times, target) {
  let lo = 0, hi = times.length;
  while (lo < hi) {
    const mid = (lo + hi) >>> 1;
    if (times[mid] < target) lo = mid + 1;
    else hi = mid;
  }
  return lo;
}

// Across languages, require a unique pattern across at least twelve separate cues.
function timingDelay(samples, index) {
  if (samples.length < 12 || samples.at(-1).timeMs - samples[0].timeMs < 25000) return null;
  const times = index.times;
  const target = samples[0].timeMs;
  const candidates = [];
  for (let i = lowerBound(times, target - 120000); i < times.length && times[i] <= target + 120000; i++) {
    const delta = target - times[i];
    const offsets = [];
    let previous = -1;
    for (const sample of samples) {
      const pos = lowerBound(times, sample.timeMs - delta);
      const nearest = pos > 0 && (pos === times.length ||
        Math.abs(sample.timeMs - delta - times[pos - 1]) < Math.abs(sample.timeMs - delta - times[pos])) ? pos - 1 : pos;
      if (nearest >= times.length || nearest <= previous) continue;
      const actual = sample.timeMs - times[nearest];
      if (Math.abs(actual - delta) <= 160) {
        offsets.push(actual);
        previous = nearest;
      }
    }
    if (offsets.length >= samples.length * 0.75) candidates.push({ n: offsets.length, delta: medianDelay(offsets) });
  }
  candidates.sort((a, b) => b.n - a.n);
  const best = candidates[0];
  if (!best || best.n < samples.length * 0.9) return null;
  const other = candidates.find(c => Math.abs(c.delta - best.delta) > 300);
  return other && other.n >= best.n - 2 ? null : best.delta;
}

function cancelSubtitleSync(ctx, resetDelay = false) {
  if (ctx.subtitleSync?.track) invoke('player_set_reference_track', { sid: null }).catch(() => {});
  ctx.subtitleSync = null;
  if (resetDelay && ctx.autoSyncedApplied) {
    ctx.subtitleDelayMs = 0;
    invoke('player_set_subtitle_delay', { delayMs: 0 }).catch(() => {});
  }
  ctx.autoSyncedApplied = false;
}

async function beginSubtitleSync(ctx, sub) {
  const thorough = appSettings.playback.thorough_sync;
  const run = { references: [], samples: [], tried: new Set(), thorough, indexes: [] };
  ctx.subtitleSync = run;
  const current = () => activePlayerCtx === ctx && ctx.subtitleSync === run;
  async function load(url) {
    let cues = ctx.cuesCache.get(url);
    if (!cues) {
      cues = await invoke('player_load_subtitle_cues', { url });
      if (current()) ctx.cuesCache.set(url, cues);
    }
    return cues;
  }
  try {
    const cues = await load(sub.url);
    if (!current() || cues.length < 4) return;
    run.indexes.push({ index: subtitleIndex(cues), correction: 0 });
    if (!thorough) return;
    // Other translations/releases can supply matching lines only after verifying
    // how their clocks relate to the user's selected file across its full length.
    const language = normalizeLangCode(sub.lang);
    const urls = new Set(ctx.addonSubtitles.filter(s => s.url !== sub.url &&
      normalizeLangCode(s.lang) === language).map(s => s.url));
    for (const url of Array.from(urls).slice(0, 4)) {
      if (!current()) return;
      const alt = await load(url).catch(() => []);
      if (!current() || alt.length < 12) continue;
      const index = subtitleIndex(alt);
      const stride = Math.max(1, Math.floor(cues.length / 150));
      const correction = textDelay(cues.filter((_, i) => i % stride === 0), index, 12);
      if (correction !== null) run.indexes.push({ index, correction });
    }
  } catch {
    if (current()) cancelSubtitleSync(ctx);
  }
}

function observeSubtitleSync(ctx, state) {
  const run = ctx.subtitleSync;
  if (!run || !run.indexes.length || !ctx.activeAddonSub || state.paused || state.loading || run.busy) return;
  if (!run.track) {
    const language = normalizeLangCode(ctx.activeAddonSub.lang);
    const tracks = state.subtitle_tracks.filter(t => !t.external && !t.forced && !run.tried.has(t.id) &&
      !/dvd|dvb|pgs|bitmap|xsub/i.test(t.codec));
    const track = tracks.find(t => normalizeLangCode(t.lang) === language) || tracks[0];
    if (!track) return;
    run.track = track.id;
    run.tried.add(track.id);
    run.start = state.position_secs;
    run.samples = [];
    invoke('player_set_reference_track', { sid: track.id }).catch(() => {
      if (ctx.subtitleSync === run) cancelSubtitleSync(ctx);
    });
    return;
  }

  if (state.reference_id === run.track && state.reference_text) {
    const timeMs = Math.round(state.reference_start_secs * 1000);
    const last = run.samples.at(-1)?.timeMs;
    if (last !== timeMs) {
      if (last != null && (timeMs < last || timeMs - last > 90000)) run.samples = [];
      run.samples.push({ text: state.reference_text, timeMs });
      if (run.samples.length > 32) run.samples.shift();
      if (run.samples.length >= 4 && timeMs - run.samples[0].timeMs >= 8000) {
        const matches = [];
        for (const { index, correction } of run.indexes) {
          const offset = textDelay(run.samples, index) ?? timingDelay(run.samples, index);
          if (offset !== null) matches.push(offset - correction);
        }
        if (matches.length && Math.max(...matches) - Math.min(...matches) < 350) {
          const delta = medianDelay(matches);
          run.busy = true;
          invoke('player_set_subtitle_delay', { delayMs: delta }).then(() => {
            if (activePlayerCtx !== ctx || ctx.subtitleSync !== run) return;
            cancelSubtitleSync(ctx);
            ctx.subtitleDelayMs = delta;
            ctx.autoSyncedApplied = true;
            pushPlayerControls();
          }).catch(() => { if (ctx.subtitleSync === run) cancelSubtitleSync(ctx); });
          return;
        }
      }
    }
  }
  if (Math.abs(state.position_secs - run.start) >= (run.thorough ? 90 : 60)) {
    if (!run.thorough || run.tried.size >= state.subtitle_tracks.filter(t => !t.external && !t.forced).length) {
      cancelSubtitleSync(ctx);
    } else {
      invoke('player_set_reference_track', { sid: null }).catch(() => {});
      run.track = null;
    }
  }
}

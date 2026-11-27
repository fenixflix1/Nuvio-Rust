(function () {
  'use strict';

  function injectQrInto(container, url, labelText, sizePx) {
    if (!container) return;
    var cleanUrl = String(url || '').trim();
    var existing = container.querySelector('.webos-qr-wrap');
    if (!cleanUrl || !window.NuvioQR) {
      if (existing) existing.remove();
      return;
    }
    if (!existing) {
      existing = document.createElement('div');
      existing.className = 'webos-qr-wrap';
      container.prepend(existing);
    }
    if (existing.dataset.url === cleanUrl) return;
    existing.dataset.url = cleanUrl;
    try {
      existing.innerHTML =
        '<div class="webos-qr-box">' + window.NuvioQR.renderSvg(cleanUrl, sizePx || 180) + '</div>' +
        (labelText ? '<div class="webos-qr-caption">' + labelText + '</div>' : '');
    } catch (_) {
      existing.remove();
    }
  }

  function wireQrObservers() {
    // 1. Nuvio Login com Código -> injeta QR Code grande dentro de #device-link-waiting
    var devBox = document.getElementById('device-link-waiting');
    var devUrlEl = document.getElementById('device-link-url');
    if (devBox && devUrlEl) {
      var syncDeviceQr = function () {
        var visible = !devBox.classList.contains('hidden');
        var href = devUrlEl.getAttribute('href') || '';
        if (visible && /^https?:\/\//i.test(href)) {
          injectQrInto(devBox, href, 'Escaneie com a câmera do celular para entrar', 196);
        } else {
          injectQrInto(devBox, '');
        }
      };
      new MutationObserver(syncDeviceQr).observe(devBox, { attributes: true, attributeFilter: ['class'] });
      new MutationObserver(syncDeviceQr).observe(devUrlEl, { attributes: true, attributeFilter: ['href'] });
      syncDeviceQr();
    }

    // 2. Simkl PIN -> injeta QR Code quando o código aparecer em #simkl-status
    var simklStatus = document.getElementById('simkl-status');
    if (simklStatus && simklStatus.parentNode) {
      var syncSimklQr = function () {
        var txt = simklStatus.textContent || '';
        var mUrl = /https?:\/\/\S+/.exec(txt);
        var mPin = /código\s+([A-Z0-9-]+)/i.exec(txt);
        var qrHolder = document.getElementById('webos-simkl-qr');
        if (mUrl && mPin) {
          if (!qrHolder) {
            qrHolder = document.createElement('div');
            qrHolder.id = 'webos-simkl-qr';
            simklStatus.insertAdjacentElement('afterend', qrHolder);
          }
          var base = mUrl[0].replace(/[.,;)]+$/, '');
          var qrUrl = base.indexOf('?') >= 0 ? base : (base + '?pin=' + encodeURIComponent(mPin[1]));
          injectQrInto(qrHolder, qrUrl, 'Escaneie no celular para autorizar o Simkl', 164);
        } else if (qrHolder) {
          qrHolder.innerHTML = '';
        }
      };
      new MutationObserver(syncSimklQr).observe(simklStatus, { childList: true, characterData: true, subtree: true });
    }

    // 3. Serviços conectados (Torbox / Premiumize) -> QR Code quando aparecer URL/código no status
    var connRoot = document.getElementById('connected-provider-cards');
    if (connRoot) {
      new MutationObserver(function () {
        connRoot.querySelectorAll('[data-account]').forEach(function (card) {
          var st = card.querySelector('.nv-account-status');
          var txt = (st && st.textContent) || '';
          var mUrl = /https?:\/\/\S+/.exec(txt);
          var holder = card.querySelector('.webos-conn-qr');
          if (mUrl) {
            if (!holder) {
              holder = document.createElement('div');
              holder.className = 'webos-conn-qr';
              card.appendChild(holder);
            }
            injectQrInto(holder, mUrl[0].replace(/[.,;)]+$/, ''), 'Escaneie para autorizar no celular', 156);
          } else if (holder) {
            holder.remove();
          }
        });
      }).observe(connRoot, { childList: true, characterData: true, subtree: true });
    }
  }

  function showAddonConfigureQrModal(url, title) {
    var modal = document.getElementById('plugin-settings-modal');
    var titleEl = document.getElementById('plugin-settings-title');
    var bodyEl = document.getElementById('plugin-settings-body');
    var cancelBtn = document.getElementById('btn-plugin-settings-cancel');
    var saveBtn = document.getElementById('btn-plugin-settings-save');
    if (!modal || !bodyEl) return;
    titleEl.textContent = title || 'Abrir no celular';
    bodyEl.innerHTML = '<div class="webos-qr-modal-box"></div><p class="plugin-setting-desc" style="word-break:break-all;text-align:center;">' + url + '</p>';
    injectQrInto(bodyEl.querySelector('.webos-qr-modal-box'), url, 'Escaneie com o celular para abrir e configurar', 210);
    if (saveBtn) saveBtn.style.display = 'none';
    if (cancelBtn) {
      cancelBtn.textContent = 'Fechar';
      cancelBtn.onclick = function () {
        modal.classList.add('hidden');
        if (saveBtn) saveBtn.style.display = '';
        cancelBtn.textContent = 'Cancelar';
      };
    }
    modal.classList.remove('hidden');
  }

  window.NuvioWebosQr = {
    injectQrInto: injectQrInto,
    showModal: showAddonConfigureQrModal
  };

  if (document.body) {
    document.body.classList.add('webos-tv');
    wireQrObservers();
  } else {
    document.addEventListener('DOMContentLoaded', function () {
      document.body.classList.add('webos-tv');
      wireQrObservers();
    });
  }

  var KEY = {
    LEFT: 37,
    UP: 38,
    RIGHT: 39,
    DOWN: 40,
    ENTER: 13,
    BACK_WEBOS: 461,
    ESCAPE: 27,
    PLAY: 415,
    PAUSE: 19,
    STOP: 413,
    FF: 417,
    RW: 412,
    PLAY_PAUSE: 10252
  };

  var FOCUS_SELECTOR = [
    'button:not([disabled])',
    'a[href]',
    'input:not([disabled]):not([type="hidden"])',
    'select:not([disabled])',
    'summary',
    '.poster-card',
    '.cw-card',
    '.stream-row',
    '.episode-card',
    '.nav-item',
    '.settings-cat-btn'
  ].join(',');

  function activeScope() {
    var openModal = Array.prototype.slice.call(document.querySelectorAll('.modal-backdrop')).find(function (m) {
      return !m.classList.contains('hidden');
    });
    if (openModal) return openModal;
    var activeView = document.querySelector('.view.active') || document.body;
    return activeView;
  }

  function isVisibleFocusable(el) {
    if (!el || (el.offsetParent === null && getComputedStyle(el).position !== 'fixed')) return false;
    var r = el.getBoundingClientRect();
    return r.width > 4 && r.height > 4 && getComputedStyle(el).visibility !== 'hidden';
  }

  function focusElement(el) {
    if (!el) return;
    document.querySelectorAll('.tv-focused').forEach(function (prev) { prev.classList.remove('tv-focused'); });
    if (!el.hasAttribute('tabindex') && !/^(BUTTON|A|INPUT|SELECT|TEXTAREA|SUMMARY)$/.test(el.tagName)) {
      el.setAttribute('tabindex', '0');
    }
    el.classList.add('tv-focused');
    try { el.focus({ preventScroll: true }); } catch (_) { el.focus(); }
    try { el.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'nearest' }); } catch (_) { el.scrollIntoView(); }
  }

  function moveDirection(dirX, dirY) {
    var scope = activeScope();
    var all = Array.prototype.slice.call(scope.querySelectorAll(FOCUS_SELECTOR)).filter(isVisibleFocusable);
    if (!all.length) return;

    var cur = document.activeElement;
    if (!cur || scope.contains(cur) === false || !isVisibleFocusable(cur)) {
      focusElement(all[0]);
      return;
    }

    var cr = cur.getBoundingClientRect();
    var cx = cr.left + cr.width / 2;
    var cy = cr.top + cr.height / 2;
    var best = null;
    var bestScore = Infinity;

    for (var i = 0; i < all.length; i++) {
      var cand = all[i];
      if (cand === cur) continue;
      var rr = cand.getBoundingClientRect();
      var rx = rr.left + rr.width / 2;
      var ry = rr.top + rr.height / 2;
      var dx = rx - cx;
      var dy = ry - cy;

      if (dirX < 0 && dx >= -4) continue;
      if (dirX > 0 && dx <= 4) continue;
      if (dirY < 0 && dy >= -4) continue;
      if (dirY > 0 && dy <= 4) continue;

      var primary = dirX !== 0 ? Math.abs(dx) : Math.abs(dy);
      var perp = dirX !== 0 ? Math.abs(dy) : Math.abs(dx);
      var score = primary + perp * 2.5;
      if (score < bestScore) {
        bestScore = score;
        best = cand;
      }
    }

    if (best) focusElement(best);
  }

  function handleBackKey() {
    var modals = document.querySelectorAll('.modal-backdrop:not(.hidden)');
    if (modals.length) {
      modals[modals.length - 1].classList.add('hidden');
      return true;
    }
    var playerView = document.getElementById('view-player');
    if (playerView && playerView.classList.contains('active')) {
      var bBtn = document.getElementById('btn-player-back');
      if (bBtn) bBtn.click();
      return true;
    }
    var activeTab = document.querySelector('#view-app .tab-panel.active');
    if (activeTab && activeTab.id !== 'tab-home') {
      var homeBtn = document.querySelector('[data-tab="home"]');
      if (homeBtn) homeBtn.click();
      return true;
    }
    if (typeof window.webOS !== 'undefined' && window.webOS.platformBack) {
      window.webOS.platformBack();
      return true;
    }
    return false;
  }

  window.addEventListener('keydown', function (e) {
    var tag = document.activeElement && document.activeElement.tagName;
    var inTextInput = tag === 'INPUT' && !/^(checkbox|radio|range|button)$/i.test(document.activeElement.type);

    if (e.keyCode === KEY.BACK_WEBOS || e.keyCode === KEY.ESCAPE) {
      if (handleBackKey()) {
        e.preventDefault();
        e.stopPropagation();
      }
      return;
    }

    if (e.keyCode === KEY.PLAY || e.keyCode === KEY.PAUSE || e.keyCode === KEY.PLAY_PAUSE) {
      var pp = document.getElementById('btn-player-playpause');
      var pv = document.getElementById('view-player');
      if (pp && pv && pv.classList.contains('active')) {
        e.preventDefault();
        pp.click();
      }
      return;
    }
    if (e.keyCode === KEY.STOP && document.getElementById('view-player') && document.getElementById('view-player').classList.contains('active')) {
      e.preventDefault();
      var bb = document.getElementById('btn-player-back');
      if (bb) bb.click();
      return;
    }
    if (e.keyCode === KEY.FF && document.getElementById('view-player') && document.getElementById('view-player').classList.contains('active')) {
      e.preventDefault();
      var fb = document.getElementById('btn-player-fwd10');
      if (fb) fb.click();
      return;
    }
    if (e.keyCode === KEY.RW && document.getElementById('view-player') && document.getElementById('view-player').classList.contains('active')) {
      e.preventDefault();
      var rb = document.getElementById('btn-player-back10');
      if (rb) rb.click();
      return;
    }

    if (inTextInput && (e.keyCode === KEY.LEFT || e.keyCode === KEY.RIGHT)) return;

    if (e.keyCode === KEY.LEFT) { e.preventDefault(); moveDirection(-1, 0); }
    else if (e.keyCode === KEY.RIGHT) { e.preventDefault(); moveDirection(1, 0); }
    else if (e.keyCode === KEY.UP) { e.preventDefault(); moveDirection(0, -1); }
    else if (e.keyCode === KEY.DOWN) { e.preventDefault(); moveDirection(0, 1); }
    else if (e.keyCode === KEY.ENTER && document.activeElement && !/^(BUTTON|A|INPUT|SELECT|SUMMARY)$/.test(tag || '')) {
      e.preventDefault();
      document.activeElement.click();
    }
  }, true);
})();

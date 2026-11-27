(function (root) {
  'use strict';

  var GF_EXP = new Uint8Array(512);
  var GF_LOG = new Uint8Array(256);
  (function initGalois() {
    var x = 1;
    for (var i = 0; i < 255; i++) {
      GF_EXP[i] = x;
      GF_LOG[x] = i;
      x <<= 1;
      if (x & 0x100) x ^= 0x11d;
    }
    for (var j = 255; j < 512; j++) GF_EXP[j] = GF_EXP[j - 255];
  })();

  function gfMul(a, b) {
    if (a === 0 || b === 0) return 0;
    return GF_EXP[GF_LOG[a] + GF_LOG[b]];
  }

  function rsGenerator(degree) {
    var poly = new Uint8Array([1]);
    for (var i = 0; i < degree; i++) {
      var next = new Uint8Array(poly.length + 1);
      var factor = GF_EXP[i];
      for (var j = 0; j < poly.length; j++) {
        next[j] ^= poly[j];
        next[j + 1] ^= gfMul(poly[j], factor);
      }
      poly = next;
    }
    return poly;
  }

  function rsEncode(data, ecCount) {
    var gen = rsGenerator(ecCount);
    var res = new Uint8Array(data.length + ecCount);
    res.set(data);
    for (var i = 0; i < data.length; i++) {
      var coef = res[i];
      if (coef !== 0) {
        for (var j = 0; j < gen.length; j++) {
          res[i + j] ^= gfMul(gen[j], coef);
        }
      }
    }
    return res.slice(data.length);
  }

  // ISO/IEC 18004 ECC Level M (00) specs for Versions 1..10
  var SPECS_M = [
    null,
    { size: 21, ecPerBlock: 10, blocks: [[1, 16]], align: [] },
    { size: 25, ecPerBlock: 16, blocks: [[1, 28]], align: [6, 18] },
    { size: 29, ecPerBlock: 26, blocks: [[1, 44]], align: [6, 22] },
    { size: 33, ecPerBlock: 18, blocks: [[2, 32]], align: [6, 26] },
    { size: 37, ecPerBlock: 24, blocks: [[2, 43]], align: [6, 30] },
    { size: 41, ecPerBlock: 16, blocks: [[4, 27]], align: [6, 34] },
    { size: 45, ecPerBlock: 18, blocks: [[4, 31]], align: [6, 22, 38] },
    { size: 49, ecPerBlock: 22, blocks: [[2, 38], [2, 39]], align: [6, 24, 42] },
    { size: 53, ecPerBlock: 22, blocks: [[3, 36], [2, 37]], align: [6, 26, 46] },
    { size: 57, ecPerBlock: 26, blocks: [[4, 43], [1, 44]], align: [6, 28, 50] }
  ];

  function toUtf8Bytes(str) {
    if (typeof TextEncoder !== 'undefined') return Array.from(new TextEncoder().encode(String(str)));
    var utf8 = unescape(encodeURIComponent(String(str)));
    var out = [];
    for (var i = 0; i < utf8.length; i++) out.push(utf8.charCodeAt(i));
    return out;
  }

  function bitLength(n) {
    var l = 0;
    while (n > 0) { l++; n >>= 1; }
    return l;
  }

  function formatInfoBits(mask) {
    var rem = mask << 10;
    while (bitLength(rem) >= 11) rem ^= 0x537 << (bitLength(rem) - 11);
    return ((mask << 10) | rem) ^ 0x5412;
  }

  function versionInfoBits(ver) {
    var rem = ver << 12;
    while (bitLength(rem) >= 13) rem ^= 0x1F25 << (bitLength(rem) - 13);
    return (ver << 12) | rem;
  }

  function makeDataCodewords(bytes, version, totalDataBytes) {
    var bits = [];
    function pushBits(val, len) {
      for (var i = len - 1; i >= 0; i--) bits.push((val >> i) & 1);
    }
    pushBits(4, 4); // Byte mode
    pushBits(bytes.length, version >= 10 ? 16 : 8);
    for (var i = 0; i < bytes.length; i++) pushBits(bytes[i], 8);
    var capBits = totalDataBytes * 8;
    var term = Math.min(4, capBits - bits.length);
    pushBits(0, term);
    while (bits.length % 8 !== 0) bits.push(0);
    var pad = [0xEC, 0x11];
    var p = 0;
    while (bits.length < capBits) {
      pushBits(pad[p % 2], 8);
      p++;
    }
    var out = new Uint8Array(totalDataBytes);
    for (var b = 0; b < totalDataBytes; b++) {
      var val = 0;
      for (var k = 0; k < 8; k++) val = (val << 1) | bits[b * 8 + k];
      out[b] = val;
    }
    return out;
  }

  function interleaveBlocks(dataBytes, spec) {
    var dataBlocks = [];
    var ecBlocks = [];
    var offset = 0;
    var maxData = 0;
    for (var g = 0; g < spec.blocks.length; g++) {
      var count = spec.blocks[g][0];
      var len = spec.blocks[g][1];
      if (len > maxData) maxData = len;
      for (var c = 0; c < count; c++) {
        var slice = dataBytes.slice(offset, offset + len);
        offset += len;
        dataBlocks.push(slice);
        ecBlocks.push(rsEncode(slice, spec.ecPerBlock));
      }
    }
    var result = [];
    for (var i = 0; i < maxData; i++) {
      for (var b = 0; b < dataBlocks.length; b++) {
        if (i < dataBlocks[b].length) result.push(dataBlocks[b][i]);
      }
    }
    for (var e = 0; e < spec.ecPerBlock; e++) {
      for (var eb = 0; eb < ecBlocks.length; eb++) result.push(ecBlocks[eb][e]);
    }
    return new Uint8Array(result);
  }

  function buildMatrix(codewords, spec, version) {
    var size = spec.size;
    var grid = Array.from({ length: size }, function () { return new Int8Array(size).fill(-1); });
    var isFunc = Array.from({ length: size }, function () { return new Uint8Array(size); });

    function setFunc(r, c, val) {
      if (r >= 0 && c >= 0 && r < size && c < size) {
        grid[r][c] = val ? 1 : 0;
        isFunc[r][c] = 1;
      }
    }

    function drawFinder(top, left) {
      for (var r = -1; r <= 7; r++) {
        for (var c = -1; c <= 7; c++) {
          var rr = top + r;
          var cc = left + c;
          if (rr < 0 || cc < 0 || rr >= size || cc >= size) continue;
          var on = r >= 0 && r <= 6 && c >= 0 && c <= 6 &&
            (r === 0 || r === 6 || c === 0 || c === 6 || (r >= 2 && r <= 4 && c >= 2 && c <= 4));
          setFunc(rr, cc, on);
        }
      }
    }

    drawFinder(0, 0);
    drawFinder(0, size - 7);
    drawFinder(size - 7, 0);

    for (var i = 8; i < size - 8; i++) {
      setFunc(6, i, i % 2 === 0);
      setFunc(i, 6, i % 2 === 0);
    }

    var centers = spec.align;
    for (var ai = 0; ai < centers.length; ai++) {
      for (var aj = 0; aj < centers.length; aj++) {
        var ar = centers[ai];
        var ac = centers[aj];
        if (isFunc[ar][ac]) continue;
        for (var dr = -2; dr <= 2; dr++) {
          for (var dc = -2; dc <= 2; dc++) {
            setFunc(ar + dr, ac + dc, Math.max(Math.abs(dr), Math.abs(dc)) !== 1);
          }
        }
      }
    }

    setFunc(size - 8, 8, true);

    // Reserve format & version areas
    for (var f = 0; f <= 8; f++) {
      if (f !== 6) {
        if (!isFunc[8][f]) setFunc(8, f, false);
        if (!isFunc[f][8]) setFunc(f, 8, false);
      }
    }
    for (var f2 = size - 8; f2 < size; f2++) {
      if (!isFunc[8][f2]) setFunc(8, f2, false);
    }
    for (var f3 = size - 7; f3 < size; f3++) {
      if (!isFunc[f3][8]) setFunc(f3, 8, false);
    }
    if (version >= 7) {
      var vBits = versionInfoBits(version);
      for (var vi = 0; vi < 18; vi++) {
        var bit = ((vBits >> vi) & 1) === 1;
        var a = Math.floor(vi / 3);
        var b = vi % 3;
        setFunc(a, size - 11 + b, bit);
        setFunc(size - 11 + b, a, bit);
      }
    }

    // Place data modules in zig-zag
    var bitIdx = 0;
    var totalBits = codewords.length * 8;
    for (var col = size - 1; col >= 1; col -= 2) {
      if (col === 6) col--;
      var upward = (((size - 1 - col) >> 1) & 1) === 0;
      for (var step = 0; step < size; step++) {
        var r = upward ? size - 1 - step : step;
        for (var dx = 0; dx < 2; dx++) {
          var c = col - dx;
          if (!isFunc[r][c]) {
            var bitVal = 0;
            if (bitIdx < totalBits) {
              bitVal = (codewords[bitIdx >> 3] >> (7 - (bitIdx & 7))) & 1;
              bitIdx++;
            }
            grid[r][c] = bitVal;
          }
        }
      }
    }

    function maskBit(mask, r, c) {
      switch (mask) {
        case 0: return (r + c) % 2 === 0;
        case 1: return r % 2 === 0;
        case 2: return c % 3 === 0;
        case 3: return (r + c) % 3 === 0;
        case 4: return (Math.floor(r / 2) + Math.floor(c / 3)) % 2 === 0;
        case 5: return ((r * c) % 2 + (r * c) % 3) === 0;
        case 6: return (((r * c) % 2 + (r * c) % 3) % 2) === 0;
        default: return (((r + c) % 2 + (r * c) % 3) % 2) === 0;
      }
    }

    function applyFormat(mGrid, mask) {
      var fmt = formatInfoBits(mask);
      for (var k = 0; k < 15; k++) {
        var v = ((fmt >> k) & 1);
        if (k < 6) mGrid[k][8] = v;
        else if (k < 8) mGrid[k + 1][8] = v;
        else mGrid[size - 15 + k][8] = v;

        if (k < 8) mGrid[8][size - 1 - k] = v;
        else if (k === 8) mGrid[8][7] = v;
        else mGrid[8][14 - k] = v;
      }
    }

    function penalty(mGrid) {
      var score = 0;
      for (var r = 0; r < size; r++) {
        var run = 1;
        for (var c = 1; c < size; c++) {
          if (mGrid[r][c] === mGrid[r][c - 1]) {
            run++;
            if (run === 5) score += 3;
            else if (run > 5) score += 1;
          } else run = 1;
        }
      }
      for (var c2 = 0; c2 < size; c2++) {
        var run2 = 1;
        for (var r2 = 1; r2 < size; r2++) {
          if (mGrid[r2][c2] === mGrid[r2 - 1][c2]) {
            run2++;
            if (run2 === 5) score += 3;
            else if (run2 > 5) score += 1;
          } else run2 = 1;
        }
      }
      for (var r3 = 0; r3 < size - 1; r3++) {
        for (var c3 = 0; c3 < size - 1; c3++) {
          var v = mGrid[r3][c3];
          if (v === mGrid[r3][c3 + 1] && v === mGrid[r3 + 1][c3] && v === mGrid[r3 + 1][c3 + 1]) score += 3;
        }
      }
      return score;
    }

    var bestGrid = null;
    var bestScore = Infinity;
    for (var mask = 0; mask < 8; mask++) {
      var cand = grid.map(function (row, rr) {
        return row.map(function (cell, cc) {
          return isFunc[rr][cc] ? cell : (cell ^ (maskBit(mask, rr, cc) ? 1 : 0));
        });
      });
      applyFormat(cand, mask);
      var sc = penalty(cand);
      if (sc < bestScore) {
        bestScore = sc;
        bestGrid = cand;
      }
    }
    return bestGrid;
  }

  function generateMatrix(text) {
    var bytes = toUtf8Bytes(text);
    for (var v = 1; v < SPECS_M.length; v++) {
      var spec = SPECS_M[v];
      var dataCap = 0;
      for (var g = 0; g < spec.blocks.length; g++) dataCap += spec.blocks[g][0] * spec.blocks[g][1];
      var headerBits = 4 + (v >= 10 ? 16 : 8);
      if (headerBits + bytes.length * 8 <= dataCap * 8) {
        var dc = makeDataCodewords(bytes, v, dataCap);
        var cw = interleaveBlocks(dc, spec);
        return buildMatrix(cw, spec, v);
      }
    }
    throw new Error('Texto muito longo para QR code');
  }

  function renderSvg(text, pixelSize) {
    var matrix = generateMatrix(text);
    var n = matrix.length;
    var quiet = 4;
    var dim = n + quiet * 2;
    var px = Number(pixelSize || 184);
    var path = '';
    for (var r = 0; r < n; r++) {
      for (var c = 0; c < n; c++) {
        if (matrix[r][c] === 1) {
          path += 'M' + (c + quiet) + ',' + (r + quiet) + 'h1v1h-1z';
        }
      }
    }
    return '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ' + dim + ' ' + dim + '" width="' + px + '" height="' + px + '" shape-rendering="crispEdges" role="img" aria-label="QR Code">' +
      '<rect width="100%" height="100%" fill="#ffffff"/>' +
      '<path d="' + path + '" fill="#000000"/>' +
      '</svg>';
  }

  root.NuvioQR = {
    generateMatrix: generateMatrix,
    renderSvg: renderSvg
  };
})(typeof window !== 'undefined' ? window : globalThis);

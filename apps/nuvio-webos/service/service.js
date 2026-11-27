'use strict';

const fs = require('fs');
const path = require('path');
const http = require('http');
const https = require('https');
const zlib = require('zlib');
const { URL } = require('url');

const MAX_REDIRECTS = 5;
const MAX_BODY_BYTES = 16 * 1024 * 1024;
const DEFAULT_TIMEOUT_MS = 20000;

function resolveStorageDir() {
  const candidates = [
    process.env.NUVIO_WEBOS_DATA_DIR,
    '/media/internal/.nuvio-tv',
    path.join(__dirname, '.data'),
    '/tmp/nuvio-tv',
  ].filter(Boolean);

  for (const dir of candidates) {
    try {
      fs.mkdirSync(dir, { recursive: true });
      fs.accessSync(dir, fs.constants.W_OK);
      return dir;
    } catch (_) {}
  }
  return null;
}

const DATA_DIR = resolveStorageDir();
const KV_FILE = DATA_DIR ? path.join(DATA_DIR, 'kv.json') : null;
let memoryKv = {};

function loadKv() {
  if (!KV_FILE) return memoryKv;
  try {
    const raw = fs.readFileSync(KV_FILE, 'utf8');
    memoryKv = JSON.parse(raw) || {};
  } catch (_) {}
  return memoryKv;
}

function saveKv(data) {
  memoryKv = data;
  if (!KV_FILE) return;
  const tmp = `${KV_FILE}.tmp`;
  fs.writeFileSync(tmp, JSON.stringify(data), 'utf8');
  fs.renameSync(tmp, KV_FILE);
}

loadKv();

function httpFetch(params) {
  return new Promise((resolve) => {
    const rawUrl = String((params && params.url) || '').trim();
    if (!/^https?:\/\//i.test(rawUrl)) {
      resolve({ returnValue: false, ok: false, status: 0, errorText: 'Invalid URL', headers: {}, body: '' });
      return;
    }

    const method = String((params && params.method) || 'GET').toUpperCase();
    const reqHeaders = Object.assign({ 'User-Agent': 'Nuvio-webOS/1.0' }, (params && params.headers) || {});
    const reqBody = params && params.body != null ? String(params.body) : '';
    const timeoutMs = Math.max(1000, Math.min(60000, Number(params && params.timeoutMs) || DEFAULT_TIMEOUT_MS));

    function perform(targetUrl, redirectsLeft) {
      let parsed;
      try {
        parsed = new URL(targetUrl);
      } catch (e) {
        resolve({ returnValue: false, ok: false, status: 0, errorText: String(e), headers: {}, body: '' });
        return;
      }

      const transport = parsed.protocol === 'https:' ? https : http;
      const headers = Object.assign({}, reqHeaders);
      if (reqBody && !headers['Content-Length'] && !headers['content-length']) {
        headers['Content-Length'] = Buffer.byteLength(reqBody);
      }

      const req = transport.request(
        {
          protocol: parsed.protocol,
          hostname: parsed.hostname,
          port: parsed.port || undefined,
          path: `${parsed.pathname}${parsed.search}`,
          method,
          headers,
          timeout: timeoutMs,
        },
        (res) => {
          const status = res.statusCode || 0;
          const loc = res.headers.location;
          if ([301, 302, 303, 307, 308].includes(status) && loc && redirectsLeft > 0) {
            res.resume();
            try {
              const nextUrl = new URL(loc, parsed).toString();
              perform(nextUrl, redirectsLeft - 1);
            } catch (e) {
              resolve({ returnValue: false, ok: false, status, errorText: String(e), headers: {}, body: '' });
            }
            return;
          }

          const chunks = [];
          let total = 0;
          res.on('data', (chunk) => {
            total += chunk.length;
            if (total <= MAX_BODY_BYTES) chunks.push(chunk);
          });
          res.on('end', () => {
            const outHeaders = {};
            for (const [k, v] of Object.entries(res.headers || {})) {
              outHeaders[k.toLowerCase()] = Array.isArray(v) ? v.join(', ') : String(v);
            }
            let rawBuf = Buffer.concat(chunks);
            const enc = (outHeaders['content-encoding'] || '').toLowerCase();
            try {
              if (enc.includes('gzip')) rawBuf = zlib.gunzipSync(rawBuf);
              else if (enc.includes('deflate')) rawBuf = zlib.inflateSync(rawBuf);
              else if (enc.includes('br') && zlib.brotliDecompressSync) rawBuf = zlib.brotliDecompressSync(rawBuf);
            } catch (_) {}
            resolve({
              returnValue: true,
              ok: status >= 200 && status < 300,
              status,
              url: targetUrl,
              headers: outHeaders,
              body: rawBuf.toString('utf8'),
            });
          });
        }
      );

      req.on('timeout', () => {
        req.destroy(new Error('Request timed out'));
      });
      req.on('error', (err) => {
        resolve({ returnValue: false, ok: false, status: 0, errorText: err.message || String(err), headers: {}, body: '' });
      });

      if (reqBody && method !== 'GET' && method !== 'HEAD') {
        req.write(reqBody);
      }
      req.end();
    }

    perform(rawUrl, MAX_REDIRECTS);
  });
}

function kvGet(params) {
  const key = String((params && params.key) || '');
  const data = loadKv();
  return {
    returnValue: true,
    value: Object.prototype.hasOwnProperty.call(data, key) ? data[key] : null,
  };
}

function kvSet(params) {
  const key = String((params && params.key) || '');
  if (!key) return { returnValue: false, errorText: 'Missing key' };
  const data = loadKv();
  if (params.value === null || params.value === undefined) {
    delete data[key];
  } else {
    data[key] = params.value;
  }
  saveKv(data);
  return { returnValue: true };
}

function kvDelete(params) {
  const key = String((params && params.key) || '');
  const data = loadKv();
  delete data[key];
  saveKv(data);
  return { returnValue: true };
}

function startLunaService() {
  const Service = require('webos-service');
  const service = new Service('com.nuvio.tv.service');

  service.register('ping', (message) => {
    message.respond({ returnValue: true, node: process.version, storage: DATA_DIR || 'memory' });
  });

  service.register('httpFetch', async (message) => {
    const out = await httpFetch(message.payload || {});
    message.respond(out);
  });

  service.register('kvGet', (message) => {
    message.respond(kvGet(message.payload || {}));
  });

  service.register('kvSet', (message) => {
    try {
      message.respond(kvSet(message.payload || {}));
    } catch (e) {
      message.respond({ returnValue: false, errorText: e.message || String(e) });
    }
  });

  service.register('kvDelete', (message) => {
    try {
      message.respond(kvDelete(message.payload || {}));
    } catch (e) {
      message.respond({ returnValue: false, errorText: e.message || String(e) });
    }
  });
}

function startLocalDevServer(port) {
  const distApp = path.resolve(__dirname, '..', 'dist', 'app');
  const MIME = {
    '.html': 'text/html; charset=utf-8',
    '.js': 'application/javascript; charset=utf-8',
    '.css': 'text/css; charset=utf-8',
    '.json': 'application/json; charset=utf-8',
    '.svg': 'image/svg+xml',
    '.png': 'image/png',
    '.ttf': 'font/ttf',
    '.ico': 'image/x-icon',
  };

  function readJsonBody(req) {
    return new Promise((resolve) => {
      const chunks = [];
      req.on('data', (c) => chunks.push(c));
      req.on('end', () => {
        try {
          resolve(JSON.parse(Buffer.concat(chunks).toString('utf8') || '{}'));
        } catch (_) {
          resolve({});
        }
      });
    });
  }

  const server = http.createServer(async (req, res) => {
    const u = new URL(req.url, `http://127.0.0.1:${port}`);
    if (req.method === 'POST' && u.pathname.startsWith('/__webos/')) {
      const body = await readJsonBody(req);
      let payload = { returnValue: false, errorText: 'Unknown endpoint' };
      if (u.pathname === '/__webos/httpFetch') payload = await httpFetch(body);
      else if (u.pathname === '/__webos/kvGet') payload = kvGet(body);
      else if (u.pathname === '/__webos/kvSet') payload = kvSet(body);
      else if (u.pathname === '/__webos/kvDelete') payload = kvDelete(body);
      res.writeHead(200, { 'Content-Type': 'application/json; charset=utf-8' });
      res.end(JSON.stringify(payload));
      return;
    }

    const reqPath = u.pathname === '/' ? '/index.html' : decodeURIComponent(u.pathname);
    const full = path.resolve(distApp, `.${reqPath}`);
    if (!full.startsWith(distApp) || !fs.existsSync(full) || fs.statSync(full).isDirectory()) {
      res.writeHead(404);
      res.end('Not found');
      return;
    }
    const ext = path.extname(full).toLowerCase();
    res.writeHead(200, { 'Content-Type': MIME[ext] || 'application/octet-stream' });
    fs.createReadStream(full).pipe(res);
  });

  server.listen(port, '127.0.0.1', () => {
    console.log(`[nuvio-webos dev] Abra http://127.0.0.1:${port} no navegador para testar.`);
  });
}

if (process.argv.includes('--dev')) {
  const portIdx = process.argv.indexOf('--port');
  const port = portIdx >= 0 ? Number(process.argv[portIdx + 1]) || 8787 : 8787;
  startLocalDevServer(port);
} else {
  startLunaService();
}

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");

const source = fs.readFileSync(path.join(__dirname, "../static/regional-terrain-request.js"), "utf8");
const { createRegionalTerrainRequests, RegionalTerrainLoadError } = Function(
  `${source.replaceAll("export ", "")}\nreturn { createRegionalTerrainRequests, RegionalTerrainLoadError };`)();
const deferred = () => { let resolve;
  const promise = new Promise(yes => { resolve = yes; }); return { promise, resolve }; };
const turn = () => new Promise(resolve => setImmediate(resolve));
const request = (latitude = 50_500_000) => ({ source: "a".repeat(64), scale: "neighborhood",
  origin: { latitude, longitude: 10_500_000 } });
const product = input => ({ source: input.source,
  request: { origin: input.origin, scale: input.scale }, vertices: Array(65 * 65).fill(null) });
const response = value => ({ ok: true, json: async () => value });
const forUrl = url => { const parsed = new URL(url, "https://fixture.invalid");
  return request(Number(parsed.searchParams.get("latitude"))); };

test("warm reopening reuses the installed terrain without fetching or reinstalling", async () => {
  let fetches = 0;
  const installed = [], states = [];
  const terrain = createRegionalTerrainRequests({ runtimePromise: Promise.resolve(),
    fetchTerrain: async () => { fetches++; return response(product(request())); },
    install: value => installed.push(value), changed: state => states.push(state) });
  assert.equal((await terrain.request(request())).status, "prepared");
  terrain.cancel();
  assert.equal((await terrain.request(request())).status, "reused");
  assert.equal(fetches, 1); assert.equal(installed.length, 1);
  assert.equal(terrain.state.phase, "prepared");
  assert.ok(states.every(Object.isFrozen));
});

test("identical in-flight requests share ownership and wait for the existing runtime", async () => {
  const runtime = deferred(), server = deferred(), installed = [];
  const input = request();
  const terrain = createRegionalTerrainRequests({ runtimePromise: runtime.promise,
    fetchTerrain: () => server.promise, install: value => installed.push(value) });
  const first = terrain.request(input);
  assert.equal(terrain.request(request()), first);
  input.origin.latitude = 1; // Caller mutations cannot change the admitted window.
  server.resolve(response(product(request()))); await turn();
  assert.equal(installed.length, 0); assert.equal(terrain.state.phase, "loading");
  runtime.resolve();
  assert.equal((await first).status, "prepared");
  assert.equal(installed[0].request.origin.latitude, 50_500_000);
});

test("obsolete responses cannot replace the latest camera window", async () => {
  const oldJson = deferred(), installed = [], signals = [];
  const old = request(), latest = request(50_600_000);
  const terrain = createRegionalTerrainRequests({ runtimePromise: Promise.resolve(),
    fetchTerrain: (url, { signal }) => { signals.push(signal);
      return forUrl(url).origin.latitude === old.origin.latitude
        ? { ok: true, json: () => oldJson.promise } : response(product(latest)); },
    install: value => installed.push(value) });
  const abandoned = terrain.request(old); await turn();
  assert.equal((await terrain.request(latest)).status, "prepared");
  oldJson.resolve(product(old));
  assert.equal((await abandoned).status, "superseded");
  assert.equal(signals[0].aborted, true);
  assert.deepEqual(installed, [product(latest)]);
});

test("hiding cancels pending work and retains the last installed window", async () => {
  const pending = deferred(), installed = [], signals = [];
  const terrain = createRegionalTerrainRequests({ runtimePromise: Promise.resolve(),
    fetchTerrain: (url, { signal }) => { signals.push(signal);
      return forUrl(url).origin.latitude === 50_500_000
        ? response(product(request())) : pending.promise; },
    install: value => installed.push(value) });
  await terrain.request(request());
  const abandoned = terrain.request(request(50_600_000)); await turn();
  terrain.cancel();
  assert.equal(signals[1].aborted, true);
  pending.resolve(response(product(request(50_600_000))));
  assert.equal((await abandoned).status, "superseded");
  assert.equal(terrain.state.request.origin.latitude, 50_500_000);
  assert.equal((await terrain.request(request())).status, "reused");
  assert.equal(installed.length, 1);
});

test("busy terrain does not retry every frame; explicit retry can recover", async () => {
  let fetches = 0;
  const terrain = createRegionalTerrainRequests({ runtimePromise: Promise.resolve(), install() {},
    fetchTerrain: () => ++fetches === 1 ? { ok: false, status: 503 } : response(product(request())) });
  const failed = await terrain.request(request());
  assert.equal(failed.cause.code, "map/terrain-http");
  assert.equal((await terrain.request(request())).cause, failed.cause);
  assert.equal(fetches, 1);
  assert.equal((await terrain.retry()).status, "prepared");
  assert.equal(fetches, 2);
});

test("recent windows have bounded least-recently-used retention", async () => {
  let fetches = 0;
  const terrain = createRegionalTerrainRequests({ runtimePromise: Promise.resolve(), install() {},
    fetchTerrain: url => { fetches++; return response(product(forUrl(url))); } });
  for (let latitude = 1; latitude <= 4; latitude++) await terrain.request(request(latitude));
  assert.equal((await terrain.request(request(1))).status, "reused");
  await terrain.request(request(5));
  assert.equal((await terrain.request(request(2))).status, "prepared");
  assert.equal(fetches, 6);
});

test("invalid origins and mismatched or truncated windows never reach the renderer", async () => {
  let installations = 0;
  const malformed = [
    { ...product(request()), source: "b".repeat(64) },
    { ...product(request()), request: { ...product(request()).request, scale: "country" } },
    { ...product(request()), request: { ...product(request()).request, origin: request(4).origin } },
    { ...product(request()), vertices: [null] },
  ];
  for (const value of malformed) {
    const terrain = createRegionalTerrainRequests({ runtimePromise: Promise.resolve(),
      install() { installations++; }, fetchTerrain: () => response(value) });
    assert.equal((await terrain.request(request())).cause.code, "map/terrain-response");
  }
  const terrain = createRegionalTerrainRequests({ runtimePromise: Promise.resolve(), install() {} });
  for (const value of [request(90_000_001), request(NaN), { ...request(), scale: "city" },
    { ...request(), source: "A".repeat(64) }]) {
    assert.throws(() => terrain.request(value), RegionalTerrainLoadError);
  }
  assert.equal(installations, 0); assert.equal(terrain.state.phase, "idle");
});

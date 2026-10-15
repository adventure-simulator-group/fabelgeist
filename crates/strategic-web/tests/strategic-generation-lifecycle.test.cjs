const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");

function fixture() {
  const instances = [], timers = new Map();
  let sequence = 0, nextTimer = 0, fail = false, hold = false;
  const createGenerationPool = module => {
    const pool = { module, closed: false, runs: 0, pending: null,
      async run(jobs, receive) {
        assert.equal(this.closed, false);
        this.runs++;
        if (hold) await new Promise((resolve, reject) => { this.pending = { resolve, reject }; });
        if (fail) throw Error("generation failed");
        for (const job of jobs) receive({ job, bytes: new Uint8Array([1]), workerMilliseconds: 1 });
        return { createdWorkers: this.runs === 1 ? 1 : 0, initializationMilliseconds: 0 };
      },
      close() { this.closed = true; this.pending?.reject(Error("cancelled")); },
    };
    instances.push(pool); return pool;
  };
  const source = fs.readFileSync(path.join(__dirname, "../static/strategic-generation.js"), "utf8")
    .replace(/^import .*;\r?\n/gm, "").replaceAll("export async function", "async function");
  const state = {};
  const api = new Function("createGenerationPool", "openGeneratedCache", "window", "setTimeout", "clearTimeout",
    `${source}\nreturn {scene:prepareGeneratedScene,city:prepareRegionalCity};`)(
    createGenerationPool,
    async () => ({ get() {}, put() { return { status: "accepted", disposition: "inserted" }; }, close: async () => {} }),
    state,
    callback => { const id = ++nextTimer; timers.set(id, callback); return id; },
    id => timers.delete(id),
  );
  const runtime = { generationModule: {}, generationRevision: "fixture",
    wasm_begin_generation: owner => JSON.stringify({ owner: JSON.parse(owner), sequence: ++sequence }),
    wasm_finish_generation() {}, wasm_finish_regional_city() {}, wasm_cancel_generation() {},
    wasm_generation_jobs: () => '["1"]', wasm_venue_jobs: () => "[]", wasm_regional_city_jobs: () => '["2"]',
    wasm_receive_job() {}, wasm_generation_dependencies: () => new Uint8Array(),
  };
  return { api, runtime, instances, timers, state,
    setFailure(value) { fail = value; }, setHold(value) { hold = value; } };
}

test("scene and map arrivals reuse one idle pool and expire it", async () => {
  const f = fixture();
  await f.api.scene(f.runtime, "opaque input", []);
  await f.api.city(f.runtime, "opaque city");
  assert.equal(f.instances.length, 1);
  assert.equal(f.instances[0].runs, 2);
  assert.equal(f.state.strategicGenerationMetrics["regional-map"].workers, 0);
  assert.equal(f.timers.size, 1);
  [...f.timers.values()][0]();
  assert.equal(f.instances[0].closed, true);
  await f.api.scene(f.runtime, "another input", []);
  assert.equal(f.instances.length, 2);
});

test("a changed Wasm module cannot reuse workers from an earlier module", async () => {
  const f = fixture();
  await f.api.scene(f.runtime, "input", []);
  f.runtime.generationModule = {};
  await f.api.scene(f.runtime, "input", []);
  assert.equal(f.instances.length, 2);
  assert.equal(f.instances[0].closed, true);
  assert.equal(f.instances[1].module, f.runtime.generationModule);
});

test("a failed lease is discarded before the next arrival", async () => {
  const f = fixture();
  await f.api.scene(f.runtime, "input", []);
  f.setFailure(true);
  await assert.rejects(f.api.scene(f.runtime, "input", []), /generation failed/);
  assert.equal(f.instances[0].closed, true);
  assert.equal(f.timers.size, 0);
  f.setFailure(false);
  await f.api.scene(f.runtime, "input", []);
  assert.equal(f.instances.length, 2);
});

test("overlapping owners lease separate pools and retain only one idle result", async () => {
  const f = fixture();
  f.setHold(true);
  const scene = f.api.scene(f.runtime, "input", []);
  const city = f.api.city(f.runtime, "city");
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(f.instances.length, 2);
  f.instances[0].pending.resolve(); await scene;
  f.instances[1].pending.resolve(); await city;
  assert.equal(f.instances[0].closed, true);
  assert.equal(f.instances[1].closed, false);
  assert.equal(f.timers.size, 1);
});

test("cancelling a lease cannot poison the next arrival", async () => {
  const f = fixture(), controller = new AbortController();
  f.setHold(true);
  const running = f.api.scene(f.runtime, "input", [], { signal: controller.signal });
  const rejected = assert.rejects(running, /cancelled/);
  await new Promise(resolve => setImmediate(resolve));
  controller.abort(); await rejected;
  assert.equal(f.instances[0].closed, true);
  assert.equal(f.timers.size, 0);
  f.setHold(false);
  await f.api.city(f.runtime, "city");
  assert.equal(f.instances.length, 2);
  assert.equal(f.instances[1].closed, false);
});

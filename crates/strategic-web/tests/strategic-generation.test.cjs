const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");
const pool = import("data:text/javascript;base64," + fs.readFileSync(path.join(__dirname,
  "../static/strategic-generation-pool.js")).toString("base64"));

function workers(fail = false) {
  const instances = [];
  return { instances, createWorker() {
    const worker = { terminated: false, initialized: 0, postMessage(message) {
      queueMicrotask(() => {
        if (this.terminated) return;
        if (message.kind === "initialize") {
          this.initialized++; this.onmessage({ data: { kind: "ready", dispatch: message.dispatch } });
        } else this.onmessage({ data: fail ? { kind: "failed", dispatch: message.dispatch,
          error: { code: "generation/building", message: "generation failed" } }
          : { kind: "generated", dispatch: message.dispatch,
            bytes: new Uint8Array([Number(message.job)]), milliseconds: 1 } });
      });
    }, terminate() { this.terminated = true; } };
    instances.push(worker); return worker;
  } };
}

test("bounded workers serve dependent phases without restarting their runtimes", async () => {
  const mock = workers(); const results = [];
  const { createGenerationPool } = await pool;
  const generation = createGenerationPool({}, { createWorker: mock.createWorker, hardwareConcurrency: 32 });
  const receive = ({ job, bytes }) => { assert.equal(bytes[0], Number(job)); results.push(Number(job)); };
  assert.deepEqual(await generation.run([1, 2, 3, 4, 5, 6].map(String), receive),
    { status: "completed", createdWorkers: 4 });
  assert.equal((await generation.run(["7"], receive)).createdWorkers, 0);
  assert.equal((await generation.run(["8", "9"], receive)).createdWorkers, 0);
  assert.deepEqual(results.sort((a, b) => a - b), [1, 2, 3, 4, 5, 6, 7, 8, 9]);
  assert.ok(mock.instances.every(worker => worker.initialized === 1 && !worker.terminated));
  generation.close();
  assert.ok(mock.instances.every(worker => worker.terminated));
  await assert.rejects(generation.run([], receive), /not available/);
});

test("generation and installation errors reject readiness and terminate workers", async () => {
  const { createGenerationPool } = await pool;
  for (const fail of [true, false]) {
    const mock = workers(fail);
    const generation = createGenerationPool({}, { createWorker: mock.createWorker, hardwareConcurrency: 3 });
    await assert.rejects(generation.run(["1", "2", "3"], () => { throw new Error("install failed"); }),
      error => fail ? error.code === "generation/building" && error.message === "generation failed"
        : error.message === "install failed");
    assert.ok(mock.instances.every(worker => worker.terminated));
  }
});

test("empty and fully cached phases start no workers", async () => {
  const { createGenerationPool } = await pool;
  const generation = createGenerationPool({}, {
    createWorker() { throw new Error("unnecessary worker"); }, hardwareConcurrency: 1,
  });
  assert.equal((await generation.run([], () => {})).createdWorkers, 0);
  assert.equal((await generation.run(["1", "2"], () => { throw Error("cached delivery"); }, {
    resolveJob: () => ({ status: "reused" }),
  })).createdWorkers, 0);
  generation.close();
});

test("a miss generates while an unrelated cache read is still pending", async () => {
  const { createGenerationPool } = await pool;
  const mock = workers();
  const generation = createGenerationPool({}, { createWorker: mock.createWorker, hardwareConcurrency: 3 });
  let releaseRead;
  const reading = new Promise(resolve => { releaseRead = resolve; });
  const results = [];
  const running = generation.run(["1", "2"], ({ job }) => {
    results.push(Number(job)); releaseRead({ status: "reused" });
  }, { resolveJob: job => job === "1" ? { status: "generate", job } : reading });
  assert.equal((await running).createdWorkers, 1);
  assert.deepEqual(results, [1]);
  generation.close();
  assert.ok(mock.instances.every(worker => worker.terminated));
});

test("dependency failures close the pool and a pending phase cannot overlap another", async () => {
  const { createGenerationPool } = await pool;
  const mock = workers();
  const generation = createGenerationPool({}, { createWorker: mock.createWorker, hardwareConcurrency: 3 });
  const running = generation.run(["1"], () => {}, {
    dependencies() { throw Error("missing dependency"); },
  });
  await assert.rejects(generation.run(["2"], () => {}), error => error.code === "busy");
  await assert.rejects(running, /missing dependency/);
  assert.ok(mock.instances.every(worker => worker.terminated));
});

test("a later miss lazily fills worker slots left unused by cached jobs", async () => {
  const { createGenerationPool } = await pool;
  const mock = workers();
  const generation = createGenerationPool({}, { createWorker: mock.createWorker, hardwareConcurrency: 3 });
  assert.equal((await generation.run(["1", "2"], () => {}, {
    resolveJob: job => job === "1" ? { status: "reused" } : { status: "generate", job },
  })).createdWorkers, 1);
  assert.equal((await generation.run(["3", "4"], () => {})).createdWorkers, 1);
  assert.equal(mock.instances.length, 2);
  generation.close();
  assert.ok(mock.instances.every(worker => worker.terminated));
});

test("close cancels worker initialization and detaches pending handlers", async () => {
  const { createGenerationPool } = await pool;
  const worker = { postMessage() {}, terminate() { this.terminated = true; } };
  const generation = createGenerationPool({}, { createWorker: () => worker, hardwareConcurrency: 2 });
  const running = generation.run(["1"], () => { throw new Error("late delivery"); });
  const rejected = assert.rejects(running, error => error.code === "cancelled");
  generation.close();
  await rejected;
  assert.equal(worker.terminated, true);
  assert.equal(worker.onmessage, null);
  assert.equal(worker.onerror, null);
  assert.equal(worker.onmessageerror, null);
  await assert.rejects(generation.run([], () => {}), error => error.code === "closed");
});

test("closing during cache resolution prevents any worker creation or delivery", async () => {
  const { createGenerationPool } = await pool;
  let resolve;
  const generation = createGenerationPool({}, {
    createWorker() { throw new Error("unnecessary worker"); }, hardwareConcurrency: 2,
  });
  const running = generation.run(["1"], () => { throw new Error("late delivery"); }, {
    resolveJob: () => new Promise(done => { resolve = done; }),
  });
  const rejected = assert.rejects(running, error => error.code === "cancelled");
  generation.close();
  resolve({ status: "generate", job: "1" });
  await rejected;
});

test("a preparation failure prevents a pending cache hit from installing later", async () => {
  const { createGenerationPool } = await pool;
  const source = fs.readFileSync(path.join(__dirname, "../static/strategic-generation.js"), "utf8")
    .replace(/^import .*;\r?\n/gm, "").replace("export async function", "async function");
  const mock = workers(true), received = [];
  let finishLookup, closed = false;
  const jobs = ['{"Building":{"seed":18446744073709551615}}',
    '{"Building":{"seed":18446744073709551614}}'];
  const cache = {
    get: job => job === jobs[0] ? new Promise(resolve => { finishLookup = resolve; }) : undefined,
    remove() { throw new Error("cancelled lookup must not evict products"); },
    put() { throw new Error("failed preparation must not persist products"); },
    close() { closed = true; return Promise.resolve({ status: "flushed" }); },
  };
  const prepare = new Function("createGenerationPool", "openGeneratedCache", "window",
    `${source}\nreturn prepareGeneratedScene;`)(
    module => createGenerationPool(module, { hardwareConcurrency: 3, createWorker: mock.createWorker }),
    async () => cache, {});
  const runtime = { generationModule: {}, generationRevision: "fixture",
    wasm_begin_generation() { return JSON.stringify({owner:"scene",sequence:1}); }, wasm_finish_generation() {}, wasm_cancel_generation() {}, wasm_generation_jobs: () => JSON.stringify(jobs),
    wasm_venue_jobs: () => "[]", wasm_landscape_jobs: () => "[]",
    wasm_generation_dependencies: () => new Uint8Array(),
    wasm_receive_job(preparation, job) { received.push(job); },
  };
  await assert.rejects(prepare(runtime, "opaque scene", []), error => error.code === "generation/building");
  assert.equal(closed, true);
  finishLookup(new Uint8Array([1, 2, 3]));
  await new Promise(resolve => setImmediate(resolve));
  assert.deepEqual(received, []);
  assert.ok(mock.instances.every(worker => worker.terminated));
});

test("cached product rejection regenerates corruption and preserves preparation failures", async () => {
  const { createGenerationPool } = await pool;
  const source = fs.readFileSync(path.join(__dirname, "../static/strategic-generation.js"), "utf8")
    .replace(/^import .*;\r?\n/gm, "").replace("export async function", "async function");
  for (const code of ["generation/product-decode", "generation/product-mismatch",
    "generation/scene-bindings-mismatch", "generation/residency-poisoned",
    "generation/not-prepared", "generation/scene-input", "generation/stale-preparation",
    "generation/preparation-owner", "generation/preparation-input", "unexpected-host-failure"]) {
    const corrupt = ["generation/product-decode", "generation/product-mismatch",
      "generation/scene-bindings-mismatch"].includes(code);
    const rejection = new Error("fixture rejection"); rejection.name = code;
    const mock = workers(), state = {}, installed = [];
    const job = '{"Building":{"seed":18446744073709551615}}';
    let removed = 0, persisted = 0, closed = 0;
    const cache = { get: () => new Uint8Array([9]),
      remove(address) { assert.equal(address, job); removed++; },
      put(address) { assert.equal(address, job); persisted++; return { status: "accepted", disposition: "inserted" }; },
      close() { closed++; return Promise.resolve({ status: "flushed" }); } };
    const prepare = new Function("createGenerationPool", "openGeneratedCache", "window",
      `${source}\nreturn prepareGeneratedScene;`)(
      module => createGenerationPool(module, { hardwareConcurrency: 2, createWorker: mock.createWorker }),
      async () => cache, state);
    const runtime = { generationModule: {}, generationRevision: "fixture",
      wasm_begin_generation() { return JSON.stringify({owner:"scene",sequence:1}); }, wasm_finish_generation() {}, wasm_cancel_generation() {}, wasm_generation_jobs: () => JSON.stringify([job]),
      wasm_venue_jobs: () => "[]", wasm_landscape_jobs: () => "[]",
      wasm_generation_dependencies: () => new Uint8Array(),
      wasm_receive_job(preparation, address, bytes) {
        if (bytes[0] === 9) throw rejection;
        installed.push(address);
      } };
    const running = prepare(runtime, "opaque scene", []);
    if (corrupt) {
      await running;
      assert.deepEqual(installed, [job]);
      assert.equal(state.strategicGenerationMetrics.scene.cacheHits, 0);
      assert.equal(state.strategicGenerationMetrics.scene.cacheMisses, 1);
    } else {
      await assert.rejects(running, error => error === rejection);
      assert.deepEqual(installed, []);
    }
    assert.equal(removed, corrupt ? 1 : 0);
    assert.equal(persisted, corrupt ? 1 : 0);
    assert.equal(mock.instances.length, corrupt ? 1 : 0);
    assert.equal(closed, 1);
  }
});

test("destination cancellation prevents installation during cache open, lookup and worker generation", async () => {
  const { createGenerationPool } = await pool;
  const source = fs.readFileSync(path.join(__dirname, "../static/strategic-generation.js"), "utf8")
    .replace(/^import .*;\r?\n/gm, "").replace("export async function", "async function");
  for (const stage of ["open", "lookup", "worker"]) {
    const controller = new AbortController();
    let finishOpen, finishLookup, lateReply, closed = 0, terminated = 0, installed = 0;
    const job = '{"Building":{"seed":18446744073709551615}}';
    const cache = {
      get: () => stage === "lookup" ? new Promise(resolve => { finishLookup = resolve; }) : undefined,
      put() { throw Error("Cancelled product persisted"); },
      remove() { throw Error("Cancelled product evicted"); },
      close() { closed++; return Promise.resolve({ status: "flushed" }); },
    };
    const worker = { postMessage(message) {
      if (message.kind === "initialize") queueMicrotask(() => this.onmessage({
        data: { kind: "ready", dispatch: message.dispatch } }));
      else { const handler = this.onmessage; lateReply = () => handler({ data: {
        kind: "generated", dispatch: message.dispatch, bytes: new Uint8Array([1]), milliseconds: 1 } }); }
    }, terminate() { terminated++; } };
    const prepare = Function("createGenerationPool", "openGeneratedCache", "window",
      `${source}\nreturn prepareGeneratedScene;`)(
      module => createGenerationPool(module, { hardwareConcurrency: 2, createWorker: () => worker }),
      () => stage === "open" ? new Promise(resolve => { finishOpen = resolve; }) : Promise.resolve(cache), {});
    const runtime = { generationModule: {}, generationRevision: "fixture",
      wasm_begin_generation() { return JSON.stringify({owner:"scene",sequence:1}); }, wasm_finish_generation() {}, wasm_cancel_generation() {}, wasm_generation_jobs: () => JSON.stringify([job]),
      wasm_venue_jobs: () => "[]", wasm_landscape_jobs: () => "[]",
      wasm_generation_dependencies: () => new Uint8Array(),
      wasm_receive_job(preparation) { installed++; },
    };
    const running = prepare(runtime, "opaque scene", [], { signal: controller.signal });
    const rejected = assert.rejects(running, error => error.name === "AbortError" || error.code === "cancelled");
    await new Promise(resolve => setImmediate(resolve));
    controller.abort();
    finishOpen?.(cache); finishLookup?.(new Uint8Array([1]));
    await rejected; lateReply?.();
    assert.equal(installed, 0); assert.equal(closed, 1);
    assert.equal(terminated, stage === "worker" ? 1 : 0);
    assert.equal(worker.onmessage, stage === "worker" ? null : undefined);
  }
});

test("a stale dispatch cannot satisfy a later job on the same worker", async () => {
  const { createGenerationPool } = await pool;
  let previousDispatch;
  const worker = { postMessage(message) {
    queueMicrotask(() => {
      if (message.kind === "initialize") {
        previousDispatch = message.dispatch;
        this.onmessage({ data: { kind: "ready", dispatch: message.dispatch } });
      } else {
        this.onmessage({ data: { kind: "generated", dispatch: previousDispatch,
          bytes: new Uint8Array([99]), milliseconds: 1 } });
        previousDispatch = message.dispatch;
        this.onmessage({ data: { kind: "generated", dispatch: message.dispatch,
          bytes: new Uint8Array([Number(message.job)]), milliseconds: 2 } });
      }
    });
  }, terminate() {} };
  const generation = createGenerationPool({}, { createWorker: () => worker, hardwareConcurrency: 2 });
  const delivered = [];
  await generation.run(["1", "2"], product => delivered.push({ job: product.job,
    bytes: Array.from(product.bytes), workerMilliseconds: product.workerMilliseconds }));
  assert.deepEqual(delivered, [
    { job: "1", bytes: [1], workerMilliseconds: 2 },
    { job: "2", bytes: [2], workerMilliseconds: 2 },
  ]);
  generation.close();
});

test("invalid capacity, resolution and job addresses fail with stable causes", async () => {
  const { createGenerationPool } = await pool;
  for (const hardwareConcurrency of [0, -1, 1.5, NaN, Infinity, "4"]) {
    assert.throws(() => createGenerationPool({}, { hardwareConcurrency }),
      error => error.code === "invalid-capacity");
  }
  for (const [jobs, resolveJob, expected] of [
    [42, undefined, "invalid-jobs"], [[1], undefined, "invalid-job"],
    [["1"], () => null, "invalid-resolution"],
    [["1"], () => ({ status: "generate", job: 1 }), "invalid-job"],
    [["1"], () => ({ status: "generate", job: "other" }), "job-address-mismatch"],
  ]) {
    const generation = createGenerationPool({}, { hardwareConcurrency: 2,
      createWorker() { throw new Error("unnecessary worker"); } });
    await assert.rejects(generation.run(jobs, () => {}, { resolveJob }), error => error.code === expected);
    generation.close();
  }
});

test("malformed results reject readiness and terminate the pool", async () => {
  const { createGenerationPool } = await pool;
  for (const malformed of [
    () => ({ kind: "ready" }),
    dispatch => ({ kind: "generated", dispatch, bytes: new Uint8Array(), milliseconds: 1 }),
    dispatch => ({ kind: "failed", dispatch, error: "unstructured" }),
    dispatch => ({ kind: "ready", dispatch: dispatch + 0.5 }),
  ]) {
    const worker = { postMessage(message) {
      queueMicrotask(() => this.onmessage({ data: malformed(message.dispatch) }));
    }, terminate() { this.terminated = true; } };
    const generation = createGenerationPool({}, { hardwareConcurrency: 2, createWorker: () => worker });
    await assert.rejects(generation.run(["1"], () => {}), error => error.code === "worker-protocol");
    assert.equal(worker.terminated, true);
  }
});

test("dependency transfer consumes its allocation and preserves full-width job text", async () => {
  const { createGenerationPool } = await pool;
  const address = '{"seed":18446744073709551615}';
  const source = new Uint8Array([0, 128, 255]);
  let actual;
  const worker = { postMessage(message, transfer) {
    const owned = structuredClone(message, { transfer });
    queueMicrotask(() => {
      if (owned.kind === "initialize") this.onmessage({ data: { kind: "ready", dispatch: owned.dispatch } });
      else {
        actual = { job: owned.job, dependencies: Array.from(owned.dependencies) };
        this.onmessage({ data: { kind: "generated", dispatch: owned.dispatch,
          bytes: new Uint8Array([7]), milliseconds: 1 } });
      }
    });
  }, terminate() {} };
  const generation = createGenerationPool({}, { hardwareConcurrency: 2, createWorker: () => worker });
  await generation.run([address], () => {}, { dependencies: () => source });
  assert.deepEqual(actual, { job: address, dependencies: [0, 128, 255] });
  assert.equal(source.byteLength, 0);
  generation.close();
});

test("invalid generated bytes and timings are rejected before delivery", async () => {
  const { createGenerationPool } = await pool;
  for (const product of [
    { bytes: [1], milliseconds: 1 },
    { bytes: new Uint8Array(new SharedArrayBuffer(1)), milliseconds: 1 },
    { bytes: new Uint8Array([1]), milliseconds: NaN },
    { bytes: new Uint8Array([1]), milliseconds: -1 },
    { bytes: new Uint8Array([1]), milliseconds: Infinity },
  ]) {
    const worker = { postMessage(message) {
      queueMicrotask(() => this.onmessage({ data: message.kind === "initialize"
        ? { kind: "ready", dispatch: message.dispatch }
        : { kind: "generated", dispatch: message.dispatch, ...product } }));
    }, terminate() { this.terminated = true; } };
    const generation = createGenerationPool({}, { hardwareConcurrency: 2, createWorker: () => worker });
    let delivered = false;
    await assert.rejects(generation.run(["1"], () => { delivered = true; }), error => error.code === "worker-protocol");
    assert.equal(delivered, false);
    assert.equal(worker.terminated, true);
  }
});

test("worker startup and dispatch failures preserve their underlying causes", async () => {
  const { createGenerationPool } = await pool;
  const cause = new Error("host rejected the operation");
  const startup = createGenerationPool({}, { hardwareConcurrency: 2, createWorker() { throw cause; } });
  await assert.rejects(startup.run(["1"], () => {}), error => error.code === "worker-start-failed" && error.cause === cause);
  const worker = { postMessage() { throw cause; }, terminate() { this.terminated = true; } };
  const dispatch = createGenerationPool({}, { hardwareConcurrency: 2, createWorker: () => worker });
  await assert.rejects(dispatch.run(["1"], () => {}), error => error.code === "dispatch-failed" && error.cause === cause);
  assert.equal(worker.terminated, true);
  assert.equal(worker.onmessage, null);
});

test("a worker deadline rejects readiness and terminates its runtime", async () => {
  const { createGenerationPool } = await pool;
  const originalTimer = globalThis.setTimeout;
  const worker = { postMessage() {}, terminate() { this.terminated = true; } };
  const generation = createGenerationPool({}, { hardwareConcurrency: 2, createWorker: () => worker });
  globalThis.setTimeout = callback => { queueMicrotask(callback); return 0; };
  try { await assert.rejects(generation.run(["1"], () => {}), error => error.code === "worker-timeout"); }
  finally { globalThis.setTimeout = originalTimer; }
  assert.equal(worker.terminated, true);
  assert.equal(worker.onmessage, null);
});

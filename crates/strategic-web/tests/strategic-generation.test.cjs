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
        if (message.module) { this.initialized++; this.onmessage({ data: { ready: true } }); }
        else this.onmessage({ data: fail ? { error: "generation failed" }
          : { bytes: new Uint8Array([message.job]), milliseconds: 1 } });
      });
    }, terminate() { this.terminated = true; } };
    instances.push(worker); return worker;
  } };
}

test("bounded workers serve dependent phases without restarting their runtimes", async () => {
  const mock = workers(); const results = [];
  const { createGenerationPool } = await pool;
  const generation = createGenerationPool({}, { createWorker: mock.createWorker, hardwareConcurrency: 32 });
  const receive = (job, bytes) => { assert.equal(bytes[0], job); results.push(job); };
  assert.equal(await generation.run([1, 2, 3, 4, 5, 6], receive), 4);
  assert.equal(await generation.run([7], receive), 0);
  assert.equal(await generation.run([8, 9], receive), 0);
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
    await assert.rejects(generation.run([1, 2, 3], () => { throw new Error("install failed"); }), /failed/);
    assert.ok(mock.instances.every(worker => worker.terminated));
  }
});

test("empty and fully cached phases start no workers", async () => {
  const { createGenerationPool } = await pool;
  const generation = createGenerationPool({}, {
    createWorker() { throw new Error("unnecessary worker"); }, hardwareConcurrency: 1,
  });
  assert.equal(await generation.run([], () => {}), 0);
  assert.equal(await generation.run([1, 2], () => { throw Error("cached delivery"); }, {
    resolveJob: () => null,
  }), 0);
  generation.close();
});

test("a miss generates while an unrelated cache read is still pending", async () => {
  const { createGenerationPool } = await pool;
  const mock = workers();
  const generation = createGenerationPool({}, { createWorker: mock.createWorker, hardwareConcurrency: 3 });
  let releaseRead;
  const reading = new Promise(resolve => { releaseRead = resolve; });
  const results = [];
  const running = generation.run([1, 2], (job) => {
    results.push(job); releaseRead(null);
  }, { resolveJob: job => job === 1 ? job : reading });
  assert.equal(await running, 1);
  assert.deepEqual(results, [1]);
  generation.close();
  assert.ok(mock.instances.every(worker => worker.terminated));
});

test("dependency failures close the pool and a pending phase cannot overlap another", async () => {
  const { createGenerationPool } = await pool;
  const mock = workers();
  const generation = createGenerationPool({}, { createWorker: mock.createWorker, hardwareConcurrency: 3 });
  const running = generation.run([1], () => {}, {
    dependencies() { throw Error("missing dependency"); },
  });
  await assert.rejects(generation.run([2], () => {}), /not available/);
  await assert.rejects(running, /missing dependency/);
  assert.ok(mock.instances.every(worker => worker.terminated));
});

test("a later miss lazily fills worker slots left unused by cached jobs", async () => {
  const { createGenerationPool } = await pool;
  const mock = workers();
  const generation = createGenerationPool({}, { createWorker: mock.createWorker, hardwareConcurrency: 3 });
  assert.equal(await generation.run([1, 2], () => {}, { resolveJob: x => x === 1 ? null : x }), 1);
  assert.equal(await generation.run([3, 4], () => {}), 1);
  assert.equal(mock.instances.length, 2);
  generation.close();
  assert.ok(mock.instances.every(worker => worker.terminated));
});

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");
const pool = import(`data:text/javascript;base64,${fs.readFileSync(path.join(__dirname,
  "../static/strategic-generation-pool.js")).toString("base64")}`);

function workers(fail = false) {
  const instances = [];
  return { instances, createWorker() {
    const worker = { terminated: false, postMessage(message) {
      queueMicrotask(() => {
        if (this.terminated) return;
        if (message.module) this.onmessage({ data: { ready: true } });
        else this.onmessage({ data: fail ? { error: "generation failed" }
          : { bytes: new Uint8Array([message.job]), milliseconds: 1 } });
      });
    }, terminate() { this.terminated = true; } };
    instances.push(worker); return worker;
  } };
}

test("bounded workers deliver every job once and release their instances", async () => {
  const mock = workers(); const results = [];
  const { generateJobs } = await pool;
  const count = await generateJobs({}, [1, 2, 3, 4, 5, 6], (job, bytes) => {
    assert.equal(bytes[0], job); results.push(job);
  }, { createWorker: mock.createWorker, hardwareConcurrency: 32 });
  assert.equal(count, 4);
  assert.deepEqual(results.sort(), [1, 2, 3, 4, 5, 6]);
  assert.ok(mock.instances.every(worker => worker.terminated));
});

test("generation and installation errors reject readiness and terminate workers", async () => {
  const { generateJobs } = await pool;
  for (const fail of [true, false]) {
    const mock = workers(fail);
    await assert.rejects(generateJobs({}, [1, 2, 3], () => { throw new Error("install failed"); },
      { createWorker: mock.createWorker, hardwareConcurrency: 3 }), /failed/);
    assert.ok(mock.instances.every(worker => worker.terminated));
  }
});

test("no jobs start no workers", async () => {
  const { generateJobs } = await pool;
  assert.equal(await generateJobs({}, [], () => {}, {
    createWorker() { throw new Error("unnecessary worker"); }, hardwareConcurrency: 1,
  }), 0);
});

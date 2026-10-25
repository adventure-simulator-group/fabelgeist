const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");
const queueModule = import("data:text/javascript;base64," + fs.readFileSync(path.join(__dirname,
  "../static/strategic-generation-write-queue.js")).toString("base64"));
const nextTurn = () => new Promise(resolve => setImmediate(resolve));

test("persistence starts after readiness, consumes immutable buffers and bounds write lanes", async () => {
  const { createDeferredWrites } = await queueModule;
  const calls = [], releases = [];
  let active = 0, maximumActive = 0;
  const writes = createDeferredWrites(async (job, bytes) => {
    active++; maximumActive = Math.max(active, maximumActive);
    calls.push([job, Array.from(bytes)]);
    await new Promise(resolve => releases.push(resolve));
    active--;
  }, { maximumBytes: 100, maximumProductBytes: 20, maximumProducts: 8, concurrency: 2 });
  const source = new Uint8Array([0, 128, 255]);
  assert.deepEqual(writes.put("first", source), { status: "accepted", disposition: "inserted" });
  assert.equal(source.byteLength, 0, "accepted product ownership moves to the cache");
  writes.put("second", new Uint8Array([4]));
  writes.put("third", new Uint8Array([5]));
  await nextTurn();
  assert.deepEqual(calls, [], "generation never starts optional compression");
  const settled = writes.close();
  assert.equal(writes.close(), settled, "closing does not create more write lanes");
  assert.deepEqual(calls, [["first", [0, 128, 255]], ["second", [4]]]);
  const late = new Uint8Array([6]);
  assert.deepEqual(writes.put("late", late), { status: "rejected", reason: "closed" });
  assert.equal(late.byteLength, 1);
  releases.shift()(); await nextTurn();
  assert.deepEqual(calls[2], ["third", [5]]);
  while (releases.length) releases.shift()();
  assert.deepEqual(await settled, { status: "flushed", writtenProducts: 3, failures: [] });
  assert.equal(maximumActive, 2);
  assert.deepEqual(writes.snapshot(), {
    phase: "closed", pendingProducts: 0, inFlightProducts: 0, retainedBytes: 0,
  });
});

test("pending products account for full backing buffers and job text, with bounded replacement and removal", async () => {
  const { createDeferredWrites } = await queueModule;
  const calls = [];
  const writes = createDeferredWrites((job, bytes) => { calls.push([job, Array.from(bytes)]); },
    { maximumBytes: 10, maximumProductBytes: 8, maximumProducts: 2, concurrency: 2 });
  assert.equal(writes.put("a", new Uint8Array(4)).status, "accepted");
  const tooMuch = new Uint8Array(4);
  assert.deepEqual(writes.put("b", tooMuch), { status: "rejected", reason: "byte-limit" });
  assert.equal(tooMuch.byteLength, 4, "rejected ownership stays with its caller");
  assert.equal(writes.put("b", new Uint8Array(2)).status, "accepted");
  assert.deepEqual(writes.put("", new Uint8Array()), { status: "rejected", reason: "record-limit" },
    "empty buffers cannot grow the record count");
  assert.deepEqual(writes.remove("a"), { status: "removed" });
  assert.deepEqual(writes.remove("b"), { status: "removed" });
  assert.deepEqual(writes.remove("missing"), { status: "absent" });
  const oversizedBacking = new Uint8Array(new ArrayBuffer(9), 0, 2);
  assert.deepEqual(writes.put("c", oversizedBacking), { status: "rejected", reason: "product-limit" });
  assert.equal(oversizedBacking.byteLength, 2);
  assert.equal(writes.put("c", new Uint8Array(6)).status, "accepted");
  assert.deepEqual(writes.put("c", new Uint8Array(8).fill(7)),
    { status: "accepted", disposition: "replaced" }, "latest bytes replace the pending product");
  await writes.close();
  assert.deepEqual(calls, [["c", Array(8).fill(7)]]);
});

test("failed replacement preserves the old product and caller ownership", async () => {
  const { createDeferredWrites } = await queueModule;
  const calls = [];
  const writes = createDeferredWrites((job, bytes) => calls.push([job, Array.from(bytes)]),
    { maximumBytes: 6, maximumProductBytes: 8, maximumProducts: 1, concurrency: 1 });
  writes.put("a", new Uint8Array([1, 2]));
  const replacement = new Uint8Array(5);
  assert.deepEqual(writes.put("a", replacement), { status: "rejected", reason: "byte-limit" });
  assert.equal(replacement.byteLength, 5);
  assert.equal(writes.snapshot().retainedBytes, 4);
  await writes.close();
  assert.deepEqual(calls, [["a", [1, 2]]]);
});

test("replacement keeps admission order and exact job addresses", async () => {
  const { createDeferredWrites } = await queueModule;
  const calls = [];
  const writes = createDeferredWrites((job, bytes) => calls.push([job, Array.from(bytes)]),
    { maximumBytes: 200, maximumProductBytes: 8, maximumProducts: 2, concurrency: 1 });
  const first = '{"seed":18446744073709551615}';
  const second = '{"seed":18446744073709551614}';
  writes.put(first, new Uint8Array([1]));
  writes.put(second, new Uint8Array([2]));
  assert.deepEqual(writes.put(first, new Uint8Array([3])), { status: "accepted", disposition: "replaced" });
  await writes.close();
  assert.deepEqual(calls, [[first, [3]], [second, [2]]]);
});

test("transfer failures and invalid inputs do not consume a queue slot", async () => {
  const { createDeferredWrites } = await queueModule;
  const calls = [];
  const writes = createDeferredWrites((job, bytes) => calls.push([job, Array.from(bytes)]),
    { maximumBytes: 20, maximumProductBytes: 8, maximumProducts: 1, concurrency: 1 });
  const detached = new Uint8Array([2]);
  structuredClone(detached, { transfer: [detached.buffer] });
  const rejected = writes.put("a", detached);
  assert.equal(rejected.status, "rejected");
  assert.equal(rejected.reason, "transfer-failed");
  assert.equal(rejected.cause.name, "DataCloneError");
  assert.deepEqual(writes.put(42, new Uint8Array([3])), { status: "rejected", reason: "invalid-job" });
  assert.deepEqual(writes.put("a", new Uint16Array([3])), { status: "rejected", reason: "invalid-buffer" });
  const shared = new Uint8Array(new SharedArrayBuffer(2));
  assert.deepEqual(writes.put("a", shared), { status: "rejected", reason: "invalid-buffer" });
  writes.put("a", new Uint8Array([4]));
  await writes.close();
  assert.deepEqual(calls, [["a", [4]]]);
});

test("failed writes settle every lane and release all retained allocations", async () => {
  const { createDeferredWrites } = await queueModule;
  const failure = new Error("storage failed");
  const calls = [];
  let release;
  const writes = createDeferredWrites(async job => {
    calls.push(job);
    if (job === "a") throw failure;
    if (job === "b") await new Promise(resolve => { release = resolve; });
  }, { maximumBytes: 40, maximumProductBytes: 8, maximumProducts: 4, concurrency: 2 });
  for (const job of ["a", "b", "c", "d"]) writes.put(job, new Uint8Array([1]));
  const settled = writes.close();
  let finished = false;
  settled.then(() => { finished = true; });
  await nextTurn();
  assert.deepEqual(calls, ["a", "b", "c", "d"]);
  assert.equal(finished, false, "close waits for the other lane after a failure");
  assert.deepEqual(writes.snapshot(), {
    phase: "draining", pendingProducts: 0, inFlightProducts: 1, retainedBytes: 3,
  });
  release();
  assert.deepEqual(await settled, {
    status: "failed", writtenProducts: 3, failures: [{ job: "a", cause: failure }],
  });
  assert.deepEqual(writes.snapshot(), {
    phase: "closed", pendingProducts: 0, inFlightProducts: 0, retainedBytes: 0,
  });
});

test("invalid budgets fail before admission and caller mutation cannot change them", async () => {
  const { createDeferredWrites } = await queueModule;
  const limits = { maximumBytes: 8, maximumProductBytes: 4, maximumProducts: 1, concurrency: 1 };
  for (const field of Object.keys(limits)) {
    for (const value of [0, -1, 1.5, NaN, Infinity, Number.MAX_SAFE_INTEGER + 1, "2"]) {
      assert.throws(() => createDeferredWrites(() => {}, { ...limits, [field]: value }), RangeError);
    }
  }
  const writes = createDeferredWrites(() => {}, limits);
  limits.maximumProducts = 2;
  writes.put("a", new Uint8Array([1]));
  const rejected = new Uint8Array([2]);
  assert.deepEqual(writes.put("b", rejected), { status: "rejected", reason: "record-limit" });
  assert.equal(rejected.byteLength, 1);
  await writes.close();
});

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
  assert.equal(writes.put("first", source), true);
  assert.equal(source.byteLength, 0, "accepted product ownership moves to the cache");
  writes.put("second", new Uint8Array([4]));
  writes.put("third", new Uint8Array([5]));
  await nextTurn();
  assert.deepEqual(calls, [], "generation never starts optional compression");
  const settled = writes.close();
  assert.equal(writes.close(), settled, "closing does not create more write lanes");
  assert.deepEqual(calls, [["first", [0, 128, 255]], ["second", [4]]]);
  const late = new Uint8Array([6]);
  assert.equal(writes.put("late", late), false);
  assert.equal(late.byteLength, 1);
  releases.shift()(); await nextTurn();
  assert.deepEqual(calls[2], ["third", [5]]);
  while (releases.length) releases.shift()();
  await settled;
  assert.equal(maximumActive, 2);
});

test("pending products account for full backing buffers and job text, with bounded replacement and removal", async () => {
  const { createDeferredWrites } = await queueModule;
  const calls = [];
  const writes = createDeferredWrites((job, bytes) => { calls.push([job, Array.from(bytes)]); },
    { maximumBytes: 10, maximumProductBytes: 8, maximumProducts: 2, concurrency: 2 });
  assert.equal(writes.put("a", new Uint8Array(4)), true);
  const tooMuch = new Uint8Array(4);
  assert.equal(writes.put("b", tooMuch), false);
  assert.equal(tooMuch.byteLength, 4, "rejected ownership stays with its caller");
  assert.equal(writes.put("b", new Uint8Array(2)), true);
  assert.equal(writes.put("", new Uint8Array()), false, "empty buffers cannot grow the record count");
  writes.remove("a"); writes.remove("b");
  const oversizedBacking = new Uint8Array(new ArrayBuffer(9), 0, 2);
  assert.equal(writes.put("c", oversizedBacking), false);
  assert.equal(oversizedBacking.byteLength, 2);
  assert.equal(writes.put("c", new Uint8Array(6)), true);
  assert.equal(writes.put("c", new Uint8Array(8).fill(7)), true, "latest bytes replace the pending product");
  await writes.close();
  assert.deepEqual(calls, [["c", Array(8).fill(7)]]);
});

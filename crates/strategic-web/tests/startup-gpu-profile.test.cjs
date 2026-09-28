const test = require("node:test");
const assert = require("node:assert/strict");
const vm = require("node:vm");
const { install } = require("./startup-gpu-profile.cjs");

test("startup GPU observation preserves calls and never waits for its queue probe", async () => {
  const events = [], calls = [], pipeline = {}, queuedFrames = [];
  let complete, clock = 0;
  const fence = new Promise(resolve => { complete = resolve; });
  const promise = Promise.resolve(pipeline);
  class Device {
    createRenderPipelineAsync(descriptor) { calls.push([this, descriptor]); return promise; }
    createComputePipelineAsync() { return promise; }
    createRenderPipeline() { return pipeline; }
    createComputePipeline() { return pipeline; }
    createShaderModule() { return pipeline; }
  }
  class Queue {
    submit(buffers) { calls.push(buffers); return "submitted"; }
    onSubmittedWorkDone() { calls.push("fence"); return fence; }
  }
  const profile = { active: true, mark(kind, data) { if (this.active) events.push({ kind, ...data }); } };
  vm.runInNewContext(`(${install.toString()})();`, { window: { startupProfile: profile },
    GPUDevice: Device, GPUQueue: Queue, performance: { now: () => ++clock },
    requestAnimationFrame: callback => queuedFrames.push(callback) });
  const device = new Device(), queue = new Queue(), descriptor = { label: "test" };
  assert.equal(device.createRenderPipelineAsync(descriptor), promise);
  assert.deepEqual(calls[0], [device, descriptor]);
  for (let i = 0; i < 24; i++) assert.equal(queue.submit([]), "submitted");
  assert.equal(calls.filter(value => value === "fence").length, 1);
  await promise;
  assert.equal(events.filter(event => event.kind === "gpu-create-end").length, 1);
  assert.equal(events.filter(event => event.kind === "gpu-queue-complete").length, 0);
  complete(); await fence;
  assert.equal(events.filter(event => event.kind === "gpu-queue-complete").length, 1);
  profile.active = false;
  const count = events.length;
  assert.equal(device.createShaderModule({}), pipeline);
  queue.submit([]); queuedFrames[0](42);
  assert.equal(events.length, count);
});

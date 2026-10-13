const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");
const vm = require("node:vm");
const source = fs.readFileSync(path.join(__dirname, "../static/strategic-generation-worker.js"), "utf8")
  .replace(/^import[^\n]+\n/, "");

function worker(generate) {
  const responses = [], modules = [];
  const self = { postMessage(message, transfer = []) {
    responses.push(structuredClone(message, { transfer }));
  } };
  vm.runInNewContext(source, { self, performance, init: async request => modules.push(request),
    wasm_generate_job: generate });
  return { self, responses, modules };
}

test("worker envelopes echo dispatch identity and transfer exact product bytes", async () => {
  const bytes = new Uint8Array([0, 128, 255]);
  const job = '{"seed":18446744073709551615}';
  const dependencies = new Uint8Array([7, 8]);
  const calls = [];
  const runtime = worker((address, input) => { calls.push({ address, input }); return bytes; });
  await runtime.self.onmessage({ data: { kind: "initialize", dispatch: 1, module: "compiled-module" } });
  assert.deepEqual(runtime.responses[0], { kind: "ready", dispatch: 1 });
  assert.deepEqual(runtime.modules.map(module => module.module_or_path), ["compiled-module"]);
  await runtime.self.onmessage({ data: { kind: "generate", dispatch: 2, job, dependencies } });
  const response = runtime.responses[1];
  assert.equal(response.kind, "generated");
  assert.equal(response.dispatch, 2);
  assert.deepEqual(Array.from(response.bytes), [0, 128, 255]);
  assert.ok(Number.isFinite(response.milliseconds) && response.milliseconds >= 0);
  assert.equal(bytes.byteLength, 0);
  assert.deepEqual(calls, [{ address: job, input: dependencies }]);
});

test("worker failures retain stable codes and malformed requests never invoke Wasm", async () => {
  const runtime = worker(() => {
    const failure = new Error("scene not prepared");
    failure.name = "generation/scene-not-prepared";
    throw failure;
  });
  await runtime.self.onmessage({ data: { kind: "generate", dispatch: 7, job: "opaque-job" } });
  assert.deepEqual(runtime.responses[0], { kind: "failed", dispatch: 7,
    error: { code: "generation/scene-not-prepared", message: "scene not prepared" } });
  const malformed = worker(() => { throw new Error("Wasm must not run"); });
  for (const data of [null, { kind: "initialize", dispatch: 0 }, { kind: "unknown", dispatch: 1 },
    { kind: "generate", dispatch: 2, job: 42 }]) await malformed.self.onmessage({ data });
  assert.ok(malformed.responses.every(response => response.kind === "failed"
    && response.error.code === "worker-protocol"));
  assert.deepEqual(malformed.modules, []);
});

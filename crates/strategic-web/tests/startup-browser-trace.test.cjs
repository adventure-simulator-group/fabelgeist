const test = require("node:test");
const assert = require("node:assert/strict");
const { EventEmitter } = require("node:events");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { attach } = require("./startup-browser-trace.cjs");

test("browser traces retain received evidence when the GPU tracing RPC fails", async () => {
  const output = fs.mkdtempSync(path.join(os.tmpdir(), "startup-trace-"));
  const cdp = new EventEmitter();
  cdp.send = async method => {
    if (method === "Tracing.end") throw new Error("GPU process unavailable");
  };
  try {
    const trace = attach(cdp, output);
    await trace.start();
    cdp.emit("Tracing.dataCollected", {value: [{name: "shader compilation", ph: "B"}]});
    await trace.stop("failed");
    const saved = JSON.parse(fs.readFileSync(path.join(output, "failed-browser.json"), "utf8"));
    assert.equal(saved.complete, false);
    assert.match(saved.error, /GPU process unavailable/);
    assert.deepEqual(saved.traceEvents, [{name: "shader compilation", ph: "B"}]);
    assert.equal(cdp.listenerCount("Tracing.tracingComplete"), 0);
    await assert.rejects(trace.start(), /incomplete browser trace/);
  } finally {
    fs.unlinkSync(path.join(output, "failed-browser.json")); fs.rmdirSync(output);
  }
});

test("complete browser traces retain the browser's data-loss verdict", async () => {
  const output = fs.mkdtempSync(path.join(os.tmpdir(), "startup-trace-"));
  const cdp = new EventEmitter();
  cdp.send = async method => {
    if (method === "Tracing.end") cdp.emit("Tracing.tracingComplete", {dataLossOccurred: true});
  };
  try {
    const trace = attach(cdp, output); await trace.start(); await trace.stop("complete");
    const saved = JSON.parse(fs.readFileSync(path.join(output, "complete-browser.json"), "utf8"));
    assert.equal(saved.complete, true); assert.equal(saved.dataLossOccurred, true);
    assert.equal(saved.error, undefined);
  } finally {
    fs.unlinkSync(path.join(output, "complete-browser.json")); fs.rmdirSync(output);
  }
});

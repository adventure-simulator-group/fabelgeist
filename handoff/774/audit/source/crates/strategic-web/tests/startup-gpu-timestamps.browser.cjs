const test = require("node:test");
const assert = require("node:assert/strict");
const http = require("node:http");
const { chromium } = require("playwright");

test("startup timestamps sample real submitted passes outside animation callbacks", async () => {
  const server = http.createServer((_, response) => response.end("<!doctype html>"));
  await new Promise(resolve => server.listen(0, "127.0.0.1", resolve));
  const browser = await chromium.launch({ headless: true,
    channel: process.platform === "win32" ? "msedge" : undefined, args: ["--enable-unsafe-webgpu"] });
  try {
    const page = await browser.newPage();
    await page.goto(`http://127.0.0.1:${server.address().port}`);
    await page.evaluate(() => { window.startupProfile = { active: true, events: [],
      mark(kind, data) { this.events.push({ kind, ...data }); } }; });
    await page.evaluate(require("./startup-gpu-timestamps.cjs").install);
    await page.evaluate(async () => {
      const adapter = await navigator.gpu.requestAdapter(), device = await adapter.requestDevice();
      window.device = device;
      const module = device.createShaderModule({ code: "@compute @workgroup_size(1) fn main() {}" });
      const pipeline = device.createComputePipeline({ layout: "auto", compute: { module, entryPoint: "main" } });
      for (let i = 0; i < 16; i++) {
        const encoder = device.createCommandEncoder();
        for (const label of ["first", "second"]) {
          const pass = encoder.beginComputePass({ label });
          pass.setPipeline(pipeline); pass.dispatchWorkgroups(1); pass.end();
        }
        device.queue.submit([encoder.finish()]);
      }
    });
    await page.waitForFunction(() => window.startupProfile.events.length >= 2);
    const events = await page.evaluate(() => window.startupProfile.events);
    assert.equal(events.length, 2);
    for (const event of events) {
      assert.equal(event.kind, "gpu-pass-sample");
      assert.deepEqual(event.passes.map(pass => pass.label), ["first", "second"]);
      assert(event.passes.every(pass => pass.milliseconds >= 0 && Number.isFinite(pass.milliseconds)));
    }
  } finally { await browser.close(); await new Promise(resolve => server.close(resolve)); }
});

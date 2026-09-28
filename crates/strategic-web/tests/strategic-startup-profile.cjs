// Opt-in diagnostics; production sources and readiness behavior are unchanged.
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

function instrument(source, replacements) {
  for (const [from, to] of replacements) {
    assert(source.includes(from), `Startup profiling anchor missing: ${from}`);
    source = source.replace(from, to);
  }
  return source;
}

exports.attach = async (page, output) => {
  await page.addInitScript(() => {
    const events = [], resources = [], tasks = [];
    window.startupProfile = { events, resources, tasks,
      mark(kind, data = {}) { events.push({ at: performance.now(), kind, ...data }); } };
    performance.setResourceTimingBufferSize(4000);
    new PerformanceObserver(list => resources.push(...list.getEntries().map(entry => entry.toJSON())))
      .observe({ type: "resource", buffered: true });
    new PerformanceObserver(list => tasks.push(...list.getEntries().map(entry => entry.toJSON())))
      .observe({ type: "longtask", buffered: true });
    for (const method of ["info", "log"]) {
      const original = console[method];
      console[method] = function (...args) {
        const text = args.filter(value => typeof value === "string").join(" ");
        if (/GPU city|City fixed boundary|strategic snapshot/.test(text))
          window.startupProfile.mark("renderer", { text });
        return original.apply(this, args);
      };
    }
    window.startupProfile.mark("document-start");
  });
  const cdp = await page.context().newCDPSession(page);
  await cdp.send("Profiler.enable");
  await cdp.send("Profiler.setSamplingInterval", { interval: 1000 });
  return {
    start: () => cdp.send("Profiler.start"),
    async stop(name) {
      const { profile } = await cdp.send("Profiler.stop");
      const trace = await page.evaluate(() => ({ ...window.startupProfile,
        now: performance.now(), generation: window.strategicGenerationMetrics,
        metrics: window.strategicRendererMetrics }));
      fs.writeFileSync(path.join(output, `${name}.cpuprofile`), JSON.stringify(profile));
      fs.writeFileSync(path.join(output, `${name}-trace.json`), JSON.stringify(trace));
    },
  };
};

const transforms = {
  "strategic-renderer.js": [
    ['const response = await fetch("/tactical/wasm/adventuresim-tactical-client_bg.wasm");',
      'window.startupProfile.mark("wasm-fetch-start"); const response = await fetch("/tactical/wasm/adventuresim-tactical-client_bg.wasm"); window.startupProfile.mark("wasm-headers");'],
    ['await runtime.default({ module_or_path: generationModule });',
      'window.startupProfile.mark("wasm-compile-hash-done"); await runtime.default({ module_or_path: generationModule }); window.startupProfile.mark("wasm-initialized");'],
    ['runtime.wasm_boot(await graphics.text(), await audio.text());',
      'window.startupProfile.mark("boot-start"); runtime.wasm_boot(await graphics.text(), await audio.text()); window.startupProfile.mark("boot-end");'],
  ],
  "strategic-scene.js": [
    ['await prepareGeneratedScene(await runtimePromise, input);',
      'window.startupProfile.mark("scene-response"); await prepareGeneratedScene(await runtimePromise, input); window.startupProfile.mark("generation-ready");'],
    ['metrics.state = state;',
      'metrics.state = state; { const {daylight, view_lighting, street, ...brief} = state; window.startupProfile.mark("readiness", brief); }'],
  ],
  "strategic-generation.js": [
    ['const started = performance.now();',
      'const started = performance.now(); window.startupProfile.mark("generation-start");'],
    ['runtime.wasm_receive_job(job, bytes);',
      'runtime.wasm_receive_job(job, bytes); window.startupProfile.mark("receive", {start: receiveStarted, duration: performance.now() - receiveStarted, bytes: bytes.byteLength});'],
  ],
  "strategic-generation-cache.js": [
    ['async function decode(record) {',
      'async function decode(record) { const started = performance.now();'],
    ['return bytes;',
      'window.startupProfile.mark("decompress", {start: started, duration: performance.now() - started, bytes: record.size, decoded: length}); return bytes;'],
    ['const record = await transaction("readonly",',
      'const readStarted = performance.now(); const record = await transaction("readonly",'],
    ['if (!record) return undefined;',
      'window.startupProfile.mark("cache-read", {start: readStarted, duration: performance.now() - readStarted, bytes: record?.size || 0}); if (!record) return undefined;'],
  ],
};
exports.rewrite = (name, source) => transforms[name] ? instrument(source, transforms[name]) : source;

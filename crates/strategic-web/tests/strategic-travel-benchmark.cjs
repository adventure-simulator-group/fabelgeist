// City transitions keep the document, renderer, device and warm shader cache.
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

exports.run = async (page, output, ready, profiler) => {
  const results = [];
  await page.evaluate(() => window.strategicGenerationCacheSettled);
  for (const settlement of ["travel-destination", ...(process.env.STRATEGIC_TRAVEL_SECOND_INPUT ? ["travel-second"] : []), "scene-review"]) {
    const destination = `/locations/settlement/${settlement}/places/inn`;
    await profiler?.start();
    const started = performance.now();
    await page.evaluate(href => {
      window.travelReadiness = { start: performance.now(), states: [], active: true };
      let previous;
      function sample() {
        const trace = window.travelReadiness;
        if (!trace.active) return;
        const state = window.strategicRendererMetrics?.state;
        if (state) {
          const { revision, ready, assets_ready, installed_ready, snapshots_pending,
            terrain_pending, city_pending, waiting_pipelines, sky_ready } = state;
          const brief = { revision, ready, assets_ready, installed_ready, snapshots_pending,
            terrain_pending, city_pending, waiting_pipelines, sky_ready };
          const key = JSON.stringify(brief);
          if (key !== previous) trace.states.push({ milliseconds: performance.now() - trace.start, ...brief });
          previous = key;
        }
        requestAnimationFrame(sample);
      }
      requestAnimationFrame(sample);
      const link = document.createElement("a"); link.href = href;
      document.body.append(link); link.click(); link.remove();
    }, destination);
    await page.waitForFunction(expected => location.pathname === expected &&
      document.querySelector("#strategic-page")?.dataset.path === expected, destination);
    await ready();
    await page.waitForFunction(expected =>
      window.strategicRendererMetrics.navigations.at(-1)?.path === expected, destination);
    const elapsed = performance.now() - started;
    await profiler?.stop(`travel-${settlement}`);
    assert.equal(await page.evaluate(() => window.originalCanvas === document.querySelector("#game-canvas")), true);
    assert.equal(await page.locator("canvas").count(), 1);
    results.push({ settlement, milliseconds: elapsed,
      ...await page.evaluate(() => { window.travelReadiness.active = false; return ({ generation: window.strategicGenerationMetrics,
        readiness: window.travelReadiness.states,
        navigation: window.strategicRendererMetrics.navigations.at(-1),
        state: window.strategicRendererMetrics.state }); }) });
    fs.writeFileSync(path.join(output, "travel.json"), JSON.stringify(results, null, 2));
    await page.screenshot({ path: path.join(output, `travel-${settlement}.png`) });
    await page.evaluate(() => window.strategicGenerationCacheSettled);
  }
};

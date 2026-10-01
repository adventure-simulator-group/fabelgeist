const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');

exports.profile = async (page, output, ready) => {
  const directory = path.join(output, 'gpu-profile'); fs.mkdirSync(directory, {recursive: true});
  const cdp = await page.context().newCDPSession(page);
  await cdp.send('Profiler.enable');
  await cdp.send('Profiler.setSamplingInterval', {interval: 1000});
  async function capture(name) {
    const timing = await page.evaluate(() => window.renderProbe.capture('timing', 60));
    fs.writeFileSync(path.join(directory, `${name}-timing.json`), JSON.stringify(timing));
    const gpu = await page.evaluate(() => window.renderProbe.capture('gpu', 8));
    fs.writeFileSync(path.join(directory, `${name}-gpu.json`), JSON.stringify(gpu));
    assert.deepEqual(gpu.failures, [], 'GPU profiling must not introduce validation errors');
    assert(gpu.frames.every(frame => frame.passes.length > 0), 'Probe must observe real GPU passes');
    assert(gpu.frames.every(frame => frame.passes.every(pass => pass.unsupportedBundles === 0)), 'Render bundles require explicit profiling support');
    assert(gpu.frames.every(frame => frame.passes.every(pass => pass.gpuMs !== null)), 'Actual GPU timestamps required');
    for (const frame of gpu.frames) {
      const city = frame.passes.flatMap(pass => pass.draws).filter(draw => draw.pipeline?.label === 'gpu_city');
      assert(city.length > 0, 'The resident city must reach the actual render passes');
      assert(city.every(draw => draw.method === 'drawIndirect'), 'City batches must consume GPU-generated draw arguments');
      assert(frame.passes.some(pass => pass.label === 'city_building_lod'), 'GPU LOD selection must execute');
      assert(frame.passes.some(pass => pass.label === 'city_visible_clusters'), 'GPU visibility compaction must execute');
    }
    await cdp.send('Profiler.start');
    await page.waitForTimeout(8000);
    const {profile} = await cdp.send('Profiler.stop');
    fs.writeFileSync(path.join(directory, `${name}.cpuprofile`), JSON.stringify(profile));
    await page.screenshot({path: path.join(directory, `${name}.png`)});
  }
  if (process.env.STRATEGIC_PROFILE_PLACE !== 'public-square') {
    await capture('inn');
    if (process.env.STRATEGIC_PROFILE_PLACE === 'inn') {
      await cdp.detach();
      return;
    }
    await page.locator('[data-building-id="public-square"]').click();
    await ready();
  }
  await capture('public-square');
  // Diagnostic ablation only: keep Wasm/ECS/render preparation, omit raster draws.
  // Restored immediately afterward; never confuse these timings with rendered FPS.
  await page.evaluate(() => {
    window.probeSavedDraws = {};
    for (const method of ['draw', 'drawIndexed', 'drawIndirect', 'drawIndexedIndirect']) {
      window.probeSavedDraws[method] = GPURenderPassEncoder.prototype[method];
      GPURenderPassEncoder.prototype[method] = function() {};
    }
  });
  try {
    const timing = await page.evaluate(() => window.renderProbe.capture('timing', 60));
    timing.diagnostic = 'Raster draw calls suppressed; scene update and render preparation retained';
    fs.writeFileSync(path.join(directory, 'public-square-no-draws-timing.json'), JSON.stringify(timing));
  } finally {
    await page.evaluate(() => {
      Object.assign(GPURenderPassEncoder.prototype, window.probeSavedDraws);
      delete window.probeSavedDraws;
    });
  }
  const restored = await page.evaluate(() => window.renderProbe.capture('timing', 60));
  fs.writeFileSync(path.join(directory, 'public-square-restored-timing.json'), JSON.stringify(restored));
  await cdp.detach();
};

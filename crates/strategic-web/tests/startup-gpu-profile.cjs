// Runs in the page. Observe promises without adding waits to the render loop.
exports.install = function installStartupGpuProfile() {
  const profile = window.startupProfile;
  if (!profile || !globalThis.GPUDevice) return;
  let nextPipeline = 0, submissions = 0, pendingFence = false;
  const shaderIds = new WeakMap();
  const mark = (kind, data) => profile.mark(kind, data);
  for (const method of ["createRenderPipelineAsync", "createComputePipelineAsync",
    "createRenderPipeline", "createComputePipeline", "createShaderModule"]) {
    const original = GPUDevice.prototype[method];
    GPUDevice.prototype[method] = function (descriptor) {
      if (!profile.active) return original.call(this, descriptor);
      const id = ++nextPipeline, start = performance.now();
      mark("gpu-create-start", { id, method, label: descriptor.label || "",
        codeCharacters: descriptor.code?.length,
        source: descriptor.code,
        vertex: shaderIds.get(descriptor.vertex?.module),
        fragment: shaderIds.get(descriptor.fragment?.module),
        compute: shaderIds.get(descriptor.compute?.module),
        samples: descriptor.multisample?.count,
        buffers: descriptor.vertex?.buffers,
        targets: descriptor.fragment?.targets,
        primitive: descriptor.primitive, depthStencil: descriptor.depthStencil });
      const result = original.call(this, descriptor);
      if (method === "createShaderModule") shaderIds.set(result, id);
      const finish = error => mark("gpu-create-end", {
        id, method, start, duration: performance.now() - start, error: error?.message,
      });
      if (method.endsWith("Async")) result.then(() => finish(), finish);
      else finish();
      return result;
    };
  }
  const submit = GPUQueue.prototype.submit;
  GPUQueue.prototype.submit = function (buffers) {
    if (!profile.active) return submit.call(this, buffers);
    const start = performance.now(), id = ++submissions;
    const result = submit.call(this, buffers);
    mark("gpu-submit", { id, start, duration: performance.now() - start });
    // One outstanding probe, every eighth submission. Its callback latency
    // includes main-thread scheduling and is not a GPU timestamp.
    if (!pendingFence && id % 8 === 0) {
      pendingFence = true;
      this.onSubmittedWorkDone().then(() => {
        mark("gpu-queue-complete", { id, start, duration: performance.now() - start });
        pendingFence = false;
      }, error => { mark("gpu-queue-error", { message: error.message }); pendingFence = false; });
    }
    return result;
  };
  let previous;
  function frame(now) {
    if (!profile.active) return;
    if (previous !== undefined) mark("animation-frame", { interval: now - previous });
    previous = now;
    requestAnimationFrame(frame);
  }
  requestAnimationFrame(frame);
};

exports.summarize = events => {
  const starts = new Map(events.filter(event => event.kind === "gpu-create-start").map(event => [event.id, event]));
  const creates = events.filter(event => event.kind === "gpu-create-end")
    .map(event => ({ ...starts.get(event.id), ...event }));
  const groups = Object.groupBy(creates, event => event.method);
  return {
    creation: Object.fromEntries(Object.entries(groups).map(([method, entries]) => [method, {
      count: entries.length, summedMilliseconds: entries.reduce((sum, entry) => sum + entry.duration, 0),
      longest: entries.sort((a, b) => b.duration - a.duration).slice(0, 12),
    }])),
    submissions: events.filter(event => event.kind === "gpu-submit").length,
    queueCallbacks: events.filter(event => event.kind === "gpu-queue-complete")
      .sort((a, b) => b.duration - a.duration).slice(0, 20),
    passSamples: events.filter(event => event.kind === "gpu-pass-sample").map(event => ({
      start: event.start, at: event.at, callbackMilliseconds: event.callbackMilliseconds,
      gpuMilliseconds: event.passes.reduce((sum, pass) => sum + pass.milliseconds, 0),
      passes: event.passes,
    })),
    errors: events.filter(event => ["gpu-timestamp-error", "gpu-queue-error"].includes(event.kind)),
    longestAnimationFrames: events.filter(event => event.kind === "animation-frame")
      .sort((a, b) => b.interval - a.interval).slice(0, 20),
  };
};

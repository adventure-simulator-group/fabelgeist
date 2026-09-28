// Sample command encoders directly: startup rendering need not run inside RAF.
exports.install = function installStartupGpuTimestamps() {
  const profile = window.startupProfile;
  const encoders = new WeakMap(), commands = new WeakMap();
  let count = 0, outstanding = 0;
  const capacity = 512, maximumOutstanding = 2, sampleEvery = 8;
  const request = GPUAdapter.prototype.requestDevice;
  GPUAdapter.prototype.requestDevice = function (descriptor = {}) {
    if (!this.features.has("timestamp-query")) throw new Error("Startup GPU timestamps unavailable");
    return request.call(this, { ...descriptor, requiredFeatures:
      [...new Set([...(descriptor.requiredFeatures || []), "timestamp-query"])] });
  };
  const create = GPUDevice.prototype.createCommandEncoder;
  GPUDevice.prototype.createCommandEncoder = function (...args) {
    const encoder = create.apply(this, args);
    if (profile.active && ++count % sampleEvery === 0 && outstanding < maximumOutstanding) {
      outstanding++;
      encoders.set(encoder, { device: this, start: performance.now(), passes: [],
        queries: this.createQuerySet({ type: "timestamp", count: capacity }), count: 0 });
    }
    return encoder;
  };
  for (const method of ["beginRenderPass", "beginComputePass"]) {
    const begin = GPUCommandEncoder.prototype[method];
    GPUCommandEncoder.prototype[method] = function (descriptor = {}) {
      const state = encoders.get(this);
      if (!state || descriptor.timestampWrites || state.count + 2 > capacity)
        return begin.call(this, descriptor);
      const first = state.count; state.count += 2;
      state.passes.push({ label: descriptor.label || "", kind: method, first });
      return begin.call(this, { ...descriptor, timestampWrites: { querySet: state.queries,
        beginningOfPassWriteIndex: first, endOfPassWriteIndex: first + 1 } });
    };
  }
  const finish = GPUCommandEncoder.prototype.finish;
  GPUCommandEncoder.prototype.finish = function (...args) {
    const state = encoders.get(this);
    if (state?.count) {
      const size = state.count * 8;
      state.resolved = state.device.createBuffer({ size,
        usage: GPUBufferUsage.QUERY_RESOLVE | GPUBufferUsage.COPY_SRC });
      state.readback = state.device.createBuffer({ size,
        usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ });
      this.resolveQuerySet(state.queries, 0, state.count, state.resolved, 0);
      this.copyBufferToBuffer(state.resolved, 0, state.readback, 0, size);
    } else if (state) { state.queries.destroy(); outstanding--; }
    const command = finish.apply(this, args);
    if (state?.count) commands.set(command, state);
    return command;
  };
  const submit = GPUQueue.prototype.submit;
  GPUQueue.prototype.submit = function (buffers) {
    const batch = Array.from(buffers);
    const result = submit.call(this, batch);
    for (const buffer of batch) {
      const state = commands.get(buffer);
      if (!state) continue;
      commands.delete(buffer);
      state.readback.mapAsync(GPUMapMode.READ).then(() => {
        const times = new BigUint64Array(state.readback.getMappedRange());
        profile.mark("gpu-pass-sample", { start: state.start,
          callbackMilliseconds: performance.now() - state.start,
          passes: state.passes.map(pass => ({ label: pass.label, kind: pass.kind,
            milliseconds: Number(times[pass.first + 1] - times[pass.first]) / 1e6 })) });
        state.readback.unmap();
      }, error => profile.mark("gpu-timestamp-error", { message: error.message })).finally(() => {
        state.readback.destroy(); state.resolved.destroy(); state.queries.destroy(); outstanding--;
      });
    }
    return result;
  };
};

// Opt-in test instrumentation. Loaded before Wasm; never shipped by the app.
(() => {
  const encoders = new WeakMap(), passes = new WeakMap(), pipelines = new WeakMap();
  const ids = new WeakMap(); let nextId = 1, current = null, request = null, latest = null;
  const id = object => { if (!ids.has(object)) ids.set(object, nextId++); return ids.get(object); };
  const capabilities = [], failures = [];
  const wrap = (prototype, method, intercept) => {
    const original = prototype[method];
    prototype[method] = function(...args) { return intercept.call(this, original, args); };
  };
  const triangles = (topology, count, instances) =>
    (topology === 'triangle-list' ? Math.floor(count / 3) : topology === 'triangle-strip' ? Math.max(0, count - 2) : 0) * instances;
  const nativeBuffer = GPUDevice.prototype.createBuffer;

  wrap(GPUAdapter.prototype, 'requestDevice', async function(original, [descriptor = {}]) {
    const features = [...new Set([...(descriptor.requiredFeatures || []),
      ...(this.features.has('timestamp-query') ? ['timestamp-query'] : [])])];
    const device = await original.call(this, {...descriptor, requiredFeatures: features});
    capabilities.push({adapter: {...this.info, vendor: this.info.vendor, architecture: this.info.architecture,
      description: this.info.description}, supportedFeatures: [...this.features], enabledFeatures: [...device.features]});
    device.addEventListener('uncapturederror', event => failures.push(event.error.message));
    return device;
  });
  wrap(GPUDevice.prototype, 'createBuffer', function(original, [descriptor]) {
    return original.call(this, {...descriptor, usage: descriptor.usage |
      (descriptor.usage & GPUBufferUsage.INDIRECT ? GPUBufferUsage.COPY_SRC : 0)});
  });
  for (const method of ['createRenderPipeline', 'createRenderPipelineAsync']) {
    wrap(GPUDevice.prototype, method, function(original, [descriptor]) {
      const remember = pipeline => {
        pipelines.set(pipeline, {id: id(pipeline), label: descriptor.label || '',
          vertex: descriptor.vertex.module.label, fragment: descriptor.fragment?.module.label,
          topology: descriptor.primitive?.topology || 'triangle-list'});
        return pipeline;
      };
      const pipeline = original.call(this, descriptor);
      return method.endsWith('Async') ? pipeline.then(remember) : remember(pipeline);
    });
  }
  wrap(GPUDevice.prototype, 'createCommandEncoder', function(original, args) {
    const encoder = original.apply(this, args);
    if (current?.mode === 'gpu') {
      encoders.set(encoder, {device: this, frame: current, passes: [], queryCount: 0,
        querySet: this.features.has('timestamp-query') ? this.createQuerySet({type: 'timestamp', count: 1024}) : null});
    }
    return encoder;
  });
  for (const method of ['beginRenderPass', 'beginComputePass']) {
    wrap(GPUCommandEncoder.prototype, method, function(original, [descriptor = {}]) {
      const encoder = encoders.get(this);
      if (!encoder) return original.call(this, descriptor);
      const pass = {kind: method === 'beginRenderPass' ? 'render' : 'compute', label: descriptor.label || '',
        draws: [], dispatches: 0, pipeline: null, indexBuffer: null, vertexBuffers: {},
        query: encoder.queryCount, gpuMs: null, unsupportedBundles: 0};
      if (encoder.querySet && !descriptor.timestampWrites) {
        if (encoder.queryCount + 2 > 1024) throw new Error('Probe timestamp capacity exceeded');
        descriptor = {...descriptor, timestampWrites: {querySet: encoder.querySet,
          beginningOfPassWriteIndex: encoder.queryCount++, endOfPassWriteIndex: encoder.queryCount++}};
      } else pass.query = null;
      const result = original.call(this, descriptor);
      encoder.passes.push(pass); encoder.frame.passes.push(pass);
      passes.set(result, {pass, encoder, commandEncoder: this});
      return result;
    });
  }
  wrap(GPURenderPassEncoder.prototype, 'setPipeline', function(original, args) {
    const state = passes.get(this); if (state) state.pass.pipeline = pipelines.get(args[0]);
    return original.apply(this, args);
  });
  wrap(GPURenderPassEncoder.prototype, 'setIndexBuffer', function(original, args) {
    const state = passes.get(this); if (state) state.pass.indexBuffer = {id: id(args[0]), offset: args[2] || 0, format: args[1]};
    return original.apply(this, args);
  });
  wrap(GPURenderPassEncoder.prototype, 'setVertexBuffer', function(original, args) {
    const state = passes.get(this); if (state && args[1]) state.pass.vertexBuffers[args[0]] = {id: id(args[1]), offset: args[2] || 0};
    return original.apply(this, args);
  });
  for (const method of ['draw', 'drawIndexed', 'drawIndirect', 'drawIndexedIndirect']) {
    wrap(GPURenderPassEncoder.prototype, method, function(original, args) {
      const state = passes.get(this);
      if (state) {
        const indirect = method.endsWith('Indirect');
        state.pass.draws.push({method, pipeline: state.pass.pipeline, indexBuffer: state.pass.indexBuffer,
          vertexBuffers: {...state.pass.vertexBuffers},
          ...(indirect ? {indirect: {buffer: args[0], offset: args[1], indexed: method === 'drawIndexedIndirect'}} :
            {elements: args[0], instances: args[1] ?? 1, first: args[2] ?? 0})});
      }
      return original.apply(this, args);
    });
  }
  wrap(GPURenderPassEncoder.prototype, 'executeBundles', function(original, args) {
    const state = passes.get(this); if (state) state.pass.unsupportedBundles += args[0].length;
    return original.apply(this, args);
  });
  for (const method of ['dispatchWorkgroups', 'dispatchWorkgroupsIndirect']) {
    wrap(GPUComputePassEncoder.prototype, method, function(original, args) {
      const state = passes.get(this); if (state) state.pass.dispatches++;
      return original.apply(this, args);
    });
  }
  // Copy at each pass boundary: later compute passes may overwrite the same args.
  wrap(GPURenderPassEncoder.prototype, 'end', function(original, args) {
    const result = original.apply(this, args), state = passes.get(this);
    if (!state) return result;
    const groups = new Map();
    for (const draw of state.pass.draws) if (draw.indirect) {
      const {buffer, offset, indexed} = draw.indirect;
      const group = groups.get(buffer) || {min: offset, max: offset, draws: []};
      group.min = Math.min(group.min, offset); group.max = Math.max(group.max, offset + (indexed ? 20 : 16));
      group.draws.push(draw); groups.set(buffer, group);
    }
    for (const [source, group] of groups) {
      const buffer = nativeBuffer.call(state.encoder.device, {size: group.max - group.min,
        usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ});
      state.commandEncoder.copyBufferToBuffer(source, group.min, buffer, 0, group.max - group.min);
      state.encoder.frame.reads.push({buffer, decode: bytes => {
        for (const draw of group.draws) {
          const words = new Uint32Array(bytes, draw.indirect.offset - group.min, draw.indirect.indexed ? 5 : 4);
          draw.elements = words[0]; draw.instances = words[1]; draw.first = words[2];
          draw.indirect = {bufferId: id(source), offset: draw.indirect.offset};
        }
      }});
    }
    return result;
  });
  wrap(GPUCommandEncoder.prototype, 'finish', function(original, args) {
    const encoder = encoders.get(this);
    if (encoder?.queryCount) {
      const size = encoder.queryCount * 8;
      const resolved = nativeBuffer.call(encoder.device, {size, usage: GPUBufferUsage.QUERY_RESOLVE | GPUBufferUsage.COPY_SRC});
      const buffer = nativeBuffer.call(encoder.device, {size, usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ});
      this.resolveQuerySet(encoder.querySet, 0, encoder.queryCount, resolved, 0);
      this.copyBufferToBuffer(resolved, 0, buffer, 0, size);
      encoder.frame.reads.push({buffer, decode: bytes => {
        const times = new BigUint64Array(bytes);
        for (const pass of encoder.passes) if (pass.query !== null)
          pass.gpuMs = Number(times[pass.query + 1] - times[pass.query]) / 1e6;
        resolved.destroy(); encoder.querySet.destroy();
      }});
    }
    return original.apply(this, args);
  });
  wrap(GPUQueue.prototype, 'submit', function(original, args) {
    if (current) { current.submits++; current.commandBuffers += args[0].length; }
    return original.apply(this, args);
  });
  wrap(GPUQueue.prototype, 'writeBuffer', function(original, args) {
    if (current) {
      const elementBytes = args[2].BYTES_PER_ELEMENT || 1;
      current.bufferWrites++;
      current.bufferBytes += (args[4] ?? (args[2].byteLength / elementBytes - (args[3] || 0))) * elementBytes;
    }
    return original.apply(this, args);
  });
  const raf = window.requestAnimationFrame.bind(window);
  window.requestAnimationFrame = callback => raf(time => {
    const owner = request;
    if (!owner || owner.started >= owner.count) return callback(time);
    const frame = {mode: owner.mode, rafTime: time, cpuMs: 0, submits: 0, commandBuffers: 0,
      bufferWrites: 0, bufferBytes: 0, passes: [], reads: []};
    current = frame; const start = performance.now();
    try { return callback(time); }
    finally {
      frame.cpuMs = performance.now() - start; current = null;
      if (frame.submits) {
        owner.started++;
        owner.frames.push((async () => {
          await Promise.all(frame.reads.map(async read => {
            await read.buffer.mapAsync(GPUMapMode.READ); read.decode(read.buffer.getMappedRange());
            read.buffer.unmap(); read.buffer.destroy();
          }));
          delete frame.reads;
          for (const pass of frame.passes) for (const draw of pass.draws)
            draw.triangles = triangles(draw.pipeline?.topology, draw.elements, draw.instances);
          owner.finished++;
          return frame;
        })());
        if (owner.started === owner.count) {
          request = null;
          Promise.all(owner.frames).then(frames => {
            clearTimeout(owner.timeout); owner.resolve({capabilities, failures, frames});
          }, error => { clearTimeout(owner.timeout); owner.reject(error); });
        }
      }
    }
  });
  window.renderProbe = {
    capture(mode = 'gpu', count = 8) {
      if (request) throw new Error('Capture already active');
      return new Promise((resolve, reject) => {
        request = latest = {mode, count, started: 0, finished: 0, frames: [], resolve, reject};
        const owner = request;
        owner.timeout = setTimeout(() => {
          if (request === owner) request = null;
          reject(new Error(`Probe timeout: ${owner.mode}, ${owner.started}/${owner.count} submitted, ${owner.finished} read back`));
        }, 120000);
      });
    }, capabilities, failures,
    status: () => latest && {mode: latest.mode, started: latest.started, count: latest.count, finished: latest.finished},
  };
})();

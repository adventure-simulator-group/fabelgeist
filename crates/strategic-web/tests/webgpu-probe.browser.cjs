const test = require('node:test');
const assert = require('node:assert/strict');
const http = require('node:http');
const path = require('node:path');
const {chromium} = require('playwright');

test('probe reads GPU-written indirect arguments and timestamps actual passes', async () => {
  const server = http.createServer((_, response) => {
    response.setHeader('Content-Type', 'text/html');
    response.end('<canvas width="32" height="32"></canvas>');
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const browser = await chromium.launch({headless: true, channel: process.platform === 'win32' ? 'msedge' : undefined,
    args: ['--enable-unsafe-webgpu']});
  try {
    const page = await browser.newPage();
    await page.addInitScript({path: path.join(__dirname, 'webgpu-probe.js')});
    await page.goto(`http://127.0.0.1:${server.address().port}`);
    const capture = await page.evaluate(async () => {
      const adapter = await navigator.gpu.requestAdapter();
      const device = await adapter.requestDevice();
      const context = document.querySelector('canvas').getContext('webgpu');
      const format = navigator.gpu.getPreferredCanvasFormat();
      context.configure({device, format});
      const module = device.createShaderModule({code: `
        @vertex fn vs(@builtin(vertex_index) index: u32) -> @builtin(position) vec4f {
          var points = array(vec2f(-0.5, -0.5), vec2f(0.5, -0.5), vec2f(0.0, 0.5));
          return vec4f(points[index], 0.0, 1.0);
        }
        @fragment fn fs() -> @location(0) vec4f { return vec4f(1.0); }`});
      const pipeline = device.createRenderPipeline({layout: 'auto', vertex: {module, entryPoint: 'vs'},
        fragment: {module, entryPoint: 'fs', targets: [{format}]}});
      const indices = device.createBuffer({size: 12, usage: GPUBufferUsage.INDEX | GPUBufferUsage.COPY_DST});
      device.queue.writeBuffer(indices, 0, new Uint32Array([0, 1, 2]));
      const indirect = device.createBuffer({size: 16, usage: GPUBufferUsage.STORAGE | GPUBufferUsage.INDIRECT | GPUBufferUsage.COPY_DST});
      const compute = device.createComputePipeline({layout: 'auto', compute: {entryPoint: 'main',
        module: device.createShaderModule({code: `
          @group(0) @binding(0) var<storage, read_write> args: array<u32>;
          @compute @workgroup_size(1) fn main() { args[0] = 3; args[1] += 5; args[2] = 0; args[3] = 0; }`})}});
      const binding = device.createBindGroup({layout: compute.getBindGroupLayout(0), entries: [{binding: 0, resource: {buffer: indirect}}]});
      const result = window.renderProbe.capture('gpu', 2);
      let remaining = 2;
      function frame() {
        device.queue.writeBuffer(indirect, 0, new Uint32Array(4));
        const encoder = device.createCommandEncoder();
        const update = encoder.beginComputePass({label: 'known-compute'});
        update.setPipeline(compute); update.setBindGroup(0, binding); update.dispatchWorkgroups(1); update.end();
        const render = encoder.beginRenderPass({label: 'known-render', colorAttachments: [{
          view: context.getCurrentTexture().createView(), loadOp: 'clear', storeOp: 'store'}]});
        render.setPipeline(pipeline); render.setIndexBuffer(indices, 'uint32');
        render.drawIndexed(3, 4); render.drawIndirect(indirect, 0); render.end();
        const overwrite = encoder.beginComputePass({label: 'overwrite-indirect'});
        overwrite.setPipeline(compute); overwrite.setBindGroup(0, binding); overwrite.dispatchWorkgroups(1); overwrite.end();
        const second = encoder.beginRenderPass({label: 'after-overwrite', colorAttachments: [{
          view: context.getCurrentTexture().createView(), loadOp: 'load', storeOp: 'store'}]});
        second.setPipeline(pipeline); second.drawIndirect(indirect, 0); second.end();
        device.queue.submit([encoder.finish()]);
        if (--remaining) requestAnimationFrame(frame);
      }
      requestAnimationFrame(frame);
      return result;
    });
    assert.deepEqual(capture.failures, []);
    assert.equal(capture.frames.length, 2);
    for (const frame of capture.frames) {
      const render = frame.passes.find(pass => pass.kind === 'render');
      assert.equal(render.draws.length, 2);
      assert.equal(render.draws.reduce((sum, draw) => sum + draw.triangles, 0), 9);
      assert.equal(render.draws[1].instances, 5);
      assert.equal(frame.passes.find(pass => pass.label === 'after-overwrite').draws[0].instances, 10,
        'Each pass must retain its own argument values when compute reuses the buffer');
      assert(frame.passes.every(pass => pass.gpuMs !== null && pass.gpuMs >= 0));
      assert(frame.cpuMs > 0);
    }
  } finally { await browser.close(); server.close(); }
});

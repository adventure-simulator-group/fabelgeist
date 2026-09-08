// Exercise the production rock projection WGSL on WebGPU against displaced-height normals.
// Requires Playwright and a WebGPU-capable Chromium browser.
const {chromium} = require('playwright');
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');

function shaderFunction(source, name) {
  const start = source.indexOf(`fn ${name}(`);
  assert(start >= 0, `Missing production shader function ${name}`);
  let end = source.indexOf('{', start) + 1;
  let depth = 1;
  while (depth) {
    assert(end < source.length, `Unclosed function ${name}`);
    if (source[end] === '{') depth++;
    if (source[end] === '}') depth--;
    end++;
  }
  return source.slice(start, end);
}

async function main() {
  const source = fs.readFileSync(path.join(__dirname, '../assets/shaders/tactical_rock.wgsl'), 'utf8');
  const rotations = source.match(/const ROCK_PROJECTION_ROTATIONS = [\s\S]*?\n\);/);
  assert(rotations, 'Missing production UV rotations');
  const production = rotations[0] + '\n' + ['axis_uvs', 'unproject_normal', 'triplanar_surface_normal']
    .map(name => shaderFunction(source, name)).join('\n');
  const code = `
struct TacticalRockMaterial { surface: vec4<f32>, geology: vec4<f32> }
const rock = TacticalRockMaterial(vec4<f32>(0.0), vec4<f32>(0.37, 1.0, 0.0, 1.0));
${production}
fn relief(uv: vec2<f32>) -> f32 {
  return 0.2 * sin(uv.x * 3.1) + 0.15 * cos(uv.y * 2.7);
}
fn baked_normal(uv: vec2<f32>) -> vec3<f32> {
  let step = 0.001;
  let du = (relief(uv + vec2<f32>(step, 0.0)) - relief(uv - vec2<f32>(step, 0.0))) / (2.0 * step);
  let dv = (relief(uv + vec2<f32>(0.0, step)) - relief(uv - vec2<f32>(0.0, step))) / (2.0 * step);
  return normalize(vec3<f32>(-du, dv, 1.0));
}
fn projected_height(p: vec3<f32>, axis: u32) -> f32 {
  return relief(axis_uvs(p)[axis]);
}
@group(0) @binding(0) var<storage, read_write> results: array<vec4<f32>>;
@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
  let axis = id.x % 3u;
  let side = select(-1.0, 1.0, (id.x / 3u) % 2u == 0u);
  let p = vec3<f32>(0.17, 0.4, 0.15) + f32(id.x) * vec3<f32>(0.113, 0.013, 0.07);
  var face_normal = vec3<f32>(0.0);
  face_normal[axis] = side;
  let uv = axis_uvs(p);
  let actual = triplanar_surface_normal(baked_normal(uv[0]), baked_normal(uv[1]), baked_normal(uv[2]), abs(face_normal), face_normal);
  var gradient = vec3<f32>(0.0);
  for (var a = 0u; a < 3u; a++) {
    var delta = vec3<f32>(0.0);
    delta[a] = 0.001;
    gradient[a] = (projected_height(p + delta, axis) - projected_height(p - delta, axis)) / 0.002;
  }
  let geometric = normalize(face_normal - gradient);
  results[id.x] = vec4<f32>(dot(actual, geometric), actual);
}`;
  const browser = await chromium.launch({headless: true,
    channel: process.platform === 'win32' ? 'msedge' : undefined,
    args: ['--enable-unsafe-webgpu']});
  try {
    const page = await browser.newPage();
    await page.route('**/normal-audit', route => route.fulfill({contentType: 'text/html', body: '<!doctype html><title>Normal projection test</title>'}));
    await page.goto(new URL('/normal-audit', process.argv[2] || 'http://127.0.0.1:8783').href);
    const result = await page.evaluate(async code => {
      const adapter = await navigator.gpu?.requestAdapter();
      if (!adapter) throw new Error('WebGPU adapter required');
      const device = await adapter.requestDevice();
      try {
        const module = device.createShaderModule({code});
        const messages = (await module.getCompilationInfo()).messages.filter(m => m.type === 'error');
        if (messages.length) throw new Error(messages.map(m => m.message).join('\n'));
        const pipeline = await device.createComputePipelineAsync({layout: 'auto', compute: {module, entryPoint: 'main'}});
        const count = 72, size = count * 16;
        const output = device.createBuffer({size, usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC});
        const readback = device.createBuffer({size, usage: GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_DST});
        const group = device.createBindGroup({layout: pipeline.getBindGroupLayout(0), entries: [{binding: 0, resource: {buffer: output}}]});
        const encoder = device.createCommandEncoder();
        const pass = encoder.beginComputePass();
        pass.setPipeline(pipeline); pass.setBindGroup(0, group); pass.dispatchWorkgroups(count); pass.end();
        encoder.copyBufferToBuffer(output, 0, readback, 0, size);
        device.queue.submit([encoder.finish()]);
        await readback.mapAsync(GPUMapMode.READ);
        const values = new Float32Array(readback.getMappedRange()).slice();
        readback.unmap(); output.destroy(); readback.destroy();
        return {cases: count, minimumAlignment: Math.min(...Array.from({length: count}, (_, i) => values[i * 4]))};
      } finally { device.destroy(); }
    }, code);
    assert(result.minimumAlignment > 0.9999, JSON.stringify(result));
    console.log(JSON.stringify(result));
  } finally { await browser.close(); }
}
main().catch(error => { console.error(error); process.exitCode = 1; });

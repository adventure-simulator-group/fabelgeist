const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const http = require('node:http');
const {chromium} = require('playwright');

// Public eidolon compute ABI, also exercised by the integrated scene fixture.
const bindings = `
struct Instance { position:vec4<f32>, rotation:f32, index:u32, batch_id:u32, seed:u32 }
struct Draw { indices:u32, instance_count:atomic<u32>, first:u32, base:i32, start:u32 }
struct Batch { id:u32, start_index:u32, end_index:u32 }
struct Uniforms { color:vec4<f32>, visibility_range:vec4<f32>, world_from_local:mat4x4<f32>, previous:mat4x4<f32>, center:vec4<f32>, extent:vec4<f32> }
struct Camera { view_pos:vec4<f32>, frustum:array<vec4<f32>,6> }
@group(0) @binding(0) var<storage,read> source_buffer:array<Instance>;
@group(0) @binding(1) var<storage,read_write> instance_buffer:array<Instance>;
@group(0) @binding(2) var<storage,read_write> indirect_args:array<Draw>;
@group(0) @binding(3) var<storage,read> batch_offsets:array<Batch>;
@group(1) @binding(0) var<storage,read> instance_uniforms:array<Uniforms>;
@group(2) @binding(0) var<uniform> camera:Camera;
`;

test('grass culling rejects individual tufts and retains animated/scaled edge bounds', async () => {
  const shader = fs.readFileSync(path.resolve(__dirname, '../../../assets/shaders/tactical_grass_cull.wgsl'), 'utf8');
  const code = bindings + shader.replace(/^#import.*$/gm, '').replaceAll('instance.pos_and_scale', 'instance.position');
  const server = http.createServer((_, response) => response.end('<title>Grass cull test</title>'));
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const browser = await chromium.launch({headless:true, channel:process.platform === 'win32' ? 'msedge' : undefined,
    args:['--enable-unsafe-webgpu']});
  try {
    const page = await browser.newPage();
    await page.goto(`http://127.0.0.1:${server.address().port}`);
    const result = await page.evaluate(async code => {
      const adapter = await navigator.gpu.requestAdapter();
      const device = await adapter.requestDevice();
      const errors = [];
      device.addEventListener('uncapturederror', event => errors.push(event.error.message));
      const shader = device.createShaderModule({code});
      const storage = type => ({visibility:GPUShaderStage.COMPUTE,buffer:{type}});
      const layouts = [
        ['read-only-storage','storage','storage','read-only-storage'],
        ['read-only-storage'], ['uniform'],
      ].map(types => device.createBindGroupLayout({entries:types.map((type,binding)=>({binding,...storage(type)}))}));
      const layout=device.createPipelineLayout({bindGroupLayouts:layouts});
      const pipeline = await device.createComputePipelineAsync({layout,compute:{module:shader,entryPoint:'main'}});
      const reset = await device.createComputePipelineAsync({layout,compute:{module:shader,entryPoint:'reset'}});
      const create = (data, uniform=false) => {
        const buffer = device.createBuffer({size:data.byteLength, usage:GPUBufferUsage.COPY_DST|GPUBufferUsage.COPY_SRC|
          (uniform ? GPUBufferUsage.UNIFORM : GPUBufferUsage.STORAGE)});
        device.queue.writeBuffer(buffer,0,data); return buffer;
      };
      const points = [[0,1,0], [10,1,0], [-1.2,1,0], [-2,1,0], [-2,4,0], [21,1,0], [0,1,0xffffffff], [0,1,2]];
      const source = new ArrayBuffer(points.length*32), floats = new Float32Array(source), uints = new Uint32Array(source);
      points.forEach(([x,scale,batch],i) => {floats.set([x,0,0,scale],i*8);uints.set([i,batch,0],i*8+5);});
      const input = create(source), output = create(new Uint32Array(points.length*8));
      const draws = create(new Uint32Array([36,0,0,0,0])), offsets=create(new Uint32Array([0,0,points.length]));
      const uniforms = new Float32Array(48);
      uniforms.set([1,1,1,0.25,0,0,20,20]);
      const identity=[1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1];
      uniforms.set(identity,8); uniforms.set(identity,24);
      const material = create(uniforms);
      const bind = (index,buffers) => device.createBindGroup({layout:pipeline.getBindGroupLayout(index),
        entries:buffers.map((buffer,binding)=>({binding,resource:{buffer}}))});
      const captures=[];
      for (const centre of [0,10]) {
        const camera=new Float32Array([centre,0,0,0, 1,0,0,1-centre, -1,0,0,1+centre, 0,1,0,1, 0,-1,0,1, 0,0,1,1, 0,0,-1,1]);
        const view=create(camera,true);
        const encoder=device.createCommandEncoder();
        const pass=encoder.beginComputePass(); pass.setPipeline(reset);
        pass.setBindGroup(0,bind(0,[input,output,draws,offsets]));pass.setBindGroup(1,bind(1,[material]));pass.setBindGroup(2,bind(2,[view]));
        pass.dispatchWorkgroups(1);pass.setPipeline(pipeline);pass.dispatchWorkgroups(1,2);pass.end();
        const read=device.createBuffer({size:20+source.byteLength,usage:GPUBufferUsage.COPY_DST|GPUBufferUsage.MAP_READ});
        encoder.copyBufferToBuffer(draws,0,read,0,20);encoder.copyBufferToBuffer(output,0,read,20,source.byteLength);
        device.queue.submit([encoder.finish()]);await read.mapAsync(GPUMapMode.READ);
        const data=new Uint32Array(read.getMappedRange()),count=data[1];
        captures.push(Array.from({length:count},(_,i)=>data[5+i*8+5]).sort((a,b)=>a-b));
        read.unmap();read.destroy();
      }
      await device.queue.onSubmittedWorkDone();device.destroy();
      return {captures,errors};
    }, code);
    assert.deepEqual(result.errors,[]);
    assert.deepEqual(result.captures,[[0,2,4],[1]],'camera changes recull retained sources; edge tufts survive');
  } finally {await browser.close();await new Promise(resolve=>server.close(resolve));}
});

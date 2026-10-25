const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const http = require('node:http');
const {chromium} = require('playwright');

test('room lighting replaces outdoor ambient, preserves direct lights and exterior PBR', async () => {
  const source = fs.readFileSync(path.resolve(__dirname, '../../../assets/shaders/tactical_interior_lighting.wgsl'), 'utf8')
    .replace(/^#.*$/gm, '').replaceAll('#{MATERIAL_BIND_GROUP}', '0');
  const server = http.createServer((_, response) => response.end('<title>Interior shader correctness</title>'));
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const browser = await chromium.launch({headless:true, channel:process.platform==='win32'?'msedge':undefined,
    args:['--enable-unsafe-webgpu']});
  try {
    const page = await browser.newPage();
    await page.goto(`http://127.0.0.1:${server.address().port}`);
    const result = await page.evaluate(async source => {
      const adapter = await navigator.gpu.requestAdapter();
      const device = await adapter.requestDevice();
      const errors=[];
      device.addEventListener('uncapturederror', e=>errors.push(e.error.message));
      // Stub only Bevy's external PBR entry point. Execute the production room
      // lookup, occlusion policy and irradiance calculation on the actual GPU.
      const code = `
        struct Material { base_color: vec4<f32>, metallic:f32 }
        struct PbrInput { world_position:vec4<f32>, N:vec3<f32>, material:Material,
          diffuse_occlusion:vec3<f32>, specular_occlusion:f32 }
        struct View { exposure:f32 }
        const view = View(1.0);
        fn apply_pbr_lighting(pbr:PbrInput)->vec4<f32> {
          return vec4(vec3(2.0) + pbr.diffuse_occlusion*100.0 + pbr.specular_occlusion*50.0,1.0);
        }
        ${source}
        @group(1) @binding(0) var<storage,read_write> output:array<vec4<f32>>;
        @compute @workgroup_size(1) fn main(@builtin(global_invocation_id) id:vec3<u32>) {
          var pbr:PbrInput;
          pbr.world_position=vec4(0.5+f32(id.x),0.5,0.5,1.0);
          pbr.N=vec3(0.0,1.0,0.0);
          pbr.material=Material(vec4(1.0),0.0);
          pbr.diffuse_occlusion=vec3(0.5);
          pbr.specular_occlusion=1.0;
          output[id.x]=apply_interior_lighting(pbr);
        }`;
      const data=new ArrayBuffer(48+16*144+2*32), f=new Float32Array(data), u=new Uint32Array(data);
      f.set([100,100,100,1, 1000,1000,1000,0]); u[8]=1;
      const b=12;
      f.set([1,0,0,0, 0,1,0,0, 0,0,1,0, 0,0,0,1],b);
      f.set([0,0,0,0, 2,1,1,0, 0,0,0,1],b+16);
      u.set([2,1,1,0],b+28); f[b+32]=1;
      f.set([.25,.25,.25,1, .25,.25,.25,0, 0,0,0,2, 0,0,0,0],(48+16*144)/4);
      const field=device.createBuffer({size:data.byteLength,usage:GPUBufferUsage.STORAGE|GPUBufferUsage.COPY_DST});
      device.queue.writeBuffer(field,0,data);
      const output=device.createBuffer({size:48,usage:GPUBufferUsage.STORAGE|GPUBufferUsage.COPY_SRC});
      const module=device.createShaderModule({code});
      const pipeline=device.createComputePipeline({layout:'auto',compute:{module,entryPoint:'main'}});
      const bindings=[device.createBindGroup({layout:pipeline.getBindGroupLayout(0),entries:[{binding:200,resource:{buffer:field}}]}),
        device.createBindGroup({layout:pipeline.getBindGroupLayout(1),entries:[{binding:0,resource:{buffer:output}}]})];
      const capture=async()=>{
        const encoder=device.createCommandEncoder(), pass=encoder.beginComputePass();
        pass.setPipeline(pipeline);bindings.forEach((b,i)=>pass.setBindGroup(i,b));pass.dispatchWorkgroups(3);pass.end();
        const read=device.createBuffer({size:48,usage:GPUBufferUsage.COPY_DST|GPUBufferUsage.MAP_READ});
        encoder.copyBufferToBuffer(output,0,read,0,48);device.queue.submit([encoder.finish()]);
        await read.mapAsync(GPUMapMode.READ);return [...new Float32Array(read.getMappedRange())];
      };
      const day=await capture();
      device.queue.writeBuffer(field,0,new Float32Array(4));
      const night=await capture();
      return {day,night,errors};
    },source);
    assert.deepEqual(result.errors,[]);
    assert(Math.abs(result.day[0]-(2+50*.025+25*.5/Math.PI))<.0001);
    assert.equal(result.day[4],2,'a dark room keeps direct lights without outdoor ambient');
    assert.equal(result.day[8],102,'exterior PBR is unchanged');
    assert.equal(result.night[0],2,'night does not reopen the outdoor ambient leak');
    assert.equal(result.night[8],102);
  } finally {await browser.close();server.close();}
});

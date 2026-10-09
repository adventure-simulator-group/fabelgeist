const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const http = require('node:http');
const {chromium} = require('playwright');

test('city compute selects LOD per view, rejects hidden buildings and compacts matching clusters', async () => {
  const shader = fs.readFileSync(path.resolve(__dirname, '../../../assets/shaders/tactical_city_cull.wgsl'), 'utf8');
  const server = http.createServer((_, response) => {
    response.setHeader('Content-Type', 'text/html'); response.end('<title>City GPU test</title>');
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const browser = await chromium.launch({headless: true, channel: process.platform === 'win32' ? 'msedge' : undefined,
    args: ['--enable-unsafe-webgpu']});
  try {
    const page = await browser.newPage();
    await page.goto(`http://127.0.0.1:${server.address().port}`);
    const result = await page.evaluate(async shader => {
      const adapter = await navigator.gpu.requestAdapter();
      const device = await adapter.requestDevice();
      const failures = [];
      device.addEventListener('uncapturederror', event => failures.push(event.error.message));
      const create = (data, extra = 0) => {
        const buffer = device.createBuffer({size: data.byteLength,
          usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST | GPUBufferUsage.COPY_SRC | extra});
        device.queue.writeBuffer(buffer, 0, data); return buffer;
      };
      const identity = [1,0,0,0, 0,1,0,0, 0,0,1,0, 0,0,0,1];
      const data = new ArrayBuffer(3 * 112), floats = new Float32Array(data), words = new Uint32Array(data);
      for (let i=0; i<3; i++) {
        floats.set(identity, i*28); floats.set([i===1 ? 10 : 0, 0, i===2 ? 2 : 0.5, 0.2], i*28+16);
        words[i*28+20] = 6; // Both facade and shell exist.
      }
      const buildings = create(data);
      const selection = create(new Uint32Array(15).fill(0xffffffff));
      // A range spanning more than one workgroup's emission loop, partial tail,
      // hidden ranges, and two scratch slots exercise reservation boundaries.
      // Geometry identifies a shared owner-list span; instance words retain
      // the placement count, cluster count and vertices per cluster.
      const jobData = new Uint32Array(67*8);
      jobData.set([
        0,10,12483,1, 1,66,192,0,
        0,20,198,2, 1,2,192,0,
        1,30,3,1, 1,1,192,0,
        1,40,3,2, 1,1,192,0,
        2,50,384,1, 1,2,192,0,
        2,60,3,2, 1,1,192,0,
      ]);
      jobData.set([0,100,3,258, 1,1,192,0],66*8); // A real range in the second dispatch row.
      const jobs = create(jobData);
      const owners = create(new Uint32Array([0,1,2]));
      const capacity = 74;
      const output = create(new Uint32Array(2 * capacity * 2).fill(0xffffffff));
      const indirect = create(new Uint32Array([192,0,0,0, 192,0,0,capacity]), GPUBufferUsage.INDIRECT | GPUBufferUsage.VERTEX);
      const layout = device.createBindGroupLayout({entries: [
        {binding:0,visibility:GPUShaderStage.COMPUTE,buffer:{type:'uniform'}},
        ...[1,2,3,4,5,6].map(binding => ({binding,visibility:GPUShaderStage.COMPUTE,
          buffer:{type: [1,3,6].includes(binding) ? 'read-only-storage' : 'storage'}})),
      ]});
      const module = device.createShaderModule({code:shader});
      const pipelineLayout = device.createPipelineLayout({bindGroupLayouts:[layout]});
      const select = device.createComputePipeline({layout:pipelineLayout,compute:{module,entryPoint:'select_buildings'}});
      const compact = device.createComputePipeline({layout:pipelineLayout,compute:{module,entryPoint:'compact_ranges'}});
      const encoder = device.createCommandEncoder();
      const reads = [];
      const read = (buffer,size) => {
        const target = device.createBuffer({size,usage:GPUBufferUsage.COPY_DST|GPUBufferUsage.MAP_READ});
        encoder.copyBufferToBuffer(buffer,0,target,0,size); return target;
      };
      for (let iteration=0; iteration<6; iteration++) {
        const slot = iteration === 3 ? 0 : iteration > 3 ? iteration - 1 : iteration;
        const scratch = slot % 2;
        const params = new ArrayBuffer(144), f = new Float32Array(params), u = new Uint32Array(params);
        f.set(identity); f.set([0,0,5,slot===2 ? 0 : 1],16);
        f.set([iteration===3 ? 224 : slot===1 ? 64 : 256,48,180,iteration===3 ? 1 : 0],20);
        u.set([3,capacity,slot,scratch],24); // LOD history persists independently of scratch reuse.
        u[28] = iteration === 4 ? 1 : 0; // Shadow role is distinct from orthographic projection.
        const uniform = create(params,GPUBufferUsage.UNIFORM);
        const binding = device.createBindGroup({layout,entries:[uniform,buildings,selection,jobs,output,indirect,owners]
          .map((buffer,binding)=>({binding,resource:{buffer}}))});
        if (iteration !== 3) encoder.clearBuffer(indirect,scratch*16+4,4);
        for(const pipeline of iteration===3 ? [select] : [select,compact]) {
          const pass = encoder.beginComputePass(); pass.setPipeline(pipeline); pass.setBindGroup(0,binding);
          // The second row includes excess workgroups which must emit nothing.
          if (pipeline === compact) pass.dispatchWorkgroups(1,2);
          else pass.dispatchWorkgroups(1);
          pass.end();
        }
        if (iteration !== 3) reads.push(read(indirect,32),read(output,output.size));
      }
      reads.push(read(selection,60));
      // Bevy's mesh specializer requires slot zero. An empty, zero-stride
      // layout uses the argument buffer as an unread vertex-buffer binding.
      const drawShader = device.createShaderModule({code: `
        @vertex fn vertex() -> @builtin(position) vec4<f32> { return vec4(0.,0.,0.,1.); }
        @fragment fn fragment() -> @location(0) vec4<f32> { return vec4(1.); }
      `});
      const drawPipeline = device.createRenderPipeline({layout:'auto',
        vertex:{module:drawShader,entryPoint:'vertex',buffers:[{arrayStride:0,attributes:[]}]},
        fragment:{module:drawShader,entryPoint:'fragment',targets:[{format:'rgba8unorm'}]}});
      const target = device.createTexture({size:[1,1],format:'rgba8unorm',usage:GPUTextureUsage.RENDER_ATTACHMENT});
      const pass = encoder.beginRenderPass({colorAttachments:[{view:target.createView(),loadOp:'clear',storeOp:'store',clearValue:[0,0,0,0]}]});
      pass.setPipeline(drawPipeline); pass.setVertexBuffer(0,indirect); pass.drawIndirect(indirect,0); pass.end();
      device.queue.submit([encoder.finish()]);
      const arrays = await Promise.all(reads.map(async buffer => {
        await buffer.mapAsync(GPUMapMode.READ); return [...new Uint32Array(buffer.getMappedRange())];
      }));
      const prop = new ArrayBuffer(112), propFloats=new Float32Array(prop), propWords=new Uint32Array(prop);
      propFloats.set(identity);propFloats.set([0,0,0.5,0.2],16);
      propWords.set([4,1],20);propFloats.set([2,10],22);
      const props=create(prop), propSelection=create(new Uint32Array(1));
      const propResults=[];
      for (const distance of [2,20]) for (const shadow of [0,1,2]) {
        const params=new ArrayBuffer(144), f=new Float32Array(params), u=new Uint32Array(params);
        f.set(identity);f.set([distance,0,0,1],16);f.set([256,48,180,0],20);u.set([1,capacity,0,0],24);u[28]=shadow;u[29]=shadow===2 ? 1 : 0;
        const uniform=create(params,GPUBufferUsage.UNIFORM);
        const binding=device.createBindGroup({layout,entries:[uniform,props,propSelection,jobs,output,indirect,owners]
          .map((buffer,binding)=>({binding,resource:{buffer}}))});
        const command=device.createCommandEncoder(), pass=command.beginComputePass();
        pass.setPipeline(select);pass.setBindGroup(0,binding);pass.dispatchWorkgroups(1);pass.end();
        const read=device.createBuffer({size:4,usage:GPUBufferUsage.COPY_DST|GPUBufferUsage.MAP_READ});
        command.copyBufferToBuffer(propSelection,0,read,0,4);device.queue.submit([command.finish()]);
        await read.mapAsync(GPUMapMode.READ);propResults.push(new Uint32Array(read.getMappedRange())[0]);
        read.unmap();read.destroy();
      }
      // The finite camera plane is independent of the infinite-far matrix.
      const finiteResults=[];
      for (const far of [0.1,0.4,0.8]) {
        const params=new ArrayBuffer(144), f=new Float32Array(params), u=new Uint32Array(params);
        f.set(identity);f.set([0,0,0,1],16);f.set([256,48,180,0],20);u.set([1,capacity,0,0],24);
        f.set([0,0,-1,far],32);
        const uniform=create(params,GPUBufferUsage.UNIFORM);
        const binding=device.createBindGroup({layout,entries:[uniform,props,propSelection,jobs,output,indirect,owners]
          .map((buffer,binding)=>({binding,resource:{buffer}}))});
        const command=device.createCommandEncoder(), pass=command.beginComputePass();
        pass.setPipeline(select);pass.setBindGroup(0,binding);pass.dispatchWorkgroups(1);pass.end();
        const read=device.createBuffer({size:4,usage:GPUBufferUsage.COPY_DST|GPUBufferUsage.MAP_READ});
        command.copyBufferToBuffer(propSelection,0,read,0,4);device.queue.submit([command.finish()]);
        await read.mapAsync(GPUMapMode.READ);finiteResults.push(new Uint32Array(read.getMappedRange())[0]);
        read.unmap();read.destroy();
      }
      return {arrays,failures,propResults,finiteResults};
    }, shader);
    assert.deepEqual(result.failures, []);
    assert.deepEqual(result.propResults,[4,4,4,0,0,4], 'scenery follows parent camera distance; shared point/spot shadows retain casters for all cameras');
    assert.deepEqual(result.finiteResults,[0,4,4], 'finite far plane rejects distant bounds and retains intersecting bounds');
    const [args0,visible0,args1,visible1,args2,visible2,shadowArgs,shadowVisible,orthoArgs,orthoVisible,selection] = result.arrays;
    assert.deepEqual(selection,[2,0,0, 4,0,0, 2,0,0, 4,0,4, 2,0,0],
      'LOD history remains per view while scratch storage is reused');
    assert.deepEqual(args0.slice(0,4),[192,66,0,0]);
    assert.deepEqual(args1.slice(4),[192,3,0,74]);
    assert.deepEqual(args2.slice(0,4),[192,66,0,0]);
    const pairs = (words,start,count) => Array.from({length:count},(_,i)=>words.slice((start+i)*2,(start+i)*2+2))
      .sort((a,b)=>a[0]-b[0] || a[1]-b[1]);
    const largeRange = Array.from({length:66},(_,i)=>[0,i]);
    assert.deepEqual(pairs(visible0,0,66),largeRange);
    assert.deepEqual(pairs(visible1,74,3),[[1,0],[1,1],[66,0]],
      'camera shell includes facade overlays');
    assert.deepEqual(pairs(visible2,0,66),largeRange);
    assert.deepEqual(visible1.slice(0,136),visible0.slice(0,136),
      'writing scratch slot one preserves slot zero');
    assert.deepEqual(visible2.slice(148),visible1.slice(148),
      'reusing slot zero preserves slot one');
    assert.deepEqual(shadowArgs.slice(4),[192,3,0,74]);
    assert.deepEqual(pairs(shadowVisible,74,3),[[1,0],[1,1],[5,2]],
      'shadow shells retain off-camera casters and omit facade overlays');
    assert.deepEqual(orthoArgs.slice(0,4),[192,66,0,0]);
    assert.deepEqual(pairs(orthoVisible,0,66),largeRange,
      'orthographic cameras still select facade LOD and obey their near plane');
  } finally { await browser.close(); server.close(); }
});

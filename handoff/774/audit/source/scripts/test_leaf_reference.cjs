// Compare the CPU morphology port with its pinned upstream WGSL on an actual WebGPU device.
// First run: FABELGEIST_LEAF_PARITY_OUTPUT=<file> cargo test -p adventuresim-procedural-textures export_reference_vectors -- --ignored
// Then: node scripts/test_leaf_reference.cjs <file>
const {chromium}=require('playwright');
const fs=require('node:fs');const path=require('node:path');const assert=require('node:assert/strict');
async function main(){
 const cases=JSON.parse(fs.readFileSync(process.argv[2],'utf8'));
 const source=fs.readFileSync(path.join(__dirname,'../crates/adventuresim-procedural-textures/src/leaf/fixtures/reference.wgsl'),'utf8');
 const fields=[...source.slice(0,source.indexOf('@group')).matchAll(/(\w+): vec4f/g)].map(m=>m[1]);
 let code=source.slice(0,source.indexOf('struct VertexOutput'))+source.slice(source.indexOf('fn axis_x'),source.indexOf('@fragment'));
 let body=source.slice(source.indexOf('    let field=blade_field(p);'));body=body.slice(0,body.lastIndexOf('}'));
 body=body.replaceAll('return leaf.vein_color','return 2u').replaceAll('return leaf.blade_color','return 1u').replaceAll('return vec4f(0.0)','return 0u');
 code+='\nfn classify(p:vec2f)->u32 {\n'+body+'}\n';
 code+='@group(0) @binding(1) var<storage,read_write> results:array<u32>;\n@compute @workgroup_size(64) fn main(@builtin(global_invocation_id) id:vec3u) {let p=(vec2f(f32(id.x%32u)+0.5,f32(id.x/32u)+0.5)/32.0-vec2f(0.5))*2.0;results[id.x]=classify(p);}\n';
 const browser=await chromium.launch({headless:true,channel:process.platform==='win32'?'msedge':undefined,args:['--enable-unsafe-webgpu']});
 try{
  const page=await browser.newPage();await page.goto('http://127.0.0.1:8783');
  const result=await page.evaluate(async({code,cases,fields})=>{
   const adapter=await navigator.gpu.requestAdapter();if(!adapter)throw Error('No WebGPU adapter');const device=await adapter.requestDevice();
   const module=device.createShaderModule({code});const info=await module.getCompilationInfo();const errors=info.messages.filter(m=>m.type==='error');if(errors.length)throw Error(JSON.stringify(errors));
   const pipeline=await device.createComputePipelineAsync({layout:'auto',compute:{module,entryPoint:'main'}});
   const uniform=device.createBuffer({size:320,usage:GPUBufferUsage.UNIFORM|GPUBufferUsage.COPY_DST});
   const output=device.createBuffer({size:4096,usage:GPUBufferUsage.STORAGE|GPUBufferUsage.COPY_SRC});const read=device.createBuffer({size:4096,usage:GPUBufferUsage.MAP_READ|GPUBufferUsage.COPY_DST});
   const group=device.createBindGroup({layout:pipeline.getBindGroupLayout(0),entries:[{binding:0,resource:{buffer:uniform}},{binding:1,resource:{buffer:output}}]});
   const results=[];
   for(const c of cases){
    const values=fields.flatMap(f=>c.uniform[f]||[0,0,0,0]);device.queue.writeBuffer(uniform,0,new Float32Array(values));
    const encoder=device.createCommandEncoder();const pass=encoder.beginComputePass();pass.setPipeline(pipeline);pass.setBindGroup(0,group);pass.dispatchWorkgroups(16);pass.end();encoder.copyBufferToBuffer(output,0,read,0,4096);device.queue.submit([encoder.finish()]);
    await read.mapAsync(GPUMapMode.READ);const actual=Array.from(new Uint32Array(read.getMappedRange()));read.unmap();
    results.push({name:c.name,differences:actual.reduce((n,v,i)=>n+Number(v!==c.classes[i]),0),samples:actual.length});
   }device.destroy();return results;
  },{code,cases,fields});
  fs.writeFileSync(process.argv[2]+'.gpu.json',JSON.stringify(result,null,2));
  const mismatches=result.filter(c=>c.differences>1);assert.equal(mismatches.length,0,JSON.stringify(mismatches));
  console.log(`${result.length} reference cases; ${result.reduce((n,c)=>n+c.samples,0)} samples; ${result.reduce((n,c)=>n+c.differences,0)} CPU/GPU differences`);
 }finally{await browser.close();}
}
main().catch(e=>{console.error(e);process.exitCode=1;});

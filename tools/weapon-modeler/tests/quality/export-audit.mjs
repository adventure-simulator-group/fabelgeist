import { automaticGripPoint, buildSkinnedWeaponGlb, encodeGlb, parseGlb } from "../../src/glb-export.js";
import { triangle, distance, dot } from "./geometry.mjs";

// An actual GLB encode/decode through a translated attachment catches precision
// loss in the export path as well as the preview's float32 buffers.
export function auditExport(mesh){
  const rig=encodeGlb({asset:{version:"2.0"},scene:0,scenes:[{nodes:[0]}],nodes:[{name:"r_weapon",translation:[1,2,3]}],skins:[{joints:[0]}],meshes:[],materials:[],accessors:[],bufferViews:[],buffers:[{byteLength:0}]},new Uint8Array());
  const output=buildSkinnedWeaponGlb(rig,mesh,{gripPoint:automaticGripPoint(mesh.resolvedDefinition)}),parsed=parseGlb(output),findings=[];
  const read=index=>{
    const a=parsed.document.accessors[index],v=parsed.document.bufferViews[a.bufferView],width={SCALAR:1,VEC3:3}[a.type],start=parsed.binary.byteOffset+(v.byteOffset??0)+(a.byteOffset??0);
    const Type={5126:Float32Array,5125:Uint32Array,5123:Uint16Array}[a.componentType];
    if(!Type||!width||v.byteStride)throw new Error("unexpected exported accessor layout");
    return new Type(parsed.binary.buffer,start,a.count*width);
  };
  const record=(code,example)=>{let f=findings.find(f=>f.code===code);if(!f){f={code,severity:"error",count:0,examples:[]};findings.push(f);}f.count++;if(f.examples.length<8)f.examples.push(example);};
  let triangles=0;
  for(const [meshIndex,m]of parsed.document.meshes.entries())for(const [primitiveIndex,p]of m.primitives.entries()){
    const positions=read(p.attributes.POSITION),normals=read(p.attributes.NORMAL),indices=read(p.indices);
    if(positions.length!==normals.length||indices.length%3||!positions.every(Number.isFinite)||!normals.every(Number.isFinite)){record("export-invalid-buffer",{meshIndex,primitiveIndex});continue;}
    for(let i=0;i<indices.length;i+=3){
      const ids=Array.from(indices.slice(i,i+3)),ts=ids.map(id=>Array.from(positions.slice(id*3,id*3+3)));
      if(ts.some(t=>t.length!==3)){record("export-invalid-index",{meshIndex,primitiveIndex,triangle:i/3});continue;}
      const t=triangle(ts,i/3),longest=Math.max(...ts.map((v,j)=>distance(v,ts[(j+1)%3])));triangles++;
      if(!t.area2||t.area2<=1e-9*longest)record("export-degenerate-triangle",{meshIndex,primitiveIndex,triangle:i/3,points:ts});
      else if(ids.some(id=>dot(t.normal,Array.from(normals.slice(id*3,id*3+3)))<=0))record("export-normal-winding",{meshIndex,primitiveIndex,triangle:i/3});
    }
  }
  return {findings,triangles,attachmentTranslation:[1,2,3]};
}

import assert from "node:assert/strict";
import test from "node:test";
import { auditPart, auditRelation, lodDifferences } from "./quality/audit.mjs";
import { triangle, triangleContact, unexpectedContact, triangleDistance, closestSurfaces } from "./quality/geometry.mjs";
import { tetra, combine, soup } from "./quality/fixtures.mjs";
import { box, lathe, closedManifoldErrors } from "../src/mesh.js";
import { auditExport } from "./quality/export-audit.mjs";
import { minimizeChanges } from "./quality/minimize.mjs";

const codes=a=>a.findings.map(f=>f.code);
const errors=a=>a.findings.filter(f=>f.severity==="error");
const cube=(size=1,offset=[0,0,0])=>{
  const vertices=[[-1,-1,-1],[1,-1,-1],[1,1,-1],[-1,1,-1],[-1,-1,1],[1,-1,1],[1,1,1],[-1,1,1]].map(p=>p.map((v,i)=>v*size/2+offset[i]));
  const faces=[[0,2,1],[0,3,2],[4,5,6],[4,6,7],[0,1,5],[0,5,4],[3,7,6],[3,6,2],[0,4,7],[0,7,3],[1,2,6],[1,6,5]];
  const positions=faces.flatMap(f=>f.flatMap(i=>vertices[i])), normals=faces.flatMap(f=>{const t=triangle(f.map(i=>vertices[i]),0);return [...t.normal,...t.normal,...t.normal];});
  return {positions,normals,colors:positions.map(()=>0.5),indices:positions.map((_,i)=>i).slice(0,positions.length/3),material:{color:[0.5,0.5,0.5],metallic:1,roughness:0.4}};
};

test("quality audit accepts closed solids with deliberately split shading vertices",()=>{
  for(const mesh of [tetra(),cube()])assert.deepEqual(errors(auditPart(mesh)),[]);
});
test("quality audit detects vertex pinches missed by the production edge validator",()=>{
  const mesh=combine(tetra(),tetra([0,0,0],-1,true));
  assert.deepEqual(closedManifoldErrors(mesh),[]);
  assert.ok(codes(auditPart(mesh)).includes("vertex-manifold"));
});
test("quality audit rejects invalid buffers, degenerate faces and duplicate faces",()=>{
  assert.ok(codes(auditPart({positions:[NaN,0,0],indices:[0,0,0]})).includes("invalid-buffer"));
  assert.ok(codes(auditPart({positions:[0,0,0],indices:[0,1,2]})).includes("invalid-buffer"));
  const degenerate=soup([[[0,0,0],[1,0,0],[2,0,0]]]);
  assert.ok(codes(auditPart(degenerate)).includes("degenerate-triangle"));
  for(const inward of [false,true]){
    const m=tetra(),face=inward?[0,1,2]:m.indices.slice(0,3);m.indices.push(...face);
    assert.ok(codes(auditPart(m)).includes("duplicate-triangle"));
  }
});
test("quality audit identifies missing faces and reversed winding",()=>{
  const open=tetra();open.indices.splice(0,3);
  assert.ok(codes(auditPart(open)).includes("edge-manifold"));
  const reversed=tetra();[reversed.indices[1],reversed.indices[2]]=[reversed.indices[2],reversed.indices[1]];
  assert.ok(codes(auditPart(reversed)).includes("edge-winding"));
});
test("quality audit catches inward shells despite positive aggregate volume",()=>{
  const mesh=combine(tetra(),tetra([3,0,0],0.2,true));
  assert.ok(codes(auditPart(mesh)).includes("shell-orientation"));
});
test("quality audit permits inward cavity boundaries and outward islands in cavities",()=>{
  const mesh=combine(tetra([0,0,0],4),tetra([0.2,0.2,0.2],1,true),tetra([0.3,0.3,0.3],0.1));
  assert.deepEqual(errors(auditPart(mesh)),[]);
  const wrong=combine(tetra([0,0,0],4),tetra([0.2,0.2,0.2],1));
  assert.ok(codes(auditPart(wrong)).includes("shell-orientation"));
});
test("triangle predicates distinguish legal adjacency from overlaps and crossings",()=>{
  const a=triangle([[0,0,0],[2,0,0],[0,2,0]],0);
  const legal=triangle([[2,0,0],[0,0,0],[1,-1,0]],1);
  assert.equal(unexpectedContact(a,legal,triangleContact(a,legal,1e-9),1e-9),false);
  const folded=triangle([[0,0,0],[2,0,0],[1,0.5,0]],1);
  assert.equal(unexpectedContact(a,folded,triangleContact(a,folded,1e-9),1e-9),true);
  const crossing=triangle([[0.5,0.5,-1],[0.5,0.5,1],[1.5,0.5,0]],1);
  assert.ok(triangleContact(a,crossing,1e-9));
  const commonVertex=triangle([[0,0,0],[1,1,-1],[1,1,1]],1);
  assert.equal(unexpectedContact(a,commonVertex,triangleContact(a,commonVertex,1e-9),1e-9),true);
  const disjoint=triangle([[3,0,0],[4,0,0],[3,1,0]],1);
  assert.equal(triangleContact(a,disjoint,1e-9),null);
});
test("triangle predicates handle coplanar containment, opposite normals and point contact",()=>{
  const a=triangle([[0,0,0],[2,0,0],[0,2,0]],0);
  const b=triangle([[0.2,0.2,0],[0.2,0.5,0],[0.5,0.2,0]],1);
  assert.ok(triangleContact(a,b,1e-9).area>0);
  const p=triangle([[2,0,0],[3,-1,0],[3,1,0]],1),hit=triangleContact(a,p,1e-9);
  assert.ok(hit);assert.equal(unexpectedContact(a,p,hit,1e-9),false);
});
test("intersection evidence is invariant to triangle order and rigid transforms",()=>{
  const points=[[[0,0,0],[2,0,0],[0,2,0]],[[0.5,0.5,-1],[0.5,0.5,1],[1.5,0.5,0]]];
  for(const scale of [0.001,1,100])for(const reverse of [false,true]){
    const ts=points.map((ps,id)=>triangle((reverse?[...ps].reverse():ps).map(([x,y,z])=>[z*scale+7,x*scale-4,y*scale+3]),id));
    assert.ok(triangleContact(ts[0],ts[1],1e-10*scale));assert.ok(triangleContact(ts[1],ts[0],1e-10*scale));
  }
});
test("nearly coplanar faces sharing a vertex do not invent an intersection segment",()=>{
  const pairs=[
    [[[0.07441786739216404,0.9422754222615832,0.0037027483410330708],[0.031071014651784953,1.046132811629134,0.006768586330233906],[0.03033804901194409,1.045815360531602,0.0034713265697185044]],[[0.07520418159507537,0.9426034892216619,0.007219825418916166],[0.03225171690118113,1.0466441796995534,0.00972644090899624],[0.031071014651784953,1.046132811629134,0.006768586330233906]]],
    [[[-0.030777994688290455,-0.6478496929643695,0.009306301555525467],[-0.10406780654196719,-0.7100841755753324,0.010046213805872305],[-0.10379976698241415,-0.710396702288307,0.008328596722843803]],[[-0.10414,-0.71,0],[-0.10406780654196719,-0.7100841755753324,0.010046213805872305],[-0.10435238475273007,-0.709752365252475,0.009462364375976397]]],
  ];
  for(const points of pairs)for(const reverse of [false,true]){
    const [a,b]=points.map((p,i)=>triangle(reverse?[...p].reverse():p,i)),hit=triangleContact(a,b,1e-9);
    assert.ok(hit);assert.equal(unexpectedContact(a,b,hit,1e-9),false);
  }
});
test("closed tapered cylinders keep valid periodic seams after float32 conversion",()=>{
  for(const radius of [0.0045,0.02,1])for(const length of [0.76,1.82])for(const segments of [8,9,16]){
    const mesh=lathe([[0,radius],[length,radius*0.92]],segments,"steel",[0,0,0],"fixture",1,true);
    for(const round of [false,true]){
      const candidate=round?{...mesh,positions:mesh.positions.map(Math.fround),normals:mesh.normals.map(Math.fround)}:mesh;
      assert.deepEqual(errors(auditPart(candidate)),[],`${radius}/${length}/${segments}/${round}`);
    }
  }
});
test("quality audit detects a folded single shell and distinguishes compound-part contact",()=>{
  const mesh=cube();
  // Push one corner through the opposite side, preserving face connectivity.
  for(let i=0;i<mesh.positions.length;i+=3)if(mesh.positions[i]>0&&mesh.positions[i+1]>0&&mesh.positions[i+2]>0)mesh.positions[i]=-0.8;
  assert.ok(codes(auditPart(mesh)).includes("self-intersection"));
  assert.ok(codes(auditPart(combine(tetra(),tetra([0.2,0.2,0.2])))).includes("inter-shell-contact"));
});
test("aspect and sampled thickness budgets report geometry without imposing practicality",()=>{
  const slender=box([1,0.00001,0.1],"steel",[0,0,0]);
  const a=auditPart(slender,{thickness:true});
  assert.ok(codes(a).includes("high-aspect-triangle"));assert.ok(codes(a).includes("thin-wall-sample"));
  assert.deepEqual(errors(a),[]);
});
test("float32 audit catches a triangle that collapses only after coordinate conversion",()=>{
  const source=soup([[[100,0,0],[100.000001,0,0],[100,1,0]]]);
  assert.equal(codes(auditPart(source)).includes("degenerate-triangle"),false);
  assert.ok(codes(auditPart({...source,positions:source.positions.map(Math.fround)})).includes("degenerate-triangle"));
});
test("joint contracts detect forbidden crossings, full containment, clearance and missing contact",()=>{
  const a=auditPart(cube()),overlap=auditPart(cube(1,[0.8,0,0])),far=auditPart(cube(1,[2,0,0])),buried=auditPart(cube(0.1));
  assert.ok(codes(auditRelation(a,overlap)).includes("forbidden-contact"));
  assert.ok(codes(auditRelation(a,buried)).includes("forbidden-containment"));
  assert.ok(codes(auditRelation(a,far,{mode:"contact"})).includes("missing-contact"));
  assert.deepEqual(errors(auditRelation(a,overlap,{mode:"joint",joint:p=>p[0]>=0.29&&p[0]<=0.51})),[]);
  assert.ok(codes(auditRelation(a,overlap,{mode:"joint",joint:p=>p[0]>0.9})).includes("outside-joint"));
  const near=auditPart(cube(1,[1.00001,0,0]));
  assert.ok(codes(auditRelation(a,near,{clearance:0.0001})).includes("insufficient-clearance"));
  const firstX=buried.triangles[0].points[0][0];
  assert.ok(codes(auditRelation(a,buried,{mode:"joint",joint:p=>Math.abs(p[0]-firstX)<0.01})).includes("outside-joint"));
});
test("triangle distance includes edge-interior proximity and tangential contact",()=>{
  const a=triangle([[-1,0,0],[1,0,0],[0,-1,0]],0),b=triangle([[0,-0.5,1],[0,0.5,1],[0,0,2]],1);
  assert.ok(Math.abs(triangleDistance(a,b)-1)<1e-9);
  const left=auditPart(cube()),right=auditPart(cube(1,[1,0,0]));
  assert.deepEqual(errors(auditRelation(left,right,{mode:"contact"})),[]);
});
test("LOD topology comparison detects a disappeared shell",()=>{
  assert.equal(lodDifferences([auditPart(combine(tetra(),tetra([3,0,0])))],[auditPart(tetra())])[0].code,"lod-topology");
});
test("actual GLB round trip accepts a solid and catches attachment-space float32 collapse",()=>{
  assert.deepEqual(auditExport(cube()).findings,[]);
  const mesh=soup([[[100,0,0],[100.000001,0,0],[100,1,0]]]);
  mesh.normals=[0,0,1,0,0,1,0,0,1];mesh.colors=new Array(9).fill(0.5);
  assert.ok(codes(auditExport(mesh)).includes("export-degenerate-triangle"));
});
test("failure minimization retains interacting controls and reports budget exhaustion",()=>{
  const fails=xs=>[2,5,7].every(i=>xs.includes(i)),changes=Array.from({length:10},(_,i)=>i);
  const result=minimizeChanges(changes,fails);assert.deepEqual(result.changes,[2,5,7]);assert.equal(result.complete,true);
  assert.equal(minimizeChanges(changes,fails,1).complete,false);
  assert.throws(()=>minimizeChanges([0],fails),/does not reproduce/);
});
test("nearest-surface evidence measures a gap beyond the contact tolerance",()=>{
  const a=auditPart(cube()),b=auditPart(cube(1,[1.005,0,0]));
  assert.ok(Math.abs(closestSurfaces(a.tree,b.tree).metres-0.005)<1e-12);
});

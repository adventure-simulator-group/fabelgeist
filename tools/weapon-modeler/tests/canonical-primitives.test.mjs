import assert from "node:assert/strict";
import test from "node:test";
import { auditPart } from "./quality/audit.mjs";
import { componentFixture } from "./canonical-fixtures.mjs";

function audited(part, context) {
  assert.ok(part, context);
  for (const float32 of [false, true]) {
    const mesh = float32 ? { ...part, positions: part.positions.map(Math.fround), normals: part.normals.map(Math.fround) } : part;
    assert.deepEqual(auditPart(mesh).findings.filter(f => f.severity === "error"), [], `${context}/${float32 ? "float32" : "double"}`);
  }
}

test("canonical primitive surfaces pass independent intersection, pinch, winding and normal audits", () => {
  for (const lod of ["low", "medium", "high"]) {
    for (const part of componentFixture({kind:"bill",length:.38,width:.09,hook:.08,thickness:.02,root:.03,rootLength:.06,bellyPosition:.48,hookDepth:.19,hookCurvature:.22,pointLength:.24},lod).parts) audited(part,`concave prism/${lod}`);
    for (const profile of [[[0,0],[.04,.02],[.08,0]],[[0,.02],[.04,0]],[[0,0],[.04,.02]]]) {
      const mesh=componentFixture({kind:"pommel",construction:"lathed",profile,segments:12},lod);
      for (const part of mesh.parts) audited(part,`lathe pole/${lod}`);
    }
    for (const radius of [.0045,.02,1]) for (const length of [.76,1.82]) for (const segments of [8,9,16]) {
      const mesh=componentFixture({kind:"shaft",radius,length,segments,bottomScale:1,topScale:.92},lod);
      for (const part of mesh.parts) audited(part,`tapered cylinder/${radius}/${length}/${segments}/${lod}`);
    }
  }
});

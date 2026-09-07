import assert from "node:assert/strict";
import test from "node:test";
import { buildWeapon, crossbowStockLayout, lathe, prism, validateWeapon } from "../src/mesh.js";
import { PRESETS, copyPreset, setControlValue } from "../src/presets.js";
import { subdividePath } from "../src/construction.js";
import { auditPart, auditRelation } from "./quality/audit.mjs";

const preset = id => copyPreset(PRESETS.find(p => p.id === id));
const errors = mesh => auditPart(mesh).findings.filter(f => f.severity === "error");
function cleanParts(definition, lod = "low") {
  const result = validateWeapon(definition, undefined, { lod });
  assert.equal(result.valid, true, result.errors.join("; "));
  for (const part of result.mesh.parts) assert.deepEqual(errors(part), [], part.label);
  return result.mesh;
}

test("shared cap and wall topology closes a refined concave prism", () => {
  const mesh = prism([[0,0],[0.12,0],[0.12,0.04],[0.04,0.04],[0.04,0.12],[0,0.12]], 0.006, "steel", [0,0,0]);
  assert.deepEqual(errors(mesh), []);
});

test("zero-radius lathe endpoints construct poles without collapsed triangles", () => {
  assert.deepEqual(errors(lathe([[0,0],[0.04,0.02],[0.08,0]], 12)), []);
});

test("path refinement preserves authored stations and parameter values", () => {
  const result = subdividePath([[0,0],[0.01,0],[0.11,0]], 0.025, [1,0.5,0.2]);
  const corner = result.points.findIndex(p => p[0] === 0.01);
  assert.equal(result.progress[corner], 0.5);
  assert.equal(result.scales[corner], 0.5);
  assert.deepEqual(result.points.at(-1), [0.11,0]);
  assert.equal(result.progress.at(-1), 1);
});

test("fullered blade caps and two-hole figure-eight guards remain manifold at every LOD", () => {
  for (const id of ["landsknecht-longsword", "katzbalger", "estoc"]) for (const lod of ["low","medium","high"]) cleanParts(preset(id).definition, lod);
});

test("crossbow stock stations stay ordered when a long tiller has a rearward nut", () => {
  const specimen = preset("central-composite-arbalest"), component = specimen.definition.components.find(c => c.kind === "crossbow");
  component.length = 0.92; component.nutPosition = 0.30;
  const layout = crossbowStockLayout(component);
  for (const stations of [layout.rearStations, layout.foreStations]) for (let i = 1; i < stations.length; i++) assert.ok(stations[i].y > stations[i-1].y);
  cleanParts(specimen.definition);
});

test("deep tapered heater shields have closed bodies, valid rims and seated handle feet", () => {
  const specimen = preset("heater-shield");
  for (const [label,value] of [["Bottom depth",0.30],["Side taper",0.65]]) setControlValue(specimen.definition, specimen.controls.find(c => c.label === label), value);
  for (const lod of ["low","medium","high"]) {
    const mesh = cleanParts(specimen.definition, lod), body = auditPart(mesh.parts.find(p => p.shieldRole === "body"));
    const feet = mesh.parts.filter(p => p.label === "shield handle foot");
    assert.equal(feet.length, 2);
    for (const foot of feet) assert.equal(auditRelation(body, auditPart(foot), {mode:"contact"}).contact, true);
  }
});

test("bottom attachment insertion seats the butt cap into its shaft", () => {
  const mesh = buildWeapon(preset("halberd-1540").definition, {lod:"low"});
  const shaft = mesh.parts.find(p => p.label === "shaft"), cap = mesh.parts.find(p => p.label === "butt cap");
  assert.equal(auditRelation(auditPart(shaft), auditPart(cap), {mode:"contact"}).contact, true);
});

test("pointed shield rims and tight binding loops stay valid at high detail", () => {
  for (const id of ["kite-shield", "german-cranequin-crossbow-1544", "light-target-crossbow-comparative", "small-arms-ball-pouch"]) cleanParts(preset(id).definition, "high");
});

test("bow loop slack changes geometry while retaining a seated nock", () => {
  const specimen = preset("german-self-bow-1544"), component = specimen.definition.components.find(c => c.kind === "archeryBow");
  const before = buildWeapon(specimen.definition, {lod:"low"});
  component.loopRadius *= 1.2;
  const after = cleanParts(specimen.definition);
  const loop = after.parts.find(p => p.label === "upper bowstring end loop"), nock = after.parts.find(p => p.label === "upper horn nock overlay");
  assert.notDeepEqual(loop.positions, before.parts.find(p => p.label === loop.label).positions);
  assert.equal(auditRelation(auditPart(loop), auditPart(nock), {mode:"contact"}).contact, true);
});

test("taper and fitting spacing extremes seat every shield attachment", () => {
  const specimen = preset("heater-shield");
  for (const [label,value] of [["Side taper",0.65],["Fitting spacing",0.24]]) setControlValue(specimen.definition, specimen.controls.find(c => c.label === label), value);
  for (const lod of ["low","medium","high"]) {
    const mesh = cleanParts(specimen.definition, lod), body = auditPart(mesh.parts.find(p => p.shieldRole === "body"));
    for (const part of mesh.parts.filter(p => ["shield handle foot","strap attachment"].includes(p.label))) assert.equal(auditRelation(body,auditPart(part),{mode:"contact"}).contact,true,part.label);
  }
});

import { generateModel, validateWeapon } from "../src/kernel.js";
import { triangleVertices } from "../src/topology.js";
import assert from "node:assert/strict";
import test from "node:test";
import { measureMassProperties, signedVolume } from "./quality/mesh-measurements.mjs";
import { HEAD_KINDS, PRESETS, copyPreset, getPath, setControlValue, setPath } from "../src/presets.js";
import { fitDistance, projectedFit } from "../src/renderer.js";


test("every preset produces finite nonempty geometry", () => {
  for (const preset of PRESETS) {
    const mesh = generateModel(preset.definition);
    assert.ok(mesh.positions.length > 0, preset.id);
    assert.ok(mesh.positions.every(Number.isFinite), preset.id);
    assert.ok(mesh.normals.every(Number.isFinite), preset.id);
    assert.ok(mesh.stats.triangles > 100, preset.id);
    const isShield = preset.definition.components.some((component) => ["roundShield", "shapedShield"].includes(component.kind)),
      isCarrier = preset.definition.components.some((component) => ["arrowQuiver", "boltQuiver"].includes(component.kind)),
      isBall = preset.definition.components.some((component) => component.kind === "leadBall"),
      isPouch = preset.definition.components.some((component) => component.kind === "ballPouch"),
      isDagger = preset.id === "rondel-dagger";
    assert.ok(mesh.stats.dimensions[1] > (isBall ? 0.005 : isPouch ? 0.1 : isShield ? 0.3 : isCarrier ? 0.4 : isDagger ? 0.40 : 0.45), preset.id);
    assert.ok(mesh.stats.dimensions.every((value) => Number.isFinite(value) && value > 0), preset.id);
    assert.ok(mesh.stats.volume > 0, preset.id);
    for (const control of preset.controls) {
      const value = getPath(preset.definition, control.path);
      assert.ok(Number.isFinite(value), `${preset.id}: ${control.path}`);
      assert.ok(value >= control.min && value <= control.max, `${preset.id}: ${control.path}`);
    }
  }
});

test("mesh mass distribution derives realistic pommel mass and handling", () => {
  const preset = copyPreset(PRESETS.find((candidate) => candidate.id === "landsknecht-longsword"));
  const mesh = generateModel(preset.definition);
  const baseline = measureMassProperties(mesh, mesh.physical.controlPoint);
  const pommelMass = baseline.components.find((component) => component.id === "pommel").massKg;
  assert.ok(baseline.massKg > 1 && baseline.massKg < 3, baseline.massKg);
  assert.ok(pommelMass > 0.1 && pommelMass < 0.5, pommelMass);
  assert.ok(pommelMass < baseline.massKg * 0.25, pommelMass / baseline.massKg);
  assert.ok(baseline.centerOfMassFromGripM > 0);
  assert.ok(baseline.momentOfInertiaKgM2 > 0);
  assert.ok(baseline.balance > 0 && baseline.balance < 1);

  const largerPommel = preset.definition.components.find((component) => component.id === "pommel");
  largerPommel.profile = largerPommel.profile.map(([height, radius]) => [height, radius * 1.08]);
  const changedMesh = generateModel(preset.definition);
  const changed = measureMassProperties(changedMesh, changedMesh.physical.controlPoint);
  assert.ok(changed.centerOfMassFromGripM < baseline.centerOfMassFromGripM);
  assert.ok(changed.balance < baseline.balance);
});

test("sword presets expose a hand-center clearance below the guard", () => {
  const swords = PRESETS.filter((preset) => preset.definition.gripClearance !== undefined);
  assert.ok(swords.length >= 8);
  for (const source of swords) {
    const preset = copyPreset(source);
    const control = preset.controls.find((candidate) => candidate.path === "gripClearance");
    assert.ok(control, source.id);
    assert.equal(preset.definition.gripClearance, 0.05, source.id);
    const mesh = generateModel(preset.definition);
    const point = mesh.physical.controlPoint;
    const top = mesh.resolvedDefinition._frames["grip.top"];
    assert.ok(Math.abs(Math.hypot(...point.map((value, axis) => value - top[axis])) - 0.05) < 1e-9, source.id);
  }

  const invalid = copyPreset(swords[0]);
  invalid.definition.gripClearance = 0.5;
  assert.equal(validateWeapon(invalid.definition, []).valid, false);
});

test("preset parameters can be independently copied and changed", () => {
  const source = PRESETS.find((preset) => preset.definition.shaft);
  const copy = copyPreset(source);
  const original = getPath(copy.definition, "shaft.length");
  setPath(copy.definition, "shaft.length", original + 0.2);
  assert.equal(getPath(copy.definition, "shaft.length"), original + 0.2);
  assert.equal(getPath(source.definition, "shaft.length"), original);
});

test("hafted weapons share a shaft-top mount and remain attached when resized", () => {
  const hafted = PRESETS.filter((preset) => preset.definition.shaft);
  assert.ok(hafted.length >= 10);
  for (const preset of hafted) {
    assert.ok(preset.definition.components.some((component) => (component.kind === "socket" && component.mount === "shaft-top") || (component.kind === "sleeve" && component.mount === "shaft-top-sleeve")), preset.id);
    const copy = copyPreset(preset);
    const before = generateModel(copy.definition).stats.bounds.max[1];
    copy.definition.shaft.length += 0.2;
    const after = generateModel(copy.definition).stats.bounds.max[1];
    assert.ok(Math.abs((after - before) - 0.2) < 1e-6, preset.id);
  }
});

test("the library covers reusable head families and supports a modular head swap", () => {
  const used = new Set(PRESETS.flatMap((preset) => preset.definition.components.map((component) => component.kind)));
  for (const kind of HEAD_KINDS) assert.ok(used.has(kind), kind);
  const halberd = copyPreset(PRESETS.find((preset) => preset.id === "halberd-1540"));
  const lucerne = PRESETS.find((preset) => preset.id === "lucerne-hammer");
  const slotId = halberd.definition.components[1].id ?? halberd.definition.components[1].label ?? "component-1";
  halberd.definition.components[1] = { ...deepClone(lucerne.definition.components[1]), id: slotId };
  const mesh = generateModel(halberd.definition);
  assert.ok(mesh.positions.every(Number.isFinite));
  assert.ok(mesh.stats.partCount > 5);
});

test("front camera fit contains every preset on portrait and landscape canvases", () => {
  const fov = 35 * Math.PI / 180;
  for (const preset of PRESETS) {
    const bounds = generateModel(preset.definition).stats.bounds;
    for (const aspect of [0.55, 1, 1.8]) {
      const distance = fitDistance(bounds, aspect, fov);
      const visibleHalfHeight = distance * Math.tan(fov / 2);
      const visibleHalfWidth = visibleHalfHeight * aspect;
      assert.ok(visibleHalfHeight > (bounds.max[1] - bounds.min[1]) / 2, `${preset.id}: vertical`);
      assert.ok(visibleHalfWidth > (bounds.max[0] - bounds.min[0]) / 2, `${preset.id}: horizontal`);
    }
  }
});

test("all curated presets fit the live 1280x720 viewer canvas with margin", () => {
  assert.ok(PRESETS.length >= 30);
  const canvasAspect = (1280 - 350) / (720 - 88), fov = 35 * Math.PI / 180;
  for (const preset of PRESETS) {
    const bounds = generateModel(preset.definition).stats.bounds, distance = fitDistance(bounds, canvasAspect, fov);
    const normalizedX = ((bounds.max[0] - bounds.min[0]) / 2) / (distance * Math.tan(fov / 2) * canvasAspect);
    const normalizedY = ((bounds.max[1] - bounds.min[1]) / 2) / (distance * Math.tan(fov / 2));
    assert.ok(normalizedX <= 0.8 && normalizedY <= 0.8, `${preset.id}: ${normalizedX}, ${normalizedY}`);
  }
});

test("shared renderer projection contains front and oblique vertices inside margin", () => {
  for (const preset of PRESETS) {
    const mesh = generateModel(preset.definition);
    for (const [pose, yaw, pitch] of [["front", 0, 0], ["oblique", 0.68, 0.18]]) {
      const fit = projectedFit(mesh.positions, mesh.stats.bounds, 1280 / 720, yaw, pitch);
      assert.equal(fit.contained, true, `${preset.id} ${pose}`); assert.ok(fit.maxProjected <= 0.8 + 1e-7, `${preset.id} ${pose} margin`);
    }
  }
});

test("camera framing handles a dense mesh beyond the JavaScript argument limit", () => {
  const positions = new Float64Array(600_000);
  for (let i = 0; i < positions.length; i += 3) {
    positions[i] = i % 2 ? -0.04 : 0.04;
    positions[i + 1] = i / positions.length;
    positions[i + 2] = i % 2 ? 0.04 : -0.04;
  }
  for (const aspect of [0.6, 1.8]) {
    const fit = projectedFit(positions, { min: [-0.04, 0, -0.04], max: [0.04, 1, 0.04] }, aspect, 0.68, 0.18);
    assert.equal(fit.contained, true);
    assert.ok(fit.distance > 0 && Number.isFinite(fit.distance));
  }
});

test("component-local rotation reorients an interchangeable mounted head", () => {
  const base = { shaft: { length: 1, radius: 0.02 }, components: [{ kind: "hammer", label: "poll", mount: "shaft-top", offset: [0, 0, 0], length: 0.2, face: 0.08, neck: 0.044, thickness: 0.05, direction: 1 }] };
  const horizontal = generateModel(base).stats.dimensions;
  base.components[0].rotation = [0, 90, 0];
  const rotated = generateModel(base).stats.dimensions;
  assert.ok(horizontal[0] > rotated[0] * 2);
  assert.ok(rotated[2] > horizontal[2] * 2);
});

test("swept knuckle bow leaves a real aperture", () => {
  const mesh = generateModel({components:[{kind:"knuckleBow", width:0.12,length:0.18,bar:0.015,thickness:0.012,side:1,attach:{to:"weapon.root"}}]});
  const point = [0.025, 0.09];
  let coveringTriangles = 0;
  for (const vertices of triangleVertices(mesh)) {
    const triangle = vertices.map((point) => point.slice(0, 2));
    if (contains2d(point, ...triangle)) coveringTriangles += 1;
  }
  assert.equal(coveringTriangles, 0);
});

test("compound controls remain coherent at both extremes", () => {
  for (const preset of PRESETS) for (const control of preset.controls.filter((candidate) => candidate.paths?.length > 1)) {
    const copy = copyPreset(preset);
    for (const value of [control.min, control.max]) {
      setControlValue(copy.definition, control, value);
      assert.ok(control.paths.every((path) => getPath(copy.definition, path) === value), `${preset.id}: ${control.label}`);
      assert.ok(generateModel(copy.definition).positions.every(Number.isFinite));
    }
  }
});

test("Kriegsspiess control cannot collapse into a short hand weapon", () => {
  const pike = PRESETS.find((preset) => preset.id === "kriegsspiess");
  assert.ok(pike.controls.find((control) => control.path === "shaft.length").min >= 3);
});

function contains2d([px, py], [ax, ay], [bx, by], [cx, cy]) {
  const cross2d = (x0, y0, x1, y1, x2, y2) => (x0 - x2) * (y1 - y2) - (x1 - x2) * (y0 - y2);
  const d1 = cross2d(px, py, ax, ay, bx, by), d2 = cross2d(px, py, bx, by, cx, cy), d3 = cross2d(px, py, cx, cy, ax, ay);
  return !((d1 < -1e-8 || d2 < -1e-8 || d3 < -1e-8) && (d1 > 1e-8 || d2 > 1e-8 || d3 > 1e-8));
}

test("figure-eight guard has two recognizable apertures", () => {
  const mesh = generateModel({components:[{kind:"figureEight",width:0.22,height:0.055,bar:0.009,attach:{to:"weapon.root"}}]});
  for (const point of [[-0.055, 0], [0.055, 0]]) {
    let covering = 0;
    for (const vertices of triangleVertices(mesh)) if (contains2d(point, ...vertices.map((vertex) => vertex.slice(0, 2)))) covering += 1;
    assert.equal(covering, 0);
  }
});

test("Reitschwert has compound rings, an open bow, and a sectioned straight blade", () => {
  const preset = PRESETS.find((candidate) => candidate.id === "reitschwert-1540");
  assert.ok(preset);
  const graph = preset.definition.components.find((component) => component.kind === "guardAssembly");
  assert.ok(graph);
  assert.ok(graph.members.some((member) => member.label === "side ring"));
  assert.ok(graph.members.some((member) => member.label === "knuckle bow"));
  assert.ok(graph.members.some((member) => member.label === "finger loop"));
  assert.equal(graph.anchorNode, "root");
  assert.equal(preset.definition.components.at(-1).kind, "sectionBlade");
});

test("curated compact heads stay inside reference-scale breadth envelopes", () => {
  const breadth = (id) => generateModel(PRESETS.find((preset) => preset.id === id).definition).stats.dimensions[0];
  assert.ok(breadth("halberd-1540") >= 0.24 && breadth("halberd-1540") <= 0.28);
  assert.ok(breadth("lucerne-hammer") <= 0.27);
  assert.ok(breadth("pollaxe") <= 0.26);
  assert.ok(breadth("reiter-war-hammer") >= 0.11 && breadth("reiter-war-hammer") <= 0.16);
  assert.ok(breadth("katzbalger") <= 0.17);
});



test("Messer Nagel projects 40-50 mm normal to the blade plane with a button", () => {
  const messer = PRESETS.find((preset) => preset.id === "grosse-messer"), stem = messer.definition.components.find((component) => component.label?.includes("Nagel stem")), button = messer.definition.components.find((component) => component.label?.includes("Nagel button"));
  assert.ok(stem && button);
  assert.ok(stem.points.at(-1)[1] >= 0.04 && stem.points.at(-1)[1] <= 0.05);
  assert.ok(button.profile[0][1] * 2 >= 0.01 && button.profile[0][1] * 2 <= 0.015);
});

test("hooked bill uses one continuous exposed-hook component", () => {
  const bill = PRESETS.find((preset) => preset.id === "hooked-bill"), head = bill.definition.components.find((component) => component.kind === "bill");
  assert.ok(head);
  assert.ok(head.hook >= 0.06 && head.hook <= 0.09);
  assert.equal(bill.definition.components.filter((component) => ["axe", "beak"].includes(component.kind)).length, 0);
});



test("every generated preset part has consistent positive winding", () => {
  for (const preset of PRESETS) {
    const mesh = generateModel(preset.definition);
    for (const part of mesh.parts) {
      assert.ok(signedVolume(part) >= -1e-9, `${preset.id}: ${part.label} (${signedVolume(part)})`);
      for (let index = 0; index < part.indices.length; index += 3) {
        const [a, b, c] = part.indices.slice(index, index + 3).map((vertex) => part.positions.slice(vertex * 3, vertex * 3 + 3));
        const stored = part.normals.slice(part.indices[index] * 3, part.indices[index] * 3 + 3);
        const ab = b.map((value, axis) => value - a[axis]), ac = c.map((value, axis) => value - a[axis]);
        const geometric = [ab[1] * ac[2] - ab[2] * ac[1], ab[2] * ac[0] - ab[0] * ac[2], ab[0] * ac[1] - ab[1] * ac[0]];
        const magnitude = Math.hypot(...geometric);
        if (magnitude > 1e-10) assert.ok(geometric.reduce((sum, value, axis) => sum + value / magnitude * stored[axis], 0) > 0, `${preset.id}: ${part.label} normal`);
      }
    }
    assert.ok(signedVolume(mesh) > 0, preset.id);
  }
});

function assertClosedTriangleMesh(mesh, context) {
  const precision = 1e7;
  const vertexKey = (values, offset) => values.slice(offset, offset + 3).map((value) => Math.round(value * precision)).join(",");
  const edges = new Map();
  for (let index = 0; index < mesh.indices.length; index += 3) {
    const vertices = mesh.indices.slice(index, index + 3).map((vertex) => vertexKey(mesh.positions, vertex * 3));
    for (const [from, to] of [[vertices[0], vertices[1]], [vertices[1], vertices[2]], [vertices[2], vertices[0]]]) {
      const key = from < to ? `${from}|${to}` : `${to}|${from}`;
      const incidence = edges.get(key) ?? { forward: 0, reverse: 0 };
      if (from < to) incidence.forward += 1; else incidence.reverse += 1;
      edges.set(key, incidence);
    }
  }
  const boundaries = [...edges.entries()].filter(([, incidence]) => incidence.forward !== 1 || incidence.reverse !== 1);
  assert.equal(boundaries.length, 0, `${context}: ${boundaries.length} non-manifold or boundary edges; first ${boundaries[0]?.[0] ?? "none"}`);
}

test("every generated component is a closed oriented two-manifold", () => {
  for (const preset of PRESETS) {
    const mesh = generateModel(preset.definition);
    for (const part of mesh.parts) assertClosedTriangleMesh(part, `${preset.id}: ${part.label}`);
  }
});

test("flanged-mace parameter space stays finite, closed, and oriented", () => {
  const source = PRESETS.find((preset) => preset.id === "flanged-mace");
  const controls = Object.fromEntries(source.controls.map((control) => [control.label, control]));
  const levels = (control) => [control.min, (control.min + control.max) / 2, control.max];
  for (const flangeCount of levels(controls["Flange count"])) for (const headLength of levels(controls["Head length"])) for (const concavity of levels(controls["Side concavity"])) {
    const preset = copyPreset(source);
    for (const control of preset.controls) setControlValue(preset.definition, control, (control.min + control.max) / 2);
    setControlValue(preset.definition, controls["Flange count"], Math.round(flangeCount));
    setControlValue(preset.definition, controls["Head length"], headLength);
    setControlValue(preset.definition, controls["Side concavity"], concavity);
    const mesh = generateModel(preset.definition);
    assert.ok(mesh.positions.every(Number.isFinite));
    assert.ok(signedVolume(mesh) > 0);
    for (const part of mesh.parts) assertClosedTriangleMesh(part, `mace ${flangeCount}/${headLength}/${concavity}: ${part.label}`);
  }
  for (const endpoint of ["min", "max"]) {
    const preset = copyPreset(source);
    for (const control of preset.controls) setControlValue(preset.definition, control, control[endpoint]);
    const mesh = generateModel(preset.definition);
    assert.ok(mesh.positions.every(Number.isFinite), endpoint);
    for (const part of mesh.parts) assertClosedTriangleMesh(part, `mace all-${endpoint}: ${part.label}`);
  }
});


test("Gothic mace endpoint keeps its polished crown, curve, and private dark grip", () => {
  const gothic = PRESETS.find((preset) => preset.id === "gothic-flanged-mace"), head = gothic.definition.components[4], grip = gothic.definition.components[0];
  assert.equal(head.crownLength, 0.015);
  assert.equal(head.concavity, 0.92);
  assert.equal(grip.material, "darkLeather");
  assert.equal(PRESETS.find((preset) => preset.id === "flanged-mace").definition.components[0].material, "leather");
});

function deepClone(value) { return JSON.parse(JSON.stringify(value)); }

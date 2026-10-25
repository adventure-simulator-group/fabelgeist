import { plateFixture } from "./canonical-fixtures.mjs";
import { generateModel, validateWeapon } from "../src/kernel.js";
import assert from "node:assert/strict";
import test from "node:test";
import { closedManifoldErrors, signedVolume } from "./quality/mesh-measurements.mjs";
import { PRESETS, copyPreset, controlVisible, setControlValue } from "../src/presets.js";
import { triangleVertices } from "../src/topology.js";
import { adversarialReviewCases } from "../src/review-cases.js";

const sword = () => copyPreset(PRESETS.find((preset) => preset.id === "landsknecht-longsword"));

test("pommel controls follow composite base construction and exclude fan-only dead controls", () => {
  const preset = sword(), pommel = preset.definition.components[0];
  pommel.construction = "composite";
  for (const [base, label] of [["faceted", "Facet count"], ["plate", "Wheel face convexity"], ["writhen", "Flute depth"], ["outline", "Fish-tail notch"]]) {
    pommel.baseConstruction = base;
    const control = preset.controls.find((item) => item.label === label);
    assert.equal(controlVisible(preset.definition, control), true, label);
    const before = generateModel(preset.definition).positions;
    setControlValue(preset.definition, control, control.min);
    assert.notDeepEqual(generateModel(preset.definition).positions, before, label);
  }
  pommel.outlineStyle = "fan";
  for (const label of ["Fish-tail notch", "Fish-tail lobe spread"]) assert.equal(controlVisible(preset.definition, preset.controls.find((control) => control.label === label)), false, label);
  const assemblyPreset = copyPreset(PRESETS.find((item) => item.id === "reitschwert-1540"));
  const shell = assemblyPreset.choiceControls.find((control) => control.label === "Compound-hilt shell study").options[1].value;
  const graph = assemblyPreset.definition.components.find((component) => component.kind === "guardAssembly");
  graph.plates = structuredClone(shell); delete graph.plates[0].cutout;
  assert.ok(validateWeapon(assemblyPreset.definition).errors.some((error) => error.includes("cutout")));
});

test("writhen default and deep-twist stress pommels retain a bounded semantic LOD budget", () => {
  const cases = [PRESETS.find((preset) => preset.id === "zweihander"), adversarialReviewCases().find((specimen) => specimen.id.includes("writhen-pommel-extreme"))];
  for (const specimen of cases) {
    const counts = ["low", "medium", "high"].map((lod) => {
      const mesh = generateModel(specimen.definition, { lod }), pommel = mesh.parts.find((part) => part.componentId === "pommel");
      assert.deepEqual(closedManifoldErrors(pommel), [], `${specimen.id}/${lod}`);
      assert.ok(pommel.normals.every(Number.isFinite));
      return pommel.indices.length / 3;
    });
    assert.ok(counts[0] <= counts[1] && counts[1] < counts[2]);
    assert.ok(counts[0] < 2000 && counts[2] < 6000, `${specimen.id}: ${counts}`);
  }
});

test("pommel construction families generate distinct closed silhouettes at low and high detail", () => {
  const source = sword(), choice = source.choiceControls.find((control) => control.label === "Pommel construction"), silhouettes = new Set();
  for (const option of choice.options) {
    const preset = sword(); setControlValue(preset.definition, choice, option.value);
    for (const lod of ["low", "high"]) {
      const result = validateWeapon(preset.definition, preset.controls, { lod });
      assert.equal(result.valid, true, `${option.value}/${lod}: ${result.errors.join(" | ")}`);
      const pommel = result.mesh.parts.find((part) => part.componentId === "pommel");
      assert.ok(signedVolume(pommel) > 0);
      if (lod === "high" && option.value !== "composite") silhouettes.add(pommel.positions.join(","));
    }
  }
  assert.equal(silhouettes.size, 5);
});

test("composite crowns, escutcheons, and authored indexed ornaments survive export topology", () => {
  const source = sword(), ornaments = source.choiceControls.find((control) => control.label === "Pommel ornament");
  source.definition.components[0].construction = "composite";
  const base = generateModel(source.definition).stats.triangles;
  for (const option of ornaments.options.slice(1)) {
    const preset = sword(); preset.definition.components[0].construction = "composite";
    setControlValue(preset.definition, ornaments, structuredClone(option.value));
    const result = validateWeapon(preset.definition, preset.controls);
    assert.equal(result.valid, true, `${option.label}: ${result.errors.join(" | ")}`);
    assert.ok(result.mesh.stats.triangles > base);
    const p = result.mesh.parts.find((part) => part.componentId === "pommel");
    assert.deepEqual(closedManifoldErrors(p), []);
  }
});


test("dished shell plates preserve a true opening, enclosed volume, and opposite hole-wall winding", () => {
  const nodes = { a: [-0.05, -0.05, 0], b: [0.05, -0.05, 0], c: [0.05, 0.05, 0], d: [-0.05, 0.05, 0], e: [-0.025, -0.025, 0], f: [0.025, -0.025, 0], g: [0.025, 0.025, 0], h: [-0.025, 0.025, 0] };
  const plate = { outline: ["a", "b", "c", "d"], cutout: ["e", "f", "g", "h"], thickness: 0.003, dishDepth: 0.01 };
  const mesh = plateFixture(plate, nodes);
  assert.deepEqual(closedManifoldErrors(mesh), []);
  const flat = plateFixture({ ...plate, dishDepth: 0 }, nodes);
  assert.ok(signedVolume(mesh) > 0);
  assert.ok(Math.abs(signedVolume(mesh) - signedVolume(flat)) < 1e-9);
  assert.ok(Math.max(...mesh.positions.filter((_, i) => i % 3 === 2)) > 0.009);
  for (const triangle of triangleVertices(mesh)) {
    const signs = triangle.map((a, i) => { const b = triangle[(i + 1) % 3]; return a[0] * b[1] - a[1] * b[0]; });
    assert.ok(!(signs.every((value) => value > 1e-10) || signs.every((value) => value < -1e-10)), "a face covers the aperture");
  }
});

test("compound hilts use a connected named-node graph and keep later shells opt-in", () => {
  const preset = copyPreset(PRESETS.find((item) => item.id === "reitschwert-1540"));
  const assembly = preset.definition.components.find((component) => component.kind === "guardAssembly");
  assert.deepEqual(assembly.plates, []);
  assert.ok(assembly.members.some((member) => member.label === "finger loop"));
  const baseline = generateModel(preset.definition);
  assembly.nodes.right[2] += 0.005;
  const changed = validateWeapon(preset.definition, preset.controls);
  assert.equal(changed.valid, true, changed.errors.join(" | "));
  assert.notDeepEqual(changed.mesh.positions, baseline.positions);
  assembly.members[0].path[0] = "missing-node";
  assert.ok(validateWeapon(preset.definition).errors.some(error=>error.includes("missing node")));
});

test("independent quillons alter each arm in all three dimensions", () => {
  const preset = sword(), guard = preset.definition.components.find((component) => component.kind === "guard");
  const baseline = generateModel(preset.definition);
  Object.assign(guard, { mirrorMode: "independent", leftLength: 0.09, rightLength: 0.21, leftSweep: -0.05, rightSweep: 0.06, leftSet: -0.025, rightSet: 0.03, section: "diamond", terminal: "scroll" });
  const result = validateWeapon(preset.definition, preset.controls);
  assert.equal(result.valid, true, result.errors.join(" | "));
  assert.notDeepEqual(result.mesh.positions, baseline.positions);
  assert.ok(result.mesh.stats.dimensions[2] > baseline.stats.dimensions[2]);
});

test("wheel flat faces remain manifold at zero convexity", () => {
  const preset = sword(); Object.assign(preset.definition.components[0], { construction: "plate", faceConvexity: 0 });
  const result = validateWeapon(preset.definition, preset.controls);
  assert.equal(result.valid, true, result.errors.join(" | "));
});

test("compound bow endpoints follow the grip frame at both slider extremes", () => {
  const source = PRESETS.find((preset) => preset.id === "reitschwert-1540"), control = source.controls.find((candidate) => candidate.label === "Grip length");
  for (const length of [control.min, control.max]) {
    const preset = copyPreset(source); setControlValue(preset.definition, control, length);
    const result = validateWeapon(preset.definition, preset.controls);
    assert.equal(result.valid, true, result.errors.join(" | "));
    const assembly = result.resolved.components.find((component) => component.kind === "guardAssembly"), frame = result.resolved._frames["grip.base"];
    assert.ok(Math.abs(assembly.nodes.bowLower[1] + assembly.offset[1] - frame[1] - 0.002) < 1e-9);
  }
});

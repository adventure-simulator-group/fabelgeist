import { validateWeapon } from "../src/kernel.js";
import assert from "node:assert/strict";
import test from "node:test";
import { closedManifoldErrors, signedVolume } from "./quality/mesh-measurements.mjs";
import { PRESETS, copyPreset, setControlValue } from "../src/presets.js";
import { triangleVertices } from "../src/topology.js";

const preset = (id) => PRESETS.find((candidate) => candidate.id === id);

test("bows, arrows, and quivers are independently selectable outputs", () => {
  assert.deepEqual(PRESETS.filter((candidate) => candidate.definition.components.some((component) => component.kind === "archeryBow")).map((candidate) => candidate.id), ["german-self-bow-1544", "composite-recurve-bow-1544"]);
  assert.deepEqual(preset("german-self-bow-1544").definition.components.map((part) => part.kind), ["archeryBow"]);
  assert.deepEqual(preset("flight-arrow-1544").definition.components.map((part) => part.kind), ["arrow"]);
  assert.deepEqual(preset("arrow-quiver-1544").definition.components.map((part) => part.kind), ["arrowQuiver"]);
});

test("one served string retains independent spans and closed tip end-loops without center jewelry", () => {
  for (const id of ["german-self-bow-1544", "composite-recurve-bow-1544"]) {
    const source = preset(id), result = validateWeapon(source.definition, source.controls);
    assert.equal(result.valid, true, `${id}: ${result.errors.join(" | ")}`);
    const labels = result.mesh.parts.map((part) => part.label);
    for (const label of ["upper bowstring control span", "lower bowstring control span", "served nocking control span", "upper bowstring end loop", "lower bowstring end loop"])
      assert.equal(labels.filter((candidate) => candidate === label).length, 1, `${id}: ${label}`);
    assert.equal(labels.some((label) => label.includes("nocking loop")), false, "center toroidal loops were removed");
    for (const label of ["upper bowstring end loop", "lower bowstring end loop"]) {
      const loop = result.mesh.parts.find((part) => part.label === label);
      assert.ok(signedVolume(loop) > 0);
      assert.deepEqual(closedManifoldErrors(loop, label), []);
    }
  }
});

test("self bow uses a deep D-section and composite bow exposes three material layers", () => {
  const self = validateWeapon(preset("german-self-bow-1544").definition, preset("german-self-bow-1544").controls),
    composite = validateWeapon(preset("composite-recurve-bow-1544").definition, preset("composite-recurve-bow-1544").controls);
  assert.equal(self.valid, true, self.errors.join(" | "));
  assert.ok(self.mesh.parts.some((part) => part.label === "upper D-section bow limb"));
  assert.ok(preset("german-self-bow-1544").definition.components[0].limbDepth >= 0.03);
  assert.equal(composite.valid, true, composite.errors.join(" | "));
  for (const layer of ["wood core", "horn belly", "sinew backing"])
    assert.equal(composite.mesh.parts.filter((part) => part.label.endsWith(layer)).length, 2, layer);
  assert.equal(new Set(composite.mesh.parts.filter((part) => /core|belly|backing/.test(part.label)).map((part) => part.materialId)).size, 3);
});



test("arrow nock has a real open slot wider than the default bowstring", () => {
  const source = preset("flight-arrow-1544"), result = validateWeapon(source.definition, source.controls),
    component = source.definition.components[0], nock = result.mesh.parts.find((part) => part.label === "slotted arrow nock");
  assert.equal(result.valid, true, result.errors.join(" | "));
  assert.ok(component.nockSlotWidth > preset("german-self-bow-1544").definition.components[0].stringRadius * 2);
  for (const triangle of triangleVertices(nock)) {
    const xs = triangle.map((point) => point[0]), ys = triangle.map((point) => point[1]);
    assert.ok(!(Math.max(...ys) < -component.nockLength * 0.45 && Math.min(...xs) < -component.nockSlotWidth / 2 && Math.max(...xs) > component.nockSlotWidth / 2), "triangle bridges the open nock slot");
  }
});

test("arrow nock clearance metadata covers every compatible bow-string endpoint", () => {
  const arrow = preset("flight-arrow-1544"), slot = arrow.controls.find((control) => control.label === "Nock slot width"),
    maximumStringRadius = Math.max(...["german-self-bow-1544", "composite-recurve-bow-1544"].map((id) => preset(id).controls.find((control) => control.label === "String thickness").max)),
    component = arrow.definition.components[0];
  assert.equal(component.maximumStringRadius, maximumStringRadius);
  assert.ok(slot.min >= maximumStringRadius * 2 + component.nockClearance);
  for (const slotWidth of [slot.min, slot.max]) {
    const changed = copyPreset(arrow); changed.definition.components[0].nockSlotWidth = slotWidth;
    assert.equal(validateWeapon(changed.definition, changed.controls).valid, true, `slot ${slotWidth}`);
  }
});

test("quiver has a sealed bottom and remains hollow at the mouth", () => {
  const source = preset("arrow-quiver-1544"), result = validateWeapon(source.definition, source.controls),
    body = result.mesh.parts.find((part) => part.label === "open arrow quiver body"),
    cap = result.mesh.parts.find((part) => part.label === "sealed quiver bottom"),
    mouthY = source.definition.components[0].length;
  assert.equal(result.valid, true, result.errors.join(" | "));
  assert.ok(cap && signedVolume(cap) > 0);
  for (const triangle of triangleVertices(body)) {
    if (triangle.every((point) => Math.abs(point[1] - mouthY) < 1e-7))
      assert.ok(triangle.every((point) => Math.hypot(point[0], point[2]) > source.definition.components[0].mouthRadius - source.definition.components[0].wall - 1e-6), "mouth contains a center cap");
  }
});



test("representative bow, arrow, and carrier controls materially alter owned geometry", () => {
  for (const [id, label] of [["german-self-bow-1544", "Limb reflex"], ["flight-arrow-1544", "Nock slot width"], ["arrow-quiver-1544", "Quiver mouth radius"]]) {
    const source = preset(id), baseline = validateWeapon(source.definition, source.controls), changed = copyPreset(source),
      control = changed.controls.find((candidate) => candidate.label === label);
    setControlValue(changed.definition, control, control.max);
    const result = validateWeapon(changed.definition, changed.controls);
    assert.equal(result.valid, true, `${id}/${label}: ${result.errors.join(" | ")}`);
    assert.notDeepEqual(result.mesh.positions, baseline.mesh.positions);
  }
});

test("discrete bow, ammunition, and carrier choices remain structurally valid", () => {
  for (const id of ["german-self-bow-1544", "composite-recurve-bow-1544", "flight-arrow-1544", "arrow-quiver-1544"]) {
    const source = preset(id);
    for (const choice of source.choiceControls ?? []) for (const option of choice.options) {
      const changed = copyPreset(source);
      setControlValue(changed.definition, choice, option.value);
      const result = validateWeapon(changed.definition, changed.controls);
      assert.equal(result.valid, true, `${id}/${choice.label}/${option.label}: ${result.errors.join(" | ")}`);
    }
  }
});

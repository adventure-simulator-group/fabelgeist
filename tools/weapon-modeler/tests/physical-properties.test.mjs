import assert from "node:assert/strict";
import test from "node:test";
import { buildWeapon, measureMassProperties } from "../src/mesh.js";
import { automaticGripPoint } from "../src/glb-export.js";
import { PRESETS, copyPreset } from "../src/presets.js";

function physical(definition) {
  const mesh = buildWeapon(definition);
  return measureMassProperties(mesh, automaticGripPoint(mesh.resolvedDefinition));
}

test("historical default families retain plausible mass and length", () => {
  // Broad family envelopes, not claims that a single museum object defines a type.
  const envelopes = [
    ["kriegsspiess", 4.5, 5.5, 2.5, 5],
    ["katzbalger", 0.75, 0.9, 0.8, 1.6],
    ["rondel-dagger", 0.35, 0.55, 0.15, 0.55],
    ["grosse-messer", 0.95, 1.25, 0.9, 1.7],
    ["reitschwert-1540", 0.95, 1.2, 0.9, 1.6],
  ];
  for (const [id, minimumLength, maximumLength, minimumMass, maximumMass] of envelopes) {
    const preset = PRESETS.find((entry) => entry.id === id);
    assert.ok(preset, id);
    const mesh = buildWeapon(preset.definition);
    const properties = physical(preset.definition);
    assert.ok(mesh.stats.dimensions[1] >= minimumLength && mesh.stats.dimensions[1] <= maximumLength, `${id}: ${mesh.stats.dimensions[1]} m`);
    assert.ok(properties.massKg >= minimumMass && properties.massKg <= maximumMass, `${id}: ${properties.massKg} kg`);
  }
});

test("blade depth changes rendered section, mass and inertia together", () => {
  for (const id of ["landsknecht-longsword", "grosse-messer", "glaive"]) {
    const preset = copyPreset(PRESETS.find((entry) => entry.id === id));
    const blade = preset.definition.components.find((part) => ["sectionBlade", "blade", "glaive"].includes(part.kind));
    assert.ok(blade, id);
    const before = physical(preset.definition);
    const meshBefore = buildWeapon(preset.definition);
    blade.thickness *= 1.5;
    const after = physical(preset.definition);
    const meshAfter = buildWeapon(preset.definition);
    const partDepth = (mesh) => {
      const part = mesh.parts.find((part) => part.label === blade.label);
      const depths = part.positions.filter((_, index) => index % 3 === 2);
      return Math.max(...depths) - Math.min(...depths);
    };
    assert.ok(partDepth(meshAfter) > partDepth(meshBefore) * 1.4, id);
    assert.ok(after.massKg > before.massKg, id);
    assert.ok(after.momentOfInertiaKgM2 > before.momentOfInertiaKgM2, id);
  }
});

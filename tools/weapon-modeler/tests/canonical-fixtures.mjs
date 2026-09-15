import { generateModel } from "../src/kernel.js";

// Test fixtures enter the same recipe boundary as the editor. No geometry is
// constructed here; the independent auditors inspect the returned surfaces.
export function plateFixture(plate, nodes, lod = "medium") {
  const [from, to] = plate.outline;
  const mesh = generateModel({ components: [{
    id: "fixture", kind: "guardAssembly", label: "fixture", material: "steel",
    attach: { to: "weapon.root" }, nodes: { ...nodes, root: [0, 0, 0] },
    anchorNode: "root", members: [{ path: [from, to], section: "round",
      sectionWidth: 0.002, sectionDepth: 0.002 }], plates: [plate],
  }] }, { lod });
  return mesh.parts.find(part => part.label === "fixture plate 1");
}

export function componentFixture(component, lod = "medium") {
  return generateModel({ components: [{ ...component, attach: { to: "weapon.root" } }] }, { lod });
}

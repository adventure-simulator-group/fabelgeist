// Reproducible analytical corpus for every authored browser recipe.
import { mkdir, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { PRESETS, copyPreset, HAFT_MODULES, HEAD_ASSEMBLIES, composeWeapon, compositionControls } from "./src/presets.js";
import { validateWeapon, measureMassProperties } from "./src/mesh.js";
import { automaticGripPoint } from "./src/glb-export.js";

const output = resolve(process.argv[2] ?? "../../output/weapon-audit/browser");
await mkdir(output, { recursive: true });
const cases = [], specimens = [];
const assemblies = HAFT_MODULES.flatMap((haft) => HEAD_ASSEMBLIES.map((head) => {
  const definition = composeWeapon(haft.id, head.id);
  return { id: `${haft.id}--${head.id}`, name: `${haft.name} / ${head.name}`, definition,
    description: "Freeform assembly study; not an independently authenticated 1544 weapon type.",
    controls: compositionControls(definition), choiceControls: [] };
}));
for (const preset of [...PRESETS, ...assemblies]) {
  const { definition, controls } = copyPreset(preset);
  const result = validateWeapon(definition, controls, { lod: "high" });
  if (!result.valid) throw new Error(`${preset.id}: ${result.errors.join("; ")}`);
  cases.push({ id: `${preset.id}-default`, name: preset.name, variant: "Default", definition, changes: [], rejected: [] });
  specimens.push({ id: preset.id, group: preset.id.includes("--") ? "assembly" : "browser", name: preset.name, description: preset.description,
    design: definition, physical: measureMassProperties(result.mesh, automaticGripPoint(result.resolved)),
    parts: result.mesh.parts.map((part) => ({ id: part.componentId ?? part.label,
      material: part.material, density: part.material?.density,
      positions: Array.from({ length: part.positions.length / 3 }, (_, i) => part.positions.slice(i * 3, i * 3 + 3)),
      indices: part.indices,
    })),
  });
}
await writeFile(resolve(output, "browser.json"), JSON.stringify(specimens));
await writeFile(resolve(output, "fixtures.json"), JSON.stringify({ seed: 1544, cases }, null, 2));
console.log(`Exported ${specimens.length} browser recipes`);

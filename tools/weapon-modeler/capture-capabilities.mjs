// Capture authoring inputs before replacing the generator. Generated evidence
// belongs outside the source tree; this command never updates its own baseline.
import { mkdir, writeFile, readFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { PRESETS, HAFT_MODULES, HEAD_ASSEMBLIES, copyPreset, composeWeapon, compositionControls } from "./src/presets.js";
import { validateWeapon } from "./src/kernel.js";


const root = fileURLToPath(new URL("../../", import.meta.url));
const output = process.argv[2] ? resolve(process.argv[2]) : resolve(root, "target/weapon-unification/baseline");
await mkdir(dirname(output), { recursive: true });
await mkdir(output);
const presets = PRESETS.map(copyPreset);
const compositions = HAFT_MODULES.flatMap(haft => HEAD_ASSEMBLIES.map(head => {
  const definition = composeWeapon(haft.id, head.id);
  return { id: `${haft.id}--${head.id}`, definition, controls: compositionControls(definition) };
}));
const sourceDirectory = dirname(fileURLToPath(import.meta.url));
const sourceNames = ["kernel.js", "math.js", "renderer.js", "topology.js", "presets.js", "glb-export.js"];
const sources = {};
for (const name of sourceNames) {
  const contents = await readFile(resolve(sourceDirectory, "src", name));
  sources[name] = createHash("sha256").update(contents).digest("hex");
}
sources["weapon-kernel.wasm"] = createHash("sha256").update(await readFile(resolve(root,"target/weapon-modeler-kernel/nodejs/weapon-kernel_bg.wasm"))).digest("hex");
const cases = [];
for (const specimen of [...presets, ...compositions]) {
  for (const lod of ["low", "medium", "high"]) {
    const validation = validateWeapon(specimen.definition, specimen.controls, { lod });
    if (!validation.valid) throw new Error(`${specimen.id}/${lod}: ${validation.errors.join("; ")}`);
    const grip = validation.mesh.physical.controlPoint;
    cases.push({ id: specimen.id, lod, grip, physical: validation.mesh.physical, stats: validation.mesh.stats,
      parts: validation.mesh.parts });
  }
}
await writeFile(resolve(output, "capabilities.json"), JSON.stringify({ sources,  presets, compositions }, null, 2), { flag: "wx" });
await writeFile(resolve(output, "geometry.json"), JSON.stringify(cases), { flag: "wx" });
console.log(`Captured ${presets.length} presets, ${compositions.length} compositions, and ${cases.length} generated cases in ${output}`);

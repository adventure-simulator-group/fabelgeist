// Exercise the actual WASM binding and native transport with identical recipes.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import { resolve } from "node:path";
import { mkdir, writeFile } from "node:fs/promises";

const root = fileURLToPath(new URL("../../", import.meta.url));
const output = resolve(root, "target/weapon-unification/parity");
await mkdir(output, { recursive: true });
const require = createRequire(import.meta.url);
execFileSync(process.execPath, [resolve(root, "tools/weapon-modeler/build-kernel.mjs")], { cwd: root, stdio: "inherit" });
const { weapon_model_request } = require(resolve(root, "target/weapon-modeler-kernel/nodejs/weapon-kernel.js"));
const target = resolve(root, process.env.CARGO_TARGET_DIR ?? "target");
execFileSync("cargo", ["build", "-p", "adventuresim-weapon-model-browser", "--example", "request"], { cwd: root, stdio: "inherit" });
const executable = resolve(target, `debug/examples/request${process.platform === "win32" ? ".exe" : ""}`);
const wasm = request => JSON.parse(weapon_model_request(JSON.stringify(request)));
const native = request => JSON.parse(execFileSync(executable, { input: JSON.stringify(request), encoding: "utf8", maxBuffer: 64 * 1024 * 1024 }));
// Small platform differences in libm may move a vertex by a few float32 ULPs.
const tolerance = 2e-6;
function compare(left, right, path = "result") {
  if (typeof left === "number" && typeof right === "number") {
    assert.ok(Number.isFinite(left) && Number.isFinite(right));
    assert.ok(Math.abs(left - right) <= tolerance * Math.max(1, Math.abs(left), Math.abs(right)), `${path}: ${left} != ${right}`);
  } else if (left && typeof left === "object") {
    assert.deepEqual(Object.keys(left), Object.keys(right), path);
    for (const key of Object.keys(left)) compare(left[key], right[key], `${path}.${key}`);
  } else assert.equal(left, right, path);
}
const catalog = wasm({ operation: "catalog" });
assert.deepEqual(catalog, native({ operation: "catalog" }));
const results = [];
for (const id of catalog.gameplay) {
  const design = wasm({ operation: "gameplay-design", id });
  const request = { operation: "generate", design };
  const actual = wasm(request), expected = native(request);
  // Encoded recipes and connectivity must be exact, independent of float math.
  assert.deepEqual(actual.recipe, expected.recipe);
  for (let part = 0; part < actual.mesh.parts.length; part++) assert.deepEqual(actual.mesh.parts[part].indices, expected.mesh.parts[part].indices);
  compare(actual, expected, id);
  results.push({ id, parts: actual.mesh.parts.length, massKg: actual.mesh.derived.mass_kg });
}
await writeFile(resolve(output, "report.json"), JSON.stringify({ tolerance, results }, null, 2));
console.log(`Native/WASM parity passed for ${results.length} gameplay recipes`);

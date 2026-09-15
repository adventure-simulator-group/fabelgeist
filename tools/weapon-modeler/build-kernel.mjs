import { execFileSync } from "node:child_process";
import { mkdir, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { resolve } from "node:path";

const root = fileURLToPath(new URL("../../", import.meta.url));
const output = resolve(root, "target/weapon-modeler-kernel");
const target = resolve(root, process.env.CARGO_TARGET_DIR ?? "target");
execFileSync("cargo", ["build", "-p", "adventuresim-weapon-model-browser", "--target", "wasm32-unknown-unknown"], { cwd: root, stdio: "inherit" });
for (const format of ["web", "nodejs"]) {
  const destination = resolve(output, format);
  await mkdir(destination, { recursive: true });
  execFileSync("wasm-bindgen", [resolve(target, "wasm32-unknown-unknown/debug/adventuresim_weapon_model_browser.wasm"), "--target", format, "--out-dir", destination, "--out-name", "weapon-kernel"], { cwd: root, stdio: "inherit" });
  await writeFile(resolve(destination, "package.json"), JSON.stringify({ type: format === "nodejs" ? "commonjs" : "module" }));
}
console.log(`Built canonical weapon bindings in ${output}`);

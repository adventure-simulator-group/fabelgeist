const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { spawnSync } = require("node:child_process");
const test = require("node:test");
const { rewrite } = require("./strategic-startup-profile.cjs");

test("startup diagnostics retain valid JavaScript across the production module boundaries", () => {
  for (const name of ["strategic-renderer.js", "strategic-scene.js", "strategic-scene-request.js",
    "strategic-generation.js", "strategic-generation-cache.js"]) {
    const source = fs.readFileSync(path.join(__dirname, "../static", name), "utf8");
    const instrumented = rewrite(name, source);
    const checked = spawnSync(process.execPath, ["--check", "--input-type=module"],
      { input: instrumented, encoding: "utf8" });
    assert.equal(checked.status, 0, `${name}: ${checked.stderr}`);
    assert.ok(instrumented.includes("startupProfile.mark"), name);
  }
});

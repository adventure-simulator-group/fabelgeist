// A focused-map arrival uses the same populated client and retained document
// as settlement travel. Require complete city, terrain and actor readiness.
const assert = require("node:assert/strict");
const path = require("node:path");
const { spawnSync } = require("node:child_process");
assert(process.env.STRATEGIC_STARTUP_FIXTURE_DIR,
  "Export canonical distinct settlement fixtures before benchmarking");
const result = spawnSync(process.execPath,
  ["--test", path.join(__dirname, "strategic-scene.browser.cjs")], {
    stdio: "inherit",
    env: { ...process.env, STRATEGIC_RENDER_BENCHMARK: "1",
      STRATEGIC_TRAVEL_BENCHMARK: "1", STRATEGIC_PROFILE_PLACE: "map",
      STRATEGIC_TRAVEL_PLACE: "map" },
  });
if (result.error) throw result.error;
process.exitCode = result.status ?? 1;

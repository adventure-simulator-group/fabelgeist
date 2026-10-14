# Strategic client initialization benchmarks

Initialization starts with browser navigation or an in-document destination
change and ends when the requested presentation is usable. Server startup and
native fixture generation happen before timing starts. Cold means the first
connection of a game client, with an empty browser context. Warm means travel
in that running client, including its first visit to a settlement. It retains
one document, canvas, WebAssembly instance, graphics device, and shared assets.

The targets are 30 seconds cold and 3 seconds warm. The first visit to a new
settlement has the warm target. Returning to a cached city is a separate
measurement and cannot establish that target.

## Canonical destinations

Export both destinations through the native dispatcher. They have different
settlement identities, geographic origins, building programs, and operator
identities. Each export contains the actor input, focused city input, and
matching neighborhood and district environments. Serve the original JSON text
to preserve full-width physical seeds.

```powershell
$env:STRATEGIC_STARTUP_FIXTURE_DIR = "$PWD/target/startup-fixtures"
cargo test -p adventuresim-tactical-server-dispatcher `
  export_distinct_startup_destinations -- --ignored --nocapture
```

Population defaults to 900; `STRATEGIC_STARTUP_FIXTURE_POPULATION` selects a
larger representative workload. Record population, building counts, input
hashes, source revision, browser, hardware, and executable hashes with each
measurement. Fixtures use isolated synthetic terrain and local HTTP responses;
these runs do not establish production network or database latency.

## Client workloads

Build the release browser client and synchronize assets. Export the equipment
fixture into the review directory before the populated browser test:

```powershell
$env:STRATEGIC_SCENE_REVIEW_DIR = "$PWD/target/startup-review"
$env:STRATEGIC_SCENE_FIXTURE_DIR = "$env:STRATEGIC_SCENE_REVIEW_DIR/fixtures"
cargo test -p adventuresim-tactical-client --bin adventuresim-tactical-client `
  procedural_carry_recipes_survive_the_browser_appearance_boundary
$env:STRATEGIC_WASM_DIR = "$PWD/crates/adventuresim-stdb-module/static/wasm"
$env:STRATEGIC_ASSET_DIR = "$PWD/crates/adventuresim-stdb-module/static/assets"
$env:STRATEGIC_RENDER_BENCHMARK = '1'
$env:STRATEGIC_TRAVEL_BENCHMARK = '1'
node --test crates/strategic-web/tests/strategic-scene.browser.cjs
```

The settlement workload opens an inn with a 22-person roster and two visible
portraits. Travel replaces the native settlement input and resident IDs. Shared
architectural families, body templates, and already used equipment kernels may
be reused; the complete destination is unprepared before the first arrival.

Run the focused-map workload with the same environment:

```powershell
node crates/strategic-web/tests/strategic-init-benchmark.cjs
```

Its finish condition simultaneously requires current actor readiness, the
requested geographic window, drawn terrain coverage, environment readiness,
settled presentation, completed city preparation, acknowledged city installation,
and visible city detail. A terrain-only ready flag is insufficient. Both
workloads record cold A, unseen B, and return A while retaining the original
canvas. `cold.json` and `travel.json` contain timings and readiness metrics.

Hardware selection is enabled in headless Chromium. The actual application's
adapter request is recorded without a preliminary GPU probe, and fallback
software adapters fail acceptance. `STRATEGIC_BROWSER_CHANNEL` selects an
installed channel; record it and use the same channel for comparisons.

## Attribution

Generation metrics report discovery, dependency transfer, worker generation,
worker initialization, product receipt, and final admission. Overlapping worker
and cache durations cannot be added to obtain elapsed loading time. Native
logs report focused ground geometry, support indexing, and ground surface
texture preparation.

Use `STRATEGIC_STARTUP_PROFILE=1` for separate diagnostic runs. Profiling changes
execution cost; acceptance uses repeated unprofiled runs on an idle host.
Keep timings, screenshots, profiles, input hashes, and failure evidence in
ignored output directories. Missing assets and GPU failures prevent full
acceptance; a recorded timeout is not a completed initialization time.

Diagnostic page snapshots and profiler RPCs have independent deadlines. Each
arrival has a five-minute investigation deadline, separate from the 30-second
and 3-second acceptance targets. Failure reports retain the last successful
checkpoint with its age. A timeout never establishes a completed startup time.
Browser traces use a browser-level connection and are saved before attempting
renderer-thread snapshots; incomplete traces are marked explicitly. The harness
owns and terminates its browser process tree, including a stalled GPU child.
The mock bridge test disables the GPU and cannot establish rendering acceptance.

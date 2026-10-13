# Strategic client initialization benchmarks

Initialization starts with a browser navigation or an in-document destination
change and ends when the requested presentation is usable. Server startup and
fixture generation happen before timing starts. A cold run uses a fresh browser
context. A warm run retains the document, canvas, WebAssembly module, graphics
device, and asset caches while replacing its destination.

The targets are 30 seconds cold and 3 seconds warm. A return to previously
prepared content is a separate measurement from an unseen destination.

## Settlement scenes

Use the real-renderer travel workflow in [UNIFIED_RENDERER.md](UNIFIED_RENDERER.md)
to record cold scene readiness and warm settlement changes. The fixture loads
the production bridge and renderer with captured inputs and local HTTP
responses. It measures client initialization; it does not establish production
network or database response times.

## Geographic maps

After building the release browser client, provide matching canonical city,
actor, and environment captures to the map benchmark:

```powershell
$env:STRATEGIC_INIT_WASM = "$PWD/crates/adventuresim-stdb-module/static/wasm"
$env:STRATEGIC_INIT_ASSETS = "$PWD/crates/adventuresim-stdb-module/static/assets"
$env:STRATEGIC_INIT_CITY = "$PWD/target/map-capture/canonical-city.json"
$env:STRATEGIC_INIT_ACTOR = "$PWD/target/map-capture/canonical-actor.json"
$env:STRATEGIC_INIT_ENVIRONMENT = "$PWD/target/map-capture/canonical-environments.json"
$env:STRATEGIC_INIT_OUTPUT = "$PWD/target/strategic-init-benchmark"
node crates/strategic-web/tests/strategic-init-benchmark.cjs
```

The city and environment documents must share a source and location. Native
city origins use ten-millionths of a degree; browser map positions and terrain
requests use microdegrees. The benchmark preserves physical seeds as opaque
document text when producing a second destination.

The benchmark records A → B → A. B changes scene identity and terrain seed while
retaining the captured building programs. This isolates destination replacement
and reusable geometry. It does not establish performance for new building
programs or new residents. Use distinct canonical captures for those cases.

`results.json` records terrain readiness and complete focused-city readiness
separately. The latter requires the requested city, its native installation
acknowledgement, and visible city detail. The usual map-ready attribute can
precede city detail and is insufficient on its own. The benchmark also records
generation metrics, native state transitions, requests, errors, missing assets,
and screenshots, and verifies that navigation retains one canvas.

The map fixture has no residents. It nevertheless exercises the production
bridge's actor-scene loading behavior. Its timings are separate from the scene
fixture's populated settlement timings.

## Attribution

Generation metrics report discovery, dependency transfer, worker generation,
worker initialization, product receipt, and final admission. Worker durations
and cache lookups can overlap; their sums are not elapsed loading time. Native
logs separately report focused ground geometry, support indexing, and ground
surface texture preparation.

Use the existing optional startup profiler for CPU and graphics attribution.
Profiled runs add overhead. Compare latency with separate unprofiled runs, using
the same inputs, release build settings, browser, hardware, and display size.
Retain input and executable hashes with results under an ignored `target/`
directory. Do not commit measurements or profiler transcripts as documentation.

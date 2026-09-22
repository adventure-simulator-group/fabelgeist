# Bench handoff

Where the GPU bench stands, how to work on it from WSL, what bit us, and the
next task. Read this and `bench/README.md` before touching anything.

## Where

- Worktree: `/home/v/dev/fresh-start/.claude/worktrees/bench`, branch `bench`
  (`git worktree list` from the main checkout shows it). Enter it with the
  worktree tool by path, or just `cd` there. The main checkout on
  `climb-cleanup` is the user's; never touch it.
- The bench is its own cargo package in `bench/` with its own target dir
  (`bench/target`, via `bench/.cargo/config.toml`), so its builds never take
  the main checkout's cargo lock. Run cargo from `bench/`.
- Commits so far: `cbb83b5` testbed, `404776b` two grass modes + affectors,
  `7569d5d` triangle blades, `4074d47` WebGPU prepass fix, `42dda83` camera
  recreation. Commit on `bench`; the user asked for commits there, not on
  their branch. Nothing is pushed.

## Workflow (WSL, Windows renderer, browser)

WSL's own renderer is not worth testing on. Everything runs through the
Windows binary or the browser:

- `bench/run-win.sh` builds for `x86_64-pc-windows-gnu` (`win-dev`
  profile), stages exe + the assets the bench loads to
  `/mnt/e/animation-playground-dev/bench`, and runs it through `cmd.exe`.
  `BUILD=0` skips the build. Env passthrough (WSLENV is set in the script):
  `BENCH_SETTINGS` (partial JSON of `settings.rs::BenchSettings`),
  `BENCH_LOG=run.csv`, `BENCH_EXIT_AFTER=secs`, `BENCH_SCREENSHOT=x.png`,
  `BENCH_SCREENSHOT_AT=secs` (default 6; the first ~3 s are pipeline
  compilation and render empty), `BENCH_SWITCH='4={"aa":"Fxaa"};8={...}'`
  (settings patches at those seconds, for scripted A/B and for reproducing
  live-switch crashes), `BENCH_GRASS_TIER_MASK`, `BENCH_MESH_NO_RANGE`.
  Relative log/screenshot paths land in the stage dir; read screenshots
  from `/mnt/e/animation-playground-dev/bench/*.png` with the image reader.
- `bench/scripts/smoke-win.sh '<json>' '<json>' ...` runs presets back to
  back (18–20 s each), prints the grass census, errors, and the median of
  the 2 s samples after warm-up via `scripts/summarize-csv.py`; screenshots
  `smoke_<n>.png`. It retries a preset once when a launch yields no samples.
- The CSV columns: t, mean/p50/p95/p99 ms, fps, settings line, GPU pass
  times (bevy render diagnostics; native only) plus `cpu_cull=` for the
  CPU-culled grass.
- Web: `just bench-web` (from the worktree root) builds `wasm-dev`
  (WebGPU, DWARF kept, ~360 MB bundle) into `bench/web/public` and serves it
  on http://localhost:8080 with the no-store + `/v<hash>/` prefix-strip
  server (`bench/scripts/serve-web.py`). `bench-web-webgl2`, `bench-trace`,
  `bench-web-release` as in skatepark. `bench/scripts/build-wasm-client.sh
  dev` alone rebuilds without serving; the running server picks up the new
  files (the engine URL carries a new `?v=` hash, tell the user to hard
  reload). A wasm build takes ~4–5 min; it shares `bench/target`'s cargo
  lock with Windows builds, so queue them.
- The user cannot be watched: report numbers from the CSV and screenshots,
  and always say which engine hash is being served.

## Pitfalls we hit (all real, all fixed in code, do not reintroduce)

- Windows throttles an unfocused window to the refresh rate: an exact
  16.67 ms in a run means the window never got focus, not GPU time. The
  runner retries `AppActivate('GPU bench')`; still treat 16.67 as suspect.
- bevy 0.19 replaces a provided `Aabb` when `Mesh3d` is added unless the
  entity has `NoAutoAabb`.
- bevy's `queue_material_meshes` dequeues visible meshes it never
  specialized; a custom opaque queue must run after it and re-add each
  frame (`grass/simple.rs`).
- `VisibilityRange { use_aabb: true }` culled every chunk; use the entity
  translation with a padded range.
- WebGPU-only failures that Vulkan lets through: `textureSample` in
  non-uniform control flow (branch on a per-object storage value, or any
  early `return` before the sample); naga_oil rejects identifiers starting
  with `_` or ending in digits inside composable modules; `from` is a
  reserved word; the prepass view layout has `globals` at binding 1, not 11.
- bevy keeps the background motion-vector pipeline on a retained render
  view after `MotionVectorPrepass` is removed: switching TAA → other AA →
  depth pre-pass failed validation. The camera entity is respawned on any
  AA / pre-pass / occlusion change (`scene::camera_bundle`).
- A depth-only pre-pass runs no fragment stage for opaque materials in
  bevy 0.19, so dithered fades leave sky holes under it; with TAA the
  motion-vector target forces the fragment and the custom prepass applies
  the dither. Documented, not fixable on our side.
- Empty meshes must not be added as assets (mesh allocator use-after-free
  log).
- Live settings changes that despawn entities need `try_insert` in systems
  touching those entities the same frame (wasm panics, native only warns).

## Numbers (RTX 3060 Ti, 720p, medians; see README for the caveats)

Full Fabelgeist density, 20 oaks: no grass 7.4 ms; simple 21.0; eidolon
22.3. Shading with oak + simple grass: Standard 21.5, LineBoil 17.9, Custom
18.2. Foliage alpha modes within noise. TAA + prepass + occlusion 15.0.

Bench budget (density 0.35, range 30, oaks, custom shading): mesh chunks
4.6 ms (triangle blades; 1.7 ms opaque pass), CPU-culled 6.9, simple 7.7,
eidolon 7.8, mesh chunks + TAA + occlusion + grass shadows 5.5. The mesh
mode draws far fewer blades per tuft than the game tufts the other three
share: compare by blade count.

## Tree v1 grass: tried and removed

Commit `1d2c9ff` ported commit 2896aad's triangle grass as a sixth mode; the
user judged it unimpressive and it was removed again (the eidolon
`try_insert` fix from that commit stays). `git show 1d2c9ff` has the port if
it is ever wanted. The panel is always on now (no F7).

## Since then: sprite cards and the displacement map

- `InstancingMode::Cards` (`grass/cards.rs`, `tools/pack_foliage_atlas.py`):
  triangle sprite cards, shader-side family/sprite picking from hashes, a
  sprite table storage buffer (binding 7) bound by every custom material,
  three atlas images for the mip knob, anisotropy knob. Cards ~11 k at the
  default density, opaque pass ~1.5 ms with the oak.
- `InstancingMode::MeshChunksMap` (`displacement.rs`): a 2D camera (layer
  1, order -10, active only in this mode) renders wind + affectors into an
  Rgba16Float map; grass kind 3 samples it (`grass_bend_map`, bindings
  8/9). Mapping verified with `BENCH_MAP_DEBUG=1` (rings centred on the
  balls). A 3D camera for this pass cost ~0.6 ms more than the analytic
  mode (per-view clustering and nodes) and clobbered the overlay's
  `main_opaque_pass_3d`; the 2D camera fixes both. bevy skips shadow views
  whose `RenderLayers` don't intersect the light's, so layer 1 builds no
  cascades.
- Measurements taken while the user was working on the machine were
  throttled (exact 16.67 ms tails); re-measure the map vs analytic delta on
  an idle desktop. `BENCH_SWITCH` A/B inside one run is the robust way.
- Trails: `TrampleMaterial` (two alpha-blended quads, layer 2, camera
  order -20, `ClearColorConfig::None`), the displacement quad samples the
  trample map (bindings 2/3). Verified with the debug exaggeration: swaths
  behind the balls. Two extra 2D draws per frame; the grass cost is
  unchanged.

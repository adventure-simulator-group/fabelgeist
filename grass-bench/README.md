# GPU bench

A standalone Bevy 0.19 package (`bench/`) that reproduces the tactical scene's
rendering load with every rendering choice on a runtime switch, so each one
can be measured on its own. No server, no netcode. Runs on Windows through
`just bench-win` (the development loop) and on WebGPU through `just bench-web`
(the product).

For the three stripped WebGL2 laptop tests, see [DOWNLEVEL.md](DOWNLEVEL.md).

## Scene

- The game's terrain heightfield with Fabelgeist's forest-floor textures
  (`tools/export_fabelgeist_ptex.py` bakes `assets/textures/ground/` from the
  `.ptex` recipes; the litter map is the terrain shader's id field, so it is
  shaded into a leaf-litter brown rather than used raw).
- Grass (`bench/src/grass/`), four paths sharing one placement lattice:
  - `bevy_eidolon`: the game's path (resident instances, GPU compute cull,
    indirect draws).
  - Simple (chunked): 8 m chunks, one static instance buffer each, bevy's
    CPU frustum + `VisibilityRange` culling, one `draw_indexed` per visible
    chunk, no compute pass. Same tuft meshes, bands and shader as eidolon.
  - Simple, CPU-culled: one persistent instance buffer per tier written in
    place each frame with the tufts that survive a CPU band + frustum test
    (chunk AABBs first, tufts only in straddling chunks), one draw per tier.
    The particles_and_trails buffer discipline applied to instances.
  - Mesh chunks (no instancing): every 8 m chunk near the camera is one baked
    mesh drawn through the custom material like any other opaque object;
    bevy culls, fades, shadows and prepasses it. Wind and the affector array
    run in the custom material's vertex shader behind a per-object flag. Its
    tuft is the reduced budget (4x4 blades, 3 rows), not the game's 64-blade
    tuft, so compare it by blade count, not tuft count.
  - Mesh chunks + displacement map (`displacement.rs`): the mesh-chunk
    geometry, but the affectors come from a small top-down render target
    over the patch instead of the per-vertex affector loop (wind stays
    analytic per blade: in the map it smeared under TAA). A 2D camera on its
    own render layer draws one quad with `shaders/displacement.wgsl` into an
    `Rgba16Float` image every frame (`Rgba8UnormSrgb` on WebGL2), containing
    every affector's push plus the trails; each grass vertex samples it once
    (`grass_bend_map`).
    The affector count never reaches the grass shader. The map camera is
    active only in this mode,
    so its pass (listed as `main_opaque_pass_2d`) is part of this mode's
    cost and no other. `Map resolution` knob: 128..1024 texels over the
    128 m square. `BENCH_MAP_DEBUG=1` exaggerates the pushes (radius x4,
    push x3) to eyeball the mapping. Trails (`Trails` knob, on by default):
    a ping-pong pair of maps (`shaders/trample.wgsl`) reads the previous
    frame's trails and writes their decay plus fresh footprints, without
    blending or CPU readback. The displacement pass adds the result so
    affectors leave flattened grass that relaxes over `Trail seconds`.
  - Sprite cards (no instancing) (`grass/cards.rs`): mesh chunks of textured
    plant cards from `assets/textures/foliage/atlas.png` (17 sprites in three
    families, packed by `tools/pack_foliage_atlas.py` from the janexx plant
    photos). Each plant is one triangle, the tightest apex-down triangle the
    packer fits over the sprite (about the same rasterised area as the tight
    rectangle: ratios 0.9 to 1.5), or a quad behind a knob. The mesh stores
    only the root (identical on every vertex), the facing normal and
    (corner, hash); the custom material's vertex shader picks the family from
    a value noise over the world root, the sprite and size from hashes, and
    reads the corner offset and atlas UV from a fixed uniform sprite
    table. Nothing is uploaded after the bake. The atlas ships as three
    images for the mip knob (mip 0 only, plain box chain, coverage-preserving
    chain with per-sprite alpha scaling) and the anisotropy knob rewrites the
    sampler. Alpha goes through the foliage alpha knob. 8 plants/m² at
    density 1.
  - `Range (m)` caps geometric grass for every path (tiers past it are
    dropped, the last one fades at the range); `Affectors` spawns N moving
    balls that push grass aside (uniform array in the custom material, the
    game's single interaction slot on the instanced paths).
- Trees: Fabelgeist's English oak exported to `assets/models/oak.glb`
  (`tools/build_oak_glb.py`, LOD0 630k tris + LOD1 110k tris, leaf cards
  alpha-masked, bark normal + AO), or a procedural placeholder.
- Characters: `assets/models/fabelgeist.glb` spawned N times (every armor
  piece visible; StandardMaterial load).
- One sun with two cascades, sunlight exposure, an orbiting camera.

## Knobs (always-on panel)

| Knob | Values | Notes |
|---|---|---|
| Anti-aliasing | Off, MSAA 2x/4x, FXAA, SMAA, TAA | TAA inserts and removes its whole component set (jitter, mip bias, depth + motion-vector prepass). |
| Depth pre-pass | on/off | Forced and greyed out under TAA. |
| GPU occlusion culling | on/off | Needs the depth pre-pass. |
| Sun shadows | on/off | |
| Shading | StandardMaterial, LineBoil (as-is), Custom (fixed) | Every StandardMaterial mesh gets equivalent boil and custom materials; the knob swaps the component. |
| Foliage alpha | A2C, Blend, Mask, Opaque + discard | Applied to every alpha-textured material in all three sets. Opaque + discard exists only in the custom shader. A2C silently runs as Mask without MSAA. |
| Instancing | No grass, Simple (chunked), Simple CPU-culled, bevy_eidolon, Mesh chunks, Mesh chunks + displacement map, Sprite cards | The three instanced paths share the tuft meshes, placement and shader body. |
| Map resolution | 128, 256, 512, 1024 | Displacement map texels per side; shown in the map mode. |
| Trails | on/off, trail strength, scorch, regrow seconds (0 = never) | Accumulated trail map (ping-pong pair, `max`, not a blend); flattens along the direction of travel, crushes the blades down and dries their colour out; regrows on its own; shown in the map mode. |
| Sprite cards | shape (triangle / quad), scale, atlas mips (off / plain / coverage preserving), anisotropy (1..16) | Shown when the cards mode is selected. Foliage alpha applies to the cards. |
| Density | 0.05..1 | Scales tufts per cell (default 0.35; 1 is Fabelgeist's). |
| Range | 8..72 m | Geometric grass cutoff for every path (default 30). |
| Affectors | 0..16 | Moving balls that push grass aside. |
| Trees | model, count, LOD mode | LOD0 only / LOD1 only / distance switch at 22–26 m. |
| Characters | count | |

Launch-time knobs (texture compression, atlas, mipmaps) are not implemented
yet; they belong to the asset pipeline phase.

## Custom material

`bench/src/custom_material.rs` is what the game's line-boil material should
become for batching and early-Z: bevy's stock mesh and prepass vertex
shaders, one small fragment shader, `discard` only under `MAY_DISCARD`
(alpha-tested pipelines) with a `FORCE_DISCARD` variant for the opaque +
discard test, a prepass fragment shader that applies the same alpha test, and
per-object colour and parameters in one storage buffer indexed by `MeshTag`
so every object sharing a texture shares one material handle and batch. The
grass flag in that table switches on the wind + affector vertex bend; the
affector list is a persistent raw buffer bound by every custom material and
written in place each frame, so it never touches a material asset.

The vendored line-boil material is measured as-is on purpose. Two things to
know when reading its numbers: its fragment shader discards unconditionally
(early-Z is off for everything it draws), and it has no prepass fragment
shader, so under a depth pre-pass or TAA its cut-out leaves write full-quad
depth (visible as sky-coloured rectangles).

## Running

```sh
just bench-win                                   # Windows renderer, interactive
BENCH_SETTINGS='{"aa":"Taa","shading":"Custom"}' \
BENCH_LOG=run.csv BENCH_EXIT_AFTER=20 BENCH_SCREENSHOT=shot.png just bench-win
bench/scripts/smoke-win.sh '{"instancing":"Simple"}' '{"instancing":"Eidolon"}'
just bench-web                                   # WebGPU build + http://localhost:8080
just bench-web-webgl2                            # WebGL2 engine
just bench-trace                                 # Firefox Profiler instrumentation
just bench-web-release                           # deploy-shaped bundle into bench/web/deploy
```

`BENCH_SETTINGS` is a partial JSON of the settings struct (`bench/src/settings.rs`);
on the web the same JSON goes in the `s` query parameter. The CSV has the
frame-time window (mean, p50, p95, p99), the settings line and bevy's GPU
pass timings (native only). `bench/scripts/smoke-win.sh` runs presets back to
back and prints medians; `BENCH_GRASS_TIER_MASK` isolates grass tiers.

The Windows runner brings the window to the front after launch: Windows
throttles unfocused windows to the refresh rate, which reads as a suspicious
exact 16.67 ms. `BENCH_SWITCH='5={"shading":"Custom"};10={"shading":"Standard"}'`
applies settings patches at those seconds for scripted A/B within one run.

Known gaps: the depth-only pre-pass (no TAA) cannot run a fragment stage for
opaque materials in bevy 0.19, so dithered fades (mesh-chunk grass, LOD
crossfades) leave sky-coloured holes in the fade band under it; with TAA the
motion-vector target forces the fragment and the custom prepass applies the
dither. Neither instanced grass path has a shadow phase; mesh chunks do.

## Layout

```
bench/Cargo.toml            own package, own target dir (bench/target)
bench/src/main.rs           app, plugins, asset prefix for the web build
bench/src/settings.rs       knobs, always-on panel, camera-side application
bench/src/stats.rs          frame stats, overlay, CSV / screenshot / exit hooks
bench/src/scene.rs          terrain, ground, sun, camera, trees, characters, LOD
bench/src/shading.rs        StandardMaterial -> boil/custom twins, swap, foliage alpha
bench/src/custom_material.rs + shaders/custom*.wgsl
bench/src/displacement.rs   the wind + affector displacement map pass (2D camera, render target)
bench/src/grass/            tiers, meshes, placement; eidolon.rs, simple.rs, culled.rs,
                            mesh_chunks.rs, cards.rs, shaders/
tools/pack_foliage_atlas.py plant photos -> bench/assets/textures/foliage/atlas.{png,json}
bench/scripts/build-wasm-client.sh, serve-web.py, smoke-win.sh, summarize-csv.py
bench/run-win.sh            stage to /mnt/e/animation-playground-dev/bench and run
bench/web/index.html        boot shell (streamed wasm, WebGPU probe, WebGL2 fallback)
```

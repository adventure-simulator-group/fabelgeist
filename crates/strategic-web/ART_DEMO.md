# Procedural art demo

The strategic server serves `/art-demo` without a session, character selection,
or database query. It uses one standalone Bevy application and canvas for the
document's lifetime. No tactical connection is opened.

Build the browser bundles with `just build-wasm`, then start the strategic
server through the usual development workflow. Both `adventuresim-tactical-client`
and `art-demo` JavaScript/Wasm bundles are generated into the existing tactical
static directory. The demo loads assets from `/tactical/assets` and its page
scripts and styles from `/static/art-demo`.

The `wasm-bindgen` CLI must match the version in `Cargo.lock`; the build checks
this before compiling. To use a separate matching installation, run
`python scripts/build_wasm.py --bindgen PATH_TO_WASM_BINDGEN`.

Open `/art-demo` over HTTPS or localhost in a browser with WebGPU. Drag to orbit
and scroll to zoom. WASD pans across all scenes. Touch supports one-finger orbit
and pinch zoom. With the canvas focused, arrow keys orbit and plus/minus zoom.
Tabs support arrow keys,
Home, and End. Exhibit hashes, such as `/art-demo#oak`, are directly linkable.
Browser Back and Forward select exhibits within the same document and canvas.

## Exhibits

- Armor: the full Henry VIII and fluted Nuremberg museum assemblies from
  `codex/museum-armor-sets` (PR #653), packaged as static GLBs with their saved
  poses and finishes. Each is shown beside its Met reference photograph.
- Weapons: six default recipes from `adventuresim-weapon-model`, generated in
  the viewer using the browser forge's material palette.
- Cityscape: a complete 30,000-resident settlement from the production city
  generator, using the massive-city fixture's seed, economy and building recipe
  selection. Exterior LODs include close-up framing, doors and windows. Zoom
  reaches a one-metre orbit distance; WASD explores the ground plane. Full
  gameplay collision is omitted. Outdoor furniture is prepared offline with
  the full city's buildings, gates and access reservations, then displayed
  through the shared furniture renderer. Display buildings are supplied after
  terrain generation, avoiding recipe compilation during tab navigation.
  Street materials load their detailed traffic masks within 110 metres of the
  camera, one tile per frame, and release distant masks. The overview retains
  all street and yard surfaces without allocating a city-wide wheel network.
  Building meshes are compiled offline with the production generator. Browser
  assembly yields between batches and reports processed building counts. It
  prioritizes buildings near the camera, requests at most four facade assets
  concurrently, and skips pending assets to display other ready buildings.
  Full detail is loaded for at most sixteen nearby buildings, one per
  frame, and its cache is released as the camera moves away. Facades remain
  visible while full detail loads. Switching tabs cancels pending buildings
  and releases inspection detail. Prepared facade handles remain cached for
  subsequent visits.
- Oak: an exposed generated oak in `sparse-woodland`, with its hilly terrain,
  production bark, foliage, and ground scatter. The initial low close-up frames
  the roots, trunk, and lower canopy. Its fixed playable tree and vista tree
  impostors are prepared offline in `assets/art-demo/oak.tree-impostors` rather
  than software-baked during browser startup. Other tactical scenes continue to
  bake tree impostors on demand for their generated specimens.

The oak generator's existing root-spread and root-exposure parameters extend
and raise the buttress shoulders while leaving the tapered tips buried. This
applies to exposed production oaks as well as the demo; the unexposed recipe is
unchanged.

[The exhibit catalog](../../assets/art-demo/catalog.json) owns labels, fixed
initial framing, shipped asset paths, and museum metadata. The Rust boundary
accepts only known exhibit IDs and finite orbit/zoom/pan inputs. Museum
photographs are local assets with
[source and license records](../../assets/art-demo/ATTRIBUTION.md).

Armor and weapon entities remain resident after their first load. Background
prefetch prepares one studio exhibit at a time after outstanding studio loads
and city assembly finish. Only the selected exhibit is visible; studio lights
exist only while a studio exhibit is selected. Each exhibit remembers its
camera position. The fixed catalog bounds the studio cache to two armor scenes
and six weapons. City facade residency is bounded by the fixed city's recipes.

Scenery changes release terrain, scatter, tree caches, city inspection detail,
and traffic masks. New scenery allocations wait four frames for render-world
removals; cached studio exhibits can switch immediately. Returning to scenery
rebuilds its presentation and reuses prepared city facades. The GPU device,
sky, and shared procedural textures remain resident. The viewer uses the
production presentation plugin without running gameplay or persisting state.

The loading plate covers only renderer startup. Once the application responds,
progress appears in a small overlay and camera controls remain available while
assets arrive. Ready status describes exhibit assembly and tracked scene
dependencies. Shared textures, shader preparation, and uploads may continue.
Failed armor or building assets produce an exhibit-local message without
preventing navigation. A renderer failure still requires reloading the document.

The demo selects 4× MSAA because WebGPU does not support the desktop 2× preset.
Terrain shares identical litter and cliff samplers to keep its production
material within WebGPU's sixteen-sampler limit.
Cached litter prototypes retain CPU geometry for later scene assembly; completed
batches upload to the renderer without retaining their CPU copy.
Bark evaluates soil texture derivatives in uniform control flow, as WebGPU
requires, while keeping terrain-height lookups bounded to the root-contact band.

## Museum assembly packaging

`scripts/export_art_demo_armor.py` runs inside Blender and reuses the museum
branch's `render_museum_armor.py` import, material and pose rules. Pass
`--renderer`, a finished `--preset` comparison JSON, `--output`, and `--lod 4`.
It bakes that saved pose into static meshes and embeds the existing images.
Packaging then runs `scripts/optimize_art_demo_armor.py`: finish and normal maps
remain at up to 512 pixels, while broad occlusion-only maps become 256-pixel,
single-channel PNGs. Exact encoded duplicates share one buffer view. KTX2 is not
used because this Bevy 0.19 WebGPU glTF build does not support
`KHR_texture_basisu`.

The adjacent `.sources.json` files record input hashes, output hash, pose, LOD,
and separate metal, clothing, fastener, and body triangle counts. The generated
`.textures.json` inventory records every image's role, source and shipped
dimensions, channels, bytes, hash, duplicate status, geometry bytes, and
estimated decoded GPU residency. Repacking requires Pillow 11.3.0 and is
deterministic for that recorded encoder version. Packaging rejects metal totals
above the included body's triangle count or mixed equipment LODs.

Runtime character and armor exports support LOD4 through LOD6. Armor samples
the original recipe surfaces directly; no mesh decimation or vertex merging is
used. Fitting uses complete shells, while display metal retains the outer
sheets with two-sided materials. Dense geometry exists only as bake input.
The checked-in source and prepared character bases both use LOD4.

To rebuild a museum assembly, use its `body.json`, `armor.json`,
`breastplate.json`, `vambrace.json`, and `fasteners.json` from
`crates/adventuresim-armor-model/review/museum/{henry,nuremberg}` with the
character creator's recipe and design flags. Export at `--lod 4` with
`--armor-review-selection recipe --armor-review-dir RUNTIME_DIRECTORY`.
Repeat to a separate source directory with `--armor-bake-source`. Run
`scripts/finish_equipment.py RUNTIME_DIRECTORY --stage uv`, then use
`--stage bake --source-directory SOURCE_DIRECTORY` to bake the runtime GLBs.
Apply the museum's `metal.json` with `scripts/color_armor.py` and then its
`trim.json` with `scripts/trim_armor.py`.

Copy the museum's saved comparison preset and point its equipment items to the
finished runtime GLBs, its body to the runtime review's `body.json`, and its
skeleton to `assets_src/biped/unarmed/base.glb`. Review front, side, and quarter
views before packaging. These are standalone display assets; serving the demo
does not require a museum worktree or any dense bake inputs.

Regenerate the city layout with
`cargo run -p adventuresim-tactical-client --example generate-art-demo-city`.
The browser reads `assets/art-demo/city-layout.json` and the companion
`city-furniture.json`; expensive settlement recipe validation and furniture
placement run during asset generation instead of tab navigation.

Then run
`cargo run -p adventuresim-tactical-client --example prepare-art-demo-buildings`.
Pass `-- --output target/art-demo-buildings` to prepare an isolated copy for
validation without replacing the shipped assets.
This writes the production facade, shell and detail meshes to
`assets/art-demo/buildings`, keyed by the serialized recipe. Overview and
inspection assets are separate; placement materials and shop names remain
deterministic at runtime. Regenerate these assets after changing building
recipes or the city layout. The browser does not compile building geometry
or collision.

After changing the fixed oak specimen, tree geometry, or impostor renderer,
regenerate its canonical impostors with:

```console
cargo test -p adventuresim-tactical-client --bin art-demo \
  regenerate_art_demo_tree_impostors -- --ignored --nocapture
```

The non-ignored art-demo tests verify that every expected playable and vista
impostor exists and was baked from the current native source geometry.

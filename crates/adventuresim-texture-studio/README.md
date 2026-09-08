# Texture Studio

A standalone material editor and artistic-review renderer. The environment is on
the left, the Bevy viewport in the center, and the selected recipe's controls on
the right. It has no dependency on the strategic website or a running game server.

## Run and build

```text
cargo run -p adventuresim-texture-studio --bin adventuresim-texture-studio
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.108 --locked
python scripts/build_texture_studio.py
python -m http.server 8783 --directory target/texture-studio/site
```

Open `http://localhost:8783`. Publish the contents of `target/texture-studio/site`
to any static HTTPS host. The host must serve `.wasm` as `application/wasm`.
WebGPU is required; the loading screen reports unsupported browsers. There is
one WASM binary, instantiated for the renderer and a dedicated baking worker,
with a small HTML/JavaScript boundary for browser files, storage and workers.
No API server, accounts, cross-origin isolation headers or shared memory are
required. The build script checks the CLI version against `Cargo.lock`.

## Authoring

Choose any of the 24 catalog recipes. Controls cover physical scale, morphology,
color palettes, relief, roughness, feature placement and sampling coefficients.
Wood knots can be added, removed, positioned, stretched and leaned. Ring spacing,
flow around knots, fibers, vessels and rays are independent controls. Brick and
stone unit palettes are independent of mortar. Leaf palettes preserve species
shape. Advanced coefficients expose finer recipe tuning; hover for defaults and
bounds, or search by name. Mathematical constants, encoding rules, hash mixing
and fixed antialiasing kernels remain implementation details.

Edits bake a 128-pixel draft before refining to the selected output resolution.
Dragging coalesces edits; obsolete browser refinement jobs are terminated. Light,
camera and surface-response edits do not regenerate the procedural maps. The
native editor uses one background baking thread and discards obsolete results.

View a plane, sphere, beveled cube, cylinder or beam. Drag to orbit, scroll to
zoom, and use UV repeats/offsets to inspect tiling or a detail. Channel views show
the generated maps without lighting. Pin a completed material for comparison
under the same camera and lights. Diagnostic displacement applies height to a
subdivided plane and disables normal relief to avoid applying it twice.

Undo/redo covers document edits. Autosave restores the last document; named
presets are stored locally. Save preset downloads JSON, and Import reads it back.
Export maps downloads a ZIP containing base-level PNGs, complete raw mip payloads,
a packed bake, metadata and the authoring document. Exports wait for the latest
final-quality bake. Screenshot downloads the complete studio view.

## Native artistic review

The reviewer uses the same scene, material adapters, mip uploads, tonemapping and
portable documents as the editor. Bark and leaf bindings and shaders are shared
with the tactical renderer through `adventuresim-procedural-materials`.

```text
cargo run -p adventuresim-texture-studio --bin procedural-texture-lab -- list
cargo run -p adventuresim-texture-studio --bin procedural-texture-lab -- preset hewn-oak --output target/wood.texture.json
cargo run -p adventuresim-texture-studio --bin procedural-texture-lab -- capture --preset target/wood.texture.json --output target/wood.png
cargo run -p adventuresim-texture-studio --bin procedural-texture-lab -- compare --before target/before.texture.json --after target/after.texture.json --output target/compare.png
cargo run -p adventuresim-texture-studio --bin procedural-texture-lab -- export hewn-oak --preset target/wood.texture.json --output target/wood-maps
```

These are material diagnostics. They omit gameplay geometry, weather and the
complete tactical lighting setup. Forest litter uses a material-only adapter for
its packed terrain channels. Glass uses Bevy's transmission material and nominal
thickness; its per-texel thickness channel remains available for inspection.
Native captures require a graphics adapter. All generated evidence belongs under
`target/`.

# Heraldry Studio

A standalone authoring tool for parametric coats of arms on painted display
shields and panels. Rust controls, generation, and Bevy materials are shared by
the native and WebGPU applications. It has no game-state or database
integration.

```sh
cargo run -p adventuresim-heraldry-studio --bin heraldry-studio
```

Choose a reference study, then edit the arms in the left panel. Fields can be
divided, patterned, or quartered recursively. Add ordinaries, duplicate charges
for repeated arrangements, counterchange a charge, or add an inescutcheon.
The right panel controls anatomy, painted modeling, construction,
palette, and viewing. Drag the preview to turn it; scroll to zoom.

`German lion · c. 1530` uses the attributed Tom-L/Rinaldum drawing on a broad
450 × 450 mm shield. Under `Lion paint`, `Flat` retains its interior lines
with flat tinctures; `Modeled` applies restrained painted tones. `Shadows`
and `Highlights` independently control paint coverage. These choices preserve
the drawing and asymmetry. Painted marks stay fixed as physical lighting moves.
The source panel links the evidence and credits carried by exported artwork.

Under `Paint recipes`, each tincture selects a pigment and binder preparation.
`Recipe details` links its source and distinguishes German conservation
evidence from earlier Italian workshop instructions. `Open measured paint
mixer` offers ingredient sliders, reachable-color strips, and least-cost
matching within a color tolerance. Its limited gum-Arabic/parchment calibration
is separate from the catalog estimates. Base, shadow and highlight save
independent ingredient recipes; batch prices are editable session scenarios.
See the [measured mixer guide](../adventuresim-heraldry/references/MEASURED_PAINT.md)
for controls, CLI requests, sources and limits.
Recipe colors and finish are estimates, not measured pigment optics. The
[paint catalog](../adventuresim-heraldry/references/PAINT_RECIPES.md) documents
the choices and limits. Exported bundles and GLBs retain the ingredient and
source records.

`View` switches between the physical material, flat artwork, base color,
normals, roughness, metallic, coating coverage, and height. `Pin comparison`
retains a completed design beside the current one under the same viewing
conditions. `Undo` and `Redo` restore the complete document. JSON editing uses
the generator's validation; rejected text leaves the current design intact.

Under `Paint and metal leaf`, choose a technique independently for Or and
Argent: pigment, water gilding, oil gilding, or raised mordant gilding. Or also
offers yellow-glazed silver. Water gilding exposes a burnishing control;
raised mordant exposes adhesive relief; glazed silver exposes yellow optical
depth. Overall clear coating defaults to zero. `Sources` links the conservation
evidence behind these choices. Numeric strengths are artistic interpretations.

Edits first request a 128-pixel draft, then refine to the selected quality after
an idle interval. Native baking runs on one background thread at a time;
browser baking runs in a module worker. Obsolete results are discarded. Light,
exposure, rotation, and zoom preserve the bake. `Export bundle` requests the
selected quality and produces a ZIP containing the recipe, SVG, PNGs, material
metadata, mips, and GLB. Edits during export invalidate its download.

The browser imports JSON through a file picker and downloads exports. It
autosaves the document in local storage. The native JSON window accepts an
explicit filesystem path for reading or writing; `Read` loads text and
`Apply validated JSON` changes the design. Native autosaves and exports are
under `target/heraldry/`. `Capture PNG` saves the visible studio window. The CLI
produces a clean lit capture without editor controls. Add `--paint-mixer` to
`capture` to include the editor with the Or mixer open.

## Command-line workflow

```sh
# List recipes, then write one for editing.
cargo run -p adventuresim-heraldry-studio --bin heraldry-lab -- preset
cargo run -p adventuresim-heraldry-studio --bin heraldry-lab -- \
  preset imperial-eagle --output target/heraldry/eagle.json

cargo run -p adventuresim-heraldry-studio --bin heraldry-lab -- \
  validate target/heraldry/eagle.json
cargo run -p adventuresim-heraldry-studio --bin heraldry-lab -- \
  export target/heraldry/eagle.json --quality final --output target/heraldry/eagle
cargo run -p adventuresim-heraldry-studio --bin heraldry-lab -- \
  capture target/heraldry/eagle.json --output target/heraldry/eagle-lit.png

# Load the exported asset through Bevy's glTF importer for comparison.
cargo run -p adventuresim-heraldry-studio --bin heraldry-lab -- \
  capture target/heraldry/eagle.json --reload-glb target/heraldry/eagle/display.glb \
  --output target/heraldry/eagle-reloaded.png

cargo run -p adventuresim-heraldry-studio --bin heraldry-lab -- \
  compare target/heraldry/before.json target/heraldry/after.json \
  --output target/heraldry/comparison.png
```

Validation and exports need no GPU or window. Captures need a Bevy-supported
graphics adapter, including a software adapter when available. `--quality`
accepts `draft`, `preview`, `high`, or `final`. The GLB is a closed display
support in metres; it is not combat equipment.

## Browser build

Install `wasm32-unknown-unknown` and the `wasm-bindgen-cli` version matching
`Cargo.lock`, then build and serve the independent static application:

```sh
python3 scripts/build_heraldry_studio.py
python3 -m http.server 8137 --bind 127.0.0.1 --directory target/heraldry-studio/site
```

Open `http://localhost:8137` in a WebGPU browser with a usable graphics adapter.
The startup page reports missing adapter support before starting the renderer.
Use `--dev` for a development build and `--bindgen PATH` for an explicitly
installed CLI. No database, API
server, cross-origin isolation, or strategic website is required.

See the [generator contract](../adventuresim-heraldry/README.md) for formats and
the [reference guide](../adventuresim-heraldry/REFERENCES.md) for source
evidence and the limits of the historical interpretations.

After building the browser app and exporting a native recipe, compare the
worker's maps and height values against the native output without a GPU:

```sh
node crates/adventuresim-heraldry-studio/tests/wasm-parity.mjs \
  target/heraldry-studio/site target/heraldry/eagle.json \
  target/heraldry/eagle/material.bake
```

The check requires identical dimensions and bake identity. It allows one byte
of channel quantization and 0.00001 millimetres of height difference between
platforms, and reports the actual maximum errors.

# Procedural texture review

Recipes produce deterministic, repeating material maps. Hewn oak uses irregular,
asymmetric growth bands, tapered branch knots, longitudinal vessel tracks and
sparse transverse ray flecks in a shared deformed coordinate field. A 4x4
footprint filter integrates grain before height differentiation, reducing
aliasing where knots compress the narrow ridges. Continuously
blended adze facets supply broader relief. Dressed stone retains its ashlar bond
and planar centers while clustered fractures, occasional diagonal corner cuts,
locally varying bevels and sparse grouped cavities shape its edges and faces.
Lime mortar combines discrete aggregate, tapered application ridges, recessed
pockets and buildup where it meets stone.

These two recipes follow the environment art direction: oak has one solid
intrinsic color and roughness; stone has six solid block colors and three block
finishes, with one color and finish for mortar. Wear, tool marks and microdetail
exist only in height, normals and AO. Texture filtering can blend palette colors
at boundaries and in distant mip levels; it does not author surface history into
base color. The recipes approximate visible anatomy and fracture structure;
they do not simulate volumetric tree growth or physical erosion.

Hewn oak uses a 1024-pixel tile over 2 metres. Dressed stone uses a 2048-pixel tile
over 7.2 metres. Their nominal texel spacings are 1.95 mm and 3.52 mm respectively.
Stone cavities are deliberately sparse and large enough to resolve at that
density. The four RGBA maps, including mips, occupy approximately 21.3 MiB for
oak and 85.3 MiB for stone before GPU compression. This is four times their
previous texture allocation. Footprint integration increases oak bake work without
adding texture allocation; measure generation time separately from rendering.

## Export and compare

Run these from the repository root. Before changing a recipe, export its current
maps into `before`; after the change, export into `after`:

```text
cargo run -p adventuresim-procedural-textures --bin procedural-texture-lab -- export hewn-oak --output target/procedural-texture-lab/before
cargo run -p adventuresim-procedural-textures --bin procedural-texture-lab -- export hewn-oak --output target/procedural-texture-lab/after
cargo run -p adventuresim-procedural-textures --bin procedural-texture-lab -- compare hewn-oak --directory target/procedural-texture-lab --output target/procedural-texture-lab/hewn-oak-detail.png
cargo run -p adventuresim-procedural-textures --bin procedural-texture-lab -- compare hewn-oak --directory target/procedural-texture-lab --output target/procedural-texture-lab/hewn-oak-overview.png --view overview
```

Use `dressed-stone` for the masonry comparison. Each export writes inspectable
base-level PNGs and `.mips` sidecars containing the generator's complete byte
payload. Both are required for comparison; re-export both revisions when making
a new comparison. Hewn oak and dressed stone exports generate only that recipe.

The headless comparison requires a graphics adapter and uses Bevy
`StandardMaterial` with color, normal and packed AO/roughness/metallic maps, full
production mip chains and 8x anisotropic filtering, without height displacement.
`--view oblique --light-angle 55` rotates the panels and lights their facing side; `--view distance` displays four repeats per
axis to exercise minification. `--light-angle 55` changes the light azimuth;
`--diagnostic` replaces color and roughness with constants while retaining
normal and AO detail. All options apply equally to both panels. Capture multiple
light angles to distinguish lighting response from baked shading.

These are material diagnostics, not a tactical scene acceptance test: they omit
building geometry, the gameplay camera, weather and the game's complete lighting
setup. All generated evidence stays under `target/`.

Run `cargo test -p adventuresim-procedural-textures` for recipe repeatability,
tiling, feature scale, channel packing, and complete mip chains. Additional
behavioral tests check grain deflection, chipped boundary area, cavity coverage,
and mortar variation.

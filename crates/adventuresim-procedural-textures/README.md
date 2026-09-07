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

Hewn oak authors two intrinsic colors on broad, knot-deflected grain regions.
Handmade brick uses five unit colors and dressed stone uses six; each has an
independently selected mortar color. These are discrete material regions, with
no painted lighting, wear or continuous color gradients. Wood coverage is
integrated over a 4x4 texel footprint; masonry uses its boundary coverage. Both
boundary blending and color mip generation average in linear light. Intermediate
colors therefore represent antialiasing and minification, not extra painted detail.

Roughness stays simple: one value for oak, one each for brick and mortar, and
three block finishes for stone. Fine detail remains in height, normals and AO.
The recipes approximate visible anatomy and fracture structure; they do not
simulate volumetric tree growth or physical erosion.

## Color parameters

`generate_hewn_oak_textures` takes `&HewnOakColors { light, dark }`.
`generate_handmade_brick_textures` and `generate_dressed_stone_textures` take
`&MasonryColors<5>` and `&MasonryColors<6>` respectively. Their `units` array and
`mortar` color are independent. Each color is an `SrgbColor([r, g, b])` with
8-bit sRGB components. Colors do not affect height, normals, AO or roughness.

The standard texture collection passes the authored defaults `HEWN_OAK_COLORS`,
`HANDMADE_BRICK_COLORS` and `DRESSED_STONE_COLORS`. A caller can copy a default,
change any color, and pass it to the same generator:

```rust
use adventuresim_procedural_textures::{
    HANDMADE_BRICK_COLORS, SrgbColor, generate_handmade_brick_textures,
};
// `images` is the caller's Assets<Image> collection.
let mut colors = HANDMADE_BRICK_COLORS;
colors.mortar = SrgbColor([183, 173, 160]);
colors.units[0] = SrgbColor([152, 103, 76]);
let textures = generate_handmade_brick_textures(images, &colors);
```

The lab accepts `--wood-light '#84623e' --wood-dark '#6f5133'` for wood, and
`--unit-color '#98674c' --mortar-color '#b7ada0'` for either masonry recipe.
One `--unit-color` fills the whole unit palette; repeat it five times for brick
or six times for stone to choose each entry separately. Omitted controls retain
their default colors. Invalid hex colors, palette lengths, and options for the
wrong material are rejected.

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

Use `handmade-brick` or `dressed-stone` for masonry comparisons. Each export writes inspectable
base-level PNGs and `.mips` sidecars containing the generator's complete byte
payload. Both are required for comparison; re-export both revisions when making
a new comparison. These three exports generate only the selected recipe.

The headless comparison requires a graphics adapter and uses Bevy
`StandardMaterial` with color, normal and packed AO/roughness/metallic maps, full
production mip chains and 8x anisotropic filtering, without height displacement.
`--view oblique --light-angle 55` rotates the panels and lights their facing
side; `--view distance` displays four repeats per axis to exercise minification. `--light-angle 55` changes the light azimuth;
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

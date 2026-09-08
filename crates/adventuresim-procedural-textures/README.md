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

Hewn oak authors two intrinsic colors, darkening selected latewood ridges along
their exact relief boundaries, including where grain bends around knots. Fine
fibers and pores remain relief detail; color does not trace every feature.
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

## Parameters and authoring

Every generator accepts `&TextureParameters` plus the destination `Assets<Image>`.
Its typed recipe block owns the canonical defaults. For example:

```rust
use adventuresim_procedural_textures::{TextureParameters, SrgbColor, generate_handmade_brick_textures};
let mut parameters = TextureParameters::default();
parameters.handmade_brick.colors.mortar = SrgbColor([183, 173, 160]);
parameters.handmade_brick.colors.units[0] = SrgbColor([152, 103, 76]);
// `images` is the caller's Assets<Image> collection.
let textures = generate_handmade_brick_textures(&parameters, images);
```

`BakedRecipe::generate` generates only the selected recipe, including its complete
mip chain. `TextureParameters::from_value` validates imported JSON before typed
deserialization. Recipe control paths in `src/parameters/active.json` enumerate
the fields used by each recipe; update this authored metadata when adding or
moving controls. Tests verify that every catalog recipe has valid control paths.

Use the standalone [Texture Studio](../adventuresim-texture-studio/README.md) to
edit parameters live, export maps and presets, and capture native Bevy reviews.
The native reviewer and browser editor share the same scene and material bindings.

Hewn oak uses a 1024-pixel tile over 2 metres. Dressed stone uses a 2048-pixel tile
over 7.2 metres. Their nominal texel spacings are 1.95 mm and 3.52 mm respectively.
The four RGBA maps with mips occupy approximately 21.3 MiB for oak and 85.3 MiB
for stone before GPU compression. The editor first bakes a 128-pixel draft, then
refines to the chosen final resolution without blocking its rendering thread.

Run `cargo test -p adventuresim-procedural-textures` for repeatability, tiling,
feature scale, channel packing, mip completeness and parameter behavior.

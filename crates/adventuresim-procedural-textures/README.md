# Procedural texture review

The [catalogue approach review](catalogue-review.md) records the construction,
reference techniques and iteration decisions for all 24 recipes, including the
shared leaf presets. It also describes the finite surface features and filtered
grain controls added to the remaining building materials.

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

Tactical clients load [committed runtime bakes](../../assets/textures/procedural/README.md)
instead of generating textures during initialization. After editing a recipe,
run `just bake-procedural-textures <recipe-slug>` and commit its `.ptex` asset
with the source changes. Omit the slug to rebuild all recipes. The bake retains
the full-resolution pixels, complete mip chains, color spaces, and samplers.

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

## Normal direction

RGBA surface and optical normals use OpenGL tangent space: red follows U and
positive green points opposite image-row V. Increasing height down the image
must tilt the normal upward. Custom projections must also transform the sampled
normal through the same UV rotations used for color and height.

Leaf fronts already use this convention; their back maps reverse green for the
back-facing leaf frame. Forest litter instead packs world X/Z normals into RG.
Bark and soil derive their shading normals directly from height. Those distinct
contracts are intentional and must not receive a blanket green-channel flip.

Run `cargo test -p adventuresim-texture-studio --lib normal_orientation` for
height-versus-light direction checks using Bevy-generated tangents. The optical
height check is included in this crate's tests. With Node, Playwright and a
WebGPU-capable Chromium browser, `node scripts/test_texture_normal_projection.cjs`
checks the production rock projection WGSL against geometric height gradients
on all six axis directions.

## Shared leaves

Leaf species are presets of one compositional generator with 68 botanical controls,
including continuous lobe/leaflet counts and simple-to-compound separation.
`TextureParameters.leaves` owns shape and relief; `leaf_colors` owns independent
front/back palettes. The studio exposes the shape catalog and parameter morphing.
See [the model and reference verification](src/leaf/README.md).

## Forged iron and split slate

Ironwork uses overlapping die impressions, clustered angular scale losses and
small pits at a full resolution of 1024 pixels over 0.64 metres. The default
finish is intact black forge film. Bare iron and oxide have separate editable colors; their shared
coverage mask also controls metal/dielectric response. Albedo has two flat
colors plus coverage antialiasing, with linear-light mip filtering.

Slate retains its rising, overlapping roof layout and adds directional split
terraces, broken ledges and edge delamination. Its full bake is 2048 pixels over
4.8 metres; draft and medium remain 128/256. Surface detail changes relief and
response, while albedo stays flat per piece. `slate_roof.cleft` exposes layer,
warp, direction, fracture and flake controls; `ironwork` exposes stamp geometry,
scale/pit coverage, palettes and response. Relief depths are fractions of each
recipe's declared height range. Integer cell counts preserve periodicity.

The [ironwork research](prior-art/ironwork/prior-art.md) and
[slate research](prior-art/slate-roof/prior-art.md) describe the source techniques,
adaptations and limits. These generators describe reusable surfaces; object
contact wear and roof-boundary construction need consuming mesh/scene inputs.

## Surface review and precision

Bark and soil pack normalized 16-bit height into RG (most significant byte first),
AO into B, and opaque alpha into A. Mips average float heights before packing;
renderers decode with `decode_height_ao` or its linear WGSL equivalent. This
preserves shallow slopes and byte carries through texture filtering.

Masonry and clay roof edges have a physical rollover width independent of pigment
antialias coverage. Rubble combines stone-local fracture planes with varied course
boundaries. Flooring owns grain coordinates per butt segment; the shared cut-board
program creates open cathedral traces and locally deflected knots. Game leaf
presets refine the reference shape controls while retaining the common morphing
program and the original reference catalogue. Beech's game preset targets European
beech; reference presets remain available in the Studio.

The native material lab provides folded-sheet, framed-pane and repeated-crown
views, a reflection studio, and adjustable glass background separation. These
views supplement neutral/raking, tiling and raw-channel comparisons.

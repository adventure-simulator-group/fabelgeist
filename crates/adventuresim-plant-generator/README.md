# Parametric plants

Flowers use one organ generator. Presets supply physical dimensions, petal
shape, pigment, leaf arrangement, and flowering-shoot structure. The corolla
topology distinguishes free petals, ray-and-disk heads, and fused bells.
Species names never select geometry algorithms.

```powershell
cargo run -p adventuresim-plant-generator --features viewer --bin plant-viewer
```

Plant Studio edits all recipe fields, loads flower and fungus presets, and saves
JSON to `target/plant-recipe.json`. Drag to orbit and scroll to zoom. Pass
`--document target/plant-recipe.json` to reopen a recipe. Invalid dimensions
are rejected with a field-specific error before generation.

Recipes contain a `family` tag (`Flower` or `Fungus`) and a `parameters`
object. The family selector switches between the two shared generators.

For deterministic PBR captures of a preset (indices 0 through 4):

```powershell
cargo run -p adventuresim-plant-generator --features viewer --bin plant-viewer -- --preset 3 --view head --output target/poppy-head
```

Use `--view full` for the complete plant and `--lod high|medium|low` for the
mesh tier. Plant Studio also has an interactive detail selector.
Captures include two settled readbacks and a camera, seed,
parameter, and geometry-count manifest. New review runs should use fresh
output directories.

`FlowerParameters::generate` returns indexed `PlantMesh` data rooted at Y=0
in metres. `PlantMesh::into_bevy` uploads the same geometry used by the
preview and tactical renderer. `PlantLod` bounds geometry to 512, 128, and 32
triangles for High, Medium, and Low. Complex recipes aggregate organs within
those budgets, preserving the primary head and sampling the shoot arrangement.
Pigments are solid sRGB regions converted to linear vertex colors; lighting
and roughness belong to the renderer.

Run `cargo run -p adventuresim-plant-generator --example catalog_metrics` for
mesh counts and repeated CPU generation timings. These observations exclude
GPU rendering and should be collected without concurrent builds or captures.

## Tactical placement

The tactical client samples local ground and terrain, with canopy,
cultivation, soil moisture, snow, and calendar-day filters. Stable community
seeds select species, while separate specimen seeds control root jitter and
orientation. Roads, water, stone, reeds, steep slopes, and snowy sites are
excluded. Poppies favor cultivated open ground; wood anemones occur on
woodland litter in spring. These are conservative scene-level habitat
approximations, not a botanical survey of an individual location.

Flowering shoots select existing openings using the same feathered cover
mask as the grass renderer. Dense tall swards reject low herb placements;
flowers do not clear or shorten surrounding grass. Ground-cover inputs must
represent a suitable meadow margin, disturbed soil, or woodland understory.

Each scene retains at most 512 specimens, selected by stable hash priority
across the whole scene. Each specimen has three co-located mesh tiers, with
shared mesh/material handles for automatic instancing. Per-view distance from
the plant root controls complementary dithered crossfades:

| Tier | Triangle ceiling | Visibility |
| --- | --- | --- |
| High | 512 | Full below 2.25 m; fades to Medium over 2.25–2.75 m |
| Medium | 128 | Full from 2.75–6.5 m; fades to Low over 6.5–7.5 m |
| Low | 32 | Full from 7.5–22 m; fades out over 22–27 m |

Adjacent tiers briefly overlap during transitions. Identical roots and union
bounds prevent mismatched fade distances and frustum culling across tiers.
The `plant-lod-review` tactical capture profile records actual selected tiers
and triangle counts before, within, and after both transition bands.
`plant-lod-isolated` repeats those production roots and camera positions with
grass, understory, trees, and vista hidden for an unobstructed LOD diagnostic.
Its visual results concern switching and mesh shape, not habitat integration.

Plants are client presentation only; they add no
strategic database rows, tactical combat state, harvesting, or collision.

The initial five presets are a representative foundation for central Germany
in 1544. They do not exhaust its flora. Botanical and procedural references,
including the distinction between period evidence and modern distribution,
are in [SOURCES.md](SOURCES.md).

## Fungi

```powershell
cargo run -p adventuresim-plant-generator --features viewer --bin plant-viewer -- --family fungi --preset 2
```

Fungus preset indices are fly agaric, porcini, chanterelle, and common
puffball. `FungusParameters` controls the shared cap and stem profiles,
depression, thickness, rim waves, asymmetry, veil ring, ornaments, pigments,
and spore-bearing surface. `cap_elevation_m` sets the rim's nominal height;
the stem attaches to the underside of that cap, including a depressed funnel.
Validation rejects profiles whose upper and lower surfaces intersect.

The high tier retains gill/ridge relief, veil rings, and
ornaments. Submillimetre pores are aggregated into a continuous underside;
lower tiers spend their geometry on the cap, stem, and broad silhouette.
Use `--view underside` to inspect them and `--view head` for the cap. The same
capture and JSON editing workflow applies to fungi.

The combined `PlantSpecies` catalog shares one tactical population budget.
Moisture, seasonal fruiting intervals, woodland cover, and cultivation filter
fungal placement into existing ground openings. The current catalog contains
ground-fruiting fungi; wood-attached brackets require a separate attachment
policy. Representative native range supports their inclusion in the setting;
these recipes are not an exact reconstruction of a recorded 1544 locality.

For production habitat captures, use `plant-review` with
`flower-woodland-edge` in spring, or `fungus-review` with `fungi-woodland` in
autumn. The fungal profile frames actual roots by species, then their habitat.
The `flower-meadow` fixture represents dense tall grass and is an exclusion
case.

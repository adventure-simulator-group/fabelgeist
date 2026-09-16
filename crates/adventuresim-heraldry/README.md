# Parametric heraldry

The generator keeps heraldic identity, drawing interpretation, painted
construction, and viewing conditions separate. It has no Bevy dependency and
does not write game state. The
[studio](../adventuresim-heraldry-studio/README.md) provides native and browser
controls, JSON editing, and a command-line interface.

The current paint model estimates appearance from catalog swatches or measured
specimens and adds approximate surface relief. It does not yet simulate paint
batches deposited and dried in ordered layers. The
[physical paint contract](PHYSICAL_PAINT.md) defines the required material
model, calibration and acceptance tests.

Development currently focuses on the stylized lion as a complete worked
example, followed by geometric fields and ordinaries. Existing non-geometric
SVG artwork with suitable licenses is welcome; search Wikimedia Commons first
and inspect its construction before adapting it. Newly invented non-geometric
SVGs and text generation remain deferred while Adler Halbe develops related
2D weapon, armor, plant, and creature tooling. Before implementing non-trivial
patterns, consult existing techniques in Houdini and Shadertoy communities.
The [prior-art review](references/PRIOR_ART.md) identifies useful composition
and asset workflows; the [source record](references/ATTRIBUTION.md) defines
the provenance required for adapted artwork.

```rust
use adventuresim_heraldry::{
    artwork::Artwork,
    bake::{Baked, Resolution},
    document::{Document, Ratio},
    export,
};

let mut document = Document::default();
document.drawing.eagle.wing_lift = Ratio(1.2);
document.validate()?;
let artwork = Artwork::compose(&document)?;
let material = Baked::generate(&document, Resolution::Final)?;
let files = export::bundle(&document, &material, Resolution::Final)?;
# Ok::<(), adventuresim_heraldry::Error>(())
```

## Design contract

`Document.arms` owns tinctures, recursive fields, ordinaries, charge identities,
placement, repetition through multiple charges, and an optional inescutcheon.
Fields may be solid, divided, patterned, or quartered. Partitions and ordinaries
have straight, wavy, indented, and embattled boundaries. Dexter is the bearer's
right, which faces the observer's left. Positions start at the observer's upper
left. The chapter names and JSON enum tags are the public interface; there is no
blazon parser or text-to-image step.

`Document.drawing` owns anatomy and interpretation. Eagles grow from a torso,
neck profiles, wings with feather attachment roots, feet, and a tail fan. Lions
adapt Tom-L's Wikimedia Commons SVG after Rinaldum, based on a south German
armorial of c. 1530. A shared spatial deformation moves its compound contours,
internal strokes and painted tones together. Source claws, teeth and tongue
remain separately colored.
Proportion controls alter that construction; reference names are confined to
recipes. Eagle detail adds feather shafts, breast and shoulder hatching,
neck lines and outlines; contour character controls feather curvature.
Lion paint has separate shadow and highlight coverage controls. Flat paint
retains the source's mane drawing. Asymmetry is an independent drawing control.
These are authored interpretations; the charge identity stays fixed. Crowns,
beaks, claws, and tongues are explicit parts. These families do not cover every
historical pose.

Composition produces clipped Bézier paths. SVG and CPU rasterization consume
the same paths. Counterchanging clips the primary charge into the underlying
field regions and exchanges the selected pair of tinctures. It retains separate
accent colors. Quarter charges and ordinaries do not redefine the field used
for counterchanging. A bordure follows its containing outline.

`Document.surface` defines the display object in millimetres: outline, width,
height, thickness, curvature, wood covering, prepared ground, paint buildup,
brush width and direction, gilding method, and glaze. Ground attenuates the
covering's relief; seeded brushwork affects pigment and roughness. Leaf has
uniform conductor reflectance, independent of the tincture palette. Application
and burnishing affect relief and reflections. There is no sheet color grid.
Features smaller than a pixel footprint are attenuated. The finish is intact;
there is no damage or aging layer.

`surface.gold` and `surface.silver` select `MetalFinish` using a `technique`
tag:

| Technique | Additional JSON fields | Material behavior |
| --- | --- | --- |
| `Pigment` | None | Yellow or white paint from the selected recipe or custom appearance. |
| `WaterGilding` | `"burnish": 0.85` | Leaf on prepared bole; polishing smooths relief and sharpens reflections. |
| `OilGilding` | None | Unburnished oil-adhered leaf with fine adhesive texture. |
| `MordantGilding` | `"relief": 0.015` | Raised wax/resin adhesive in millimetres, carrying the leaf. |
| `YellowGlazedSilver` | `"depth": 0.65` | Oil-adhered silver with a yellow glaze, available for Or only. |

For example, `"gold": {"technique": "WaterGilding", "burnish": 0.85}`.
Burnishing and glaze depth are dimensionless values from zero to one. Mordant
relief is bounded to 0–0.1 mm. The default eagle uses water gilding; the German
lion study uses pigment on a broad 450 × 450 mm shield, and the Dürer study
uses raised mordant work. These are
authored material interpretations,
not evidence about the original prints' colors or fabrication.

Yellow-glazed silver combines RGB absorption with a neutral dielectric coating
masked to Or. `glaze_roughness` controls that coating; `glaze` adds an optional
clear coating over the whole face and defaults to zero. This conventional
metallic/roughness model approximates the documented layer structure. It does
not simulate spectral pigments, coating chemistry, or angle-dependent light
paths through a colored glaze. See the [source limits](REFERENCES.md).

Lion modeling uses separately classified shadow and highlight shapes with
partial paint coverage. It changes pigment color independently of illumination.
Over leaf, those marks add a thin pigment layer and reduce the exposed metallic
response while preserving the underlying relief. Their positions follow the
modern SVG; the manuscript attests tonal modeling, not these exact marks or
mixture strengths. `drawing.painted_modeling.shadows` and
`drawing.painted_modeling.highlights` independently control paint coverage from
zero to one. The modeled preset uses 0.30 and 0.20 respectively; setting both
to zero gives flat paint. These controls do not change anatomy or line work.
`drawing.detail` controls supplemental eagle line work.

`Document.view` owns orientation, light angle, exposure, and zoom. It is
excluded from the bake stamp. A change to identity, drawing, surface, or
resolution changes that stamp. Exports reject a bake from another document.

Validation bounds recursion, counts, dimensions, numerical values, and input
size. Unknown JSON fields and unsupported enum values fail explicitly. Tincture
contrast and clipping advice do not block valid historical exceptions.

The seven tinctures are Or (gold/yellow), Argent (silver/white), Gules (red),
Azure (blue), Sable (black), Vert (green), and Purpure (purple). Each selects a
sourced catalog recipe or measured stock mixtures under **Paint recipes**.
The catalog records pigments, binders and source evidence, including probable
egg tempera on German shields and comparative recipes from Cennini. Recipe
choices determine estimated color and roughness; they do not compute pigment
optics. The [measured mixer](references/MEASURED_PAINT.md) adds ingredient
sliders, a reachable-color picker, and deterministic least-cost recipe search
for a limited gum-Arabic/parchment calibration. Prices are editable scenarios.
Metal leaf retains its separate conductor response. See the
[recipe catalog and limitations](references/PAINT_RECIPES.md).

Future game integration must account for the material and labor costs of
commissioning, maintaining, and repairing decorated equipment, so practical
finishes can emerge from player choices. See the
[gameplay integration requirements](GAMEPLAY_INTEGRATION.md), which separate
painted modeling from physical relief and identify what still needs economic
calibration. These requirements are not yet implemented by the generator.

## Output contract

- `arms.json` is the complete editable source; `arms.svg` is the vector artwork.
- `paint-recipes.json` records palette ingredients, binders, sources and
  appearance estimates. The GLB also carries this information in its extras.
- Lion artwork uses CC BY-SA 3.0. Its source credit and modification notice
  accompany the bundle in `ATTRIBUTION.txt`, SVG metadata and GLB copyright.
- `flat.png` contains unlit tinctures and painted modeling. `base-color.png`
  contains pigment color or metal reflectance, including glaze absorption,
  without lighting. Both use sRGB and straight alpha.
- `normal.png` uses linear RGB and tangent-space positive Y pointing up the
  artwork. The GLB and studio carry explicit tangent handedness for this basis.
- `orm.png` packs linear occlusion, perceptual roughness, and metallic into
  R, G, and B. Occlusion is one: cavity shadows are not invented in the albedo.
- `coat.png` packs linear clearcoat coverage and coating roughness into R and
  G, with B zero. The studio and GLB use both channels with factors of one.
- `height.png` is unsigned 16-bit microrelief. `material.json` gives its offset
  and scale in millimetres. Support curvature is in the mesh, not this map.
- `mips/` contains the complete chain. Color filtering uses linear light and
  alpha weighting; normal filtering renormalizes vectors.
- `display.glb` embeds the maps, closed curved geometry in metres, a wood back
  and edge, and `KHR_materials_clearcoat`. It is a display support without
  straps, grips, collision behavior, or combat properties.

Maps use a square UV chart, stretched to the object's declared width and height.
Use those physical dimensions when displaying a flat image. SVG is a normalized
1000-unit chart. Material files describe the same construction across draft
(128), preview (512), high (1024), and final (2048) resolutions.

The [reference guide](REFERENCES.md) distinguishes historical evidence from the
generator's authored choices. Rebuild the clean/painterly recipe gallery with:

```sh
cargo run -p adventuresim-heraldry --example reference_gallery
cargo test -p adventuresim-heraldry
```

Generated galleries belong in `target/heraldry/`, outside source control.

# Authoring museum-derived armor

These controls describe reusable constructions for museum-derived outfits.
Choose the plate construction first, then fit its proportions to the reference
body. A recipe's existence does not establish that an outfit reproduces a
particular museum display; compare the generated assembly with the source
photographs and check the exported meshes separately.

## Recipes and anatomical sides

The creator's `--armor-designs` file contains `defaults` and `placements` maps.
Each item has a shared default; a placement override contains a complete recipe
for `left`, `right`, or `worn`. Left and right refer to the wearer's anatomy.
Use independent pauldron recipes when the reference's shoulder defenses differ.
The front and rear controls retain their anatomical meaning on either side.

Generate an editable document with `--write-armor-designs`, then load it with
`--armor-designs`. The embedded `assets_src/equipment/armor-designs.json` is the
authored catalog, not a saved override document. Breastplate and vambrace
designs remain separate files, selected by `--breastplate-design` and
`--bracer-design`. See the [creator usage
guide](../../../adventuresim-character-creator/README.md) for saving, export,
material finishing and validation commands.

Distances are in millimetres, ratios in permille, and angular controls
explicitly named below are in milliradians. Missing required fields and unknown
controls are rejected. Start from current defaults rather than an older recipe
schema.

## Torso and waist constructions

`BreastplateDesign.construction` selects `Solid` or `Anime`. The latter makes
horizontal overlapping front and rear plates beneath a larger upper plate.
`lame_count` controls the lower courses (3–10); `articulated_height` controls
the fraction of the shortest torso column they occupy (450–950). `overlap` and
`lap_lift` control axial overlap and outward edge separation. The lift must be
at least twice the metal gauge. `chevron_slope` and `rear_chevron_slope` shape
the front and rear course boundaries independently. These are modeled courses;
their internal leather connections and sliding rivets are not simulated.

The existing breastplate profile controls its silhouette independently of the
course pattern. `back_depth` adjusts the rear shell while retaining wearer
enclosure. Fit the torso's length, waist and side returns before tuning flutes
or edge ornament.

The `tassets` item's `WaistAssembly` contains independently designed `fauld` and
`tassets` components. The fauld supports the tasset tops. Its `waist_rise`,
`front_arch`, `front_arch_width` and `chevron_slope` shape its suspension height
and central opening. A larger arch opens the front without shortening the rear
skirt. Keep the fauld and breastplate hem coherent as an assembly.

For long thigh defenses, choose `GarmentPlateShape.WrappedTassets` rather than
stretching a flat front panel. Its controls are:

| Control | Meaning |
| --- | --- |
| `inner_wrap`, `outer_wrap` | Independent fractions of a half circumference around the thigh. |
| `inner_gap` | Minimum distance of each inner edge from the body midline, 0–80 mm; the complete gap is twice this value. |
| `upper_edge_slope` | Rise of the suspension edge away from the midline. |
| `knee_reach` | Fraction of hip-to-knee distance reached by the hem, 650–1100. |
| `inner_cutaway` | Additional opening at the upper inner corner. |
| `hem_rounding` | Shape of the lower corners. |
| `section_break` | Number of upper courses before the lower section; zero leaves the section unsplit. |
| `section_gap` | Axial separation of that lower section. |

The enclosing garment recipe sets the number of courses, gauge and fluting. The
section break must fall before its final course. The open rear and the medial
cut are construction boundaries. If the requested inner gap leaves no valid
region on the fitted thigh carrier, generation fails; it does not widen or
collapse the plate to make the recipe fit. A section break does not create a
runtime action for detaching armor.

## Shoulder, elbow and knee plates

Full pauldrons use a common formed shoulder carrier for the main plate and upper
lames, with narrower courses below the shoulder. The `outline` controls the
boundaries independently of crown fullness:

| Control | Meaning |
| --- | --- |
| `front_return`, `rear_return` | Angular reach of each hanging main wing, 1800–2800 mrad. |
| `front_extension`, `rear_extension` | Extra downward extent of the main wings, independent of the lower arm courses. |
| `wing_start` | Angle at which the hanging wing begins descending. |
| `corner_rounding` | Rounding of the angular return into the main wing. |
| `front_wing_position`, `rear_wing_position` | Center of the lowest hanging region along the arm-to-neck span (250–750), independently of angular coverage. |
| `front_wing_rounding`, `rear_wing_rounding` | Rounded approaches to that region (100–1000). Higher values narrow the low interval; 1000 gives a continuously rounded point. Depth remains the authored extension. |
| `upper_span` | Fraction of the shoulder occupied by upper lames. |
| `arm_wrap` | Half-angle covered by the narrower arm lames. |

Use the main design's front/rear reach and drop for the broader saddle shape.
`plate_clearance` reserves space over the supporting torso plates;
`arm_allowance` reserves space for the rerebrace. Neither control substitutes
for the other. The common carrier preserves the relationship between the main
plate and upper courses; it is not an articulated mechanical solver.

Spaulders have `crown_coverage` (200–1000), which trims the crown's covered
arc without squeezing its formed bulk. At 1000 the crown reaches its closed
apex; smaller values expose a trimmed rim. `wrap` controls side coverage
separately. An optional `besagew` adds a separate circular armpit defense. Its
radius, boss, shoulder drop, medial offset, tilt and torso-plate clearance are
independent. `RadialFluting` governs spokes around the disc, with 4–48 flutes
and independent width, depth, radial start/end and fade. The disc follows its
torso suspension instead of inheriting arm skinning. Shoulder retention bands
follow the attached plates and inner arm assembly behind this suspended disc;
the disc does not define the band's load-bearing envelope.

Poleyns and couters select `Wrapped` or `RaisedCop` construction. The wrapped
plate has axial upper and lower edges; the raised cop has a rounded perimeter
around an enclosing dish and returned lateral fan. Its `wing` can spread to
2000 permille without increasing the fan's distance in front of the arm;
wrapped plates retain a 650-permille limit. `dome` raises the dish without
moving its perimeter. `proximal_flare` and `distal_wing_scale` accompany
the cup, wing and notch controls. Raised cops have independent `medial_wrap`
and `lateral_wrap` return angles, expressed in permille of a half-turn. Their
defaults, 520 and 850, retain a deeply returned elbow fan; a knee plate can use
a shorter lateral return. `flute_orientation` selects `Longitudinal` or
`Transverse` relief on either construction. The existing flute spread controls
fan the ridges in that direction without changing the unfluted carrier.
On a wrapped plate, `distal_extension` adds
1–4 closed lower plates. `length` controls total reach (40–180 mm),
`terminal_share` assigns 350–850 permille to the lowest plate when there are
multiple courses, and `distal_taper` varies the lower girth (700–1100). Taper
retains the attachment at the cup; `wrap` controls the angular extent.
`hem_rounding` must remain short enough for the terminal course's longitudinal
sections to stay monotone. The main cop retains a separate `Plate` component
and follows the anatomical lower-arm or lower-leg joint rigidly. Extension
plates preserve their independent component and existing joint-region skinning;
there is no separate sliding-joint animation system.

## Helmet constructions

Use a burgonet for an open crown with peak, cheek plates and nape defense. Its
optional `buffe` is a separate face-defense mesh beneath the peak, rather than a
reshaped close-helmet visor. The burgonet's `peak_rise` (0–20 mm) raises the
front of the peak; the buffe's sight edge follows it. Cheek depth, taper and
chin tab affect the space that remains for the face defense.

The buffe has independent sight gap, face projection, chin width, throat depth,
neck drop and side return. `medial_ridge` controls ridge height;
`ridge_sharpness` changes its rounded-to-angular cross-section. `chin_point`
shapes the lower outline. A narrow requested chin cannot remove the clearance
needed around long cheek plates. The mesh is separate, but a detachable or
falling-buffe animation is not implemented.

Optional `buffe.courses` forms two or three individually closed overlapping
faceplates within the separate Buffe component. `plate_count` selects their
number. `lower_boundary` (200–700) and `upper_boundary` (450–850) place the
internal seams upward from chin to sight edge; the upper boundary applies to
three plates. Keep at least 180 between those boundaries. `overlap` (2–12 mm)
extends the covered lower edges. `lap_clearance` (0–4 mm) adds space beyond
twice the plate gauge, and `boundary_drop` (0–25 mm) gives the seams a median
chevron. Each upper edge retains the original fitted carrier, so extra courses
do not cumulatively widen the sight rim. On this formed shape,
`ridge_sharpness` blends toward tangent face flats around the median ridge.

Optional `buffe.breaths` uses the shared `VisorBreaths` slot pattern. Set it to
`null` for an unpierced face defense. Count per row, rows, width, length, span,
row spacing, lateral offset, rounding, inclination and anatomical side are
independent controls. Buffe `height` runs downward from the sight edge
(50–950); the upper-row starting recipe uses 160, with vertical slots.
Close-helmet breaths retain their existing 450–850 interval. The slot
dimensions belong to the authored plate chart; body fitting changes its
proportions without changing the slot topology. Piercing creates closed metal
walls around every opening and rejects overlapping slots or cuts that leave
insufficient metal at the rim. With multiple faceplates, the pattern pierces
only the upper course; its slots must fit inside that course's edges.

A close helmet can instead enable `bellows`, which forms 1–5 horizontal folds.
Depth, sharpness and the folded interval are independent; `cheek_rise` sweeps
that face upward toward the hinges. Existing pierced visor openings remain part
of the surface, and decorative longitudinal fluting stays a separate option.
Solid helmet components retain rigid head attachment and their separate
mesh/hinge metadata.

Its `brow_overlap` (8–40 mm) and `brow_peak` (0–30 mm) shape the upper visor
edge independently of the sight opening. Use a small peak for a narrow forehead
band; the eye slit remains aligned with the fitted anatomical brow.

Tasset suspenders bind to the nearest physical plate triangle. The same
barycentric attachment drives their skin weights and shape targets, so a nearby
plate with a different attachment does not pull the leather through its support.
Opposite leather walls share one attachment, preserving their gauge. This does
not add mechanical articulation; reference geometry and texture coordinates
remain independent of the attachment binding.

## Reproducible comparison views

The `henry/` and `nuremberg/` directories contain the selected outfit, body,
plate, fastening, metal and trim recipes. Henry has front, quarter and rear
camera presets; Nuremberg has the museum's quarter view. These are authored
interpretations of the displays described under Primary references below.
Henry's camera presets also set the unlit ochre colors of the boots and cloth.
They do not reproduce the reference's engraving or soft-material wrinkles.

Generate equipment with the selected directory's `body.json` as `--recipe`,
`armor.json` as `--armor-designs`, `breastplate.json` as
`--breastplate-design`, `vambrace.json` as `--bracer-design`, and
`fasteners.json` as `--fastener-designs`. Restrict `--equipment-item` to the
unique item IDs in the body's `clothing` array. Export the body separately
using a temporary copy of that recipe with an empty `clothing` array and
`--export-only --glb OUTPUT/body.glb`; this avoids drawing its armor twice.

Keep the raw equipment as a validation source. Unwrap a copy with
`scripts/unwrap_armor.py`, bake it with `scripts/bake_armor.py`, set metal
factors with `scripts/color_armor.py` and the selected `metal.json`, then apply
`scripts/trim_armor.py --recipe` with the selected `trim.json`. Place the
finished equipment and its referenced PNGs in `OUTPUT/equipment/`. The UV,
bake and trim checkers compare each stage with its immediate predecessor.
Use `--source-root OUTPUT` when rendering one of the selected camera presets.

`scripts/render_museum_armor.py` renders a saved camera and item list from
actual exported GLBs or review JSON:

```sh
blender --background --python-exit-code 1 \
  --python scripts/render_museum_armor.py -- PRESET.json OUTPUT_DIRECTORY
```

Pass `--source-root` to resolve item and skeleton paths against a generated
asset directory while keeping the camera preset with its source recipes.

Preset paths are relative to the preset file. Camera positions and look-at
points use game coordinates: Y up, Z forward, in metres. Each item names its
input path; include the body explicitly. Choose exactly one camera projection:
`orthographic_scale` for parallel comparisons, or `focal_length_mm` for a
perspective photograph with a 36 mm horizontal sensor. Set camera position and
lens together when matching a photograph; orthographic views suppress the
depth cues visible on feet and shoulder tops.
Save separate presets when matching photographs taken from different angles.

Match the photograph's portrait bounds and aspect ratio before adjusting the
lens. Check the helmet crest, soles, shoulder asymmetry, elbows and fingertips
in image space. Pose the arms to approximate the displayed mannequin; hands
that disappear inside thigh plates obscure the assembly and invalidate that
comparison. Derive rotations from the current body's joints when its skeletal
proportions change. This display pose is separate from unposed clearance and
morph validation.

[SAM 3D Body](https://github.com/facebookresearch/sam-3d-body) estimates MHR
body and hand poses from photographs and can provide an initial fit. Its model
checkpoints require access through the upstream installation workflow. Treat
an estimate through armor as a starting point: verify visible landmarks against
the photograph before accepting the comparison pose or body proportions.

GLB inputs retain their material textures. Optional material overrides replace
only the supplied scalar/color channels, so a base-color override also replaces
the visible trim texture in that channel. Omit it when evaluating a finished
gilt atlas. Renderer colors are linear RGBA; trim recipe colors are sRGB hex.
JSON inputs have scalar materials and cannot demonstrate baked normal or AO
maps. A static comparison render does not establish clearance or morph validity.

To color the metal itself, run `scripts/color_armor.py EQUIPMENT MATERIAL.json`
after baking and before trimming. Its material recipe supplies `color` as an
sRGB hex value, `metallic` in 0.8–1 and `roughness` in 0–1. The metallic range
preserves the existing finishing pipeline's metal classification. For example:

```json
{"color": "#5A6165", "metallic": 1, "roughness": 0.43}
```

This sets unlit material factors on the existing metal primitives, including
metal fasteners. Nonmetal materials and body-derived mail/padding retain their
own appearance. It preserves the geometry, skinning, UVs, normal and occlusion
maps. Apply gold borders afterward with the ordinary trim recipe; highlights
and occlusion remain lighting and map effects, never painted into the palette.
Recolor a fresh export when changing an already trimmed material.

Validate equipment against a body GLB exported from the same character recipe.
The clearance scripts' `neutral` configuration uses absolute zero skeletal
proportions. For a museum body with custom proportions, evaluate its explicit
reference proportions through the existing clearance checker as well. The
identity-only `export_armor_morph_review.py` script uses the repository's base
body, so it must not stand in for a matching custom-body assembly check.

For installed-asset runtime validation, the native animation viewer supports
`--armor-harness museum-henry` and `--armor-harness museum-nuremberg`. Point
`--asset-root` at an isolated asset directory containing the corresponding
finished equipment and matching character body. Pass the recipe's nine skeletal
coefficients as a separate JSON array through `--body-proportions`.

The burgonet and close helmet reserve their head/face mounting locations;
independent gorgets reserve the neck. Their visible neck coverage and combat
protection are separate catalog fields, so a helmet's tail does not prevent
wearing a collar.

These fixtures retain gameplay's deterministic character-ID identity variation;
the capture manifest labels that variation. They exercise the installed meshes,
materials, morphs and skins on a varied body. Use the ordinary source GLBs and
museum comparison camera above for exact reference-body comparisons.

## Primary references and interpretation

The following museum records provide original photographs and object metadata.
The cited constructions motivate the controls above; they are not promises that
every combination of parameters is historical.

- [Henry VIII's field armor, ca. 1544, Met 32.130.7a–l](https://www.metmuseum.org/art/collection/search/23936)
  supplies the anime torso, long thigh defenses and asymmetric shoulder
  reference. The Met describes overlapping torso plates joined by rivets and
  internal leather straps. It records an original detachable breast reinforce
  and left-pauldron reinforce; the displayed assembly must not be treated as
  the complete original configuration. Its ornamental figures and foliage
  require more than the available edge patterns and straight material bands.
- [Nuremberg field armor, ca. 1525, Met 04.3.289](https://www.metmuseum.org/art/collection/search/22001)
  supplies the fluted torso, shoulder/disc arrangement and folded visor
  reference. The display combines parts from at least three armors. Its left
  arm is a nineteenth-century restoration, and its rondels date to 1923.
  Matching this display does not establish that those replacement elements
  reproduce an original sixteenth-century assembly.
- [Italian pauldrons and arm defenses, ca. 1560, Met 14.25.827a–d](https://www.metmuseum.org/art/collection/search/22301)
  provide an additional source for the relationship between broad shoulder
  plates and narrower descending arm courses.
- [Blair and Pyhrr's study of Henry VIII's armor](https://resources.metmuseum.org/resources/metpublications/pdf/Wilton_Montmorency_Armor_Italian_Armor_for_Henry_VIII_The_Metropolitan_Museum_Journal_v_38_2003.pdf)
  provides the detailed construction reference for that harness.

Preserve camera angle, body proportions and material choices alongside each
outfit recipe when comparing it with a particular photograph. Keep renders,
working recipes and review results in ignored output directories until they are
promoted deliberately as reusable source assets. Crests, figurative engraving,
custom curved ornament paths and physical plate articulation require additional
systems; the controls above do not approximate them automatically.

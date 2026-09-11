# Armor model

This crate generates deterministic armor geometry from anatomical body samples.
`generate_bracer` fits forearm contours. `generate_breastplate` builds separate
front and rear carriers, fits their clearance to the wearer, closes each shell,
and transfers anatomical UVs, skin weights, and morph correspondence.

The rest of the armor catalog uses authored parametric surfaces through
`generate_helmet`, `generate_limb_armor`, and `generate_garment_armor`. Their
`PartFrame` inputs contain anatomical placement and physical dimensions, not
body triangles. The character creator measures those frames and fits independent
carrier surfaces to body cross sections. `PartMesh` adds physical inner walls
and boundary returns, preserves outward winding under reflection, and retains
carrier correspondence when a body morph refits the surface.

Construction determines the recipe family. A barbute has a continuous bowl and
cheek opening; a burgonet has a separate peak, cheek plates and neck defense.
They share geometric helpers, but have separate designs because changing a few
bowl dimensions cannot express those structural differences. Morions, kettle
hats, sallets and close helmets likewise expose their own brim, tail, visor,
crest, opening and lower-edge controls. Cap and coif recipes have soft covering
boundaries. Clearance and gauge remain separate from style controls.

Long limb plates expose length, taper and wrap. Elbow and knee cops expose
localized projection and wing dimensions; shoulder, finger and foot defenses
have overlapping lame controls. Gauntlets fit the hand and thumb independently.
Textile and mail envelopes use garment patterns with neck, arm and hem openings,
plus joint-following sleeves and leg coverings. They represent the garment's
volume; individual mail rings and closures are outside this geometry layer.

Metal limb and garment plates, vambraces, helmet crowns and close-helmet visors
share `PlateFluting`. Set the recipe's optional fluting field to `None` (`null`
in JSON) for a plain plate. Count, width as a fraction of pitch, physical relief
depth, pattern spread, lower spread, start/end position and fade length are
independent. More flutes at the same spread produce narrower flutes; changing
width changes the balance of raised metal and intervening smooth land. Width
scales with the wearer, while depth and gauge retain their millimetre values.
Breastplates use the same flute parameters with a torso-specific distribution.

Each construction supplies a surface chart for its relief. The generator fits
the smooth carrier to anatomy, then applies relief to both shell surfaces and
closes their boundaries. Gauge follows the carrier's extrusion direction; it
does not follow the steep local flute slopes. A body morph retains the selected
design's vertex and triangle correspondence. Changing a design, including its
flute count or pattern, may rebuild topology and requires new morph targets.

Boundary controls are specific to the construction: sallets expose opening
width/sweep and visor side-panel depth, greaves expose ankle extension, and
joint cops expose wing notches. Tassets have independent inner cutaway,
roundness and point controls; gorgets expose collar height and slope, separate
front and rear hem flatness, rear sweep, independent front/rear bib depth and
width, and neck clearance. Gorget fluting occupies the front bib; the shoulder
return remains plain.
The canonical gorget uses [DIA 53.202.2](https://dia.org/collection/gorget/110870)
for its shape. Its 1 mm nominal wall is supported by the measured
[German gorget A-25, about 1550](https://www.allenantiques.com/A-25.html);
the DIA object record does not publish wall thickness. The creator fits these
authored outlines to anatomical sections. Export checks measure intersections
and sampled clearance across body morphs.

Use the character creator's `--write-armor-designs` command to obtain the
current catalog recipe schema. Vambrace and breastplate designs load from their
separate `--bracer-design` and `--breastplate-design` files. See the
[creator authoring guide](../adventuresim-character-creator/README.md#parametric-armor-authoring)
for editing, saving and exporting all three files. Required fields must be
present; obsolete recipe schemas are not accepted.

Every catalog armor item must resolve to an authored recipe. The character
creator rejects an unmapped armor item instead of sending it through the
body-topology clothing shell path. Ordinary non-armor clothing still uses its
separate clothing generator.

The breastplate implementation lives in `src/breastplate_carrier.rs` and its
child modules:

- `wearer` measures the anatomical frame and fit scales.
- `shape` evaluates the authored carrier profiles.
- `grid` connects the plate, shoulder, and skirt surfaces.
- `fit` resolves body clearance and the gap between plates.
- `shell` adds thickness and validates welded closure.
- `sampling` transfers body surface attributes.
- `math` owns shared vector and interpolation operations.

The former fixed-surface implementation, its topology API, superseded tests, and
target-profile experiment have been removed. The paired-carrier regression test
covers the supported generator, including closed components, skinning, morph
correspondence, and parameter variation.

Run `cargo test -p adventuresim-armor-model` for the crate's regression tests.
Run `just fmt-check` and `just lint` for the repository quality gates.
See [armor references and validation](review/README.md) for source references
and reproducible checks.
Set `BREASTPLATE_REPORT_FIT` to emit fitting diagnostics when investigating a
body or design that fails clearance validation.

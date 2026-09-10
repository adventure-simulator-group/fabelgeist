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

The former fixed-surface implementation, its topology API, superseded tests,
and target-profile experiment have been removed. The paired-carrier regression
test covers the supported generator, including closed components, skinning,
morph correspondence, and parameter variation.

Run `cargo test -p adventuresim-armor-model` for the crate's regression tests.
Run `just fmt-check` and `just lint` for the repository quality gates.
See [armor references and validation](review/README.md) for source references
and reproducible checks.
Set `BREASTPLATE_REPORT_FIT` to emit fitting diagnostics when investigating a
body or design that fails clearance validation.

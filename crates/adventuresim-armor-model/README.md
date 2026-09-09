# Armor model

This crate generates deterministic armor geometry from anatomical body samples.
`generate_bracer` fits forearm contours. `generate_breastplate` builds separate
front and rear carriers, fits their clearance to the wearer, closes each shell,
and transfers anatomical UVs, skin weights, and morph correspondence.

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
Set `BREASTPLATE_REPORT_FIT` to emit fitting diagnostics when investigating a
body or design that fails clearance validation.

# Selected PBR graph Code Review

One actionable migration gap remains: the shared textile example exporter
still publishes source components into the native Python review protocol.
The two early syntax defects are corrected in the actual current source.
This is not selected-family clearance or final PR clearance.

Reviewed HEAD is `d000e848f2f5bc28b0728c2cd664c727a41ecf50` plus the exact
74 current paths in `material_pbr_final_preinventory.current_files`. All
74 captured SHA-256 fingerprints match that ledger. The adjacent JSON records
all hashes, the baseline, supplementary dependencies and precise read extents.
The before source is `target/pr780-audit/material-pbr-before-d000e848f/source`;
the separately ledgered original `armor_metal` and `main` accounts are retained.

## PBR-CODE-1: project the textile examples into the native component protocol

**P2, incomplete producer propagation.**
`crates/adventuresim-character-creator/examples/textile_review/mod.rs:70`
still writes `"components": generated.components`. The newly selected source
schema serializes `appearance`, while the unchanged native receiver
`scripts/render_armor_review.py:73-75` reads `material.base_color` and uses gray
when the material field is absent.

This is an actual connected caller, not a speculative artifact consumer:
`examples/skirt_review.rs:22` explicitly documents that renderer, and
`:116-121` calls the shared writer using `--worn` filenames that match the
renderer’s `*--*.json` selection. Before the enum migration,
`device_textile_materials::apply` set `component.material` from the fabric;
that field was then serialized into this native artifact. A skirt with an
authored textile color now reaches the gray fallback. The main binary's new
`ReviewComponent` projection does not cover this second producer.

Move the existing native component view and checked conversion to one shared
Creator library owner, then consume it from both the binary and this example
writer. Reuse `LinearSurfaceMaterial::try_from` for the scalar projection.
Preserve role, ranges, hinge and mount metadata and component order. Do not
add a Python conversion formula, a source-schema compatibility parser, a
second native schema, or new fixture logic. The governing source rules require
clean final APIs and complete producer propagation
(`AGENTS.md`, architecture/backward-compatibility rule;
`crates/AGENTS.md`, semantic values and types).

A complete pre-repair thirteen-lens extension, dependencies and acceptance
criteria are in `/tmp/pr780-review-component-all13-discovery.md` and its JSON.
Exact connected before sources are archived under
`/tmp/pr780-review-component-extension-before/source`.

## Reviewed contracts and dispositions

- `Srgb` and `Srgba` retain arbitrary encoded float channels with transparent
  serde, and alpha bypasses RGB transfer. They do not falsely claim numerical
  admission. The existing canonical sRGB transfer is reused.
- Armor `SurfaceMaterial` retains named encoded color and canonical ratio
  leaves. Recipe validators for Metal, Lacing and MailWeave retain their
  existing bounds and default values. All six early malformed field
  separators are corrected.
- Required `ComponentAppearance` owns Inherit, scalar Surface or mutable
  Textile data. Deriving textile material from the current recipe removes the
  previous independently stored scalar copy. Fastener/eyelet geometry choices,
  component literal metadata, endpoint cloning, seam copying, relief mutation
  and support appearance routing were inspected through actual source.
- Garment specifications, catalog defaults, rigged shells, compacted shells,
  component overrides, hair roughness and both actual hair receivers carry
  the selected owners to their native boundary. Compaction preserves the sole
  UV column and scalar appearance. `surfaces()` confirms component order for
  `component_materials` rather than relying on a zip alone.
- Bevy scalar projection uses encoded `Color::srgba`; its alpha, material
  defaults and caller-specific texture/roughness/face overrides remain intact.
  The pinned local Bevy source confirms `Color::srgb` is the alpha-one Srgba
  representation. The pinned egui source confirms its float RGB picker is
  linear, matching the new explicit picker transfer boundary.
- `LinearSurfaceMaterial` has private fields and no unchecked construction or
  deserialization route. Its checked conversion validates decoded output RGB,
  unchanged alpha, metallic and roughness. Writer propagates that typed error
  through anyhow Context before artifact publication. Source input finiteness
  remains a separate validation guard.
- Base-color PNG presence selects white before checked linear conversion;
  normal-only textures retain authored factors. This preserves the previous
  effective factor behavior, including finite source colors shadowed by a
  base-color map, without widening the source finite-input policy. Existing
  map alpha, sides, sampling, channels and final roughness-map override remain.
- Body GLB and both body preview paths use the public skin owner. The binary
  review adapter preserves actual metadata and emits the same checked linear
  native factor record used by GLB. The example omission above is the remaining
  actionable path.
- Existing fixture diffs were read as API adaptations. No cases, helpers,
  assertions or doctests were added. Existing assertions retain their values
  and meaning. The `skin_rules` helper now has the proper original structure
  with only the required enum-field API difference.

The directly authored component JSON in `cloth_panel_review.rs` and
`prepared_panel_review.rs` was read completely. Those unchanged native
examples already supply `material` factors and do not serialize
`ArmorComponent` or implement an RGB transfer. They are distinct from the
missed source-component projection and are not declared new source-schema
regressions. The role/range-only bake receiver was also read.

## Explicit limits

UV-R1 remains open: typed export admission, body finite geometry and native
assembly error handling. UV-R2 remains open: checked GLB byte layout,
allocation and native length conversion. Their source-backed inventory is
`/tmp/pr780-export-admission-glb-all13-discovery.md`; this PBR review does not
clear them. The unchanged production expects in draped-mail preview also
remain outside this selected scalar migration, as do broader material,
scalp/Grid/generated-program and GPU composition families.

All 74 selected paths have explicit coverage in the JSON. Changed material
responsibilities and actual receiving bodies were read; unrelated unchanged
geometry algorithms and the complete dependency lock graph were not assigned
clearance. No local project build, test, formatter, lint, quality tool,
scanner, parser probe or hosted CI was run. This reviewer edited no source or
fixture. A repair review and final independent reviews must follow on the
actual final revision.

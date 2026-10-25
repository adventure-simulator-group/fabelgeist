# Material fixture and encoded-color frontier

Read-only discovery before the material type repair. No source, tests, fixture
cases, helpers, assertions or doctests were authored or changed. No fixture,
project check, build, scanner, probe or hosted CI was executed. A source read is
not a successful test run or clearance for the complete material graph.

Worktree:
`/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator`.
Exact source fingerprints and read extents are appended below. All four requested
fixture files were read completely, including existing helpers and every oracle.
Canonical transfer.rs was read completely; egui/ecolor reads are bounded source
regions listed in the accounts. Both workspace and Creator lockfiles identify
egui/ecolor0.34.3.

## Existing fixture API and oracle preservation

### Creator `tests/textile_export.rs` — complete 110 lines, one test

`portable_sheer_cloth_blends_once_and_retains_opaque_pattern_pixels` builds a
TextileAppearance with encoded byte colors `[240,235,220]` and `[190,150,55]`,
opacity Permille200 and an Along band from750 to1000. Its RiggedShell is built
from `fabric.material()`'s current base_color/metallic/roughness fields, with
textile finish, surface=None and textured UVs. A metre triangle and one identity
joint are actual portable-file fixture inputs; they are not new semantic owners.

Preserve the exact existing observations: glTF Blend alpha, single-sided faces,
baseColorFactor `[1;4]` after texture application, embedded image ownership,
pixel `[240,235,220,51]` in the sheer field and `[190,150,55,255]` in the opaque
ornament band. A shared PBR record or RiggedShell field change must adapt this
existing construction without a parallel tuple API, new helper or changed pixel
oracle. It must not multiply opacity twice or linearize the PNG byte payload.
No Metal/Trim/Relief/Ornament count is constructed in this file.

### Creator `export/textures/tests.rs` — complete 205 lines, two tests

Existing `maps` returns SurfaceTextures with embedded fixture PNG bytes, Mask,
Both sides and Tile sampling. Existing `export_equipment_with_maps` takes raw
PBR factors, then fills one RiggedShell; this is an existing fixture helper to
adapt if the PBR API changes, not permission to add another helper.

`normal_only_material_preserves_its_authored_metal_properties` uses encoded
factor `[.25,.5,.75,1]`, metallic.8, roughness.2, no base-color/occlusion map,
Opaque and WrappedGarment. Preserve no color texture, nonwhite color factor,
exact.8/.2 factors, one normal image, Repeat-S/Clamp-T and sibling PNG ownership.
It proves preservation of authored factors in this normal-only route, not exact
round-trip values for every encoded color.

`separate_equipment_glbs_share_exact_surface_files_and_material_roles` exports
same maps twice. Preserve three exact PNG payloads/content-addressed sibling
URIs, image MIME/path requirements, stable shared names, color/normal/AO texture
indices0/1/2, Mask alpha, small mesh-only GLB, exactly five resulting directory
entries, and the existing corrupted-content-addressed-file rejection. The error
oracle matches `unexpected contents`; no new corruption case is authorized.
No Metal/Trim/Relief/motif-count fixture construction is present here.

### Tactical `equipment/runtime_equipment/mesh_tests.rs` — complete 141 lines

One existing test constructs red and gold ArmorComponentMaterial records with
encoded colors `[1,0,0,1]` and `[1,.8,0,1]`, metallic0 and roughness.9/.8.
These are actual external component-PBR constructors. A shared owner migration
must update them directly, preserving color alpha and their distinct factors.

The existing geometry oracle preserves two partitions, three compacted vertices
per partition, indices `[0,1,2]`, UV `[.25,.75]`, strongest-four joint indices
`[2,0,0,0]`, referenced center `(4,3,0)` excluding the unused far vertex, and
correct dropped relative positions `(-3,-1,0)` / `(1,-1,0)` after skin attributes
are removed. Base colors must still equal the exact corresponding Bevy
Color::srgba values. This test does not assert the roughness factors, and that
coverage gap is not authorization to add assertions. No material-bake, trim,
relief or motif-count constructor appears in this fixture.

### Armor `tiling/tests.rs` — complete 192 lines, six tests

The existing sleeve helper constructs a cylindrical SurfaceGrid using radius.05,
height.25,11 rows,48 columns; it optionally reverses column order and attaches
skin and a.01m translation morph. Preserve its original fixture and helpers.

- Solid construction returns the exact original piece, no lacing.
- Scale construction for both existing column orders retains nonempty finite
  geometry, valid indices, normalized skin weights, complete morph arrays and
  outward outer-face normals with the original.8 dot threshold.
- Lamellar plate positions remain within the original thickness/height bounds.
- Removing optional lacing retains no lacing, while original laced cords cover
  the prescribed row region; existing design hashes differ.
- Both existing invalid grid scenarios (cleared grids and a1x4 regular grid)
  return exactly ConstructionError::NoSurfaceGrid.
- The final test uses `Construction::Scale(Tiling::scale())`,
  `construction.trim_on(&Trim::default())`, and
  `TrimBand {width:trim.width,period:trim.period()}`. After trimming it preserves
  both positive band and positive face outer-triangle populations. Typed length
  and named trim-output APIs must reach this unchanged consumer.

These are the existing six scenarios. No count overflow, zero motif count,
invalid metal color or new invalid trim case is present here. Do not invent
additional cases to fill gaps. Generated vertex/triangle loop counts are not
motif repeat counts or native launch counts.

## Existing bounded-count rejection frontier

The actual ornament fixture at `ornament.rs:166-192` has four existing invalid
inputs: repeat count0; line.5; Guilloche strand count0 with amplitude.5; and Wave
amplitudeNaN. Its existing oracle rejects all four via validation. A final bounded
repeat/strand owner must retain these *same* rejection scenarios at the first
real admission boundary. Once zero cannot inhabit an admitted count, adapt the
existing invalid case to exercise that owner's fallible admission; do not keep
an unchecked public constructor or add a raw compatibility field just to build
an invalid Ornament. Do not delete either count rejection scenario, replace it
with a passing value, add a case, add a helper or multiply assertions.
The line and NaN cases remain recipe-policy validation concerns if Ratio itself
continues to admit arbitrary numerical values.

Current policy is repeats1..=8 and Guilloche strands1..=6. Suggested motif repeat
counts2/3/4/6 and default Guilloche count3 are admitted values; preserve them and
picker reset behavior. Scratch count has a different domain0..=3000 and allows
zero. Plate hole_pairs zero is also meaningful for unlaced construction; it is
not a reusable strictly-positive motif count. Native invocations/workgroups,
container lengths and tile pixel sizes are different owners.

The existing GPU fixture `gpu/metal/tests.rs:219-261` builds every canonical
Motif with fillets=false and a Beads radius.3/line.06 default-fillet recipe. It
retains deterministic repeats, original coverage.03.. .6 and original untouched
side/fillet row observations. Final fillet policy and bounded motif counts must
project the same native codes/counts/flags. This tranche read only that bounded
GPU-fixture region; the earlier discovery's whole-file read is separate evidence,
not a substitute for final review on repaired bytes.

## Float encoded-color adapter contract

The current Metal and small-plate Lacing arrays are artist-facing encoded float
RGB: actual preview consumes Color::srgb and export converts sRGB factors once.
Current Metal control (`metal_controls.rs:12`) and Lacing control
(`construction_controls.rs:73`) pass those arrays directly into egui's linear
float RGB widget. This is a transfer-space mismatch, not an invitation to
reinterpret stored recipe colors as linear.

Pinned egui0.34.3 actual source:

- `ui.rs:2275-2280` explicitly says color_edit_button_rgb expects linear RGB
  `[f32;3]`; `color_edit_button_srgb` at2271 takes `[u8;3]`.
- `color_picker.rs:538-555` constructs float Rgba, round-trips through HSVA and
  writes floats back unconditionally. `Response.changed()` comes from the actual
  popup picker; clicking/opening the popup by itself is not the commit contract.
- `color_picker.rs:259-313` has byte-gamma and float-linear numeric editing
  modes. It guards those internal conversions with actual edits. Normal widget
  rendering uses Color32, but float picker storage stays Rgba/HSVA. Selecting the
  user's byte numeric mode may itself author byte-grid values; the adapter must
  not impose an extra byte conversion on all colors or all frames.
- `ecolor/rgba.rs:3-10,80-83` owns linear f32 Rgba; `hsva.rs` converts through
  float components. A no-edit round trip is not a source-level promise of
  bit-identical arbitrary RGB inputs.

Exact adapter obligation before changing recipe types:

1. Keep the persisted typed encoded float triplet as authority. Copy its native
   channels into a local picker value, converting with the existing canonical
   `TransferFunction::Srgb.to_linear_rgb` API.
2. Pass the local linear float triplet to the pinned float picker. This is the
   framework boundary, not an internal primitive recipe API.
3. Only if the returned response reports `changed()` convert the edited local
   floats through `TransferFunction::Srgb.from_linear_rgb` and write the encoded
   value back. Do not commit the decode/HSV/encode round trip on an unchanged
   frame. This preserves the original stored floats when the user did not edit.
4. Use canonical float transfer functions; do not add another sRGB formula,
   normalize through bytes, use linear_to_srgb_u8, or wrap Color32 as recipe
   storage. Do not promise numerical bitwise identity for an actual edited color.
5. Return/use the actual response/change result so the containing recipe's
   changed/dirty logic follows user changes. Preserve alpha policy: these two
   float triplets are opaque; do not invent an alpha field or drop unrelated
   RGBA opacity factors.

Canonical `fabelgeist-color/src/transfer.rs:98-120` provides both RGB triplet
methods; its scalar sRGB functions at133-150 are already the owner. They do not
quantize or round through u8. The separate `linear_to_srgb_u8` explicitly does,
so it is an unsuitable adapter for authored f32 material colors. This review
read implementation contracts and did not independently research transfer-curve
standards, profile accuracy or run a numerical round-trip experiment.

The cloth-eyelet path is intentionally different:
`cloth_lacing_controls.rs:157-159` edits TextileColor `[u8;3]` with
`color_edit_button_srgb`. Its stored representation already matches that widget.
Do not apply the float-material adapter to it or migrate the entire TextileColor
family solely because its UI is adjacent to small-plate Lacing. Metal eyelet
variants and worked-steel Metal recipes are separate concepts.

## Acceptance and limits

Before edits, carry these construction/use accounts into the all-thirteen
inventory, including partial source dependencies. Root remains the sole source
implementer. Existing fixture API changes may be necessary, but their scenarios,
helpers and oracles are the acceptance baseline. No compatibility adapter or
additional tests are authorized. Color adapter acceptance requires the actual
changed-response guard, canonical transfer calls, opaque float storage, no new
quantization and correct recipe/UI receiving code; source-only reasoning is not
an interactive UI or hosted runtime pass.

This discovery closes the prior read limit for the four named fixture files,
not the whole export/material/component constructor graph. The egui UI file,
color_picker and ecolor files were read only at the declared relevant regions.
The complete byte/float color widget, display pipeline and entire external
numeric color style graph were not audited. Required final independent reviews
and hosted checks on repaired final bytes remain pending.

## Source snapshot

HEAD at accounting: `f2c7422fd6a0f995eb90b72cc9c2a5bd12de4d35`.

| Source | Read extent | SHA-256 |
|---|---|---|
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/adventuresim-character-creator/tests/textile_export.rs` | full110 | `3f0d6674ab64abd4e90c50836ccc7028edb39f616d8758b2c6ff25e8380d9059` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/adventuresim-character-creator/src/export/textures/tests.rs` | full205 | `43e385b5074a6c8884aabe7f27f70c016f9b16b16e19a0020d58cd01090de0b8` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/adventuresim-tactical-client/src/equipment/runtime_equipment/mesh_tests.rs` | full141 | `142460bac2304c4a63a77c9759a003d9bd57a9d5b6210b8d7954105d6ee3e827` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/fabelgeist-armor/src/tiling/tests.rs` | full192 | `166ec2be7855c465b709326152b2be1babfdcd538a062213cc6a5d8ae680d1f9` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/fabelgeist-color/src/transfer.rs` | full source | `81b5c5f4a55d8191a178ea71fc618c968db91c2b601fa7b72e0cadf466ecad62` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/fabelgeist-armor/src/material.rs` | full source | `bf177dcb1b678b06f9174b3aa283e4aa7634638681ce88c8722a900052734d72` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/fabelgeist-armor/src/ornament.rs` | 130-EOF; count/validation fixture region | `74d886336dc4fbeeb4f9ff4ec29530a32fef42865dcb7ef2f6b2220b678c1ebf` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/adventuresim-character-creator/src/ornament_controls.rs` | full source | `c10940e30bdb0ae77feaca37d5d33b6aac96e7cd7a97a48fc729c69be81aad3a` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/adventuresim-character-creator/src/cloth_lacing_controls.rs` | 1-180; full source | `0a39c720dd1b9d91efd80aca570810c22128f1a5e263ef4137860a4867013e20` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/adventuresim-character-creator/src/metal_controls.rs` | 1-90 only | `fa3594e3f5dc55f7ad90ba02d05fedfff207c5a0d394e03faad2da235b3d6d14` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/adventuresim-character-creator/src/construction_controls.rs` | 60-85 only | `3f3fa3760c04b76557e2a6927b461eb83c134a41d561952d22579c3f3f286599` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/fabelgeist-armor/src/gpu/metal/tests.rs` | 200-275 only | `fdc04f19250688292a3a3f1830e0b6ca7cdab927e166dd2940ddd549f6587e9c` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/Cargo.lock` | egui/ecolor version entries only | `e76fa8ed410f3a4c8609d2b7528761832a8103ba075659ecfb3d1cc85a268d69` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/adventuresim-character-creator/Cargo.lock` | egui/ecolor version entries only | `ca7c957fbd5d63fc3b507a0023e05ba2e74a3aef48c3dba54276013f7de6769a` |
| `/home/brunosegovia/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/egui-0.34.3/src/ui.rs` | 2245-2350 only | `30995097d9587e0576344e54f99e5eb33b5aa255167c9f90e3f4efc99ac1606b` |
| `/home/brunosegovia/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/egui-0.34.3/src/widgets/color_picker.rs` | 185-565 only | `ef6853846e055d2134346b166ea83e75a93b4eddfb20472e4eaee57e083c96d7` |
| `/home/brunosegovia/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ecolor-0.34.3/src/rgba.rs` | 1-150 only | `a149456582b50c16d55299994f6fbec507b8e0800610fbb7bf7ee2f616c63dbc` |
| `/home/brunosegovia/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ecolor-0.34.3/src/hsva.rs` | 1-220 only | `27a77496a0665c119c342bbd3cebbd28a5812392406765df049774b763e0e080` |

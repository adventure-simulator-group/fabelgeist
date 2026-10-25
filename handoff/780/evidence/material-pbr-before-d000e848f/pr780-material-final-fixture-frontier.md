# Final PBR fixture and native boundary frontier

Pre-edit discovery at HEAD `d000e848f2f5bc28b0728c2cd664c727a41ecf50`.
**Not implementation clearance.** Root remains the sole implementer. No source
edits, project execution, tests, probes, cases, helpers, assertions or doctests
were added or run. This account preserves the prior UV and density verdicts.

## Required existing fixture adaptations

All bodies listed here were freshly read, including their existing helpers,
callers and assertions. The JSON companion pins exact current source hashes and
partial production read limits.

| Current source | Direct final-API adaptation | Existing scenario/oracles to preserve |
| --- | --- | --- |
| `garment_material.rs:269-278` | Replace positional `surface.pbr().2` with the named returned material's roughness, projecting Ratio only for the existing native numeric comparison. | Same MailWeave roughness 0.6, metre-to-ring UVs 3 by2 and original tolerances. Other ring/image/invalid weave fixtures unchanged. |
| `clothing.rs:637-665` | Access specification.appearance metallic/roughness in the two original comparisons, using native getters as needed. | Same polished steel material, rigid channel, offset0.024, metallic1, roughness0.20, coverage0.3. No added comparison. |
| `clothing.rs:673-703` | Replace the triple in the existing literal with one SurfaceMaterial appearance: white RGBA, metallic0, roughness1. | Existing MissingLandmark/C_NECK admission stage, all body arrays/placement/skin/error match remain unchanged. Physical EquipmentMaterial::Linen stays independently retained. |
| `tests/textile_export.rs:35-53` | Carry `fabric.material()` whole into RiggedShell.appearance. | Same opacity200, band750..1000, colors[240,235,220]/[190,150,55], Blend, front-only faces, white factor, pixels[240,235,220,51] and[190,150,55,255]. |
| Tactical `mesh_tests.rs:6-16,42-60` | Red/gold literals become SurfaceMaterial using Srgba/Ratio; both components become explicit Surface appearance, remove independent material/textile fields. | Two surface groups, same colors, roughness0.9/0.8, same topology/UV/skin and center(4,3,0), dropped positions(-3,-1,0)/(1,-1,0). All existing assertions unchanged. |
| `export.rs:1151` | Whole appearance([0.1,0.2,0.3,1],0,0.9). | Separate Tunic, body-topology skin/morph correspondence, red decode0.0100228, body4/shell3 compacted rows. |
| `export.rs:1275` | Same appearance as preceding row; preserve maps-only surface and sole explicit UVs. | Independent4-vertex garment, nine indices, five attributes, two embedded preview-size PNGs, normal/MR roles, red decode oracle unchanged. |
| `export.rs:1402` | Whole appearance([0.5,0.35,0.23,1],0,0.58). | Shell-only Belt, no body primitive/attachment geometry, pelvis-local socket/extras and same rig assertions. |
| `export.rs:1547` | Whole appearance([0.7,0.7,0.7,1],1,0.2). | Independent skinned bracer and named forearm_width morphs, zero default weights. |
| `export.rs:1695,1753` | Both whole appearances([0.5,0.3,0.2,1],0,0.8). | Existing fitted/bootstrapped anatomical UV socket and open-belt rejection remain; these socket UVs are separate from rendering UV authority. |
| `export/assembly.rs:162-179` | Whole appearance(white,1,0.2) inside existing array map. | All three skull/bevor/visor names, hinges, nine image bytes, AlphaMask, compacted used vertices[1,2,3], UVs/skin/plate edges and every morph unchanged. |
| `export/morphs.rs:135-152` | Whole appearance(white,0,1). | Existing mismatch error, same-name success, NaN delta error in the same order. |
| `export/textures/tests.rs:22-31,40-57,82-95` | Existing helper accepts one SurfaceMaterial instead of three primitive parameters; its existing callers construct the same whole record directly. No replacement/helper layering. | Default white/1/0.3; normal-only[0.25,0.5,0.75,1]/0.8/0.2. Preserve nonwhite factor, metallic/roughness equality, one PNG, wrapping, shared three image roles/content, five files and corrupted-content rejection. |
| Armor `trim/tests.rs:218,227` | Required appearance Inherit in both existing components. | Same skull/visor ranges, appended geometry/morphs and all original component ownership/trim assertions. No new material oracle. |
| `skin_rules.rs:268-278` | Existing component helper uses Inherit. No new helper or changes to its callers. | All original gorget/helmet/coif/cops/spaulder/besagew/fauld/tasset skin and missing-anchor cases unchanged. |
| `tests/textile_appearance.rs:197-204` | Existing skinned helper assigns ComponentAppearance::Textile directly. | Same velvet recipe/color[70,15,25], native seam split, chart circumference, exact triangles/normals/skin/morph endpoints and original invalid normals case. Whole file read: all other alpha/pixel/relief/motif scenarios and helpers remain unchanged. |

This inventories ten RiggedShell fixture literal sites (six export literals,
one array-map literal, one morph literal, one texture helper literal and one
external textile literal), five component fixture literal sites (two Tactical,
two trim, one skin helper), one GarmentSpecification literal, and one existing
textile assignment. Numeric leaves are legitimate literal/fixture boundaries;
carry the whole admitted named record thereafter. Do not add tests, cases,
helpers, assertions or doctests to complete this API migration.

## Producer and native boundary obligations

The current four tuple producers are `clothing_material::pbr`, `CLOTH_PBR`,
`MailWeave::pbr` and `MailSurface::pbr`. Their actual consumers include catalog
and equipment export, underlayer preview, armor_metal, GarmentSpecification,
garment::rigged, drape preview, and Tactical runtime equipment. Current source
occurrence enumeration is recorded as a frontier, not broad receiving clearance.
All internal recipients must carry the canonical SurfaceMaterial record;
extract encoded channels/Ratio scalars only at actual numerical or Bevy/GLB
boundaries. Preserve numeric coefficients and the physical EquipmentMaterial
selection. Remove ArmorComponentMaterial directly with no compatibility alias.

Actual component receivers currently prioritize textile over independently
stored material in preview/Tactical; Character export first applies component
material, clears catalog maps/boundaries, then attaches optional textile maps.
The final required Inherit/Surface/Textile enum must express those final states
without retaining a derived material beside mutable TextileAppearance. Derive
textile material at actual use, preserve textile geometry/seam/relief operations
and the explicit inherit-versus-own-material behavior. Update these comments
against actual final matching code; do not claim the enum alone validates mesh
or texture payloads. Required externally tagged snake_case schema, unknown-field
rejection and no deserialize default are selected Root design, not present
implementation or independently cleared wire behavior.

Srgb/Srgba are encoding owners, not numerical-policy admission certificates.
Their docs must distinguish encoded RGB from unencoded alpha, preserve float
precision and transparent array serialization, and avoid claiming finite/unit
range without actual enforcement. Existing Color is a separate public RGBA
struct with named-field serialization; do not silently change its representation.
Math Ratio explicitly adds no range policy. SurfaceMaterial must not introduce
universal clamping or a default that changes authored coefficients.

The shared Creator skin source must retain encoded RGBA[0.64,0.39,0.30,1],
metallic0 and roughness0.52. Current body preview and BareBody use encoded
Color::srgb; GLB currently writes those encoded values directly as linear factors.
Correct the GLB body through the same checked canonical projection as shells.
Keep Bevy-only reflectance0.46, preview tint and diffuse transmission0.045 at
their existing actual owners. BareBody currently has no transmission override;
sharing PBR must not implicitly copy preview's other policies into it. The
hair-painted scalp cap remains a separate hair material (`scalp_shade::material`),
with the body skin visible beneath it; do not replace that cap with skin.

The existing GLB local channel decoder and Color's `srgb_to_linear` have the
same operation body after identifier/whitespace normalization (recorded source
comparison true): threshold0.04045, divide12.92, then
`((encoded + 0.055) / 1.055).powf(2.4)`. Use the canonical owner once for RGB,
preserve alpha exactly, and retain f32 arithmetic/no clamp. Bevy0.19.1's gamma
function has an additional nonpositive identity branch, so routing GLB through
Bevy would change existing negative-channel behavior. Bevy receivers should
continue to receive encoded channels through Color::srgba.

Finite encoded input does not guarantee a finite decoded f32. Positive finite
channels can overflow the exponent result. Pinned serde_json1.0.151
value/from.rs:30-44 maps nonfinite floats to Value::Null, and value/ser.rs:151-153
forwards through that conversion. The final actual GLB projection must reject
nonfinite decoded factors (including alpha and PBR scalars) before material JSON
and before any image/GLB file publication, propagate Result through Writer and
export entry, and preserve error causes. It is a terminal native record, never
another retained authority. This is source-derived evidence; no arithmetic
probe or new case was executed. Finite out-of-unit values whose actual native
projection remains finite stay admitted under the selected contract; this is
not a claim of full glTF range validity.

Texture ordering is independent of the internal PBR record. Retain scalar
projection, then general SurfaceTextures, textile finish, then ShellTextures.
A color PNG deliberately installs white factor including alpha1, avoiding
repeated opacity multiplication; normal-only maps retain scalar factors.
Plate maps overwrite normal/MR and roughness factor1, while earlier color/AO,
alpha mode, sidedness and textile extension survive. Sole texcoords and existing
compactor remapping remain as reviewed in the six-file UV tranche.

## Actionable comment findings

- **PBR-FIXTURE-C1, P3 — garment_material/rings.rs:69.** “Color and alpha are
  sRGB” incorrectly includes direct coverage alpha. Replace with “RGB color
  channels are sRGB; alpha, normal and occlusion are linear data.” Actual map
  construction and both canonical/Bevy native conversions preserve alpha.
- **PBR-FIXTURE-C2, P3 — preview.rs:189.** skin_material claims to shade the
  scalp cap, but actual cap uses scalp_shade::material based on hair_surface.
  Describe the body's skin visible beneath the separate hair-painted cap.
- **PBR-FIXTURE-C3, P3 — Armor components.rs:122.** “Unlit surface parameters”
  conflicts with the actual StandardMaterial PBR receivers. This reconfirms an
  already inventoried issue; replace it at the canonical SurfaceMaterial owner.

## Limits and acceptance

No repaired API exists yet, and no final comment/type/behavior clearance is
issued here. Exact hashes and read extents are in the JSON companion. Complete
fixture bodies were read; production fitting, native rasterizer, all serialized
external consumers and every graphics receiver are not globally cleared.
Unrelated dirty generated-program work is excluded. The encoded float color
picker bug remains open in the next Metal semantic scope; this report does not
clear it. Global thirteen-lens waves, three independent final reviews and hosted
checks on the final pushed revision remain outstanding.

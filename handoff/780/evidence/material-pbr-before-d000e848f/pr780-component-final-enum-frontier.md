# PR #780 final component appearance enum frontier

Read-only preimplementation source closure at HEAD d000e848f2f5bc28b0728c2cd664c727a41ecf50.
The companion JSON pins 61 whole-file SHA256 identities and exact read extents.
Twelve actual ArmorComponent literal expressions were read and classified;
the definition itself is not counted as a constructor. Source aliases,
non-component .material matches and unrelated ECS/weapon components were
separated by their actual owners. Searches located sites; source bodies
established the dispositions below. This is not repair or global clearance.
No source edits, checks, tests, probes, cases, helpers or assertions were made.
Completed density and UV reviews remain preserved.

## Selected final contract

ArmorComponent.appearance is required and owns ComponentAppearance:
Inherit, Surface(canonical Armor SurfaceMaterial), or
Textile(TextileAppearance). Serialization is snake_case externally tagged;
component unknown fields are rejected. There is no default deserialization,
legacy material/textile field, compatibility alias, or independently retained
derived PBR beside Textile. Inherit is explicit at source literals and means
catalog inheritance, not a white/default surface.

Keep ComponentAppearance Clone/Debug/PartialEq/Serialize/Deserialize with
payload-compatible bounds; it cannot be Copy or Eq because its variants
include mutable recipe ownership and float SurfaceMaterial. ArmorComponent
already requires deep Clone and PartialEq and can retain those derives.
SurfaceMaterial alone remains Copy. Existing serde arrays/scalar fields of
the scalar material remain canonical encoded source representations.

The enum owns textile()/textile_mut() and material() semantic operations (or
receivers exhaustively match it). material() returns no surface for Inherit,
the actual stored scalar surface for Surface, and derives the current scalar
surface from Textile's current recipe. Do not replace this operation with a
primitive tuple adapter or retain its derived result beside the recipe.

## Every actual component literal

| Source | Current construction | Final disposition |
|---|---|---|
| crates/fabelgeist-armor/src/gpu/part/layout.rs:296 | production; material None, textile None; role/mount/hinge later retained | Inherit |
| crates/fabelgeist-armor/src/gpu/breastplate/anime_layout.rs:81 | production; material None, textile None; course mount unchanged | Inherit |
| crates/adventuresim-character-creator/src/device_fit.rs:305 | production; whole fauld/tasset components preserve physical roles and lengths | Inherit |
| crates/adventuresim-character-creator/src/underlayer_armor.rs:95 | production; design.color.material; no actual textile recipe; mail returns empty vector | Surface |
| crates/adventuresim-character-creator/src/underlayer_armor.rs:178 | production; primary/secondary TextileColor scalar appearances; shared component ranges intact | Surface |
| crates/adventuresim-character-creator/src/waist_support_mesh.rs:151 | production; retain actual main.fabric clone; remove cached derived scalar copy | Textile |
| crates/adventuresim-character-creator/src/cloth_fastener_mesh.rs:186 | production; hardware Surface or thread/button/cord Textile; same producer payload reaches component | clone actual ComponentAppearance |
| crates/fabelgeist-armor/src/trim/tests.rs:218 | existing fixture; same Skull metadata; no new assertions/cases | Inherit |
| crates/fabelgeist-armor/src/trim/tests.rs:227 | existing fixture; same Visor metadata; no new assertions/cases | Inherit |
| crates/adventuresim-character-creator/src/skin_rules.rs:269 | existing fixture helper; existing helper unchanged except field/API literal | Inherit |
| crates/adventuresim-tactical-client/src/equipment/runtime_equipment/mesh_tests.rs:42 | existing fixture; same red native color/factors and ranges | Surface(red) |
| crates/adventuresim-tactical-client/src/equipment/runtime_equipment/mesh_tests.rs:51 | existing fixture; same gold native color/factors and ranges | Surface(gold) |

No other ArmorComponent constructor/default/builder implementation was found
in the actual source graph. Public serde is an additional construction route
and must require the selected enum field. Empty GeneratedArmor/BuiltPart
component vectors are deliberate absence of partitions and remain unchanged.

## Exact enum-owned producers and mutations

cloth_fastener_mesh.rs:42's dual ClosureMaterial is removed directly. All
its producers and receivers migrate together:
- new HookEye (:100) and cloth_hook_mesh::single_hook (:127) construct Surface
  from the same encoded TextileColor, metallic one, authored roughness.
- button_loop (:131), cloth_lacing_mesh::lacing_cord (:72), sewn_eyelet_mesh
  (:20), and sewn lacing rings (:131) retain the actual Textile recipe clone.
- metal lacing rings (:123) construct Surface with metallic one and roughness
  .35. Do not substitute a fabric recipe or generic metal default.
- component (:186) clones that exact enum into ArmorComponent.appearance.
- cord (:222) branches on textile presence to choose its existing native
  section-side policy. Keep the branch through Textile presence even though
  the current two side counts happen to equal three. Preserve all tube/bead/
  hook numerical bodies, seam positions, ownership and face order.

waist_support_mesh.rs:151 retains main.fabric as Textile. Underlayer scalar
color constructions at underlayer_armor.rs:95 and :178 remain Surface: they
have TextileColor scalar appearance but no full procedural textile recipe.
Mail underlayer returns an empty component vector and retains its separate
map policy; do not manufacture a Textile variant just from the word cloth.

All device_textile_materials.rs replacement paths were read in full. Bodice,
partlet, gown, skirt and puff/slash fabric selections replace appearance with
Textile(recipe.clone()), rather than store both derived surface and recipe.
skirt_fabric's Boning/Hoops arm (:92-103) mutates appearance to Surface from
the actual support color and returns no fabric; gown_materials forwards into
that mutation. All optional design/context/error policies remain unchanged.

All four device_textile_relief.rs mutations must borrow the actual mutable
Textile payload: existing relief chart, existing needlework chart, inserted
partlet collar free-edge relief, inserted skirt/gown waist relief. Preserve
the complete chart/pleat/zone/retention/release expression bodies. Borrowing
component.appearance.textile_mut() keeps disjoint component.indices access;
a method borrowing the whole ArmorComponent would require extra borrow
restructuring. No surface snapshot may outlive a later recipe mutation.

textile_seams.rs has two Textile presence predicates (:16 and :44). They must
continue to distinguish procedural Textile from a scalar Surface with a cloth
color. The method clones the source and each component, reconstructs vertex
channels, changes only ranges, and pushes the cloned component. The enum
payload deep-clones with it. Physical cyclic-role exclusions and every seam
numerical condition remain exact. This investigation does not extend to
fixing its unrelated fallible reconstruction/geometry policies.

Existing tests/textile_appearance.rs::skinned (:196-204) directly sets each
component.textile; replace with the corresponding Textile enum assignment
using the same recipe. This is the existing helper, not authorization to add
one. examples/textile_review/mod.rs::write_textile_maps (:90-105) reads each
textile payload and outputs its maps/report; it is an actual receiver and
must use the new owner. Existing tactical runtime_equipment_bench.rs:313-314
compares two components' scalar materials and requires both present. Preserve
those same two assertions through the actual derived material operation;
comparing only different recipe tags would weaken the existing oracle.

TextileColor::material and TextileAppearance::material remain canonical
scalar producers, now returning SurfaceMaterial with typed color/ratio leaves.
Keep f32 channel division precision, current alpha/finish roughness and
metallic-thread coefficients. They do not become enum constructors unless
meaning actually calls for a retained recipe.

## Clone, assembly, endpoint and range-only paths

BuiltPart::append (gpu/part.rs:347-389) moves each whole component and adjusts
only vertices/indices; preserve the appearance payload without reconstructing
it. DevicePart::finish obtains PartLayout component metadata then sets hinge;
layout/components and anime course constructors initially supply Inherit.
Articulation stores those components on the host; breastplate::read clones
that complete vector into GeneratedArmor. These are not shader attributes,
and the enum migration requires no native buffer/WGSL change.

DevicePiece derives Clone over base/endpoints. ComponentPartition::new clones
the first component into classified groups and clones remaining components;
it rewrites roles/ranges while retaining complete appearance. apply copies
that same frozen component vector into base and all endpoint BuiltParts.
DevicePiece::append uses BuiltPart::append for base and every endpoint.
The actual staged readback finishes a BuiltPart for each realization, selects
base once, and retains endpoints. No source appearance is serialized through
GPU buffers or recomputed by a separate endpoint material pipeline.

assemble_recipe checks endpoint component role/vertex/index correspondence,
consumes base.components as final metadata, derives morph geometry from the
endpoints, then applies textile material/relief to the final GeneratedArmor.
It intentionally does not require endpoint appearance equality. Preserve that
relationship rather than expanding physical admission with new enum equality.
Waist assembly creates its explicit whole Fauld/Tassets Inherit components
before moving/appending them.

trim::Split retains its owned GeneratedArmor, changes only index/vertex ranges
through finish/group_by_component, and leaves appearance untouched. surfaces()
reads indices/roles; skin_rules, gown/partlet charts, body-layer and sewn
interface constraints, triangle budgets, supports and waist-fastener selection
are role/range metadata consumers. Their affected bodies were read and need
no appearance API edits. Sewn retain_faces updates ranges across base and
endpoints without replacing component payloads.

GeneratedArmor::constructed(Solid) returns the exact self including enum;
non-solid embedded tiling intentionally creates output with components empty,
with appearance assigned by existing catalog/trim/lacing receiver policy.
That established behavior is not new enum data loss and is not changed here.
The source files and receiving constructor were read without expanding the
numerical tiling algorithm audit.

## Native receivers and scalar policy

preview::spawn_armor currently derives an optional component scalar material,
then replaces it with procedural textile material if textile exists. Final
match: Inherit uses plate fallback; Surface projects its actual surface;
Textile uses textile_surface::material for current scalar recipe and maps.
Preserve trim's independent native material and existing textures/alpha/finish.

Tactical MaterialPartition currently groups (optional scalar, optional textile)
and chooses textile first. Group by the complete canonical appearance, while
normalizing missing component to Inherit so it still groups with explicit
inheritance. Do not keep two parallel group fields or classify by the enum's
variant alone. Surface must keep native color/factors; Textile must retain
recipe-equality grouping and map generation; Inherit uses the actual catalog
fallback. Existing red/gold fixture component literals become Surface with
unchanged scalar data and geometry/UV/skin/color assertions.

character_morphs::component_materials must derive the current material for
Surface or Textile, replace shell appearance, clear inherited generic/plate
maps and plate edges, then apply Textile finish/maps from the actual payload.
Inherit leaves catalog shell policy alone. This preserves valid existing
paired textile/surface behavior while removing impossible mixed-authority
states. Completed maps-only surface/sole texcoords ownership is retained.

## Serialized review frontier and concrete final native view

A missing dependency was found and resolved in design before implementation:
review_export.rs:109 serializes the actual component record today.
render_museum_armor.py::import_review (:288) consumes its material field;
render_armor_review.py:75-76 also consumes material/base_color. Merely replacing
source component fields would silently replace explicit materials with each
script's fallback. These are actual consumers, not speculative external users.

Root selected the concrete coherent boundary: private ReviewComponent derives
Serialize and preserves role, borrowed vertex/index ranges, hinge and borrowed
mount; material is Option of the SAME checked LinearSurfaceMaterial used by
GLB. Inherit yields None. Surface projects its own canonical material;
Textile derives the canonical current Rust scalar material and projects it.
The terminal artifact emits no source textile recipe beside derived material.
ArmorComponent's source serde remains the selected clean required enum.
No Python sRGB formula, compatibility parser or second source PBR owner is
needed; both scripts' current scalar material field contract stays intact.

Creator library surface_projection.rs owns LinearSurfaceMaterial with native
RGBA4/f32 scalar fields. Its TryFrom of canonical Armor SurfaceMaterial uses
the canonical Color decoder exactly once, checks native output finiteness,
and returns typed BaseColor/Metallic/Roughness failures; no clamp/widening.
This also corrects the real old boundary mismatch: Rust component color is
encoded sRGB, while museum renderer explicitly defines linear RGBA and writes
that array directly into its native shader socket.

The GLB projection must consume effective appearance: the selected owning
SurfaceTextures.scalar_appearance(source) replaces the whole factor with
Srgba::WHITE iff base_color_png is Some; all other factors survive. Move the
white-factor policy out of TextureImages::apply before checked projection.
Original nonfinite input remains rejected by input validation. Huge but finite
encoded color hidden by an actual color map remains accepted because its
observable factor is white, preserving the old override behavior; without a
color map its decoded nonfinite result rejects before JSON/file publication.
Image, alpha, sides, sampling and final plate/textile effect order stay exact.

ReviewComponent projection finishes before item JSON construction/file write;
no nonfinite projected number can become JSON null. The existing overall
review export writes body.json before later item errors and is not promised
as an atomic whole-directory transaction by this repair.

Museum consumer needs role, ranges and native scalar material. The Workbench
consumer additionally reads hinge. mount is preserved as real geometry
metadata even though current Python does not inspect it. armor_bake_projection
reads only component role/ranges and can remain byte-identical. Preset
Item.components is a DIFFERENT map of user material overrides, not serialized
ArmorComponent; its schema and override order remain unchanged. Existing
scripts/test_render_museum_armor.py tests presets, GLB and pose contracts;
it has no direct import_review appearance case and none is added. No additional
first-party serialized ArmorComponent field receiver was found in actual source.
This does not claim visibility into external consumers.

## Acceptance and limits

Before implementation, Root's all-thirteen inventory must include the two
Python review receivers and native ReviewComponent/projection owner, the
textile review example, bench assertions, every literal below and all four
mutable relief branches. After repair, require direct enum use with no old
fields/local ClosureMaterial/parallel partition fields; preserve complete
recipe Clone/serde/mutation, physical role/range correspondence and original
numerical geometry bodies. Existing fixture oracles/cases/helpers remain
unchanged except direct API expressions. Source projection checks occur at
actual native adapters and share their one canonical owner.

No complete physical geometry, shader/native dispatch, rasterizer, arbitrary
component range validation or external-data-client clearance is asserted.
Protected generated-program dirty files are unrelated and untouched. This
preimplementation frontier is source closure for the enum decisions only;
full repair waves, independent reviews and required hosted project checks on
the same final pushed revision remain pending.

## Source identities and read extents

| Source | Read extent | SHA256 |
|---|---|---|
| `crates/fabelgeist-armor/src/components.rs` | 105-129 canonical component fields/serde and scalar owner; role/hinge source previously read, no new numerical scope | `0cee8ea4265d2ec61509c2c7a516a7dfa6d9d46a483d83fdae928bd1fe010ad5` |
| `crates/fabelgeist-armor/src/textile.rs` | whole source material producer | `15db7df94796b698c31fed85b836eddca6737ede411b7b1a4fcb88ed8f29dc30` |
| `crates/fabelgeist-armor/src/textile_pattern.rs` | 1-46 payload derives/fields; 218-240 canonical material derivation; no raster math clearance | `5eb511d902255a5f0b52b7f0de48232b1ee202e4e069cae7802be6cad5ec4f09` |
| `crates/fabelgeist-armor/src/textile_seams.rs` | whole source including cloned appearance and textile predicates | `3882e3a17758fc8e0ec7ee8c6559353c07b0902f93bd84b70734484950ddc128` |
| `crates/fabelgeist-armor/src/design.rs` | 128-155 GeneratedArmor derive/retained component vector | `86dac554a42b48cf1547e5656337fb718b4a4fc7e231a30009ce00ff7207dd00` |
| `crates/fabelgeist-armor/src/plate_mount.rs` | 45-61 mount serde/Copy fields | `6b0650a1f0aa61c043d101edb7ed5f577b7291820201c1e5c4583559ab52f3f9` |
| `crates/fabelgeist-armor/src/lib.rs` | 90-105 canonical component public exports | `d4b6a4af8b3137b359c2d08f8ef03445e100edc2d1280ca74ec816212af31264` |
| `crates/fabelgeist-armor/src/gpu/part/layout.rs` | 113-143 metadata builder/mount and 281-312 final component constructor | `82a6a4729b279187a9e7012181e5b85601f4ab3924c707d3243816168a83bc62` |
| `crates/fabelgeist-armor/src/gpu/part.rs` | 250-317 complete finish appearance construction; 326-389 BuiltPart/append move and offset | `def762808a49aeb39b51f92cbc23450db6811d6f0eef2db8c97c963625114f73` |
| `crates/fabelgeist-armor/src/gpu/breastplate/anime_layout.rs` | 55-104 complete literal/mount role context | `b7381d70280af87d9fdd65c064d5048239c9f001182df31961346e76eb07e602` |
| `crates/fabelgeist-armor/src/gpu/breastplate/anime.rs` | 1-58 Articulation metadata retention and publication | `a41d7583511916f837e5a9402fe4658589c54e4cfe15fa29cb04dd6a585ca000` |
| `crates/fabelgeist-armor/src/gpu/breastplate/mod.rs` | 278-307 component clone in GeneratedArmor construction | `5a3e370fabbd7ab8eedb1cdf0bbe4454e3cd7430cf9d5eb75f2a1b75de89fe31` |
| `crates/fabelgeist-armor/src/trim.rs` | 80-125 trimmed/surfaces component-range metadata paths | `81abc6494df0d4f051b0c4548517fe80061204de7e90d9d2eb190635a12de5f2` |
| `crates/fabelgeist-armor/src/trim/split.rs` | 1-75 intake; 250-341 finish/group_by_component metadata moves/ranges | `44aafa5468b01108bee89840514846df79aa2075ea6c2e154b21d93cee321b9a` |
| `crates/fabelgeist-armor/src/trim/tests.rs` | 204-267 existing two component literals and affected metadata oracle | `4b726c8fb494223caa41cf5539d152fc924226f7c1baa3b4b40506bbe560da35` |
| `crates/fabelgeist-armor/src/tiling/mod.rs` | whole constructed transfer/source behavior; numerical tile algorithm not expanded | `7ff5bfe0408afa8433957ee7575dd1fbb9f83d6afe4bb3af59a581a81724280d` |
| `crates/fabelgeist-armor/src/tiling/embedded.rs` | 68-110 realized output constructor intentionally has no component metadata | `6c7f84f8e6b242d32e19ddecedfa1cfdb5f849bd984008eda7b0ea1da79e5345` |
| `crates/fabelgeist-armor/src/tiling/surface.rs` | 174-213 surface clone is GridSurface, not ArmorComponent appearance clone | `b11e892b1c5cebf91866d5ffeed3801659fd98201fe446753a33aadf65598020` |
| `crates/fabelgeist-armor/tests/common/mod.rs` | 205-230 component accessor returns borrowed record; no appearance projection | `2a42c8782162e902ce082adcd8baeb03ea6ea6dcf9b9ef98edad988d11ef68bc` |
| `crates/adventuresim-character-creator/src/device_piece_components.rs` | whole source: clone/repartition/apply to base/endpoints | `f6fb5fcd8c755d324faac65bffc7a5fa82abd5f87be2e40263deab5dd925ed7d` |
| `crates/adventuresim-character-creator/src/device_fit.rs` | 1-125 DevicePiece declarations; 285-326 whole-component construction; 410-505 assemble_recipe metadata checks and transfer | `dcd91f8fa4f1fc251a4195f28cd8b7954b77f03a174d2f6d455c70550c53e7dc` |
| `crates/adventuresim-character-creator/src/device_fit_readback.rs` | whole staged BuiltPart -> DevicePiece read/endpoint ownership | `4fda819f23879d921f044faec4e511bc017d63a09b33adb5b3aeb82a82d49b41` |
| `crates/adventuresim-character-creator/src/device_piece.rs` | whole recording/check types; no host appearance retained on device | `01c4a60fd64a4c0fc9c1c368d85e1c693d5e1e09849c67bead477704da65362f` |
| `crates/adventuresim-character-creator/src/device_textile_materials.rs` | whole all replacement branches and nested support mutation | `5d457ed74d03a8130d204d48803cb881c977c580d8e36a25ff94971dd2288f37` |
| `crates/adventuresim-character-creator/src/device_textile_relief.rs` | whole four textile mutation branches | `de0860c5622ce947641038fe25033226ed7902722064cc5bcb4c8dd2e03e1542` |
| `crates/adventuresim-character-creator/src/gown_materials.rs` | whole role->fabric selector and skirt mutation forwarding | `140584a846a75ecd280dc9df4346f9c98fca07bbddd53be8b3df8129ec96f242` |
| `crates/adventuresim-character-creator/src/cloth_fastener_mesh.rs` | 1-245 appearance owner/constructors/component publication/cord predicate; bead signature at292; rest geometric bodies previously read and no new clearance | `bae2477655a7d54619d1f5265775f85fce6b4e1ec02693613f69a6b697ca3657` |
| `crates/adventuresim-character-creator/src/cloth_hook_mesh.rs` | 1-146 full appearance parameter propagation and single-hook constructor, geometry unchanged | `f0561bc0a88d651c54ed6c5d6334c2fdc9bb6b1e6d2408a35132eac95a07a131` |
| `crates/adventuresim-character-creator/src/cloth_lacing_mesh.rs` | whole ring/cord/metal/sewn appearance producers | `328fbd586aca214f25884be7bcf0d69be4ba4375e6b3baada8f4f6c4ddbb7b99` |
| `crates/adventuresim-character-creator/src/sewn_eyelet_mesh.rs` | 1-36 textile producer;90-114 last-component range extension; geometry body unchanged | `33ef4daa09c4bb138ca81a7cbcf163235670f05842c3671154ce5246d844fc95` |
| `crates/adventuresim-character-creator/src/waist_support_mesh.rs` | 145-165 Textile literal and metadata publication | `3e272c03e30186833e70e0b054c64b4b153f852114a5a5ce03b4f1926e2c1b0f` |
| `crates/adventuresim-character-creator/src/underlayer_armor.rs` | 80-107 Surface producer;161-188 two-color Surface component partition | `0aeea0b47034016130f42d75d0cc2f050d6ab6447c71d05accff1c34e517cc32` |
| `crates/adventuresim-character-creator/src/cloth_sewn_interfaces.rs` | whole endpoint correspondence and retain_faces range mutations | `ed665e0c43e176a185f2b7271bc59fcf7f9205e076146c042580c6ba161e9902` |
| `crates/adventuresim-character-creator/src/cloth_stack_interfaces.rs` | 1-120 relationship entry;140-185 apply/collect;232-293 component range/role-only readers | `3ee6a6b7c269680ac4b5b971a88875d28a47a61e296b636de1aa9de41c5f3fbb` |
| `crates/adventuresim-character-creator/src/body_cloth_plan.rs` | 1-125 bind_skin component ranges and physical-layer creation | `ed03fb038be60644eb643167faa3800dab68f3ad4e5c80b244d04039ed19a9bf` |
| `crates/adventuresim-character-creator/src/gown_charts.rs` | whole; component role/range only | `94c744868bd1623d75557925ec8eb7d833aba75a02e1a3eafc4f44ec31b82e29` |
| `crates/adventuresim-character-creator/src/partlet_plan.rs` | 140-168 material-role/range chart update only | `b6726d360c5188bf5ee3cfd6b8a08b038fb701402e380f4e43e4d4d1f1719c38` |
| `crates/adventuresim-character-creator/src/cloth_lacing_support.rs` | 15-42 first-component geometry/range only | `421e4a76a36f47086e91868ae8ff44f4729c0640ebef96776f9a53f2bde794b0` |
| `crates/adventuresim-character-creator/src/cloth_layer_constraints.rs` | 275-300 torso-role/range only | `88671b61f088a1f4ba13dc92fa8c9038caeeab1a993fc828a5bf8a60562d86d0` |
| `crates/adventuresim-character-creator/src/armor_layer.rs` | 1-55 component slice source;72-110 range admission only | `d09c70cba9d5be3410f99e3676c357683d3cd44cfd59ed48ba0797695e132978` |
| `crates/adventuresim-character-creator/src/cloth_triangle_budget.rs` | whole component-role/range diagnostic only | `19f1c1abd1ed60c7ad6d963acb3d00775b354d729780721cc206a78ba3a282e4` |
| `crates/adventuresim-character-creator/src/waist_fastener_sources.rs` | 270-292,322-340 component role/range query only | `1b58a81258bbce0d4f7b7b406f35c853027f6e653a448298b866d2bc3a036e73` |
| `crates/adventuresim-character-creator/src/device_wrapped_tasset.rs` | 210-243 range-only endpoint validation callback | `dc575790bf67f066d6c4dfd64c61f158383fc04efb95bc5dfd3633fe2ce17033` |
| `crates/adventuresim-character-creator/src/skin_rules.rs` | 90-127,150-172 role-only attachment;246-288 existing component fixture constructor | `f1ab48b207688209504ac8ab5ddef2286d7ebac5d6e2d8545313f87e1e3db474` |
| `crates/adventuresim-character-creator/src/preview.rs` | 85-124 actual native component appearance selection; broader PBR source previously traced, no new geometry clearance | `f7fa8bef88e0b718991dab2245cce57436b6748186df1769f7fadcef64ceb698` |
| `crates/adventuresim-character-creator/src/character_morphs.rs` | 185-203 component material/textile receiver;233-279 surface selection/range relation | `8f76594ec3871a68df2f8e98740aa629e479b1e6fc7a0ad9a63e4e7f029e30a1` |
| `crates/adventuresim-character-creator/src/textile_surface.rs` | 1-125 native derived PBR/textures/alpha/finish receiver and map conversion | `6eb0a624f247ea8c1ed2ffd6b25f65ece4b8742d30dd44d190914c9e0b399726` |
| `crates/adventuresim-character-creator/src/review_export.rs` | whole producer/export/file boundary/JSON component serialization | `e9ae7ed8089bca0317e77661168325bad4467de6bdc6ab5c08696808b15205f0` |
| `crates/adventuresim-character-creator/examples/textile_review/mod.rs` | 80-106 actual component textile texture-report receiver | `7c4de4807e57d720db52ce9295b725866c0e5a83a369f487a34babec1cb03029` |
| `crates/adventuresim-character-creator/tests/textile_appearance.rs` | 176-232 existing skinned fixture helper mutates Textile payload, not new helper | `2e1613f877cb04558ff0524ed8f83feb75eb0ec8d3655c45b8c3efd10e323840` |
| `crates/adventuresim-character-creator/tests/skirt_construction.rs` | 140-166,180-204 role/range existing oracles only | `a1ec603ea2f3a48d91c136f5db6bedb408fc342a6bcc17c306d0a45ff3fdd872` |
| `crates/adventuresim-character-creator/tests/body_cloth_layers.rs` | 68-103 component roles/ranges existing oracles only | `de7fe94a71246b0953ae4d8c416c71458c04577c04a179e40f19bbbc31539b39` |
| `crates/adventuresim-character-creator/tests/skirt_surface.rs` | 180-199,234-251 component-role/range existing oracles only | `6be4bdd141d88d087e4e53c63b87a81abb09e34f15541005b7320fb0da5d2560` |
| `crates/adventuresim-character-creator/tests/textile_treatment.rs` | 58-74,98-112 component-role/range existing oracles only | `584a09367f17674a14660adb1e8469da6e050cce200ab4f6f83433725d7b2ae2` |
| `crates/adventuresim-tactical-client/src/equipment/runtime_equipment/meshes.rs` | 1-101 actual partition equality/native selection; remaining mesh body read in preceding current-source frontier | `4994b1974149becac0c96f9e0dafeb0d26c1db776cc4adef9d2e1b388b033058` |
| `crates/adventuresim-tactical-client/src/equipment/runtime_equipment/mesh_tests.rs` | 1-90 two Surface constructors, two component literals, original material color oracles | `142460bac2304c4a63a77c9759a003d9bd57a9d5b6210b8d7954105d6ee3e827` |
| `crates/adventuresim-tactical-client/examples/runtime_equipment_bench.rs` | 292-322 existing raw material assertion receiver must directly adapt | `9a24028910994944c57a84fe00c6720e2b5def20808d919f2c1d3e8fc1a50d14` |
| `scripts/render_museum_armor.py` | 1-130 parser contracts;160-182 native shader assignment;240-327 import_review;356-405 display pipeline and override order | `6bd9fd305c94e259b76a706c76ea6a491ab7456c4eae238a464faffee0e59f31` |
| `scripts/render_armor_review.py` | 1-110 complete component/material rendering path | `f6e41c0f0f4ecd90ce83153901ed7b9426acfb5597a8c6cb18a3077a351167c7` |
| `scripts/armor_bake_projection.py` | whole; actual serialized role/range reader, no appearance read | `1207b3fa8367208256823e7efba0bc78a9171c53f1dd26423f71c570df5e9945` |
| `scripts/test_render_museum_armor.py` | whole existing fixture module; preset/GLB/pose tests, no review-component appearance case | `33f1212402df10175523bc4e1d0121835e8dfe68bed5e492996d20d45bdf7485` |

Final source drift: [].

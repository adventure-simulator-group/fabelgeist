# Final selected material API: source-backed thirteen-lens preinventory

**Preimplementation inventory complete for the selected API responsibilities**
at HEAD `d000e848f2f5bc28b0728c2cd664c727a41ecf50`. This is an inventory/repair
plan,
not code clearance. Root has not started these type repairs. Every lens below
has current source contracts, dispositions, connected repair, dependencies and
acceptance recorded in the companion JSON. No source edits, tests, probes,
project checks, new cases/helpers/assertions or CI were performed.

Scope: Color Srgb/Srgba; Armor SurfaceMaterial/ComponentAppearance; actual
encoded-color and roughness producer fields; existing PBR tuple producers and
retained fields; all identified component/catalog/cloth/mail/hair/trim/lacing
scalar appearance consumers; authored body skin; native GLB and review artifact
projection; actual picker/image/native roughness boundaries; direct manifests
and both locks. Remaining Metal physical units/counts/seeds, scratch semantics,
closed engraving, trim/cause/Inventory/native Grid/generated families remain
explicitly open. This does not expand into their repairs or grant clearance.

## Current source and drift

The earlier reports were used to locate actual responsibilities, not to inherit
verdicts. Current source and the new review artifact consumers were inspected.
The exact source accounts/read extents are in
`/tmp/pr780-material-final-all13-preinventory.json`. Partial reads name their
boundaries; a fingerprint is not whole-file clearance.

The complete committed change range since f2c7422fd contains exactly the
density/
Armory commit 59bb96d07 and UV authority commit d000e848f. Against the API42
receipt, only its six UV repair files drifted. They are the exact maps-only/
sole-texcoords changes independently reviewed, including the existing fixture's
two API lines. Material/color values were preserved. Density additionally
introduced material/density.rs and changed material.rs's export plus its actual
caller/Armory graph, all separately reviewed. No manifest or lock drift exists.
Dirty generated Map/native files are outside this inventory. No other source
change is silently treated as prior clearance.

## Frozen final design and exact native policies

- Armor `surface_material.rs::SurfaceMaterial` owns `base_color: Srgba`,
  `metallic: Ratio`, `roughness: Ratio`. Replace ArmorComponentMaterial directly
  and export the actual new definition; no old alias. Preserve Copy/PartialEq/
  serde scalar-array behavior; no universal Default or global [0,1] cap.
- Color adds actual private-array Srgb/Srgba leaves with named encoded/native
  constructors/read access and feature-gated transparent array serde. Existing
  Color struct and u8 TextileColor remain. Canonical TransferFunction::Srgb is
  the sole formula; alpha never receives RGB transfer. Srgba.with_alpha(f32)
  is an explicit native component operation for actual Permille decoding.
- ArmorComponent has required `appearance: ComponentAppearance`, snake_case
  externally tagged Inherit/Surface(SurfaceMaterial)/Textile(TextileAppearance),
  unknown-field rejection and no serde missing-field default/legacy fields.
  Textiles remain mutable payloads for seams, relief and needlework. Surface
  PBR is derived from that exact recipe, never cached beside it.
- Delete ClosureMaterial dual authority; publish canonical appearance directly
  for all button/hook/ring/cord/sewn-eyelet components. Inherit constructors in
  GPU part/anime layouts and device fauld/tassets become explicit Inherit;
  underlayer/trunk-hose surfaces use Surface; waist/fabric uses Textile.
- Metal.color, construction::Lacing.color and MailWeave.steel_color_srgb carry
  canonical Srgb. Their three roughness fields and EquipmentMaps/MetalImages
  metallic API carry Ratio directly. Keep actual owner validation: Metal
  .08..=.9, Mail .05..=1, Lacing existing0..=1. Other physical/seed fields remain
  separately pending, not hidden inside this PBR clearance claim.
- All four tuple owners return SurfaceMaterial directly: clothing_material::pbr
  / equipment_pbr, CLOTH_PBR, MailWeave::pbr and MailSurface::pbr. Replace raw
  PBR columns in GarmentSpecification with appearance while retaining its
  EquipmentMaterial selection; replace RiggedShell columns with appearance,
  retaining the separately repaired maps-only surface and sole texcoords.
- Creator library exports authored SKIN_APPEARANCE for binary preview/BareBody
  and library GLB output. Exact encoded [.64,.39,.30,1], metallic0, roughness.52.
  Bevy reflectance .46, preview specular tint/transmission .045 remain native
  receiver policies. Actual scalp cap uses hair material and is unchanged;
  remove the stale skin_material doc claim rather than changing its rendering.
- Creator **library** surface_projection.rs owns terminal LinearSurfaceMaterial
  {base_color:[f32;4],metallic:f32,roughness:f32}, cohesive Result alias and
  SurfaceProjectionError::{BaseColor,Metallic,Roughness}. TryFrom<SurfaceMaterial>
  performs canonical f32 RGB decode once and checks all native fields finite,
  preserving alpha and finite-out-of-unit policy without clamping/widening.
  GLB Writer propagates Result before any file publication; body skin and the
  review binary consume the same actual owner. Keep construction under this
  checked owner; avoid a publicly mutable record advertised as a finite proof.
- SurfaceTextures gains intrinsic scalar_appearance(SurfaceMaterial) policy:
  color PNG Some chooses typed white, None retains the original. GLB projects
  this effective appearance; remove the later white-factor write from
  TextureImages::apply so there is one policy owner. Other image/alpha/side/
  sampler effects and their order remain. Existing validation still rejects
  nonfinite ORIGINAL source fields. An enormous finite but texture-shadowed
  factor is not newly rejected; unshadowed decode overflow fails before JSON.
- ReviewComponent is a dedicated Serialize native artifact adapter, retaining
  role, borrowed vertex/index ranges, hinge and borrowed optional mount.
  Its optional native material comes from actual appearance.material() mapped
  through the same checked projection. Inherit -> None preserves the Python
  authored fallback. Do not leak the new source appearance serde or cache a
  duplicate textile/PBR inside the artifact. Python readers keep one native
  linear protocol, no new formula/dual parser.

The last two native choices incorporate Root's concrete pre-edit policy receipt;
they supersede the earlier placement/effective-factor ambiguity. There is no
remaining required final-owner choice identified in this selected inventory.

## Concrete constructor, mutation and receiving closure

TextileColor retains exact f32::from(u8)/255. TextileAppearance's source serde,
opacity/child admission and authored finish coefficients remain; material()
returns the canonical record with alpha from Permille and its actual finish.
Public mutable recipes do not acquire a false checked certificate. Surface
scalar opacity survives when no color bytes exist; rasterized color sets white
through the selected scalar policy. Textile finish, normal/AO formats, chart
sampling and alpha/side policy stay separate from the simple PBR record.

Device textile assignment sets one enum payload. The skirt support branch sets
Surface; later relief/needlework mutation borrows the canonical Textile payload.
Seam splitting and device component partitioning clone that payload while
preserving every exact range/vertex/skin/morph association. No appearance
mutation may leave an old derived PBR behind. GPU part/anime layout and device
fauld/tasset literals retain their exact geometry/metadata with Inherit.

All catalog/specification/rigged shell constructors and rebuilders carry the
record whole. Component receivers are one exhaustive policy: Inherit leaves
inherited catalog/maps; Surface replaces PBR and clears inherited map/plate
bindings; Textile derives PBR from the actual recipe and supplies its finish
and optional maps after clearing inherited plate bindings. Studio, Tactical,
GLB and review artifact must agree. Tactical partition keys borrow canonical
appearance, comparing the full textile recipe; normalize untagged faces and
explicit Inherit into the same inherited group. No separate scalar-only key
may merge different textile maps.

Trim and cord scalar construction use their selected Metal/Lacing values,
not copied defaults. Existing MetalImages map roughness factor remains one
because the map owns variation; native bake's roughness parameter gets the
Ratio scalar at that genuine PassParameters boundary. Do not alter protected
native dispatch/scratch/new/batch/upload functions. Mail keeps white scalar
factor, color in its actual encoded image; ring image byte rounding remains
exact. Its deliberate byte sRGB widget commits only changed values; no blanket
float-picker migration there. Metal/Lacing float pickers must decode to local
linear floats, call the actual egui float widget, and encode/write ONLY when
response.changed(). Preserve the actual containing changed/dirty logic.

Current generic export validates finite source PBR only, not [0,1]. A finite
encoded1e20 channel decodes beyond f32; pinned serde_json1.0.151 serializes
nonfinite floats as Null. Native checked projection fixes that actual output
hole. Its authoritative scalar source is the effective appearance after the
known white-color override; still validate original nonfinite inputs. Do not
silently tighten generic PBR to Python/Metal bounds or widen the formula.

## Actual review artifact and protocol dependencies

Full current review_export.rs, render_museum_armor.py and its existing test
source were read. review_export currently serializes source ArmorComponent
at :109. Python explicitly declares linear colors; Material.parse additionally
requires every supplied RGBA/metallic/roughness number in [0,1]. That narrower
native review protocol remains unchanged and distinct from GLB's finite-only
input admission. Native projection must not impose Python bounds globally.
Current actual authored review sources project inside its supported range;
malformed/out-of-protocol native records remain rejected by the same parser.

render_armor_review.py also reads material.base_color and hinge metadata, so
ReviewComponent must retain hinge. armor_bake_projection.py consumes role and
ranges only. The adapter preserves mount as unrelated metadata too. All use
the same linear projected scalar artifact; there is no need for Python to
interpret the new source enum or for an old/new artifact dual path. Native
artifact metadata and geometry are unchanged; encoded-to-linear scalar output
is a deliberate correction. No full physical Blender/render acceptance is
inferred from reading these consumers.

## Direct manifests and both lockfiles

All actual owner manifests were read. Armor and Creator currently lack a direct
Color edge; Tactical's existing fixtures directly construct the old PBR and need
actual typed color construction, so add a direct Color edge for direct imports
there too unless the final literals use an already exported actual constructor
without naming Color. Do not use transitive Rust dependency access or an alias.
Armor requires Color serde; Color wgpu is unnecessary. Existing Math serde is
available; Creator/Armor already have thiserror. No new registry version or
high-level crate is required.

Root Cargo.lock contains Armor, Creator, Tactical and Color packages; the
standalone Creator lock contains Armor, Creator and Color, without Tactical.
Their current exact dependency arrays and package versions are recorded in the
JSON. Add Color to actual importing package arrays in BOTH applicable locks.
Retain each lock's existing version qualification (standalone has wgpu29.0.4
among multiple versions; root graph differs). Current serde_json1.0.151 and
thiserror2.0.20 are actual pins, not manifest minimums. Feature-gate new color
serde derives AND transparent attributes consistently so Color without serde
still compiles; avoid accidental wgpu feature activation.

## Existing fixtures: API adaptation only

The exact selected constructor/adaptor graph includes:

- Existing RiggedShell literals in export.rs, assembly/morphs/texture fixtures,
  textile_export and production garment/hair/compact constructors. Replace
  triple fields with one appearance; preserve all existing native inputs and
  oracles. The existing export_equipment_with_maps helper changes its existing
  signature/callers to the record; add no helper.
- Tactical red/gold material literals and two component literals retain their
  exact colors/ratios/geometry expectations; inherited Armor trim/skin-rule
  component literals become Inherit. Preserve every role/range/hinge/mount.
- Clothing's existing specification literal and catalog factor assertions use
  the new record/getters without new assertions or policy changes.
- Existing MailWeave .6 roughness oracle and NaN case, and existing GPU metal
  invalid-roughness NaN case wrap the same values in unit-only Ratio. Never
  replace invalid input with a passing one or add cases/assertions.
- Existing mail color-byte fixture uses the same typed Srgb native channels
  before its original image rounding; same expected bytes. Existing skin
  render fixture continues to call skin_material with no new scenario.
- Existing Python preset/Blender fixtures remain byte-exact unchanged: they
  already represent linear native overrides and do not deserialize source
  ArmorComponent. No new review case/helper/assertion is authorized.

Exact full fixture coverage stays attributed to its actual read extent/peer
source account, never inferred from a search hit. The implementation must
source-compare every adapted fixture after edits; no executed coverage is
claimed here or supplied by newly invented cases.

## Thirteen sequential preinventory cells

1. **producer-propagation**

Contract: Encoded RGB/RGBA, roughness/metallic and PBR records retain one
canonical semantic owner from actual recipe/catalog producer through mutable
state and native export/UI/image paths.

Disposition: Actionable migration: current naked triple/PBR columns and raw
color/roughness leaves are unfinished.

Connected repair: Replace four exact triple owners:
equipment_pbr/clothing_material::pbr, CLOTH_PBR, MailWeave::pbr,
MailSurface::pbr. Type Metal.color/Lacing.color/MailWeave.steel_color_srgb with
Srgb and their roughness with Ratio; EquipmentMaps/MetalImages metallic API with
Ratio. Carry SurfaceMaterial in GarmentSpecification.appearance and
RiggedShell.appearance; migrate every
component/skin/catalog/trim/cord/hair/mail/native receiver directly.

Dependencies: Armor -> Color serde + existing Math; Creator/Tactical actual
direct imports and two locks. No physical/count/seed/engraving widening.

Acceptance: No raw retained duplicate triple, old ArmorComponentMaterial or
ClosureMaterial, tuple adapter, repeated primitive admission or duplicate color
formula in selected graph. Native `.ratio()`/encoded channels only at real
admission/arithmetic/image/UI/Bevy/JSON boundaries.

Source: components.rs; material.rs; construction.rs; garment_material.rs;
clothing_material.rs; clothing.rs; underlayer_preview.rs; metal_preview.rs;
character_morphs.rs; character_export.rs; equipment_export.rs; meshes.rs.

2. **existing-type-owners**

Contract: Reuse the lowest shared existing concept; equal primitive layout does
not merge different encoding/unit/role authority.

Disposition: Canonical final owners selected; no necessary alternate PBR
framework found.

Connected repair: Promote ArmorComponentMaterial into sole Armor
SurfaceMaterial; Color adds encoded Srgb/Srgba without replacing existing
unqualified Color or u8 TextileColor; Math Ratio reused. Creator library owns
authored SKIN_APPEARANCE and shared native surface_projection for GLB/review
binary.

Dependencies: Existing Color optional serde, existing Math unrestricted Ratio;
no Color -> Armor/Creator or Bevy/GPU edge.

Acceptance: One real type definition and direct export per concept, no
compatibility alias/forwarder; native record never retained beside PBR; physical
ShellMaterial, image maps and anatomical SurfaceUvLayout stay distinct.

Source: color/lib.rs; color/color.rs; color/transfer.rs; components.rs;
textile.rs; math/units.rs; lib.rs; bare_body.rs; preview.rs; review_export.rs.

3. **admission-contracts**

Contract: Encoding/unit leaves are not finite or unit-range certificates; source
recipe and actual native output admission retain distinct policies.

Disposition: Actionable PBR-PROJECTION-1 plus exact schema change identified,
all selected admissions inventoried.

Connected repair: Private encoded arrays with transparent feature-gated serde;
required appearance enum, snake_case and component deny_unknown_fields without
serde default/alias. Preserve source finite validator and each recipe bound.
LinearSurfaceMaterial::try_from uses canonical decode and finite checks, errors
BaseColor/Metallic/Roughness.

Dependencies: Metal .08..=.9; Mail .05..=1; Lacing current0..=1; generic finite
out-of-unit allowed. Python review Material remains a stricter [0,1] protocol
unchanged.

Acceptance: No invalid original source field bypass, no NaN/Inf reaches native
JSON, finite projected out-of-unit not clamped/rejected globally. Existing
zero/NaN fixtures stay same cases; old component wire fails cleanly instead of
disappearing into Inherit.

Source: textile_pattern.rs:191; material.rs:77; construction.rs:276;
garment_material.rs:90; export/validation.rs:141; export.rs:578;
serde_json1.0.151/value/from.rs; render_museum_armor.py:Material.parse.

4. **invariant-operations**

Contract: Replacement, transfer and mutation preserve encoding/alpha/recipe
bounds and geometry/appearance relationships; finite inputs can overflow
conversion.

Disposition: Actionable native f32 projection overflow, known widget mismatch
and mutable dual authority.

Connected repair: Keep f32 canonical formula/order, decode RGB once and alpha
never; guarded float picker local conversion; typed producer field updates
preserve exact recipes. Apply effective color-map scalar policy before
projection; preserve UV-only repair.

Dependencies: Srgba.with_alpha(f32) is explicit native component operation for
existing Permille decode; public Ratio/Srgb still require owner validation.

Acceptance: One exclusive payload update; no unchanged-frame float round-trip
writeback, quantization of Metal/Lacing floats, arbitrary clamping/tolerance,
redecoded linear pigment image or native numerical behavior changes. Huge finite
shadowed factor still succeeds if final white factor represents it; unshadowed
overflow errors before publication.

Source: transfer.rs:98-150; egui0.34.3/color_picker.rs:536;
metal_controls.rs:12; construction_controls.rs:73; fabric_controls.rs:48;
textures.rs:180; export.rs:578.

5. **aggregate-states**

Contract: One component appearance determines its scalar PBR, actual textile
recipe, finish and selected maps; independent source facts cannot drift.

Disposition: MCF-1 remains actionable in current source and final payload schema
closes it.

Connected repair: ArmorComponent.appearance Inherit/Surface/Textile; direct
payload queries and mutable textile access. Migrate seams, device
assignment/relief, component clone/partition and every renderer/export receiver.
Delete ClosureMaterial dual fields. ReviewComponent native record retains
role/ranges/hinge/mount, optional projected material, no textile cache.

Dependencies: Source metadata/vertex/index/range associations stay exact;
inherited absence remains catalog/default policy.

Acceptance: Changing textile finish changes scalar result in
preview/Tactical/GLB/review consistently; Surface clears inherited maps, Textile
clears inherited plate maps and supplies its own optional maps, Inherit
preserves inherited data. No independent PBR beside textile.

Source: components.rs:111; device_textile_materials.rs;
device_textile_relief.rs; textile_seams.rs; cloth_fastener_mesh.rs;
waist_support_mesh.rs; character_morphs.rs:184; review_export.rs:107.

6. **type-ownership**

Contract: Definitions sit at their actual lowest shared owner and module/API
access follows real binary/library and native composition.

Disposition: Final shared ownership selected; scope excludes protected #826
helpers.

Connected repair: Armor surface_material.rs owns PBR, components owns
appearance; Creator library surface_projection.rs owns terminal
LinearSurfaceMaterial/error/alias and SKIN_APPEARANCE public export; review
binary uses it. Focused new modules before source growth.

Dependencies: Root owns async metal bake function; do not change protected
dispatch/scratch/new/batch/upload helpers or generated tranche. Preserve
function bodies on required declaration moves.

Acceptance: No Color PBR dependency, renderer into Armor, or private binary skin
source inaccessible to library. Rust module order imports/reexports -> mods ->
const/alias -> records -> enum -> impl -> functions -> tests last.

Source: Creator/lib.rs; review_export.rs; bare_body.rs; preview.rs;
Color/Cargo.toml; Armor/Cargo.toml; gpu/metal/bake.rs:24.

7. **redundant-authority**

Contract: Store each source fact once; legitimate native derived output is
ephemeral and caches own a distinct fact.

Disposition: MCF-1/current PBR columns/duplicate skin and transfer formula
actionable; UV authority already repaired.

Connected repair: One canonical enum/PBR, one skin constant, one existing
transfer implementation, one effective scalar policy
SurfaceTextures::scalar_appearance. Remove TextureImages::apply white-factor
write when policy moves. Retain image cache and recipe snapshot policies.

Dependencies: Native LinearSurfaceMaterial can serialize only actual checked
projection output in authoritative code; consider private fields/read access
where needed.

Acceptance: No second Python formula/dual parser; same checked projection for
GLB/review, image identity cache unaffected; CompactShell sole texcoords
preserved.

Source: textures.rs:174-185; export.rs:578; preview.rs:158; bare_body.rs:57;
review_export.rs:109; export/compact.rs.

8. **behavior-preservation**

Contract: Mechanical types preserve source scalars/arrays/encoding/native
precedence; explicitly inventoried corrections are schema, color-space and
overflow changes.

Disposition: Selected deliberate changes recorded; remaining physical/material
numerical families not cleared.

Connected repair: Preserve catalog coefficients, u8/255, roughness policies,
alpha/side/sampler/specular/maps order, native scratch roughness bits. Correct
skin encoded-to-linear GLB output and review encoded-to-linear output. Keep
native shadowed-factor behavior via effective projection.

Dependencies: Python renderer accepts linear [0,1] and stays unchanged; source
component serde new schema differs intentionally, artifact explicit native
adapter preserves consumed metadata.

Acceptance: No case/assertion/helper added, fixture numeric/color/geometry/PNG
oracles preserved; existing NaN cases still reject; source finite-out-of-unit
admission remains. Skin body only: leave actual scalp cap hair material; fix
stale doc only.

Source: clothing_material.rs; textile.rs; textile_surface.rs;
export/textures.rs; review_export.rs; render_museum_armor.py;
render_armor_review.py; gpu/metal/bake.rs.

9. **enum-shapes**

Contract: Closed states own their payload and absence is intentional;
independent texture families remain independent.

Disposition: Required component enum selected; no need to turn independent maps
into exclusive enum.

Connected repair: Inherit/Surface/Textile replaces two independent Options;
semantic native failure enum BaseColor/Metallic/Roughness. Keep
SurfaceTextures/ShellTextures independent, alpha/sides/sampling enums. Real
textile_mut handles physical payload use.

Dependencies: ComponentAppearance Clone/PartialEq, not Copy because textile owns
memory; SurfaceMaterial leaves Copy.

Acceptance: No fake pending/default PBR, bool material role, compatibility
fields or original enum deserialize fallback. Tactical explicit Inherit/untagged
faces normalize same inherited partition.

Source: components.rs; textile_pattern.rs; meshes.rs; export/textures.rs;
device_textile_relief.rs.

10. **derivability**

Contract: Derive representations/traits when behavior matches, without weakening
admission or inventing policies.

Disposition: Current derive contracts and new projection error have direct
derive solutions.

Connected repair: Srgb/Srgba derive Clone/Copy/Debug/PartialEq, cfg serde + cfg
transparent consistently; SurfaceMaterial same serde/Copy; component enum
Clone/Debug/PartialEq/serde; projection error thiserror with cohesive owning
alias, no fabricated cause for pure arithmetic failure.

Dependencies: Existing Color manual authored Default remains outside; do not add
universal PBR Default, Eq for float, serde default appearance or unnecessary
bytemuck/POD/Hash policy.

Acceptance: No new manual Error/Clone boilerplate, source-erasing string wrapper
or unchecked claimed-finite deserializer. Existing thiserror2.0.20 covers actual
lock; no new dependency.

Source: Color/color.rs; Math/units.rs; components.rs; Color/Cargo.toml;
Creator/Cargo.toml; both locks.

11. **type-granularity**

Contract: Generic encoded colors, roughness ratios, recipe-specific bounds, PBR
aggregate, appearance state and terminal linear scalar projection express
distinct useful roles.

Disposition: Scope matches actual hazards without universal wrapper/validator
expansion.

Connected repair: One Srgba/SurfaceMaterial and one closed component payload;
Srgb on real RGB producers. Terminal record uses raw native arrays/scalars only
at actual GLB/review boundary, never application storage. Keep textile u8,
physical coordinates and image bytes separate.

Dependencies: Metal/Lacing/Mail typed roughness scope ends before physical
dimensions/counts/seeds/engraving/trim migration.

Acceptance: No per-consumer PBR type, generic launch/sort/count reuse, RGB/RGBA
byte conversion shim or alternate native color implementation.

Source: Color/color.rs; textile.rs; material.rs; construction.rs;
garment_material.rs; export/rigged.rs; review_export.rs.

12. **magic-strings**

Contract: Closed schema/role tokens have explicit native/serde owners; open
names/paths and terminal diagnostics remain real text.

Disposition: No new prose-based branching required; exact schema strings now
selected.

Connected repair: Snake_case inherit/surface/textile tags and projection failure
variants; retain role.name, catalog IDs, file paths, glTF native keys, PNG MIME
and actual Blender socket labels. Review uses a single native material record,
no legacy parser.

Dependencies: Library public reexport names/API direct; no display strings used
as identity/status classifiers.

Acceptance: No string round-trip for causes, debug-based classification,
duplicate transfer policy text turned into code, or extra alias decoder. Python
readers preserve exact own protocol checks.

Source: components.rs; review_export.rs; render_museum_armor.py;
render_armor_review.py; export/shell.rs.

13. **magic-numbers**

Contract: Every selected scalar has an authored unit/encoding/native
representation and owner; repeated policy not waived by one use.

Disposition: Selected RGB/RGBA/roughness/metallic facts fully inventoried;
physical tuning outside scope remains open.

Connected repair: Typed leaf construction at actual catalogs/defaults/native
fixtures; canonical skin source. Preserve 255/1000 encoding conversions,
canonical transfer constants, native f32 roughness insert. No unit cap/tolerance
added to generic PBR.

Dependencies: Distinct roughness bounds remain owning recipe policies; u8 mail
picker/image quantization remains deliberate actual native boundary,
Metal/Lacing float picker adds none.

Acceptance: Exact f32 formula/order and coefficients, alpha unchanged; finite
final conversion checked before JSON. Plate/hair/textile map policies and UV f32
pairs unchanged. No new GPU quantities/ceilings.

Source: textile.rs; material.rs; construction.rs; garment_material.rs;
fabric_controls.rs; garment_material/rings.rs; gpu/metal/bake.rs; transfer.rs.

## Completion boundary and explicit unread limits

All thirteen selected final-API implementation inventory cells are complete
before Root's type edits. They are actionable findings/acceptance, not repaired
verdicts. Native library owner, required source serde schema, mutable textile
access, shared skin visibility, effective color-factor policy, checked output,
review artifact metadata/protocol and direct dependency/both-lock composition
are now concrete. No remaining necessary API decision was found in this selected
scope after Root's final native policy receipt.

This does not claim a fresh full audit of every file hashed: unrelated geometry,
physical ring tracing, native shaders and dispatch helpers, every exporter
socket/morph implementation, whole UI framework, all public external clients,
archive artifacts and other color profiles remain outside the named reads.
The companion receipts record exact partial paths. Existing Drape preview
production expects/cause erasure remain the separate pending cause family;
this PBR inventory does not waive them because the containing code was read.
Remaining physical units/counts/seeds/scratch/Engraving/trim/Inventory/native
Grid/generated families remain open, with no push/hosted CI/merge authorized
by this report. Root remains sole implementer.

After implementation, complete whole connected source review and all thirteen
waves again, source-preserve existing fixtures, run independent Code/Rule/
Comment reviews, and satisfy required hosted gates on the same final pushed
revision. No current repair or test result is implied by this inventory.

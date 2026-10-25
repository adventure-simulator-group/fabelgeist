# Material constructor/geometry frontier: all-thirteen extension

Read-only preimplementation Rule discovery at HEAD
`f2c7422fd6a0f995eb90b72cc9c2a5bd12de4d35`.
Root is the sole implementer. No source, manifest, fixture, test, helper or
assertion was changed. No project checks, builds, tests, probes, scanners or CI
were run. Source reads, revision/status reads and SHA-256 bookkeeping only.
The dirty generated Map/native leaf tranche is excluded. This extends the
original material semantic discovery; it does not replace its open findings or
grant whole-PR clearance.

## Scope and fresh source receipt

All eight requested files were read in full, including unchanged producers and
all existing inline fixtures. Actual construction and receiving dependencies
were read to the extents recorded below. Exact path/hash/read-extent accounts
are in `/tmp/pr780-material-constructor-frontier-all13-sourceaccounts.json`.
Partial dependency reads are explicitly distinguished from full-file coverage.
The original report and Root's 127-account ledger are dependencies, not fresh
source clearance for files outside this receipt.

- `crates/fabelgeist-armor/src/textile_pattern.rs`:
  `5eb511d902255a5f0b52b7f0de48232b1ee202e4e069cae7802be6cad5ec4f09` (358
  lines).
- `crates/adventuresim-character-creator/src/cloth_fastener_mesh.rs`:
  `bae2477655a7d54619d1f5265775f85fce6b4e1ec02693613f69a6b697ca3657` (381
  lines).
- `crates/adventuresim-character-creator/src/cloth_hook_mesh.rs`:
  `f0561bc0a88d651c54ed6c5d6334c2fdc9bb6b1e6d2408a35132eac95a07a131` (147
  lines).
- `crates/adventuresim-character-creator/src/cloth_lacing_mesh.rs`:
  `328fbd586aca214f25884be7bcf0d69be4ba4375e6b3baada8f4f6c4ddbb7b99` (136
  lines).
- `crates/adventuresim-character-creator/src/clothing.rs`:
  `d3bce4f4d9a53afc7ea141c5edf1163e85f55cb1b455b976b3bd5e329902ce33` (699
  lines).
- `crates/fabelgeist-armor/src/tiling/mod.rs`:
  `7ff5bfe0408afa8433957ee7575dd1fbb9f83d6afe4bb3af59a581a81724280d` (137
  lines).
- `crates/fabelgeist-armor/src/tiling/lacing.rs`:
  `cce09840d83fc6c71d1020fc45961df2a0c9b4c2a85d5702bc0f65d6fad902a4` (321
  lines).
- `crates/adventuresim-character-creator/src/parametric_equipment.rs`:
  `fb8ed7e496fe124cf6ea6c4deb962b9602cf62f5b516d362b5e50ddca9829c49` (478
  lines).

## Constructor and replacement contracts

`Armor/src/textile.rs:7-21` owns the authored `TextileColor([u8;3])` wire
representation. Its material conversion divides each encoded sRGB channel by
255 in f32, sets alpha one, metallic zero, roughness .92. These are encoded
values: preview uses Bevy `Color::srgba`, and export converts the color to
linear glTF factors. Replacing this with float encoded color must preserve the
actual division and must not quantize other catalog float colors through u8.
The u8 authored textile owner need not disappear to reuse a float Color owner
at the PBR boundary.

`textile_pattern.rs:176-237` builds a solid recipe with opacity 1000, no layers,
no relief or needlework, and a cyclic-across chart. `validate` admits opacity
0..1000, at most twelve layers, matching relief/needlework charts and each
child contract. The public fields and derived deserializer do not themselves
admit the recipe. `material()` does not validate: it obtains TextileColor PBR,
replaces alpha with `Permille::unit()`, then replaces roughness by finish
(Linen .88, Wool .92, Silk .46, Velvet .78, MetallicThread .65), and metallic
with .8 for metallic thread or zero otherwise. A checked PBR design must
account for this actual unvalidated public conversion rather than silently
clamping malformed input or pretending all Permille values are bounded.
Unit-only leaves may legitimately leave the existing terminal validation in
charge. `is_translucent` is a named predicate based on base cloth opacity;
opaque decorations do not make the underlying cloth opaque.

`cloth_closures.rs:19-104` already owns ButtonLoop and HookEye variant payloads.
Validated ButtonLoop fabric and physical dimensions differ from HookEye's
color and roughness. Hook roughness admits 80..900 permille. The design also
checks cord/wire aperture relationships and width <= length. `cloth_lacing.rs`
already separates Metal eyelets from Sewn eyelets with fabric. It validates
fabric, gauge and aperture, ordered stations, inset >= radius, and sewn-seat
zero. Public serde inputs remain mutable recipes. The conversion from these
authored millimetres/micrometres to native local-frame geometry is explicit;
this review does not invent a new geometry validation policy.

`Creator/cloth_fastener_mesh.rs:84-196` constructs:

- ButtonLoop: PBR `fabric.material()` plus a clone of the same fabric.
- HookEye: color from TextileColor conversion, metallic one, roughness from
  its authored permille, no fabric.
- Component publication: exact just-appended vertex/index ranges, role
  ClothFasteners, no hinge/mount, PBR copied and optional fabric cloned; faces
  resized to triangle count with Outer classification.

`cloth_hook_mesh.rs:115-147` builds a return hook with the same direct HookEye
material rule. `cloth_lacing_mesh.rs:121-136` builds metal rings with the same
color conversion, metallic one and authored .35 roughness; Sewn eyelets use
fabric-derived PBR plus fabric. `lacing_cord:65-104` likewise retains its
fabric-derived PBR and recipe. These are direct users of the canonical shared
PBR record, not reasons for a new Creator PBR owner or primitive adapter.

`clothing.rs:59-106` resolves EquipmentMaterial or returns typed
MissingMaterial, uses its channel for physical offset, obtains catalog PBR,
and stores the three returned tuple columns in GarmentSpecification. That
record is public and Clone, not serde; generation clones the entire
specification into each ClothingShell and never recalculates appearance.
`generate_clothing_shells` validates body/rig/fitted geometry, not PBR fields.
`clothing_material.rs` contains ten authored surface entries; PolishedSteel
alone derives from Metal::default. Wool .92 roughness and alpha one remain
valid catalog data even though Metal recipe roughness ends at .9.
EquipmentMaterial also controls class/placement semantics, so retaining that
selection beside explicit appearance does not establish that they must always
be numerically identical. Do not invent such an invariant.

Actual receiving paths are `preview.rs:45-55` (clothing copied PBR),
`preview.rs:99-117` (component PBR then complete textile replacement),
`character_morphs.rs:184-203` (component PBR replacement then textile
finish/textures), and `rigged_clothing:206-230` (copied specification PBR).
`textile_surface.rs` validates fabric before conversion; preview supplies
alpha blend from base opacity, back-face culling for a closed thin shell,
Rgba8UnormSrgb color and Rgba8Unorm normals. Rasterized color replaces the
base factor with white. Exported TextileMaps retain translucent/chart-boundary
classification, front-only sides, and wrapped/open garment sampling.
`export/shell.rs` writes linear color factor, metallic and roughness, then
applies texture data; `export/textures.rs:180-185` replaces the factor with
white when color texture bytes are present. No double tint should be introduced
by making the PBR fields typed.

## Connected actionable authority gap: MCF-1 (P2)

At `components.rs:111-120`, material and textile are independent public,
serialized optional fields. The constructor paths above initially make them
agree. That is not a maintained record invariant: changing only textile.finish
from Wool to MetallicThread leaves retained PBR metallic zero/.92 while
`preview::spawn_armor` recomputes metallic .8/.65. Export's
`character_morphs::component_materials` copies the old PBR and adds only textile
finish/textures. This is a concrete representable API/serde state and an actual
receiving-path disagreement, not a claim that current healthy constructors
already produce it.

Governing rules: one semantic authority; use enums for closed roles with their
payloads; invalid aggregate combinations rejected by the representation; carry
typed owner through retained producers/consumers. Merely promoting the PBR
record leaves this gap open.

Smallest coherent correction: component appearance has one authority for
inherited/plain-PBR/textile cases, with textile-derived effective PBR obtained
from the actual textile recipe for every receiver. A single optional tagged
appearance payload can express inherited absence, direct PBR or textile;
local ClosureMaterial can consume that same authority or an equivalent borrowed
view, not cache a competing mutable PBR copy. Migrate constructors, mutation,
serde, preview, Tactical and export directly; do not retain old independent
fields, compatibility decoding, an alias, a duplicate Creator PBR record or a
temporary tuple adapter. If Root chooses another representation, it must prove
one authority and identical receiver precedence. Current fields permit
material None plus textile Some, or both absent; absence semantics
must remain explicit. Construction does not need to synthesize missing-PBR
errors when textile itself determines the PBR.

Acceptance: every present textile derives alpha/color/metallic/roughness from
that recipe in all receivers, with finish and textures from the same recipe;
plain PBR remains independent of textiles; inherited appearance remains
possible; malformed source still reaches the same typed admission boundary.
No new cases/tests/helpers/assertions are authorized. Existing fixture API
adaptations must retain all current case data/oracles.

## Geometry/coordinate association contracts

`FastenerMesh::point` appends position, normal, optional UV and ClosureSide in
one operation. Component ranges are recorded after each cord/bead append.
Cord U is polygon-column fraction; V is station/path-length fraction, not
physical metres. Bead UV is a local normal projection. Lacing cord changes
only the latter half of retained ClosureSide owners to Right; it does not
change vertex order or appearance. These arrays and ranges must remain
associated through any component representation migration. Do not reinterpret
these dimensionless UVs as the trim's physical along/inset coordinates.

`tiling/mod.rs:49-96` validates construction before geometry. Solid returns the
original fitted piece unchanged; small plates require spanning grids and
produce independent plate and optional lacing meshes before trimming. Layout
uses physical plate/radius input and MAX_TILES. Tile UV is template physical
coordinate * canonical Metal::TILES_PER_METRE plus a named deterministic scatter
fraction. Do not scale scatter itself by metres or treat it as a seed word.

`tiling/lacing.rs` reads actual canonical Lacing.radius in local metre geometry.
It lays running/hanging cords through plate holes, subdivides physical distance
with the .006-metre STEP, interpolates grid coordinates with cyclic-column
wrapping, and accumulates realized physical cord length. Its U is accumulated
length * texture repeats/metre; its V is polygon-side fraction. Closed cords
reuse rings cyclically; caps reuse the first ring UV. `embedded.rs` realizes
positions/normals in exact retained vertex order; skin weights and every morph
use that same order; UVs are collected from the same Embedded vector. Output
components/grids are empty, trim absent. Material promotion must preserve this
existing association and empty output policy.

`parametric_equipment.rs:179-224` admits construction, zips generated lacing
with the selected construction's actual optional recipe, and keeps both in
SelectedLacing. Non-steel is Solid. Trim applies only to steel with a selected
trim; construction narrows it, `armor_metal::trimmed` returns a new mesh and
its correlated per-vertex texture coordinates, and SelectedTrim retains the
selected metal plus those coordinates. Finish keeps resulting generated armor,
trim and lacing together. The current result tuples are semantic records and
remain MAT-R2 migration work; use a named output carrying both mesh and
coordinates rather than fabricating independently reconstructable columns.
Existing fixture bodies explicitly check coordinate count, morph counts,
component ranges and valid skin associations; they were read, not executed or
expanded. The entire remaining tail was read, including all fitting order and
unchanged fixture behavior.

This does not clear the independent physical algorithms: layout/surface/
template/tile-map implementations were not freshly read in this extension;
there is no full proof here of subdivision capacity, generated index overflow,
normalization/finite geometry, closest-point heuristics or all source-grid
memberships. The hard-coded local metres/angles, numerical thresholds,
closed/leaving flags and tuple candidates in the lacing math remain explicitly
unverified as a complete physical semantic migration. They are not waived by
being unchanged, and not all are dependencies of replacing PBR storage.

## Canonical owner decision and dependencies

Reuse and promote the one existing ArmorComponentMaterial record. A focused
renderer-independent Armor surface-material module is a suitable lowest shared
owner: both textiles and components already live in Armor and Creator already
depends directly on Armor. A direct re-export of the actual moved definition is
fine; keeping an old compatibility type alias is forbidden. Do not move PBR
into Creator, the renderer, GPU resource owners or physical ShellMaterial.
There is no need for a new high-level crate or a Bevy dependency in this owner.
The current documentation calls this PBR record "Unlit" despite actual lit
Bevy/glTF use; correct that description when moving its owning definition.

The record should own encoded float RGBA plus metallic and perceptual roughness
through the shared semantic leaves. Preserve Color's separate u8 authored
storage and proposed float encoded owners; Math generic Ratio does not by itself
prove [0,1]. The accepted design uses terminal material validation and unit-only
leaves; do not apply Metal-specific ranges to the shared PBR surface. If a
stronger checked shared PBR admission is chosen, inventory every public/raw
constructor and serde path first, including TextileAppearance::material's
current unchecked opacity and all authored catalog values, and propagate its
fallibility directly. No silent policy tightening.

Actual Cargo accounts show Armor and Creator do not yet directly depend on
fabelgeist-color; adding that canonical owner requires their real manifest/lock
composition, as already tracked in Root's design inventory. Armor already owns
Math and serde. Both workspace locks and affected consumers must agree before
claiming closure. Exact final public name/visibility and the component tagged
appearance schema must be recorded before Root starts implementation.

## Thirteen sequential lens dispositions

1. **producer-propagation**: OPEN: replace clothing_material tuple,
GarmentSpecification raw PBR columns, all fastener literals and retained/export
PBR columns with one canonical record; native Bevy/glTF projection only at real
adapters. Keep authored textile/color conversion at producer.

2. **existing-type-owners**: RESOLVED DESIGN DEPENDENCY, NOT IMPLEMENTED:
move/reuse ArmorComponentMaterial as one renderer-independent Armor PBR owner,
no Creator duplicate or old alias; reuse proposed canonical Color Srgba and Math
Ratio/Metres. TextileColor u8 remains a distinct authored storage owner.

3. **admission-contracts**: OPEN/POLICY EXPLICIT:
TextileAppearance/ClothFastener/ClothLacing/ArmorComponent public fields and
derives are recipe input, not checked records. Existing terminal validation
retained; material() itself is infallible/unvalidated. A new checked PBR API
cannot silently rely on those derives or clamp opacity.

4. **invariant-operations**: OPEN: alpha/finish replacement must construct
coherent typed PBR, preserve arithmetic order; trim/mesh coordinate association
must survive split/permutation. Existing native geometry algorithms are not
certified by this extension.

5. **aggregate-states**: ACTIONABLE MCF-1: component.material and textile can
disagree and actual preview/export choose differently. Named trim results must
keep output mesh and coordinates together. Current constructors produce aligned
initial states; public mutation/serde bypass that relation.

6. **type-ownership**: RECOMMENDED CANONICAL PLACEMENT: focused
renderer-independent Armor surface-material responsibility, reexport actual
moved type once; Color owns transfer/encoded components, Math owns generic
units, geometry owns vertex attributes. No dependency on Creator/Bevy/GPU for
PBR identity.

7. **redundant-authority**: ACTIONABLE MCF-1: store plain PBR or textile
appearance once and derive effective PBR for every receiver. Catalog
EquipmentMaterial may remain alongside explicit appearance because it also
controls class/offset; do not infer an undocumented equality invariant.

8. **behavior-preservation**: ACCEPTANCE REQUIRED: preserve 8-bit sRGB division,
arbitrary float sRGB catalog precision, opacity, finish .92 roughness,
Metal-specific .9 maximum only at Metal admission, white base factor when
rasterized, texture alpha/sampling, geometry order, and valid absence states.

9. **enum-shapes**: OPEN MCF-1: exclusive plain/textile component appearance
closes neighboring payload authority; optional inherited appearance remains
legitimate. ClothFastener/LacingEyelet and Construction already own variant
payloads. Unrelated Lacer leaving bool/closed state not granted clearance.

10. **derivability**: NO NEW MANUAL-TRAIT NEED: current Clone/Copy/serde derives
fit wire and value behavior; retain authored defaults. If a checked leaf is
selected, derive through its checked conversion rather than bypass admission.
Cohesive same-error Result aliases remain MAT-R3 extension work.

11. **type-granularity**: OPEN: PBR record with typed encoded color/ratios,
distinct metre quantities and dimensionless UV/repeat coordinates. No
per-material duplicate PBR wrappers, no generic RGB owner for u8/float storage,
and no GPU launch/sort count reused for CPU geometry ordinal roles.

12. **magic-strings**: RETAIN: names and suffixes are actual exported
presentation/file identities, serde enum tags and glTF keys are actual external
protocol. No observed branch on mutable error prose in these sources. Existing
typed errors remain; do not replace diagnostic strings with artificial closed
identifiers.

13. **magic-numbers**: OPEN/BOUNDARY CLASSIFIED: PBR finish/catalog coefficients
are authored values projected through canonical typed leaves; opacity/255/1000
conversions have source-owned encoding meanings; Metal density is canonical
inverse metres. Named texture scatter is intentional UV offset. STEP/geometry
policy and physical count migrations remain separately open.

## Adjacent governing-rule inventory

MAT-R3 extends to same-error modules actually read here: clothing uses explicit
ClothingError throughout; textile_pattern, cloth_closures and cloth_lacing use
GenerateError; tiling files use ConstructionError. Prefer owning cohesive
Result aliases when changing these APIs, without erasing typed distinctions.
The anyhow-based Creator geometry modules already import anyhow::Result.

MAT-R4 exact source locations include textile_pattern's ChartBoundary impl
before subsequent structs, components' Role impl before Hinge/Component/PBR
structs, construction's enum/impl before its later records, tiling/mod's mod
items before imports, tiling/lacing's Style enum before subsequent structs,
and parametric_equipment's functions/SelectedArmor impl before later records.
Keep docs/attrs with moved items. Creator clothing has about 530 production
lines before its tests, so materially expanding it requires extracting a real
responsibility first; changing only retained PBR field shape need not invent a
pass-through module. Do not add growth exemptions or baseline ceilings.
Existing `#[allow(clippy::too_many_arguments)]` in clothing is adjacent debt;
if altered, use a narrowly reasoned expect or real responsibility change,
without adding a broad suppression. No assertion/helper/test addition is
needed for any of these source repairs.

## Required connected acceptance and limits

Before implementation, Root must compose this extension with its already
source-backed material ledger, make the single PBR/appearance owner and API
choice concrete, record each constructor/mutator/serde/receiving migration,
and keep physical geometry findings separately explicit. The shared record
must reach catalog returns, garment state, all fastener/eyelet constructors,
components, preview, Tactical, rigged/export state and every texture/PBR
replacement path. One typed definition with raw retained duplicates is not
completion. Imported Color/Math owners and both lockfiles must be reconciled.

All original material findings (including density/cause, units/counts/seeds,
closed engraving schema, Result aliases and item order), native scalp/Grid,
generated family composition and final review/hosted-CI requirements remain
open until Root proves them on the final pushed revision. This report grants
no global correctness, migration-completeness or CI verdict. Full exporter,
Tactical material selection, textile rasterizer/relief internals and physical
fitting/layout algorithms are not newly cleared by this extension. No checks
were unavailable or attempted; execution was explicitly prohibited.

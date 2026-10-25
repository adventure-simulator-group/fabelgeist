# Material semantic producers and consumers: MAT-R1 / MAT-R2 discovery

Discovery only. **No whole-graph or implementation clearance.** No source edits,
new tests/cases/helpers/assertions/doctests, project executions, builds, scanners,
probes, or CI requests were performed. Existing fixtures were read as source.

Worktree:
`/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator`.
Final source-account HEAD: `47c23647be6d0e764076237b8a0a692906663d0a`.
The root committed earlier work during this read-only discovery. The fingerprints
below identify the material bytes inspected; HEAD alone does not pin dirty bytes.

Applied the Comment Review skill, repository and crates rules, and the shared
Armor equipment guide. This is maintenance/source-contract discovery, not a new
shape or artistic iteration. No existing report or search was used as clearance.

## Evidence accounts and occurrence frontier

Freshly read 54 complete files and the explicitly bounded regions of 12 other
files. Exact fingerprints and read extents are in:

- `/tmp/pr780-material-semantic-consumers-source-accounts.json`
- `/tmp/pr780-material-semantic-occurrences.txt`
- `/tmp/pr780-material-semantic-boundaries.txt`
- `/tmp/pr780-material-semantic-symbol-occurrences.txt`

The occurrence files record repository-wide Rust searches for material recipe
fields, associated constructors, constants, symbol imports, trim operations,
PBR calls and depth packets. They are occurrence/frontier evidence, not proof
that an inferred type has no additional callers. Textile motifs, weapon ornaments,
cloth-lacing `Metal` variants and hair trimming are distinct homonyms. Complete
receiving reads are distinguished from occurrence-only entries in the accounts.
No new fixture was created by those source searches.

## MAT-R1: exact authored and native contracts

The field owner is `fabelgeist-armor/src/material.rs:15-33`. Construction paths
include `Metal::default`, `Trim::default`, catalog `armor_metal::metal`, serde,
UI mutation and existing GPU fixtures. `ValidatedMetal` is an immutable borrow
admitted through `Metal::validate`; it does not carry semantic leaves through
upstream producer fields. Its validation is useful and must survive a unit-only
migration. The shared Math units explicitly impose no finite/range policy.

| Field or API | Actual meaning and bounds | Disposition / connected consumers |
|---|---|---|
| `Metal.color: [f32;3]` | Opaque artist-facing sRGB channels, each finite 0..=1. Default `[0.769,0.776,0.776]`. | Carry an encoded three-channel owner. Creator `color_edit_button_rgb`, catalog PBR, `Color::srgb`, trim export append alpha 1. No color enters the metal bake kernels. |
| `roughness` | Perceptual roughness coefficient 0.08..=0.9. | Ratio/unit owner plus existing material bound; default0.20. Preview map multiplier is 1, export map application also sets roughnessFactor1. |
| `scratch_density` | **Count of scratches in one tile**, 0..=3000, not per-metre frequency. Native workgroup-grid Y is this count. | Count owner must allow zero. Preserve count-times-five draw offset. Do not use `PerMetre`, `InvocationCount`, strand count or sorting quantity. |
| `scratch_length` | Fraction of tile length, 0.005..=0.4. Shader `span = length * size * random_reach`; reach0.3..1. | Dimensionless ratio, not metres. Preserve steps/allocation projection and exact arithmetic. At four tiles/metre the nominal physical span is this fraction/4. |
| `scratch_width` | Radius-like **texel** coefficient0.5..=3 used as divisor of Euclidean texel distance, with fixed stamp reach3 texels. | Distinct from metres and tile fraction. Physical width changes with bake size under existing behavior. Do not silently make it resolution-independent during typing. |
| `scratch_depth` | Nonnegative normalized groove coefficient0..=1, scaled by random0.4..1 and sin envelope; feeds slope/roughness. | Ratio/coefficient, not engraving cut depth in metres. Preserve positive-float atomic-max native representation. |
| `scratch_angle` / `scratch_spread` | Radians; angle -pi..=pi, spread0..=pi. Swing is `(draw-.5)*spread*2`. | Shared radians quantity plus existing authored bounds. Never substitute degrees or radians/metre. |
| `seed` | Unsigned32-bit word, all bits admitted, default1544. | Preserve the wrapping32-bit LCG and skip offsets. UI is `DragValue<u32>`. See seed-owner reconciliation below. |
| `waviness` | Metre-valued amplitude of planishing field,0..=.002; default.0004. | `Metres` carries the unit; source validation retains finite/range policy. GUI displays mm and divides by1000. See comment mismatch below about exact peak-to-peak wording. |
| `grain` | Strength coefficient0..=1; default.35. | Ratio; native slope and roughness contributions retain existing coefficients. |
| `smudge` | Roughness modulation coefficient0..=.4; default.08. | Ratio plus material-specific bound, not a distance. |
| `Metal::TILES_PER_METRE` | Spatial frequency4/m. | Canonical `PerMetre` candidate. All actual consumers are listed below; extracting at WGSL formatting and UV numerical arithmetic is legitimate. |
| `Engraving.tiles` | Dimensionless image/cell repeats per metal tile, .25..=64; default1. Fractional repeats are allowed. | Ratio, not integer motif repeat count or native workgroup count. Carries through GUI, trim fitting, image address admission, ornament native params. |
| `Engraving.rotation` | Radians -pi..=pi; default0. | Same angle quantity as scratch angles; sin/cos are numerical boundary coefficients. |
| `recess_roughness` | Added roughness at the floor,0..=1; default.35. | Ratio retained through cut recipe and bake; not a physical cut depth. |
| `Relief::Height.depth` | Metres0..=.005; default etch.0004. Zero is valid. | Metres, not positive-only length. Native cut slopes divide by physical texel spacing. Preview depth is dimensionless after multiplying by4/m. |
| `Relief::Normal.strength` | Dimensionless slope multiplier0..=4; default1. | Ratio plus bound. This route produces no parallax depth map. |
| `Ornament.repeats` | Integer repeats per engraving cell1..=8. Picker suggestions2/3/4/6 depend on motif. | Distinct positive bounded count; not scratch count and not fractional `tiles`. |
| `Ornament.line` | Fraction of cell cut-line width .01..=.2; default.06. | Ratio. Native half width/fillet margin derive from it; preserve existing antialiasing coefficients. |
| `Ornament.fillets` | Application policy: no fillet lines or one along each long side. Defaultboth. | Named role enum is appropriate; native `u32 !=0` and GUI checkbox are actual adapters. Preserve the two-side meaning, not an arbitrary generic enabled flag. |
| Wave/Zigzag/Guilloche/Vine `amplitude` | Fraction of room between fillets0..=1. | Ratio; no metre conversion. |
| Guilloche `strands` | Integer count1..=6, default3. | Distinct motif strand count; cannot reuse hair strand count by name alone. |
| Rope `slant` | Dimensionless repeat shift across room height,.1..=3. | Ratio; not radians despite UI label Twist. |
| Beads `radius` | Fraction of available repeat/room radius,.1..=1. | Ratio; not physical radius. |
| Vine `leaf` | Fractional size coefficient0..=1. | Ratio. Native min with soft coverage is intentional existing policy. |
| `Trim.width` | Metres .003..=.06; default.02. | Carry unit into serialization, GUI, construction `trim_on`, `narrowed`, trim geometry and UV production. |
| `TrimBand.width/period` | Finite **positive metres**, independent geometric admission. | Existing `PositiveLength` is a matching admitted-owner candidate; Math `Metres` alone requires retaining this check. No new equivalent positive-length owner. |
| `Trim.period()` | Metres `1/(4/m * engraving.tiles-or-1)`. | Return physical length, preserve reciprocal arithmetic order. |
| `Trim.cell_origin()` | Dimensionless metal-tile coordinate `.5-.5/tiles`. | Ratio/coordinate role, not length. Do not apply4/m twice. |
| `Trim.cell_tiles()` | Dimensionless repeat ratio fitting a cell to width, clamped .25..64. | Return ratio; clamp is existing supported-repeat policy. |
| `Trim.narrowed(width)` | Width in metres; only shrinks, scales engraving.tiles by old/new then clamps, otherwise clones unchanged. | Unit and public-operation admission must be inventoried. Bare Metres does not prevent zero/NaN/infinity. The current method can produce invalid authored trim for invalid public inputs. Actual construction caller supplies `Plate::widest_trim()`. |
| `DepthMap.uv_scale` | Dimensionless full-cut depth in metal texture coordinates, `height_depth *4/m`; finite nonnegative map admission. | Ratio; preserve Some-at-zero-height versus None-for-normal/no-engraving. Carry through preview record to Bevy `parallax_depth_scale`. |

Native evidence is `gpu/metal/bake.rs`, `textures.rs`, `finish.rs`, `ornament.rs`
and `wgsl.rs`, all read in full. Every scratch uses five draws (position x/y,
angle, reach, depth), native wrapping32-bit state, top24-bit fractional draws;
no distribution or hash algorithm change is justified by replacing the seed type.
Ornament native codes0..5, shape-field packing and fillet0/1 are explicit ABI.
No shader parameter slot, packed map lane, default recipe, random draw count or
texture-size policy should change in a semantic producer migration.

`TILES_PER_METRE` actual receiving source:

- `engraving.rs` physical-cell documentation.
- `trim/finish.rs` period, fitted repeats and cell origin relationship.
- `tiling/mod.rs:116` per-plate metric UVs plus existing scatter offset.
- `tiling/lacing.rs:283` accumulated cord length to UV coordinate.
- `gpu/metal/finish.rs:41,113,133` metre amplitude to physical slope.
- `gpu/metal/textures.rs:207,223,230` physical texel and cut slopes.
- `gpu/metal/bake.rs:263` height depth to dimensionless parallax scale.
- `armor_metal.rs:58,83` body-surface UV density and trim metric UV conversion.
- Existing `trim/finish`, GPU and Creator armor_metal fixtures retain original
  numeric expectations. They are verification inputs, not exempt production APIs.

## Canonical-owner reconciliation

1. `fabelgeist-math/src/units.rs`: `Metres`, `Ratio`, `PerMetre` are transparent
   scalar serde records and quantity tags, deliberately without numeric policy.
   They fit the unit roles above, but do not replace `Metal::validate`,
   `Engraving::validate`, `Ornament::validate` or trim bounds. `RadiansPerMetre`
   is not a scalar angle. No shared scalar Math radians owner was found in the
   inspected canonical module.
2. `fabelgeist-geometry::{PositiveLength,SignedLength}` already own finite metre
   admission. PositiveLength matches the geometric TrimBand positive contract;
   it does not match zero height depth. SignedLength admits negative values and
   needs application bounds for nonnegative recipe fields. Do not duplicate
   these guarantees under a new material-specific equivalent length name.
3. Building generator `spatial_geometry/orientation.rs:15` owns finite `Radians`
   tied to its GeometryError, axis/role and Bevy reflection. It is existing owner
   evidence but an unsuitable dependency direction from shared Armor/Math.
   A shared scalar angle decision must explicitly reconcile that overlap;
   importing radians/metre or writing a second arbitrary angle implementation
   does not resolve ownership.
4. `fabelgeist-determinism::Seed` stores64 bits, serializes a `{word:...}` record,
   derives domain-separated64-bit seeds and produces the crate's RNG.
   It does not itself constrain a raw word to32 bits or implement the metal LCG.
   `HairSeed` is a signed32-bit hair-specific projection backed by Seed; its
   signed JSON domain and hash owner are not the metal's unsigned seed contract.
   A matching fixed-width seed owner should live with canonical determinism,
   preserving every admitted u32 bit and the metal algorithm at the actual WGSL
   boundary. Do not silently truncate arbitrary Seed64 or substitute its RNG.
5. `fabelgeist-color::Color` is an untagged four-channel f32 vector. It has no
   sRGB3 admission/space contract. Procedural-textures `SrgbColor` is an opaque
   byte triplet, whose conversion quantizes; using it would alter current float
   recipes. Bevy `Srgba` is an actual framework adapter. Reconcile a shared float
   encoded RGB owner at the Color boundary; do not add alpha, linearize twice,
   quantize colors, or import the procedural texture crate to name this value.
6. Existing `fabelgeist-armor/src/components.rs:124` already owns the PBR record
   `ArmorComponentMaterial {base_color,metallic,roughness}`. Its shape and actual
   preview/export receivers match the proposed named catalog PBR record. That
   owner must be consumed or explicitly promoted/unified before adding another
   equivalent record. Its leaf quantities are still primitive, and its current
   name is specific to components; matching fields alone is not final proof that
   every broader material authority has been migrated.

Armor currently depends on Math and Geometry, but does not depend on Color or
Determinism; those two dependency additions and optional feature/serde behavior
are real migration dependencies, not implicit imports. Manifests were read.

## MAT-R2: named associations and complete observed receiving routes

### Catalog PBR

Producer `clothing_material::pbr` returns the positional
`([f32;4],f32,f32)` tuple. `PolishedSteel` consumes `Metal::default` color and
roughness; rough/oxidized/mail/leather/cloth/wood/lead retain their separate
catalog literals. Do not accidentally recolor them to the polished default.

Observed direct receivers, each source read at the receiving code:

- `armor_metal.rs:27`: takes RGB and roughness for plate-steel Metal recipes.
- `clothing.rs:92`: persists color/metallic/roughness on GarmentSpecification.
- `underlayer_preview.rs:49`: base catalog StandardMaterial or metal preview.
- `equipment_export.rs:85`: assigns factors on every exported plate shell.
- `character_export.rs:171`: assigns worn plate-shell factors, maps and trim.
- `adventuresim-tactical-client/src/equipment/runtime_equipment/meshes.rs:178`:
  **unchanged external crate receiver**, constructs runtime StandardMaterial.
- Creator `lib.rs:6` publicly re-exports this API as `equipment_pbr`.

A named PBR record needs named fields with shared encoded color and ratio leaves
and must reach every receiver directly. Keeping the tuple API plus a new record
or adding a temporary destructuring adapter would leave a competing path.

Existing component PBR consumers are already `preview.rs:103-108`,
`character_morphs.rs:185-205` and tactical `meshes.rs:67-79`. Export
`RiggedShell.base_color` documents artist-facing sRGB; `export/shell.rs:65`
calls `linear_base_color` once, whose implementation is `export.rs:578-588`.
Preview calls `Color::srgb/srgba`. The recipe must retain encoded values until
these framework/export boundaries. Alpha remains a separate channel and must
be preserved for non-opaque catalog/component surfaces.

GarmentSpecification and RiggedShell retain separate raw PBR fields. If the
chosen shared record replaces that retained state, every producer of those
structs becomes a connected migration family; the present occurrence/read
accounts do **not** claim that larger graph complete. Component material and
textile overrides must continue clearing or replacing plate maps as they do now.

### Trim mesh and its coordinates

Creator `armor_metal::trimmed` returns `(GeneratedArmor,Vec<[f32;2]>)`, produced
from the same trimmed mesh's per-vertex metric coordinates. A named record should
retain this association, and carries the generated mesh and its corresponding
UV collection together. It must not use the pre-trim vertex count after cut
vertices/seam duplicates/component regrouping are created.

Actual receiving paths:

- `parametric_equipment.rs:216`: stores resulting mesh and SelectedTrim with
  corresponding coordinates, metal and name.
- `armory_scene.rs:85-108`: pairs resulting UVs with trim preview material.
- `parametric_equipment.rs:444`: existing fixture adaptation only.
- `character_export.rs:191-210` and `studio_generation.rs:31-43`: consume the
  retained SelectedTrim coordinates on band shells/preview.

Underlying `GeneratedArmor::trimmed` owns finite-positive band admission, face
count and already-trimmed checks; Rims and Split (read in full) weld positions,
find rim distances, rescale closed-rim along coordinates to whole periods,
interpolate cut vertices/skin/morphs, unwrap seam copies and regroup components.
`ArmorTrim.coordinates` is retained as `[along_metres,inset_metres]` per vertex;
that existing positional metric coordinate pair is a related owner decision.
Native mesh/UV arrays may be numerical adapter representations, but the Rust
mesh-plus-UV return is not a framework-required tuple.

Construction `trim_on` calls `Plate::widest_trim` then `Trim::narrowed`; the widest
trim is `.15 * min(width,height)`, with minimum authored trim width. `Trim::width`
and `TrimBand::width` therefore cannot be repaired only at the final call literal.
The construction fields and geometric kernel entry are real connected producers.

### Preview depth

`DepthMap` already names pixels and scale. `MetalImages.depth` then reduces that
association to `Option<(Handle<Image>,f32)>`, maps decoded pixel ownership into a
handle, and unpacks it again into Bevy depth_map/parallax_depth_scale.
A named preview depth record should retain the handle and dimensionless scale.
No need to introduce an enum per channel or change the image/upload policy.
`ShellTextures::armor` intentionally exports normal and metallic-roughness PNGs
only; it does not export this preview parallax map. Do not claim export parallax
or invent an unrelated export feature as part of the record repair.

### Motif native packet and bounds

`Motif::words` returns `(code,first,second,strands)`; `record_ornament` assigns
those values to named PassParameters. A named native motif packet can preserve
these exact four slot roles. Codes remain0..5, unused shape slots remain0, and
single-strand defaults remain1. The actual ABI is native scalar fields, not the
Rust tuple. Do not create an independent second motif-to-code mapping in the GUI
or GPU module. The semantic Motif enum already owns the variant payloads.

The heterogeneous `Metal::validate` bounds table carries `(value,low,high)`
triples; shared named bounds or direct typed field validation can remove positional
roles. Do not type-erase all metre/ratio/angle leaves into raw records merely to
retain the table. Keep zero-enabled versus positive counts and per-field ranges.

## GUI and serialized producer obligations

`metal_controls.rs`, `ornament_controls.rs`, `decoration_controls.rs` and the
inventory article editor were read in full. The UI edits real recipe state,
not a separate persisted GUI model. The generic `slider(&mut f32,RangeInclusive)`
is an actual egui bridge; construction_controls uses it too. A changed helper
signature must account for that unchanged consumer. Project a typed leaf only
at the local egui boundary, retain the existing min/max policy and read it back
without changing the recipe unit. Millimetres are presentation units for physical
fields, while scratch width remains texels and scratch length remains a fraction.

Picker behavior: selecting another motif replaces its variant at its canonical
`Motif::ALL` proportions and resets repeats to suggested_repeats. No reset when
selecting the current discriminant. Adding trim uses Trim::default. Adding its
engraving fits repeats to current band width; the Fit cell button repeats that
operation. Switching source to procedural ornament forces height relief; image
source edits retain path and permit height/normal choices. Optional engraving
and optional trim are independent; they must not become mutually exclusive.

Wire shapes are current serde records: Metal/Engraving/Ornament/Trim derive
serde; Motif/Relief/ReliefSource use snake_case externally tagged enums. The
source and relief are independently stored, allowing Ornament+Normal in memory
or decoded JSON until validate rejects it. A future source enum repair must
carry its payload and update the picker/serde/native Cut consumers together.
No backwards-compatible alias/fallback is warranted by the user instructions.
Quantity wrappers can preserve the scalar JSON values without a compatibility
layer. A fillet enum may use an explicit bool serialization adapter if that is
the clean chosen wire schema, distinct from keeping a primitive application API.

DecorationLibrary load/save/insert validates its nested engraving/trim and stores
copies by LibraryName. Choosing a library entry copies it into CatalogArticle;
recipes do not retain an external library link. CharacterRecipe contains
Inventory -> CatalogArticle -> Decoration, and main's load/save/offline entry
validates recipes after decoding/before saving. Inventory validation currently
formats some causes into Strings; that is a separate causal-error finding,
not resolved by this quantity/record migration.

The museum `trim.json` files read here use a distinct authored review shape
with `width_mm`, hex color and defaults/items. Their keys are not the serde
fields of `trim::Trim`. Museum body snapshots found by the occurrence search
carry null engraving/trim, not another positive material recipe. No reader for
those review trim files was audited; do not rewrite them as production serde
fixtures on the basis of the filename.

## Comment findings to include before repair

- **MAT-SEM-C1 (P3), `material.rs:25-26`: exact peak-to-peak waviness claim.**
  Current wording calls waviness the peak-to-peak height of the planishing
  undulation. `gpu/metal/finish.rs:110-115` scales the derivative of the sum
  `wave + .35*ripple`; no extrema normalization gives that combined field the
  authored exact peak-to-peak range. Describe a metre-valued height/amplitude
  scale for the planishing layers, rather than a measured exact peak-to-peak
  height, while preserving numeric policy. This is not a request to alter the
  native finish recipe to fit prose.
- **MAT-SEM-C2 (P3), `components.rs:122`: "Unlit surface parameters" is stale.**
  Preview and tactical consumers construct normal StandardMaterial PBR values
  from these color/metallic/roughness factors; they do not set unlit=true. Export
  writes glTF pbrMetallicRoughness. Describe PBR surface factors, with generated
  textures supplying normal/visibility detail. Reconcile this wording with the
  existing PBR owner instead of adding a second competing record.

No claim of measured anti-aliasing quality, hardware performance or historical
acid-etch depth was independently verified by this source-only semantic task.
Those unchanged external/numerical prose claims remain outside this discovery.

## Dependencies and acceptance for Root's all-thirteen inventory

1. Choose canonical leaf and named-record owners first, including the existing
   component PBR owner, shared angle, native32 seed and float encoded color
   contracts. Record why each reused owner matches or differs. Keep count,
   physical length, tile ratio, texel coefficient and launch population distinct.
2. Inventory each construction, serde, GUI mutation, retained state, arithmetic,
   numerical projection and error path in the receiving sources above before
   edits. Related construction/geometry/PBR graphs left partial here remain
   dependencies to read, not accepted exceptions.
3. Complete the recipe/trim/depth/motif/PBR families through unchanged callers,
   including tactical equipment. Remove replaced APIs directly; no adapters,
   duplicate implementation, alias or migration fallback. Preserve unrelated
   component/textile overrides and all existing valid defaults/catalog values.
4. Preserve exact numerical/native behavior: shader strings and ABI fields,
   LCG sequence/draw counts/seed bits, encoded color values and one conversion,
   signed image sampling admission, trim cut/seam/component association, zero
   relief behavior, clamp policy and dimensions. Establish source comparisons
   for each changed numerical handoff. Typed conversion alone is not proof.
5. Existing fixture bodies/expectations are acceptance evidence only. Adapt
   their required constructors/field API without adding tests, cases, helpers,
   assertions or doctests. Run only the hosted project checks required by the
   authorized root workflow after connected families are complete.
6. Re-read resulting source and complete independent Code, Rule and Comment
   Reviews plus all thirteen lenses. The present task does not clear MAT-R1,
   MAT-R2, MAT-C1/R3-R5, native scalp/Grid, or broader PR work. Required checks
   and final reviews must eventually cover the same final pushed revision.

## Remaining read limits

The exact source accounts identify 12 partial files. In particular the full
GarmentSpecification/RiggedShell constructor graph, component PBR authored
producers, entire inventory/loadout validation graph, all native tiling/lacing
geometry and the full exported GLB writer/fixtures were not completed here.
Only the received material roles in those sources are supported. Repository-wide
occurrence searches can miss aliased or inferred receivers and are not clearance.
A fresh complete dependency audit is required before claiming either MAT finding
closed. The next all-thirteen ledger should record each remaining region and
its acceptance disposition, rather than convert this discovery into a pass.

## Exact source fingerprints

| Source | Read extent | SHA-256 |
|---|---|---|
| `crates/fabelgeist-armor/src/material.rs` | full source | `bf177dcb1b678b06f9174b3aa283e4aa7634638681ce88c8722a900052734d72` |
| `crates/fabelgeist-armor/src/material/textures.rs` | full source | `89a5b76b6d0a2783ce6dbacb7329e7a2cda93ddceaf4fc12c060e7ff2c96036e` |
| `crates/fabelgeist-armor/src/material/tile.rs` | full source | `a094fe52ceba98b71a00ae2fa974ba24cf769131e3c265373727f353f3885869` |
| `crates/fabelgeist-armor/src/engraving.rs` | full source | `c612e940bd0898395fad63b7d819dce7b863022f3258ade844f72d8a6a1c3dd2` |
| `crates/fabelgeist-armor/src/engraving/decoded_image.rs` | full source | `7ef1b572bf5a5c2e5684487da81194d1793c4025654f099c0898d97dfba064f1` |
| `crates/fabelgeist-armor/src/engraving/sampling.rs` | full source | `d042b91ea2f8b98dd53b7733dd4f5f11a282b4763211f3ca3288b2e18879d548` |
| `crates/fabelgeist-armor/src/ornament.rs` | full source | `74d886336dc4fbeeb4f9ff4ec29530a32fef42865dcb7ef2f6b2220b678c1ebf` |
| `crates/fabelgeist-armor/src/trim.rs` | full source | `81abc6494df0d4f051b0c4548517fe80061204de7e90d9d2eb190635a12de5f2` |
| `crates/fabelgeist-armor/src/trim/finish.rs` | full source | `5ce0e0d224987f30347d1f4ae5feaf79f58a61aa3eec607ca22ee0ad7fe5c34b` |
| `crates/fabelgeist-armor/src/trim/edges.rs` | full source | `ffdc73a54de37023064d476bba8fbf666fe57f7597a910468c96fa3e9f63b2c7` |
| `crates/fabelgeist-armor/src/trim/split.rs` | full source | `44aafa5468b01108bee89840514846df79aa2075ea6c2e154b21d93cee321b9a` |
| `crates/fabelgeist-armor/src/construction.rs` | full source | `c589129c7386c136feaf7f63fc21ed925f81dd035642ea3dacdc92d054659854` |
| `crates/fabelgeist-armor/src/components.rs` | full source | `0cee8ea4265d2ec61509c2c7a516a7dfa6d9d46a483d83fdae928bd1fe010ad5` |
| `crates/fabelgeist-armor/src/tiling/mod.rs` | full source | `7ff5bfe0408afa8433957ee7575dd1fbb9f83d6afe4bb3af59a581a81724280d` |
| `crates/fabelgeist-armor/src/gpu/metal/mod.rs` | full source | `b94c9bc6659a4ca3a7e9ef3bfbe5a52026c70dd86abd4c40f91c5066c9ebde8e` |
| `crates/fabelgeist-armor/src/gpu/metal/bake.rs` | full source | `09072dcebe7d27d7866f20305953af487020335484821994e5b90db655683268` |
| `crates/fabelgeist-armor/src/gpu/metal/finish.rs` | full source | `d49752ba70390e3801be29f335f22fea33532331b7e73c555474f8a55e866741` |
| `crates/fabelgeist-armor/src/gpu/metal/textures.rs` | full source | `e6e7f6eb9a2ef8dbb4d28132efa4d82f95ef220730cc20d12ccd376c43a14a9d` |
| `crates/fabelgeist-armor/src/gpu/metal/ornament.rs` | full source | `6a9f887c5c746b8b70fc8a3c9741ffa1a68dd3580a61483ca88fbc244dcb4152` |
| `crates/fabelgeist-armor/src/gpu/metal/wgsl.rs` | full source | `90083b6c258f47bcf6e52babf50abc2cda917c32ea982b0404edcd75b8157f83` |
| `crates/fabelgeist-armor/src/gpu/metal/tests.rs` | full source | `fdc04f19250688292a3a3f1830e0b6ca7cdab927e166dd2940ddd549f6587e9c` |
| `crates/adventuresim-character-creator/src/metal_controls.rs` | full source | `fa3594e3f5dc55f7ad90ba02d05fedfff207c5a0d394e03faad2da235b3d6d14` |
| `crates/adventuresim-character-creator/src/ornament_controls.rs` | full source | `c10940e30bdb0ae77feaca37d5d33b6aac96e7cd7a97a48fc729c69be81aad3a` |
| `crates/adventuresim-character-creator/src/armor_metal.rs` | full source | `ae70b9acd477b6b29e03bd540b451d5e8556222a33a7cbb10babafab1206fbba` |
| `crates/adventuresim-character-creator/src/clothing_material.rs` | full source | `879486f9beb55903e6b6788555437f70413b4449dbf5ebfcf6295b1a81e5d2b5` |
| `crates/adventuresim-character-creator/src/metal_preview.rs` | full source | `09af59779312ad82dd8a82bace7284c98503db7e55eab5018e1deee5dae3bf14` |
| `crates/adventuresim-character-creator/src/underlayer_preview.rs` | full source | `48e7ef34b4f757817c54281efd85160d1920dcb9e61fd8536113fd0927e879ed` |
| `crates/adventuresim-character-creator/src/equipment_export.rs` | full source | `2906c6c5115f7924743595c1808b2337d4b167c27d301436ac22304c75479428` |
| `crates/adventuresim-character-creator/src/character_export.rs` | full source | `33f6b05450b7b1eb8e1080aa330b35696e6ee4c57da0bb4d208a18f66ebec963` |
| `crates/adventuresim-character-creator/src/decoration.rs` | full source | `7b677abc0f1a167c6b25343e2964d611aa479132a32330ebd2bd8b03fbf2619e` |
| `crates/adventuresim-character-creator/src/decoration_controls.rs` | full source | `ca0d51e0b1984ca87534afb472882f25280b26fb445232ec0e3e178920e3f3a2` |
| `crates/adventuresim-character-creator/src/inventory_article_editor.rs` | full source | `bfaf8082b9b0e3aba214dbe23016ae3d83f78b3a66f777ea6e2601ddc5475cf9` |
| `crates/adventuresim-character-creator/src/construction_controls.rs` | full source | `3f3fa3760c04b76557e2a6927b461eb83c134a41d561952d22579c3f3f286599` |
| `crates/adventuresim-character-creator/src/armory.rs` | full source | `aa66e0b18f919eb3351635241249df969eeed4c36c36aa302888c320222a54df` |
| `crates/adventuresim-character-creator/src/armory_scene.rs` | full source | `aad333ef6af184900a6ac5da03ab5dea1e6bea08a26dea86cd8da164b37b6fbc` |
| `crates/adventuresim-character-creator/src/studio_generation.rs` | full source | `3031832a29165a676c81ee7b80a78ee5463dab02dbc69bdd959cd8bd3fd5d066` |
| `crates/adventuresim-character-creator/src/export/shell.rs` | full source | `93b0aa654eacc98d66c072b54cb1a1d0444b75e6bdc3cc0ecd7d76325f085bd6` |
| `crates/adventuresim-character-creator/src/export/textures.rs` | full source | `1d5fcb4d3997b5c4137ad31b6fc871cc065f0b8ee3689cb87beb35e64b8e07fd` |
| `crates/adventuresim-character-creator/src/export/rigged.rs` | full source | `b53b6a874c7a23f38f7895c75a03c65c599c21e122d45b1843e5b3efa2b3a25e` |
| `crates/adventuresim-tactical-client/src/equipment/runtime_equipment/meshes.rs` | full source | `4994b1974149becac0c96f9e0dafeb0d26c1db776cc4adef9d2e1b388b033058` |
| `crates/fabelgeist-math/src/units.rs` | full source | `0c76a1a79a18e833d21fc2ffb011e2a351139ee7a299df13df12fdbcf3c27492` |
| `crates/fabelgeist-determinism/src/seed.rs` | full source | `ad0f8971f300e87ee98b0cd1d44273bb361c576566444a2b45216f9ddb65dff4` |
| `crates/fabelgeist-color/src/color.rs` | full source | `ec95ce41e0de2b14f31c2d8a206461d2ed69f95183e18ae020c7ce956de86c74` |
| `crates/fabelgeist-color/src/lib.rs` | full source | `2aa390bfe12f98f358c698461b399a3f3f02d3dfc2a2edf6be3c761b88d556fa` |
| `crates/adventuresim-procedural-textures/src/palette.rs` | full source | `ed8f9afcc9b1ee24aa5e7d8aa42160703e0e6132c99775df7e29fe2e71f7eea8` |
| `crates/adventuresim-building-generator/src/spatial_geometry/orientation.rs` | full source | `05f48ed8a27959acd5930fc56690d454e5307482f673cbdb5628d88970c3db1d` |
| `crates/fabelgeist-geometry/src/length.rs` | full source | `7ce3b7a0720e800087df009c9726c0869988dfad3c24cc4a67bfbf49e059f48e` |
| `crates/fabelgeist-geometry/src/measurement.rs` | full source | `72ee58eb8ae01af4b7f17ef81f007dfa63c7dc818a1ed71ce5bbba0707236e70` |
| `crates/fabelgeist-hair/src/parameters.rs` | full source | `e8ddc101f1ef8295f3ab1943b4cae0257e483fd9fb67e2255bf614c1e51557e2` |
| `crates/fabelgeist-armor/Cargo.toml` | full source | `5a51ad1fc7ac2e848b3d8fd8b016f5e5d12c4e433c5ff600353d0530d0b6eba1` |
| `crates/fabelgeist-color/Cargo.toml` | full source | `a6c66c1e272fc13f282c7b0c03cb1a3291cac8af10876bccdbc1a44a85504f46` |
| `crates/fabelgeist-determinism/Cargo.toml` | full source | `c39a8a5055ca88572a587d0eadbde46eeb889d9060ec6b8930f59bfb76e44303` |
| `crates/fabelgeist-armor/review/museum/henry/trim.json` | full source | `e9ebce2edb4db3d4d4a87cce7400ef478390384d601925de284b350f0a078482` |
| `crates/fabelgeist-armor/review/museum/nuremberg/trim.json` | full source | `dc8bd427433cf3c8ee954b903597248a21b3bfa3332ad90c83bdb5f8af2df1a6` |
| `crates/adventuresim-character-creator/src/clothing.rs` | 1-120; larger earlier output truncated, no full-file claim | `d3bce4f4d9a53afc7ea141c5edf1163e85f55cb1b455b976b3bd5e329902ce33` |
| `crates/adventuresim-character-creator/src/inventory.rs` | 1-220 only | `0f820bca06e27b164da010e95dc7d3500a79a6feb0b7be9411ed8a41e88feb18` |
| `crates/adventuresim-character-creator/src/parametric_equipment.rs` | 1-260 only | `fb8ed7e496fe124cf6ea6c4deb962b9602cf62f5b516d362b5e50ddca9829c49` |
| `crates/adventuresim-character-creator/src/lib.rs` | 140-230; equipment_pbr re-export location found | `e468ea09b75d60d64e6b1d7612bc80f852b125d62c7473f34bfcd2a878f1b2dc` |
| `crates/adventuresim-character-creator/src/main.rs` | 205-235 and 350-390 | `566a69fc6cd50bd7d7ea5c0347567c39f2bdc37e9f8004675dc5328e3751e12e` |
| `crates/adventuresim-character-creator/src/character_morphs.rs` | 1-230 only | `8f76594ec3871a68df2f8e98740aa629e479b1e6fc7a0ad9a63e4e7f029e30a1` |
| `crates/adventuresim-character-creator/src/preview.rs` | 1-155 only | `f7fa8bef88e0b718991dab2245cce57436b6748186df1769f7fadcef64ceb698` |
| `crates/adventuresim-character-creator/src/export.rs` | 565-590 only | `82c083e495d9e3b992abf5ad23d3f99f48acc42da6900cd44ca99405140095de` |
| `crates/fabelgeist-armor/src/tiling/lacing.rs` | 235-305 only | `cce09840d83fc6c71d1020fc45961df2a0c9b4c2a85d5702bc0f65d6fad902a4` |
| `crates/fabelgeist-armor/src/lib.rs` | public re-export occurrences only | `d4b6a4af8b3137b359c2d08f8ef03445e100edc2d1280ca74ec816212af31264` |
| `crates/fabelgeist-armor/src/trim/tests.rs` | constructor occurrence discovery only; full fixture unread | `4b726c8fb494223caa41cf5539d152fc924226f7c1baa3b4b40506bbe560da35` |
| `crates/fabelgeist-armor/src/tiling/tests.rs` | constructor occurrence discovery only; full fixture unread | `166ec2be7855c465b709326152b2be1babfdcd538a062213cc6a5d8ae680d1f9` |

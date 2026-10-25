# Initial independent PBR Rule Review and thirteen-lens receipt

Base: `d000e848f2f5bc28b0728c2cd664c727a41ecf50`. The initial 74-file
fingerprint set was read directly, including the changed production
responsibilities, actual unchanged producers/receivers and fixture API hunks.
This is an initial review with actionable findings, not final clearance. Root
changed several sources while this review completed; the JSON preserves both
initial and observed successor fingerprints. Successor corrections require a
fresh independent source receipt.

No source edits, project execution, checks, tests, probes, scanners or hosted
CI were performed. No tests, cases, helpers or assertions were added.

## Actionable findings

### PBR-RULE-1 (P2)

Location: `crates/fabelgeist-armor/src/trim/finish.rs`.

crates/AGENTS.md: carry canonical leaves through all producers; unchanged producers belong to the migration.

Trim::default still assigned native [1.0, 0.77, 0.34] and 0.25 to Metal.color:Srgb and Metal.roughness:Ratio. This production default cannot type-check against the changed record.

Admit Srgb::from_encoded_channels([1.0, 0.77, 0.34]) and Ratio::from_ratio(0.25) at this exact authored literal, retaining scratch_density 0 and all other fields/defaults.

Initial-pin actionable finding. Root changed this source during review; current successor has not received full independent clearance.

### PBR-RULE-2 (P3)

Location: `crates/fabelgeist-armor/src/material.rs; construction.rs; lib.rs; Creator clothing.rs; Armor trim/finish.rs`.

crates/AGENTS.md: cohesive fallible APIs sharing one error use an owning Result alias; trait error parameters remain explicit.

At initial pin Metal::validate/ValidatedMetal::new repeat MetalError; Construction/Tiling/Plate/Lacing validate repeat ConstructionError; six Armor root encode/decode/hash/validate functions repeat DesignError; Clothing constructors and generator repeat ClothingError. Newly touched trim validate has TrimMaterialError. New LinearSurfaceMaterial TryFrom instead uses SurfaceProjectionResult<Self>, whose only user is that trait method.

Use MetalResult, ConstructionResult, DesignResult, ClothingResult and TrimMaterialResult in their cohesive ordinary APIs, retaining exactly the existing error types. Use standard Result<Self, Self::Error> for the projection trait, removing its otherwise unused alias; do not add a forwarding helper merely to use an alias.

Initial-pin R3 finding; Root successor modifications underway, not verified by this initial receipt.

### PBR-RULE-3 (P3)

Location: `modified owner declaration blocks, detailed in MD`.

crates/AGENTS.md: imports/reexports, modules, constants/aliases, structs, enums, traits, impls, then public/private functions; tests last.

Concrete initial-pin examples: Armor lib reexports interspersed with modules including new surface_material reexport at94; construction enum/impl before Tiling/Plate/Lacing/error; garment_material MailMaps reexport after mod and MailSurface after impls; Tactical meshes tests at88 before private native functions. More receiving locations are enumerated in MD.

Move whole declaration blocks with docs/attributes; keep executable bodies, cfg, visibility, error order and fixture bodies byte-exact. No pass-through module.

Initial-pin R4 finding; multiple successor blocks changed during review and require fresh receipt.

## Exact R4 receiving locations and proportionate corrections

- Armor `lib.rs`: all existing imports/reexports before module declarations;
  new `SurfaceMaterial` reexport was added inside the interleaved block. Keep
  every cfg, visibility and exported item unchanged. Six ordinary root
  `DesignError` functions can use `DesignResult`, with identical error values.
- `construction.rs`: Tiling, Plate and Lacing records precede Construction /
  ConstructionError enums and all impls. Move whole docs/attribute blocks.
- `garment_material.rs`: MailMaps reexport precedes rings mod; MailSurface
  record precedes MailWeave impls. Keep feature gates attached.
- Tactical `runtime_equipment/meshes.rs`: cfg(test) mesh_tests declaration
  moves after private native functions. Geometry-first material-second
  evaluation remains exact.
- `textile_pattern.rs`: TextileChartBoundary enum/impl precedes later structs;
  put every struct before enums and every impl after all type declarations.
  This is declaration movement; unrelated raster/physical policy stays open.
- `garment_material/rings.rs`: Ring, Chart and Hit records currently follow
  free functions; move declarations before impls/functions, preserving bodies.
- `bare_body.rs`: BareBody record follows BodyKey impl; both records should
  precede impls.
- `drape_preview.rs`: DrapeJob impl precedes DrapeMesh/MailMaterials records;
  private cloth_material precedes public input/poll/spawn/refresh functions.
  Existing production expect debt remains in the pending cause family.
- `character_export.rs`: CatalogMetals alias and CatalogGeometry record follow
  functions; move declarations before functions. No tuple/physical rewrite is
  inferred merely from declaration order; broader native schema remains open.
- `textile_surface.rs`: From<TextileMaps> impl follows material/textures free
  functions; move impl before functions with its cfg unchanged.
- `metal_controls.rs`: public slider follows private relief/image/depth/tiling
  functions; move the whole unchanged slider before private functions.
- `export.rs`: buffer/validation imports and texture/rigged reexports are
  interleaved with modules/constants, and private functions precede public
  export/socket APIs. Reorder declarations/functions only; do not absorb the
  separately inventoried socket math or export-admission/container repair.
- `main.rs`: imports/reexports are interleaved with modules; Libraries struct
  occurs after main. Its selected change is one module declaration; any whole
  order repair must preserve all numerical bodies and existing fixture cfgs.

These are pre-existing violations retained in modified owners, explicitly
reported rather than described as new runtime regressions. This report does
not assert an executed formatter/checker result.

## Source-backed trace

Color `Srgb`/`Srgba` owns encoded float channels and canonical TransferFunction
conversion. Math Ratio owns dimensionless unit identity. Armor SurfaceMaterial
owns named color/metallic/roughness fields; ComponentAppearance owns precisely
Inherit, scalar Surface or one mutable Textile recipe. TextileColor/finish
produce that same SurfaceMaterial, with alpha copied as native opacity.

Catalog clothing PBR, Metal/Lacing defaults and retained fields, MailWeave,
garment specifications, hair roughness and body skin carry these owners into
Creator preview, Tactical partitioning, GLB shell writing and review artifacts.
The unchanged gilt Trim::default was missed and is PBR-RULE-1. Component cloning
through BuiltPart append, DevicePiece partition, seam splitting and trim
reordering retains appearance alongside role/range/hinge/mount metadata.

Bevy standard_material projects encoded factors through Color::srgba. GLB and
ReviewComponent share sealed LinearSurfaceMaterial TryFrom: canonical RGB
decode once, alpha copied, every native field checked finite before Serialize.
GLB validates original source finiteness first, applies SurfaceTextures white
policy before decoding, and retains normal-only source color. No clamp narrows
generic finite-outside-unit PBR. Python independently retains its native [0,1]
material protocol. Missing material Inherit remains None in the review record;
the adapter retains borrowed ranges, role, mount and hinge.

Direct Color serde dependencies are present in Armor, Creator and Tactical.
Both parsed locks retain the exact package/version sets; only Color edges are
added (three root packages, two standalone packages). No Color GPU feature is
added. Existing generic Color and actual scalp hair material remain distinct.

## Trim correction extension: all thirteen lenses before review closure

The initial unchanged `trim/finish.rs` was read whole, including its existing
fixtures. Exact proposed repair: only import/use canonical Srgb and Ratio at
the gilt default literals [1.0, 0.77, 0.34] and 0.25. Preserve scratch_density 0,
all struct-update fields, widths and engraving operations. Add a cohesive
TrimMaterialResult for its actual ordinary validate API; move private tiles
after public validate. No physical trim arithmetic or new cause policy is
cleared or authorized by this literal correction. Actual receivers include
decoration Finish default, Construction::trim_on/Trim::narrowed, catalog and
character trim exports, studio/Armory EquipmentMaps.metal, MetalImages material,
and GLB checked SurfaceMaterial projection. Existing numeric fixtures remain
exact; no new fixture scenario is requested.

The following sequential lens cells apply to the selected PBR graph and
explicitly include that exact trim extension. They state independent contracts
and dispositions, not a repeated blanket pass.

### producer-propagation

Contract: Every actual catalog/recipe/default, mutable retained appearance and native receiver carries canonical color/Ratio/SurfaceMaterial.
Disposition: Incomplete at initial pin: PBR-RULE-1 missed unchanged Trim::default. Other inspected catalog/Metal/Lacing/Mail/skin/hair/component paths propagate direct leaves.
Connected repairs: Close gilt default and inspect actual receivers directly; no primitive adapters or new fixtures.
Dependencies: Metal.color:Srgb, roughness:Ratio; Trim consumers and two shared native projections.
Acceptance: All real defaults compile by source type contract and preserve exact coefficients; final checks still pending.

### existing-type-owners

Contract: Use existing lowest shared encoding/unit/appearance owners.
Disposition: Selected Color Srgb/Srgba, Math Ratio, Armor SurfaceMaterial and ComponentAppearance are coherent. Existing generic Color/u8 TextileColor and physical ShellMaterial remain distinct.
Connected repairs: Reuse these same owners in missed trim producer.
Dependencies: Color TransferFunction, Math units, Armor textile recipes.
Acceptance: No competing PBR wrapper, duplicate transfer formula, compatibility alias or tuple shim.

### admission-contracts

Contract: Encoding/unit leaves do not promise finiteness/range; recipe policy and terminal projection own admission.
Disposition: Transparent serde preserves float arrays/scalars; Metal .08..=.9, Mail .05..=1 and Lacing 0..=1 validation retained. Component appearance is required; old fields are rejected by deny_unknown_fields.
Connected repairs: Keep original source finite validation and checked native decoded projection.
Dependencies: SurfaceMaterial::is_finite; LinearSurfaceMaterial TryFrom; pinned serde_json native null behavior.
Acceptance: Finite decoded output before GLB/review JSON, no invented global [0,1] restriction; deliberate final schema only.

### invariant-operations

Contract: Actual mutation/conversion preserves authority, precision and accepted arithmetic.
Disposition: Color picker converts through canonical transfer and writes only on changed; alpha bypasses RGB transfer; native Ratio extraction occurs at real widgets/shader/material/JSON boundaries.
Connected repairs: Keep exact authored trim values and original operation order.
Dependencies: egui 0.34.3 float-linear and u8-sRGB widgets; native Metal bake roughness insert.
Acceptance: No no-op color roundtrip writes, accidental quantization, clamps or loss of original causes.

### aggregate-states

Contract: Correlated material/textile choice has one representable payload owner.
Disposition: ComponentAppearance excludes simultaneous retained Surface and Textile; Inherit is explicit. ClosureMaterial duplicate authority is removed.
Connected repairs: Preserve component role/range/hinge/mount association in append/partition/seam/trim paths.
Dependencies: ArmorComponent; BuiltPart append; DevicePiece partition; surfaces and component_materials.
Acceptance: Textile mutation derives fresh scalar appearance; no stale cached PBR beside fabric.

### type-ownership

Contract: Each concept belongs to its lowest shared actual owner.
Disposition: Color owns encoding, Armor owns shared scalar appearance, Creator owns authored body skin and native Bevy/GLB/review adapters.
Connected repairs: Keep trim literal construction in Trim::default, shared leaves imported from canonical owners.
Dependencies: Direct Armor/Creator/Tactical Color serde edges; two exact host metadata functions agreed with #826.
Acceptance: No Color->Armor/Bevy inversion, protected native takeover or duplicate local PBR implementation.

### redundant-authority

Contract: One retained authority per appearance fact, separate terminal native projections.
Disposition: Sealed LinearSurfaceMaterial is terminal and checked; source SurfaceMaterial is sole scalar authority. SurfaceTextures owns effective-white policy before GLB projection; apply no longer overwrites factor later.
Connected repairs: Retain only actual fabric in Textile variant and derive every receiver from it.
Dependencies: GLB shell writer; native ReviewComponent; Tactical/preview textile adapters.
Acceptance: Normal-only maps retain source color; base-color maps choose white before decode; original nonfinite input remains rejected.

### behavior-preservation

Contract: Preserve supported values, numeric policy, native protocol and meaningful ordering except explicit fixes.
Disposition: Catalog/recipe coefficients retained; intended skin GLB correction decodes same encoded skin source as preview; float widgets now use documented linear API correctly. Geometry-first Tactical evaluation retained.
Connected repairs: Gilt default must retain [1,.77,.34], .25 and all remaining recipe values.
Dependencies: Existing fixture diffs/oracles; external Blender linear RGBA parser and ranges.
Acceptance: No new tests/cases/helpers/assertions; existing invalid NaN and numeric fixtures preserved; executed validation not claimed.

### enum-shapes

Contract: Closed semantic alternatives own their payloads; ordinary predicates stay bool.
Disposition: ComponentAppearance Inherit/Surface/Textile is sufficient and direct; no duplicate selector booleans. Existing SurfaceAlpha/Sampling/Sides remain native semantic owners.
Connected repairs: No new trim enum is needed for a literal default.
Dependencies: Serde external tag snake_case; material/textile/textile_mut methods.
Acceptance: Old independent nullable material/textile fields and compatibility defaults absent in the selected graph.

### derivability

Contract: Derive traits only when preserving representation/admission; avoid needless handwritten boilerplate.
Disposition: New color/PBR records derive transparent serde and structural Clone/Copy/PartialEq appropriately. Native projection errors derive thiserror. Authored Metal/Trim defaults cannot be equivalent zero-derived Defaults.
Connected repairs: Retain authored Trim Default; alias/trait correction PBR-RULE-2.
Dependencies: Existing thiserror 2.0.20 and serde implementations; no new registry versions.
Acceptance: No manual serialization duplicating derive, bypassing checked native constructor, or invented test helper.

### type-granularity

Contract: Distinguish encoding, units, recipe admission and terminal representation without gratuitous wrappers.
Disposition: Three-channel Srgb and four-channel Srgba match real producers; Ratio is unit-only and different recipe bounds remain local; full PBR is one named record.
Connected repairs: Use the exact existing leaves in trim; retain physical/count/seed/Engraving work as separate pending families.
Dependencies: Public source arrays/scalar wire; native LinearSurfaceMaterial private fields/getters.
Acceptance: No broader bounds invented, no tuple adapter or retained second material record.

### magic-strings

Contract: Closed roles use enums while real authored/native text stays text.
Disposition: Appearance and material error classifications are typed. Review/GLB field names, image names, shader parameter strings and component stable names are native protocols.
Connected repairs: Retain exact external protocol names; R3 aliases do not widen causes.
Dependencies: Python Material.parse; glTF JSON fields; Metal bake roughness field.
Acceptance: No branch on display/Debug text introduced. Deferred export admission/Inventory prose causes remain open.

### magic-numbers

Contract: Primitive extraction is limited to authored literals and actual native arithmetic/protocol boundaries.
Disposition: PBR catalog constants are authored data; Ratio/color constructors at those literals are valid. Trim default is the one missed literal producer.
Connected repairs: Wrap only exact gilt literals; keep native shader, image-byte and factor values exact.
Dependencies: Canonical transfer constants unchanged; color byte divisor255 at actual u8 palette boundary.
Acceptance: No new arbitrary cap/tolerance/default, no unit-bearing primitive retained through selected PBR graph.

## Coverage and remaining limits

The JSON contains every initial 74-file fingerprint and the extra unchanged
producer/dependency source accounts with explicit read extents. Source hashes
are identity evidence, never substitutes for review. Large files were read at
their complete affected semantic responsibility and complete changed hunks;
unrelated physical algorithms, unchanged fixture tails and the entire external
client universe are not granted clearance. Existing fixture adaptations retain
their cases, helpers and oracles; numerical leaves are admitted only at fixture
literals or native assertions. No new tests/cases/helpers/assertions identified
in the selected diff.

Remaining physical units, counts, seed words, Engraving, trim causes/geometry,
Inventory causes, export admission and checked GLB layout/body finiteness,
native Grid and generated composition remain open. Existing drape production
expect calls, export anonymous roots/unwraps and related cause debt are not
waived by this selected PBR review. R3/R4 corrections and the missing trim
producer need current-byte successor review, followed by applicable hosted
checks and final same-pushed-revision reviews. No global pass is issued.

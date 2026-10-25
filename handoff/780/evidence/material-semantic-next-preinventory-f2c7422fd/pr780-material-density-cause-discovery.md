# Material density and validation-cause discovery

Read-only source discovery at HEAD
`47c23647be6d0e764076237b8a0a692906663d0a`, in the isolated audit-pr-780
worktree. Source identities accompany this report in
`/tmp/pr780-material-density-cause-discovery-fingerprints.json`.
This is a preimplementation repair graph, not clearance. No source edits,
local checks, tests, runtime probes or new fixtures/helpers/assertions were
performed. Root remains the sole implementer.

## MAT-C1: what density means and what currently breaks

`armor_metal::scale_to_metal_density` (`armor_metal.rs:39-61`) changes texture
repeat density, not steel mass density. Fitted positions are metres in the
wearer's model frame; input anatomical/body-surface UV coordinates are
dimensionless. The operation sums the lengths of all three edges of every
triangle, including shared/repeated edges, separately in world and texture
space. Their ratio has metres per UV unit; multiplying by the canonical
`Metal::TILES_PER_METRE` (actual value **4.0**, material.rs:68) produces the
coordinate scale. Preserve that existing edge weighting and arithmetic on
ordinary supported input; do not silently substitute area ratios, unique
edges, outer plate faces or bounding-box dimensions.

The local length expression subtracts in f32, then widens each difference to
f64 before squaring/summing/sqrt. Accumulations are f64. The ratio is narrowed
to f32 before multiplying by the f32 tile density. Every final UV is multiplied
in f32, in place. No success/failure value is returned.

Concrete defects:

1. Finite source coordinates do not imply finite output. Take an ordinary
   metre-scale triangle but a positive texture edge on the order of the
   smallest positive f32 subnormal. f64 squared lengths and texture sum remain
   positive and finite. The surface/texture ratio exceeds f32::MAX, so its
   narrowing yields infinity. A zero UV coordinate then becomes NaN through
   zero times infinity, while positive ones become infinity. The function
   publishes these values without a check, despite all source values being
   finite. Less extreme finite scales can overflow a large finite coordinate.
2. The public input is not an admitted geometry view. `GeneratedArmor` exposes
   independently mutable positions, texcoords and indices (`design.rs:134-154`).
   A missing UV/position or out-of-range index reaches direct indexing and
   panics. An incomplete final index triple is silently ignored by
   `as_chunks::<3>().0`. No general GeneratedArmor admission constructor is
   consumed here.
3. Nonfinite source/accumulation states are conflated with no density:
   nonfinite surface returns silently; NaN texture returns silently; infinite
   texture can pass the guard and produce scale zero. A nonfinite UV in an
   unreferenced vertex is still multiplied on the success path. These are not
   an admitted no-density protocol.

A discrete repair is a fallible density operation with a source-backed shape
and finite-arithmetic contract, followed by validation of the complete
candidate output **before any UV is assigned**. Two pure passes (validate all
candidate coordinates, then assign) can provide atomic failure without adding
a second retained coordinate authority. An owning checked candidate packet is
also possible if needed; no arbitrary epsilon or UV/length ceiling is justified.
Keep real failure types and source context. A public Result<()> is sufficient
if callers do not need to branch on successful no-op versus scaled output;
do not introduce a result-state abstraction solely to add names.

Preserve the existing f32 evaluation order for ordinary admitted values. If
widening subtraction or applying a f64 ratio instead is proposed to accept more
extreme inputs, record that numerical/acceptance change explicitly rather than
calling it behavior-neutral. Rejecting an unrepresentable native f32 scaling
operation with a typed error is a coherent policy; silently clamping or leaving
poisoned coordinates is not.

## Exact zero and no-density policies to retain or decide explicitly

| Current source situation | Actual existing behavior / repair obligation |
| --- | --- |
| No complete triangles | Both sums zero; UVs unchanged. Preserve a valid empty/no-density input rather than inventing one triangle. Reject a trailing partial triangle separately. |
| All referenced UV edges exactly zero, finite source | texture==0; UVs unchanged. This can be a legitimate constant/no-density chart. No machine epsilon threshold. |
| surface==0 and positive finite texture | scale==0; every UV becomes signed zero according to existing multiplication. Do not silently replace with no-op unless that behavior change is deliberate. |
| Zero coordinates in an otherwise positive chart | Legitimate; their candidate product must remain finite. They expose infinite-scale NaN immediately. |
| Nonfinite source, overflowed length or sum | Currently often ignored/coerced; distinguish a malformed/unrepresentable input from the finite zero cases and return a real error before mutation. |
| Material absent or not plate steel | fitted_catalog_item does not call density scaling. Preserve this gate and its accepted absence; do not make leather/mail/cloth/lead need plate UV density. |

`is_plate_steel` selects PolishedSteel, RoughSteel, OxidizedSteel. MailSteel,
leather, textiles, hardwood and lead do not use this operation. Material
classification is already a closed enum; do not branch on material labels.
The default/tile density constant is the existing owner; raw f32 material
leaves and their complete units migration remain MAT-R1, not solved here.

## GeneratedArmor admission and actual producer graph

GeneratedArmor is a public mutable aggregate. Its declaration does not certify
triangular topology, column correspondence, finite values, component ranges or
UV validity. `device_fit::assemble_recipe` (442-508) checks morph topology and
component correspondence, then directly assembles the aggregate from native
base/skin records and applies textile/skin operations. Dedicated bracer and
breastplate routes and underlayer routes also return GeneratedArmor; fitting
corrects morph deltas and does not add a universal geometry admission.

Existing narrower owners are not interchangeable: GpuBody::new checks a
wearer body's column counts/index membership/rig association, while
GeneratedArmor::validate_textile_attributes is private and additionally
requires textile component partitioning, no trim and morph correspondence.
Do not reuse the latter as a generic density predicate and reject plain
pieces with empty components. Canonical mesh counts describe topology counts
but do not certify dynamic index membership or finite values. A density view
should consume an existing common topology proof if one is extracted in the
owning armor family, plus the exact columns it reads/writes. It should not
claim to validate unrelated normals, skin, trims and every morph merely because
it receives the full aggregate, nor duplicate a universal GeneratedArmor schema
inside Creator. A host-only calculation does not inherently need a new GPU
count ceiling.

Actual flow:

- fitted_item chooses Recipe, Vambrace or Breastplate. Recipe's Underlayer and
  TrunkHose branch before the usual device_equipment/assemble route.
- fitted_catalog_item obtains the actual material from the selected placement,
  and invokes scale_to_metal_density only for plate steel. This is the sole
  production direct density caller; its existing Result can propagate the new
  typed cause with `?`, preferably contextualized with actual item/placement.
- parametric_equipment::fit -> fitted_catalog_item adds fitting context;
  selected and StudioCache::armor consume fit, then construction/trim.
  Their current Result/context propagation retains source errors.
- EquipmentExporter::armor calls fitted_catalog_item directly, then writes
  equipment assets; generate_equipment_assets adds item/placement context.
- Armory::fit_exhibits calls fitted_catalog_item on workers and currently
  formats its error into String before retaining it (see below).
- Character export uses selected; review export deliberately calls fitted_item
  instead, exporting geometry without density scaling. Do not add density to
  that shape-only review path as a side effect of this repair.

Direct existing fixture API adaptation: the one density call in
armor_metal.rs::texture_coordinates_repeat_at_the_metal_density must handle the
new Result. Its existing square helper, numeric data and assertions are to be
preserved. Other existing armor_metal fixtures check material classification
and trim density; the parametric fixture exercises fitted_catalog_item with
existing geometry/UV shape and finite geometry checks. Do not add cases,
fixture helpers or assertions to make up for missing failure-path coverage.

## MAT-R5: source-backed cause-loss locations

`Article::validate` is private and returns Result<(),String>
(inventory.rs:174-198). Catalog design errors are formatted with alternate
Display; decoration and construction errors with Display; Draped errors with
to_string; Settled errors with formatted name and alternate Display. Inventory
(353-361) returns those strings, or a duplicate-ID string. CharacterRecipe
(lib.rs:200-224) returns strings for its own policies and forwards Inventory's
string. Once this happens, real DesignError, EngravingError,
TrimMaterialError, ConstructionError and nested anyhow errors cannot be
recovered from the eventual error object.

Four actual receiver barriers are `map_err(anyhow::Error::msg)`:

- generation.rs:7, before generating body tensors;
- main.rs:223, initial recipe parse/load;
- main.rs:363, save_recipe;
- main.rs:377, load_recipe.

Removing only those calls is insufficient: the source error has already been
formatted in Article. Repair each owning stage, retaining stable policy
variants and an error source, then propagate typed errors with `?`/Context.
Keep article/item/inventory identity as fields or source-preserving context,
not as text replacing the cause. A clean hierarchy can be ArticleValidationError
-> InventoryValidationError (duplicate ID or actual item source) ->
CharacterRecipeError (version/layout/name/coefficients or Inventory source).
Nested source fields can retain the already authoritative anyhow::Error where
the heterogeneous producer intentionally returns it; do not transcribe every
leaf error into an invented parallel enum or parse its Display string.

Source-owner inventory:

| Article input | Existing validation owner and causal contract |
| --- | --- |
| Catalog own design | ItemDesign::validate -> ParametricDesign::validate (typed design/generation errors converted with Error::new), dedicated vambrace/breastplate validation uses Context. Real cause retained before Article formats it. |
| Catalog decoration | Decoration::validate -> optional Engraving::validate and Trim::validate, propagating actual errors into anyhow. Plain decoration is valid. |
| Catalog construction | Construction::validate returns ConstructionError; Solid is valid; tiling/lacing validate their own bounds. |
| Draped selection | GarmentSelection::validate returns anyhow, owning resolution/pattern/drape/mail checks. Pattern context is preserved; coif path has an additional real cause barrier below. |
| Settled garment | SettledGarment::validate -> selection plus SettledDrape::validate. Existing saved-drape shape/binding checks use anyhow errors; preserve these exact causes and policies. |

Do not alter which inputs validate: all inventory articles, including carried
ones, are currently checked; Catalog IDs are rejected when empty, not merely
because a catalog is unavailable at this schema boundary; own designs are
optional; plain decoration and Solid construction are valid; empty inventory
and ID zero are allowed. Catalog resolution/occupancy is a separate operation,
not a hidden requirement that CharacterRecipe::validate currently performs.

CharacterRecipe preserves exact version, identity/expression counts, nonempty
trimmed name and finite coefficients. Defaults and canonical base fixture use
zero coefficients legitimately. CharacterProportions is already a private
aggregate with checked set and checked serde try_from admission. Do not add a
second scalar policy to recipe validation. This discovery does not certify
all HairDesign/other physical input admission.

### Nested connected barrier: coif validation

GarmentSelection::validate at selection.rs:218-220 converts actual
HelmetDesign::MailCoif(...).validate() DesignError into a debug-formatted
anyhow message. Use source-preserving context there too. Changing only Article
would still leave this nested cause irrecoverable. Pattern, drape and mail
paths otherwise already forward their existing errors; this does not mean all
their raw physical policy leaves are migrated.

### Connected generation barriers: three TensorData conversions

Full generation.rs also replaces three actual errors with debug strings:
vertex, normal and skeleton into_vec::<f32> at lines 33/48/60. The pinned Burn
b1502de source (`burn-std/src/data/tensor.rs:177,787`) returns DataError deriving
Debug+Error (CastError or TypeMismatch). These are genuine retainable errors,
not diagnostic strings. Add Context to the existing Results rather than
constructing new message errors. Keep distinct vertex/normal/skeleton context.
Subsequent locally detected remainder/rig-count errors do not have an
underlying external cause; retain their stable local meaning separately.

Character_morphs.rs:97 also uses Error::msg, but its actual Core producer
CharacterProportions::set returns &'static str. It is not another replacement
of a typed error at that call. LibraryName's two UI adapters similarly consume
its current String error. A broader migration may give those owners typed
errors; do not blindly delete these adapters or invent a false source chain
without migrating their actual canonical producer.

## Complete retained receiver and display closure

StudioCache/body/armor, parametric selected/fit, equipment export and character
export currently propagate anyhow/context. Studio generation converts to
`studio.status` only when handling its final error; studio_ui's save/load/export
handlers do likewise. studio_overlay.rs:208 displays that final status string.
CLI main returns anyhow::Result, preserving causes to its final process error
report. Those are legitimate presentation boundaries, not errors to keep
wrapping as protocol messages.

Armory is a connected earlier loss point for the new density failure:

- Exhibit.fitted is Vec<Result<GeneratedArmor,String>> (armory.rs:96).
- fit_exhibits formats each fitted_catalog_item error at ~463 before storing it.
- Spawn/material/trim failure is also formatted and assigned to fitted[0]
  (~385), so a fit's error channel doubles as a later display error channel.
- Initial entries use fake Err("not fitted") as a pending state (~418).
- errors() returns &str and armory_ui creates hover/editor labels from it
  (UI lines 122,176), the actual final presentation boundary.

Retain actual error values through this cache/worker-to-UI channel. If its
shape is repaired, distinguish Pending, fitted mesh and actual failure rather
than synthesizing an error for an unattempted fit. No Clone requirement for
an anyhow::Error should be invented: the current fit results are moved from
worker vectors into exhibits. Preserve cause on the later spawn failure too,
then format only in armory_ui. This is part of the complete new density cause
receiver graph, not a reason to clear an early formatted cache as "just UI".

Outfit's EquipConflict and Inventory::fit/loadout are separate existing
occupancy/catalog-resolution protocols. EquipConflict::Invalid(String),
outfit::loadout's explained-message conversion and review_export's debug
conversion still use prose. They do not currently receive Article::validate
errors, so replacing the validation hierarchy alone neither repairs nor
clears them. Inventory::explain is intentionally a final naming/presentation
operation. Record these paths distinctly in the broader inventory audit;
do not silently expand this repair into a competing equipment-graph model.

## Additional concrete adjacent inventory finding

Inventory::add computes `item.id.0 + 1` unchecked for every item
(inventory.rs:232-239). InventoryItemId is a public u64 and serde accepts
u64::MAX; validation only rejects duplicates. Adding after one such valid ID
panics with checked arithmetic or wraps and can create an invalid/duplicate
identity. This is not MAT-C1's UV or MAT-R5's cause bug, but it is an actual
admission/operation issue encountered in the fully read inventory owner and
must receive an explicit disposition in the complete audit. Do not reject ID
zero or reset IDs as an unrelated shortcut.

## Acceptance and scope limits

Before implementing, inventory all thirteen lenses for the exact API/error
owners and full receivers above. The density operation must leave all UVs
unchanged on failure, preserve finite zero/no-density policies, keep the
existing plate-material gate and native density owner, and produce finite
coordinates on success. Its caller must propagate the real cause through
fitting/export/cache to final display. Validation must retain each actual
nested error through Article/Inventory/Recipe, all four receivers, coif and
TensorData context, without compatibility String overloads. Preserve saved
schema and all existing fixture logic; new validation policy is not required
to change serialization. Do not add tests/cases/helpers/assertions.

Full reads include armor_metal and its fixtures, Inventory and its existing
fixtures, ItemDesign, Decoration, generation.rs, fitted_existing, the actual
parametric fitting/finish/caller bodies, CharacterRecipe/default/validation
and its existing fixtures. Targeted full relevant bodies read for
GeneratedArmor/device assembly, morph correction, narrow body/textile
admission, material/engraving/trim/construction errors, garment validators,
Core proportions, main loading/save, StudioCache, Armory worker/cache/display,
UI terminal rendering and exports. Burn's actual into_vec and DataError were
read from the pinned local source, not guessed.

Not end-to-end re-reviewed: every native armor generator and numerical shader,
all trim split/edge/construction geometry, garment simulation/conforming,
complete body tensor internals, whole UI/Armory interaction bodies, all error
sources in occupancy/catalog graph, or all other material/physical leaves.
The report inventories their actual boundaries without granting clearance.
No implementation, final same-revision independent review or hosted validation
has yet been performed for these repairs.

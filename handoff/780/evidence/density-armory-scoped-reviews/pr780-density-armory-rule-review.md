# Density and Armory: independent Rule + thirteen-lens increment review

Verdict: **no new actionable Rule finding in the exact eight-source increment**
at base HEAD `f2c7422fd6a0f995eb90b72cc9c2a5bd12de4d35` plus the pinned
working-tree files below. This is source-level increment clearance only. The
six broader material groups, native scalp/Grid, generated composition, global
reviews and final same-pushed-revision hosted gates remain open.

Read-only review under the repository/root and crates AGENTS, Rule Review and
its bespoke-type reference, and the thirteen lens contracts. No source edits,
project checks, compiler/lint runs, tests, probes, helpers, assertions or hosted
CI were performed. Only source reads, source comparisons and hash bookkeeping.
Previous verdicts and searches are not used as clearance.

## Exact current source receipt

All eight current sources were fully read, including existing inline fixtures;
their changes were compared against the preinventory source backup. The two
new modules were reread after Root's final doc-only corrections. Every hash
matches the current ledger receipt. Full details and dependency read extents
are in `/tmp/pr780-density-armory-rule-receipt.json`.

- `crates/fabelgeist-armor/src/material/density.rs`:
  `94db413bd0eeefd9fc860abb8eee368ee1d11a55cb48cbb4cd7ba2113fd0d665`.
- `crates/fabelgeist-armor/src/material.rs`:
  `12debec1a00cb2ff34e5ef09978a4a3c523bde4bfb1926a1dbef86b7decaf6ea`.
- `crates/adventuresim-character-creator/src/armor_metal.rs`:
  `00f657217f6dc1b7b5677176fad2153836a7aabc133a1c7b518bba391a607c16`.
- `crates/adventuresim-character-creator/src/parametric_equipment.rs`:
  `d71953e80a22f666f64d7733ae41279fe18700e1f6b0531ee1cfd404f4c95559`.
- `crates/adventuresim-character-creator/src/armory/exhibit.rs`:
  `b2f4f006cf78814970012ab94d85d178176813c53cd2fbd9768ff2b17349c221`.
- `crates/adventuresim-character-creator/src/armory.rs`:
  `74f95648715bc2456ec986093c649779f02d3c773607d24af7ff0ccfc4ca18ef`.
- `crates/adventuresim-character-creator/src/armory_scene.rs`:
  `ff67d80dde117a3e4dd23e394d0a5bc6efe5049ca032c0e5fc0a30eaaeec3429`.
- `crates/adventuresim-character-creator/src/armory_ui.rs`:
  `dea23f6efe94e9b5439739548675be1b0cfd109a3e153e2a083d7a816db43aeb`.

## Actual contracts and receiving graph

`material/density.rs:38-56` now owns GeneratedArmor::scale_to_metal_density.
Its source is the actual public geometry record in `design.rs:134-154`, whose
independent arrays do not themselves prove admission. `DensityInput::new`
checks only what density needs: complete triangles; equal position/UV rows;
all finite source coordinates, including unused rows; host-address conversion;
and membership of every index in that exact position/UV collection. These
checks precede direct indexing. Normals, skin, component partitions, trim and
morphs are explicitly outside this proof. No unrelated MeshData validation
policy or GPU u32 count limit is imposed.

`DensityInput::scale` uses all three edges of every triangle, including repeated
shared edges. Native differences are calculated in f32 before widening to f64
for squaring, summing and square root; accumulation is f64. The ratio narrows
to f32 before multiplication by canonical Metal::TILES_PER_METRE, as before.
The final admitted scale is canonical Math Ratio. Every output UV product is
finite-checked before any UV is assigned. The assignment loop recomputes the
same products under the same exclusive borrow, so an error cannot publish a
partial replacement. This closes MAT-C1 for this operation without caching a
second geometry or scale authority.

Finite empty topology and zero accumulated chart-edge length yield a no-op.
A zero-length surface with positive chart-edge sum still scales all UVs by
zero. Tiny positive charts are not floored; if native scale or any coordinate
product is unrepresentable it returns a typed error before mutation. Partial
triangles, malformed columns/indices and nonfinite input now fail explicitly
rather than being ignored, panicking or publishing poison. These are the
inventoried deliberate behavior changes, not a claim of byte-identical invalid
input behavior.

The old Creator free function is deleted. The sole production caller remains
`parametric_equipment::fitted_catalog_item:97-112`, still gated to actual plate
steel, and uses `?` into its existing anyhow Result. Other materials retain
their accepted absence/bypass. Existing fit/selected/export call structure is
unchanged by that one-line adaptation; this review does not add density to
shape-only review-export paths.

`armory::fit_exhibits` now carries actual anyhow Result<GeneratedArmor> values
out of the scoped workers. Native errors acquire placement context through
with_context and are moved into `ExhibitFit::Failed`, rather than formatted
into String. Catalog design absence has its own existing diagnostic, with no
lower source to preserve. The worker panic path still resumes unwinding;
it is not converted into an ordinary fit failure as a side effect.

`armory/exhibit.rs` owns Pending/Fitted/Failed payload states and a separate
optional actual preview error. Initial construction makes Pending once per
retained placement. Refit fills the same ordered cardinality from actual
worker results. Bounds, triangle counts and spawn_exhibit consume only Fitted
states. A later preview failure is retained separately; it no longer destroys
a successful first fitted mesh. Subsequent preview attempts clear/replace that
error, and `finished` records the last attempted finish, including failures,
matching existing retry scheduling. This does not introduce automatic retries
or alter lower GPU cache policies.

`Exhibit::errors` yields borrowed actual errors. Armory failed-count queries
only test their presence. The list/editor UI formats complete error chains at
the terminal hover/label boundary; errors are not reclassified from prose.
The moved Exhibit methods preserve current bounds/triangle behavior.

Nonempty placement indexing has an actual producer proof:
`ItemCatalog::wearable -> is_wearable` requires at least one placement with a
nonempty surface. `exhibits` applies the same surface predicate to actual
retained placements from that item, and constructs fit states from precisely
that vector. `OwnedCatalogPlacement` is an immutable shared item snapshot;
its ordinal comes from the same present equipment definition. The changed
receiving graph does not mutate placement arrays or fit cardinality elsewhere.
Thus no missing-element fallback is needed in this increment. This is not a
claim that pub(crate) mutable records provide a universal type-level guarantee.

Camera exit matches the actual Option<OrbitCamera> payload. OrbitCamera and
OrbitGoal are Copy in their real owner, so matching through the resource borrow
is legal and the previous save/restore/clear sequence is preserved without
expect. Refit with stale exhibits but no generated body reports terminal status
and returns, replacing the old production expect with an explicit branch.

## Thirteen complete sequential increment dispositions

1. **Producer propagation**: CLEAR for this increment. The Creator free density
function is removed, not forwarded. The owning GeneratedArmor method is exported
through Armor material error/result definitions. The only actual production
density call is fitted_catalog_item, which consumes Result with ?. The existing
density fixture handles Result at its existing call only. The Armory worker ->
ExhibitFit -> bounds/triangles/scene -> terminal UI graph consumes the new
retained representation directly.

2. **Existing type owners**: CLEAR for this increment. GeneratedArmor owns
geometry mutation, canonical Metal owns repeat density, canonical Math Ratio
carries the final dimensionless scale. DensityInput borrows the exact
columns/topology required by this operation; it does not compete with a general
MeshData validator or certify unrelated normals/skin/morphs. Existing
OwnedCatalogPlacement remains the catalog snapshot owner.

3. **Admission contracts**: CLEAR for the new density operation. Before indexing
or mutation it rejects partial triangle triples, differing position/UV
population, nonfinite positions or UVs including unused rows, unrepresentable
host indices and out-of-range indices. Its private borrowed fields cannot bypass
admission. It checks both accumulated lengths, narrowed native scale and every
final coordinate product. No new serde/default path exists. ExhibitFit admits
only pending, actual mesh or actual error payloads.

4. **Invariant operations**: CLEAR for the new operation. Source values and
replacement products are inspected before the sole write loop. Private scale
calculation and subsequent assignment occur under one exclusive GeneratedArmor
borrow, so the checked columns cannot drift between passes. No retained output
cache is needed. Existing shared-edge weighting and f32/f64 evaluation order are
preserved for supported finite input.

5. **Aggregate states**: CLEAR for this increment. DensityInput binds admitted
triangle members to corresponding finite position/UV columns. Armory successful
fitting survives independent preview failure. Pending is distinct from failure.
Current producer creates one fit state per retained placement and replacement
preserves that cardinality/order; no aggregate mutation was found in other
actual receiving bodies. This is a source-traced internal invariant, not a
public branded nonempty-collection guarantee.

6. **Type ownership**: CLEAR. Density module owns admission and mutation on the
actual local geometry type; exhibit module owns retained fit/preview outcomes
and their queries. Both are real responsibilities. There is no Creator density
shim, duplicate ratio type, launch adapter or new framework. Root owns these
functions under the user transfer; the unrelated #826 generated tranche is
excluded.

7. **Redundant authority**: CLEAR for the changed facts. No competing density
implementation, stored scale, copied UV candidate authority or stringified cause
remains in the repaired path. Preview failure is an independent stage outcome,
replaced/cleared by each actual respawn; it does not overwrite the first fit
result. Camera exit consumes the actual copied Option payload rather than
re-deriving it from a boolean.

8. **Behavior preservation**: CLEAR at source for authorized deltas. Valid empty
triangles and zero accumulated UV-edge length remain unchanged; zero surface
with positive UV-edge length keeps zero scaling, including the existing f32
multiplication semantics. Malformed/nonfinite/overflowing data now errors
atomically. Non-plate materials bypass density as before. Successful fit bounds
and triangle counts remain available after preview failure; that is the intended
correction. Current preview retry scheduling is preserved: finished records last
attempted finish, including failure.

9. **Enum shapes**: CLEAR for changed state. ExhibitFit
Pending/Fitted(GeneratedArmor)/Failed(anyhow::Error) owns the payload chosen by
the state; no fake not-fitted diagnostic remains. Optional preview_error means
no later failure and carries the actual failure otherwise. Camera saved state
uses Option<OrbitCamera> directly. Existing unrelated Armory booleans and raw
scene tuples remain outside this increment and are not thereby cleared.

10. **Derivability**: CLEAR for new code. MetalDensityError derives
thiserror::Error using the existing dependency, preserving the concrete
TryFromIntError source. There is no unnecessary handwritten Error/Display/Clone
implementation, error stringify round trip, artificial Default or serde bypass.
ExhibitFit need not derive Clone because the actual error and mesh are moved
into retained state. The existing authored Finish/Armory defaults retain their
prior behavior.

11. **Type granularity**: CLEAR for changed types. Ratio expresses native
dimensionless scaling without adding a new bounded universal scalar.
DensityInput is the actual narrow borrowed proof; no universal geometry or
GPU-count certificate is asserted. Error usize/u32 fields report actual
malformed host input and do not become launch/sort quantities. The fit-state
enum is a local stage outcome rather than a generic shared cache abstraction.

12. **Magic strings**: CLEAR for new/changed paths. No behavior branches on
error text, Debug or UI prose. Worker failures keep anyhow causes and placement
context; UI alone formats the chain. The missing-design diagnostic denotes
absence with no discarded lower source. Existing names/IDs/egui labels remain
actual catalog or presentation boundaries; no artificial enum is introduced for
free text.

13. **Magic numbers**: CLEAR for this increment. Triangle arity three and
coordinate dimensions are explicit native geometry structure. The one
tile-frequency value stays under canonical Metal::TILES_PER_METRE. No epsilon,
invented minimum UV scale, GPU capacity ceiling or extra tolerance appears. The
private numerical kernel retains actual f32 input vectors and f64 accumulation;
broad material unit leaves and unchanged Armory camera/layout tuning values
remain separately open.

## Governing style, scope and existing-fixture provenance

The new density file follows imports -> owning Result alias -> private borrowed
record -> typed error enum -> impls -> private numerical kernel. Its functions
remain focused and below the 100-line growth ceiling. The new exhibit file
follows imports -> record -> enum -> impls; it extracts actual retained outcome
and query responsibilities, rather than a forwarding facade. Armory now orders
imports/reexports before mod declarations, then constants, records, enums,
impls and functions. Region::label precedes the private Region::of method.
The parent Armory extraction reduces its production size to 427 lines before
further material changes; the new files are 122 and 95 lines. This review does
not raise any ceiling or generate any baseline.

The new density error retains the concrete TryFromIntError source. No dependency
or lock change is introduced, and there is no new Error::msg/stringification
barrier in the repaired path. MetalDensityResult is the owning cohesive alias;
Creator already uses anyhow::Result. There is no old function alias/forwarder,
primitive migration adapter, duplicate count/grid/seed/color type, or
cross-owner
native edit in these eight source deltas.

Literal source comparison proves:

- armor_metal's entire existing test suffix is byte-exact after substituting
  only `scale_to_metal_density(&mut armor);` with the existing call's direct
  `armor.scale_to_metal_density().unwrap();` Result adaptation.
- parametric_equipment's entire existing test suffix is byte-exact.
- armory_scene's entire existing test suffix is byte-exact.

The density and exhibit modules introduce no test module, fixture helper, case,
assertion or doctest. Existing numeric data, helper bodies and oracles remain
unchanged. These are source provenance comparisons, not executed tests.

## Explicit remaining limits

This fresh read covers the complete changed density/Armory responsibilities
and their stated canonical dependency contracts; it does not claim that every
unchanged field in these source files is a finished bespoke migration. Existing
raw material/PBR leaves, trim result tuples, material Result aliases, broader
parameter/physical roles and adjacent whole-family item-order work remain in
the original six material groups. In particular, parametric_equipment retains
its inventoried functions-before-records order, and armory_ui's unchanged public
labels function follows private functions; these were not repaired or silently
cleared by a one-call density or UI formatting delta. Existing Armory layout
bounds tuples, physical camera coefficients and flags are not newly certified.

A full general GeneratedArmor validation, mesh/normal/skin algorithms, complete
Tactical/export consumers, native scalp/Grid and the separate dirty generated
Map/native leaves are outside this increment. No compilation, Clippy,
formatting, runtime, failure-path execution or hosted gate result is asserted.
The broader final review must compose these exact sources with every connected
material/schema repair, ownership agreement and required project check on the
same final pushed revision. No merge is authorized.

# Generated-program receiving discovery

Root current HEAD `ead46619dec5ef45993f63202cb289bd3986b018`; incoming #826
`ccb1b6944d674c0bf29d10e6820c0ac4c55092b7`.

This is a read-only pre-integration inventory, not family or PR clearance. All
51 incoming generated sources (6,186 lines) were freshly read; their hashes
match the frozen manifest. All 136 composition hashes match, which establishes
byte identity only. Most of the lower native graph is explicitly unread. The
earlier PBR report is preserved.

Generated manifest SHA-256
`65379822010b70c8c34beb478dc299ee193b5267ee6ef97fbf803d2d122f00cb`.
Composition SHA-256
`5a1670cc3065b74bd0b9ec62863d0edaee690e52f2dff9c59c60746aa4119e9c`.

Exact path/hash/read-extent accounts, 136 identity receipts, 19 note identities,
independent literal comparisons and dependencies are in the sibling JSON. Root
HEAD advanced during review from d000e848f to the displayed HEAD; there are no
committed Compute/GPU changes between those commits. Dirty source hashes are
authoritative.

## Findings and connected closure

### RECEIVE-G1 (P1) — Required integration work, expected unreceived API mismatch

Six Root MC/DC Gather/Stream constructions still pass context. Root lib lacks
source_error and resource_parameters declarations and
GeneratedProgramError/Result/Function/FluidOperation/Binding exports. Root
Buffer owns capacity and stride modules missing from incoming whole module;
incoming adds mapping/readback native reexports and admission_error.

Sources: `crates/fabelgeist-compute/src/lib.rs`,
`crates/fabelgeist-compute/src/marching_cubes/mod.rs:65,73`,
`crates/fabelgeist-compute/src/dual_contouring/mod.rs:41,42,50,54`,
`crates/fabelgeist-gpu/src/data/gpu/buffer/mod.rs`.

Repair: Compose broad Root lib/prelude and Buffer module contents, retain Root
capacity/stride, receive native mapping/readback visibility/admission error;
adapt all six constructors directly. Receive the complete incoming generated51
as one owned packet, never restore Definition::parse_signature.

Acceptance: All six calls use final signature; Both Buffer capacity/stride
owners and required native reexports remain; No compatibility forwarding
API/temporary primitive adapter; Required hosted checks on final composed pushed
revision.

### RECEIVE-G2 (P2) — Canonical-owner migration gap in unchanged Root Reduce

Reduce independently reflects the output Array stride, then separately divides
Buffer bytes by stride and narrows to u32. Incoming
ResourcePipeline::storage_stride_for(Output) and canonical
BufferElementCount::from_extent already own those operations.

Sources: `crates/fabelgeist-compute/src/reduce/mod.rs:159`,
`crates/fabelgeist-compute/src/reduce/layout.rs:14`,
`crates/fabelgeist-compute/src/resource_pipeline.rs`,
`crates/fabelgeist-gpu/src/data/gpu/buffer/stride.rs`.

Repair: Have Root Reduce consume the canonical reflected pipeline/stride and
buffer-count admission. Preserve the distinct live reduction role for both
texture and buffer populations, exact scratch live extents, <=1 behavior, and
first/recurring pipeline semantics. Do not mechanically replace texture axes
with a Buffer count.

Acceptance: No second storage-layout reflection implementation for the same
generated source; Buffer quotient/u32 admission reaches canonical owner; Same
padded strides, supported zero/tiny paths, scratch-device association and
arithmetic order; No new tests/cases/helpers/assertions.

### RECEIVE-G3 (P2) — Typed local-policy error migration gap

Root Reduce/Min/Max still construct fixed local
layout/overflow/unsupported-result failures with anyhow! strings. The
byte-admission conversion keeps its integer cause, but the policy roots have no
stable typed role.

Sources: `crates/fabelgeist-compute/src/reduce/mod.rs:178`,
`crates/fabelgeist-compute/src/reduce/layout.rs:26`,
`crates/fabelgeist-compute/src/reduce/execution.rs:39,154`,
`crates/fabelgeist-compute/src/reduce/min.rs:65`,
`crates/fabelgeist-compute/src/reduce/max.rs:65`.

Repair: Inventory and establish an owning reduction error vocabulary or consume
canonical ResourcePipeline failures where those already match. Preserve original
SDK and checked narrowing causes, supported input domains and present error
order; remove impossible fallbacks only with a source-backed state proof. Do not
enlarge #826 GeneratedProgramError without ownership agreement.

Acceptance: No fixed local reduction policy root relies on presentation text;
Same original causes reachable via Error::source; Existing fixture oracles
unchanged; hosted checks final SHA.

### RECEIVE-G4 (P2) — Existing reachable source-API weakness in connected Min/Max wrappers

parse_compare_signature accepts whitespace after min/max; custom.replace only
renames the exact no-space spelling. A valid admitted spaced declaration remains
named min/max while the generated reduction calls min_cmp/max_cmp, leaving the
wrapper unresolved at native compilation.

Sources: `crates/fabelgeist-compute/src/reduce/min.rs:15,24`,
`crates/fabelgeist-compute/src/reduce/max.rs:15,24`,
`crates/fabelgeist-gpu/src/data/gpu/compute/signature.rs:214`.

Repair: Reconcile renaming with the canonical admitted function/source owner
directly. Preserve accepted whitespace and user identifier/body content; no
exact-spelling fallback or duplicate parser. Coordinate any claimed shared
signature edit with #826.

Acceptance: Every lexical form admitted by the canonical signature parser is
renamed consistently; No unintended body/comment/global replacements; Existing
custom-code fixtures preserved; no new fixture/case/assertion.

### RECEIVE-C1 (P3) — Confirmed comment mismatch in current Root Scan

ScannedAuxiliary is documented as a completed auxiliary scan immediately after
ComputePass::dispatch. Native validation polling, where enabled, does not make
this a universal completion guarantee.

Sources: `crates/fabelgeist-compute/src/scan.rs:31`,
`crates/fabelgeist-gpu/src/data/gpu/compute/pass/mod.rs:18`,
`crates/fabelgeist-gpu/src/globals/wgpu_context/submission.rs`.

Repair: Describe the submitted/recorded auxiliary output and its allocated
population without claiming completion. No completion wait is required to repair
this prose.

Acceptance: Doc matches dispatch/submission and separate completion contracts;
No native/progress/ordering implementation change needed.

### RECEIVE-D1 (P3) — Derivability finding, Root-owned

Manual ReduceDefinition::default mechanically constructs String::new and
ComputeSourceCache::default; deriving Default preserves the same fields and
bounds because ComputeSourceCache has an intentionally unbounded manual Default.

Sources: `crates/fabelgeist-compute/src/reduce/mod.rs:36`,
`crates/fabelgeist-gpu/src/data/gpu/compute/source_cache.rs:30`.

Repair: Use derive(Default) for ReduceDefinition while retaining the cache
generic manual Default and cache-excluding PartialEq.

Acceptance: Same empty code/default cache and no new ReductionPipeline Default
bound.

### RECEIVE-D2 (P3) — Derivability finding, #826 ownership dependency

GeneratedProgramError has a boilerplate empty std::error::Error impl. Existing
Compute thiserror dependency can derive only Error while retaining the complete
semantic/manual Display unchanged; no source fields currently exist. Actual
pinned macro source emits Display only when display attributes are present.

Sources: `crates/fabelgeist-compute/src/source_error.rs:209`,
`crates/fabelgeist-compute/Cargo.toml`,
`thiserror-impl-2.0.20/src/expand.rs:222,383`.

Repair: Discuss a narrow derive(Error)-only correction with #826; preserve exact
manual Display and cause behavior. This is not permission for Root to edit a
claimed source. GPU manual errors do not have an existing thiserror dependency
and are not automatically equivalent repairs.

Acceptance: Same Display behavior including Formatter flags; Same
Error::source(None), no new dependency or generated-message rewrite.

## Sequential thirteen-lens inventory

### P1: producer-propagation

Map primary parameter is chosen from actual input presence; secondary resources
from supplied parameters; authored role order survives into declarations/calls.
Gather/Scatter/Stream source-only constructors remove unused context. Actual
MC/DC counts->Scan->Stream use the same admitted lattice; JFA/noise emit final
Map source and named parameters.

Disposition: Actionable receiving gaps G1/G2/G3/G4; packet source roles followed
through all generated definitions/programs; external/downstream closure
incomplete.

Connected repairs: RECEIVE-G1, RECEIVE-G2, RECEIVE-G3, RECEIVE-G4. Dependencies:
Root MC/DC/lib/Buffer and Reduce closure; #826 owns generated51.

Acceptance: Every real producer/caller and retained state uses final owners, no
forwarded/primitive adapter, no new fixture population.

Status: receiving inventory incomplete; no clearance.

### P2: existing-type-owners

BufferElementCount admits complete storage records and valid zero; stride
includes native padding; InvocationCount and WorkgroupCount are launch
quantities. SortItemCount and ScanElementCount retain different algorithm roles.
ResourcePipeline reflects Naga rather than ResourceBaseType::element_size.

Disposition: Reuse established count/stride, workgroup, coordinate, descriptor
and boundary owners; G2 remains actionable.

Connected repairs: RECEIVE-G1, RECEIVE-G2. Dependencies: Canonical GPU
modules/exports; actual protected native83 receipt.

Acceptance: Consume exact matching owners while preserving role distinctions;
only real native boundaries extract primitive representations.

Status: receiving inventory incomplete; no clearance.

### P3: type-ownership

Native metadata/TextureValue/CoordinateParameter/ResourceFetch live in GPU;
authored algorithm roles/source errors live in Compute; BoundaryMode lives in
Root Compute boundary; shared cache is GPU.

Disposition: Generated51 owners are clear; Root-owned broad API, counts/stride
and algorithms unreceived; no takeover authorized.

Connected repairs: RECEIVE-G1, RECEIVE-G2, RECEIVE-G4, RECEIVE-D2. Dependencies:
Explicit agreement before any shared #826 function edit; #774 claims not
revisited in this scope.

Acceptance: One canonical owner per concept; complete
library/module/export/dependency composition, preserved unrelated work.

Status: receiving inventory incomplete; no clearance.

### P4: type-granularity

Coordinates/LinearIndex/LinearPopulation encode different native projections.
CoordinateShape is not a workgroup shape. ReductionElements covers live
buffer/texture values, not scratch capacity; may reuse Buffer count leaf only at
buffer admission. SourceParameter retains declaration with role; unit
Center/Offset/Other carry no unused value type.

Disposition: Useful role distinctions retained; no new universal quantity
framework proposed; downstream physical semantics incomplete.

Connected repairs: RECEIVE-G2. Dependencies: Canonical numeric leaves and
downstream caller-unit inventory.

Acceptance: No erasure of texture-axis vs flat population or sorting vs launch
distinction; no duplicate validation merely to create a type.

Status: receiving inventory incomplete; no clearance.

### P5: redundant-authority

Public mutable authored code is reparsed at build/execute and included in source
cache key. Cache key carries real device and observation policy; sorted
secondary recipe is stable. ResourcePipeline holds shape/layout from the same
private immutable shader. Builtin fluid empty source is valid because every
varying descriptor/boundary is in recipe.

Disposition: Caches/source/schema/pipeline invariants inspected;
native/submitted lifetime closure incomplete.

Connected repairs: RECEIVE-G2. Dependencies: Receive exact cache/native pipeline
owners; lower provider implementations remain incomplete here.

Acceptance: No stale source/role/shape/cache identity; derive native facts from
one source without a competing reflection implementation.

Status: receiving inventory incomplete; no clearance.

### P6: admission-contracts

Default or mutable definitions do not certify authored WGSL; current source is
parsed again. NativeParameter checks duplicate names and two-part declarations
while native WGSL validity stays compiler-owned. TextureValue accepts numeric
scalar/vector lanes; CoordinateParameter admits reserved unsigned shapes;
neighborhood admits actual reflected input/output capacities before dispatch.

Disposition: All51 source admissions inspected; lower native
admission/device/usage/texture view closure incomplete.

Connected repairs: RECEIVE-G1, RECEIVE-G2, RECEIVE-G4. Dependencies: Protected
shader/pipeline/binding/texture owners and actual callers.

Acceptance: No broad guessed source validator, unsupported new ceiling,
swallowed native cause or admission-bypassing compatibility path.

Status: receiving inventory incomplete; no clearance.

### P7: invariant-operations

TextureAddressing uses checked u64 population and actual u32 index/population
projection. NeighborhoodLayout checks positive axes, clamp extent,
wrap2extent-1, signed flattening, divisors and actual sampled/source coverage.
Root extraction caps prefix before u32 sums and writer bounds; Scan/Reduce
retain exact live scratch extents.

Disposition: Fresh address arithmetic traced; whole repository physical/native
numerical acceptance remains open.

Connected repairs: RECEIVE-G2, RECEIVE-G3. Dependencies: Native device
limits/actual resources plus remaining physical-family review.

Acceptance: All published counts/addresses fit actual expressions; preserve
native operations/tie/order/valid zero and no guessed payload stride.

Status: receiving inventory incomplete; no clearance.

### P8: aggregate-states

SourceParameter binds authored spelling to role; private signature lists
preserve order. Stream counts and inclusive offsets are separate supplied
buffers with documented same-population/capacity obligations. MC/DC construct
counts/Scan from same grid and guard whole primitive publication; shared vertex
native format matches deinterleave.

Disposition: Packet private schema/layout/cache state inspected; Stream
preconditions followed through MC/DC; external consumers remain incomplete.

Connected repairs: RECEIVE-G1, RECEIVE-G2. Dependencies: Final actual source
receipt and all extraction/downstream fixture callers.

Acceptance: Same population/order/capacity authority from producer to native
writer, device-associated scratch reuse, no partial host-state claim.

Status: receiving inventory incomplete; no clearance.

### P9: enum-shapes

BoundaryMode is canonical Clamp/Wrap. ResourceLayout and parameter role enums
carry their actual payloads. Stream has_input_value derives from Value roles;
Center/Offset/Other are deliberate unit roles. JFA local ping predicate only
selects alternating temporary textures.

Disposition: No new carried semantic bool defect established in generated51;
complete wider graph remains open.

Connected repairs: RECEIVE-G1. Dependencies: Boundary export/native source
agreement; wider enum-state review outside this packet.

Acceptance: No parallel boolean authority or unrelated role collapsed into
representation; native booleans/indices remain at true adapters.

Status: receiving inventory incomplete; no clearance.

### P10: derivability

Cache generic manual Default prevents spurious Parameters/Value Default bounds.
Definition PartialEq intentionally ignores cache. TextureValue TryFrom is
semantic admission; Display roots can be manual while Error derives
independently.

Disposition: D1 and D2 actionable narrow boilerplate findings; GPU manual
implementations not presumed derivable without dependency/equivalence proof.

Connected repairs: RECEIVE-D1, RECEIVE-D2. Dependencies: Pinned thiserror source
and #826 agreement for shared error owner.

Acceptance: Equivalent API/bounds/format flags/causes; no new serialization or
dependency solely for a derive.

Status: receiving inventory incomplete; no clearance.

### P11: magic-numbers

Generated workgroup widths are shader declaration policies and reflected launch
owners consume them; four texture-store lanes, vector dimensions and binding
ordinals are explicit native formats. DEFAULT_NEIGHBORHOOD_SIDE64 owns existing
fallback policy. Fluid dt/half-inverse-cell-size docs explain current native
texel/time/numerical frame; no physical SI policy invented.

Disposition: Native generated51 roles reviewed; broad
physical/noise/JFA/count/frame occurrence closure incomplete.

Connected repairs: RECEIVE-G2, RECEIVE-G3. Dependencies: Caller
units/parameters, noise and JFA public contract closure; native limits.

Acceptance: Every remaining primitive has actual boundary/owner and frame; keep
sorting vs launch quantities distinct and preserve supported values.

Status: receiving inventory incomplete; no clearance.

### P12: magic-strings

WGSL identifiers/type names/declarations/format fragments are explicit source
protocol boundaries; closed algorithm/binding/failure classes use enums. No
display-text branch introduced. Source code Strings remain open authored input.
Min/Max exact replacement currently disagrees with native signature lexical
admission.

Disposition: Typed generated policy roots and open native source text inspected;
Root reduction G3/G4 remain.

Connected repairs: RECEIVE-G3, RECEIVE-G4. Dependencies: Root reduction error
vocabulary and canonical source-owner agreement.

Acceptance: No presentation string as policy identity; retain native lexical
acceptance, wire syntax and exact error causes.

Status: receiving inventory incomplete; no clearance.

### P13: behavior-preservation

All incoming fluid program string occurrences exist identically in current Root
source with multiplicity; root-only strings are diagnostics/binding
names/cache/entry tokens. Map fixture files byte-identical to current Root;
Gather/Stream changes are final-API-only, Stencil preserves existing moved test
scenarios/oracles with getter/constructor adaptation. Clamp upper-endpoint min,
fallback64, flat2D/3D coordinates and projected texture lanes inspected
directly.

Disposition: Current incoming bodies/fixtures read, direct fluid literal
evidence and fixture diffs reviewed; full base/main/hosted/native behavior
clearance not achieved.

Connected repairs: RECEIVE-G1, RECEIVE-G2, RECEIVE-G3, RECEIVE-G4, RECEIVE-C1.
Dependencies: Remaining exact inverse source proofs, actual lower
provider/caller receipt, main/PR re-pin, final hosted checks.

Acceptance: Preserve all existing scenarios/oracles/native instructions except
explicitly inventoried repairs; no new tests/cases/helpers/assertions; complete
families before push/CI.

Status: receiving inventory incomplete; no clearance.

## Canonical composition and native contract

Compose incoming GPU83 then Buffer919 then generated51 overrides; preserve
current Root capacity/stride modules and broad Compute exports. The exact
SourceCache bytes and GPU Cargo.toml already match current Root. SourceCache
keys source, complete recipe, real device and native observation policy; failed
builds never publish. Do not duplicate native
bindings/pipeline/submission/validation functions. Protected edits require
explicit ownership agreement.

ResourcePipeline derives workgroup shape and storage stride from the same
immutable source. BufferElementCount owns full-record quotient and checked u32
narrowing; BufferElementStride includes padding. Native launch
InvocationCount/WorkgroupCount are separate. The Stencil non-controlling input
stride uses storage_stride_for(Input), so copying only the older Root
ResourcePipeline loses a required API. BoundaryMode is the canonical Root
Clamp/Wrap owner and must remain exported.

Incoming ComputePass dispatch and WgpuContext submit distinguish submission from
completion. Native validation scopes retain actual polling and SDK failures;
browser/nonblocking paths use configured device error handling without a
synchronous verdict. None of these prove later device health or rollback.

Incoming GPU Cargo.toml exactly equals current Root Cargo.toml. Compute already
depends on thiserror2, GPU/math/mesh, naga29/wgpu29; current Cargo.lock package
entries reflect those edges. No generated51 new manifest dependency identified.
Full lock resolution/target-feature closure and project check remain unexecuted,
incomplete. Preserve unrelated PBR Color edges.

The incoming packet still retains ResourceBaseType::element_size pending Root
final consumer closure. A fresh repository search found only its recursive
self-calls in the inspected crates, and Reduce source now uses real stride; that
search is not complete proof that the public obsolete guessing API has no actual
consumers. Close the final actual consumer graph and coordinate its removal with
#826.

Native surface extraction was traced through grid admission, same-grid
Gather/Scan/Stream, checked prefix limits, whole-triangle/quad writer
capacities, sentinel cell publication and shared deinterleave. Existing native
arithmetic/ties and the local normal finite-difference policy were read, not
changed or verified on hardware.

For fluids, a direct current-Root/incoming-program Rust string multiset
comparison found every incoming literal with exact multiplicity; only old
diagnostics and orchestration identifiers remain outside the incoming program
literals. This supports source extraction provenance, not instruction ordering,
native execution or numerical equivalence on all devices.

## Limits and remaining acceptance

- Most lower native83 implementation and tests/config bodies are identity-only in composition_accounts, not fresh semantic coverage.
- Root marching_cubes/tables.rs, Reduce tests/error owner, general test_utils and complete unchanged extraction fixtures/downstream caller fixtures are not fully read here.
- All aliases/macros/reexports/repository-wide consumers beyond explicit named caller map are not exhaustively closed; search results are discovery only.
- Noise/JFA typed physical/frame/count/seed policies need complete outer producer/caller inventory; source-only numerical behavior is not hardware proof.
- Main and PR remote heads were not refreshed in this subtask; final whole-PR review must pin them.
- No project checks, compile/lint/format commands, scans, native shader generation, tests, runtime, new fixtures or CI ran. No source edit, integration, push or external post occurred.

Current Root lib.rs host_float summary remains unqualified while owning
host_float/mod.rs Limits states subnormal/fma/trig bounds. This broader known
numerical prose family is not cleared by this generated receiving report.

No repairs were made. Complete the remaining inventory and connected families
before push or hosted CI; repeat all thirteen lenses and independent Code, Rule
and Comment reviews, then require hosted project checks on the same final pushed
revision. Do not merge.

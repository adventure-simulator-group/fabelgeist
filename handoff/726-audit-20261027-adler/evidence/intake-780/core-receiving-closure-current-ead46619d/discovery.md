# Root receiving closure: source inventory extension

Pinned Root HEAD `ead46619dec5ef45993f63202cb289bd3986b018`. No production edits
or project execution.

This extends the preceding receiving discovery with fresh actual fixture,
catalog, owner and consumer reads. All thirteen lenses have explicit independent
dispositions below. Complete/incomplete scope is per source account; searches
and prior verdicts are not clearance.

## Actionable connected findings

### CORE-R1 (P2) — Connected submission API/fixture migration gap

Nine existing fixture calls discard the returned Result. This loses the direct
submission cause; the hosted workspace/all-target Clippy step denies warnings,
including unused Results.

Contract: KernelBatch::submit returns the native submission Result. Existing
result-bearing functions/worker contexts must handle it before
readback/publication.

Connected repair: Adapt these existing calls with their established ? or worker
unwrap convention, preserving every scenario/helper/oracle/assertion byte.
Complete the discovered external submission family separately before claiming
whole-repository closure.

Acceptance: All nine actual calls consume the Result; Same fixture population,
inputs, output oracles and assertion invocations; Required hosted checks on
final pushed revision, no local execution.

### CORE-R2 (P2) — Connected canonical count/launch and clean constructor API gaps; independent mathematics excluded

All four immutable cached recipes still divide raw output bytes by four, cast
unchecked to u32, and manually divide by64 for launches. Their constructors
accept but ignore code and context. Cache identity itself correctly includes
immutable source, real device and validation policy.

Contract: Native f32-array counts and dispatch shape consume canonical
count/stride/launch owners; constructors do not carry ignored source/context
parameters.

Connected repair: Inventory a final canonical count/shape and constructor API
for these actual cache consumers without adapters or guessed stride. Remove
unused constructor arguments directly or otherwise choose the clean final unused
API disposition. Preserve the current kernel numerical behavior unless separate
functional work is authorized.

Broadcast/Reshape/Transpose are f32 copies and MatMul copies input_a while
ignoring input_b. No actual in-repository callers or fixtures were found after
reading named occurrences and export chains. This pre-existing placeholder
mathematics does not authorize implementing new algorithms during this
migration. It remains explicitly outside sorting/extraction functional scope,
while connected owner/API gaps remain required.

Acceptance: No unchecked byte-to-u32 count or repeated guessed workgroup
calculation; No ignored native definition arguments/temporary forwarding APIs;
No new algorithm, fixture, case, helper or assertion.

### CORE-R3 (P2) — Connected Mesh cache-consumer launch/producer migration gap; broad geometry family open

Displacement narrows byte length to u32 before dividing by12 and hard-codes64
groups; GpuMesh retains raw vertex_count separately. Canonical native array
stride/count and Mesh vertex count owners are not propagated through this
producer/consumer family.

Contract: Displacement dispatches grouped xyz vertices from packed native scalar
storage; storage words, mesh vertices, bytes and launch groups are different
quantities.

Connected repair: Trace packed-f32 source record count through the actual native
declaration, derive the grouped vertex population with the owning Mesh count
policy, and use canonical launch shape. Do not call12 a reflected WGSL array
stride: the shader declares array<f32>, with three words per vertex. Finish
actual producer/caller admission before changing retained vertex_count.

Acceptance: No narrowing before record-count division; One source for admitted
vertex population/actual packed prefix; Ordinary native arithmetic, output
layout and all existing geometry scenarios preserved; Broader GpuMesh geometry
frame/topology/finite/source validation not falsely cleared.

### CORE-C1 (P3) — Confirmed signed-count versus unsigned-address comment mismatch

The comment says the admitted address extent is below i32::MAX, but admission
checks BufferElementCount checked_mul against u32. The threshold clamp is
justified by the strand population per lock, not the unsigned address extent.

Contract: CardWork admits u32 lock addresses; signed per-lock strand counts are
below i32::MAX because lock stride is at least12 words.

Connected repair: Describe the bounded per-lock strand count and retain the
exact threshold clamp/native instructions.

Acceptance: Doc differentiates signed counts from unsigned addresses; No
CardWork/LOCKS numerical or test change.

### CORE-C2 (P3) — Fixture helper coverage documentation overclaims supported configurations

run_compute_test says it iterates all compatible resource types, but the format
table omits supported numeric Vec3 texture lanes and other source shapes. Its
actual scope is the configurations selected by the fixture table.

Contract: get_formats supports scalar, Vec2 and Vec4 lane projections;
unknown/custom/Vec3 cases receive no texture formats and still get the Buffer
configuration.

Connected repair: Narrow the existing runner comment to its actual selected
table/configurations; preserve helper behavior and all existing oracles.

Acceptance: No universal compatible-resource coverage claim; No new
test/helper/case/assertion or fixture behavior change.

### CORE-N1 (P2) — Outer noise semantic producer boundary remains required

The public producers carry raw octave, lacunarity/gain and Vec3 scale/offset
with no frame/unit docs. Canonical Ratio exactly fits lacunarity and gain
without adding bounds. BufferElementCount, InvocationCount, PerMetre and Seed do
not describe octave iterations, texel-to-noise transforms or nearest-feature
coordinates.

Contract: Both producers consume index/texel coordinates, transform to
noise-lattice coordinates, iterate octave count, and multiply
frequency/amplitude by dimensionless ratios. No RNG seed parameter exists.

Connected repair: Complete the final native numerical handoff contract: reuse
Ratio through the actual parameter boundary, give octave iteration count its
justified owner, and document the index/texel-to-noise scale/offset frame. If
raw vectors are retained at this immediate mathematical/native boundary, record
that actual role rather than inventing metre units. Preserve every native
hash/permutation/coefficient instruction and accepted value, including zero
octaves.

Acceptance: Canonical matching owners consumed without arbitrary new
clamp/finite policy; Frame/unit/reason for native coefficients explicit; Same
generated Map parameter names/order/source and output formats; No new seed
policy, random algorithm, fixture or assertion.

## Bounded complete/incomplete scope map

### Reduce fixtures and numeric readback

source scope closed; execution unperformed.

Six existing numerical scenarios: f32 sum128, f32 min/max5, two-field Item
custom minimum,10x10 texture minimum and numeric minimum. EmptyReductionResult
owns missing projected host component.

Limits: No existing
zero/one/padded-vec3/reused-scratch/foreign-device/custom-whitespace scenario is
authored here. run_compute_test initializes output from expected values; a no-op
can preserve an expected output. No claim that passing current fixtures proves
all writes/branches. No scenario/helper/oracle repair is authorized by this
report.

### Extraction catalog/front entry

source catalog and actual named entry/export closure read; no existing
extraction fixture found.

EDGE_TABLE256, TRI_TABLE4096 in16-word cases with-1 sentinel and edge
ordinals0..11, TRI_COUNT_TABLE256 word counts. The exact count table is uploaded
by MC and also supplies host maximum for prefix admission; shared native
writers/stride were read in the preceding current-source pass. AdvancingFront
directly calls shared DualContouring in shortest-diagonal mode with one vertex
budget projected to index slots.

Limits: No new fixture is invented. Fixed catalog/source preservation is
required; broad SAFT/geometry/finite-performance capability is not inferred.
Three catalog arrays are one immutable authored protocol; no mutable competing
producer found.

### Noise/JFA actual outer callers

named caller/export closure complete for current first-party source; native
numerical/type contracts remain open as N1.

Noise only has public definitions and direct reexports; no in-repository invoke
found. JFA only has its two existing center-pixel/voxel tests. JFA reads
mask>=0.5, stores feature coordinates in RG32/RGBA32, carries step as the native
uniform, alternates temporary textures, and outputs distances in index/texel
units. Its seed variables are nearest-feature coordinates/validity, not RNG
seeds.

Limits: Preserve fixed sentinel and validity values, float/integer
conversion/step order, strict tie behavior and existing tolerances. Current
tests do not cover arbitrary dimensions/multiple/tied/no features; no new cases
are authorized. Texture-native/source/device numerical closure still belongs to
its native owner.

### Canonical count leaves in actual consumer state

all named direct leaf occurrences classified; whole surrounding physical
families remain open.

Domain source counts stay separate from native binding floors, float/vec4
storage words, padded mip rows and invocation/workgroup/sort populations. Hair
shape admission bounds strand*point; CardWork checks count*lock stride and+1
sentinel. Collision sole make_sim producer clamps points>=2 before subtraction,
including empty population. Rod factors validate14 vec4 words/sample. Metal
side32..1024 bounds square and checked word expansion. Creator fixed2048 chain
checks total bytes/word projection and retains padded rows.

Limits: Do not derive logical zero domain population from the floored native
allocation extent. Do not infer
geometry/normal/skin/source/device/physical/numerical health from cardinality
checks. Generic Scratch fit actual device/source proof and surrounding
Card/simulation/material/Grid/kernel families remain broader required work.

### element_size removal and export/alias/macro closure

current named consumer closure supports removal after actual canonical Reduce
receipt and #826 agreement.

element_size has no current named first-party consumer beyond self-recursive
method branches. Its guessing implementation is distinct from
component_count/base_type/as_str used by actual consumers. Compute exposes its
algorithms through lib and prelude, with GPU glob imports pub(crate) only; GPU
publicly reexports its own data/compute/buffer vocabulary. Existing Map/Stencil
fixture macros call their admitted final signature helpers. Generic params!
inserts via Into<PassParameter> at actual native binding boundary.

Limits: Method removal is #826-owned and conditional on actual Root Reduce
canonical receipt; do not restore a forwarding or guessing adapter. Whole
macro-generated/unavailable external downstream code is not claimed audited.
False-positive Stream/Map matches resolve to Hair RNG and serde_json imports,
not generated APIs.

## Sequential thirteen-lens extension

### 1. producer-propagation

Direct signature/ResourcePipeline/BoundaryMode/cache/count uses and actual named
constructors/callers are accounted; R1/R2/R3/N1 plus previous six MC/DC/module
composition remain connected.

Acceptance: Trace source count/frame once through actual producer and every
retained consumer; finish final API callers before integration.

Dependencies: exact received generated/native owners; prior Root connected
findings; all13 pre-edit inventory; final same-revision reviews and hosted
checks. No whole-family clearance.

### 2. existing-type-owners

Ratio fits noise gain/lacunarity; canonical count/stride/WorkgroupShape owners
fit byte/launch calculations. Storage words are not vertex/strand/octave/RNG
counts.

Acceptance: Reuse exact owners, preserve named semantic roles and true native
projections; no borrowed wrong-unit wrappers.

Dependencies: exact received generated/native owners; prior Root connected
findings; all13 pre-edit inventory; final same-revision reviews and hosted
checks. No whole-family clearance.

### 3. admission-contracts

Leaf consumers trace checked StrandShape/StrandLayout/Card/Metal/mip admissions.
Empty points candidate is closed by the sole >=2 producer proof. R2/R3 unchecked
byte casts remain actual owner gaps.

Acceptance: Every current valid-zero, host/native-width and fallible shape path
reaches correct admission without a compatibility path or arbitrary new bound.

Dependencies: exact received generated/native owners; prior Root connected
findings; all13 pre-edit inventory; final same-revision reviews and hosted
checks. No whole-family clearance.

### 4. invariant-operations

Hair count expansion products are bounded by actual admitted shapes/checked_mul;
mip/tile fixed bounds retain safe products. R2/R3 casts and unconsumed submit
Results remain.

Acceptance: Keep arithmetic/order/value domains; canonical checked
quotient/launch projections and no premature publication after failed
submission.

Dependencies: exact received generated/native owners; prior Root connected
findings; all13 pre-edit inventory; final same-revision reviews and hosted
checks. No whole-family clearance.

### 5. aggregate-states

Count/native floor separation is explicit; current consumer records group
buffers for admitted shape. Existing fixture output seeding limits evidence.
Mesh retained vertex_count versus positions remains open.

Acceptance: One source for population/extent/device where actually required;
complete surrounding state contracts before whole-family verdict.

Dependencies: exact received generated/native owners; prior Root connected
findings; all13 pre-edit inventory; final same-revision reviews and hosted
checks. No whole-family clearance.

### 6. enum-shapes

JFA ping and ResourceType predicates are local control, while feature-coordinate
validity is a native ABI. No RNG Seed owner fits feature seeds. Named
source/error roles remain typed.

Acceptance: No carried semantic bool authority or enum payload mismatch; retain
actual native validity and independent axes.

Dependencies: exact received generated/native owners; prior Root connected
findings; all13 pre-edit inventory; final same-revision reviews and hosted
checks. No whole-family clearance.

### 7. redundant-authority

Source caches consume actual immutable/native source and device/policy key; no
cache fix inferred merely from module API match. Placeholder kernels are
immutable authored copies; mathematics is out of active scope.

Acceptance: Connected ignored constructor args and duplicate layout/count
computations removed or assigned clean final API; outside functional disposition
explicit.

Dependencies: exact received generated/native owners; prior Root connected
findings; all13 pre-edit inventory; final same-revision reviews and hosted
checks. No whole-family clearance.

### 8. behavior-preservation

Read existing Reduce/JFA/Map/Stencil source/helper frontiers and9 ignored submit
fixture calls. Preserve all cases/oracles/native strings; hosted execution
remains future acceptance.

Acceptance: No new tests/helpers/cases/assertions; exact native/catalog/fixture
source preservation except inventoried API/cause/prose repairs.

Dependencies: exact received generated/native owners; prior Root connected
findings; all13 pre-edit inventory; final same-revision reviews and hosted
checks. No whole-family clearance.

### 9. derivability

Previous Reduce Default and GeneratedProgramError Error-only findings remain
routed. Native validation/TryFrom/semantic frame conversions are not mechanical
derives; unchanged fixture helpers retained.

Acceptance: Keep exact bounds/Display/source semantics; no new dependency or
wire schema solely for derivation.

Dependencies: exact received generated/native owners; prior Root connected
findings; all13 pre-edit inventory; final same-revision reviews and hosted
checks. No whole-family clearance.

### 10. magic-strings

WGSL declarations/parameters/case tables and fixture text are native/source
boundaries; fixed Root reduction policy strings remain previously inventoried.
Public ignored constructor source text is not justified data.

Acceptance: No display-based branching or competing parser; canonical native
names/source roles consumed, no ignored compatibility arguments.

Dependencies: exact received generated/native owners; prior Root connected
findings; all13 pre-edit inventory; final same-revision reviews and hosted
checks. No whole-family clearance.

### 11. magic-numbers

Fresh all direct count/stride consumers distinguish padded rows,
lanes,words,vertices,strands,launches. Noise frame/octet and raw Mesh/copy
launch sites remain required; fixed fixture/native catalog literals retain
concrete role.

Acceptance: Ratio/unit/count owner or exact native numerical-boundary frame per
value; no accidental modulus/finite/clamp policy changes.

Dependencies: exact received generated/native owners; prior Root connected
findings; all13 pre-edit inventory; final same-revision reviews and hosted
checks. No whole-family clearance.

### 12. type-ownership

Root sole implementer; #826 owns shared native/generated remedies. New leaf call
sites fit existing low GPU owner; broad Mesh/physical/cache/submit families are
explicitly open outside this source subset.

Acceptance: Coordinate claimed shared function edits and preserve Root
exports/capacity/stride; do not implement unrelated algorithms.

Dependencies: exact received generated/native owners; prior Root connected
findings; all13 pre-edit inventory; final same-revision reviews and hosted
checks. No whole-family clearance.

### 13. type-granularity

Canonical bound-record count is not complete logical strand/card population,
octave iteration count or feature identity; launch quantities stay separate from
sort/storage. C1 corrects count/address prose.

Acceptance: Role distinctions prevent real mixups without wrapper proliferation;
native vector frames documented at actual boundary.

Dependencies: exact received generated/native owners; prior Root connected
findings; all13 pre-edit inventory; final same-revision reviews and hosted
checks. No whole-family clearance.

## Frontier and proof limits

The sibling JSON identifies every fresh source hash/read extent, earlier
actual-source receipt rechecked against current bytes and reassessed for the
receiving contract, raw occurrence candidate and ignored-submit frontier. Most
outside submission bodies remain unread. No exact count of correctly typed
external submit receivers is inferred from spelling.

The empty-collision subtraction candidate is withdrawn as a reachable bug:
actual make_sim admits at least two points before the sole CollisionBuffers
constructor call, even for empty populations. Preserve that producer
relationship; do not impose a new empty-shape restriction.

No existing MC/DC/extraction fixture was found in the read module/test target
graph. The immutable marching-cubes tables and actual entry/export paths were
read; no fixture has been invented to replace missing coverage. The two existing
JFA tests and six reduction scenarios have exact source receipts. A passing
helper can preserve expected preinitialized output, so fixture success must not
be described as proof of every generated store.

Noise has no current external call site or serialized recipe in the inspected
first-party graph. Its native fixed hashes are not missing random seeds. JFA
feature seeds are coordinate payloads, also not RNG state. Ratio reuses the
actual dimensionless multiplicative meaning without clamping or a finite-only
policy. Arbitrary physical metre units must not be invented for index/texel
coordinates.

Public native ShaderParameter/ResourceBaseType owners remain open native
metadata, not certificates of complete shader/device validity. element_size can
be removed by826 after final actual canonical Reduce receipt: complete named-use
closure found only definition/self-recursion; component_count/base_type
consumers remain and were read. No forwarding method or stride guess should be
restored.

- Complete external ignored-submit actual bodies and their publication/error policies; submit_frontier entries without body receipts remain unread discovery.
- Finish independent broad Mesh geometry/displacement producer, retained vertex count and actual consumers; no Mesh whole-family clearance from cache-owner review.
- Broader Hair physics/Card/Grid/material/scalp native source/device/count/frame/state paths remain open unless independently current reviewed; cardinality source evidence here is bounded.
- Receive native91 exact Rule review for lower provider bodies, but never use that as Root caller clearance.
- Compose exact modules/exports/counts/strides/locks/baseline reductions after all13 preinventory; do not leave element_size fallback after final receiver census.
- Final current-main/PR-head and complete final source/reviews/required hosted checks must agree on one pushed revision; no merge.

No source edits, integration, native/source generator execution, project
checks/build/lint/format/scans/tests/probes/newcase/helper/assertion, CI
request, push or external post. Reports/hash/source-diff/occurrence discovery
only.

INCOMPLETE_WHOLE_RECEIVING_FAMILY; explicit source-scope inventory and
actionable connected gaps, no historical/search-based clearance.

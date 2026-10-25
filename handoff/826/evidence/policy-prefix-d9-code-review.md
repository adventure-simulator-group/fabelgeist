# Independent Code Review: policy and prefix checkpoint d9

Head: `d9c3521f30f52d3437cfef2850203c9df3192b0e`.
Tree: `660e61a63abe42f2ff07b50114b34729def12ffe`.
Parent: `8b32a1860079c64073028033b9fc4629a64ef451`.
Comparison checkpoint: `73b6e3a2ddf319dec749e3b1ff4a25e99b9c6612`.
Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.

The three previous findings are fixed. One connected logical-write finding
remains at this snapshot. This is an intermediate source review, not complete
renderer-family, compilation, thirteen-lens, whole-PR or final clearance.

All 8 changed files were read fully. The JSON records 41 source
identities, exact coverage and eight SDK proof hashes. The complete current
policy/renderer receiving family and buffer prefix/read/map/bind/extent paths
were reviewed; unrelated pending producer bodies are not claimed accepted.

## [P2] Enforce the logical prefix in Buffer::write

`crates/fabelgeist-gpu/src/data/gpu/buffer/mod.rs:53-55`.

An ordinary parent with logical16 and native allocation16 admits prefix4.
Calling prefix.write with eight bytes goes directly to
BufferUpload::write_to(nativeBuffer,START), so native validation sees capacity16
and accepts bytes0..8, mutating bytes4..8 outside the admitted logical4 view.
The same request via write_at(START,upload) correctly returns a bounds error.
Zero prefixes likewise allow nonempty writes through this entrypoint.

prefix.rs36-48 checks requested<=logical and physical then returns same Arc with
size=requested; mod.rs53-55 never reads self.size. upload.rs52-59 hands only
native handle/offset/data to queue. mod.rs69-78 write_at enforces
end<=self.size. No runtime witness or stronger arbitrary alias guarantee
required.

Affected unchanged consumer gap exposed by the new canonical admitted prefix
path, not a defect in prefix range predicate and not a pending780 caller
adaptation. Root has separately inventoried the complete write/write_at
extension; this report remains pinned to d9 where it is not repaired.

Minimum repair: Route zero-offset write through the existing checked logical
write_at operation, retaining actual native upload validation/cause, exact
device/queue preconditions and valid zero-length native behavior. No new
duplicate bounds owner or compatibility API.

Acceptance: No queue upload occurs for bytes beyond the current logical extent;
both write entrypoints agree at START while valid aligned in-range uploads
preserve bytes and native errors. Original tests only; this review executes
none.

## Previous findings

P73-C1 — fixed at this snapshot: lod.rs35-40 publicly reexports canonical
Representation from private policy owner; all importing modules retain one
public path.

P73-C2 — fixed at this snapshot for the reported finite history transition:
History Empty/Current now models temporal image validity; accumulating with
!current triggers invalidate_view before frames/weight. Single-frame resolved-
only→multi target resets to0 and first blend weight1, publishes Current only
when accumulation recorded. Existing current history can extend target without
unnecessary reset.

P73-C3 — fixed at this snapshot: DrawPlan logical count clamps only to native
u32 representability before handover; each actual draw is then independently
clamped to resident population in render. At amount1 incoming endpoint reaches
its own capacity. handed upper-bounded by integer count after float rounding, so
subtraction cannot underflow near large u32 counts.

## Contracts revalidated

Fresh complete current renderer and policy/count/quality/preset/frame-
state/uniform/draw/deformation/readback source review, not merely checking the
three prior finding locations. Full current math, all three shaders, error
carrier and material sources read. Dataset Sample reordering changes no
method/body/serialized field order. Unchanged manifests/shared leaves/callback
helper and partial producers separately identified by exact source hashes.

Canonical Buffer::prefix is pure admission: native size getter returns cached
immutable descriptor extent; compare requested against parent logical size and
physical extent, including forged parent logical metadata. Reject both overruns
with named typed requested/logical/physical byte fields, allow zero/unaligned
logical bytes, clone same Arc, original untouched, no
allocation/copy/map/submit/wait. Nested prefix cannot regrow a reduced logical
parent. No global sealing of public Buffer fields or proof external aliases obey
context/mapping rules claimed.

Buffer read checks physical extent, then MappingExtent checked ceil-to-
COPY_BUFFER_ALIGNMENT before host Readback admission. Logical3/native4 maps4 and
returns3 u8 bytes; logical4/native6 maps4; logical0 returnsempty after host
ZST/partial-element check and no native request. A prefix whose aligned map
needs unavailable padding still returns a typed read error even when prefix
admission itself succeeds. Binding requires nonzero logical extent and bounds
actual allocation independently; native binding alignment/capability remains
provider admission.

MappingRequest arms cleanup before immediate validation observation; failed map,
canceled await, poll/channel/native error drops request and unmaps under native
cleanup-only scope without replacing first failure. Mappedview drops before
release/unmap; host vector allocated in Readback::new before mappedview. Native
progress Wait remains explicit, browser await callback yields eventloop. No SDK
scope crosses await. Readback destination uses AnyBitPattern, checked host
count, whole elements, allocate-before-copy and set_len only after exact copy.
These contracts are unchanged at this pin.

Ranged KernelBatch copy separately bounds both logical operands with checked
ends; native scope/alignment/usage/physical/device/samebuffer checks remain
provider/context and deferred finish policy. No prefix consumer now silently
treats zero as a nonempty binding. Buffer::write remains the one newly reported
contrary logical consumer.

Renderer admission checks actual Globals including moving head matrix before
replacing pose in still-view key; finite effective pixels/minwidth/strand
widths/blend amounts before scene publication/framequeue. Converged recomputes
selected representation/blend and effective shading/enabled shadow resolution
from cached size. Nonfinite rejected state returns typed RenderInputError cause
rather than NaN equality loop. No stronger arbitrary finite shader-output or
numerical-physics validity claim.

History now has semantic Empty and Current(slot), no redundant generation. Scene
replacements reset last/frames/history. Resolved-only static frames leave Empty;
first later accumulating frame resets framecount, uses weight1, writes alternate
slot, records Current and count1. Current mean extended to larger target
preserves samples; moving keeps max(weight,0.35) and per-level Pending
obligations. Sample target0 still means1. Saturating u32 count and u64
denominator avoid arithmetic wrap.

DrawPlan shared logical prefix is split before per-level resident clamp, native
population saturates to u32::MAX before narrowing, rounded handed upper-clamped
to integer population. Therefore ranges never invert and cards→strands
subtraction cannot underflow. Amount generated by finite policy selection
lies[0,1]; external arbitrary Selection is not a public constructor of private
DrawPlan. Card outgoing/incoming threshold behavior preserves exact supplied
endpoint-one cuts. Native layout/population admission remains780 required
integration.

Full renderer raw-native constructor/shader/layout/group/upload/deformation/head
/cover/resize/render/readback ports keep agreed device-handled error policy and
device+pairedqueue/resource preconditions. No observed native pass/transactional
source publication or browser-safe synchronous headless constructor inferred.
Private staging cancellation/drop proof exactSDK identity unchanged, no shared
alias guard invented. Readback checked format/row/image allocation/BGRA reorder
and typed map/poll/channel causes remain.

## Required integration and validation limits

D780-producers — REQUIRED incomplete: HairLods::counts lod.rs219-225
deliberately unchanged and still supplies rawusize to bespoke fields;
GpuLods::counts gpu/lods.rs66-73 has same required mismatch. Actual native
strand/card/sim/atlas checked byte/population/layout producer integration,
sourceassociation, folded addressing and creator/publiccaller adaptation must
land canonically with780 before source completion. These are not new receiver
findings and are not accepted deferrals.

D780-head-source — REQUIRED incomplete: Current set_gpu_groom only replaces
cover onSome, and set_head replaces surface/cover; canonical associated
head/groom source identity and column validity remain open. Numeric wrappers
cannot establish this relation.

D780-lock-caller — REQUIRED incomplete: Cargo.lock must record futures-channel
and new fabelgeist-mesh dependency; named selected entry/API/count/caller
migrations and retained774/M06/F19 paths remain required. No complete compile
claim.

empty-resource-layout — REQUIRED incomplete: Policy Empty semantics alone do not
repair native empty strand upload infos slice and native empty/storage binding
rules; producer/layout receiving family remains open. Same malformed public
resource arithmetic/indexing scopes remain required, not silently cleared.

No source edits, local project checks/build/tests/scanners/generators/runtime,
new tests/cases/helpers/assertions/doctests, push or CI. Source/Git and ignored
provenance reports only.

All8 changed files fully read; full current policy/renderer receiving family and
buffer prefix/map/read/binding/extent/upload/error/host-readback/existing test
files freshly reviewed. Producer partial proofs and unchanged manifest/shared
callback proofs explicitly distinguished from fresh full reads.

Prior3 findings fixed at this snapshot; one new connected logical-write finding
remains. HairLods/GpuLods raw counts, creator/native layout/head-
source/caller/lock/retained774/M06/F19 paths remain REQUIRED incomplete, not
accepted deferrals.

No full family/sourcecompile/13-lens/PR/final clearance. Final independent
Code/Rule/Comment reviews and required hosted checks must repeat on same final
pushed revision.

Existing renderer and dataset test modules, buffer tests and readback test
module are byte-identical to the comparison checkpoint. All three shader
files are byte-identical. No tests or project checks ran. The root pre-repair
ledger now separately identifies the complete write-consumer extension; this
review reports the immutable d9 code without credit for later source edits.

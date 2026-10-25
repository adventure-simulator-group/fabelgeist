# Immediate-admission checkpoint Comment Review

Status: **no new owned comment mismatch; owner dependencies remain**.
This is an independent source checkpoint review, not final clearance.

## Source identity

- Independently resolved HEAD:
  `0bc4fd52bc376a22c6f7e73bc7f9ea427f662f17`.
- Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.
- `git status --short` was empty at entry; reviewed source remained at the
  exact HEAD above. The ignored report is the only authored artifact.
- Compared the complete current scope with the previously read reflection
  checkpoint, `2eb1129b0a8d09289aab9e7cc83bc3499a8a3d9e`. Independently verified
  that exactly three source/document paths changed: pipeline error,
  reflection admission and the GPU guide. The remainder of the previously
  inspected comment graph has identical source, not an assumed inherited pass.
- Applied the Comment Review skill and governing repository rules already read
  during this continuous review sequence.

## Immediate-data finding resolved

The previous reflection report's R1 is resolved by an explicit admission
contract rather than narrowing the prose around the skipped path.

In `pipeline/reflection.rs:30-50`, compute-entry usage is determined before
binding filtering. An actually used Naga `AddressSpace::Immediate` now returns
`ComputePipelineError::UnsupportedImmediate { name }` before native reflected
layouts are created. The variant's display text identifies the unsupported
named immediate without pretending to retain an SDK rejection cause. Its
source-less `Error` behavior is consistent with that host admission error.

The full guide's new immediate-data sentence matches this code. Both static and
cached dispatch paths provide no native immediate-value commands, and pipeline
layout size remains zero. Unused immediate globals still skip admission via the
usage check; ordinary used private/workgroup globals without bindings continue
through the no-binding filter. Resource usage is unioned over compute entry
points, so an immediate used by a later compute entry is not missed merely
because another entry supplied the default entry name.

No further wording or behavior change is requested for this repair.

## Current contract revalidation

Read the complete guide again with current source. Its following claims remain
consistent with the current implementing graph:

- Reflection preserves one shared compute layout from the union of actual
  resource uses and image sampling sets. No per-entry-point specialized layout
  is promised. A load-only float image does not acquire filterability unless
  another compute entry samples the same image. Comparison samplers,
  multisampling, depth/integer image kinds and storage access roles are carried
  to native descriptors. Provider features and limits still govern acceptance.
- Uniform admission handles direct scalar/float-vector/square-float-matrix
  values and flat structs, rejects unsupported representations and multiple
  used blocks per group, and records each admitted `UniformValueShape`.
  Static and fast packers check shape before their writes. Static missing or
  mismatched values retain zero; the cached path returns typed errors and
  retains its narrower supported matrix set. A wider supplied shape cannot
  overwrite the next member.
- Re-read unchanged `From<i32>` in `parameters/parameter.rs:125-128`, the shape
  admission and both packers. `IntegerWord` admits both declared native `Sint32`
  and `Uint32`; the existing adapter stores `value as u32` and both packers
  write its little-endian word. Signed-input native bits remain preserved.
- Binding arrays, external textures and unrepresented bound handles are typed
  pipeline admission failures. Unused bound globals impose no parameters.
  Used immediate data now has its own explicit admission rejection as above.
- Immutable shader/module/device provenance and immutable reflected pipeline
  interfaces remain accurate. `validate_naga` applies the same validation
  policy to module and reflected-layout admission. Native parser/validation/
  translation/creation causes remain concrete where available.
- Re-read pipeline cache outcome handling. Observed errors remain retained
  across clones/lookups; device-handled entries do not imply an observed native
  validation pass. A later readback-enabled lookup recreates an unchecked
  entry. Context policy documentation agrees with those branches and pinned
  wgpu's poll-entry submission semantics.
- Shared binding assembly receives the admitted pipeline, retains sparse
  reflected group indices, respects logical buffer prefixes and validates
  selected texture view format/dimension/device. Static uniform allocation
  checks padded limits before packing, allocates unmapped before upload and
  observes available native causes. Its corrected module comment matches the
  actual order.
- The three texture shapes, generic views, cube face/array geometry, cached
  format identity, allocation/cache absence states and removal-only owner APIs
  retain their documented contracts. Cache-first return after allocation
  detachment and alternate-view device admission remain distinct. Descriptor
  sRGB linearization concerns linear storage textures' alternate requests,
  rather than an unconstructible logical-sRGB storage state.
- Converter retained-device checks, source-then-destination cache absence,
  selected views, fixed shader workgroup dimensions, and native scopes around
  its pipeline/recording/submission still match guide/comments. It does not
  claim a native verdict unavailable to borrowed/browser contexts.
- Batch-owned uniform chunks, fresh groups and per-dispatch dynamic offsets
  match their comments. The revised fast-path prose makes no unsupported
  identity/barrier or measured performance assertion. Empty batch launches,
  standalone empty admission, folded ordering and after-submission callback
  ordering remain described accurately.

The complete base-to-head changed/moved documentation graph was accounted for,
including shader/reflection/error owners, static and fast binding/packing,
texture owners/views/converter, context policy, GPU README link, test comments
and workflow validation descriptions. Affected primitive/BVH/hair/mesh/physics
adapter comments and the existing texture-view fixture and sort zero-count
oracle contracts retain identical reviewed source outside the three-path
delta. Re-read the changed reflection/packing boundaries against those retained
producers/consumers; no new fixture or example incompatibility was found by
source review. Historical observations and hosted-check requirements in the
guide retain their stated limits; no passing run is inferred.

## Remaining dependencies and limits

The `Kernel` module introduction/fast-field allocation wording and `KernelCache`
compile-once statement still await the accepted #780 owner repair alongside its
producer/cache migration. These comments remain unresolved at this exact SHA,
not waived or independently repaired. All #780 count/shape/cache/source families
and #774 retained hair/armor/body/engraving families remain required integration
dependencies. Their final source and wording must be reviewed after integration.

No local project checks, builds, tests, scanners, generators, runtime, source
edits, new tests/cases/helpers/assertions, push, hosted CI request or external
posting occurred. Existing source and fixtures were read, not executed. This
review gives no hardware/browser/performance validation or source-complete
claim. Final independent reviews and required hosted checks must still agree
on the same final pushed revision after the connected families finish.

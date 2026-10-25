# Successor checkpoint Comment Review

Status: **two owned documentation findings and one pending owner finding**.
This is an independent checkpoint review, not final clearance.

## Independently verified snapshot

- `git rev-parse HEAD` resolved to
  `219d407be5b89e6a910244ba3362326088603701`.
- Base: `990430b77a9f63548ffcb642074185f61ddb3322`.
- `git status --short` was empty on entry and after source review. HEAD remained
  the exact revision above. The ignored report is the only authored artifact.
- Re-read the Comment Review skill, root rules and `crates/AGENTS.md`.
- Reviewed the full base-to-head change inventory and changed/moved comment
  diff, the full GPU guide and relevant implementation/consumer graph. The
  preceding checkpoint report was evidence to revalidate, not a clearance.

## Actionable owned findings

### S1 — Uniform module introduction conflates extent admission and allocation

Location:
`crates/fabelgeist-gpu/src/data/gpu/bind_group/uniform_buffer.rs:1`.

“Static uniforms use a checked allocation before host packing or queue upload”
places allocation before packing. The actual order is extent/limit admission
at lines 23-36, host packing at lines 37-38, native allocation at lines 45-51,
available validation readback at lines 52-58, then upload at line 66. The guide
already describes this order accurately. Keep the implementation and correct
the module introduction to distinguish these two checks:

> Static uniforms check device limits before host packing and observe available
> native allocation validation before queue upload.

This is a wording correction; no additional check or test is requested.

### S2 — Kernel's remaining introduction repeats stale dispatch/allocation claims

Locations:
`crates/fabelgeist-compute/src/kernel/mod.rs:12-16` and `51-55`.

The introduction says “Every other primitive here” uses standalone
`ComputePass::dispatch`. Existing `PrefixSum::record` (`prefix_sum.rs:240`) and
`RadixSort::record` (`sort/mod.rs:233`) instead record through `KernelBatch`.
Both are directly affected consumers in this change. The private `fast` field
also says every dispatch without the fast path builds a uniform buffer, but
the static assembler omits that buffer when reflected uniform size is zero
(`bind_group/uniform_buffer.rs:18-20`). Its unqualified CPU/allocation-cost
claim repeats the performance assertion intentionally removed from `fast.rs`.

Suggested introduction:

> `ComputePass::dispatch` creates and submits an encoder for a standalone
> dispatch. `KernelBatch` lets callers record multiple dispatches into one
> encoder and submit them together.

Suggested field documentation:

> The cached dispatch path, when this kernel's interface allows it. It shares
> the dispatch pipeline and stores uniform values in batch-owned slots. Other
> interfaces use `ComputePass::record`.

These replacements describe current behavior without assuming all primitives
use one API or asserting performance not measured at this revision. This
finding concerns documentation in the already-owned kernel/fast recording
graph; it does not request independent changes to #780's producer/cache design.

## Pending owner finding and dependencies

**D780-C1 remains unresolved:** `KernelCache` documentation at
`kernel/mod.rs:172-177` promises one compilation per distinct source, while
`get` at lines 188-201 permits duplicate racing compilations outside the write
lock and retains one published result. This was re-read at this exact head.
Its source identity, device policy and cache behavior belong to the active
#780 family. The coordinator must reconcile the documentation with that final
contract, then include it in the final review. Do not implement an independent
cache or mark this as an accepted deferral.

All #780 count/shape/cache/source producer families and #774's retained
hair/armor/body/engraving families are required dependencies. Core GPU ownership
was transferred and reconciled; the former core-function ownership blocker is
not carried forward. This report does not replace those retained owners'
integration or approval. The two restored hair adapter sites remain in their
owner's graph, not silently counted as finished by this review.

## Revalidated corrections and wider coverage

The preceding checkpoint's C1-C4 are corrected:

- Fast-path prose now accurately describes per-batch chunks and fresh bind
  groups. The unconditional identity/barrier explanation, unsupported timings,
  stale ring wording and allocation-free packing promise are removed.
  `kernel/fast/uniforms.rs` still allocates a vector for a matrix, and the revised
  capacity comment explicitly permits value writers to allocate. The retained
  BVH test is described as an observable refit guard rather than proof of a
  barrier mechanism.
- The cube-face optional context comment now describes identity checking on
  every target, matching the unconditional `ensure_device` call. The obsolete
  browser unused-variable annotation is removed.
- The guide now describes a linear storage texture's alternate sRGB request,
  and the constructors' exclusion of storage usage for logical sRGB. Removing
  the generic cached-view format duplicate and unreachable initial-view
  fallback matches those admitted paths. The unchanged conversion from
  `Texture2d` to `TextureView` carries the admitted cached format.

Read guide and comments with actual implementations for immutable shader/module
and texture provenance, all three texture shapes, cached/absent ports, selected
effective format/dimension/device, context policy, cube faces and array views,
typed native errors and standard error conversions. Cache-first sharing after
allocation detachment remains described accurately. Pinned wgpu 29.0.4 polling
and tracker semantics, inspected during this review sequence, remain applicable;
no new broad native-behavior assertion was introduced.

Re-read shared binding assembly and its now pipeline-admitted public API,
`ComputeBindGroup` sparse indices, static packing/checked byte arithmetic,
`Buffer::binding_resource`, and the newly separated uniform allocator. The
guide's unmapped-allocation and before-upload observation descriptions match
`BufferDefinition::allocate_native` (`mapped_at_creation: false`) and the scopes.
The padding comment agrees with installed pinned
`wgpu-29.0.4/src/util/device.rs:40-65`, which pads nonempty initialized buffers
to copy alignment. The sampler named record preserves its reflected binding
role; the assembly consumes its named fields.

Read the converter's shader, layout, pipeline/device admission, source-first
then destination absence checks, selected cached views, scopes around creation,
recording and submission, and fixed dispatch dimensions. Its comments and guide
description match those paths. The float sample type remains unfilterable for
`textureLoad`; the shader's workgroup declaration matches its dispatch divisor.

Re-read parser/shader errors, pipeline preparation and native cache outcome
handling, pass/grid/shape boundaries, kernel recording/arena/folded launches and
callbacks. The observed-versus-device-handled validation distinction remains
accurate. Preparation creates reflected binding layouts without compiling the
static compute pipeline. Typed errors retain available SDK causes; the guide
does not claim a native cause when no scope reports one.

The base-to-head consumer inventory includes the BVH, compute primitive,
hair, mesh and physics adapter graph. Revalidated the accessor/grid/byte API
adapters with their documented resource, empty-dispatch and copy ordering.
The restored hair `grow.rs` and `guides.rs` sites were explicitly read against
their local retained API. Their future producer migration is still an owner
dependency. Removed forwarding APIs have no lingering example in the reviewed
guide. The GPU README link resolves to the guide; root README/wiki prose are
untouched.

Re-read the public texture-view fixture JSON, existing fixture adapters and
assertion contracts, plus the changed zero-count expectation in the sort test.
Their documented tiny extents, absence priority, sharing, requested format,
native cause chain and zero-dispatch semantics still match their inputs and
oracles by source. No assertion was added or changed by this reviewer. Workflow
step names, serial integration target, separate doctest command and source/log
artifact paths remain consistent. The guide preserves its historical-evidence
and hardware/browser/performance limits. Some guide paragraphs still need
mechanical 80-column wrapping; this does not add a contract finding.

## Limits

No local checks, builds, tests, scanners, generators, runtime probes, new tests,
cases, helpers or assertions, source edits, pushes, hosted CI requests or
external posts were performed. Existing tests and examples were inspected, not
executed. No performance or hardware/browser acceptance was established.

This is a source checkpoint with unresolved findings and required owner
dependencies. It makes no source-complete or final-clear claim. Resolve the
connected families, then repeat all required independent reviews and hosted
checks on the same final pushed SHA.

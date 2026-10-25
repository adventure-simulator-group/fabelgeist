# Deferred submission post-repair Comment Review

This checkpoint has two actionable documentation findings: the submission
precondition must name the same logical device, and the Kernel compilation
comments retain a cross-thread scope rationale contradicted by the newly enabled
native `std` feature. Neither has been repaired in this frozen source.

## Exact source

- HEAD: `f2b5ffb1320bc6d3610e8726ac694546ed1a465b`.
- Tree: `f515184263ae03e6dc11eeee539f64fd44d7f364`.
- Main base: `990430b77a9f63548ffcb642074185f61ddb3322`.
- Pre-repair checkpoint: `5a34b608ccacc1950313b501d30b036f0a4f8b5b`.
- Tracked worktree was clean before review and report writes.
- All twelve repair paths and materially affected unchanged sources have SHA-256
  fingerprints in the adjacent JSON report. The reports are Git-ignored.

I applied the independent Comment Review skill and the root/crates agent rules.
This is a source-only review of the twelve-path repair and affected documentation
graph, separate from a final integrated main-to-HEAD review.

## C1: State logical device identity precisely

Priority P2. Locations:

- `crates/fabelgeist-gpu/src/globals/wgpu_context/submission.rs:6`.
- `crates/fabelgeist-gpu/texture-views.md:135-136`.

The encoder, context device and queue belonging to the "same native provider"
does not establish the required logical-device identity. Two devices can share
an adapter or backend. The helper's first sentence already requires an encoder
created by this context's device; the subsequent precondition must also require
the queue to belong to that same device.

Pinned `wgpu-core 29.0.4/src/device/queue.rs:1823` explicitly calls
`command_buffer.same_device_as(queue)`. Native encoder finalization routes an
error to the encoder's creating device sink; submission routes errors to the
queue sink. A scope on an unrelated context device cannot promise to observe
either error.

Use this contract at both locations:

> The encoder must be created by `context.device`, and `context.queue` must
> belong to that same logical `wgpu::Device`. Sharing a backend or physical
> adapter is insufficient.

Internal migrated consumers construct encoders from the supplied context device.
That observation does not prove arbitrary public `WgpuContext` field pairings.
State a precondition rather than imply the helper checks provenance. Root owns
this wording repair.

## C2: Remove the stale cross-thread scope rationale

Priority P2. Locations:

- `crates/fabelgeist-compute/src/kernel/mod.rs:30`.
- `crates/fabelgeist-compute/src/kernel/mod.rs:90-91`.

These comments say scopes on a device must not interleave and shader/pipeline
scope readback interleaves badly when two threads compile at once. The native
manifest now explicitly enables wgpu's `std` feature. In pinned SDK
`src/api/device.rs:434-435`, scope stacks are thread-local under `std`;
`src/backend/wgpu_core/thread_id.rs` selects actual `std::thread::ThreadId`.
The claimed cross-thread interference no longer follows from the SDK config.

Remove that explanation. If `COMPILING` remains, describe only its established
application purpose, for example serializing this module's kernel compilation.
Do not claim the SDK requires cross-thread scope serialization. This finding
does not require removing a lock without understanding its other purposes.

This is an actionable #780-owned Kernel comment correction. Root has requested
canonical wording/ownership agreement; no source was independently edited.

## Contracts checked against implementation and SDK

The new `WgpuContext::submit` surrounds `encoder.finish()` with a Validation
scope, consumes that scope, and returns the captured SDK error before queue
submission. A separate scope surrounds `queue.submit`. This addresses the
previously missed deferred boundary: Core copy recording queues an `ArcCommand`;
`encode_commands` performs encoding during finalization. Destroyed-resource
failures can be delayed again until submission, so the second scope is material.
The recorder comments and guide distinguish recording from native acceptance.

Core scope pop removes the current-thread scope and returns a ready future.
The synchronous helper consumes each guard on the same thread; the guard is
neither Send nor Sync. There is no `Device::poll(Wait)` in this helper, and no
claim that its submission index proves GPU completion. Native manifest `std`
selection matches the guide's thread-tracking claim. This is source verification,
not compiled target-feature validation.

The helper captures the Validation filter only. Other error classes use their
configured handler or matching outer scope; default uncaptured errors may panic.
It does not promise every error becomes its `Err`, later device health, rollback,
or a global native success verdict. Under native observing policy, the returned
cause is the actual SDK error. Under device-handled or browser paths, `Ok` does
not prove native acceptance.

The wasm helper compiles out synchronous scopes. The project's browser
constructor selects Browser WebGPU and disables blocking validation. Pinned
WebGPU scope pop produces a JavaScript promise and device polling is a no-op.
There is no browser event-loop or runtime observation here. Wasm can have other
providers, so this does not equate all wasm configurations with one backend.

`KernelBatch::submit` propagates `ComputePassError::Native` before application
callbacks. Those callbacks run in registration order only after helper `Ok`;
an error or discarded batch drops them unrun. This differs from SDK command-buffer
deferred actions, which SDK queue submission can execute even on an error, and
from `on_submitted_work_done` GPU-completion callbacks. The owning API docs say
host submission callback and mapping initiation rather than completion.

Standalone `ComputePass::dispatch` uses the helper and preserves the actual
cause. No `ComputePass::submit` method is introduced. The ten delegated texture
submissions use context-created encoders and preserve captured errors through
`ComputePassError::Native`. Read operations return an error before starting later
mapping on captured submission failure. They do not promise capture of earlier
immediate resource-creation errors outside their submission scopes.

The unchanged converter scopes binding creation and finalization together, then
submission separately. Its earliest native cause remains covered. There is no
basis to move its initial scope after creation solely to reuse the helper.
Separate explicit completion/readback operations retain their own wait behavior.

`RecordedDispatchCount` documents host-recorded dispatches independently from
items/workgroups/compilation. It does not claim final native acceptance or GPU
execution. Its increment follows successful wrapper recording; submit can fail.

`ComputeSourceCache` keys source, recipe, device identity and observing policy,
builds outside the lock, and publishes closure-`Ok` outcomes. Races can build
twice. The old exactly-once KernelCache promise is gone. Construction `Ok` does
not prove browser or device-handled native acceptance; no such verdict is given.

## Required integration dependencies

The #780 owner must finish owning readback docs and existing callers. At
`readback.rs:121`, "when submitted" must mean mapping initiation after helper
`Ok`; map completion is separate. A handle obtained before a failed or discarded
batch remains Pending because its mapping-initiation callback is dropped. The
owner must document discarding that handle. The guide now states this accurately,
but does not replace owning API documentation.

The module's `readback.rs:10-14` promise that the CPU never stalls is unsupported.
Native `Readback::read` performs an explicit device wait. Document actual
pending/polling versus waiting behavior without a blanket performance promise.
Existing fixtures at lines 248 and 274 still ignore the new submit Result;
owner adaptation remains necessary. Existing inputs, assertions and oracles were
only read; no test was run or changed.

Other accepted #780 Kernel prose remains incomplete: the module says every other
primitive standalone-submits, although PrefixSum records a batch, and the fast
field says every noncached dispatch allocates a uniform buffer despite
zero-uniform interfaces. Those architecture/allocation/performance claims need
the owner's final source and wording together. C2 is an additional concrete
stale rationale exposed by the native feature repair.

Remaining agreed #780 producer/count/cache/layout/parser/mode/caller families
and retained #774/F19 paths remain required integration work. This review does
not silently defer them or clear them.

## Validation limits and disposition

No source edits, project checks, builds, tests, scanners, generators, runtime
execution, push, CI requests or external posts occurred. No tests, cases,
helpers, assertions or doctests were added or changed. Read-only Git operations,
source reads and hashes establish the reviewed identity.

Pinned SDK evidence is local Cargo registry wgpu/wgpu-core 29.0.4 source matched
to Cargo.lock. Important SDK files are fingerprinted in the JSON report. No
browser, hardware-wide, performance or GPU-completion verdict is supplied.

Earlier recorder-scope acceptance assurances remain withdrawn. This review does
not reinstate them. The final integrated thirteen-lens inventory, independent
Code/Rule/Comment reviews and required hosted checks must all pass on the same
final pushed revision. This checkpoint grants no whole-PR or final clearance.

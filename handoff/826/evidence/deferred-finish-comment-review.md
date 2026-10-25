# Deferred finish/submission documentation discovery

Status: **pre-repair contract input; existing deferred-validation finding open**.
The agreed helper is not implemented at this snapshot. This report is not an
implementation, source-completeness or final-clearance verdict.

## Frozen source and scope accounting

- Independently resolved HEAD:
  `5a34b608ccacc1950313b501d30b036f0a4f8b5b`.
- Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.
- Tree: `572e4198d85822fd6f103a70e31223a02304aa45`.
- Working tree is clean and unchanged. Only this ignored report is authored.
- Applied Comment Review and the repository rules read during this continuous
  review. The authoritative erratum in `ranged-copy-comment-review.md` remains
  controlling; earlier blanket recorder-scope acceptance is withdrawn.
- The root's new full thirteen-lens discovery/inventory remains incomplete.
  This is focused independent source-backed discovery for that inventory.

Read/accounted source:

| Source | Contract or affected evidence |
| --- | --- |
| GPU/compute/math Cargo manifests; pinned wgpu Cargo features | Target features, default-feature disabling and scope-locality limits |
| `globals/wgpu_context/mod.rs` | Context policy, borrowed/native/browser constructors, separate completion API |
| `compute/pass/mod.rs` | Standalone record versus unscoped finish/submission |
| `kernel/mod.rs`, `copy.rs`, `dispatch_count.rs` | Batch append/count, copy host checks, finalization and callback boundary |
| `compute/readback.rs` | Raw copies, mapping registration, pending/failed/ready/taken states and existing fixtures |
| GPU buffer `mod.rs` and `readback.rs` | Separate mapping/completion, existing staging path and typed mapped-byte safety comments |
| `texture/srgb_converter.rs` | Earliest bind-group creation scope, finish scope and separate submission scope |
| GPU guide, existing README link and public texture fixture comments | Cause-retention, current native limits and examples |
| Pinned SDK sources listed below | Actual guard, futures, error sink, deferred encoding and completion semantics |

Current SHA-256 fingerprints of the principal affected documentation sources:

```text
64b53f0c6c350b1de37be00b9ce3de2fff5261912a7514322940b15ff6e2079b  crates/fabelgeist-gpu/src/globals/wgpu_context/mod.rs
0de7bed006752d346fa4cf6c7763cca9ccbaed89dbf4809ec56aa123836f5f27  crates/fabelgeist-gpu/src/data/gpu/compute/pass/mod.rs
08fc1088252b1f0337d747aa8288f930b222fa3905bfcb0e8f1c9447cc815fa5  crates/fabelgeist-compute/src/kernel/mod.rs
470fe0d4dbb1a0862d2683915413eab3c297accec32a95464f3323b7ef30d15f  crates/fabelgeist-compute/src/kernel/copy.rs
caaf8c9738e40e94a3dc17914bebeaeda33a6b37bacb74158619c9fca955af76  crates/fabelgeist-compute/src/readback.rs
3e73d10fdc2d644e8d0e3909147979a1c7023dfff0d82b12a7a88c751770fa02  crates/fabelgeist-gpu/src/data/gpu/buffer/mod.rs
339eb4bc4fecb2b3bf33e8b486b0a42214398ce4a391687745cce49464b267d9  crates/fabelgeist-gpu/src/data/gpu/texture/srgb_converter.rs
02b9c6fe5419780c4da5b6a10cfc7ace0c852866cc7c687336ee8a91a3190c25  crates/fabelgeist-gpu/texture-views.md
```

## Independently verified SDK 29.0.4 contracts

**Recording differs from encoding.** Core transfer `827-855` appends a copy
command; compute pass `472-507` appends a `RunComputePass`. Actual copy checks
run through command `encode_commands` at `command/mod.rs:1130-1140`, called by
encoder `finish` at `1232-1240`. Destroyed-resource errors can be deliberately
delayed to submission (`1252-1276`). Zero-size copies still undergo native
checks before their helper's zero return (`transfer.rs:962-1083`).

**Pop is immediate and guard drop loses captured evidence.** SDK
`api/device.rs:431-461,837-879` documents the stack, requires explicit `pop` to
retrieve errors, marks the guard `!Send`/`!Sync`, and says the pop takes effect
immediately. Dropping a guard pops its scope and discards captured errors.
Native Core `backend/wgpu_core.rs:1847-1884` removes the scope and returns an
already-ready future containing its captured error. Keeping the future alive
after popping does not keep the scope active for later encoder finalization.

**Capturing native API validation is not a GPU completion wait.** Core finish
reports errors through the current error sink (`2645-2661`); queue submit handles
its error similarly and can return an index if the handler returns
(`2086-2108`). Reading the Core pop result does not require inserting a GPU
completion `Wait`: its future is ready. Do not add a completion wait or claim
one is necessary merely to establish this host API error boundary. Later
device/runtime errors outside the active scope are not covered by that proof.

**Filters and scope locality are real limits.** `api/device.rs:764-775` has three
distinct filters: Validation, Internal and OutOfMemory. Core error sink
`626-687` routes an error to the innermost matching filter, retains only the
first error in that scope, otherwise invokes the configured uncaptured handler
or the default panic. The agreed `wgpu::Error` result type does not by itself
mean every native error class is captured. Frozen code uses Validation scopes;
final documentation must match the implemented filter policy.

With wgpu's `std` feature the Core scope key is a real thread ID; without it,
`backend/wgpu_core/thread_id.rs` uses a single stub key and the scope stack is
shared across threads. The direct GPU/compute/math dependency declarations
disable wgpu defaults and do not request `std`; wgpu's passed backend/language
feature lists do not themselves enable it. Other dependencies/build selections
can unify features. No feature-resolution command was run, so a globally fixed
thread-local scope claim is not established. The helper must respect the actual
stack/order/concurrency policy; the guard cannot be moved between threads.

**Browser WebGPU has a different future.** `backend/webgpu.rs:2515-2550` invokes
JavaScript `popErrorScope` and returns a JS Promise-backed future. Browser poll
is automatic/no-op at `2559-2562`; `poll(Wait)` cannot turn that event-loop
promise into native synchronous readback. The GPU constructors explicitly use
`BROWSER_WEBGPU` on Wasm and disable synchronous validation policy there.
Wasm features also compile a Core/WebGL option, so do not equate every possible
Wasm backend with that JS implementation. The agreed synchronous helper's Wasm
contract remains device-handled, not an observed native pass.

**Three callback categories must remain distinct.** SDK `api/queue.rs:257-276`
executes its own deferred command-buffer actions after the inner submit call,
even on returned errors. Application `KernelBatch::after_submit` closures are a
different queue. `Queue::on_submitted_work_done` is a GPU-execution completion
notification whose native callbacks require runtime progress
(`api/queue.rs:284-308`). A buffer mapping callback reports its mapping outcome.
Neither native callback is the application closure that initiates a mapping
after host submission returns. No general rollback or “no callback of any kind
can run on error” guarantee follows from gating application callbacks.

## Agreed boundary to document after implementation, not a current fact

The canonical new operation is exactly:

`WgpuContext::submit(&self, CommandEncoder) -> std::result::Result<SubmissionIndex, wgpu::Error>`.

There is no second `ComputePass::submit` API. Batch submission becomes fallible,
maps the original native cause at its existing error boundary, and runs its
application callbacks only on the helper's `Ok` branch. Standalone
`ComputePass::dispatch` delegates finalization/submission to this same owner.

Required implementation/documentation contract for review:

1. Scope the operation at the SDK boundary where it actually reports errors.
   Observe deferred finish failure before proceeding to submission; do not
   submit a known failed command buffer. Observe available queue-submission
   failure separately, including failures delayed until that call.
2. Return the actual captured `wgpu::Error` with its SDK source intact. Wrappers
   preserve that cause via their existing typed Native owner/standard conversion;
   a formatted string or a reconstructed validation-looking error is not proof.
3. State the selected filter/policy explicitly. Native captured-error `Err`
   consumes the encoder and is not an encoder-rollback result. `Ok` establishes
   only that the documented host/captured boundaries returned without a captured
   error. It does not establish GPU completion, later device health, performance
   or rendering correctness. Device-handled/browser `Ok` lacks a synchronous
   native validation verdict even though it permits the same application
   control-flow continuation.
4. Batch callbacks run once, in registration order, only after helper `Ok`.
   On helper `Err` they are dropped unrun. That is a host submission boundary;
   callbacks can initiate mapping, but are not GPU-completion callbacks.
   Native SDK-owned deferred actions are outside that application-only promise.

## Precise documentation corrections to carry into the implementation

### Context policy and canonical submit

`WgpuContext::blocking_validation` currently names only construction paths and
describes a blocking queue poll. Extend it to the verified finish/submission
boundaries without suggesting those boundaries require draining GPU execution.
Candidate wording after the agreed helper exists and is verified:

> Select synchronous native validation readback at the documented construction,
> encoder-finalization and submission boundaries. Owned native contexts enable
> it; borrowed contexts disable it. Browser builds use device error handling.
> This policy does not wait for GPU completion or report errors outside the
> active error scope. Shader source is separately validated by Naga.

If some existing constructors still explicitly poll, document those calls
separately rather than making a global timing/queue-draining claim. The submit
method's own docs must identify the actual captured filter and describe its
device-handled limit. Existing `submitted_work_done` remains a separate
completion operation; do not conflate it with the new helper or infer that a
completion wait retroactively recovers an earlier uncaptured validation cause.

### Batch recording, count, copy and submission

For `dispatch`, `dispatch_items`, `dispatch_folded` and `copy_buffer`, describe
their host admission and encoder append result. Their `Ok` is not a deferred
native encoding verdict. Zero-copy native checks still occur at finalization.
The existing count increments after a wrapper records successfully; it counts
recorded/appended dispatches, not observed successful GPU executions. Preserve
that distinction in `RecordedDispatchCount` and `dispatch_count` prose.

Replace the ambiguous “once the batch has been submitted” callback description
with the agreed branch explicitly:

> Run the closure after this batch's submission helper returns `Ok`, in
> registration order. Drop it unrun if the batch is discarded or submission
> returns `Err`. This is a host submission callback, not GPU completion.

Batch submit docs should say it consumes/finalizes the encoder, propagates
available captured errors, returns an index on `Ok`, and then runs those
application callbacks. On device-handled paths `Ok` still does not imply an
observed native pass. The returned index can be used by a separate completion
operation; it is not itself completion or a guarantee that later mapping works.
The raw-encoder accessor remains an append escape hatch whose finalization
belongs to the same batch submission owner.

### Guide and standalone dispatch

The guide's native-cause/callback paragraph must identify actual resource
creation, encoder finish and queue submission as separate observation points.
Do not imply recorder scopes capture all dispatch/copy failures merely because
they were opened and polled. Standalone dispatch should document propagation
from `WgpuContext::submit`; it should not grow a parallel finalization API.
Retain the source-cache `Ok` versus observed native acceptance distinction.

### Readback and pending-readback lifecycle

`compute/readback.rs:10-14` promises a per-frame read “without the CPU ever
stalling.” That is an unsupported global performance claim; polling, allocations,
driver calls and unrelated work do not establish it. Prefer:

> Pending readback registers mapping after host submission succeeds. Readiness
> checks poll without waiting for mapping completion and return `None` until the
> mapping callback reports readiness.

`record_pending` returns its handle before the batch is submitted. Its registered
application callback starts `map_async` only after batch helper `Ok`; the native
mapping callback later changes Pending to Ready or Failed. If submission fails
and callbacks are dropped, the current state machine has no cancellation state:
the handle remains Pending. Docs must require callers to propagate submission
failure and discard that pending handle, rather than claiming it automatically
receives a mapping or submission error. Do not invent a cancellation outcome
unless the owning implementation actually adds it.

`Readback::read` should require the producing batch's submission result to have
been handled successfully before mapping. It performs a separate map/completion
operation; a successful submit alone cannot supply results. Its existing async
signature is not a nonblocking-performance guarantee on native code, which
currently explicitly polls with Wait. No new completion wait is required in the
submission helper to make these docs true.

Existing readback fixtures submit before read/poll but ignore the currently
infallible submission return. Their owner must adapt those existing call sites
to propagate the new result, preserving inputs/assertions/oracles. The pending
fixture's pre-submit `None` remains valid; it is not evidence that a failed
submission eventually leaves Pending. No new test/case/assertion is requested.

### Converter and immediate creation evidence

The converter is an important counterexample to blanket withdrawal: its current
scope begins before bind-group creation (`srgb_converter.rs:119-137`), includes
`encoder.finish` (`157`), and is read back afterward. A separate scope covers
queue submission (`165-175`). Its earliest available creation/finish/submission
cause capture is at the actual boundaries. If delegating finalization to the
new context helper, preserve the earlier immediate creation scope and its
failure priority; moving every scope to after construction would lose that
evidence. Creation-cause claims must be independently traced rather than
discarded or globally promoted from recorder evidence.

## Dependencies and validation limits

The root must complete its new thirteen-lens discovery and shared ownership
accounting before implementation. This report does not authorize an independent
shared-function patch. #780 owns the full caller/fixture migration, including
readback/read-words and existing copy/submit consumers. Retained #774 paths,
remaining #780 producer/count/layout/parser/mode/folded/Reduce and stale Kernel
wording, F19 and final same-pushed-SHA hosted gates/reviews remain required.

No source edits, local project checks/build/tests/scanners/generators/runtime,
new tests/cases/helpers/assertions/doctests, push, CI request or external post
occurred. The new helper and its callback/error guarantees remain intended
contracts to verify after implementation. No GPU completion, timing,
performance, hardware/browser or whole-PR acceptance claim is made.

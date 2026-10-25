# Independent deferred-submission post-repair Code Review

No additional owned implementation defect was found in the 12-path receiving
repair. The R1 receiver now observes native Validation errors at encoder
finish and queue submit under the stated context policy. Its connected caller
and error/state families remain incomplete; this is not whole-PR,
compilation/source-complete or final clearance.

## Exact revision and independently checked source

- Reviewed HEAD: `f2b5ffb1320bc6d3610e8726ac694546ed1a465b`.
- Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.
- Receiving base: `5a34b608ccacc1950313b501d30b036f0a4f8b5b`.
- Clean Git working tree. Independently checked all 132 affected paths from
  the pre-repair inventory plus the new submission module against exact HEAD;
  every working file matches its committed bytes.
- Complete main diff: 77 paths; receiving delta: exactly 12 paths.
- SHA-256 of the binary main-to-HEAD diff:
  `316244cf323a98992153d32e1680a8b4c5b8781f77b7257dfd38dbf191d4bb80`.
- SHA-256 of sorted compact JSON changed-file path-to-SHA-256 map:
  `c2df614c5fbbe69ead6670f055280a627a2ee2593e149a3f4d4fcb8cec4be15f`.

Companion deferred-submission-post-code-review.json stores all 132 file hashes,
the 12 delta hashes, pinned SDK file hashes, the helper's source contracts and
current per-function disposition for all 104 prior function records covering
115 inventory locations. Old pre-repair fingerprints for the 11 edited
existing paths are expected to differ; the new module has no pre-repair hash.
All other pre-repair source identities are stable. No live owner source or
uncommitted owner edits were reviewed as part of this checkpoint.

Read the complete new helper and receiving delta, current Batch/pass error and
callback implementation, native and browser cfg/manifest traits, exact pinned
SDK finish/scope/queue implementation and materially affected consumers.
Revisited the unchanged caller, error-owner and retained-metadata source
proofs. Prior R1 source reasoning was reassessed against the repaired code;
its earlier verdict was not automatically carried forward.

## R1 receiver and exact SDK timing

WgpuContext::submit is the sole new submission-policy owner. It consumes one
CommandEncoder and returns std::result::Result<SubmissionIndex,wgpu::Error>.
There is no additional ComputePass submit wrapper or raw compatibility method.

At submission.rs:20-28 a Validation scope surrounds encoder.finish. Native
scope pop is read before attempting queue submission. If finish rejects an
ordinary invalid copy/pass, the original SDK error is returned and no command
buffer is queued. The zero-byte same-buffer scenario from R1 now reaches this
scope: it remains recorded, and native validation still checks same-buffer
before its zero-extent return.

At submission.rs:30-38 a separate Validation scope surrounds queue.submit.
That second scope is necessary even if finish succeeded: the pinned SDK delays
some destroyed-resource errors until submission and also validates mapped
resources and command-buffer device membership there. The helper preserves
the queue error instead of treating a returned native submission index as
proof of accepted work.

Independently inspected pinned wgpu/wgpu-core 29.0.4:

- core command/transfer.rs:827-855 queues ArcCommand; command/mod.rs:1130-1140
  encodes copies, and :1225-1241 invokes encoding at finish.
- command/transfer.rs:970-1083 validates native copy requirements before the
  zero-byte return. No new local alignment/device/usage validator was added
  in place of native authority.
- command/mod.rs:1265-1273 intentionally withholds destroyed-resource finish
  errors until submit.
- backend/wgpu_core.rs:2645-2655 delivers actual finish errors; :2086-2107
  synchronously reports queue-submit errors through the queue's sink.
- core device/queue.rs:1189 onward processes and validates the submitted
  command buffer. validate_command_buffer at :1822-1852 checks queue/device,
  destroyed buffers and still-mapped buffers. QueueSubmitError's Validation
  classifications are explicit at :502-511.
- backend/wgpu_core.rs:650-676 handles errors through scopes present on the
  handling thread; :1847-1889 removes the scope and returns a ready future
  with its saved error. A GPU completion poll is not required for these
  synchronous native report paths.

The helper adds no device.poll, completion callback, sleep or queue drain.
Existing unrelated waits in recorders/readbacks are unchanged; this receipt
does not claim that every earlier recording API is nonblocking. It does not
claim that a later device-health failure is reported by an earlier Ok index.

## Context provenance, filters, native threads and browser behavior

This is a Validation-filter contract. OutOfMemory/Internal and any later
asynchronous device-health errors follow configured device handling. They do
not acquire a synthetic validation outcome or a fabricated cause. The guide
and helper documentation state that limit.

For native builds, blocking_validation controls both scopes. Native owned
contexts enable it, borrowed contexts disable it as before. The explicit wgpu
std feature is important: pinned backend/wgpu_core/thread_id.rs chooses actual
std::thread::ThreadId only under cfg(feature = "std"); its other branch is a
single stub identifier. Scope push/pop/error handling now distinguish native
threads even when other dependencies do not enable that feature. No cross-
thread guard migration is added: the synchronous helper pushes, handles and
pops on its current thread, without suspension onto a task executor.

The SDK's Error implements std/core Error and retains its own lower-level
source; native Error is Send+Sync. The existing browser feature policy enables
fragile-send-sync-non-atomic-wasm; it is unchanged. On browser builds the native
scope and pollster code is removed by cfg, and direct finish/queue submission
returns Ok without a synchronous native-validation verdict. The helper does
not add native-only imports or polling to that path. This is source/API trait
reasoning, not evidence of a successful native or browser compile.

Provenance remains an explicit caller precondition. The first helper doc line
requires an encoder created by this context's device. The queue must also be
paired with that exact device for its scope sink to be authoritative. All
migrated internal consumers create the encoder from their supplied context's
device. Public context fields/from_parts and arbitrary external encoders are
not runtime-proven pairs by this helper. Same native provider alone is weaker
than exact device/queue identity: a foreign device's encoder or queue can
report through a different error sink, outside self.device's scopes. No actual
mismatched internal pairing was found. This review does not grant the helper
a stronger provenance guarantee than its receiving precondition.

## Canonical causes and callbacks

KernelBatch::submit at kernel/mod.rs:358-368 consumes its encoder through the
context helper. It wraps the original error once in
ComputePassError::Native(Arc<wgpu::Error>) and propagates with `?` before the
callback loop. ComputePassError::source returns the actual wgpu error, whose
source continues to the SDK cause. There is no formatted replacement or new
error type owner. The unconditional Arc import already serves Kernel/KernelCache
and remains available on browser builds too.

On success the original registration-order FnOnce loop runs once, followed by
Ok(index). On failure early return drops the unrun callbacks and remaining
batch resources. No callback runs merely because encoding was attempted.
These are host submission callbacks, not completion notifications. No extra
submit, eager kernel preparation or wait was introduced.

ComputePass::dispatch constructs the encoder from the supplied context,
records the existing pass, then consumes the same helper and preserves the
same canonical error cause. Recording scopes remain useful for immediate
reported failures, but the repaired submit now observes deferred errors; a
record Result alone still does not establish native encoding acceptance.
The existing empty-first Batch admission, folded launches, sparse layouts,
static/fast uniform packers and dispatch count ordering are unchanged by this
12-path repair.

## Ten context-backed texture consumers

Each converted method retains one encoder constructed from its supplied
context, passes that encoder through the helper, and maps the original error
to the existing ComputePassError::Native before its next continuation:

| Receiver | Current helper call lines | Continuation gated by Result |
| --- | --- | --- |
| Texture2d::read | texture_2d/mod.rs:319 | Staging map/read |
| Texture2d::blit | texture_2d/mod.rs:502 | Successful unit return |
| Texture2d::clear_raw | texture_2d/mod.rs:591 | Successful unit return |
| Texture3d::read | texture_3d/mod.rs:181 | Staging map/read |
| TextureCube::set_side_texture | texture_cube/mod.rs:375 | Copy fast-branch return |
| TextureCube::blit_texture_to_face | texture_cube/mod.rs:549 | Successful unit return |
| TextureCube::render_face_shader_raw | texture_cube/mod.rs:728 | Successful unit return |
| TextureCube::render_shader_raw | texture_cube/mod.rs:894 | Successful unit return |
| TextureView::clear_raw | texture_view/mod.rs:181 | Successful unit return |
| TextureView::read | texture_view/mod.rs:242 | Staging map/read |

Missing/uninitialized cache admission, selected-view effective format versus
logical/native allocation, cube face/array geometry, texture/device admission,
readback row layout, map completion waits and clear values retain their exact
previous implementation. Wrapper methods still propagate their recipient
Result before returning a successful target/image. No stored-format owner or
parallel view cache was introduced.

This repair observes finalization/submission. It does not retroactively scope
an earlier unscoped native resource, shader or render-pipeline creation, nor a
queue write that happened before entering the helper. No whole native-render
construction policy is cleared by the table. TextureSrgbConverter is unchanged
and already scopes its finish and submission separately, including its earlier
bind-group recording boundary; that correct receiving contract was preserved.

## Still-required connected work and source limits

The receiver implementation is repaired, but whole R1 cannot clear while
unchanged callers ignore its new Result. Current sort, PrefixSum, stereo,
physics/XPBD/shell, hair, armor and creator callers still contain their old
submit expressions. Existing fixture callers still need the agreed API-only
adaptation. They must stop before status/geometry reads, dependent submissions,
clock/counter publication and polling when submit fails. No source-compilation
claim is made; known ranged-copy signature dependencies also remain open.

The #780-owned Reduce manual ComputePass::record loop still finishes/submits
its encoder directly at reduce/mod.rs:344. It remains a required consumer of
the canonical helper, including its final copy. The unchanged Buffer readback,
DistanceField texture-copy precursors, raw renderer/Bevy queues and empty queue
marker retain their explicit separate classification. No blanket native-error
policy approval or permission to duplicate their owners is implied.

E1 is unchanged: GenerateError::Gpu stores Arc<str>, device_error stringifies,
and metal bake returns String. They need the agreed retained #774 canonical
error handoff to preserve this newly available native cause. Hair's existing
Backend(anyhow::Error) can preserve it. A new typed cause at the receiver does
not erase a later stringifying boundary.

M1 is unchanged: hair reset/shape/time/h_last/collision selectors and BVH/hash
recorded metadata can change before actual finish. This review does not invent
a transactional rollback promise, but a safe recorded/submitted reuse or
discard contract remains required for newly observable submission failure.
The owner's canonical recorded-layout/count migration must be consumed rather
than independently recreated. Ordinary source/pipeline construction cache
success is separate from external execution success and must not be
indiscriminately invalidated on a bad dispatch.

Readback::record_pending still registers map_async only in after_submit. It
receives the corrected callback gating, but a PendingReadback obtained before
failed submission has no mapping outcome and must be discarded by its caller.
The guide now documents this limit. A caller that ignores Err and polls the
pending object is not repaired by callback suppression alone.

All #780 producer/cache/recipe/layout/stride/count/mode/parser/Reduce/folded/
ranged-copy caller families, retained #774/F19 hair/body/armor/error paths and
unresolved raw-renderer policies remain required coordinated work. No owner
source was edited or independently repaired by this reviewer. These are not
accepted permanent deferrals.

No local project execution, checks, builds, tests, scanners, generators or
runtime occurred. No tests/cases/helpers/assertions/doctests, source edits,
push or CI were added or run. Only the ignored JSON/Markdown review artifacts
were written. Final all13 lenses and all three independent reviews must
repeat on the same final pushed SHA, with all required hosted checks passing
there. This local checkpoint supplies no final approval.

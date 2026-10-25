# Ranged-copy checkpoint Comment Review

**Authoritative erratum: reopened at the same exact revision.** The original
no-owned-finding disposition and recorder-scope native-validation/cause
assurance below are withdrawn. See the appended erratum for the independently
verified deferred-validation trace and actionable finding. The original receipt
is preserved as historical review evidence, not a current clearance.

Status: **no new owned comment mismatch; caller integration and final validation
remain incomplete**. This is not source-complete or final clearance.

## Exact independently verified source

- HEAD: `5a34b608ccacc1950313b501d30b036f0a4f8b5b`.
- Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.
- Commit tree: `572e4198d85822fd6f103a70e31223a02304aa45`.
- Working tree was clean on entry and source remained frozen during review.
- Independently compared the full delta against the previous cache-integrated
  checkpoint `7dee3d5a69fa6e5132efa740d8b0f23a42cb435f`: exactly three paths
  differ, adding the focused copy module, updating its error payload, and
  exporting/removing the old implementation in `kernel/mod.rs`. All other
  tracked source, documentation, fixture and workflow blobs are identical.
- Applied the Comment Review skill and governing repository rules already read
  during this continuous review sequence.

Current SHA-256 fingerprints:

```text
470fe0d4dbb1a0862d2683915413eab3c297accec32a95464f3323b7ef30d15f  crates/fabelgeist-compute/src/kernel/copy.rs
17d100ac7cc26a65191ed6c6fb7aaa7756f48cda00d2fc59cff3d831089bc6d9  crates/fabelgeist-compute/src/kernel/batch_error.rs
08fc1088252b1f0337d747aa8288f930b222fa3905bfcb0e8f1c9447cc815fa5  crates/fabelgeist-compute/src/kernel/mod.rs
```

## Copy request and receiver contract

Read all new module/type/method comments with the implementation. `BufferCopy`
accurately describes requested source/destination ranges; its public fields
carry the canonical `BufferByteOffset` and `BufferByteLength` leaves. Constructing
the record does not establish that it fits particular buffers, and the comment
correctly places bounds admission at recording time.

Both logical end calculations use `checked_after`; overflow and either logical
extent overrun return `KernelBatchError::CopyOutOfBounds` before invoking the
native encoder. Its diagnostic now retains the complete request and both
logical extents. It does not masquerade as a native SDK cause.

The method's separate native-requirements statement is important and accurate.
The host logical checks do not prove device identity, native allocation bounds,
copy usage, alignment or distinct source/destination resources. Native recording
still runs inside the context's existing native validation-scope policy and
retains an available `wgpu::Error` through `ComputePassError::Native`. Borrowed
and browser paths still provide no synchronous native acceptance verdict.

There is no host zero-byte early return. The comment explicitly includes zero
copies in native validation policy. Verified this against installed pinned
`wgpu-core` 29.0.4, `src/command/transfer.rs:962-1083`: same-buffer/device/usage,
offset/alignment and native range checks precede the `size == 0` return. Thus
the new request API does not document a zero-size exemption that the SDK lacks.

“Logical bounds are checked before recording” refers to those host range checks;
it does not promise successful native recording or encoder rollback. No comment
claims a native validation error leaves the batch encoder usable. The unchanged
native failure path must not be interpreted as transactional rollback.

The ordering comment matches recording the copy in the batch's existing encoder
between surrounding operations. The only copy receiver is now in `kernel/copy.rs`;
the old signature/body and attached comment are removed rather than retained as
a forwarding API. The new module owns a focused request/recording responsibility.

## Affected consumers and current documentation map

Read the existing sort, contact-recovery and StrandSet settling call sites and
their nearby comments. Their intended whole-prefix copy behavior remains clear,
but they still use the old final argument form and require the accepted #780
named-request migration. No compilation or completed-family conclusion follows
from reviewing the receiver alone. Raw readback/read-words producers and all
copy callers/fixtures remain that owner's explicit integration work.

Re-read the complete current GPU guide against the retained source map. It still
describes provided-value uniform packing rather than the withdrawn expected-shape
protocol: whole-block bounds, ordered overlapping writes, initial zero plus
skips, direct nonstruct zero blocks, reflected Mat3 span/3, typed formerly
panicking component windows and concrete host-narrowing causes. The copy change
does not alter those exact source blobs or assert a new packing contract.

Its selected texture-view/cache/format/device and cube/face/array descriptions,
admitted shared static assembly, logical buffer prefixes, native cause chains,
immutable shader/reflection provenance, used-resource/sampling union and used
immediate rejection remain consistent with the unchanged core implementation.
Native pipeline/source caches still distinguish reported validation policy from
construction `Ok`; a browser/nonblocking constructor result is not an observed
native pass. Empty dispatch/callback ordering and converter/native scope limits
remain documented accurately.

The changed/moved comment map includes the three-file request/receiver/error
delta and the retained primitive/cache, GPU packing/reflection/texture/context,
mesh/BVH/hair/physics and fixture/workflow graph. Their identities outside the
delta were established by committed comparison. No current guide snippet uses
the removed copy signature. Historical/software/hardware/browser/performance
limits and final hosted-check requirements remain explicit; no successful run
is inferred from source or workflow files.

## Required dependencies and limits

No new owned wording remedy is requested. The accepted #780 caller/fixture and
raw-readback/read-words migration is still required, together with the remaining
Kernel/module/fast-field/producer/count/layout/parser/mode/folded-caller/Reduce
work. The previous KernelCache exactly-once claim remains resolved, but the
retained Kernel module/fast-field stale claims are not cleared here. Retained
#774 paths, F19 and final same-pushed-SHA reviews/hosted gates remain required.

No source edit, project check, build, test, scanner, generator, runtime,
new test/case/helper/assertion/doctest, push, CI request or external post
occurred. Only Git/source/pinned-SDK reads, requested fingerprints and this
ignored report were used. This receiver checkpoint is incomplete as a connected
migration family and cannot establish whole-PR clearance.

## Authoritative erratum: native validation occurs after recorder scopes close

Current disposition: **actionable owned/shared finalization-contract finding;
review incomplete**. Exact source is still
`5a34b608ccacc1950313b501d30b036f0a4f8b5b`, tree
`572e4198d85822fd6f103a70e31223a02304aa45`, against pinned main
`990430b77a9f63548ffcb642074185f61ddb3322`. All three fingerprints above were
re-read and remain unchanged; working source remains frozen and clean.

### Corrected pinned-SDK trace

Independently read the entire call chain in installed wgpu/wgpu-core 29.0.4:

1. `wgpu-core/src/command/transfer.rs:827-855` resolves buffers and appends
   `ArcCommand::CopyBufferToBuffer`. It does not execute the usage, device,
   alignment and range checks from the later encoding helper.
2. `wgpu-core/src/command/mod.rs:1130-1140` invokes that helper while
   `encode_commands` consumes recorded commands.
3. The same file's `finish` calls `encode_commands` at lines 1232-1240 and
   creates an error-state command buffer on failure. Destroyed-resource errors
   can be delayed further until submission (`1252-1276`).
4. `wgpu-core/src/command/compute.rs:472-507` similarly appends
   `ArcCommand::RunComputePass`; its own comment says most encoding errors are
   detected in `finish`. Closing a compute pass does not establish acceptance.
5. `wgpu/src/backend/wgpu_core.rs:1847-1884` pops/removes the scope and returns
   its current error. The later native `finish` routes its error through the
   then-current error sink (`2645-2661`). A scope popped while recording cannot
   capture a failure generated only by subsequent finalization.
6. The backend's queue submission handles a failure through the device error
   sink and still returns the supplied index if that handler returns
   (`2086-2108`). A returned index alone does not prove accepted GPU execution.

The native zero-size ordering previously read at `transfer.rs:962-1083` is
correct **inside this deferred helper**: usage/device/alignment/range checks
precede zero's return. It does not show that those checks run inside
`KernelBatch::copy_buffer` or its returned `Result`.

### R1: Recorder and guide claims need the real finalization boundary

Source locations:

- `kernel/copy.rs:21-23,47-69`: native context-policy wording and scope popped
  before this recorder returns.
- `kernel/mod.rs:265-291`: batched dispatch's recording scope is also popped
  before encoder finalization.
- `compute/pass/mod.rs:45-69`: standalone recording closes its scope before
  the caller's later `finish`.
- `kernel/mod.rs:345-359`: batch finish/submission is unscoped; callbacks run
  after `queue.submit` returns.
- `compute/pass/mod.rs:28-31`: standalone dispatch finishes/submits unscoped.
- `texture-views.md:127-133`: broad cause-retention and callback/native-verdict
  paragraph needs to describe the verified finalization/submission boundary.

The exact checkpoint can return `Ok` from the copy recorder on a logically
fitting request that will fail native device/usage/alignment validation when
the encoder is later finished. Polling during the recording scope does not
encode that future command. Blocking context policy therefore does not
currently capture those deferred native errors into this recorder's typed
result. The same gap affects deferred compute-pass recording checks.

Consequently, the original report's claim that native validation/cause retention
was established by the recorder scope is withdrawn. Any equivalent blanket
assurance in preceding checkpoint receipts is also withdrawn: it cannot be used
to clear encoder recording/finalization. This does not withdraw independently
verified immediate shader/pipeline/view/bind-group creation causes or concrete
host admission errors; each operation must be traced at its actual SDK boundary.

The copy method's accurate **current-source** wording would be:

> Check logical byte-range ends and append the copy to this batch's encoder.
> `Ok` reports host admission and recording, not native acceptance. Native copy
> requirements, including zero-size validation, are checked when the enclosing
> encoder is finished. This recording scope does not observe those deferred
> failures or later queue-submission failures.

That wording exposes the current gap; it is not sufficient to complete the
user's native-cause contract. Extend the connected ledger/ownership agreement
and repair the actual finish/submission receivers and their callers before
final clearance. `KernelBatch::submit`, standalone `ComputePass::dispatch` and
all dependent callers are part of that connected repair. No independent shared
function edit was made by this reviewer.

### Precise intended documentation after the repair is verified

Document three separate stages, matching the final implementation rather than
promising an unverified native pass:

- Recorders establish their explicit host admission checks and append commands;
  they do not report finalization/submission acceptance.
- The receiver that owns `encoder.finish()` must document and actually observe
  available deferred encoding errors under the selected context policy. The
  submission receiver must separately handle available submission errors,
  including errors deliberately delayed to that boundary. Its typed success
  means the documented host/observed boundaries reported no captured error,
  not GPU completion. Borrowed/browser device-handled outcomes must be stated
  explicitly as lacking a synchronous native verdict.
- Callback documentation must name the chosen boundary. A concrete consistent
  target is: callbacks run once in registration order only after queue submit
  returns and any enabled finalization/submission readback reports no captured
  error; captured failure before that point drops them unrun. On a
  device-handled path this establishes only a submission attempt, not native
  acceptance or completion. Adoption and implementation of this policy are part
  of the shared repair, not a fact about the present checkpoint.

For the **present** implementation, the callback truth is narrower: callbacks
run after the `queue.submit` call returns, even if an uncaptured error handler
consumes a failure and lets it return. There is no typed finalization/submission
outcome gating them. “Once the batch has been submitted” must not be read as
“after validated successful GPU submission.” Waiting for returned work is a
separate completion boundary and cannot retroactively capture the error cause.

The guide should summarize exactly the final verified stage/policy distinctions
and retain its existing hardware/browser/performance limits. It must not replace
this missing path with a permanent blanket “available native causes” claim or
silently defer finalization as out of scope.

### Reopened coverage and limits

Logical copy bounds, canonical range leaves, ordered encoder append and the
absence of a zero-byte shortcut remain correctly described. No native error
rollback or encoder usability guarantee is established. Caller/fixture/raw
readback migrations, remaining #780 families, retained #774 paths, F19 and final
same-pushed-SHA reviews/hosted gates remain required.

Only source/Git/pinned-SDK reads and this ignored report correction occurred.
No source edit, local check/build/test/scanner/generator/runtime, new
test/case/helper/assertion/doctest, push, CI request or external post occurred.
The original receipt is preserved above specifically to make the mistaken
validation inference reviewable. This erratum supersedes it for clearance.

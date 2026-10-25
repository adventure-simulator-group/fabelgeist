# Independent post-repair submission Rule Review

No additional owned rule defect beyond the known C1/C2 documentation corrections
was identified in the implemented R1 receiver increment at
`f2b5ffb1320bc6d3610e8726ac694546ed1a465b`. The canonical context
operation now observes Validation failures at actual encoder finish and queue
submission boundaries. Batch and standalone dispatch preserve the actual cause;
batch callbacks run only after Ok. The complete family remains incomplete:
E1/M1, #780 caller integration, retained #774, C1/C2 and hosted gates remain
required. Bare-renderer device-handled policy is now explicitly agreed, with
the documented observation limit. This is a checkpoint review, not compilation or
whole-PR/final clearance.

## Source identities and review limits

- Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.
- Previous discovery/repair checkpoint:
  `5a34b608ccacc1950313b501d30b036f0a4f8b5b`.
- The exact increment changes 12 paths; 77 paths differ from main. All 77
  working contents matched HEAD blobs, with a clean tree. The other 65
  main-diff files retain their previous source identities.
- Current broader source accounting contains 132 paths: the final 131-path
  pre-repair inventory plus the new context submission module. The companion
  JSON retains current fingerprints for every one, plus all changed-file hashes.
- Main-diff fingerprint digest:
  `a6238669bae8c9735567644cb0795d6a53cdf655a100d0eff651e02568e8d4e5`.
- Broader frontier fingerprint digest:
  `0de664f09e192f08f8fed95e00ce16d54873d860e942eac2812530103f53defc`.

Read the actual 12-path committed change and complete new/changed submission
responsibilities, current texture native ports, native SDK scope implementation,
Cargo features and source/baseline rules. Prior source receipts account for the
unchanged bodies and caller graph; exact identities and current receiving
connections were independently revalidated. This does not pretend every
unchanged architectural body was freshly reread in this checkpoint.

The preceding discovery MD/JSON were first corrected explicitly: native
`cfg(std)` does not establish Cargo `feature = "std"` thread identity. Their
unconditional thread-local statement is withdrawn. The new feature-specific
proof below supersedes it; the earlier deferred-command erratum remains valid
for frozen 5a34 and is not rewritten into a passing historical receipt.

Only ignored review artifacts were written. No source edits, project checks,
builds, tests, cases, test helpers, assertions, doctests, project scanners,
generators, runtime, push, hosted CI or external posts were performed. The
increment changes no test paths or test bodies and adds no test/assertion or
production unwrap/expect markers. This is source evidence, not executed gates.

## Canonical owner and actual native outcome

`globals/wgpu_context/submission.rs:3-40` is the sole new submission-policy
owner. WgpuContext::submit consumes one native CommandEncoder and returns
`std::result::Result<wgpu::SubmissionIndex, wgpu::Error>` directly. Its explicit
standard Result preserves the distinct SDK error rather than widening to the
context module's other anyhow operations. No bespoke submission-error clone,
new device token or equivalent backend was introduced.

The verified sequence is:

    encoder created by the context's actual device
      -> optional native Validation scope -> encoder.finish
      -> pop/return original native cause before any queue submission
      -> separate optional Validation scope -> queue.submit
      -> pop/return original native cause -> Ok native submission index

At lines 20-28 the finish cause is returned before entering the submission
phase. A rejected encoder is not submitted as an apparent success. At lines
30-39 the queue phase has its own scope and cause return. Neither phase waits
for GPU completion or fabricates a completion/index result on failure.

Installed SDK 29.0.4 `backend/wgpu_core.rs:1832-1884` supplies the actual native
scope implementation: pop removes the current scope and returns
`ready(scope.error)`. Therefore these two synchronous pollster::block_on calls
read the actual current native result without needing device.poll or a queue
completion wait. The existing core deferred copy/pass checks execute inside
finish, now within the correct scope. Scope guards are created and popped on
one synchronous thread and are not retained in a batch or callback.

This proves the selected Validation observation contract. Internal/OOM/other
device outcomes remain governed by the provider's configured error handling,
as documented. It does not prove later device health or GPU execution success.
Borrowed contexts with blocking_validation false and browser builds install no
such synchronous scopes. Their Ok is device-handled submission, not an observed
native validation verdict. Documentation preserves these distinctions.

Native encoder provenance remains an explicit precondition, not a new checked
identity guarantee. A foreign encoder's finish reports to its originating
device sink. The intended relationship is the **same actual native Device and
its paired Queue**, not merely a common backend/adapter/provider. All newly
migrated first-party callers create the encoder from their supplied context.
Known C1 requires tightening the helper/guide's "same native provider" wording
to that precise relationship; the current signature cannot prove arbitrary
external encoder membership.

## Feature and cause trait proof

The native GPU wgpu dependency now explicitly enables `std`
(`fabelgeist-gpu/Cargo.toml:30`). This selects the real std::thread::ThreadId
implementation in `backend/wgpu_core/thread_id.rs:7-16`. The alternative at
lines 18-29 returns the same unit key for every thread. `build.rs:51-62`'s
`cfg(std)` alias can be true on native without Cargo feature std, so the explicit
feature is necessary; unrelated feature-unifying dependencies are not the
library's authority for this behavior.

The dependency's std feature forwards to `wgpu-core?/std` and related SDK
dependencies (`wgpu/Cargo.toml:89-97`). Cargo.lock is byte-identical to 5a34 and
main; it already contains the relevant packages and records no per-consumer
feature list. That unchanged lock is not evidence of an executed resolution or
compilation, and no lock generator was run.

`wgpu/api/device.rs:774-825` keeps ErrorSource and Error Send/Sync conditional on
cfg(send_sync). Native satisfies it; the existing GPU Wasm manifest enables
fragile-send-sync-non-atomic-wasm. The installed SDK build condition does not
establish browser atomics support. Native pollster code is excluded on Wasm,
where no new synchronous wait was introduced. No new unsafe Send/Sync impl,
formatted/reparsed error or trait-bound workaround was added. Actual native and
Wasm compilation remain unverified.

## Complete changed receiver trace

| Source responsibility | Retained contract and actual receiving boundary |
| --- | --- |
| KernelBatch::submit `kernel/mod.rs:354-367` | Existing owning anyhow Result carries SubmissionIndex or canonical ComputePassError::Native(Arc<wgpu::Error>). The helper result propagates before the callback loop; a finish/submission Err consumes the batch and drops callbacks unrun. Registration order is unchanged on Ok. No fallible result is discarded here. |
| Standalone ComputePass::dispatch `compute/pass/mod.rs:18-35` | Creates its encoder from context.device, admits recording, delegates once to context.submit and maps the same canonical Native cause. No ComputePass::submit forwarding helper or duplicate finish path remains. |
| Recording and copy docs | `kernel/copy.rs:21-25`, KernelBatch::dispatch and ComputePass::record now explicitly allow deferred native errors until submission. Typed logical bounds, metadata, recorded dispatch count and supplied-value packing are unchanged. |
| Texture2d read/blit/clear_raw | Three exact delegated ports preserve absence/device precedence, source/target selected view handling, original encoder commands and native context provenance. Submit errors propagate before readback mapping or Ok. |
| Texture3d read | One exact delegated port preserves the three-dimensional copy/stride path and returns submission failure before initiating map_async. |
| TextureCube set_side_texture/blit_texture_to_face/render_face_shader_raw/render_shader_raw | Four exact ports retain fast-copy versus render branching, face/layer/dimension and selected-view contracts; the same encoder/context reaches the helper, with its native cause returned. No view/cache format authority was replaced. |
| TextureView clear_raw/read | Two exact ports retain depth/color behavior, layer selection, missing/uninitialized and device precedence, and mapping after accepted submission. |
| ComputePassError and guide | The same native payload/source owner now explicitly covers finalization/submission. The owning GPU guide distinguishes record admission, Validation observation, callback initiation, mapping and GPU completion. |

The ten texture delegations preserve their existing public signatures and use
the already canonical Native wrapper at the actual GPU port. Repeated mapping
here does not introduce an internal primitive conversion cycle or an equivalent
error type. Source search found only the one context method and the meaningful
fallible batch facade; no proposed forbidden ComputePass helper was added.

The guide explicitly requires discarding a PendingReadback obtained before a
failed submission, since the dropped callback supplies no mapping outcome.
That is the receiver's documented failure contract. Correct caller consumption
and M1 retained-state handling remain required dependencies; this documentation
does not by itself repair the caller graph or prove atomic state rollback.

## Ownership, maintainability and no-growth evidence

The source delta is confined to the explicitly agreed root receiver/module,
native std declaration, docs/error docs and ten texture sites. Exact #780
agreement names those texture functions and retains all Batch caller repairs,
Reduce/DistanceField and metadata work with #780. Buffer::read and retained
#774 armor/metal/fitting/hair whole functions are unchanged. No unrelated ZIP/
NumPy or sorting implementation was repaired by this reviewer or this delta.

WgpuContext orders imports/re-exports before module declarations. The new
41-line submission module has one cohesive native policy and inherent receiver;
it is not a pass-through module. The batch facade adds real canonical-error and
callback policy. The helper's parameter is the actual SDK encoder, not a domain
record hidden in positional primitives. No magic application role/string,
extra generic abstraction, unnecessary trait boilerplate or compatibility API
was introduced. Existing derives/byte/device owners remain unchanged.

Source arithmetic was compared directly to the checker's actual meaningful-line
definition (`scan.rs:425-450`: nonblank and not starting //), with existing test
module ranges excluded. This is not execution of the checker or an independent
general-purpose source scanner:

| Responsibility | Current source count | Recorded limit |
| --- | ---: | ---: |
| Texture2d production module | 527 | 538 baseline |
| TextureCube production module | 807 | 899 baseline |
| Cube blit_texture_to_face | 157 | 161 baseline |
| Cube render_face_shader_raw | 149 | 149 baseline |
| Cube render_shader_raw | 142 | 142 baseline |
| Texture2d/3d/View read methods | 91 / 95 / 97 | 100 each |

The exact cube function ranges are 382-552, 568-731 and 742-897. The new helper,
batch submit and standalone submit path are below the ordinary function/module
limits. Existing large modules gained boundary delegation, not a second new
responsibility. No baseline ceiling, registry exemption or lint configuration
changed. Required relevant baseline reductions remain pending hosted work;
source counts do not establish all format/lint/baseline gates.

## Required unresolved paths and acceptance limits

Known C1/C2 are still outstanding in this frozen source. C1 is the exact native
Device/paired-Queue wording above. C2 removes only the obsolete interleaving
rationale by static COMPILING and Kernel::new; with explicit native std, SDK
scope keys now distinguish threads. #780 expressly authorized those comment
deletions while preserving the mutex and constructor/launch implementation.
These are documentation corrections under root's behavior/documentation rule;
they are upcoming source changes and cannot be treated as already repaired or
covered by this exact-SHA verdict. Their successor needs a fresh audit wave.

The 90 batch submission call sites from discovery are not migrated in this
increment. Existing old copy callers also remain required. An unconsumed Result
or a text-only mapping is not a completed fallible API migration. E1 still
includes ArmorGpu::device_error/GenerateError::Gpu Arc<str>, metal String errors
and retained source-erasure/cache recipients. M1 still includes CPU publication,
history/counter/cache mutations and interleaved continuation around failed GPU
submissions. Adding a leaf `?` does not automatically prove those relationships.

Retained #774 complete armor/metal/Buffer::read/fitting/hair/error families and
#780 canonical count/layout/parser/mode/Reduce/source integrations must be
reconciled through their owners. The latest explicit #780 agreement resolves
HairRenderer/Bevy bare-provider policy as intentionally device-handled, with
owning documentation and all-thirteen delta accounting assigned to #780. No
fake context or second helper is appropriate. This is an explicit observation
limit, not a remaining policy ambiguity or a synchronous-cause guarantee. Their
other arithmetic/retained-state findings are not cleared by that agreement.
The distinct empty queue completion marker is not an encoder operation. No
required caller/error/state integration is accepted as a silent deferral.

The SDK source supports the reviewed native mechanism; no actual provider run
was performed. Historical software-GPU/browser/hardware/performance/rendering
limits remain. Hosted format/lint/native/Wasm/project checks, relevant baseline
reductions, complete final audit waves and all three independent reviews on the
same final pushed SHA remain required. No compilation, push or whole-PR/final
clearance is granted.

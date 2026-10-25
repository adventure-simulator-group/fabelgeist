# Five-scope receiving checkpoint Comment Review

Reviewed `48e5995839d008ef3373343919d17915357544bc`, tree
`532b456ee225747cbaa22ed05da8643147df82fd`, against
`2e2cda74b681620d3162f30ce26dd8d946b059de`. Main remains
`990430b77a9f63548ffcb642074185f61ddb3322`.

Two focused documentation findings remain in the received Kernel module.
All seven changed files were read completely; the JSON accounts complete and
bounded caller, error, scope, grid, shape, arena and cache source reads.

## SCOPE48-C1 [P3]: False universal submission claim

`crates/fabelgeist-compute/src/kernel/mod.rs:12-16` says every other primitive
calls ComputePass::dispatch and that hundreds of submissions lose a frame.
Current Reduce instead records all reduction passes into one encoder through
ComputePass::record and submits once (`reduce/mod.rs:266-343`). The performance
outcome is also stated universally without measured evidence.

Suggested replacement: “A standalone ComputePass::dispatch finishes and submits
its encoder. A solver can instead record many dependent dispatches into
KernelBatch and submit them together.”

## SCOPE48-C2 [P3]: Overstated static uniform allocation

`crates/fabelgeist-compute/src/kernel/mod.rs:65-69` says every dispatch without
the cached path builds a uniform buffer, with a universal affordability claim.
The static uniform owner returns None for zero uniform extent
(`fabelgeist-gpu/src/data/gpu/bind_group/uniform_buffer.rs:18-20`). The existing
buffer-only DOUBLE fixture is a concrete interface requiring no such uniform.

Suggested replacement: “The cached path uses per-batch uniform slots for
eligible interfaces and creates a fresh bind group per dispatch. See fast.”

Both are owning Kernel comments received from the coordinated peer source.
Root can coordinate the precise canonical wording. This reviewer edited no
source.

## Contracts retained

The scope/error documentation matches the new implementation. Native observation
attempts the existing wait then pops even after polling failure, retaining real
progress and captured native causes. Fast recording drains observation before
resolving an earlier operation; the primary operation and secondary observation
remain accessible. Counts advance after successful recording outcomes, not proof
of native finish acceptance. Empty batch grids skip admission and counts.

Arena preparation can consume a slot before a later error. Recording can mutate
the caller's encoder before an error returns. No rollback is promised. Fresh
bind-group identity remains existing policy; it is not documented as a proven
barrier mechanism. Native synchronization belongs to wgpu.

Cache keys retain source, logical Device and validation policy. Constructor
Result success on a borrowed/browser path is not native-observed success, and
concurrent builders can race. Static pipeline outcomes distinguish completed
native rejection from retryable Progress. No blanket compiled-state reset is
inferred. Finalization/submission retain their separate no-completion-wait
helper; callbacks run only after helper Ok under the documented policy.

Existing dispatch fixtures were compared to published #779 at
`71f9cab8a5f97760e2e61ca4c74876b715f8aebc`. Both cases and assertions remain;
only the required forced-General compilation, same twelve-byte typed BufferCopy
and fallible submit propagation adapt APIs. The compile_fail example is exact
published bytes. Four canonical peer files differ only by reviewed formatting or
import order; the remaining three are exact peer bytes. Source hashes record the
actual current revision, not the pre-format receipt.

The known grouped Readback blanket CPU-never-stalls and old context/API docs
remain a required #780 integration dependency. The full producer/caller/cache/
lock graph, renderer source/frame/body work and exact host SimFrame receipt,
retained #774 families and final same-revision hosted gates remain open. #895
ownership is untouched. These are not silently waived by this bounded review.

No source edits, project execution/checks/build/lint/tests/scanners/generators or
runtime occurred. No tests/cases/helpers/assertions/doctests were added or run.
No push, CI, compilation, native-runtime, performance, whole-family/global/final
clearance or merge is claimed.

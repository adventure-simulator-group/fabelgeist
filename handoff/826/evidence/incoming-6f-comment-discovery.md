# Incoming #780 Comment Review discovery

This is bounded pre-integration discovery, not acceptance. Three concrete
documentation findings remain in the exact candidate, in addition to known
Readback and Kernel wording work. The 210-path union inventory remains incomplete.

## Source identity and coverage

- Candidate: `6f6d77878daad401abe414da83e17c71d4ad8091`.
- Candidate tree: `beae5d0e429160fbfb457b27f5a55c2c729b1d6f`.
- Root: `1f7ecbe13f6f1cff79520e8ad4bdd9c460704bfd`.
- Main: `990430b77a9f63548ffcb642074185f61ddb3322`.
- Root tracked worktree stayed clean; candidate source was read as Git objects.

The adjacent JSON fingerprints 59 candidate files: 46 full source reads and
13 targeted reads, explicitly distinguished. Coverage includes sorting/counts,
native layouts/stride/grid, recipe identity/coordinates, Reduce, extraction and
mesh quantities, BVH/hash state, staged GpuSim history, Readback/hair word reads,
and bare-provider documentation. Targeted generated-program and render reads do
not constitute full body coverage of those files.

Candidate guide hash matches the root guide already read in full:
`610d980a0657a3f089b0178979f77f22ce6cb529e46f37ca9c3ecd495709e206`.
Git comparisons show core view/pass/helper paths unchanged. Their pinned SDK
receiver evidence is carried explicitly; no native assurance is inherited from
the old withdrawn recorder-scope verdict. This review does not replace root's
all-thirteen pre-integration inventory.

## New or uniquely exposed findings

**C-CANDIDATE-CONTEXT-REGRESSION — P2.** Candidate
`crates/fabelgeist-gpu/src/globals/wgpu_context/mod.rs:278-283` and the existing
assertion diagnostic at line 300 restore the claim that enabling borrowed-device
waiting necessarily hangs until a driver watchdog removes the device. This is a
regression of previously repaired wording. Reviewed SDK Wait semantics concern
a submission snapshot; the helper's scope-result reads add no GPU completion
wait. Neither establishes the claimed inevitable watchdog outcome. Exclude this
candidate regression and preserve root's canonical borrowed-context policy and
diagnostic wording. No assertion, input or oracle change is required.

**C-MESH-TIE-EXPLANATION — P2.** At
`crates/fabelgeist-compute/src/mesh_query/wgsl.rs:8-10`, lowest-index ties are
explained by ascending target visitation and strictly-better replacement. But
`target_id` returns the caller's arbitrary candidate order; all three kernels
now explicitly replace equal distance or ray parameter with a smaller target
index (lines 94, 221 and 336). Document the explicit target-ID tie comparison,
independent of candidate order. Full-target traversal is ascending; candidate
traversal follows the supplied list. The resulting lowest-index behavior is
consistent with the current code; its mechanism explanation is stale.

**C-REDUCE-RESULT-EXTENT — P3.** At
`crates/fabelgeist-compute/src/reduce/mod.rs:22`, the unconditional description
of one returned native record omits the early return in
`reduce/execution.rs:35-40`. At most one complete buffer record returns the
original Buffer unchanged, including zero complete records or trailing bytes;
such texture populations return an error. State that larger inputs reduce to
one record and describe these buffer/texture branches. This requests accurate
documentation, not a behavior change.

The mesh-query and Reduce wording belongs to #780. Context-comment preservation
requires root/owner source reconciliation. No source was independently edited.

## Known work, deduplicated against current source

Readback remains partially repaired. Its `record_pending` docs at lines 149-150
now say to discard the handle after submit failure or batch drop, and its
existing test callers now propagate the submit Result. Those subparts should
not be reported as still absent. The module's lines 10-14 still promise the CPU
never stalls, while `read` at lines 176-187 omits both the producing-batch
submission prerequisite and its native explicit wait. The owner must document
helper-`Ok` handling, policy-relative acceptance, mapping completion and native
waiting without a global no-stall promise. This is the existing F9 dependency.

Kernel's known blanket claims remain at lines 12-16 and 68-70: every other
primitive supposedly standalone-submits and every noncached dispatch supposedly
allocates uniforms. PrefixSum/RadixSort batch, and static zero-uniform interfaces
need no uniform allocation. Complete these known owner repairs with canonical
source; do not add unsupported timing or allocation promises.

## Source-backed contract observations

Sorting docs match the inspected distinct count/key-width types, early no-op
priority, eight-bit schedule, scan stage and copy-back ordering. They preserve
the provider-limit caveat. Stride/record count and invocation/workgroup docs
distinguish native roles and avoid claiming contents or allocation identity.
Recipe secondary bindings sort by name; coordinate shape and scalar projection
requirements are retained separately. Not every generated program was read.

Extraction capacity and complete-primitive descriptions align with the inspected
emission guards and indirect rounding. Mesh quantities and target selections
explicitly leave index membership and actual attributes caller-owned.

RecordedHashGrid says recording is not a submission receipt, retains mutable
native contents and explains bindings remaining with original allocations after
builder reallocation. BVH count docs describe the recorded build rather than an
observed submission. GpuSim stages clock, shape and collision-history locals and
publishes them after helper `Ok`; this is policy-relative and proves neither
GPU completion nor global native acceptance.

Hair `read_words` uses checked word-to-byte copies, one grouped Readback mapping
and fallible submit propagation. Bare HairRenderer and Bevy additions state the
agreed device-handled policy. Construction/rendering `Ok` is not observed native
validation or completion; readback retains separate poll/map/channel outcomes.
The headless provider requests its device without installing a custom handler,
consistent with its default-handler documentation. These are source observations,
not runtime verdicts.

## Integration gates and limits

The candidate deletes root's original texture-view fixtures/tests/doctest and
hosted workflow step. Do not import those removals wholesale; the unchanged
guide's public-test evidence must remain true in the integrated source. The
candidate also introduces tests/cases and compile-fail examples relative to
root. Rule/owner export reconciliation must honor the user's existing-tests-only
constraint. This review neither imports nor adds those artifacts.

As a connected code observation sent to root, candidate `GpuBvh::build` and
`PrefixSum::run` still ignore fallible `batch.submit` results. Their native-error
behavior is not cleared by this documentation review.

No source integration or edits, project checks/builds/tests/scanners/generators,
runtime execution, new tests/helpers/assertions, push, CI requests or external
posts occurred. Git source reads, hashes and diffs only establish identity and
bounded review evidence. No compile, runtime, browser/hardware, performance or
GPU-completion verdict is supplied.

Remaining #780 and retained #774/F19 families, the full 210-path/thirteen-lens
inventory, final independent reviews and required hosted checks on the same
final pushed revision remain required. No whole-candidate, whole-family or
whole-PR clearance is awarded.

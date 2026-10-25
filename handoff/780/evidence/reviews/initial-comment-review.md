# Independent Comment Review: initial discovery

Snapshot: PR HEAD `e14407cc494615f0b16522fb7aa7672a3589a3c3`;
current main `3a50b708381ad00c116445c2c999a49c3943088b`;
PR #779 `71f9cab8a5f97760e2e61ca4c74876b715f8aebc`.
The worktree was clean before and after this read-only review.

This is initial discovery, not final clearance. No source files, tests,
assertions, or helpers were added or changed. No project checks, builds,
scanners, hosted CI requests, or external posts were performed.

## Actionable findings

1. P3: correct the stable-reduction rationale at
   `crates/fabelgeist-compute/src/vertex_normals.rs:4-5`.
   The comment says floating-point addition "does not commute". The reason
   corner ordering matters is that floating-point addition is not associative.
   The producer writes the original triangle-corner index into `corners`
   (`write_corner`, lines 105-111), stable sorting preserves that order for
   equal vertex keys, and `gather_code` sums the indexed contributions
   sequentially (lines 182-190). Suggested wording: "floating-point addition
   is not associative: different accumulation orders can disagree in the last
   bit." This is a pre-existing defect in the explicitly requested unchanged
   producer/consumer scope.

2. P3: update the consumer inventory at
   `crates/fabelgeist-compute/src/sort/mod.rs:3-6`.
   "Two things upstream need it" and "Both" claim an exhaustive inventory that
   omits vertex-normal corner grouping and both underlayer uses (weld hashes
   and triangle-cell buckets). Evidence: `vertex_normals.rs:331`,
   `device_underlayer/weld.rs:49`, `device_underlayer/gap.rs:62`,
   alongside BVH `gpu/mod.rs:236` and self-collision `selfcollision.rs:208`.
   Replace this with non-exhaustive examples or enumerate the complete family.
   Preserve the useful explanation that callers record into their own batch.

3. P3: narrow the exactness claim at
   `crates/fabelgeist-shell/src/wgsl.rs:198-203`.
   SELF_COLLIDE docs claim accumulating then applying "is exact". Its
   accumulator converts `delta * FIXED_SCALE` to `i32` (lines 249-252), and
   the fourth lane counts contributors for averaging (lines 253-256).
   That avoids lost atomic updates but quantizes each correction; it does not
   preserve an exact floating-point result. Suggested last sentence:
   "Accumulating and then applying costs one extra pass and avoids lost
   updates; corrections use the fixed-point resolution below." This is also
   pre-existing in the explicitly requested cloth consumer scope.

## Dispatch reconciliation dependency

`sort/quantities.rs:61` says the existing provider admits a native grid at
recording, above `From<SortTileCount> for [u32; 3]`. This accurately describes
this initial PR snapshot: `KernelBatch::dispatch` still accepts `[u32; 3]`.
It must change together with the required #779 reconciliation, whose canonical
`fabelgeist-gpu/.../pass/grid.rs` owns `WorkgroupGrid` and emits native axes in
`WorkgroupGrid::record`; #779's `KernelBatch::dispatch` accepts that owner and
`dispatch_items` accepts `InvocationCount`. Do not retain the primitive-grid
comment or adapter after integrating the final API. Sorting item/tile counts
remain separate quantities, with an explicit conversion at the launch handoff.
This is a connected repair dependency, not an inaccurate present-head comment.

## Source-backed coverage

- Read root and Rust AGENTS; read character-creator and shared armor AGENTS.
  The shape-review workflow does not apply to this comment-only review.
- Read the complete sort implementation, quantities module, new README,
  existing and newly introduced sort test code, crate exports, and all changed
  consumer hunks. Shader comments were read with histogram, scan, scatter,
  clamping, no-op admission, scratch sentinel, growth, and odd-copy-back code.
  The README's count/width distinction and no-op/rejection order describe this
  snapshot. This does not certify that public scratch fields uphold all
  aggregate invariants; that is the admission/aggregate audit's concern.
- Read BVH allocation/build/refit consumers, Morton CPU producer, and GPU
  Morton-code/hierarchy producers. The 30-bit producer and four-pass schedule
  agree. Refit locality advice is a quality consideration; it does not affect
  this migration's sorting quantity contract.
- Read vertex-normal corner, range, gather and record paths plus existing host
  reference/test comments. Stable sorting preserves corner order. No change to
  weighting or topology certification is inferred.
- Read cloth allocation/hash-sort-range-collide orchestration and unchanged
  WGSL hash/range/collision consumer. Stable grouping and bucket-contiguity
  comments match the code; the exactness claim above does not.
- Read complete underlayer workspace, weld and gap paths. Weld stable ordering
  supports the lowest/next-vertex links. Gap width includes the unused-slot
  sentinel `BUCKETS` (`BUCKET_BITS = trailing_zeros + 1`); slot keys and range
  filtering agree. The overflow-list handling is consistent with the larger-
  triangle comment.
- Compared current main/PR diffs and read #779's canonical grid and documented
  launch contracts plus kernel API diff. No live-main movement or future
  dispatch integration is represented as cleared.

## Limits and acceptance

The review used direct source reads rather than historical verdicts. Existing
JSON fixtures were inspected through their consuming tests; their native-output
provenance was not independently regenerated, and no runtime behavior or
performance measurement was claimed. Full hosted checks and all-thirteen-lens
clearance remain outside this reviewer assignment. After connected repairs,
repeat Comment Review on the final pushed revision, including comments in any
new owning modules and #779 handoff changes. Verify that all three findings are
resolved or have an explicit accepted disposition.

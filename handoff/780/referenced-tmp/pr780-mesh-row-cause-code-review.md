# Independent Code Review: Mesh strip row failure causes

No actionable correctness defect found in this narrow error-schema change.
Snapshot: Root HEAD `b11d6697c5b2fc05ca59cf7497f3d6ab626580b5` plus current working strip.rs. Read the complete current/baseline module,
all-thirteen preinventory and actual Card/Creator row admission/error callers.
No source edits, project checks, tests/cases/helpers/assertions or external
operations were performed.

The old and new acceptance predicates are identical: usize rows must convert to
u32 and be >=2. The new RowAddresses stores the actual TryFromIntError from
that conversion with #[source]; values 0 or 1 instead yield MinimumRows.
from_segments still performs the same checked usize addition then the same row
admission. Only addition overflow now yields SegmentRows with its original
segment context, without manufacturing a conversion cause. On 32-bit hosts no
usize-to-u32 overflow exists, but the conversion and its error representation
remain valid. On 64-bit hosts values above u32::MAX reject with the real cause.
Zero-segment, maximum-row and usize-overflow outcomes remain rejected exactly
where they were; only failure classification/source detail changes.

Subdivision and row native projection are byte-identical to the baseline.
The entire MeshStripLayout implementation, including its checked cardinalities,
vertices/triangles/corners and face order/index formulas, is byte-identical.
Consequently the existing strip/empty/native-byte proofs are unaffected by this
delta. No rows/layout construction bypass was added.

CardStripLayout::new continues using ? on the canonical MeshStripResult;
CardError::Strip #[from] keeps MeshStripError in its source chain. Creator
ribbon_rows/cards/ribbons likewise propagate through anyhow with ?. Actual
callers do not pattern-match the removed Rows variant or classify errors via
prose, so the enum split introduces no dangling match/API link in this family.
No new dependency or parallel error owner was introduced.

This is a static scoped verdict, not the final combined audit or hosted CI
acceptance. All broader pending families remain pending.

```text
92f8eede012a1018363a73fcd79a8f703862a3f6c40291a30e2cd02264e95ffe  crates/fabelgeist-mesh/src/count/strip.rs
dd53d7fa66134977308f2dd0279c5d38bef8e907c739f6098904a40541a37208  target/pr780-audit/mesh-strip-row-cause-before.rs
7affba536778dd1ae65da9b834c2400b2eddd4ce6bc6a7dc7aba5048a5fbf878  crates/fabelgeist-hair/src/cards/layout.rs
818294f1347dd1b4d5bcd5535dae5c0cfe30b75dd1d4380091bf98a06e906b6b  crates/adventuresim-character-creator/src/hair/gpu/strips.rs
```

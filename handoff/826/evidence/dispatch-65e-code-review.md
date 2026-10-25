# Independent dispatch checkpoint Code Review

Revision: `65e8aebedb6490bc2945eac6d48479ea6a2cd10a`.
Tree: `24cf778b808d5be97c829afcc4c6bebbdbcfa64a`.
Parent: `b874e611826c27085da897903cacfc31998d0210`.
Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.
Canonical fixture source: `95f9d31b7d9fbb56aa8330c72fffb46fb8e0c0be`.

## Finding

**DG65-C1 / P2 — Admit empty geometry before nonempty shape checks.**
At `shape.rs:85–86`, an admitted `[64,2,1]` shape rejects zero invocations
with `NonLinearShape`. No launched lane requires flattening. Current root
`KernelBatch::dispatch_folded` returns early for zero before resource work.
Canonical95 instead evaluates `kernel.groups_folded(items)` before batch
occupancy admission. Propagating the new fallible result would turn the
established empty no-op into an error before that guard. The root ledger
already records the empty-first contract as requiring repair. This remains
an owned actionable gap, not a newly assigned caller defect.

Put empty admission in the canonical WorkgroupCount-based folded projection,
then derive invocation coverage through that owner. Preserve empty `[1,0,1]`,
nonempty one-dimensional checks and the padded-address bound. The scan
producer must share this admission owner for its actual native stride.

## Source conclusions

The full padded rectangle is checked as `x*y*width`, using checked u64
products and at most `2^32` lanes. The greatest launched index therefore
fits u32 even in the padded final row. A live-count check alone would miss
those lanes. Nonempty y/z admission matches one-dimensional flattening.
Ordinary covering retains ceiling division and prior y/z behavior.

Actual receiving-device admission checks every axis. The lower receiver runs
it before pipeline/binding work. SDK29.0.4 `command/compute.rs:835–861` likewise
rejects any excessive axis, even when another is zero. Pure host admission
errors have no native cause; their concrete values survive direct anyhow
propagation. Native recording/submission errors retain their actual cause.

Both existing fixture files and the existing compile-fail block are
byte-identical to canonical95. No authored test-population change is credited
to this review. Exact hashes and coverage are in the JSON companion.

## Required connected work and limits

The empty-first and shared WorkgroupCount projection remain receiving repairs.
#780 must migrate Kernel/FastPath/callers to the fallible projection and
actual-device grid admission after its empty guard. PrefixSum must close its
actual block-stride/padded-address contract through the canonical owner or an
explicitly proven custom contract; raw folding supplies geometry only.
Current old APIs are required integration gaps, not accepted deferrals.

All five changed files were read completely. Full fast-path, lower error,
submission and export sources were read; affected Kernel production functions,
PrefixSum production and pipeline/device portions were inspected at the exact
pins. JSON distinguishes partial evidence and unchanged caller reference
inventory from fresh whole-file reads.

This was source/Git/provenance review only. No project check, build, test,
scanner, generator, runtime, push or CI ran, and no source was edited. Native
producer/layout/source/creator and lock integrations remain required. This
checkpoint does not establish compilation, full-family, all13-lens,
whole-PR or final clearance. Final reviews and hosted gates remain required
on the same final pushed revision.

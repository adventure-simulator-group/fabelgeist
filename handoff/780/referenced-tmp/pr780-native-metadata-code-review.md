# Independent Code Review: native level/source/simulation delta

Base and HEAD: `d1364416d19aeacef79b8cbb4abd7eec07f1bc1b`.
Reviewed the uncommitted working delta; fingerprints appear below. Minor
formatting/documentation edits occurred while source was being read, so this
report pins the final captured contents rather than claiming an immutable
committed revision. Review-only: no source changes, local builds, tests,
format/quality tools, CI requests, pushes, new tests, or external posts.

## Findings

No actionable defect demonstrated in the requested native metadata delta.
This is a scoped static Code Review verdict, not global thirteen-lens
clearance, compilation evidence, or the final combined same-SHA review.

## Source-backed contracts checked

- `gpu/lods/levels.rs::record_levels` always starts with the actual groom
  layout, then filters requested point quantities before native conversion.
  Accepted added levels remain strictly descending, at least two points and
  at most 32. Every returned `Arc<GpuLevel>` receives the same checked layout
  used for its position/info allocation and the actual allocating device.
  `gpu/lods/records.rs` seals public construction/mutation of those records
  and exposes a borrowed slice. The sole `GpuLods` producer in
  `gpu/lods.rs::lods_against` receives that nonempty result; first-level
  indexing in `GpuLods::device` and simulation construction therefore has a
  producer-backed nonempty invariant. This does not review or clear the
  separately pending card-layout contract.
- `gpu/sim.rs::make_sim` caps requested guide cardinality in host width before
  constructing the native `StrandLayout`; requested points retain the old
  minimum-two policy and receive checked native admission. It retains the
  actual first-level `Arc`, verifies its device before commands, and
  `resimulate` verifies the old simulation source before preparing the new
  one. Resimulation intentionally permits a different actual source; it
  does not conflate transfer compatibility with source identity.
- `gpu/sim/binding.rs:33-36` verifies both actual devices and admits both
  requested populations as prefixes of their own source layouts before
  allocating binding arrays. Lines 46-54 pass distinct source buffers,
  distinct strides, separate populations and actual `Arc` identity to the
  one canonical `wgsl/sim.rs::sim_bind` implementation. The shader's direct
  prefix path requires that identity plus `s < guide_count`; otherwise its
  nearest-guide loop reads the drawn root from the drawn source and each
  guide root from the guide source. The original nearest-four sorting and
  inverse-square weighting arithmetic remain unchanged.
- Empty drawn populations dispatch no binding invocations. An empty guide
  population with a populated drawn source runs no nearest-guide iteration,
  produces zero weights, and `sim_transfer` skips every zero-weight lookup
  before `old_at`. Thus transferring from an empty source does not use its
  physical sentinel as a logical guide. The new simulation starts at its
  transformed rest points in that case.
- `gpu/sim/rods.rs::RodBuffers::new` checks the native sample population times
  fourteen before allocations. `wgsl/sim.rs::store_factor`, `factor_l` and
  the back-substitution body address fourteen vec4 records per point, through
  offset thirteen. The same admission bounds the two-record shape/frame
  arrays. Population-times-points is already admitted by `StrandLayout::new`.
- `gpu/sim/collision.rs:43-69` retains a private checked population. The first
  segment multiplication cannot overflow: simulation points are at least
  two, and its enclosing `StrandLayout` proves strands-times-points fits
  u32, so strands-times-(points-minus-one) also fits. Checked multiplication
  then bounds per-segment expansion and eight records per sample. This
  bounds the shader's five-record sample data, two-record sample slots/
  asks, and eight-record sorted output. Buffer byte layouts preserve the
  original native strides. Logical zero remains zero while `storage` alone
  provides physical binding minima. The separately pending finite physical
  grid/radius contracts are not cleared by this arithmetic proof.
- `gpu/sim/execution.rs` derives every simulation and collision count/launch
  from those retained layouts. The shader addresses in `sim_samples`,
  `sim_sort_samples`, `sim_collide` and `sim_collide_apply` fit the admitted
  expansions; range-scanning order remains unchanged. State publication
  remains after accepted submission, including collision progress.
- `gpu/sim/inspect.rs::read_points` verifies the source device, returns an
  empty vector for logical zero and otherwise admits exactly
  `layout.positions_bytes()` via `Buffer::prefix` before readback. The
  existing provider's `buffer/prefix.rs` checks logical and physical extents,
  and `buffer/read.rs` returns the logical byte extent rather than physical
  allocation padding. Both positions and rest reads use the same path.

## Coverage and limits

Read the ledger's `native_level_source_metadata_repair`, its thirteen
lens dispositions/acceptance, collision extent addendum and binding transfer
addendum. Reviewed the complete new records, level recording, binding,
collision, rod-buffer and inspection modules; complete changed simulation
constructor/execution/density bodies; relevant old versions for behavioral
comparison; the changed LOD recipe and host conversion; canonical binding,
transfer, rest, factor and collision WGSL receiving bodies; native layout,
storage, prefix/readback providers; and existing test API adaptations. Existing
case/oracle meaning remains intact by source comparison. No tests were run.

Explicitly excluded from this delta verdict: CardLayout and Creator receiving
work; #826 renderer receiving work; physical/grid units and finite geometry
admission; host SimFrame and settings semantic migration. Those remain
separate incomplete families. No full unrelated renderer/creator audit,
hosted-check verification, global complete audit wave, or final combined
revision review was performed.

## Captured working-delta fingerprints

```json
[
  {
    "path": "crates/fabelgeist-hair/src/gpu/error.rs",
    "sha256": "ab1dbe6caa208c95dcf21431f847bc3fc68c8ef0c42bccb6a03dcb28f3396e6a"
  },
  {
    "path": "crates/fabelgeist-hair/src/gpu/lods.rs",
    "sha256": "4c77d68591854015441a69a653f7fc1ed0bd7f44f88ac35cb1e7facf061eb7d9"
  },
  {
    "path": "crates/fabelgeist-hair/src/gpu/lods/levels.rs",
    "sha256": "650943d671d88ceb81a808c6e1f94eddb4780e018518f930fce3d3b1bf4bac3b"
  },
  {
    "path": "crates/fabelgeist-hair/src/gpu/lods/records.rs",
    "sha256": "1c6a3dfe68c68c02c79ebf1157be9981e4c2a31f8a7667dd64148e30386bd70d"
  },
  {
    "path": "crates/fabelgeist-hair/src/gpu/mod.rs",
    "sha256": "2883a6d8baa2934fcd41d27b8af312abe2a8a581de78f961c1593b45d257bab1"
  },
  {
    "path": "crates/fabelgeist-hair/src/gpu/sim.rs",
    "sha256": "5ae5a922ea1e5a8d28c918ddfc36deb028ee2d3c7e8f85bd3cb660db99533ec0"
  },
  {
    "path": "crates/fabelgeist-hair/src/gpu/sim/binding.rs",
    "sha256": "c5459f6db0f1b23d67efb540f90f14140a65d958b4bf251dbd945150e551bed0"
  },
  {
    "path": "crates/fabelgeist-hair/src/gpu/sim/collision.rs",
    "sha256": "b4e16d6c3ce2d7c25351c3fff93e224adad4847aaed7f40b968607ae0bacc7d4"
  },
  {
    "path": "crates/fabelgeist-hair/src/gpu/sim/density.rs",
    "sha256": "3e2fdda60c4b86a8b581252240b5d806979cb4c5eee5d161ff7171169f77dbe2"
  },
  {
    "path": "crates/fabelgeist-hair/src/gpu/sim/execution.rs",
    "sha256": "159c908f914cf7e272a86ad90d384e3738a9a7e3bef7253cdad72b866783f391"
  },
  {
    "path": "crates/fabelgeist-hair/src/gpu/sim/inspect.rs",
    "sha256": "074f9fe7a8b1c8094eb081ca14597c452b041bc9c30814dcf092fb494bf4790a"
  },
  {
    "path": "crates/fabelgeist-hair/src/gpu/sim/rods.rs",
    "sha256": "8ec4a8182b2b06542654966c75b6120c9acc24852b3d9e05f23b920806bb3425"
  },
  {
    "path": "crates/fabelgeist-hair/src/gpu/wgsl/sim.rs",
    "sha256": "05e3c529151d982db674bce7bf65db2e3126946cb848148d7c50d3047915ced6"
  },
  {
    "path": "crates/fabelgeist-hair/src/lod.rs",
    "sha256": "a4cd680b59ae0b7c37be4829566c1f975805e08ef83d766c1f3812411025ab04"
  }
]
```

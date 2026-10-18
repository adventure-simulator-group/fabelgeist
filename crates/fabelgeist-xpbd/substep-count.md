# Solver substep cardinality

`SubstepCount` is the bespoke type for subdivisions of one solver step and for
substeps covered by one shell host-contact projection. It lives in XPBD and is
re-exported by `fabelgeist-shell`. It is distinct from constraint sweeps, host
contact sweeps, outer-layer passes and elapsed time.

Every native `u32` count is admitted, including zero and `u32::MAX`.
`From<u32>` and the const `from_native` constructor share that policy. Private
storage prevents accidental interchange with unrelated counts. Native `Debug`
formatting preserves settings diagnostics. A large admitted count can still
request impractical work; admission introduces no new work limit.

Zero policies remain at their consumers. `Solver::record_step` and
`Solver::step_interleaved` disable work for zero. Shell interleaving and creator
preview explicitly call `at_least_one`. A shell host interval of zero disables
host projections. Creator stages require 1 through 32 substeps and a host
interval no greater than that count; preview controls retain 1 through 8.
Neither range constrains other callers. Solver, fit and preview defaults remain
10, 12 and 2 respectively.

Native projections occur directly at the mathematical or representation port:
XPBD's `u32` loop and `f32` duration division; shell's loop, modulo, final-index
comparison and duration division; creator's loop, duration division and motion
interpolation; egui's mutable scalar slider; and scalar JSON serialization.
The shell retains its interval and count as typed values between those ports.
Its native index is a loop position, rather than another scheduling owner.
Arithmetic order, constraints, callbacks, submissions and final host projection
policy are unchanged. Creator JSON still admits every `u32` before validation.

Construct public settings explicitly:

```rust
use fabelgeist_xpbd::{SolverSettings, SubstepCount};
let settings = SolverSettings {
    substeps: SubstepCount::from(3),
    ..Default::default()
};
```

Public GPU regressions exercise zero, single and multiple counts, hook timing,
constraint order and batched versus interleaved readbacks. Creator tests cover
scalar serialization, stage validation, private preview minimum-one behavior
and native controls. These checks preserve count behavior; they do not establish
hardware performance, garment design, rendered appearance or collision-free
body equipment. Duration and callback scalar types remain separate work.

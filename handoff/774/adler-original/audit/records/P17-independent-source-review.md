# P17 independent source review

No actionable Code Review or Rule Review finding in the connected local
report/capture increment `90a422a453cd990ebc7f0ecf2ad5ed3e2d57d696`, compared
only with parent `18360adba0fb43add88c75dfbd2f45d7e71d8ace`. This is a
source-only verdict for the 32 changed paths and their directly affected
receivers. It does not grant full P17 acceptance.

P17 remains incomplete: both optional `skeleton_tick` projections in
[`physics.rs:600`](SNAPSHOTS/report-p17/crates/adventuresim-tactical-core/src/physics.rs:600)
and
[`physics.rs:628`](SNAPSHOTS/report-p17/crates/adventuresim-tactical-core/src/physics.rs:628)
still fabricate zero for absence at this pinned revision. Their camera-owned
repair is an accepted pending dependency inside unfinished P16. The whole motor
repair was not reviewed or imported here. The parent extrema prerequisite is
already integrated separately and does not require replay.

The source trace was assessed before checking the handoff and ownership claims.
The ledger records the pending motor dependency and unintegrated P17 source.

| Boundary | Source-backed trace and verdict |
| --- | --- |
| Native bouncy producer | [`bouncy_bones.rs:205`](SNAPSHOTS/report-p17/crates/adventuresim-tactical-client/src/animation/bouncy_bones.rs:205) admits each native spring length before typed extrema. Telemetry retains the first geometry error across rig aggregation; the unchanged update body resets it each frame and preserves spring dynamics, coupling, tuning, clamp order and rig traversal. |
| Capture admission and retained state | [`capture/admission.rs:87`](SNAPSHOTS/report-p17/crates/adventuresim-tactical-client/src/animation_viewer/capture/admission.rs:87) preserves optional IK diagnostics and requires actual support where the procedural solver owns support. `advance_distance` retains the original distance + speed * elapsed f32 order and admits the result before storing it. [`capture/root_motion.rs:20`](SNAPSHOTS/report-p17/crates/adventuresim-tactical-client/src/animation_viewer/capture/root_motion.rs:20) retains terrain absence and checked signed clearance. |
| Frame construction | [`capture/sample.rs:25`](SNAPSHOTS/report-p17/crates/adventuresim-tactical-client/src/animation_viewer/capture/sample.rs:25) admits native body degrees once, consumes typed bouncy errors, and retains optional body response, compression, IK release/yaw and settle observations. The caller pushes the frame only after successful construction. Global trace elapsed time retains the existing elapsed owner through serialization. |
| Repeated evaluation | [`capture/repeated_evaluation.rs:132`](SNAPSHOTS/report-p17/crates/adventuresim-tactical-client/src/animation_viewer/capture/repeated_evaluation.rs:132) separates missing/unexpected bones, invalid numerical observations and measured transform differences. Optional IK presence is compared explicitly. The unchanged captured-bone receiver guarantees equal lexical iteration order after key-set equality; existing numerical noise thresholds remain unchanged. |
| Metric owners | [`report/observation.rs:45`](SNAPSHOTS/report-p17/crates/adventuresim-tactical-client/src/animation_viewer/report/observation.rs:45) owns distinct NotApplicable, Unavailable(reason), and Measured states. Signed ranges, wrapped phase-height samples, aggregate per-cycle peaks, finite knee dots and quality fractions preserve their guarantees. The clearance, height, pose, facing, continuity, seam, slip, drift and guard-liveness producers retain existing checked scalar owners through metric state. |
| Acceptance and quality | [`report/validation.rs:6`](SNAPSHOTS/report-p17/crates/adventuresim-tactical-client/src/animation_viewer/report/validation.rs:6), extracted acceptance modules, and [`validation.rs:457`](SNAPSHOTS/report-p17/crates/adventuresim-tactical-client/src/animation_viewer/report/validation.rs:457) consume actual typed measurements. Required missing evidence fails; intentional no-eligible-plant slip/drift and scenario exclusions remain exempt. Incomplete capture quality is unavailable, while measured quality zero remains zero. |
| Error and output boundaries | [`report/mod.rs:49`](SNAPSHOTS/report-p17/crates/adventuresim-tactical-client/src/animation_viewer/report/mod.rs:49) builds the fallible report before opening trace/manifest/HTML files. New report admission failures preserve their cause and return failure before those writes. Capture admission uses terminal rejection; unchanged schedule guards prevent continuation into a valid completed report. Existing disk-write failure behavior is not transactional. |
| JSON and HTML receivers | [`report/model.rs:126`](SNAPSHOTS/report-p17/crates/adventuresim-tactical-client/src/animation_viewer/report/model.rs:126) serializes observation states and existing scalar owners. [`report/html.rs:69`](SNAPSHOTS/report-p17/crates/adventuresim-tactical-client/src/animation_viewer/report/html.rs:69) displays actual measurements separately from absence. The unchanged player reads scalar frame fields; global-trace consumers keep the same scalar elapsed representation. |

Source comparison covered the original f32 arithmetic grouping, frame/bone
ordering, maximum tie selection, phase wrapping, support/turn thresholds and
acceptance limits in the moved numerical kernels. Measured zero, negative signed
clearance/gain and knee dot -1 remain representable. Checked arithmetic failures
propagate through the existing geometry/animation errors or owning report error.
Existing fixture adaptations were inspected as source; no fixture was executed.

The rule review applied root/scoped instructions and the rule-review skill's
bespoke-type reference. Relevant contracts included existing ownership and typed
producer propagation in
[`crates/AGENTS.md:28`](SNAPSHOTS/report-p17/crates/AGENTS.md:28),
[`crates/AGENTS.md:36`](SNAPSHOTS/report-p17/crates/AGENTS.md:36),
named typed records at
[`crates/AGENTS.md:45`](SNAPSHOTS/report-p17/crates/AGENTS.md:45),
and the user-specified item ordering reflected at
[`crates/AGENTS.md:90`](SNAPSHOTS/report-p17/crates/AGENTS.md:90).
No enforcement suppression or baseline change is part of this increment.

No Cargo, build, check, test, fmt, Clippy, metadata, scanner, parser, generator,
runtime or numerical probe ran. No tests, cases, assertions, helpers or doctests
were authored. There was no Comment Review, broader bespoke-audit wave,
full-repository consumer proof, motor implementation review, renderer execution,
HTML execution, or hosted validation. The integration head's hosted run excludes
P17 and supplies no validation evidence for this source. Source review does not
establish compilation, numerical execution, formatting or size-gate success.

The additional workflow check was limited to the exact Check formatting diff in
coordinator commit `6533e444fa37552927070cc8d8190907fd3b413e`. The same four
existing cargo fmt --check commands remain. Each failure sets `format_status=1`,
later commands remain reachable, and the final quoted exit rejects any failure.
No actionable shell error or gate weakening was found. No workflow commands or
probes ran, and no semantic verdict was made on the separate formatter replay.

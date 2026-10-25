Joint names, rig ordinals, animation clocks, physical measurements and combat
identities have different meanings. Primitive arguments and retained fields let
callers lose those distinctions between loading, state updates and rendering.
This change carries shared bespoke types through those paths and checks derived
values before replacing the state they own.

This is a migration increment. The remaining producer migrations listed below
are still open; this PR does not complete the repository-wide migration.

## Resulting contracts

- The shared rig crate owns joint identities, ordinals and ordered collections.
  Momentum Human Rig naming policy is separate from clothing selection. Checked
  access handles unproven ordinals; an ordinal does not establish membership in
  a particular rig. Exact `root` is the pelvis and exact `body_world` is the
  motion root; `ROOT` and `Root` remain unbound.
- Binary Filmbox (FBX) names distinguish qualified names from normalized labels.
  Admission removes the first `[0, 1]` class suffix before lossy UTF-8 decoding.
  Qualified names retain namespaces; normalized labels use the last colon
  suffix. Empty and unfamiliar names remain valid, and duplicate caches retain
  the first match.
- Animation coordinates, nonnegative spans, frequencies, unwrapped cycle
  displacements, interpolation fractions and additive weights retain their
  distinct contracts. Signed sample times and reverse playback remain valid;
  additive weights may exceed one. Declared zero-weight clips retain playback
  topology. A baked clip has a nonzero sample count and matching channel lengths;
  single-sample clips remain valid.
- Physical speeds, lengths, authored angles and configuration values retain
  their units and accepted ranges through their actual consumers. Shared leaves
  live in the lowest common crate, with direct re-exports where appropriate.
  Numerical conversions stay with the operation or format that consumes them.
- Melee arrival is `NotNeeded`, `ArrivesIn` or `CannotArrive`; recovery is
  `After` or `CannotRecover`. Unavailable timing does not start a timer with a
  fabricated value. The server's original pre-delay attack identity remains
  distinct from its animation clock through preparation, contact, outcomes,
  diagnostics and deterministic seed construction.
- Action and posture progress remain typed through retained state and playback.
  Signed attack-pose coordinates retain drawback and overshoot. Held-input and
  sprint timers use checked elapsed values and positive configured thresholds.
  Failed preparation does not publish a new attack or consume its forced
  trigger; clearing a stale previous-frame attack edge remains intentional.
- Body-facing and camera smoothing preserve their original scalar arithmetic
  and clamp order. Their shared shaping-time input is finite and signed.
  Unavailable orientation history is explicit; a real identity orientation is
  still valid.
- Inverse kinematics (IK) retains checked offsets, clearances, phases, support
  confidence and reach inputs through terrain and solver operations. Plant and
  terminal-contact states own their required payloads. Missing current rig
  frames and uninitialized history are explicit, and a missing visible foot
  cannot become a fabricated observation at the rig origin. Historical frames
  remain historical.
- Optional procedural support confidence remains optional through diagnostics,
  capture, JSON and report readers. A real zero differs from unavailable data.
  Repeated or zero-elapsed solves retain their existing confidence estimates;
  physical-contact acceptance separately checks captured ankle/terrain geometry.
  Missing required data cannot become a passing check.
- Managed asset requests retain their identity through initial load failure and
  replacement. Invalidations retire derived availability, baked clips,
  calibration and plan caches before reuse. Replacements can recover after a
  matching asset/store change. Both evaluation and presentation reject
  calibration from an obsolete rig or relevant runtime configuration.
- Autoresolve battle coordinates remain finite and signed through readiness,
  scheduling, defense, movement and timeline records. They are separate from
  elapsed intervals and tactical-authority time. A negative battle coordinate
  remains valid; an unrepresentable owned interval returns a structured error.
- All maneuver durations retain their checked positive-seconds type through
  conversion to animation samples. Quantization preserves multiplication,
  rounding, a one-sample minimum and saturating count conversion.
- Playback response rates, phase-error limits and step cadence remain checked
  through gait and attack presentation. Step cadence permits zero and caps an
  explicitly unrepresentable demand. Contact and landing differences retain
  event-count types through coalescing, height, IK observation and report
  validation; event counts remain distinct from animation sample spans.
- Movement audio carries deterministic seeds from event derivation through sound
  selection without a numeric round trip. Stream framing and draws are unchanged.
- Capture elapsed time stays checked through jitter-frame storage and interval
  calculation, with scalar seconds projected only at derivative division. The
  configured knee-facing angle stays typed until its angular clamp.
- Phase curves explicitly distinguish identity behavior from admitted sampled
  data. Replacement curves and calibration fits are prepared before publication.
  Capture startup prepares checked inspection data before publishing outputs;
  rejection reaches the existing failure lifecycle instead of retrying forever.

Character-generation, clothing and equipment paths retain admitted catalogue
records and authored order. Every packed joint slot is checked, including
zero-weight slots. The item `leather_belt` becomes `belt`, displayed as
`Belt`; portrait filenames, catalogue revision and item-dependent hashes
follow that identity. Portrait bytes, physical parameters and attachment
topology are preserved. There is no compatibility path or database operation.

## Deliberate admission and wire changes

The committed combat YAML owns validated defaults. Required playback thresholds,
landing-impact cutoff and pose-buffer numerical policies are configuration
fields with their previous numerical defaults. Six previously admitted negative
IK clamp limits or rates are rejected: pelvis correction speed and maximum step,
maximum hip drop, maximum planned run reach drop, terrain blend speed, and
knee-pole facing offset. Zero remains valid for these fields.

Playback now groups `maximum_per_second`, `transition_width_per_second` and
`minimum_step_distance_metres` under
`animation.playback.authored_step_cadence`, with values 5.0, 1.0 and 0.01.
The minimum calibrated step distance was previously a native numeric floor.
Admission rejects a half transition width greater than the maximum rate,
which could otherwise produce backwards phase for forward motion. Zero rates
and zero-width hard caps remain valid. The configuration schema and its
JSON-derived digest change; the old field names have no fallback.

Nonfinite owned action progress is rejected. Smoothing duration must be
positive. A native ragdoll capsule requires a positive radius while its signed
segment length retains its original meaning. Muscle release is explicit
`Never`/null or a positive age instead of a zero sentinel. Initial optional
input acknowledgement and unavailable owned diagnostics serialize as null;
ordinary present scalar values retain their encodings. A native timer uses
fallible duration admission instead of the old 30-second crash-protection cap.
Unordered reach bounds now produce an explicit no-solution result before native
clamp, preserving zero/signed angle admission and the ordered-input arithmetic.

These changes preserve real zero values, supported negative values, subnormal
values and existing successful arithmetic order. They do not promise that
every combination of individually finite inputs produces a representable result:
an unrepresentable final owned result is rejected before the affected
replacement is published. Bounded laws retain valid results from transient
overflow: a negative-infinite exponential argument gives full response, and
an explicit excessive cadence demand still reaches the finite configured cap.
The guarantee applies to the owned operation, not a global transaction. An
extreme guard-recovery time difference now returns an error instead of silently
turning an infinite intermediate into zero effectiveness.

First-party Cargo metadata uses the repository's AGPL-3.0 licence. Vendored
licensing is unchanged.

## Validation

Published revision: c59c01e3ea5471925a8954c26a08afadf5d22942.
Its [hosted CI run](https://github.com/adventure-simulator-group/fabelgeist/actions/runs/39430056255)
is running; exact-head validation remains pending. Predecessor formatting,
generated-binding verification and semantic/growth checks passed, but do not
establish acceptance of this revision. The full native/WebAssembly lint matrix
and relevant existing behavioral selections must still pass on this source.

The repeated-view capture comparison now has its own module, with its two
unchanged existing tests. The liveness-applicability predicate belongs to its
metric owner. These extractions address the reported growth violations without
raising ceilings or changing measurement thresholds.

Code and Rule Review pass for the previously reviewed IK increment. The newer
audio, jitter, knee-angle, battle-time, maneuver-duration and
playback/event-count paths received two source-review passes by the
implementation agent, including producers and identified downstream consumers.
This is not a new independent review of the full PR. Comment Review remains
omitted as requested. No local project checks ran and no test cases or
assertions were added. Existing fixture admission was adapted to checked APIs.

The previous [successful hosted run](https://github.com/adventure-simulator-group/fabelgeist/actions/runs/39310207896)
validated an earlier source revision: all 36 steps passed, including four native
Clippy configurations, WebAssembly lint, 20 required nonzero cells and 16
existing Creator command vectors. Its 427 selected executions were 424
unit/integration executions and three doctests, not the full suite. That result
does not validate the new source.

Hosted validation retains the existing commands, revision/nonzero checks,
lockfile guards, quality checker and exclusions. It also executes the existing
movement-audio, Core autoresolve/defense, server posture-projection and viewer
report tests, requiring nonempty passing output for each selection. Quality
ceilings do not increase. Numeric debt reductions come from the official hosted
report; obsolete records are removed without transferring their debt to new
owners.

## Remaining work and acceptance limits

Producer propagation remains unfinished in camera geometry, FBX source
admission, joint-indexed producer data, garment fitting, landmark payloads,
item/placement relationships, scenario handoffs, rig roles and state, and some
capture/report contracts. The remaining configuration and aggregate contracts
also need their assigned investigation. These are active work, not blanket
exceptions for primitive fields or conversions.

FBX graph identifiers remain with [#778](https://github.com/adventure-simulator-group/fabelgeist/pull/778).
The remaining migration families include shared terrain support, camera
geometry, joint/topology producers, garments, equipment, reports, per-foot
influences and the four animation-continuation parameters (`start_coordinate`,
`incoming_tangent`, `ready_phase`, `progress`). A broader rig record checking
all per-joint columns and parent membership together remains a separate
acceptance concern.

Selected hosted checks do not establish full Creator generation, specialized
fitting, production assets, UI or calibrated renderer acceptance. A historical
full Creator run failed with a segmentation fault on both its main and candidate
revisions; this does not establish a current-main baseline failure. Existing FBX
regressions do not directly validate the new name and duration behavior.
Compilation of viewer and capture cases is distinct from executing them.

There is no dedicated runtime acceptance for every replacement/recovery path,
serialized golden shape, zero-weight topology, extreme arithmetic case,
contact-gap episode, newly rejected cadence-policy combination, excessive
cadence demand or optional-observation failure. Existing combat range-fit checks
do not establish measured stance-slip/contact-timing coverage.

## Implementation context

Material decisions are canonical shared ownership, separate semantic roles,
explicit unavailable states, checked derived operations and conversion at actual
format or native boundaries. Tactical tick state remains transient in Bevy.
The final hosted result and the limits above define the verification scope.

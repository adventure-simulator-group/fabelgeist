# P04: playback policy and event-count producer migration

Surveyed on bfab9d01437fc250f77bce2c3c0571c475b19c7a. No P04 source
changes have begun. This refines the original complete family; it does not defer
its difficult paths or turn conversion syntax into an automatic finding.

## Canonical owners and final data flow

1. Use the existing Tactical Core ResponseRate for
   velocity_response_per_second. Keep zero valid. Inspect all callers of its
   exponential_fraction before using it here: the current method rejects a
   nonfinite exponent even though the complete old exponential law produces a
   valid bounded response for a negative-infinite intermediate. Preserve the
   complete original bounded law in the canonical operation; do not create a
   competing response type or a second compatibility path. Body response is a
   connected consumer of this operation and must be checked too.

2. Add purpose-specific finite nonnegative PhaseCorrectionRate and
   PhaseErrorLimit owners in a focused Tactical Core animation module.
   Correction is normalized phase units/second; limits are normalized error
   magnitudes and may exceed one. They are neither positive frequencies,
   wrapped CyclePhase, signed CycleDelta, nor UnitInterval. Retain CycleDelta
   for gait correction and ActionProgressDelta for attack correction. Remove
   the two raw-float presentation getter helpers and carry the checked policy
   through both presentation.rs and presentation/attack.rs. Native local
   subtraction, sign, ratio, clamp and interpolation keep their original f32
   order. A transient infinite correction budget must not reject a finite
   clamped correction or become retained unavailable-state data.

3. Give authored step cadence a zero-permitting step-rate owner, separate from
   positive clip sampling frequency. Own the maximum rate, smoothing width and
   existing 0.01-metre calibrated-step floor together in a checked cadence
   policy. The producer is NonnegativeSpeed plus calibrated NonnegativeLength;
   do not flatten those to an application helper taking requested:f32.
   A finite speed divided by the positive floor can exceed the scalar range.
   Model the demand explicitly as Finite(rate) or ExceedsScalarRange, so the
   real cap still bounds it without rejecting valid finite inputs or retaining
   infinity as application state. The C2 cap's native polynomial is the actual
   numeric endpoint. Its applied rate stays typed through CycleDelta admission
   and the retained gait phase.

4. Enforce the cadence-policy relationship at admission. Current independent
   scalar validation accepts ceiling=0 and width=1; the old polynomial gives
   about -0.0512 steps/second for a request of 0.1. This is a source-derived
   counterexample, not an executed test. A forward-motion ceiling must not
   create backwards phase motion. Reject smoothing widths whose half-width
   exceeds the ceiling. Preserve hard caps (width zero), a zero ceiling with
   zero width, all other valid scalar ranges and the existing default 5/1
   polynomial. Record this as an explicit aggregate-invariant repair.
   Use one checked policy with private fields and a small decoding DTO; prefer
   a clean nested YAML cadence record over compatibility aliases, cached
   classifications or a second independently mutable pair of fields. Move
   the existing metre floor into that authored policy with its current value.
   Update every serializer/default/caller atomically and document the final
   schema. No migration/fallback for the old configuration is wanted.

5. Move the client cadence state and application responsibility out of the
   near-500-line presentation facade before expanding it. Move its existing
   tests unchanged except for necessary typed API/fixture admission. The
   existing limited/cadence_capped pair has only three meaningful outcomes:
   unrestricted, stance-slip limited, and cadence capped (possibly also slip
   limited). Use an explicit state rather than admitting the contradictory
   combination capped=true, limited=false. Preserve warning transition and
   phase retention behavior. Trace logs are presentation endpoints; report an
   ExceedsScalarRange demand explicitly, never by constructing a fake sample.

6. Add a shared animation-event span/count owner and a positive coalescing-budget
   owner. Do not reuse AnimationSampleSpan: events and simulation samples are
   different quantities. Both ContactSequence::events_since and
   LandingSequence::events_since produce Option<event span>; they currently
   use checked subtraction, not half-range or wrapping chronology. Preserve
   None on a backwards/reset counter and Some(zero) for a real empty interval.
   Carry the span through ContactSequence::preceding, is_successor_of,
   presentation coalescing and height landing detection. The separate
   wrapping_distance_from observations retain their exact full-width wrapping
   policy and also return the shared event span. Migrate their IK observation
   and report-validation consumers to the actual numerical/report boundary.
   Keep the complete u64 width and native iteration order. The positive budget
   may use private NonZeroU64 with derived scalar Serde or checked admission;
   do not expose a raw u64 helper and reconstruct the budget at each caller.

7. Re-run the source trace after changes. Distinguish new calculations from
   pass-through scalar helpers. Retained records, parameters and returns must
   preserve the owner; extraction belongs only at actual vector, polynomial,
   exponential, clamping, native iteration, logging or Serde endpoints. Do not
   move P19's per-foot influence work into a generic policy waiver, and do not
   change P20's continuation contracts opportunistically.

## Complete connected inventory

- Tactical Core: combat_config/playback.rs, the parent config/default decoder,
  animation/mod.rs, response_parameters.rs, value_error.rs, contact.rs,
  landing.rs and the new phase/cadence/event owner modules.
- Authored data: content/tactical/combat.yaml. Preserve every numerical default;
  only the cadence grouping and explicit copied metre-floor policy change the
  schema. Recheck canonical YAML/default digest behavior.
- Client: animation/presentation.rs, presentation/attack.rs,
  presentation/events.rs, authored_cadence.rs (typed stride-measurement owner),
  procedural/height.rs and procedural/ik/observation.rs.
- Event-span report consumers: animation_viewer/report/validation.rs at its
  continuity comparisons/count accumulation. Do not re-erase a span merely
  because the containing record later serializes.
- Existing tests: animation/tests.rs and the authored-cadence/attack/packet
  reconciliation modules currently inside presentation.rs. Adapt numeric
  fixtures and expected scalar comparisons only; no new cases/assertions.
- Existing ResponseRate exponential callers must be enumerated before changing
  that operation. Neither an unexamined receiving method nor a private function
  is a migration boundary.

The typed stride measurement is AuthoredStrideMeasurement.step_distance:
NonnegativeLength in client animation/authored_cadence.rs. Preserve that owner.
The event-span consumers found so far are the two sequence owners,
presentation events, height, report validation and IK observation. Confirm the
complete list with current source again before editing.

## Existing validation and limits

Use hosted checks only. Keep all current gates and ceilings. Existing client
animation tests cover cadence cap continuity, gait drift/snap, attack
reconciliation, source coalescing and bounded output. Existing Tactical Core
animation/config tests cover admission/defaults. Existing viewer tests cover
report continuity; select them explicitly if the current workflow only compiles
that binary. Require nonempty execution results. Run native and WASM Clippy,
canonical config validation and the semantic/growth gate on the exact new head.
Apply relevant baseline reductions from the official hosted export.

No test cases, assertions, fixture helpers or local parser/numerical probes are
authorized by this plan. Static counterexamples must be labeled as source
reasoning; green compilation cannot stand in for existing behavioral execution.
The new cadence aggregate rejection and final YAML shape must be explained in
the PR. Do not claim visual/live production acceptance from these unit gates.

# P03: remaining maneuver-duration producers

Reviewed twice against the source at f12e7dafd and the exact prior scalar
calculation. Source structure is complete for this family; no hosted PASS is
claimed. This is the single implementation agent's Code/Rule source review.
Comment Review remains omitted.

## Complete producer-to-consumer trace

`ManeuverTimingConfig` now carries PositiveAnimationDuration for roll, dive,
backward dive and sprint slide. Get-up and quickstep already use that canonical
owner. Its private AnimationElapsed representation and checked TryFrom/Serde
admit exactly finite positive f32 seconds, including positive subnormals and
large finite values. The committed YAML and its values are unchanged; scalar
serialization and configuration digest inputs retain the original numeric bits.

The six maneuver values no longer leave their checked owner merely to enter
the generic positive-float validation array. This removes redundant validation:
all six fields already prove positivity through their actual admission paths.
The rest of that array and all unrelated range/relationship checks remain.

`apply_posture_action` selects one typed duration for slide, backward dive,
ordinary dive, roll or get-up. The authoritative locomotion sampling rate is
passed explicitly to AnimationSampleSpan::from_positive_duration. The separate
configured get-up producer and the existing slide fixture use the same
operation. Both combat_seconds_to_ticks(f32) and the configuration's duplicate
get_up_sample_span method are removed, with no forwarding compatibility path.

The sample-span owner performs the only primitive multiplication and cast in
this handoff. Its output passes directly through SkeletonState's typed
begin_posture_transition API into PostureTransitionState.duration_ticks.
The existing minimum-one normalization and typed sample-clock progression
remain. Get-up facing also retains its existing checked duration input to
DownedFacingStep. Quickstep's already typed producer paths were inspected;
their action/contact timing and physics remain unchanged.

## Numerical and boundary review

The conversion preserves the exact original expression:

```rust
(duration.seconds() * rate.hertz()).round().max(1.0) as u64
```

Both operands are finite and positive. Their product may underflow to zero,
round to zero, or overflow to positive infinity. Minimum one and Rust's native
saturating integer conversion preserve all three prior outcomes, including
u64::MAX for an overflowing positive product. That endpoint is the actual
bounded sample count, not an unavailable-value sentinel. No new finite-product
rejection, reciprocal calculation, rescaling, ceil or different rounding is
introduced. The duration and sampling-rate owners remain canonical and distinct.

The old validator rejected zero, signed zero, negatives and nonfinite durations;
checked admission rejects the same set. Serialized keys, scalar shape and YAML
defaults are unchanged. Admission diagnostics may now originate at the typed
field's decoding boundary rather than the generic validation array.

A second source pass searched all Rust uses of the four fields, both deleted
helpers and the new constructor. It followed the resulting span into retained
posture state. No naked maneuver-duration field/helper or extraction followed
by re-admission remains on these paths. The remaining clock/count casts occur
inside the sample-span quantization owner, not between application functions.

## Rules and existing validation

The owning Core animation types satisfy crates/AGENTS.md's shared ownership,
producer propagation and native-adapter rules. Tactical Core's authored runtime
parameter guide remains satisfied: no authored value, fallback, tuning constant
or configuration default was added or changed. The sample conversion moved to
the type it constructs instead of a pass-through configuration method.
No numeric ceiling, suppression, dependency, database schema or generated file
changed.

No new unit tests, assertions or fixture helpers were added. The existing slide
fixture now calls the checked API and imports the authoritative sampling-rate
function explicitly. Hosted validation retains the existing canonical config
and shared animation selections, and adds execution of the existing server
player_projection::tests module with required nonempty output. That covers the
existing roll, backward-dive, get-up and slide scenarios and their facing/input
handoffs. The full native and WASM Clippy gates cover other callers.

No local Cargo, formatter, quality scanner, build or test command ran. Hosted
compilation, formatting, semantic/growth checks and existing execution remain
pending. Those results are required before final acceptance; source review
alone does not prove runtime visual or extreme-input acceptance.

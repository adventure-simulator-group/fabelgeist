# P02: signed battle-time migration

This review covers the complete connected coordinate family, including
unchanged receiving code. It does not clear the other migration families.
The single implementation agent performed a producer trace and a second review
against the original scalar arithmetic. Comment Review remains omitted.

## Owner, admission and wire boundary

`adventuresim_core::battle_time::BattleTime` owns finite signed f32 seconds
relative to the autoresolve battle origin. It deliberately admits negative and
signed-zero coordinates; AnimationElapsed continues to own finite nonnegative
intervals. BattleTime is independent of tactical-authority time. Its private
field, checked constructor, TryFrom and checked Serde admission enforce the
same contract. Default and ZERO are the real battle origin, not missing state.
Optional attack starts/contacts retain Option. Serialization stays a scalar
under the existing field names.

The new owner exposes checked addition of AnimationElapsed and checked or
explicitly saturating elapsed differences. Native comparison kernels preserve
the old subtraction-before-tolerance and addition-before-window-comparison
orders, including their outcome if an unused transient coefficient overflows.
No nonfinite coordinate is retained. ContactTimingOffset continues to own the
signed own-contact-relative-to-incoming quantity; between_contacts now accepts
BattleTime inputs and preserves its ContactOffsetOverflow classification.

## Full connected trace

The round/count-to-time producers in joint_melee and melee_round/scheduling
admit the completed native count multiplication once. They retain BattleTime
through per-step windows, recursive remainders, minimum-contact selection,
simultaneous batches, both-side scheduling and movement interval starts.
Initialization interprets its already checked delay relative to the battle
origin without a float round trip. Combatant readiness, optional attack start
and optional contact, ScheduledMeleeTiming, occupied recovery and AttackStartTrace
all keep the coordinate owner.

Available attack start accepts and returns coordinates. Movement records add
elapsed before publishing either movement state or timeline time. Scheduling
prepares contact, recovery and jitter before publishing the affected engagement
and attack fields. Defender commitment, recovery, adaptation and reciprocal
interception receive typed coordinates. The reflex comparison first proves a
positive finite interval within the configured window; only then is that span
used in its numerical ratio.

Response state and telemetry use one prepared DefenderReadiness record. The
old duplicated after-readiness calculation is removed. A deadline failure
cannot publish that replacement or its timeline. The offhand-power update can
occur earlier only on the existing transformed branch whose added recovery is
zero, so its coordinate addition cannot overflow. No global transaction
property is claimed.

The actual simultaneous-contact evidence producer is also migrated. Timeline
records retain typed time/readiness through deterministic evidence and report
serialization. Their record and enum owners now live in autoresolve/timeline,
with ordinary canonical re-exports. Tick quantization is the actual native
adapter: max-zero, multiply by the existing 64 Hz, round, then saturating u64
cast. The serde scalar adapter is the other terminal coordinate projection.
The tactical causal consumer was inspected and does not read absolute timeline
coordinates; it needs no untyped forwarding adapter.

## Behavior checks and deliberate invariant repair

The second source review followed all changed signatures and every construction
of ScheduledMeleeTiming and MeleeTimelineEvent. It checked the existing -0.1
start, zero-duration windows, actual absence, same-time tie membership and
ordering, reflex-window boundaries, recursion remainder and original timestamp
rounding. Round-start multiplication, step scaling, addition and duration
subtraction preserve their original f32 order. Terminal fallback admission is
lazy, so an unused alternative cannot reject a real recorded terminal time.

An overflowing owned coordinate or positive elapsed difference now returns a
structured failure. In particular, a guard-recovery difference between extreme
opposite signed coordinates can no longer overflow to infinity and silently
produce zero effectiveness. This is a deliberate finite-value invariant
repair. Existing ordinary successful arithmetic, valid negatives, real zero,
clock separation, probability calculations and wire shapes remain unchanged.
Existing span admission errors otherwise preserve their owning ElapsedError.

Round enumeration, physical distances, reaction probabilities, fatigue and
combat IDs retain their separate existing responsibilities. No same-quantity
coordinate forwarding was excluded as a private helper or JSON-only record.
Bespoke coordinate types alone do not prove a matching battle instance or an
ordered aggregate schedule; this increment makes no such claim.

## Governing rules and validation

Core and crates guidance require canonical ownership, typed producers and
retained fields, explicit absence, checked derived operations and clean APIs.
The new shared owner and the migrated signatures satisfy that scoped contract.
The oversized autoresolve facade delegates the timeline responsibility rather
than accumulating another clock/record implementation. No database binding,
compatibility shim, numeric ceiling, threshold, new unit test, assertion or
fixture helper was introduced.

Existing test inputs and comparisons were adapted at their literal fixture
boundaries. Hosted CI now executes the existing Core autoresolve and defense
test selections and requires nonempty passing output. The existing all-target
native Clippy and WASM checks exercise downstream compilation. No local Cargo,
formatter, parser, quality scanner, build or test command has run.

Source completeness is established for this coordinate path. Final acceptance
is pending hosted formatting, compilation, semantic/growth gates and existing
test execution on its published revision. A green early gate will not be
reported as a whole-workflow PASS. Live tactical renderer acceptance and
unrelated semantic families remain outside this narrow source-review verdict.

# Next family: signed autoresolve battle-time coordinates

Source inspected at `347ee81578781d71aeedc50b897bfb2e1866daba`.
No P02 source changes have been made. This inventory refines the existing plan;
it does not narrow or replace its exit requirement.

## Assigned source and data flow

- `crates/adventuresim-core/src/autoresolve.rs`: Combatant readiness and the
  optional attack-start/contact fields; constructor defaults; MeleeTimelineEvent
  time/readiness fields; `at`, `tick_at` and terminal-coordinate construction.
  The embedded existing tests include a valid -0.1-second attack start.
- `autoresolve/joint_melee.rs`: round origin, per-step window start/end,
  earliest-contact selection, simultaneous-contact tolerance, recursive window
  remainder, ordering, attack identity, both-side scheduling and movement start.
- `autoresolve/melee_round/scheduling.rs`: AttackStartTrace readiness/window,
  initiative delay publication, round and window producers, pending scheduling,
  configured duration addition, contact filtering and ScheduledMeleeTiming reuse.
- `autoresolve/melee_round/mobility.rs`: available_attack_start inputs/result,
  movement interval start, movement time, phase selection and timeline producer.
- `autoresolve/melee_defense.rs`: ScheduledMeleeTiming fields, occupied recovery,
  intercept-window comparison, response delay, own windup, recovery guard and
  timing commit. The existing reaction sample and displacement elapsed are
  separate quantities and must retain their existing roles.
- `autoresolve/melee_round/commitment.rs`: adaptation gap, phase/availability
  comparisons and reconstruction of the defender's retained schedule.
- `autoresolve/melee_round/mod.rs`: contact/response timeline production,
  readiness-before/after and defense-recovery addition. Compute any fallible
  replacement before publishing its affected state and avoid adding a second
  independently recomputed readiness authority.
- `combat/contact_timing.rs`: between_contacts must accept typed coordinates;
  ContactTimingOffset remains the signed own-contact-relative-to-incoming role.
- `autoresolve/melee_iteration/evidence/timeline.rs`: the real simultaneous
  contact evidence producer currently builds raw schedule coordinates.
- `autoresolve/melee_iteration.rs`: existing bounded-progress assertions and
  timeline ordering consume typed values, including serialization evidence.
- `autoresolve/error.rs` and the Core module export: expose the canonical
  coordinate owner and propagate any new coordinate admission/overflow error.
- Existing tests in `autoresolve.rs` and `melee_round/mod.rs` need checked
  fixture admission and result comparisons, preserving their assertions.
- `adventuresim-tactical-server/src/bin/melee_combat_iteration/causal.rs`
  consumes timeline events but currently reads no absolute time/readiness
  coordinate. Inspect it again after the migration rather than adding needless
  primitive adapters.

All relative paths above are beneath `crates/adventuresim-core/src/` except
where explicitly qualified. No matching BattleTime/BattleInstant owner was
found in current handwritten Rust. No database binding owns this transient
pure-resolution clock.

## Implementation contract

Use one Core-owned finite signed battle-time coordinate, separate from elapsed
and tactical-authority clocks. Its zero is the battle origin; signed zero and
negative authored coordinates remain valid. Checked addition accepts
AnimationElapsed. Nonnegative differences use checked or explicitly saturating
elapsed operations; contact-relative signed differences retain the existing
ContactTimingOffset. Primitive projection belongs only to the actual ordering,
ratio, serialization and timestamp-to-tick numerical adapters.

Do not accept arbitrary floats at scheduling/defense/movement/timeline APIs or
re-admit a coordinate in the next helper. Preserve f32 multiplication/addition
order, round origin, step rounding and saturation, simultaneous tolerance and
native tick conversion. Do not convert this clock into Duration, and do not
reuse a nonnegative elapsed wrapper for a signed coordinate. Check overflow
before replacing the affected schedule; do not add infinity or -1 sentinels.
Any comparison that used a transient overflowing difference must retain its
original ordering outcome or document a deliberate invariant repair.

Keep the same scalar JSON shape through checked serialization. Do not add
compatibility APIs or duplicate the ContactTimingOffset concept. If response
readiness is prepared once, pass the same typed result to state and timeline.

## Existing validation and acceptance

Use hosted CI only. Run the existing Core autoresolve and defense selections:

```sh
cargo test --locked --package adventuresim-core --lib autoresolve:: -- --test-threads=1
cargo test --locked --package adventuresim-core --lib combat::defense:: -- --test-threads=1
```

The workflow's existing native/all-target and WASM Clippy gates cover downstream
consumers. Add only executions of existing relevant tests, with nonempty-result
verification; do not add cases, assertions or fixture helpers. Run the existing
semantic-quality gate and accept only relevant debt reductions from its hosted
report. No numeric ceiling may increase.

Inspect deterministic battle/timeline tests, negative-start fixtures,
simultaneous contact, window subdivision, readiness, reciprocal interception,
phase adaptation and cancellation. Source review must trace all fields and
constructors again after edits; compile success alone is not completeness.
The broader live tactical production acceptance remains distinct from this
pure-resolution coordinate migration and from green unit tests.

# Audio, jitter and angular-cap migration review

Reviewed against `8db8d83f1d0bde61db1dba37c1bdee79bd934c04`.
This is a source review by the implementation agent, checked twice against
both the original implementation and all identified callers. It is not an
independent reviewer verdict or a hosted validation pass. Comment Review was
omitted as requested.

## P01: movement-audio seeds

The owner is `fabelgeist_determinism::Seed`. Both
`play_locomotion_audio` and `play_grounded_dive_impacts` now keep the seed
returned by `StreamId::seed`. Contact footsteps, grass rustle, understory
rustle and landing impacts pass that owner through `spawn_spatial_variant`
or `spawn_rustle`, then `sound_seed` and `spawn_spatial_sound`. Understory
keeps its separate stream derivation, now taking the seed directly.

The character ID is deliberately admitted as root seed material. Contact and
landing sequence words, the authoritative sample tick and the three position
bit patterns remain numeric context at the seed derivation adapter. Each
stream label, field order, eight-byte framing and draw count is unchanged.
The old `to_u64` followed by `From<u64>` was an identity on Seed's sole word;
its removal changes no sampled bits. No internal audio helper retains or
accepts the derived seed as `u64`.

A second source pass checked every helper call, including both understory and
grounded-dive producers. The only remaining `.into()` admissions in production
are the two character-ID seed derivations. The existing movement-audio test is
now included in hosted validation and in the nonempty-result verifier.

## P05: capture elapsed through jitter

`PlannedFrame` and `FrameSample` already own `AnimationElapsed`.
`capture::jitter_frames` now copies it directly to `JitterFrame`.
The only other construction is the existing numeric fixture builder, adapted
to the checked field without adding a test or assertion.

`AnimationElapsed::abs_diff` owns finite nonnegative subtraction. Its result
cannot exceed the larger finite operand. `dt` returns the same owner and uses
typed constants for the existing one-millisecond floor and 1/64-second
missing-frame fallback. These constants move existing diagnostic policy into
checked values; they introduce no new tuning or fallback. Angular, scalar and
vector derivative divisions are the only sites projecting interval seconds.

A second source pass checked every JitterFrame construction and consumer.
The operation remains subtract, absolute value, floor, then divide, in f32.
Sample offsets, scenario boundaries, contact segmentation, warmup and all
measurement thresholds remain unchanged. Missing endpoints and reversed or
identical sample times retain their prior behavior. Finite nonnegative
observations cannot produce the nonfinite subtraction that would invalidate
the checked result.

## P07: knee-cap angle handoff

The canonical `AngularMagnitudeDegrees` is already admitted by the combat
configuration. Both `stabilized_knee_pole` and
`constrain_rendered_leg_pole` now pass it directly to
`constrain_knee_pole_to_foot_facing`. That helper projects radians directly
before the atan2/clamp/sin_cos kernel. It does not introduce a new angle type.

The previous BodyAngularMagnitude admission was redundant: a finite
nonnegative degree magnitude times the degree-to-radian factor is finite and
nonnegative, including signed zero. The conversion order is unchanged. The
three existing helper tests now author the same 22.5-degree cap at their
fixture boundary; no tests or assertions were added. A second source pass
checked all five production/fixture calls and found no remaining intermediate
angle reconstruction in this family.

## Governing rules and correctness review

`crates/AGENTS.md` requires preserving checked values across producer fields,
returns and parameters and reusing canonical owners. These changes close those
specific paths. `AnimationElapsed` operations remain in its Core owner;
configuration angle admission and Seed derivation remain with their existing
owners. No database boundary, wire schema, compatibility alias, new error
family, threshold suppression or numeric ceiling was introduced.

Hosted run 39376148571 reported exactly three stale debt records: the
orientation file, collect_loaded_packs and the old player::try_start_attack.
The original source confirms orientation is below the file limit, the loader
has a short delegated motion-admission loop, and attack admission has moved to
player/attack_start.rs. Only those three obsolete records are removed. The
remaining orientation function record and all other ceilings are preserved.
The hosted scanner, rather than a local parser approximation, will verify this.

No actionable correctness or rule finding remains in this narrow source
review. Migration structure is closed for these three items, but their final
acceptance remains unverified until hosted checks execute on the published
head. This statement does not clear the remaining seventeen plan items.

## Existing checks and coverage limits

Only static source inspection, Git diff/whitespace checks and source metadata
were used locally. No local project commands or new tests were run or added.
Hosted validation will execute the existing animation tests (including jitter
and all three knee-cap tests), the existing movement-audio test, native/WASM
Clippy and the repository semantic-quality gate. The full workflow also runs
its existing shared rig, MHR, creator, equipment and configuration checks.

The prior hosted run passed formatting, generated-binding validation and the
early schema/FBX tests, then stopped at stale semantic debt. Those results do
not validate this newer revision. Visual/native production-path acceptance
is not established by these source checks or by hosted unit tests.

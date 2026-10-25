# P04 source review: playback policy and event counts

Scope: the complete connected P04 change against f9070a1b1. All twenty
migration-plan items remain in scope for the active Goal. This increment has
source coverage; it is not finally accepted until exact-head hosted checks run.
These are two source passes by the sole implementation agent, not fresh
independent Code/Rule approval. Comment Review is omitted as requested.

## Contract trace

- `combat_config/playback.rs` admits velocity response as existing ResponseRate.
  `animation/response_parameters.rs::exponential_fraction` owns the complete
  native exponential before UnitInterval admission. Both consumers were read:
  presentation gait prediction projects the fraction into Vec3 interpolation;
  procedural/body_response.rs::smooth_acceleration projects it into the native
  body-local acceleration interpolation. Zero remains valid. Finite operands
  that produce an infinite negative exponent now reach the valid full-response
  result. This removes an over-strict intermediate rejection rather than
  adding a competing calculation path.
- `animation/phase_response.rs` admits finite nonnegative correction rates and
  phase-error limits through checked TryFrom and derived scalar Serde. Limits
  above one remain valid. Gait and attack prediction consume these policies in
  their complete subtraction/deadband/clamp kernels. CycleDelta and
  ActionProgressDelta remain the retained displacement owners. The raw scalar
  presentation getter helpers were removed. An infinite transient correction
  budget can still clamp a finite error without becoming retained state.
- `animation/cadence_limit.rs` owns StepCadence, CadenceDemand,
  CadenceSelection and AuthoredCadenceCap. The producer takes checked
  NonnegativeSpeed and NonnegativeLength, with existing PositiveLength for the
  authored distance floor. Demand beyond f32 is an explicit enum variant and
  is immediately capped by the checked policy. Applied rate stays typed into
  cycle-delta arithmetic and retained CyclePhase. Native scalar projections
  occur inside the demand/polynomial kernels and at warning presentation.
- The cadence policy's three private fields are admitted together. A private
  deny-unknown-fields decoder goes through the same checked constructor as
  application construction. Public replacement can replace only a complete
  valid policy. Authored bits, including valid zero rates, are preserved;
  there is no reciprocal requirement or compatibility decoder.
- `presentation/cadence.rs` owns client cadence application and phase state.
  The prior independent limited/cadence_capped flags are replaced by the
  three meaningful states: unrestricted, stance-slip limited, and cadence
  capped. A cap always denotes a limit. Mode switching, stationary retention,
  warning transitions, multiply-before-halving and phase wrapping are intact.
  All five existing cadence tests moved with this responsibility; only their
  typed API access changed. The threshold, cases and assertion count remain.
- `animation/event_span.rs` owns full-width AnimationEventSpan and positive
  CoalescedEventBudget. ContactSequence and LandingSequence produce the span
  at native subtraction, with checked and wrapping observations kept separate.
  A backwards checked counter still yields None; unchanged yields Some(zero).
  Contact coalescing carries typed offsets through reverse iteration, parity
  and preceding contact lookup. Budget comparisons stay typed in event
  coalescing and height landing detection. IK observation and report continuity
  compare typed spans. Sequence encodings used for seeds and report/log output
  remain actual native endpoints. No u64 budget pass-through helper remains.

## Material behavior and schema

The old scalar pair maximum=0 and transition width=1 admitted a smoothing
interval beginning below zero. At requested cadence 0.1, the old formula
produces about -0.0512 steps/second. This is a source-derived counterexample,
not a locally executed test. The new aggregate rejects half-width greater
than maximum, preventing backwards phase from forward demand. Width zero and
zero maximum with zero width remain valid. The default 5/1 C2 polynomial and
all numeric operation ordering remain unchanged.

YAML now groups `maximum_per_second`, `transition_width_per_second` and
`minimum_step_distance_metres` under `animation.playback.authored_step_cadence`.
Their defaults are 5.0, 1.0 and 0.01. The existing 0.01-metre native floor has
moved into the authored policy. Default construction decodes the same embedded
YAML, so there is no independent copied Rust default. The final schema and its
JSON-derived digest change deliberately; old field names have no fallback.
All other numerical defaults and scalar encodings remain intact.

The warning now prints CadenceDemand explicitly, including
ExceedsScalarRange. A repository search found no programmatic parser of that
warning's requested/applied cadence fields; there is no hidden text branch.

## Code and rule checks

Pass one followed owners, admission, operations and every identified consumer.
Pass two checked concrete bypass/edge cases: private-field mutation, Serde
admission, zero/nonfinite input, cap-width relationships, overflowed demand,
bounded exponent/correction overflow, zero-event intervals, backwards counters,
u64 wrapping, event iteration order and candidate publication. No remaining
P04 producer-propagation gap was found. This is static source evidence; it does
not establish exercised boundary behavior.

Governing sources are the root AGENTS.md, crates/AGENTS.md semantic-value,
canonical-owner, private-invariant and module rules; Tactical Core AGENTS.md
and client animation/AGENTS.md place runtime tuning in canonical combat YAML.
The cadence responsibility and its existing tests were extracted before
expanding the presentation facade. No compatibility path, local tuning
constant, lint suppression, quality exemption or raised ceiling was added.
The original stash and four unrelated primary Cargo edits are untouched.

## Validation and limits

Git whitespace checking passed before this report. Existing test names and
assertion counts were checked as source metadata; no new unit tests, cases,
assertions, fixtures or fixture helpers were authored. The workflow now runs
existing animation_viewer::report:: tests and requires a nonempty successful
result, in addition to existing animation, Core/configuration, startup,
maneuver and full lint gates. No gate was removed or weakened.

No local Cargo, formatter, quality scanner, parser, generator or runtime
command was run. Hosted format/compiler/lint and behavioral results are
pending for P04. The new invalid aggregate rejection and extreme bounded
response/demand cases lack dedicated executed cases; user instructions forbid
adding tests. Native renderer/live-play acceptance is not claimed. Other plan
families remain incomplete or unverified as tracked in execution-ledger.json.

## Hosted follow-up on 82528917d

The source change is committed at c59c01e3e and its exact hosted formatting at
82528917d. Run 39430479607/job 118653923352 has passed formatting, generated
bindings, semantic/growth, calendar ownership and WASM Clippy. This confirms
the new gameplay-client source compiles for WASM. Native/standalone Clippy,
existing behavioral selections and official baseline export remain pending.
No full-matrix, new aggregate-edge or live-renderer acceptance is claimed.

## Native all-target fixture repair

Run 39430479607 reached native all-target Clippy after passing all earlier
gates, including deterministic native/WASM draws. Rust rejected the existing
Walker::attempt_walk_to fixture because its route-tick equation divided a raw
distance by NonnegativeSpeed. The configured speed is already the correct
owner; the division is the actual native numerical endpoint. Commit 11d956e9f
projects metres per second there. The formula, ceiling/saturating usize count,
minimum route window, cases and assertions are unchanged. This is an existing
fixture adaptation; it does not implement or accept P06. No new tests or local
checks were run. Run 39432148548/job 118659647422 validates the new exact head.

Hosted formatting follow-up: the first native-fixture repair run stopped at
one line-layout request. Commit 374977162 applies its exact output. The
current live run is 39432660084/job 118661423273 on that head. The earlier
native-failure record remains historical evidence; no full PASS is claimed.

## Posture-facing native import repair

Revision 374977162 passed formatting, bindings, semantic/growth, WASM lint and
deterministic native/WASM draws. Native all-target checking then reported five
uses of a missing AnimationValueResult in the extracted posture-facing module.
Commit 6c31bf753 imports the canonical Tactical Core alias. The complete module
and all five errors were read, then the one-import diff and owning alias were
checked. No signature, error contract, operation, case or assertion changed.
This bounded self review is not independent full-PR approval. Current validation
is run 39434297927/job 118667133316; full acceptance is still pending.

The latest observation on 6c31bf753 confirms formatting, bindings,
semantic/growth, calendar ownership, WASM Clippy and deterministic native/WASM
draws passed. The native all-target/standalone Clippy step is still live in
run 39434297927/job 118667133316. Existing rig/animation/configuration behavior
and official baseline export have not executed yet. No P04 or Goal completion
is claimed. See posture-import-hosted-6c31.json for the scoped gate snapshot.

## Native server-defense control flow

The 6c31bf753 run got past the five posture-facing import errors. Its next
native gate failure was three control-flow lints: two nested reaction handlers
and a block inside the active-defense match input. Commit 13bbd34d1 uses the
suggested flat conditions and a named response-preparation result. Source
review verified the same query/check/call order, RNG/command behavior, error
logging and lexical authority-query scope. No operation, contract, test,
suppression or ceiling changed. Current hosted validation is run 39439132880,
job 118684016902; formatting passed and remaining acceptance is pending.

## Lunge import scope in the server binary

After the three server-defense lint fixes, run 39439132880 reached the server
binary and reported unused facade exports. All server consumers were searched:
collision clearance is used by the existing ingress fixture, and target
reachability is used only inside its lunge owner. Commit 2ab9bb7b5 narrows those
imports without removing functions or cases. Commit 4c8c559fe applies the exact
hosted grouped-import ordering. Formatting and bindings pass on the latter;
current run 39441499527/job 118692243316 is live. Full native/behavioral and
baseline acceptance remain pending. No new tests or suppressions were added.

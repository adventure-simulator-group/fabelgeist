# Bespoke type refactor: progress

Updated: 2026-11-07T11:37:20.934447+00:00

Latest verified checkpoint: **3275**. Goal: **active**.

## Quantitative summary

| Category | Done or recorded | Remaining or status |
| --- | --- | --- |
| Original inventory roles, lenses 11â€“13 | 8,559 / 8,559 accounted | 0 original roles |
| Registered revision-delta lens/path checks | 3,172 / 3,172 closed | 0 in these registered deltas |
| Queue entries explicitly marked source-discovery closed | 574 / 3,854 marked | 3,280 other entries |
| Registered source ranges qualified, connected work open | 295 entries | Connected contracts remain open |
| Historical unread entries with recorded range evidence | 76 / 189 entries | 113 without these markers |
| Entries with explicit repair status | 441 recorded | Implementation gate not yet passed |
| Scoped repair obligations on entries | 75 recorded | 3 conditional; 10 policy/design |

These categories overlap and have different meanings. Original roles and
registered delta checks are not findings or completed repairs. Queue entries
include reminders and independent obligations; the other entries are not a count
of unique unfinished bugs. The source-closed count includes only explicit flags;
qualified-open entries use a separate status and are not completed contracts.
Historical range markers indicate evidence on an entry, not closure; other
historical entries may already have evidence in different forms. Scoped repair
obligations are counted separately and may overlap other entries. The full
audit, implementation, review and validation remain incomplete.

The refactor remains in the required source audit. Its 13 perspectives include
reuse of existing types and preservation of semantic values through native
boundaries. Completing a scoped question or correcting an old status note does
not complete the audit or authorize implementation. Checkpoint 3130 records an
evidence-integrity correction for recent work; adjacent checkpoints still
require review. Six P02 note versions were recovered by exact hash; two remain
unresolved. A bounded scan checked 489 nearby decision evidence references;
seven mismatches all concern the same note. The reducer adapter scope has been
reread after its registration; related contract work remains open. Web and
simulation error consumers have been reread after registration; runtime delivery
remains unverified. Strategic reducer error propagation has been reread after
registration; runtime delivery remains unverified. P02 mobility and timeline
scopes were reread after registration; the connected contract remains open. P02
scheduling was reread after registration; initialization failure behavior
remains open. The initialization caller owns its vectors and returns no outcome
on failure; other connected obligations remain open. The battle clock and joint
window scheduler were reread; contact batching policy remains an open question.
Existing test source does not explicitly settle near-equal contact batching.
Contact batch membership affects a defeated-attacker condition as well as
recorded telemetry; intended near-equal policy remains open. Authored initiative
delay varies; a supported near-equal contact witness and policy remain open.
Normal joint scheduling fixes new starts at its window endpoint; surprise-
opening timing remains to be traced. Surprise opening can retain out-of-window
timing in principle; a supported near-equal witness remains unproven. Retained
contact coordinates feed joint windows; supported near-equal reachability and
cancellation remain open. Same-weapon defensive cancellation clears retained
timing; supported near-equal reachability remains open. Scene-assets has a
browser generation consumer; independent tactical-server delivery remains open.
Dispatcher request minutes feed a separately materialized tactical-server scene;
their producer remains open. Tactical requests use a wilderness clock;
standalone bootstrap has a separate world-start fallback. Standalone bootstrap
initializes its local clock before the fallback; other clock producers remain
open. Wilderness elapsed updates saturate and project within the anchor day;
supported boundary reachability remains open. Inspected journey transitions
retain the clock outside settlements and clear it on settlement arrival.
Official clock conversion clamps negative elapsed and uses saturating
arithmetic; native extremes remain open. The native timestamp accessor returns
signed microseconds directly; runtime range and clock policy remain open. Scene
and displayed clocks handle missing or unrepresentable time differently;
supported failure policy remains open. Cloud seeds remain typed through
sampling; other generators and coordinate admission remain open. Cloud ray and
layer inputs are constrained; endpoint and wind admission remain open. Cloud
endpoints advance per completed bake; wind admission and supported long-session
behavior remain open. Validated scene wind speed is bounded; complete
environment-producer coverage remains open. Tactical startup validates scene
weather before snapshot; browser and alternate environment paths remain open.
Strategic retained scene parsing validates weather; pre-preparation and
alternate environment paths remain open. Standard browser pre-preparation
validates scene input before venue discovery; alternate entry paths remain open.
Tactical scene environments are inserted on replicated entities; client
admission and alternate producers remain open. Inspected viewer and demo
environment paths have bounded fixture wind or validate their input; other
producers remain open. Generated weather caps wind speed and normalizes
direction; other producers remain open. Strategic weather uses typed seeds and
fixed-point lattice coordinates; supported extreme arithmetic remains open. The
official clock is below the weather lattice cap; other clock producers remain
open. Journey elapsed time wraps within its stored anchor day; anchor provenance
and other weather callers remain open. Four more weather callers have distinct
time and coordinate paths; their upstream bounds remain open. Route planning and
reducer validation use distinct clock sources; divergence and intended behavior
remain open. A selected departure can differ from personal time and trigger a
stale route rejection; end-to-end admission remains to be confirmed. The planned
case-site payload carries personal-clock weather to the reducer, which can
reject it against the wilderness departure interval. Member approval reaches the
same planned route clock boundary; coordinate-frame questions remain separate.
Member approval reaches the same planned route clock boundary; coordinate-frame
questions remain separate. Forecast starting times come from independent
personal clocks; supported extreme bounds remain open. Newborn and encounter
character clocks derive from parent and encounter times; those producer bounds
remain open. Inspected road-cast clocks initialize from official time; newborn
chronology and mortality bounds remain open. Conception follows personal leisure
time and age display saturates; independent mortality bounds remain open. A
concrete representable weather input underflows the signed lattice; supported
gameplay reachability remains open. The lattice witness matches a byte-identical
frozen Lens 13 operation; independent qualification remains open. Lens 13
independently reread the matching current weather lattice; native admission
remains open. Lens 13 native weather paths use distinct official, personal,
wilderness and dispatcher clocks; admission remains open. Tactical request
weather time is bounded for officially initialized wilderness anchors; stored
provenance and personal clocks remain open. Current production anchor writes
initialize from official time or clear the anchor; personal-clock admission
remains open. Road-cast overrides use official time and one social action has an
eight-hour limit; cumulative personal time remains open. Personal and official
clocks have separate documented authority; extreme supported time policy remains
open. Full typed-input lattice indices fit i64 after wider intermediate
arithmetic; late-time rules behavior remains open. Weather rules version affects
route cache identity and route and tactical snapshot admission. Raw minimum
latitude can overflow cell reconstruction; supported coordinate admission
remains open. Inspected native abstract E7 conversions cannot reach the raw
i32-min latitude witness; other consumers remain open. Forecast endpoints are
geographic, but interpolation depends on itinerary movement invariants.
Generated itinerary segments constrain movement offsets; extreme-time and cast
conditions remain open. Itinerary forecasting can fail to advance at
StrategicMinute MAX; supported native reachability remains open. On-site
readiness passes personal time directly to itinerary forecasting; supported
clock exhaustion remains open. Correction: the full-range lattice bound includes
one-cell pressure offsets and still fits i128. Returned generated itineraries
accepted by the thermal horizon keep movement offsets and geographic
interpolation bounded. Thermal forecasts consume survival state and thresholds
while frostbite event consequences remain to be assessed. Frostbite events
require colder strain than the thermal unsafe threshold; safe forecasts cannot
contain an ignored event. Inspected readiness decisions combine thermal and
other safety checks and treat missing return projections as unsafe. Thermal
forecasts and wilderness exposure use different weather clocks; reachable
readiness impact remains open. Forecast weather clock differences feed seasonal
and lattice inputs; actual readiness impact remains open. A source-level
overnight itinerary yields different forecast and wilderness dates; admission
and readiness impact remain open. Camp rest samples wrapped wilderness weather
before advancing party elapsed; complete route impact remains open. Generated
site distances can exceed an eight-hour forecast leg; a concrete admitted route
remains open. Safe-departure selection configures journey-local time without
advancing personal time in the inspected method. Planned party movement lacks
the exposure call present in ordinary travel; outer admission remains to be
confirmed. Three party travel entry paths use disease-plan movement without the
ordinary exposure call; end-to-end validation remains pending. Planned movement
exposure repair must preserve shared disease-plan clipping and lifecycle
ordering. Movement exposure repair must preserve route and wilderness-time
cursors across chunks and settlement departure. Lens 12 source qualification
connects planned movement weather exposure to an existing open question; aliases
remain open. Lens 12 source qualification connects planned movement weather
exposure to an existing open question; aliases remain open. Three journey entry
paths use the planned party movement seam after encounter clipping; related
aliases remain open. Direct rest and stationary exposure calls cover their own
intervals; movement terminal ordering remains open. Weather exposure persists
thermal state and frostbite limb damage; movement terminal policy remains open.
Core exposure is minute-stepped; case-site recovery cause presentation has an
unverified thermal omission. A representable thermal-only incapacitation would
yield generic case-site recovery text if the shown notice path is reached; live
reachability remains open. The case-site portrait wheel source includes
temperature, narrowing the separate recovery wording question; live rendering
remains unverified. The movement exposure partial plan was scoped without
discarding independent battle-time and forecast obligations. Signed elevation
admission matches its historical owner at the current pin; related aliases
remain open. Signed elevation admission matches its historical owner at the
current pin; related aliases remain open. Compiled-default diagnostic source is
reconciled; dynamic configuration error policy remains separate. Compiled-
default diagnostic source is reconciled; dynamic configuration error policy
remains separate. Dynamic configuration loader and receiver paths are partially
traced; snapshot producer and outcome policy remain open. Server join and
reconnect publish the loaded configuration snapshot; delivery and rejection
consequences remain open. Checkpoint 3219 incorrectly closed the dynamic
configuration entry; checkpoint 3220 restores it to open and retains the prior
record. Checkpoint 3219 incorrectly closed the dynamic configuration entry;
checkpoint 3220 restored it to open and retained the prior record. Client
animation and server defense consume their respective config states; rejected-
snapshot policy remains open. A validated config digest has a distinct
serialization error; supported failure reachability remains open. The JSON
serializer maps nonfinite scalar floats to null; complete config admission and
serialization semantics remain open. A public AI facing threshold lacks a top-
level finite check; file admission and intended range remain open. YAML float
parsing admits nonfinite AI facing thresholds in the source-level path;
threshold policy and runtime effects remain open. UnitInterval is a possible AI
threshold type; finite out-of-range policy remains open. AI precision admission
is already owned separately and intentionally accepts every finite value. The AI
facing-threshold gap now has its own open entry; its numeric-lens qualification
remains separate. The AI facing-threshold gap now has its own open entry; its
numeric-lens qualification remains separate. AI facing admission now has
separate Lens 12 and Lens 13 source evidence; finite policy remains open. AI
facing admission has independent type, string and numeric lens evidence; finite
policy remains open. Ratio is unchecked; UnitInterval adds bounded admission, so
the threshold policy must decide the appropriate owner. Two historical range
markers now reference existing qualified reads; connected work remains open.
Native audio existing-type ownership is partially qualified under Lens 2;
project producers and other lenses remain open. Audio gain and pitch admission
is traced into selected native playback settings; other consumers remain open.
Native audio forwards composed playback speed; the existing pitch-product repair
remains independently owned. Checked UnitInterval is a reuse candidate for
bounded audio gains; native volume and pitch retain separate semantics. Combat
one-shots and rain-loop gain updates have been traced; remaining movement and
lifecycle scope is open. Movement and wind playback and the registered native
Volume tail have been read; alias and independent lens work remain open. Native
sink lifecycle is traced; the relationship between global volume and subsequent
ambience updates remains open. Native and browser startup validate audio config;
Bevy Pitch is a sine-wave source and global volume defaults to unity. Audio
decision templates contained unrelated inherited labels; checkpoint 3241
corrects the current record while preserving history. Physical areal density
does not own audio playback speed; checked pitch composition remains separately
tracked. Audio config errors, visual gating, pitch composition and scene
selection have distinct existing owners. Pinned rodio stores playback speed
without checking and applies it to a speed wrapper; remaining registered scopes
are open. The registered rodio player and spatial-player source ranges have been
read under Lens 2; independent contracts remain open. Procedural anticipation
source matches the prior assessed bytes at the current pin; exact aliases remain
open. UnitInterval lacks a fixed-step method; the anticipation repair must
preserve exact step arithmetic. The registered current procedural animation file
is read under Lens 2; its helper test does not cover the full anticipation
system. Jump anticipation is ordered between body response and later pose
refinements and fixed-tick stabilization. Jump charging admission is restricted
to grounded upright state without a posture transition. Strategic catalog
projection retains the ranged flag but has no projectile-kind field. Selected
bow and firearm catalog records lack explicit projectile or ammunition
compatibility. Tactical ranged attacks select arrows and record aggregate party
ammunition use; receipt representation remains open. Ranged contact loses
projectile identity before shared body-part injury aggregation; receipt-only
repair is insufficient. Tactical item projection carries the ranged flag and ID
but no projectile or ammunition compatibility. Tactical receipt validation
bounds records and checks party authority before strategic injury commitment.
The missing projectile-kind receipt question overlaps one existing Lens 12
entry; its wear and repair obligations remain distinct. Tactical injury
accumulation is limited per hit by current limb health; aggregate receipt
reachability remains unproven. Tactical projection copies snapshot limb health
without a local clamp; enemy scaling alone does not establish party receipt
reachability. Ordinary strategic limb creation starts at one and injury
projection clamps; other writers and finite admission remain open. Terminal
receipts are frozen before submission; projectile multiplicity needs an explicit
bounded policy. Strategic ranged combat counts and consumes arrows independently
of weapon kind; containment reachability remains conditional. Tactical receipt
refreshes capability before equipment wear; later lazy refresh and visible
effects remain to be traced. Web refresh errors are ignored before a stored
capability read; server readiness also reads stored combat power. Combat power
uses wear-sensitive precision and resistance; the located readiness caller
refreshes before reading. The registered settlement capability-consumer file is
read; unrelated displays use separate data paths. Condition loss can affect
effective equipment stats; a measurable post-receipt stale value remains
unproven. Live subscriptions invalidate on item-condition and capability
changes; render and refresh timing remain open. Melee contact force feeds wear;
a supported above-threshold contact and visible stale value remain unproven. An
authored longsword threshold and synthetic contact example are recorded; an end-
to-end contact remains unproven. Tactical contact aggregation and per-exchange
autoresolve can differ under the nonlinear wear formula; intended policy and
reachability remain open. Receipt participant truncation can conflict with
retained contact owners; supported overflow reachability remains open. Captured
tactical rosters share the 16-member receipt bound; temporary and tactical-side
provenance remain open. Party-side projection feeds routine and combat
consequences; strategic side provenance remains open. A temporary enemy creator
assigns characters without adding them to the captured enemy roster; invocation
and receipt reachability remain open.

| Phase | Status |
| --- | --- |
| Required audit | In progress |
| Product implementation | Not started |
| Independent review | Not started |
| Product and runtime validation | Deferred until audit completion; earlier prohibited probes are historical evidence only |

## Latest checkpoint

Traced temporary character creation to tactical assignment.

Other consumers and aliases remain open; no runtime execution.

## Working on now

Trace temporary-character invocation and projection into receipts.

- Finish capture rotation, asset scale, report serialization and native boundary
  reconciliation.

- Resolve supported spring tuning and telemetry failure semantics, retaining
  independent numeric-lens duties.

- Reconcile MHR skin admission, shared zero-influence policy and corrective
  index conversion.

- Complete all 13 perspectives, 20 migration families and connected obligations
  before implementation.

- Implement the final types and complete independent review and validation.

## Work completed since the handoff

- Corrected an audit-evidence filename collision: checkpoints 2160 and 2161
  referred to different hashes at one local qualification path. Both exact
  original byte sequences were reconstructed and hash-verified under distinct
  RECOVERED-QUALIFICATION files; the shared path holds the latest historical
  version. Earlier checkpoint references retain the collision and require this
  correction when inspected.

- Traced capture rig scales and native quaternion decomposition; isolated probes
  distinguish degenerate and sheared transforms from observed supported assets.

- Confirmed isolated extreme spring tuning can yield nonfinite state and mask
  its telemetry; recorded scoped repair obligations without claiming live
  capture behavior.

- Rechecked checkpoint 1536 against the frozen selected pipeline revision after
  discovering the clean checkout was older. The substantive dispositions hold;
  validation-observation API and line positions differ. Future reads use frozen
  registered objects.

- Confirmed in isolated product probes that a constructed finite sloped support
  triangle can report zero grade and pass surface, bounded-terrain and collider
  construction; normal generation remains unverified.

- Confirmed in a selected-source module harness that a synthetic FBX rig retains
  a NaN skin weight, and that zero influence reduction yields NaN; downstream
  GPU behavior remains unverified.

- Completed original inventories for lenses 11, 12 and 13 while preserving
  historical attribution and separate connected obligations.

- Recorded source investigations across skill and book targets, rest and travel,
  identity codecs, injury clocks and strategic-to-tactical boundaries.

- Assessed all 403 lens/path checks in the latest 31-path source delta. The
  preceding 59-path delta has 767 of 767 checks assessed; connected source and
  acceptance obligations remain open. Earlier populations and connected work
  remain separate.

- Traced the replacement foliage renderer, shader layouts, material and loading
  readiness paths; retained admission, failure, lifecycle and visual acceptance
  questions.

- Read both registered lockfile revisions and both quality baseline revisions;
  reconciled the lockfile deletion and traced the quality baseline change to a
  function-length ceiling.

- Connected terrain admission, selected geographic input bounds, GPU limit
  selection and native/browser buffer error evidence to an existing open terrain
  allocation obligation.

- Preserved exact checkpoint history and published progress snapshots; the full
  audit evidence graph remains local.

- Assessed all 2,002 path-and-lens cells in the older source delta; connected
  and unchanged-source obligations remain open.

- Consolidated a source-backed attack-identity repair constraint: retain full
  participant and distinct attack instances through scheduling, cancellation,
  batching and diagnostic output.

- Correction to checkpoint 1673: loading resets the empty-party duration, but
  zero reconnect grace can still trigger abandonment. The separate zero-grace
  finding remains open.

- Traced mission enrollment, roster and timeout boundaries, including
  conditional stale membership and discarded enrollment acknowledgement; runtime
  and exact aliases remain open.

- Traced MHR skin errors and filesystem URI boundaries, including conditional
  missing-cluster, HTTP status, project-root and native path differences;
  supported policy and runtime remain open.

- Reconciled the Unix root round-trip observed at checkpoint 1694 with the pre-
  existing Lens 13 ledger entry; it is not a second unique finding.

- Traced the settlement Socializing button through the immediate-activity route:
  the submitted name is rejected. Scheduled Socializing has separate target,
  receipt and training semantics; browser runtime remains untested.

- Reconciled the profession-fame browser preview multiplier discrepancy with
  existing repair entry 2491; the related reward-enum conversion is entry 540.
  No additional unique finding is claimed.

- Traced focused capture plan completeness, absent body-response gates and the
  unsupported-scenario assertion; the assertion is already owned by entry 46.
  Prior report files remain on pre-admission failure.

- Traced pose-trace consumers and a conditional stale armor-readiness sidecar
  path when an output directory is reused for a no-harness capture. Runtime and
  exact alias reconciliation remain open.

- Traced a conditional capture readiness gap: a persistent unresolved playback
  can bypass the inspected frame timeout, while the separate watchdog covers
  armor readiness. External watchdogs and runtime behavior remain unverified.

- Traced a conditional isolated-simulation schedule admission mismatch: a non-
  quarter-hour allocation can pass its local total check, while stationary,
  settlement and location-aware preview consumers expect a validated schedule.
  Simulation commit and end-to-end reachability remain unverified.

- Confirmed in an isolated selected-method probe that an out-of-range anime
  breastplate gauge can panic before validation under checked arithmetic;
  unchecked arithmetic reaches the outer range error. Full loader and GPU
  execution remain untested.

## What remains

- Finish the required audit across all 13 lenses, the 20 migration families,
  connected source dependencies, exact canonical mappings, and current-source
  changes. The implementation gate is still incomplete.

- Resolve or explicitly disposition the remaining type ownership, bounds, units,
  frames, authority, construction, and policy questions. Preserve existing
  evidence without treating a narrow source result as whole-family completion.

- Implement the clean final bespoke types and APIs, update generated bindings
  through the prescribed generator where required, and synchronize the owning
  documentation.

- Complete independent review, remediation, required formatting and lint checks,
  applicable product tests, and isolated observable acceptance.

- Reconcile shared filesystem status, URI identity, containment and fallback
  policy with MHR and image consumers; validate platform behavior after the
  audit gate.

## Coverage and verification limits

The newest source-revision batch contains 31 changed paths and 403 lens/path
checks; 403 of those checks are closed. These are audit coverage obligations,
not bug counts. Earlier revision evidence remains separately qualified.

The canonical queue contains 3854 entries. Entries and views overlap, so this is
not a count of unique bugs or a meaningful completion percentage.

No product files have been changed in this continuation.

Earlier probes, including the checkpoint 2123 synthetic parser probe, ran
despite the handoff restriction. A later isolated exporter test build was
stopped before the test ran and its added fixture was removed. No product builds
or runtime probes will be run until the audit gate is complete; final validation
remains outstanding.

No subagents have been used in this continuation. Checkpoint verification is not
independent review.

## Evidence and report updates

Selected source revision: `ff9c1d7d0ea168b7e14eedc5e44879ee7b84fb04`.

- [Checkpoint 3275 publication record](CHECKPOINT-3275.json)
- [Saved report for checkpoint 3275](PROGRESS-3275.md)

PROGRESS.md is the latest working report. It is updated during ongoing work and
whenever a new checkpoint is published. PROGRESS-N.md preserves the report saved
for checkpoint N; later work updates the latest report without rewriting that
snapshot.

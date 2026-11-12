# Bounded Lens 2 assessment: errantry camp coordinates and identity

Reader: `/root/verifier_scope`. Source target:
`95e63628f9d70b8dd6e6fede64334696c6995009`. This report is a bounded
source-only assessment, not a whole-lens, queue, repair, or checkpoint-3821
closure. No product code, tests, fixtures, builds, or runtime probes were run.

## Historical records and chronology

Original 059 evidence revision `56f24f6457f6cec8ffcdc46f03f5ff26a8abb50e`
contains q2341 `L2-EXT-ERRANTRY-CAMP-COORDINATE-OWNER-FIT` and q2342
`L2-EXT-ERRANTRY-FRAME-ID-OWNER-FIT`. Both have status
`open-producer-owner-current-alias-reconciliation`, zero
`new_body_or_population_credit`, and `implementation_gate_complete=false`.
Three similarly named optional status fields are absent in the original; the
nulls in the compact summary are not original explicit nulls or false values.
The exact entry slices match the separately designated public reconstruction at
`26e360920b7b2ec32d5c76238623eaf6d6617d7e`; the whole queue streams differ.
`HISTORY-IDENTITIES.json` records their original and reconstruction hashes and
offsets. The reconstructed selection does not restore checkpoint 3821.

Seven recorded source rows name six original frozen files at Git
`6533e444fa37552927070cc8d8190907fd3b413e`, attributed to the historical
root's personal Lens 2 post-registration displays, with their original ranges
and no credit from adoption. The initial root extension receipts are dated
2026-10-30 02:11–02:15 UTC. Subsequent historical e275 records retain the
original root reader and separate chronology: `CURRENT-001` records exact
same-lens frozen-source reuse with no fresh body reread; `CURRENT-002` through
`006` record later selected source assessments. The previous reviewer was not
this reader. The original queue statuses remain open. See
`HISTORICAL-CHRONOLOGY.json`, `Q2341-SOURCES.json`, and `Q2342-SOURCES.json`.

The first later granular proposal records a scoped source-backed reuse gap for
the bound trial-site referent; the second retains it and records bounded
comparisons of existing minute owners and the canonical ID factory. Those are
historical scoped dispositions, with unbound/bound policy, other ID roles,
current populations, and exact global aliases still open. Neither proposal
asserts all-lens clearance or a repair/runtime pass. `Q2341-GRANULAR-SUMMARY`
and `Q2342-GRANULAR-SUMMARY` preserve the actual wording and qualification.

All six historical file hashes match their frozen identities; all 18 files
captured for this assessment are byte-equal between e275 and 95e. This is
file-specific comparison, not whole-revision, dependency, or runtime identity.
`SOURCE-READ-RECEIPT-V2.json` identifies 37 registered 95e source displays and
each unread complement; the earlier receipt remains preserved and is
superseded only because a later registered display was added.

## q2341: movement and elapsed coordinates

`errantry.rs:100–129` compares a typed departure, completed movement, reached
camp movement values and a separately qualified elapsed time. Rested-road
admission additionally requires elapsed time to reach its availability
coordinate. `authority_model.rs:598–605,643–670` retains these roles as raw
u64 fields beside a `StrategicMinute` departure. Native journey initialization
sets completed movement and elapsed to zero (`journey_camp.rs:175–190`);
recording a stop advances the two separately (:223–249), while resting advances
elapsed without advancing movement (:722–764). These source paths support zero
as a real initial coordinate and distinguish movement from elapsed.

`RequestedDuration` requires a nonzero number (`strategic_action.rs:318–329`),
so it is not an exact owner for these zero-valid coordinates. `MinuteSpan`
represents two absolute endpoints (`courtship.rs:531–545`),
`StrategicMinuteOfDay` a normalized within-day u16 (`strategic_time.rs:34–60`),
`ActivityMinutes` a u16 constrained to 15-minute segments (`allocation.rs:19–36`,
`activity.rs:28`), `ReligionMinutes` a religious allocation aggregate
(`world-schema/lib.rs:266–297`), and `InjuryRecoveryMinutes` a crate-private
recovery role (`surgery.rs:366–378`). These inspected candidates are not exact
replacements for either journey-relative role. This finite comparison does not
prove the repository lacks every potentially reusable numeric leaf or settle
whether distinct role owners should share one.

The typed absolute departure must remain distinct from relative progress.
`journey_local_minute` intentionally wraps within the anchor's day
(`journey_camp.rs:616–618`; `calendar.rs:304–321`); this selected behavior does
not prove a generic world-clock loss. Required classification includes bounds,
zero, reference frame, persistence/adapter ownership, and actual callers.

## q2342: bound trial site and other IDs

`errantry.rs:21–35` retains frame ID, trial ID, optional challenge ID and site
ID as distinct raw string roles. Native creation makes frame/trial/challenge
IDs and writes an explicit `journey-camp:unbound` trial site before JSON
serialization (`challenges.rs:2125–2157,2267–2303`). The binder selects the
matching party/accepted case and destination, decodes that JSON, replaces the
unbound marker with `journey-camp:{departure}:{movement}`, and persists both the
frame and challenge coordinates (:1141–1217). Submission rechecks the live
party/case/accepted contract/issuance and revision, then compares that text
before accepting the bound puzzle trial (:1265–1412). Retry behavior has its
own path and must not be flattened into a new-attempt check.

The existing `StrategicPlaceId::JourneyCamp` holds a checked party component,
`StrategicMinute` departure, and movement u64 (`strategic_place.rs:17–35,
214–238,286–296`). Native `current_journey_camp_place` constructs that owner
only after its current-camp predicate (`journey_camp.rs:262–308`). It is a
source-supported existing owner for the same bound camp referent when its
component admission holds. Its `place:v1:journey-camp:...` codec also carries
party identity (`strategic_place.rs:455–505,508–553`), whereas the trial marker
is shorter private text; the two strings are not interchangeable. Unbound state
and the separate challenge/party authority remain explicit design questions.
`CaseSiteId` represents a case site and is not a camp owner
(`strategic_place.rs:43–72`).

The observed canonical producer composes fixed puzzle slugs and decimal u64
IDs/ordinals (`challenges.rs:1806–1849,2125–2157`;
`puzzles.rs:14–44`); that bounded factory is compatible with the displayed
ASCII/length component admission. It does not prove all arbitrary frame,
trial, challenge, party, case-site, Serde, or native admission paths valid.
The roles remain semantically distinct even if a checked component leaf could
be shared. Exact party-ID admission and the shape of an unbound carrier remain
open before any connected repair decision.

A separate source question exists around the binder's use of
`journey_departure_minute == StrategicMinute::ZERO` as a candidate selector.
`StrategicMinute` itself admits zero (`calendar.rs:82–105`), while the binder
writes the real departure back into the same field. If a supported actual
departure can be zero and another binding occurs, an already bound candidate
could still satisfy that test; this is a conditional code-level observation,
not an established reachable defect or an alias of the relative-coordinate
owner question. Default clock initialization uses a positive day-231 start
(`calendar.rs:15–21`; `time/clock.rs:1–30`), while departure synchronization
may reuse a stored anchor (`time/settlement_rest.rs:571–608`). Full anchor
writer/invariant and clock reachability analysis was not performed.

## Related records and evidence limits

Original q245 / ledger row `SEGOVB-L12-COND-OPTIONAL-ROAD-TRIAL-CONTINUE`
concerns whether a bound optional road challenge may be ignored when continuing;
its original ledger classifies a conditional behavior/policy discrepancy,
keeps global aliases/current reconciliation open and records no runtime pass.
It shares a camp predicate but has a different first operation. Original q3293
`PL13-297-ERRANTRY-COORDINATE-BINDING-PENDING` concerns courier geographical
longitude/latitude and a conditional combat consumer, not the journey-minute
coordinates; its original selected record has no status key. Neither relation
establishes an alias. Their selected historic queue/ledger records and three
hash-cited references are captured with their own attribution. The ledger row
is reserialized from the original evidence stream; the separately reserialized
public reconstruction row is structurally equal, not a byte-exact raw slice.

All 22 direct q2341/42 hash-cited historical records and three linked-record
references were recovered with expected SHA-256 and matched to same-machine
transport metadata. Transport and local timestamps are byte/process-order
evidence, not independent custody or proof of personal comprehension. Two
historical files include local Windows home paths; the public successor redacts
those fields and retains encrypted exact originals, with the separate key
required for full original-byte replay. A derived path-classification artifact
is likewise redacted and encrypted. Whole-file identities are retained, but no
individual removed-path value digest is published. Same-machine key storage is
a recovery dependency, not independent public recoverability.

The privacy scan and locator scopes are bounded; they do not clear unknown
secrets, encodings, external Git history, all writers, or every historical
alias. Test-module text was inspected as text only. No product validation,
implementation, or repair is implied.

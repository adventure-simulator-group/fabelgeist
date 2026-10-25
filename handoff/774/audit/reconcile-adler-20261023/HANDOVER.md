# PR #774 continuation handover — 23 October 2026

Prepared by Adler's Codex agent. This is an incomplete-work handover, requested
by Adler now. It is not a report that the audits or refactor have finished.

## Read this first

The immediate task is to finish the remaining initial audits, preserving their
source evidence and qualifications. No implementation began in this audit
continuation. Do not treat this document, a completed local audit, or the merge
of #774 as permission to skip the remaining initial audit gate.

The full objective remains: recover and integrate the preserved work, complete
mandatory P16/P17 and every connected or reopened repair, obtain all thirteen
final lens clearances, independent Code/Rule/Comment reviews, and required
hosted validation on one final pushed revision. Do not merge further PRs or
mark #726 complete.

Adler wants the clean final design, including broad changes across producers,
retained state and recipients. Avoid preserving primitive internal APIs merely
to limit the blast radius. Reuse canonical bespoke types and actual validated
products. Conversions remain appropriate at genuine native, numerical,
serialization and framework boundaries. Do not add compatibility paths.

Earlier Adler requested a handover after the audits. His latest request is to
hand over now. The timing has changed; the outstanding audit requirements have
not been declared complete or waived.

## Where the work is

Repository: `REPOSITORY`.

This document:
`HANDOVER_PACKAGE/HANDOVER.md`.

The adjacent [STATE.json](STATE.json) records the latest handover snapshot,
progress records, hashes, remote observation and duplicate-registration
correction. Use it before older checkpoint summaries.

[EVIDENCE-MANIFEST.json](EVIDENCE-MANIFEST.json) fingerprints the complete saved
audit-record directory and indexes the frozen scoped source fingerprints.

Complete audit records:
`audit`.

Frozen source trees:
`SNAPSHOTS`.

The audit/ directory in this package contains the audit evidence.
Frozen source trees are not included. Recover their exact preserved
revisions through the original handoff and verify source fingerprints
before continuing source review. See README.md for path mapping.

Original handoff:
[2026-10-21 HANDOFF.md](https://github.com/adventure-simulator-group/fabelgeist/blob/codex/774-handoff-20261021-records/handoff/774/HANDOFF.md).

## Source versions and the merged PR

| Source | Revision |
| --- | --- |
| Frozen PR baseline | `6533e444fa37552927070cc8d8190907fd3b413e` |
| Frozen main baseline | `3a50b708381ad00c116445c2c999a49c3943088b` |
| Preserved P16 | `039329518e264f19206cf5b0b98d177433f40c17` |
| Preserved P17 | `90a422a453cd990ebc7f0ecf2ad5ed3e2d57d696` |
| Rig quality work | `cbbbca08c89a8768926799371f54cf6057312e5e` |
| Last verified #774 head | `6223320d09f4691b6de54382d9709b233274d435` |
| #774 merge commit | `13fc9dcc71ab879de364117e5462310673e43210` |
| Main observed during this handover | `990430b77a9f63548ffcb642074185f61ddb3322` |

GitHub confirms #774 merged on 23 October. We did not perform that merge.
The frozen audit does not clear the later PR/main changes. Explicitly compare
that delta before integration, retire findings only with actual current-source
proof, and preserve obligations that the merge did not satisfy.

The frozen `pr-working` tree also contains four preserved integration/quality
edits recorded in the original export. They are preserved work, not new edits
made during this audit.

Saved implementation checkout:
`IMPLEMENTATION_CHECKOUT`.
Read-only Git checks during handover found it clean at the frozen PR baseline.
Its recorded branch is `codex/continue-774-20261021`. It is outside the current
workspace write boundary; obtain applicable tool approval if later editing it.

Adler's separate GPU working changes are not included here.
They remain preserved locally on codex/propagate-buffer-byte-types.
Do not overwrite or silently integrate that independent work.

## Current audit status

The scope is 2,853 source variants plus 246 mandatory range overlays.
These are not 2,853 complete files in every case: some roles are exact partial
ranges. Original broader producer ranges and every older overlay remain
mandatory. The 312-role queue below is additional reconciliation for lenses
01/02, not a replacement for their historical inventories.

| Lens | Latest saved initial status |
| --- | --- |
| 01 Producer propagation | Historical inventory preserved; 312-role delta pending |
| 02 Existing type owners | Historical inventory preserved; 312-role delta pending |
| 03 Admission contracts | Current scope accounted with findings; not final clearance |
| 04 Invariant operations | 612 / 2,853; 61 local finding records |
| 05 Aggregate states | 1,472 / 2,853; incomplete |
| 06 Type ownership | Local discovery exhausted at scope 072, including all 246 overlays; 52 local records |
| 07 Redundant authority | 1,758 / 2,853; 69 local records; incomplete |
| 08 Behavior preservation | Not started |
| 09 Enum shapes | 132 / 2,853; 11 local records; incomplete |
| 10 Derivability | Not started |
| 11 Type granularity | Not started |
| 12 Magic strings | Not started |
| 13 Magic numbers | Not started |

Counts measure saved progress, not clearance. Finding records across agents and
lenses overlap and must not be summed as globally unique defects. A local
discovery completion with findings does not establish repaired-source
acceptance. All final audit waves, reviews and final validation remain due.

All three existing audit agents are stopped at handover: Camera and Native
are interrupted; Rig completed its turn with lens 09 unfinished. No process or
hosted job is being handed over as a live wait. Do not infer fresh evidence
from an agent's last message if it was not saved.

### Exact resume records

Paths below are relative to the complete audit-record directory.

- Root 04: `continuation/root/initial04-invariant-operations/`.
  Authority: `scope-authority-001.json`. Latest packet:
  `contract-groups-165.json`. Next work includes full PR/P16 dialogue
  `build.rs` (580 lines) and runtime `src/lib.rs` (1,785 lines), then remaining
  original roles, including P16 Core `road_encounter_catalog.rs` (2,689 lines).
  The five smaller dialogue schema/policy/hash/condition/prompt files and their
  actual identical counterparts are already saved in packet 165.
- Native 05: `continuation/native/initial05-aggregate-states/`.
  Latest progress: `progress-457.json`; latest saved packet 456. Its resume
  pointer names quest solver 975, assembly 969 and connected quest fixtures.
  Check coverage before rereading: solver was reported read, but unsaved work
  receives no completion credit. Then lenses 10 and 13.
- Rig 06: `continuation/rig/initial06-type-ownership/completion-483.json`,
  `frontier-reconciliation-482.json`, and
  `reference-qualification-484.json`. These supersede the pending-frontier
  wording of older completion 481. Current 072 roles are locally reconciled.
- Rig 09: `continuation/rig/initial09-enum-shapes/resume-frontier-021.json`.
  This is a resume document, not `contracts-021.json`; that latter file does
  not exist. Latest contract is 020. First unfinished global index is 1790:
  tactical-core `src/animation/tests.rs`, 1,969 lines. Then remaining lens 09
  roles and overlays, followed by lens 12.
- Camera 07: `continuation/camera/initial07-redundant-authority/`.
  Latest saved contract is `CAMERA-INITIAL07-CONTRACT-354` in
  `contracts.jsonl`; latest `progress.jsonl` gives first unfinished index
  1492. New URL/trade roles also require its own comparison. Then independently
  reconcile lenses 01/02's 312-role queue and perform lens 11.

Read the appropriate exported lens instructions under
`target/774-continuation-20261021/skills/<lens>/SKILL.md` when resuming that
lens. Parallel read-only audit work was explicitly authorized in
`continuation/root/parallel-initial-audits-081.json`; this did not authorize
parallel source repair or bulk inherited verdicts.

## Coverage authority and qualifications

Current scope:
`continuation/cumulative-source-scope-parallel-072.json`, SHA-256
`153b17c462414e6fbae07f3b89f8854c3090e134a6b15175a82c1c8e9bb423eb`.

Mandatory overlays:
`continuation/owner-range-extensions.jsonl`, SHA-256
`f72b47d0ba315b4bb19a15bbea493ca30e1cd911127b01c43ab89c347a723b0d`.

Bounded producer/owner delta:
`continuation/bounded01-02-newmaterial-queue-087.json`. It contains the complete
mandatory qualification list. Preserve all entries, not just the latest
qualification 249. The latest scope reconciliation is
`continuation/root/parallel-scope-reconciliation-251.json`.

Native REG056 is a duplicate, not a new scope extension. PR melee-round
`assignment.rs` already exists at global index 2741 with the same SHA and full
1–72 range. `qualification-426.json` (QUAL098) also corrects a duplicate
durability count and the assignment-list uniqueness shorthand. Do not create
another variant, overlay or 01/02 queue item for REG056.

Keep every source-specific qualification beside its proof. Recent examples
include Native developer-quest and investigation partial ranges, Rig 09's
actual P16 coordinate/orientation EOF limits, and Camera's corrected finding
family references. Some newer local qualifications have not yet been promoted
into a new global qualification record; reconcile them before whole-inventory
acceptance. Reading a nearby declaration, matching a hash, or finding no diff
does not by itself constitute an independent lens comparison.

Actual source narratives can support a new lens when personally consumed and
reanalyzed for that lens, with focused fresh critical bodies where required.
Do not bulk stamp coverage from earlier verdicts. Prospectively register
genuinely new material before body analysis, reconcile it across affected
lenses and preserve original ranges, exclusions and actual EOF.

Deleted-source roles remain explicit obligations with recovered-base/removal
evidence. Missing current files do not earn current-body credit.

## Findings and repair scope

The complete findings, supplements and qualifications remain in each lens
folder and in historical admission/producer/owner evidence. This section is
navigation, not a substitute for that evidence.

Important connected groups include:

- Geometry/physical units, coordinate frames, finite computed results, support
  availability and atomic publication through actual recipients.
- Rig rows, packed skin indices, topology, garment fitting, catalogue placement,
  cache identity and resource/layout association.
- Physical weapon construction and evaluation, melee timing/range/authority,
  strategic equipment projection, and concrete cause propagation through the
  actual SpacetimeDB boundary.
- Camera state, configured physical measurements, report time/foot/support
  applicability, cache dependencies and truthful validation evidence.
- Canonical enum/vocabulary reuse, correlated state shapes, derivable data and
  typed facts retained through internal APIs.

Recent exact anchors:

- Root F060, packet 151: orbit goal completion checks focus/radius while
  ignoring requested angle convergence; ordinary turn-only commands expose
  the source relation. No UI run was performed.
- Root F061, packet 163: a fully traced static accepted large-ratio lofted blade
  yields finite mass large enough that forge-time casting reaches u64 MAX;
  adding base time overflows before stock-capacity rejection. Checks-disabled
  execution still reaches the later rejection; no wrong successful quote or
  normal catalogue occurrence was claimed. Physical conversion support joins
  existing CAM-ADM03-F002 rather than creating another unique finding.
- Native F062: surface-only UV compaction can retain an unremapped accessor
  population. API-only; ordinary generated unused-row occurrence unproved.
- Native F064: Creator inventory identity exhaustion can overflow or duplicate
  an existing MAX identity. API-only, not ordinary UI exhaustion.
- Native F065, packet 400: surprise resolution can retain scheduled contact
  state for a later collection. Conditional surviving, uncancelled opener;
  no executed duplicate health-hit claim.
- Camera F062–069: runtime configuration versus baked-grid cache; conditional
  double application of partial primary-hand targets; duplicated clearance
  equations; configured sole offset versus a report literal; configurable
  capture timing versus fixed 64 Hz labels/playback; duplicated camera
  defaults; diagnostic labels affecting pace; and duplicated topology traversal
  for grab addressing versus HUD previews.
  Read each exact saved record for its own limits and deduplication.

Local negative dedup replies are navigation, not new source proof. Several last
agent messages contain qualified observations rather than findings, including
clinical optional-pair validation, bridge namespace parsing and transformed
blood-decal distance. Do not upgrade these to accepted normal-path defects
without the missing gates and recipient evidence.

## All twenty original families remain in scope

| Family | Subject | Preserved status requiring later reconciliation |
| --- | --- | --- |
| P01 | Movement-audio seeds | Earlier increment integrated |
| P02 | Battle-time coordinates | Earlier increment integrated |
| P03 | Maneuver duration producers | Earlier increment integrated |
| P04 | Playback rates, phase and event budget | Earlier increment integrated |
| P05 | Capture elapsed and jitter | Earlier increment integrated |
| P06 | Terrain support through IK | Reopened connected repairs |
| P07 | IK angular caps | Earlier increment integrated |
| P08 | Camera measures and history | Reopened connected repairs |
| P09 | Joint columns and skin ordinals | Reopened connected repairs |
| P10 | Parent topology | Earlier increment integrated |
| P11 | MHR rows and parameter columns | Reopened connected repairs |
| P12 | Garment offsets and upright anchors | Reopened connected repairs |
| P13 | Shader candidate/cursor states | Earlier increment integrated |
| P14 | Catalogue-bound equipment placement | Reopened connected repairs |
| P15 | Equipment reservations and chronology | Preserve current implementation |
| P16 | Melee geometry and targeting policy | Mandatory unfinished integration |
| P17 | Typed reports and applicability | Mandatory unfinished integration |
| P18 | Causal report quantities | Earlier increment integrated |
| P19 | Per-foot influence | Earlier increment integrated |
| P20 | Continuation coordinates and progress | Preserve current implementation |

The exact remaining gaps, commit mappings and historical evidence are in
`records/execution-ledger.json`. Historical source-complete or integrated
labels do not establish final current-source acceptance.

Do not cherry-pick the entire stale P16 branch. Reconcile complete changed
contracts with current rig/catalogue APIs, P15 and typed equipment
revision/sequence policy. P17's canonical-length geometry commit
`c912fd7a7c7346c187399644e32919dde8aa2bbd` is already integrated; do not replay
it as new work. Preserve each recovered increment and its native causes,
arithmetic order, zero/signed/full-width policies and fixture semantics.

Exact whole-function ownership remains in
`records/parallel-ownership/{root,camera_targeting,rig_garments,native_reports}-claims.json`.
Generated ownership maps are summaries, not transfer authority.

## Coordination with segovb's agents

Adler authorized replying. Every posted reply explicitly says it is Adler's
Codex agent, not a comment personally written by Adler:

- [#779/#780 ownership reply](https://github.com/adventure-simulator-group/fabelgeist/pull/774#issuecomment-6356105349).
- [#826 ownership reply](https://github.com/adventure-simulator-group/fabelgeist/pull/774#issuecomment-6356111438).
- [Expanded claims and merge update](https://github.com/adventure-simulator-group/fabelgeist/pull/774#issuecomment-6360167523).

Existing #774 whole-function claims were retained; no ownership transfer was
made. Final shared API signatures and an integrated implementation revision
remain unresolved. The merged PR did not transfer those responsibilities.

The combined integration must preserve canonical #779 WorkgroupGrid and
BufferByteLength; distinct mesh/particle/query/sort/hair populations; coherent
buffers, layouts and uploaded host data; selected texture-view interpretation
and owning-device validation; and actual native causes through parser,
pipeline, retained state and recording.

Expanded claims include surface-cut point admission, realization/body cache
association, skin/body/correspondence, bracer/breastplate, the complete armor
and metal facades, metal-bake functions and engraving cut/decode. Armor scratch uses
bytes with its minimum-four policy; metal scratch uses words. Preserve
empty-query-first behavior and computed signed image-sampling admission.

Receipt files are under
`continuation/root/github-coordination-20261022/` and
`continuation/root/github-coordination-20261023/`.

## Validation and implementation gate

No project builds, tests, analyzers, generators, runtime/UI acceptance or hosted
CI were run in this audit continuation. Source/Git inspection and ignored
text/JSON/hash/preservation bookkeeping were used.

The last preserved hosted attempt at frozen head 6533 failed at semantic/growth
quality checks before downstream acceptance. Its recorded run is
[39576450512](https://github.com/adventure-simulator-group/fabelgeist/actions/runs/39576450512).
This is historical evidence for that revision. It is not a claim about current
merged main. No final revision or final hosted selection is prepared.

Required sequence remains:

1. Finish all initial lens inventories and reconcile every material delta,
   qualification and connected finding.
2. Reconcile the merged/new-main delta with preserved work and exact shared
   owners; prepare one complete connected repair plan.
3. Perform authorized isolated implementation with one implementation agent for
   reproduce/fix/test loops. Keep unrelated GPU work separate.
4. Use existing tests only. Add no test cases, assertions, fixture helpers or
   doctests. Do not run local project checks under the preserved workflow.
5. Finish source correctness across producers and recipients before publication
   or hosted compiler feedback; then do formatting and quality debt cleanup.
6. Obtain complete repeated final lens clearances, independent Code, Rule and
   Comment reviews, and all required hosted checks on the same final pushed
   revision. Inspect the actual selection and artifacts; green compilation is
   not migration completeness.
7. Report verified scope and remaining limits. Do not merge or close #726.

Repository AGENTS.md and crates/AGENTS.md still apply. Tactical tick state stays
in transient Bevy authority. Do not hand-edit generated database bindings,
reset public/player data, add compatibility paths, or change combat fatigue
policy. Root README and wiki branch/review rules remain intact.

## Suggested recipient start

Read this document and STATE.json, verify the pinned scope and source trees,
then resume the unfinished initial audits from their actual saved coverage.
Preserve all prior work and qualifications. Treat new-main reconciliation,
P16/P17 integration, repairs, final audit waves, independent reviews and hosted
validation as outstanding. Ask Adler only if a concrete objective proves
undesirable or not reasonably achievable.

Do not infer completion from the age or size of the audit record. The handover
is complete as a transfer of saved state; the #774 continuation objective is
not complete.


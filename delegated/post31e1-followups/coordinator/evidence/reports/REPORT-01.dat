# q2354 NPC age chronology and eligibility: source-only assessment

## Historical custody and scope

The selected original and path-redacted reconstructed 059 queue entries are
byte-identical at SHA-256
`05380ec7fe4b86dc1a8d85ce7b25758a1e63ac1a707a7b530f11c5158b6efd1e`.
The original `parallel-l13-20261029` PL13-268 reviewer assessed the frozen
6533 NPC causal file; root `segovb` adopted that evidence without a root
personal reread. Five attached granular views preserve that reviewer's later
PL13-273–276 and PL13-290 findings, their registered body ranges and distinct
root-adoption receipts. All 21 directly referenced records, six nested
verification records, and 11 frozen source objects matched their recorded
hashes. The selected entry remains open; historical repair and runtime passes
were not claimed. This is recovered 059 evidence, not missing checkpoint 3821.

This reviewer assessed only the numbered, prospectively registered 95e displays
in this packet. Eleven frozen and 21 current paths have commit/tree/blob
proofs. Ten frozen/current source pairs are byte-identical; `character.rs`
differs and was inspected at 95e. Per-file identity is not whole-tree identity
or a transfer of another reviewer's personal coverage.

## Current source relationships

* `settlement_population/resident_persistence.rs:18–48,114–166,267–285`
  supplies a draft's exact age or its age-band default (8, 15, 30, 68),
  constructs historical name birth year at minute zero, and passes the age and
  `None` initial time to the persistent NPC constructor. `character.rs:
  2010–2036,2192–2214,2445–2455` stores that age, anchors birth at zero when
  no initial minute is supplied, and initializes the character clock.
  `time/clock.rs:125–127` delegates to `time/schedule.rs:99–125`, which
  initially inserts the official minute without an explicit age settlement.
* `relationship/family.rs:94–113` derives effective age from birth and the
  requested personal minute, and can update the cached character age.
  `time/mod.rs:54–74` calls the age settlement first at the lifecycle boundary
  following an authoritative personal-time write; `time/stationary.rs:23–35,
  87–135` returns immediately for zero requested elapsed and otherwise invokes
  that lifecycle after updating time. At a successful normal advance the cache
  therefore has a source path to refresh; this does not prove all producers or
  intervening operations settle it before the first policy decision.
* `npc_causal.rs:99–124,195–220,266–285,325–373,464–507` selects a live
  behind-clock cohort, processes schedule/housing/romance choices at its
  existing frontier, and only afterward advances it. Schedule uses effective
  age; housing and the early romance actor gate read cached age. Decisions
  have per-character/day/phase receipts (`npc_causal.rs:125–171`).
* The adult threshold is 16 (`adventuresim-core/src/courtship.rs:19`); a
  strategic year has 365 days (`adventuresim-world-schema/src/calendar.rs:
  10–14,189–204`). For a newly constructed resident with draft age 15 and a
  personal initial minute at least one strategic year after zero, birth-based
  effective age is at least 16 while its initial cached age remains 15. If no
  intervening operation refreshes it, it survives and is later selected at
  that initial frontier, a first schedule phase can admit adult scheduling
  while housing records ineligible; romance also records ineligible if the
  actor passes its separate presence gate. A same-day receipt can retain that
  phase decision after a subsequent age refresh. This is a source-conditional
  sequence, not a runtime observation or a proved reachable play incident.
* `relationship/courtship.rs:55–91,283–323` separately checks birth-based
  effective ages of both participants at a shared or maximum personal frontier.
  The candidate list in `npc_causal/romance_candidates.rs:1–28` first filters
  presence and NPC policy. The scheduler's raw-age actor gate is earlier than
  this native admission and must be assessed separately. Residence admission
  uses cached age at `residence.rs:680–698`, whereas its due-time billing
  classifies dependents by effective age at `residence.rs:792–806`; these are
  related authority surfaces, not automatically the same first contract.
* Real production entry points include `strategic/mission_bootstrap.rs:
  1082–1087,1118–1132` and `tactical/request.rs:38–53`, subject to their
  own guards. `ensure_settlement_population` creates/reuses drafts and invokes
  seeded family and housing binding (`settlement_population.rs:427–460`).
  `relationship/family.rs:166–201` and
  `residence/properties/materialization.rs:14–111` show their direct bodies;
  no broad claim is made about every indirect side effect. The pre-cohort
  global lifecycle dispatcher handles due weddings and births
  (`relationship/mod.rs:60–153`); the effects of all its nested callees remain
  separately qualified.

## Remaining work and connected repair direction

Decide whether a resident draft's age is age at world epoch or at creation,
then reconcile its birth, initial personal clock and early age gates according
to that policy. Qualify real late population creation, cohort selection,
intervening lifecycle effects, present-at-home behavior, daily receipt effects,
transaction outcome and actual impact before any incident or unique count is
claimed. Compare exact existing aliases only after those predicates and first
contracts are established. Historical PL13-290 explicitly kept this policy
and actual witness open.

The 1,060-row ledger snapshot's literal PL13-268/290, NPC-age, chronology,
`age_years`, `birth_minute` and `effective_age_years` queries found no rows;
this is bounded metadata navigation, not an absence proof. The related
`REVIEW-POPULATION-RESEED-HOUSEHOLD-001` shares population/family materialization
context but addresses replay of changed household authority, a different first
contract. Resident profession-text and stale resident-selection hits concern
other surfaces. No aliases or new unique count are asserted, and no ledger or
publication was edited.

No product execution, product parser, build, test, fixture, macro, runtime probe
or repair was performed. Evidence extraction, hashing and the evidence-only
verifier are separate. See the read-limit and privacy qualifications.

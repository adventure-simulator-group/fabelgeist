# q2322: forage time and capability commit

## Disposition and historical evidence

This is a bounded newly assessed source follow-up to recovered 059 queue entry
2322, `PL13-254-FORAGE-TIME-CAPABILITY-COMMIT-PENDING`, at pinned
`95e63628f9d70b8dd6e6fede64334696c6995009`. The historical item remains open;
this review does not establish lens or queue closure, a unique defect count,
or a recovered checkpoint 3821 final ledger or queue.

The recovered original and reconstructed 059 entry bytes match, while their
whole parent queues differ. Historical frozen revision
`6533e444fa37552927070cc8d8190907fd3b413e` and original Lens 13 attribution
are retained. The historical reviewer assessed `foraging.rs:1-1381` as an
exact cumulative partial union; the historical record explicitly disclaims a
single full reread. The two partial checkpoints are recovered and hash-bound.
`foraging/timing.rs:1-43` was separately assessed in full. Parent adoption
states zero fresh root body credit and no root reread. Neither attribution is
transferred as fresh current work. Registration, read, partial checkpoints,
verification, parent adoption, proposal and exact frozen source objects are
retained under `history/`.

The two historical source files and the additionally captured core foraging
file have individually equal old/current bytes and verified commit-to-tree-to-
blob chains. The root trees differ; no whole-revision equality follows.
`READ-LIMITS.json` records only the current numbered source windows actually
read. File copies and undisplayed registered windows add no personal credit.

## Current source-supported mechanism

The source supports a conditional cancellation mechanism: the forage action
advances time, its own time path increases fatigue, and the post-time guard
requires exactly the same rounded Stealth check as before the advance. It can
reject its own action when that change crosses a millirank rounding boundary.
The analysis does not show that every action fails, a particular persisted
runtime result, or the intended policy for changed capability.

* The reducer validates a duration of 60 through 1,440 minutes in whole hours,
  validates current authority, computes terrain and Stealth checks, and
  plans/replans before mutation (`stdb-module/src/foraging.rs:792-927`;
  `core/src/foraging.rs:29-35,329-334`).
* It verifies the planned actor and requested minutes, advances investigation
  time, compares actual elapsed/completion with the plan, and then recomputes
  and compares both rounded checks exactly before training and material effects
  (`stdb-module/src/foraging.rs:931-981`). Each check is rounded after clamping
  to 0–5 and scaling by 1,000 (`:467-515`).
* Investigation advancement calls character time advancement, which calls
  `apply_travel_condition` after ordinary nonterminal elapsed time
  (`stdb-module/src/time/advancement.rs:6-72,262-296`). That condition path
  increments `CharacterStats.calories_used` in proportion to elapsed time
  (`stdb-module/src/condition.rs:1880-1924`); its rate is 6,000 kcal/day in
  `core/src/provisioning.rs:1-4`. The `CharacterStats` implementation exposes
  that same field through `PlayerEssentials`
  (`stdb-module/src/capability.rs:190-197`).
* `PlayerEssentials` derives a fatigue penalty from calories used and
  endurance; physical skill checks multiply by it
  (`core/src/essential.rs:15-38`, `core/src/skill.rs:1132-1163`). Stealth is
  physical while the four terrain skills used in the blend are mental
  (`core/src/skill.rs:84-101`). A nonzero effective Stealth check can therefore
  change solely because of ordinary time fatigue. If the rounded value differs,
  the reducer returns `Foraging authority changed during the search` before the
  subsequent training, yields, and receipt writes. Zero/rank-clamped or
  saturated values and rounding can preserve equality. Exact admitted initial
  states and public reachability remain to be established.

The code says time effects may change physiology while capability inputs must
remain identical. Whether the guard should reject expected fatigue, compare a
stable subset, replan, or apply another policy is unresolved; those are policy
questions, not approved repairs. No assumption is made about transaction
rollback, persisted elapsed time or terminal outcomes after an error.

## Related contracts and open bounds

* Requested time is bounded and core `resolve` repeats duration and source
  validation. Quantity calculation after discovery uses saturating
  multiplication and converts to `u16` with saturation; the earlier expected
  discovery calculation multiplies `u64` factors separately
  (`core/src/foraging.rs:396-471`). This is not an exhaustive proof of numeric
  bounds, reachable resources, catalog items, or native conversions.
* Core plan time accepts only positive `RequestedDuration`, models terminal
  endpoints explicitly, and treats a terminal boundary as non-completion
  (`core/src/strategic_action.rs:318-410`). The forage timing adapter retains
  explicit injury/disease terminal flags even at the requested endpoint
  (`stdb-module/src/foraging/timing.rs:17-43`,
  `stdb-module/src/disease.rs:1995-2005`). Partial elapsed searches can carry
  a stealth exposure resolution with empty yields, while zero elapsed has no
  resolution (`stdb-module/src/foraging.rs:679-712`,
  `core/src/foraging.rs:780-812`). The `None`/`Some(MAX)` digest encoding alone
  is not proof of an effects collision: the plan separately receives the
  optional terminal time. Full endpoint/clock-overflow and terminal persistence
  behavior remain open.
* Later material code iterates planned yielded quantities, requests one
  checked inventory row per unit, and associates each row with an object and
  food lot (`stdb-module/src/foraging.rs:1022-1074`,
  `stdb-module/src/item.rs:410-426`). The complete item constructor, catalog
  admission, row resource bounds, and all native consequences were not
  assessed. Training is based on elapsed time and is applied after the guard
  (`core/src/foraging.rs:501-510`, `stdb-module/src/foraging.rs:978-1020`).
* The time-bound 95e ledger snapshot identifies separate related records for
  strategic overfatigue transfer, forage completion receipt identity, and
  noticed illegal-forage feedback. Their metadata is reproduced in
  `LEDGER-CANDIDATE-DISPLAY.json`; shared fatigue or forage does not prove a
  semantic alias, common remedy, or unique count. Pinned global alias searches,
  caller/public reachability and central ledger reconciliation remain open.

The first attempt to reuse an earlier ledger digest failed because the live
shared publication had changed; no entries were displayed on that failed
attempt. A separately registered snapshot was captured and its exact digest
and time recorded. Some literal locator lists are expressly truncated, and all
physical-worktree searches are unpinned navigation. See
`READ-QUALIFICATIONS.json`. No product code, product parser, build, test,
fixture, repair, or runtime probe was executed; displayed test-source lines were
read only as text.

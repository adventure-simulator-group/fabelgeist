# Bespoke type refactor: progress

Updated: 2026-11-08T22:40:20.210833+00:00

| Overall measure | Done | Remaining |
| --- | ---: | ---: |
| Ledger source-discovery entries | 590 / 3,866 | 3,276, including overlapping obligations |
| Historical repair families validated on combined head | 0 / 20 | 20 |
| Handoff audit gate | Not complete | Connected ledger and reconciliation open |
| Registered duration source packets | 10 adopted | Contracts and wider aliases open |
| Estate custody source packets | 5 adopted | 1 scoped discovery closed; policy and validation open |
| Death-cascade source packets | 1 adopted | Child continuations and mapping open |
| Fireplace custody source packets | 7 adopted | 1 scoped discovery closed; policy and validation open |
| Nested party merge row orders exercised | 2 / 2 passed | Broader cases open |
| Fungible currency merge scenario | 1 / 1 passed | Other stack cases open |
| Corrupt-backing merge failure scenario | 0 / 1 | 1 |
| Seed-party populated-inventory rejection | 1 / 1 on prior draft | Stake-only and reserve-only cases open |
| Seed-party recruitment-role rejection | 1 / 1 on prior draft | Other obligation cases open |
| Seed-party mission and case-site rejection | 1 / 1 on prior draft | Individual mission/incident branches open |
| Direct and offer member-transfer scenarios | 2 / 2 passed | Missing-object rejection untested |
| Normal direct fungible transfer | 1 / 1 passed | Anomalous backing cases open |
| Discard scenarios exercised | 2 / 2 on prior draft | Corrupt rows and liquidation open |
| Generic nested removal scenarios | 2 / 2 passed | Broader cases open |
| Measured drink adjustment | 3 / 3 passed on parent | Other cases open |
| Personal corrupt-condition operations | 3 / 3 rejected | Other backing types open |
| Guarded authority workflow groups | 26 / 26 passed | Coverage is scenario-specific |
| Personal currency wedding fixture | 1 / 1 passed on parent | Corrupt backing untested |
| Currency debit paths live tested | 3 / 3 | Broader cases open |
| Corrupt party condition rejection | 1 / 1 on parent | Other corrupt backing cases open |
| Rest payment error sites corrected | 2 / 2 | Enclosing rest rollback open |
| Tavern and inn failure paths exercised | 1 / 2 | Inn remains open |
| Storefront live-sale scenarios reached | 0 / 2 | 2 |

The handoff requires the audit ledger to be complete before code repairs. The
ledger's implementation gate remains false; the incremental implementation
work documented below was performed before that prerequisite was met. Future
work is returning to audit discovery and reconciliation. The implementation
branches are preserved as drafts, with their validation limits unchanged.

A bounded source read of the expedition elapsed-time contract found that dead
party membership persists while camp rest excludes dead members, and the
simulator includes membership clocks when subtracting maxima. A later dead
clock can therefore mask a living member's elapsed time when the relevant
unequal clocks and projections occur. The pinned SDK applies a successful
reducer update before its callback. This remains a conditional source finding:
runtime reproduction, projection completeness, aliases, and the intended
interval are open. A subsequent preregistered reread of 10 project files and
the pinned SDK passed preflight, adoption, and verification at audit checkpoint
3738. It adds scoped evidence without closing the contract or claiming new
population coverage.

A second registered packet compared the expedition elapsed contract with two
related ledger entries. Missing expedition projections are a prerequisite to
the rest metric; generated investigation uses separate maximum-clock
consumers for progress and action duration. The shared arithmetic does not
make these exact aliases. Both entries remain independent and open; no new
population coverage or contract closure was claimed.

A registered server-side investigation read found that exact retries, stale
capability reissue, and incomplete synchronization can return successfully
without a new outcome; clipped intervals also do so. The outcome projection
can additionally omit rows. Thus an absent new projected outcome is not a
unique source-level witness for clipping. Generated-flow reachability, timing,
remaining aliases, and runtime acceptance remain open. The scoped evidence was
adopted and verified at audit checkpoint 3740 without closing the contract.

A registered synchronization read found that the implementation returns the
leader's clock while preserving independent participant clocks, despite a
nearby comment describing convergence on the furthest clock. The interrupted
investigation receipt uses a maximum participant end time relative to that
leader start. The zero-preview boundary and unequal-clock runtime behavior
remain open. This scoped evidence was adopted and verified at audit checkpoint
3741 without closing the contract.

A registered boundary read found that disease preview can identify a terminal
crossing at the current minute, while zero-minute disease clipping returns no
terminal and zero-minute injury settlement returns alive without wound work.
Other lifecycle effects and actual reachability remain unverified; this is a
conditional source finding, not a demonstrated runtime failure. The evidence
was adopted and verified at audit checkpoint 3742 without closing the contract.

A follow-up registered lifecycle read confirms that disease completion does
nothing when given no terminal cause, and the inspected outbreak refresh reacts
to patients already dead or recovered. Other lifecycle hooks, the supported
current-minute state, and the end-to-end transaction remain unverified. The
scoped packet was adopted and verified at audit checkpoint 3743; the contract
remains open.

A registered admission read found that investigation requires living members
to pass refreshed strategic incapacitation checks. The inspected condition
calculation does not directly query the private terminal-meter calculation; a
separate terminal helper is used for preparation changes. This does not prove a
terminal private state can pass readiness. The relationship, indirect calls,
and runtime behavior remain open. The packet was verified at audit checkpoint
3744 without closing the contract.

A registered physiology read found that private terminal meters combine a
baseline, disease episodes, and interventions, while public illness flags come
from episode stages when notices are written. These distinct inputs do not by
themselves prove that a terminal character can pass action and simulator
readiness. Supported histories and runtime behavior remain open. The packet was
verified at audit checkpoint 3745 without closing the contract.

A registered simulator readiness read found that generated actions invoke
medical safety checks and require observed readiness; the inspected predicate
rejects observed symptomatic or critical illness but treats a missing illness
status as neither. Medical-check internals and coexistence with a private
terminal state remain unverified. The evidence was adopted and verified at
audit checkpoint 3746 without closing the contract.

A registered medical policy read found that an alive character with an
observed ready condition and no observed symptoms selects the ready path.
Missing illness status counts as no symptoms, and private infection episodes
are excluded from the policy. First-aid and schedule-restoration effects and
private terminal coexistence remain unverified. Audit checkpoint 3747 adopted
and verified this bounded evidence without closing the contract.

A registered estate-custody read found that death-time fireplace cleanup
precedes estate disposition, and that personal object roots are moved through
subtree validation before remaining personal rows move. Succession includes
cycle and death-time guards. Containment admission, outer transaction
behavior, and aliases remain open. Audit checkpoint 3748 adopted and verified
this evidence without closing the contract.

A follow-up registered read found that subtree validation constructs a
global containment graph and custody resolution checks ancestry and carried
backing. Inspected direct death callers propagate errors; malformed-state
reachability and transitive transaction behavior remain open. Audit checkpoint
3749 verified the bounded evidence without closing the estate contract.

A registered containment read found that the normal insertion reducer
checks custody, graph invariants and capacity. It removes a prior edge before
fallible validation, making rollback material. Selected encounter and camp-rest
paths propagate death-related errors, but other mutations and complete outer
transaction chains remain open. Audit checkpoint 3750 verified the bounded
evidence without closing the estate contract.

A registered transaction read traced the selected camp-rest and encounter
death paths to reducer error boundaries. The direct containment insertion is
also a reducer. Current [SpacetimeDB documentation](https://spacetimedb.com/docs/functions/reducers/)
describes database rollback on reducer error; it is not pinned runtime proof.
Other callers and mutation paths remain open. Audit checkpoint 3751 verified
the bounded evidence without closing the estate contract.

A registered fireplace read traced placed vessels with retained subtree
edges and the separate no-object return path on death. The related estate and
fireplace contracts remain distinct. Remaining merge branches, amount
initialization, and station writers are open. Audit checkpoint 3752 adopted
and verified this evidence without closing either contract.

A registered initialization read found that a no-object instrument return
to party inventory uses a helper that initializes applicable backing, while
the personal return inserts a plain row. Supported no-object station states
and affected item types remain unproven, so this is a conditional difference.
Audit checkpoint 3753 verified the evidence without closing the contract.

A registered station-writer review found that the inspected normal creators
produce either an instrument-free station or a vessel with both an instrument
and physical object. The earlier no-object initialization difference therefore
has no demonstrated trigger in those paths. Alternate writes and anomalous
persisted states remain open. Audit checkpoint 3754 verified this evidence
without closing the contract.

A registered state-model read found that three independent optional station
fields can represent an instrument without an object even though the inspected
normal writers pair them. This is a bespoke-type candidate, not a demonstrated
runtime failure. The related cooking-amount, estate and death contracts remain
distinct. Audit checkpoint 3755 verified the evidence without closing the
contract; complete writer and schema impact qualification remains open.

A registered projection read traced the station identities through the
gateway-only view and selected strategic web routes and template. A future
state-model change must coordinate these consumers and regenerate database
bindings. The full consumer inventory remains open. Audit checkpoint 3756
verified the bounded evidence without closing the contract.

A registered consumer read found that selected station query failures become
empty or absent UI data. Existing source tests check query escaping and use of
gateway views, not changed-schema behavior. The refactor will need a query and
UI verification that distinguishes failure from an empty result. Audit
checkpoint 3757 verified the evidence; broader policy remains open.

Audit checkpoint 3758 closed the fireplace custody source-discovery entry
with a conditional policy, using prior full-body evidence whose bytes match the
selected revision and newly qualified retrieval and death ordering. Normal
writers pair a vessel with an object; the extra representable no-object
instrument state still needs a product decision. Implementation, regenerated
bindings, runtime validation, and related estate and cooking contracts remain
open. No runtime failure is asserted.

Audit checkpoint 3759 closed the estate custody source-discovery entry with
a conditional policy. Equipment attachments are recursively torn down before
validated subtree transfer and chronological succession. Malformed attachment
handling, broader transaction coverage, implementation and runtime validation
remain open; related death and cooking contracts remain independent.

A registered death-cascade read traced selected corpse, browser, social,
relationship and party-leadership cleanup owners. Their full continuations
and the historical-to-current character source mapping remain open. Audit
checkpoint 3760 verified this bounded evidence without closing the cascade.

| Increment status | Count |
| --- | ---: |
| Queued | 0 |
| Auditing | 0 |
| Implementing | 2 |
| Awaiting or running checks | 2 |
| Awaiting review | 2 |
| Integrated and validated | 0 |
| Blocked | 0 |

| Current P16 quality scan | Count |
| --- | ---: |
| File and function size findings on inventory draft | 0; 3 cleared |
| Previously reported ingress size findings cleared | 2 |
| Pinned spawn error-handler source check | 1 / 1 |
| Loading reconnect mailbox gap | Draft; live validation open |
| Reconnect-filtered tests after extraction | 7 / 7 passed |
| Reconnect draft lint and formatting | 2 / 2 passed |
| Reconnect timing scenarios validated live | 0; open |
| SDK callback ordering and cache timing inspected | 2 / 2 source checks |
| Deletion and replacement scenarios validated live | 0; open |
| Reconnect and abandonment grace relationship | 1 source-derived question; open |
| Inventory row lookup references mapped | 33 in 12 files |
| Persisted row identity kinds drafted | 2; migration incomplete |
| Storefront request scope variants in adapted fixture | 2 / 2 passed on parent |
| Live storefront sale scenarios reached | 0 / 2; seed exhausted energy |
| Trade authority source checks present | 12 / 12; source inspected |
| Transfer source row boundaries consolidated | 2 / 2 |
| Isolated bottle and apple deposit/withdrawal on previous draft | 2 / 2 passed |
| Apple lot backing and value conservation on previous draft | 1 / 1 passed |
| Workspace production check on prior draft | Passed |
| Database module production check on current draft | Passed |
| Workspace all-target all-feature Clippy | Passed on current draft |
| Isolated inventory acceptance on previous draft | 1 / 1 targeted run passed |
| Focused identity, simulation and web custody tests on previous draft | 3 / 3 passed |
| Focused web custody retry | 1 / 1 passed |
| Food lot lookup/deletion helpers typed | 4 / 4 |
| Food lot creation APIs typed | 2 / 2; live validation open |
| Error-discarding party inventory wrappers | 1 removed; rollback unverified |
| Nested party merge row order scenarios | 2 / 2 passed on prior draft |
| Fungible currency merge scenario | 1 / 1 passed on prior draft |
| Corrupt backing merge failure scenarios | 0 validated; open |
| Current quality size findings | 0 in targeted quality scan |
| Existing core location fixture on previous draft | 1 / 1 passed |
| Generated bindings refreshed for identity draft | 1 / 1 |
| Previously reported function-size findings cleared elsewhere | 28 |
| Textual-conflict paths reviewed against both inputs | 3 / 3 |
| Additional overlapping paths reviewed against both inputs | 20 / 20 |
| Overlapping paths still awaiting bounded comparison | 0 / 23 |
| Existing contact-geometry tests passed | 11 / 11 |
| Existing autoresolve tests passed, including 9 melee iteration tests | 76 / 76 |
| Existing core equipment tests passed | 19 / 19 |
| Tactical server ingress tests passed, including range matrix | 9 / 9 |
| Tactical server player-projection tests passed | 33 / 33 |
| Tactical server bot tests passed | 13 / 13 |
| Tactical server equipment tests passed | 16 / 16 |
| Tactical iteration harness tests passed | 10 / 10 |
| Client targeting and acquisition helper tests passed | 6 / 6 |
| Client equipment fit and selection fixtures passed | 5 / 5 |
| Core catalog tests passed | 31 / 31 |
| Guarded developer authority, investigability and view fixture | Passed |
| Persisted selected and instance reach distinction | 1 / 1 fixture passed |
| Full P16 lint recipe on parent commit | Passed |
| Full P16 lint on previous Rust commit | Passed |
| Full P16 lint on previous reconnect draft | Passed |
| Full P16 lint on previous identity draft | Passed |
| Full P16 lint on current identity draft | Running |
| P17 current-head report tests | 26 / 26 passed |
| Previous jitter draft focused tests | 7 / 7 passed |
| Revised per-scenario focused fixture | 1 / 1 passed |
| Report tests on local jitter draft | 26 / 26 passed |
| P17 jitter observation distinction | Committed draft |
| Ragdoll jitter observation in actual viewer manifest | 1 / 1 correct |
| Previous ragdoll viewer acceptance | Failed |
| First ragdoll physics-pause candidate | Failed |
| Removed candidate repeated-view check | Passed |
| Removed candidate ragdoll continuity gate | Failed |
| Coordinated fixed-step ragdoll viewer run | Passed |
| Fixed-step draft formatting and full lint | Passed |
| Fixed-step draft capture tests | 5 / 5 passed |
| Fixed-step draft report tests | 26 / 26 passed |
| Ragdoll within-sample save intervals with zero physics steps | 18 / 18 |
| Ragdoll between-sample save intervals with one physics step | 8 / 8 |
| Ordinary viewer scenario | Jitter gate failed |
| Immediate-parent comparison | Same jitter failure |
| Ordinary first-frame clock investigation | Cause unconfirmed |
| First upper-spine rotation, frames 0 to 1 | 10.50 degrees |
| Upper/lower spine relative rotation, frames 0 to 1 | 10.28 degrees |
| Previous draft lint quality gate | Failed on size ceilings |
| Extracted draft formatting and lint gates | Passed |
| Extracted draft report tests | 26 / 26 passed |
| Extracted draft jitter tests | 7 / 7 passed |
| Previous draft native capture and attack excursion | Passed |
| Revised draft native capture and attack excursion | Passed |
| One-seed melee report matchups completed | 5 / 5 |
| Melee smoke report top-level audit flags true | 11 / 11 |
| Full-scale melee report at previous draft | Failed in first matchup |
| Reproduced failing seed after repair | 1 / 1 completed |
| Nearby autoresolve traces checked against aggregates | 100 / 100 |
| Nearby batch top-level audit flags true | 10 / 11 |
| Full-scale autoresolve traces completed before failure | 467 / 1,000 |

Reconnect draft `9d8dd35c2` retains a connected-player row while its loading
session is disconnected within its reconnect window. A delete callback checks
the current subscribed cache by character ID before discarding a deferred
row. The pinned SDK applies the full diff and unlocks the cache before row
callbacks; it dispatches inserts before deletes. This avoids relying on float
equality, but deletion and replacement behavior has not been tested live.

Seven existing reconnect tests passed without covering the new callback or
row-arrival timing. Full `just lint` passed before rustfmt adjusted the callback
layout; `just fmt-check` then passed. Live disconnect and reconnect timing,
deleted and replacement subscription rows, event ordering, grace expiry, and
whole-batch preparation failures remain open. Independent review is pending.

A separate source-derived lifecycle conflict remains open: the default empty-
party abandonment grace is 10 seconds, while a disconnected player retains a
reconnect capability for 30 seconds. A disconnected loading snapshot is not a
`LoadingPlayer`, so the mission system can count the party as empty during
that interval. Actual runtime timing and the intended relationship between
these two grace periods have not been established. The existing tactical
lifecycle fixture uses an isolated SpacetimeDB-backed mission, but its shared
`.env.tactical` handling and uncontrolled event timing require adaptation
before it can safely verify the new row-arrival case.

The inventory row-identity migration is a draft at P16 `8e63dde85`.
The complete guarded authority workflow passed all 26 reported groups on this
revision, including the newly integrated currency custody fixtures. It used a
separate local profile and stopped its listener afterward. This is regression
evidence for the covered scenarios; it does not close the historical repair
families or establish the missing storefront cases and independent review.

At the same revision,
A guarded fixture verified generic increase, generic decrease, and personal
currency credit reject an unexpected condition record without changing the
wallet; the same operations succeeded after removing that record. Party,
personal, and tavern currency fixtures passed together in an isolated run and
are now wired into the standard authority workflow. The earlier isolated run stopped
after those fixtures; the complete authority workflow later passed as noted
above.
`just fmt-check` and full `just lint` passed. Other backing types and
independent review remain open.

At parent `edf0c4a36`,
An isolated measured-drink scenario created two individual small-beer rows,
verified zero adjustment left both intact, then removed them one at a time.
The surviving row retained its amount record and physical object, and the
second removal cleared both objects. This adds one non-food measured item to
runtime coverage. Corrupt backing and independent review remain open.

At the same revision,
A guarded isolated scenario verified generic removal rejects a nonempty bottle
without changing the apple's containment, then removes the measured apple and
the empty bottle in order. It checked the apple amount row, food lot, both
physical objects, backing rows, and containment edge. Other measured items,
corrupt fungible backing, and independent review remain open.

At the same revision,
Generic inventory adjustment now sends stable food and measured items through
object-aware instance removal instead of deleting their backing rows directly.
Fungible changes and personal currency credit also check existing backing state.
An isolated apple-removal run confirmed deletion of the row, physical object,
and food lot. The first full lint attempt found an unused import; formatting and
full lint passed after its removal. The isolated run preceded that import-only
correction. Other measured items, nested-container rejection, corrupt fungible
rows, and independent review remain open.

At parent `ba48c4958`,
A bootstrap-protected isolated fixture confirmed that direct tavern purchase
rejects a currency row with unexpected condition state without debiting the
wallet, and that a normal purchase charges the expected price. The first run
stopped during setup because the new character already had starting currency;
the corrected fixture passed. `just fmt-check` and full `just lint` passed.
Enclosing rest rollback, inn failure, other corrupt states, and independent
review remain open.

At parent `1872b7bff`,
The currency caller audit found two rest-payment error handling sites: tavern
purchases swallowed all debit failures, and inn payments labeled every debit
failure as insufficient funds. Both now propagate the underlying error.
Authority-feature compilation, `just fmt-check`, and full `just lint` passed.
Corrupt-currency failure through a tavern or inn remains untested at runtime;
independent review remains open.

At parent `f61abf752`,
A bootstrap-protected authority-feature fixture exercised normal party currency
consumption and party-to-personal transfer. It also placed unexpected condition
state on the second of two party stacks and verified that both operations
rejected it before partially debiting either stack or crediting the recipient.
The isolated run passed before a module-order-only formatting correction;
`just fmt-check` and full `just lint` passed after that correction. Other
corrupt backing forms, broader currency cases, and independent review are open.

At parent `29d66f56a`,
A guarded isolated withdrawal exhausted the leader's stake, then withdrew a
bottle and its contents using a personal-currency top-up. Personal and party
coin changes matched, exercising one of the three newly guarded debit paths.
The first attempt used an apple whose value did not require a top-up; the
corrected bottle scenario passed. Party consumption, party-to-personal
currency transfer, corrupt-backing rejection, and independent review remain
open.

At the same revision,
Party currency consumption and both personal/party currency transfer paths now
preflight each selected stack before mutation. Empty stacks are skipped. The
currency code was extracted from the oversized trade file. Authority-feature
compilation, `just fmt-check`, and full `just lint` passed. A guarded party
inventory run passed nested merges, offers, and currency conservation, but did
not establish coverage of the three newly guarded debit paths; the later top-up
scenario above covers one of them. Corrupt-backing rejection and independent
review remain open.

At parent `82109bfb3`,
Personal currency withdrawal plans now carry typed personal row IDs and
preflight every selected stack before mutation. Authority-feature compilation,
`just fmt-check`, and full `just lint` passed. An isolated wedding authority
run passed its currency setup, rollback and success fixtures. Deliberately
corrupt currency backing and other payment scenarios remain untested. Party
currency consumption and transfers still need the corresponding audit and
repair; independent review remains open.

At parent `8431256e9`,
Storefront sale admission now rejects objectless stable or backed rows before
quoting, and sale application no longer recreates legacy lot or amount cleanup
paths. Authority-feature compilation, `just fmt-check`, and full `just lint`
passed. Live storefront sale and corrupt-row behavior remain unverified; an
older storefront setup exhausted energy while seeding. Currency movement
paths still need an invariant audit, and independent review remains open.

At parent `0d2fb48ab`,
Liquidation and discard now reject objectless stable or backed rows during
staging instead of deleting their associated state through legacy paths.
Authority-feature compilation, `just fmt-check`, and full `just lint` passed.
A guarded isolated run confirmed apple row, object and food-lot deletion and
partial fungible currency discard. Its first attempt passed the apple check but
stopped in fixture setup because currency merged into an existing stack; the
corrected retry passed both checks. Liquidation and corrupt-row failures have
not been exercised at runtime, and independent review remains open.

At parent `1b0b95c80`, A shared
classification and backing-state guard now rejects objectless stable rows in
deposit, withdrawal, direct transfer, and party merge. Object custody paths
remain in place. Authority-feature compilation, `just fmt-check`, and full
`just lint` passed. Two guarded isolated database runs passed bottle and apple
deposit and withdrawal, nested merges and member transfers, party offers, and
fungible currency transfers. Missing-object and corrupt-backing rejection have
not been exercised at runtime; independent review remains open.

At parent `de85048d3`, full `just lint` also passed. Fungible
operations now reject rows with physical objects or persisted lot, amount or
condition state. This check covers personal transfer sources and destinations,
party merge sources and party-credit destinations. The existing stacking rule
and party-credit operation moved to their owning fragments. Authority-feature
compilation, targeted quality and formatting passed. Isolated nested-object,
party-offer, party currency merge and direct currency transfer cases passed.
Anomalous-backing rejection and other inventory paths remain unverified; full
lint is running.

On `b5d7ba515`, party offer lines began carrying a personal row ID from parsing
through transfer; missing physical objects for stable rows return an error.
The request fixture, isolated transfer checks and full lint passed there.

An earlier seed helper could delete a solo party with inventory or active party
obligations.
The draft rejects reassignment for active bound missions and pending incidents,
as well as inventory, stake, reserve, travel, contract, encounter, fireplace and
recruitment/action records. On `060b6e367`, the empty-source fixture and role
rejection passed. A composite bound-mission and case-site scenario rejected
reassignment and preserved the mission, tactical request, membership and
clock. The case-site check fired first, so the separate mission-status and
pending-incident branches remain untested. Inventory rejection passed on
`1d2c4f963`. Compilation, formatting and full lint passed on `060b6e367`. A
misplaced helper comment was restored to its owning function.

On `5d33a1e9a`, a party merge lost the original backing rows for a nested
bottle and apple while their physical objects still referenced the former party.
The `6bfbe3ea7` draft moves root object trees before fungible rows and rejects
stable rows lacking an object. The isolated scenario passes for both
parent-first and child-first row orders, preserving backing rows, object IDs, containment and food-lot identity.
Authority-feature compilation, formatting and full lint passed on that draft.
An additional isolated scenario preserved the sum of source and destination
currency stacks.
Corrupt-backing rejection, broader merge cases and review remain open.

A party inventory wrapper previously discarded creation errors even when callers
subsequently removed source inventory. Commit `5d33a1e9a` propagates those
errors. Formatting, full lint and isolated bottle/apple transfer passed there;
an induced failure and rollback remain unverified.

Personal and party food-lot creation now accept distinct row ID types at the
database insert boundary. Formatting and full lint passed on parent `574c0e004`.
Live party food creation remains unverified. Storefront requests now admit
personal or party sale row IDs at parsing and carry them through validation and
application. Private request fields keep the sale scope
consistent with its rows. Deposit and withdrawal now each admit their source
row ID once and reuse it through object and food operations. On `d042cfbfc`,
the adapted request fixture, authority-feature check, formatting and full lint
passed. Commit `cb4d21bb3` updates a stale trade time source assertion; its
text was checked against the implementation, but the native module test has not
run. Formatting and full lint passed on `cb4d21bb3`; transfer draft `9dc0de32c`
passed authority-feature compilation, targeted quality, formatting and full
lint. Isolated bottle and
apple deposit/withdrawal round trips preserved object identity; the apple lot
also retained its identity, mass, nutrition and value while its backing changed
with custody. A storefront attempt stopped before either sale when settlement
seeding exhausted the module energy budget; the first attempt had a separate
runner decoding error. Three focused custody tests and
inventory lifecycle acceptance passed on `81531ce5d`. Live storefront and
cooking behavior, broader custody and repair, and review remain open.

The empty-party grace is explicitly configurable. The 10- and 30-second
values therefore establish an observable policy question, not by themselves
a proof that either duration is wrong.

The quality counts describe the latest scan, not a count of defects. At P16
commit `ea44c97b9`, the complete `just lint` recipe passed: binding checks,
quality checks, WebAssembly and workspace Clippy, and three standalone Clippy
commands. The build used a shared target directory; selected artifact
provenance for this complete run has not been independently rechecked. Earlier
focused checks and the guarded authority workflow passed as described below.

| Historical audit measure | Count |
| --- | --- |
| Original inventory roles, lenses 11Ã¢â‚¬â€œ13 | 8,559 / 8,559 accounted |
| Registered revision-delta lens/path checks | 3,172 / 3,172 closed |
| Ledger entries with source discovery explicitly closed | 588 / 3,866 |
| Other ledger entries | 3,278, including overlapping obligations |
| Equipment-law question rechecked on current draft | 1 / 1; still open |
| Authored equipable items without durability | 7 / 188 catalog items |
| Equipment-law typed draft | Lint passed; runtime unverified |
| Non-durable equipable items needing stable objects | 7 / 7 |
| Shared instance policy tests | 2 / 2 passed |
| Checked instance count and row-ID draft | Lint and focused fixtures passed |
| Persisted personal row-ID migration | Open |
| Same-kind nested container catalog witness | 1 / 188 definitions |
| Nested bottle and insufficient-removal database fixture | 1 / 1 passed |
| Previous one-container removal | 2 bottle rows deleted |
| Unselected nested inventory protection fixture | 1 / 1 passed |
| Tincture-spirit removal observation | 150 ml removed with bottle |
| Active-tincture demo setup | Failed: energy budget |
| Liquid and medicine removal policy | Open |
| Belt addition, spare-first and cleanup database fixture | 1 / 1 passed |
| Missing-inventory law preflight database fixture | 1 / 1 passed |
| Isolated runner listener cleanup | 1 / 1 passed |
| Stable inventory lifecycle draft | 2 database fixtures passed |
| Previous instance-draft full lint | Passed |

| Named repair families in the historical handoff | Count |
| --- | ---: |
| Source marked complete or complete-preserved | 12 / 20 |
| Source reopened for repair | 6 / 20 |
| Source marked implementing | 2 / 20 |
| Validated against the current combined draft | 0 / 20 |

These family labels come from the historical ledger and are not current-head
acceptance. P16 and P17 were the two marked implementing. The P17 restoration is an
ancestor of the current P16 draft; its review and validation remain open.
P16 covers melee geometry and targeting-range policy; the historical ledger
identifies 17 source gaps and shared dependencies that still need current-head
reconciliation. That historical count is not a current defect count. The
unfinished P16 recovery snapshot changes 159 paths; 23 of those also differ
between its original parent and the current draft. This is a path overlap,
not a count of conflicts. A three-way preview found 3 textual conflicts:
the core manifest, melee mobility, and server causal reporting. Textually
merged paths still require semantic review; validation remains open.

Historical coverage is not completed repair work. The revised user direction
allows independently audited implementation increments while retaining the
ledger as backlog. Full refactor completion and validation remain required.

## Active increments

- [Audio playback speed](https://github.com/adventure-simulator-group/fabelgeist/tree/codex/726-audio-pitch)
  commit `2d318efbb21d044680070c525554cb04ee286437`: focused `cargo check` and two existing audio
  configuration tests passed. Review and integration remain open.
- [Landform preparation outcome](https://github.com/adventure-simulator-group/fabelgeist/tree/codex/726-landform-outcome)
  commit `c40d70aeceeedfd87d7c9528628bf9c9bf87512b`: a draft based on the
  audio commit. It retains typed landform failures and excludes failed scenes
  from readiness. Recipe and patch arrivals now trigger reconciliation; changed
  recipes rebuild terrain. Focused Clippy passed at this commit; the focused
  client check and Rust quality checker passed at its parent.
  At the parent commit, 19 existing terrain tests passed and one benchmark
  was ignored; they do not exercise the new lifecycle or failure path. The
  replication library documents per-entity atomic updates
  within a tick, but cross-tick arrival and pre-arrival readiness remain open.
  Review and integration remain open.

- [Wedding capacity atomicity](https://github.com/adventure-simulator-group/fabelgeist/tree/codex/726-wedding-atomicity)
  commit `4510a2685ea541dd3963bd22aa7a6df6ea29a2a5`: the draft performs
  joint effective-dated capacity preflight and propagates unexpected settlement
  errors. Focused Clippy and the Rust quality check passed before the landform
  lint correction was added. The disposable-database workflow passed at
  `f33d5ebe37407718a2283dc9313924c32ab58973` for joint capacity, scoped
  removal, rejection, escrow refund, household preservation, rollback, and a
  successful wedding with both spouses admitted and the escrow paid. The
  landform lint correction is now included.
  `just fmt-check` and full `just lint` passed in a non-nested verification
  worktree at this commit. Review and final integrated validation remain open.

- [P17 report restoration](https://github.com/adventure-simulator-group/fabelgeist/tree/codex/726-p17-restoration)
  commit `dcd6ea1aaddb0e542a418ae7ebe35c0bf73d4ecc`: the preserved report
  implementation is restored with sample-tick and `LeadFoot` fixes. The focused
  animation-viewer compile passed; the Rust quality check passed before the
  relevant baseline reductions. Existing report tests passed: 26 passed,
  0 failed at this commit. Source review found local-position jitter derivative timing and incident labels use
  unshifted frames; a separate repair is present in the P16 ancestry. Current-
  head report tests passed: 26 of 26. A subsequent fingerprint trace selected
  42 local records with no markers from two known other worktrees; this does
  not rule out all shared-target contamination. Review and integrated gates
  remain open.
  Source inspection found that empty jitter frames report complete diagnostics:
  this can mean absent samples or an intentionally excluded ragdoll-only
  capture. Commit `ab1f7568a` uses the report observation type to
  distinguish unavailable evidence, measured diagnostics and ragdoll-only
  non-applicability. It requires a derivative window before claiming complete
  diagnostics. The draft checks each contiguous scenario, preventing
  one scenario from masking another with no complete window. An adapted
  existing focused fixture passed for a complete scenario followed by a short
  one. Seven jitter and 26 report tests passed on the previous draft;
  A hidden ragdoll-only viewer run produced a manifest with jitter status
  `not_applicable`, but overall acceptance failed: repeated evaluation changed
  the chest at scenario frame 0. The first logged differences were 0.0136 m
  and 3.81 degrees. This is a separate unresolved viewer failure; no threshold
  was changed. A local candidate holds the presentation physics clock during
  the three views of a ragdoll sample and resumes a pause it owns for the next
  sample. The new viewer run still failed repeated evaluation at frame 0:
  chest differed by 0.000726 m and 0.483 degrees in both later views. The
  earlier run differed by 0.0136 m and 3.81 degrees at the first later view.
  The first candidate paused after sampling the baseline. A revised local
  candidate pauses before the baseline and lets the paused pose settle before
  taking views. Its viewer manifest passed repeated evaluation and retained
  jitter `not_applicable`, but failed the next continuity gate: right-hand
  rotation was 63.97 degrees from frames 5 to 6, above the 60-degree limit.
  Earlier captures also exceeded that limit. Source inspection found that
  ragdoll muscles and damping run in Bevy fixed updates outside the paused
  physics clock, advancing age and angular velocity while the solver is held.
  The candidate was removed because it could distort the simulation. The
  branch is clean; a coordinated fixed-step capture repair remains open.
  Bevy offers a fixed-timestep update strategy that may coordinate fixed
  systems, but screenshot timing, settling and clock ownership require
  verification before using it. Commit `57b146829` controls the viewer time
  strategy around ragdoll views and requests one fixed update for each
  subsequent sample. Its nine-frame ragdoll viewer run passed all acceptance
  gates: repeated views were stable, jitter was not applicable, and the worst
  rotation was 42.88 degrees against the 60-degree limit. Formatting and full
  lint passed, as did five existing capture and 26 report tests. A second
  passing ragdoll capture traced the physics schedule: between logged view
  saves there were zero steps in all 18 within-sample intervals and one in
  all eight between-sample intervals. This evidence uses save-log ordering.
  The 65-frame ordinary camera-pitch capture completed but failed its jitter
  gate: chest position acceleration at frame 2 was 71.40 against an 18.0
  threshold; the worst incident was chest jerk at frame 3, 4569.44 against
  900.0. A bounded projection shows the chest and pelvis change between frames
  0 and 1, then remain nearly stationary through frame 7. Source shows the
  clock candidate arms only for ragdoll. The immediate parent `ab1f7568a`
  reproduced all four jitter incidents with identical summaries. Across the
  first eight frames, chest/pelvis position components differed by at most
  0.0000006 m and rotations matched. A trace of 42 selected parent records
  found none of three known other-worktree markers; broader shared-target
  provenance is not proved. Source inspection confirms the camera-pitch
  scenario has no required-motion readiness gate. Frame zero uses a zero
  animation interval; frame one uses one sample interval. The fixed-tick
  pose cache retains the first pose for repeated renders, and pose-buffer
  observations skip repeated ticks. These facts do not identify the source of
  the initial pose jump. A bounded global-bone projection shows the root
  rotation unchanged while upper spine c_spine3 rotates 10.50 degrees
  between frames 0 and 1; frames 1 and 2 show effectively no rotation.
  Relative orientation of c_spine3 to c_spine2 changes 10.28 degrees over
  that interval and approximately zero over the next. This comparison uses
  global orientations; it does not establish the direct hierarchy or owner.
  The saved frames show stationary speed, lowered guard and no action.
  Playback readiness
  checks that the plan is not bind pose; it does not prove the pose buffer has
  presented or settled that plan. Current entry points select ragdoll alone;
  mixed-plan
  clock restoration is not runtime-verified.
  Terrain correction runs outside the physics schedule; its causal role
  in the failure remains unverified. Do not relax the continuity gate. The first lint run also found report validation had grown
  beyond existing file and function size ceilings. Jitter applicability and
  acceptance were extracted to a focused module; formatting and full lint
  passed without raising the ceilings. After extraction, all 26 existing
  report tests and all seven existing jitter tests passed. The native diagnostic
  captured successfully in 859.9 seconds; its compact attack analyzer passed
  with 12 attack frames and a maximum forward hand excursion of 0.472 m,
  exceeding the 0.05 m minimum. This checks the native attack, not the viewer's
  jitter output. The first invocation failed before startup due to an invalid
  command argument; the corrected invocation completed. A native diagnostic
  for the revised per-scenario draft completed in 202.0 seconds. Its attack
  analyzer passed with 10 attack frames and 0.470 m maximum forward excursion
  against the 0.05 m minimum. This does not exercise viewer jitter output.

- [P16 recovery draft](https://github.com/adventure-simulator-group/fabelgeist/tree/codex/726-p16-restoration)
  commit `57b146829`: the 159-path snapshot is restored, with three textual
  conflicts resolved. A bounded source review compared all three conflict
  paths with both inputs. It found the typed values preserved, the speed
  conversion value-preserving, and all required movement observations set
  by the sole movement-event constructor. Behavioral verification of the
  new error path remain open. Three further overlap paths preserve both
  equipment chronology and typed geometry changes. Two more preserve typed
  combat settings, their validation, and length negation. Strategic weapon
  carry and tactical equipment retain the placement, chronology, and typed
  geometry changes. Four report files retain typed length conversions;
  their competing changes were formatting. Lifecycle, residence and client
  pickup changes also survive their overlaps. Harness, hit debug and
  equipment-generation fixture changes retain both inputs. Client equipment
  retains the typed pickup range and revision admission. The strategic
  character and lockfile comparisons complete the 23-path bounded review.
  Full dependency resolution, independent review, the other recovered paths,
  and client visual, input and fixture behavior remain unverified. The
  targeting and lunge-acquisition source paths were also reviewed; six
  existing helper tests passed. Five older selected shared dependency
  records match a previously checked server selection; one client build
  script record remains unverified. Five equipment fit and selection
  fixtures also passed with the same provenance qualification.
  The movement report uses typed measurements and rejects missing required
  observations; the new failure behavior still needs behavioral validation.
  A one-seed smoke run completed five matchups and all 11 top-level audit
  flags were true. The full-scale run then failed in the first matchup
  after 467 autoresolve traces: seed 468 reproduced an unavailable recovery
  interval. The report rejected `CannotRecover`, a valid combat state.
  The repair counts unavailable recovery separately and computes the mean
  only from available intervals, with no mean when none exist. Seed 468 now
  completes: its 11 available and one unavailable samples match the trace,
  as does the mean. In a 20-seed batch across five matchups, all 100 trace
  aggregates matched; knife novice seed 465 timed out at 256 rounds, so
  10 of 11 audit flags passed. An existing core fixture permits this bounded
  timeout while the report demands none. The pre-restoration draft
  `8ff11c82f` produced the same timeout, combatant outcomes, complete log
  and timeline for this seed. The zero-timeout audit remains unsatisfied;
  the comparison does not establish behavior for other seeds. The 3 GB bulk
  trace from the failed run was removed after recording its size and record
  count. Full-scale rerun remains open.
  A Rust quality scan passed at `f22bd2c26`, after behavior-preserving
  extractions and baseline reductions. At parent `aaf0a66c7`, formatting,
  workspace Clippy, all three standalone Clippy commands, and the guarded
  disposable-database authority workflow passed. The workflow included wedding
  capacity and rollback, investigation retry, puzzle, mission and outbreak
  checks. Repair, dialogue and development-fixture behavior remain unverified.
  Two subsequent commits changed test fixtures, and `48682faff` changed
  report aggregation with the focused validation described above. Commit
  `c438e0abe` corrected an authority-runner success message. Commit
  `5837f86b5` updated catalog fixture counts, kind totals and ID hash. All
  31 catalog tests passed. Commit `4afaf7b59` carries validated investigation
  authority through its consumers. Later commits extend an existing guarded
  fixture through the reducer and public view. Focused Clippy, formatting,
  Rust quality, and the isolated authority workflow passed for the change.
  The repair passed formatting, focused report Clippy and Rust quality. Full
  lint passed on the current draft; broader current-head behavioral validation,
  review and integration remain open.

  Existing tests passed: 11 contact-geometry and 9 melee iteration tests at
  `aaf0a66c7`. At `5837f86b5`, all 76 autoresolve tests passed, including
  those nine, as did 19 core equipment tests. Nine tactical ingress and 33
  player-projection tests passed at
  `1c0f6299b`; and 13 bot tests at `e9c405a77` after correcting two existing
  attacker fixtures. The range matrix is included in the ingress count.
  Selected local dependency fingerprints were checked for these shared-target
  runs. A prior authority attempt used stale artifacts from a different
  worktree; five specific fingerprint markers were invalidated under the
  build lock, and the subsequent authority build selected nine refreshed local
  records without the other worktree marker.

  An existing mission source guard was updated after its assertion moved with
  the extracted code. Its revised assertions passed a direct source audit and
  compiled under Clippy. Its native test could not link SpacetimeDB host
  functions, so the test itself has not run. The guarded authority workflow
  now loads a persisted diagnostic longsword and deliberately gives its catalog
  row a different reach from its stored instance geometry. It confirms that
  the strategic equipment capability returns the selected catalog reach and
  the combat equipment returns the instance reach, then restores the row.
  This passed for one equipped melee weapon in an isolated local database;
  it does not establish every weapon or error path. Focused authority-feature
  Clippy, formatting and the full lint recipe passed on `7e169b976`. The
  shared target provenance qualification applies to the full run.
  Investigation consumers now retain the validated manifest and context,
  including developer-context bases, and the reversed investigability tuple
  lookup is removed. Existing guarded cases passed; the extended guarded
  fixture stored a developer wrapper and verified reducer lookup and hostile
  investigability. A subsequent guarded run also verified the public action
  view returns a developer-generated action under its public case ID. Native
  unit fixtures only compiled. Player spawn admission prepares all applicable players before
  issuing any spawn commands. The input queue is drained first and is not
  requeued on preparation failure. The pinned Bevy 0.19.1 source assigns
  panic severity to ordinary converted errors and its default fallback
  dispatches that severity to a panic handler. No server-local override was
  found in the inspected code; deployment configuration remains unverified.
  A separate source-derived reconnect gap exists if the database row arrives
  after the disconnected loading entity is gone: admission skips the row,
  while reconnect restores the saved loading state without a new enrollment
  request. The subscription registers an insert callback; redelivery and the
  precise event timing need a live check. The dispatcher does not monitor or
  restart its child
  in the inspected source; deployed supervisor and row redelivery behavior
  remain unverified.
  The core build uses geometry without default engine features; an offline
  dependency tree contained no Bevy packages. The iteration observer source
  retains the first typed error for readback;
  ten existing harness tests passed on unchanged iteration production code,
  but they do not inject an observer failure. Remaining review, behavior
  checks and integrated gates stay open.

- [Jitter diagnostics draft](https://github.com/adventure-simulator-group/fabelgeist/tree/codex/726-jitter-quality)
  commit `8ff11c82fb4430ee17a170c61bea72c3d5173625`: source-derived timing
  correction and incomplete-evidence detection. Seven existing jitter tests passed on parent
  `96a20906e7e8e78c61dde90260fa16ecc5e47020`; they do not cover the new
  missing-evidence behavior. The size refactor compiled and passed the Rust
  quality checker before its baseline reduction and unused-import cleanup.
  At parent `8b3da2e16be30895a7c65f8e31bd3c7614562096`, `just fmt-check`
  and seven existing jitter tests passed. Full lint found a redundant borrow;
  the current commit fixes it, and full `just lint` passed at that commit.
  Behavioral verification of missing-evidence cases and review remain open.

The build queue remains serialized. Earlier [checkpoint 3737](CHECKPOINT-3737.json)
preserves the historical audit; its former global implementation gate has been
superseded, while its evidence and unresolved questions remain in the backlog.

## Current source reconciliation

- Rechecked the equipment-law error-classification question on commit
  57b146829. In a settlement with a known policy, the legality function can
  return errors for a missing character, inventory row or item definition as
  well as a prohibited item. Compliance treats every error as prohibition
  and removes the equipped item and any attached descendants. If the character
  has no settlement or its policy is unknown, inventory and definition checks
  are skipped. Presentation changes and reconciliation call this function.
  The equipment table has an inventory-item primary key and character index;
  a source-derived stale-row sequence is now identified: an equipped belt
  is non-durable, has an authored worn placement and attachment points, and
  the generic inventory-change reducer deletes its depleted inventory row
  without the durable branch's unequip operation or stable-object deletion.
  Seven of 188 authored catalog items have placements but no durability; all
  seven have attachment points. The generic positive-change branch can also
  merge an existing row rather than creating separate stable objects. This is
  source evidence, not a database reproduction. The catalog projection retains
  attachment points, and the shared stable-object predicate consequently
  selects all seven. Existing durable removal first unequips the selected root
  and deletes its carried-object subtree; subtree deletion validates custody
  and backing rows before mutation. The durable quantity limit and selection
  helpers currently live in durability, so sharing their policy requires a
  deliberate common owner and preserving the existing checked behavior. Draft
  commit 882a4d024 moves the prior bounded change and spare-first selection
  rules to a shared instance policy and applies stable-object addition and
  removal to non-food, non-measured instances. The two relocated tests and
  authority-feature compile, formatting and full lint passed on 882a4d024.
  Database behavior, rollback and batches selecting a container and its own
  descendant are unverified. One authored definition, glass_bottle, has a
  250 ml exterior and 250 ml capacity; the containment reducer allows a
  same-kind child subject to custody and capacity checks. If selected parent
  and child are deleted parent-first, the earlier draft can attempt to delete
  the child twice. Commit a17f710d2 plans selected removals in descendant-first
  order. Its authority-feature compile, formatting and full lint passed. A
  guarded check in a uniquely named disposable local profile passed: two
  bottles were created as separate rows, nested, preserved after an excessive
  removal request, then removed without extra physical-object or containment
  rows. The same isolated runner also passed separate belt additions, removal
  of an unequipped spare before an equipped belt, and removal of the equipped
  belt with no remaining equipment occupancy or extra physical object. Its
  first setup attempted to replace an existing belt with attached items and
  was correctly rejected; the retry cleared the fixture's belt first. Initial
  harness attempts failed on profile naming, character lookup
  and SQL aggregate syntax before reaching the fixture; an owned listener was
  cleaned up and the runner was corrected. This single scenario does not prove
  every equipment-law failure, all custody cases or post-write rollback. An
  extension to the existing guarded authority fixture on commit 7cac4364a
  passed in the same isolated workflow: with a prohibited equipped weapon and
  an equipped row lacking inventory, compliance returned the typed missing-row
  error and retained both rows. Formatting and full lint passed on 7cac4364a.
  Subsequent commit 4fa2f22e5 changes the Python runner to stop its verified
  listener and launcher; the focused fixture passed again and no listener
  remained on its port. Other
  missing-data variants and unrelated cases remain unverified.
  A local unadopted contract-revisit record preserves source hashes and these
  results without changing the historical queue or claiming audit closure.
  Commit a54fec4f4 carries a bounded instance count and a distinct personal
  inventory row ID through selection, with primitive conversion at database
  boundaries. Two existing tests, authority-feature compilation, formatting
  and full lint passed. The guarded bottle, belt, enrollment and equipment-law
  fixtures also passed on this commit. An additional isolated run requested
  removal of one outer glass bottle containing another and observed both rows
  deleted, consistent with the subtree helper. Commit d893ab083 preflights
  selected subtrees and rejects deletion of an unselected inventory object.
  The isolated check passed for rejecting one selected outer bottle while
  preserving both, then explicitly removing both. Belt and equipment-law
  fixtures also passed. Formatting and full lint passed. Source inspection
  shows subtree deletion removes liquid, water-contribution and medicine
  records, and the quantity path does not call the helper that protects
  active cooking, immature tinctures and fireplace ancestry. Intended
  disposal policy remains open. A guarded database run used the normal pour
  reducer to put 150 ml of tincture spirit in a bottle; removing the bottle
  also removed that liquid. This is an unprocessed spirit case, not evidence
  about an active or completed tincture or about water. A subsequent attempt
  to seed the guarded herbalism demo exhausted the module energy budget
  before reaching the active-tincture case; earlier checks in that run passed.
  No active-tincture result is claimed.
  The persisted personal inventory location still holds a raw row ID, and a
  shared accessor collapses personal, party and repair IDs into one optional
  integer. The producer-to-consumer type migration and
  schema/client-binding consequences remain open. A separate local record
  retains exact source hashes without claiming historical audit credit.
  This is not a completed repair. Remaining writer paths,
  transaction behavior and the inventory lifecycle repair remain open. Draft
  commit 9711eae2e distinguishes policy prohibitions from missing records and
  preflights all equipped rows before any unequip. The authority-feature
  package check, formatting check and full lint passed. No database behavior
  has been tested. The historical ledger is not marked closed.


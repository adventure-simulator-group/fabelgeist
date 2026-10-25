# Current migration state

Generated from execution-ledger.json by render-current-state.py. Edit family
status in the ledger and exact ownership in the claim files; regenerate this
view after each integration, transfer or validation outcome and before a
handoff/context reset.

Ledger snapshot: `2026-10-23T14:08:15.726258+00:00`.

Integration head: `6533e444fa37552927070cc8d8190907fd3b413e`.

Recorded published head at preserved baseline: `6533e444fa37552927070cc8d8190907fd3b413e`.

Frozen audit baseline main: `3a50b708381ad00c116445c2c999a49c3943088b`.

Goal #726 remains incomplete. GitHub reports PR #774 merged at
13fc9dcc71ab879de364117e5462310673e43210; remaining continuation contracts are
not cleared by that merge. Do not merge further PRs or mark #726 complete.

Final acceptance gate: initial_inventory_incomplete_no_further_fixes. Audit
coverage authority: final-acceptance-20261021/audit-ledger.json. Mandatory
P16/P17 implementation and integration remain in scope.

## Family state

Source completion, integration and validation are independent. The six earlier
acceptances remain historical evidence at their exact head; none validates the
current integration head.

| Family | Owner | Source | Integration | Current validation |
| --- | --- | --- | --- | --- |
| P01 | coordinator | complete preserved | integrated | blocked at semantic quality before contract execution; historical acceptance retained |
| P02 | coordinator | complete preserved | integrated | blocked at semantic quality before contract execution; historical acceptance retained |
| P03 | coordinator | complete preserved | integrated | blocked at semantic quality before contract execution; historical acceptance retained |
| P04 | coordinator | complete preserved | integrated | blocked at semantic quality before contract execution; historical acceptance retained |
| P05 | coordinator | complete preserved | integrated | blocked at semantic quality before contract execution; historical acceptance retained |
| P06 | coordinator | reopened | integrated prior increment repair pending | blocked at semantic quality before contract execution |
| P07 | coordinator | complete preserved | integrated | blocked at semantic quality before contract execution; historical acceptance retained |
| P08 | camera/targeting | reopened | integrated prior increment repair pending | blocked at semantic quality before contract execution |
| P09 | rig/garments | reopened | integrated prior increment repair pending | blocked at semantic quality before contract execution |
| P10 | rig/garments | complete | integrated | blocked at semantic quality before contract execution |
| P11 | rig/garments | reopened | integrated prior increment repair pending | blocked at semantic quality before contract execution |
| P12 | rig/garments | reopened | integrated prior increment repair pending | blocked at semantic quality before contract execution |
| P13 | rig/garments | complete | integrated | blocked at semantic quality before contract execution |
| P14 | rig/garments | reopened | integrated prior increment repair pending | blocked at semantic quality before contract execution |
| P15 | coordinator | complete | integrated | blocked at semantic quality before contract execution |
| P16 | camera/targeting | implementing | not integrated | not run for current changes |
| P17 | reports | implementing | waiting for source repairs and shared dependency | not run for current changes |
| P18 | reports | complete | integrated | blocked at semantic quality before contract execution |
| P19 | coordinator | complete | integrated | blocked at semantic quality before contract execution |
| P20 | coordinator | complete | integrated | blocked at semantic quality before contract execution |

## Active ownership and next actions

coordinator: P06. Exact scopes: parallel-ownership/root-claims.json. See the
ledger for each family's remaining gaps, shared dependencies, commit mapping and
next action.

camera/targeting: P08, P16. Exact scopes:
parallel-ownership/camera_targeting-claims.json. See the ledger for each
family's remaining gaps, shared dependencies, commit mapping and next action.

rig/garments: P09, P11, P12, P14. Exact scopes:
parallel-ownership/rig_garments-claims.json. See the ledger for each family's
remaining gaps, shared dependencies, commit mapping and next action.

reports: P17. Exact scopes: parallel-ownership/native_reports-claims.json. See
the ledger for each family's remaining gaps, shared dependencies, commit mapping
and next action.

Finish remaining initial audits and exact deltas before any implementation;
explicitly reconcile merged/new main with frozen source before code integration.
Preserve mandatory P16/P17 and all connected repairs, final13 lenses,
independent reviews and same-head hosted validation. Handover requested now;
initial audits are incomplete. Read target/774-handover-20261023/HANDOVER.md
first.

## Hosted evidence

Latest completed attempt: run 39576450512, job 119143045807, completed/failure
at head 6533e444fa37552927070cc8d8190907fd3b413e. Semantic/growth checker:
scoped production/function growth, moved/native model-token decoder registry
entries, and three stale baseline entries; checker tests themselves passed.
Covers the current integration head: True. No run is pending for the new
increments.

Next hosted checkpoint: not prepared/requested; exhaustive initial discovery and
complete connected source repair batch required first. No next head or
acceptance selection has been declared. The previous completed selection is
historical evidence in last_completed_hosted_contract.

Historical acceptance: rig-artifact-verification-82cb2.json, head
82cb2a3acccc852bba6a7ef9c706c337ba9fe765. Historical closure and CI reports stay
pinned to their recorded revisions.

## Sequencing and constraints

Only request hosted CI, including pushing, after a family/increment is presumed
source-complete through known producers, unchanged callers and receiving bodies.
Integrate no unfinished parallel source just for compiler feedback. If
compilation reveals an unknown gap, reopen and finish its connected repair
before another run.

Formatting and file/function-size cleanup follows source correctness.

Parallel read-only discovery is authorized during the initial all-thirteen-lens
inventory gate. Source edits/integrations/publication are frozen until every
initial lens discovery is complete and shared repairs are reconciled. Then
resume isolated implementation with one coordinator and exact existing ownership
claims.

Prohibited; source/Git/rg and ignored stdlib text/JSON/TOML/hash/ZIP bookkeeping
remain allowed.

Use existing tests only; add no tests/cases/assertions/fixture helpers/doctests.
Preserve existing fixtures except explicit previously authorized adaptations.

All thirteen current bespoke-audit lenses must complete full discovery before
new source repair, then repeat complete waves until clear. Code Review, Rule
Review and Comment Review plus required hosted CI must apply to the same final
pushed revision. Earlier three-wave/Comment Review exclusions are superseded.

Preserve the original stash, unrelated work and historical evidence. No
schema/state/generated binding changes or live database operations are
authorized by this bookkeeping. Coverage limits remain explicit in each family
and the pinned acceptance reports.

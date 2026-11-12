# Bounded review of historical 059 q2319

## Evidence and historical status

Original 059 queue `/entries/2319` is
`PL13-248-FIREPLACE-SUBTREE-DEATH-ESTATE-LIFECYCLE-PENDING`. It remained
`open-connected-source-current-alias-qualification`: discovery and repair or
runtime were false, with zero new body or population credit. The granular
helper view was also open. This is historical state, not checkpoint 3821 or a
current global closure. The original queue revision
`56f24f6457f6cec8ffcdc46f03f5ff26a8abb50e` and public path-redacted
reconstruction `26e360920b7b2ec32d5c76238623eaf6d6617d7e` have different
full-stream identities, but their exact q2319 byte slices are identical.

Eight cited proposal, registration, read and root-adoption records were
recovered from the protected extraction and match their recorded hashes. The
published transport inventory also has a matching digest, size and relative
suffix for all eight; it does not prove independent off-machine custody. The
Lens 13 reader recorded full personal assessments of frozen
`6533e444fa37552927070cc8d8190907fd3b413e` fireplace-custody and
inventory-container files. Both reader receipts said root adoption was then
pending; separately recovered later root-adoption records preserve the
original delegated attribution, record no root personal reread, and do not
claim current-source reconciliation or runtime validation.

At pinned `95e63628f9d70b8dd6e6fede64334696c6995009`, the entire
fireplace-custody file matches its frozen predecessor; the whole
inventory-container file does not. Exact current helper ranges do each occur
once byte-for-byte in the frozen file:

| 95e inventory-container lines | Frozen 6533 lines | Scope |
| --- | --- | --- |
| 286-350 | 307-371 | Prevalidate subtree rehome |
| 371-526 | 392-547 | Rehome subtree |
| 528-601 | 549-622 | Delete subtree |
| 628-767 | 649-788 | Merge empty container |

This is local byte identity, not equality of the complete revisions,
dependencies, other writers or behavior. Full captures and comparison hashes
do not transfer the old reader's personal credit. The present reviewer's
prospectively registered personal displays and unread complements are listed in
`SOURCE-READ-RECEIPT.json`; some displayed lines were test source text only
and no tests were executed.

## Source findings

1. **The ordinary death path has a specific sequence.** In
   `character.rs:361-425`, a new death inserts the death record, runs
   `cleanup_fireplace_custody_for_death`, persists the corpse, updates the
   character as dead, settles incoming pending inheritances and records the
   decedent's estate. An already recorded death instead reruns fireplace
   cleanup and returns the recorded death. Thus cleanup preflight precedes its
   own dish and station mutations, but the death-record insert has already
   occurred in the new-death path; do not describe the entire outer operation
   as mutation-free before cleanup. The code comments describe reducer
   transactions, but this review did not execute or independently establish
   rollback, interleavings or error outcomes.

2. **Destination and subsequent estate treatment differ.** Cleanup validates
   each station and plans all returns before deleting the character's dishes
   (`food/fireplace_custody.rs:150-253`). An available recorded party
   destination is used; otherwise an existing character receives the vessel
   into personal custody; if neither exists the plan abandons the installed
   tool. The no-destination case is conditional and the normal new-death path
   first required the character row. Personal return is before the later
   estate decision; `continuity.rs:382-430,449-489,569-615` shows an estate
   may settle immediately when the heir frontier permits, or remain pending,
   and personal root objects are rehomed through the subtree helper on actual
   transfer. Party-returned objects are not part of that displayed personal
   estate scan. This does not prove every chronological entry point, delayed
   route, exception or source state is supported.

3. **Preflight and rehome have distinct guarantees.**
   `inventory_container.rs:286-350` checks destination existence, enumerates
   the graph, resolves custody, requires carried backing rows to exist and have
   quantity one, and admits the fixture-held root. `371-526` changes owners
   for same-scope rows; for cross-scope descendants it creates a new row and
   moves food, measured amount and condition when those states exist. It
   calls the preflight again and retains the physical object ID. It does not,
   by these displayed statements alone, establish every side-row admission,
   absence of aliases, helper failure, graph invariant or database effect.
   `528-601` separately validates exact backing for all enumerated objects,
   reverses that list and removes attached state and objects. Whether that
   reverse is child-first depends on the uninspected graph traversal order.

4. **Installed-root side state and initialization need a policy boundary.**
   Placement requires the physical object and its resolved carried custody,
   removes its root inventory row, puts the stable object at the fixture and
   records item, object and source custody (`food/fireplace_custody.rs:508-565`;
   `inventory_container.rs:914-929`). Normal retrieval and death cleanup create
   a new quantity-one root row, update the object and then rehome the subtree
   (`fireplace_custody.rs:253-324,569-655`). Those displayed root-row paths do
   not explicitly transfer the old root amount or condition or call their
   initializers. The subtree helper's conditional amount/condition moves apply
   when a carried row actually crosses scopes; the recreated root is already
   in the destination scope before that helper. `inventory_amount.rs:33-60,
   163-205` and `repair.rs:113-134` show which side-state mechanisms exist.
   The three authored method-approved vessel entries are `simple` catalog
   items (`content/items/catalog.yaml:1544-1581,4765-4782`), but the mapping
   defaults, every supported writer and database side-row deletion semantics
   were not established. No supported lost-condition, lost-amount, orphan or
   conservation incident is asserted. Establish whether those root side rows
   can validly exist and how they survive placement before classifying this
   connected question.

5. **No-object stations need an explicit supported-state decision.** The
   station validator accepts an absent object even when an instrument item and
   return custody are present (`fireplace_custody.rs:25-59`); death cleanup
   then uses a separate item-only return branch (`253-324`). The inspected
   regular empty-roast station constructs neither item nor object
   (`474-491`, `food/cooking.rs:219-242,599-608`), while the displayed vessel
   writer constructs both (`fireplace_custody.rs:508-565`). A registered
   fixed-string search across the module found those source literals and no
   other `instrument_object_id: None` literal, but it does not exclude
   indirect writes, external persisted input, earlier data or dynamic paths.
   A supported item-present/object-absent producer and intended policy remain
   unproved; do not infer a compatibility requirement.

6. **Normal retrieval has a separate merge stage.** Once the vessel is
   rehomed, `fireplace_custody.rs:652-655` removes its station and invokes
   `merge_empty_container`. The displayed merge validates root and candidate
   custody, excludes nonempty or nested containers, checks instance condition,
   uses checked quantity addition and removes merged object identities
   (`inventory_container.rs:628-767`). Its lookup and all item/amount/native
   policies remain separate dependencies; no runtime merge or complete
   conservation proof was performed.

## Connected historical and current records

The separately selected original 059 q2286 is
`PL13-241-ESTATE-SUBTREE-CUSTODY-AND-LIFECYCLE-PENDING`, also open. Its
historical estate/death and native authority question overlaps this source
family, but this review does not assign an alias or close it. Original 059
q2318, `PL13-247-COOKING-MATERIAL-AMOUNT-NATIVE-PENDING`, is a separate open
material/amount/volume and provenance question which q2319 lists as a
dependency. Preserve both original revisions and attributions.

The published `ASSESSMENT-L01-PARTY-MERGE-FOOD-95E.json` separately
classifies a conditional, source-derived successful party-merge food
reinitialization question, with actual reducer execution unverified. It
explicitly says this q2319 subtree lifecycle record is related but does not
establish that distinct first-failure question. Its remaining nested-order,
side-row and alias work remains open. This packet does not independently
reassess that party-merge finding or assert a new unique ledger item.

Outstanding work includes supported station writers and admission, root and
descendant side-row lifecycle, all native exception paths, graph ordering,
chronological entry points, current aliases and product policy. The evidence
is source-only and does not close the full audit or authorize implementation.

## Recovery and verification limits

The registered historical selectors, exact queue slices, eight recovered
records, source captures and displays, local-range identity comparison,
related assessment selection, locator results, transport metadata and
`VERIFY-EVIDENCE.py` are included. Locators return names and lines only and
confer no source-body credit; output is bounded to at most 100 results per
pattern and the lifecycle fireplace search omitted 19. Console displays show
only the first 20, 25 or 30 as recorded in the respective registrations. Full
source captures, hashes and chronology are not proof of comprehension or
behavior. Bounded privacy scans do not clear hidden originals, unexamined Git
history, unknown secrets or unsupported formats. No product execution, tests,
builds, fixtures, runtime probes, repairs, ledger edits or publication-tree
changes were performed.

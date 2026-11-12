# Bounded q2329 inventory diagnostic and terminal boundaries

Fresh source-only review by `/root/verifier_scope`, pinned to product revision
`95e63628f9d70b8dd6e6fede64334696c6995009`. This is not a thirteen-lens
closure, a ledger decision, repair, or product validation. No product code,
tests, fixtures, builds, graphics, or runtime probes were run. Verification of
these evidence files is separately recorded.

## Recovered history and attribution

The original 059 queue at evidence revision
`56f24f6457f6cec8ffcdc46f03f5ff26a8abb50e`, `/entries/2329`, records
`REVIEW-L11-CREATOR-INVENTORY-TERMINAL-ERROR-BOUNDARIES` as
`documented-conditional-policy-question`, with discovery closed **within its
exact named scope**. It explicitly does not close the coarse parent or all
lenses, complete the global alias inventory, pass repairs/runtime, or grant
body/population credit. It distinguishes internal adapters from terminal
presentation and asks what structured diagnostic contract application-only
loadout/review adapters need. It preserves earlier predicate-origin questions
as separate owner work.

The exact entry also matches the separately identified reconstructed public 059
entry at evidence revision `26e360920b7b2ec32d5c76238623eaf6d6617d7e`; the two
whole streams differ. Neither is recovered checkpoint 3821. Seventeen original
records directly cited by this q2329 entry matched their recorded hashes and
were copied as evidence. The original e275 delegated `lens11_reconcile` agent
recorded the personal reads, and the later root records adopt registrations
without claiming a root reread. The source comparison matches all six cited
e275 files to 95e byte for byte, but does not equate revisions, dependencies,
semantics, or read credit. See the historical selections, identities and
`SOURCE-READ-RECEIPT.json`. The transport inventory also matches all 17
records by digest, length and relative suffix; that is metadata, not proof of
independent off-machine custody.

## Current source trace

- `inventory.rs:69-84,97-135` defines `EquipConflict`: occupied and unsupported
  variants carry structured fields; missing catalog placements, a mismatched
  selected identity, and an incompatible design are constructed as
  `Invalid(String)` at their respective predicates. A typed earlier cause is
  not present in the shown `Option` or predicate. Other validation routines
  likewise format their own errors (`inventory.rs:174-198`); this does not
  establish every upstream error type or global alias.
- `inventory.rs:308-334` retains `(culprit InventoryItemId, EquipConflict)`
  while fitting worn items. `inventory/occupancy.rs:123-169,240-277` builds
  the graph and returns structured unsupported/occupied cases, otherwise
  `Invalid`. The core `EquipmentGraph::equip` already returns `&'static str`
  for its predicates (`adventuresim-core/src/equipment.rs:216-278`), so this
  adapter is not dropping a typed core error. `Inventory::loadout` first fits
  then performs its additional wearable/surface and design checks, retaining
  the item id and conflict (`inventory/loadout.rs:45-108`).
- `Inventory::explain` and `EquipConflict::describe` explicitly render names,
  locations and channels as presentation (`inventory.rs:336-351,370-394`).
  The inventory summary uses it at the UI (`inventory_ui.rs:46-69`), while
  `outfit::loadout` converts the tuple to presentation text inside an
  application `anyhow` error (`outfit.rs:13-22`). That adapter is used by
  Studio regeneration (`studio_generation.rs:32-59`) and character export
  (`character_export.rs:8-16`); errors then reach Studio status
  (`studio_ui.rs:242-267`) or the CLI return path (`main.rs:250-255`).
  `review_export::worn_pieces` separately debug-formats `resolve` and
  `design_for` conflicts before returning its application error
  (`review_export.rs:75-95`, `main.rs:219-232`). Whether a structured cause
  must survive these application adapters is the recorded policy question;
  final human-readable status alone is not an automatic defect or alias.

## Conditional observations requiring central reconciliation

1. **Culprit identity can be dropped before presentation.**
   `Inventory::wear` calls `fit`; its general error arm destructures away the
   returned culprit id, restores the previous items, and returns only the
   conflict (`inventory.rs:264-285`). `inventory_ui::wear` constructs the
   explain tuple using the requested id (`inventory_ui.rs:186-214`); the
   checkbox supplies that id (`inventory_ui.rs:111-155`). If a different,
   already-worn item is the first invalid culprit, the status names the
   requested item while the conflict describes the other failure. A nonempty
   nonexistent catalog placement illustrates a source-level condition:
   recipe validation checks structural item data and duplicate inventory IDs,
   not catalog resolution (`lib.rs:198-221`, `inventory.rs:174-198,353-360`),
   and the main path passes the validated recipe to Studio
   (`main.rs:219-223,258-294`). This is conditional static reasoning, not an
   executed recipe, proof of an ordinary player encounter, severity decision,
   or automatic ledger alias. The first lost value is the `wear` return tuple,
   before `explain`; q2330's separate fresh-ID arithmetic is not this operation.

2. **The creator graph adapter can substitute a recomputed cause.**
   Core `equip` checks duplicate internal body occupancy before checking other
   nodes; on *any* core error the creator adapter searches placed candidates
   for a body conflict and prefers `Occupied` over the returned reason
   (`equipment.rs:229-258`, `inventory/occupancy.rs:254-274`). If both
   conditions occur, this changes the first returned graph reason into an
   occupied variant; it does not mean the occupied relation itself is false.
   The separate core catalog validator rejects pairwise conflicting occupancy
   (`item_catalog_validation_occupancy.rs:58-139`), so that example is blocked
   when that validator is actually applied. The inspected creator loader,
   however, directly deserializes `ItemCatalogDocument` and its `new` path
   checks armor recipes (`equipment_catalog.rs:45-69,113-121`,
   `item_catalog_schema.rs:15-20,88-113`); those shown calls do not invoke that
   separate occupancy validator. A fully specified accepted catalog/recipe,
   intended error-priority policy and practical effect remain unproved. Do not
   conflate this adapter's recomputation with final text formatting.

3. **Review output precedes recipe-item resolution.**
   In recipe review mode `review_export::export` creates the directory and
   writes `body.json` before `worn_pieces` resolves worn catalog placements
   (`review_export.rs:22-45,75-90`). Main's recipe validation is separate and
   catalog-independent as described above. If earlier model/body generation
   and file writes succeed and a later placement/design resolution fails,
   this ordering leaves an earlier write before the returned error. Later
   piece writes are also sequential. No file was written by product execution;
   partial-output policy, preexisting destination contents, atomicity
   expectations, and any actual impact remain open. This is an output-commit
   question, not a newly asserted error-formatting alias.

These three observations have different first operations. The coordinator owns
cross-lens deduplication, any accepted finding count and the repair ledger.
None changes the original q2329 record or establishes checkpoint 3821.

## Connected historical/current records

Original 059 q2330 is a separately scoped, retained source finding about
`Inventory::add` successor exhaustion and an unresolved canonical alias; it
is not the `wear` culprit-identity issue. The separately attributed existing
95e Lens 1 inventory assessment covers that successor and same-article
attachment capacity as partial traces, with no new unique historical count.
The collaborator assigned q2330 is tracing its producer and consumer paths
independently. Original q1748 and q1893 are broader pending catalog/equipment
identity and conflict reviews; q1873 concerns other inventory/HTTP predicate
and presentation paths. These are related searches or distinct consumers,
not established aliases. The 059 field and whole-entry locators are bounded;
a negative search is not a global alias inventory.

## Evidence and limits

The receipt lists 31 personally displayed 95e chunks across 18 captured files,
including unread complements. `DISPLAY-QUALIFICATIONS.json` preserves the
initial unregistered, truncated handoff README navigation, the original
reviewer's truncated navigation qualification, and test source displayed
without execution. All locators have explicit scopes and output limits.
`VERIFY-EVIDENCE.py` checks exact packet contents, source hashes, registrations,
reader/root roles, selected records and, with supplied repositories, fixed Git
membership and transport metadata. A separately sealed privacy scan receipt
binds the packet manifest. Neither timestamps, hashes, source-only reasoning
nor scans prove comprehension, supported inputs, runtime behavior, product
validation, independent custody, or completeness outside the bounded scope.

# q2321: preparation projection and material quantum

## Scope and historical status

This is a bounded source-only follow-up to recovered 059 queue entry 2321,
`PL13-252-PREPARATION-VIEW-MATERIAL-QUANTUM-PENDING`, against pinned revision
`95e63628f9d70b8dd6e6fede64334696c6995009`. It does not close the pending
contract, a lens or the queue; assign a unique defect count; or restore the
missing checkpoint 3821 final queue or ledger.

The original and path-redacted reconstructed 059 entries are byte-identical;
their containing queues are not. The earlier Lens 13 reviewer personally
assessed frozen `food/projections.rs:1-538` at revision
`6533e444fa37552927070cc8d8190907fd3b413e`, with root adoption reporting no
root personal reread. Its later granular source-reasoned quantum case and own
reuse proof are separate historical evidence: that discovery specifically
left strict root source adoption, current reconciliation, aliases and actual
browser presentation open. The earlier catalog review had a recorded gap at
lines 4582–4611; the selected historical apple is outside that gap. All 26
directly referenced historical records, the indirect reuse proof, the original
verification and eight exact historical source objects are hash-bound here.
No historical review is promoted into a fresh 95e read.

The seven captured non-catalog support files are individually byte-identical
between the two revisions; the catalog and commit root trees differ. The old
and current apple entries both specify `weight_kg: 0.18`, but that selected
fact does not imply catalog or tree equivalence. Exact Git chains bind 19
current and eight historical source paths. `READ-LIMITS.json` lists only this
reviewer's actually displayed current and historical source windows.

## Source-supported conditional case

The view can offer an action for a positive retained food amount that the
corresponding preparation authority cannot represent as a nonempty
whole-milligram material measure. The following trace is conditional on the
actor, fireplace, object, custody and other unrelated native guards succeeding
and on unchanged state at preparation. It is not a product execution or an
observed browser event.

* The current catalog defines an apple of 0.18 kg (`content/items/catalog.yaml:335-352`). Core maps catalog `weight_kg` to `FoodDefinition.mass_kg_per_unit`
  (`crates/adventuresim-core/src/food.rs:247-310`). A normal food grant creates
  individual quantity-one rows and initializes a whole measured amount
  (`stdb-module/src/item.rs:340-400`, `inventory_amount.rs:13-20,33-76`).
  Lot creation records raw fruit with that mass, revision one and a stable
  object (`food/lot_inventory.rs:11-93`).
* The native loose cooking reducer accepts a `u32` micros fraction, admits
  positive selections no larger than available, and on partial selection
  retains the lot proportionally and stores the checked remaining fraction
  (`food/cooking.rs:73-102,281-365,508-543`,
  `core/src/inventory_measurement.rs:30-44,89-94`,
  `food/lot_inventory.rs:225-241`, `core/src/food.rs:548-554`). A selection of
  999,998 of 1,000,000 micros leaves two micros if the other guards pass.
  Its selected near-whole apple mass lies within the core cooking duration's
  positive, at-most-25 kg mass range (`food/cooking.rs:420-490`,
  `core/src/food.rs:371-392`).
* The ideal residual is 0.18 × 2/1,000,000 kg, or 0.36 mg. The implemented
  `f32` division/subtraction and multiplication perturb that value. Without
  executing product arithmetic, a conservative source-reasoned bound is under
  0.4 mg: both integer operands are exactly representable, the near-one
  division error is below 2^-24, and the 0.18 kg `f32` is below 0.181 kg.
  The residual remains positive and below the 0.5 mg rounding threshold.
* For an eligible carried raw lot, the gateway view emits a Grind plan using
  the lot/object/revision/custody and skill/duration without calling the
  material snapshot admission (`food/projections.rs:288-475`). The checked
  authority converts the current lot mass to whole milligrams and constructs
  `MaterialMeasure(mass, 0)` (`food/preparation_authority.rs:168-193,404-418`).
  `Milligrams::try_from_kilograms` rounds; `MaterialMeasure::try_new(0, 0)`
  rejects `EmptyMeasure` (`core/src/material.rs:20-35,264-288`). The exact
  native preparation reducer loads that authority before planning/waiting
  (`food/ingredient_preparation.rs:292-363`). Thus the unchanged tiny lot can
  be offered by the view but rejected at the checked material boundary.
  `EmptyMeasure` is an intentional nonempty-material guard; this evidence does
  not justify weakening it or silently dropping or inflating the lot.
* The current web route queries the view for the active actor
  (`training_activity.rs:294-306,326-335,405-417,434-443`). Personal and party inventory templates render
  matching Grind buttons and tuple-backed forms (`templates/settlement/trade/discard.rs:67-91,120-145`, `party_pool.rs:31-58`). The router binds the
  form action to the handler (`routes/settlements/router.rs:386-391`), and the
  handler forwards the tuple to the reducer and returns an error response on
  reducer error (`routes/settlements/party/ingredient_preparation.rs:16-51`).
  This is a source trace, not an observed button, request, or error. The
  specific ability to create the 999,998-micro selection through web controls
  was not established; the native reducer accepts that input subject to guards.
* The personal inventory count cell renders `item.quantity` (`discard.rs:73-81,107`), whereas the remaining fraction is a separate native amount and
  the row stays quantity one. This is a bounded presentation observation; it
  does not prove that all other UI hides the amount or establish a separate
  defect or count.

The material quantum and whether the view should suppress such an action or
represent it differently require an explicit product/native policy. A supported
native input and the intended preview guarantee should be resolved before a
repair direction or acceptance criterion is finalized.

## Related work and remaining questions

The independently attributed q2320 packet, manifest SHA-256
`321188e137578c2c7bc2579896bc107cdb9799710f733a50687d972cfe545939`, reviews
preparation's guarded authority and timing. Its conditional MAX-clock and
high-load questions are separate from this view/quantum trace; shared source
is not proof of an alias. Its assessment is copied and digest-bound as related
work, without claiming its reader's coverage as this review's coverage.

A time-bound central 95e ledger snapshot is included. Selected entries for
preparation rights, cooking-water precision and vessel nested-content wording
are related by source or context but have different stated contracts. Literal
searches did not identify this exact q ID or the selected quantum terms; that
is not a semantic alias search or a new count. Complete producer reachability,
all native material and UI paths, truthful presentation policy, full current
alias reconciliation and runtime behavior remain open.

The first combined historical-entry tool output was truncated; the exact
numbered files and separately registered key metadata are retained. A shallow
metadata projection's null discovery field was corrected to its exact nested
pointer. The first q2320 manifest lookup failed due to its list shape and was
retried. Truncated literal locator lists and unpinned physical navigation are
not coverage. See `READ-QUALIFICATIONS.json`. No product code, product parser,
build, test, fixture, repair, or runtime probe was executed; displayed test
source was read only as text.

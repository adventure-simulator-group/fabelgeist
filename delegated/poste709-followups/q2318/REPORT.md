# Bounded review of historical 059 q2318

## Scope and attribution

The selected original 059 entry `/entries/2318` is
`PL13-247-COOKING-MATERIAL-AMOUNT-NATIVE-PENDING`. It remained
`open-connected-source-current-alias-qualification`: discovery and repair or
runtime were false, with zero new body or population credit. Its adjacent
preview question, `/entries/2317`, was also open. These are original historical
records, not checkpoint 3821 or current closure. Original queue revision
`56f24f6457f6cec8ffcdc46f03f5ff26a8abb50e` and public path-redacted
reconstruction `26e360920b7b2ec32d5c76238623eaf6d6617d7e` have different
full stream hashes. Each pair of exact q2317 and q2318 entry slices is byte
identical, separately recorded and bound by offsets and hashes.

The prior Lens 13 reader recorded full personal assessments of the frozen
`6533e444fa37552927070cc8d8190907fd3b413e` core food and module cooking
files. The separately recovered root adoption records preserve that reader's
attribution, record no root personal reread, and do not credit current source
or runtime. The cooking reader's receipt said root adoption was then pending;
the later root adoption is a separate event. Eight cited proposal,
qualification, registration, read and adoption originals match their recorded
hashes; the published transport inventory independently contains a matching
size, digest and relative suffix for each. This metadata does not establish
independent off-machine custody.

Both frozen source files are byte-identical at pinned product revision
`95e63628f9d70b8dd6e6fede64334696c6995009`. This only compares those two
files; it does not prove the entire revisions, their dependencies, or their
behavior equal. The present reviewer independently displayed only the exact
ranges in `SOURCE-READ-RECEIPT.json`; undisplayed bytes and the historical
reader's full-file work are not credited to this reviewer. This is a bounded
source assessment, not a claim of global q2318 closure or product validation.

## Findings and qualifications

1. **The shared duration guard has a defined, narrow boundary.** Core
   `food.rs:371-410` rejects empty safety input and nonfinite, nonpositive or
   above-25 kg total mass; it uses checked setup, maximum safety and batch
   addition. Module `food/cooking.rs:313-315,359-391,420-490` rejects zero,
   duplicate and unavailable selections, checks eight listed lot fields for
   finite nonnegative values, adds water mass and requests the shared duration
   before the explicit cooking water and inventory mutation block begins at
   line 498. It does not establish bounds or admission for every other numeric
   field, load, producer or output.

   The earlier qualification's broad phrase about the first mutation must not
   be read as a guarantee of no earlier state writes: the operation calls
   `initialize_character_condition` at line 294, and `condition.rs:152-180`
   shows that helper can insert state when missing. Only the explicit cooking
   water and inventory block follows the duration guard. The transaction's
   rollback behavior, all helpers, and actual execution were not assessed.

2. **Amounts and derived totals still depend on native producers.** On the
   displayed path, `cooking.rs:281-285` limits the non-vessel lane to 32 lots
   but exempts the vessel lane from that specific count limit. The operation
   checks the listed per-lot values and positive selected fractions, then
   accumulates mass, nutrition, value, flavor, loads and provenance
   (`313-447`); it stores calculated fields and contribution vectors in a dish
   (`541-598`). The mass check does not itself prove that the other aggregates
   and source vectors are finite, consistent, or bounded. Whether supported
   producers or other admission rules prevent problematic combinations remains
   unproved. This is a conditional source question, not a demonstrated
   reachable bad meal or conservation failure.

3. **Vessel scope is direct children in the shown loop.** The vessel entry
   locates the canonical fixture and return custody, then iterates edges whose
   parent is the specified container (`cooking.rs:110-180`); nested lots are
   not visited by that loop. Lines 105-108 describe food at any nesting depth,
   while lines 150-151 expressly say direct contents. This source comment
   discrepancy and the intended product policy need separate reconciliation;
   no nested-container operation was run. The displayed code does not prove
   volume/custody bounds in every producer or helper.

4. **Output identity has a conditional index assumption.** Retrieval creates
   a row and, for a vessel, an object/containment edge, then calls
   `ensure_food_material_object` before inserting the output lot
   (`cooking.rs:690-758`). The helper requires a quantity-one row, reuses a
   matching object or inserts one (`lot_inventory.rs:11-57`). Its
   `object_for_row` lookup searches the current row's item index and rejects a
   second matching object (`inventory_container/object_lookup.rs:10-55`). If
   state could contain an object with the same row location but a different
   item, that object would be outside the index search while the row exists;
   the local mismatch check alone does not prove absence of that state. No
   supported producer, fresh-row collision, database behavior or transaction
   that could create such state was established. Check all relevant creators
   and custody/admission invariants before treating it as reachable or unique.

5. **Preview remains a separate historical question.** `preview_cooking`
   (`cooking.rs:819-868`) uses personal inventory, raw definition safety
   and selected ingredient mass. The actual path applies preparation factors
   and adds contained water and vessel-specific method
   (`cooking.rs:281-291,381-391,449-490`). A fixed-string search for the exact
   name across `crates` at 95e located its definition and two references in
   test files only; no production call was found by that limited search.
   Dynamic/indirect names and other scopes were not ruled out. The gateway
   projection `food/projections.rs:506-538` carries the already stored dish
   target; the UI route inspected at `cooking.rs:446-483,532-553` calls the
   start/retrieve reducers, not this preview. Thus the arithmetic distinction
   corroborates q2317 but does not show an actual user-visible mismatch.
   Decide whether a supported preview exists and whether it promises an exact
   target or an estimate before classifying the behavior.

## Related assessments and remaining work

The published `ASSESSMENT-L04-COOKING-WATER-PRECISION-95E.json` separately
identifies a conditional binary32 exact-volume comparison for hypothetical
larger vessels. It explicitly leaves q2318 broad, states the currently authored
vessels are below its demonstrated threshold, and reports no execution or
validation. Preserve that narrower result, its authorship and its open
capacity and producer qualifications; this review has not independently
reproduced its catalogue or arithmetic investigation.

`ASSESSMENT-L1-MICROBIAL-AGGREGATES.json` is a separately attributed earlier
assessment at `c100638b720a7281a5fc96f6a3c5c23d5dda85da`, not a current
95e assessment by identity. It describes a conditional invalid-load policy
question and expressly did not establish a supported unsafe path. Further
source comparison and aggregate bounds are required before relying on it as
current or deduplicating it with q2318.

Unresolved work includes native lot creation/transfer/amount and object
admission; vessel and water capacities, provenance and conservation across all
writers; actual preview callers and policy; current first-operation aliases;
and exact relationships to other ledger findings. This packet does not decide
those issues or assign global ledger closure. No product code, fixtures,
builds, tests, runtime probes, repair or implementation was run.

## Recovery and verification limits

`HISTORY-REGISTRATION.json`, `Q2317-REGISTRATION.json`, the source and related
registrations, exact displays, captured byte objects, read receipts, locator
results, recovered historical records, and `VERIFY-EVIDENCE.py` provide a
portable evidence trail. Fixed-string locator output has explicit caps and
omissions in `DISPLAY-QUALIFICATIONS.json`: the broad preview and inventory
searches omit many results and console output showed at most 35 per pattern.
One historical metadata structure display was truncated; no unseen content was
credited, and selected rows and compact keys were subsequently displayed. One
source-display filename lookup failed after EOF clamping and was corrected.
A restricted predecessor with two local path candidates was sealed; the safe
successor omits only the two self-authored helper scripts, with the crosswalk
in `PRIVACY-BOUNDARY.json`. The scans are bounded and do not clear unexamined
Git history, hidden originals, unknown secrets or unsupported formats. Hashes
and timestamps cannot prove semantic understanding or actual runtime effects.

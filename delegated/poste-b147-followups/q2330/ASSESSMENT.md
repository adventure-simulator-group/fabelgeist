# Bounded q2330 reconciliation: inventory ID successor

Reviewer: `/root/mhr_character`. Current source is pinned to
`95e63628f9d70b8dd6e6fede64334696c6995009`. Historical 059 q2330 records a
source finding at `e2755a732c84e67296b5e3441a4259a84f461b2f`, with exact
canonical alias and root acceptance pending. The e275 inventory source, pinned
95e source, original frozen role 809 object, and the individual file at Git
`6533e444fa37552927070cc8d8190907fd3b413e` have identical bytes and SHA-256
`0f820bca06e27b164da010e95dc7d3500a79a6feb0b7be9411ed8a41e88feb18`.
This proves only that file's identity: the earlier `pr-working` snapshot had
unrelated modifications and is not established as a clean tree. No historical
evidence is promoted to checkpoint 3821.

At 95e, `inventory.rs:23–41` exposes a transparent `u64` article ID and a
serde-deserializable inventory. `Inventory::validate` at `:353–360` checks
unique existing IDs and article validity, but places no upper bound on the ID.
`CharacterRecipe::validate` calls it (`lib.rs:153–164,198–222`). Startup parses
and validates the recipe before passing it to the studio
(`main.rs:183–223,258–294`); a successful explicit recipe load is also assigned
to the studio (`main.rs:375–379`, `studio_ui.rs:211–231`). An acquisition UI
handler ultimately calls `Inventory::add` and selects its returned ID
(`inventory_acquire.rs:100–119,126–137`). Those source paths are conditional
on an otherwise valid recipe and successful prerequisite initialization; no
particular loaded recipe or UI event was observed.

For the numeric component of that external boundary, the current lock pins
`serde_json` 1.0.151 and `serde_core` 1.0.229. The archived crate bytes match the
lock checksums. The selected `serde_json/src/de.rs:101–105,109–128,319–335,
462–515,1341–1353,1500–1507` preserve a nonnegative integer as `U64` through
`u64::MAX`, dispatching it to `visit_u64`; the overflow predicate switches
only when the next digit would exceed that maximum. The selected
`serde_core/src/de/impls.rs:129–161,436–440` show the `u64` visitor returning
that value. This is source reasoning about the numeric step, not a parser or
macro execution, feature-resolution claim, complete JSON witness, or product
validation. The source declaration uses serde's transparent wrapper; generated
derive code was not separately expanded or personally inspected.

`Inventory::add` maps **each** existing ID to `id.0 + 1`, takes the maximum,
and only then pushes the new article (`inventory.rs:225–239`). For the
symbolic distinct existing IDs `MAX-1` and `MAX`, checked overflow would panic
before the push. Under wrapping overflow, their mapped successors are `MAX` and
`0`, so `max` returns the already occupied `MAX`; the new item would duplicate
that ID. The build's effective overflow behavior, an actual input recipe and
player impact are unverified. `get` and `get_mut` return the first matching
item (`:213–219`), making later ID consumers potentially ambiguous after a
wrapped duplicate; their runtime effects are not established. This is an
admitted high-ID boundary question, not a claim of ordinary UI growth to 2^64
items. A future owner must choose an admission bound, explicit exhaustion, or a
fresh available identity strategy; this packet does not prescribe a repair.

## Provenance and relationships

Recovered 059 q2330 is stored both as its exact original byte slice and as a
reconstructed JSON projection. The selected object equals the corresponding
object in the later public reconstructed queue; whole-queue bytes differ. The
historical e275 personal reader was `parallel lens11_reconcile`, with separate
root `segovb` adoption: `READ-180 /rows/2`, `PLAN-198 /rows/2`,
`READ-L11-716 /rows/2`, and its verifier are preserved with exact whole-file
hashes. The original record also names the drape worker and preview; their
historical reader ranges are preserved as historical context only, and were
not re-read as current q2330 source.

The independently recovered historical `PL13-116-INVENTORY-ID-SUCCESSOR`
proposal points to the earlier PL13-113 full personal read of frozen
`pr-working` original role 809 (`READ-PL13-113 /rows/0` and its prospective
registration `/sources/0`). The PL13-116 reader's fresh sources were roles
819–821, not a new reading of role 809. Its proposal says the exact aliases and
current reconciliation remained pending, with zero accepted unique increment.
The 059 L11 reader, PL13-113 reader, and PL13-116 reviewer are distinct
recorded scopes; a specific individual is not inferred where the evidence does
not name one. Current committed ledger entry 59,
`REVIEW-CREATOR-INVENTORY-ID-SUCCESSOR-001`, identifies this as a bounded trace
of the existing source question, with player impact and wider aliases open.
Neither this review nor the earlier acceptance requirements change a count,
close a lens, prove a repair, or resolve all relationships.

`READ-LIMITS.json` and source-view receipts distinguish our current bounded
reads from full source bytes retained for proof, and enumerate unread
complements. The shown `main.rs:466–535` is test-source text and was never
executed. A metadata-only view of an earlier native registration was
unregistered and truncated, and exposed a local cache path in the tool
transcript; no native-body credit is drawn from it and its value is not copied
into the packet. A helper first selected an invalid scratch Git path; another
support-capture attempt exceeded its planned source EOF before registration or
display. The separately recorded accidental publication helper write was
removed after an identity check, with its bytes and pre-existence limitation
preserved in `PUBLICATION-WRITE-INCIDENT.json`. Subsequent evidence output used
a second exclusive scratch and absolute output paths.

## Open boundaries and lens scope

- Admission, operations, aggregate state, ownership and producer propagation:
  the selected high-ID source case reaches the first successor arithmetic;
  all alternate producers, graph consumers and exact aliases remain open.
- Behavior preservation and redundant authority: after a collision, matching
  IDs can select the first stored article. No execution or consumer effect is
  claimed; intended exhaustion and stable-identity policy remain open.
- Existing type owners, granularity, enum shape, derivability, magic numbers
  and strings: this focused identity trace does not establish broad findings or
  clean-lens coverage in those areas.
- q2329's terminal/error review and q2328's drape-worker review remain
  separately attributed, with their first operations distinct.

No product code, parser, macro, fixture, renderer, build, test, probe or repair
was executed. Verification establishes evidence identities and selected source
relations, not product validation or overall audit closure. The two retained
exact upstream crate archives contain nine email-pattern occurrences in
published authorship/contact metadata; their contexts and locations are
recorded with email values redacted in `UPSTREAM-EMAIL-REVIEW.json`. They are
not user-home paths, and unknown sensitive patterns remain outside the bounded
scan. The original archives are retained unchanged to preserve their locked
checksums.

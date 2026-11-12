# Bounded source assessment: measured zero and contained water

Reviewer: `/root/verifier_scope`. Product source revision:
`95e63628f9d70b8dd6e6fede64334696c6995009`. This is source-only work and
is not a lens, queue, repair, product-validation, or checkpoint-3821 closure.
No product code, build, test, fixture, or runtime probe was executed.

## Original 059 records and later historical work

The original 059 queue at evidence revision
`56f24f6457f6cec8ffcdc46f03f5ff26a8abb50e` has q2334
`PL13-256-MEASURED-ZERO-VOLUME-OPTION-PENDING` and q2335
`PL13-256-CONTAINED-WATER-VOLUME-AND-PROVENANCE-PENDING`. Both were
`open-connected-source-current-alias-qualification`, with
`audit_discovery_closed=false` and `repair_or_runtime_passed=false`. The
individually captured q2334 and q2335 entry slices exactly match their selected
counterparts in the independently designated public reconstruction at
`26e360920b7b2ec32d5c76238623eaf6d6617d7e`; the complete streams differ.
Identical slices do not make that reconstruction an original, or recover 3821.
`HISTORY-IDENTITIES.json` records exact offsets and hashes.

The frozen original review is attributed to the historical Lens13 delegated
reviewer `PL13-256`, not to this reviewer. Its registration and read receipt
identify original row 1119, frozen `pr-working` Git
`6533e444fa37552927070cc8d8190907fd3b413e`,
`crates/adventuresim-stdb-module/src/inventory_container.rs` SHA-256
`1e8cafa848ca3965b804ce9d1766c9ea2d50d6a0cc123467ff317644e96c30bc`, lines
1–1451. The claimed union was prior 1–660 plus new 661–1451; it was not a
full reread. Historical root adoption `READ-L13-PARALLEL-254` was recorded at
2026-10-30 01:59:44 UTC and embeds that attribution. It does not transfer
personal source-reading credit to the historical root or this reviewer.
Selected older core rereads were narrower than prior full reads, as the
historical qualification states in `ATTRIBUTION-FOCUS-PRIMARY.json`.

Later historical `READ-L13-CURRENT-TEAM-002`, `003`, and `004` separately
adopted selected Lens13 source work; `003` and `004` identify e275 as their
then-current source. The 003 proposal records an Apple 225 mL / retained two
millionths source chain and considers that selected local zero-versus-absence
contract documented, while preserving supported-placement, capacity,
catalogue-wide delta, alias, current-global and runtime qualifications. The
004 subtree/depth record adds two current-source ranges to a qualified prior
union. Neither changes either 059 queue status, confers other-lens credit, or
promotes older evidence into 3821. These are historical dispositions, not our
new independent full producer-chain proof. See `ATTRIBUTION-FOCUS-CURRENT.json`.

The six historically cited product files have 6/6 expected frozen hashes;
5/6 have equal bytes at e275 and 95e. The native container file differs:
`DIFF-INVENTORY.txt` records movement of `object_for_row` to the separate
`inventory_container/object_lookup.rs`. That split has been inspected, but
byte equality of selected files does not establish equality of complete
revisions or their callers.

## q2334: measured zero and absent measurement

At 95e, `inventory_amount.rs:33–46,90–123` and `inventory_measurement.rs:30–45,
78–80` distinguish an admitted fraction and floor its scaled whole-unit
amount. `inventory_container.rs:932–965` retrieves a personal or party
fraction, returns zero if no amount row is found, selects serving or exterior
mL, and applies that floor. `inventory_container.rs:985–990` translates every
zero result to `None`. In `inventory_containers.rs:29–40`, a measured zero is
admissible; at :58–74 and :92–95 `None` selects the full exterior volume for
occupancy and placement. Thus the adapter loses the distinction between a
missing measurement and a measured zero. `Milliliters` itself admits zero
(`material.rs:74–96`).

The current catalog contains an apple with 225 mL exterior volume
(`catalog.yaml:335–346`); two millionths of 225 mL floor to zero. The
source-only example illustrates the first-operation mismatch, and the
historical selected e275 proposal separately traces an admitted 999,998
millionths cooking selection to a retained two millionths. This review
personally inspected the selection and retention boundaries in
`food/cooking.rs:75–102,313–360,508–554`, but not the full catalog compiler,
food admission, lifecycle, or all possible earlier failures. A concrete
capacity rejection still depends on a supported parent placement and free
capacity. Intended sub-mL occupancy and quantization policy remain open; no
runtime witness or universal capacity failure is claimed.

## q2335: water volume and provenance

`inventory_container.rs:1013–1066` sums matching-custody water with checked
addition and computes remaining capacity from graph solid volume plus liquid,
with checked arithmetic. Filling standard waterskins visits IDs in order,
checks custody and `require_mutable`, skips non-water contents, and uses checked
capacity/addition (:1069–1117). Direct discard checks positive amount, actor
custody, mutability, water identity and available volume, then guarded
mL-to-µL conversion before provenance and public-volume mutation
(:1372–1430; `material.rs:120–138`).

Automatic consumption instead selects water containers in the expected custody,
sorts by object ID, consumes the minimum of requested and available volume,
transfers provenance before decrement, and deletes/merges an emptied container
(:1120–1173). It does **not** itself call `require_mutable`; the guard stated
for filling must not be generalized to consumption. The inspected condition
caller :643–674 uses party then personal custody when needs require stored
water. Whether any supported processing or locked-state overlap can actually
reach that path remains unresolved; no defect, transaction rollback behavior,
or arbitrary u64-max witness is asserted.

`outbreak.rs:128–155,172–238` stores water-output and per-container material
contributions, converts volumes with checked mL-to-µL arithmetic and transfers
proportionally in material-lot order. The separate collection path writes public
volume, contributions and output lots (:592–635); cooking checks contributed
material volume and removes contributions when it consumes vessel water
(`food/cooking.rs:470–506`). This is a selection of the provenance network,
not proof that every writer, component, clock edge, or conversion conserves
volume or contaminant load. Canonical authored volume bounds, initial clean
water representation, all writers, error atomicity, and processing overlap
remain open.

## Connected historical records

* Original q2321 `PL13-252-PREPARATION-VIEW-MATERIAL-QUANTUM-PENDING` concerns
  preparation projection versus guarded material admission. A common quantum
  policy does not establish an alias to q2334's `graph` first operation.
* Original q2299 `PL13-246-FOOD-WATER-EVENT-NATIVE-PRODUCER-PENDING` concerns
  food/water event authority and downstream envelope; it may constrain
  provenance/exposure but is a distinct question.
* Original q2392 `PL13-271-WATER-COLLECTION-TERMINAL-PROPAGATION-PENDING`
  concerns post-time-advance collection completion and transaction semantics;
  it may constrain collection provenance but is not identified as an alias.

These three were selected from the original queue, remain historically open,
and their selected reconstruction slices also match. Only their registered
summary fields were displayed; `RELATED-RECORDS-IDENTITY.json` holds offsets
and hashes. Root owns all cross-lens reconciliation and ledger decisions.

## Evidence and limits

`SOURCE-READ-RECEIPT.json` records this reviewer's 26 exact current displays,
10 personally viewed paths, exact unions and unread complements; the eleventh
captured source has no personal body-read credit. Test-module text was
inspected as text only. The historical attempted catalogue199 full156 display
was truncated and received no additional credit; the replacement and other
historical selected rereads keep their stated limits. A combined historical
metadata output in this review was also truncated and claims no personal read
credit; the two short, registered focused displays replace only their own
selected content. See `DISPLAY-QUALIFICATIONS.json` for locator truncation,
failed attempts, and chronology.

All 36 directly hash-cited original records were recovered with their expected
SHA-256 and matched to same-machine original transport metadata. The capture
and transport matching show byte identity, not independent custody, semantic
adoption or body-reading credit. `VERIFY-EVIDENCE.py` checks the flat packet,
registered source displays, record identities and, when supplied the relevant
Git object store and transport manifest, source and queue membership. Its
optional checks require those external inputs. Local timestamps attest process
order only. The privacy scan is bounded and does not clear unknown secrets,
encodings, external Git history, or unsupported archives. Nothing here is
permission to begin implementation or product validation.

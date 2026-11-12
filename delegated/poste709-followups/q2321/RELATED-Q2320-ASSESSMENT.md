# Bounded q2320 preparation authority review

Independent reader: `/root/mhr_character`. Current product source is pinned to
`95e63628f9d70b8dd6e6fede64334696c6995009`. This source-only review concerns
original 059 queue entry 2320, `PL13-249-PREPARATION-GUARDED-NATIVE-AUTHORITY-PENDING`.
It does not close that queue entry or a lens, implement a repair, execute
product/tests/fixtures, or reconstruct missing checkpoint 3821.

## Historical identity and attribution

The original 059 queue object is the version at publication commit
`56f24f6457f6cec8ffcdc46f03f5ff26a8abb50e`, with whole-object SHA-256
`2e1f4430edb375adff24b6b46d434ac58bf5fd18c0d9dda4cf92fe215e5cfe08`.
`QUEUE-2320-RAW.json` is the exact half-open byte slice `[10478972,10491108)`
of that object. Its parsed content equals both this packet's displayed
reconstructed projection and the matching selected object in the later
privacy-reconstructed queue at publication `a9d979e4f4fd84b797da89a33263d78604fe9224`.
This comparison does not make those two whole queues byte-identical or restore
3821. The full original queue is not embedded; independent verification of the
slice against the whole original requires separately available exact bytes.

The historical PL13-249 delegated reader reported a full personal read of
`ingredient_preparation.rs` lines 1–652, original role 1105, and the historical
PL13-251 delegated reader separately reported a full personal read of
`preparation_authority.rs` lines 1–635, original role 1109. Their respective
registrations and receipts, and the `segovb` root adoption records, are copied
with their recorded hashes in `history/`. Each root adoption explicitly reports
no root personal reread, no accepted unique-count change and no current-source
reconciliation. Preserve the PL13 namespaces and do not invent an individual's
name from them. The later PL13-251 granular view is an update to the same
pending contract, not an automatically separate failure. The historic working
snapshot has unrelated modifications; these per-file identities do not prove a
clean whole tree.

Both recorded original 059 source objects compare byte for byte with the
individual files in Git `6533e444fa37552927070cc8d8190907fd3b413e` and pinned
95e: ingredient SHA-256
`8424b8d20fe3619b57687d4e31b3715b141fe47d21a1f21d7c78ed11958c0b9f`,
authority SHA-256
`4f06d127e8d7d358ded01b2d1b7eba9116644b53146a476dd780e98344689613`.
I independently registered and displayed the complete 95e bodies. The earlier
historical reads keep their original attribution; byte identity does not
transfer them. `/root/wrapped_scope` earlier read the specific 95e preparation
windows recorded in `PRIOR-Q2317-SELECTED.json` as q2317 support, without
assessing this full contract. `/root/verifier_scope` separately reported q2318
cooking/preview source scope; that message is contextual and its reads are not
credited here.

## Conditional source discrepancy: wait at an exhausted clock

With all of the following conditions, source composition yields different
elapsed values for planning and for the wait adapter:

- A valid authorized ingredient-preparation request starts at
  `StrategicMinute::MAX` with its ordinary positive duration, a matching actor,
  lot, object, custody, revision, generation and canonical request.
- The injury and disease previews find no terminal boundary; no earlier
  lifecycle segment changes the path; and the invoked helpers and native
  transaction succeed. No supported path to the MAX clock is established here.

`ingredient_preparation.rs:161–177,332–455` previews and validates the
operation. `calendar.rs:99–105,137–150` and
`strategic_action.rs:318–330,367–410` show that adding a positive duration at
MAX has no representable endpoint, resolves as `ClockExhausted`, and has
`elapsed_minutes == 0`. Nevertheless `herbalism.rs:245–276` emits an
`AttemptWait` containing the **requested** duration, while suppressing the
completion effect. `ingredient_preparation.rs:457–495` checks that duration
against the authority and passes it to `advance_character_wait_time`.

For the stated no-terminal case, `disease.rs:1823–1856,1915–1916,1953–2004`
returns the requested amount rather than the representable clock difference;
`time/advancement.rs:417–498` passes it to injury settlement, saturates the
stored clock at MAX, then passes settled elapsed into other time consumers.
`adventuresim-core/src/surgery.rs:127–167` returns the requested elapsed when
no blood terminal occurs, and native `surgery.rs:442–486,510–548,555–565`
uses it for recovery and injury-state updates. For example, under the stated
conditions, a positive bruise and positive healing rate make the source invoke
healing for that elapsed while planned elapsed and stored clock advancement
are zero. No simulation or observed product outcome is claimed. The reducer's
interrupted branch can then record a receipt and checked next generation if
all subsequent operations succeed (`ingredient_preparation.rs:492–533`).
Repeated requests, supported admission of MAX, actual database rollback or
commit behavior, and all affected time consumers remain unproved.

A future repair discussion should connect the planned representable interval,
wait helper and lifecycle/disease/injury semantics, with acceptance decided
only after the audit and separate authorization. This is a conditional
source-level candidate; it is not automatically a distinct defect count or an
alias of another clock record. Ledger entries
`REVIEW-ITINERARY-CLOCK-NONPROGRESS-001` and
`REVIEW-RESIDENCE-BILLING-CLOCK-EXHAUSTION-001` concern other consumers at a
saturated clock. Historical q2210 `PL13-212-ITINERARY-CLOCK-EXHAUSTION-NATIVE-PENDING`
and q2291 `PL13-243-NATIVE-TERMINAL-STRUCTURAL-BOUNDARIES-PENDING` are related
clock/terminal questions, not a proved identity with this first operation.

## Safeguards and unresolved domain questions

- Exact request replay checks all submitted tuple fields before mutable-state
  access; new attempts check generation, stable row/object identity, carried
  custody and revision. The reducer replans and validates both strategic and
  material snapshots before waiting, then reloads and checks place, lot truth,
  custody, tool, skill, duration and source digest before mutation
  (`ingredient_preparation.rs:292–455,535–585`;
  `preparation_authority.rs:337–440`; `material.rs:1263–1305`). These are
  source safeguards, not proof of all gateway or transaction behavior.
- `preparation_authority.rs:598–600` maps both no terminal and an actual MAX
  terminal to the same digest representation. The full shared `TimeResolution`
  is also compared during commit (`strategic_action.rs:665–713`). If the
  requested endpoint is below MAX, both terminal settings resolve as completed;
  if it reaches/exceeds MAX, the outcomes differ (`TerminalBoundary` versus
  `ClockExhausted`) and that comparison rejects a changed plan. This trace does
  not establish an unsafe completion caused by the digest collision.
- `food.rs:10,322–327` bounds current concentration at 1e9, and
  `preparation_authority.rs:189–193,216–232` converts lot mass to milligrams
  then rounds the product and casts it to `u64` without a product-domain error.
  A representable 20,000 kg lot at concentration 1e9 would exceed the `u64`
  load domain and saturate the cast. The material receipt preserves the
  resulting snapshot; this trace does not show that native contamination was
  changed or that such a lot is supported. Selected normal inventory producers
  use one food unit per stable object (`item.rs:336–348,360–400` and
  `inventory_trade.rs:481–518`); other creation/merge routes and catalog bounds
  remain open. This refines the original PL13-251 load-bound question, rather
  than establishing a new counted defect.
- The `REVIEW-PREPARATION-RIGHTS-BINDING-001` ledger row concerns the public
  planner/trusted-caller rights contract. The selected native caller here builds
  matched rights, but other callers and policy are outside this review. The
  original q2291 concern about terminal crossing structure also remains open.

## Scope and recovery limits

`READ-LIMITS.json` is the authoritative list of this reviewer's actual source
windows. Numbered views and their prospective registrations are retained.
Historical JSON selections and source-body hashes are not full-body read
credit. Some prospective support windows landed in test source; they were read
as text only and never run. An initial historical-key listing and a time
metadata locator output were truncated; neither is source-body credit. An
initial ledger search used a nonexistent JSON path and was corrected; its empty
result does not establish absence. One herbalism display was initially planned
past EOF, then registered exactly to EOF before display. The first clock locator
found `StrategicMinuteOfDay`, not the absolute clock; the correct calendar owner
was subsequently registered and inspected.

Exact Git tree chains for captured source files are provided. Their named
commit-to-tree association depends on the independently available Git objects;
raw commit-author metadata is omitted. The complete 059 queue, all supported
clock/lot producers, external database semantics, broader historical aliases,
other lenses and independent-machine custody remain open. Privacy scanning is
bounded and does not prove global absence of sensitive values.

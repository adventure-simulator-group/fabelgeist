# q2340 case occupancy and same-minute identity: source-only assessment

## Custody and attribution

The selected original and path-redacted reconstructed 059 entries are byte
identical (SHA-256 `1d91ba37f3d89cd77b501984cf0dd3e42f58fd1aa015d5e1c6cd6620746a3833`).
Original reviewer `parallel-l13-20261029` personally assessed the frozen
6533 `investigation/model.rs` lines 1–1399; root `segovb` later adopted that
assessment without a root personal reread or a current reconciliation. Its
status was `open-connected-source-current-alias-qualification`, with no
repair or runtime pass. Four directly cited records, the nested verification
and the frozen source object matched their historical hashes. This is recovered
059 evidence, not checkpoint 3821; the latter remains missing.

This reviewer newly assessed the registered source displays at pinned 95e.
All fifteen captured current source paths and the frozen 6533 model path have
commit-to-tree-to-blob proofs. The frozen and current model bytes are identical
at SHA-256 `ce5e06fe7f483ecbd5c30af9a9475686d4c7acf147a8bb8de2e8a9e738211cb4`;
this proves identity of this file only, not tree equivalence or transferred
review coverage. Navigation of the physical worktree was unpinned and used only
to locate candidate paths.

## Bounded present assessment

* `investigation/model.rs:1027–1070` stores occupancy intervals and accepts
  exactly one active row or exactly one half-open interval match. Ambiguity is
  represented as no unique occupancy, rather than an arbitrarily chosen row.
* `investigation/model.rs:1077–1155` validates the site, uses the character's
  own current minute, rejects multiple active rows, and makes the same-site
  transition a no-op. Otherwise it closes associated contacts, invokes the
  outbreak transition, closes the active occupancy and inserts a key formed
  from character, minute and site. A leave/re-enter of the same site within a
  single character minute conditionally reuses an existing closed row's key.
  Database primary-key outcome, transaction behavior, reachability and impact
  remain open; no unique defect count is asserted.
* `strategic/travel_planning.rs:108–119,137–164,205–235` makes calculated
  journey duration at least one and rejects zero route minutes and zero span
  duration. Nevertheless `strategic/journey_camp.rs:1–29,544–590` allows a
  zero initial walking leg and returns before advancing the character clocks
  when the common safe movement prefix is zero. In
  `strategic/travel_reducers.rs:261–339,706–758`, an incomplete leg clears
  the site and can record a camp at movement minute zero. This supports a
  conditional same-minute exit, not a demonstrated supported re-entry.
  `time/settlement_rest.rs:568–608` establishes a journey-local departure clock
  without changing subjective age; no equivalence to character time is assumed.
* `outbreak.rs:1890–1943` runs before occupancy insertion and closes open
  outbreak spans; it starts a span only for an active source place and skips
  insertion if the deterministic character/site/minute key already exists.
  On the conditional same-minute return to an active source place, a previously
  closed same-key span would be skipped. Durability, order of any actual
  failure and rollback depend on the enclosing transaction and remain open.
  `outbreak.rs:1850–1885` reads interval intersections. This is a connected
  chronology question, not a separately counted finding or asserted alias.
* `world_actor/presence.rs:64–111` has a separate half-open lookup for
  observer/party presence and rejects ambiguity. `investigation/sites.rs:
  323–385` includes disclosure and alive-at gates. Other inspected callers
  include case-site travel, settlement arrival, join handling, incident and
  corpse transitions, and explicitly marked demo or seed setup. Source-only
  inspection of those callers does not establish exhaustive producer coverage.

## Questions and relationships

Qualify supported same-minute re-entry, its exact authoritative caller and
clock constraints; inspect database uniqueness and transaction semantics
without treating an unrun scenario as observed; establish intended temporal
identity and shared presence/outbreak chronology policy; and reconcile any
aliases once their full evidence is compared. The selected 17:48:19Z ledger
snapshot had 1,060 rows. Literal `PL13-261`, `case-occupancy`, `case site`,
`outbreak-presence`, and `same minute` searches found no rows; the four
`occupancy` hits had unrelated IDs. This is bounded metadata navigation only,
not an absence or duplicate proof. No ledger or publication was edited.

No product execution, parser, build, test, fixture, macro, runtime probe, or
repair was performed. Evidence-only extraction, hashing and verification are
separate. See read-limit and privacy qualifications before reuse.

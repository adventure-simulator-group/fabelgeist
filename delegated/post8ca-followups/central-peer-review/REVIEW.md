# Independent review of four-record reconciliation draft v3

Reviewer: `final_guard_peer`. This is a review of an unadopted central
synthesis, not a product-source review. The draft SHA-256 is
`615e914ede1c9c389790e064ab9cb76ba262dd6dd72b5e04942d3baeb317b4c6`.
The pinned product source is `95e63628f9d70b8dd6e6fede64334696c6995009`;
`8caeb2003d10d08320566206817aa0c694231778` supplies the current evidence
queue and ledger snapshot. Earlier materials retain their own revisions and
readers; neither this draft nor this review reconstructs the missing original
checkpoint 3821 ledger and queue.

The q24, q185, q689, and q3100 descriptions should be refined before adoption
as set out below. These are limitations of v3, not findings that repair or
runtime behavior has been established. The original source displays remain
their authors' and independent peers' attestations; I did not display any
product source lines in this review. I previously authored the separate q2561
assessment and checkpoint custody reviews, but did not author these four
source reports or their peer reports. Prior coordinator summaries were visible
to me. All referenced records and Git objects are on the same machine; this
review does not supply independently held custody or prove the underlying
source observations by replaying their readers' sessions.

## Requested corrections and limits

- **q24:** The selected helper first chooses a matching row and returns if
  that row's amount is zero. Only a nonzero selected amount triggers the
  transfer. After the helper returns successfully, a remaining positive
  matching row triggers the guard before removal; a transfer error exits
  before the guard. Multiple supported rows, indirect effects, transaction
  behavior and reach remain open. The original row's recorded revision,
  Git-path, and attribution are all null; the separately recovered historical
  receipt retains its original credit. The existing ledger 147 candidate and
  q240 address an overlapping helper-to-removal chain; q16 is a related
  inherited Lens 13 question whose own first failure is unresolved. This is a
  scoped relationship, not proof of a global alias, closure, or new defect.
- **q185:** The corrected author and peer reports distinguish planner failure,
  `Ok(None)`, and propagated errors. A terrain-present character or profile
  lookup can error before the non-travel-absence branch, and ordinary native
  approval independently decodes and checks kind. Thus a failed web decode
  alone does not establish the native result. The historical requested view
  ending at 935 had actual EOF 930. To support the case-site route and
  settlement-guard distinctions directly, add the peer's cited
  `crates/adventuresim-stdb-module/src/strategic/travel_reducers.rs:51–127,530–549`
  to the draft references. These are report-to-draft citation suggestions,
  not my independent interpretation of those source lines.
- **q689:** Qualify the retained-power multiplication as the selected
  nonempty attempted effective shield block with a positive shield bonus during
  the committed-attack phase. The peer describes a clamp on each factor, not
  on the accumulated field. Four factors near 0.6 before reset would yield
  about 0.1296 only if that sequence is reachable; reach and policy remain
  open. The two half-second producers have different roles. If the displayed
  authored configuration is in use, 0.35 seconds bounds weapon-work energy,
  while active work keeps the maximum of its previous value and the original
  span; dodge energy has a distinct calculation. Historical e275 read credit
  and per-file equality must not be presented as a current personal reread or
  whole-tree equality.
- **q3100:** Qualify the statement about no fresh e275 personal reread as a
  statement about the historical receipts. The new source author separately
  reports personal views in the 95e-session of 6533, e275 and 95e; that new
  work is distinct from the historic receipt. The e275 1–62 reuse remains
  conditional on frozen-source adoption, 63–90 uses a mixed fresh/inherited
  basis, and no historical Lens 1 credit transfers. The selected historical
  q3100 row itself names no reader and has empty assessed ranges. q3134's
  original status member is absent, whereas its current wrapper projects null.
  For claims in the synthesis, add the peer's cited
  `crates/fabelgeist-determinism/src/lib.rs:12–19` for the selected half-open
  sampler and `crates/adventuresim-tactical-server/src/main.rs:66–89,119–120,168–175`
  for the argument-to-mission handoff; other supporting attack-start and
  combat-config citations should be considered. Preserve separate contact and
  defense-alignment draws, the closed endpoint admitted by `UnitInterval`,
  sample and error order, and fixture versus no-fixture paths. A generic
  sampler is not automatically a defect.

The q3100 draft's count of 37 peer source windows is not established by the
six peer members captured in this review. A separately held peer manifest or
replay may establish the total; bind a count-bearing receipt or omit the number
until it is captured. This does not question the peer's authorship or imply the
count is wrong. The q185 captured personal-display receipt itself records 17.
The q3100 peer's author-count distinction, correction, over-EOF attempts and
builder launch error remain separately attributed.

## Independent metadata checks

The captured draft's inputs, predecessor and correction hashes matched. I
compared eight report members and ten q185/q3100 peer members to their captured
identities; eleven selected original, reconstructed and current queue records;
and thirteen selected ledger records including 147. Selected original and
reconstructed records matched as parsed; their whole queue streams differ.
The existing assessment candidate matches the captured stake relationship and
ledger 147 identity. The q24/q185 historical null fields and the status-member
distinction are not authority to alter their earlier evidence.

All 23 cited draft paths exist at the pinned source revision, and their stated
ranges are within actual EOF. Source bytes were read into memory only to count
lines; the check does not assess the content or sufficiency of any citation.
The original checkers also explicitly limit themselves to provenance, counts,
and path existence rather than semantic or peer verification.

The current snapshot has 3,713 queue rows: 1,400 partial and 2,313 unassessed.
All four selected rows are unassessed. If the four proposed partial records
and seven distinct planning relationships were adopted, the projection is
1,404 partial, 2,309 unassessed and ledger 1,128 to 1,135. These are
prospective administrative counts, with zero queue closures, lens completions
or accepted unique-defect increment. Seven distinct proposed IDs are absent
from the current ledger; a lexical or ID check cannot rule out semantic
aliases.

The initial metadata script failed before its comparisons because it looked
for its registration in the snapshot instead of the review directory. The
next version reported a false count discrepancy by separating `partial:` from
`partial`; the corrected version passed, with both predecessors retained in
the restricted process evidence. A later q2561 coordination message also
conflated another record's null fields and timing statement; it was promptly
retracted to the recipient. The sealed q2561 report itself was unchanged.

The seven relationships are planning links, not proof of exhaustive producers,
policy, supported reachability, transactions or global identity. No product
repair, build, test, runtime probe, fixture run or product validation occurred.

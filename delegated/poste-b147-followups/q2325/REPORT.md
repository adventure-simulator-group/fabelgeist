# Bounded q2325 native preview skin layout review

This is a fresh source-only review by `/root/verifier_scope`, at authoritative
product revision `95e63628f9d70b8dd6e6fede64334696c6995009`. It is neither a
thirteen-lens closure nor a repair, rendering, product validation, or global
alias inventory. No product code, tests, fixtures, builds, shaders, rendering,
or runtime probes were run. Evidence-only verification is separately recorded.

## Recovered record and attribution

The original 059 queue stream at evidence revision
`56f24f6457f6cec8ffcdc46f03f5ff26a8abb50e`, `/entries/2325`, identifies
`REVIEW-L11-CREATOR-NATIVE-PREVIEW-SKIN-LAYOUT` and records
`rejected-suspicion-source-qualified` with discovery closed *within that named
scope*. It does not close all lenses or a coarse parent, establish the global
alias inventory, or establish repair/runtime success. Its explicit outstanding
policy qualification is that eight-influence analytical cloth/body collision
versus four-influence native rendering is not blanket visual equivalence.
The exact original queue entry is also byte-identical to its separately labeled
public reconstructed 059 entry at evidence revision
`26e360920b7b2ec32d5c76238623eaf6d6617d7e`, although the whole streams differ.
Neither entry is represented as recovered checkpoint 3821.

Thirteen directly cited original records were recovered against their queue
hashes. Their original delegated and root roles are preserved in
`HISTORICAL-READERS-*.json`, `HISTORICAL-ROOT-VERIFICATION.json`, and
`HISTORICAL-PROPOSAL-SELECT.json`. The e275 delegated lens11_reconcile agent
recorded personal reads; the later root mappings explicitly adopt those
registrations without root rereads. The source identity comparison binds the
eight historical cited e275 files to byte-identical 95e files. That does not
make revisions, dependencies, assets, interpretation, or historical and current
personal read credit interchangeable. See `SOURCE-READ-RECEIPT.json` for this
reviewer's displayed 95e intervals and unread complements; capture alone is
not a read. Transport inventory metadata matches digest, byte count, and
relative suffix for 13/13 cited records, without proving independent
out-of-machine custody.

## Current source observations

- `animation_preview/binding.rs:22-58` zips packed index and separate weight
  rows without a local row-count or mesh-position check, sorts eight individual
  slots by descending weight and ascending joint ordinal, checks conversion of
  four selected ordinals to u16, and supplies `Uint16x4` joint indices plus
  four weights. A positive selected total is divided out. The fallback sets
  only the first weight to one; it is an all-zero-row fallback for all-zero
  input, not a general guarantee of normalization for arbitrary negative or
  non-finite rows. An error converting an ordinal occurs before attributes are
  inserted. These are source statements, not a demonstrated reachable failure.
- `fabelgeist-rig/src/skin.rs:79-99` instead combines equal joint ordinals
  *before* selecting the four strongest, and `export/skinning.rs:4-32` uses
  that reduction. The source comment calls it a Bevy/glTF shared reduction,
  while the inspected preview adapter sorts individual slots itself. A purely
  symbolic eight-slot row with joint ordinals `[0,0,1,2,3,4,0,0]` and weights
  `[0.25,0.25,0.1875,0.125,0.09375,0.09375,0,0]` makes the preview choose
  slots `[0,0,1,2]`, whereas combining equal ordinals first yields distinct
  ordinals `[0,1,2,3]`. These fractions are exactly representable; this is
  reasoning from source, not execution. Whether supported private assets
  actually produce duplicate positive joint slots, whether those paths should
  agree, and whether any visual effect follows remain open. Do not count this
  as an accepted unique finding or an established alias without central review.
- `fabelgeist-mhr/src/character/skin.rs:58-152` builds paired per-vertex
  index/weight rows from clusters, checks cluster array lengths and referenced
  vertices, rejects vertices without influences, keeps up to eight, and
  normalizes a retained total. The inspected loop does not explicitly
  deduplicate equal bone ordinals or explicitly require finite weights: a NaN
  is not excluded by the shown `<= 0` predicates, subject to upstream decoding
  and supported asset admission that were not established here. The packed rig
  constructor checks each ordinal against the rig bound (`skin.rs:112-123`);
  it does not validate the separate numerical weight array or prove rig
  instance identity. This does not establish that malformed or duplicate
  supported assets exist.
- The ordinary clothing producer checks input positions, normals, skin rows,
  weight rows, joint bounds and face bounds (`clothing.rs:119-134`); its fitted
  surface and helpers preserve row count/order under those checks
  (`clothing/fitting.rs:1-49`, `clothing.rs:359-441`). Preview body and shell
  calls use the current model's rows (`preview.rs:18-44,160-188`). The separate
  q2323 pairing review must be consulted for retained generated/model identity;
  equal row counts alone do not prove the pairing of independently retained
  instances or all other producer paths.
- `animation_preview.rs:117-129` checks weighted cloth index/weight row counts
  against positions; `drape_preview.rs:220-280` conditions creation on row
  lengths. During deformation, however, `animation_preview.rs:311-340` first
  forms body/armor posed colliders and only then checks each weighted cloth
  target's joint count. The collider calculation calls the eight-influence
  `skinned_positions` (`animation_preview/colliders.rs:1-26`,
  `animation_preview.rs:411-439,461-485`), whose position iteration indexes
  retained rows and positive-weight joint matrices. Thus the later target gate
  must not be described as validating earlier body/armor rows. A supported
  mismatch, a panic, and any runtime effect have not been established.
  Unweighted-physics/ignored-weight behavior is a separately selected branch.

## Relationships and open decisions

The original q167 concerns MHR numeric weight admission; q175 concerns shared
rig normalization; q436 records an all-zero nonempty public influence row and
its unresolved zero-total policy; q572 is a pending connected admission/export
revisit. These are recovered original 059 entries, not findings newly created
by this reviewer. The separately attributed current 95e rig/equipment
assessments preserve the symbolic all-zero `strongest_four` issue and qualify
that the examined glTF exporter validates its input. That all-zero question
is distinct from the positive duplicate-slot selection question above, and a
zero row rejected by one exporter does not settle the shared API or every
consumer. Exact alias and ledger classification belong to the coordinator.

The named historical q2325 disposition remains historical even though its
cited source bytes match 95e. The present observations refine its limits and
surface a policy/reach question; they do not retroactively change its status.
The related q2323 cached body/model pairing, q2324 orientation, and q2326
hair/body instance ownership reviews have separate scopes and readers. No
cross-lens credit or causal alias is asserted here.

Resolve, by future authorized review and eventual product validation: whether
supported MHR clusters may repeat a joint within one vertex; whether preview
and export are intended to use identical four-slot reduction; whether upstream
admission rules exclude non-finite or problematic weights; and whether any
permitted model/skin/position mismatch reaches the collider sequence. Assess
policy and acceptance only after the audit/ledger phase; this packet proposes no
implementation and performs no product validation.

## Verification and limits

`VERIFY-EVIDENCE.py` verifies packet bytes, recorded source displays and
registrations, historical citations/selections and optional fixed Git/transport
bindings. `DISPLAY-QUALIFICATIONS.json` preserves three requested-range EOF
clamps, the narrow helper locator followed by a wider search, capped locators,
and displayed test source with no execution. A separate privacy scan receipt
binds the packet manifest and describes scanner limits. Hashes, timestamps,
locators and scans cannot prove comprehension, source semantics, actual asset
reach, rendered output, product behavior, independent custody, or unchanged
uninspected dependencies. The packet contains no exact checkpoint-3821 claim.

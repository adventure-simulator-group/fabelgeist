# Evidence following checkpoint 76fe

This packet uses the explicitly incomplete working audit authority at source
revision `95e63628f9d70b8dd6e6fede64334696c6995009`. It does not recover the
missing original checkpoint 3821. Historical original 059 records,
reconstructed 059 records, normalized 774 summaries and current source
assessments retain their separate identities and authorship.

- `source-transport/` holds 21 bounded public source-review, correction and
  receipt archives. Its map identifies the exact members and earlier readers.
- `reconciliation/` binds 17 selected report members and compares 21 selected
  original and reconstructed 059 records as parsed. The whole streams differ;
  a missing original `status` member is not the later wrapper's `null`.
- `source-encrypted/` and its restoration and scan receipts preserve the
  separately attributed source packets and fixed central captures. The
  authenticated same-machine restore matched 24 collections, 1,594 files and
  83 directory or root modes; later v4 review artifacts are separate.
- `central-encrypted/` and `central-receipt/` preserve the later v4, peer and
  projection-review evidence at a separate cutoff. Their authenticated restore
  matched 31 collections, 423 files and 36 directory or root modes. The full
  later post-build control is separately held, not inside its earlier snapshot.
- `metadata-encrypted/` and `metadata-receipt/` preserve the later source
  metadata-binding control at another cutoff. Its authenticated restore
  matched four collections, 71 files and four directory or root modes; the
  full post-build control remains separately held.
- `application-encrypted/` and `application-custody-receipt/` preserve
  the independently reviewed candidate at a later cutoff. The original
  authenticated restore matched 12 collections, 109 files and 20 directory
  modes; the full later control delivery remains separately held. See
  `APPLICATION-CUSTODY.md` for the precise dependencies and limits.
- `process-encrypted/` and `process-receipt/` preserve the preceding checkpoint's
  later process evidence. The same-machine authenticated restore matched 17
  collections, 198 files and 22 directory or root modes. Its earlier excluded
  staged-tree copy was checked against Git, with local mode limits recorded.
- `process-binding/` and `binding-transport/` preserve the separately held later
  control binding, including the first mode-representation failure and corrected
  replay. That control was not retroactively included in the earlier ciphertext.
- `minimal-recovery/` distinguishes the initial failed successor guard, the
  later publication-object-store alternate replay, and the separate replay using
  seven restored Git objects plus the original queue blob. The minimal subset is
  not full Git closure or a complete source checkout.

The encrypted originals, earlier ciphertexts, separately held key, selected
Git objects and historical anchors remain recovery dependencies. Same-machine
copies are not independent custody, and finite scans cannot prove absence of
unknown private values. The process and source evidence have distinct cutoffs;
later source reviews, including q689 and q24, are outside this packet until
separately preserved in published custody and reconciled.

All thirteen lenses and semantic queue reconciliation remain open. Planning
relationships are not unique-defect counts. No product repair, build, test,
runtime probe or product validation is performed by this packet.

# NumPy README and ZIP member-count authored contracts

## Historical evidence and current identity

The original 059 q2360 and q2361 entries are byte-identical to their respective
path-redacted reconstructed entries. Their SHA-256 values are respectively
`6190ed7960999ef65a034545ee5bdc352521531fb44bbaba06716311667be0cb` and
`49640c637efce14dcca3b1274398044ccf3f38753efd7fe68e4b7bb0141d4149`.
Both are Lens 11 named authored-document dispositions recorded at e275. The
original `lens11_reconcile` reviewer personally read the registered e275
ranges; the cited `segovb` records adopted that work. The historical entries
close only their exact named documentation scope, with no global lens or parent
closure, complete alias inventory, binary fixture assessment, implementation
gate or runtime pass. They explicitly retain the separately attributed
SOURCE004 native qualifications and 73 nontext/no-positive-EOF contracts.
Seventeen directly cited plan, read, adoption, verification and proposal records
were recovered by their original hashes. These are 059-era evidence, not the
missing checkpoint 3821.

Seven 95e source/document paths and their seven e275 counterparts have
commit-to-tree-to-blob proofs. Each pair has the same bytes, and matches its
historically recorded SHA-256. A bounded search in the protected parallel-L11
folder did not locate separately stored historical source-object filenames;
the e275 bodies in this packet are **reconstructed from the matching current
bytes**, not claimed as recovered original source objects. Per-file identity is
not whole-tree equivalence or transfer of personal review credit.

## Current authored contracts and selected source

* `README.md:16–19,29–56` distinguishes declared uncompressed size from verified
  decoded size; gives rawzip framing and the application query, search and
  decoding roles separate owners; and describes checksum, encryption, size and
  resource limitations. `zip.rs:86–109,151–184` retains native entry handles,
  uses the declaration for initial Deflate allocation, and reads selected
  Deflate bytes to an owned buffer. It does not itself select a CRC or decoded
  length verification step in the displayed path. Actual native behavior,
  policy, input trust and resource ceilings remain separate questions.
* `member-count.md:3–16,32–35`, `zip/member_count.rs:7–25`, and
  `zip.rs:86–109` describe and implement the selected `u64` declaration as a
  maximum number of first-party iterator requests. Initial vector reservation
  is capped at 4,096, but later pushes may grow the vector. Clean `None` ends
  iteration early; an encountered iterator error rejects construction. There is
  no declared-versus-actual equality or extra exhaustion check after the bound.
  An underdeclaration can leave further records unrequested, including any
  errors in those unrequested records. The cap is not a total memory or work
  budget.
* Zero causes no first-party `next_entry()` requests; this is not a claim that
  no ZIP structure is parsed. The independently attributed prior q2312 review
  recorded that rawzip's locator can probe a fixed central header before that
  first-party loop, and separately documented clean exhaustion, digital
  signature termination and selected parser errors. This report does not
  re-credit those native body reads or establish malformed-archive runtime
  acceptance.
* `README.md:64–110,115–133,159–186` presents shape, payload and storage-value
  roles separately from tensor/importer semantics; it also documents lossy
  names, directory order, first-match queries and archive-bound handles.
  `npy.rs:23–38,77–94,118–130`, `npy/metadata.rs:8–28,83–98`, and
  `npz.rs:17–28,45–85` support the displayed immutable-array and selected-member
  relationships. Exact header admission, length policy and numeric/importer
  semantics retain their own q2306/q2307 and other owner scopes. README examples
  and historical descriptions of fixture observations are documentation, not
  evidence that this review executed or validated those cases.
* `member-count.md:18–30` states the selected end-record and ZIP64 policy and
  mentions earlier fixture observations. Their native implementation and
  practical input reachability belong to earlier q2309–2312 source reviews;
  the named document disposition does not certify all native variants or binary
  fixtures. Documentation claims are not a standards or runtime verdict.

## Relationships and remaining decisions

The previously published q2312 count review is retained as a separately
attributed source assessment; it distinguishes count bounding, clean native
termination and entry-vector reservation. q2316 concerns declared-size-driven
Deflate reservation, decoded-size/CRC and output budgets, a different first
operation. q2307 concerns selected-member/archive association. They must not be
automatically merged into one defect, assigned a unique count, or treated as a
new native read from the authored-document entries.

The bounded 1,060-row ledger snapshot links count/ZIP64, end-record selection,
member identity, error context and decoded integrity/budgets as distinct
relationships. Their listed policy, native input reachability, caller/UI and
runtime questions remain open. Product owners must decide whether exact
cardinality, alternate end-record recovery, CRC/decoded-size verification and
resource ceilings are required for supported assets. No policy choice or
repair is implied by this assessment.

No archive, binary fixture, product parser, test, doctest, build, macro, runtime
probe or repair was executed. Only registered source displays, evidence JSON
extraction and identity verification were performed. See read and privacy
limits before reuse.

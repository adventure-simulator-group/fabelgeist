# ZIP extent and error authored contracts: source-only reconciliation

## Historical custody and limited disposition

Original 059 q2362 and q2363 entries match their path-redacted reconstructed
entries byte-for-byte, at SHA-256 respectively
`24885f61432db9343ecb364faa26d0c6e941f210124c2a172e1f0f4c4833b69c` and
`b1802fa0226bef94102778bf810dc281bb5d28765e71e391443722e6c7ca6b2e`.
The original `lens11_reconcile` reviewer personally read registered e275
ranges; cited `segovb` records adopted that evidence. The entries record
closure of only the named authored-document scopes, leaving SOURCE004 native
qualifications, 73 nontext/no-positive-EOF contracts, binary fixtures, runtime,
other lenses and exact aliases open. Ten directly cited historical plan, read,
proposal, adoption and verification records match their recorded hashes. These
are recovered 059 records, not the missing checkpoint 3821.

Six selected e275 and 95e file pairs match byte-for-byte, with commit/tree/blob
proofs at each revision. The bounded protected parallel-L11 filename search did
not locate the original hash-named source objects; e275 bodies here are
reconstructed from matching current bytes and the recorded historical hashes,
not represented as recovered originals. Two additional current MHR paths have
95e tree proofs. Per-file identity is neither tree equivalence nor transferred
personal source-read credit.

## Extent contract and implementation

`zip-extents.md:3–14,25–43` describes an unverified declared decoded size,
including NPY header bytes, as distinct from a measured result, existence or
budget. `zip/extent.rs:25–42` retains the full `u64`, including zero and
`u64::MAX`, and projects it by a native `as usize` cast for initial allocation.
`zip.rs:86–109,151–184` associates a central entry and its archive, obtains
native member access first, returns stored bytes by borrow or reserves and
reads Deflate into an owned vector. It does not select a ZIP CRC/decoded-size
verifier or output budget in that displayed path. `npz.rs:45–85` retains the
selected first-match handle and reports its declared size without a second
lookup. NPY payload and tensor dimensions remain separate owners.

The earlier q2316 report is retained as a separately attributed assessment. It
traced checksum-bound rawzip and Deflate source and identified the native
unverified-size warning, optional verifier and conditional backend details. It
did not run an archive, establish asset trust, allocator outcome or a required
CRC/size policy, or prove a specific incident. Those qualifications are not
replaced by the authored-document disposition or by this source review.

## Error contract, source chains and presentation

`zip-reader-errors.md:17–49` documents error ownership and describes selected
formatting. `zip/error.rs:34–69,71–142` implements these distinct branches:

* Rawzip's missing-end classification becomes unit `MissingEndRecord` with no
  retained source. Other constructor parser errors become `Archive` with the
  native source, but the variant does not separately tag locator and directory
  iteration. The archive display includes that source's display.
* Member rawzip `Eof` maps to `MemberPastEnd`; other member parser errors map to
  `CorruptMember`. Both retain member name and source; their own default and
  alternate display branches do not append the nested parser source. Native open
  and map errors retain operation, path and I/O cause, and Deflate failures
  retain member and I/O cause; alternate formatting appends those I/O causes.
* Missing-member and unsupported-compression variants retain query/name or
  code, respectively, without inventing a cause. `zip.rs:160–181` obtains
  native member access before choosing compression and may return a structural
  error before unsupported-method classification. Paths and names are error
  context, not proof of file identity or path safety.

`npz.rs:65–85` adds missing-query context at lookup and a parsing context after
member-byte retrieval; the earlier byte-access `?` is outside that latter
context. Current `fabelgeist-mhr/src/correctives/archive.rs:39–78` adds basis,
sparse-index and sparse-weight context around selected member-array operations;
its archive `open`/`from_bytes` calls do not add those array-specific strings at
that layer. The named prior q2315 source assessment, retained separately here,
also qualifies rawzip error kinds and candidate offsets. These are source and
formatting branches. Actual downstream downcasting, outer formatting, logging,
user-facing presentation and all callers remain open.

## Relationships and open policy

The 1,072-row time-bound ledger snapshot has separate selected relationships
for NPZ member identity, error context, and decoded integrity/budget. q2307,
q2315, q2316 and q2360 remain related historical/current records with distinct
first operations; no automatic alias or unique defect count is assigned. The
owner must decide trust assumptions, allocation and output budgets, whether and
how full decoded size/CRC are verified, and which diagnostic contexts users
need. Reachability and transaction/platform outcomes are not established.

No product code, ZIP/NPY parser, binary archive, fixture, test, doctest, build,
macro, runtime probe or repair was executed. Source snippets and examples are
inspection only. See `READ-LIMITS.json` and privacy qualifications for details.

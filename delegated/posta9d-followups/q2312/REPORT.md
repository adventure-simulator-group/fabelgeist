# NPZ declared member count: bounded source reconciliation

The assessed current revision is
`95e63628f9d70b8dd6e6fede64334696c6995009`. This is a source-only review, not
an archive execution, product validation, repair, or implementation. Its scope
is 059 entry 2312, not the whole lens, queue, or a count of unique defects.

## Current first-party behavior

`crates/fabelgeist-numpy-storage/src/zip.rs:86-110` passes bytes through
rawzip's locator, wraps `entries_hint()` in `ArchiveMemberCount`, reserves a
vector, then calls `next_entry()` in `0..declared_count`. `member_count.rs:7-25`
retains the count as u64; only `min(count, 4096)` is converted to usize for the
initial capacity. This cap is not a total cardinality or allocation limit:
each successful header is pushed and the vector may grow. Errors from a
`next_entry()` call are propagated; this function does not return a partial
`ZipArchive` on that error. An early `Ok(None)` ends the loop successfully.
`zip/error.rs:126-132` retains the native error as an archive error except for
its special missing-end-record mapping.

A declaration of zero makes no `next_entry()` calls, but constructing the
locator is not a no-parse operation. `rawzip/src/locator.rs:62-95` probes a
fixed central header at the declared offset; if that probe fails for non-ZIP64
it may probe a computed alternative, correcting offsets on success. Thus the
historical shorthand about zero skipping central bodies means skipping the
first-party directory iteration, not that native code never parses a header.

## Native source and conditional consequences

The child workspace manifest pins rawzip 0.5.2 and its lock supplies SHA-256
`ac9999039f74cd73c03158cddfe4362991703c74efe5ce5c8073026f80a5e48c`.
The separately held original `.crate` matches that checksum and the two
captured source members match the package bytes.

`rawzip/src/archive.rs:75-111` returns the selected EOCD count as a *hint* and
constructs a native iterator over bytes between the selected directory offset
and head EOCD offset. `locator.rs:101-146,181-281,290-307,402-431` shows the
classic count or a consistent selected ZIP64 count being used, with fallback
to classic for a missing or rejected ZIP64 candidate. That selected declaration
is not an independently verified number of file headers.

`archive.rs:360-434,1557-1648` shows that each returned record requires a
46-byte fixed central header and available length-delimited variable fields,
then advances the remaining byte slice. An empty slice returns `Ok(None)`.
After a fixed-header parse error, a remaining span beginning with the digital
signature bytes and no longer than the native maximum is treated as a clean
terminator; otherwise the error is returned. A variable-field truncation or
checked-offset failure is an error; after any iterator error, its remaining
slice is cleared. These claims describe source control flow and are not a
runtime or malformed-archive acceptance result.

If the declaration is below the number of otherwise available valid headers,
the loop stops after that many and does not ask the iterator whether more
remain. If the declaration is larger than the headers and the iterator reaches
a clean terminator, the loop accepts fewer. Malformed remaining bytes instead
may yield a propagated error. No equality check or extra exhaustion check is
present in the assessed loop. Whether asset admission should require exact
cardinality, and what total entry/memory/work budget should apply, remain
policy decisions. The finite input span and advancing iterator avoid claiming a
u64-to-usize narrowing or an empty-loop failure here; they do not establish a
practical resource budget, input reachability, accepted ZIP variants, or safety
on every target platform.

## Historical and cross-record relationships

The recovered original q2312 and path-redacted reconstruction have identical
entry bytes; their full queue files differ. The original Lens 11 reviewer
recorded e275 source and child-lock rawzip 0.5.2 reads and a conditional policy
disposition. Original root adoption and verification records 707, 708, 711,
and 712 retain that attribution, zero other-lens credit, and no runtime
validation. The historical closure is expressly scoped; it is neither a 95e
whole-lens completion nor checkpoint 3821 evidence. The final 3821 ledger and
queue remain unrecovered in this evidence chain.

The earlier q2307 selected-member/declaration-roles review and q2316
size/CRC/resource review are separate relationships, with their original 059
entry identities preserved here. A size-hint-derived decoded-byte allocation
is a distinct operation from the initial entry-vector capacity. Prior q2316's
bounded, time-specific ledger keyword search is not an exhaustive semantic
alias inventory. Central ownership must reconcile any current ledger links;
this packet creates no ledger assignment or unique-defect count.

`zip.rs`, `zip/member_count.rs`, `zip/error.rs`, and the child lock have
identical per-file Git blob identities at e275 and 95e. The child manifests
are different. Those facts do not establish that the whole crate or commit
trees are identical. Personal source-read credit is restricted to the exact
displayed ranges in `READ-LIMITS.json`; copied full files are preservation, not
whole-file review.

## Qualifications and open questions

The current working Git object store could not resolve the pinned commit; the
failed direct lookup is retained. An initial overbroad identity assertion also
failed, and the later per-file proof records the manifest difference. Offline
proof uses separate raw commit anchors and included content-addressed tree
bodies. An initial registered locator metadata output was reconstructed from
the same captured files because its original console text was not saved as a
file. Both events and the `Cargo.toml` requested-window EOF are recorded.

Open: whether exact declared-versus-actual equality is required, the intended
entry-count/resource ceilings and trusted-input assumptions, practical archive
and platform reachability, and full current ledger/alias reconciliation. No
ZIP file, fixture, product parser, build, test, runtime probe, or repair was
run. Native crate archives remain separately held: their earlier bounded scan
reported email candidates and nested ZIP/JAR formats that were not inspected.
The public-safe packet itself is scanned separately and cannot replace those
excluded originals as checksum or Git-commit anchors.

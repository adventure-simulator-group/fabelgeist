# NPZ native operation and cause ownership

This source-only reconciliation covers historical 059 entry 2315 against pinned
current revision `95e63628f9d70b8dd6e6fede64334696c6995009`. The original and
reconstructed entry bytes match at this index, while the full queues differ. The
historical disposition rejects a blanket cause-erasure suspicion within its
named scope; it does not establish that every error retains every detail, close
all of Lens 11, or assign a unique-defect count.

## Current first-party boundaries

`crates/fabelgeist-numpy-storage/src/zip/error.rs:19-69,114-142` defines the
public ZIP result and error variants. The source trace in `zip.rs:64-110,
130-181` supports these distinct mappings:

- Native file open and mapping failures retain the original `io::Error`, the
  respective `Open` or `Map` operation, and the attempted path. That path is
  diagnostic context, not proof of successful file identity.
- A rawzip `MissingEndOfCentralDirectory` passed through `From` becomes the
  unit `MissingEndRecord`; the rawzip object is discarded. Other rawzip
  failures become `Archive { source }`. Both locator and directory iteration
  use this conversion, so `Archive` has no separate explicit locator-versus-
  iteration operation tag or native path field.
- The selected member's rawzip `Eof` becomes `MemberPastEnd { name, source }`;
  other member framing failures become `CorruptMember { name, source }`.
  Deflate read failures become `Inflate { name, source }` with `io::Error`.
  The name is the selected parsed central-directory name.
- `MissingMember` retains the query name and `UnsupportedCompression` retains
  its method code; neither variant invents an underlying source.

`Error::source` returns the retained rawzip or I/O object for the applicable
variants, and returns `None` for the unit/owned-context variants. This is an
inspectable cause chain at the ZIP error boundary, not evidence that every
consumer displays it. For rawzip `Archive`, the first-party `Display` includes
rawzip's kind text. For `CorruptMember` and `MemberPastEnd`, its display uses
the member-level wording without displaying the nested native error, including
under alternate formatting. For `Native` and `Inflate`, alternate formatting
appends the source; normal formatting does not. These are source formatting
branches, not observed UI output.

`npz.rs:31-42,65-85` returns `anyhow::Result` at the NPZ surface. Missing lookup
adds the query as context to the absent `Option`. The `reading array` query
context is attached to the later NPY parse; the member-byte `?` occurs before
that context. Actual downstream error-chain formatting, downcasting, logging,
and user presentation have not been examined. No claim is made that the NPZ
wrapper necessarily presents the same text as the ZIP variant.

## Locked native parser detail

The child Cargo workspace declares rawzip `=0.5.2`; its lock contains package
SHA-256 `ac9999039f74cd73c03158cddfe4362991703c74efe5ce5c8073026f80a5e48c`.
The captured source members match that separately held package. Its manifest
states that default `std` enables `alloc`; no feature-resolution build was run.

`rawzip/src/errors.rs:4-116,120-175` stores an `ErrorKind` and optional EOCD
candidate offset. Its `source()` exposes an underlying I/O or UTF-8 error for
those corresponding feature-gated variants and no invented source for other
kinds. Its display formats the kind, not the optional candidate offset.
`rawzip/src/locator.rs:62-69` creates the missing-signature error before a
candidate offset exists; errors parsing an identified candidate receive that
offset. Consequently the first-party unit projection discards the native
object, but the inspected no-signature slice-locator branch has no candidate
offset to preserve. A non-missing candidate-parse error is retained inside
`Archive` and its rawzip offset remains available to an inspecting caller.
This does not prove all other possible producer paths or presentations.

`rawzip/src/archive.rs:176-191,360-433` distinguishes local member framing and
central-directory iteration errors; the first-party call sites choose the
mappings above. Native error kinds for checksum and size do not show that this
first-party route performed those validations. Historical/current q2316 covers
verification and resource policy separately.

## History, links, and open questions

The original e275 Lens 11 reviewer recorded `zip/error.rs:1-142`, locked
rawzip `errors.rs:1-215`, and `locator.rs:1-531`. The original source receipts,
registrations, proposal, and root 707/708/710 adoption and verification files
are retained and match all 12 queue-listed hashes. Historical attribution and
source revisions remain unchanged; root records transfer no other-lens credit
and report no runtime validation. The historic named disposition is not a
current global verdict or checkpoint 3821 record. The final 3821 ledger and
queue remain unrecovered in this evidence chain.

The captured 95e and e275 first-party source blobs and child lock are equal
individually and share the numpy-storage `src` tree; their crate trees and
Cargo.toml blobs differ. `READ-LIMITS.json` records only this reviewer's actual
source displays and complements. Exact prior q2307, q2313 and q2316 entries
and prior packet manifest identities are linked as separate historical/current
relationships; no alias or finding count is inferred. A time-bound, literal
current-ledger lookup and exact snapshot are included, not a semantic alias
inventory.

Open questions include whether callers need explicit locator-versus-directory
operation context, what diagnostic information is intended to reach users, how
outer `anyhow` chains are actually handled, reachable error variants and data
trust assumptions, and any policy chosen for checksums, sizes, or resource
limits. No parser, product code, fixture, build, test, runtime probe, or repair
was executed. The raw commits, complete queues and native source archive are
separate verification inputs; the excluded archive's prior bounded privacy scan
found email candidates and nested ZIP/JAR signatures left unread.

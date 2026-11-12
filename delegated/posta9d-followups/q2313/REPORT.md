# NPZ selected-member/archive association

This is a source-only reconciliation of historical 059 queue entry 2313 at
pinned current revision `95e63628f9d70b8dd6e6fede64334696c6995009`. Its
original and reconstructed 059 entry bytes match; the complete queue files do
not. The conclusion is scoped to the selected-member route and does not close
all of Lens 11, the queue, or a count of unique defects.

## Current selected-member route

`crates/fabelgeist-numpy-storage/src/npz.rs:17-85` shows `Npz::get` selecting a
member once, retaining that member and a clone of the caller's query in private
`NpzMember` fields. `name()`, `uncompressed_size()`, and `array()` use that
same member; `array()` gets its bytes before passing them to the NPY reader.
The query is retained for diagnostics. `Npz::array` delegates through this
selection. The public member API shown takes no second archive parameter.

`src/zip.rs:31-48,86-108,130-181` records the central header's parsed name,
compression, declared uncompressed size and native wayfinder in one private
`Entry`. Crate-private `ZipArchive::find` constructs a `ZipMember` holding
references to both `self` and the chosen entry. Its metadata methods use that
entry; its bytes method calls the same archive's rawzip `get_entry` with that
entry's wayfinder. `ZipMember` and its fields are not a public constructor;
`src/lib.rs:19-29` shows the exposed first-party archive and member surface.
This supports the historical rejection of a supported cross-archive or
second-lookup metadata/payload split **within this selected-member route**.
Separate `ZipArchive` name-based API calls select independently; this evidence
does not guarantee a transaction across distinct calls.

The child workspace pins rawzip 0.5.2. Its `src/archive.rs:1222-1233,1449-1477`
shows the wayfinder is a copyable value carrying central-header data rather
than an archive lifetime or identity. `archive.rs:176-207,211-230` uses its
local-header offset and compressed-size hint to return a bounded slice from
the receiver archive. Thus rawzip's independent API does not itself prove
archive association; the assessed first-party private pairing supplies the
association. Selection is based on the retained central-directory entry. This
path does not compare the local header's name with the central name; it does
not establish a general local/central consistency policy or CRC validation.

## Name and admission boundaries

`src/npz/name.rs:7-14,40-60` matches a query against a full parsed member name
or that member name with one trailing `.npy` removed. The first match wins even
if a later entry would match the full spelling exactly; the query itself is
not normalized. For example, the query `weights.npy` can match an earlier
`weights.npy.npy` before a later exact `weights.npy`. `src/zip/names.rs:5-17`
uses lossy UTF-8 for stored names and permits distinct invalid byte spellings to
produce the same parsed name. These are observed selection policies, not a
proof of path validity, exact wire identity, uniqueness, or an automatic
additional defect.

The first match is among the entries actually retained by the first-party
archive. Historical/current q2312 separately addresses the declared-count
upper bound and possible unvisited directory records. q2307 separately covers
selected-member and declaration roles; q2316 separately covers declared size,
CRC and decompression-resource policy. Their original entries and prior packet
manifest identities are linked here without asserting alias equivalence or a
unique-defect count. Actual input reachability, duplicate-name policy,
central/local consistency requirements, mmap mutation assumptions, full
consumer coverage, and later runtime acceptance remain open.

## Historical identity and recovery

The original e275 Lens 11 reviewer recorded first-party source reads and
locked rawzip 0.5.2 source ranges; historical root adoption 707 and 711
preserved that attribution, with no transferred other-lens credit or runtime
validation. All nine q2313-referenced receipt, registration, proposal,
adoption and verification records in this packet match their queue hashes.
The e275 and 95e file blobs are identical for the examined source files and
child Cargo.lock, and both point to the same numpy-storage `src` tree
`c17733266a6817b5313f94bd38edb5a45a35caa5`. Their crate trees and Cargo.toml
blobs differ; no whole-crate or whole-repository equivalence follows.

The q2313 historical disposition is `rejected-suspicion-source-qualified`.
That original named disposition is not a current whole-lens verdict or
checkpoint 3821 evidence. The final 3821 ledger and queue remain unrecovered
in this evidence chain. A time-bound current-ledger literal lookup and its
exact snapshot are included; zero literal matches are not a semantic alias
inventory or evidence that related rows do not exist. `READ-LIMITS.json`
records this reviewer's actual displayed intervals and their complements.

The rawzip package is checksum-bound by the child lock and held separately.
Its earlier bounded privacy scan reported email candidates and nested ZIP/JAR
signatures that were not traversed. Raw commit anchors and complete original
queues are also separately held for verification. No archive, fixture, parser,
product code, build, test, runtime probe, or repair was executed.

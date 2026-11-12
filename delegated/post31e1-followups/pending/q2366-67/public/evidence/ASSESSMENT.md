# q2366 and q2367: bounded source-only review

Current baseline: commit `95e63628f9d70b8dd6e6fede64334696c6995009`.
Historical baseline: `e2755a732c84e67296b5e3441a4259a84f461b2f`.
The IDs below are zero-based indices of the recovered 059 queue `entries` array.
Original and reconstructed queue bytes match for each selected entry. These are
recovered historical records; this assessment is newly written work.

## q2366: record-kind source test owner

The e275 plan and fresh personal whole-file source read were attributed to the
parallel `lens11_reconcile` reviewer; `segovb` subsequently adopted the named
historical read. Plan 199 rows 6 and READ-181 row 6 identify
`crates/fabelgeist-numpy-storage/tests/record_kinds.rs`, 321 lines, SHA-256
`69bcfa97928a9d376cc26bd7945bfebd8a769c48b4fe6f7b4690544d6c52deb8`.
Its 95e source and historical e275 Git path resolve to the same blob. This is
per-file identity, not whole-tree equivalence, and the e275 bytes in this packet
are reconstructed from matching 95e candidate bytes, not recovered original
historical reviewer bodies.

The test text names binary fixtures at lines 12–65, without this review opening
them. Lines 70–166 specify expected admitted names or archive-level rejection
for selected end-record search, ZIP64 and central-record cases, including a
separate missing-end string for an excluded window. Lines 169–189 specify
member error expectations for two bad local record labels after archive
admission. Lines 191–258 express borrowed stored versus owned deflated storage,
selected array shape and float bits, across owned and mapped constructors.
Lines 261–313 express duplicate-name first selection and selected disagreeing
central/local cases; lines 315–321 expect a structural archive error for a
malformed name frame. These are authored expectations, not evidence that binary
fixtures have the indicated contents or that any result occurs at runtime.

Independently inspected first-party source `src/zip.rs:86–109,112–147,160–181`
selects central entries and their metadata, enumerates names in stored order,
chooses the first matching entry, obtains a local entry before compression
selection, and returns borrowed stored or owned deflated bytes. `src/npz.rs:55–69`
selects the first matching directory member. Native record/search parsing, actual
binary payloads, matching breadth, all input cases and product behavior remain
outside this source assessment.

## q2367: error source test owner

The historical plan and personal read have the same reviewer and adopter, with
plan 199 row 7 and READ-181 row 7 identifying `tests/zip_errors.rs`, 217 lines,
SHA-256 `bef087f9ba1c49676479978b296644b5b1340f9470b85f16852cbba5b9b115a5`.
Its e275 and 95e tree paths likewise prove per-file identity, subject to the
same reconstructed-body qualification.

The test text at lines 18–40 expects missing end and missing member to be
separate variants with no source. Lines 43–84 propose mutations of central
compression words on two pre-existing malformed fixtures and expect corrupt
local framing or past-end errors first, retaining rawzip causes. Lines 87–101
expect an unsupported compression code and no invented cause. Lines 104–138
express expected inflate-cause identity and selected `anyhow` downcast/chain
behavior. Lines 140–187 express native open versus map operation, path, and
I/O source preservation; the assertion of OS error 19 is explicitly Linux
and environment qualified. Lines 189–217 express borrowed/owned storage and
bit-preserved zero, subnormal, infinities and NaN. No assertion, macro, fixture,
parser or test was executed in this review.

First-party `src/zip.rs:160–181` obtains native member framing before the
compression match, a static ordering basis for the named precedence expectation.
`src/zip/error.rs:34–69,71–142` retains rawzip causes on archive/member variants
and I/O causes on native/inflate variants, and specifies variant-dependent
formatting and `Error::source`. `src/npz.rs:65–85` calls member bytes before
attaching its NPY parse context. The authored `anyhow` assertions are not
independent proof of actual runtime behavior. Caller/UI presentation, actual
fixtures, architecture/OS effects, dependency internals and policy are open.

## History, relationships and limitations

The e275 records say `named_disposition_recorded` and
`audit_discovery_closed_within_exact_named_scope`; the disposition is limited
to these named authored test-source owners. It is not current global lens,
queue, binary, native, parser, product or runtime closure. The 73 nontext/
no-positive-EOF qualifications and missing checkpoint 3821 qualification remain
unresolved. Earlier rawzip SOURCE004 evidence retains its original scope and
reviewer; this packet adds no native-source credit. q2362/63, q2364/65 and
q2315/16 are related independently attributed reviews, and the three selected
ledger row IDs are relationships only, not adjudicated aliases or defect counts.

Ten cited historical records were copied from protected recovered evidence and
matched their recorded SHA-256 values. The two old test source body hashes found
zero matching filenames within only the older parallel-L11 directory; this
filename-only search does not establish global absence. Source candidates were
accepted only after raw commit-anchor-to-tree-to-blob proof and recorded e275
SHA-256 agreement. A failed physical checkout commit lookup was bypassed with
separately held raw commit anchors and verified tree objects, not ignored.

The first queue hash check failed because a transcribed expected digest omitted
one hex character. The first selected-entry extraction mistakenly used one-based
indexing and displayed array entry 2365 under the q2366 label; that unregistered
historical view, exact wrong extraction, and parent corrections are retained.
Corrected extraction separately checks original/reconstructed queues by SHA-256,
verifies actual zero-based entries 2366 and 2367, and captures exact byte ranges.
Any source display occurred only after the corrected scope and prospective
registration. No product code, build, test, fixture, parser, or runtime probe
was run, and no publication or ledger was edited.

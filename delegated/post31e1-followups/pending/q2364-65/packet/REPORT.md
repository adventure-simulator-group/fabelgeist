# Bounded 059 q2364–q2365: ZIP test-source owners

## Historical dispositions and recovery

This is a fresh, separately attributed `verifier_scope` source-only review
at pinned `95e63628f9d70b8dd6e6fede64334696c6995009` (95e), limited to two
named test-source-owner contracts and their directly connected owners. The
original 059 queue comes from `56f24f6457f6cec8ffcdc46f03f5ff26a8abb50e`;
the public reconstructed queue is from
`26e360920b7b2ec32d5c76238623eaf6d6617d7e`. Their whole streams differ;
the two selected q2364/q2365 byte slices and four related q2315/q2316/q2362/
q2363 slices match across those revisions. This does not promote older
records to checkpoint 3821. Identities and byte offsets are in
`HISTORY-IDENTITIES.json` and `RELATED-IDENTITIES.json`.

Original `REVIEW-L11-ZIP-MEMBER-COUNT-SOURCE-TEST-OWNER` and
`REVIEW-L11-ZIP-EXTENT-SOURCE-TEST-OWNER` each record
`documented-source-owner-disposition-binary-runtime-independent`, with named
disposition and discovery closed within that exact named scope. Both leave
all-lenses/coarse-parent closure, global aliases, repair/runtime passage and
the implementation gate false; each has zero queue body/population credit and
no parent canonical ID. Source expectations are not proof of fixture bytes or
test outcomes. The record reserves the independently qualified SOURCE004
native policy context, all 73 historically noted nontext/no-positive-EOF
contracts and later acceptance; it does not claim global current clearance.

The two named test bodies were personally read at historical e275 by the
parallel `lens11_reconcile` reader under PLAN-199/READ-181; the selected
application owners were the separate same-Lens-11 READ-171 personal reads
under PLAN-191. Their source-row wording says root adoption pending at that
earlier receipt stage. The later root READ-L11-707 (01:14:18 UTC) and
READ-L11-717 (02:55:13 UTC on 2026-10-30) record separate strict delegated
adoptions with original attribution, zero added unique counts, no other-lens
transfer, runtime validation not run and gate false. The associated verifier
records are metadata, not fresh source readings. All ten directly cited
historical records were recovered from the protected original extraction and
hash matched; four selected one-hop agent-verifier and root-state records
were also recovered. Two root-state paths were inferred from batch numbers,
then checked against their cited hashes. The 14 matching same-machine
transport entries do not establish independent custody or complete
transitive recovery. See the historical focus and recovery files.

## q2364: member-count test source

`crates/fabelgeist-numpy-storage/tests/member_counts.rs:11–277` lists ordinary
and ZIP64-labeled cases and expresses expected directory names, order, NPZ
keys and archive-error classifications. Its explicit expectations include
zero, saturation and mismatch cases; a label is not verified fixture
content. Lines 280–320 express owned and mapped error expectations. Lines
322–351 expect a 4,097-member result despite the initial cap and first
selection for duplicate names; lines 353–361 expect malformed selected-name
rejection. None was executed, and included binary fixtures were not read.
Those assertions cannot be reported as observed counts, byte layouts or
passes.

The selected application code at `src/zip/member_count.rs:7–25` keeps the
selected count in u64, caps only initial capacity at 4,096 and preserves a
u64 iteration range. `src/zip.rs:86–109` uses that range as an upper bound,
stops when the native directory reports no next entry, and returns an error
on the selected native error path. `src/zip.rs:112–147` retains directory
order and uses the first matching member; `src/npz.rs:45–69` projects names,
keys and first matching NPZ query. This is a selected application source
trace, not proof of rawzip's complete directory/framing behavior, exact
classic/ZIP64 count admission or a policy decision that actual and declared
counts must agree.

## q2365: extent test source

`tests/zip_extents.rs:7–70` names separate compressed-size, decoded-size and
local-offset fields. Its helper searches its seed bytes for the first central
signature and patches those fields and an extra field; no seed bytes were
inspected, so the actual signature placement or resulting archive is not
established. Lines 73–191 express expectations for saturation replacement,
partial/first extra fields and unverified full-width, zero or mismatched
declarations. Lines 193–234 express stored-borrowed/deflated-owned and
first-duplicate/float-bit expectations. Lines 236–267 express formatting and
out-of-archive offset error expectations. These are source contracts only;
no runtime success, native bounds proof, fixture contents or requirement to
preserve historical diagnostic prose is asserted.

`src/zip/extent.rs:5–42` identifies `DeclaredUncompressedByteLength` as
unverified u64 archive metadata, including zero and maximum; its native
allocation projection uses `as usize`. `src/zip.rs:151–181` returns stored
raw data borrowed and, for Deflate, allocates from that declaration before
`read_to_end`. Thus selected full-width declared-size test text does not by
itself establish a decoded-output budget, CRC/size verification, platform
allocation behavior, or acceptance policy. Actual trusted inputs, resource
limits, integrity requirements and native behavior remain open separately.

## Dependencies and connected records

The six named Rust files (two tests and four first-party owner supports) match
their original e275 hashes and 95e bytes. The standalone crate manifest
changed: its exact registered diff adds dev-profile settings. Its separately
committed child lock is byte-identical e275 to 95e and records rawzip 0.5.2;
the root workspace excludes the crate (`Cargo.toml:50–60`). This proves only
the selected manifest/lock identities and stated dependency, not installed
native code, feature resolution, library behavior or a build. Historic
SOURCE004's own rawzip 0.5.2 archive/member/range qualifications remain with
its original reader; no native source was read for this packet.

Original q2315 records a named rejection of blanket native-cause erasure;
q2316 records a conditional CRC/decoded-size/allocation policy question.
Original q2362/q2363 are separately attributed authored extent and error
document dispositions. Their relationships do not make either test-source
scope a duplicate. The independently reviewed q2362/q2363 packet and
q2366/q2367 test scopes have distinct reader credits.

Four selected centrally assessed live-ledger relationships were read from
a separately held 18:35:33 UTC snapshot, whose whole SHA-256 is recorded in
`LIVE-LEDGER-REGISTRATION.json`. The selected objects are reserialized,
not exact stream slices. `relationship-npz-count-zip64-policy-95e` keeps
count/terminator, ZIP64 consistency and budgets open;
`relationship-npz-member-selection-identity-95e` keeps name/duplicate and
local-central policies open; `relationship-npz-error-context-policy-95e`
retains diagnostic-context questions; and
`relationship-npz-decoded-integrity-budget-95e` connects the unverified
Deflate reservation to pending trust, budget and CRC/size policy. These
partial current relationships are not historical 059 dispositions, exact
aliases, unique-defect counts or 3821 evidence. An independently checkable
membership replay requires the separately held whole snapshot, which is a
same-machine recovery dependency; central reconciliation owns its custody
and any publication/privacy decision.

## Method and limits

The packet captures ten pinned source/manifest/lock files. Seventeen
prospectively registered displays cover 1,269 unique personally displayed
lines; 18,944 captured lines remain unread. Exact ranges, identity checks,
qualified locators, the changed child-manifest diff, historical attributions
and source selections are retained. The first filename locator retained 80
of 271 candidates; its narrower non-delegated rerun retained all 11. These
locators are navigation only, not completeness evidence. A portable verifier
checks identities and optionally Git, transport and separately held ledger
snapshot membership; it does not certify meaning, full archive contracts or
independent custody. The separately sealed bounded scan records privacy
limitations.

Test and doctest text, `include_bytes!`, attributes and derive syntax were
inspected only. No fixture bytes, test, product code, parser, macro, shader,
build or runtime probe was executed; no repair was attempted.

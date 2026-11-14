# Quest open-catalog identifiers: pre-repair audit checkpoint

This checkpoint follows the [weather slice](https://github.com/adventure-simulator-group/fabelgeist/blob/6b819c92fae387544761e83c59b5c56ea7bb408c/delegated/iterative/weather-snapshot-20261114/INDEX.md).
The newly assessed product snapshot is commit
`9591ab3d79485390fd079aefd7e98c5cc088ceca`, tree
`4437959479dd8fff02c87ae3db0f8b4f955464de`, the head of
[PR #1007](https://github.com/adventure-simulator-group/fabelgeist/pull/1007)
at this cutoff. The stack is based on
`95e63628f9d70b8dd6e6fede64334696c6995009`; `main` was observed at that
revision during preparation. That observation can change.

The [ledger](LEDGER-v4.json) records bounded pre-repair work through all
thirteen lenses for the four quest open-catalog identifier types, separate
reader attributions, source traces, conditional questions and acceptance
limits. Uninspected callers, native or generated paths, open policies and
repository-wide lens coverage remain incomplete. Thirty-seven selected source
and rule files were captured from the pinned Git tree; capture is not a claim
that every file was personally read or that the whole tree was preserved.

The source-level constructor inconsistency is the existing f2c ledger entry 2,
`REVIEW-OPEN-CATALOG-CONST-ADMISSION-001`. Its public `from_static` accepts
some values that `try_new` and deserialization reject. The review found no
selected production path passing malformed input to that constructor. This is
not a new defect count or a demonstrated user incident. No repair, product
build, test or runtime probe occurred for this slice before this cutoff.

The intended repair preserves syntactically valid unknown IDs, the existing
string wire representation and the current length-first ordering. Invalid
`from_static` inputs would newly fail const evaluation or assert at runtime;
`try_new` and deserialization remain fallible. The crate's object-test rule and
the absence of an established higher-level caller leave direct negative
constructor regression coverage open; a passing malformed-JSON test alone would
not prove the proposed change. The separate terrain, demographic and ordering
policy questions in the ledger are not resolved by this repair.

## Historical and recovery boundaries

The earlier [95e assessment](https://github.com/adventure-simulator-group/fabelgeist/blob/f2c6d28a48bfa72c0801e9f33387eb6e448f9a18/ASSESSMENT-L01-QUEST-ID-95E.json)
is a distinct bounded assessment. Original registry role 972's `pr-working`
read was recorded against head `6533e444fa37552927070cc8d8190907fd3b413e`
plus working state; physical-p16 role 131 has a separate read without an
explicit revision in the selected record. Matching selected model-file bytes
are per-file evidence, not whole-tree identity or transferred reader credit.
Original checkpoint 3821's ledger and queue remain missing; SOURCE004,
nontext/no-positive-EOF obligations and other recovery limits remain open.
Earlier evidence is not represented as checkpoint 3821.

The f2c and weather checkpoints, earlier originals, historical Git objects and
previous keys remain separate dependencies. Candidate terrain and BodyMesh
selection reports are preserved as selection evidence, not as findings closed
by this slice. No independent backup, full Git-object closure or
repository-wide audit closure is claimed.

## Recoverable evidence

`custody/` holds an AES-256-GCM encrypted archive of the slice's pre-repair
records, source captures and selected weather post-cutoff controls. A fresh key
is separately held on the same machine. Same-machine authentication and
restoration succeeded; an independent comparator matched 124 files, 31
directories, file bytes and modes, root membership and registration. The 154
pre-capture inventory entries also matched the restored copy. The keyed restore
receipt and private-index digest remain restricted. Owners, ACLs, timestamps,
hardlinks, crash durability and independent custody are not established.

From `custody/`, `python3 -I -B VERIFY.py` checks ciphertext identity only.
With the separately held key and a fresh disjoint destination,
`VERIFY.py --key KEY --restore DESTINATION` authenticates and restores the
indexed archive. This needs POSIX, Python, `cryptography`, and trusted,
nonconcurrent parent directories. No-key verification does not authenticate
or prove origin. The envelope's build, restore, publication and key-locator
controls, any subsequent review and later product work are outside this
checkpoint's cutoff and must be retained in a later checkpoint.

- Ciphertext SHA-256: `87f4f4c22af0c63c6758d71ed8747b020b05045ae32726724433874483f765be`
- Public verifier SHA-256: `c4ce2a0f5223d9608404de2dfaa3f8197cd0b17f0d26fe67d787c9a4e8766cf7`
- Ledger SHA-256: `39d954a5c44fe581435c33e3527fb50461c402239fb56913e048c34faae2f3bb`

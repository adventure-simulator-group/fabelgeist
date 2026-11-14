# TerrainPack source follow-up

This source-only follow-up extends the immutable
[terrain discovery checkpoint](https://github.com/adventure-simulator-group/fabelgeist/blob/a74e83de2726beefbaf5e99bc7b5ce6565482ce8/delegated/iterative/terrain-pack-20261114/discovery/INDEX.md)
at product commit `29ecf3ca12e3dfbfacb36b72a56ae130d4fc48fd`, tree
`06a79b077db5283ba95bb1801141a60c21482ac0`. It records bounded source
inspection and historical reconciliation; it does not change the earlier
checkpoint's cutoff or reader attribution.

The builder omits whole chunks that do not intersect a crop and retains the
full tile-derived dimensions of emitted chunks, including true edge
remainders. A selected short entry can still alias a different row under the
currently admitted metadata. Reconstructed 059 q120 and f2c ledger 411
already record that first operation. Whether external partial chunks are
supported, whether the outcome should be absence or error, and whether
geometry should be enforced at load or lookup remain open. The timing of a
check affects corrupt-chunk error precedence and selected fixture admission.
Exact upper-bound ownership is a separate question.

A textual search identified 13 direct non-test `TerrainPack::cell` call
expressions in six source modules. It is not a compiler-resolved or external
call graph. Consumers distinguish absent cells and errors: examples include
water substitution in routing, holes in regional capture, required tactical
samples, and differing handling of optional samples. Later travel and road
paths have separate error-handling operations. Q128's relief handling, q130's
road-plan omission, and q283's worker error formatting are related records,
not automatically the same operation.

The earlier ledger's limited literal canopy search missed a related record.
Reconstructed q1668 asks the percent-to-basis-point and regional admission
question using a hypothetical 255-to-25,500 example. F2c ledger 280 reconnects
it to an assessment of source at `95e63628f9d70b8dd6e6fede64334696c6995009`;
that assessment uses the 101-to-10,100 example in this follow-up. No new
historical unique count is added. Q1668's own source and receipt
arrays remain empty with zero population credit; the separate assessment's
historical reader and locator qualifications do not fill those arrays.

Separately, for sufficiently extreme finite admitted bounds, a route-window
dimension can saturate in a float-to-integer conversion and then overflow at
`+ 1` before checked node-count multiplication. Reconstructed q603 is broadly
related to coordinate and window admission, but its row does not identify
this operation. It is distinct from the previously noted `cell` southern
coordinate addition and builder tile-label negation. Supported extreme input,
external producers, complete aliases and intended geographic policy remain
unproved. The arithmetic examples are symbolic source reasoning: no package or
route was run, and no incident or gameplay impact is claimed.

The [connected ledger](LEDGER-v5.json) retains the thirteen bounded lens
records, links, proposed acceptance and separate discovery, repair and
validation states. Further terrain scope and policy remain open. No terrain
repair, product build, test or runtime probe was performed in this follow-up.

## Recovery and custody

The original checkpoint 3821 ledger and queue remain missing. SOURCE004, the
73 historical nontext/no-positive-EOF contracts, and other recovery obligations
remain open. Earlier records are not promoted into checkpoint 3821. Matching
source bytes do not establish whole-tree identity or transfer reader credit.
The selected reconstructed rows retain their recorded revision and reader
fields, including null or empty fields; separately attributed assessments
remain distinct in the encrypted evidence.

The encrypted archive contains the source follow-up and the previous terrain
checkpoint's post-cutoff controls, including failed and corrected attempts.
Earlier immutable checkpoints, original receipts, Git objects and separately
held keys remain dependencies. The fresh key is separately held on the same
machine; this is not independent backup or full Git object closure.

Before publication, authenticated restoration to a fresh disjoint directory,
an independent same-machine file and mode comparison, and a pre-capture
inventory comparison succeeded. These controls do not establish owners, ACLs,
timestamps, hardlink topology, crash durability or independent custody.
`python3 -I -B VERIFY.py` in `custody/` checks ciphertext identity only. With
the separately held key and a fresh disjoint destination, `VERIFY.py --key
KEY --restore DESTINATION` authenticates and restores the indexed archive.
POSIX, Python, `cryptography`, and trusted nonconcurrent parent directories are
required. The current build, keyed receipt, locator, publication controls and
later work are outside this envelope and must be retained for a later
checkpoint.

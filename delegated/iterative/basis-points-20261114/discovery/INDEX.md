# Unit basis-point types: pre-repair audit checkpoint

This checkpoint follows the [quest identifier repair](https://github.com/adventure-simulator-group/fabelgeist/blob/b50bfc4b626f7aa216b74bf9d07571b6568394db/delegated/iterative/quest-open-catalog-20261114/repair/INDEX.md).
The newly assessed product revision is
`5f84afc713601804ca8f9f096d3781018c59a63d`, tree
`8364b9c9f3b65989831943bd8d3da07d5f5b97c9`, head of
[PR #1008](https://github.com/adventure-simulator-group/fabelgeist/pull/1008)
at this cutoff. The pinned stack base is
`89cc57b63215086b11410cc27b8c274944734ad3`.

The [ledger](LEDGER-v5.json) records bounded pre-repair reviews through all
thirteen lenses for `UnitBasisPoints` and `SignedUnitBasisPoints`, their
admission, operations and selected consumers. Uninspected producers, native
and serialized ingress, deployed features, policies and global aliases remain
open. Thirty-one selected source and rule files were captured at the product
revision; their inclusion is not a claim of personal reading or full-tree
preservation.

The existing 0–10,000 unsigned and −10,000–10,000 signed constructor bounds
are bypassed by the current transparent Serde implementation. The feature-gated
SpacetimeDB native decoder is a separate path. A locked offline feature query
and a later local compiler macro expansion show that the selected feature build
produces named one-field native products whose visitors construct the primitive
fields directly; generated native validation checks the primitive/product
shape. The expansion is preserved with its command and review. No malformed
payload was decoded, no runtime panic or player impact was observed, and the
selected search found no current table or reducer containing either exact unit
owner. This is not a new unique defect count.

The intended repair must consider both independent decoding routes while
preserving valid numeric Serde forms, native product names and fields, explicit
saturation and the separate land-use sum invariant. Previously admitted
out-of-range values should be rejected under the repository's clean-schema and
no-compatibility rule; actual stored invalid values and their consequences are
unproved. A schema-preserving native design is reviewed but remains
unimplemented and unvalidated. The neighboring raw-backed settlement role
wrappers and their conditional native ingress are distinct open questions.

## Historical and recovery boundaries

The existing [95e Serde assessment](https://github.com/adventure-simulator-group/fabelgeist/blob/f2c6d28a48bfa72c0801e9f33387eb6e448f9a18/ASSESSMENT-L01-BASIS-WIRE-95E.json)
is ledger entry 27 in the same first-failure family as recovered checkpoint 059
q131 and ledger entry 443. The q131 `basis_points.rs` source row records
`pr-working` with null revision, Git path and reader attribution; its separate
Adler337 receipt is preserved as recorded. An earlier version of this slice's
ledger ambiguously attached attribution to that row; the correction and review
are retained. The PL13 native read and q2332 are separately attributed
relationships, not automatically global aliases. Matching selected file hashes
are not whole-tree identity or transferred reader credit.

The original checkpoint 3821 ledger and queue remain missing. SOURCE004,
nontext/no-positive-EOF and other recovery obligations remain open. No earlier
record is represented as checkpoint 3821. The f2c baseline, prior checkpoints,
locked Serde/SATS dependency evidence, protected original receipts, selected
Git objects and earlier keys remain dependencies; no full Git closure or
independent backup is claimed.

## Recoverable evidence

`custody/` contains an authenticated AES-256-GCM archive of this bounded slice,
its compiler expansion, selected source files, attributed reviews, ledger
versions and the previous repair checkpoint's post-cutoff controls. The fresh
key is separately held on the same machine. Authentication and restoration
succeeded; an independent comparator matched 129 files, 33 directories,
source bytes and modes, root membership and registration. All 161 pre-capture
inventory entries also matched the restore. Owners, ACLs, timestamps, hardlink
topology, crash durability and independent custody are not established.

From `custody/`, `python3 -I -B VERIFY.py` checks ciphertext identity only.
With the separately held key and a fresh disjoint destination,
`VERIFY.py --key KEY --restore DESTINATION` authenticates and restores the
indexed archive. POSIX, Python, `cryptography`, and trusted nonconcurrent parent
directories are required. No-key verification does not authenticate or prove
origin. The builder scripts retained inside the archive need their separately
embedded registration and package layout to run; the keyed restore receipt and
private-index digest remain restricted.

The envelope's own build, restore, publication, key-locator controls and later
work are outside this cutoff and must be retained with a later checkpoint. No
repository-wide audit closure is claimed.

- Ciphertext SHA-256: `b6eef39811e062299f6f1ddafce4eb6e005dfe5217077aa1751b71c3038bd8fa`
- Public verifier SHA-256: `3fd9c20dd3697ae8c1df016553041bbd6d0fc925bc35502b3a32ea08048bc51f`
- Ledger SHA-256: `53f0950c3faf26b8dba6f37f2796c5e9c903a97ea57f3fab224811c42d671e1f`

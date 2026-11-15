# TerrainPack geometry source assessment

This source-only checkpoint extends the immutable
[terrain policy assessment](https://github.com/adventure-simulator-group/fabelgeist/blob/0ae59fa2732116f93f737710a15193b2c5925381/delegated/iterative/terrain-pack-20261114/policy/INDEX.md)
at product commit `29ecf3ca12e3dfbfacb36b72a56ae130d4fc48fd`, tree
`06a79b077db5283ba95bb1801141a60c21482ac0`. Historical records retain their
recorded revisions, readers and cutoffs,
including recorded nulls; identical bytes do not transfer credit.

The selected compiler emits full native chunks or true tile-edge remainders
and can omit whole chunks outside a crop. The current loader admits some other
declared shapes. For one source-derived example, a 1-by-2 declaration at the
first chunk of a larger tile can make a lookup for the next column use the
second row instead. This is a symbolic trace, not an observed package or
runtime incident. The operation was already recorded in reconstructed q120
and f2c ledger 411, with a separately attributed 95e assessment. Selected
q120 source rows retain their recorded null revision, Git path and reader.

A new bounded thirteen-lens geometry inventory proposes checking widened
chunk ordinals and using geometry derived from tile dimensions and chunk
position. It does not establish all external producers, tile/bounds
intersections, complete geographic coverage or global lens clearance. The
tracked builder uses the same computed width and height for both encoding and
wire declarations. Retaining those fields as checked assertions can reject
an explicitly mismatched declaration early; removing the derivable fields
would simplify the schema and may defer a short-payload error until decoding.
Neither proves actual row order from bytes. The central choice and its wire,
identity, producer, consumer and error-timing consequences remain open.

Two source lens notes called the manifest signed. The loader checks an
unkeyed self-digest, not a cryptographic signature or producer authentication.
The original notes are preserved unchanged, with that correction and the
assertion tradeoff in the [ledger](LEDGER-v11.json).
The ledger retains nine earlier findings and separates related geographic,
cache, allocation, fraction, race and error-handling operations. No terrain
repair, product build, test or runtime probe was performed in this assessment.

## Recovery and custody

The original checkpoint 3821 ledger and queue remain missing. A historical
[SOURCE-004 proposal bundle](https://github.com/adventure-simulator-group/fabelgeist/blob/f2c6d28a48bfa72c0801e9f33387eb6e448f9a18/delegated/postc7dc-followups/npz-native-q2316/historical-originals/parallel-l11-20261029/PROPOSAL-BUNDLE-L11-SOURCE-004.json)
matches the extracted original and its registered hash on this machine. Its
recorded snapshot is `e2755a732c84e67296b5e3441a4259a84f461b2f`; its six
original qualifications and historical partial and completed ranges remain
accessible. Its native qualification and receipt obligations remain open.
The 73 historical
nontext/no-positive-EOF contracts and other recovery obligations remain open.
No earlier record or new assessment is promoted into checkpoint 3821.

The encrypted archive contains the new source assessments, reviews, central
reconciliation, a recheck of the already-addressed #1007 review, and the
previous checkpoint's post-cutoff controls including its stage-review
correction. It also retains the complete historical Lens 11 receipt and the
registration for its cited terrain source, used for the related
cache-accounting question. The prior
[weather-review checkpoint](https://github.com/adventure-simulator-group/fabelgeist/blob/25d7e8621e7939ff553f58c9a994630c3a39ac26/delegated/iterative/basis-points-20261114/repair/INDEX.md)
retains the earlier repair and validation evidence. Earlier immutable
checkpoints, original receipts, Git objects and separately held keys remain
dependencies. Same-machine custody is not an independent backup or full Git
object closure.

Publication requires authenticated restoration to a fresh disjoint directory,
an independent same-machine source byte and mode comparison, a pre-capture
inventory comparison, and finite selected-key and path scans. These controls
do not establish owners, ACLs, timestamps, hardlink topology, crash durability
or independent custody. `python3 -I -B VERIFY.py` in `custody/` checks only
ciphertext identity without a key. To authenticate and restore with the
separately held key and a fresh disjoint destination, set `RECEIPT` to a
fresh path outside the public directory in a restricted parent, then run:

```sh
(umask 077; set -C; python3 -I -B VERIFY.py --key KEY \
  --restore DESTINATION > "$RECEIPT")
```

The new receipt contains a private-index digest; keep it restricted and do
not display it. The restored private index is also restricted. POSIX, Python,
`cryptography` for keyed authentication, and trusted nonconcurrent parent
directories are required. This build's keyed receipt, locator, publication
controls and later work are outside the envelope and must be preserved with a
later checkpoint.

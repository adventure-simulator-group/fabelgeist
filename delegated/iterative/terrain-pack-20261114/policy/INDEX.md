# TerrainPack policy and resource follow-up

This source-only checkpoint extends the immutable
[TerrainPack follow-up](https://github.com/adventure-simulator-group/fabelgeist/blob/016c74c51f8e86a8f50400d062e67954fe90ea62/delegated/iterative/terrain-pack-20261114/followup/INDEX.md)
at product commit `29ecf3ca12e3dfbfacb36b72a56ae130d4fc48fd`, tree
`06a79b077db5283ba95bb1801141a60c21482ac0`. Earlier records keep their own
revision, reader and cutoff; matching file bytes do not transfer credit.

The tracked terrain compiler emits full interior chunks and true tile-edge
remainders, and may omit whole chunks outside a crop. Exact geometry admission
would enforce that producer's layout, but the public loader currently admits
other partial or padded layouts. That policy choice affects error timing and
existing fixture purposes. A later axis check would stop the documented row
substitution while leaving other malformed geometry admitted. The row-alias
contract was already recorded in reconstructed q120 and f2c ledger 411.

Runtime-bundle construction checks strict geographic bounds, but the selected
public builder and loader accept broader finite ordered bounds, and the
release verify/install paths do not themselves rerun that source check. A
strict package-bound policy could exclude the previously described extreme
cell and route-window examples on the selected 32-bit-or-wider targets.
Intermediate native samples may intentionally fall outside package coverage.
An exact integral upper edge also has a separate conditional coverage question.

The strategic route cache normalizes nearby endpoints. A warm hit can bypass
native coordinate admission; a cold miss can reach the native out-of-bounds
result after its other prerequisites. This is the same first contract as
reconstructed q194 and f2c ledger 782, not a new historical finding. Q194's
selected `pr-working` source rows retain their recorded null revision, Git
path and reader fields; separately attributed receipts and assessments remain
distinct. The examples are source arithmetic, not observed routes or gameplay.

Source-raster canopy no-data and finished-pack admission are different
boundaries: the selected raster initializer uses 255 for no data, and the
terrain builder currently maps source values above 100 to zero. Public cells,
regional wire samples and tactical grid validation have distinct admission
paths. The earlier q1668 question and separately attributed 95e assessment
remain the historical relationship; no new unique historical count is added.

The terrain chunk cache charges five decoded wire bytes per retained cell
against 32 MiB. Its typed cell arrays and other allocations are not a 32 MiB
resident-memory ceiling. Separately, an otherwise digest-consistent package
can declare a large compressed entry; on a cache miss the current code
attempts to allocate its whole range after checking declared output size but
before validating actual decompressed length and digest. The loader rejects
an observed pack metadata size above 2 GiB and bounds entries by that earlier
observation; a separate metadata/read race can change bytes later consumed.
Intended resource policy, supported malformed-input reach and actual memory
impact remain open; no allocation or package was executed.

The [connected ledger](LEDGER-v8.json) preserves the thirteen bounded lens
records, distinct source questions, their historical links, proposed
acceptance and separate discovery, repair and validation states. The policy
reviews narrow decisions but do not complete the remaining terrain scope,
supported-producer census or historical aliases. Bounded peers checked the
geometry and geographic reports; later ledger checks covered selected cache
and fraction claims with reviewer overlap disclosed in their reports.
No terrain repair, product build, test or runtime probe was performed here.

## Recovery and custody

The original checkpoint 3821 ledger and queue remain missing. SOURCE004, the
73 historical nontext/no-positive-EOF contracts, and other recovery obligations
remain open. Earlier records are not promoted into checkpoint 3821. Selected
reconstructed rows retain their recorded null or empty fields; separately
attributed evidence remains separate.

The encrypted archive contains the new source reviews, their peer reviews,
central reconciliation and the previous checkpoint's post-cutoff controls.
Earlier immutable checkpoints, original receipts, Git objects and separately
held keys remain dependencies. The fresh key is separately held on the same
machine; this is not an independent backup or full Git object closure.

Before publication, authenticated restoration to a fresh disjoint directory,
an independent same-machine file and mode comparison, and a pre-capture
inventory comparison succeeded. These checks do not establish owners, ACLs,
timestamps, hardlink topology, crash durability or independent custody.
`python3 -I -B VERIFY.py` in `custody/` checks ciphertext identity only. To
authenticate and restore with the separately held key and a fresh disjoint
destination, set `RECEIPT` to a fresh path outside the public directory in a
restricted parent, then run the following from `custody/`:

```sh
(umask 077; set -C; python3 -I -B VERIFY.py --key KEY \
  --restore DESTINATION > "$RECEIPT")
```

The newly created receipt contains a private-index digest; keep it restricted
and do not display it. The restored private index is also restricted. POSIX,
Python, `cryptography` for keyed authentication, and trusted nonconcurrent
parent directories are required. This build's keyed receipt, locator,
publication controls and later work are outside the envelope and must be
preserved with a later checkpoint.

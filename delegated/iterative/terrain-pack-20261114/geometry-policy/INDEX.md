# Terrain geometry policy and release inputs

This source-only checkpoint extends the
[terrain loader and release assessment](https://github.com/adventure-simulator-group/fabelgeist/blob/a2291420ea899260fa359747c05a286a4e0baee1/delegated/iterative/terrain-pack-20261114/release-followup/INDEX.md)
at product commit `29ecf3ca12e3dfbfacb36b72a56ae130d4fc48fd`, tree
`06a79b077db5283ba95bb1801141a60c21482ac0`. Earlier records retain their
own revisions, readers and cutoffs, including recorded nulls. Identical bytes
do not transfer assessment credit.

The proposed next repair derives a checked full-or-true-edge chunk shape from
native tile dimensions and chunk ordinals and specifies row-major producer
encoding. The current reader instead uses separately declared chunk dimensions
for both decoded length and row stride; a symbolic short-width declaration can
select a cell from another row. Deriving shape removes that competing
operational authority. Retaining dimensions as checked assertions is a
different defensible contract: it can reject a truthfully declared equal-area
wrong shape earlier. Neither approach proves that arbitrary same-length bytes
encode the producer's intended source cells.

The reconstructed q120 acceptance asks for valid placement and exact
source-cell behavior as well as preventing out-of-extent row substitution.
A future repair must validate nonuniform known source cells through the
supported producer and consumer, while stating the producer trust boundary.
That evidence will cover selected cases, not arbitrary external producers.
The q120 crop, alias, current-source and other acceptance obligations remain
open. No thirteen-lens or historical finding is declared clear here.

A separately preserved manifest matching one tracked runtime-lock member has
483 entries, all with dimensions matching their recorded native tile and
ordinal. An independent calculation confirmed the result. This checks the
manifest's declarations, not decoded payloads, complete geographic coverage,
runtime loading or historical build provenance. Its entries use
2,400-by-3,600 tiles; the observation does not cover the other supported tile
widths.

A separate source-bundle finding is conditional. The pinned bundle code
accepts an exact Viabundus file set that omits `water-1500.csv`, while the
strategic map builder requires that file and a matching sidecar entry. After
a successful fresh or replacement install of that component, the normal
initializer sees the five bundled core CSVs and does not add water. Without
separate compatible hydration, the map build cannot obtain the required input
after earlier checks pass. This is a source trace, not an executed install or
build. The bundle, initializer, sidecar, derived-profile identity and release
artifacts require coordinated consideration; this operation is distinct from
chunk geometry and runtime object publication.

Two older same-machine runtime ZIP copies match an older installation marker.
One is retained in this checkpoint; all seven marker-listed members were
streamed and matched by size and SHA-256, with four also matching current
lock members. The archive is the older release and lacks the documented-base
terrain and regional-road pair under the inspected names. Neither it nor the
bounded local searches establish a complete rebuild input set or current
remote availability. The [ledger](LEDGER-v13.json) records the decisions,
findings, dependencies, acceptance proposals and their limits.

## Recovery and custody

The original checkpoint 3821 ledger and queue remain missing. The original
Adler builder registration was recovered and matched to its separately
attributed receipt; it does not supply the null revision, Git path or named
reader of selected reconstructed q120 rows. SOURCE-004 proposal bytes were
identified separately, while native qualification and receipt obligations
remain open. The 73 historical nontext/no-positive-EOF contracts and other
recovery obligations also remain open. No earlier record or new assessment is
promoted into checkpoint 3821.

The encrypted archive includes these assessments, bounded peers, one older
runtime ZIP, and the previous checkpoint's post-cutoff controls. The previous
continuation archive and its inherited earlier archives, Git objects and
separately held keys remain dependencies. Same-machine custody is not an
independent backup or full Git object closure. No product repair, build, test,
runtime probe, release upload or database operation was performed in this
assessment. Evidence-only scripts, archive-member reads and byte checks ran.

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

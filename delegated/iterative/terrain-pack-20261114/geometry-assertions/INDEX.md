# Terrain geometry decision and rebased stack

This checkpoint extends the [terrain geometry policy and release-input
assessment](https://github.com/adventure-simulator-group/fabelgeist/blob/d6fd3838d6c03d12025467d6c154ab6b5891a241/delegated/iterative/terrain-pack-20261114/geometry-policy/INDEX.md).
That source assessment remains at product commit
`29ecf3ca12e3dfbfacb36b72a56ae130d4fc48fd`, tree
`06a79b077db5283ba95bb1801141a60c21482ac0`; its earlier readers, evidence
cutoffs and recorded nulls retain their original attribution. The observed
rebased combined stack tip was `e47535b6968bf86b39312ab0436e89d885201c46`,
whose delta from that tree is the separately preserved quest import review
fix. This bounded comparison is not an assessment of all indirect behavior.

The coordinator now selects retaining terrain chunk width and height as
checked assertions of the encoder's row-major shape, while using only geometry
derived from native tile dimensions and chunk ordinals for decoding and cell
lookup. This changes the prior checkpoint's provisional derive-only
recommendation. The reason is a real semantic tradeoff: a truthful but wrong
16-by-8 assertion for a canonical 8-by-16 edge has the same cell count and can
be rejected by checking both axes. Neither assertion nor hash proves that
arbitrary same-length bytes contain the intended source cells. The example is
hypothetical; a terrain repair and its feature validation remain future work.
The [connected ledger](LEDGER-v14.json) preserves the earlier bounded lens
records and the competing policy reasoning, new source references, selected
acceptance proposals and remaining questions.

After PR 1007 squash-merged, its tree was checked against the earlier head.
The remaining stack was replayed while preserving a concurrent review-fix
commit and published with explicit leases. At the recorded observation, PR
1008 was based on main and PR 1009 was stacked on it. Formatting, full lint,
the existing 61-test quest
suite and one investigation regression passed on the rebased combined tree.
Head, tree and nonignored status were checked before and after, not atomically
throughout each run. Earlier test results retain their earlier tree and limits;
signed and native basis-point decoding cases remain open. The current stack
does not establish terrain product validation.

## Recovery and custody

The original checkpoint 3821 queue and ledger remain missing. Selected q120
records are reconstructed with null revision, Git path and named-reader
fields; the separately attributed Adler and 95e records do not fill them.
SOURCE-004 proposal bytes were separately recovered, while native
qualification and receipt obligations remain open. Seventy-three historical
nontext/no-positive-EOF obligations also remain open. No earlier record or new
assessment is promoted into checkpoint 3821.

The encrypted evidence includes the previous checkpoint's post-cutoff
controls, new peer assessments, rebase attempts and validation evidence. The
earlier immutable archives, product Git objects and separately held keys remain
dependencies. The included Git bundles require their named prerequisites and
are not complete closure. Same-machine copies are not an independent backup.
The 483-entry manifest observation checks declarations only, not decoded
payloads, full geography, installation or release provenance. The conditional
source-bundle water gap and the separate object-store race remain open.

Publication requires a fresh authenticated restore, comparison of source
bytes and modes, pre-capture inventory checks and finite selected-key and path
scans. Those controls do not establish owners, ACLs, timestamps, hardlink
topology, crash durability or independent custody. `python3 -I -B VERIFY.py`
in `custody/` checks ciphertext identity without a key. To authenticate and
restore with the separately held key and a fresh disjoint destination, set
`RECEIPT` to a fresh path outside the public directory in a restricted parent,
then run:

```sh
(umask 077; set -C; python3 -I -B VERIFY.py --key KEY \
  --restore DESTINATION > "$RECEIPT")
```

The keyed receipt contains a private-index digest; keep it restricted and do
not display it. The restored private index is also restricted. POSIX, Python,
`cryptography` for keyed authentication, and trusted nonconcurrent parent
directories are required. This build's keyed receipt, locator, publication
controls and later work are outside the envelope and require later custody.

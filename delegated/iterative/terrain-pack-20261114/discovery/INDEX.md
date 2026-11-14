# TerrainPack addressed-cell discovery

This source-only checkpoint assesses the `TerrainPack` addressed-cell path at
product commit `29ecf3ca12e3dfbfacb36b72a56ae130d4fc48fd`, tree
`06a79b077db5283ba95bb1801141a60c21482ac0`. That commit was observed as the
head of
[PR #1009](https://github.com/adventure-simulator-group/fabelgeist/pull/1009),
stacked on #1008 and #1007. The preceding
[basis-point repair checkpoint](https://github.com/adventure-simulator-group/fabelgeist/blob/25d7e8621e7939ff553f58c9a994630c3a39ac26/delegated/iterative/basis-points-20261114/repair/INDEX.md)
is a dependency.

## Conditional source findings

- A self-consistent chunk declared as one cell wide by two high can map local
  coordinate `(1, 0)` to the stored cell at `(0, 1)`. The flattened index is in
  range even though the x coordinate is outside the declared chunk. This is
  the first operation already recorded by reconstructed 059 q120 and f2c
  ledger entry 411; it is not a new unique historical count.
- Separately, finite ordered manifest bounds can admit a tile whose southern
  coordinate is `i16::MAX`; public lookup then evaluates `south + 1` in
  `i16`. F2c ledger 417 describes a different builder operation at
  `i16::MIN`. The two are related by geographic-bounds admission, without an
  established alias.
- Separately, a digest-consistent decoded cell can contain canopy value 101.
  The selected builder normalizes source values above 100; dispatcher
  conversion would produce 10,100 basis points, exceeding a later tactical
  grid bound. The intended external package contract and actual reach remain
  open.

These are source-level conditional witnesses. No corresponding package or call
was executed, and supported malformed input, deployment reach and gameplay
impact are not established. No terrain repair, product build or test was
performed in this slice.

## Decisions and dependencies

The chunk's intended extent, missing-chunk policy, and absence-versus-error
outcome remain open. The current lookup decodes the selected chunk before
flattening, so moving an axis guard ahead of decoding could change corrupt-chunk
error precedence. Routing converts an absent cell to water but propagates an
error. Tactical callers, strategic foraging, cultivation sampling and road
inference make different choices about errors and absence. Reconstructed 059
q128 and f2c entry 441 concern relief-sample errors; q130
and f2c entry 442 concern the separate road-plan omission. Entry 441 also
relates q283, whose recorded first operation is worker/CLI error formatting.
Those relationships do not establish that q128 and q283 are aliases.

The [connected ledger](LEDGER-v3.json) records three reports covering the
thirteen bounded lenses, source references, proposed acceptance and separate
discovery, repair and validation statuses. The reports cover the selected
paths, not the whole terrain crate or all consumers; discovery remains
incomplete and no lens is claimed globally clear. A possible future enum
wire-order hazard and other ownership opportunities are kept distinct from
demonstrated mismatches.

## Recovery and custody

The historical assessment revision is
`95e63628f9d70b8dd6e6fede64334696c6995009`; selected matching source blobs do
not establish whole-tree identity or transfer reader credit. The selected
`pr-working` source rows of reconstructed q120, q128 and q130
retain null revision, Git path and named reader fields; separately identified
receipts do not fill those nulls. Q283 separately records revision
`6533e444fa37552927070cc8d8190907fd3b413e`, path
`crates/adventuresim-world-import/src/bin/build-strategic-map.rs`, and reader
`Subagent /root/lens12_frontier personal prospective bounded untruncated read`.
The original checkpoint 3821 ledger and queue remain missing. SOURCE004, the
73 historical nontext/no-positive-EOF contracts, and other recovery
obligations remain open. No earlier record is promoted into checkpoint 3821.

The encrypted archive contains the terrain slice and the prior basis repair
checkpoint's post-cutoff controls, including the PR publication observation.
Its fresh key is held separately on the same machine. Earlier immutable
checkpoints, original receipts, Git objects and earlier separately held keys
remain dependencies. This is not an independent backup or full Git closure.

Authenticated restoration succeeded to a fresh disjoint directory. An
independent same-machine comparator matched 131 files and eight directories,
their bytes and modes. A separate check matched 138 pre-capture inventory
entries; the full comparator also checked the self-excluded inventory itself.
These checks do not establish owners, ACLs, timestamps, hardlink topology,
crash durability or independent custody.

From `custody/`, `python3 -I -B VERIFY.py` checks ciphertext identity only.
With the separately held key and a fresh disjoint destination,
`VERIFY.py --key KEY --restore DESTINATION` authenticates and restores the
indexed archive. POSIX, Python, `cryptography`, and trusted nonconcurrent
parent directories are required. The current build and keyed receipt, key
locator, publication controls and later work are outside this envelope and
must be preserved with a later checkpoint.

- Ciphertext SHA-256:
  `f14d77c88fa6b868d7ed5f0ac16d56696c28533d3f95b603d93aa27dd2e76807`
- Public verifier SHA-256:
  `9eed4782526631597b1a09a099312950f7ef298a30855950afc9af1c6a39a964`
- Ledger SHA-256:
  `f671ad876d5447f90ce6dce94706afeb63caa76fb37f9143fb9c37df3ed22dad`

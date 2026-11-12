# Bounded NPZ q2307 reconciliation

`REPORT.json` records the current source trace, historical attribution, related
ledger questions and remaining work. The baseline is pinned to 95e; the
original history retains e275 and checkpoint 059 identities. The missing final
checkpoint 3821 ledger and queue are not reconstructed here.

Historical queue, metadata and source views were prospectively registered.
Numbered displays and receipts identify what was actually read. Exact original
historical files and byte slices are separate from reconstructed JSON
projections and the path-redacted public queue. `READ-LIMITS.json` preserves
read and metadata-search limits. The registered metadata search is not a
semantic alias search.

`sources/` contains the eight exact source files and `git-trees/` contains the
canonical Git tree bodies. `connector-trees/` contains normalized connector
responses used for two additional tree bodies. Their Git identities are
checked, not inferred from revision labels. The e275 and 95e source subtrees
are identical, but the crate manifests and full roots differ.

The raw commit anchors contain public author/committer email metadata and are
not included in this privacy-scanned packet. `ANCHORS.json` identifies their
exact separate bytes and preexisting publication locations. The packet can be
moved together with those separately controlled anchors, or verified against
the prior publication:

```text
python3 verify-evidence.py --anchors-dir PATH_TO_RAW_COMMIT_FILES
python3 verify-evidence.py --publication-root PATH_TO_PUBLICATION
```

Append `--original-queue PATH_TO_ORIGINAL_059_QUEUE` to verify the original
full queue hash and ten exact entry slices. That file contains unrelated local
paths and remains in the protected source extraction. Without it, the verifier
reports the limitation. `--check-metadata`, used with `--publication-root`,
replays the bounded search only if the captured source-file hashes still
match; the working ledger can change. These checks are evidence verification,
not product testing.

`MANIFEST.json` records every packet file other than itself. A separately
retained bounded scan covers the completed packet; it does not cover Git
history, excluded anchors or original full queue, or establish general privacy.
All custody described here is same-machine, not independent custody.

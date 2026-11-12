# Bounded changed-commit privacy review

This packet inventories new blob streams in the specified commit compared
with its direct parent. `REPORT.json` records findings, classification, exact
scope, baseline distinctions and exclusions. `CHANGED-BLOBS.json` records each
changed new blob and its stored identity. `CANDIDATE-LOCATIONS.json.gz` retains
privacy-safe per-occurrence locations and opaque groups without matched values,
email digests or context. `GIT-COMMIT-OBJECTS.json` lists author-bearing commit
object occurrences and the parent checkpoint19 comparison.

`SCAN-METHOD.py.txt` is inert source text of the evidence scanner, including
its signatures and bounds. A re-run requires the target Git repository and a
private output file; its context-bearing results need separate privacy review.
`python3 verify.py` reads only this packet. Optionally pass `--repo <path>`
to check pinned Git blobs, nested commit objects, parent checkpoint19 and the
four upstream member identities without extraction or product execution.

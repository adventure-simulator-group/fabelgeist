# 059 baseline privacy reconstruction proposal

This directory contains proposed public reconstructions of the historical
`baseline-059/queue.json` and `baseline-059/ledger.json`. The originals are
identified by their unchanged historical SHA-256 values, lengths, Git blobs,
and transport paths in `RECONSTRUCTION.json`. The proposed public bytes have
separate identities. They replace only 12 and 14 original local path values,
respectively; all other bytes are retained.

`REPORT.json` describes affected exact-byte checks, historical attribution,
implementation options and unresolved scope. `HASH-LINK-INVENTORY.json` gives
checkpoint and verifier bindings. The loose and delegated-archive inventories
record literal matches, captured hashes, and bounded omissions. The separately
attributed previous transport review is represented as a filtered identity
index, without transferring source-read credit.

The exact originals and the original-to-marker map are retained in a restricted
local recovery directory outside this public directory. The local user-supplied
extraction was read only. No public data was changed and no Git history was
rewritten. Earlier published commits and exports may still contain the original
paths; correcting the current files alone cannot purge them.

Run `python3 verify.py` to check the public representation and inventories.
Supply `--private-dir PATH` to compare the exact retained originals and prove
reversibility. Add `--check-extraction` to reread the original user-supplied
files identified by the private recovery record. Verification is read-only and
performs no product, shader, test, fixture, build or runtime execution.

The original immutable checkpoint manifests and source references continue to
identify the original bytes. A future integration needs separately qualified
successor checks for reconstructed public bytes; this proposal does not claim
that existing exact-byte checks pass with the replacement files.

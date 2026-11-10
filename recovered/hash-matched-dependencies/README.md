# Hash-matched historical dependencies

These 81 candidate files match SHA-256 hashes cited by
separately recovered historical JSON files. The manifest records the cited
path, citing record and candidate source ordinal. A match identifies historical
bytes; it does not prove the latest version, complete dependency graph or
audit authority. The files include scripts and test fixtures: treat them as
inert evidence and do not execute them without independent review.

`DEPENDENCY-COVERAGE.json` inventories 1,486 path/hash citations in the
212 recovered JSON files. Its limited inventory located 189 distinct cited hashes in the recovered files,
dependency objects or original manifest; 528 distinct cited hashes were not
located. Some matches are hash-only. Two were found under archived names, including a
historical ledger snapshot; they are not the final ledger. These
counts overlap other evidence and do not prove those bytes are irretrievable.

The `bevy_mikktspace` 1.0.0 crate archive was recovered from the local Cargo
cache; its own metadata declares `Zlib AND (MIT OR Apache-2.0)`.

The terminal interpolation registration was reconstructed from an exact
base registration, the recorded edit and a bounded timestamp search; its
bytes match the historical cited SHA-256.

To reproduce that registration, run
`python reconstruct_terminal_registration.py NEW_DESTINATION`. The script
refuses an existing file and verifies the result against the cited hash.

The private-readiness registration was reconstructed from pinned Git source
rows, the recorded write and a bounded timestamp search; its bytes match the
historical cited SHA-256.

Run `python reconstruct_private_readiness.py REPOSITORY NEW_DESTINATION`
to reproduce the second registration from pinned Git bytes. It refuses an
existing destination and verifies the cited hash.

Fourteen additional registrations were reconstructed from narrowly parsed
recorded templates, pinned Git source bytes, and bounded timestamp searches.
Their exact bytes match independently cited hashes; original commands were
not executed.

Six more registrations were reconstructed from recorded helper templates,
pinned Git bytes and bounded timestamp searches. The cited hashes match; the
recorded helpers were not executed.

Thirty-seven additional JSON values were reconstructed from printed command
output and matched cited hashes. Output alone does not independently prove the
cited path, latest authority or acceptance.

[LATER-MATCHES.json](LATER-MATCHES.json) reconciles 44 of the earlier missing
hashes with subsequently recovered bytes. The remaining 484 are unmatched only
within this limited inventory. These matches overlap other counts.

A repeatable scan of 10,352 distinct preserved outputs decoded 13,319 JSON
values starting on a line. None matched the 484 hashes with the tested
serializations. Other encodings, embedded values and sources remain untested.
Run `python recovered/hash-matched-dependencies/scan_unmatched_output_values.py`
from the package to reproduce the scan.

The separate private-snapshot scan also checks nested objects and arrays in
65,972 distinct tool-output blocks. It found no matching hashes. With the
original hash-verified private snapshot available, reproduce it with:

```sh
python recovered/hash-matched-dependencies/scan_private_embedded.py SNAPSHOT NEW_DESTINATION
```

The destination is created only if matches exist and must not already exist.
The snapshot itself is private and is not part of this package.

A content-hash scan of three surviving pinned Git trees checked 31,726
text-blob instances (with overlap) against the 484 hashes and found no match.
Only selected text-like extensions and blobs up to 4,000,000 bytes were
checked. Reproduce with:

```sh
python recovered/hash-matched-dependencies/scan_git_trees.py SOURCE_REPOSITORY PROGRESS_REPOSITORY
```

A later scan checked 141 newly recorded state, plan, verification and final
checkpoint hashes against nested private transcript output, including wrapped
tool results. No exact matches were found. With the hash-verified private
snapshot, reproduce it with:

```sh
python recovered/hash-matched-dependencies/scan_state_hashes.py SNAPSHOT NEW_DESTINATION
```

The destination is created only if matches exist and must not already exist.
Review any recovered content before publication. Truncated output and sources
outside the snapshot remain untested.

# Quest open-catalog identifiers: repair checkpoint

This additive checkpoint follows the [pre-repair discovery](../discovery/INDEX.md)
at `8a9f57d00bf5b28474d286ef64d010d7bc10a8eb`. The repaired slice is commit
`5f84afc713601804ca8f9f096d3781018c59a63d`, tree
`8364b9c9f3b65989831943bd8d3da07d5f5b97c9`, stacked on weather commit
`210d840aec9f4577fdf871f4ba1057f0c4450f78` in
[PR #1007](https://github.com/adventure-simulator-group/fabelgeist/pull/1007).
The quest PR had not yet been opened at this cutoff.

The earlier party PR was merged, so the stack was rebased onto pinned main
`89cc57b63215086b11410cc27b8c274944734ad3`. The weather and quest patches are
each byte-identical before and after rebase. The selected quest source and rule
inventory, and the selected weather files, had no changed paths in the main
advance; this is a bounded comparison, not whole-tree identity. Historical
source reviews remain attributed to their original `9591` snapshot. The weather
PR was retargeted to main and its two-file scope verified.

The four quest-generation open-catalog ID types now share one const-compatible
1–63-byte grammar and capacity across `from_static` and `try_new`. Valid open
IDs, string serialization, representation and length-first ordering are
preserved by the reviewed source change. Invalid `from_static` inputs now fail
const evaluation or assert at runtime; `try_new` and deserialization remain
fallible. This addresses existing f2c ledger entry 2; no malformed production
caller or new defect count is claimed. The [repair patch](repair.patch) and
[connected ledger](LEDGER-v8.json) retain the exact scope and outstanding
terrain, demographic, ordering and other questions.

On the rebased combined tree, the existing quest-generation suite passed 58
tests, the weather scene-file integration test passed, and `just fmt-check`
and full `just lint` passed. A first formatting command failed in the snap
wrapper before the recipe, and a first lint command failed at a read-only cache
path before quality checks; the corrected invocations and failures are retained.
The repository prohibits object-focused unit tests without an explicit request.
The selected source paths did not establish a higher-level dynamic caller of
`from_static`, so direct negative-constructor regression coverage remains a
validation limit. The existing malformed-JSON decoder already rejected such
values before this repair.

Independent reviewers rechecked all thirteen lenses within the same bounded
scope against the source patch. Uninspected callers, native or generated
interfaces, policy decisions and repository-wide lens coverage remain open.
The earlier f2c assessment and original registry roles retain their own reader
and revision qualifications; no earlier record is promoted to missing
checkpoint 3821. Its ledger and queue, SOURCE004, nontext/no-positive-EOF and
other recovery obligations remain open.

## Recoverable evidence

`custody/` contains an AES-256-GCM encrypted archive of post-discovery repair,
rebase, validation, reviewer and publication-control records. The earlier
published discovery and weather archives, original evidence, Git objects and
separately held keys remain dependencies; they are not repeatedly packaged.
The discovery publication's initial local-only push and successful direct
GitHub push are separately recorded. The fresh encryption key is held on the
same machine, not in independent custody.

Authentication and restoration succeeded. An independent comparator matched
80 files, 15 directories, original file bytes and modes, root membership and
registration; all 94 pre-capture inventory entries matched the restored copy.
Owners, ACLs, timestamps, hardlinks, crash durability and full Git-object closure
are not established. From `custody/`, `python3 -I -B VERIFY.py` checks only the
ciphertext identity. With the separately held key and a fresh disjoint
location, `VERIFY.py --key KEY --restore DESTINATION` authenticates and restores
the indexed archive. This requires POSIX, Python, `cryptography`, and trusted,
nonconcurrent parent directories. The keyed restore receipt and private-index
digest remain restricted. No-key verification does not prove authenticity or
origin.

The envelope's own build, restore, publication, key-locator controls and later
PR/CI events are outside this cutoff and must be retained at a later checkpoint.
No independent backup or repository-wide audit closure is claimed.

- Ciphertext SHA-256: `6a70fe9596ff691929040c6063fa7f08806706815de73433aacd9b1b16445375`
- Public verifier SHA-256: `7e0b1c9908b44c2286ac300e937883811863e5650fcbe246c13c1920ae4960ca`
- Ledger SHA-256: `eebfc4782685c7860a1429a6143f3def7ba0a1f9a4ba7dec74eb618022522cbc`
- Repair patch SHA-256: `43600655ff0c6999a9a77e45b11974cfa8f3e7c30ba93e81649c443239bd2fb2`

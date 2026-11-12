# Read-only #726/#774 evidence and worktree inventory

Observed 2026-11-11 around 23:25–23:31 UTC. No existing file, directory,
worktree or Git reference was deleted, moved, overwritten, cleaned or modified.
Temporary report and verification outputs were created only under this `/tmp`
directory. Sizes from `du` are allocated bytes; directory totals overlap their
children. Available space was approximately 731 MiB at one observation.

## Critical recovery: live physical P16 worktree

`<LOCAL_PATH_0>
uses 543,547,392 allocated bytes (about 519 MiB) and must be protected. Its
HEAD and original dirty P16 base are both
`a120390947c89b703dc74c4255a13c035d054c50`. Its 159 current porcelain entries
exactly equal the recorded P16 status list: 151 modified, one deleted and seven
untracked. All 158 existing files match their own recorded P16 source SHA-256;
the deleted source is absent locally and present as a HEAD blob.

A read-only `git diff --binary HEAD` stream has 524,542 bytes and SHA-256
`fc9002ab1135a438f68eede9217f67d22ef737d2c6938509e6e6660f2fc5b7f7`,
exactly the previously recorded patch hash. The inspected historical registry
does not specify the original command: this is a matching reproduced byte
stream, not proof of the historical invocation. It was hashed in memory and
not saved as a patch file. The stream covers tracked changes and deletion; the
seven untracked files are separately hash-matched and must also be preserved.
The complete status and per-file comparisons are in `p16-live-comparison.json`.
Prior audit packets that said the patch bytes were unrecovered remain accurate
for their original observation time; this later source observation should be
published separately without rewriting their authorship or revision.

The top-level `ae46` worktree has four modified Cargo manifests. Its target is
about 907 GiB, including `target/semantic-goal` about 682 GiB and
`target/debug` about 216 GiB. The first contains the dirty P16 worktree and
other registered worktrees. These sizes are not an invitation to clean them.
`git worktree list` returned 112 registered worktrees; this inventory did not
verify every one. Ordinary `git status` excludes ignored `target` content.

## Original 059 transport and user-supplied extraction

`<LOCAL_PATH_1>
uses 12,228,632,576 allocated bytes (about 11.4 GiB). Against the original
transport manifest, all 36,487 files match both SHA-256 and Git blob hash and
size; there are no missing, changed or additional files. The corresponding
payload Git tree has the same 36,487 entries. This is the recovered historical
059 collection, not a restoration of missing checkpoint 3821.

GitHub's live API returned the handoff branch at
[`eb407672813424d64ec31f208193969f5a62620c`](https://github.com/adventure-simulator-group/fabelgeist/commit/eb407672813424d64ec31f208193969f5a62620c).
All 40 remote payload entries' Git object IDs and sizes match the local handoff
Git tree. All 40 local part SHA-256 values match the manifest; their ordered
concatenation matches both local full bundles, with SHA-256
`d7e093537228a92168b25f296545080e3b209d63d844b1d8171231c25c194430`.
GitHub metadata was checked; the 40 remote binaries were not freshly downloaded,
and no independent-machine restoration was performed in this inventory.
`transport-comparison.json` and `github-remote-observation.json` contain the
exact checks. The extraction is still an active source path for audit and
verification, and the user explicitly directed its reuse; preserve it.

## Older 774 evidence and missing remote asset

`<LOCAL_PATH_2>
uses 1,089,220,608 allocated bytes. Its 20,917 manifest files all match the
local original manifest. At the published
[`f82adcabd9d89c4fa2dd7f99253a45297a0681ee`](https://github.com/adventure-simulator-group/fabelgeist/commit/f82adcabd9d89c4fa2dd7f99253a45297a0681ee)
Git tree, 20,915 original files match by Git blob and size; the historical
`README.md` differs and the original `adler-original-20261023.zip` container
is absent at its original path. The published README says that ZIP's members
were expanded for GitHub size limits. This does not preserve its exact container
bytes or prove other originals are absent elsewhere.

The nested original ZIP is 119,809,110 bytes, SHA-256
`02bb3c8b7b0d5fde4fb1ef2f6942ce70d0e642ad498d42388ef947bf1fd9b04d`.
The outer local archive
`<LOCAL_PATH_3>
uses 625,532,928 allocated bytes and hashes to
`7a91ee2341142d06acf49bdc9e868dde73c875eddd7445843c745ba3ab34d56f`;
streaming its nested ZIP member produced the same nested hash. An older local
record names an encrypted GitHub release asset, but a fresh GitHub API request
for release tag `audit-774-handoff-20261025` returned 404. That means the asset
is not verified accessible through this connection; it does not distinguish
deletion from permissions. Preserve both local original archive copies and the
original README. `774-package-comparison.json` and
`github-older-observation.json` retain evidence.

`<LOCAL_PATH_4>
uses 301,387,776 allocated bytes (about 288 MiB) and is actively referenced for
original registry, source objects, supplier receipts and revision
qualifications. The 2,526,003,200-byte-allocated `774-publication-branch` and
the encrypted hosting directory were not exhaustively assessed for independent
contents or live path dependencies. Preserve them pending that assessment.

## Current audit and recovery checkout

`<LOCAL_PATH_5>
uses 8,204,103,680 allocated bytes (about 7.7 GiB). It contains the working
publication, recovered historical evidence, original transport and a separate
recovery checkout. GitHub API returned the active audit branch at
[`094094c53d4fba22c3f1123f19183bf83db7ee74`](https://github.com/adventure-simulator-group/fabelgeist/commit/094094c53d4fba22c3f1123f19183bf83db7ee74),
the same HEAD observed in the clean publication and recovery checkouts.
The coordinator reports the 094 recovery verification passed. An accessible
`RECOVERY-VERIFICATION-5AE8DFCC.json` separately records an earlier successful
same-machine check; it is not an independent-machine attestation.

The recovery checkout uses 2,636,525,568 allocated bytes. It is shallow: its
local branch still points to `e5aa898f...`, and its ahead/behind display is
misleading. The non-shallow publication repository verifies `e5aa898f...` is
an ancestor of `094094c5...`; it is not a divergent history. The checkout and
its Git metadata remain part of the verification workflow until that dependency
is retired. Do not infer whole-audit completion or product validation from
these evidence checks.

The `original-transport/payload.git` pack is 1,328,605,891 bytes; a read-only
byte comparison confirms it is exactly the payload bundle after the 83-byte
bundle header. The Git repository and its index are actively used for original
object verification; byte duplication alone is not a cleanup authorization.

## Exact duplicate candidates for review

These are byte-preservation candidates, not unconditional path or workflow
safety approvals. No item has been removed.

1. The single file
   `<LOCAL_PATH_6>
   uses 1,328,611,328 allocated bytes. It is byte-identical by SHA-256 to the
   separately allocated
   `<LOCAL_PATH_7>
   and to the reassembled 40-part remotely indexed handoff. The two local files
   have different inodes and one link each. A momentary `/proc` scan found no
   open descriptor for either. The old `finalize_payload.py` expects the prep
   pathname; a future rerun would require restoring the identical file there.
   Resolve that path dependency before selecting it for removal.
2. `<LOCAL_PATH_8>
   uses 557,531,136 allocated bytes (about 532 MiB). Every one of its 18,145
   files under `handoff/` matches the Git blob at
   `3b9728c1a9b266bba4b5481add6f96c2119c2ed2`, which GitHub confirms and which
   is the direct parent of the published handoff commit; no extra handoff files
   were found. `return-transport-comparison.json` contains the comparison. Its
   root also has a `target` symlink not represented in Git, and historical
   records still cite the local path. There were no open descriptors under the
   directory in a momentary scan. Resolve those references and symlink handling
   before considering removal.

The dirty P16 worktree, the user-supplied extraction, exact 774 ZIP and README,
current publication and recovery paths are not presented as deletion candidates.
No implementation, product build, test, macro, runtime probe or product
validation was performed.

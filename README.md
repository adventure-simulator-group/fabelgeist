# #726 audit recovery handoff

This is a **partial recovery**, not a lossless handoff of the latest audit
state. The continuation's local ledger, queues, receipts, source registrations
and checkpoint 3821 evidence were deleted before they were published. Their
hashes and summaries cannot reconstruct their bytes. Do not represent this
package as the authoritative checkpoint 3821 or resume its writer from it.

The recoverable material consists of:

- Segovb's original, lossless checkpoint 059 package at commit
  `eb407672813424d64ec31f208193969f5a62620c`. Its transport manifest covers
  36,487 evidence files and its own restoration verifier checks their hashes.
- The later public progress branch at commit
  `78b1d089cb5ee52df0b01c0bf6a020f049b8a126`, containing 3,249 numbered
  checkpoint summaries and reports through 3737, plus 348 incremental
  checkpoint summaries and reports through the reported audit checkpoint 3821.
  These are summaries, not the continuation's full evidence graph.
- Published implementation draft branches listed in `SOURCE-LOCATORS.json`.
  Their existence does not imply review, integration or current-head tests.

`CURRENT.json` records the latest *reported* numbers and known missing state.
`AUDIT-RESUME.md` and `PROMPT.md` explain how to proceed without overstating
recovery. The latest public [progress report](PROGRESS.md) and
[incremental checkpoint](CHECKPOINT-INCREMENT-348.json) are copied here for
convenience; their originals remain at the pinned progress commit. The original
handoff remains authoritative for its own checkpoint. See
[VALIDATION.md](VALIDATION.md) for the checks actually performed.

Run `python restore.py DESTINATION` to clone the two pinned public sources,
verify the original manifests and restore its evidence, and verify every file
in the progress manifest. The destination must not already exist. The base
restoration requires at least 12 GB for extracted evidence plus space for the
Git clones and original transport package. Use `--verify-only` to verify the
transport without writing the extracted base evidence. Internet access, Git
and Python 3.9 or newer are required. Full extraction requires a
case-sensitive filesystem: the original manifest has two pairs of distinct
paths differing only in letter case. On a case-insensitive filesystem the
script refuses extraction before downloading; `--verify-only` remains
available. A failed or interrupted run leaves `RECOVERY-INCOMPLETE` and must
not be treated as a completed recovery. This does not recreate the deleted
post-handoff ledger or authorize code repair.

The handoff contains no credentials. It does not merge a branch, contact a
person, or change a database.

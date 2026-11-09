# #726 audit recovery handoff

## Recovery counts

- 36,487 original evidence files restored and hash-verified.
- 7,196 later public progress files verified against the recovery manifest.
- 6 historical JSON versions recovered from the transcript and matched to
  later recorded path and SHA-256 references.
- 2,968 historical printed contract objects indexed, covering 1,564
  distinct contract IDs; most object contents remain private and unverified.
- 19 late audit notes recovered as transcript text, with original bytes
  and supporting evidence unverified.
- 2,480 candidate file states and 519 literal-write records extracted for
  further investigation; 2,386 candidate states are Python scripts. These
  counts overlap and are not verified totals.

This is a **partial recovery**, not a lossless handoff of the latest audit
state. The continuation's local ledger, queues, receipts, source registrations
and checkpoint 3821 evidence were deleted before they were published. Their
hashes and summaries cannot reconstruct their bytes. Do not represent this
package as the authoritative checkpoint 3821 or resume its writer from it.

The originating Codex chat's history tool was subsequently confirmed to
expose older turns, including some commands and tool outputs from before
compaction. A bounded private snapshot contains recorded commands, file changes and
outputs; its completeness remains unverified and the transcript itself is
not included in this package. Some outputs or argument previews may be truncated. Review any
recovered material for sensitive data and verify its provenance and hashes
before publishing or adopting it.

The recoverable material consists of:

- Five [historical queue and selected-lens values](recovered/transcript-queue-fragments/README.md)
  and an index of further printed contract candidates.
- Nineteen [late audit notes](recovered/transcript-notes/README.md)
  transcribed from recorded commands, with explicit evidence limitations.
- Six [historical transcript-derived records](recovered/transcript-hash-matched/README.md)
  with matching later path and hash references. Their dependencies and
  authority as current state remain unverified.

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

Future checkpoints must publish the complete recoverable audit state to a
remote branch, not just summaries. Rebase the active working branch onto
current `main` early and reconcile source changes so the audit stays current.

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

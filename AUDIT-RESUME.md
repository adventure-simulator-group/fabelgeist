# Recovery and continuation limits

1. Restore this package into a new directory. Read the recovered original
   `handoff/726-audit-20261030-adler/README.md`, `CURRENT.json`,
   `AUDIT-RESUME.md`, `SOURCE-LOCATORS.json` and `PROMPT.md` first. Its own
   verification identifies exact base evidence; restoration grants no audit
   coverage or current-source reconciliation. This wrapper places the base at
   `original/target/726-recovered-base`; historical helpers contain fixed
   paths and need separate review before use at an alternate location.
2. Read `progress/PROGRESS.md` and `progress/CHECKPOINT-INCREMENT-348.json`
   at the pinned progress commit. Earlier `CHECKPOINT-*.json`,
   `PROGRESS-*.md`, and incremental snapshots contain public summaries.
   Recorded hashes identify missing local originals; they do not contain
   those originals.
3. The reported checkpoint 3821 had 3,866 queue entries, 599 explicitly
   source-discovery closed and 3,267 not explicitly closed. The entries
   overlap; they are not unique findings or a percentage complete. The gate
   was false and 0 of 20 historical repair families were validated on the
   combined head. Treat these as reported values, not a recovered queue.
4. The user reports that no independent backup exists.
   Verify any recovered `CURRENT.json`, ledger, queue and checkpoint 3821
   against the hashes in the incremental checkpoint before adopting them.
   Otherwise a human and the continuing agent must decide how to re-establish
   an authoritative ledger from the original package and public summaries;
   do not silently promote summaries into evidence or repeat audit credit.
   The originating Codex chat can be paged through the chat-history tool:
   older checkpoint commands, verification outputs and bounded source reads
   have been observed. The recovery package contains 212 historical JSON
   versions matching recorded path and hash references, as well as partial
   ledger, queue, registration and source-output collections. See their
   individual manifests and `RECOVERY-PROGRESS.md`; their dependencies and
   current-state authority remain unverified. Transcript completeness and
   accessibility from another machine are unverified.
   Do not blindly replay old commands or publish the transcript: review
   sensitive content, preserve attribution, and validate reconstructed bytes
   and evidence against surviving hashes and the original protocol.
5. Preserve the existing drafts as unverified drafts. No product code repair
   is authorized before the handoff's audit gate is satisfied. Reconcile main
   and the selected source pin before any future source assessment.
6. The user accepted this partial recovery as the handoff and authorized
   publication. Keep the complete recoverable audit state on a remote branch.
   At every
   checkpoint, publish the ledger, queues, selected states and supporting
   evidence alongside the quantitative progress report; verify the pushed
   commit and its restoration procedure. Review material for secrets and
   private data first. A report or a list of hashes is not a backup.
7. Rebase the active working branch onto current `main` early, then regularly
   as needed. Preserve existing work and checkpoint references in an isolated
   worktree first. Record the new source pin and reconcile the changed files
   and their lens obligations after each rebase. Historical evidence stays
   tied to its original revision; rebasing grants no audit or test credit.

The final Lens 13 source packet is recovered, but its original plan and
predecessor state are not. The candidate plan and historical self-reports
in `recovered/l13-selected-state-transition-trace/` preserve a conditional
cross-check of four predecessor references. Its prior state hash is unknown;
neither the candidate nor the reported post-apply hashes establish adoption
or the final audit authority. `RECOVERY-CHECKPOINT.json` records the scopes
and limits of the searches. Do not treat a failed candidate search as proof
of loss, or use the candidate to resume writes.

The original frozen revision was `6533e444fa37552927070cc8d8190907fd3b413e`.
The latest reported selected source was
`ff9c1d7d0ea168b7e14eedc5e44879ee7b84fb04`. Historical validation belongs to
its exact recorded revision and environment; no full integration or
post-recovery acceptance is claimed.

The original transport's Windows verification passed for all 36,487 logical
files. An initial extraction on a case-insensitive directory stopped at a
case-only collision and left an incomplete directory. A separate extraction
into an isolated case-sensitive directory restored all 36,487 entries and
passed their hash checks. Use a case-sensitive directory for full restoration.

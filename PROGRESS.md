# Bespoke type refactor: progress

Updated: 2026-11-08T07:01:06.548191+00:00

| Increment status | Count |
| --- | ---: |
| Queued | 0 |
| Auditing | 0 |
| Implementing | 2 |
| Awaiting or running checks | 1 |
| Awaiting review | 2 |
| Integrated and validated | 0 |
| Blocked | 0 |

| Historical audit measure | Count |
| --- | --- |
| Original inventory roles, lenses 11–13 | 8,559 / 8,559 accounted |
| Registered revision-delta lens/path checks | 3,172 / 3,172 closed |
| Ledger entries with source discovery explicitly closed | 588 / 3,866 |
| Other ledger entries | 3,278, including overlapping obligations |

| Named repair families in the historical handoff | Count |
| --- | ---: |
| Source marked complete or complete-preserved | 12 / 20 |
| Source reopened for repair | 6 / 20 |
| Source marked implementing | 2 / 20 |
| Validated against the current combined draft | 0 / 20 |

These family labels come from the historical ledger and are not current-head
acceptance. P16 and P17 were the two marked implementing; the P17 restoration
is now a draft and still needs its dependencies, review, and validation.

Historical coverage is not completed repair work. The revised user direction
allows independently audited implementation increments while retaining the
ledger as backlog. Full refactor completion and validation remain required.

## Active increments

- [Audio playback speed](https://github.com/adventure-simulator-group/fabelgeist/tree/codex/726-audio-pitch)
  commit `2d318efbb21d044680070c525554cb04ee286437`: focused `cargo check` and two existing audio
  configuration tests passed. Review and integration remain open.
- [Landform preparation outcome](https://github.com/adventure-simulator-group/fabelgeist/tree/codex/726-landform-outcome)
  commit `c40d70aeceeedfd87d7c9528628bf9c9bf87512b`: a draft based on the
  audio commit. It retains typed landform failures and excludes failed scenes
  from readiness. Recipe and patch arrivals now trigger reconciliation; changed
  recipes rebuild terrain. Focused Clippy passed at this commit; the focused
  client check and Rust quality checker passed at its parent.
  At the parent commit, 19 existing terrain tests passed and one benchmark
  was ignored; they do not exercise the new lifecycle or failure path. The
  replication library documents per-entity atomic updates
  within a tick, but cross-tick arrival and pre-arrival readiness remain open.
  Review and integration remain open.

- [Wedding capacity atomicity](https://github.com/adventure-simulator-group/fabelgeist/tree/codex/726-wedding-atomicity)
  commit `4510a2685ea541dd3963bd22aa7a6df6ea29a2a5`: the draft performs
  joint effective-dated capacity preflight and propagates unexpected settlement
  errors. Focused Clippy and the Rust quality check passed before the landform
  lint correction was added. The disposable-database workflow passed at
  `f33d5ebe37407718a2283dc9313924c32ab58973` for joint capacity, scoped
  removal, rejection, escrow refund, household preservation, rollback, and a
  successful wedding with both spouses admitted and the escrow paid. The
  landform lint correction is now included.
  `just fmt-check` and full `just lint` passed in a non-nested verification
  worktree at this commit. Review and final integrated validation remain open.

- [P17 report restoration](https://github.com/adventure-simulator-group/fabelgeist/tree/codex/726-p17-restoration)
  commit `dcd6ea1aaddb0e542a418ae7ebe35c0bf73d4ecc`: the preserved report
  implementation is restored with sample-tick and `LeadFoot` fixes. The focused
  animation-viewer compile passed; the Rust quality check passed before the
  relevant baseline reductions. Existing report tests passed: 26 passed, 0 failed at this commit. Source
  review found local-position jitter derivative timing and incident labels use
  unshifted frames; a separate repair is drafted below. Dependencies,
  review, and final integrated gates also remain open.

- [Jitter diagnostics draft](https://github.com/adventure-simulator-group/fabelgeist/tree/codex/726-jitter-timing)
  commit `96a20906e7e8e78c61dde90260fa16ecc5e47020`: corrects source-derived derivative timing and
  records missing-parent, inconsistent-bone and non-finite evidence as
  incomplete. The existing jitter tests are compiling; behavior of the new
  failure cases has not been verified. The Rust quality checker found an
  oversized analysis function and growth beyond the capture module ceiling;
  both require refactoring before the quality gate can pass.

The build queue remains serialized. Earlier [checkpoint 3737](CHECKPOINT-3737.json)
preserves the historical audit; its former global implementation gate has been
superseded, while its evidence and unresolved questions remain in the backlog.

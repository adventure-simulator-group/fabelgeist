# Bespoke type refactor: progress

Updated: 2026-11-08T06:12:47.038778+00:00

| Increment status | Count |
| --- | ---: |
| Queued | 0 |
| Auditing | 0 |
| Implementing | 1 |
| Awaiting or running checks | 0 |
| Awaiting review | 2 |
| Integrated and validated | 0 |
| Blocked | 0 |

| Historical audit measure | Count |
| --- | --- |
| Original inventory roles, lenses 11–13 | 8,559 / 8,559 accounted |
| Registered revision-delta lens/path checks | 3,172 / 3,172 closed |
| Ledger entries with source discovery explicitly closed | 588 / 3,866 |
| Other ledger entries | 3,278, including overlapping obligations |

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
  commit `f33d5ebe37407718a2283dc9313924c32ab58973`: the draft performs
  joint effective-dated capacity preflight and propagates unexpected settlement
  errors. Focused Clippy passed with the authority feature. The guarded
  disposable-database workflow passed at the parent for joint capacity, scoped
  removal, rejection, escrow refund, household preservation, and rollback.
  The current commit also passed a successful wedding with both spouses
  admitted and the escrow paid. Review remains open. The
  branch precedes the latest landform lint-only commit and needs reconciliation.

The build queue remains serialized. Earlier [checkpoint 3737](CHECKPOINT-3737.json)
preserves the historical audit; its former global implementation gate has been
superseded, while its evidence and unresolved questions remain in the backlog.

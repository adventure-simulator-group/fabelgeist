# Bespoke type refactor: progress

Updated: 2026-11-08T05:42:02.598772+00:00

| Increment status | Count |
| --- | ---: |
| Queued | 0 |
| Auditing | 1 |
| Implementing | 0 |
| Awaiting or running checks | 1 |
| Awaiting review | 1 |
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
  commit `689710cc3d7a0631bde03a33dcaa4927734aab74`: a draft based on the
  audio commit. It retains typed landform failures and excludes failed scenes
  from readiness. Recipe and patch arrivals now trigger reconciliation; changed
  recipes rebuild terrain. The focused client check passed. The Rust quality
  checker passed after extracting named presentation queries.
  Existing terrain tests are being rerun; they do not exercise the new lifecycle
  or failure path. The replication library documents per-entity atomic updates
  within a tick, but cross-tick arrival and pre-arrival readiness remain open.
  Review and integration remain open.

Wedding capacity atomicity is in bounded audit. Its preflight must account for
both participants jointly across effective-dated occupancy intervals, including
existing residents and later moves; the implementation has not started.
The build queue remains serialized. Earlier [checkpoint 3737](CHECKPOINT-3737.json)
preserves the historical audit; its former global implementation gate has been
superseded, while its evidence and unresolved questions remain in the backlog.

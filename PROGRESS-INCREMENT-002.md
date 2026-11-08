# Bespoke type refactor: progress

Updated: 2026-11-08T05:06:03.193431+00:00

| Increment status | Count |
| --- | ---: |
| Queued candidates | 2 |
| Auditing | 0 |
| Implementing or awaiting build | 1 |
| Awaiting review | 0 |
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

## Audio playback speed

Status: **focused tests running**. The isolated [implementation branch](https://github.com/adventure-simulator-group/fabelgeist/tree/codex/726-audio-pitch)
contains commit `2d318efbb21d044680070c525554cb04ee286437`. It carries checked playback-speed and ordered
pitch-range values through movement and combat audio, and rejects foliage
combinations whose endpoint products are zero or nonfinite. The focused client
`cargo check` passed; existing audio configuration tests are running against the
shared target under an exclusive process-held lock. Formatting was applied.
Review, repository-wide gates and integration remain outstanding.

## Queued

Landform preparation error outcome and wedding capacity atomicity are separate
candidates. Each needs its own bounded audit, acceptance and implementation.

## Historical evidence

[Checkpoint 3737](CHECKPOINT-3737.json) preserves the earlier audit state. Its
former global implementation gate has been superseded by user direction, while
its evidence and unresolved questions remain part of the backlog.

# Bespoke type refactor: progress

Updated: 2026-11-08T05:32:36.879960+00:00

| Increment status | Count |
| --- | ---: |
| Queued | 0 |
| Auditing | 1 |
| Implementing | 1 |
| Awaiting or running checks | 0 |
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
  commit `1122faa84c4ee58a7fae59a553fa0c82cfde3b03`: a draft based on the audio commit. It retains a typed
  initial landform failure with scene and recipe identity and distinguishes
  natural ground from a patch. Refresh and late-arrival behavior remain open.
  The loading gate now excludes failed terrain from its readiness count. The
  focused client check passed at this commit. At its parent, 19 existing
  terrain tests passed and one benchmark was ignored; they did not prove the
  failure path or late recipe behavior. Review and integration remain open.

Wedding capacity atomicity is in bounded audit. Its preflight must account for
both participants jointly across effective-dated occupancy intervals, including
existing residents and later moves; the implementation has not started.
The build queue remains serialized. Earlier [checkpoint 3737](CHECKPOINT-3737.json)
preserves the historical audit; its former global implementation gate has been
superseded, while its evidence and unresolved questions remain in the backlog.

# Bespoke type refactor: progress

Updated: 2026-11-08T10:57:27.738397+00:00

| Increment status | Count |
| --- | ---: |
| Queued | 0 |
| Auditing | 0 |
| Implementing | 2 |
| Awaiting or running checks | 2 |
| Awaiting review | 2 |
| Integrated and validated | 0 |
| Blocked | 0 |

| Current P16 quality scan | Count |
| --- | ---: |
| File and function size findings remaining | 0 |
| Previously reported ingress size findings cleared | 2 |
| Previously reported function-size findings cleared elsewhere | 28 |
| Existing contact-geometry tests passed | 11 / 11 |
| Existing melee iteration tests passed | 9 / 9 |

The quality counts are findings from the latest scan, not a count of defects
or independent work items. The quality scan and WebAssembly client Clippy
passed on a tree with two unstaged formatter-only files. Full workspace Clippy
found test compilation and two redundant conversions; corrections are pushed
and the workspace retry found one more fixture type qualification. That fix
is pushed and focused core/database Clippy passed. Workspace Clippy then
found tactical fixture type errors and a lint issue; those corrections are
pushed and focused tactical Clippy passed. The next workspace run found an
unused client re-export; native client Clippy passed after its removal.
Workspace Clippy passed on the current commit. Full lint was stopped during
standalone manifest checks when free disk fell below 3 GB; the disposable
P16-only native build cache was removed, restoring about 8.6 GB free.
Automatic approval review rejected a separate standalone target as conflicting
with the shared-target requirement. All three standalone Clippy commands passed
under the shared build lock. Selected local fingerprint records for creator
and MHR postdate that validation sequence; NumPy-storage was rechecked after
refreshing its P16 source timestamps, and all eight selected local records
postdate that recheck.

| Historical audit measure | Count |
| --- | --- |
| Original inventory roles, lenses 11Ã¢â‚¬â€œ13 | 8,559 / 8,559 accounted |
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
P16 covers melee geometry and targeting-range policy; the historical ledger
identifies 17 source gaps and shared dependencies that still need current-head
reconciliation. That historical count is not a current defect count. The
unfinished P16 recovery snapshot changes 159 paths; 23 of those also differ
between its original parent and the current draft. This is a path overlap,
not a count of conflicts. A three-way preview found 3 textual conflicts:
the core manifest, melee mobility, and server causal reporting. Textually
merged paths still require semantic review; validation remains open.

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
  relevant baseline reductions. Existing report tests passed: 26 passed,
  0 failed at this commit. Source review found local-position jitter derivative timing and incident labels use
  unshifted frames; a separate repair is drafted below. Dependencies,
  review, and final integrated gates also remain open.

- [P16 recovery draft](https://github.com/adventure-simulator-group/fabelgeist/tree/codex/726-p16-restoration)
  commit `aaf0a66c7` (quality and fixture repairs after `623d52a`): the unfinished
  159-path snapshot is restored on the current draft with its three textual conflicts reconciled.
  The movement report retains typed measurements and rejects absent required
  displacement and distance observations instead of assuming zero. This is
  a partially compiled recovery draft. Manifest dependency errors found by the
  formatter were corrected; `just fmt` and `just fmt-check` passed at the
  earlier draft. The focused core compile passed at parent `20ed2a263`; the
  tactical-core check passed on this commit after fixing a duplicate import.
  The tactical-client binary check passed at `09ae853ff`. Default server
  binaries compiled after the impact-position fix. The iteration-feature
  check then found unsupported observer return types and a trace conversion.
  The current commit retains the first typed observer failure for the duel
  runner; feature-gated server binaries compiled at parent `89962dc0d`.
  The database-module check initially reported 81 errors; after typed-error
  propagation through four paths, it reported 72. Typed quest materialization
  reduced the next snapshot to 71 before formatting; these are compiler
  snapshots, not remaining defect counts. The latest draft stores a typed
  cause and settlement-stage context separately; its callers are still being
  migrated. Propagation through additional strategic reducers brought the
  formatted draft to a passing focused database-module `cargo check`
  without warnings at parent `bb63239b8`. The authority-test feature now
  compiles at this commit after adapting existing fixtures; neither check is
  behavioral validation. Typed failures now cross tactical, settlement, wedding,
  custody, party, encounter, trade, disease, and exposure callers. Wedding
  rollback behavior must be revalidated; the guarded isolated authority
  workflow stopped during the WebAssembly release build before any assertions.
  Cargo reused a geometry artifact from the wedding worktree; its dependency
  record explicitly names that worktree; four other local crates have matching
  records. Shared-target results need revalidation on their proper worktrees.
  A P16-specific WebAssembly release build and the guarded local authority
  workflow passed at parent `623d52a`, including the wedding capacity and rollback
  fixtures. An earlier retry found its chosen port occupied. Other runtime
  behavior, semantic review of automatic merges, full formatting and lint,
  and integrated validation remain open. `just fmt-check` passed at parent
  `71ffc473d`. Lint confirmed generated bindings, then found an exact-text
  calendar guard expecting the former string error; the guard now requires
  the typed strategic error. On retry, lint passed that guard and stopped at
  Rust quality: file and function growth, one moved raw-string finding, and
  stale baseline entries require repair. Five stale entries were removed, and
  the existing raw-string finding was moved to its new owner without raising
  a ceiling. A small authored-weapon cleanup removed one oversized function
  finding; a fresh quality scan confirms those specific findings cleared. Other
  size findings remain, and Clippy has not yet run. The isolated core check
  passed at `b41391400`. Ranged observers were split from combat ingress and
  repeated defense rejection handling was consolidated; the iteration-feature
  server binaries compile and both ingress size findings cleared in a rescan.
  A ranged sight simplification cleared another function-size finding; the
  iteration-feature server binaries compiled at `f844072ef`. Tactical
  authority configuration validation is now separate; tactical core compiles
  at `1273ffded`, and its file and function ceilings were reduced. Equipment
  input handling now has its own module; the isolated client binary check
  passed at `6f71aad45`, and the equipment file ceiling was reduced. A
  movement jump-height policy extraction compiles in tactical core at the
  `51b877a0c` and clears another function-size finding. Personal travel-food
  handling was separated from the party path; the authority-feature database
  module compiled at `cf29e5d97`, and its function ceiling was reduced. Item
  equipment projection now has a separate owner; the authority-feature
  database module compiled at `0ed3a3d62`, and related ceilings fell. Wedding
  preflight helpers were separated; the authority-feature database module
  compiled at `fcec86658` and the marriage file-size finding cleared. The
  wedding acceptance setup is now separate and compiles at the current commit;
  the guarded authority workflow passed again on a disposable local profile,
  including wedding capacity and rollback fixtures. Repair escrow restoration
  was then separated and the authority-feature module compiles at the current
  commit; both repair size findings cleared. Repair behavior has not been
  rerun since the extraction. Dialogue topic handling was separated; the
  authority-feature database module compiles at the current commit and both
  dialogue size findings cleared. Outbreak source projection was separated;
  the authority-feature database module compiles at the current commit and
  both outbreak size findings cleared. Dialogue and outbreak behavior have
  not been rerun since these extractions. Development fixture loadouts were
  separated, clearing three findings; the authority-feature module compiles
  at `72ac60c2b`, but those fixture behaviors have not been rerun. The
  investigation retry comparison was separated; the authority-feature module
  compiles at the current commit and its function-size finding cleared.
  Retry behavior has not been rerun since the extraction. Tactical request
  conflict detection was separated, clearing two size findings; the
  authority-feature database module compiles at the current commit. Tactical
  request behavior has not been rerun since the extraction. Incident
  coordinate admission was separated, clearing two findings; the
  authority-feature module compiles at the current commit. Incident behavior
  has not been rerun since this extraction. Morale duration and decay helpers
  were separated, clearing one file-size finding; the authority-feature module
  compiled at `4e4a8704f`; morale behavior has not been rerun. Social clock
  lookup was separated, clearing a file-size finding. The authority-feature
  module compiled at `9e4853a26`; social behavior has not been rerun. Dialogue
  startup now reuses the admitted living character instead of looking it up
  again. The authority-feature module compiles at the current commit; dialogue
  startup behavior has not been rerun. Puzzle and errantry leader admission
  now shares a helper; three size findings cleared and the authority-feature
  module compiled at `5e85d4f7c`. Travel paths now reuse their admitted living
  character; three size findings cleared and the authority-feature module
  compiles at the current commit. Challenge and travel behaviors have not
  been rerun since these changes. Mission replay and visibility admission were
  separated, clearing two findings; the authority-feature module compiles at
  the current commit. An adapted source guard remains unrun: Cargo reported
  that this package has no `--tests` target. Mission host and recruitment
  setup were separated, clearing three findings; the authority-feature module
  compiled at `5f6e076a3`; their behavior has not been rerun. Trade and
  party-disband checks were simplified, clearing four findings. The
  authority-feature module compiled at `37a79c985`; their behavior has not
  been rerun. Bot windup cancellation was separated, clearing its size
  finding; iteration-feature server binaries compile at the current commit.
  Bot behavior has not been rerun. Player input tracing was separated,
  clearing the final size finding; iteration-feature server binaries compile
  and the complete Rust quality scan passes at the current commit. Full
  formatting check and WebAssembly client Clippy passed with two unstaged
  formatter-only files, now included in the current commit. Full workspace
  Clippy found existing fixtures needing typed API adaptations and two
  redundant conversions. The retry exposed a fixture type qualification;
  that correction passed focused core/database Clippy. A workspace retry
  found tactical fixture type errors and a lint issue; corrections are pushed
  and focused tactical Clippy passed. Workspace retry then found an unused
  client re-export; native client Clippy passed after the fix. Workspace
  Clippy passed at the current commit. The complete formatting check passed;
  full lint was interrupted during standalone manifest checks to protect disk
  space. The P16-only native build cache was removed to restore space.
  Automatic approval review rejected a separate standalone target because of
  the shared-target requirement. The first standalone check is running with
  the shared target and lock; all three standalone Clippy commands passed.
  Selected creator and MHR local fingerprint records postdate the validation
  sequence. NumPy-storage was rechecked after refreshing P16 source timestamps;
  all eight selected local records postdate that recheck. Behavior, a single
  uninterrupted full lint run, and review remain open.
  Input behavior has not been rerun. Eleven existing core contact-geometry
  tests passed at the current commit after refreshing older selected shared
  dependencies; all 22 selected local fingerprint records postdate the
  validation sequence. Nine existing melee iteration tests also passed using
  the same fingerprint-checked test binary. The shared WebAssembly build
  reproduced five stale wedding-worktree records. Those exact fingerprint
  markers were invalidated under the build lock; after rebuilding dependencies,
  the module build passed and all nine selected local records postdate the
  rebuild with none naming the wedding worktree. A guarded authority rerun is
  passed on a disposable local profile at the current commit, including
  wedding, investigation retry, puzzle, mission, and outbreak checks. Repair,
  dialogue, and development-fixture behavior remain to be verified.

- [Jitter diagnostics draft](https://github.com/adventure-simulator-group/fabelgeist/tree/codex/726-jitter-quality)
  commit `8ff11c82fb4430ee17a170c61bea72c3d5173625`: source-derived timing
  correction and incomplete-evidence detection. Seven existing jitter tests passed on parent
  `96a20906e7e8e78c61dde90260fa16ecc5e47020`; they do not cover the new
  missing-evidence behavior. The size refactor compiled and passed the Rust
  quality checker before its baseline reduction and unused-import cleanup.
  At parent `8b3da2e16be30895a7c65f8e31bd3c7614562096`, `just fmt-check`
  and seven existing jitter tests passed. Full lint found a redundant borrow;
  the current commit fixes it, and full `just lint` passed at that commit.
  Behavioral verification of missing-evidence cases and review remain open.

The build queue remains serialized. Earlier [checkpoint 3737](CHECKPOINT-3737.json)
preserves the historical audit; its former global implementation gate has been
superseded, while its evidence and unresolved questions remain in the backlog.

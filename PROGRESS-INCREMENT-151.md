# Bespoke type refactor: progress

Updated: 2026-11-08T12:58:20.212592+00:00

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
| Textual-conflict paths reviewed against both inputs | 3 / 3 |
| Additional overlapping paths reviewed against both inputs | 20 / 20 |
| Overlapping paths still awaiting bounded comparison | 0 / 23 |
| Existing contact-geometry tests passed | 11 / 11 |
| Existing autoresolve tests passed, including 9 melee iteration tests | 76 / 76 |
| Existing core equipment tests passed | 19 / 19 |
| Tactical server ingress tests passed, including range matrix | 9 / 9 |
| Tactical server player-projection tests passed | 33 / 33 |
| Tactical server bot tests passed | 13 / 13 |
| Tactical server equipment tests passed | 16 / 16 |
| Tactical iteration harness tests passed | 10 / 10 |
| Client targeting and acquisition helper tests passed | 6 / 6 |
| Client equipment fit and selection fixtures passed | 5 / 5 |
| Core catalog tests passed | 31 / 31 |
| Guarded developer authority, investigability and view fixture | Passed |
| Persisted selected and instance reach distinction | 1 / 1 fixture passed |
| Full P16 lint recipe on parent commit | Passed |
| Full P16 lint recipe on current commit | Passed |
| One-seed melee report matchups completed | 5 / 5 |
| Melee smoke report top-level audit flags true | 11 / 11 |
| Full-scale melee report at previous draft | Failed in first matchup |
| Reproduced failing seed after repair | 1 / 1 completed |
| Nearby autoresolve traces checked against aggregates | 100 / 100 |
| Nearby batch top-level audit flags true | 10 / 11 |
| Full-scale autoresolve traces completed before failure | 467 / 1,000 |

The quality counts describe the latest scan, not a count of defects. At P16
commit `ea44c97b9`, the complete `just lint` recipe passed: binding checks,
quality checks, WebAssembly and workspace Clippy, and three standalone Clippy
commands. The build used a shared target directory; selected artifact
provenance for this complete run has not been independently rechecked. Earlier
focused checks and the guarded authority workflow passed as described below.

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
  commit `7e169b976`: the 159-path snapshot is restored, with three textual
  conflicts resolved. A bounded source review compared all three conflict
  paths with both inputs. It found the typed values preserved, the speed
  conversion value-preserving, and all required movement observations set
  by the sole movement-event constructor. Behavioral verification of the
  new error path remain open. Three further overlap paths preserve both
  equipment chronology and typed geometry changes. Two more preserve typed
  combat settings, their validation, and length negation. Strategic weapon
  carry and tactical equipment retain the placement, chronology, and typed
  geometry changes. Four report files retain typed length conversions;
  their competing changes were formatting. Lifecycle, residence and client
  pickup changes also survive their overlaps. Harness, hit debug and
  equipment-generation fixture changes retain both inputs. Client equipment
  retains the typed pickup range and revision admission. The strategic
  character and lockfile comparisons complete the 23-path bounded review.
  Full dependency resolution, independent review, the other recovered paths,
  and client visual, input and fixture behavior remain unverified. The
  targeting and lunge-acquisition source paths were also reviewed; six
  existing helper tests passed. Five older selected shared dependency
  records match a previously checked server selection; one client build
  script record remains unverified. Five equipment fit and selection
  fixtures also passed with the same provenance qualification.
  The movement report uses typed measurements and rejects missing required
  observations; the new failure behavior still needs behavioral validation.
  A one-seed smoke run completed five matchups and all 11 top-level audit
  flags were true. The full-scale run then failed in the first matchup
  after 467 autoresolve traces: seed 468 reproduced an unavailable recovery
  interval. The report rejected `CannotRecover`, a valid combat state.
  The repair counts unavailable recovery separately and computes the mean
  only from available intervals, with no mean when none exist. Seed 468 now
  completes: its 11 available and one unavailable samples match the trace,
  as does the mean. In a 20-seed batch across five matchups, all 100 trace
  aggregates matched; knife novice seed 465 timed out at 256 rounds, so
  10 of 11 audit flags passed. An existing core fixture permits this bounded
  timeout while the report demands none. The pre-restoration draft
  `8ff11c82f` produced the same timeout, combatant outcomes, complete log
  and timeline for this seed. The zero-timeout audit remains unsatisfied;
  the comparison does not establish behavior for other seeds. The 3 GB bulk
  trace from the failed run was removed after recording its size and record
  count. Full-scale rerun remains open.
  A Rust quality scan passed at `f22bd2c26`, after behavior-preserving
  extractions and baseline reductions. At parent `aaf0a66c7`, formatting,
  workspace Clippy, all three standalone Clippy commands, and the guarded
  disposable-database authority workflow passed. The workflow included wedding
  capacity and rollback, investigation retry, puzzle, mission and outbreak
  checks. Repair, dialogue and development-fixture behavior remain unverified.
  Two subsequent commits changed test fixtures, and `48682faff` changed
  report aggregation with the focused validation described above. Commit
  `c438e0abe` corrected an authority-runner success message. Commit
  `5837f86b5` updated catalog fixture counts, kind totals and ID hash. All
  31 catalog tests passed. Commit `4afaf7b59` carries validated investigation
  authority through its consumers. Later commits extend an existing guarded
  fixture through the reducer and public view. Focused Clippy, formatting,
  Rust quality, and the isolated authority workflow passed for the change.
  The repair passed formatting, focused report Clippy and Rust quality; full
  uninterrupted `just lint`, current-head validation, review and
  integration remain open.

  Existing tests passed: 11 contact-geometry and 9 melee iteration tests at
  `aaf0a66c7`. At `5837f86b5`, all 76 autoresolve tests passed, including
  those nine, as did 19 core equipment tests. Nine tactical ingress and 33
  player-projection tests passed at
  `1c0f6299b`; and 13 bot tests at `e9c405a77` after correcting two existing
  attacker fixtures. The range matrix is included in the ingress count.
  Selected local dependency fingerprints were checked for these shared-target
  runs. A prior authority attempt used stale artifacts from a different
  worktree; five specific fingerprint markers were invalidated under the
  build lock, and the subsequent authority build selected nine refreshed local
  records without the other worktree marker.

  An existing mission source guard was updated after its assertion moved with
  the extracted code. Its revised assertions passed a direct source audit and
  compiled under Clippy. Its native test could not link SpacetimeDB host
  functions, so the test itself has not run. The guarded authority workflow
  now loads a persisted diagnostic longsword and deliberately gives its catalog
  row a different reach from its stored instance geometry. It confirms that
  the strategic equipment capability returns the selected catalog reach and
  the combat equipment returns the instance reach, then restores the row.
  This passed for one equipped melee weapon in an isolated local database;
  it does not establish every weapon or error path. Focused authority-feature
  Clippy, formatting and the full lint recipe passed on `7e169b976`. The
  shared target provenance qualification applies to the full run.
  Investigation consumers now retain the validated manifest and context,
  including developer-context bases, and the reversed investigability tuple
  lookup is removed. Existing guarded cases passed; the extended guarded
  fixture stored a developer wrapper and verified reducer lookup and hostile
  investigability. A subsequent guarded run also verified the public action
  view returns a developer-generated action under its public case ID. Native
  unit fixtures only compiled. Player spawn admission prepares all applicable players before
  issuing any spawn commands. The input queue is drained first and is not
  requeued on preparation failure. Source indicates Bevy's default error handler would
  panic on that error. The dispatcher does not monitor or restart its child
  in the inspected source; deployed supervisor and row redelivery behavior
  remain unverified.
  The core build uses geometry without default engine features; an offline
  dependency tree contained no Bevy packages. The iteration observer source
  retains the first typed error for readback;
  ten existing harness tests passed on unchanged iteration production code,
  but they do not inject an observer failure. Remaining review, behavior
  checks and integrated gates stay open.

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

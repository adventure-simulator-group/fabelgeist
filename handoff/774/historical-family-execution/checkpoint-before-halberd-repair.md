# Current execution checkpoint

The active Goal is the complete twenty-item plan in
`../pr774-canary-audit-20261020/migration-plan.json`. All twenty original exit
requirements are preserved literally in execution-ledger.json. No family is
accepted merely because its source is committed. Do not merge PR #774 or mark
the Goal complete until every requirement is implemented and verified.

## Authoritative source state

- PR: #774, open, unmerged; remote branch codex/shared-rig-identity-726.
- Published/local HEAD: 3544d8e99f1d9f765a813cc71b56f089e31c5118.
- Local branch: codex/774-review-contracts.
- Implementation checkout:
  `/home/brunosegovia/.codex/worktrees/ae46/adventure-simulator/target/semantic-goal/rig-review-774/worktree`.
- Implementation checkout is clean. Current main last fetched is
  c20169551087b86c0f639d7e8bbe151b510f410a, freshly fetched and already an
  ancestor; no conflicts.
- Six connected source families are published: P01 audio Seed, P05 jitter
  elapsed, P07 knee angle, P02 autoresolve BattleTime, P03 maneuver durations,
  P04 playback rates/phase/cadence/event spans. Full acceptance is pending.
- Fourteen families have no implementation: P06 and P08 through P20.

## Current hosted run: first unfinished step

Run 39452715702, job 118731479728, is confirmed live on the exact HEAD above.
Current early gates, WASM/deterministic draws and the complete four-command
native/standalone Clippy matrix pass. The native result was rechecked against
the actual current workflow commands. Revision recording also passed. Native
compiler repairs are validated on this head; this is not full family acceptance.
Constraint/shell artifact digest and both tested-head files match this SHA;
it proves 67 unit and 5 doctest executions, zero failures/ignores. Rig and
autoresolve steps pass; their nonempty logs await final artifact inspection.
MHR, Creator, and tactical animation/audio tests also pass. The job is live at
existing animation report tests.
Remaining selected behavior, required
nonempty-cell verification and official baseline are unfinished.
Read native-matrix-pass-3544d.json and constraint-artifact-verification-3544d.json.
Predecessor af3fd093a passed early gates,
semantic/growth, WASM and deterministic draws, then native checking found four
SignedSpeed argument errors in one report fixture. The previous eight IK/report
diagnostics were absent. Current repair 3544d8e99 adapts only those fixture
arguments; all current native/selected behavior/baseline acceptance is pending.

Re-poll this same run/job. Do not restart solely because an observation times
out or returns a transient error. On terminal failure, inspect only the earliest
failed contract with bounded log projection, repair that cause with the same
implementation agent, preserve checks and publish the repair. Do not chase a
missing-artifact upload symptom before its earlier compilation/gate cause.
Do not accept P04 or advance to P06 implementation without the complete
required matrix and relevant nonempty behavioral evidence. Double-check the
actual tested revision, command coverage, outputs and source before acceptance.

Latest commit 3544d8e99 uses Core SignedSpeed::ZERO for existing zero-speed
arguments and admits the fixture's existing 5.5 m/s value once. Production
source before cfg(test) is byte-for-byte unchanged; all function names, cases,
tolerances and 83 assertions remain. Read report-speed-fixture-native-repair.json
for the four diagnostics, canonical owner/signature, metadata and review scope.
No tests, fixture helpers, cases or assertions were added. Diff check passes;
source is committed and pushed, and the worktree is clean.

Predecessor af3fd093a fixed the preceding eight native diagnostics: bool
posture assertion, UnitInterval terminal-contact fixture weights, a bogus getter
on a native fixture height and a stale cadence local. It extracted event-stream
acceptance without changing ordering, uniqueness or four/one scenario counts.
The two IK files retained 16 and 396 assertions. See ik-fixture-native-repair.json.
The earlier ingress file-debt removal remains; its function ceiling is unchanged.

## Published source evidence

P04 source commit is c59c01e3e; exact formatting is 82528917d. Its twenty-file
trace and deliberate YAML/admission changes are in P04-review.md and
P04-source-review.json. Published P04 file hashes were compared with current
source and still match. Five existing cadence tests were moved with their
owner, preserving their names and fourteen assertions. No new tests, assertions,
cases or fixture helpers were added.

Code and Rule source review were performed twice by this implementation agent;
this is not independent whole-PR approval. Comment Review is omitted. No new
full bespoke-audit wave is authorized; the earlier three-wave limit was used.

Repairs already published include the typed route-fixture numerical endpoint,
canonical posture-facing Result import, server defense control flow and lunge
facade import scope/order. Their reports preserve the exact evidence and
contracts. Do not undo them or restore historical patches from local output.

## Next family: P06 pre-edit facts

No P06 source has changed. Read these updated artifacts:

- P06-execution-plan.md: complete intended connected scope and ordered actions;
  remaining pre-edit source traces are explicit, so this is not acceptance.
- P06-reference-classification.json: 185 original query references in 74 files;
  157 SceneTerrain/application query candidates, 28 other native or already
  typed owners. These are reference counts, not defect or completed-trace counts.
- P06-expanded-survey.json: canonical owners and additional retained consumers.
- P06-absence-policy-plan.json: required support, spawn sequencing, native
  failure policy and the locked Bevy error handling inspected from source.
- P06-validation-selection-plan.json: existing fixture/module/feature coverage.
- P06-coordinate-producer-frontier.json: retained shrub/plant coordinate records
  and grass/litter/foliage forwarders missing from the direct query inventory.

Reuse ScenePlanPoint, SupportElevation and SurfaceHit/Dir3. Propagate through
all actual IK/settle/ragdoll, diagnostics, scene/garden/furniture, vista/backdrop,
server and viewer consumers. Preserve triangle/LOD, multilevel selection,
absence, signed/zero elevations, exact native math and normal re-normalization.
No equivalent local support-height wrapper or typed-zero missing-hit fallback.

The connected range trace reaches a retained raw Vec2 in TreePresentationCache,
through oak/beech bark wrappers to actual Vec4 shader packing. Keep checked
bounds through that chain and couple image/range availability. Owned geometry
admission currently permits empty support, so do not assume a finite range or
retain infinity. Required capture evidence must reject unavailable bounds.

Server spawn currently drains connected-player rows and creates enemy commands
before support lookup. Plan: wait for scene readiness before draining, pre-admit
a whole batch's placements before entity commands, and propagate structured
required-support failure at the existing Bevy Result boundary. Locked Bevy
0.19.1 defaults unclassified errors to panic severity; no server override was
found. Do not silently consume a failure as success, resample RNG positions or
post a fabricated battle outcome. AppExit alone does not prove same-frame
terminal submission is suppressed. This is a source plan, not runtime proof.

The proposed core vista_surface:: test filter was removed because that module
has no tests and would produce zero executions. Existing presentation vista
consumer tests provide indirect coverage. The production garden-route test is
ignored and requires FABELGEIST_TERRAIN_SCENE; do not count compilation as its
execution, invent a replacement fixture or silently unignore it. Existing
unit/compile selections do not prove calibrated renderer or live-play behavior.

Finish remaining consumer-body and fixture traces before the atomic source
migration. Preserve native grid/density/triangle endpoints; private scope alone
is not justification for raw application forwarding. Move the sampling/query
responsibility before materially expanding the large scene root.

## Preservation and operating constraints

Hosted checks only: no local Cargo, formatter, quality scanner, parser probe,
generator, runtime, test or numerical-probe commands. Static source/Git and
stdlib metadata inventory are allowed. Add no tests/cases/assertions/helpers
unless requested. Never raise ceilings, weaken a gate or add a suppression to
hide migration debt. No agents were spawned for this repair loop.

Original stash 6c109560050d97dd6deeb842b1d71ec8a280949b remains referenced.
Never restore it wholesale. Preserve exactly four primary checkout Cargo lint
edits; the implementation branch's CI does not validate them. Preserve other
worktrees, generated DB bindings, database state and unrelated work. No wiki
prose edits on this implementation branch. Do not create another Goal, PR,
worktree, chat or public conversation snapshot for this continuation.

The public PR validation paragraph identifies 3544d8e99/run 39452715702 and
accurately states full acceptance pending. Update it from fresh metadata after
new material evidence; use the existing reviewed JSON + gh REST PATCH workflow.
No discussion comment or new public snapshot was posted.

Historical chronological notes are preserved in
checkpoint-history-through-2d063.md. Current source and current-hosted.json
supersede old live-run statements in that history and historical reports.

## Latest additional P06 support interfaces

The coordinate-frontier report now also includes TuftPlacement::height and
its ScenePlacement/VistaTuftPlacement implementations, which flatten owned
SurfaceHit elevations across a trait before scatter_cell_tufts packs native
InstanceData. The same trace includes ground::tree_root_height and its
production caller in vista/details.rs, plus the existing owned/sampled root
fixtures. These interfaces do not appear as direct height_at calls. Keep
canonical elevation and scene-plan types through them; no P06 source is
implemented and remaining source traces are still explicit.

## Latest merge-conflict request

Fresh fetch confirms main c20169551087b86c0f639d7e8bbe151b510f410a is already
an ancestor of head 3544d8e99. The worktree is clean, unmerged-file list is
empty, and GitHub reports MERGEABLE (UNSTABLE refers to pending CI). No source
change or merge commit was needed. See merge-conflict-check-3544d.json.

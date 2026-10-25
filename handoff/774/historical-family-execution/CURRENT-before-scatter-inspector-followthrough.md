# Current execution checkpoint

Continue the existing Goal and original twenty-item migration plan in
`../pr774-canary-audit-20261020/migration-plan.json`. Do not complete the Goal,
merge PR #774, add a new full audit wave, or omit an original exit requirement.

## Source and preservation

- Worktree: `../rig-review-774/worktree`; use it for all implementation edits.
- Local branch: `codex/774-review-contracts`.
- Published branch: `codex/shared-rig-identity-726`.
- Published/local HEAD: `82cb2a3acccc852bba6a7ef9c706c337ba9fe765`.
- Freshly fetched main: `c20169551087b86c0f639d7e8bbe151b510f410a`, an ancestor.
  The prior merge-conflict request was verified against freshly fetched refs:
  no unmerged index, merge in progress or required merge remained.
- Current work is uncommitted: 170 Rust paths and the hosted workflow. No new
  hosted run validates it. Do not publish an incomplete P06 family.
- Original stash `6c109560050d97dd6deeb842b1d71ec8a280949b` remains present and
  untouched. Do not restore it wholesale.
- The primary ae46 checkout still has exactly four unrelated manifest edits:
  root, character creator, MHR and NumPy storage. Preserve and validate them
  separately.
- P06-current-source.patch, P06-current-source-files.zip and
  P06-current-source-backup.json preserve the latest complete working-tree
  snapshot, including new files and deletions. Read the metadata hashes before
  relying on a snapshot. No staging, commit, branch, stash or push was changed.

## Latest user steering and review follow-through

The new partial review has ten comments, submitted 2026-10-20 at 21:09:46 UTC.
The user also explicitly asks to find calls that pass `x.seconds()` or another
primitive getter to an application function that should accept `x` itself.

Read partial-review-20261020-followthrough.json and
getter-receiver-followthrough.json. All ten review suggestions have local
source changes. The two questions have replies on GitHub, explicitly stating
that these changes are uncommitted and unvalidated. Do not post duplicates or
resolve threads before the corresponding change is published and verified.

The duration validators now retain AnimationElapsed, including positive
interval checks and a typed one-second jitter limit. SpeedError, RecoveryTime
and AnimationElapsed imports were corrected. The deterministic cancellation
seed-search limit is named without changing its seed order. Authored threat
construction retains unknown-identity and elapsed failures in AuthoredThreatError;
the SpacetimeDB-facing adapter formats its required reducer error text.
Preexisting authored melee fixture String/expect paths are not claimed fixed.

The getter trace found an additional missed handoff: opposed movement passed
elapsed.seconds() into integrate_axis. Its signature now accepts AnimationElapsed
and projects only inside the complete trapezoidal integration. The same seven
existing fixture calls retain their numeric inputs. Fatigue validation and
redundant tactical/pose timing validation also stop projecting admitted leaves
into generic scalar validation arrays. Historical green acceptance of the six
published families does not validate this follow-through.

getter-handoff-candidates.json inventories 4,233 lexical getter occurrences
across the current full PR scope. It is not a completed audit: comments, tests,
application APIs and native kernels still need actual receiver classification.
Do not infer test scope from the first cfg(test) attribute. Constructors and
casts remain covered by the original inventory. Preserve actual native curve,
random sampling, arithmetic and serialization boundaries; do not bless a helper
merely because it is private or has a boundary comment.

## P06 implementation and first unfinished step

P06 remains in progress. Previous source increments include checked terrain
queries/ranges, atomic refinement callbacks, IK floor and ragdoll support,
range/raster admission, scatter/vista, strategic preparation, server spawn batch
admission, capture/viewer, dispatcher geology/inspection and art-demo errors.
Read the linked traces in execution-ledger.json; do not restart completed prefix
work based on stale inventory labels.

This turn additionally implements source follow-through for:

- Atomic SceneTerrain height rewrites: a failed callback cannot publish a mixed
  prefix of old and new height samples.
- Ragdoll penetration and complete corrected positions are admitted before
  publishing physics output; native sample and maximum equations stay intact.
- Tree and rock capture focus retain Position<Scene>. Tree presentation species
  is an immutable typed component and remains typed in CaptureRecord, with the
  same two explicit serialized labels. No display-name parsing drives framing
  or beech validation.
- Vista capture geometry reuses canonical GeographicHeightRange and controls,
  retains checked distances/elevations/points and preserves strict first ties
  and original grid-coordinate arithmetic. No samples is explicit absence;
  summary measurements serialize as null, not fabricated zero/origin values.
- Peak/valley captures reject missing vista observations through failure.txt and
  AppExit. Horizon without vista uses the real ground-eye heading and terrain
  bound. Normal valid-observation camera offsets remain unchanged.
- Complete proof-camera calculations now belong to view_camera and its
  landscape module. Required botanical/building/live-query failures are typed.
  The existing root test module was moved last without adding tests or cases.
- Interpolated detail heights reject nonfinite completed values before setting
  validity bits; invalid geometry is excluded by the existing quad mask.

Read P06-reservations-normal-policy-followthrough.json and
P06-normal-and-query-source-comparisons.json before continuing. The latest source
adds five connected follow-through areas:

- ObstacleReservation retains canonical ScenePlanPoint, PlanExtents and building
  orientation. PlanDimensions::half_extents owns the infallible half operation,
  including valid zero from subnormal underflow. There is no zero floor field.
- Removed the old BuildingPad type and its unused grading API. The sole
  geographical refinement caller previously supplied an empty pad slice. The
  unchanged apron is 5.5 metres; a complete retained obstacle vector is admitted
  before publication. Invalid grid projection propagates a geometry cause.
- Native ground assembly receives the existing SceneTerrain and reads its grid
  dimensions/scale internally. The source-grid obstacle spacing stays distinct.
  Removing three duplicate inputs also removes the existing lint suppression.
- TerrainContactHeightmap fields are private. Weather, tree residency and viewer
  read its typed range before consuming the native image at Assets::add.
- Slope alignment and scree coverage now accept Dir3. Projection and the old
  try_normalize occur inside the original rotation calculation; production adds
  no normalization. FootDescent is produced by run_support_descent and remains
  typed through airborne clearance and floor application. Its FollowFloor and
  RaiseOnly cases preserve the exact old condition and numerical decisions.

Four native source/condition comparisons are recorded. Seven existing literal
fixture policy arguments and two normal fixtures were adapted; no tests, cases,
assertions or helpers were added. The boolean run_support_eligible_for_descent
API is removed without a forwarding path. Existing test/assertion metadata now
covers 537 names and 2,355 remaining assertions in the broader dirty scope.
These are source facts, not compilation, runtime or complete review proof.

The named getter inventory now additionally covers rates, ratios, counts,
encodings and numeric projections. It includes many intrinsic methods and tests.
No count establishes a defect or a clear verdict. Getter names outside this
explicit set, field accesses and the original constructor/cast survey remain
coverage obligations.

First finish the current producer/caller/fixture/signature reconciliation against
P06-query-candidates.txt and the getter inventory. Recheck all native/typed
exclusions against actual operations. Historical P06-reference-classification
statuses and older source hashes are not current verdicts. The new vista and
species propagation is implemented but not yet fully reviewed or validated.

Then complete two full Code/Rule source passes over the final P06 diff and new
modules. Audit admission, unavailable states, authority/frame, exact arithmetic,
module ordering and growth. No full P06 Code/Rule PASS has been recorded.
Comment Review is omitted; the earlier three full bespoke-audit waves are
exhausted. Use the single implementation agent; no new delegation is authorized.

P08 camera quantities/history, P09-P16 producer families, P17 report measurement
retention/applicability, P18 statistics, P19 influence and P20 continuation remain
unfinished. Do not use these deferrals to leave an actual P06 terrain handoff raw.

## Validation and publication constraints

Hosted CI only. Do not run local Cargo/build/check/test/fmt/Clippy/metadata,
project scanners, Rust parser probes, generators, runtime or numerical probes.
Static source/Git, stdlib JSON/TOML/ZIP/hash metadata and git diff --check are
permitted. Do not add tests, cases, assertions, fixture helpers or doctests.

The workflow now adds fourteen existing command cells and requires all 39
cells. P06-workflow-source-verification.json verifies all sixteen original
Creator vectors and every previous workflow line remain unchanged in order.
P06-validation-selection-plan.json gives exact commands and fixtures. None of
the new selections has run. Record known ignores separately from executed cases;
unit compilation does not prove missing-support lifecycle or renderer behavior.

record-source-preservation.py verifies the existing 537 test-name multiset and
2,355 remaining assertion occurrences across changed responsibilities. Exactly
two verified production assertions become typed rejections; no unit assertion
was removed. All twenty original exit requirements remain unchanged. This is
static preservation metadata, not compilation or runtime verification.

Only publish a source-complete P06 after the source reviews. Then repair hosted
failures, inspect nonempty exact-head artifacts, and apply only relevant official
hosted baseline reductions. Many changed sources are not formatted yet. Obtain
actual hosted formatting diagnostics and apply them; do not run local formatting
or weaken a gate. Do not add a broad scope, raise a ceiling or alter cache policy.

Published 82cb2a3ac remains the last validated head: run 39466774426 and job
118780099699 succeeded with 748 selected executions including eight doctests,
zero failures/ignores, 25 nonempty required cells, sixteen Creator vectors and
nineteen matching official baseline reductions. See the existing 82cb2 artifact
verification reports. It does not cover any current uncommitted source or the
primary checkout's four manifest edits. No active CI or local process remains
to poll. The overall Goal remains active.

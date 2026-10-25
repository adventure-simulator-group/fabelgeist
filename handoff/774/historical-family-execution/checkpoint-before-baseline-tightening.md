# Current execution checkpoint

The active Goal remains the complete twenty-item migration plan in
`../pr774-canary-audit-20261020/migration-plan.json`. All original exit
requirements are preserved literally in execution-ledger.json. Do not mark the
Goal complete with unfinished families, and do not merge PR #774.

## Authoritative source

- PR #774 remains open and unmerged, remote codex/shared-rig-identity-726.
- Published and local HEAD: a15f24ad44fc22f86e1e1fd9f2ae47b6bb0ba860.
- Local branch: codex/774-review-contracts; implementation worktree is clean.
- Checkout: `../rig-review-774/worktree` from the semantic-goal directory.
- Main c20169551087b86c0f639d7e8bbe151b510f410a was freshly fetched for the
  merge-conflict request and is already an ancestor. GitHub reports MERGEABLE.
- Six source families are published: P01 audio, P05 jitter, P07 knee angle,
  P02 battle time, P03 maneuver durations, P04 playback/phase/cadence/events.
  All remain pending complete required acceptance. Fourteen families have no
  implementation: P06 and P08 through P20.

## First unfinished step: hosted acceptance and baseline

Current run 39459842316, job 118756130752, is confirmed live on a15f24ad4.
Re-poll this exact handle; do not restart because observation times out.
Current formatting, generated bindings, semantic/growth, WASM Clippy and
deterministic draws pass. All four native Clippy configurations now pass,
rechecked through connector and direct REST metadata.
Constraint/shell artifact digest and both revision files match this head;
67 unit and five doctest executions pass with zero failures/ignores. Shared rig
and MHR steps pass. Creator tests are live; autoresolve step metadata is still
incomplete, so its result must be checked from final logs. Full matrix,
nonempty required-cell verification and official baseline remain pending.

Predecessor run 39452715702/job 118731479728 is terminal FAILURE on 3544d8e99.
It passed formatting, semantic/growth, bindings, WASM/determinism, all four
native Clippy commands, constraint/shell, rig, autoresolve/defense, MHR, all
sixteen Creator selections, tactical animation/audio, viewer reports,
equipment, Core animation/config and server startup. Its final maneuver step
had 32 passed, one failed. Baseline export and final nonempty verifier were
skipped; never call this run green or claim it accepted P04.

The sole failure was the existing halberd reach golden expecting 2 metres.
The entire test and relevant production sources match main c20169551; main
was not separately executed. Source tracing proves the authored spike tip is
at 2.18 metres, the explicit Grip component selects the 1.82-metre shaft's
centre (0.91 metres), and combat reach is their 1.27-metre difference. Do not
use the lower-priority 20-percent fallback grip rule. Current a15f24ad4 changes
only this golden and its two-line explanation. Production and all source
outside that exact replacement are byte-identical; the five assertions,
fixture inputs, other four checks and 1e-6 tolerance are preserved.
Read halberd-reach-fixture-repair.json for source references and two self-review
passes. No test, case, assertion or fixture helper was added.

Verified predecessor artifact rig-contracts-3544d.zip, id 12282006570, has
SHA-256 bd074be9050776a5e2c61e5dba375dd5b88b092c4ac551ff26b6df87c1754649.
It pins exact head 3544d8e99 and contains 643 successful selected executions
before the failing maneuver log. Every successful log is nonempty; all sixteen
Creator command vectors match current workflow selections. It lacks
quality-baseline.toml and required-cells.json because those steps were skipped.
Read rig-artifact-results-3544d.json. Separate constraint artifact proves
67 unit-test plus five doctest executions, digest and both revision files
verified; see constraint-artifact-verification-3544d.json.

After the current run completes, inspect actual exact-head artifacts and
required cells. Apply only relevant debt reductions from the official hosted
baseline export; never invent counts, raise ceilings or weaken gates. Complete
P04 acceptance before P06 implementation. Keep original all-twenty scope.

## Source and review evidence

published-family-snapshot.json pins 58 current source file hashes, all
rechecked; P04-source-review.json pins the twenty P04 file hashes, also
rechecked. Existing fixture-only and compiler repairs remain; earlier source
reviews and their technical contracts are retained in P04-review.md and repair
reports. Code/Rule checks were two implementation-agent source passes, not
independent full-PR approval. Comment Review is omitted. Do not start another
full bespoke-audit wave: the earlier authorized three-wave cap was exhausted.

## Next P06 pre-edit work

No P06 source has changed. Read P06-execution-plan.md and its inventory,
absence-policy, validation-selection and coordinate-producer reports. Reuse
ScenePlanPoint, SupportElevation, SurfaceHit and Dir3; preserve exact native
arithmetic, sampled/owned authority, no-hit behavior, visit/RNG order and the
separate meanings of support, relative clearance and ankle constraints.

Additional concrete traces completed during this hosted wait are recorded in
P06-range-consumer-trace.json and P06-viewer-dispatcher-trace.json. These
extend the direct query-name inventory to tree image/range binding, weather,
manifest bounds, furniture catalogue staging, boundary CapturePose state and
five additional inspection callback modules. GeographicHeightRange includes
exact source-control locations: do not fabricate those to reuse it for an
unlocated scene range. It does fit actual point/elevation extrema records.

terrain_heightmap_image currently substitutes minimum height for absent
support, and both bark/weather shaders decode it unconditionally; alpha alone
cannot fix absence. Finish the complete producer-to-shader or explicit image
rejection contract before implementation. Boundary CapturePose currently
retains raw x/z in a const authored table: resolve canonical checked const point
admission, then propagate the checked point rather than admitting at every
query. Remaining IK/ground-scatter body traces are still explicit; do not
claim every original query candidate fully traced.

Existing production garden-route execution needs a frozen
FABELGEIST_TERRAIN_SCENE export and is ignored normally. No new fixture/test is
authorized. Existing unit/compile coverage does not prove calibrated renderer
or live tactical acceptance. Preserve those limits explicitly.

## Preservation and operating limits

Hosted checks only. No local Cargo/build/test/fmt/Clippy/quality scanner,
parser probe, generator, runtime or numerical-probe commands. Static source,
Git and stdlib artifact/metadata inspection are permitted. No new tests, cases,
assertions or fixture helpers unless requested. No ceiling increases, hidden
suppressions, compatibility paths or weakened acceptance cells.

Preserve original stash 6c109560050d97dd6deeb842b1d71ec8a280949b and exactly
four unrelated primary Cargo lint edits. The implementation CI does not
validate those primary edits. Do not restore the stash wholesale, import
unfinished scanner work, modify another worktree, edit generated DB bindings,
reset databases, or create another Goal/PR/worktree/chat/public snapshot.
No agents were spawned. Public PR validation body now identifies a15f24ad4 and
run 39459842316, explains the failed predecessor and retains acceptance limits.
Historical checkpoints remain local; current-hosted.json is authoritative for
latest run handles and evidence.

## Latest concrete P06 pre-edit evidence

P06-ik-scatter-body-trace.json follows the floor callbacks into the actual
max/sphere/disk kernels, preserving final finite checks before f32::max. It
also traces PlantSite roots through capture anchors, retained plant_focus and
actual camera/LOD distance operations, plus TuftPlacement's coverage/community
receivers. These were additional paths beyond the direct height query search.

P06-admission-decisions.json verifies checked const point admission against
locked glam 0.32.1 and pinned Rust source. Adapt the existing capture macro; do
not introduce unchecked points or a new fixture helper. The image/range policy
rejects absent bounds or required unsupported texels; changing unused alpha is
insufficient because both consuming shaders decode unconditionally. Optional
visual consumers remain unavailable; required capture rejects. Preserve all
supported image bytes and encoding arithmetic. These are pre-edit decisions,
not implemented source or a family acceptance claim.

## Current-head native evidence and remaining strategic trace

Read native-matrix-pass-a15f2.json and
constraint-artifact-verification-a15f2.json for current-head evidence. These
are narrower than full family acceptance. The direct REST job metadata agrees
with the native pass. Step24's status lags later steps; do not accept its tests
from order alone or restart the job. The final artifacts remain decisive.

P06-vista-placement-body-trace.json verifies the real native stitch/lerp boundary
and the separate application wrappers. It also finds oak candidate support
dropped and recomputed after selection; retain the original checked observation.
P06-strategic-preparation-error-trace.json closes the new required-support error
route through street, venue and world preparation to the actual status-message
boundary. Use a cohesive typed error surface, preserving canonical causes; do
not stringify the new support failure at an internal helper or use Other(String).
No P06 source implementation has begun.

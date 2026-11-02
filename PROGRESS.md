# Bespoke type refactor: progress

Updated: 2026-11-02T23:18:32.870733+00:00

Latest verified checkpoint: **534**. Goal: **active**.

The refactor remains in the required source audit. Its 13 perspectives include
reuse of existing types and preservation of semantic values through native
boundaries. Completing a scoped question or correcting an old status note does
not complete the audit or authorize implementation.

| Phase | Status |
| --- | --- |
| Required audit | In progress |
| Product implementation | Not started |
| Independent review | Not started |
| Product and runtime validation | Not started |

## Latest checkpoint

Qualified current scene-readiness marker producers and their input queries.
Confirmed disabled grass explicitly settles, while enabled grass/terrain/tree
preparation failures retain pending work. Recorded that terrain accepts optional
ground but both scatter queries require it, and refined four existing journals
with exact current caller evidence.

This checkpoint closes no additional matrix cells: the newest revision remains
51 of 403 complete, with 352 open; the preceding revision retains 189 of 767.
Actual scene input/replication completeness, nested cache/material/geometry
outcomes and scene-specific native readiness still need evidence. No loading
stall or GPU incident was executed. Product implementation, independent review,
builds, tests and runtime validation have not begun.

## Investigation since checkpoint 534

The debug world-dump writer includes scene identity and terrain but omits ground
and environment. Debug startup skips fresh terrain generation when loading a
dump, and the terrain restore observer rebuilds collision and replication. The
existing round-trip test exercises a player and inventory, not scene inputs.

Client loading expects terrain and two ground-cover markers per scene. SceneId
addition creates pending terrain work. A persistent incomplete stage produces a
stall verdict after 90 seconds of unchanged game-time progress. The runtime
impact, component delivery and other insertion paths remain to be verified.
These source observations are exploratory and have not closed audit cells.

The separately loaded scene input supplies the vista and world bounds. The
inspected input loader and dump loader do not compare its identity with the
dump. A different input could therefore produce inconsistent scene data. The
furniture restore test uses the same input on both sides and does not test that
case. Ground derives reflection, while environment does not at its declaration,
so the snapshot representation needs further design analysis.

## Working on now

Trace scene-readiness marker populations against admitted scene input and
spawning/replication lifecycle. Disabled grass explicitly settles; terrain
accepts optional ground while both scatter queries require it. Keep missing
input and native preparation outcomes distinct before completing the remaining
source perspectives.

- Continue independent current-source assessments for the remaining changed
  paths; preserve native boundaries and exact source scope.

- Reconcile the retained build-target specification question and other connected
  findings against exact canonical owners and aliases.

- Complete all13 perspectives, all20 families, older source deltas and the
  ordered repair ledger before implementation.

## Work completed since the handoff

- Compared catalog fields with existing types and traced numerical decoding,
  weapon weights, generated weapon geometry, surface frames, attachment
  identities and slots, and protection-energy units. Recorded exact reuse
  decisions, rejected suspicions, and remaining admission questions.

- Completed the named L2 organization-training source question: existing skill,
  religion, bestiary, and language types should remain intact through curriculum
  construction, execution, and display. Implementation and validation remain
  pending.

- Completed the named L2 book-target source question: authored and runtime book
  targets share the schema owner. Recorded target-specific rank caps, selection
  priority, correlated progress, and distinct hour-result roles. Implementation
  and validation remain pending.

- Completed the named L2 fame-preview source question as a conditional finding:
  server-rendered rewards apply population scaling, while browser recomputation
  omits it. Native reputation scaling is present. No live settlement or browser
  incident was checked.

- Earlier continuation source work also covered capture and rest workflows, rest
  booking, supplies and recruitment, filth and soap, food exposure, world-event
  identity, and consequence keys. These source investigations do not constitute
  completed repairs.

- Selected the camp binding carrier: explicit unbound/bound state, reuse of the
  canonical journey-camp identity and codec, and a separate elapsed availability
  stamp. Requalified 21 earlier source rows without new body credit; parent
  identity and global alias obligations remain open.

- Selected distinct zero-valid journey movement/elapsed minute quantities and
  rejected f32 seconds for exact integer-minute roles. Inspected native/web
  challenge ID projection parsers and relevant demo filters; no incorrect-
  selection runtime incident claimed.

- Selected distinct frame/trial/challenge full key owners and a shared complete
  errantry decoder, retaining road encounters, ordinals and namespace
  distinctions. Recorded actual native/web filter behavior and later acceptance
  requirements; no runtime incident or global closure claimed.

- Traced the active native identity codec against exact locked SDK sources.
  Recorded that Serde validation does not govern the independent SATS decoder
  and selected checked native construction; no bad payload or live incident was
  executed.

- Traced the connected contract identity/lifecycle path and mapped stale
  abandonment to the existing active-party relation record. Distinguished the
  shared key from existing ContractState lifecycle ownership and retained
  direct/HTTP/queued acceptance requirements.

- Linked the camp binding repair to inherited M40 using current qualified source
  coverage. Compared73 candidate references and6 inherited records while
  preserving separate chronology, spatial coordinate, outcome/error and
  lifecycle obligations.

- Corrected stale SQL/SATS journal status using authenticated existing lens-12
  adoption and completed adapter-read evidence. Kept malformed-wire diagnostic
  erasure separate from the native checked-constructor bypass and awarded no new
  read credit.

- Traced the camp/road choice path with own lens-12 source evidence. Recorded
  explicit-choice versus automatic-ignore behavior, web/native readiness
  mismatch and the conditional blocked-ignore admission question without
  claiming a live gameplay incident.

- Observed all14 current authored ignore choices: shared IgnoreTravel
  throughout, with four bounded delays and none Blocked. Added a separate
  provisional admission question, preserving the distinction between current
  content, possible invalid admission and runtime acceptance.

- Qualified time-delay failure propagation and forecast refresh for road
  choices. Distinguished documented database rollback from untested terminal-
  outcome availability; retained the gameplay policy and supported witness as
  open obligations.

- Rejected stale zero-time blood-terminal and zero-length lifecycle recursion
  suspicions using current core/native/calendar source. Traced existing typed
  disease terminal causes and positive-time blood settlement without claiming a
  live incident.

- Linked the current native injury death timestamp chain to the existing
  canonical injury-clock finding. Retained the single clock owner and existing
  death types; distinguished disease-after-clock timing and road-delay rollback.

- Accounted for all9 exact direct injury-settlement call sites under the
  existing injury-clock repair, preserving each recovery role and terminal-
  control-flow policy. No product code or runtime check was performed.

- Closed the newer Cargo configuration cell under the magic-string perspective
  using complete before/after source. Recorded external compiler option
  ownership and left the other12 perspectives independent.

- Completed the newer Cargo configuration path under all13 perspectives using
  independent source assessments. Accounted for build policy, type ownership,
  admission, aggregate modes, derivability and numeric/native option roles
  without claiming compiler/browser validation.

- Source-accounted the WebAssembly build workflow under all13 perspectives.
  Traced Cargo-reported output paths and actual bin/profile/version
  declarations; retained internal binary-mode string classification as a
  distinct provisional owner-fit question.

- Completed both newer manifest paths under all13 perspectives at the native
  Cargo declaration boundary. Verified exact dependency/patch removal while
  leaving renderer code, shaders and lock resolution independently pending.

- Traced the new chunk renderer through five foliage producers. Recorded
  CPU/shader declaration agreement and concrete admission/lifecycle questions
  while distinguishing valid current constants and nonempty spawning from
  unproved failure paths.

- Qualified a normal configuration-source path for the existing fade-band
  question: finite equal fade-out endpoints pass validation and remain equal at
  shader packing. Distinguished shipped positive-width bands and linked current
  diagnostic formatting to the existing graphics-error repair.

- Qualified exact locked typed-float decoding through serde-saphyr1.2.0 and
  serde_core1.0.229. Distinguished generic-value rejection from concrete f32
  admission and linked NaN bounds to existing config/interaction questions
  without claiming a live crash.

- Traced grass material uniforms into the current shader and inspected
  configured mesh cardinality arithmetic. Closed ownership and enum-choice
  perspectives for eleven renderer paths plus three native-string perspectives;
  retained independent numeric, cache, native and visual duties.

- Authenticated the locked Bevy, glam, WGPU and bytemuck source chain for chunk
  instance layout. Distinguished scalar Vec3 from Vec3A, checked declared native
  vertex sizes, and recorded why finite-only bespoke values must be lowered into
  a separate raw Pod payload.

- Traced locked Bevy unbatchable storage, CPU/GPU preparation, per-entity draw
  queries and bind caching. Rejected the specific wrong-material substitution
  suspicion from hash collisions, preserving the source conditions and future
  batching reassessment duty.

- Compared eight changed scatter shader paths, including three deletions and
  three additions, with their material owners. Recorded explicit retirement or
  replacement and closed 24 ownership, enum-choice and native-string checks
  while preserving visual and native validation duties.

- Completed loose-stone owner, batch, spawn and random-purpose source
  comparisons. Traced density, mesh variant, material and physical-count
  propagation. Retained a provisional question about recovering density from an
  integer asset index and keeping parallel asset arrays correlated.

- Completed all remaining perspectives for loose-stone random-purpose
  declarations. Read the shared Seed, StreamId and RNG owner APIs and qualified
  the native fixed-output hash boundary. Preserved authored replay names and
  explicit lowering into the generic GPU payload without claiming a replay run.

- Compared combat configuration and runtime snapshot changes. The added inverse-
  kinematics cache keeps checked units intact; the main module only changes the
  public export list. Linked current digest cause formatting to its existing
  journal without accepting a duplicate finding.

- Traced server and animation-viewer configuration loading through validation,
  runtime installation and Bevy resource insertion. Confirmed the inverse-
  kinematics helper uses the existing checked configuration type. Distinguished
  early server loader cause formatting from direct viewer final panic
  formatting.

- Traced pose culling through canonical visibility, the derived ECS marker,
  visible resume and ragdoll exemption, plus its inverse-kinematics query
  consumer. Qualified default native schedule behavior and inspected equipment
  skin borrowing through shared rig-name owners. Preserved lifecycle, membership
  and pose-continuity duties.

- Read the new loading, progress, view, readiness and equipment-settlement
  modules plus changed texture installation support. Authenticated the locked
  native pipeline and asset-event boundaries: an empty waiting set does not
  certify successful pipelines, and asset events retain typed causes that the
  loading screen currently drops. Preserved the explicit policy allowing play
  after failed equipment generation settles.

- Traced full current parent setup, browser command drain, presentation facade
  and netcode exports, with bounded UI and join/reconnect support. Authenticated
  the native disconnect enum and observer borrow API. Distinguished cloneable
  asset failures from non-cloneable connection errors, preserved optional render
  failures, and linked re-entry identity behavior to an existing journal. Closed
  ten enum/string checks across five small paths without claiming lifecycle or
  runtime success.

- Traced equipment body/design preparation, asynchronous fit completion, support
  failures, reset markers and rendering-stage wiring. Selected retention of
  owned failure causes under the existing equipment-failure record while
  preserving optional-equipment settlement. Reconciled the older support-
  selection accessor inconsistency as corrected in current source and
  distinguished preview counts from stricter capture readiness; generated-
  geometry admission remains open.

- Traced DOM rectangle rounding through strategic/map protocols into locked
  native viewport and crop projection APIs. Strengthened the existing subpixel
  question with a source arithmetic example and actual unfiltered consumers,
  without claiming a browser incident. Preserved existing map admission and
  acknowledgement checks, native crop policy and the corrected current street-
  position test. Closed four protocol enum/string assessments.

- Traced regional/strategic status, city installation and assembly into the
  actual render-side readiness writer. Distinguished exact assembly roots from
  the owner-only GPU flag and command encoding from completed presentation.
  Recorded a conditional pending-candidate/newer-rejected-request ticket
  misassociation under the existing preparation-outcome owner. Preserved empty-
  city and optional-failure policies, and closed two parent enum/string checks.

- Completed all13 source perspectives for the added pipeline-telemetry module
  using independent dispositions and actual native schedule/cache APIs. Selected
  App-owned optional observation semantics while retaining zero-valid native
  counts, existing pipeline states and BacklogPeak history roles. Qualified that
  sampler ordering is weaker than rendering completion and native pipelined
  rendering is excluded on WebAssembly. Broader readiness and runtime contracts
  remain open.

- Completed the eleven remaining source perspectives for equipment settlement,
  preserving its earlier enum/string assessments. Reused actual ECS
  relationship/character owners and retained the existing pure predicate.
  Rejected transferring the preview count-substitution suspicion to its all-rows
  check, while keeping expected inventory/replication membership and stricter
  preview/capture contracts separate. No new equipment wrapper or success-
  required gameplay policy was selected.

- Completed the eleven remaining gameplay-camera binding perspectives using
  authenticated native Egui/UI APIs and the actual camera producer. Confirmed
  primary-context required components and inactive-camera fallback selection;
  retained existing typed marker owners. Recorded uniqueness, existing-context,
  replacement and first-pass scheduling limits without claiming a live camera or
  format failure. The authored one-camera test was read, not executed.

- Traced current terrain, grass, scatter and tree pending/presented marker
  producers. Rejected the specific disabled-grass stall suspicion because its
  producer explicitly settles that stage. Located the optional-ground versus
  required-ground query difference and linked current scatter caller edges to
  existing understory/litter publication records without transferring older
  child verdicts. Scene population and native failure/readiness guarantees
  remain open.

## What remains

- Finish the required audit across all 13 lenses, the 20 migration families,
  connected source dependencies, exact canonical mappings, and current-source
  changes. The implementation gate is still incomplete.

- Resolve or explicitly disposition the remaining type ownership, bounds, units,
  frames, authority, construction, and policy questions. Preserve existing
  evidence without treating a narrow source result as whole-family completion.

- Implement the clean final bespoke types and APIs, update generated bindings
  through the prescribed generator where required, and synchronize the owning
  documentation.

- Complete independent review, remediation, required formatting and lint checks,
  applicable product tests, and isolated observable acceptance.

## Coverage and verification limits

The newest source-revision batch contains 31 changed paths and 403 lens/path
checks; 51 of those checks are closed. These are audit coverage obligations, not
bug counts. Earlier revision evidence remains separately qualified.

The canonical queue contains 3848 entries. Entries and views overlap, so this is
not a count of unique bugs or a meaningful completion percentage.

No product files have been changed in this continuation.

Source and checkpoint verification only. No product builds, tests, lints,
generators, or runtime acceptance have run.

No subagents have been used in this continuation. Checkpoint verification is not
independent review.

## Evidence and report updates

Selected source revision: `ff9c1d7d0ea168b7e14eedc5e44879ee7b84fb04`.

- [Checkpoint 534 publication record](CHECKPOINT-534.json)
- [Saved report for checkpoint 534](PROGRESS-534.md)

PROGRESS.md is the latest working report. It is updated during ongoing work and
whenever a new checkpoint is published. PROGRESS-N.md preserves the report saved
for checkpoint N; later work updates the latest report without rewriting that
snapshot.

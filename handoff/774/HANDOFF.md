# Continue the bespoke-type migration in PR #774

Adler: Bruno asked for this handoff so your developer or agent can continue
without reading the original conversation. Work in the original chat is paused.
The source, unfinished changes, detailed audit records and ownership map are now
available from the remote branches below. Nothing has been merged.

The objective is to finish the connected migrations in
[PR #774](https://github.com/adventure-simulator-group/fabelgeist/pull/774),
part of the repository-wide migration in
[issue #726](https://github.com/adventure-simulator-group/fabelgeist/issues/726).
A bespoke type should retain its meaning through producers, application state,
operations and consumers. Extract its primitive representation at the actual
format, storage, framework or numerical operation that needs it. A private helper
or a future serialization step does not establish such a boundary.

**The twenty-family implementation plan is still required. Finish and integrate
P16 and P17, including their shared dependencies. The audits, reviews and CI are
additional acceptance gates; they do not replace unfinished implementation.**
Do not merge this PR or declare the overall repository migration complete.

## Retrieve the work

The PR branch remains `codex/shared-rig-identity-726` at
`6533e444fa37552927070cc8d8190907fd3b413e`. Main was checked against the remote at
`3a50b708381ad00c116445c2c999a49c3943088b` when this handoff was prepared.
The PR description contains older validation prose: use the exact results below
and recheck GitHub before resuming.

| Recovery branch | Exact revision | Preserved work |
| --- | --- | --- |
| `codex/774-handoff-20261021-integration` | `36c97923da8ee84d0ab8994e7cfedf6a0cdea641` | Four uncommitted quality/attachment-capacity edits on top of the current PR head |
| `codex/774-handoff-20261021-p16` | `039329518e264f19206cf5b0b98d177433f40c17` | All 159 changed paths, including seven previously untracked files, from unfinished P16 work; parent `a120390947c89b703dc74c4255a13c035d054c50` |
| `codex/774-handoff-20261021-p17` | `90a422a453cd990ebc7f0ecf2ad5ed3e2d57d696` | The 32-file report increment, measured against parent `18360adba0fb43add88c75dfbd2f45d7e71d8ace` |
| `codex/774-handoff-20261021-rig-quality` | `cbbbca08c89a8768926799371f54cf6057312e5e` | Eight quality-fix paths relative to current PR head |
| `codex/774-handoff-20261021-records` | This handoff branch | Transfer documents and detailed records; never merge this branch |

These are **recovery snapshots, not validated integration proposals**. P16's
older branch includes stale versions of newer PR contracts. Do not merge or
cherry-pick its entire snapshot onto the PR. Inspect coherent function-level
changes against current source. P17's geometry prerequisite was already
integrated independently as `c912fd7a7c7346c187399644e32919dde8aa2bbd`; do not
replay its old parent history.

Fetch the refs and establish separate worktrees before editing:

```sh
git fetch origin \
  codex/shared-rig-identity-726 \
  codex/774-handoff-20261021-integration \
  codex/774-handoff-20261021-p16 \
  codex/774-handoff-20261021-p17 \
  codex/774-handoff-20261021-rig-quality \
  codex/774-handoff-20261021-records

git worktree add --detach ../774-current origin/codex/shared-rig-identity-726
git worktree add --detach ../774-p16 origin/codex/774-handoff-20261021-p16
git worktree add --detach ../774-p17 origin/codex/774-handoff-20261021-p17
git worktree add --detach ../774-records origin/codex/774-handoff-20261021-records
unzip ../774-records/handoff/774/records.zip -d ../774-records-extracted
```

Create an appropriately named `codex/` implementation branch from the current PR
revision when ready. Refresh `main` and compare subsequent changes; do not reset
or discard this progress to start over. `snapshots.json` and
`FILE-INVENTORY.tsv` beside this document enumerate every preserved source path.

## Authoritative records

The archive is about 6 MB compressed and contains 475 transfer files, including
all sixteen audit/review skill documents. Extract it outside the implementation
source tree, or into ignored `target/` output. It includes:

- `records/execution-ledger.json`: the authoritative twenty-family status,
  remaining gaps, source/integrated commits, shared dependencies, validation
  heads and next actions. `CURRENT.md` is a derived view, not another authority.
- `records/parallel-ownership/*-claims.json`: exact file/function repair scopes.
  Historical worker labels identify responsibilities, not live developers.
- `records/final-acceptance-20261021/audit-ledger.json`: authoritative audit
  discovery, dispositions, repairs and validation status.
- `records/final-acceptance-20261021/wave-01/`: source-backed producer and owner
  inventories, traces, findings, current supplementary packets and range errata.
- `EXPORT-MANIFEST.json`: export contents and original/export fingerprints.
  Nonessential local paths and personal email data were removed. `SNAPSHOTS/`
  names refer to the remote source revisions above. `CARGO_REGISTRY/` means the
  pinned dependency source. Source fingerprints are unchanged; metadata-file
  fingerprints differ where paths were normalized. Embedded historical receipt
  hashes refer to the original bytes listed in this manifest.

Some source references lead to historical local validation logs not exported
here. Do not infer a result from a missing log or a historical verdict. Hosted
CI links below are the public validation evidence. Old helper scripts contain
normalized location placeholders and must be reconfigured before use.

The original unrelated recovery stash remains untouched locally and is not
published. It is not needed to recover any of the four increments above. The
primary checkout also retains four unrelated `indexing_slicing = "warn"`
manifest edits, which have not been validated and are deliberately excluded.
Do not restore the old stash wholesale or import unfinished scanner changes.

## Implementation plan and current status

“Recorded complete” below means an earlier bounded implementation was integrated;
it does not mean the current source passed all audits or CI.

| Family | Meaning | State at pause |
| --- | --- | --- |
| P01 | Movement-audio deterministic seeds | Historical accepted implementation preserved |
| P02 | Autoresolve battle-time coordinates | Historical accepted implementation preserved |
| P03 | Maneuver-duration producers | Historical accepted implementation preserved |
| P04 | Playback rates, phase bounds and event budgets | Historical accepted implementation preserved |
| P05 | Capture elapsed time through jitter analysis | Historical accepted implementation preserved |
| P06 | Terrain support elevation through inverse kinematics | Reopened, additional repairs required |
| P07 | Inverse-kinematics angular caps | Historical accepted implementation preserved |
| P08 | Camera measurements and retained history | Reopened, additional repairs required |
| P09 | Joint columns and skin ordinals from producers | Reopened, additional repairs required |
| P10 | Parent topology without the `-1` sentinel | Recorded complete and integrated |
| P11 | Momentum Human Rig joint/model columns | Reopened, additional repairs required |
| P12 | Garment offsets and named upright anchors | Reopened, additional repairs required |
| P13 | Shader candidate and cursor states | Recorded complete and integrated |
| P14 | Catalogue-bound equipment placement | Reopened, additional repairs required |
| P15 | Equipment reservations and replay chronology | Recorded complete and integrated |
| P16 | Melee geometry and targeting range policy | Mandatory unfinished implementation; isolated snapshot preserved |
| P17 | Report measurements, applicability and absence | Mandatory unfinished implementation; isolated increment preserved |
| P18 | Causal report quantities at `f64` precision | Recorded complete and integrated |
| P19 | Bounded influence for each foot | Recorded complete and integrated |
| P20 | Continuation coordinate, tangent, readiness and progress | Recorded complete and integrated |

The six historical accepted families are P01–P05 and P07. Preserve their
implementation and evidence, reconcile subsequent shared changes and revalidate
the final source. Their historical CI does not validate current source.

### P16: finish connected physical and error contracts

Start from the preserved delta, retaining newer PR contracts during integration.
The precise remaining gaps and whole-function claims are in the execution ledger.
They include:

- Finish Core contact geometry, lunge timing, power, autoresolve movement,
  defense and evidence, then the actual tactical authority, bot, client motor,
  targeting and physical inventory consumers. Arrival/recovery impossibility
  must remain explicit enum states; never reintroduce infinity or numeric codes.
- Finish authored weapon/configuration producers and their Core build-time
  dependencies, evaluated weapon/holder properties, storage admission,
  `StrategicEquipment` projections and all receiving combat traits. Distinguish
  catalogue-selected reach from per-instance grip-to-tip reach.
- Complete player admission, projection, spawn batches and every affected
  `PlayerBody` implementation. A checked human mass must not be peeled into raw
  retained `Limbs` state, but human-only admission bounds must not silently
  reject supported nonhuman bodies or confuse biological and equipped mass.
- Complete the canonical strategic operation error graph through mutually
  calling database helpers and the real reducer boundary. SpacetimeDB accepts
  `Result<(), E>` with `E: Display`; it does not require internal errors to be
  strings. Preserve original concrete causes until that boundary.
- Reconcile old fixture calls and expected error comparisons with the final API.
  Known stale sites include social `map_err(String::as_str)`, string comparisons,
  surgery `.err().as_deref()` and projectile helpers returning `Result<(), String>`.
  Adapt existing assertions only; add no tests or fixture helpers.

Two integration traps require explicit reconciliation:

1. The P16 snapshot has older equipment/lib declarations that would lose current
   P15 sequence/revision/reservation behavior. Preserve the final PR versions.
2. Its physical generation fixtures predate `RigJointTable`, `PackedRigSkin` and
   catalogue placement handles. Recover physical changes into the current rig
   contracts rather than restoring old vectors or fit-key records.

### P17: finish upstream observations before report acceptance

The preserved report commit improves many output contracts but does not finish
its producer paths. Remaining work includes:

- Complete the shared motor's optional tick evidence and its report consumers.
- Retain the authoritative animation sample tick in repeated-evaluation
  snapshots/comparisons, so clock-only mutation cannot escape detection.
- Retain canonical `LeadFoot` instead of `Option<bool>`/`Vec<bool>` in contact
  metrics, keeping original ordering, ties and minimum-observation policy.
- Trace required support, contact, knee-facing, jitter and displacement evidence
  from the earliest actual producer. Missing observations cannot become measured
  zeros or passing checks. Real zero and negative measurements remain valid.
- Repair iteration callbacks and retained failure readback using the actual
  Bevy callback contract; a fallible observer cannot simply be registered as an
  unsupported system return type. Keep structured failure through the native
  command's owning error and final report.

Finish shared functions once for all affected families. This does not by itself
complete the other families that depend on them.

## Canonical ownership and shared repairs

Consult the exact claim files before editing. Keep one coordinator responsible
for contract decisions, integration and final validation. Multiple isolated
workers are authorized, grouped by overlapping functions and dependencies.
Transfer or combine whole functions explicitly when ownership overlaps.

- `fabelgeist-geometry` owns portable `SignedLength`, `PositiveLength`, area,
  cuboid dimensions and their errors. Building geometry re-exports them and owns
  framed positions, displacements, directions and architectural conversions.
  Tactical Core owns scene and floor/collision/gate datums. Consume these final
  contracts from the earlier geometry work; do not create equivalent wrappers.
- `fabelgeist-rig` owns joint names, ordinals, ordered collections and packed
  skin data. An ordinal alone does not prove membership in a particular rig.
- `fabelgeist-determinism::Seed` and `StreamId` are canonical after the producer
  seed migration. Preserve byte framing, domain strings, draw order and existing
  hashes. New random output is different from a seed erased into retained state.
- Shared elapsed, speed, fraction, clock, schedule, rights and material owners
  already exist. Preserve signed versus nonnegative, zero versus absence,
  geographic versus abstract coordinates and authored versus realized duration.
- Physical/strategic/camera repair scopes belong together in the historical
  `camera_targeting` claims. Native report, observer, lower material/action and
  pure training scopes are in `native_reports`; rig/garment/catalogue and Web
  presentation loops are in `rig_garments`; scene/support/build/compiler,
  audio/effects and remaining pure-food scopes are in `root`.

Recently reconciled whole-function repairs are already recorded, not implemented:

- The Web observation loop owns `strategic_root_fragment`,
  `render_quest_location`, `get_active_party_members`, `soap_rest_preview` and
  travel provision forecast helpers. Failed query/body reads must stay distinct
  from successful empty results, optional absence and privacy filtering.
- Native terminal submission owns the concrete SDK failure through the callback,
  mailbox, result, lifecycle and retry consumers. An SDK transport/internal
  failure differs from the reducer's actual rejected-string wire payload.
  Preserve retry/backoff and standalone behavior.
- The pure life/curriculum helpers were combined with their existing training
  owner. Food quantities, growth, concentration, flavor potency and doneness
  have distinct units and ranges: cooking progress reaches 2, and historical
  training uses arbitrary elapsed minutes rather than quarter-hour plans.
- Evidence follow-up selection can currently panic on an overflowing weight sum
  that catalogue validation permits. Propagate the existing structured sampling
  error through the complete helper/recipient graph; preserve genuine empty
  selection. An existing overflow fixture already covers the sampler.
- Garment layer propagation remains open from millimetres through retained
  layer offsets/workspace records and the gorget support/bib/collar graph.

## Exact point to resume discovery

No source fixes were made during the current exhaustive initial inventory.
The first two lenses have extensive source-backed records, with findings.
Neither has overall clearance: the second lens reached more paths, reopening the
first lens's coverage; its newly reached roles then require an owner-fit delta.
The other eleven initial lenses remain incomplete. Do not claim thirteen waves
or comprehensive clearance from the volume of records.

Use `wave-01/producer-supplement-after-owner/` under the extracted records:

- The main camera supplementary queue has **670/716** obligations accounted
  with findings, **46 remaining**. Its `camera/progress.json` lists the exact
  indices. First unfinished cell is 78, the mission bootstrap request graph.
- `rig/traces.jsonl#RIG-PPS-038` contains the eighteen Web request source proofs.
  Their final reconciliation with Native equipment/projection receipt 027 was
  not saved as receipt 039 at pause. Do not award automatic coverage.
- `root/pure-food-life-partial.json#ROOT-PPS-PURE-FOOD-LIFE-PARTIAL-054` supplies
  food/life source and recipients; `RIG-PPS-040` supplies foraging/sampling.
  `NATIVE-PS-PURE8-SUBSET-PARTIAL-028` supplies whole survival/water/time source
  reads, but its recipient closure is still partial. Its first unread recipient
  is the clock mutation around `adventuresim-stdb-module/src/simulation.rs:307`,
  reached from `real_micros_for_official_minutes` and the epoch-shift helper.
  Finish this bounded packet, reconcile all eight obligations, then explicitly
  consume it into the main queue.
- The final five world-authority obligations are in
  `camera-world-authority-final-receiver-packet-assignment.json`: indices
  193/268/299/300/551. Reuse the exact prior partial traces listed there; do not
  restart their whole audit or assume their unread bodies are covered.
- Remaining main-queue cells include mission/roster fixtures, retained client
  player recipients, strategic place, reputation/morale, source metadata and
  native schema projections. Account for each with an actual trace/disposition.

The frozen owner inventory has 2,279 source variants. A supplementary metadata
index recognizes both `ranges` and `read_ranges`, and normalizes source-ID and
frozen-archive path aliases. Its last verification covered 1,315 referenced
source variants with no fingerprint mismatch; 184 path variants were outside
the frozen set, including 41 byte-identical counterparts in another context.
Those counts are **not audit coverage**. Reconcile exact material roles and
range qualifications, extend the scope, then perform the owner-fit delta before
admission. Preserve historical source variants and acceptance evidence.

Finish the full initial inventory for these thirteen lenses before repairs:
producer propagation; existing owners; admission; invariant operations;
aggregate states; type ownership; redundant authority; behavior preservation;
enum shapes; derivability; type granularity; raw strings; raw primitives.
The exported skills define each lens. Reload them in the receiving environment,
record exact connected repairs/dependencies/acceptance, and reconcile overlaps.
Then implement the repairs and mandatory families, repeat complete audit waves
until clear, and run independent Code Review, Rule Review and Comment Review.
Address findings and repeat affected work until all evidence matches one final
pushed revision. Search hits, old verdicts and CI alone do not establish clearance.

## Validation and restrictions

[Latest hosted run 39576450512](https://github.com/adventure-simulator-group/fabelgeist/actions/runs/39576450512)
failed at exact PR head `6533e444fa37552927070cc8d8190907fd3b413e`.
Formatting, schema/generated bindings, Bevy-version and FBX checks passed.
Semantic/growth quality failed before the selected behavioral matrix, so those
later tests were skipped. Recorded growth failures include clothing generation,
export, database equipment, armor probe, client equipment and MHR token code,
plus stale baseline entries. Diagnose the earliest remaining failed contract;
do not label skipped behavior as tested or simply increase ceilings.

[Historical accepted run 39466774426](https://github.com/adventure-simulator-group/fabelgeist/actions/runs/39466774426)
validated `82cb2a3acccc852bba6a7ef9c706c337ba9fe765`, not current source.
The old Creator segmentation fault was observed on both old main/candidate
revisions; it is not evidence of a current-main baseline failure. Calibrated
renderer/live asset acceptance remains unavailable; report that limit honestly.

Use the complete command matrix and revision/artifact/nonempty-result guards in
`.github/workflows/rust-quality.yml` at the current PR revision. It includes
formatting, semantic/growth policy, generated bindings, deterministic draws,
WebAssembly Clippy, four native Clippy configurations, geometry/rig/constraint,
Creator, animation, equipment, camera, report and terrain consumer checks. The
expanded selection expects 41 nonempty result cells and all sixteen Creator
command vectors; preserve the additional Core/startup evidence and the seven
existing ignored cases rather than changing test population.

Only request hosted CI after a migration family or repair batch is presumed
source-complete across known producers, unchanged callers and receiving bodies.
If compilation reveals an unknown gap, finish its connected repair before
requesting another run. Perform formatting/size cleanup after source correctness.
The receiving agent should use the existing workflow, for example:

```sh
gh workflow run rust-quality.yml \
  --repo adventure-simulator-group/fabelgeist \
  --ref codex/shared-rig-identity-726
```

**Do not run that command during recovery or incomplete discovery.** These
recovery branches were pushed without updating the PR head or requesting CI.

Bruno's remaining constraints continue to apply:

- Use existing tests only. Add no test, case, fixture helper or assertion.
  Existing API adaptations retain original inputs, ordering and oracle intent.
- Use hosted project validation. Do not run local builds, project checks,
  generators, scanners or runtime acceptance merely to obtain compiler feedback.
- Keep tactical tick state transient in Bevy; do not persist tactical positions,
  HP, damage or enemies in SpacetimeDB.
- Implement the clean final API/schema. No compatibility shims, dual paths,
  legacy fallbacks, duplicate bespoke owners or primitive forwarding adapters.
- Read root/scoped `AGENTS.md`. Order Rust imports before module declarations,
  then constants/statics/aliases, structs, enums, traits, implementations,
  functions public before private, and the test module last.
- Update the family ledger after integrations, ownership transfers and CI
  outcomes. Source-complete, integrated, validated and reopened are distinct.
- Do not merge PRs, alter unrelated work or mark issue #726 complete.

A successful handoff continuation ends with P16/P17 and all connected repairs
integrated, one final pushed PR revision, all thirteen lenses clear and all three
reviews and required hosted CI accepted at that same revision. Report any real
blocker or coverage limit instead of silently reducing the scope.

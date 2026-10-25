# Policy and renderer checkpoint Rule Review

Reviewed Git snapshot `73b6e3a2ddf319dec749e3b1ff4a25e99b9c6612`, tree
`c8f7763247eff6ae47b79973ef3be2051289b248`, against parent
`197eb8982c554203ac8652178cabe611d3652896`; main remains pinned at
`990430b77a9f63548ffcb642074185f61ddb3322`.

All thirteen changed files were read completely, including every line of the
renderer 1–2210, complete host LOD producer, policy/count/quality/preset modules,
dataset, draws, frame state/projection, deformation and readback. Every source
read and hash uses pinned Git bytes. The worktree later advanced independently;
its changes are recorded in JSON and do not receive this snapshot's verdict.
Supporting owners and unchanged consumers were read at the declared granularity.
The pre-repair ledger was consulted after source assessment, not substituted
for body coverage or root's independent thirteen-lens inventory.

## Findings

No new owned behavioral defect was established in this receiving increment.
Three inherited connected rule findings remain:

- **73B-ORDER-01, P3:** lod.rs:79–124 places StrandLevel's impl before
  LodSettings and HairLods structs, and LodSettings' Default impl before
  HairLods. Order existing structs before impls with their attached docs
  (`crates/AGENTS.md:90–94`), preserving the agreed #780 producer bodies.
- **73B-ORDER-02, P3:** dataset.rs:60–100 places Shot's impl before Sample's
  struct. Move the existing declaration/docs before impls under the same
  rule. No randomization, serialization, density-image or ZIP/NumPy repair is
  implicated.
- **73B-PEER-RULE-01, P3:** lod.rs:148 still calls counts.last().unwrap() in
  HairLods::build. Its initial nonempty vector proves no missing element,
  but the explicit production unwrap ban still applies
  (`crates/AGENTS.md:126–127`). This is a required #780 producer cleanup;
  coordinate the whole function rather than independently patching it or
  inventing an impossible missing-element error.

These are existing source rule debts, not new numerical/runtime regressions.
The earlier head-column/triangle bounds finding197-BOUNDS-01 remains required:
set_head and cover_head still accept the shorter normal/cover relationship.
The native count/layout/head-source handoff must close it with the actual
producer owner; this policy migration does not claim that closure.

## Contracts and repairs verified from source

**Canonical units and roles.** Existing math Ratio/Metres and hair StrandCount
remain the shared owners. New host level counts and ordinals describe distinct
roles; neither duplicates a native record count or claims collection membership.
Count.select uses checked subtraction and clamps only when a collection is
nonempty. Actual host collections remain checked at use. PixelSpan preserves
fractional, signed and non-finite numerical-policy semantics; effective render
admission is a separate contract. AccumulationFrames owns zero/unrendered state,
minimum-one target, saturating increment and widened next-sample weight.
ShadowMapSize retains the prior sixteen-texel minimum; hardware upper limits
remain actual native provider checks, not an invented policy bound.

**Selection and aggregates.** Band, SimBand and Selection replace semantic
positional tuples. HairShading and ShadowQuality own the reflectance choice and
optional deep-opacity size payload, removing independent enable/size states.
Representation::Empty explicitly represents a groom with no prepared geometry.
PolicyResult/LodPolicyError is feature-free and reports empty bands, absent
requested geometry families and strands without levels. Band selection safely
handles zero bands; all subsequent direct indices follow the established band
or position bounds. Nonempty noncomparable-height fallback, authored preset
values, finite interpolation, shadow selection and empty simulation fallback
retain their numerical behavior. General Ratio correctly permits width scales
above one; no unsupported universal [0,1] admission was imposed.

**Actual frame admission and cache identity.** Globals::for_frame constructs
native uniform lanes and validates the actual f32 upload, including moving head
pose. RenderedView validates effective pixels, minimum strand pixels, all
selected strand widths and blend amount before cache/publication mutation.
This prevents 197-POLICY-01's NaN equality/reset loop without banning raw numerical
leaf values prematurely. The error carries a typed RenderInputPart through
HairGpuError::Backend. The original valid cache remains available if an invalid
new input is rejected before publication.

The cache retains the observed pixel height, selected main/blend, shading and
shadow selection/size. converged:1640–1659 recomputes the current policy from
that observation and compares all effective choices, repairing 197-POLICY-02.
Scene replacement methods call one invalidate_view operation. last now owns
invalidation; removed generation/last_generation cannot wrap or disagree.
Simulation movement retains its separate temporal filter/deformation obligation.
History's closed First/Second states project to the native pair only at indexing.

**Draw/native adapters.** DrawPlan, StrandDraw and CardDraw give fields their
roles. DrawRole projects only the three fixed native uniform/group slots;
CardCoverage owns outgoing/incoming payloads and preserves the shader's exact
interval endpoints. The private planner produces at most two card draws, which
proves its fixed two-slot accesses. draw_gpu clamps against an already native
u32 resident population before narrowing. Typed level ordinals, StrandCount,
Ratio and PixelSpan cross only at collection, numerical-kernel or native POD
ports. Deformation's typed index still preserves the private one-to-one level
relationship and per-level Pending/Current obligations. Native layout/capacity
admission of the resident populations themselves remains #780's required work.

**Module and API ownership.** Counts, policy, quality, presets, draw planning,
frame admission and native projection/uniform construction are focused owners,
not forwarding or compatibility modules. The new canonical definitions are
reexported directly. Sample::policy now consumes AccumulationFrames and retains
the minimum-one behavior. Existing convenience select returns the main policy
form; it is not a compatibility shim for the removed tuple return contract.
Relevant fallible APIs retain PolicyResult or HairGpuError Result distinctions.
Native structs, vector lane packing and authored numerical coefficients remain
explicit mathematical/native adapters; no display strings drive behavior.
Derives preserve existing representation/admission, with no new wire migration.

**Native causes and feature limits.** The renderer still uses the agreed bare
Device/Queue device-handled provider policy, now documented on its constructor
and headless producer. It does not acquire a fake owned context or synchronous
validation certificate. Typed callback, mapping, poll, readback and allocation
sources remain actual causes. Host/device/head association is not inferred
from Arc storage. SDK cfg/thread/error/wasm evidence remains pinned source,
with unsupported browser atomics limits and browser callback delivery limits
unchanged. No platform or global-device-health verification is implied.

## Rules, population and incomplete integration

197-GROWTH-01 is repaired: resize:1527–1629 has 103 source lines, matching its
ceiling, with coherent history reset extracted to invalidate_view. Render's
physical span is 391 lines, already below its existing 478 meaningful-line
ceiling; constructor source shrinks. Every new module is below 500 lines and
all new functions are below 100 by direct source inspection. The old closure
allow is removed without an invented lint expectation. No ceiling or broad
suppression was raised. Required hosted baseline reductions still must be
applied and checked; this source review did not execute enforcement.

Both renderer tests, the existing validate helper and five assertions are
byte-identical. The dataset's existing test module is also byte-identical.
All five native POD struct bodies and three shader files are parent-identical.
No tests, cases, fixtures, test helpers, assertions or doctests were added or
deleted. The changed dataset policy API does not touch ZIP/NumPy ownership.

Completeness remains **incomplete**, independently of receiving correctness.
HairLods::counts and GpuLods::counts still populate the new leaves with raw
usize fields. Creator Form producers, ratio sliders, level consumers and
same_level's now-distinct ordinal arms still need the explicitly owned #780
adaptations. Native strand/card/simulation/atlas count and layout producers,
empty-level/bounds errors, derived card/count authority, Surface/head Arc and
GPU-cover association, application state/caller propagation and canonical lock
integration remain required. No shim, duplicate owner or silent deferral is
accepted. The new mesh manifest edge and GPU feature activation require the
agreed canonical lock update; its current absence is not a successful build.

No source edit, project scanner/check/build/test/generator/runtime, push or CI
was performed. Only source/Git inspection and ignored report accounting were
used. All final independent reviews, complete thirteen-lens audits, hosted
baseline reductions and required CI must pass the same final pushed revision.
This is not renderer-family, whole-PR, compilation, hosted or final clearance.

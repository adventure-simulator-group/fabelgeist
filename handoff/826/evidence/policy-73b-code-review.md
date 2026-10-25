# Independent Code Review: policy checkpoint 73b

Head: `73b6e3a2ddf319dec749e3b1ff4a25e99b9c6612`.
Tree: `c8f7763247eff6ae47b79973ef3be2051289b248`.
Parent: `197eb8982c554203ac8652178cabe611d3652896`.
Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.

This is an intermediate source review with three actionable findings. It is
not renderer-family, compilation, thirteen-lens, whole-PR or final clearance.

All 13 changed paths were read fully from the immutable Git snapshot,
including all 2210 renderer lines. The JSON records 29 source identities and
eight exact SDK hashes. Partial connected-source coverage is identified
explicitly rather than treated as complete producer acceptance.

## [P1] Reexport the representation moved into the private policy module

`crates/fabelgeist-hair/src/lod.rs:35-36`.

Representation moved out of lod.rs into private mod policy but is omitted from
pub use policy. lib.rs:182 and render/mod.rs:48 still import
lod::Representation, as do the new draws/frame_state/deformation modules. The
public path is unresolved independently of the known counts producer mismatch.

Minimum connected repair: Add canonical Representation to the policy reexport;
retain one enum owner and original public path.

Acceptance: Every received Representation import resolves through lod without
duplicate types or compatibility aliases. Source-only finding; no compiler run.

## [P2] Initialize temporal history when a single-frame view starts accumulating

`crates/fabelgeist-hair/src/render/mod.rs:1963-2009`.

Render a static view with accumulate=1: accumulating=false, so only resolved is
displayed while frames becomes1 and history is untouched. Raise only target to8:
target is absent from RenderedView and same stays true, so next_weight=1/2
blends the second frame against unwritten (or stale) history. Reported sample2
represents only one valid accumulated image, darkening the mean. The same gap
exists in the parent and remains in this affected policy/history family.

RenderedView fields frame_state.rs8-20 exclude history validity; same/reset
mod1718-1723, accumulating branch1963-2009, frames increment2045; post.wgsl
accumulate_fs loads before and mix(before,now,weight).

Minimum connected repair: Track validity/source of the actual history or
reset/promote correctly when entering temporal accumulation from a resolved-only
output. Preserve ordinary increased targets on an already valid mean and moving
temporal behavior.

Acceptance: A finite still view rendered once without accumulation and then
requested at multiple samples has a valid mean and accurate sample count;
stale/unwritten history is never weighted as a prior sample. Existing tests
only; no new cases requested.

## [P2] Preserve the coarser resident prefix at a capped strand transition

`crates/fabelgeist-hair/src/render/draws.rs:71-77`.

Two adjacent strand bands both select logical count1000; higher-point finer
resident is capped at100 and coarser resident at200 by upload_level storage
limit. At finer lower edge amount=1, new population(outgoing) returns100, so
incoming coarse draw is0..100. Immediately below edge its own coarse band
draws0..200. Parent split logical prefix1000 first then clamped each draw to its
own resident capacity, reaching0..200 continuously. Normal unequal point-count
levels can trigger this new discontinuity.

draws.rs population closure43-47 invokes draw_gpu, whose count is clamped at141;
strands-to-strands branch72 uses that outgoing count for incoming endpoint.
render1937 clamps again per target; upload_level1133-1140 allows different
capacities by point count.

Minimum connected repair: Define the shared logical transition prefix and bound
each native draw against its own resident population without letting the finer
capacity truncate the coarser endpoint. Coordinate canonical producer counts;
preserve native-width safety and existing valid numerical transitions.

Acceptance: At amount1 the incoming coarser prefix agrees with the coarser band
immediately below, including unequal capacities; no usize-to-u32 wrap, inverted
range or out-of-resident draw.

## Source contracts revalidated

All13 changed paths read completely from immutable Git73b, including
entire2210-line renderer, both existing renderer tests/helper/5assertions and
complete dataset including its existing fixture. Every new
policy/count/quality/catalog and frame/draw module full read; no automatic prior
verdict reuse.

Policy formulas preserve preset thresholds, shares, simulation
bands/hysteresis7/6 and5/6, interpolation max1 denominator, count
rounding/clamping, width total/count.max1, transition amount clamp and shadow
selection. PixelSpan and Ratio do not invent stronger numerical bounds.
EmptyBands typed error replaces panic; strands without levels typed rejection;
no geometry selects Empty; absent requested family with other family present
errors; requested ordinal clamps only within nonempty owner collection.

New semantic counts/indices are host usize owners with checked empty select;
native populations remain required canonical780 integration. ShadowMapSize
preserves min16 and device-owned upper limits; ShadowQuality Baked owns no
inactive size. HairShading faithfully selects native mode. AccumulationFrames
preserves zero unrendered, target max1, saturation and u64 next weight.
Sample::policy carries admitted sample count and keeps cinematic defaults.

Scene replacement invalidates last/frame/history directly at groom/GPU
groom/dynamics bind/clear/head/resize; redundant wrapping generations are
removed. Unchanged dimensions preserve output/mean. Resize plus no frame is
rejected by read_rgba. Moving marks each deformation level Pending and head
pose; only visible main/enabledshadow/blend strand obligations consumed, hidden
pending levels survive. Duplicate levels deform once. Native source/layout
population contracts still unresolved.

Globals::for_frame validates every actual f32 upload lane including full moving
head matrix before head is replaced by IDENTITY for still-view equality. Cache-
only pixels/min_pixels/representation/shadow/blend widths and blend amount are
separately finite checked before state publication or frame submission. NaN
eye/material/light/derived matrix or moving pose therefore returns actual
RenderInputError through anyhow-backed HairGpuError; no nonconverging NaN
equality loop remains on these received inputs. Finite Halton jitter occurs
after admission but width/height>=1 and n1..64 bound it.

converged now recomputes selection at cached observation and compares
representation/blend/minwidth/shading/enabledshadow/active resolution plus
moving and target count. Policy names and inactive shadow settings need not
force redraw. render Globals equality includes actual effective native inputs,
while moving pose intentionally does not restart the still mean. P73-C2 remains
separate history validity defect.

Draw role mapping remains main0/shadow1/blend2. Card intervals [amount,2)
and[-1,amount) preserve shader random comparisons; at most main+oneblend gives
two card slots. draw_gpu clamps before narrowing to avoid usize wrap and absent
level returnsNone. The P73-C3 endpoint regression is due to applying that clamp
before shared-prefix construction, not a request to restore unchecked narrowing.

Complete shader numerical source read: Globals/Draw/Card layouts, strand
point/vertex addressing and stochastic coverage, lighting/shadow/deep-
opacity/head/card paths, deformation folded addressing and post mean/tonemap.
Shader text exact parent and no new tests/oracles. This is source reasoning
only, not numerical GPU validation or proof all arbitrary finite shader inputs
produce finite results.

Full renderer native constructor/layout/pipeline/atlas/shadow/lighting/upload/dy
namics/head/cover/target/render/readback paths reread. Native validation remains
explicitly device handled, with exact logical device/queue/resource
preconditions. No observed native acceptance or transactional publication
guarantee inferred. Headless synchronous pollster browser limitation unchanged.

Readback layout remains checked four-byte RGBA/BGRA row/padding, typed
allocation/narrowing, supported formats and BGRA reorder after unmap. Private
output has COPY_SRC and nonzero extent from resize. Host image allocated before
mappedview. NativeWait +actual mappingcallback and browser canonical ready-first
poll/yield remain; actualSDK/poll/channel/map sources retained. Private native
staging drop cleanup and browser private handle lifetime use exact prior197 SDK
proofs, no newly imposed arbitrary-alias policy.

## Required dependencies

D780-producers — REQUIRED incomplete: HairLods::counts lod.rs219-225
deliberately unchanged and still supplies rawusize to bespoke fields;
GpuLods::counts gpu/lods.rs66-73 has same required mismatch. Actual native
strand/card/sim/atlas checked byte/population/layout producer integration,
sourceassociation, folded addressing and creator/publiccaller adaptation must
land canonically with780 before source completion. These are not new receiver
findings and are not accepted deferrals.

D780-head-source — REQUIRED incomplete: Current set_gpu_groom only replaces
cover onSome, and set_head replaces surface/cover; canonical associated
head/groom source identity and column validity remain open. Numeric wrappers
cannot establish this relation.

D780-lock-caller — REQUIRED incomplete: Cargo.lock must record futures-channel
and new fabelgeist-mesh dependency; named selected entry/API/count/caller
migrations and retained774/M06/F19 paths remain required. No complete compile
claim.

empty-resource-layout — REQUIRED incomplete: Policy Empty semantics alone do not
repair native empty strand upload infos slice and native empty/storage binding
rules; producer/layout receiving family remains open. Same malformed public
resource arithmetic/indexing scopes remain required, not silently cleared.

## Existing fixtures and validation limits

The complete existing renderer test module is byte-identical to the parent:
two tests, one existing validation helper and five assertions. The dataset
test module and all three WGSL shader files are also byte-identical. No new
test, fixture, assertion, helper or doctest was written.

No source edits, project commands/build/check/tests/scanners/generators/runtime,
new tests/cases/helpers/assertions/doctests, push or CI. Only Git/source reading
and ignored report accounting performed.

Fresh FULL all13 changed paths and full receiving
renderer/math/shaders/error/material/shared callback sources. Connected producer
cards/surface/sim/gpuLods sources have explicit partial coverage; pending
producer integration is not cleared. Root13-lens inventory/review is separate.

Three findings remain: one P1 missing reexport, two P2 temporal history and
unequal-capacity transition defects. Parent numerical formulas and fixtures
compared by source; no executable numerical evidence.

Final all13 audit waves and Code/Rule/Comment reviews plus hosted checks must
pass on exact final pushed SHA; this report grants no family/wholePR/final
clearance.

The machine-readable report contains per-file Git blob IDs, SHA-256 hashes,
line counts, complete versus partial coverage, and the exact inherited SDK
proof identities. SDK ports were revalidated by identity here; this policy
delta does not claim a fresh whole-SDK read.

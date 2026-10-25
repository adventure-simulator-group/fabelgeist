# Independent Code Review: frame checkpointb874

Head: `b874e611826c27085da897903cacfc31998d0210`.
Tree: `271b944ed1843f9e858c65da4947e854743e98ed`.
Comparison: `6b22364249ae1503fe070dd7b5ce64b8e6e0e25d`.
Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.

No new actionable receiving defect was found. Temporal identity, sample count,
history, movement and deduplicated deformation obligations publish only after
normal queue submission returns. This is metadata publication; it does not
certify native acceptance or GPU completion.

All three changed files were read fully. The JSON records 57 exact source
identities and coverage, affected unchanged contracts, shader/fixture
preservation and required incomplete dependencies. No family, compilation,
thirteen-lens, PR or final clearance is given.

## Source contracts reviewed

All3 changed files freshly read completely from Gitb874, including all2103
renderer lines; complete frame_state and deformation modules. Actual affected
readback, explicit scene invalidation, convergence and scalar sample formulas
source-traced. Full post/deform shader source freshly read and every shader
exactparent; no source execution used.

FrameProgress is local candidate state: prepare_frame immutably copies published
count/history/moving and selected finite view. Same invalidation predicate as6b
resets only candidate samples/history when view differs or accumulation lacks
current history. prepare_frame never mutates last, frames, history or movement.
Render keeps candidate through native recording and stores its new ping-pong
slot locally. No retained competing authority or extra generation introduced.

Earlyreturn is_complete requires frames>=wanted and !moving. Wanted is
target()>=1; any changed view or missing accumulation history resets
candidateframes0, so cannot incorrectly earlyreturn while discarding a
meaningful new identity. An actual completed sameview leaves prior published
state unchanged and returns same frames/primitives. Initial/reset/scene-replaced
state cannot claim a frame without submission.

Successful numeric behavior matches6b: candidate pre-frame count drives same
n=count%64+1 Halton values, same shader uniforms and draw plans, same
previous().next() temporal target, same u64-based next_weight and moving
max0.35. Resolved-only static source still bypasses accumulation; resolved-
only→multi target resets to0/Empty and usesweight1. Alreadycurrent mean extends
normally. publish_frame increments saturating samplecount once after normal
submission, publishes exact candidate history/view, clearsmovement, and
FrameInfo reads that same newcount.

RecordedDeformations stores at most3 actual strand ordinals in fixed original
main/shadow/blend slots. It skips absent/non-strand views, checks pending, and
deduplicates contains(Some(level)) BEFORE recording, preserving original first-
occurrence order and one deformation per selected level even though retained
state is no longer mutated to deduplicate. A missing pending/dynamic level does
not enter record. Hidden pending levels remain untouched. After normal submit,
publisher marks only recorded indices Current via checkedget_mut.

Commands finish and queue.submit both occur before publish_deformations and
publish_frame. Any synchronous error return or unwind before normal return drops
candidates/recorded ordinals without consuming published temporal identity,
samplecount, history, movement or pending obligations. Post-submit publishers
are allocation-free simple checked state assignments/count saturation. Exclusive
&mut render prevents safe concurrent mutation of its source state between
prepare/record/publish. No assumption about user-held native buffer aliases is
added.

The transaction concerns receiving metadata, not rollback of native
queuewrites/resource allocations or proof commands were accepted/completed.
Shadow map resources may be replaced and uniform queue writes issued before
submit; these are outside the stated deferred temporal/deformation metadata.
Agreed device-handled native policy remains: a custom native handler returning
normally after rejection can still let submit return, and asynchronous/later
failure is not observed here. No GPUcompletionWait or new context helper is
introduced.

Readback still requires published lastSome and frames>0; a first attempt that
fails before normal submit cannot advertise pixels. Previously published frame
identity survives a failed candidate; resize/scene replacement still explicitly
invalidates. converged still compares published effective policy and !moving;
failed movement attempt therefore remains nonconverged. Missing
targets/nonfinite admission remains typed and occurs before candidate
publication.

Shader semantics are unchanged: post mix(before,now,weight),
tonemap/exposure/sRGB, deform absolute rest/head/weightedguide and native
addressing remain exactparent. Candidate publication does not claim arbitrary
finite shader-output validity or repair raw native layout/count/source
admissions. Constructor docs now accurately distinguish returned host atlas
allocation/admission errors from configured native pipeline/resource/queue error
handling (Atlas6B-C1).

## Required dependencies

D780-atlas-propagation — REQUIRED incomplete: Current root LodSettings still has
raw atlas_variants/atlas_size; HairLods::build and GpuGroomer::lods_against
still call old paint signature/use raw atlas cache key; creator HairGeometry
atlas_levels still reads old public fields and recomputes mip sizes. These exact
old callers are intentionally pending accepted780 handoff integration. Require
canonical AtlasSettings, Seed, fallible results, actual mip metadata and cache
key propagation; no source-complete claim or accepted deferral.

D780-native-count-source — REQUIRED incomplete: Checked Strand/Card native
layouts/count/byte/binding/empty semantics, GpuLods::device actual producer
capture, sealed source associations, native simulation
id/weight/rest/body/source receiving and allcanonical producer/caller paths
remain active root780 work. Peer CardMesh/CardSettings/CardVertex methods remain
untouched. This atlas review does not accept their old raw arithmetic.

D780-imports-lock-final — REQUIRED incomplete: Canonical Surface/head/count and
typed/fallible compute descriptor imports, creator/public Result API
adaptations, new Color dependency Cargo.lock, retained774/M06/F19 and other
connected families must complete before push or hosted gates. Final
all13/reviews/CI same pushed SHA still required.

## Validation limits

No source edits, project commands/build/check/tests/scanners/generators/runtime,
new tests/cases/helpers/assertions/doctests, push or CI. Only
source/Git/provenance reports.

All3 changed files full fresh read; unchanged readback/post/deform/sample scalar
owner full fresh read. Other unchanged source/peer/native proof carried by exact
identity with prior partial-coverage limits retained. This is not a targeted
verdict inherited without source revalidation.

No new actionable receiving finding found. Successful formulas/dedup/earlyreturn
and deferred metadata publication source-traced; no executable failure
injection/GPU/browser or numeric validation evidence claimed.

Normal queue return is the publication boundary, not observed native acceptance,
GPU completion, global GPUhealth or transactional native queue/resource
rollback. Those agreed native-policy limits remain explicit.

Canonical Strand/Card/Sim source/native layouts/WorkgroupGrid producer
propagation, old atlas/creator APIs, headsource imports/GpuLods device proof,
manifests/locks and allconnected families still REQUIRED incomplete. No whole-
family/sourcecompile/13-lens/PR/final clearance; all3 final reviews and hosted
checks must repeat same final pushed SHA.

The renderer fixture remains exact: two tests, one existing helper and five
assertions. Card fixture and all three WGSL files are byte-identical to6b.
No new tests, cases, helpers or assertions were written; no project gate ran.

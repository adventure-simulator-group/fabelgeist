# Independent Code Review: head checkpoint04

Head: `04a68c493d152c5250e17bedd09d33497ab67bf3`.
Tree: `a3084113b320d4452229aee325c5f057c20e9f4b`.
Comparison: `d9c3521f30f52d3437cfef2850203c9df3192b0e`.
Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.

No new actionable receiving defect was found. The prior logical-write defect
is fixed. Required producer imports, native layout/source/device admission and
caller integration remain incomplete. This report gives no standalone
compilation, family, thirteen-lens, PR or final clearance.

All five changed files were read fully. The JSON records 45 root source
identities and 9 canonical peer source identities, with exact fresh-versus-
carried coverage and unchanged fixture/shader bytes.

Canonical head proof: `bcbc1129b76d378917d8134fbe93a5687606f129`.
Canonical typed/fallible builder proof: `bad382cb2704652c64f6289aa07633f4682a9d57`.

## Contracts reviewed

D9-C1 fixed: Buffer::write now invokes write_at(START,data); checked end and
logical bound reject oversize prefix uploads before queue write. Valid in-range
bytes preserve exact native upload path, alignment/usage/device error handling
and source. Empty upload receives same native behavior, no new early zero skip
or duplicate bounds owner. Prefix pure logical/physical admission, read padded
mapping, binding nonempty checks and actual callback cleanup remain source-
identical and revalidated.

HeadDraw owns actual Arc<Surface>, checked canonical layout, native mesh and
coverage as one Option payload. GpuMesh Clone only clones native Arcs/count;
cloned HeadDraw retains same immutable source/proof. Public Surface vectors can
mutate before capture, so fresh SurfaceRenderLayout::try_from precedes native
upload. Retained Arc ownership prevents safe mutable access to captured vectors
while installed. No copied HeadSurface or generation-token authority remains.

Canonical peer bcbc SurfaceRenderLayout validates positions/normals exact
pairing, u32 vertex/triangle counts, checked3*triangle native corner count and
every vertex index. It does not certify UVs, CDF, finite/physical geometry or
arbitrary native resource limits. Receiver uses its vertices/corners directly at
host allocation/native draw. SurfaceRenderError is actual #[from]/source of
RenderSourceError::Layout, retained through HairGpuError::Backend.

HeadDraw::new creates exact paired position+normal vertices and original index
sequence. Empty logical head uses one dummy24byte vertex, one dummy4byte index
and one8byte bare cover, but native indexed draw count remains0. Positive head
uses full checked corner count, exact normal pairing and same finite numerical
vertex data; no topology fallback. Malformed layout returns before native
candidates and keeps previous head/mean unchanged; successful source replacement
invalidates actual view/history.

CPU cover uses ScalpCover::at on the exact captured paired source. That function
emits one cover pair per paired point/normal, so validated triangle membership
proves all smoothing indices fit. Three original ordered smoothing passes
preserve sums/division and valid cover outputs; empty result gets one native
dummy cover and zero logical draw. Geometry/sampling internals of ScalpCover are
not newly certified.

GPU coverage admission: actual cover device equality, Arc::ptr_eq exact surface
capture and expected vertex_count*size_of([f32;2]) logical bytes all checked.
Multiplication fits u64 because native vertex count is u32. Physical allocation
must fit expected or minimum8. Read-only canonical GpuHeadCover
source/buffer/device getters fit receiver calls. Absent coverage for installed
head makes a fresh bare cover, preventing prior groom cover leakage. Present
cover without installed head is typed MissingHead. Same-count different Arc is
rejected; count is not substituted for identity.

set_gpu_groom first checks actual agreed GpuLods::device even for coverNone;
then head/source/cover extent admission, then constructs candidate level
groups/cards/atlas/lighting/bounds before changing
dynamics/head_matrix/levels/cards/counts/atlas/head/view. All typed admission
errors preserve previous scene. Native creation/write/submit errors still follow
explicitly agreed provider policy, so no observed native acceptance or rollback
guarantee is inferred. Full canonical GpuLods device/layout/source
implementation remains required integration, not source-complete.

Head shadow and main draws now both consume complete same HeadDraw: mesh
buffers/count and cover cannot diverge as separate Options. Empty draw remains0,
main pass cover always present for installed head. Bounds/framing/physical
geometry and all other strand/card/atlas/simulation producer contracts remain
open; new head topology/source check does not silently certify those paths.

TextureBinaryOp receiving adaptation fits immutable canonical builder bad382cb:
output binding returns StorageFormatResult<String> and ? preserves actual
StorageFormatError through anyhow; no shader/cache publication occurs on output
storage admission failure. Descriptor base_type returns ResourceBaseType,
recursively scalar for vector texture formats; direct enum matches preserve
float/int load conversion and output store behavior. Named main pipeline
selection remains canonical. ComputeSourceCache retains
source/opcode/device/validationpolicy/descriptor recipe and success-only
publication; failures retain original causes and do not cache unsuccessful
candidates.

Current complete renderer mod reread, including constructor/native
pipeline/group/shadow/atlas intake, dynamics/rest/deformation, targets, finite
uniform admission, convergence policies, logical prefix transition clamp,
temporal Empty/Current behavior, readback-facing state, headless helper and
exact existing fixture. Unchanged helper/shader/module proofs are revalidated by
exact source hashes with fresh-versus-carried coverage explicit. Policy
formulas, finite moving-pose guard, effective quality selection and valid
running mean/draw transitions preserve previous repaired contracts; no new
actionable receiver finding found.

## Required dependencies

D780-canonical-head-import — REQUIRED incomplete integration: bcbc canonical
GpuHeadCover/SurfaceRenderLayout/mesh count files and exports are not yet
imported into root04. GpuLods::device actual context capture is accepted but not
implemented at this checkpoint. Receiver contracts fit the pinned head-source
APIs; this is not compilation evidence or proof incomplete producer fields are
sealed.

D780-builder-import — REQUIRED incomplete integration: Root04 compute/mod.rs
still has old String output binding/base_type_str while texture_ops consumes
agreed fallible/typed canonical API. Independent immutable source proof found at
bad382cb; canonical complete family import is mandatory. Deliberate pending
receiving dependency, not a new mismatched implementation finding or accepted
broken build.

D780-native-family — REQUIRED incomplete: Actual checked strand/card/atlas byte
and native draw counts, host/GPU HairLods/GpuLods counts, physical mip bounds,
native empty strand/binding semantics and source-associated simulation
ids/weights/rest/head/device producer graph remain incomplete. CPU set_groom
fallible atomic publication also remains owned pending work. Do not infer head
topology admission closes these.

D780-callers-lock-retained — REQUIRED incomplete: All creator and public caller
API adaptation (set_head Arc->Result and set_gpu_groom Result included),
canonical producer imports/manifest-lock/cache/application migrations and
retained774/M06/F19 paths must complete before any source-complete/push/hosted
gate request. Same final pushed SHA reviews/CI still required.

## Validation limits

No source edits, local project
execution/build/check/tests/scanners/generators/runtime, new
tests/cases/helpers/assertions/doctests, push or CI. Only source/Git/provenance
and ignored reports.

All5 changed files freshly read FULL, including complete2135-line renderer
mod and new189-line head module; exact line count in per-file accounting.
Current policy/count/quality/frame-state/uniform/buffer read/map/prefix/bind and
source-cache sources freshly read. Other unchanged
helpers/manifests/shaders/prior partial producers carried by exact identity, not
misrepresented as fresh whole-body reads.

No new actionable root receiving defect found in reviewed scope, and D9 logical-
write gap repaired. This does not clear incomplete canonical imports/device
implementation/native layout/source/count/caller/lock families or guarantee
stand-alone root compilation.

Canonical head/layout/count proof pinned bcbc1129b; canonical typed/fallible
builder proof independently pinned bad382cb from repository Git history. This
report does not accept unrelated full peer implementation.

Root full thirteen-lens waves plus independent Code/Rule/Comment reviews and
required hosted checks must pass the same final pushed revision; no
family/wholePR/final clearance here.

Existing renderer fixtures are exact: two tests, one existing helper and
five assertions. Dataset and buffer fixtures, all three shader files and
unchanged numerical helper sources preserve exact comparison identities. No
test or local project gate ran, and no source file was edited by this review.

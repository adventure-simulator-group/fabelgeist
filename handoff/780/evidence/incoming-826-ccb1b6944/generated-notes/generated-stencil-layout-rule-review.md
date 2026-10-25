# Generated Stencil/layout independent Rule Review

Exact frozen working manifest
`target/pr826-audit/generated-working-stencil-layout-snapshot/manifest.json`,
SHA-256 `fcdac606eb126ad4bca6407958373d671e8d1f10e3fe1831764f509affa6fcf0`, base
`919ba9b9415c06981ff5dc01d21666f49407eb3e`. All50 whole owning sources (6162
lines) were freshly read. No owning body is unread. The separate complete
thirteen-lens inventory is persisted before any requested successor repair. JSON
records every hash, source range and qualified closure receipt.

**GSL-C1, P2 — valid projected texture coverage is rejected.**
neighborhood/layout.rs cover_output compares all three neighborhood axes for
Neighbors3D even when input is Texture2d. It admits input gridZ=1, then rejects
output Texture3d with matchingXY and depth>1. Native fetch.rs deliberately
projects center and every load to.xy; old supported behavior repeats the same
plane for each outputZ. Require physical input coverage only for actual sampled
axes, while separately admitting the full generated signed coordinate/neighbor
arithmetic. This is an unintended behavior/type-granularity restriction under
crates/AGENTS.md:28–57,80–86, independently confirmed and deduplicated with Code
Review. Do not remove actual signed-range checks merely to accept the
projection.

**GS-STATE1, P3 — private roles retain dead duplicate metadata.**
StreamParameterRole::Offset(ShaderParameter) is only matched Offset(_), with its
name/type already fixed to offset/u32. Stencil Center(ShaderParameter) is only
matched Center(_); neighborhood owns center loading/type. Gather
Other(ShaderParameter) is only ignored. All three already retain their exact
native declaration in SourceParameter. Eq/Hash alone does not justify another
name/type authority. Use unit cases for these three private roles, retaining
declarations and every actually consumed ShaderParameter elsewhere. This follows
crates/AGENTS.md:31–35,58–71 and the requested granularity/redundant-state
lenses. It is current rule simplification debt, not an invented mutation or
runtime corruption claim.

The source repairs otherwise establish coherent owned paths. SourceParameter
privately couples declaration and roles; Map selects primary/resource/uniform
bindings from actual call context. Gather/Scatter/Stream classify exact resource
outer forms before base-type cleaning, retain authored order and correctly
load/pass unsigned values displaced by a later explicit index. Stencil admits
exact Neighbors1D..4D, rejects unsupported dimension-shaped reserved forms, and
leaves unrelated Custom native names to shader admission. Malformed
scalar/vector envelopes no longer slice panic. Regex is a protocol extractor;
Naga still owns full native syntax and identifier/type validity. Existing
unsupported extra resource/type forms are not silently claimed as working WGSL.

GeneratedProgramError is the shared Compute algorithm-policy owner. Missing
function/return, malformed/duplicate parameters, missing Scatter value, wrong
Stream offset, multiple/unsupported neighborhood/center and
coordinate/resource/fluid admission cases retain typed kinds and actual roles.
Regex, coordinate, texture-value, storage, native compilation/device/progress
causes remain separate. NativeOperatorSignatureError stays in GPU for
fixed-arity helper admission; no reverse dependency or algorithm-specific GPU
error is introduced. Thus GW-R1 is source-resolved in the50-file snapshot. Mixed
lower errors still legitimately use anyhow carriers; there is no new prose
classification branch.

Private NeighborhoodLayout validates positive axes, checked population, signed
Clamp/Wrap neighbor expressions, Buffer signed flattening, unsigned divisors,
actual reflected stride and record capacity before dispatch. Buffer2d/3d flat
outputs now unflatten row/plane coordinates. Default64 is one shared policy for
emitter and host admission. Texture projections and fourth-axis handling remain
intentionally distinct. Empty output skips only sampling arithmetic; it does not
promise every native resource binding succeeds. GSL-C1 keeps this physical
family open.

ResourcePipeline reflects the actual selected compute entry, WorkgroupShape and
storage stride from the same pipeline source; actual resource axes drive
WorkgroupGrid. Canonical cache identity is real Device/source/validation policy
plus full named recipes, with sorted secondary resource records and publication
only on successful builds. Current Root validation scopes preserve
native/progress causes; finalization/submission remain context-owned.
Browser/disabled routing uses configured device handling. No Ok return proves
completion or global device health. Older peer native reference bodies differ
and must not overwrite these Root contracts.

All active source files are under500 raw lines and inspected function bodies
under100 raw lines; Stencil was split by actual
source/binding/grid/emission/admission responsibilities. No new production
unwrap/expect, unsafe or suppression is in the50 owning files. Definition
PartialEq intentionally excludes caches; canonical sorting, semantic validators
and conditional exact diagnostics justify retained manual implementations. No
derive bypass/serde/default checked certificate is added. The native kernel
coefficients remain documented arbitrary grid/time-frame scalars; they must not
be relabeled as unrelated physical units.

Existing fixture population is preserved. The extracted entire Stencil fixture
diff is formatting plus existing API adaptations, without new
cases/helpers/assertions/WGSL source. Current native fluid source literal proofs
use45/50/47 tokens after excluding the one diagnostic moved to typed Display;
prior46/51/48 counts are historical. These are source preservation receipts, not
emitted-program execution. The existing relevant baseline reductions and final
hosted checker export remain pending; no checker or checker clone was run.

Completeness remains partial. Root lib/prelude does not yet register all new
owners; canonical Buffer count/stride and
source/producer/caller/lock/Reduce/Scan/extraction/MC/DC/DistanceField/JFA/noise
graph remains #780-owned unreceived work. The38 immutable prior reference
receipts are carried only after exact hash comparison, and only13 match current
Root; they do not become complete current graph coverage. No forwarding
compatibility API, duplicate types or stale native helper replacement is an
acceptable integration remedy. Accepted comment wording and both findings
require a fresh full successor wave.

No source edits/copy, project
checks/scanner/build/lint/test/generator/runtime/native source
generation/push/CI were performed. No new test population was authored. This is
a complete bounded discovery and Rule Review, not compilation, whole-family
correctness, global all13 or final pushed-SHA clearance.

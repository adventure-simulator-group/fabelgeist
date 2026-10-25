# Independent successor Rule Review of PR #826

Rule Review remains **incomplete**. The prior converter, static allocation,
sampler-record and declaration-order findings are source-resolved. Two owned
families remain actionable: typed fast-path rejection propagation and faithful
reflection/admission of supported interfaces. Required #780 and retained #774
families also remain unresolved. No source-complete or final-clearance verdict
is given.

## Exact snapshot and review constraints

- HEAD: `219d407be5b89e6a910244ba3362326088603701`.
- Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.
- All 71 changed file contents were independently compared with their exact
  HEAD blobs; every content matched. The working tree was clean.
- Every file fingerprint in `ledger.json.successor_wave.files` matched, and
  that wave's revision matched HEAD.
- Independent digest of the sorted changed-path/SHA-256 records:
  `a83650b3f095bb2f3a562d5e37758462293ba07621b29db2a9228330c9293d06`.

The Rule Review skill and bespoke-type reference govern this review together
with root `AGENTS.md` and `crates/AGENTS.md`. Governing instructions are unchanged
from pinned main. No deeper guide applies to the changed GPU/compute packages.
Current source, the complete main-to-HEAD scope, receiving paths and pinned SDK
definitions were inspected; previous verdicts were not used as clearance.

Ownership follows the later direct human core-GPU transfer and coordinator
reconciliation recorded in `774-core-ownership-reconciliation.md` and
`ledger.json`, including public #774 comment `6360654609`. That assignment
covers core parser/shader/reflection/pipeline/pass and the agreed fast/batch
native receiving subset. It does not transfer retained hair, fitting, armor,
body, metal, engraving or other claimed whole functions. The two restored
hair adapters are accounted for as unresolved integration dependencies.

This review changed no source and performed no project checks, builds, tests,
scanners, generators, runtime work, push, hosted CI or external posts. Only
this ignored report was written. No tests, cases, helpers or assertions were
added. Hashing and git inspection establish source identity only. Native/Wasm
compilation and required hosted baseline reductions remain pending.

## Owned actionable findings

### S1 — P2: fast preparation still erases closed rejection categories

Locations: `kernel/fast.rs:168-175` and `kernel/fast/uniforms.rs:14-20,52-58`.
Classification: existing connected migration gap, not a new success regression.

The static assembler now returns canonical `ComputePassError` categories for
missing and wrong-kind buffers. FastPath instead combines them into one generic
anyhow message. The uniform writer similarly encodes missing/unsupported
uniform states in formatted text and uses a Debug discriminant to describe the
unsupported value. A caller traversing the same kernel receiving graph cannot
inspect the canonical rejection category on the fast branch.

Rule: `crates/AGENTS.md:23-35,36-44,69-71` requires semantic classification
owners and propagation through internal APIs. This is not a prose-branching
claim: no branch on that text was found. The gap is that the authoritative
producer still emits generic prose for closed states while the actual owner
already exists beside the static consumer.

Reuse `ComputePassError`/`ComputeBindingKind` for missing and wrong Buffer
admission. Give missing/unsupported uniform rejection a stable, appropriate
owning classification, without a duplicate parallel compute error family.
Preserve the fast writer's existing strict rejection and all successful bytes.
Do not silently change the static writer's existing optional/default-zero
policy as part of this classification repair.

Acceptance: both fast buffer cases and the distinct missing/unsupported uniform
cases retain inspectable typed categories through preparation/batch callers;
existing parameter ordering and successful packing are preserved. The root
ledger's successor S1 now records this same finding; this review independently
confirmed it from the current source.

### S2 — P2: the reflected interface admits shapes its retained record loses

Locations: `compute/pipeline/reflection.rs:96-168,170-248`,
`shader.rs:8-51`, `bind_group/assembly.rs:48-55`, static and fast uniform writers.
Classification: existing owned admission/aggregate and native-descriptor gaps.

There are two concrete uniform counterexamples:

- Two distinct uniform variables in one bind group overwrite the singular
  `uniform_binding` and `uniform_buffer_size` while appending members from both
  declarations. Native layout entries retain both bindings, but assembly emits
  only the final one. The admitted aggregate cannot represent its own layout.
- A direct uniform scalar such as `var<uniform> amount: f32` takes the
  non-Struct branch, which records its size but no member or variable name.
  Both writers therefore produce zero bytes regardless of an `amount`
  parameter. This is a silent value loss before the native boundary.

Naga's host-shareability/sized flags admit supported scalar uniforms; these
counterexamples are not manufactured malformed WGSL. They were established
from source, not executed. Several handle distinctions are also discarded:
comparison samplers, multisampling, storage read-write/atomic access and
external textures/binding arrays. Their detailed dispositions are below.

Rule: `crates/AGENTS.md:31-35,45-57,58-71` requires faithful semantic records,
controlled admission and canonical native roles. Returning a correctly typed
native error after constructing a descriptor for the wrong role does not prove
the interface was represented correctly.

Close this as one coherent interface admission family. Retain every supported
declaration's actual native facts and host-value shape, or reject unsupported
shapes with a typed admission error before creating a misleading interface.
Do not silently coerce, discard, default a supplied value, or add a second
reflection language. Preserve successful supported flat uniforms, ordinary
sampled/storage bindings, sparse groups and their packing bytes. Provider
feature/limit failures remain actual native rejections with concrete causes.

Acceptance: every materially distinct reflected form has an explicit faithful
representation or typed unsupported disposition, shared by static/fast
consumers. Supported input that previously worked remains supported. The
following inventory is source discovery, not acceptance evidence or authority
to add tests or expand language support.

## Full materially distinct reflection inventory

| Declaration / native role | Current trace and disposition |
| --- | --- |
| One flat uniform struct, supported f32/u32 scalar fields and f32 vectors | `uniform` stores member names/offsets/sizes; static packing uses Number→f32, Unsigned→u32 and Vec2/3/4; fast packing supports the same numeric/vector cases. This is the previously working path. Preserve the static missing-value zero default and fast strict missing-value policy. Native offsets/spans remain Naga's ABI. |
| Square f32 matrices / Transform | Static writer handles Mat2/3/4 and Transform; fast writer handles Mat4 and rejects the others. Preserve these existing distinctions and actual byte layouts. A fallback suggestion in an error is not an automatic fallback implementation. |
| Direct scalar/vector/matrix uniform variable | Non-Struct branch stores size only, loses the variable name, and gives both writers no members. Supplied data becomes zeros. Represent its native value shape and name, or reject explicitly. |
| Multiple uniform variables within one group | Singular binding/span overwrite plus accumulated members conflicts with multiple native layout entries. Model independent bound blocks or reject the second declaration with a typed error. Uniforms in separate groups are a different case and already retain separate BindGroupReflection records. |
| Fixed arrays, nested structs, non-square matrices, integer vectors, signed scalar and narrower/wider scalar/matrix types | Naga can admit valid host-shareable forms, subject to alignment/capabilities. The current record stores only member size/name/offset, while PassParameter/math writers do not represent every native shape. For example Number always emits f32 bits, Vec* emits f32 components, and Mat* is square f32. Unsupported declarations can silently remain zero, be mispacked, or be rejected later only by fast value classification. Define the supported packing shapes explicitly; typed rejection is sufficient for shapes the API cannot carry. Do not reinterpret unsigned bits as a new signed value contract or assume all matrices share the supported ABI. |
| Invalid uniform forms | Runtime-sized uniform arrays lack SIZED; bool uniform values lack HOST_SHAREABLE; empty/invalid structs and layout violations are rejected by Naga validation in ComputeShader::new before reflection. Preserve ShaderError::Validation and its actual cause. Do not add fictitious downstream states for forms already rejected there. |
| Single storage buffer, read versus writable | AddressSpace::Storage maps read_only from STORE; valid read and read-write buffers retain their existing SDK role. Runtime-sized storage arrays are legitimate buffer contents, not binding arrays. Buffer::binding_resource enforces logical prefix versus native allocation. Keep this path. Naga rejects illegal storage-address-space write-only declarations. |
| Ordinary sampler | `comparison=false` currently maps Filtering, which is a valid existing SDK policy. Pinned wgpu-core resource.rs:2996-3014 allows both nearest/non-filtering and linear ordinary sampler instances for this layout. Do not invent a rejection for nearest samplers or infer comparison from filter choice. |
| Comparison sampler | `comparison=true` is ignored and mapped Filtering. The SDK requires Comparison, as shown by validation.rs:493-497 and its native derivation at 667-672. Preserve comparison role, or explicitly reject it if unsupported. Sampler's public native port can carry a genuine comparison sampler; its basic constructor's defaults are not proof the shape is impossible. |
| Single-sampled float/int/uint/depth images | Dimension and arrayed mapping to D1/D2/D2Array/D3/Cube/CubeArray is retained. Uint/Sint/Depth sample roles are preserved; Float always becomes filterable=true. Existing ordinary filtered float inputs are a working path to preserve. Valid provider support and actual view dimension remain separate receiving checks. |
| Float texture used only by textureLoad | Forced filterable=true overstates the shader's requirement. A valid R32Float view on a device without FLOAT32_FILTERABLE can be rejected despite requiring no filtering. Pinned wgpu-core validation.rs:685-694 derives filterability from actual sampler use. Preserve the native distinction using shader facts/appropriate admission, without changing the working filtered path. The fixed converter's explicit unfilterable layout already preserves textureLoad behavior separately. |
| Sampled/depth multisampled image | Both native `multi` flags are discarded; the descriptor always says multisampled=false. Pinned validation.rs:550-576 compares image classes including this flag, and its derivation at 681-702 retains it. Valid multisampled declarations therefore receive the wrong layout. Preserve the flag or reject unsupported resource shapes explicitly. Whole texture constructors currently use sample_count=1; this does not justify relabeling a multisampled shader declaration. |
| Storage write-only image | STORE maps WriteOnly and retains exact format/dimension. This is the previously working path. Preserve it. |
| Storage read-only image | LOAD maps ReadOnly correctly. ReadOnly itself requires provider/feature support. Lack of that feature is a legitimate native rejection, not a reflection defect. Imported devices may supply support. |
| Storage read-write or atomic image | Any STORE access maps WriteOnly, losing LOAD and ATOMIC. Native SDK access has distinct ReadWrite/Atomic roles; pinned validation.rs:583-608 and 704-716 preserve and compare them. On a provider supporting the declaration, the current reflected layout is wrong. Represent the actual access set or return typed unsupported admission; do not mask provider feature failures or claim generic support. |
| External texture | ImageClass::External falls into the ordinary sampled-float texture descriptor. SDK expects BindingType::ExternalTexture, as pinned validation.rs:724 shows. Current PassParameter has no external-texture resource payload. Explicit unsupported admission is a coherent bounded choice; silently reclassifying it as a TextureView is not. Native/browser availability is separately provider-dependent. |
| Resource binding arrays | Handle BindingArray is ignored; uniform/storage binding arrays are treated as singular buffers, with count=None. Naga IR documents distinct TextureViewArray/SamplerArray/BufferArray resources and binding counts. PassParameter has no corresponding collection shape. These are not ordinary buffer arrays. Represent the collection with its actual SDK count/resource or reject explicitly; do not silently omit/classify as a single binding. Required device features remain native admission constraints. |
| Other bound handle kinds | AccelerationStructure/RayQuery can be represented in validated Naga capability configurations but are not represented by current PassParameter/reflection assembly. The Handle `_ => {}` drops them. Record explicit unsupported admission where these forms are reachable, preserving provider validation for unsupported capabilities. No new ray/mesh rendering subsystem is requested. |
| Unbound globals and malformed roles | Private/workgroup globals and entry-point inputs are not ordinary resource bindings and are correctly skipped when no binding exists. Naga rejects resource binding collisions used by an entry point, invalid bound address-space/type forms, D3 image arrays and invalid external texture geometry. Keep those failures at ShaderError validation, rather than broadening the reflection model for invalid inputs. |
| Device limits, formats, stages and native capabilities | Sparse group indices are retained and group limit rejection is typed. Native layout/pipeline creation retains actual provider errors under the observing policy. Unsupported format feature, unsupported stage/operation or device-limit failure is legitimate provider rejection when the descriptor faithfully represents the declaration. A descriptor role changed by this reflector must first be corrected or explicitly rejected; native rejection alone cannot stand in for faithful reflection. |

Pinned SDK paths above are under the local registry's `wgpu-core-29.0.4/src/`
and `wgpu-types-29.0.4/src/{binding,texture}.rs`. Naga shape/validation evidence
is in `naga-29.0.4/src/ir/mod.rs:816-856,1004-1051`,
`valid/interface.rs:897-1063,1119,1522` and `valid/type.rs`'s host-shareable,
sized, struct and image branches. `ComputeShader::new` validates before native
shader construction and ComputePipeline preparation, using the existing all-
capabilities Naga policy. A later native device may support fewer capabilities;
this report does not conflate Naga validity with device acceptance.

## Source-backed family review receipts

| Family | Verified current source trace and remaining limit |
| --- | --- |
| Whole texture/view ownership | The three whole constructors admit immutable-to-callers device/format/usage/extent/cache/native records; removal operations cannot replace or relabel resources. Default and both independent removals preserve all four supported availability states. Matching-format selectors return cached Arc before allocation/context use; alternate selectors require allocation and actual SDK device equality. SelectedTextureView owns effective format/dimension/device facts once. Whole, generic, cube-face and array consumers use it. The old public mutation counterexample is closed. |
| Cache format relationship | Constructors add STORAGE_BINDING only for non-sRGB formats. Their initial view uses admitted format directly. Therefore generic cached views need one admitted format; removed view_format and dead constructor branches were redundant. Alternate storage-bound sRGB requests still use the linear descriptor format and retain the original request in errors. Cube face/array layers and dimensions retain their distinct native meanings. |
| Direct texture recipients | Read/write/clear/blit, generic view operations, cube copies/face/array render entry and DistanceField copy preflight check availability/device before native use while keeping prior zero/cache precedence. Converter now retains its own device, checks both caches before device admission, uses canonical shader/selected-view construction, keeps the unfilterable float textureLoad layout and captures available native pipeline/record/submit causes. Prior R1 is source-resolved. |
| Static binding and uniform allocation | Public create_bind_groups now accepts only the immutable admitted ComputePipeline and PassParameters; raw authored reflection/layout injection is removed, with no compatibility overload. The one assembler retains sparse indexed groups and logical buffer prefixes. Padded uniform extent is checked against allocation and binding limits before host packing. Unmapped allocation and upload have balanced native scopes, with `?` applied after popping them. It reuses Buffer/Definition/Upload/byte leaves. Prior R2 is source-resolved for its allocation cause contract; S2 remains the separate interface-shape admission defect. |
| Sampler/type ownership and item order | SamplerBinding is one named name/binding record, produced in reflection and consumed by static assembly and the fast eligibility predicate. The earlier tuple is gone; no duplicate index/name owner was added. TextureViewResult now precedes the enum/impls, and shape types precede conversion impls. Prior R3/R4 are source-resolved. Sampler role fidelity remains S2. |
| Shader/pipeline cause graph | ShaderError retains actual parse/validation/translation/native causes plus diagnostics. Immutable ComputeShader admits source/module/device together. ComputePipeline verifies device, retains immutable reflection, and caches full entry-point outcomes, with observed Result distinguished from device-handled creation. Rejected outcomes are Arc-shared errors, not usable cache hits. Compilation state changes after successful creation; `compiled` requires admitted state/device. Standard generic conversions preserve the chain. |
| Native/wasm provenance | SDK Device equality proxies its native handle (`wgpu-29.0.4/src/api/device.rs:26`). Error has native/conditional Send+Sync source bounds at 774-825; build.rs:7-11 enables those on native or fragile non-atomic Wasm. Cargo.toml enables that feature for the supported browser target. Naga error types implement Error. No unsafe auto-trait replacement or format/reparse adapter was introduced. Source evidence does not prove compilation, atomics-enabled browser configurations or runtime behavior. |
| Fast/batch receiving state | FastPath retains its creating device and canonical byte lengths; logical prefix binding reuses BufferBindingError. UniformArena uses checked typed offsets and per-batch slots, Prepared checks native u32 narrowing, and CompiledLayout/record/copy retain available native causes. Empty batch grids still skip admission/count increments, while standalone dispatch retains its parameter admission. Copies retain logical bounds and callbacks retain submission ordering. S1 and the interface facts in S2 remain actionable. |
| Canonical launch/count owners | WorkgroupGrid/Shape/InvocationCount are single canonical production definitions; zero grid occupancy is explicit and Shape uses nonzero admission. RecordedDispatchCount increments only after successful nonempty recording and uses the agreed transparent Serialize. Native axes/offsets are extracted at actual SDK calls. Upstream raw producers are still #780 dependencies, not proof of a complete migration. |

The old build_compute_pipeline forwarding API and raw-axis ComputePass
overloads are absent. New modules own real assembly, packing, allocation,
compilation or arena responsibilities. No compatibility alias, weakened lint,
raised baseline ceiling or new broad registry exception was introduced.
Raw WGSL, entry-point/format tokens, open binding names and Naga offsets/spans
are native/text boundaries. Closed rejection categories still needing ownership
are S1/S2; no display-prose branch was found. No additional wrapper is required
merely for intrinsically clear native dimensions or fixed ABI arithmetic.

## Required owner dependencies, not accepted deferrals

- #780 still owns Kernel workgroup/count producers, source/device caches and
  generic definitions, including texture_ops and the mesh cache caller. Current
  Kernel stores public raw workgroup axes, groups_for/groups_folded use raw
  item counts, source caches lack device identity and lock unwraps remain.
  Producer-to-grid/byte call-site conversions in algorithms, sort/prefix sum,
  hair/physics copies and mesh do not establish producer completeness under
  `crates/AGENTS.md:36-42`. Integrate one exact canonical lineage, preserving
  folded dispatch and post-submit callbacks.
- #774 retains claimed hair/fitting/body/armor/metal/engraving and connected
  runtime equipment cause families. Current `guides.rs:242` and
  `grow.rs:597` still pass arrays to a WorkgroupGrid receiver after restoration
  of the two unagreed edits. This is a concrete current source API dependency;
  no compiler run is needed to see the mismatched signatures. No unilateral
  adapter is authorized here. Guide source is unchanged from main, and the
  grow diff touches only StrandSet::settle's byte adapter. Remaining retained
  recipients, including the handover's equipment M06 cause path, require the
  agreed whole-function implementation and exact integration revision.

These dependencies prevent source completion and push/CI permission under the
user's sequence. Lower typed native errors do not repair upstream cache
identity, raw counts, retained recipient signatures or downstream cause erasure.

## Coverage and pending validation

All 71 changed paths and affected semantic graph were accounted for. Complete
new/changed core owners and receiving responsibilities were reread, including
converter, uniform allocation/packing, shader/parser/errors, reflection,
pipeline/cache/errors, static/fast assembly, arena/Prepared/batch and all texture
view admissions. Algorithm dispatch/texture/error ports in advect, broadcast,
DistanceField/JFA, divergence, dual_contouring, gather, gradient, map,
marching_cubes, matmul, prefix_sum, reduce, reshape, scan, scatter, sort,
stencil, stereo, stream and transpose retain their prior native/error ordering.
BVH, mesh, hair and physics recipients were inspected at their actual changed
ports and identified pending ownership boundaries. Unrelated numerical kernels
and retained claimant bodies were not independently repaired or granted a pass.

Existing texture tests/fixture and API adaptations were inspected; test
population is unchanged from the original PR checkpoint. Workflow changes run
existing integration tests and doctests and record the tested checkout SHA.
Historical software-Vulkan/browser/hardware/performance/rendering limitations
remain documented; they are not validation of this revision.

Hosted quality/fmt/lint/native/Wasm/project checks and applicable baseline
reductions remain required and unrun. The unchanged baseline still carries
large-module/function receipts reduced by the new extraction. Apply only
relevant reductions and never raise ceilings (`crates/AGENTS.md:167-178`).
Source inspection does not establish the numerical no-growth gate. Root README
and wiki prose remain untouched; durable GPU guidance owns this API contract.

Correctness verdict: the prior owned repairs are sound by source inspection,
but S2 includes concrete missing/wrong-value/interface defects; S1 leaves typed
classification incomplete. Migration-completeness verdict: incomplete at S1,
S2 and every named owner dependency, with execution unverified. After integrated
source repairs, repeat complete audit waves and independent Code/Rule/Comment
reviews on the same final pushed SHA as required hosted CI. Do not merge.

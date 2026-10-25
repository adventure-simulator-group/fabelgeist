# Independent Rule Review: reflection repair checkpoint

Status: superseded while this report was being completed. Root advanced source
to `0bc4fd52bc376a22c6f7e73bc7f9ea427f662f17` before the report was delivered.
The source identity receipts and findings below apply only to the pinned
`2eb1129b0` snapshot. This is not a completion verdict for the successor; see
`immediate-rule-review.md` for its independent source review.

The reviewed repairs close successor S1 and the previously enumerated bound
resource/value cases in S2 by source inspection. One owned interface-admission
gap remains: a **used unbound Immediate input is silently skipped**. The
checkpoint is not clear or source-complete; #780 and retained #774 dependencies
and all required hosted acceptance remain unresolved.

## Exact revision and independent evidence

- HEAD: `2eb1129b0a8d09289aab9e7cc83bc3499a8a3d9e`.
- Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.
- All 73 changed paths were independently compared byte-for-byte with their
  HEAD blobs; all matched, with a clean working tree.
- Every fingerprint in `ledger.json.reflection_repair_checkpoint.files`
  matched, and the checkpoint revision matched HEAD.
- Independent digest of sorted changed-path/SHA-256 records:
  `aebc8a220d1cc1d08953633aecedf04d572af50ee6494b24346090de447839ac`.

The Rule Review skill and its bespoke-type reference, root `AGENTS.md` and
`crates/AGENTS.md` apply. These rules are unchanged from pinned main. Current
source and actual SDK definitions were inspected before ledger dispositions
were reconciled. No old review was treated as current clearance.

This review wrote only this ignored report. It performed no project checks,
builds, tests, scanners, generators, runtime execution, source edits, push,
hosted CI or external posts. No tests, cases, helpers or assertions were added.
Hashing and git reads establish source identity only. All findings below are
source evidence; native/Wasm compilation and actual GPU outcomes are unverified.

## Owned actionable finding: R5 — P2, used Immediate inputs are not admitted

Locations: `compute/pipeline/reflection.rs:29-42`,
`compute/pipeline/mod.rs:159-174`, `kernel/fast/compile.rs:27-42` and the
static/fast recording paths.

The reflector skips every global with `binding=None` before calculating whether
the compute entry-point union uses it. This correctly excludes ordinary
unbound private/workgroup data from host binding assembly. Naga
`AddressSpace::Immediate`, however, is an unbound **host-supplied input**.
Naga validation can admit it with the existing all-capabilities validation
policy. Both native pipeline layouts fix `immediate_size` to zero, and neither
recording path supplies immediate values. A used Immediate therefore has no
first-party representation or upload path despite passing this admission loop.

This is a retained owned migration gap. It is not a claim that arbitrary
malformed WGSL passes validation or that every device supports immediates.
On a device lacking the feature, actual native shader rejection remains a
legitimate provider outcome. On a supporting imported device, this interface
still cannot establish the declared host-input contract. Exact backend failure
or resulting GPU values were not executed and are not claimed here.

Pinned evidence: Naga `ir/mod.rs:375-389` defines Immediate as host-provided
data; `valid/interface.rs:1080-1095` admits compatible data with the IMMEDIATES
capability and does not require a resource binding. Wgpu-core
`device/resource.rs:3688-3703` admits the pipeline's explicit immediate range
against feature/size/alignment limits; `command/compute.rs:821-829,997-1003`
retains/sets native immediate values according to that range. None of those
operations is supplied by the current first-party pipeline/record graph.

Rule: `crates/AGENTS.md:31-35,45-57,58-71` requires faithful semantic records
and explicit admission of supported states. Skipping a used host input is not
the supported unused-global policy.

Bounded correction: calculate compute use before discarding the unbound form,
and explicitly reject a used unsupported Immediate through the canonical typed
pipeline admission owner. This requires no new immediate subsystem. Preserve
unused-global exclusion, private/workgroup handling and native feature/cause
behavior. Do not fabricate a group/binding ordinal for an unbound input.

Acceptance: used Immediate input has a typed, inspectable unsupported outcome
before a misleading reflected interface/native pipeline is admitted; unused
Immediate and ordinary unbound globals retain the stated exclusion policy.
Re-audit every other unbound host-input form admitted by the chosen Naga stage
policy before clearing this category. Comment Review first reported this shape;
this independent review verified the source and SDK contract.

## Current full reflection-shape dispositions

| Shape / responsibility | Current producer → record → consumer evidence |
| --- | --- |
| Used resources / multiple compute entry points | `ReflectedInterface::new` validates source and uses ModuleInfo global usage across all compute entry points. One retained layout serves the entry-point cache. The same union of entry-point sampling_set determines float filtering. EntryPoint FunctionInfo includes callees and transitive texture/sampler pairs (`naga/valid/analyzer.rs:240-280`). Resources unused by every compute entry point are omitted. This is one explicit shared-layout policy, not a per-entry-point compatibility path. |
| Direct supported uniform values | `reflection/uniforms.rs:16-60` now produces a named UniformMember for a non-Struct declaration at native offset zero. The variable name and supported value shape reach both packers. Scalar/vector/matrix values no longer disappear into an empty member list. |
| One supported flat uniform block | Each member retains Naga offset/span and UniformValueShape. The source is validated before `.size()` use; field names remain open shader identifiers. Static and fast writers inspect the same canonical shape before writing, preventing a wider unrelated parameter from overwriting neighboring members. Correctly typed supported packing bytes remain unchanged. |
| Signed and unsigned scalar words | `parameters/parameter.rs:119-129` already converts u32 and i32 into Unsigned, with i32 preserving native bits via `as u32`. `UniformValueShape::IntegerWord` therefore correctly accepts Naga Sint32 and Uint32 and the existing Unsigned slot. Both packers emit those same 32 bits. A separate signed wrapper or signed-scalar rejection would break this established boundary; neither is required. The earlier proposed signed rejection is withdrawn on this source evidence. |
| Float scalar/vector/square matrix | Reflection admits native f32 scalar, f32 Vec2/3/4 and f32 square Mat2/3/4. Static packing retains Mat2/3/4 and Transform→Mat4 support. The fast writer retains its narrower support and returns typed UnsupportedUniform where it cannot pack a value, including its pre-existing Transform/Mat2/Mat3 limits. No new automatic fallback was introduced. |
| Missing/wrong parameter | Static packing preserves the chosen zero-default policy for missing/mismatched fields. Fast packing retains strict admission, now through typed MissingParameter with Uniform role or UnsupportedUniform. Fast missing/wrong Buffer branches reuse the same canonical ComputePassError categories as static assembly. This closes S1 without string identity or Debug-output control flow. |
| Multiple uniform declarations in one retained group | The second declaration returns MultipleUniformBindings before mutating committed group members/span or constructing a layout. One block per shared group is explicit. Separate bind groups remain independent. This replaces the old overwrite/append contradiction with typed unsupported admission. |
| Unsupported uniform layouts | Fixed arrays, nested structs, integer vectors, non-square matrices and non-32-bit representations fail UnsupportedUniformType through the one shape admission function. No parallel host packing representation is invented. Runtime-sized uniform arrays, non-host-shareable bool/atomic forms, malformed layouts and other invalid source remain Naga validation errors with real causes. |
| Ordinary/comparison sampler | SamplerBinding is a named record produced once. Comparison now maps to SDK Comparison; ordinary samplers map to Filtering. Pinned native Filtering layouts permit both nearest and linear ordinary instances; preserving this mapping does not falsely reject nearest samplers. Actual supplied sampler/device checks remain the native bind boundary with retained causes. |
| Float filtering versus textureLoad | Float descriptor filterability comes from actual sampling_set use across the shared compute layout. A texture load-only in every compute entry no longer acquires false filtering requirements. If any retained compute entry samples it, the shared layout retains that requirement. This matches the native derivation policy and preserves working filtered inputs. Actual device format/feature acceptance remains native. |
| Sampled int/uint/depth and multisampling | `reflection/images.rs` preserves Uint/Sint/Depth and the actual Sampled/Depth multi flag. Native view dimension mapping retains D1/D2/D2Array/D3/Cube/CubeArray. First-party texture constructors' sample_count=1 does not relabel a multisampled declaration. If an actual supplied resource is incompatible, native bind validation reports the concrete failure. |
| Storage texture access and format | LOAD, STORE, LOAD|STORE and ATOMIC map respectively to ReadOnly, WriteOnly, ReadWrite and Atomic; the existing TextureFormat conversion owns exact native format mapping. Validated Naga access sets reach this helper. Native feature/format restrictions remain native errors rather than being disguised by an access downgrade. |
| Unrepresented bound resources | Used BindingArray, ExternalTexture and other Handle shapes return UnsupportedBinding with a named kind before native layout creation. Resource binding arrays are distinguished from ordinary storage-buffer array contents. No collection/sampler/format owner is duplicated. Unsupported bound address-space forms likewise have a stable category; malformed forms already rejected by Naga are not invented as valid states. |
| Unbound globals | Private/workgroup data and unused globals need no host binding and are skipped. Used Immediate is the remaining R5 gap. The bound UnsupportedBindingKind::AddressSpace branch cannot cover it because the early binding filter prevents reaching that branch. |
| Limits and provider-native failures | Group index is checked against the device limit; sparse groups retain their actual indices. A faithful but unsupported native feature/format/limit remains a real Native cause. A source module valid to Naga is not proof the chosen provider supports it. Conflicting binding aliases in a module cannot be represented as independent values in one shared interface; the retained explicit layout/native validation does not imply per-entry-point specialized support. No previously working support for such conflicting layouts is asserted. |

Source-backed SDK references are pinned local `naga-29.0.4`, `wgpu-29.0.4`,
`wgpu-core-29.0.4` and `wgpu-types-29.0.4`. Relevant native role comparisons and
derivation remain in wgpu-core `validation.rs:493-608,657-735`; sampler-instance
admission is in `device/resource.rs:2996-3014`. These were source reads only.

## Revalidated owner and receiving graph

| Family | Current trace and disposition |
| --- | --- |
| Texture2d/3d/Cube and generic views | Constructors admit native texture/cache/format/usage/device together inside the texture owner. Readonly ports and clear-only operations preserve supported availability states. Cached matching requests clone the same Arc before allocation/context use; alternates require allocation and actual SDK Device identity. SelectedTextureView carries effective format/dimension/device into PassParameter selection and shared static assembly. Face/array geometry and storage-bound sRGB descriptor policy are preserved. |
| Actual native provenance | SDK Device equality compares native dispatch identity, not a fabricated ID. Alternate, generic, copy, read/write/clear/blit and converter recipients use the admitted device. Converter retains its own native device, uses canonical shader admission and selected views, preserves cache-first missing order, and retains available pipeline/record/submit causes. Borrowed/browser contexts still have no promised synchronous native verdict. |
| Static binding / uniform allocation | The public helper takes immutable ComputePipeline plus PassParameters and shares one assembler with ComputePass. Indexed groups preserve sparse group order; Buffer binding uses the logical prefix. Uniform allocation checks padded native length against allocation and binding limits before packing, allocates unmapped, and pops native scopes before applying fallible results. Upload reuses BufferUpload and canonical byte owners. |
| Shader/pipeline causes/cache | Shared validate_naga preserves the previous all-capabilities validation and real ShaderError causes, now also furnishing ModuleInfo for admission. Immutable shader/source/device and interface remain correlated. Full entry-point cache keys retain observed Result versus device-handled outcomes; rejected pipelines never become successful cache hits. Compiled state/device admission and standard generic conversions preserve source chains. R5 limits interface completeness. |
| Fast/arena/Prepared/batch | Fast logical buffer and byte-offset contracts remain shared with static binding and canonical byte leaves. Native scopes retain concrete CompiledLayout/binding/record/copy causes. Dynamic u32 narrowing and offset addition are checked. Empty batch grids skip parameter admission and successful-dispatch increments, folded policy and callbacks remain intact. No tests or assertions were introduced. |
| Derivation and module ownership | Fieldless UniformArenaError now uses the existing thiserror dependency with the exact former messages and no new source semantics; this is equivalent to its manual implementation. Uniform shape admission and image reflection are real focused responsibilities, with canonical record/error owners in shared GPU modules. Item order follows the guide. No pass-through compatibility service or legacy forwarding path was introduced. |

The native/Wasm cause trait evidence remains the actual conditional SDK
ErrorSource bounds and the configured fragile non-atomic-Wasm feature, not new
unsafe implementations. This establishes source compatibility intent only;
compilation and atomics-enabled browser configurations remain unverified.

Raw strings in the owned graph are shader source/identifiers/format tokens and
display diagnostics. Closed language, shape, resource and rejection roles now
use canonical enums. Native byte offsets/spans, dimensions, bitflags and
filtering predicates remain at documented actual SDK/packing boundaries. No
new epsilon, tolerance, magic status-string branch or independent owner was
introduced. The old build_compute_pipeline/raw-axis compatibility APIs remain
removed.

## Required owner dependencies and verification limits

All #780 Kernel/definition/source-device-cache and checked producer families
remain required. Kernel raw workgroup fields/count parameters, producer-side
primitive-to-grid/byte adapters, source-only caches/mesh unwraps and the owned
Kernel/module/field comment fixes are not repaired or deferred by this review.
Their canonical integration must preserve folded launches, callbacks and exact
bit/packing policies established here.

Retained #774 hair/fitting/body/armor/metal/engraving and downstream equipment
cause families likewise remain required. The two restored claimed hair array
recipients still need their agreed WorkgroupGrid integration. Root's later
human core ownership transfer is reconciled with the coordinators; it does not
authorize duplicate repairs inside those retained bodies. No current
source-complete, push/CI-ready or broader handover acceptance is implied.

All 73 main-to-HEAD paths were accounted for. The review includes full current
parser/shader/reflection/pipeline/static-fast/texture admission and receiving
responsibilities, plus changed algorithm/BVH/mesh/hair/physics dispatch, copy
and error ports. Unchanged algorithm numerical kernels and retained claimant
bodies were not separately audited as independent repair scopes. Existing tests,
fixtures/workflow and durable documentation retain their stated population and
historical software-GPU/browser/hardware/rendering/performance limits.

No baseline/registry ceiling, Cargo lint inheritance or broad suppression was
relaxed. Required hosted baseline reductions after extraction, fmt/lint and
native/Wasm/project gates remain pending under `crates/AGENTS.md:167-178`.
Source inspection does not prove no-growth or runtime acceptance. Root README
and wiki prose are untouched; owning GPU guidance contains the interface policy.

Correctness: S1 and the enumerated bound/uniform S2 repairs are sound by source
inspection, with R5 remaining actionable. Completeness: incomplete at R5 and
all named owner dependencies, with execution unverified. Repeat complete audit
waves and independent Code/Rule/Comment reviews on the final pushed revision
that passes required hosted CI. Do not merge.

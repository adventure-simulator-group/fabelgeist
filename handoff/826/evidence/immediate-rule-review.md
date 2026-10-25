# Independent Rule Review of the Immediate admission checkpoint

No additional owned rule finding was identified in the reviewed source graph.
The prior used-Immediate gap is source-resolved, alongside S1–S4. This is a
checkpoint source verdict only: migration completeness remains **incomplete**
because every required #780 and retained #774 family and hosted acceptance is
still outstanding. No final clearance, push/CI readiness or merge is implied.

## Exact source and review method

- HEAD: `0bc4fd52bc376a22c6f7e73bc7f9ea427f662f17`.
- Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.
- All 73 main-to-HEAD changed paths independently matched their exact HEAD
  blobs; the working tree was clean.
- `ledger.json.immediate_repair_checkpoint` and `immediate_wave` revisions
  matched HEAD, and every listed source fingerprint matched.
- Independent digest of sorted changed-path/SHA-256 records:
  `7d7092493815740305f60bb299ffc245e0100dc493158faf4093b228c11766c3`.

The earlier `2eb1129b0` source was superseded while its report was being
completed. `reflection-rule-review.md` is explicitly marked historical. This
review uses current source and current graph relationships; it does not carry
forward that snapshot's clean status or leave its Immediate finding unresolved.
All main-to-current changes were accounted for, including the unchanged source
ports whose fingerprints were re-established at this revision.

The Rule Review skill, its bespoke-type reference and root/crates `AGENTS.md`
govern the review. The rules are unchanged from pinned main. The later human
core-GPU ownership transfer and exact coordinator reconciliation remain valid;
they do not transfer retained hair/fitting/body/armor/engraving families.

This reviewer edited no source and ran no project checks, tests, builds,
scanners, generators, runtime, push, hosted CI or external publication. Only
ignored review reports were written. No tests, cases, helpers or assertions
were added. Git reads and hashing establish the snapshot, not compilation or
runtime acceptance.

## Current owner → admission → consumer evidence

| Family | Current source-backed disposition |
| --- | --- |
| Used Immediate | `pipeline/reflection.rs:29-47` now computes validated GlobalUse across compute entry points before checking ResourceBinding. A used Immediate returns canonical ComputePipelineError::UnsupportedImmediate with its actual variable name before group/layout construction. No fake group or binding ordinal is introduced. `pipeline/error.rs` retains this typed category through Display and ordinary Result propagation; it correctly has no fabricated native source. S5/R5 is source-resolved. |
| Other unbound globals | Unused globals still leave no bindings. Used Private/WorkGroup state needs no host binding and is skipped. Naga `valid/interface.rs:1444-1453,1487-1509` rejects compute-stage use of TaskPayload without a matching task payload, RayPayload and IncomingRayPayload; these are not additional admissible compute host inputs silently reaching this reflector. The previous broad statement about unbound globals is therefore narrowed to actual validated compute roles. |
| Native immediate limits | Core and fast native layouts still specify immediate_size=0 and record no immediate-value commands. This is coherent because the reflector now rejects every used Immediate admitted by the supported compute stage policy. Actual provider rejection of unsupported source/features can still occur earlier in ComputeShader::new; it remains a real ShaderError native cause rather than a synthetic unsupported error. No runtime mechanism for immediates was added. |
| Uniform value owner | UniformValueShape is defined once beside UniformMember. The sole UniformMember producer, `reflection/uniforms.rs`, carries expected shape/name/native offset/span for supported flat members or direct values. Both static and fast writers consume it before writing. External callers cannot inject a fabricated reflection record through the immutable pipeline assembler. |
| Integer words | Existing `PassParameter::from(i32)` stores Unsigned(value as u32), while From<u32> stores the same word role. IntegerWord admits Naga Sint32/Uint32 and emits those original 32 bits in both writers. For example an i32 -1 retains its all-ones native word. This preserves the actual producer contract; signed rejection or a competing signed wrapper would be inappropriate. |
| Supported packing | Native f32, 32-bit integer word, f32 vectors and square f32 matrices have explicit admitted shapes. Direct uniforms retain the global name instead of producing an empty member list. Flat structs retain reflected offsets/padding. Correct static bytes and the chosen static missing/wrong-value zero policy remain; the fast writer retains strict/narrower matrix support. A wider mismatched value cannot overwrite a neighboring admitted member. |
| Unsupported uniform declarations | A second used uniform block in one shared group returns MultipleUniformBindings before overwrite. Nested/array values, unsupported widths, integer vectors and non-square matrices return UnsupportedUniformType. Naga invalid host-shareable/layout forms fail earlier with actual ShaderError validation causes. No successful declaration is silently reinterpreted as a different packing shape. |
| Shared entry-point interface | `ReflectedInterface::new` uses validated compute entry-point global-use union for required bindings and sampling_set union for filtering. Naga FunctionInfo includes transitive calls/pairings, so helper functions do not hide actual sampling. One retained native layout serves the cache's compute entry-point names; unused resources are excluded. No duplicate per-entry-point reflection or compatibility path is introduced. |
| Samplers/images | Comparison role is preserved; ordinary Filtering layouts retain support for both ordinary nearest and linear samplers. Float textureLoad-only declarations no longer acquire false filtering. Sampled/depth multi flags and int/uint/depth sample roles reach the native descriptor. Storage LOAD/STORE/LOAD|STORE/ATOMIC reach ReadOnly/WriteOnly/ReadWrite/Atomic with the existing canonical format conversion. Actual provider features and supplied native resource compatibility remain native validation. |
| Unrepresented bound resources | Used resource BindingArray, ExternalTexture and other Handle payloads absent from PassParameter return canonical UnsupportedBinding categories before layout creation. Ordinary buffer array contents remain distinct. Group limits are typed; sparse group identities are retained. Incompatible overlapping resource aliases are not promised independent values in the one shared layout; actual invalid native layouts retain native rejection. |
| Fast rejection propagation | Fast missing Buffer, wrong Buffer kind, missing Uniform and unsupported uniform values now use canonical ComputePassError and ComputeBindingKind. The old generic messages/Debug discriminant classification are gone from those producers. Static and fast share the stable owners while retaining their existing distinct admission policies. This closes S1. |
| Texture/cache state | Texture2d/3d/Cube constructors admit native/cache/format/extent/usage/device together within the texture owner. Public access is readonly apart from independent removal operations. All supported missing/detached states and Arc sharing remain. Cached matching requests retain cache-first precedence, alternates require allocation and actual native device identity, and SelectedTextureView carries effective view facts into the shared assembler. |
| Affected texture recipients | Generic views, face/array views, read/write/clear/blit and DistanceField copy paths use the controlled metadata/device ports. Converter retains its own creating device, checks both cache absences in the existing order, then checks devices and selected views. It preserves the unfilterable float textureLoad layout and available native pipeline/record/submit causes. No disconnected native cache/relabel path was found at these receiving ports. |
| Static allocation/bindings | `create_bind_groups` accepts immutable ComputePipeline and PassParameters only. One assembler preserves indexed sparse groups and logical buffer prefixes. Padded uniform length is checked against allocation/binding limits before host packing; allocation is unmapped and native scopes are popped before applying fallible results. Buffer/Definition/Upload/byte leaves are reused. No raw authored-reflection compatibility API survives. |
| Shader/pipeline/cache | Shared validate_naga preserves the previous all-capabilities policy and original validation cause. Immutable source/module/device and reflected interface remain correlated. Full entry-point keys cache observed Result outcomes separately from device-handled creation. Rejected pipelines remain errors across clones/cache hits; compiled state changes only after success and use checks actual device identity. Generic conversions retain native/Naga sources. |
| Arena/Prepared/batch | Canonical byte lengths/offsets, checked addition and u32 dynamic-offset narrowing reach actual native operations. Logical zero binding has a typed rejection; copy bounds retain logical versus native distinction. Empty batch grids skip parameter admission and count increments; standalone admission, folded launches and after-submit callback ordering remain. Native scopes retain available causes. |
| Derivation/module ownership | The fieldless arena error uses the existing thiserror dependency with exact former messages/source semantics. Uniform admission and image reflection are focused modules with genuine contracts; new types/aliases/impls follow required item order. No pass-through compatibility abstraction, duplicate semantic owner or new unsafe trait assertion was introduced. |

The immediate classification was independently checked against pinned Naga
`ir/mod.rs:375-389`, `valid/interface.rs:1080-1095,1430-1509`; wgpu-core's
explicit immediate-range and command handling remains in
`device/resource.rs:3688-3703` and `command/compute.rs:821-829,997-1003`.
The corrected native sampler/image mappings agree with pinned wgpu-core
`validation.rs:493-608,657-735`, and entry-point sampling-use analysis with
Naga `valid/analyzer.rs:240-280`. These are source reads, not executed probes.

The actual SDK Device equality proxy remains authoritative for device identity.
Concrete wgpu/Naga error types and conditional native/fragile-non-atomic-Wasm
Send/Sync evidence remain intact. No formatting/reparse or unsafe auto-trait
adapter replaces a cause. Supported native/Wasm compilation, browser atomics
configurations and actual provider execution remain unverified.

Raw text is shader source, native entry/format tokens, open parameter names or
presentation diagnostics. Closed language, uniform shape, resource kind,
unsupported state and admission failure roles have typed owners. Native
offsets/spans, bitflags and dimensions stay at actual ABI/SDK boundaries.
No display-prose branch, new numerical tolerance or duplicate owner was found
in the reviewed owned graph. The obsolete forwarding/raw-axis APIs remain
removed, and root README/wiki prose remain untouched.

## Required unfinished dependencies and acceptance limits

All #780 producer, count/layout, source/device-cache and mesh caller families
remain required. Current Kernel raw fields/count parameters, primitive
grid/byte producer adapters, source-only caches/unwraps and delegated
Kernel/module/field comments are not cleared by lower-level receiving repairs.
One exact source lineage must complete these paths while preserving folded
launches, callbacks and established integer/native byte contracts.

All retained #774 hair/fitting/body/armor/metal/engraving and connected equipment
cause families remain required. Restored claimed hair array recipients still
need their agreed WorkgroupGrid source integration. No unilateral adapter or
duplicate repair is authorized by this review. The later human core assignment
does not expand to those retained functions.

All 73 changed paths and affected receiving relationships were accounted for,
including core/native owners, unchanged algorithm dispatch/texture/error ports,
BVH/mesh/hair/physics adapters, existing tests/fixture and workflow. Unrelated
numerical algorithms and retained claimant bodies were not independently
repaired or granted a pass. Existing test population and documented historical
software-GPU/browser/hardware/rendering/performance limits remain unchanged.

Required hosted fmt/lint/native/Wasm/project checks and applicable quality
baseline reductions remain pending. No quality registry, Cargo lint inheritance,
suppression or baseline ceiling was relaxed. Apply the extraction's actual
reductions under `crates/AGENTS.md:167-178`; source inspection does not establish
the numerical no-growth gate.

Correctness: no additional owned actionable rule gap found at this exact
checkpoint; prior S1–S5 repairs are source-consistent. Migration completeness:
incomplete at the required owner families and unrun validation. Full audit waves
and independent Code/Rule/Comment reviews must be repeated on the final pushed
SHA accepted by required hosted CI. Do not merge.

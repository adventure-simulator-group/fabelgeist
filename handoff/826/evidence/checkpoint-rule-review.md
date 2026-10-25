# Independent integration Rule Review: PR #826 checkpoint

The checkpoint closes the former public texture/cache mutation and shared
static-binding authority gaps. It does not pass Rule Review yet: the converter
recipient, static uniform failure boundary and sampler record still need
connected repairs. File-level item order also needs correction. The #780
producer/cache dependencies remain incomplete; none is accepted as a deferral.

## Revision, authority and execution limits

- Reviewed HEAD: `a03ccb5da481872dff2aa2bdbb0222b93c7314a6`.
- Compared main: `990430b77a9f63548ffcb642074185f61ddb3322`.
- Working tree was clean at review and fingerprint verification.
- All 65 fingerprints in `shared-source-checkpoint.json` matched current source.
  Its `checkpoint_commit` identifies the reviewed revision; its earlier `head`
  field identifies the pre-repair parent and is not the reviewed HEAD.
- Manifest SHA-256:
  `affc58f0c3c964f593393f53f5cf3dfd2ea0e092939c2d42e4a065a54f2b7897`.

Read the Rule Review skill and its bespoke-type reference, root `AGENTS.md` and
`crates/AGENTS.md`. No deeper guide applies to these GPU/compute packages.
Source and the revision diff were inspected before the ledger extension was
used to reconcile dispositions. The previous review was not treated as proof.

The direct user transfer after #774's merge and #780's explicit agreement now
authorize root ownership of the shared GPU and native fast/batch graph. This
review does not revive the superseded ownership block or authorize independent
repairs in another owner's remaining producer/cache functions.

This was source-only. No source was changed; only this ignored report was
written. No project tests, builds, checks, generators, scanners, hosted jobs,
pushes or external posts were performed. No tests, cases, helpers or assertions
were added. Hashing and git inspection only established the source snapshot.
Native and Wasm compilation, lint/format gates, numerical baseline reductions
and runtime acceptance remain unverified. This is not a final review.

## Findings

### R1 — P2: the sRGB converter omits the creating-device contract

Location: `texture/srgb_converter.rs:7-10,84-101,103-136`.
Classification: existing affected-recipient migration gap, not a new regression.

`new` admits the pipeline and layout on a context device but retains no device
identity. `convert` reads both caches directly and creates/binds resources on
its supplied context without checking either texture's device or the
converter's creating device. A converter created on A with textures/context B
is representable even after the texture fields become controlled. Native
validation then handles a foreign layout/pipeline instead of the established
typed wrong-device path. A source texture created on A with a converter/context
B is another independently representable case. No runtime misuse was observed;
these are source-backed API counterexamples.

Rule: `crates/AGENTS.md:31-35,49-51` requires relevant authority/record
invariants; the requested audit explicitly includes actual device provenance
and affected unchanged consumers. The ledger's converter access/error receipt
does not establish this missing receiving contract.

Repair the whole recipient coherently: retain its actual SDK device alongside
the admitted layout/pipeline, validate it and both texture inputs before native
use, and consume the canonical texture selection/error contract. Keep the
existing missing-cache precedence and conversion shader behavior. Available
native validation failures must retain their cause under the existing context
policy; borrowed/browser limitations remain explicit. No synthetic device ID
or second texture error owner is needed.

Acceptance: converter and both resource devices are proven compatible before
the native binding/dispatch boundary, and the existing public Result graph
preserves the resulting typed failure rather than panic/uncaptured handling on
an observing native context.

### R2 — P2: static uniform allocation precedes the native failure scope

Location: `bind_group/assembly.rs:46-55,72-89`; public entry at
`bind_group.rs:25-49`.
Classification: inherited connected native-cause/admission gap exposed by the
new unified assembler, not a new successful-behavior regression.

The public helper accepts authored `ReflectionData`. A reflected uniform span
can exceed the receiving device's buffer limit. `create_group` packs that span
and calls `DeviceExt::create_buffer_init` before installing the scope that
retains `ComputePassError::Native`. Thus the allocation failure escapes the
promised native-cause path even when `blocking_validation` is enabled.

Pinned SDK evidence: `wgpu-29.0.4/src/util/device.rs:40-78` creates the native
buffer and immediately accesses its mapped range; `wgpu-core-29.0.4/src/device/
resource.rs:1007-1011` rejects a descriptor above `max_buffer_size`. The later
bind-group scope cannot capture this earlier call. Merely moving the scope
upward is not a sufficient admission proof if a rejected buffer is immediately
mapped by DeviceExt. This is static boundary evidence, not an executed case.

Rule: `crates/AGENTS.md:31-35,49-51` and the user's connected native-cause
completion contract. Check the actual allocation bound before packing or
mapping, and perform fallible native allocation/readback so the concrete cause
is retained. Keep every scope balanced on parameter/texture/packing failures;
avoid placing a scope around early `?` exits without popping it.

Acceptance: an invalid authored span produces a typed admission failure or
retained native rejection through the public Result, without unscoped buffer
creation/mapping. Ordinary packing bytes, fresh groups, sparse group indices
and borrowed/browser policy stay unchanged.

### R3 — P2: sampler reflection remains a positional domain record

Location: `shader.rs:30-37`, `pipeline/reflection.rs:123-128`,
`bind_group/assembly.rs:153-175`.
Classification: existing in-scope semantic migration gap.

`BindGroupReflection::sampler_bindings` retains `Vec<(String, u32)>`, produced by
the newly owned reflector and passed into the new assembler. The tuple carries
two distinct roles, parameter name and native binding index. This is an
internal retained reflection record; WebGPU does not require that tuple.
Destructuring it at the final consumer does not change its stored API shape.

Rule: `crates/AGENTS.md:45-51,66-71` requires named records and keeps required
positional native forms at their actual adapter. Define the sampler binding
record once next to `BufferBinding`/`TextureBinding`, propagate it through the
reflector and assembler, and adapt the fast-path shape predicate. Preserve
open parameter names and the native binding index; no invented closed name
enum is warranted. Source search found those four sampler-binding ports only.

Acceptance: no positional sampler role record remains in the receiving graph;
lookup order, binding indices and native sampler semantics are unchanged.

### R4 — P3: new production items violate required file-level order

Locations: `texture/view_error.rs:120-137` declares its Result alias after impl
blocks; `compute/pass/shape.rs:9-23` declares `WorkgroupShape` and its error
after the InvocationCount conversion impl.

Rule: `crates/AGENTS.md:90-94` explicitly orders aliases before structs/enums
and all type declarations before impls. Move the existing declarations and
attached documentation to that order. This needs no API change, forwarding
alias, test or new owner. The #779 canonical production types remain canonical;
reordering their declarations must not fork their contracts.

The older interleaving of Kernel/KernelCache/KernelBatch and CPU definition
records is also visible but belongs with the pending #780 whole-file producer
integration; it is not a reason for a competing edit by this reviewer.

## Source-backed contract receipts

| Family | Owner → admission → retained state → consumer/boundary |
| --- | --- |
| Texture/cache/view state | The three whole records and generic TextureView restrict replacement/relabeling to the texture owning module; constructors admit device, cache and metadata together. Readonly accessors and clear-only transitions preserve the four existing availability states. Selectors keep cache-first absence priority and Arc identity, require the native allocation for alternates, and check actual SDK Device equality before alternate creation. The generic conversion retains logical image format separately from effective view format. Public mutation counterexamples from the earlier snapshot are closed. R1 remains an affected unchanged recipient. |
| Effective binding facts | `texture/selection.rs` owns SelectedTextureView once. All three whole variants and generic views carry handle/effective format/dimension/device together into `PassParameter::texture_binding`; validation compares those selected facts rather than `Texture::format()`. Both public and pipeline-specific static binding entry points call `assembly::create_group`. `ComputeBindGroup` retains sparse native group indices and binds at that index. No parallel compatibility assembler survives. R2/R3 remain in the shared boundary. |
| Shader and pipeline causes | `ShaderError` owns real WGSL/GLSL parse, validation, translation and native causes plus rendered source diagnostics. `parse_naga` and ComputeShader admission preserve the typed cause; source/module/device are immutable together. ComputePipeline prepare verifies the device, reflects once and retains its immutable shader/interface. Complete entry-point strings key a cache of observed Result outcomes or explicitly device-handled native creation. Native errors are Arc-shared, and `source()` reaches the actual cause; rejected creation is never cached as success. A later observing context does not inherit an earlier unchecked cache entry. |
| Pipeline use | `compile` transitions only after a successful cache outcome; `compiled` checks device and prepared/compiled state. Static assembly and native recording preserve generic boundary conversions with `?`. The obsolete build_compute_pipeline wrapper and raw-axis forwarding API are absent by source search. No compatibility aliases or deprecated paths were introduced. |
| Buffers and fast recording | Buffer::binding_resource reuses BufferByteLength, admits NonZeroU64 only at actual WebGPU binding, checks logical prefix against allocation and returns BufferBindingError for zero/oversize. FastPath uses that same adapter; CompiledLayout keeps native causes. UniformArena retains typed lengths/offsets, checked additions and per-batch chunks. Prepared checks dynamic offset narrowing at the SDK boundary. KernelBatch checks kernel device, preserves empty-before-admission/counter semantics, records canonical WorkgroupGrid, and keeps copy bounds/callback ordering. |
| Launch/count owners | Canonical WorkgroupGrid, WorkgroupShape and InvocationCount are defined once in GPU pass modules; WorkgroupShape uses NonZeroU32 admission and has no unchecked derive/deserializer. Grid permits zero and exposes named occupancy. RecordedDispatchCount retains successful nonempty recording semantics and agreed transparent Serialize. These leaves are sound increments; Kernel still stores mutable raw workgroup_size and accepts raw invocation counts, so producer completeness is pending #780. |
| Unchanged recipients | GpuResource/PassParameter retain whole textures; ResourceDescriptor projects logical formats to generated shader syntax. Texture2d blit/read/write/clear, generic view clear/read, cube copy/face/array paths and DistanceField copies check availability and creating device. Generic ComputePass recipients preserve errors via `?` at dispatch/record. Mesh's source-only static pipeline cache still unwraps creation and retains a single device's pipeline, explicitly pending #780. |

Actual SDK identity was checked against
`wgpu-29.0.4/src/api/device.rs:26`: Device equality proxies its native dispatch
handle. This is not Arc allocation identity or a locally invented device token.
`device.rs:774-825` gives the actual Error source and conditional Send/Sync
contract. `wgpu-29.0.4/build.rs:7-11` enables it on native targets or the fragile
non-atomic-Wasm feature. `fabelgeist-gpu/Cargo.toml:33-34` enables that feature
for the supported browser build. The retained Naga types implement Error in
the pinned dependency; no formatting/reparse adapter or unsafe auto-trait impl
was added. This supplies source-level trait evidence, not an executed Wasm or
native compilation verdict. Atomics-enabled browser configurations are not
proven by this evidence.

## Pending #780 source dependencies

These are incomplete paths, not accepted follow-ups or new ownership claims:

- `kernel/mod.rs:47-55,138-149` still stores writable raw workgroup axes and
  receives raw invocation counts despite the admitted canonical Shape/Count
  leaves. Mutation can still introduce a zero divisor. Definition/kernel caches
  still use source/descriptor-only identity without device/source ownership.
  `KernelCache::get/len` at `188-205` still unwrap locks. The lower pipeline now
  rejects wrong-device use, but that does not make the producer cache correct.
- Generic CPU definitions (including texture_ops) retain mutable source and
  device-incomplete caches. Map/Stencil cache records also retain positional
  pipeline/size tuples. Mesh's static cache at `mesh/src/gpu.rs:209-212` unwraps
  the now-typed producer. These must receive #780's canonical repair.
- PrefixSum grid helpers, sort byte lengths, hair byte lengths, physics copy
  lengths and algorithm workgroup computations still convert primitive results
  immediately before native receiving APIs. Their actual producers must carry
  the agreed leaves where meaningful; call-site conversion alone is not a
  complete producer migration under `crates/AGENTS.md:36-42`.

No independently competing repair was made in these bodies. Final review must
trace the integrated #780 producer contracts back through this receiving graph.

## Coverage, rules and validation remaining

All 71 changed paths were accounted for: the 65 paths in the checkpoint manifest
plus `.github/workflows/rust-quality.yml`, the GPU README, srgb_converter,
texture_2d/image, WgpuContext and the texture-view JSON fixture. Complete new
modules and the changed producer/receiver responsibilities were inspected;
unchanged numerical shader kernels were not expanded into a separate audit.

Changed algorithm ports in advect, broadcast, distance_field/JFA, divergence,
dual_contouring, gather, gradient, map, marching_cubes, matmul, prefix_sum,
reduce, reshape, scan, scatter, sort, stencil, stereo, stream and transpose were
checked for canonical grid/byte admission and error propagation. BVH, hair
grow/guides/scalp, mesh and physics copy/dispatch recipients were likewise
checked. Existing tests and fixture oracles were read; the current adaptations
change API representations, not test population. No #779 new tests were added.

Raw WGSL, `#version` language detection, native entry points, shader format
tokens, reflected open binding names and diagnostic text are actual text/native
boundaries. ShaderLanguage and typed errors replace closed classifications;
no branch on mutable display/error prose was found. ABI byte spans/strides and
native descriptor dimensions remain at packing/SDK operations. The shared
256-pixel texture default has one owner. No new tolerance policy or unrelated
numerical repair is introduced. Remaining semantic count/byte producers are
explicitly listed above rather than excused as native merely because their
current consumers are native.

Binding assembly, uniform packing, fast compilation and arena responsibilities
were extracted into actual focused modules. No changed Cargo lint inheritance,
quality registry scope or baseline ceiling was broadened. The quality baseline
is unchanged and still contains the old large bind_group/record receipts;
required applicable reductions and no-growth checks remain pending hosted
validation under `crates/AGENTS.md:167-178`. Physical line counts or the absence
of a baseline edit cannot prove the numerical gate passed. Do not raise a
ceiling to accommodate final producer integration.

The workflow invokes existing texture integration tests and doctests and
records the tested checkout revision. Its project-wide required checks still
need the final pushed source. The durable texture document preserves its
software-GPU history and browser/hardware/performance/rendering limits; those
observations are not current acceptance evidence. Root README and wiki prose
are untouched. PR publication/context requirements remain for root to complete.

Correctness verdict: no blanket pass; R1/R2 are demonstrated boundary gaps and
R3/R4 are explicit rule gaps. Migration-completeness verdict: incomplete at
these findings plus #780 source dependencies and unrun hosted checks. Repeat
the complete audits and independent Code/Rule/Comment reviews after integration
on the same final pushed SHA as required CI. Do not infer final clearance from
this checkpoint.

# Independent successor-checkpoint Code Review

No new actionable correctness gap was found in the owned implementation at
this checkpoint. The earlier converter finding is source-resolved. The known
consumer compilation failures and source/count/cache families owned by #780
and retained consumer families owned by #774 remain required dependencies.
This review grants no source-complete or final-clearance verdict.

## Pin and independent fingerprints

- Reviewed HEAD: `219d407be5b89e6a910244ba3362326088603701`.
- Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.
- Git reported a clean working tree. The current main-to-HEAD diff contains
  71 changed files.
- SHA-256 of `git diff --binary BASE HEAD`:
  `639aaa0ce1bd197e225e3ea86664c3e66f2d80abc2424ef6c9d51bfad6785502`.
- Independently read every changed file and computed a path-to-SHA-256 map.
  SHA-256 of that map encoded as sorted, compact JSON:
  `7e1eb5325f321709438c11dad760457954ed7a39248f5d55f7e06c427c546763`.
- All file hashes in the ledger's `checkpoint_repairs.files` independently
  matched. The earlier checkpoint manifests were not accepted as current pins.

Reviewed the whole main-to-HEAD diff, the current owned source and materially
affected unchanged admission/use ports. The successor delta was read as well
to verify that earlier evidence remained applicable rather than treating the
prior verdict as clearance. Read the ledger's original shared-graph inventory,
later repair contracts, ownership reconciliation and explicit retained
dependencies. The direct human transfer governs the owned core; retained #774
consumer claims and #780 producer/cache ownership were respected.

## Owned implementation receipts

### Texture construction, cache and selected-view facts

Read all three whole-texture owners and selectors, TextureView, selection.rs,
view_error.rs, the face/array paths and their direct read/write/copy/render
recipients. Controlled constructors establish device, allocation, logical
format, usage and cache together; removal operations admit no replacement or
relabel. All four allocation/cache availability combinations remain supported.

Matching requests return the same cached Arc; missing cache precedes native
admission, including defaults and both-cleared states. Alternate requests
require the allocation, so a detached cache cannot provide an alternate view.
Cache-absent/native-present alternates remain possible. Alternate selections
are new Arcs without a cache mutation. Foreign-context alternates fail before
native view creation, whereas the explicitly retained cached return ignores
that unused context. Binding selection subsequently checks the creating
device. Generic cached-view absence precedes its device check.

Native view failures retain original requests and boxed concrete wgpu errors.
SelectedTextureView carries effective view format, geometry and creating
device, and validates these rather than the allocation format. Cube geometry
remains Cube/D2/D2Array at whole/face/array boundaries. Face context identity is
checked on both target families when a context is supplied; the context-free
face port uses its retained native allocation. Array creation receives a
context and retains its actual native cause.

The removed generic `view_format` duplicated the sole admitted generic format:
whole-texture constructors exclude STORAGE_BINDING for logical sRGB formats,
and face construction stores its actual selected format. The deleted initial
constructor storage-plus-sRGB branches were therefore unreachable. Alternate
sRGB requests on a linear storage texture still normalize at the descriptor
boundary. Whole-to-generic conversion and its readback interpretation remain
coherent without the duplicate field.

Direct whole/generic clear/read/write, whole blit, cube upload/copy/render and
DistanceField minimum-copy paths retain their source/target admission and
initialized numerical/descriptor behavior. The minimum-copy preflight remains
before native work. Zero-sized read ports retain their existing early-return
order. The constructor upload guards no longer unwrap native handles.

### Converter finding C2 is resolved in source

`texture/srgb_converter.rs:32-36` now retains the creating device.
`convert:105-120` first checks source and destination caches in the existing
order, then checks both texture devices and the converter device. It selects
the admitted cached views only after those checks. The source A/destination B
and converter A/context B scenarios from C2 now return typed admission errors
before native binding or recording.

Construction uses canonical ComputeShader admission and retains actual native
layout/pipeline failures. Conversion scopes include binding, recording and
encoder finish, followed by a separately observed queue-submission boundary.
Available native causes enter ComputePipelineError or ComputePassError through
Arc-owned actual wgpu errors. Borrowed/browser contexts preserve the explicit
lack of a synchronous native verdict.

The fixed layout still accepts unfilterable float inputs for textureLoad;
replacing it with the generic filterable reflection policy would have changed
accepted input formats. That narrowing was avoided. The shader's transfer
function, constants, destination format and 16-by-16 covering grid remain the
same. This closes the reviewed admission gap without a separate converter
error owner or duplicated device identifier.

### Shader, pipeline, reflection and binding graph

Read shader.rs/error.rs, all pipeline modules, pass modules, binding assembly,
uniform packing/allocation and the logical buffer binding adapter. The shader
admits immutable source/module/device. Pipeline reflection/layouts/cache cannot
be independently replaced. Complete entry-point strings own cache identity.
Observed native rejection is cached as an error; retries and clones preserve
its Arc-owned concrete source. Device identity is checked before shortcut or
cache use. Device-handled creation is distinguished from observed validation,
and a later observed context rebuilds an unchecked entry. Prepared/compiled
state retains lazy compilation and the separate fast-layout path.

Sparse reflected group ordinals retain None slots and the original indices in
ComputeBindGroup. Arrayed image dimensions survive reflection. The public
assembler now accepts the immutable admitted pipeline and checks its device;
it no longer accepts an independently authored interface and layout slice.
ComputePass calls that same public assembler. SamplerBinding's named record is
propagated from the actual Naga producer to binding assembly; all repository
sampler_bindings uses were checked for obsolete tuple consumption.

`bind_group/uniform_buffer.rs:18-38` checks padded native extent against both
allocation and uniform-binding limits before host packing. Allocation is
unmapped through BufferDefinition; available allocation failure is observed
before upload. Upload has its own scope. Existing four-byte trailing padding
and whole-buffer binding are preserved. Packing still follows the previous
static policy for missing values, unsigned/scalar/vector/matrix/Transform
values and Mat3 column stride. New checked range arithmetic avoids overflow
without changing valid packed bytes. Typed oversize and native failures retain
their existing owning error families.

Logical buffer prefixes remain distinct from allocation size; zero and
oversized bindings fail explicitly. Whole/generic texture bindings share
effective-format/dimension/device checks and preserve view errors through
anyhow context. No obsolete build_compute_pipeline,
get_or_create_pipeline_validated, validation_error or invalid shader/pipeline
default call remains in the searched affected Rust sources.

### Fast path, bytes, launches and affected recipients

Read the full Kernel/FastPath/CompiledLayout/Prepared/UniformArena graph and
KernelBatch dispatch/copy/clear/submit. Single-group-zero buffer-only fast
eligibility and uniform capacity are preserved. Uniform size/stride/offset
carry the canonical byte types. Arena addition and dynamic-offset narrowing
are checked; every slot advances within its retained chunk, and earlier chunks
remain alive through submission. Bind groups remain freshly created per
dispatch. Native pipeline/binding/record causes retain their concrete errors.

An empty batch launch still returns before device/parameter admission,
allocation or counter increment. Standalone empty dispatches still admit
parameters. Folded launches and index reconstruction retain current-main
behavior pending #780's producer migration. Copy checks retain logical bounds
and native zero-length behavior. Callback insertion order and after-submission
execution remain unchanged.

Re-read every current compute accessor/grid/result adapter and the changed BVH,
mesh, physics, hair and test adapters. Inspected affected unchanged texture
parameters/resources, Buffer extent/allocation/upload, source-only caches,
generic Definition producer ports, armor and character-creator dispatch
receivers, BVH Kernel::run use and XPBD dispatch-count JSON observations.
RecordedDispatchCount's transparent serialization preserves the existing JSON
number oracle. Existing texture helper bodies use admitted getters/removal
operations without adding cases/assertions/helpers. Sort's existing expected
zero has the count type. Algorithm expressions and shader sources in these
adapter-only changes retain their previous order and values.

Read the workflow, crate README link, context documentation and validation
guide. Existing hosted tests/doctests and evidence upload remain present. The
guide preserves historical-provider, browser-runtime, hardware, performance and
rendering limits; no historical observation supplies a current-source verdict.

## Pending owner dependencies, still not cleared

The earlier P1 consumer-adaptation finding remains open. KernelBatch::dispatch
still requires WorkgroupGrid, while retained consumers pass arrays. Examples
include hair guides.rs:223-241 and 292-303, grow.rs:577-597, armor
anatomy.rs:164, breastplate/kernels.rs:160, bracer.rs:237-241/261 and
metal/bake.rs:152-159, plus the character-creator device dispatch ports recorded
in checkpoint-code-review.md. Rust does not implicitly apply From to those
arguments, so the full connected source cannot compile yet. The two restored
hair adapters are explicitly retained-owner dependencies, not an accidental
owned repair omitted from this review. No compatibility overload is warranted.

#780's checked count/shape/byte producers, KernelCache and Definition/
TextureBinaryOp source/device cache families and GpuMesh caller remain required
integration dependencies. #774's retained hair, armor, body and engraving
families remain required as reconciled with the coordinators. Their presence
cannot be converted into a scoped final pass or silently deferred. No new
competing types, unilateral consumer repairs or ownership assumptions were
introduced by this reviewer.

## Limits

This is a source-only checkpoint review. No local checks, builds, tests,
scanners, generators or hosted jobs were run. No source, tests, cases, helpers
or assertions were edited or added. Only this ignored report was written.
No push, external posting, source-complete or final-clearance claim was made.

After the required owner dependencies are integrated, repeat all thirteen full
audit lenses, independent Code Review/Rule Review/Comment Review and required
hosted checks at the same final pushed revision. This report cannot satisfy
those final-revision requirements.

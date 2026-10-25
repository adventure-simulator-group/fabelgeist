# Independent integration Code Review

Two actionable findings remain at this checkpoint: missing WorkgroupGrid
adapters prevent consumer compilation, and the sRGB converter still bypasses
the coordinated device-admission contract. This is not final clearance.

## Exact source and method

- Reviewed checkpoint: `a03ccb5da481872dff2aa2bdbb0222b93c7314a6`.
- Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.
- The working tree was clean when reviewed and HEAD matched the checkpoint.
- Every file hash in `shared-source-checkpoint.json` matched. Its manifest
  SHA-256 was
  `affc58f0c3c964f593393f53f5cf3dfd2ea0e092939c2d42e4a065a54f2b7897`.
  The manifest's `head` names the preceding source-preparation snapshot;
  its `checkpoint_commit` names the reviewed commit above.

Read the complete diff of all 71 changed files, the shared implementation
modules, and materially affected unchanged API consumers. Read the ledger's
`shared_graph_pre_repair_extension`, including its ownership transfer,
binding-assembly extension and array/copy extension. Prior preliminary review
verdicts were not used as evidence for this checkpoint. Repository-root and
crates rules apply. Review was read-only except for this ignored report.

## Actionable findings

### C1 — P1: finish dispatch adapters before compiling connected consumers

`KernelBatch::dispatch` now requires `WorkgroupGrid` at
`crates/fabelgeist-compute/src/kernel/mod.rs:253-258`, but several actual
callers still supply a bare `[u32; 3]`. Rust does not implicitly call `From`
for a function argument. These are deterministic source-level type mismatches,
not a speculative runtime scenario. For example, the changed hair guide file
adapts its earlier call but leaves `[count, 1, 1]` at
`crates/fabelgeist-hair/src/gpu/guides.rs:303` unchanged.

Additional unchanged affected calls are:

- `fabelgeist-armor/src/gpu/anatomy.rs:164`;
  `breastplate/kernels.rs:160`; `bracer.rs:237-241,261`;
  `metal/bake.rs:152-159`.
- `adventuresim-character-creator/src/device_garment_kernel.rs:226`;
  `device_frames.rs:219,225`; `device_torso.rs:336`;
  `device_head_frame.rs:219,225`; `device_coif.rs:332,338,344`;
  `device_foot_frame.rs:217,223`; `device_clearance.rs:213`;
  `device_underlayer/sweep.rs:72`; `device_footwear_fit.rs:104`;
  `device_close_helmet.rs:110`.

All paths above are under `crates/`. The receiver and surrounding source were
checked to distinguish these KernelBatch calls from ArmorGpu's unrelated
dispatch method. Armor and hair are workspace members; character creator is
an excluded workspace with a direct path dependency on fabelgeist-compute and
a documented hosted Clippy gate. Exclusion does not make its broken call sites
out of scope.

Minimum connected repair: complete the canonical checked-grid producer and
receiver migration at every call site; keep fixed native literal arrays at
their actual boundary with explicit admission where appropriate. Do not add an
array-taking overload or restore the removed primitive forwarding signature.
The #780 checked count/shape producer migration owns substantial parts of this
work. Reconcile these exact paths with #780; the missed hair adapter is also
inside a file already adapted at this checkpoint. This finding must remain
unresolved until those sources are integrated. No independent source repair
was made by this reviewer.

### C2 — P2: admit converter and texture devices before native conversion

`TextureSrgbConverter::convert` at
`crates/fabelgeist-gpu/src/data/gpu/texture/srgb_converter.rs:90-115` checks
only cached-view presence. It then creates a bind group on the supplied
context device using both texture views and the converter's stored layout.
The converter's fields at `7-10` retain no creating-device identity.

For a concrete public-API scenario, create the converter and source on device
A, create the destination on device B, then call convert with A's context.
Alternatively, pass B's context to a converter and textures created on A.
Both scenarios reach native binding with resources from different devices;
neither returns the canonical WrongDevice admission error. No scope here
captures the resulting native rejection, so the configured uncaptured-error
handler may panic and the Result cannot retain the real cause.

This is an existing connected-family gap, exposed by inspecting the affected
unchanged consumer; it is not a new algorithm regression and is not part of
#780's source/count/cache ownership. The ledger already inventories this
public converter as a texture admission/use port. No repository invocation of
the converter was found, but its public input path remains representable.

Minimum connected repair: retain the converter's creating device, preserve
source/destination cache-absence precedence, and check converter/source/
destination provenance before binding or recording. Retain actual available
native rejection causes through the existing context validation policy rather
than formatting or inventing them. Preserve the shader, transfer function,
dispatch geometry and borrowed/browser validation limits.

## Checked contracts and source coverage

The following receipts are source conclusions, not executed validation.

### Texture availability, view and device states

Read all three complete selectors and their owner constructors, defaults,
removal operations, equality and read/write/clear ports. Their shared branch
behavior at this checkpoint is:

| State/request | Source outcome |
| --- | --- |
| Default, matching format | MissingView |
| Default, alternate format | Uninitialized |
| Live allocation/cache, matching format | Shared cached Arc |
| Live allocation/cache, valid alternate | Fresh Arc; cache unchanged |
| Live allocation/cache, invalid alternate, observed validation | Typed shape variant with original request and native source |
| Cache removed, allocation present, matching format | MissingView |
| Cache removed, allocation present, alternate format | Native selection remains possible |
| Allocation detached, cache retained, matching format | Shared cached Arc |
| Allocation detached, cache retained, alternate format | Uninitialized |
| Both removed, matching format | MissingView |
| Both removed, alternate format | Uninitialized |
| Alternate request, allocation present, foreign context | WrongDevice before native creation |
| Matching cached request, foreign context | Existing cache-first return; binding admission later rejects foreign device |

The public whole-texture methods intentionally ignore the supplied context
when merely returning a cached view; this is documented and preserved.
Compute selection carries the retained creating device and rejects it at the
binding boundary. Generic TextureView checks cached presence before device
admission. Whole-to-generic conversion retains logical image format separately
from effective view format. Native allocation format is no longer used as the
selected view's format authority.

Cube face and array paths preserve D2, Cube and D2Array geometry. Array creation
requires a context and retains CubeArray's actual native cause. Face creation
checks device identity when supplied a context; its context-free API acts on
the retained allocation. Direct cube upload/copy/render, whole-texture blit,
generic clear/read and DistanceField minimum-copy ports were read. Read/clear
zero and absence precedence and allocation-without-cache copy behavior remain
as inventoried. C2 is the outstanding direct converter admission port.

`TextureViewError` returns its boxed wgpu error as source; request-to-descriptor
linearization does not replace the original requested format in rejection.
ShaderError preserves concrete WGSL parse, GLSL parse, Naga validation,
translation and native causes. The pinned Naga ParseErrors Error implementation
was inspected where needed for the public source conversion.

### Pipeline, reflection and binding graph

Read shader.rs, shader/error.rs and all compute/pipeline modules. Shader source,
module and device are admitted together. Pipeline source/interface/layout/cache
cannot be independently replaced by consumers. Cache keys retain complete
entry-point strings. Observed successes and errors are cloned from the same
cache outcome, including Arc-owned native error identity; failed creation is
not cached as a successful pipeline. An unchecked device-handled entry is
recreated when a later context requests observed validation. Device checks
precede pipeline shortcuts. Prepared versus compiled state preserves lazy
static-layout compilation; fast kernels retain their independent layout.

Reflection preserves original sparse group ordinals with None slots and checks
the device's group limit before allocating layouts. ComputeBindGroup retains
its index and binds itself to that index. Arrayed image dimensions survive
reflection. Pass and public bind-group adapter share one assembler. Static
uniform packing was compared directly with the previous ComputePass packing:
scalar/unsigned/vector/matrix/Transform byte order, offsets, Mat3 padded stride,
missing-value behavior and supported kinds are retained. The old parallel
public helper no longer has a separate packing/texture/buffer policy.

Buffer binding uses the canonical logical prefix and rejects logical zero or
logical extent beyond allocation before native binding. Whole, generic and
cube texture parameter variants share selected-view format/dimension/device
validation. Native binding/recording scopes retain real wgpu causes where the
context requests readback. Alternate-view producer errors pass through anyhow
context with their concrete source intact.

### Fast path, arena, batches and unchanged recipients

Read the complete Kernel, FastPath, CompiledLayout, Prepared, UniformArena,
uniform packing, batch dispatch/copy/clear/submit and dispatch-count modules.
The buffer-only, single-group-zero eligibility and MAX_UNIFORM_BYTES policy
remain. Uniform size/stride/offset use the canonical byte owners; arena
addition and dynamic-offset narrowing are checked. Fresh bind groups remain
per dispatch. Slots advance within each chunk and a new chunk resets only its
own used offset; earlier chunks remain retained until batch submission. Uniform
writes precede recording and slots are not reused by a batch.

The zero-grid batch return still precedes device/parameter admission, uniform
allocation and dispatch-count increments. Standalone passes retain parameter
admission on an empty grid. Folded-grid arithmetic and WGSL reconstruction are
unchanged pending #780's checked producer migration. Copy logical bounds and
native zero-length behavior are retained. Submission callbacks still run in
insertion order after queue submission, and are dropped if no submit occurs.

Reviewed all changed accessor/grid/byte adapters in compute operations, BVH,
mesh, physics, hair and existing tests. Checked materially affected unchanged
ports in PassParameter/PassParameters, Buffer extent, KernelCache, definitions,
armor, character creator, XPBD dispatch-count JSON observations and BVH's Kernel
run test. No obsolete pipeline/default/get_or_create_pipeline_validated or
build_compute_pipeline call remains in the searched Rust sources. C1 identifies
the remaining dispatch signature mismatches. Existing parser/compile callers
use standard error propagation; unrelated numerical kernel internals were not
treated as a new implementation assignment.

Read both existing texture test source/fixture and the sort test adaptation.
The texture helper bodies now use readonly/removal APIs; their cases, assertion
expressions and oracle values remain the existing ones. Sort's expected zero
now has RecordedDispatchCount's zero type. RecordedDispatchCount's transparent
Serialize preserves existing XPBD JSON observations. This review added no test,
case, helper or assertion.

Read the owning README link, validation guide, context comment changes and
workflow diff. Hosted integration/doctest invocation and evidence upload are
present. The guide retains historical-provider, hardware-wide, browser runtime,
performance and calibrated-rendering limits and requires final-revision hosted
acceptance. No current-source runtime result is inferred from historical data.

## Known #780 dependencies and limits

KernelCache and generic Definition/TextureBinaryOp caches remain source-only
or lack the complete agreed device/source identity contract. GpuMesh's caller
and checked count/shape producers still await #780's implementation. Kernel's
public workgroup metadata, raw producer counts and intermediate native byte
admissions therefore are not declared complete. C1 supplies exact additional
consumer failures that must be reconciled with that active dependency. No
competing cache/count types or independent repairs were introduced here.

No local tests, builds, lints, project checks or scanners were run. Source
searches and fingerprint reads were used; no compilation result is claimed.
No source was edited, pushed or posted externally. This report reviews only
the pinned local integration checkpoint. Both findings, #780 dependencies,
all thirteen complete audit lenses, all three independent reviews and required
hosted checks must be resolved/repeated at the same final pushed revision.

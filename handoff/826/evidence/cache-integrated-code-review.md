# Independent cache-integration Code Review

No additional actionable source defect was found in the integrated cache delta
at this checkpoint. Cache identity, successful-value publication and the
receiving constructor resolutions are coherent with the inspected producers.
Required named-recipe/layout/count/mode/parser/Reduce/fold-caller and retained
#774/F19 families remain incomplete. This is not source-complete or final
clearance.

## Exact source and independently verified evidence

- Reviewed HEAD: `7dee3d5a69fa6e5132efa740d8b0f23a42cb435f`.
- Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.
- Git reported a clean working tree. All changed working files matched their
  committed HEAD bytes.
- The receiving delta from `3ff23fc9a47fe32fe514747a17585606ea001475`
  contains exactly 18 paths. The complete main-to-HEAD scope is now **74**
  paths, because source_cache.rs is new.
- SHA-256 of the complete binary main-to-HEAD diff:
  `de097f3241a2f24460700512e92b4aeae48ac7e75c0bc51a45694e2f2a16569e`.
- SHA-256 of the independently computed changed-file path-to-SHA-256 map,
  encoded as sorted compact JSON:
  `c33541401606bf3dd246203d32827eb193835d7662c571caaa356b624ae4bf82`.
- Every current cache_integrated_wave fingerprint matched independently.

Read all 18 changed paths and the complete receiving delta, then checked their
actual recipe producers and affected recipients. Independently read immutable
`handoff-cache-5c2d287936c62432/manifest.json`: SHA-256
`cd367f9ed59b7b505a19925d5f3e96e3fa85dfe15aa6a5121a23d82cc56680d3`.
Every exported source hash matched that manifest. The cache-only patch hash is
`70cdf94f32da5ffa2b684661d0826f6202e4fd27dd7ebe2d4f821ab91d2408ca`.
The shared cache source is byte-identical to the immutable export. Integration
was reviewed as a delta, not as approval of replacing newer core files with
the export's older full files. No Reduce reference source was integrated.

## Cache correctness receipts

### Canonical key and publication

`compute/source_cache.rs:21-26,48-53` retains the actual cloned wgpu Device,
the context's native observed-validation policy, complete authored source and
recipe parameters in one key. The pinned wgpu 29.0.4 api/device.rs defines
Clone and its native equality/hash proxy, supporting the actual key traits and
device identity rather than a fabricated identifier. Wasm keys do not claim
synchronous observed validation from an otherwise true context flag.

Lookup at source_cache.rs:54-60 returns only a published value. Build executes
outside the read/write locks. `build()?` at line 63 precedes publication, so a
failed shader, parser, pipeline or multi-pipeline construction does not cache a
successful-looking value. The write-side entry operation shares the first
successfully published value among racing successful builders. A failure
returns its actual error and does not clear another successful entry. Read/
write poisoning has the existing canonical typed classification. No lock is
held while creating native resources.

These are Rust Result outcomes. Borrowed/browser device-handled construction
does not acquire an observed native-validation verdict by entering the cache;
the policy key separates it from observed construction. Existing concrete
ShaderError/ComputePipelineError/native causes propagate through closure `?`
without string replacement. Inner immutable pipeline entry-outcome caches
remain distinct from outer source/recipe caches.

### Actual recipe identity

Compared each receiving key with the variables its shader builder reads:

| Family | Source identity and recipe inputs |
| --- | --- |
| KernelCache | Complete kernel source and unit recipe |
| Map | code, optional input descriptor, output descriptor, sorted secondary names/descriptors |
| Stencil | code, input/output descriptors, sorted secondary names/descriptors, boundary mode |
| Gather | code and input/output descriptors |
| Scatter | code and input/output resource kinds used by its builder |
| Stream | code and input descriptor |
| TextureBinaryOp | actual op_code and all three resource descriptors |
| Scan | code and unit recipe; both compiled pipelines and returned type form one value |
| Advect | immutable built-in recipe, three resource descriptors and mode |
| Divergence | immutable built-in recipe, input/output descriptors and boundary mode |
| Gradient | immutable built-in recipe, three resource descriptors and boundary mode |
| Broadcast/MatMul/Reshape/Transpose | Actual fixed shader text and unit recipe |
| GpuMesh displacement | Actual fixed displacement shader and unit recipe |

Each cache instance belongs to its recipe family. Empty source is used only by
the inspected immutable built-ins whose varying program inputs are in their
recipe key. Runtime uniforms, buffer contents and dispatch sizes do not change
those programs. Cloned mutable-source definitions share the cache but retain
their own source in the key; changing code/op_code or mode cannot reuse the
old program solely because resource metadata is unchanged. Map/Stencil retain
their existing sorted secondary key policy.

The table establishes current identity completeness; it does not clear the
remaining owner migration from positional recipe tuples and primitive modes
to the agreed named records/types.

### Constructor conflicts and mesh recipient

Broadcast, MatMul, Reshape and Transpose each retain exactly one canonical
ComputeShader::new/ComputePipeline::new construction inside the successful
cache-build closure. No removed build_compute_pipeline forwarding API or
pre-cache native construction was restored. Cache hits therefore do not
compile again merely because of conflict resolution. Result conversion still
preserves the concrete producer error.

GpuMesh displacement now uses a per-device/source/policy source cache rather
than the old global first-device OnceLock<ComputePipeline>. The new closure
propagates shader/pipeline failure instead of unwrapping it. The displacement
shader, parameter assembly, source buffers, output allocation and dispatch
arithmetic are otherwise unchanged. Its remaining count/stride work is not
cleared by the cache correction.

ComputeSourceCache is re-exported from its lowest shared GPU owner; existing
compute prelude and mesh imports reach it without another cache/type owner.
Remaining HashMap imports in Map/Stencil still serve actual secondary-resource
maps; removed cache imports leave no unqualified RwLock/HashMap consumer behind
in the reviewed delta.

## Folded-index and preserved core receipts

Kernel/mod.rs changed cache/import ownership and the authorized folded-index
constant/documentation only. Kernel construction/run, KernelBatch and every
fast/static uniform method retain the receiving 3ff core implementation.

The folded helper now returns u32::MAX for zero grid/width or an unrepresentable
padded index. Its guards precede division/multiplication: row-group overflow is
checked before forming row_groups, then multiplication-plus-id.x is checked
against the remaining range. Every representable previous index retains the
same arithmetic result. The sentinel is outside a u32-count live prefix, whose
largest valid index is count minus one. Actual spatial-hash helper recipients
still check their live count/table-size prefix before indexing. Full canonical
folded caller/count migration remains an explicit owner dependency, not a
blanket caller verdict from the helper alone.

The new cache's length/is_empty ports preserve their existing ordinary API
types. KernelCache no longer binds compiled kernels to only source text across
different devices. Lazy pipeline preparation, empty-first batch admission and
counter ordering, uniform slots/chunks, logical copies and after-submit
callbacks remain unchanged by this 18-path delta.

## Full changed scope and affected graph

Revalidated the complete 74-path source identity and affected native-resource
graph. The F05/F23 producer, canonical byte fields, provided-value writers,
whole-block bounds, direct nonstruct zero-only behavior, successful Mat3 order/
overlap and typed former-panic failures remain as inspected against main's
actual original packers. No expected-shape gate was reintroduced. Host address
narrowing retains its real conversion cause. The earlier invalidated c798
shape-preservation verdict is not carried forward.

Texture2d/3d/Cube cache/native/missing/device states, generic/face/array
geometry, effective selected-view facts, native error/request retention and
direct copy/read/write/clear/render recipients retain their contracts. Shader
and pipeline immutable authority, complete entry-point outcomes, resource/
sampling union, sparse groups, native comparison/multisample/storage roles,
Immediate/unsupported-resource admission, public shared assembly, static
allocation/upload and converter behavior are unchanged by the imported cache
work. Source caches feed those same constructors and consumers.

Current compute operation, parameter/buffer, mesh/BVH/physics/XPBD and retained
hair/armor/character-creator recipients remain in the affected graph. Existing
test source, cases, fixtures, helpers and assertions are unchanged by this
integration. No test result is inferred. The workflow and documented native,
browser, hardware, performance and rendering limits retain their current
source identity.

## Required unresolved families and limits

The owner-restricted three-file Reduce reference was intentionally excluded;
Reduce still has its prior source-only cache path. Required #780 named recipe
keys, cached layout records, canonical stride/count/mode/parser/Reduce and
folded callers remain open. Retained #774/F19 source and known receiving
signature conflicts remain open. These are required coordinated repairs, not
accepted deferrals or findings erased by this cache-only integration. No
competing owner repair or type was introduced by this reviewer.

Only source reads/searches, Git identity and fingerprints were used. No tests,
checks, builds, scanners, generators, runtime, source edits, push or CI occurred.
No tests/cases/helpers/assertions were added. Only this ignored report was
written; no external review was posted. After required integration, all
thirteen lenses, all three independent reviews and hosted gates must pass on
the same final pushed SHA. This local source review cannot supply that final
clearance.

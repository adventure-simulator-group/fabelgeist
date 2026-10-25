# Independent cache-integration Rule Review

No additional source rule defect was identified in the integrated cache delta.
Its source/device/policy key and publication protocol close the observed cache
identity path without overwriting the existing compute/view contracts. This is
a correct increment, not a complete migration: the named #780 producer,
recipe/layout/parser/Reduce/folded-caller families, retained #774/F19 families
and hosted gates remain required and incomplete. No final clearance is given.

## Snapshot, scope and execution limits

- Reviewed HEAD: `7dee3d5a69fa6e5132efa740d8b0f23a42cb435f`.
- Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.
- Previous reviewed checkpoint: `3ff23fc9a47fe32fe514747a17585606ea001475`.
- Main-to-HEAD scope contains 74 paths. The integration changes 18 paths;
  the other 56 are byte-identical to the preceding reviewed checkpoint.
- All 74 working files independently matched their HEAD blobs. The tree was
  clean, and all 74 `cache_integrated_wave.files` fingerprints matched.
- Independent SHA-256 of sorted changed-path/SHA-256 records, serialized with
  sorted keys and compact separators:
  `8ede28feef67dd391378965aa6111ebf3c1566b50bf1f980b96371196f93ee9a`.
- The immutable `handoff-cache-5c2d287936c62432/manifest.json` hash matched
  `cd367f9ed59b7b505a19925d5f3e96e3fa85dfe15aa6a5121a23d82cc56680d3`.

Read the Rule Review skill and bespoke-type reference, governing root and crates
AGENTS, all 18 committed integration deltas, current cache definitions and
recipe producers/receivers. Revisited current pipeline admission, native
validation, uniform reflection/packing, view selection, static assembler and
fast-path recipients. Prior read receipts describe the unchanged bodies;
their exact identities and current connections were checked independently.
This does not represent 74 new full-file architectural reads in this wave.

Only this ignored report was written. No source changes, tests, cases, test
helpers, assertions, doctests, project checks, builds, scanners, generators,
runtime, push, hosted CI or external comments were performed. Read-only Git,
source searches and fingerprint arithmetic establish source identity only.
There are no test-named changed paths or added test attributes/assertions in
the 18-path integration. Previously changed PR tests/fixtures retain their
previous bytes; this statement does not claim they are absent from the PR.

## Integrated source contracts

The canonical owner is `gpu/compute/source_cache.rs:17-69`. Its private named
key retains an actual cloned `wgpu::Device`, effective synchronous native
observation predicate, authored source and recipe parameters. SDK
`wgpu-29.0.4/src/api/device.rs:26` supplies identity through the dispatch-device
Eq/Hash proxy. This is neither an Arc-address surrogate nor a second device
identifier. Context wrappers sharing a device can share entries; devices on
the same adapter remain distinct. Device features/limits belong to that
immutable native device and need no separately mutable cache classification.

`observes_validation` is the actual native-boundary predicate used by pipeline
creation: false on Wasm, otherwise the context's blocking-validation setting.
It is an ordinary named predicate in a private native cache key, not a public
Boolean role argument. Source is authored WGSL text and remains open text.
The generic cache's empty source token is used only by fixed built-in recipe
families whose variable inputs are all in their parameters. It is not an
independently configurable source sentinel consumed by the shader parser.

The operation checks under a read lock, releases it before construction,
propagates construction failure through `?`, then publishes under a write
lock. Racing successful builders return the first published value. Failed
builders do not publish a success or replace an existing entry. Read/write
poisoning yields the existing ComputePipelineError::CachePoisoned rather than
panicking. `len` explicitly handles poisoned state by reading its count; that
does not expose an entry through the fallible build/lookup API. Failed native
construction preserves the canonical typed error and its actual cause through
the existing anyhow conversion; the cache does not format or reclassify it.

The generic cache cannot independently inspect arbitrary Value provenance.
The reviewed recipe callers establish that relationship by constructing with
the same context/source/parameters used for their key. Their private cache
fields prevent independent replacement of a definition's native entries.

| Recipe source producer | Key and retained consumer trace |
| --- | --- |
| Advect `advect/mod.rs:194-210` | Fixed template plus velocity, quantity and output descriptors and current mode reach the key; the same arguments and definition build the native pipeline inside the closure. Changing mode creates a distinct entry. |
| Divergence `divergence/mod.rs:131-144` | Fixed template plus input/output descriptors and boundary mode reaches the cache; the same inputs and mode drive generation. |
| Gradient `gradient/mod.rs:147-165` | Fixed template plus velocity/pressure/output descriptors and boundary mode reaches the cache and identical build closure. |
| Map `map/mod.rs:414-437` | Authored code plus optional input, output and sorted named secondary descriptors reaches the key. The same resource map and code produce reflected pipeline/layout information, then execute consumes that cached result. The layout tuple/byte leaves remain a required owner migration. |
| Stencil `stencil.rs:860-889` | Authored code plus input/output, sorted named secondary descriptors and boundary mode reaches the key. Every template-variable input is represented. Cached layout tuples and raw mode remain incomplete semantic migration. |
| Gather `gather/mod.rs:325-341` | Authored code and input/output descriptors reach construction and the cached pipeline/input/output-layout result. |
| Scatter `scatter.rs:313-327` | Authored code and input/output resource kinds reach construction and its cached layout result. Element types are produced from that same code. |
| Stream `stream/mod.rs:282-295` | Authored code and input descriptor reach construction and the retained immutable pipeline. |
| Scan `scan.rs:34-134` | Authored binary operation is the source key. Both generated pipelines must successfully construct before their combined result is published. Failure of the second cannot publish the first as a complete recipe. |
| Broadcast/MatMul/Reshape/Transpose | Each fixed shader literal is keyed directly with unit parameters and creates shader/pipeline inside the closure. Existing direct ComputePipeline::new construction remains; the removed forwarding helper is not revived. |
| TextureBinaryOp `texture_ops.rs:102-119` | Current mutable op_code and all three resource descriptors reach the key and matching build closure. The actual op_code spelling is preserved. |
| Kernel `kernel/mod.rs:180-204` | Exact code and context reach canonical Kernel::new inside the generic cache. The returned Arc retains that admitted static/fast native graph. No Kernel constructor, workgroup field, batch or fast implementation was replaced by the export's older baseline. |
| Mesh `mesh/src/gpu.rs:138-143,213-218` | The static cache is keyed by actual device/policy and exact displacement WGSL. Shader and pipeline creation use that same context and propagate typed failures. The prior process-wide unqualified pipeline and construction unwraps are removed. |

Complete key membership fixes stale results after authored-source mutation and
prevents cross-device or observation-policy reuse. It does not confer bespoke
unit guarantees on the raw recipe records stored beside those pipelines.

## Folded index and affected unchanged consumers

`kernel/mod.rs:41-49` now checks the native u32 multiply/add sequence before
performing it. It preserves every representable old flattened index and returns
u32::MAX for an unrepresentable padded index. Divisors are checked for zero
before division; the subtraction is bounded by the u32 invocation component.
A u32 population can address at most indices 0 through population minus one,
so the sentinel is outside its live prefix even for the maximum population.
The WGSL maximum, dimensions and arithmetic are native shader integer grammar,
not a second host quantity/type owner.

Source search found three direct consumers of FOLDED_INDEX_WGSL, all in
`spatial-hash/src/gpu.rs`. Hash at lines 90-95 and ranges at lines 133-142 reject
`i >= params.count`; clear at lines 111-116 rejects `i >= params.table_size`.
Each rejection precedes its resource access, so the overflow sentinel cannot
be used as a wrapped low index by these consumers. Width is the matching
256-invocation shader declaration in each direct recipient.

The receiving `Kernel::groups_folded` and batch item/count APIs remain raw u32
producer paths (`kernel/mod.rs:154-157,307-316`). Hair also has independent
folded arithmetic (`hair/src/gpu/wgsl/common.rs:433` and
`hair/src/gpu/wgsl/sim.rs:1500`). The shared helper increment does not migrate
these caller/producer families or establish their bounds. They remain named
owner dependencies rather than being implicitly cleared by the new guard.

## Existing core contracts preserved by reconciliation

Fresh source and exact unchanged identities establish these continuing traces:

- Naga uniform offsets/sizes/block span enter the existing BufferByteOffset and
  BufferByteLength leaves in `reflection/uniforms.rs:24-44`, remain typed in
  UniformMember/group/FastPath state, and reach checked host/SDK byte ports.
  The supplied PassParameter variant still controls encoding. Static missing
  or unsupported values skip into zero-initialized data; fast missing and
  unsupported values retain typed errors and its narrower matrix support.
  All immediate struct member metadata remains; direct nonstruct blocks retain
  extent and an empty member dictionary. No withdrawn expected-shape admission
  or direct-global-name destination has returned. Mat3 span/3 and successful
  component/overlap order remain, with typed former inner-window failures.
- Reflected global-use and sampling unions cover all compute entrypoints.
  Used Immediate input is rejected before the binding filter
  (`reflection.rs:29-66`). Sampler comparison, multisampling, image/storage
  access and unsupported resource shapes retain their actual native roles.
- Pipeline shader/device/interface state remains immutable. The pipeline's
  full entrypoint cache separates observed typed outcomes from device-handled
  outcomes (`pipeline/mod.rs:101-123,147-195`). The new outer recipe cache keeps
  the observation policies distinct. Native nonblocking/Wasm outcomes are
  governed by provider error handling; Result success is not proof of native
  observed acceptance under that policy.
- Texture2d/3d/Cube keep native allocation, cached view, logical format/usage
  and actual creating device correlated within their owner. Same-format cache
  lookup precedes allocation/context need; alternate construction requires
  allocation and native device equality. Effective selected view format,
  dimension and device reach binding validation in `texture/selection.rs`.
  The four cache/allocation availability combinations and native causes remain.
- The one immutable-pipeline static assembler checks pipeline device and uses
  admitted sparse layouts, logical buffer prefixes, selected texture facts and
  named sampler records. `bind_group/assembly.rs:15-76` still owns that complete
  relationship. FastPath/Prepared/arena/batch preserve typed byte/count state,
  actual dynamic-offset narrowing, native causes, recording and callbacks.

No core forwarding compatibility API, duplicate byte owner or duplicate device
token was introduced. The compute module now correctly places re-exports
before module declarations. New SourceCache is a focused 82-line owner with
real provider/key/publication policy, not a pass-through module. Its manual
Default avoids imposing unrelated Default bounds on generic parameters/value.
Existing derived Clone/Debug/Eq/Hash retain native key identity.

## Incomplete migrations and governing rules

These are required existing owner dependencies, not additional regressions
introduced by cache reconciliation and not accepted deferrals:

- #780 recipe/layout records: Map/Stencil/Gather/Scatter retain positional
  descriptor and pipeline/u64/u64 tuples; Scan retains two pipelines and raw
  type text as a tuple. Canonical named records and typed layout/stride leaves
  must persist through producers and execute consumers. Advect/Gradient/
  Divergence/Stencil raw modes, count/grid producers and parsing roles remain.
  `crates/AGENTS.md:28-71` requires owner reuse, typed propagation and named
  semantic records. Moving those existing tuples into a common cache does not
  fulfill that contract.
- #780 Kernel raw workgroup/count/entrypoint producer family and source parser
  integration remain required. Cache integration does not change those fields
  or validate every independent count computation. Raw arguments retained in
  fixed-recipe constructors likewise remain within that owner API migration.
- Reduce reference sources were not integrated. Current
  `reduce/mod.rs:14-16,49-55` still has descriptor-only pipeline/u64 caching and
  lock unwraps; it needs the owner's complete source/device/layout/live-prefix
  repair. `crates/AGENTS.md:126-127` prohibits production unwraps. The completed
  cache paths above do not grant clearance to this unchanged receiver family.
- Retained #774 hair/fitting/body/armor/metal/engraving and F19 cause/recipient
  families remain required. The core ownership transfer does not authorize
  independent repairs to retained whole functions. Existing hair raw-grid
  receiver adapters remain outstanding.

No manifests, registry scopes, baseline ceilings or test population changed in
this integration. Existing large definition modules still require the owner's
coherent extraction and hosted no-growth/baseline-reduction validation under
`crates/AGENTS.md:90-110` and its quality-gate rules. Source review does not
establish those numerical gates. Root README/wiki writing remains untouched;
the owning GPU guide retains the actual byte/value/native-policy contracts.

Required hosted fmt/lint/native/Wasm/project checks and baseline reductions
remain unrun here. Historical software-GPU/browser/hardware/performance and
rendering validation limits are unchanged. This report grants no native/Wasm
compilation, browser atomics or hardware execution result. Full final audit
waves, all three independent reviews and required hosted checks must still
agree on the same final pushed SHA. Do not merge.

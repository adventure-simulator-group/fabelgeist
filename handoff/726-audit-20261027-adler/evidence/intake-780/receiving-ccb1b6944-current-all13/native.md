# Native GPU receiving: all-thirteen pre-integration discovery

Root: `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator`.
Completion HEAD: `ead46619dec5ef45993f63202cb289bd3986b018` plus current
receiving changes.
Frozen incoming: `ccb1b6944d674c0bf29d10e6820c0ac4c55092b7`; composition SHA-256
`5a1670cc3065b74bd0b9ec62863d0edaee690e52f2dff9c59c60746aa4119e9c`.

This is source-backed pre-integration inventory, not integration or global
clearance. Root is sole implementer. No source edits or project execution were
performed. Historical packet reviews and searches did not establish clearance.

## Scope and receiving boundaries

91 incoming source accounts: 85 non-generated packet sources (native
GPU, two Armor sources and baseline), plus six latest shared GPU generated
signature/addressing/export leaves. Their whole source, existing inline fixtures
and documentation were read. Baseline was parsed as complete record metadata;
its 12,809 lines are not claimed as semantic prose review. Other 45 generated
files remain explicitly unread by this reviewer and independently assigned.

Exact per-path incoming/current Root hashes, extents and composition decisions
are in the companion JSON and source table below. Referenced five original
manifests are unavailable in this Root checkout; composition index receipts and
all frozen source hashes were verified directly. Original packet proof/review
claims are not inherited.

## Actionable receiving findings

### NATIVE-COMPOSE-1 [P2] — integration-required

**Location:** `crates/fabelgeist-gpu/src/data/gpu/buffer/mod.rs:5-28`;
`crates/fabelgeist-gpu/src/data/gpu/buffer/stride.rs`;
`crates/fabelgeist-gpu/src/data/gpu/buffer/capacity.rs`.

**Rule:** Canonical lowest shared owner; complete producer propagation; preserve
unrelated work.

**Source:** Incoming Buffer module adds admission_error but omits Root
capacity/stride declarations and exports. Root ScalpTextureLayout and Hair
rod/collision producers consume BufferElementStride/BufferElementCount;
SortScratch.ensure returns BufferCapacityChange. Blind replacement disconnects
existing canonical owners.

**Connected repair:** Compose the import/reexport/module union: retain capacity
and stride exactly; add admission_error and intended
MappingExtent/MappingRequest/Readback crate-GPU visibility. Copy new
admission_error.rs; do not create another count/stride/capacity owner or
adapter.

**Acceptance:** Current Root callers resolve exact original types; only the
three old local rejection roots change representation; all buffer
allocation/write/prefix/readback bodies retain byte/zero/device/poll contracts.

### NATIVE-COMPOSE-2 [P2] — integration-required

**Location:** `rust-quality-baseline.toml`.

**Rule:** Apply only relevant reductions; never raise or resurrect ceilings.

**Source:** Root has 2059 records (208 files,482 functions,1369 findings),
incoming 2055 (207,479,1369). Incoming removes seven exact old native records
but adds three already removed in Root: Hair gpu/sim.rs650,
DualContouringDefinition::new442, MarchingCubesDefinition::new224. Its
2055-survivor statement is relative to donor baseline, not current Root.

**Connected repair:** Remove only seven exact native records from current Root;
preserve all other current Root records unchanged, including its three
preexisting reductions. No whole incoming baseline replacement.

**Acceptance:** Result2052 records (206 files,477 functions,1369 findings),
seven removals/no additions/no ceiling changes against this Root; final
authoritative hosted baseline export still required.

### NATIVE-R1 [P2] — incomplete-semantic-migration

**Location:** `crates/fabelgeist-gpu/src/data/gpu/texture/format.rs:124`;
`crates/fabelgeist-gpu/src/data/gpu/texture/texture_2d/mod.rs:82-102`;
`crates/fabelgeist-gpu/src/data/gpu/texture/texture_3d/mod.rs:56`;
`crates/fabelgeist-gpu/src/data/gpu/texture/texture_cube/mod.rs:61-81,167,280`;
`crates/fabelgeist-gpu/src/data/gpu/texture/texture_2d/image.rs:94,156`;
`crates/fabelgeist-gpu/src/data/gpu/texture/texture_view/mod.rs:251`;
`crates/fabelgeist-gpu/src/data/gpu/texture/texture_cube/image_data.rs:65`.

**Rule:** Closed local error classifications have typed owning errors; native
source causes remain source errors.

**Source:** These actual constructors/conversions/reads reject known local
states with static strings or anyhow prose while adjacent native
buffer/view/pass errors already carry closed roles. For example unsupported
PixelFormat conversion is a public static-str error, and invalid native
size/unsupported read format cannot be downcast by semantic role.

**Connected repair:** Introduce/reuse focused texture-format, existing
size-admission and image-conversion error roles with payloads; convert into
existing heterogeneous anyhow boundary as needed. Preserve Display text/error
precedence and direct Naga/wgpu/IO error chains. Inventory every actual caller
before changing native extent APIs. No new float policy, size type, clamp,
exact-byte admission or broad resource framework.

**Acceptance:** All enumerated existing guards return inspectable named roles
with relevant original dimensions/format. 3D negative/zero max(1) policy,
2D/cube casts and min/max order, uploads trailing-byte behavior, and
Image::from_pixels acceptance remain unchanged. No SDK failure is replaced by
message-only cause.

### NATIVE-R2 [P2] — incomplete-api-and-record-migration

**Location:**
`crates/fabelgeist-gpu/src/data/gpu/compute/texture_ops.rs:7-10,126-179`;
`crates/fabelgeist-gpu/src/data/gpu/texture/texture_2d/mod.rs:add_raw,mix_raw`.

**Rule:** Closed supported roles in API types; named semantic records; no
compatibility adapters.

**Source:** TextureBinaryOp::execute takes three GpuResource values but rejects
every non-Texture2d variant with three prose errors. Only two actual callers
already own Texture2d references. Cache key is positional input-A/input-B/output
descriptor triple.

**Connected repair:** Change direct execute signature to Texture2d references,
migrate its two callers directly, remove impossible variant branches and
duplicate wrapping. Replace tuple cache key with named descriptor record. No
general-resource overload, temporary primitive adapter, or second
implementation.

**Acceptance:** Native generated math, input ordering, existing amount float
behavior, format descriptors, effective validation policy and device/source
cache identity unchanged; wrong dimensionality unrepresentable through this
operation.

### NATIVE-R3 [P2] — duplicate-canonical-owner

**Location:**
`crates/fabelgeist-gpu/src/data/gpu/texture/texture_cube/image_data.rs:17-21,50-54`;
`crates/fabelgeist-color/src/transfer.rs:122`;
`crates/fabelgeist-gpu/src/data/gpu/texture/srgb_converter.rs:15-19`;
`crates/fabelgeist-color/src/wgpu.rs:203-220`.

**Rule:** Reuse canonical shared numerical owner; distinguish native shader
boundary from Rust calculation.

**Source:** Cubemap conversion repeats scalar sRGB decoding twice despite direct
Color dependency. Color also exposes enabled wgsl_transfer_functions; converter
repeats linear_to_srgb. Canonical WGSL uses max(linear,0) in pow branch; local
converter omits it. Existing GPU Cargo enables Color wgpu feature.

**Connected repair:** Reuse Color scalar srgb_to_linear for cubemap Rust
branches, retaining clamp/truncation, alpha and BGRA swaps. Reconcile WGSL owner
and negative/nonfinite branch semantics explicitly before using shared emitted
transfer source; no second independently maintained transfer formula. Do not
substitute Image::packed_for, whose no-transfer byte contract differs.

**Acceptance:** Actual positive RGB/encoded bytes and alpha policies preserved;
incomplete input RGBA groups retain old handling; native shader
workgroup/dispatch/layout stays exact except intentional shared function
composition; source proof accounts for finite-negative/nonfinite select behavior
and shared owner dependency.

### NATIVE-R4 [P3] — result-alias-style

**Location:**
`crates/fabelgeist-gpu/src/data/gpu/compute/signature/texture_value.rs:32`;
`crates/fabelgeist-armor/src/gpu/staging.rs:71-75,110`.

**Rule:** Trait error parameters explicit; cohesive ordinary same-error APIs use
owning alias.

**Source:** TextureValue TryFrom return is TextureValueResult<Self> although
trait sets Self::Error. StagedResults prefix/status are cohesive
same-StagingError ordinary methods with explicit repeated Result.

**Connected repair:** TryFrom uses Result<Self,Self::Error>; retain
TextureValueResult only for actual ordinary APIs. Add/use StagingResult<T> for
prefix/status, preserving GenerateError-returning ArmorGpu methods as their
distinct family.

**Acceptance:** Signature spelling/declaration only; no error widening/source
change, new helper, fixture or assertion.

### NATIVE-R5 [P3] — item-order-and-derive

**Location:** `crates/fabelgeist-gpu/src/data/mod.rs:1-13`;
`crates/fabelgeist-gpu/src/data/gpu/mod.rs:1-17`;
`crates/fabelgeist-gpu/src/data/gpu/parameters/mod.rs:1-28`;
`crates/fabelgeist-gpu/tests/texture_views.rs:Method-and-record-declarations`.

**Rule:** Imports/reexports before modules; structs before enums; derive
equivalent traits when contracts/bounds permit.

**Source:** Three modules put mod before imports/reexports; texture fixture
Method enum precedes structs. PassParameters manual Default is exactly empty
IndexMap through new, without extra generic constraints.

**Connected repair:** Move complete declaration/attribute/doc blocks into
repository order, preserving cfg and visibility; derive Default on
PassParameters and keep new behavior equivalent. Fixture changes limited to
ordering existing blocks.

**Acceptance:** Exact inverse whole-block source comparison; no fixture
case/helper/assertion creation. Do not derive Texture2d/3d/Cube Default: pinned
wgpu TextureUsages lacks Default. Do not derive generic ComputeSourceCache
Default if it adds Parameters/Value bounds. GPU has no direct thiserror
dependency; manual Error impls are not silently derivable through a transitive
dependency.

### NATIVE-DOC-1 [P3] — durable-doc-scope

**Location:** `crates/fabelgeist-gpu/texture-views.md:192-198`.

**Rule:** Root AGENTS: iteration observations/verdicts/transcripts in ignored
output; durable guides describe usage/contracts.

**Source:** Final guide paragraph records earlier software Vulkan observations
and current acceptance-run workflow.

**Connected repair:** Keep durable runtime scope and documented limits, retain
actual observations in ignored local report rather than durable guide history.

**Acceptance:** No claimed hardware/browser/runtime validation inferred from
this review or old run; useful API documentation retained.

## Preserved source contracts

- Buffer logical extent is distinct from physical allocation/mapping. Zero
  prefix is valid and still checks membership/logical/physical fit. Readback
  rejects zero-sized/partial host elements before mapping; reserves before
  copying. Actual allocation, integer, poll, map, cancellation and SDK causes
  remain typed/source-preserving. Row pitches and mapped alignment are native
  layout facts, not another semantic population.
- Native callbacks observe a ready/canceled receiver before polling; browser
  progress yields the event loop. Completion observes exact marker submission
  on native and callback delivery on browser; it does not certify device health.
  Context effective blocking policy remains authoritative, including actual
  from_parts policy and later user configuration.
- ValidationScope preserves captured native error and progress error fields.
  Pipeline caches native rejection, retries transient progress failure, and
  revalidates previously unchecked state if requested by effective policy.
  ComputeSourceCache keys actual device/policy/source/parameters and publishes
  successful builds only. No progress failure becomes permanent cache truth.
- Reflection preserves sparse bind-group indices, used compute-entry resources,
  actual sampling filtering, Naga member offsets/spans, and unsupported native
  categories as typed errors. Static uniform packing follows the provided
  PassParameter variant and whole-block skipping contract; it is not replaced
  by speculative strict declared-schema packing.
- Textures retain private actual allocation/device/format/extent/usage facts.
  Cached-view-without-allocation is valid. Cached-first view selection and
  missing-source-before-destination precedence are deliberate. Selected view
  carries native handle/effective format/dimension/device, and bind checks those
  selected facts. Cube, face and array view dimensions remain distinct.
- Workgroup count/grid/shape, invocation count, record count, byte extent and
  stride remain distinct owners. Native folded-index proofs use wide arithmetic
  and distinguish largest address from population. LinearIndex may admit
  u32::MAX+1 population when the largest index fits; LinearPopulation may not.
- Shared native signature leaves retain open authored WGSL type identifiers;
  unknown Custom text is not a closed enum. CoordinateParameter admits exact
  supported coordinate vector shape, TextureValue owns numeric lanes/padding,
  and native TryFrom/strum source errors are not stringified.

## Sequential all-thirteen inventory

### 1. producer-propagation

**Contract and source account:** Read authoritative byte/count/stride -> actual
scalp layout/readback, Hair rod/collision and SortScratch producer excerpts ->
Buffer writes/reads -> pass/native WGSL uploads. Typed launch
WorkgroupCount/Grid/Shape and InvocationCount retain distinct roles. Six final
shared GPU generated leaves replace older native signature views.

**Disposition:** PARTIAL: native source inventory complete; integration and
R1/R2 propagation still required. Generated45 and all algorithm producer
families are not cleared.

**Connected repair:** Compose Root count/stride exports, direct Texture2d
API/callers, final generated exports without aliases.

**Dependencies:** Root capacity/stride and generated51 peer integration;
unchanged callers are explicit selected excerpts, not full algorithm verdict.

**Acceptance:** No raw adapter loops; actual canonical type identity through
retained data and terminal native encoders; generated peer review and joint
receiving review required.

### 2. existing-type-owners

**Contract and source account:** Root BufferByteLength/Offset and
ElementCount/Stride own native extents/populations; native Pass owners own
dispatch; Color owns Rust/WGSL transfer; TextureFormat/selected view metadata
own native format.

**Disposition:** ACTIONABLE owner reuse R3; mandatory exports union.

**Connected repair:** Reuse exact Color transfer and existing canonical counts;
reconcile WGSL behavior before substitution.

**Dependencies:** Color feature wgsl owner enabled by current GPU manifest;
shared native functions peer-owned require exact agreement.

**Acceptance:** No second count/grid/stride/transfer implementation or temporary
primitive wrapper; actual dependency API consumed.

### 3. admission-contracts

**Contract and source account:** Full read Buffer new/write/from_upload, logical
binding/prefix, MappingRequest, image/texture constructors, selected-view
device+format admission, pipeline reflection/native compile. Errors typed for
bytes/poll/map/channel/native reflection; closed texture local guards still
prose.

**Disposition:** ACTIONABLE existing admitted-state classifications R1; no
invented size policy.

**Connected repair:** Typed local format/NaN/current size and unsupported
conversion roles; preserve current rejection sequence and native causes.

**Dependencies:** Actual native float extent policy distinct pending producer
inventory; native SDK errors intentionally heterogeneous.

**Acceptance:** Same input acceptance/clamps/casts/trailingbytes/error ordering;
no infallible panic path introduced.

### 4. invariant-operations

**Contract and source account:** Checked byte offset+upload length; row
pitch/total copy arithmetic wide and checked; record count*stride wide;
WorkgroupGrid folded index proof uses u64; buffer reads reserve before mapped
copying; cache publication only success.

**Disposition:** Native arithmetic source accounts present; baseline merge and
R3 semantics unresolved.

**Connected repair:** Preserve exact existing arithmetic and color conversion
channel/rounding policies while composing.

**Dependencies:** No execution; exact SDK layout/device rules terminal; broader
physical/native Grid algorithms outside reviewed excerpts.

**Acceptance:** No wrap/narrow/partial publish regression; baseline seven
removals only; sources+hosted checks later.

### 5. aggregate-states

**Contract and source account:** Selected native views pair handle/effective
format/dimension/device. Private texture metadata and allocation/cache states
permit cached-view-without-allocation deliberately. Staging session Arc plus
ordinal proves member before prefix. MappingRequest owns logical/mapping
extents.

**Disposition:** No new invalid-state counterexample in inspected records; R2
general resource signature is avoidable state space.

**Connected repair:** Use typed Texture2d operation and named descriptors;
retain valid texture allocation/cache combinations.

**Dependencies:** Do not redesign valid cached-first branch into stricter
construction; direct caller/data membership checked.

**Acceptance:** Foreign ticket rejects before prefix arithmetic;
source/destination cache-absence priority preserved; named cache roles stable.

### 6. type-ownership

**Contract and source account:** GPU owns SDK buffer/texture/pass/pipeline and
resource native schema. Root retains logical ElementCount/Stride/Capacity
extensions. Color owns color transfer; Armor staged-session ownership belongs in
staging.

**Disposition:** Integration boundaries explicit; no wholesale replacement
approved.

**Connected repair:** Union canonical exports and add focused local errors in
actual owning modules.

**Dependencies:** No direct thiserror in GPU; adding dependency requires
coordinated manifest/locks, not assumed derivative availability.

**Acceptance:** One lowest owner per meaning; no NativeBuffer facade, duplicate
context, count or generated signature type.

### 7. redundant-authority

**Contract and source account:** Pipeline immutable shader/reflection + cache
distinguishes authoritative native rejection from transient validation progress;
ComputeSourceCache identity actual device/effective policy/source/parameters.
Texture selected metadata private. Signature ShaderParameter keeps name/type
once; exact original program source remains source.

**Disposition:** Cache/source facts verified locally; duplicate transfer
formulas and tuple cache owner remain.

**Connected repair:** Named descriptor key; one canonical transfer source.
Preserve source cache failedbuild retry and pipeline progress handling.

**Dependencies:** Native provider cache/transient semantics protected; generated
parser one-vector state is peer45 scope.

**Acceptance:** No failed progress cached as native rejection/success; mutable
source cannot keep stale independently authoritative signature.

### 8. behavior-preservation

**Contract and source account:** Actual callback-ready/canceled before
nonblocking poll; browser yields event loop; completion waits exact marker
submission index only native; direct SDK causes retained. Static uniform packing
whole provided-variant block contract, absent/unsupported skip unchanged.
Texture bytes/row order and cached-first selection documented.

**Disposition:** Source-preservation acceptance recorded; no runtime/CI verdict.

**Connected repair:** Compose without overriding Root policy/progress/native
causes; reconcile Color formula behavior with explicit source proof.

**Dependencies:** Existing fixture provenance exact; no new scenarios allowed;
no native/browser/GPU execution performed.

**Acceptance:** Whole source comparisons for protected families; exact fixture
API-only origins; final joint reviews + hosted final revision gates still
required.

### 9. enum-shapes

**Contract and source account:** WorkgroupShape 1D/2D/3D admitted dimension;
Texture selection cube/face/array native roles; validation result capture
distinguishes observable/native-vs-progress policy; ResourceBaseType actual open
Custom language types.

**Disposition:** R1 closed errors/R2 supported Texture2d should use exact types;
no synthetic enums for indices/native strings.

**Connected repair:** Named closed local error roles and direct dimension type.

**Dependencies:** Actual native format names and WGSL authored strings remain
open external language protocol.

**Acceptance:** No boolean mirror of payload state; no invented closed set for
source identifiers; current valid variants/selection branches retained.

### 10. derivability

**Contract and source account:** PassParameters default is exactly empty
IndexMap; texture defaults choose TextureUsages::empty (no SDK Default); generic
cache manual default avoids bounds. Error impls preserve SDK references; GPU has
no direct thiserror.

**Disposition:** One safe derive candidate; explicit noncandidates source
backed.

**Connected repair:** Derive PassParameters Default only; retain manual cases
unless actual dependency/bounds proof changes.

**Dependencies:** Pinned wgpu-types29.0.4 TextureUsages declaration inspected;
manifests direct dependencies read.

**Acceptance:** Equivalent representation/trait bounds; no source chain loss; do
not add helper/test solely for derive.

### 11. type-granularity

**Contract and source account:** Buffer storage
spacing/count/byteoffset/byteextent and invocation/workgroup count distinct;
mapping extent/alignment belongs native I/O. Focused pipeline reflection,
texture view, readback, cube shader modules extract actual responsibilities.

**Disposition:** R2 named role cache; no novel wide framework needed.

**Connected repair:** Named inputA/inputB/output descriptor record; preserve
existing focused modules and no-growth baseline.

**Dependencies:** Meaningful native numbers at WGSL/SDK adapters not
automatically new semantic types; full extent role frontier separate.

**Acceptance:** No pass-through modules/temporary primitive adapters/parallel
generic resource system; seven relevant baseline reductions, no ceiling growth.

### 12. magic-strings

**Contract and source account:** Naga/WGSL identifiers, labels, entrypoint
names, texture format token mappings and authored source strings are actual
native language/API boundaries; local rejection prose remains closed app
classification.

**Disposition:** R1 errors actionable; durable run-history doc scope actionable.

**Connected repair:** Typed local rejection classifications; durable docs keep
API contract, historical observations ignored.

**Dependencies:** Do not branch on Display/Debug; retain actual native source/IO
context and original cause.

**Acceptance:** No new parser for human-facing prose; no replacement native
error stringification; exact Display behavior if public contract.

### 13. magic-numbers

**Contract and source account:** Native byte alignment4/map alignment/texture
row256, WGSL layout offsets/workgroup16 and channel constants are owning
protocol facts; checked arithmetic consumes byte quantities. Transfer
thresholds/exponents already Color-owned. Root record counts distinct from
launch sizes.

**Disposition:** R3 canonical transfer numerical ownership; broader raw native
extents not silently strengthened.

**Connected repair:** Consume canonical color owner; retain native
layout/workgroup constants and explicit conversions.

**Dependencies:** Native shader formatting and static provided-parameter packing
contract intentionally different from declared-schema strict packing.

**Acceptance:** No arbitrary epsilon/tolerance/clamp, no count/launch
conflation; exact shader/layout policy preserved except explicitly reconciled
transfer owner.

## Exact composition guidance

Do not copy Buffer/mod.rs, baseline, or an older GPU compute/signature module
wholesale. Preserve canonical Root count/stride/capacity declarations; consume
the latest generated six leaves/exports under their exact owner agreement.
New Buffer admission_error.rs is a direct copy candidate. Other paths marked
copy candidate are transport decisions only: actionable source findings must be
repaired and the resulting combined graph freshly reviewed before clearance.
Identical paths should remain unchanged. Root native progress/cache/provider
causes are protected receiving contracts, not an invitation to replace them.

The seven baseline removals are:

- `files`: `{"ceiling": 899, "path":
  "crates/fabelgeist-gpu/src/data/gpu/texture/texture_cube/mod.rs"}`.
- `files`: `{"ceiling": 538, "path":
  "crates/fabelgeist-gpu/src/data/gpu/texture/texture_2d/mod.rs"}`.
- `functions`: `{"ceiling": 142, "item": "TextureCube::render_shader_raw",
  "path": "crates/fabelgeist-gpu/src/data/gpu/texture/texture_cube/mod.rs"}`.
- `functions`: `{"ceiling": 278, "item": "create_bind_groups", "path":
  "crates/fabelgeist-gpu/src/data/gpu/bind_group.rs"}`.
- `functions`: `{"ceiling": 410, "item": "ComputePass::record", "path":
  "crates/fabelgeist-gpu/src/data/gpu/compute/pass/mod.rs"}`.
- `functions`: `{"ceiling": 161, "item": "TextureCube::blit_texture_to_face",
  "path": "crates/fabelgeist-gpu/src/data/gpu/texture/texture_cube/mod.rs"}`.
- `functions`: `{"ceiling": 149, "item": "TextureCube::render_face_shader_raw",
  "path": "crates/fabelgeist-gpu/src/data/gpu/texture/texture_cube/mod.rs"}`.

Incoming-only entries that must not be resurrected:

- `files`: `{"ceiling": 650, "path": "crates/fabelgeist-hair/src/gpu/sim.rs"}`.
- `functions`: `{"ceiling": 442, "item": "DualContouringDefinition::new",
  "path": "crates/fabelgeist-compute/src/dual_contouring/mod.rs"}`.
- `functions`: `{"ceiling": 224, "item": "MarchingCubesDefinition::new", "path":
  "crates/fabelgeist-compute/src/marching_cubes/mod.rs"}`.

Expected current-Root result after only seven removals: 2052 records. All 1,369
semantic findings remain byte-equivalent as parsed records; this is metadata
comparison, not an authoritative checker/baseline execution.

## Existing fixture provenance

Frozen tests SHA58e43348ecc8c5bea2cdebc67be84a584982167fcbd3ff1ccad80aae8edb8d0c
exactly matches existing commit a03ccb5da. Original created d2e9f2f29; a03 delta
nine field-access/getter/clear API updates only; four existing
tests/cases/helpers/assertions preserved.

SHA2fa289e171e23b95b647a08e4cda05155e304750376625a01ba8fb34802af800 exact
d2e9f2f29/a03ccb5da. Absence from Root does not make peer-existing fixture newly
authored in this repair. No tests executed.

## Source accounts

| Path | Incoming SHA-256 | Current Root SHA-256 | Receiving disposition |
|---|---|---|---|
| `crates/fabelgeist-armor/src/gpu/facade.rs` | `c171604705f38aa9963c178b7b93ca9afbd001e81bca8e46c6ba9ea57e3abd3e` | `b17b68012994cda12b0932d58dc72f7a1bc5af97b9889f66c31e95deb023e3dd` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-armor/src/gpu/staging.rs` | `bdb2892c409cda525e88bc1a6466dd588a17aa09b1fe038fc2ddba1d561cc501` | `f634bc97a77a46930958b1bfe6a856a63b67522288b318907498c5bd5390db01` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/Cargo.toml` | `707cc7a7a86fd855e779b3233b4dd97d3fd2be4496aa550a4388a1dec80d8a58` | `707cc7a7a86fd855e779b3233b4dd97d3fd2be4496aa550a4388a1dec80d8a58` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/README.md` | `7e0fa5d18fe9771a86c334a4b44c5427f3daf5cfb9318ff046e51dcad2fe44b1` | `38199fb00b79762a561f7096004f7eda3542a2f1db94d8e2b223f1e46f421752` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/data/gpu/bind_group.rs` | `28255d76850a2ac982bd46ca52eda7fbe64ff6a5a2742b76f68fc27e143b25fd` | `28255d76850a2ac982bd46ca52eda7fbe64ff6a5a2742b76f68fc27e143b25fd` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/data/gpu/bind_group/assembly.rs` | `40d0f30f63f87aebf48b6bd354c40e14c66ff51410ee7586e3d2e59a98dec99c` | `955f5bb0e7dc346933dab81076091f23ba15158a268849dcc0121858fc648658` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/data/gpu/bind_group/uniform_buffer.rs` | `72423b754d3539247d4586dbe72adaa08bdd6ec741db66dcafca6b9147380e22` | `6531a6cac487b5a8649caeb8835197c208ba8b19ea8c13e152f686b0f1404a8f` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/data/gpu/bind_group/uniforms.rs` | `2301030d44c5a2b26819c3612a19732a7bfc20e6837ab7ddc0a88f96ddee06d9` | `2301030d44c5a2b26819c3612a19732a7bfc20e6837ab7ddc0a88f96ddee06d9` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/data/gpu/buffer/admission_error.rs` | `a5a41a86f6c2a1e31715329fe4a234d8d9c30dd4c972660c405e4cbeade13a40` | `None` | COPY_NEW typed local admission owner; preserve Root other buffer APIs |
| `crates/fabelgeist-gpu/src/data/gpu/buffer/binding.rs` | `dbca91c1cbc29914eb254809ceb100135006cff7381fb7050bbbd4f0085f0699` | `920c45e78aa599496843e3c49d1ae632e9f2f892991ca25f8092761cbcd8fc99` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/data/gpu/buffer/definition.rs` | `0fae346f555f78e5acee52b5631e6b36c62fb78b02daba9b9ff9df1ee590312c` | `0fae346f555f78e5acee52b5631e6b36c62fb78b02daba9b9ff9df1ee590312c` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/data/gpu/buffer/definition/tests.rs` | `990a77c5be95d46846d6f096e3a185f080d5975c1b92e6a6f831f9351db33b8f` | `990a77c5be95d46846d6f096e3a185f080d5975c1b92e6a6f831f9351db33b8f` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/data/gpu/buffer/extent.rs` | `45ea805bbc9988317e2dba4679cf6984d153c17764ae93a3204a6289ae126b77` | `45ea805bbc9988317e2dba4679cf6984d153c17764ae93a3204a6289ae126b77` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/data/gpu/buffer/label.rs` | `64c0cb1ac75e094cf6521bf61ea6ad9eef9aa1a1ab63ca34ae3d395d3a66969a` | `64c0cb1ac75e094cf6521bf61ea6ad9eef9aa1a1ab63ca34ae3d395d3a66969a` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/data/gpu/buffer/mapping.rs` | `561a74c4ded95e0500fa43dba219aa345c51cb1209b1552276a400f44fd1f8e0` | `cfae424a791de8c0101b188e604e68aa84d87482d343bdd665a36eaebaca8622` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/data/gpu/buffer/mod.rs` | `abac05c4f4b49a90348a933fb2655b9417c4e52edd52aad390e111d831118e02` | `86e926ab47cd284d07a0e14265aaf7b5ba3ce1b6cc5e2aff66328e05f67aee02` | MUST_COMPOSE canonical Root capacity/stride + incoming admission/visibility |
| `crates/fabelgeist-gpu/src/data/gpu/buffer/prefix.rs` | `bb628f72172a5e1920c433889caf7c60602ead3776b76665b2f8f60fdcfeecbf` | `62a435f0e609deb067d2c9e59481ee7e042d0ee1f16c5d662f0ea6b4bd95b4c0` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/data/gpu/buffer/read.rs` | `ff97b2b7c684745153c4713acc58e4d535cd97dd9008bcb8aacb9e62fdd138f2` | `ff97b2b7c684745153c4713acc58e4d535cd97dd9008bcb8aacb9e62fdd138f2` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/data/gpu/buffer/read_error.rs` | `a4633e3cff89760ee125ba52738e67ab345a901027da35e47ce87f04db77d20f` | `fd6c4991e13b343c21b396b8aada2480b03508500184761d217c1237bb1c52cb` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/data/gpu/buffer/readback.rs` | `9c13e1f7c57ffa369fa2139a37f7df4d163645fd351beb7fd8c18f1d2c045359` | `17ca9aa9db7efd1e261b086841babcf22764a6154628aca9405c085029b16baa` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/data/gpu/buffer/tests.rs` | `29c6416f82022dc58dc4990783ef1b94b0690777c6669e7ec50d2cbdac3d760d` | `29c6416f82022dc58dc4990783ef1b94b0690777c6669e7ec50d2cbdac3d760d` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/data/gpu/buffer/upload.rs` | `a53b251926e0429aa78e81daa6118c7a63bac8501e23868c8a89ece90351e8bd` | `a53b251926e0429aa78e81daa6118c7a63bac8501e23868c8a89ece90351e8bd` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/data/gpu/compute/mod.rs` | `c3a5b335a373a7b4cf5fd605e527fe3efe8fa25bf0d1c7ef1d5ae840555f72f4` | `7e21b07871659c611dbd1aa3db5fbcc4323a55adc4183f8aa3d1d4ba4cac84c3` | FINAL_GENERATED_OVERRIDE; preserve actual Root owner/caller composition; peer generated review needed |
| `crates/fabelgeist-gpu/src/data/gpu/compute/pass/error.rs` | `5b2fc6f775619321d1b3c76582804b16eab5dd4265aaf978de577279ab8204c5` | `5b2fc6f775619321d1b3c76582804b16eab5dd4265aaf978de577279ab8204c5` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/data/gpu/compute/pass/grid.rs` | `4a34f1d75e228c0e8f5c9012eda4a8cdd7b99e79aebc3ef53cfaf061ebdf3ac6` | `4a34f1d75e228c0e8f5c9012eda4a8cdd7b99e79aebc3ef53cfaf061ebdf3ac6` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/data/gpu/compute/pass/mod.rs` | `557fb91ec43eb39d8981f605e1d8d35636c612546252ec93e6c15769e89ed192` | `b9123559b221216745a20cd4f2881f91601174ffa8a6da1d51f07dc656c4cbe3` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/data/gpu/compute/pass/shape.rs` | `7998ee67967dc4ea5747e1396944628c1e9e89060c350d0e75d2a91bb85c81f2` | `7998ee67967dc4ea5747e1396944628c1e9e89060c350d0e75d2a91bb85c81f2` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/data/gpu/compute/pass/shape/native-cases.json` | `842b52b27499c33ebc45e4f2d85d239e0f5498a906056b0a942757b4f4aaabc3` | `842b52b27499c33ebc45e4f2d85d239e0f5498a906056b0a942757b4f4aaabc3` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/data/gpu/compute/pass/shape/tests.rs` | `b869b3d96cf3ddf562c56498e25d77101f2c38fbba71dec4d57ad98152036aa5` | `b869b3d96cf3ddf562c56498e25d77101f2c38fbba71dec4d57ad98152036aa5` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/compute_shader.rs` | `3939e013bf718d8d9dc1fffc7ac38cb49410a0c0c3c030801600687fb08f605d` | `f05e7916ac7e0e1474973913f4628d94cd99a9ca69d900d1306101f3286ce169` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/entry_point.rs` | `2d646038056780254cfafb4d59d6ba66963cf820ca662df217ed58ff7527bde7` | `2d646038056780254cfafb4d59d6ba66963cf820ca662df217ed58ff7527bde7` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/error.rs` | `cfb85359fdce3b2659be668874700f775c921db354a3b81c854d035a61afdf5e` | `9ad9d82531693820aff5a33f0b7ee8afdefa9ef4fb790fc87b77977cb9e3fb04` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/mod.rs` | `14f70d6c8f3fe4d7bf74bde72f8c09fba8f31484c2454f5ba369c52384e94827` | `13cfda2ca90d3fb38739dd6564b412dfb99af3289242b9e277ee8e840baee8d3` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/reflection.rs` | `f2d4fed56c6f544be8a9d7ec807f5cd71d1c2abc29a9235706f364efe88ccc5b` | `28875e4c667ce2a16c41e3640b8948fcba6f90e399b45d649ddb43f0832f8b90` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/reflection/images.rs` | `3130b402b58ceb84fa372a2d3b4108517795df791998c26dded8b5bed8d5d145` | `3130b402b58ceb84fa372a2d3b4108517795df791998c26dded8b5bed8d5d145` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/reflection/uniforms.rs` | `7c79e1c26e7c308c8e8a8e041db1764eeb135a9b2ac7a034c72cc979543d536c` | `7c79e1c26e7c308c8e8a8e041db1764eeb135a9b2ac7a034c72cc979543d536c` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/data/gpu/compute/resource_addressing.rs` | `59b58c24ca9681cbb2a48ec68854ff407ea61398fb50d85d83e1d58c34868330` | `3fc5be3aee86a4b639de572e8309ffeea38a34d0157e5533252e9b15919e9503` | FINAL_GENERATED_OVERRIDE; preserve actual Root owner/caller composition; peer generated review needed |
| `crates/fabelgeist-gpu/src/data/gpu/compute/signature.rs` | `1c9dc99c0f186c95668f0b42fffcbc51b94bd822a907476d449f51e38570db01` | `5cb92b1488c3e0ea14c98488795775c6fe4c190f5c60af878f5a96a36a867a92` | FINAL_GENERATED_OVERRIDE; preserve actual Root owner/caller composition; peer generated review needed |
| `crates/fabelgeist-gpu/src/data/gpu/compute/signature/coordinates.rs` | `709f96bf90a39b588f07d8dcee684d59de96937a1499cbd2f06d2c235a323ea5` | `505a16459267c2280842514d42eb3aded371b6a2a84584ba6faf9ebb11c4ca2f` | FINAL_GENERATED_OVERRIDE; preserve actual Root owner/caller composition; peer generated review needed |
| `crates/fabelgeist-gpu/src/data/gpu/compute/signature/native_operator_error.rs` | `c21f0a10ae14401b8f7b364c14f98de4e628de2dc5605d82e205fec09c9af99e` | `None` | FINAL_GENERATED_OVERRIDE; preserve actual Root owner/caller composition; peer generated review needed |
| `crates/fabelgeist-gpu/src/data/gpu/compute/signature/texture_value.rs` | `79124150e675e45b70b38b8287f82d80b7eda48202e0d2fa6e24f880bb153de6` | `79124150e675e45b70b38b8287f82d80b7eda48202e0d2fa6e24f880bb153de6` | FINAL_GENERATED_OVERRIDE; preserve actual Root owner/caller composition; peer generated review needed |
| `crates/fabelgeist-gpu/src/data/gpu/compute/source_cache.rs` | `686fa8c6510d57714063bb26f287a16c170e690ea25695abd94078041e1d9836` | `686fa8c6510d57714063bb26f287a16c170e690ea25695abd94078041e1d9836` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/data/gpu/compute/texture_ops.rs` | `99a525ad033c159eccb7ed9ed9202f631c1087f978370b98736936d0294c58f6` | `6374f05b0a75509d506cf5af75fd6716cbfba40a8764bfda36bbbc1df8f7cfbe` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/data/gpu/mod.rs` | `ba5bd199bd66da1a3060421b34e228b33f7a8d86f5c4c8611c3a30cf6305c45d` | `ba5bd199bd66da1a3060421b34e228b33f7a8d86f5c4c8611c3a30cf6305c45d` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/data/gpu/parameters/mod.rs` | `7ad0f885d284243d61ff532ba8151cf91916f1535ff64a9c8e29a5e855f128b0` | `7ad0f885d284243d61ff532ba8151cf91916f1535ff64a9c8e29a5e855f128b0` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/data/gpu/parameters/parameter.rs` | `8057e3bc54fcc2941ab0b231989785cbbff9c222088d96a10920c55572a4bf24` | `8057e3bc54fcc2941ab0b231989785cbbff9c222088d96a10920c55572a4bf24` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/data/gpu/resource.rs` | `d80964b752b34346f83e75fb30110cc692ac0a3a4a3283c4a09185d85175bda4` | `d80964b752b34346f83e75fb30110cc692ac0a3a4a3283c4a09185d85175bda4` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/data/gpu/sampler/address_mode.rs` | `604de4f2bb1105ea0d729780e93ce8af38aa16343cf8a7021d4315391e98d5fa` | `604de4f2bb1105ea0d729780e93ce8af38aa16343cf8a7021d4315391e98d5fa` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/data/gpu/sampler/filter_mode.rs` | `ac95b5629dc262b432ed5b277ac88a70f861a93828c0744041994035d9a80d77` | `ac95b5629dc262b432ed5b277ac88a70f861a93828c0744041994035d9a80d77` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/data/gpu/sampler/mod.rs` | `f0466918aad4e1e36aedb3586691bbe8104055d86a85a6e51e67fba76e84b198` | `acfa2a3f08dd06fa8ddef5f8b981f0f706632cc3619c6914db1b0ce36f13e8f4` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/data/gpu/shader.rs` | `04677894e5258a965c2b90cf1d252b8690984b62192dbea9e9e4882dd513dbe5` | `04677894e5258a965c2b90cf1d252b8690984b62192dbea9e9e4882dd513dbe5` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/data/gpu/shader/error.rs` | `144e811d92cebbfe99af991eec710ecb229048db20c4c7ec3a109c76941442bc` | `144e811d92cebbfe99af991eec710ecb229048db20c4c7ec3a109c76941442bc` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/data/gpu/texture/cube_face.rs` | `db49ac57de817ae29d6341288c341aacbe884102e4d98575a32590f334f90709` | `db49ac57de817ae29d6341288c341aacbe884102e4d98575a32590f334f90709` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/data/gpu/texture/format.rs` | `3cdc160eca2363969ca49ee4c5f8c1f1c05b78038f6c1df887d56b457510811b` | `3cdc160eca2363969ca49ee4c5f8c1f1c05b78038f6c1df887d56b457510811b` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/data/gpu/texture/format/storage.rs` | `5c67a62164329d7fc2a0d7b96fb4eac6e7cc7317bb6c122ba671ddc95c40335b` | `5c67a62164329d7fc2a0d7b96fb4eac6e7cc7317bb6c122ba671ddc95c40335b` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/data/gpu/texture/mod.rs` | `7fb81a38c012a749919c9addd0d99a42386e333e227042d87bd58c270dc5b9b7` | `af62bc5f779bdd68942676c201137eec786b0c405216c1626b4b414cab2fe98f` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/data/gpu/texture/readback.rs` | `0e8f2228f77e50b2c97529d2c473639157d1c24b87f15c02feb8f24ed1f95500` | `None` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/data/gpu/texture/selection.rs` | `8a600e85d0724eb9f9cfb46e9c6d781eec1186c336335f266c7cff2a7e5cc661` | `8a600e85d0724eb9f9cfb46e9c6d781eec1186c336335f266c7cff2a7e5cc661` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/data/gpu/texture/srgb_converter.rs` | `8bd4662da8129260f8e3b6c657db2321d45ae92bc884f1fab394649fe0bc7b41` | `339eb4bc4fecb2b3bf33e8b486b0a42214398ce4a391687745cce49464b267d9` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/data/gpu/texture/texture_2d/image.rs` | `a67548746be5f5abcd6d774695abff5b2b8bd8fb0ecab8628080f4a60aebf8e8` | `b7a403ce0f4a856e97457918a1b1fe9a0d4e34fc8365ce806107ff48a6d4d049` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/data/gpu/texture/texture_2d/mod.rs` | `e7408782d1ee0fc74edb3340badaa4f0acab3f3a1c02e09409b07a08dabfa758` | `537cbf39f00bc36c4f46ff60a6b550037b283eedafcf41d3975274e5f3e61184` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/data/gpu/texture/texture_2d/views.rs` | `0d8100e52a87510bab94ff9d3cad2d5428152ab3e3d74372376a82df3275881a` | `a084bd9266444ce4ed50458bfac88cc19e31ba899933eea8ad1ec20f176934ac` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/data/gpu/texture/texture_3d/mod.rs` | `0ff0d8b9fd6f211ea8a6201e62fbf88efbbe2ca000c60c7f31a1a9b3d12c9f2b` | `31fcc315c4314f2d63153aac4b2db696eae5a02d5ce7ae97881460e06a251295` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/data/gpu/texture/texture_3d/views.rs` | `d65e7afd1fd3c33a56a8df0661712768e562d6665e7e61eea7f61cd9ccdf4fee` | `2a09fff5d2d38ccd60190e1d41cd806f8298d3c40cb27bccb2a7844febdc7952` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/data/gpu/texture/texture_cube/blit.rs` | `d6499e275f4e4e5d03e9cfbfa6eb84673080435f48c7daf92843ccccef45feeb` | `None` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/data/gpu/texture/texture_cube/image_data.rs` | `d03d1c6cb9ab00b4aae865671847228dc2a59bd637c635a5f2b3538d7423dfd5` | `f87b692b2940ed4e0d4cab594bf78080bb75520040d9708e518977570b2edc42` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/data/gpu/texture/texture_cube/mod.rs` | `4c556d85a608359f62329c02108249070430615678b25adfaadf761bf08fed9f` | `1b980d4e70b78653c3c8c246e2cf7da33f4bc5cec208060e5ef4acbf059ba24e` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/data/gpu/texture/texture_cube/shader.rs` | `297719623b7aaa94d52e0aff52a5aa81bd825608020ef7dd27f6bbc575033729` | `None` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/data/gpu/texture/texture_cube/shader/source.rs` | `4f45dd45651aac781e71120454d88879e337bd5e1180fc622198cf296ad35326` | `None` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/data/gpu/texture/texture_cube/views.rs` | `cd05d6421bbca3cbc7692ed4fe9c5ff5d730a7da827247ab70f58f15a7d89d98` | `1a0b48f936366ef8573be58c96d41f8a0d591cae174944a7341e0d7812cf4f95` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/data/gpu/texture/texture_view/mod.rs` | `c30ddad210dd9cbbacce5e1d8a1039aedd92a3a0d1a0a8f55adb7ab9330afdcb` | `8582bd7ded85294c4a99ebc90c39a80ec4d6eeb00c9a711b6f3ec29dba691f17` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/data/gpu/texture/transfer.rs` | `3aa3c6b11409ec3885e7a8d509d0a98f2b393066ed1ccd9ad21123e6eb694947` | `None` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/data/gpu/texture/view_error.rs` | `79e4c931540af7e78e64ccb4819d93997800e554add2de78141df25c11f8040b` | `8c69fd43e818b86bee07c975f463afbbf8d62250a9a86cd65a995237d9ba1dcc` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/data/mod.rs` | `7a84c8ef7c760fd6292dac52b88418a52a7e4831a5a3579e05d9fd19e4e856ad` | `7a84c8ef7c760fd6292dac52b88418a52a7e4831a5a3579e05d9fd19e4e856ad` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/globals/callback_progress.rs` | `6edc84206e79466f1370de25ce9a19d46d9f94d6b209b1c77c42246d8c2ea961` | `6edc84206e79466f1370de25ce9a19d46d9f94d6b209b1c77c42246d8c2ea961` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/globals/mod.rs` | `ea88edf6a0bafded0eb2036e091a1d30a9d027ba247359ee5024983a77cce77c` | `ea88edf6a0bafded0eb2036e091a1d30a9d027ba247359ee5024983a77cce77c` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/globals/validation_scope.rs` | `78737ed29bb25ecca79c079f6676afa0719fa8bfe65a9051b56536de280cfd63` | `989b1f28cf9f2db331771b2d8fd06fe35a9ee86082b89b80d5c501577edb3202` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/globals/wgpu_context/blitter.rs` | `ff763087b001d957a2d8bcb14bbd529058804ab1338ee45433b279f15ed8fcaa` | `ff763087b001d957a2d8bcb14bbd529058804ab1338ee45433b279f15ed8fcaa` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/globals/wgpu_context/completion.rs` | `0bf57e56179c36d0bf533b46daf1971b4037363b98505b14b55cf6c4ff2f4fdf` | `0bf57e56179c36d0bf533b46daf1971b4037363b98505b14b55cf6c4ff2f4fdf` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/globals/wgpu_context/initialization.rs` | `84acca71e2e173e327d3bb0b2bca612588b4430bae0942a98a9ad5a312d34398` | `84acca71e2e173e327d3bb0b2bca612588b4430bae0942a98a9ad5a312d34398` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/globals/wgpu_context/initialization_error.rs` | `d923a37d0740fed7a02157adbe582af9f76e2f1d7d7db6cd2ee7fdb9ec22f8f4` | `d923a37d0740fed7a02157adbe582af9f76e2f1d7d7db6cd2ee7fdb9ec22f8f4` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/globals/wgpu_context/mod.rs` | `5903e47b681dbb1deeee4219ccf3a10bdb56776d681ab78edd500aa515a5f6c1` | `dde02c3c137d5ac3722f28e2eab7db9eb82506ef0b64d182dd52a42b4876bbb5` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/globals/wgpu_context/submission.rs` | `394821da3d84936474743a838439d5b673ed3a4975665ea70f0ec66a2bee93f9` | `70b94328d0cfb9388ff86b5cc7960a7bf2420867a0a0ad406002ead4dd1c5a27` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/globals/wgpu_context/validation.rs` | `21b5be4dc71a51156b9a600dd1db823baffb9cdddef8df482e6c6b16207444d1` | `33fc03fc33798d1bb62223f97adeb1ce06f802e8a359e98d414d44f1589f800b` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/src/lib.rs` | `5a9039181baf9cc6a6b45e7810f99fd9811427b80e62c9d84f52cde5f3bfa9a6` | `5a9039181baf9cc6a6b45e7810f99fd9811427b80e62c9d84f52cde5f3bfa9a6` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/src/prelude.rs` | `1d909746b1d9fbd8135c06cb27bf2193f6f9141c1b11c26a6d3d856e1dcf0666` | `1d909746b1d9fbd8135c06cb27bf2193f6f9141c1b11c26a6d3d856e1dcf0666` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/tests/fixtures/buffer_usage.json` | `15232f16dd50015c7c5f683504c0f7be05695d74a39fbebaa7916b4a038166ad` | `15232f16dd50015c7c5f683504c0f7be05695d74a39fbebaa7916b4a038166ad` | ALREADY_IDENTICAL; leave unchanged |
| `crates/fabelgeist-gpu/tests/fixtures/texture_views.json` | `2fa289e171e23b95b647a08e4cda05155e304750376625a01ba8fb34802af800` | `None` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/tests/texture_views.rs` | `58e43348ecc8c5bea2cdebc67be84a584982167fcbd3ff1ccad80aae8edb8d0c` | `None` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `crates/fabelgeist-gpu/texture-views.md` | `3d7604f584ef800e7ac26c5e0e963bf30e9a2b2909b2d97e7c9e345eb63e3bfc` | `610d980a0657a3f089b0178979f77f22ce6cb529e46f37ca9c3ecd495709e206` | COPY_CANDIDATE_ONLY after findings/ownership composition; not integration clearance |
| `rust-quality-baseline.toml` | `fa4fcada546c6e5f6350d0de7cd7d03e88cd5f4b3ec92034e12432f60c6df8dc` | `bb3a64ed8ec80028324e18ea6eecac8a3a68609eff696b9cf6c58bd556d6fa38` | MUST_COMPOSE seven removals only; never wholesale replace |

## Actual unchanged Root dependency read extents

| Path | Current SHA-256 | Extent |
|---|---|---|
| `crates/fabelgeist-gpu/src/data/gpu/buffer/stride.rs` | `9a8bfaa7b7b1d2d3c6c72d27cbd6f294c14356fee69bf72a59d4af38f89490b5` | whole |
| `crates/fabelgeist-gpu/src/data/gpu/buffer/capacity.rs` | `79d9d026debd8f149f80f1c8a7a8e1df362e310e8b5387dc4e25f1eac7032176` | whole |
| `crates/fabelgeist-hair/src/gpu/sim/rods.rs` | `8ec4a8182b2b06542654966c75b6120c9acc24852b3d9e05f23b920806bb3425` | whole |
| `crates/fabelgeist-hair/src/gpu/sim/collision.rs` | `432b9250fbd2810918ab7ca663e1596d715b4cd6a7f4491b109b3dfd6cc8fe6e` | 35-85 |
| `crates/fabelgeist-compute/src/sort/mod.rs` | `abeb15fa7f0450e9807d938e9758918753ef3c6fa0eb4b3162bf4f1832ba3a1d` | 1-150,170-215 |
| `crates/fabelgeist-compute/src/kernel/mod.rs` | `47b3faf12d1fdffc6251a9dcd840b28f8ce347bbf9e377e29671b4b171f9ff74` | 1-120 |
| `crates/fabelgeist-color/src/transfer.rs` | `81b5c5f4a55d8191a178ea71fc618c968db91c2b601fa7b72e0cadf466ecad62` | whole (production+fixtures) |
| `crates/fabelgeist-color/src/wgpu.rs` | `7e35c05565fa8fa492ddc7ea0886a7305272d5692607ecc8a91745a3a461b16a` | whole |
| `crates/fabelgeist-color/src/lib.rs` | `9586a92ca2dcf19911b96b5fd50818df42af552582a7b0955224294ef69b9013` | whole |
| `crates/fabelgeist-color/Cargo.toml` | `a6c66c1e272fc13f282c7b0c03cb1a3291cac8af10876bccdbc1a44a85504f46` | whole |
| `crates/AGENTS.md` | `2d9c7c48f7c9d02c2996ee71cd64c8dc97ef71ee260f99b16ff388a782f63f23` | whole |
| `AGENTS.md` | `51338dad10ee53bbf3fe30525a969aae466e55c746cc0f2dbd7e82a28f2cfe5a` | whole |
| `crates/adventuresim-character-creator/src/hair/gpu/bake/layout.rs` | `88c1868538af8a6862650cfe70083e9bcaff1785e72160d656629aabb23df23b` | whole |
| `crates/adventuresim-character-creator/src/hair/gpu/bake/readback.rs` | `dabc0ed6029883b7507a1df190862761c1f6936fd89f7549832e0bfa7a403bb7` | whole |

## Limits and completion disposition

- No production edits, project checks, builds, tests, probes, scanners,
  generators, push, CI, or external publication.
- All 85 non-generated incoming sources plus six shared GPU generated leaves
  read; other45 generated files explicitly not reviewed here; peer coverage does
  not imply this reviewer clearance.
- Root consumer snippets listed with exact extents; complete
  sorting/scan/extraction/native Hair Grid/physical/material/generated families
  are not cleared by this native packet review.
- Pinned SDK TextureUsages declaration only inspected; SDK implementation
  universe and actual browser/device runtime not executed.
- Root changed HEAD during long review; exact completion HEAD and current hash
  accounts are recorded. Root source account bytes are receiving comparison
  evidence, not a verdict on unrelated dirty generated edits.

All thirteen lenses have explicit pre-integration contracts, dispositions,
connected repair/dependency accounts and acceptance criteria for this named
native receiving scope. Actionable findings remain open. This does not complete
the full sorting migration, generated45, physical/material/native Grid families,
or final same-pushed-revision reviews and hosted checks. No merge authorization
or global pass is implied.

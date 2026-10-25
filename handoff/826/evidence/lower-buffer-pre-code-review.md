# Lower buffer pre-repair Code Review

Frozen root: `1f7ecbe13f6f1cff79520e8ad4bdd9c460704bfd`.
Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.

Source map SHA-256: `c07419bc13d03c4f6a24fc1eace2edf98dfcbeda97cf0782796e1a5ffe879238`.

The 43 source files in the JSON accounting match the exact frozen revision.
Tracked status was clean. Coverage labels distinguish full production reads from
selected complete function bodies; a selected grow prefix is explicitly
incomplete.

This is discovery input, with seven actionable repair obligations. It does not
clear the lower family, all thirteen lenses, source compilation, the PR or any
hosted checks. No source or tests were changed.

Human transfer resolved issuecomment-6372371468. #826 owns complete Buffer read
and lower ArmorGpu/MetalGpu facade/staging/error owners; #780 owns
application/material/Engraving/count producers. No reviewer source edits.

## LB1 P2: Observe native allocation, copy finalization and mapping validation at their actual ports

`crates/fabelgeist-gpu/src/data/gpu/buffer/mod.rs:53-75`

Invalid staging/copy/map usage/alignment/destroyed native resource emits error
through SDK sink before or independently of callback. Raw finish/submit bypasses
canonical helper; default native handler may panic, custom handler can leave
only lossy callback.

Minimum connected repair: Use context.submit and propagate actual cause before
mapping; scope immediate staging allocation/map validation under existing native
blocking policy. Preserve other-class/browser/borrowed-device limits.

## LB2 P2: Clean mapping state on failure and cancellation

`crates/fabelgeist-gpu/src/data/gpu/buffer/mod.rs:74-113`

Failed map exits before unmap. Rust map_context remains nonempty though native
may be Idle; retry on same MAP_READ buffer asserts already mapped. Dropping wasm
future at receiver await leaves original Arc buffer pending/mapped.

Minimum connected repair: Arm request cleanup and release views before unmap.
Failed-map unmap must reset Rust wrapper while capturing native Idle/NotMapped
Validation without masking primary cause. Exclusive external mapping access is a
precondition.

## LB3 P2: Handle valid logical zero without empty native slices

`crates/fabelgeist-gpu/src/data/gpu/buffer/mod.rs:45-55`

Logical zero over nonempty allocation is explicitly valid in binding docs and
mesh upload. Nonmappable read creates zero-sized staging then SDK slice(..)
panics. Mappable zero maps physical allocation and waits for no requested bytes.

Minimum connected repair: After T admission, return empty for logical zero
without native copy/map/slice. Preserve ZST rejection even for zero and
logical/native distinction.

## LB4 P2: Use native usage and admit logical extent before allocation

`crates/fabelgeist-gpu/src/data/gpu/buffer/mod.rs:45-50`

Public independently mutable size/usage can disagree with native buffer. Wrong
stored usage selects wrong branch; oversized logical extent is host-allocated
before native admission. On usize64 u64MAX with u8 Vec::with_capacity
deterministically capacity-panics.

Minimum connected repair: Check logical<=native.size before allocation/GPU work;
choose native buffer.usage authority. Keep logical extent distinct. If
allocating fallibly retain actual TryReserveError. No fake device/map-state
query.

## LB5 P2: Preserve native sources through armor and metal facades

`crates/fabelgeist-armor/src/gpu/mod.rs:359-360`

Armor device_error Display->Arcstr and Metal Display->String erase
parser/pipeline/pass/readback sources before fitting/cache, including any newly
typed Buffer failure.

Minimum connected repair: Agreed cause-preserving GenerateError/backend and
Metal carrier through full facades; #780 material/application/Engraving/cache
consumers must migrate. Existing GenerateError::Readback source chain already
works.

## LB6 P2: Stop reading or publishing success after BatchResult Err

`crates/fabelgeist-armor/src/gpu/staging.rs:73-74`

read_staged_async ignores submission result then maps staging. Armor
build_in:280 and nearest_points:351 continue likewise; metal bake:82 reads
outputs and returns MetalTextures after discarded submission failure.

Minimum connected repair: Propagate canonical BatchResult cause before
read/success; coordinate #780-owned bake/application and other callers, no
competing implementation.

## LB7 P2: Preserve adapter and device creation errors at original boundary

`crates/fabelgeist-gpu/src/globals/wgpu_context/mod.rs:225-239`

WgpuContext::open replaces RequestAdapterError and RequestDeviceError with
message-only anyhow. Armor and Metal cannot recover those native sources
downstream. Wasm surface error is also replaced.

Minimum connected repair: Preserve real SDK creation errors using existing
anyhow context or owning error; coordinate exact open ownership and browser
non-Send JS carrier traits.

## SDK and host contracts

- wgpu api/buffer.rs:312-346: slice rejects empty/out-of-bounds, unmap resets
  Rust map context then backend; size/usage publicly query native descriptor, no
  device or map-state getter.

- wgpu api/buffer.rs:566-581 and 777-783: map_async asserts no existing Rust map
  range, installs range before backend; reset requires all views dropped.

- wgpu-core resource.rs:599-675: map validates alignment, extent, usage, device
  and destroyed state before Waiting; rejection may leave core Idle.

- wgpu-core device/global.rs:2065-2093: immediate map error invokes callback
  then returns detailed core cause; backend/wgpu_core.rs:2210-2233 wraps
  callback to fieldless BufferAsyncError and separately sends detailed error to
  buffer stored error_sink.

- wgpu-core resource.rs:837-847,906-910: unmap Waiting invokes MapAborted
  callback; Idle returns NotMapped. backend:2250-2260 reports unmap error
  through sink. Failed-map cleanup must retain primary error.

- wgpu backend/wgpu_core.rs:1606-1624: allocation error goes to device sink;
  returned handle is not a Result. Same-thread sink filter picks innermost
  matching class at640-676; default handler panics691-694.

- wgpu api/device.rs:85-97 and backend/webgpu.rs:2560-2563: native poll
  progresses callbacks; browser poll no-op with automatic progression. Current
  read uses Wait(None,None), then Poll; no proven reachability of both current
  PollError variants claimed.

- wgpu backend/webgpu.rs:2794-2809: mapAsync promise receives fieldless SDK
  BufferAsyncError on rejection; JS rejection cause already erased by SDK, do
  not fabricate deeper cause.

- wgpu backend/webgpu.rs:3961-3975 read-only mapping obtains SDK-owned temporary
  bytes; retain application host allocation-before-view order and do not add
  output Vec growth while view is held.

- wgpu api/buffer.rs:851-861: actual BufferAsyncError implements
  Error+Send+Sync; wgpu-types lib.rs:296-322 actual PollError implements Error.
  Futures oneshot Canceled is actual receiver failure, preserve value rather
  than format it.

- Canonical context submission.rs:19-45: native/blocking Validation scopes
  separately observe finish before queue; browser/borrowed context uses device
  handling. No completion wait in this helper.

The pure host helper already converts host length with the real TryFromIntError,
rejects ZST and partial elements, allocates aligned Vec<T> storage, checks
source truncation before unsafe copy, copies only logical bytes, and sets length
only after initialization. It includes existing source fixtures for unaligned
input, zero host extent and ignored GPU padding. None was executed or extended.
Current Vec::with_capacity remains infallible; the deterministic forged-size
capacity witness belongs to LB4, rather than an unsupported claim that ordinary
supported GPU reads must encounter allocation failure.

## Consumer traces

direct anyhow readers: GpuResource read; Min/Max execute_to_number; QueryHits
read; VertexNormals read; MeshNeighborhood build; MeshWireframe three builders;
GpuBvh three readers; Particle three readers; project_particles; GpuMesh
readback; garment positions/readback_handles; shell read_mesh. await
Buffer::read with ? or same anyhow carrier; stop before host
reconstruction/write on error. Existing count/capacity producers still owned
#780. Mesh local read explicitly bypasses zero; lower zero fix centralizes
correct read behavior without requiring shim.

hair typed outer carrier: read_strands -> GpuGroom/GpuGuides read; GpuScalp
read; sim read_positions/read_rest; GpuGroomer read_words. Buffer anyhow error
converts through HairGpuError::Backend #[from] without text erasure. read_words
ignores submit result before lower read; pending producer propagation is #780
obligation, not newly solved by Buffer change.

creator hair anyhow consumers: GpuGeometry read; GpuScalpTexture read/chain;
longest; PlacedHair geometry; GroomedHair grow selected prefix. ? preserves
cause until later application boundary. longest currently ignores submit before
read, existing caller obligation. No full grow-body clearance; later application
graph belongs #780.

armor staged lower results: ArmorGpu read_staged_async -> compute Readback read
-> GenerateError::Readback; DevicePart read; Correspondence read/finish;
bracer/breastplate read_async. Mapping/Poll causes survive existing typed
source. BatchResult is currently discarded before map. Success reconstructs
statuses/positions/normals/skin/morphs only after read ?; rig errors typed. Do
not relabel this as current Arcstr mapping erasure.

armor direct reads: ArmorGpu read_async/read; nearest_points; DeviceFrame check;
generate_limb_armor_on. device_error loses underlying anyhow source.
nearest_points and limb submit failure also discarded before read; owned split
must preserve canonical error through #780 callers. DeviceFrame ordered
status->prose fitting loss is separate M06 owner, unchanged by lower repair.

metal facade/bake: MetalGpu new/open/upload/scratch/read/dispatch/textures ->
bake/textures and all record helpers. String Result cannot preserve source. Bake
validates size32..1024; actual bounded texels/scratch arithmetic fits for
ordinary facade. Do not claim arbitrary u64 overflow reachable through current
supported size. #780 owns checked count/byte producers and material/Engraving
String errors; carrier migration must meet them rather than compete.

cached application devices: armor_gpu_async; metal_gpu. OnceCell/OnceLock retain
Result<_,String>, later reconstruct anyhow from prose. Required #780
receiving/cache migration after lower canonical errors, not ownership
authorization blocker.

## Acceptance and required dependencies

- Keep public Buffer read anyhow carrier with focused typed source owner;
  preserve actual SDK/poll/channel/map/host narrowing/allocation sources where
  available.

- Preserve ZST/partial element rejection, Vec<T> alignment, AnyBitPattern bits,
  truncated source admission, initialize-after-copy and padding exclusion.

- Admit native extent/usage before host/GPU work; explicit logical-zero empty
  behavior.

- Allocate typed host output before mapped view; release view before all
  cleanup.

- Use canonical submission with no added completion wait there; retain native
  readback Wait/progress and browser event-loop await.

- Failed/cancelled request resets only its own mapping and never masks primary
  error; no fictitious aggregate provenance or external map-state proof.

- Native synchronous validation scopes obey documented cfg/policy;
  borrowed/browser paths keep honest device-handled limits.

- Full owning facade/source/cache chains stop on submission failure and retain
  causes through #780 receiving APIs before family clearance.

- Existing tests only, repeated root all13 waves and independent reviews plus
  final same-pushed-revision hosted checks remain required.

- Required dependency: #780 material/application/Engraving cause-preserving
  errors/cache and count/stride/metal scratch producer conversion.

- Required dependency: Shared compute Readback copy/native
  allocation/mapping/cancel remains coordinated #780; its current typed
  Mapping/Poll chain already survives armor ?.

- Required dependency: M06 fitting FrameFailure/torso/cache worker source
  erasures and retained F19 output completeness remain required connected
  dependencies.

Supplemental #774 evidence: selected M06 at REPORT.md:2635-2702 confirms the
earliest armor native cause erasure and the positive staged Readback source
path. Its later ledger receipts at33830-33860 explicitly remain incomplete. F19
is extraction output completeness, unrelated to manufacturing extra bytes or
clamping returned output. Current human transfer resolves authorization; source
split, connected implementation, all13 inventory and integration gates remain
required.

## Limits

- No project execution/checks/build/tests/scanner/generator/runtime/push/CI. No
  new tests/cases/helpers/assertions/doctests.

- No whole PR/source compilation/final clearance; root all13 inventory remains
  incomplete and not delegated.

- PollError actual API result must survive; this report does not claim each
  variant is reachable with current Wait(None,None) and Poll calls.
  Channel-closed retention is defensive; SDK guarantees callback for accepted
  native invocation.

- Validation-only policy does not promise native OOM/Internal/device loss or
  late GPU health capture.

- No public creating-device/map-state query; exact logical-device/paired queue
  and exclusive external MAP_READ mapping remain explicit preconditions, no
  invented provider/global lock.

- Selected 774 M06/F19/receipts are incomplete supplemental evidence, not
  imported clearance.

- Producer metadata rollback/freshness and raw renderer/texture mapping APIs are
  separate owner obligations.

- External armor device_error use sites were searched as receiver census, not
  all 66 complete caller bodies freshly reviewed here; root all13 receiving
  graph and #780 migration must cover them. Search output was truncated once and
  not used to claim complete body coverage.


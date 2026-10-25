# Texture readback pre-repair Code Review inventory

Pinned source `8cdcfa08f4e22804c384955c749c5640154f94c7`; tree
`934eb5dd2adbbbd85faa7a1cdb9b3679db71ace6`.
Pinned main `990430b77a9f63548ffcb642074185f61ddb3322`.

This is complete discovery for the explicitly accounted receiving family and all
thirteen lenses. Five source-backed repair families remain required. This is not
repaired-source, compilation, whole-PR, parent-all13, hosted or final clearance.
No source was edited.

Coverage: 45 first-party source/document accounts and 11 SDK source accounts.
Complete files and exact partial/function/exclusion ranges are recorded in the
JSON companion with SHA-256 and Git blob identity. Current texture files were
read completely, including native cube/image paths and retained fixture bodies.

## TR1 [P2] Fallible texture read accepts host types that panic at the final byte cast

An initialized 1x1 R8Unorm texture/view read::<u32> yields one packed byte then
bytemuck::cast_slice panics with slop; nonempty read::<()> also fails. Vec<u8>
allocation does not contractually satisfy arbitrary T alignment even when byte
count divides T.

A Result<Vec<T>> read should reject unsupported host layout without panic and
copy bytes into an aligned, initialized Vec<T>.

Reuse the existing canonical buffer Readback<T> host admission/copy owner; make
its required crate scope available, without duplicating typed
allocation/initialization logic. Preserve View empty-first before host
admission.

Locations: `texture_2d/mod.rs:370`, `texture_3d/mod.rs:236`,
`texture_view/mod.rs:293`.

- Nonempty ZST/partial elements return existing concrete host-layout error
  before native work.

- Aligned destination handles unaligned input bytes and preserves all original
  native-endian bits.

- Packed output excludes every padded GPU row and preserves z-major then
  row-major order.

## TR2 [P2] Texture staging volume narrows through unchecked u32 arithmetic

Texture3d::read computes padded_row*height*depth as u32 then casts to u64. A
1024-cubed R32Float volume has 2^32 bytes, within source dimension bounds when
the provider admits the allocation; debug panics or release wraps before the
native buffer limit can reject it. This does not claim any current device can
allocate it. A larger custom 2D device may expose the analogous 16384-square
RGBA32 case; baseline 8192 2D alone does not prove that scenario.

Rows, full padded staging volume, packed bytes and host addresses must be
related in checked wide byte units before allocation/copy/slicing.

One texture transfer-layout aggregate, native extent+pixel format at admission,
BufferByteLength/Offset for byte semantics, checked row alignment/native-u32 and
host-usize narrowing. Reuse SDK alignment constant. Check copy range against
actual mapped extent; do not invent broader constructor/schema migrations.

Locations: `texture_2d/mod.rs:276`, `texture_3d/mod.rs:138`,
`texture_view/mod.rs:200`.

- Checked padded and packed u64 products; checked native row and usize
  conversions retain actual narrowing causes.

- No u32 volume/row-index multiplication before widening.

- Queue upload rows stay tight, encoder copy rows stay 256-aligned; full padded
  allocation is permitted though SDK copy omits trailing last-row padding.

## TR3 [P2] Texture map/read progress loses concrete errors and lacks canonical request ownership

Native callback Err becomes a formatted String, oneshot cancellation becomes a
generic String, Poll failure is discarded; wasm erases even map detail.
create_buffer and map_async immediate native validation also occur outside
observe_validation; a synchronous map failure can route through the device sink
before the function reports its callback error.

Keep actual native, BufferAsyncError, PollError and Canceled values, fail at the
earliest observed boundary and clean this private request without replacing its
primary error.

Reuse canonical MappingRequest, BufferReadError, context.observe_validation and
callback_progress; add only the separately owned nonblocking texture completion
behavior. Own cleanup before scope result inspection, including synchronous
callback. Preserve existing Buffer::read explicit-wait method.

Locations: `texture_2d/mod.rs:324`, `texture_3d/mod.rs:186`,
`texture_view/mod.rs:249`.

- Ready-first callback outcome/cancellation precedes a later Poll; pending polls
  retain actual errors.

- Texture native progress retains 1ms async cadence; no added Wait, including
  borrowed context; browser yields and may drive enabled non-WebGPU providers.

- Successful view drops before unmap. Error/cancellation/unwind reset only owned
  private request; cleanup-only native scope no wait, no primary-error
  replacement.

- No unconditional unmap on unrelated external aliases, no fake SDK map/device
  state query, no new global mapping lock.

## TR4 [P2] Texture upload Result boundaries miss immediate native queue validation

For an initialized positive-size R32Float texture, write(&[]) reaches SDK queue
source-byte bounds validation; queue errors route through its sink immediately.
These functions return Ok if that handler continues, or panic under default
handling, rather than returning the actual available native cause. The same
boundary exists on image/cube uploads after packing.

Owned native validation policy must observe queue write at the actual immediate
boundary, after conversion/layout; do not rely on a later submit scope.

Wrap only queue.write_texture in context.observe_validation after fallible
conversion and checked tight layout; existing context device/queue precondition
and borrowed/browser handler policy remain. All five source call sites must be
migrated together.

Locations: `texture_2d/mod.rs:214`, `texture_2d/mod.rs:391`,
`texture_3d/mod.rs:248`, `texture_cube/mod.rs:183`, `texture_cube/mod.rs:298`.

- Actual available native cause enters the owning error chain; no message
  flattening.

- No new GPU completion wait or waiting ValidationScope for queue writes.

- Do not impose 256-byte alignment on queue writes, require exact source size
  when SDK accepts trailing data, or change existing colour/packing behavior.

- Cube per-face upload does not promise atomic rollback of already queued
  earlier faces.

## TR5 [P2] Image float decoding relies on byte-vector alignment

read_image reads Vec<u8> then casts it to &[f32] for Rgba32Float/R32Float. This
reintroduces an alignment requirement that the byte vector contract does not
guarantee after TR1 is fixed.

Image decoding must preserve native-endian float values without borrowing f32
from an alignment-one allocation.

Use exact four-byte chunks and f32::from_ne_bytes, preserving
clamp/round/greyscale/alpha and format decisions. This is not a new Image schema
or colour-policy rewrite.

Locations: `texture_2d/image.rs:125`.

- Complete float byte chunks decode in existing native endian order; successful
  numerical operations/formats unchanged.

- No mapped view held across host allocation and no new float alignment
  assumption.

## Shared contracts and state ordering

Missing and devices:

- Texture2d/3d read/write: native texture Uninitialized before ensure_device; no
  cached view required.

- TextureView::read returns empty for either zero dimension before
  texture/device/T admission; nonempty then native Uninitialized then device.

- clear_cached_view does not invalidate allocation read; clear_native_texture
  does even if cached view survives.

- A cached-format view request remains cache-first/context-unused;
  alternate/face-with-context/array proves device; PassParameter selection
  independently validates selected actual facts.

- Read native texture port and face layer are immutable admitted metadata, not
  reconstructed from cached view.

- Texture queue requires context queue paired with exact creating logical
  device; existing context public fields cannot prove arbitrary user
  replacement.

Byte and format:

- All represented TextureFormat variants are uncompressed 1x1 blocks, pixel_size
  1/2/4/8/16, Depth32Float 4.

- sRGB/linear and BGRA counterpart view changes preserve allocation byte width;
  raw read transfers bytes, not sampling conversions.

- Depth32Float source copy is permitted subject to actual native
  capabilities/usage; its destination write rejection should stay native. Do not
  reject every depth read.

- Every read uses mip 0; 2d/view depth 1; 3d full depth; View origin.z exact
  retained layer or 0.

- Full padded GPU allocation versus tight logical output are distinct; final row
  GPU padding is never returned.

Cleanup and causes:

- Buffer SDK public map_context is marked requested before backend map_async and
  remains until unmap/reset; native core failure can be synchronous and native
  state Idle.

- Canonical MappingRequest must own private staging cleanup before immediate
  native observation inspects its result.

- Scope cleanup after failed or cancelled request pops without waiting and
  suppresses cleanup-only NotMapped while preserving original
  native/map/poll/channel error.

- Dropping native CoreBuffer invokes buffer_drop -> best-effort unmap. Therefore
  old missing explicit guard does not prove permanent native leak;
  WebBuffer::Drop is no-op and no browser memory lifetime proof is claimed.

- Do not wait on GPU in cleanup, do not leave a native ErrorScopeGuard across
  await, and do not infer arbitrary alias map ownership.

- Actual map callback reports SDK BufferAsyncError, not underlying more detailed
  core reason; only available native validation scope can retain additional
  actual wgpu cause.

Scheduling:

- Texture reads currently use ready-first native try_recv then nonblocking Poll
  plus 1ms async sleep; preserve cadence with canonical polling completion
  entry.

- Canonical Buffer::read complete() explicitly Waits; do not replace texture
  behavior by calling it without a consciously changed contract.

- Browser WebGPU progresses automatically; canonical browser callback also
  drives non-WebGPU providers and yields zero timer turns.

- Borrowed false/browser perform no new validation wait or verdict scope;
  available native poll/map errors remain results.

- Submission finish and queue boundaries remain canonical immediate no-wait
  context.submit.

- Native Poll(Poll) may return no timeout on some providers; do not infer an
  observed runtime failure. Still retain every actual Result error rather than
  discard.

Publication:

- Readback has no shared result cache; errors return no successful partially
  initialized Vec<T>.

- Image upload creates private candidate before write; observe immediate write
  before returning it. Cube multiple uploads are not rollback-atomic.

- Raw render/cube paths continue existing producer/submission semantics; no
  texture readback success certifies earlier ignored producing error, later
  writes or global GPU health.

## All thirteen lenses

### producer-propagation

All three public reads, GpuResource::read/write, Image/read_to_rgba8/to_image
and Cube face producers traced; grouped deferred reads separately peer-owned.

Disposition: TR1-TR5 flow through one shared receiving contract, not three leaf-
only fixes.

Source: Texture2d/3d/View reads; image.rs; resource.rs; cube/views.rs;
compute/test_utils.rs and grouped readback.rs

Acceptance: All affected receiver families repaired, peer grouped family
coordinated before global clearance.

### existing-type-owners

BufferByteLength/Offset, BufferReadError, Readback<T>,
MappingExtent/MappingRequest, CallbackProgressError and context helpers already
own byte/mapping/native facts.

Disposition: Reuse those owners; only texture-specific row/volume layout may be
new.

Source: buffer/{extent,read_error,readback,mapping,read}.rs; globals
callback/validation/submission

Acceptance: No duplicate map/channel/poll/host destination owner or raw byte
count passed between texture internals.

### admission-contracts

Private texture fields are admitted by complete constructors; defaults and
removal APIs admit missing ports; view can be zero; typed host admission
currently happens too late.

Disposition: TR1/TR2/TR3/TR4 required; retain exact missing/device/cache and
zero order.

Source: All three mod/view constructors, selection, Buffer host/map admission
and SDK API/core

Acceptance: Reject unsupported host and checked-layout states before GPU work
after prior existing missing/device ordering; native causes at actual
operations.

### invariant-operations

u32 padded volume and row-index multiplication, host Vec capacity, bytemuck
cast, map request/cleanup and queue write are all mutation/arithmetic edges.

Disposition: TR1/TR2/TR3/TR5 required; no numerical shader or constructor
rounding change.

Source: Three reads, all five uploads, image float decode; SDK transfer.rs and
transfers.rs

Acceptance: Checked wide relationships and slices; fallible host reserve before
mapped view; native row alignment only at encoder adapter.

### aggregate-states

Native texture/cache/size/format/device/usage; selected face layer;
packed/padded rows/volume/T host elements; mapping request/release are connected
facts.

Disposition: One checked texture layout; retain canonical map obligation enum
and private-port authority.

Source: All texture structs/default/PartialEq/accessors/removals; canonical
mapping owner

Acceptance: No independently mutable row/volume capacities; preserve
cache/native availability matrix and exact face layer.

### type-ownership

GPU texture module owns row/volume transfer layout; GPU buffer owns mapped
lifetime and host typed copy; context owns native scopes; Image owns colour
conversion; compute owns grouped batches.

Disposition: Scoped crate visibility, no public compatibility wrapper or new
peer-owned implementation.

Source: texture modules; buffer modules; globals; compute/readback.rs

Acceptance: Implement in actual existing lowest owner; preserve #780 and #895
boundaries.

### redundant-authority

Three copied readback implementations recompute identical
rows/padding/staging/map/callback contracts. Canonical Buffer already owns
equivalent lifecycle/typed host contracts.

Disposition: TR1/TR2/TR3 unify; cache and format remain authoritative original
texture owners.

Source: 2d/read268,3d/read130,view/read185; buffer/readback/mapping

Acceptance: One transfer-layout calculation and native map owner; do not
reconstruct selected view metadata from allocation format.

### behavior-preservation

Raw bytes are stored native bits, counterparts differ in sampling transfer only.
3D output uses z-major then rows. View returns empty first, 2d/3d require
texture/device. Existing image/cube conversion branches differ.

Disposition: Preserve every successful byte/colour/rounding/face/cadence/default
branch; TR5 changes representation only. Adjacent colour policies explicitly
retained below.

Source: Texture family full source; pixel_size/counterparts; cube/image_data;
Image; SDK

Acceptance: No new completion wait, RGBA normalization, format expansion, size
rounding, zero/missing reordering or shader changes.

### enum-shapes

TextureViewError payloads already tie native/progress to request/method.
MappingRequest uses Required/Complete release state, callback Result owns map
outcome; CubeFace owns face role/index.

Disposition: Reuse closed states; no boolean map-status soup or new parallel
operation strings.

Source: view_error.rs,cube_face.rs,mapping.rs,callback_progress.rs

Acceptance: Actual failures remain structured; no String-as-state or loosely
coupled has_mapping flag.

### derivability

TextureFormat/CubeFace derive Copy/Eq/Hash/serde; resource PartialEq
deliberately uses Arc pointer/resource metadata. Concrete SDK errors differ in
Clone support and own real causes.

Disposition: No new derivation correctness finding; do not replace pointer
equality or flatten errors to gain derives.

Source: All texture derives/PartialEq, buffer/error source and SDK
BufferAsyncError

Acceptance: Derive only traits supported by actual contained cause/state; custom
source semantics preserved.

### type-granularity

Rows, packed bytes and padded native bytes are related but not interchangeable.
Format and CubeFace existing owners are appropriately broad; context/native SDK
extents stay boundary representations.

Disposition: Use checked aggregate and canonical byte leaves, no bespoke type
per constant/axis/format token.

Source: TextureFormat, all transfer expressions, BufferByteLength/Offset and SDK
TexelCopy layouts

Acceptance: Texture layout adds a real invariant rather than forwarding wrapper;
no duplicate PixelFormat or TextureSize owner.

### magic-strings

Mapping error/channel prose currently erases classification; native labels and
WGSL names/tokens are diagnostic or explicit shader boundaries, not state
dispatch.

Disposition: TR3 replaces erased causes with existing typed errors; leave
labels/source tokens and external URI boundary unchanged.

Source: Three reads/error messages, full cube/sRGB shaders, Image::new and
buffer/error

Acceptance: No branching on Display text and no newly invented magic
status/source strings.

### magic-numbers

255 padding mask duplicates COPY_BYTES_PER_ROW_ALIGNMENT in View, while other
reads use SDK constant. Pixel byte mapping owned by TextureFormat. One-ms
cadence is existing scheduling behavior. Cube six-face/index/channel constants
are closed geometry/format boundaries.

Disposition: TR2 centralizes SDK alignment; no domain-name-only constants or
colour/shader/cadence redesign.

Source: Three row calculations, full format mapping, CubeFace/face ABI, image
conversion and native descriptors

Acceptance: COPY_BYTES_PER_ROW_ALIGNMENT reused; documented cadence retained by
actual owner; native adapters keep required constants.

## Adjacent and intentional dispositions

A1: explicit retained pre-existing conversion policy; not normalized by this
readback change. Cube set_side_image converter applies RGB transfer for
plain8bit/Rgba32Float, unlike create_from_images/Image::packed_for. The proposed
readback repair must not silently change those successful bytes. This report
does not certify universal equivalence between those APIs.

texture_cube/image_data.rs:5-75 vs texture_2d/image.rs:65-96

A2: explicit retained pre-existing conversion surface; not a universal format
conversion claim. 2d read_to_rgba8 forwards raw bytes; View branches perform
selected conversion/reorder and generic4byte fallback. read_image is its
distinct format conversion owner. No format expansion or colour normalization is
necessary to retain raw-read successful behavior.

texture_2d/mod.rs:372-379 and texture_view/mod.rs:295-339

A3: raw renderer and Blitter API native policy retained; no synchronous native
acceptance claim. These operate directly on Device/Queue; source-agreed renderer
policy leaves native create/copy/submit to device handling, explicit waits only
for output readback. They do not delegate to Texture2d/3d/View read. Do not
invent a WgpuContext or duplicate observer.

globals/wgpu_context/blitter.rs and hair/render/readback.rs

A4: no whole-Image schema mutation. Image public raw pixels/dimensions and
permissive from_pixels remain an existing boundary. Tight upload layout plus SDK
source length validation must retain actual failure; no speculative global
dimensions/serialization or ZIP/NumPy migration proposed.

texture_2d/image.rs:8-96

A5: native invalid/capability states remain provider-observed. Do not change
float size rounding/clamping/NaN or format capability admission to fix readback.
Constructors retain Native/Progress observation, borrowed/browser limits and
success-only object publication. Readback wide layout does not promise provider
allocation success.

Texture2d::create/Texture3d::new/TextureCube::new and all views

## Required dependencies and validation limits

R-receiving (#826): All three reads; shared texture layout; canonical existing
buffer host/map scoped reuse with nonblocking completion; all five uploads;
float image decode; owning durable limits docs. TR1-TR5 and all bounded 13
acceptance verified from repaired source; no tests authored.

D780-grouped (#780): compute Readback/PendingReadback and grouped
after_submit/callback state; shared Kernel/FastPath/cache/caller migration.
Consume coordinated canonical contracts and immutable peer implementation; do
not independently edit or declare these already repaired from root8cdc old
source.

D895 (#895): ZIP/NumPy stack remains offlimits and is not a needed
implementation dependency of native texture readback. No independent
producer/schema/ZIP/NumPy changes.

D-native-producers (coordinated #780/#826): Retained renderer native
strand/card/simulation source/layout/creator/lock integration and five
validation-scope peer sites remain broader open work. Bounded texture report
never clears remaining parent ledger dependencies.

D-final (coordinated #826): Final pushed exact source all13 wave, independent
Code/Rule/Comment Review and hosted required checks. Same final pushed revision;
no source/compile/final clearance from discovery.

- No source edits; only these ignored reports created.

- No project build/check/test/scanner/generator/runtime, no numerical execution,
  no tests/cases/helpers/assertions/doctests authored or executed.

- Git source searches are discovery, not project scanners or a compile check.
  SDK consulted from exact installed29.0.4 source; bytemuck1.25.2 source
  contract.

- Every complete/partial source read is explicit above; unrelated
  fixture/producer bodies are not claimed freshly read. No prior verdict
  silently substitutes for current evidence.

- Root pending later documentation/source edits excluded by git show pin.

- No claims of observed device limits, hardware/browser success, native
  permanent mapping leak, atomic rollback, all error-class capture, arbitrary
  alias ownership, global GPU health or final clearance.

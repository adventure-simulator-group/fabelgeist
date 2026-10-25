# Texture transfer/readback checkpoint Code Review

Pin `7889102242f9ca698f4a01bb4607eb14ec72e2bb`; tree
`75e332dc604d48f8fe1aa335f4150132da648068`.
Comparison base `1b42bcfff0099a6066d67fc4859e535591b39789`.
Main `990430b77a9f63548ffcb642074185f61ddb3322`.

No actionable owned correctness finding in the 14-path texture transfer/readback
receiving repair; TR1-TR5 addressed in source. Bounded checkpoint only, no
compilation or whole-PR clearance.

All 14 changed files were freshly read completely. The report accounts for 47
current first-party files and 12 SDK/toolchain sources. Four unchanged canonical
Buffer/context/callback files were also freshly read. Other unchanged contracts
retain exact current Git/SHA256 identity and their prior explicit full/partial
coverage. The JSON companion contains every identity and coverage limit.

## TR1–TR5 reconciliation

TR1: addressed in current source. TextureReadSource::read admits canonical
Readback<T> before native allocation and keeps its aligned Vec<T> reservation.
packed_storage reserves tight byte capacity separately before mapping.
Readback::copy_from performs initialized byte copy into Vec<T>; no u8-to-T
borrowed cast remains. ZST/partial returns BufferReadError. View public empty
guard remains before host admission, so empty View<()> still returns empty.

TR2: addressed in current source. TextureTransferLayout sealed extent, row,
packed byte aggregate: row=u64(width)*u64(pixel_size), safe because the only
factors are u32 and maximum 16. NativeRow try_from retains actual
TryFromIntError. Checked volume uses u64 height/depth. TextureStagingLayout
checks positive transfer bytes, padded pitch and its native u32 field, complete
volume and actual device max_buffer_size. packed_storage narrows full padded and
packed extents to usize before GPU allocation. unpack gets complete padded
extent and checks native row/pitch usize; positive pitch and exact height*depth
rows follow sealed construction. No u32 row/volume/index products remain.

TR3: addressed in current source. Staging create immediate observe maps
BufferReadError::Native; canonical context.submit keeps original
ComputePassError::Native Arc cause for finish/submit. MappingExtent bounds
actual staging.size and aligned nonzero range; MappingRequest installs private
release obligation inside immediate operation before scope result. Native
texture code exclusively uses ready-first poll_complete and 1ms sleep; cfg
browser uses canonical complete browser branch. Buffer::read explicit native
Wait body unchanged. unpacked view is dropped before error propagation/unmap.
Drop cleanup preserves first error and resets own request with cleanup-only
native scope, no completion wait. Actual Mapping/Poll/Channel/Native error
owners and source chains remain.

TR4: addressed in current source. All five callers now use
TextureTransferLayout::write: 2d write/create_from_image,3d write,cube
create_from_images/set_side_image. Native descriptor rows tight and admitted
u32, same extent/origin/mip/aspect preserved. Immediate observe_validation only
wraps actual queue.write_texture, direct available wgpu error enters anyhow; no
wait or length-equality precheck, so supported trailing bytes preserved.
Conversion/layout finish before queue operation. Cube sequential upload remains
non-atomic, documented.

TR5: addressed in current source. Rgba32Float/R32Float read_image decode
complete size_of<f32>() chunks with f32::from_ne_bytes, then exact existing
narrow/grey/alpha transforms. Constructors/readback guarantee whole native pixel
bytes for these formats, so no new partial float path. Image fallback expected
byte count uses checked layout rather than u32 product. Cube float conversion
reserves from actual input byte count, which bounds complete four-channel group
output, preserving transfer operations and channel order; no schema mutation.

## Complete current contracts

Row and volume: For a nonempty staging record, width, height, depth and
pixel_size are positive; row>0, padded pitch>=row, full padded volume is exactly
pitch*height*depth. The sum of unpacked rows is exactly packed row*height*depth,
so reserved packed capacity covers every extend without allocation while mapped.
Full mapped range matches staging.bytes and is mapping-aligned via 256-byte
pitch. Last-row allocation padding is allowed by SDK and omitted from output.

Source: transfer.rs:35-159; texture/readback.rs:34-88.

Zero missing device: Public View retains zero-first; 2d/3d allocate-presence
then ensure_device before layout/T. Reads do not require cache, and
clear_native_texture does not become readable via cached view. Face reads retain
origin.z layer and one-layer extent. Source record zero case is checked and
cannot cause zero-length native slice.

Source: 2d/mod.rs:261-279;3d/mod.rs:129-147;view/mod.rs:185-209; exact unchanged
views/selection.

Map first cause: Synchronous callback may occur in map_async; request object is
already owned when observe_validation inspects native error. On request scope
rejection, returned request value is dropped and cleanup resets API requested
context; native Idle NotMapped is cleanup-only. Pending/error/cancel paths have
no mapped view across await. After success, unpack drops mapped view even before
a possible error, then release handles native unmap cause.

Source: buffer/mapping.rs complete; texture/readback.rs:72-88; SDK api
buffer/core mapping.

Progress and policy: Texture native path invokes only poll_complete, never
native complete Wait. Ready callback/channel state precedes unrelated later
polling. Browser branch drives canonical await_browser_callback; no native
waiting guard crosses await. Immediate scopes obey blocking_validation and
browser cfg, so borrowed/browser synchronous native verdict limits are
unchanged.

Source: texture/readback.rs:75-82;mapping.rs:120-129;canonical
callback/validation/submission complete files.

Format and consumer: Raw reads preserve bytes, native endian representation,
z-major order and face selection. Counterpart formats have same byte width.
GpuResource routes texture calls to same signatures; existing generic runner
typed elements remain complete. Image from_ne_bytes removes alignment dependency
only; distinct cube/Image/rgba convenience conversion branches remain.

Source: fresh current mod/image/cube bodies and exact unchanged
format,views,selection,resource and external consumer source accounts.

State and cache: No new result cache or partially successful Vec publication.
Layouts sealed; MappingRequest sole lifecycle owner. Existing cached view/native
state and native pipeline cache are unaffected by this14-path repair.
Queue/upload errors do not imply rollback of prior queued faces. No readback
success certifies ignored producing errors or later device health.

Source: complete 14-path diff; native context helper source; exact unchanged
source/hash proofs.

## All thirteen bounded lenses

producer-propagation: All three reads and five upload callers consume the shared
receiver directly. GpuResource/Image/cube/fixtures route through existing
methods; no shadow legacy path. Peer grouped readback remains separate required
owner work.

existing-type-owners: Reuses
BufferByteLength,BufferReadError,Readback<T>,MappingExtent/Request and
context/callback owners with scope limited to data::gpu. Texture layout adds
only the row/volume relationship absent from buffer.

admission-contracts: Sealed layout constructors establish byte bounds before
descriptor adapters; complete public existing zero/native/device ordering
preserved. Typed host admission before native work; selected format/cache
contracts unchanged.

invariant-operations: Checked u64 volume/pitch and explicit native/host
narrowing; allocation reserves before map; bounded linear row extraction and
canonical typed copy; actual error exits own cleanup.

aggregate-states: TextureTransferLayout owns extent/row/bytes;
TextureStagingLayout owns its transfer,pitch,bytes. MappingRequest retains its
release enum; cached/native texture state untouched.

type-ownership: Texture module owns native transfer geometry; Buffer owns
host/map state; context owns observation. No compute-grouped or ZIP/NumPy source
changes.

redundant-authority: Three duplicated native map/host loops removed; one texture
source/layout and one canonical mapping/typed destination owner. Native
descriptors reconstruct no independent unchecked volume.

behavior-preservation: Preserved tight successful native bytes, 3D order, face
index, labels, colour/math, empty-first, native error carrier and 1ms polling;
Buffer Wait unchanged. Original fixtures and three WGSL byte-identical.

enum-shapes: TextureReadKind owns closed diagnostic roles and MappingRequest
release owns lifecycle, errors own real causes. No boolean or String map-state
encodings.

derivability: Copy layout contains only immutable checked facts; staging not
exposed or mutated. SDK errors remain real owned types and source chains; no
handwritten trait replacement or new clone requirement.

type-granularity: One tight-layout and one staging-layout aggregate distinguish
native transfer/staging invariants without per-axis wrapper proliferation. Byte
leaves canonical; native extent fields stay framework adapters.

magic-strings: Old mapping/channel strings removed in favor of canonical typed
errors. Existing stage/encoder labels moved into closed TextureReadKind adapter;
WGSL/source strings unchanged, no text branching.

magic-numbers: COPY_BYTES_PER_ROW_ALIGNMENT used once in staging admission;
public readback no 255 mask. Existing 1ms native cadence named and preserved.
Byte component extraction size_of<f32> matches concrete native encoding.

## Existing source preservation

Independently compared all three existing fixture blocks against both the
comparison parent and preinventory revision: byte-identical. Independently
extracted all three retained cube WGSL literals: byte-identical against both
revisions. No test, fixture helper, case, assertion or shader was authored.

Image/cube conversion differences and convenience-read format rules retain their
preinventory dispositions. Raw renderer native error policy remains
device-handled, separate from context-owned texture readback. No broader Image
schema or ZIP/NumPy change is inferred.

## Required dependencies and limits

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

- No source edits, project build/check/lint/test/scanner/generator/runtime or
  new test/case/helper/assertion/doctest; only ignored report output.

- All 14 changed files freshly read in full, including unchanged bodies/native
  shaders/fixture blocks. Four unchanged canonical Buffer/context/callback files
  freshly re-read. Other unchanged first-party source contracts revalidated by
  exact Git/SHA256 identity and prior explicit full/partial coverage, not fresh
  full reads.

- Installed 29.0.4 SDK bytes independently rehashed; listed fresh source
  excerpts plus carried explicit preinventory source proofs. Std prelude source
  confirms size_of API, not a compile check.

- No claim of observed numerical/rendering performance, browser/hardware
  execution, arbitrary external alias mapping safety, permanent native leak,
  rollback, all native error-class capture or global GPU health.

- Separate #780 grouped readback/cache/producer/validation-wait callers and
  coordinated native renderer producer/caller/manifest-lock integration remain
  required; #895 offlimits.

- Final full all13 and independent Code/Rule/Comment reviews plus required
  hosted checks must pass on same final pushed revision; this source checkpoint
  has no such clearance.

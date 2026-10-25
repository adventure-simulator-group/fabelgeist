Independent Rule Review at `7889102242f9ca698f4a01bb4607eb14ec72e2bb` (tree
`75e332dc604d48f8fe1aa335f4150132da648068`), compared with
`1b42bcfff0099a6066d67fc4859e535591b39789`. Main remains pinned at
`990430b77a9f63548ffcb642074185f61ddb3322`.

No actionable new Rule finding in this bounded receiving increment. All 14
changed files and their exact deltas were read completely, including all three
reads, all five uploads, image conversions and the native/map/typed-host
paths. Truncated output was filled with explicit follow-up source ranges. JSON
records exact source identities, responsibility granularity, all thirteen
relationship lenses, contracts, TR1-TR5 dispositions and validation limits.
Source inspection preceded the independent pre-repair inventory and conversion
extension review.

TR1: Texture2d/3d retain allocation absence before creating-device admission;
TextureView retains its zero-dimension empty-first branch before allocation,
device or host-type checks. Nonempty routes converge on
TextureReadSource::read, which admits canonical Readback<T> before native
work. That existing owner rejects partial/zero-sized elements and reserves
aligned typed storage, copies bytes and only then exposes initialized
elements. No borrowed byte-Vec-to-T cast remains in these reads. Raw bits and
logical output lengths are retained.

TR2: TextureFormat owns native pixel width. Native Extent3d plus that format
admit TextureTransferLayout with private extent, row and total bytes. Row
width is calculated in u64 then checked for its u32 native port; height/depth
volume products are checked. TextureStagingLayout separately admits positive
aligned pitch, full wide volume and actual max_buffer_size. All retained byte
extents use BufferByteLength; native quantities are extracted at
descriptor/copy/slice ports. Private fields preserve the aggregate after
admission. Packed host capacity is reserved before mapping; unpack checks the
full mapped extent and iterates positive-pitch rows, excluding padding in
z-major then row-major order.

TR3: Texture staging allocation uses existing immediate observe_validation;
copy finalization/submission uses the canonical context.submit. Existing
MappingExtent and MappingRequest now have narrowly shared GPU visibility.
Request owns private staging cleanup before inspecting immediate map failure,
including synchronous callback behavior. Native poll_complete reuses
ready-first callback/map/cancellation/poll causes. Pending texture reads keep
nonblocking polls and the one-millisecond async interval; Buffer::read retains
its explicit native wait. Browser completion yields through the existing
helper. Mapped views drop before error propagation/unmap; cleanup preserves
the primary cause and no guard spans await. No arbitrary external-alias
ownership is inferred.

TR4: Both direct texture writes, image upload, six-image cube upload and cube
set_side_image use the shared tight-row transfer adapter. Only
Queue.write_texture is wrapped by immediate observation; it adds no completion
wait or encoder row alignment requirement. Source/extent validation remains
native, including accepted trailing bytes. Original absence/device checks,
color conversion and face origin order remain. Earlier queued faces are not
rolled back on failure.

TR5: Float read_image decoding now uses native-endian fixed byte chunks rather
than borrowing f32 from an alignment-one byte allocation. Existing clamp,
rounding, grayscale, alpha and format branches are preserved. The connected
conversion extension also replaces Image fallback u32 extent multiplication
with the canonical transfer byte projection and reserves cube float capacity
from actual input length, retaining allocation errors. It preserves permissive
Image admission and the intentionally distinct image/cube transfer policies.

Canonical owners remain separate: texture owns row/volume relationships;
buffer owns mapping and typed copying; context owns native
observation/submission; Image owns conversion; compute owns grouped readback.
No duplicate map/poll error owner, public compatibility API, unused
abstraction/import, unsupported cfg-only interface, new unsafe implementation,
production unwrap/expect or suppression was found. Owning aliases preserve
error distinctions. Module item order follows crates/AGENTS.md:90-94;
API/unit/owner rules at 28-71 and 118-127 are respected. Native/framework
records are explicit boundary packaging.

New transfer/readback modules have 184/107 raw lines; the shared read method
is 59 lines and the new polling method is eight. New functions stay below 100;
existing large texture files shrink, and baseline bytes are unchanged.
Required hosted baseline reductions remain pending. Three existing fixture
modules and all three embedded cube WGSL literals match parent and recorded
source proof; code fences are unchanged. No new cases, helpers, assertions or
doctests were added. The entire guide has no overlong prose. Another 122
retained source identities match, retaining their prior declared whole/slice
body-read coverage; hashes alone provide no unread-body credit.

This is a receiving source verdict, not compilation or full-family clearance.
The coordinated #780 grouped Readback/PendingReadback callback/cancellation
and five scoped-wait integrations remain required. Renderer/native
source/device, cardinality/layout/producer/creator/caller/application/lock
paths remain open. The #895 ZIP/NumPy stack is untouched. Full integrated
thirteen-lens waves, three independent reviews and required hosted project
checks must pass on the same final pushed revision. No global or final PR
clearance is granted.

No source was edited. No project execution, build, check, lint, test, scanner,
generator, runtime, push or CI ran. Only ignored reports were written. Source
inspection supplies no hardware/browser success, numerical calibration, atomic
rollback, universal error capture or global device-health guarantee.

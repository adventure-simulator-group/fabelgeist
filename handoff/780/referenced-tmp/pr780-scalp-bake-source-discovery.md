# Scalp bake producer-family source investigation

Snapshot: HEAD `12a14b478506044f85a062f07aa05855cae0c8c7`, with concurrent parent work advancing the branch
from the task's `29fe6e752` reference. The four investigated scalp source files
remain their existing implementation. Exact per-file fingerprints below pin
this investigation independently of commits elsewhere. This is discovery for a
forthcoming all-thirteen-lens inventory, not review or audit clearance.

Read-only: no source edits, project checks, builds, tests, fixture additions,
formatting, hosted CI or external posts. Arithmetic below is static evaluation
of the actual fixed recipe, not project execution.

## Concrete findings and migration obligations

1. **Incomplete producer migration, already identified by the parent.**
   `hair/gpu/bake/mod.rs:153–168` produces raw `Level { side, stride,
   offset }`. `bake:180–202` recomputes raw base and chain populations for
   allocation; `Grid::bare:331–367` and `GpuScalpTexture::finish:385–420`
   pass raw counts to four dispatch kinds: bare, blur, finish and halve.
   `KernelBatch::dispatch_folded` at compute `kernel/mod.rs:293–300` requires
   `InvocationCount`, while canonical buffer `stride.rs` already provides
   `From<BufferElementCount> for InvocationCount`. Adding four conversions
   would leave this producer and its consumers incomplete. Sorting quantities
   are unnecessary here: these are native record populations and launch
   quantities with an explicit one-record-per-invocation relationship.

2. **P2: public scalp readback loses its creating-device association.**
   `GpuScalpTexture` at bake `mod.rs:111–114` retains only buffers, and public
   `read` at `229–233` passes the caller's arbitrary `HairGpu` context directly
   to `Buffer::read`. Two valid independently created `HairGpu` instances are
   sufficient to call a texture baked on A with B. The lower operation's exact
   contract, buffer `read.rs:10–17`, requires the exact creating device and its
   paired queue. Its staging allocation/copy at `read.rs:60–80` uses B for an
   A-owned buffer. This violates that required association and can result in a
   native validation error instead of the owning API's `DeviceMismatch`.
   Retain the actual device alongside this resource and reject mismatch before
   native readback, following the existing Creator `GpuGeometry` owner at
   `hair/gpu/strips/geometry.rs:75–79`. This is a representable public API
   defect, not an observed GPU run.

3. **Retained upload metadata is part of the same migration family.**
   `hair_preview/scalp_upload.rs:36–55` queues a bare `Buffer`; image dimensions
   and all copy metadata are independently recomputed from global
   `scalp_levels()` at `62–73` and `106–130`. The actual production callers at
   `hair_preview/assets.rs:184–186` currently use the private bake's exact
   outputs, so this is not evidence of an actual wrong-sized upload today.
   Complete the resource/layout handoff rather than declaring the source family
   finished while upload still accepts an unrelated buffer under an implicit
   global packing assumption. The existing render fixture at
   `hair_preview/render_test.rs:309–312,367,401–402` retains the same chain
   buffers and needs API-only adaptation if this representation changes.

## Actual contracts and address proof

The authored recipe is exactly 2048, square, power-of-two and positive. It
produces twelve levels down to one; it does not admit arbitrary dimensions.
`max(side,64)` is aligned here because each side above 64 is itself a power of
two divisible by 64. It would not be a general row-alignment implementation for
arbitrary widths, but no such input is accepted today.

The shader storage arrays contain one native four-byte word per texel:
`topmost`/`shadowed` use atomic u32, bare ping-pong uses f32, final chain uses
packed u32 RGBA. This is exactly `BufferElementStride`'s native record spacing
contract. `BufferElementCount` owns populations; `BufferByteLength` owns extents;
`BufferByteOffset` owns byte addresses. Pixel dimensions/row count are not byte
addresses. WGSL word offsets are byte-address/stride projections, not invocation
counts or sorting counts.

| Side | Row records | Word offset | Byte offset | Row bytes |
| ---: | ---: | ---: | ---: | ---: |
| 2048 | 2048 | 0 | 0 | 8192 |
| 1024 | 1024 | 4194304 | 16777216 | 4096 |
| 512 | 512 | 5242880 | 20971520 | 2048 |
| 256 | 256 | 5505024 | 22020096 | 1024 |
| 128 | 128 | 5570560 | 22282240 | 512 |
| 64 | 64 | 5586944 | 22347776 | 256 |
| 32 | 64 | 5591040 | 22364160 | 256 |
| 16 | 64 | 5593088 | 22372352 | 256 |
| 8 | 64 | 5594112 | 22376448 | 256 |
| 4 | 64 | 5594624 | 22378496 | 256 |
| 2 | 64 | 5594880 | 22379520 | 256 |
| 1 | 64 | 5595008 | 22380032 | 256 |

Total padded chain: **5,595,072 records / 22,380,288 bytes**. Visible texels:
5,592,405. Largest visible chain address: word 5,595,008. Base population:
4,194,304. Every row pitch is 256-byte aligned, every level offset is four-byte
aligned, and every level fits 32-bit host addresses as well as native u32 word
addresses. Maximum row pitch is 8192 bytes. Every copy's last visible byte lies
within its padded level and the full chain. Last-level padding is allocated but
not required to be written or copied as pixels.

`HALVE` at `bake/wgsl.rs:338–371` reads four clamped source texels and writes a
separate, later destination region. Each coordinate and offset fits the above
bounds. The maximum component sum is 1020; unsigned channel averaging and
packing cannot overflow. The order remains largest-to-smallest, each smaller
level for each of the two textures. All current padded source/destination
regions are disjoint. `FINISH:315–334` writes the complete contiguous base
level. `BARE:247–256` and `BLUR:272–286` guard all base-population accesses.
The 64-wide workgroup/item-flattening ABI is existing source, not a proposed
new launch implementation.

`block_side=max(round(SHADOW_BLOCK/texel),1)` is nonzero, so
`blocks=ceil(2048/block_side)` is in 1..2048 even when the rounded native value
saturates; `blocks*blocks` fits u32 and the shadow map is nonempty. The
row/column guards in `DRAW:165–181` and block div/ceil relationship make its
shadowed address less than `blocks²`.

## Minimal coherent receiving design

Create one focused scalp bake layout owner, private construction and readonly
metadata. Keep this fixed authored recipe; do not add arbitrary dimensions or
invent a shared universal texture abstraction. The owner should certify the
positive side, padded row population, disjoint level byte ranges, native
four-byte stride, whole-chain byte extent and host/native address projections.
A local positive side owner (or privately retained `NonZeroU32`) can carry the
edge invariant without borrowing the unrelated card atlas tile semantic type.

Use canonical `BufferElementCount` for the base, each destination texel
population and padded record populations; canonical `BufferElementStride` for
actual four-byte storage records; `BufferByteLength` for allocations/pitches;
and `BufferByteOffset` for retained byte starts. Derive lengths from counts and
stride. A shader adapter may project row/side values and checked exact-divisible
byte offsets to their existing u32 WGSL fields; a copy adapter projects byte
pitch/offset at the actual Bevy native copy boundary. Do not retain competing
raw and typed copies of the same facts. Convert `BufferElementCount` into
`InvocationCount` where one texel becomes one launch invocation, leaving all
workgroup folding owned by #779's canonical dispatch API.

Retain this layout with the actual baked resource and its creating device.
Carry the same authority through host readback and the queued Bevy resource
copy, for example with a scalp-specific chain record exposing readonly buffer
and layout access rather than independently admitted `(buffer, layout)` pairs.
The output is two role-named texture chains; preserve their existing shade and
strand meaning. Existing public `ScalpLevel` fields should become readonly
projections of this owner rather than a second constructible layout authority.
Update actual assets and existing fixture retained state together; no new
fixtures, cases, helpers or assertions are needed.

The private `CardAtlasMipLayout` at hair `cards/atlas/layout.rs:35–41,140–190`
certifies **tightly packed** arbitrary rectangular card RGBA image extents and
card tile/variant recipes. It has no padded buffer offset, full-chain identity
or device association. Exposing/reusing it would conflate different contracts.
`fabelgeist-hair/gpu/scalp/layout.rs` is a surface-net geometry grid, also not
this texture layout.

Keep the existing WGSL bytes, packing, arithmetic, blur order and mip order.
Preserve Bevy's documented configured device-error handling at
`scalp_upload.rs:9–10`; this investigation does not justify adding a competing
native submission mechanism or claiming synchronous validation from Bevy's
queue call.

## Physical units and policies

`ScalpMap::texel_per_side` at `scalp_map.rs:47` and `texel:103–104` carry
metres as f32. Reuse canonical `Metres` through the producer, retained field and
return value. `STEP`, `REACH`, `LOCK_WIDTH`, `SHADOW_REACH`, `SHADOW_BLOCK` and
`BARE_REACH` at bake `mod.rs:58–83` are metre distances; the shade, spread,
coverage and interpolation constants at `64–99` are canonical `Ratio` values.
Extract native scalars only for actual numerical divisions or WGSL parameters.
Neither unit owner imposes finite/positive/clamp policy (`math/units.rs`). Do
not silently strengthen numerical admission or change rounding when carrying
them. The scalp map's coordinate frame remains the actual bust metre frame;
its direction/rim math remains unchanged unless separately inventoried.

## Empty, short-read and overflow dispositions

- The mip recipe never produces a zero side or empty chain. Replacing
  `scalp_image`'s `first().map_or(1,...)` fallback with the positive source
  owner's base dimension removes an impossible-state fallback; it must not
  create phantom texels for an actually empty input.
- A zero-strand groom has zero strand invocations, but fixed nonempty scalp
  texture passes still run, with cleared topmost/shadow maps. This does not
  create phantom strands. `LONGEST` retains clear+guard behavior and yields
  zero metres for zero strands.
- There is no demonstrated short-read indexing bug in current scalp chains:
  private bake allocates exact full-chain logical extents. `Buffer::read` and
  `Readback::copy_from` either return all logical bytes or a typed error;
  `readback.rs:30–39` explicitly rejects truncation. The direct host row slices
  lie within that proven extent. A fallible read is not permission to invent
  a missing-level/short-buffer fallback.
- `longest` allocates 16 logical bytes and reads four u32 words or errors; its
  `first().unwrap_or(0)` fallback cannot represent a successful empty read in
  this actual producer. This redundant fallback is not evidence of a runtime
  zero-result bug.
- `ScalpMap::fit` can represent zero radius for an empty or coincident surface.
  A zero texel metre value causes division saturation and a potentially huge
  blur reach; `position+reach` at WGSL `281` can then wrap. However its **only
  actual Creator fit caller** is `GroomedHair::grow`, and actual `cut_scalp`
  rejects zero triangles in native `GpuGroomer::cut_scalp` after the triangle-count read. No admitted Creator
  zero-radius path was established. Record numerical-policy review as open,
  not an observed empty bug or permission for a speculative shader rewrite.
- The broader native surface-net/grid producer remains an explicitly open
  family. Its own raw counts, sorting arguments and discarded submit are not
  granted clearance by this texture investigation.

## Thirteen-lens discovery support

| Lens | Concrete evidence and disposition for this family |
| --- | --- |
| Admission contracts | Fixed recipe positive/nonempty; new layout construction must certify row/offset projections; device association finding above. |
| Aggregate states | Side, stride, offset, whole-chain bytes and resource identity form one admitted record; current global recomputation has implicit coherence. |
| Behavior preservation | Exact shader source, pass order, rounding, RGBA packing, mip averaging and Bevy error policy must remain; bounds above give acceptance evidence. |
| Derivability | Small readonly value records can derive ordinary value traits; no requirement for a manual error hierarchy or speculative serialization was found. |
| Enum shapes | Existing shade/strands roles already have named record fields; avoid a new bool role. Ping-pong pass parity remains local iteration math. |
| Existing type owners | Buffer count/stride/byte quantities, InvocationCount, Metres and Ratio exist; card AtlasMipLayout owns a different contract. |
| Invariant operations | Side halving, padded row products, byte offsets, block div/ceil, shader row addresses and 32-bit bounds inventoried above. |
| Magic numbers | 2048 is authored texture resolution; 64 records comes from 256-byte row alignment and actual stride4. Physical and ratio constants have existing unit owners. RGBA channel shifts and four samples are native format/math. |
| Magic strings | Kernel parameter names are exact reflected WGSL field/binding names; labels are diagnostics. Preserve spelling and shader linkage, do not invent semantic string wrappers. |
| Producer propagation | Entire levels→allocation→dispatch→read→image→queued copy path required; actual fixture retained Buffer pair is a connected receiver. |
| Redundant authority | Global recipe is currently recomputed, but private fixed construction keeps current outputs coherent. New owner must avoid independent stored primitive/typed metadata or fresh upload recipe derivation. |
| Type granularity | Distinguish side, populations, word/byte addresses and launch quantities; use canonical native record populations without misusing card/strand/sort counts. |
| Type ownership | Padded scalp chain belongs to Creator scalp bake; GPU owns generic buffer quantities and dispatch, math owns units, Bevy owns actual copy/presentation API. |

## Coverage and limits

Full fresh reads: four requested source files; canonical buffer stride, extent,
read, readback, prefix and Buffer owner; card atlas layout; Creator GPU owner;
Creator GpuGeometry; native scalp producer and its grid-layout owner; Surface;
math units. Targeted connected bodies: GroomedHair::grow and retained texture;
Creator hair/gpu reexports; actual preview assets scalp_images; render fixture
read/stage/copy bodies; KernelBatch dispatch adapter. Fingerprinting a targeted
file below does not assert full-file or global clearance. Other producers,
Renderer, physical/grid families, AST, lock composition and final hosted
checks remain outside this discovery verdict.

The render fixture is an existing actual caller; no borrowed or invented bake
test was used. No bake-specific tests file/module exists in this source tree.
Existing lower readback fixtures were read for the documented extent contract,
not executed or extended.

## Source fingerprints (SHA-256)

| Path | SHA-256 |
| --- | --- |
| `crates/adventuresim-character-creator/src/hair/gpu/bake/mod.rs` | `70a93715a6a926de40b3f71c676839e81df669ff56f41dcbbfa1486560603466` |
| `crates/adventuresim-character-creator/src/hair/gpu/bake/wgsl.rs` | `7965432924278f1a7cbcf9a47a295a9c2b9fe4103783e8d641f54dbbd9bc70ec` |
| `crates/adventuresim-character-creator/src/hair/scalp_map.rs` | `717bbedb859bc7b5df413aeea35d20cccbdf1e08dc53f2f8d692b72dc2cc662e` |
| `crates/adventuresim-character-creator/src/hair_preview/scalp_upload.rs` | `3b0756f8287d6c95da9e9ab146be7ab5792c584fa232481cde0b6470d549320d` |
| `crates/adventuresim-character-creator/src/hair/gpu/mod.rs` | `aea64b1f78e5def266d32a1ecf4ffce201b11292c344a40ad123fa28118f5644` |
| `crates/adventuresim-character-creator/src/hair.rs` | `a9d4f30809a845df7b5e43bac564eb66824d90bb6cfed878e96c6afae8ff332c` |
| `crates/adventuresim-character-creator/src/hair/groom.rs` | `d6c46f6ef0b1f037cc45648c2baac76b9e383aad331cf1c3757f1e1150702b43` |
| `crates/adventuresim-character-creator/src/hair_preview/assets.rs` | `436b82cac0058641b1c230f488b1ade01d293b23c6822584d39ed6846476e143` |
| `crates/adventuresim-character-creator/src/hair_preview/render_test.rs` | `af7733cf9f722b473e08584c2c35569cc0962376cadc4e73875ba71bef94e739` |
| `crates/adventuresim-character-creator/src/hair/gpu/strips/geometry.rs` | `57f15b94c1f1c08bfcc0e214f95fd19640c418ad76f96d698661717076bcd6e4` |
| `crates/fabelgeist-hair/src/cards/atlas/layout.rs` | `df403b648c0f3ba7f93f2f45c4deea2f33fc0c72fedef23d9f454397dbf428dd` |
| `crates/fabelgeist-hair/src/gpu/scalp.rs` | `8497dc68d32bee89caf41854e8cb0492416acb664f9546b98b5f027e530b0b06` |
| `crates/fabelgeist-hair/src/gpu/scalp/layout.rs` | `1934e3eae9d1344b28b6cfd751a0fb691b3a81c64c1a18f2aea41a65f4e3f036` |
| `crates/fabelgeist-hair/src/surface.rs` | `0c54c21fb94e4b1df748a29cdbef562802d717acfc810211072be30d236a3298` |
| `crates/fabelgeist-gpu/src/data/gpu/buffer/stride.rs` | `9a8bfaa7b7b1d2d3c6c72d27cbd6f294c14356fee69bf72a59d4af38f89490b5` |
| `crates/fabelgeist-gpu/src/data/gpu/buffer/extent.rs` | `45ea805bbc9988317e2dba4679cf6984d153c17764ae93a3204a6289ae126b77` |
| `crates/fabelgeist-gpu/src/data/gpu/buffer/read.rs` | `ff97b2b7c684745153c4713acc58e4d535cd97dd9008bcb8aacb9e62fdd138f2` |
| `crates/fabelgeist-gpu/src/data/gpu/buffer/readback.rs` | `17ca9aa9db7efd1e261b086841babcf22764a6154628aca9405c085029b16baa` |
| `crates/fabelgeist-gpu/src/data/gpu/buffer/prefix.rs` | `62a435f0e609deb067d2c9e59481ee7e042d0ee1f16c5d662f0ea6b4bd95b4c0` |
| `crates/fabelgeist-gpu/src/data/gpu/buffer/mod.rs` | `86e926ab47cd284d07a0e14265aaf7b5ba3ce1b6cc5e2aff66328e05f67aee02` |
| `crates/fabelgeist-compute/src/kernel/mod.rs` | `47b3faf12d1fdffc6251a9dcd840b28f8ce347bbf9e377e29671b4b171f9ff74` |
| `crates/fabelgeist-gpu/src/data/gpu/compute/pass/shape.rs` | `7998ee67967dc4ea5747e1396944628c1e9e89060c350d0e75d2a91bb85c81f2` |
| `crates/fabelgeist-math/src/units.rs` | `0c76a1a79a18e833d21fc2ffb011e2a351139ee7a299df13df12fdbcf3c27492` |

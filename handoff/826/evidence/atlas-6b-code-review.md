# Independent Code Review: atlas checkpoint6b

Head: `6b22364249ae1503fe070dd7b5ce64b8e6e0e25d`.
Tree: `4868f4a5c6ee9eef4a85ed9994a02790bc61249b`.
Comparison: `04a68c493d152c5250e17bedd09d33497ab67bf3`.
Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.

No new actionable owned atlas defect was found. Required producer, creator,
native source/layout, canonical import and lock integration remains incomplete.
This is a bounded source review without family, compilation, thirteen-lens,
PR or final clearance.

All nine changed files were read fully. The JSON records 57 root source
identities and exact coverage, canonical peer identities, SDK source ports,
and existing production/fixture/shader preservation proofs.

## Source contracts reviewed

All9 changed paths fully read from Git6b, including full renderer production and
existing fixture (initial long-output truncation repaired by bounded540-1540
source reread). Full atlas layout/paint/mip/uploader ownership covered. CardMesh
production method exact parent; changed mixed fixture only one existing case
with exact numeric recipe4/32/128/seed1 and same assertions after API inverse.
No test execution or new oracle.

AtlasVariantCount rejects0; AtlasTileSpan rejects0 and uses actual i32::try_from
cause for signed raster modulus/height addresses. Positive tile and variant
leaves are private, AtlasSettings composite public fields carry these checked
owners. Default8/64x256 and transparent1/1x1 use private admitted catalog
leaves. Recipe can have native/full width overflow only at composite layout,
where checked_mul rejects before allocation.

AtlasMipLayout admits full atlas width=tilewidth*variants, height, full
bytes_per_row=width*sizeof(RgbaTexel) in u32, positive extents, u64 pixel
product narrowed to usize, checked byte product and isize addressability. Tile
layout is separately admitted. This proves host row/index/stitch extents, each
f32 working image byte capacity and height*2 step arithmetic. Smaller dimensions
monotonically shrink, so their native row/host products cannot overflow the
parent proof. zeroed<T> uses actual try_reserve_exact error then nonallocating
resize_with for private known-default arrays/Vecs.

CardAtlas holds required base plus private smaller Vec, private immutable
settings, private mip layout/pixels; no public mutation or missing first level
can diverge dimensions/pixels/variant count. AtlasMip read-only getters and
pixel_bytes cast expose actual contiguous [u8;4] data. All owned large
allocations (base, columns, coverage, shade, depth, output, smaller levels)
preserve TryReserveError/host integer causes through AtlasError. Parallel
failure returns no CardAtlas candidate; per-variant buffers drop normally. This
is not a guarantee against arbitrary allocator/provider internal failure.

Deterministic paint preserves old per-variant seed word seed XOR(v*0x9E37),
canonical Seed::xor_word is exact XOR and to_u64 returns same word. u32 variant
product fitsu64; enumerate cannot exceed admitted count. Rng SplitMix
initialization/advance and every index/range/next_f32 call order unchanged.
Parallel per-column ordering uses enumerated fixed mutable slots rather than
completion order; each column stitches its original row range. Raster path,
steps, wrapping, coverage winner, tone/depth and byte rounding/truncation are
source-equivalent for every previously successful bounded input; only indexing
uses admitted hostusize addresses.

2x2 downsample reads identical four clamped neighbor coordinates, sums each u8
channel in u32 (<=1020), rounds(sum+2)/4 to u8 and retains channel order
coverage/shade/depth/coverage. Odd extents retain original floor-halving
behavior. Loop has one required base, appends at most11 smaller levels and stops
if either axis<=1; it does not extend formerly shorter chains to both-axis1.
Native extent floor-shift and max-mip SDK rules match each actual mip;
count1..12 never exceeds full-chain max for admitted dimensions.

UploadedAtlas first checks actual base dimensions against
device.max_texture_dimension_2d and converts canonical PixelFormat::Rgba8Unorm
via Color::to_wgpu; Color ChannelLayout::Rgba.channels() owns four-channel array
shape. Feature edge color/wgpu undergpu makes adapter available only where
needed. Every copy uses actual mip extent,row bytes,height,data and ordinal
bounded12, with D2/layer1/sample1/COPY_DST|TEXTURE_BINDING. Queue::write_texture
does not require encoder-style256-row alignment; admitted tight rows remain
valid. Native device/queue pairing and provider-handled validation policy
unchanged; no global native-health verdict.

UploadedAtlas owns native view+exact canonical variant count from same upload;
independent renderer atlas_variants field removed. Initial transparent source is
zeroRGBA1x1 with one variant. Lighting groups receive that view; Globals.mode.y
extracts only that count at native shader boundary. No recomputed row/mip shape
or separately mutable source population retained in this received graph.

Host set_groom now returnsResult and admits/uploads atlas before building
candidate strand/card/head/groups/counts/bounds, then publishes allscene fields
and invalidates view. GPU set_gpu_groom preserves foreign-
device/head/source/cover guards, builds native candidates and atlasResult before
logical publication. Atlas admission failure leaves prior logical scene/mean
intact; submitted native writes target new private atlas and preserve existing
device policy. Remaining native strand/card/sim admission is not silently
certified by this candidate ordering.

04 documentation edits are source-accurate: CPU smoothing prose describes
present behavior; set_head distinguishes native allocation handling and old-head
preservation on malformed layout; set_gpu_groom documents actual
device/source/exactbytes/Nonebare/admission semantics. Its non-doc head source
bytes remain exact, prior typedcause/head identity contracts unchanged. Current
renderer temporal/finite input/convergence/draw-prefix/head/readback
relationships reread and unchanged by atlas publication.

## Required dependencies

D780-atlas-propagation — REQUIRED incomplete: Current root LodSettings still has
raw atlas_variants/atlas_size; HairLods::build and GpuGroomer::lods_against
still call old paint signature/use raw atlas cache key; creator HairGeometry
atlas_levels still reads old public fields and recomputes mip sizes. These exact
old callers are intentionally pending accepted780 handoff integration. Require
canonical AtlasSettings, Seed, fallible results, actual mip metadata and cache
key propagation; no source-complete claim or accepted deferral.

D780-native-count-source — REQUIRED incomplete: Checked Strand/Card native
layouts/count/byte/binding/empty semantics, GpuLods::device actual producer
capture, sealed source associations, native simulation
id/weight/rest/body/source receiving and allcanonical producer/caller paths
remain active root780 work. Peer CardMesh/CardSettings/CardVertex methods remain
untouched. This atlas review does not accept their old raw arithmetic.

D780-imports-lock-final — REQUIRED incomplete: Canonical Surface/head/count and
typed/fallible compute descriptor imports, creator/public Result API
adaptations, new Color dependency Cargo.lock, retained774/M06/F19 and other
connected families must complete before push or hosted gates. Final
all13/reviews/CI same pushed SHA still required.

## Validation limits

No source edits, project commands/build/check/tests/scanners/generators/runtime,
new tests/cases/helpers/assertions/doctests, push or CI. Only source/Git and
ignored report accounting.

All9 changed files read FULL, complete atlas
production/mip/nativeupload/candidate renderer paths and
usedColor/RNG/Seed/smoothstep ports source-reviewed. Prior unchanged
helper/SDK/peer source proof explicitly carried by identity; no prior verdict
accepted automatically.

No new actionable owned atlas finding. This is source reasoning about
deterministic bytes and bounds, not rendered/numerical execution evidence or
allocation/runtime success. Original valid paint/mip formulas compared directly;
no new test vectors/cases executed or written.

Required old producer/creator/native/source/count/lock/import paths are
explicitly incomplete. No family/sourcecompile/13lens/PR/final clearance. All
required independent reviews/hosted checks must repeat on same final pushed
revision.

CardMesh production implementation is byte-identical. The mixed card fixture
still has one case and four assertions; normalization of only the required
atlas constructor/getters restores the exact old fixture. Renderer fixture
and all three shaders are byte-identical. No executable project check ran.

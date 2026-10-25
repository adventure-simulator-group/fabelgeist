# Independent Code Review: retained scalp-chain family

**No actionable defect found in this scoped repair by source review.** The
previously reported public readback device-association defect is addressed.
This is not final global Code Review or all-thirteen-lens clearance for #780.

Comparison base/current HEAD:
`99d70e7c3f0bcf8db563c602f39869b91a9f23bf`, plus nine working source paths:
Creator `hair.rs`, `hair/gpu/mod.rs`, `hair/gpu/bake/mod.rs`, new
`bake/{layout,grid,readback}.rs`, `hair/scalp_map.rs`,
`hair_preview/scalp_upload.rs`, and existing `hair_preview/render_test.rs`.
The final fingerprint table pins this exact working snapshot.

The owning ledger key `scalp_bake_current_discovery` was read separately from
other ledger families. Applicable root/crates/Creator instructions and the
independent Code Review method apply. No source edits, project checks,
compiler probes, tests, formatting, hosted CI, pushes or external posts occurred.

## Successor review after ownership and comment corrections

All nine current source files were freshly reread in full, including the entire
existing render fixture. No actionable defect was found in the complete
current family. This successor review retains the same scoped limits and
comparison base; it is not global/final PR clearance.

The actual `Grid` now stores `ShadowBlockAxisCount`, a private cardinality owner,
rather than treating the number of blocks as a texel span. Its only construction
path computes ceiling division from positive spans. The actual sole caller
supplies the fixed admitted layout base (2048 texels), so the result is positive
and at most 2048 without a second clamp. Its squared population is therefore at
most 4,194,304, and encoding through `PassParameter::Unsigned` matches the old
WGSL field exactly. `shadow_elements()` derives the canonical record population
from that same owner. No independent raw block count or new numerical policy
was introduced.

`ScalpMap::mapped_side: Metres` is now named for the square root of the estimated
mapped area, with matching documentation. Its construction remains
`Metres::from_metres(area.sqrt())`; the texel return remains division by the
same positive native side. The exact original source is recovered after
reversing only this private name, docs, unit wrappers and positive-side API.
No geometry, rounding, blur radius or mapping formula changes were introduced.

Fresh reading also revalidated constructor positivity/address bounds, chain
layout/device privacy and clone preservation, read rejection before lower
native calls, canonical launch conversions, exact finish/mip uniforms, queued
Bevy layout ownership and actual existing fixture API closure. Seven other
changed-family files and all recorded dependencies remain byte-identical to
the prior scoped review; this is supported by current fingerprints, not a
historical verdict standing in for source review.

The parent's source-preservation report was read and its exact source hashes
were independently matched to current files. All 15 comparisons are true,
including complete draw/bare/finish recording bodies after exact typed
projection reversal, WGSL, existing Assets receiver, fixture adaptations,
pigment/design functions and all twelve mip address/copy equations. These are
static source/algebra comparisons, not project checks or executed tests.

## Contracts checked

### One admitted, immutable padded layout

`ScalpTextureLayout::new` is private to the bake family. It admits the native
four-byte stride through canonical `BufferElementStride`; its fixed authored
2048 side is nonzero, and it builds exactly 12 descending square mips ending
at one. It reserves all 12 metadata records fallibly before pushing. Private
`ScalpMipLevel` fields retain only positive side and canonical byte offset;
row populations, logical texel populations, row bytes and whole-chain bytes
are derived. Public readonly access cannot fabricate an invalid level/layout.
There is no independently stored raw counterpart to the byte address.

The constructor's total extent is exactly 22,380,288 bytes / 5,595,072 words.
Visible base population is 4,194,304 records. Every row is `max(side,64)`
records; all sides are powers of two, so the row pitch is 256-byte aligned.
The last visible chain word is 5,595,008, below the complete allocation.
The last mip's complete padded row remains allocated. The fixed source bounds
all additions/multiplications before the final checked host/native extent
projections. `usize` and `isize` checks cover the supported 32-bit and 64-bit
hosts; the u32 word check covers the existing shader index ABI.

`native_word_offset` divides exact word-aligned byte starts by the admitted
four-byte stride. The final cast is justified by that private constructor's
whole-chain bound. `elements` and `row_elements` also project only immutable
levels descended from the fixed 2048 side. Public `row_bytes` can receive
another admitted level, but all possible layouts use this same fixed recipe
and stride; no conflicting configurable layout is admitted.

### Typed population and unchanged native ABI

The base population, shadow-map population and destination mip populations are
canonical `BufferElementCount`. Allocation uses canonical `BufferByteLength`
derived with the admitted stride. Bare, blur, finish and halve convert these
populations through the existing `From<BufferElementCount> for InvocationCount`
only where one native texel record becomes one invocation. Strand draws retain
the existing checked groom layout's strand launch. No sorting quantity or new
workgroup/folding implementation is introduced.

`ScalpTexelSpan` admits the existing positive integer/clamp policy and encodes
directly to `PassParameter::Unsigned`; canonical row populations likewise use
the existing `PassParameter` conversion. Shader fields/bindings, u32 address
units and four-byte record layouts remain unchanged. HALVE's source and
destination regions remain disjoint and ordered; the maximum channel sum is
1020. Its four-source integer averaging and packed RGBA ordering are unchanged.

All authored physical distances now use canonical `Metres`; all migrated
shade/coverage/spread constants use `Ratio`. Their literal values are unchanged.
`ScalpMap` carries its retained metre value and return type through the actual
consumer. Native scalar extraction occurs for numerical division or shader
encoding. Base side 2048 has the same f32 representation whether the original
usize or new u32 is projected, so texel sizing and subsequent round/cast/clamp
order remain unchanged. No positivity, finiteness or alternate clamp policy is
imposed by the unit wrappers.

The complete `bake/wgsl.rs` was freshly read and is byte-identical to the base.
All seven source strings, shader arithmetic and packing are preserved. The
actual dispatch order remains draw, bare, four alternating blur passes, finish,
then each smaller mip for shade and strands. Clear-before-draw, ping-pong order,
submission-result handling and native allocation labels remain intact. The
minimum 16-byte scratch allocation policy is preserved, including a one-block
shadow grid; the chain allocations are much larger and logically exact.

### Resource identity and readback

`ScalpTextureChain` privately retains the actual `Buffer`, one shared
`Arc<ScalpTextureLayout>` and the actual cloned creating `wgpu::Device`.
Construction occurs only inside the fallible bake; shade and strands receive
the same Arc authority. Cloning the chain preserves all three associations.
It publishes only after successful `batch.submit()?`. External native aliases
remain subject to the lower access/mapping contract; immutable metadata does
not certify externally written contents or global device health.

`ScalpTextureChain::read` compares the actual device before prefix/readback and
returns existing `HairGpuError::DeviceMismatch` on mismatch. It narrows the
read to the admitted chain extent with canonical `Buffer::prefix`; lower
`Buffer::read`/`Readback` either return that entire logical prefix or preserve
a typed error. Every row slice is within that complete admitted prefix.
Padding is skipped when producing host RGBA rows; the final result keeps the
existing `AtlasLevel` width/height and unpadded byte representation.

The supplied context's queue must still be paired with its device, as required
by `WgpuContext::from_parts` and `Buffer::read`. This patch does not claim that
comparing the device can prove an arbitrary caller-assembled queue pairing.
The actual production HairGpu constructors use the existing context owner.

The longest-strand reduction still allocates/clears 16 logical bytes and reads
four u32 values or errors. Replacing the impossible `first().unwrap_or(0)`
fallback with `words[0]` is justified by that positive read extent; no successful
empty read is newly admitted. Zero strands retain the existing no-op launch and
clear result of zero metres.

### Complete actual receiver graph

Library reexports now expose the final chain/layout/mip APIs directly; no free
`levels`, `ScalpLevel` alias or compatibility receiver remains in this graph.
Visibility of `ScalpTexelSpan` is consistently restricted to `crate::hair` at
its declaration and reexports, permitting the sibling `ScalpMap` consumer.
Grid's extracted upload import reaches the actual grandparent GPU owner.
Readback, Grid, layout and module imports are consistent by source reasoning;
no compiler-success claim is made.

Production preview assets enqueue the complete `ScalpTextureChain`; the queue
retains it across frames until Bevy has prepared the destination image. Image
size and mip count come from the exact retained layout. Each native copy uses
that chain's byte offset, retained row pitch, positive side and ordinal mip.
All native casts fit the fixed 12-level/8192-byte maximum-row proof. No global
layout reconstruction or impossible 1x1 fallback remains.

Actual private call paths were inspected: the preview GPU obtains the
application's RenderDevice and paired RenderQueue in `render_context`, retains
that context in `StudioHairGpu`, grows/places in its worker and hands the same
prepared texture to `assets::scalp_images`. The existing render fixture also
creates HairGpu from that render app's exact context, clones its complete
chains into Stage and enqueues those chains for the same renderer. The private
upload path does not admit arbitrary external buffers/layouts. Bevy retains its
configured error handling; removing a completed queue item after encoding is
unchanged policy, not a new synchronous-validation claim.

The existing fixture was read in full. Its entire diff is exactly the necessary
chain import, removal of the obsolete Buffer import and Stage retained type
change. No tests, cases, fixture helpers, assertions or doctests were added.
Hair.rs declaration/order movement leaves its pigment/design bodies unchanged.

## Source comparisons and scope limits

`/tmp/pr780-scalp-chain-source-proof.json` records current fingerprints and
source-only comparisons. All three comparisons are true:

- complete WGSL file byte-identical to the base;
- complete render fixture identical after reversing only its imports/Stage type;
- complete ScalpMap identical after reversing only its units, positive-side API,
  private `mapped_side` name and documentation.

The computed fixed metadata table reproduces all original mip offsets/pitches
and the original whole-chain extent. This is static evaluation of the source
recipe, not a new project test or execution result.

Full fresh reads covered all nine changed source paths plus unchanged WGSL and
canonical buffer stride/extent/prefix/read/readback owners. Fresh connected reads
included the complete preview GPU context owner, actual grow/worker/collect and
assets upload bodies, library exports, dispatch adapter, PassParameter variants
and native units. Fingerprints of targeted files identify exact source without
claiming full-file or application clearance.

No new texture zero/empty-path defect is established: the fixed chain is always
positive, native empty scalp production is rejected upstream, and zero strands
do not imply zero texture dimensions. Pre-existing broader numerical policies
for pathological scalp geometry remain separate from this byte-identical math
migration. Native scalp geometry/grid/sorting, other physical families,
generated programs, native GPU composition, final all-thirteen-lens waves,
independent final reviews and hosted project checks remain open.

## Fingerprints (SHA-256)

Current successor snapshot; paths are relative to the isolated audit-pr-780
worktree.

| Path | SHA-256 |
| --- | --- |
| `crates/adventuresim-character-creator/src/hair.rs` | `7c06f11d607b958e8a9e000bf5b6debeda01e3c253fc863c331317382b196a9a` |
| `crates/adventuresim-character-creator/src/hair/gpu/mod.rs` | `02378baa2d1ed60f1c8337c24b9e0e5872b240203fc5f7944f3f8eb93744d2f6` |
| `crates/adventuresim-character-creator/src/hair/gpu/bake/mod.rs` | `a9be367893c88b158dac2f0368c4ef28d521e59290a428b08753c342881225bd` |
| `crates/adventuresim-character-creator/src/hair/gpu/bake/layout.rs` | `88c1868538af8a6862650cfe70083e9bcaff1785e72160d656629aabb23df23b` |
| `crates/adventuresim-character-creator/src/hair/gpu/bake/grid.rs` | `8ad5ee14907efa7548e6ddb0ba1c47be55253701644626a9879045df06a9b438` |
| `crates/adventuresim-character-creator/src/hair/gpu/bake/readback.rs` | `958943b91a03470381744264d571a83be30421501989fa92a56a403cbbec4bee` |
| `crates/adventuresim-character-creator/src/hair/scalp_map.rs` | `a410d423ff4188de5f759094665c53095193198745913dcb411d9a822f7c3f53` |
| `crates/adventuresim-character-creator/src/hair_preview/scalp_upload.rs` | `8f70a54fc5032060ecf8e46d898e5a9e02a93416e224393437a3b1852946dddd` |
| `crates/adventuresim-character-creator/src/hair_preview/render_test.rs` | `d21005ddea5a4a2ec07fd70fc03357ef5f291191d00d5b430319b685f676d531` |
| `crates/adventuresim-character-creator/src/hair/gpu/bake/wgsl.rs` | `7965432924278f1a7cbcf9a47a295a9c2b9fe4103783e8d641f54dbbd9bc70ec` |
| `crates/adventuresim-character-creator/src/hair/groom.rs` | `d6c46f6ef0b1f037cc45648c2baac76b9e383aad331cf1c3757f1e1150702b43` |
| `crates/adventuresim-character-creator/src/hair_preview/assets.rs` | `436b82cac0058641b1c230f488b1ade01d293b23c6822584d39ed6846476e143` |
| `crates/adventuresim-character-creator/src/hair_preview/gpu.rs` | `b37621447a0627773c821af9942ebf6806ec0d9936b9f4884fef81cc43dda429` |
| `crates/adventuresim-character-creator/src/hair_preview.rs` | `83895f2f6e29bda031bc1764ed1d51cc133051198e963ead3eb0ee728138ffff` |
| `crates/fabelgeist-gpu/src/data/gpu/buffer/stride.rs` | `9a8bfaa7b7b1d2d3c6c72d27cbd6f294c14356fee69bf72a59d4af38f89490b5` |
| `crates/fabelgeist-gpu/src/data/gpu/buffer/extent.rs` | `45ea805bbc9988317e2dba4679cf6984d153c17764ae93a3204a6289ae126b77` |
| `crates/fabelgeist-gpu/src/data/gpu/buffer/read.rs` | `ff97b2b7c684745153c4713acc58e4d535cd97dd9008bcb8aacb9e62fdd138f2` |
| `crates/fabelgeist-gpu/src/data/gpu/buffer/readback.rs` | `17ca9aa9db7efd1e261b086841babcf22764a6154628aca9405c085029b16baa` |
| `crates/fabelgeist-gpu/src/data/gpu/buffer/prefix.rs` | `62a435f0e609deb067d2c9e59481ee7e042d0ee1f16c5d662f0ea6b4bd95b4c0` |
| `crates/fabelgeist-gpu/src/globals/wgpu_context/mod.rs` | `dde02c3c137d5ac3722f28e2eab7db9eb82506ef0b64d182dd52a42b4876bbb5` |
| `crates/fabelgeist-compute/src/kernel/mod.rs` | `47b3faf12d1fdffc6251a9dcd840b28f8ce347bbf9e377e29671b4b171f9ff74` |
| `crates/fabelgeist-gpu/src/data/gpu/parameters/parameter.rs` | `8057e3bc54fcc2941ab0b231989785cbbff9c222088d96a10920c55572a4bf24` |
| `crates/fabelgeist-gpu/src/prelude.rs` | `1d909746b1d9fbd8135c06cb27bf2193f6f9141c1b11c26a6d3d856e1dcf0666` |
| `crates/fabelgeist-math/src/units.rs` | `0c76a1a79a18e833d21fc2ffb011e2a351139ee7a299df13df12fdbcf3c27492` |

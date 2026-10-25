# Independent Code Review: final scalp prose successor

**No actionable defect found in the complete current ten-source scalp family.**
Comparison base/current HEAD:
`c6659f6cfa4610f066c659411ade6636e21bd3b5`, plus the documentation corrections
pinned below. The separate dirty Map/native-leaf tranche is unreviewed and
outside this scalp verdict. This is a scoped receiving receipt, not global
Code Review, thirteen-lens clearance or final hosted-check acceptance for #780.

All ten source files were freshly read in full, including every WGSL string and
the complete existing render fixture. Fresh actual-consumer reads covered
GroomedHair::grow, preview request/collect, assets::scalp_images and the complete
preview GPU context owner. No source edits, project checks, compiler probes,
builds, tests, new fixtures/cases/helpers/assertions, CI or external posts were
performed. Source comparisons below are static evidence only.

## Prose corrections checked against actual code

SCALP-C1 remains corrected: the GPU introduction states the compute-kernel
architecture without the unsupported CPU/GPU speed comparison.

The bake introduction no longer makes a universal subpixel-size claim. It
explains the actual root-based noise and smaller per-strand tone variation,
which correspond to lock_noise(positions[base]/lock_width) and strand-id noise
in DRAW. No rendering-distance guarantee is asserted.

Selection is described as a packed-sample choice, not physically exact topmost
geometry. PACKING uses twelve height bits, twelve direction bits and eight
shade bits, after quantize clamps and rounds the normalized components. DRAW
uses atomicMax on that word. Direction and shade therefore participate in ties
between height bins (and clamped heights); the revised Rustdoc states that
ordering. Zero remains the absence sentinel because pack returns at least one.
The selected-sample output description matches FINISH decoding that same word.

REACH is described as the threshold for drawing sampled segments: after the
existing mapped/seen checks, visit draws the segment when the lower endpoint
height is below reach. draw iterates that entire segment; it clamps normalized
height for shading rather than clipping its geometry at reach. The revised
comment correctly avoids claiming a physical clip plane. AT_SKIN/AT_REACH are
now described as base shade shares before tone variation, matching the shader
multiplication. No constants, numerical expressions or policy changed.

## Whole current family review

The private fallible layout factory still admits one positive fixed 2048 recipe,
twelve mips and native stride 4. The complete allocation is 22,380,288 bytes /
5,595,072 records, with largest visible word 5,595,008. All row pitches are 256-byte
aligned; checked whole host/native bounds support the existing side/product,
byte/word and copy projections. No arbitrary mutable layout or second address
authority is exposed.

Grid keeps positive texel spans distinct from private ShadowBlockAxisCount.
Its actual sole ceiling-division caller uses the admitted base, so blocks lie
in 1..2048 and their squared record population fits u32. The same canonical
BufferElementCount determines allocation and converts to InvocationCount only
at the one-record-per-invocation boundary. Sorting quantities and alternative
workgroup implementations are absent.

Private ScalpTextureChain construction retains the buffer, shared Arc layout
and actual creating device together; cloning preserves the association. Bake
publishes only after submit succeeds. Chain read checks actual device identity
before prefix/native readback and reads precisely the admitted extent. Every
host row slice is bounded by that prefix. Device equality does not certify an
arbitrary caller-assembled queue pair; the lower paired-queue and external
alias/mapping preconditions remain. The actual preview/fixture context is the
same render app's device/queue.

The complete pass sequence, scratch padding, clear policy, four blur passes,
output packing, mip averaging, shader signatures and allocation labels remain
unchanged. Zero strands still produce no strand invocation and zero longest
length, while the fixed texture chain remains positive. Longest reads 16 logical
bytes or errors, so its existing word-zero index remains proved. No new physical
admission or pathological-geometry guarantees were inferred.

Actual assets and fixture Stage retain the full chains. Image dimensions,
queued byte offsets/pitches, mip ordinals and copied extents use the retained
layout rather than a reconstructed global recipe. Bevy's configured device
error handling remains unchanged; queue submission is not presented as a
synchronous validation result or proof of completion. Reexports and sibling/
child visibility remain closed over the actual consumers.

The entire existing fixture is byte-identical to c665; there are no new tests,
cases, helpers, assertions or doctests. Public source item and API bodies,
including pigment/design behavior, are unchanged. These are source-review
conclusions, not compiler/runtime results.

## Static preservation proof

`/tmp/pr780-scalp-chain-final-prose-code-proof.json` records six true comparisons:

- all ten executable source bytes match c665 after removing only Rustdoc lines;
- only hair/gpu/mod.rs, bake/mod.rs and bake/wgsl.rs differ as whole files;
- complete PACKING declaration through EOF is byte-identical;
- all seven complete WGSL strings are byte-identical;
- the entire existing render fixture is byte-identical;
- all four actual consumer files are byte-identical.

Fresh source reads accompanied these comparisons. No earlier verdict or search
result was treated as clearance. Native scalp grid, physical families,
generated programs/native-leaf changes, lower GPU composition, full main/PR
integration, final three independent reviews, final thirteen-lens waves and
hosted checks remain outside this verdict and pending. Do not merge.

## Exact ten-source SHA-256 receipt

Paths are relative to the isolated audit-pr-780 worktree.

| Path | SHA-256 |
| --- | --- |
| `crates/adventuresim-character-creator/src/hair.rs` | `7c06f11d607b958e8a9e000bf5b6debeda01e3c253fc863c331317382b196a9a` |
| `crates/adventuresim-character-creator/src/hair/gpu/mod.rs` | `e2a6fab2ecfbae1ae85e86dd40677284d6d03a50747810242247a2be2fbcf194` |
| `crates/adventuresim-character-creator/src/hair/gpu/bake/mod.rs` | `84e709e87c44bce9a1944ff54427c5b7f84cfb30eb94ff1346f8e3c181b4c5cf` |
| `crates/adventuresim-character-creator/src/hair/gpu/bake/layout.rs` | `88c1868538af8a6862650cfe70083e9bcaff1785e72160d656629aabb23df23b` |
| `crates/adventuresim-character-creator/src/hair/gpu/bake/grid.rs` | `8ad5ee14907efa7548e6ddb0ba1c47be55253701644626a9879045df06a9b438` |
| `crates/adventuresim-character-creator/src/hair/gpu/bake/readback.rs` | `958943b91a03470381744264d571a83be30421501989fa92a56a403cbbec4bee` |
| `crates/adventuresim-character-creator/src/hair/gpu/bake/wgsl.rs` | `3a2e06f64636fadaceac6bb7c0af36cfb12416a4d55a5f203e17b6551455e8ef` |
| `crates/adventuresim-character-creator/src/hair/scalp_map.rs` | `a410d423ff4188de5f759094665c53095193198745913dcb411d9a822f7c3f53` |
| `crates/adventuresim-character-creator/src/hair_preview/scalp_upload.rs` | `8f70a54fc5032060ecf8e46d898e5a9e02a93416e224393437a3b1852946dddd` |
| `crates/adventuresim-character-creator/src/hair_preview/render_test.rs` | `d21005ddea5a4a2ec07fd70fc03357ef5f291191d00d5b430319b685f676d531` |

## Actual consumers

Hashes identify full files; the explicitly named consuming bodies above were
freshly reviewed, without claiming all other behavior in those files clear.

| Path | SHA-256 |
| --- | --- |
| `crates/adventuresim-character-creator/src/hair/groom.rs` | `d6c46f6ef0b1f037cc45648c2baac76b9e383aad331cf1c3757f1e1150702b43` |
| `crates/adventuresim-character-creator/src/hair_preview.rs` | `83895f2f6e29bda031bc1764ed1d51cc133051198e963ead3eb0ee728138ffff` |
| `crates/adventuresim-character-creator/src/hair_preview/assets.rs` | `436b82cac0058641b1c230f488b1ade01d293b23c6822584d39ed6846476e143` |
| `crates/adventuresim-character-creator/src/hair_preview/gpu.rs` | `b37621447a0627773c821af9942ebf6806ec0d9936b9f4884fef81cc43dda429` |

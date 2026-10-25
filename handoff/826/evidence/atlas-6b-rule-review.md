# Independent Rule Review: admitted atlas and renderer upload

Pinned revision `6b22364249ae1503fe070dd7b5ce64b8e6e0e25d`, tree
`4868f4a5c6ee9eef4a85ed9994a02790bc61249b`, compared with parent
`04a68c493d152c5250e17bedd09d33497ab67bf3`. Main remains pinned at
`990430b77a9f63548ffcb642074185f61ddb3322`.

No new actionable owned rule finding. The atlas producer and native receiving
increment has source-backed admission, state and error contracts. Completeness
is still open: concrete peer producer/creator/native/caller/lock integrations
and hosted gates remain required. No compilation or final clearance is granted.

## Contract traces

| Family | Source-backed evidence and practical limit |
| --- | --- |
| Owners and admission | Canonical Seed, ChannelLayout::Rgba and PixelFormat::Rgba8Unorm are reused. Positive AtlasVariantCount and positive signed-addressable AtlasTileSpan enter named AtlasTileSize/AtlasSettings. `layout.rs:136–174` checks total width, u32 row bytes, usize pixels/bytes and isize allocation bounds before painting. Authored Default and the private transparent recipe satisfy those same invariants. No deserialization bypass or duplicate seed/pixel-format owner is introduced. |
| Paint and host indexing | `paint_base:11–38` validates base/tile layouts, reserves fallibly, fills each parallel column successfully, then stitches admitted rows. The enumeration is bounded by a u32 population, making its u64 seed-stride product exact. Canonical Seed::xor_word and the existing Rng preserve the old xor bits and draw order. Coverage/shade/depth calculations and output-channel quantization retain the old formulas. Index products occur at the actual host raster boundary after layout proof. |
| Mips and retained state | Private CardAtlas owns mandatory base plus smaller mips; each private AtlasMip owns its exact layout and pixels. There is no independently mutable width/height/data record. Downsampling retains the old rounded integer 2x2 mean and stops at either unit axis or twelve levels. Shrinking dimensions preserve the parent's host and native product bounds. The nonempty base and at-most-twelve chain justify native count/ordinal narrowing without unwrap. |
| Native upload | UploadedAtlas uses actual base extent, the actual Device max_texture_dimension_2d, canonical Color native format conversion, and each actual mip's bytes, row bytes and extent. The view and typed variant count are retained together. SDK29 Queue::write_texture does not require the copy encoder's 256-byte row alignment. Native creation/write errors retain the intentionally configured device policy; the Result checks image admission/limits, not global GPU health. |
| Renderer state | Initial state uses an admitted transparent 1x1 atlas. Both CPU/GPU groom receivers build the uploaded atlas and lighting groups before scene publication. Frame uniforms read the variant count from that paired upload record at the actual shader port. The separate atlas_variants authority and receiver-recomputed mip extents are removed. CPU set_groom now returns Result and preserves the prior scene on typed atlas admission failure. Remaining physical level/card/simulation admission is still required. |
| Errors and features | AtlasResult is the owning alias. AtlasError retains actual TryFromIntError and TryReserveError causes; HairGpuError's anyhow carrier retains them. Named settings and native error context keep their meanings. Color is a direct CPU dependency with defaults disabled; Color/wgpu is enabled only through Hair's gpu feature. Canonical lock integration remains peer-owned and required. |
| Existing head/native graph | The entire current head and frame-uniform bodies were re-read. Head/source byte/device proofs and default native provider policy remain unchanged. Updated set_head and set_gpu_groom documentation qualifies native handling accurately. Earlier history/view/draw/native contracts remain supported by exact preserved source regions, not automatically by old verdicts. |

The leaf texel span serves the rasterizer's positive i32 address contract; it
is distinct from screen pixels and physical lengths. Private mip dimensions and
row getters are deliberate host/native image adapters. Color's channel count
and size_of determine RGBA storage; local neighbor coordinates, 2x2 rounded
means and existing authored paint coefficients remain numerical-kernel values.

## Required connected work and inherited debt

- `lod.rs:204–208` still passes the old raw paint arguments; `gpu/lods.rs:242–249`
  retains the old arguments and tuple cache fields. #780 must carry AtlasSettings,
  the agreed canonical fixed seed and fallible results through whole producers
  and publish only successful cache values.
- Creator `hair/geometry.rs:62,100–119` still reads old variant/width/height/levels
  fields and independently halves mip dimensions. It must consume typed
  variants and the actual admitted mip chain. CardMesh and LodSettings variants
  and settings propagation remain required peer-owned work.
- The actual GpuLods device, physical strand/card/simulation layout/count/source
  admission, creator/caller/fixture/completion publication and canonical lock
  integrations remain incomplete. No accessor signature alone proves that
  complete native graph.
- Existing `lod.rs` impl ordering and production unwrap remain required #780
  repairs. `cards.rs:48–83` likewise retains an existing Default impl before later
  structs, contrary to `crates/AGENTS.md:90–94`. CardSettings, CardVertex, CardMesh
  and their production impl bodies are exact parent bytes and remain peer-owned;
  this is inherited integration debt, not a new atlas extraction regression.

## Coverage and validation limits

All nine changed paths, complete exact diffs, all four new atlas module bodies,
whole cards/head/frame-uniform bodies, manifest and current atlas renderer
constructor/state/lighting/CPU/GPU receiver paths were read. Whole renderer
responsibilities were reconciled against the prior complete D9 body read and
exact current equal-region/delta receipts; this is not claimed as a fresh read
of unchanged bodies. Eighty-five other declared source receipts match current
bytes. Canonical Seed/Color/Rng, native SDK adapters and affected peer producer
and creator source slices are recorded with exact hashes and coverage in JSON.
The thirteen-lens pre-repair ledger was checked after inspecting source.

The existing mixed card/atlas fixture has one case and four assertions. Reversing
only its authorized checked API admissions and getters reproduces the exact
parent test-module bytes; the original numbers, geometry, assertions and oracles
are unchanged. Existing renderer/dataset/Buffer test modules, five renderer POD
bodies and three shaders are exact parent bytes. No new case, fixture, test
helper, assertion or doctest was authored or executed.

Focused atlas modules have 83–212 lines; every new/expanded function is under
100 lines. The renderer shrinks from 2135 to 2103 lines, and its existing resize
allowance does not grow. Module/item order, coherent aliases, derives, cause
ownership and dependency feature gates were inspected. No new production
unwrap/expect, unsafe or lint suppression was introduced. Baseline reductions
are still required through hosted checks; no ceiling was raised.

No source edits, project scanner/check/build/test/runtime/generator, push or CI
was run. Only read-only Git/source/hash accounting and ignored report writing
occurred. Pixel equivalence is a source formula/bit-flow argument, not executed
validation. Full integrated audit waves, all three independent reviews and
required hosted checks must pass on the same final pushed revision.

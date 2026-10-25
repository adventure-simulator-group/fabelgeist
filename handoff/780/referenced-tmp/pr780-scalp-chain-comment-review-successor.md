# Fresh scalp-chain Comment Review successor

Snapshot: HEAD `c6659f6cfa4610f066c659411ade6636e21bd3b5` with the current
single dirty scalp source `hair/gpu/mod.rs`. Separate dirty generated-Map work
was excluded. Repository:
`/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator`.

This report supersedes the prior nine-file Comment Review snapshot. It follows
fresh complete reads of all nine sources listed below and actual current
owners/consumers. Historical verdicts and discovery searches are not clearance.
Applied comment-review skill and root/crates/Creator AGENTS. No source edits,
local project checks, tests, cases, helpers, assertions or external posts.

## Corrected finding

SCALP-C1 is resolved in the current GPU module introduction (lines3-5).
The unsupported ten-times/all-CPU-cores comparison and multiplier are absent.
The replacement says independent strand sweeps run as compute kernels. Actual
GpuGroomer::grow_look/grow records GPU kernels, and the bake DRAW/LONGEST native
programs select one strand per invocation. Studio render_context retains the
renderer Device/Queue; export HairGpu::headless calls headless_context, which
requests a new native device. The revised introduction makes no numerical speed,
benchmark, hardware or workload claim.

## Further actionable P3 comment findings

### SCALP-C2: Unbounded pixel-footprint claim

`crates/adventuresim-character-creator/src/hair/gpu/bake/mod.rs:12-13` says
single strands are much finer than one pixel at any viewing distance. No minimum
camera distance, image resolution or projected-strand-width bound establishes
that claim. The existing render fixture owns five finite camera positions and
640-pixel images, not a general renderer restriction. Grid/bake code instead
expresses a deliberate lock-tone representation: lock noise is sampled at the
strand root and combined with per-strand variation (wgsl.rs:216-218).

Describe that intent without the universal claim, for example: "The bake shades
locks rather than resolving each strand's width: strands rooted together share
a tone, with a smaller variation among them." No rendering change is requested.

### SCALP-C3: Topmost means quantized packed-height order

`hair/gpu/bake/mod.rs:9-10,27-29` describes the physically topmost strand as the
winner. Native `PACKING` uses 12-bit rounded/clamped height, then12-bit direction,
then8-bit shade (wgsl.rs:8-28); `draw` also clamps normalized height to1
(wgsl.rs:166-168). Within one height bin, and above the clamped reach, direction
and shade can select a physically lower sample. That is deterministic packed
order, not exact physical topmost selection.

Qualify the description with "quantized height" and state that direction/shade
break ties, for example: "An atomic maximum selects the largest packed value:
quantized height first, then direction and shade." The unchanged WGSL header
at wgsl.rs:5-6 has the same physical-topmost wording and should be qualified in
the connected comment repair. Preserve the actual format and code bytes.

### SCALP-C4: REACH is a segment threshold, not exact draw clipping

`hair/gpu/bake/mod.rs:58-60` describes REACH as how far off the skin a strand is
drawn. `visit` instead draws a whole sampled segment when both endpoints are
within SHADOW_REACH and either endpoint is below REACH
(wgsl.rs:194-204). Thus a crossing segment includes samples above REACH; its
height/shade are clamped rather than geometrically clipped (166-168).
Subdivision at STEP limits segment length (224-228) but does not make REACH a
hard clipping plane.

Use wording such as "The near-skin threshold for drawing a sampled strand
segment; segments crossing it are drawn whole, with height clamped for shading."
Do not change the sampling/clipping policy to satisfy the old wording.

## Complete current source contracts rechecked

All nine files were read from beginning to end, including the complete existing
render fixture. Additional current source read: complete bake WGSL, GroomedHair/
PlacedHair producer, ScalpCap UV consumer, buffer prefix/read owners, explicit
completion owner; actual Studio device constructor, export headless initializer,
preview scalp_images call, KernelCache get, KernelBatch folded dispatch/submit,
GpuGroomer grow_look/grow and headless_context relevant bodies.

- ScalpMap retains square-root estimated spherical-cap area in Metres and
  divides by positive fixed texture side. Its 'about how much skin' estimate
  stays qualified. CPU/GPU mapping uses the same radial sine-half-angle and
  inverse concentric-square sectors. The named concentric area/adjacency
  property is supported by the authors' paper [A Low Distortion Map Between
  Disk and Square](https://www.tandfonline.com/doi/abs/10.1080/10867651.1997.10487479).
  The GPU hash core matches the author repository's lowbias32 constants and
  xor/multiply sequence, after this implementation's input offset:
  [Hash Function Prospector](https://github.com/skeeto/hash-prospector).
- Private ScalpTextureLayout factory admits fixed positive2048 base,12 descending
  mips through1, four-byte stride, padded row pitch at least64 texels and checked
  complete host/native word extent. `bytes()` includes the final row padding.
  Private level construction supports direct indexing; no arbitrary caller
  layout can enter the chain. Native offsets and row-byte comments match values.
- Grid carries the actual immutable layout and separate texel spans/block count.
  Positive side/block implies positive ceiling count no greater than base side;
  fixed base <=2048 proves its square fits native u32. Its docs do not confuse
  shadow-block count with texel length. Draw records strand invocations; bare
  records texel invocations; four blur passes are row/column twice.
- A chain privately retains actual allocation, Arc layout and actual native
  creating Device. Bake returns after fallible batch submit, without claiming
  native completion. Chain access docs allow external aliases under the buffer
  contract. Read rejects a foreign Device before prefix/read, reads the admitted
  logical extent and strips each stored row's padding into AtlasLevel RGBA.
- Preview queue clones whole chains; its empty images and copy offsets/pitches/
  mips derive from each chain's layout. Actual preview assets and fixture Stage
  consume shade/strands chains directly. Upload source explicitly disclaims a
  synchronous native validation verdict and follows configured Bevy error
  handling. No CPU texel readback occurs on this upload path.
- Mip code averages four stored byte channels with integer division. Host longest
  returns Metres from its own completed buffer read; GPU arc-length accumulation
  and nonnegative-bit atomic ordering were read. No different physical policy,
  shader arithmetic or renderer projection is proposed by this review.
- HairGpu finish directly awaits canonical submitted_work_done; current docs
  distinguish native marker/poll errors, browser callback delivery, prior
  producing submission and later health. Actual owner and fixture await calls
  agree with those statements.
- Existing render fixture uses the Studio renderer Device, typed chain Stage,
  explicit texture read for PNG exports, five current views, environment switches,
  and two fallible awaited finish calls. Fixed frame settling/capture/write
  delays are policy, not an audited guarantee that asynchronous screenshot I/O
  completes. No fixture execution or screenshot result is claimed.

## Scope limits

The three new P3s mean this snapshot is not clear. Parent must inventory any
repair before editing and obtain a fresh successor for the exact resulting
whole-file bytes. No source change was made here. SCALP-C1 is resolved; that
alone does not clear the nine-file family or the PR.

Broader unchanged hair-library Viewer/Dataset prose, physical/native Grid,
whole-groom poison/scratch recovery claims, full finite collision guarantees,
pigment constructor admission, native shader/backend behavior, GPU resource
composition and global final reviews remain open. This wave read the nine
whole sources and their scalp contracts; it did not prove every unrelated API
claim inside hair.rs or the entire GPU grow dependency graph. The HairGpu
poison-recovery explanation cannot become global lock-family clearance from
this local source pass. Known global dependencies remain explicitly unresolved.

## Exact nine-source SHA-256 snapshot

Paths relative to `crates/adventuresim-character-creator/src/`.

| Source | SHA-256 |
| --- | --- |
| `hair.rs` | `7c06f11d607b958e8a9e000bf5b6debeda01e3c253fc863c331317382b196a9a` |
| `hair/gpu/mod.rs` | `e2a6fab2ecfbae1ae85e86dd40677284d6d03a50747810242247a2be2fbcf194` |
| `hair/gpu/bake/mod.rs` | `a9be367893c88b158dac2f0368c4ef28d521e59290a428b08753c342881225bd` |
| `hair/gpu/bake/layout.rs` | `88c1868538af8a6862650cfe70083e9bcaff1785e72160d656629aabb23df23b` |
| `hair/gpu/bake/grid.rs` | `8ad5ee14907efa7548e6ddb0ba1c47be55253701644626a9879045df06a9b438` |
| `hair/gpu/bake/readback.rs` | `958943b91a03470381744264d571a83be30421501989fa92a56a403cbbec4bee` |
| `hair/scalp_map.rs` | `a410d423ff4188de5f759094665c53095193198745913dcb411d9a822f7c3f53` |
| `hair_preview/scalp_upload.rs` | `8f70a54fc5032060ecf8e46d898e5a9e02a93416e224393437a3b1852946dddd` |
| `hair_preview/render_test.rs` | `d21005ddea5a4a2ec07fd70fc03357ef5f291191d00d5b430319b685f676d531` |

# Final ten-source scalp-chain Comment Review successor

Snapshot: HEAD `c6659f6cfa4610f066c659411ade6636e21bd3b5` plus three
scalp Rustdoc-only dirty files: hair/gpu/mod.rs, bake/mod.rs and bake/wgsl.rs.
Separate generated Map/native-leaf work is excluded. Repository:
`/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator`.

Verdict: no remaining actionable Comment Review finding in the current scalp
migration/documentation scope. SCALP-C1/C2/C3/C4 are resolved in these exact
bytes. This is source-only scoped clearance, not global hair or PR clearance.

Applied comment-review and repository/crates/Creator rules. Freshly read all
ten files from beginning to end, including the complete existing render fixture
and all bake native strings. No historical verdict replaced those reads. No
source edits, project checks, tests, new cases/helpers/assertions or hosted CI.
Only read-only source/reverse-byte comparisons and report writing.

## Findings resolved against actual current implementation

- C1: GPU introduction describes independent strand sweeps as compute kernels;
  the numerical GPU/CPU speed comparison is gone. Actual GpuGroomer GPU recording
  and each-strand bake DRAW/LONGEST invocation support this description.
- C2: the unlimited viewing-distance/subpixel assertion is gone. Root-based lock
  noise and smaller strand variation directly describe DRAW's root noise,
  LOCK_SPREAD 0.5 and STRAND_SPREAD 0.15.
- C3: bake introduction now states selected sample, clamped normalized 12-bit
  height and direction/shade tie order. API docs say selected strand sample.
  PACKING Rustdoc states quantized/clamped packed ordering rather than exact
  physical topmost. The actual 12/12/8 bit packing, clamp, atomicMax and zero
  absence sentinel match these statements. No exact physical winner is promised.
- C4: REACH now describes near-skin sampled-segment threshold and whole crossing
  segments with clamped shading. visit requires both endpoints inside shadow
  reach and either below draw reach, then draws the complete segment. Base shade
  coefficients are documented before lock/strand tone variation, matching draw's
  multiplication. STEP remains a segment subdivision bound, not a draw clip.

## Whole-family source evidence

- Fixed private layout starts positive 2048, descends through 12 mips to 1, carries
  four-byte stride and exact padded byte offsets/row pitches. Full extent admits
  host usize/isize and native u32 word projection. Final row padding is included.
  The immutable layout and private level constructors substantiate base/direct
  indexing comments. ShadowBlockAxisCount is separate from ScalpTexelSpan; its
  positive ceiling is bounded by fixed base side, and its square fits u32.
- ScalpMap's mapped_side is square root of estimated spherical-cap area in Metres,
  divided by positive texture side. 'About' texel-width wording stays qualified.
  CPU and GPU radial and concentric-square mapping formulas agree in this
  source contract. Named projection/lowbias32 provenance remains unchanged;
  primary paper and author repository evidence were checked during the prior
  finding discovery and are listed below, not used as source-read clearance.
- Bake retains native Buffer plus Arc layout plus actual creating Device in each
  chain. Read rejects foreign Device before Buffer prefix/mapping, reads admitted
  logical extent and removes per-row padding into the matching AtlasLevel bytes.
  No bake-return completion guarantee is made. Public buffer aliases explicitly
  follow the buffer access contract.
- Grid's actual uniforms use typed Metres/Ratio projections, texel spans and block
  counts; it records strand versus texel invocation populations separately. Four
  blur passes are rows/columns twice. FINISH emits selected direction/shade and
  coverage; HALVE averages four stored channels with integer division. LONGEST
  sums strand arc lengths and host read produces Metres. Shader policy unchanged.
- Fresh producer/caller reads: complete GroomedHair/PlacedHair; preview scalp_images
  actual call; Studio render_context actual device/queue construction; ScalpCap UV
  consumer; fixture Stage and spawn. Preview upload retains whole chains, builds
  images from their layout, copies each mip using that chain's offsets/pitches,
  and expressly uses configured Bevy error handling without synchronous verdict.
- Fresh receiving-owner reads: Buffer.prefix and Buffer.read complete sources;
  WgpuContext.submitted_work_done complete source; KernelBatch folded dispatch
  and submit bodies. They preserve logical prefix, exact device ownership,
  producing-submission handling and separate completion/readback semantics.
  HairGpu finish and existing awaited fixture calls match those owners.
- Existing fixture's cameras, environment controls, typed-chain Stage, host scalp
  PNG read and GPU upload path were read completely. Fixed settling/write frame
  delays remain fixture policy, not executed evidence of file-writing completion.

## Limits

This clears the corrected scalp migration/docs against the read receiving
contracts only. Global hair Viewer/Dataset prose, full physical/native Grid,
poison/scratch recovery, complete finite collision guarantees, pigment admission,
all native backends/resource composition and final composed PR reviews remain
open. Reading the whole hair.rs/GPU module does not prove every unrelated API
invariant or every dependency path. No local/native execution or image quality
result is claimed. Parent must use this exact whole-file snapshot and obtain
fresh review if its bytes change.

Primary provenance references:
[A Low Distortion Map Between Disk and Square](https://www.tandfonline.com/doi/abs/10.1080/10867651.1997.10487479),
[Hash Function Prospector](https://github.com/skeeto/hash-prospector).

## Exact final ten-source fingerprints

Paths relative to `crates/adventuresim-character-creator/src/`.

| Source | Current SHA-256 |
| --- | --- |
| `hair.rs` | `7c06f11d607b958e8a9e000bf5b6debeda01e3c253fc863c331317382b196a9a` |
| `hair/gpu/mod.rs` | `e2a6fab2ecfbae1ae85e86dd40677284d6d03a50747810242247a2be2fbcf194` |
| `hair/gpu/bake/mod.rs` | `84e709e87c44bce9a1944ff54427c5b7f84cfb30eb94ff1346f8e3c181b4c5cf` |
| `hair/gpu/bake/layout.rs` | `88c1868538af8a6862650cfe70083e9bcaff1785e72160d656629aabb23df23b` |
| `hair/gpu/bake/grid.rs` | `8ad5ee14907efa7548e6ddb0ba1c47be55253701644626a9879045df06a9b438` |
| `hair/gpu/bake/readback.rs` | `958943b91a03470381744264d571a83be30421501989fa92a56a403cbbec4bee` |
| `hair/scalp_map.rs` | `a410d423ff4188de5f759094665c53095193198745913dcb411d9a822f7c3f53` |
| `hair_preview/scalp_upload.rs` | `8f70a54fc5032060ecf8e46d898e5a9e02a93416e224393437a3b1852946dddd` |
| `hair_preview/render_test.rs` | `d21005ddea5a4a2ec07fd70fc03357ef5f291191d00d5b430319b685f676d531` |
| `hair/gpu/bake/wgsl.rs` | `3a2e06f64636fadaceac6bb7c0af36cfb12416a4d55a5f203e17b6551455e8ef` |

## Exact source reversal and native-string evidence

Compared each complete current file against baseline HEAD above. Reversing the
complete changed line regions in memory reconstructed each complete baseline
file byte-for-byte; no production source was written. Every changed line in
three changed files is Rust documentation (blank lines permitted), with no Rust
body/native-string change. The other seven files are whole-file byte-exact.

| Source | Changed regions | Changed lines only Rustdoc | Exact whole-file reverse |
| --- | --- | --- | --- |
| `hair.rs` | 0 | True | True |
| `hair/gpu/mod.rs` | 1 | True | True |
| `hair/gpu/bake/mod.rs` | 6 | True | True |
| `hair/gpu/bake/layout.rs` | 0 | True | True |
| `hair/gpu/bake/grid.rs` | 0 | True | True |
| `hair/gpu/bake/readback.rs` | 0 | True | True |
| `hair/scalp_map.rs` | 0 | True | True |
| `hair_preview/scalp_upload.rs` | 0 | True | True |
| `hair_preview/render_test.rs` | 0 | True | True |
| `hair/gpu/bake/wgsl.rs` | 1 | True | True |

Complete PACKING declaration through EOF byte-exact: `True`.
All seven named native strings and their identities byte-exact: `True`.

| Native string | Current SHA-256 | Byte-exact baseline |
| --- | --- | --- |
| `PACKING` | `68e562d16824babb89a78c62cae0bd197a2c0734fbac4bbac1be5e00e9e515d4` | True |
| `DRAW` | `737ebf512f46ab78f4cf381667067918ebae471dba4823ce106c6f042133dec2` | True |
| `BARE` | `759c6c248d6ceea790701bb70d6aa6fef4e0837319662df268041dacaac78d2d` | True |
| `BLUR` | `dea5efa5228f388a7ffb56f1a6b792052bb1fa822ec09d515c0337abbb48ee95` | True |
| `FINISH` | `961d4e6cb01f4125a58d832554cf6844991bdfae752bc5075032a22dbf8e85ee` | True |
| `HALVE` | `cee3bacebdef987519c18c01262f55b6c1766605bfd433d6b4662d7722b3c2a7` | True |
| `LONGEST` | `1f77f2de6f898bcc5a7f977bfaa23cfa42c3f4ced5298027f106cb0c1b7fdbc7` | True |

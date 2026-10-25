# Independent Code Review: Creator strip receiving

No remaining actionable defect found in the reviewed Creator strip receiving
family after Root corrected the retained longest-length field. This is a scoped
static verdict; the complete PR and pending lower/physical families are not
cleared, and no compile or hosted-check success is claimed.

## Snapshot and method

Root HEAD `fbab0a1c55b7e1421ec2dbe609b6df539833171c` plus the current working changes, pinned below.
Read `creator_strip_owner_current_discovery` contracts across all thirteen
lenses, along with actual root/crates/Creator instructions. Equipment workflow
instructions do not apply to this hair receiving change. Independently read the
complete strip source/geometry/WGSL, hair geometry/groom, preview/assets/material
and vertex shader, controls/export bodies, existing render fixture and its
capture tail. Read Studio::new in full and bake::longest plus its producer chain;
the bake module was also read in full. Revalidated canonical mesh strip/count
and hair source/card/atlas/policy dependencies and actual kernel/buffer APIs.
Existing verdicts and searches were not substituted for those source reads.

No source edits, local project builds/checks/tests/formatting/scanners, hosted CI,
pushes or external posts were performed by this reviewer. No tests, cases,
helpers or assertions were added.

## Finding resolved during review

[P1] The initial snapshot retained `PlacedHair.longest: f32` at
`hair/groom.rs:64`, while bake::longest and GroomedHair.longest now carried
canonical Metres. `GroomedHair::prepare` therefore assigned Metres to f32 at
line 184, and `PlacedHair::strip_source` passed f32 to StripSource's Metres at
`hair/geometry.rs:85`. These were source-demonstrable incompatible types in this
new migration, not an out-of-scope lower-layer dependency. Reported immediately
to Root. Root changed the retained field to `Metres`; reread the corrected
GroomedHair/PlacedHair declarations, prepare assignment and StripSource flow.
Both retained fields now use the same canonical unit, extracted only by actual
subdivision numerical policy. This finding is resolved in the groom fingerprint
below. No primitive adapter was introduced.

## Source-backed review coverage

- MeshStripRows admits >=2 rows and checked subdivision. NonZeroU32 pieces
  retain the existing clamped 1..8/16 numerical policy. MeshStripLayout admits
  complete native vertices and corners before output allocation; its canonical
  face iterator preserves the original strip/segment order and winding.
- Ribbon drawn populations are canonical StrandCount prefixes of the actual
  GpuLevel. Source stride remains its original admitted point count; subdivided
  output rows are separate. Nonempty source stride >=2; empty sources return
  zero from strands_drawn before clamping, so no phantom strand is selected.
  Empty outputs still have positive native allocation floors, but zero logical
  vertices/faces/dispatch work. A budget too small for one strip may return zero.
  Existing zero-share minimum one for nonempty affordable populations is retained.
- Card output shape consumes actual GpuCards::strips().topology(), with its
  separate source rows and subdivision. Its source was already admitted by the
  native four-vec4-per-vertex writer proof; output attributes index one vec4 per
  generic vertex. Actual hair read-only level/card records retain private
  canonical layouts and immutable allocating-device metadata.
- For nonempty strips the canonical corner bound limits output rows and all
  WGSL `row + spread`, row/edge and lift addresses; zero strips execute no row
  body. Source card addresses are bounded by the native writer proof; ribbon
  source addresses use the checked native source prefix. Generic face indices
  never exceed total vertices minus one. Byte extents use u64 canonical vec4
  bytes; the lift buffer has one row record for two vertex records.
- Strip entry points check both PlacedHair's source device and actual native
  level/card device against HairGpu. PlacedHair privately pairs body with LODs
  prepared on that same context. GpuGeometry privately retains the allocating
  device and one immutable layout with its four attribute buffers. Readback
  checks the actual device, then admits exactly layout.vertices().vec4_bytes()
  through Buffer::prefix. Actual Buffer::read returns zero logical elements for
  an empty prefix and never returns positive allocation padding.
- KernelBatch::dispatch_folded consumes the canonical MeshStripCount-to-
  InvocationCount conversion and actual reflected declaration; canonical folded
  address admission and actual device-grid checks precede recording. Empty
  grids skip recording. Strip build propagates dispatch and submit errors before
  returning GpuGeometry; submission Ok is not claimed as GPU completion.
- Bevy assets derive placeholder/skin vertex counts and triangle indices from
  canonical GpuGeometry. StrandShading continues binding the actual same four
  native buffers to 102..105; the vertex shader reads the matching attributes.
  Existing source/device association is preserved into the preview's borrowed
  rendering context. Shader strings BINDINGS, CARD_ROWS, RIBBON_ROWS and WRAP
  are byte-identical to their prior inline versions; strand_shading.rs and
  hair_vertices.wgsl are byte-identical to Root HEAD.
- Form indices, StrandCount, PointsPerStrand, CardCount and Ratio flow through
  PlacedHair metadata, preview labels/slider, export form, Studio initial form
  and existing fixture formatting. Distinct card/strand indices use separate
  same_level arms. Fallible strands_drawn propagates through preview to the
  existing status channel. Card missing-level fallback and strand missing-level
  None behavior remain as before. Atlas presentation consumes each canonical
  mip's actual extents/bytes instead of independently halving dimensions.
- Creator's direct mesh dependency enables its default GPU byte/parameter
  adapters; direct wgpu 29 dependency matches the device type used by hair/GPU
  and existing lock resolution. Both locks have no package additions/removals.
  Relative to Root HEAD each adds only Creator -> fabelgeist-mesh, Creator ->
  existing wgpu (29.0.4 in the Creator lock), and mesh -> existing thiserror
  2.0.20. Canonical exports, method visibility and error-to-anyhow conversions
  used by this family were inspected in their actual owners.
- Existing render_test.rs is byte-identical to HEAD after reversing exactly
  the two format calls from level.index() to the former primitive interpolation.
  No fixture cases/helpers/assertions or oracles changed. This source comparison
  was performed without running the ignored fixture or any project checks.

## Boundaries

This covers the connected receiving source family, not all Creator systems.
Larger dependency modules were read for relevant public types, source records,
exports and dispatch/read/submit operations, not every unrelated implementation.
Main coverage is Studio::new and its imports/state declaration, not unrelated
application setup. Existing raw dispatches in untouched scalp bake/grid code,
broader Root GPU LOD grid math and other physical/lower families remain pending
in the overall audit. They are neither waived nor granted clearance by this
report. Complete thirteen-lens waves, independent final combined reviews and
required hosted project checks remain required on one final pushed revision.

## Current SHA-256 fingerprints

```text
bd579f300ecfbd985eea5d4c4f5a177883380b6282f63075b005b791092c52ee  crates/adventuresim-character-creator/src/hair/gpu/strips.rs
57f15b94c1f1c08bfcc0e214f95fd19640c418ad76f96d698661717076bcd6e4  crates/adventuresim-character-creator/src/hair/gpu/strips/geometry.rs
525528762142f6b0e781d6f29ee1b58f66055b6e41873a86c6da96ba14b15e10  crates/adventuresim-character-creator/src/hair/gpu/strips/wgsl.rs
eb74c6b921850545dc9e83a00ac3091d2ffa4cfe661bd5f685c172b7da607056  crates/adventuresim-character-creator/src/hair/geometry.rs
d6c46f6ef0b1f037cc45648c2baac76b9e383aad331cf1c3757f1e1150702b43  crates/adventuresim-character-creator/src/hair/groom.rs
70a93715a6a926de40b3f71c676839e81df669ff56f41dcbbfa1486560603466  crates/adventuresim-character-creator/src/hair/gpu/bake/mod.rs
7dbe38c17f4941b7200bd8d506fa5055f75abd4dee1e434716d0967ae8f035c1  crates/adventuresim-character-creator/src/hair_preview.rs
da0250550989fb53fd8708be317c7e5aed3c002c9e02e4ab1e7103df0edb5613  crates/adventuresim-character-creator/src/hair_preview/assets.rs
13592232826ab0676fdf0cf93edd150713c436e3c73f743bd5d828072e1224b9  crates/adventuresim-character-creator/src/hair_preview/strand_shading.rs
62cbd9e3f606c85f4212494802e2675daa55387a27ac0133c8d79f80c0282b92  crates/adventuresim-character-creator/src/hair_preview/hair_vertices.wgsl
f141e4878ee8868ded50d8a481f8f1effb5c1ee4bb382fbcec321918c254c57e  crates/adventuresim-character-creator/src/hair_preview/render_test.rs
b412435396639226eabf4bb624b52eaa6c4ec25f1697b3b5987d569d7bf7440a  crates/adventuresim-character-creator/src/hair_controls.rs
ac11b7bb583e07eaa622342d8830d2bf203f8d3666706eb055af056f0abc3bd4  crates/adventuresim-character-creator/src/character_export/hair.rs
566a69fc6cd50bd7d7ea5c0347567c39f2bdc37e9f8004675dc5328e3751e12e  crates/adventuresim-character-creator/src/main.rs
1346098aa561b83aa6d045c8744f19edc6605674fd0bbf313af4dcfc4ef595e5  crates/adventuresim-character-creator/Cargo.toml
e76fa8ed410f3a4c8609d2b7528761832a8103ba075659ecfb3d1cc85a268d69  Cargo.lock
7e585078424d2838c9d07724cfa687802d6de81a4b1be33ca856b49122075a0c  crates/adventuresim-character-creator/Cargo.lock
a9d4f30809a845df7b5e43bac564eb66824d90bb6cfed878e96c6afae8ff332c  crates/adventuresim-character-creator/src/hair.rs
f8844fc7362364c5217079615e4459865865020ba4a36f5b25a8997cdd9c8f92  crates/adventuresim-character-creator/src/hair/gpu/mod.rs
462eb12c93f953673bc177a3cef00835889022f59d278bf1884086fc328c2adc  crates/adventuresim-character-creator/src/hair_preview/gpu.rs
b6a30a9372b2584916183463c9c88b45f69dc4f295c9de1c2428875036f73f94  crates/fabelgeist-mesh/Cargo.toml
a1d2e436ae07dc79d25a7f0b670df3a3d7d3b3554d495db6542bd34f0c63f653  crates/fabelgeist-mesh/src/lib.rs
e609778b6bd24b4ee231f3d3281d463764a55d5b787f02f4015f4ff7c4b4072b  crates/fabelgeist-mesh/src/count.rs
dd53d7fa66134977308f2dd0279c5d38bef8e907c739f6098904a40541a37208  crates/fabelgeist-mesh/src/count/strip.rs
bcead2da272edc0fbcd1d285004d9671ad310603a5870bf1c7df114be337c4c4  crates/fabelgeist-mesh/src/count/gpu.rs
ac4ba0f97b16faedfa175a5d366122267e2066f857f748a87803db40a3b3ac17  crates/fabelgeist-hair/Cargo.toml
7814ed1212fc58bea86aebf290862d5937f4e3e20d2b508050d20983948323a8  crates/fabelgeist-hair/src/gpu/lods/records.rs
ecdfc0cc3c30a0ec0b276f91a9799b7362430cd4ac9a281d208f68fcadc071a0  crates/fabelgeist-hair/src/gpu/layout.rs
7affba536778dd1ae65da9b834c2400b2eddd4ce6bc6a7dc7aba5048a5fbf878  crates/fabelgeist-hair/src/cards/layout.rs
6b7a180a3ee2faa7fc26a79e2ae14ceeac4a54c19f96a308dc57f417c56ea49f  crates/fabelgeist-hair/src/cards/layout/gpu.rs
7cb59f0ac4dcfa2ea8643dc6384fdc15652d99d576f26a12fad2b43145cfc74b  crates/fabelgeist-hair/src/cards/atlas.rs
16109d7ba1a21b99e1aaebad3835ed0dcd18d540dcd3e2961d9d05f706c857dd  crates/fabelgeist-hair/src/lod/policy.rs
86e926ab47cd284d07a0e14265aaf7b5ba3ce1b6cc5e2aff66328e05f67aee02  crates/fabelgeist-gpu/src/data/gpu/buffer/mod.rs
62a435f0e609deb067d2c9e59481ee7e042d0ee1f16c5d662f0ea6b4bd95b4c0  crates/fabelgeist-gpu/src/data/gpu/buffer/prefix.rs
ff97b2b7c684745153c4713acc58e4d535cd97dd9008bcb8aacb9e62fdd138f2  crates/fabelgeist-gpu/src/data/gpu/buffer/read.rs
17ca9aa9db7efd1e261b086841babcf22764a6154628aca9405c085029b16baa  crates/fabelgeist-gpu/src/data/gpu/buffer/readback.rs
0fae346f555f78e5acee52b5631e6b36c62fb78b02daba9b9ff9df1ee590312c  crates/fabelgeist-gpu/src/data/gpu/buffer/definition.rs
47b3faf12d1fdffc6251a9dcd840b28f8ce347bbf9e377e29671b4b171f9ff74  crates/fabelgeist-compute/src/kernel/mod.rs
0c76a1a79a18e833d21fc2ffb011e2a351139ee7a299df13df12fdbcf3c27492  crates/fabelgeist-math/src/units.rs
```

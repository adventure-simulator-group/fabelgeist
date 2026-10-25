# Independent complete Creator strip Comment Review

Baseline HEAD: `fbab0a1c55b7e1421ec2dbe609b6df539833171c`.
Snapshot: current dirty receiving graph after the corrected PlacedHair.longest
field was reread as Metres. SHA-256 fingerprints below pin the actual source.

Verdict: FINDINGS — three actionable P3 comment/API mismatches in the requested
family. They are pre-existing prose carried through this migration and require
disposition; this review does not claim global or final-revision clearance.

## Findings

1. P3 — qualify the segment-length guarantee in
   `src/hair/gpu/strips.rs:15-16` and its LONGEST_SEGMENT doc at line 56.
   The header promises every segment is cut into pieces no longer than the
   target. StripSource::pieces (lines 97-99) estimates a segment from longest
   strand arc length divided by source segment count, then caps subdivisions
   at MOST_CARD_PIECES=16 or MOST_RIBBON_PIECES=8. It does not measure the
   longest individual source segment and cannot guarantee the target after
   hitting the cap. For example, an estimated 0.2 m ribbon segment needs twenty
   pieces for a 0.01 m target but receives eight, leaving 0.025 m pieces.
   Suggested header: "Segments are subdivided using an estimate from the
   longest strand and a target of LONGEST_SEGMENT, up to the card or ribbon
   subdivision cap." Suggested constant doc: "Target straight-segment length
   for the bounded subdivision policy." Keep clearance as a separate sampled
   geometric operation; do not imply a continuous skin-clearance proof.

2. P3 — align the missing-form contract at
   `src/hair/geometry.rs:50-51` and `src/hair_preview/assets.rs:30-32`.
   Both promise None for a form the hair lacks. The card branch instead calls
   get(level.index()).or_else(|| self.lods.cards().last()) at geometry.rs:59-60.
   A requested absent card ordinal therefore yields the last cut whenever one
   exists; absent strand ordinals yield None. Decide the canonical receiving
   behavior before changing prose: if fallback is intentionally retained, name
   that exact behavior in both comments; if missing ordinals should yield None
   under the migration rules, remove the fallback and retain the comments.
   This is an observable API-contract discrepancy, not a request to silently
   choose a new implementation policy.

3. P3 — identify the pre-taper width floor at
   `src/hair/gpu/strips.rs:8-10` and MIN_RIBBON_WIDTH doc at line 64.
   RIBBON_ROWS computes width=max(2*radius*widen,min_width), then applies
   half=0.5*width*taper_at(along). Taper::default().tip is 0.2 in
   fabelgeist-hair/src/material.rs:84. A minimum-width 0.4 mm ribbon therefore
   reaches 0.08 mm at its tip. The current hard minimum drawn-width wording is
   inaccurate. Suggested constant doc: "Minimum ribbon width before taper."
   Suggested header: "Ribbons use a minimum base width for the preview, then
   taper towards their tips."

## Source coverage and matched contracts

- Read complete strip orchestration, extracted geometry and WGSL modules.
  Canonical MeshStripRows admits at least two rows; subdivision has a nonzero
  factor; MeshStripLayout checks complete u32 vertex/corner populations.
  Native faces are strip-major and preserve the prior two triangles per quad.
  MeshStripCount is distinct from InvocationCount and converts at the native
  dispatch boundary. Geometry attribute arrays use one vec4 per vertex and
  lift scratch uses one vec4 per two-vertex row.
- Native storage retains at least one vec4 for zero geometry. GpuGeometry::read
  takes the layout-derived logical prefix for each attribute, preserving empty
  host arrays and excluding padding. Buffer::prefix checks both logical and
  physical extents; Buffer::read explicitly handles logical zero. Source device
  association is checked before construction and before readback. Strip build
  handles batch.submit() errors before returning geometry; no new comment
  claims provider-independent native validation or GPU completion.
- Read complete Hair geometry/Groom producer modules. Longest arc length flows
  from bake LONGEST native reduction to Metres, into GroomedHair and corrected
  PlacedHair, then StripSource; conversion to f32 is confined to numerical
  subdivision and native parameters. Taper samples preserve the previous
  algorithm. Ribbon budgets count two native row vertices and empty native
  populations remain empty rather than fabricating a strand.
- Read current Mesh strip/count/GPU conversion/export owners, CardStripLayout
  and card build. CardStripLayout consumes canonical MeshStripLayout; it does
  not retain an independent row/count implementation. Existing CardLayout
  docs correctly distinguish indexed topology from index-membership validation.
- Read preview state/form choices, its changed controls route, assets, strand
  shading/bindings, all three hair vertex-source files and studio device bridge.
  Placeholder mesh positions count logical vertices; custom shader attributes
  come from the native geometry buffers. Per-sample stochastic coverage remains
  in the existing shading implementation; mesh skinning remains wholly on the
  head joint. Typed slider counts/ratios cross UI representation boundaries
  explicitly and row-admission failures propagate to studio status.
- Read character export hair in full and the changed Studio initial-form and
  existing render-test format adaptations. Export uses card level zero, creates
  model-space HairGeometry by checked readback, then retains atlas/morph/skin
  data. No tests, cases, helpers or assertions were introduced by the reviewed
  adaptations. Creator Cargo now accurately describes GPU preparation on the
  preview or export device.
- Read canonical atlas extent/variant owners, GPU strand layout/LOD records,
  policy and Buffer prefix/read dependencies used by these claims. Broader
  library viewer/dataset claims, unchanged performance/driver provenance,
  physical/lower families and global review remain outside this verdict.

## Extracted WGSL comparison

- `BINDINGS` raw WGSL content is byte-exact baseline HEAD: `True`.
- `CARD_ROWS` raw WGSL content is byte-exact baseline HEAD: `True`.
- `RIBBON_ROWS` raw WGSL content is byte-exact baseline HEAD: `True`.
- `WRAP` raw WGSL content is byte-exact baseline HEAD: `True`.

All four native program strings were compared directly with their previous
definitions in the original strips.rs, rather than inferred from extraction
or historical verdicts.

## Snapshot fingerprints

- `crates/adventuresim-character-creator/Cargo.toml`: `1346098aa561b83aa6d045c8744f19edc6605674fd0bbf313af4dcfc4ef595e5`
- `crates/adventuresim-character-creator/src/hair/gpu/strips.rs`: `bd579f300ecfbd985eea5d4c4f5a177883380b6282f63075b005b791092c52ee`
- `crates/adventuresim-character-creator/src/hair/gpu/strips/geometry.rs`: `57f15b94c1f1c08bfcc0e214f95fd19640c418ad76f96d698661717076bcd6e4`
- `crates/adventuresim-character-creator/src/hair/gpu/strips/wgsl.rs`: `525528762142f6b0e781d6f29ee1b58f66055b6e41873a86c6da96ba14b15e10`
- `crates/adventuresim-character-creator/src/hair/geometry.rs`: `eb74c6b921850545dc9e83a00ac3091d2ffa4cfe661bd5f685c172b7da607056`
- `crates/adventuresim-character-creator/src/hair/groom.rs`: `d6c46f6ef0b1f037cc45648c2baac76b9e383aad331cf1c3757f1e1150702b43`
- `crates/adventuresim-character-creator/src/hair/gpu/bake/mod.rs`: `70a93715a6a926de40b3f71c676839e81df669ff56f41dcbbfa1486560603466`
- `crates/adventuresim-character-creator/src/hair/gpu/bake/wgsl.rs`: `7965432924278f1a7cbcf9a47a295a9c2b9fe4103783e8d641f54dbbd9bc70ec`
- `crates/adventuresim-character-creator/src/hair/gpu/mod.rs`: `f8844fc7362364c5217079615e4459865865020ba4a36f5b25a8997cdd9c8f92`
- `crates/adventuresim-character-creator/src/hair_preview.rs`: `7dbe38c17f4941b7200bd8d506fa5055f75abd4dee1e434716d0967ae8f035c1`
- `crates/adventuresim-character-creator/src/hair_preview/assets.rs`: `da0250550989fb53fd8708be317c7e5aed3c002c9e02e4ab1e7103df0edb5613`
- `crates/adventuresim-character-creator/src/hair_preview/strand_shading.rs`: `13592232826ab0676fdf0cf93edd150713c436e3c73f743bd5d828072e1224b9`
- `crates/adventuresim-character-creator/src/hair_preview/strand_shading.wgsl`: `f534e560355fbbb1b4c4faf19813dcd8234a56bcd97e48770fb5ffebdfca2689`
- `crates/adventuresim-character-creator/src/hair_preview/hair_vertices.wgsl`: `62cbd9e3f606c85f4212494802e2675daa55387a27ac0133c8d79f80c0282b92`
- `crates/adventuresim-character-creator/src/hair_preview/hair_vertex.wgsl`: `55f804b00bd26d681d5796d5d64af70f7c6152fe0b3b05bb1d0f7f55491df019`
- `crates/adventuresim-character-creator/src/hair_preview/hair_prepass.wgsl`: `652c5f6053ebc46183d0be27ae1e3f4630700241081ec2cd62a99b974d0b0747`
- `crates/adventuresim-character-creator/src/hair_preview/gpu.rs`: `462eb12c93f953673bc177a3cef00835889022f59d278bf1884086fc328c2adc`
- `crates/adventuresim-character-creator/src/hair_controls.rs`: `b412435396639226eabf4bb624b52eaa6c4ec25f1697b3b5987d569d7bf7440a`
- `crates/adventuresim-character-creator/src/character_export/hair.rs`: `ac11b7bb583e07eaa622342d8830d2bf203f8d3666706eb055af056f0abc3bd4`
- `crates/adventuresim-character-creator/src/main.rs`: `566a69fc6cd50bd7d7ea5c0347567c39f2bdc37e9f8004675dc5328e3751e12e`
- `crates/adventuresim-character-creator/src/hair_preview/render_test.rs`: `f141e4878ee8868ded50d8a481f8f1effb5c1ee4bb382fbcec321918c254c57e`
- `crates/fabelgeist-mesh/Cargo.toml`: `b6a30a9372b2584916183463c9c88b45f69dc4f295c9de1c2428875036f73f94`
- `crates/fabelgeist-mesh/src/lib.rs`: `a1d2e436ae07dc79d25a7f0b670df3a3d7d3b3554d495db6542bd34f0c63f653`
- `crates/fabelgeist-mesh/src/count.rs`: `e609778b6bd24b4ee231f3d3281d463764a55d5b787f02f4015f4ff7c4b4072b`
- `crates/fabelgeist-mesh/src/count/gpu.rs`: `bcead2da272edc0fbcd1d285004d9671ad310603a5870bf1c7df114be337c4c4`
- `crates/fabelgeist-mesh/src/count/strip.rs`: `dd53d7fa66134977308f2dd0279c5d38bef8e907c739f6098904a40541a37208`
- `crates/fabelgeist-hair/src/cards/layout.rs`: `7affba536778dd1ae65da9b834c2400b2eddd4ce6bc6a7dc7aba5048a5fbf878`
- `crates/fabelgeist-hair/src/cards/build.rs`: `d0e1af5a572ecbdfdebcde495932dae897361eb446f4348824d6bcddd0cd2516`
- `crates/fabelgeist-hair/src/cards/atlas.rs`: `7cb59f0ac4dcfa2ea8643dc6384fdc15652d99d576f26a12fad2b43145cfc74b`
- `crates/fabelgeist-hair/src/cards/atlas/layout.rs`: `df403b648c0f3ba7f93f2f45c4deea2f33fc0c72fedef23d9f454397dbf428dd`
- `crates/fabelgeist-hair/src/gpu/layout.rs`: `ecdfc0cc3c30a0ec0b276f91a9799b7362430cd4ac9a281d208f68fcadc071a0`
- `crates/fabelgeist-hair/src/gpu/lods/records.rs`: `7814ed1212fc58bea86aebf290862d5937f4e3e20d2b508050d20983948323a8`
- `crates/fabelgeist-hair/src/lod/policy.rs`: `16109d7ba1a21b99e1aaebad3835ed0dcd18d540dcd3e2961d9d05f706c857dd`
- `crates/fabelgeist-hair/src/material.rs`: `20fdf72de21b47f4596eb6538d58a2d89aeb5217a8b797ca53730c769b5d5b1e`
- `crates/fabelgeist-gpu/src/data/gpu/buffer/prefix.rs`: `62a435f0e609deb067d2c9e59481ee7e042d0ee1f16c5d662f0ea6b4bd95b4c0`
- `crates/fabelgeist-gpu/src/data/gpu/buffer/read.rs`: `ff97b2b7c684745153c4713acc58e4d535cd97dd9008bcb8aacb9e62fdd138f2`

## Limits

The creator_strip_owner_current_discovery ledger contains thirteen pre-edit
lens inventories; it was read as scope and ownership context. Clearance here
does not come from that ledger or earlier verdicts. Governing root, Rust,
Creator and shared armor rules were read previously and remain applicable.
No geometry was edited or rendered, no tests or project checks were run, and
no external posts or CI requests were made. The independent Code/Rule reviewers
also received the temporary longest-field mismatch; the final field was reread
as Metres before this report. Re-review the three findings after disposition
and reconfirm final source fingerprints. Final combined global review pending.

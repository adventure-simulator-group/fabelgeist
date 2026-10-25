# Independent Comment Review: final current scalp-chain family

HEAD: `99d70e7c3f0bcf8db563c602f39869b91a9f23bf`, plus the current
nine assigned receiving sources. Every file below was reread completely for
this review; the prior verdict did not substitute for current-source coverage.

Verdict: CLEAR for the complete nine-source scalp-chain receiving family.
The mapped physical-length field finding is resolved. No actionable scoped
comment finding remains. Broader global clearance is pending.

## Final current SHA-256 fingerprints

- `crates/adventuresim-character-creator/src/hair.rs`: `7c06f11d607b958e8a9e000bf5b6debeda01e3c253fc863c331317382b196a9a`
- `crates/adventuresim-character-creator/src/hair/gpu/mod.rs`: `02378baa2d1ed60f1c8337c24b9e0e5872b240203fc5f7944f3f8eb93744d2f6`
- `crates/adventuresim-character-creator/src/hair/gpu/bake/mod.rs`: `a9be367893c88b158dac2f0368c4ef28d521e59290a428b08753c342881225bd`
- `crates/adventuresim-character-creator/src/hair/gpu/bake/layout.rs`: `88c1868538af8a6862650cfe70083e9bcaff1785e72160d656629aabb23df23b`
- `crates/adventuresim-character-creator/src/hair/gpu/bake/grid.rs`: `8ad5ee14907efa7548e6ddb0ba1c47be55253701644626a9879045df06a9b438`
- `crates/adventuresim-character-creator/src/hair/gpu/bake/readback.rs`: `958943b91a03470381744264d571a83be30421501989fa92a56a403cbbec4bee`
- `crates/adventuresim-character-creator/src/hair/scalp_map.rs`: `a410d423ff4188de5f759094665c53095193198745913dcb411d9a822f7c3f53`
- `crates/adventuresim-character-creator/src/hair_preview/scalp_upload.rs`: `8f70a54fc5032060ecf8e46d898e5a9e02a93416e224393437a3b1852946dddd`
- `crates/adventuresim-character-creator/src/hair_preview/render_test.rs`: `d21005ddea5a4a2ec07fd70fc03357ef5f291191d00d5b430319b685f676d531`

## Current source evidence

- ScalpMap now stores mapped_side: Metres. Its documentation identifies the
  square root of estimated mapped scalp area; fit actually stores sqrt(area),
  and texel(size) divides by the positive texture-side count. Whole mapped
  length and per-texel physical-width estimate are no longer conflated.
- Grid distinguishes physical texel spans from ShadowBlockAxisCount. The
  private count factory receives positive side and block spans, uses ceiling
  division and therefore produces at least one block and no more than the
  texture side. Its sole current caller supplies the fixed admitted base side,
  2048. Consequently the element projection squares a count no larger than
  2048, so its existing native u32 arithmetic remains bounded. The comments
  about count positivity, base bound and scalar projection match this actual
  closed construction path; they do not promise arbitrary future row sizes.
- ScalpTextureLayout owns the fixed positive 2048 recipe, descending twelve
  mip levels ending at one, four-byte element stride, padded row extents and
  byte offsets. Private level admission and checked full extent support the
  documented host/native addressing bound. Byte pitch and word-offset shader
  projection retain distinct units and the same native values.
- GpuScalpTexture privately creates its two ScalpTextureChains with the actual
  allocated buffers, common Arc layout and creating native Device. It returns
  only after handling batch.submit()?; its documentation makes no global
  completion or source-geometry certification claim. External buffer aliases
  remain governed by the buffer access contract named by buffer().
- Chain readback rejects a foreign device before Buffer::prefix or native
  Buffer::read. It consumes its retained full padded layout and strips row
  padding from host images. This matches the creating-device/readback docs
  without promising later native writes or global device health.
- ScalpUploads retains the typed chain, creates its image from that chain
  layout and uses its actual per-level byte offsets and padded row pitches.
  The explicit Bevy error-policy comment remains accurate: this render system
  does not observe a synchronous native validation verdict. The new chain does
  not introduce a documented same-device rejection in the Bevy upload path.
- Grid DRAW/BARE/BLUR parameters preserve the numerical physical/share values,
  strand/texel invocation populations and four alternating row/column blur
  passes. Mip completion follows the same unchanged HALVE program. Metres and
  Ratio remain present until their explicit numerical/shader boundaries.
- Public exports in hair.rs and hair/gpu/mod.rs consume the canonical chain,
  mip and layout owners. The old global levels API and ScalpLevel alias are
  removed; moved documentation remains attached to NO_DYE, HairDesign and
  ScalpUploadPlugin. Current fixture Stage retains typed chains and the upload
  calls consume those actual objects.

## Source preservation and limits

- Entire bake/wgsl.rs byte-exact baseline comparison: `True`.
- Fixture reverse import/Stage API adaptations byte-exact baseline: `True`.

These are source comparisons, not project checks or test execution. The prior
native texture-copy row-alignment and buffer/read contracts remain directly
supported by their source paths and are unchanged by this correction. The
source-backed thirteen-lens preinventory is scope context, not clearance.

No source edits, project checks, tests, new cases/helpers/assertions, doctests,
external posts or hosted CI requests were performed. Only this local report
was updated. This scoped source review does not establish runtime bake quality,
native validation, GPU completion or final-pushed-revision acceptance. Broader
native scalp-grid, physical/shaped/native83 families and global final reviews
remain open.

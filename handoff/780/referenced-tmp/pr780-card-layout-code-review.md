# Independent Code Review: card topology and native producer delta

Base and HEAD: `0df94b91011a415be93c14174d4233b09eb01c00`.
Reviewed the uncommitted working delta; captured fingerprints appear below.
Read-only source review: no source edits, new tests/cases/helpers/assertions,
builds, tests, formatting, quality tools, hosted CI requests, pushes or posts.

## Findings

No actionable correctness defect demonstrated in the requested producer delta.
This is scoped static review, not compilation/runtime evidence, global
thirteen-lens clearance, or the eventual combined same-revision Code Review.

## Contracts and source evidence

- `cards/mesh.rs::from_triangles` retains the supplied triangle list exactly.
  `CardLayout::for_triangles` rejects partial triangles, admits vertex/triangle
  cardinalities through canonical mesh owners, checks the expanded corner
  count and every index's membership. Generic triangles have their own
  topology variant and `strips()` returns `None`; no fabricated card count
  or uniform-strip assumption is imposed. Default produces an empty generic
  layout with empty collections. Sealed fields and borrowed slices preserve
  the admitted contents/layout relationship externally.
- `cards/layout.rs::CardStripLayout::new` rejects zero segments, checks
  rows/per-card vertices/total vertices/triangles before constructing the
  shape, and uses `CardLayout::new` to check three corners per triangle.
  `cards()` divides the admitted vertex population by the exact admitted
  per-card row width. Its denominator cannot overflow or be zero because
  construction checked `(segments + 1) * 2`; the product is exactly divisible
  by that denominator, including zero cards. Vertices, triangles, corners,
  physical strips and segments retain distinct quantities.
- `cards/build.rs::build` preserves the old minimum-one host segment policy,
  checks one strip before row generation and the complete strip population
  before native index construction/concatenation. The native loop casts are
  bounded by the checked per-strip/full shape. Compared the full old body
  with `CardBake::lock` and `emit`: member/guide order, resampling, normals,
  width smoothing, random streams, atlas variants, winding and vertex packing
  retain the same operations. Removing the unused per-card StrandId tuple
  does not alter emitted geometry. Final vector reserves now return typed
  allocation errors. This does not claim every intermediate collection can
  recover from host allocation exhaustion or malformed source strands.
- `gpu/lods/cards.rs::CardWork::new` preserves the native clamp to 1..31
  segments and proves `source_count * lock_stride` fits native u32 addresses,
  plus the trailing scan element. The stride is `4 + 4 * (segments + 1)` and
  is at least twelve, so `source_count <= u32::MAX / 12 < i32::MAX`. Every
  signed lock count is bounded by that source population. Saturating a larger
  min-strands threshold to `i32::MAX` therefore excludes all locks, rather
  than wrapping it into a small/negative threshold. The shader's existing
  max-one rule for zero thresholds remains intact.
- `compact_cards` preserves map/accumulate/flags/exclusive-scan/readback order.
  The trailing scan count admits actual `CardCount`; `work.strips` rejects
  counts above the source population and validates both strip topology and
  the writer's additional vec4 expansion before output allocation/emission.
  Read `gpu/readback.rs` to confirm that the requested trailing word is read
  after accepted submission and short readback is handled.
- `cards/layout/gpu.rs::admit_native_writer` checks four vec4 records per
  CardVertex. In the unchanged `gpu/wgsl/lod.rs::card_build`, vertex accesses
  are `(first + 2 * row + side) * 4` through offset three; corner accesses
  are `(slot * segments + segment) * 6` through offset five. Admitted vertex
  and corner populations bound all those addresses. Read the complete LOCKS,
  accumulation, flag, spine, spread and build bodies; no shader text changed.
  The layout proof does not certify separate fixed-point physical sums.
- `gpu/lods/prepare.rs::CardBuffers::new` derives logical population from
  the admitted strip layout and provides physical minima of one CardVertex
  and one u32 index for empty output. A zero-card result causes every source
  lock to fail the same emission predicate that generated compaction flags,
  so the physical minima are not treated as fake logical geometry.
- The sole changed native card producer runs beneath
  `GpuGroomer::lods_against`, which checks the groom's actual device first.
  Card geometry, visibility, metadata and commands use that groomer context;
  `GpuCards` records the actual allocating device and admitted strips alongside
  its private buffers. The card output is standalone geometry; it does not
  require a retained strand-source identity to interpret its stored vertices.
- `lod.rs::HairLods::build` propagates the new fallible card result through
  `LodError::Cards`; atlas failures retain their typed cause separately.
  Public exports/doctest error types follow that result. Existing host/native
  card assertions preserve their old populations/oracles through getter/type
  adaptations; no additional tests or assertions were introduced in this
  reviewed delta.

## Coverage and limits

Read only the requested ledger `card_layout_producer_repair` key, including its
representation refinement and acceptance. Reviewed all changed/new producer
files, old host/native bodies, canonical mesh counts and GPU projections,
existing card fixture adaptations, native shader receiving bodies, existing
storage/readback and Cargo feature wiring.

Known unintegrated renderer/API receivers owned by #826, Creator readonly/
policy receiving work, broader host source admission, physical/grid units and
fixed-point sums remain outside this delta verdict and explicitly incomplete.
The current working tree therefore receives no compile/global clearance from
this report. Renderer mixed-fixture integration was not claimed complete;
the new generic-triangle constructor itself retains that capability.
Final combined independent review and required hosted checks remain necessary.

## Captured working-delta fingerprints

```json
[
  {
    "path": "crates/fabelgeist-hair/src/cards.rs",
    "sha256": "75508eb07b8f3292152f27310bdc7193128efbf211bb4ce49bc55c8a9c3f6987"
  },
  {
    "path": "crates/fabelgeist-hair/src/cards/build.rs",
    "sha256": "ae375dba80589dd2d7773952f3aadee391a4ac82fdefcd76b3387c400975205b"
  },
  {
    "path": "crates/fabelgeist-hair/src/cards/layout.rs",
    "sha256": "a15b5f567400e7c5428970ec6bef603d18fc37834b4719fff60aea756a2bb9e1"
  },
  {
    "path": "crates/fabelgeist-hair/src/cards/layout/gpu.rs",
    "sha256": "6b7a180a3ee2faa7fc26a79e2ae14ceeac4a54c19f96a308dc57f417c56ea49f"
  },
  {
    "path": "crates/fabelgeist-hair/src/cards/mesh.rs",
    "sha256": "322747ae7f6a10847b6f82813fd1b27cfd8caafb01f4d13aca401dc78346ab21"
  },
  {
    "path": "crates/fabelgeist-hair/src/gpu/error.rs",
    "sha256": "8c9e0cfc1381290739c5cb166996fa345de9def1e25532ad367f1963cfd5c8af"
  },
  {
    "path": "crates/fabelgeist-hair/src/gpu/lods.rs",
    "sha256": "44ffb5c296d0d5948e0b64f76f2dea59c4d41238f3ef8deaf6d549007a376f61"
  },
  {
    "path": "crates/fabelgeist-hair/src/gpu/lods/cards.rs",
    "sha256": "77e815b2a949e96a4e90b90acc7f184021d395dcbb1881ee293fa0d94f2c69d6"
  },
  {
    "path": "crates/fabelgeist-hair/src/gpu/lods/prepare.rs",
    "sha256": "c94f30467215f10aee321cdfbac303d5dccd4f6f1c8c8d5a5630feb0ed8ed107"
  },
  {
    "path": "crates/fabelgeist-hair/src/gpu/lods/records.rs",
    "sha256": "7814ed1212fc58bea86aebf290862d5937f4e3e20d2b508050d20983948323a8"
  },
  {
    "path": "crates/fabelgeist-hair/src/gpu/mod.rs",
    "sha256": "e943f9ba2a9213beb84d843d3c73b2f13f1658add9cb47cbd35daa12f6161647"
  },
  {
    "path": "crates/fabelgeist-hair/src/lib.rs",
    "sha256": "c2df7e182028b42251d7f1c453046962f13ae98e9b4cb0d85795ac7550a15031"
  },
  {
    "path": "crates/fabelgeist-hair/src/lod.rs",
    "sha256": "41f0e66a7ec7906210fd0050ade6a7bc8d96d6742bf8fd6a7150393aff3d5f8a"
  }
]
```

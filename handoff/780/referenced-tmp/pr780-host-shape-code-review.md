# Independent Code Review: canonical host strand shape and source records

Base and HEAD: `03ad4103f2e9a892e1682be580cb9d3c9a52669f`.
Reviewed the current uncommitted eight-file delta; captured fingerprints below.
Read only the requested ledger `host_strand_shape_preinventory` entry.

## Findings

No actionable correctness defect demonstrated in this scoped delta.

This is source-only review, not build/runtime evidence, global thirteen-lens
clearance, or final independent review of one combined pushed revision. No
source edits, project checks, tests, test cases/helpers/assertions, hosted CI,
pushes or external posts were performed.

## Source-backed contracts checked

- `strands/shape.rs::StrandShape::new` validates root/tip cardinality for a
  populated source and checks the host-width strands-times-points product.
  Empty sources retain valid zero- or one-point strides. Private fields and
  the absence of unchecked/default/serialization constructors preserve that
  product. `samples` derives rather than independently retains its count;
  `prefix` rejects growth and `limited_to` only shrinks. Shrinking therefore
  preserves both host product and populated root/tip admission.
- `StrandShape::try_from(&Strands)` takes population from `Strands::len`, whose
  canonical collection is `root_uv`. It checks the complete position length
  and root-normal, ID, parting-group and radius columns against that population.
  `guide_map` is not an omitted parallel column consumed by these producers:
  the card path constructs a fresh GuideMap and LOD generation does not reuse
  it. The shape does not claim to validate geometric values.
- `lod.rs::StrandLevel::new` admits the complete positions/infos relationship
  before storing the checked shape. Its fields are private and returned arrays
  are borrowed immutable slices. `HairLods` privately retains Arc levels and
  exposes a borrowed slice, so external receivers cannot replace/reorder levels
  or independently mutate their cardinality. Public standalone StrandLevel
  construction does not allow insertion into a fabricated HairLods collection.
  The sole HairLods builder always starts with the source stride, proving its
  first-level access valid even for an empty strand population.
- `HairLods::build` now validates source storage before atlas/occlusion work,
  attribute indexing or source-array slicing. Each generated level checks its
  host sample shape before resampling, and the completed arrays are admitted
  into the sealed Arc record. Compared with base, source shuffle, ID/random
  keys, attribute order, resampling, body clearance, atlas stream and card/
  cover inputs retain their existing numerical expressions and order.
- `StrandLevel::bytes` derives actual slice byte sizes. Each allocated Vec
  slice is bounded by isize::MAX; adding the two byte extents therefore fits
  usize. No new independent raw byte-capacity authority is introduced.
- `gpu/layout.rs::StrandLayout` retains the canonical host shape and adds only
  native strand/stride/sample/ribbon-width admission through
  `TryFrom<StrandShape>`. `TryFrom<&Strands>` shares full host column admission;
  `TryFrom<&StrandLevel>` consumes the sealed level's existing guarantee.
  The duplicate host root/tip and column validator has been removed.
- `StrandLayout::for_host_level` obtains the already admitted full host shape,
  computes its resident prefix by division before multiplication, then applies
  native admission to the reduced shape. It does not reject a large complete
  host population merely because the original population exceeds native widths.
  Native stride and sample/ribbon checks remain in force for that resident
  shape. Zero populations avoid division, and nonzero ones have a proven
  positive stride. Native `prefix` and `fit_level_binding` only shrink an
  already admitted native population, preserving all native arithmetic bounds.
- `cards/build.rs::CardMesh::build` performs full host shape/column admission
  and checks source population fits u32 before `GuideMap::new` or members'
  narrowing/indexing. In `ops/mod.rs::GuideMap::new`, u32 represents the
  exclusive population and every actual index is below it; admitting exactly
  u32::MAX is therefore valid, with maximum live index u32::MAX-1. The card
  builder's existing member/geometry logic remains unchanged after admission.
  Errors retain canonical shape or numeric-conversion causes.
- The current host simulation caller adapts its level selection and position
  reads to immutable accessors; source selection and fixture population oracles
  retain their prior meaning. This adaptation does not complete or certify the
  separate pending host binding/frame family.

## Coverage and limits

Reviewed complete new shape source; all seven changed source files/diffs;
canonical count leaves; complete relevant Strands storage/accessors;
GuideMap construction; LOD source/level/card production and current simulation
reader; native layout conversions/projections and existing fixture adaptations.
Read the relevant unchanged occlusion/cover source boundaries to verify which
validated columns they consume.

Renderer/#826 and Creator receiving composition, sealed SimBinding/SimFrame,
settings semantic migration, PointGrid/general physical/grid admission and
nonfinite geometry remain known pending work, outside this verdict. The
repository's raw mutable Strands construction/operation APIs are not globally
sealed or globally cleared by the new checked producer boundaries. Final full
waves, combined independent reviews and hosted checks remain necessary.

## Captured working-delta fingerprints

```json
[
  {
    "path": "crates/fabelgeist-hair/src/cards/build.rs",
    "sha256": "43a2bfd975e30c60f0738e6a4c746b99ce16f42b74ccf794bf150f75f113e506"
  },
  {
    "path": "crates/fabelgeist-hair/src/cards/layout.rs",
    "sha256": "ba42c5f262f91abcd074353ea1c9bc7b17f513bde0310c88e13b8743bc007633"
  },
  {
    "path": "crates/fabelgeist-hair/src/gpu/layout.rs",
    "sha256": "ecdfc0cc3c30a0ec0b276f91a9799b7362430cd4ac9a281d208f68fcadc071a0"
  },
  {
    "path": "crates/fabelgeist-hair/src/gpu/mod.rs",
    "sha256": "b900f53dd9b2751914c9b756562e8d9a54465c958b53709b2052d3f70cb7e788"
  },
  {
    "path": "crates/fabelgeist-hair/src/lod.rs",
    "sha256": "66205be78231f6f2176c92e1edf72e615e3528d57d4f598157b4b855af1ba196"
  },
  {
    "path": "crates/fabelgeist-hair/src/sim.rs",
    "sha256": "3e2784bef61f86973ae6de8cc995023a4d95787bb958638f799e4c3abc6646e5"
  },
  {
    "path": "crates/fabelgeist-hair/src/strands.rs",
    "sha256": "3ab9c91b1153b1f7a21e2dd57e9f53c5a6f4b220abde40f9ef666c4b60c4d336"
  },
  {
    "path": "crates/fabelgeist-hair/src/strands/shape.rs",
    "sha256": "1fb88c54d6c627d55e959484008c0ae49665912f7226fc6aca78e901bafd39b5"
  }
]
```

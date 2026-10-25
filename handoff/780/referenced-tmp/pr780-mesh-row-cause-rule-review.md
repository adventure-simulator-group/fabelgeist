# Narrow Mesh row-cause Rule Review

Reviewed clean HEAD b11d6697c5b2fc05ca59cf7497f3d6ab626580b5 plus the one
modified file crates/fabelgeist-mesh/src/count/strip.rs. Read applicable root and
crates rules and all thirteen entries of mesh_strip_row_cause_preinventory.
Read the whole changed file and actual Card/Creator receiving functions, rather
than relying on earlier verdicts. No actionable finding remains in this delta.
This narrow verdict does not clear the larger migration or private ordered
shader-signature discovery.

MeshStripRows remains private, non-default, non-deserializable, and admitted only
through usize->u32 plus minimum two rows, segment checked-add-one, or checked
subdivision with a NonZeroU32 factor. The same values succeed and fail as before.
The old combined Rows error dropped the real TryFromIntError and conflated
minimum-row, native-address and additive-overflow failures. MinimumRows now owns
requested count; RowAddresses owns requested count and the actual conversion
error annotated #[source]; SegmentRows owns the actual overflowing segment
count without inventing a standard conversion cause. thiserror derives the
source chain using an already-present dependency. No compatibility Rows variant,
optional cause, diagnostic string classification or primitive adapter remains.

CardStripLayout::new passes canonical MeshStripCount and MeshStripRows through
MeshStripLayout. CardError::Strip(#[from] MeshStripError) preserves the concrete
mesh cause; thiserror's from field also supplies the source relationship.
Creator StripSource::ribbon_rows/cards/ribbons and build use the same canonical
layouts and propagate ? into anyhow::Result. No caller matches the deleted Rows
variant, extracts display strings for classification or suppresses the cause.
PlacedHair's geometry/gpu_geometry/strands_drawn propagate those Results.

The full tail from subdivided through native_rows, all MeshStripLayout admission,
vertex/triangle/corner projections and face iteration is byte-identical HEAD.
Only the error alternatives, row narrowing/minimum classification and the
checked segment-add error changed. The topology still proves all native u32
vertex/corner arithmetic before projection; zero strip populations remain valid,
rows remain at least two, subdivision remains nonzero. Mesh GPU adapters retain
separate strip/row/topology counts and InvocationCount launch ownership; native
PassParameter projections remain unchanged. Card and Creator source, native
programs, units, dependencies and existing fixtures are unchanged in the delta.
Item order, owning MeshStripResult alias and error payload granularity comply
with the scoped rules.

Evidence is read-only source inspection and direct source comparison, not
execution. No source edits, tests/cases/helpers/assertions, project checks,
builds, lints, scanners, generators, hosted CI or external posts were performed.
Root all13 waves, complete final independent reviews and required hosted checks
on the same final pushed SHA remain open. Larger parameter AST, bake/Grid,
completion and native error-cache families are outside this narrow clearance.

## Source SHA-256

- `crates/fabelgeist-mesh/src/count/strip.rs`: `92f8eede012a1018363a73fcd79a8f703862a3f6c40291a30e2cd02264e95ffe`

- `crates/fabelgeist-mesh/src/count/gpu.rs`: `bcead2da272edc0fbcd1d285004d9671ad310603a5870bf1c7df114be337c4c4`

- `crates/fabelgeist-mesh/src/count.rs`: `e609778b6bd24b4ee231f3d3281d463764a55d5b787f02f4015f4ff7c4b4072b`

- `crates/fabelgeist-mesh/Cargo.toml`: `b6a30a9372b2584916183463c9c88b45f69dc4f295c9de1c2428875036f73f94`

- `crates/fabelgeist-hair/src/cards/layout.rs`: `7affba536778dd1ae65da9b834c2400b2eddd4ce6bc6a7dc7aba5048a5fbf878`

- `crates/fabelgeist-hair/src/cards/layout/gpu.rs`: `6b7a180a3ee2faa7fc26a79e2ae14ceeac4a54c19f96a308dc57f417c56ea49f`

- `crates/adventuresim-character-creator/src/hair/gpu/strips.rs`: `818294f1347dd1b4d5bcd5535dae5c0cfe30b75dd1d4380091bf98a06e906b6b`

- `crates/adventuresim-character-creator/src/hair/gpu/strips/geometry.rs`: `57f15b94c1f1c08bfcc0e214f95fd19640c418ad76f96d698661717076bcd6e4`

- `crates/adventuresim-character-creator/src/hair/geometry.rs`: `ba9b0696088257a6f3d984edf137b59435823ff73ba1f176a3fa82d96a3915fe`

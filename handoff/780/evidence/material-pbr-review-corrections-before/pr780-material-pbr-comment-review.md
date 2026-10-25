# Selected PBR graph: independent Comment Review

Snapshot: HEAD `d000e848f2f5bc28b0728c2cd664c727a41ecf50` plus the initial
74 files in `material_pbr_final_preinventory.current_files`. Every current hash
matches that requested manifest. Exact hashes and per-file read extents are in
the JSON companion. No source edits, tests, checks, builds, probes, or project
execution were performed; no tests, cases, helpers, assertions or doctests were
added.

**Provisional scope result: no new actionable PBR prose mismatch found.** Final
native review-protocol closure remains pending the two acknowledged connected
repairs below. This is not whole-material or whole-PR clearance.

## Current material contracts

The complete new encoded, scalar-material, body-appearance and terminal-projection
owners were freshly read. Srgb/Srgba explicitly add neither range nor finiteness
policy, expose native channel adapters, preserve array serialization and transfer
only RGB. Srgba alpha is copied. SurfaceMaterial's leaves distinguish encoding
and dimensionless factors, while its is_finite operation documents its actual
input-only guarantee. LinearSurfaceMaterial has private fields, no Deserialize,
and a checked TryFrom that rejects any nonfinite decoded color/alpha or native
metallic/roughness. Its docs correctly describe a native projection, not retained
recipe state or universal glTF range admission. standard_material independently
preserves Bevy encoded-color policy. The canonical transfer arithmetic remains
owned by Color, avoiding Bevy's different nonpositive branch at the GLB boundary.

The complete component enum/accessors and actual textile mutation, seam,
preview, Tactical and export receiver paths were read. Required Inherit/Surface/
Textile state owns the appearance payload; textile material is derived from the
retained mutable fabric rather than stored alongside it. Cloth closures directly
carry the same enum, and real relief/needlework mutations use textile_mut. Current
comments describe those actual owners without claiming mesh validation or native
bake completion from a type tag. Metal/Lacing and all four former scalar tuple
producers now return/carry the canonical named material. Generic Ratio and
encoding leaves are not advertised as recipe-policy validators.

## Native output, alpha and precedence

The complete current export texture/Writer/validation/compaction sources and
actual body-material and fallible Writer callsites were read. Maps-only surface
and sole texcoords remain accurate. Color-map factor selection occurs in
SurfaceTextures.scalar_appearance BEFORE checked projection; it sets base color
including alpha to white because the PNG carries both. Normal-only maps retain
authored factors. The new comment accurately describes effective factor choice.
Writer then applies general images, textile finish and plate maps in the existing
order. Plate normal/MR and roughness factor override remain; earlier color/AO,
alpha, sidedness, sampling and textile extension are retained. Projection failure
propagates before TextureImages.finish or GLB writing. This does not certify
unrelated image admission, GLB container bounds or exporter atomicity.

SKIN_APPEARANCE owns the original encoded[0.64,0.39,0.30,1], metallic0 and
roughness0.52 for body preview, BareBody and GLB. GLB now actually decodes it.
Bevy-only reflectance/tint/transmission remain at their existing receivers. The
actual separate hair-painted cap still uses scalp_shade::material/hair_surface;
body skin stays below it. The three original P3 mismatches are corrected:
mail-map RGB versus alpha, body skin versus cap, and the former unlit claim.

The complete material_color_controls source and Metal/Lacing actual calls were
read. The float RGB picker uses canonical encoded-to-linear input and writes
canonical encoded output only on changed(), matching the source-backed egui
contract. Ratio sliders similarly write only on a changed response. This closes
the selected Metal/Lacing float picker mismatch by source inspection; broader
Metal semantic fields remain open. Mail's existing byte picker remains an
explicit native quantization policy; no whole-pipeline precision guarantee is
inferred from the encoding type. TextileColor remains the distinct byte palette.

## Existing fixture provenance

The current complete API deltas were freshly read for every inventoried fixture
family: six export literals, assembly/morphs/texture helper, external textile
export, cloth admission/canonical-factor assertions, ring scalar/invalid recipe,
Tactical partitions, trim components, skin helper, textile seam helper, native
metal NaN admission, and runtime bench appearance assertions. The original full
fixture bodies were read in the immediately preceding pre-edit frontier; current
diffs carry the same authored values, scenario operations and oracles into named
records/getters. There are no new executable tests/helpers/assertions in those
reviewed deltas. Geometry, image roles, alpha pixels and texture precedence still
match the actual current receiving implementations. Root's broader source-proof
artifact is supplemental evidence, not substituted for this receiving review.
The JSON explicitly limits unchanged large-body and native fitting coverage.

## Pending connected repairs

- `examples/textile_review/mod.rs:1,70` still claims the shared native review
  protocol while serializing source components with the new appearance schema.
  Independent Code Review identified this boundary and Root acknowledged a shared
  public ReviewComponent owner. The final owner and BOTH actual callers must be
  read before that protocol gets prose/receiving clearance. Do not treat a local
  banner or another reviewer's future plan as the fix.
- Armor `trim/finish.rs` still constructs raw color/roughness in its Metal default.
  Independent Rule Review identified this compile/API frontier and Root owns the
  correction. Its existing gilt-band prose is consistent with the authored
  values, but Comment Review does not certify an unrepaired type boundary.

## Explicit limits

Remaining physical units, counts, seeds, closed engraving/motif, trim, causes,
UV-R1/UV-R2, DENS-R1, native numerical/GPU composition and global whole-PR reviews
remain open. Existing material.rs:30 peak-to-peak waviness and hair.rs:7-8 scalp
avoidance claims are not silently cleared by a PBR leaf migration. No external
historical/artistic provenance was newly verified. Unrelated dirty generated
work is excluded. Full thirteen-lens waves, all three final reviews and required
hosted checks must still agree on the final pushed revision.

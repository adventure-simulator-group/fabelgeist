# Independent candidate 07 code review

Read-only follow-up on 2026-09-09, covering the changed waist profile, lower
back return, medial crease metadata, and render-vertex aliases. No tests or
geometry edits were performed.

## Findings

- **P2: The enlarged back return rejects permitted editor inputs.**
  `src/breastplate_carrier.rs:69` raises the lower return to 82.5 degrees, but
  `back_theta` still multiplies that angle by both side return and neck width.
  A default recipe with `neck_width = 1100` and `side_return = 1080` passes
  validation. Its outer main-grid row 15 is at reference height approximately
  1.197 m; the interpolated return and neckline scale produce an angle near
  92 degrees. `back_raw` (`src/breastplate_carrier/shape.rs:118`) consequently
  rejects the surface because its cosine is nonpositive. The previous lower
  return produces approximately 87 degrees for this same sampled point.
  Keep the lower return within its geometric angular domain across the exposed
  control ranges, or explicitly validate the dependent parameter restriction.

- **P2: Moving projection height from zero to one removes the waist projection.**
  `src/breastplate_carrier/shape.rs:90-95` makes the bell equal to one at the
  waist only when the requested height is exactly zero. At every positive
  height, it is zero there. Starting from the peascod preset and changing
  `projection_height` from 0 to 1 therefore removes the entire authored 65 mm
  waist projection, although the requested peak moved only 0.1% of plate
  height. `src/breastplate_carrier/grid.rs:228-230` copies this discontinuity
  into the whole skirt. The new peak also lies below the first nonzero main
  grid row. Use a profile whose boundary changes continuously as its peak
  approaches the waist, with sampling appropriate to its smallest supported
  transition; this control should not jump between two silhouettes at zero.

## Closure and morph inspection

No additional defect identified in `SolidMesh::split_medial_crease`. It copies
positions and source indices without changing face winding; the same design
chart selects the same aliases for every body morph. This preserves physical
seam coincidence and skin/UV source correspondence while recomputing distinct
left/right normals for both base and target meshes. Separate aliases at the
waist crease retain the existing skirt normal split. Metadata length/order
matches main, shoulder, and skirt vertex construction.

The shared waist-point weight and copied waist projection keep the front/skirt
boundary consistent. The lower back exponent blends smoothly into the upper
section. These source observations do not certify absence of self-intersections,
full-range body clearance, historical proportions, or animation fit.

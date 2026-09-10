# Independent candidate 10 code review

Read-only source follow-up on 2026-09-09. No implementation edits, tests, or
renders were performed by this reviewer.

Both P2 findings in `code-07.md` are resolved in the inspected source:

- `back_theta` caps the complete rear edge angle at 89 degrees before applying
  the lateral chart coordinate. This prevents the previously reported valid
  neck-width/side-return combination from leaving the rear cosine domain, and
  leaves room for the carrier's 0.0005-radian derivative probe.
- Waist projection now has an independent validated parameter. Projection
  height is restricted to 150–800 permille, avoiding division by zero and the
  former zero-height special case. Adjusting chest projection height leaves
  waist projection unchanged, including the flange's copied seam projection.

No new concrete P1/P2 defects identified in this review.

The ruled shoulder connector interpolates between its attachment coordinates
without the former Hermite tangent overshoot. Its rear terminal angle is at
most approximately 83.84 degrees within the shoulder-width range; its starting
angle uses the capped rear chart. The shared first-row indices, column order,
and subsequent solidification remain intact. This source observation does not
prove the resulting surface has no intersections for every combined setting.

Medial aliases still copy position and source indices deterministically from
the recipe's chart. Left/right normals are recomputed separately in both base
and morph meshes; physical seam correspondence and skin/UV source assignment
remain consistent. The lower back-depth multiplier is carried into the skirt
at the same waist boundary and blends back to the upper profile.

The breastplate audit is appropriate for its stated static-identity scope:
it checks physical edge closure/winding, nonadjacent triangle intersections,
minimum triangle area, and signed body-distance samples at vertices, edge
midpoints, and triangle centroids. GLB preflight also checks finite attributes,
skin weight sums, target ordering, and positional coincidence of welded morph
seams. The full sweep uses 110 reproducible configurations: neutral, 90 signed
identity-channel extrema, three combined extrema, and 16 seeded corners.

Limitations remain material: adjacent triangles sharing welded vertices are
excluded from the intersection list; body triangle contacts are reported but
the body acceptance threshold is sampled penetration over 1 mm; nearest-face
signs and finite samples do not prove continuous containment. No skeletal
poses or animation are included. Source review therefore does not replace the
implementation owner's neutral/morph checks or historical visual review.

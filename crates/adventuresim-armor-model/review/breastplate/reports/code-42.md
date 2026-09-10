# Independent candidate 42 code review

Read-only review on 2026-09-10 of the current lateral lap alignment, its rotated
wearer-frame regression, shell render aliases, and identity-morph transport.
No implementation edits, test runs, or geometry generation were performed.

No concrete P1/P2 defect identified in this bounded review.

- `align_back_lap_width` transforms a purely lateral displacement through the
  wearer's frame. It leaves local depth and height unchanged, uses disjoint
  left/right strips, and fades smoothly to zero at reference height 1.27 m.
  Both meshes still have the same coarse column count at this call; alignment
  precedes front refinement. The subsequent rear clearance fit is necessary
  because width alignment can move an initially wider return inward. That
  refit can change depth, so the helper's invariant should not be described as
  a promise that the complete fitting pipeline never changes depth.
- The new regression calls the actual helper with a rotated frame and a
  nonzero width correction. It checks preserved local depth/height and an
  untouched upper row. This meaningfully catches the previous sagittal
  translation and a mistaken world-X implementation. It is a helper contract
  test, not evidence of all-preset seam clearance or continuous overlap.
- Each cut-wall alias copies the correct inner/outer position and its original
  mid-vertex correspondence. Its two triangles reverse the adjoining surface
  boundary correctly. Welded closure is checked before the medial render split;
  the later split only duplicates coincident vertices and preserves their
  correspondence. Separate render normals therefore do not tear physical
  seams under the implemented identity displacement.
- Front/back correspondence offsets remain in the final mid-vertex domain.
  Refined flute vertices interpolate fixed coarse-carrier body displacement;
  all inner/outer, skirt, rim, and medial aliases share the resulting sample.
  Target normals are recomputed using the same split render topology. No new
  index, alias, or morph-normal mismatch was found.

The actual candidate 42 rounded no-shadow worn rear-quarter image was viewed
alongside candidate 36. The sharp inset dark wedge beneath the visible armhole
in candidate 36 is absent in candidate 42. A broad, gentle shading transition
remains, without the previous pinched diagonal crease. This supports the
depth-seating diagnosis: the previous corner-normal experiments addressed
vertices above the affected patch. The candidate 42 section image also shows
coherent lower side returns. This is a localized regression assessment, not
historical approval of all recipes or views.

The inspected `target/breastplate-review/42/rounded/check.json` records one
neutral configuration, zero self intersections, zero body triangle contacts,
zero samples inside by more than 1 mm, and minimum signed clearance 3.985 mm.
Its armor SHA-256 is
`fd0f92627a94b037822a7366a12eb602fcc913f1c31c7984daee2f8b7a0fc8a0`.
Earlier morph passes cannot be inherited by this changed geometry. Current
full identity sweeps, other presets, and independent artistic acceptance remain
separate checks; skeletal animation and continuous containment are outside
the inspected neutral evidence.

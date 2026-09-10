# Independent candidate 31 code review

Read-only source review on 2026-09-09. No implementation edits, tests, or
geometry generation were performed. The four existing candidate 29 morph
reports were inspected; each records 110 configurations and zero failures.
Those results do not cover the subsequent experimental fairing or new recipe
settings in candidate 31.

No new concrete P1/P2 defect identified in the reviewed changes.

- Dense flute vertices retain the coarse row interpolation index and blend
  used to construct their base positions. Each index is bounded to leave a
  valid next vertex in that same coarse row. `carrier_samples` creates two
  fixed body correspondences and the target path interpolates their body
  displacements with the same blend. The unrefined path supplies identical
  endpoints with zero blend. Combined front/back offsets and all render aliases
  continue to index the final mid-vertex domain, not the coarse domain.
- Inner/outer, waist, rim, and medial aliases inherit the same `MorphSample`,
  retaining their gauge vectors and physical seam coincidence. Normals remain
  recomputed on the transported triangles with their split render indices.
- Upper extrusion affects the front's final two main rows and uses the fixed
  interior reference row. Its index range excludes the skirt; the reference
  vectors are not changed during the interpolation. The resulting vectors
  pass through normalization and then through the same refinement used for
  the fluted positions. This retains a consistent extrusion field, without
  proving a universal minimum normal thickness or absence of intersections.
- `upper_chest_recession` is present in the shared design, defaults, example
  recipes, validation, and editor. The editor reuses the 0–50 mm domain range.
  Its smooth upper bell vanishes at the waist and neckline, independently of
  the waist-projection term. Combined parameter/body clearance still depends
  on the actual fitted-mesh checks.

The experimental `fair_back_armholes` has bounded neighbors and synchronous
passes. It changes upper lateral X/Z positions while retaining vertex Y and
all indices. Its height/lateral weights leave the lower and medial regions
untouched. However, it runs after clearance fitting and may move a fitted
point inward; the previous candidate's clearance pass cannot be inherited.
Whether it improves the local surface flow remains an artistic question,
and whether its resulting geometry fits remains an objective-check question.

The coarse interpolation fix currently covers morph displacement only. Skin
weights still come from independent nearest-body samples at detailed vertices.
Therefore successful static identity blends do not establish narrow-flute
integrity under skeletal animation. Runtime and skeletal-residual acceptance
remain separate, as do full-range parameter coverage and continuous clearance.

Current documentation should identify the finally retained fairing/shape
candidate and the exact assets audited; it must not label candidate 31 accepted
solely from the four passing candidate 29 reports.

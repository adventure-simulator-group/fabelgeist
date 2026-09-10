# Independent code review

Reviewed the tracked diff against `origin/main` and the new breastplate design,
fluting, editor controls, and exported-asset audit sources on 2026-09-09.

No concrete P1/P2 defects identified in this source review.

- Flute columns depend on the recipe, not the wearer, preserving index
  correspondence across body morphs. The shoulder and skirt sample offsets match
  the refined grid layout, and the solidifier uses the actual front column count
  for the duplicated waist crease.
- Flutes use carrier extrusion vectors for both shell surfaces. Closure retains
  opposite inner/outer winding and the physical waist seam aliases. This source
  inspection does not establish absence of geometric self-intersections.
- Shape/flute JSON rejects unknown nested fields. Count, width, depth, spread,
  and fade limits are checked before generation; the interval checks short-circuit
  before unsigned subtraction or multiplication could overflow. Editor interval
  bounds preserve valid start/end/fade spacing during ordinary single-control
  interaction.
- The new exported-asset check audits welded closure, nonadjacent triangle
  intersections, body clearance samples, and 110 static identity configurations.
  Its GLB preflight checks morph seam correspondence and finite attributes.

Acceptance remains conditional on the implementation owner's actual-body
geometry checks and independent visual comparison with surviving breastplates.
No tests or renders were run by this reviewer. Existing synthetic test success
does not establish fit for the full parameter domain, simultaneous arbitrary
morph weights, or skeletal animation. The audit explicitly excludes animation;
its finite body samples and nearest-surface signs do not prove continuous
containment.

# Independent candidate 18 code review

Source follow-up on 2026-09-09, with bounded arithmetic replay against the
existing candidate 15 rounded, tapul, and peascod carrier JSON. No geometry
was regenerated and no implementation was edited by this reviewer.

The inspected replacement resolves the findings in `code-15.md`:

- The active remapping weight is zero at the first/last main rows, preserving
  their shoulder and skirt attachment sampling.
- The outer material region interpolates to fixed endpoint indices rather
  than inverting returned side-edge x coordinates. The previously displaced
  row-16 right endpoint now maps to exactly 1.0.
- The field is centered on the actual coarse center vertex and bounded by
  both current half-widths. Exact `u = 0` retains its baseline sample, keeping
  the chart's medial normal aliases on the authored ridge.

The bounded replay used the default pattern and a second permitted pattern
with spread 850, lower spread 1000, start 50, end 950, and fade 250, both with
16 flutes and width 650. All sampled rows on the three carriers had strictly
increasing sample coordinates; the smallest successive difference was about
0.0002008. The top `u = 0.5` sample returned to 0.75, the bottom right endpoint
to 1.0, and center samples remained 0.5. Rounding-scale endpoint excursions
before the existing sample clamp are not a material range violation. This
replay is a regression witness, not coverage of every pattern or morph.

No new concrete P1/P2 defect identified. The replacement `height_weights`
uses nonnegative weights summing to one. Its two intervals meet continuously
at the chest with zero derivative, and its update denominator stays positive.
It removes the former direct coupling between waist and shoulder controls;
actual fitting convergence and body clearance still require the running checks.

A useful generated-mesh regression is to enable an even flute count on tapul
with lower spread 1000 and broad end fades: preserve the original flange/side
boundaries and the inner medial vertices, retain distinct left/right normals
at coincident medial vertices, and verify those aliases remain coincident in
the exported morphs. This checks the resulting geometry and shading seams,
rather than mirroring the helper's equations.

The verdict supersedes the open mapping findings in `code-15.md`. Full-domain
collision freedom, arbitrary morph combinations, historical visual acceptance,
and animated fit are outside this source review.

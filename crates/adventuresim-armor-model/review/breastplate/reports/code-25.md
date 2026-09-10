# Independent candidate 25 code review

Read-only source review on 2026-09-09, including bounded evaluation of the
current raw chart equations and inspection of existing morph-audit summaries.
No implementation edits or mesh generation were performed.

**P2: The upper armscye patch does not meet the inner chart continuously.**
`src/breastplate_carrier/shape.rs:204` returns the original angle at
`|u| <= 0.5`. Immediately outside that interval, lines 220–228 blend toward
`asin(neck_x / radius(y))`, where `neck_x` is measured at the upper neckline.
Below the top boundary, that target generally differs from the original
half-chart angle at the current height. The vertical blend is already nonzero
in upper interior rows, producing a finite jump as `u` crosses 0.5. Bounded
evaluation gives an 8.37 mm lateral jump in the authored front chart at row 30
with ordinary neck controls and armscye 1100. Valid wide/deep rear controls
(neck width 1300, neck depth 1400, armscye 1300, side return 1080) give 14.09 mm
at row 29. These are raw-chart values before wearer scaling/fitting, not
measurements of a hole in the rendered mesh. Finite triangles bridge the
jump, concentrating distortion in the first cell outside the inner chart.
The extension should match the original inner chart at every height while
retaining the intended upper boundary.

The implementation owner proposed extending the boundary by the difference
between the current inner-chart x and the inner-chart x evaluated at the
current column's top, multiplied by `1 - along`. Algebraically this cancels
the mismatch at `u = 0.5`, vanishes on the top boundary, and preserves the
outer endpoint. It establishes positional continuity; it does not by itself
prove matching derivatives, clearance, or triangle quality. That proposal
was not yet implemented in the source inspected for this report.

No additional concrete indexing/morph-correspondence defect identified in
the seating/refit or ridge-fade changes. Rear refitting after seating retains
the same topology and attempts to restore body enclosure, but may alter the
initial lap relationship; final lap geometry still requires inspection. The
ridge fade across phases 0.65–1 has bounded smoothstep behavior and zero end
derivatives. The rear angular blend remains inside its cosine domain.

Current morph acceptance remains unresolved. Existing reports show:

- Candidate 25 peascod: six failures in 110 configurations, all with zero
  reported self-intersections. Positive aggregate penetration is 10.963 mm;
  corner 08 reaches 4.644 mm.
- Candidate 24 fluted: six failures in 110 configurations, likewise with zero
  reported self-intersections. Positive aggregate penetration is 10.957 mm;
  corner 08 reaches 4.669 mm.

Independent endpoint refits are nonlinear, so fixed connectivity and successful
endpoint fitting do not establish clearance for their linear combinations.
These reports establish actual sampled blend failures. They do not alone prove
which fitter operation causes them. A displacement-transfer replacement remains
a hypothesis requiring the same generated-asset audits; this source review does
not approve that unimplemented alternative or waive the existing failures.

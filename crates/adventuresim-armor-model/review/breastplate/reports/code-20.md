# Independent candidate 20 code review

Bounded read-only source review on 2026-09-09. No implementation edits,
geometry generation, or tests were run by this reviewer. The implementation
owner's `tests-20.log` records the passing armor tests; full candidate 20 morph
sweeps were still running at review time.

No new concrete P1/P2 defects identified in the reviewed changes.

- The restored quadratic height weights are nonnegative, sum to one, and
  retain a positive fit-update denominator. They restore the original smooth
  coupling rather than the local two-interval basis reviewed in `code-18.md`.
  That report's local-basis assessment is superseded; it was a source review,
  and subsequent geometric evidence rejected that candidate.
- The rear shoulder now interpolates actual 3D attachment/end positions. It
  uses the exact shared main-grid endpoint at the join and retains the same
  vertex ordering and winding across morphs. The front still interpolates in
  the carrier chart. Subsequent fitting and solidification can affect either
  surface, so the linear construction is not a general no-intersection proof.
- The 6 mm fitting margin changes the generated reserve, not the audit's
  1 mm penetration tolerance. It is a bounded response to measured morph
  interpolation error; passing current sweeps is still required, and no
  continuous or arbitrary-blend clearance guarantee follows from that margin.
- `wide_flute_fades_preserve_side_edges_and_medial_ridge` compares generated
  smooth/fluted boundary and center positions, checks their morph positions,
  and checks physical closure. This meaningfully exercises the previous
  mapping regressions. It does not directly assert distinct left/right
  render-normal groups; the earlier normal-alias source review remains the
  evidence for that implementation path.

Minor cleanup observed for the final checkpoint: the breastplate review README
still describes candidate 05 and its next planned candidate; replace that with
the final accepted candidate and exact verification scope. The shoulder comment
calls both branches a chart band although the rear branch now interpolates 3D
points. Historical reviewer outputs should remain exact, with their supersession
identified in the current checkpoint rather than rewritten.

The mapping fixes accepted in `code-18.md` remain present. Full-range collision
freedom, final historical visual acceptance, and runtime animation acceptance
are outside this source-review verdict.

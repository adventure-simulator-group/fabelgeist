# Independent candidate 15 code review

Read-only source review and bounded arithmetic inspection of the existing
candidate 15 rounded carrier JSON on 2026-09-09. No geometry was regenerated
and no implementation was edited.

**P2: Lateral inversion changes the carrier's side boundaries.**
`flute_sample_coordinate` in `src/breastplate_carrier/fluting.rs` searches for
the first ascending lateral segment containing a target. The fitted carrier
has returned side edges: its lateral coordinate reaches a maximum before the
last column. Consequently the first inverse of the right endpoint's lateral
coordinate is an interior point, not the endpoint. Using the existing rounded
carrier as the smooth input for the default fluted recipe, main row 16 maps
`u = 1` to `along = 0.93580362`, rather than 1. The endpoint moves 44.83 mm,
mostly forward. This changes plate coverage when merely enabling fluting.
The mapping must preserve both endpoint indices and handle the return region
without treating the whole row as an invertible lateral function.

**P2: Valid fades also remap the shoulder and skirt attachment rows.**
The activation formula can remain nonzero at `t = 0` and `t = 1` for valid
`start = 50`, `end = 950`, `fade = 250`. On the same carrier, with
`spread = 850` and `lower_spread = 1000`, the top `u = 0.5` changes from
`along = 0.75` to approximately 0.71882758. Subsequent shoulder rows still use
the original map, and subsequent skirt rows use the baseline fan. The shared
indices keep these joins topologically closed, but their first strips now
receive an unintended change in shape. Activation should vanish at both
attachment rows and preserve their material sampling.

The inspected two patterns had no descending sample-coordinate pairs on this
neutral carrier. That observation is not a full-domain monotonicity guarantee:
the general algorithm also lacks a constraint keeping the projected field
inside narrower current rows, and its no-match baseline fallback is not a
continuous inverse.

The replacement must also retain the authored center column for a ridged
recipe. In the existing tapul carrier, row 16's center vertex is 1.69 mm to the
right of the midpoint of its endpoint x coordinates. Inverting that endpoint
midpoint for `u = 0` moves the material column that owns the medial normal
aliases off the geometric ridge when fluting is enabled. Anchor the field to
the actual center column and bound the left/right spans from that anchor.

No additional concrete P1/P2 found in the remaining changes. `plate_gap` and
its global translation have been removed from current source, editor, recipes,
and searched documentation. Scaling the fit-travel guard by the existing
bounded torso-depth scale retains a finite bound (39–108 mm); it does not by
itself prove clearance, so actual-body checks remain necessary. The back
radius/trim edits retain the capped rear-angle domain. Their physical plate
contacts and artistic proportions require the separate geometry/visual checks.

The implementation owner proposed a follow-up: zero activation at attachment
rows, fit the flute field inside the current row, and interpolate the outer
material region in index space to fixed endpoints. That proposal had not yet
been inspected when this report was written.

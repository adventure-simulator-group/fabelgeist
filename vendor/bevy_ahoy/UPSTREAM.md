# Retained controller source

This directory contains the crates.io release of
[Jan Hohenheim's bevy_ahoy](https://github.com/janhohenheim/bevy_ahoy), version
0.2.0, with local corrections to stair support and platform point motion.

- Upstream revision recorded by the release's `.cargo_vcs_info.json`:
  `630a4573725242d23260ca6c18602d037cb713c5`.
- Published package SHA-256, recorded in the original Cargo lockfile:
  `a6e8da253b84a4d375c1cf0970cdc8936c3375bd68019a97ea3c1355283a60a4`.
- License: MIT OR Apache-2.0. The original `license-mit.txt` and
  `license-apache.txt` are retained.
- The normalized release manifest, original manifest, source, examples and
  upstream readme are retained. Registry completion metadata and its standalone
  lockfile are omitted. The repository workspace excludes this third-party
  package and selects its source through the shared workspace dependency.

## Stair support query

Direct collision movement stops before a riser at the configured contact skin.
At low speed, one fixed tick advances less than that separation. The original
raised candidate then queries downward support before reaching the next tread
and selects the blocked direct movement again.

The correction queries support one existing contact-skin width forward along
horizontal travel. It changes only the candidate query position; actual
character travel does not include that offset. Existing upward clearance,
forward ledge clearance, walkable slope, step height, contact skin, collider,
gravity and motor settings remain authoritative.

Source-clipped convex foundation cells can report a separating corner normal
instead of their gently inclined bearing face. `src/kcc/stair.rs` confirms that
face with a vertical ray just inside the existing contact envelope, on the same
body and within one existing skin width of the original contact height. The
vertical ray covers both signs of that existing contact-height interval; a
rounded shape contact can lie a few micrometres above the bearing face. The
same walkable-slope limit applies to the physical face. This does not replace
clearance casts, enlarge the step limit, change actual travel or permit a
steep physical bearing. The exact rotated Goslar property 2 stair cells provide
a regression alongside a steep-face negative control.

The owning integration regressions are in the tactical core's
`city_layout/grounding/tests/movement.rs`. They exercise the production physics
plugin from rest on an ordinary step and both directions of a terraced property,
alongside rejection of excessive risers and insufficient headroom. A passing
controller check does not establish complete geographic city acceptance.

## Platform point motion

`src/kcc.rs::calculate_platform_movement` previously transformed the contact
point into platform-local coordinates and back to world coordinates, then
subtracted the original point. Single-precision cancellation could produce
nonzero movement on static generated terrain. The motor could repeatedly brake
that opposing drift to rest instead of starting along a garden lane.

`src/kcc/platform.rs` computes the same rigid-body displacement as linear travel
plus angular travel of the point relative to the platform centre. Stationary
platforms contribute exactly zero movement. Translation and world angular
velocity retain their physical meaning; this changes no authored motor limits.
Unit regressions cover static geographic offsets, translation and rotation.
The garden controller regression exercises the observed Goslar property 964
route against the exact composed terrain and fixed neighboring geometry.

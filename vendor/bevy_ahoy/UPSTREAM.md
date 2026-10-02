# Retained controller source

This directory contains the crates.io release of
[Jan Hohenheim's bevy_ahoy](https://github.com/janhohenheim/bevy_ahoy), version
0.2.0, with one local correction in `src/kcc.rs::step_move`.

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

The owning integration regressions are in the tactical core's
`city_layout/grounding/tests/movement.rs`. They exercise the production physics
plugin from rest on an ordinary step and both directions of a terraced property,
alongside rejection of excessive risers and insufficient headroom. A passing
controller check does not establish complete geographic city acceptance.

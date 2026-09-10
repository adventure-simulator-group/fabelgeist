# Independent candidate 27 code review

Read-only source review on 2026-09-09. No implementation edits, geometry
generation, or tests were performed by this reviewer. Full candidate 27
generated-asset audits were still running at review time.

The transfinite upper-chart extension resolves the positional discontinuity
reported in `code-25.md`. At `|u| = 0.5`, its correction cancels the fixed
neckline x and restores the current inner-chart x. On the top boundary the
correction vanishes; at the outer endpoint its multiplier vanishes. The new
continuity test samples the join over both front/rear charts and multiple
opening settings. It tests this contract without substituting a second copy
of the extension formula. Matching cross-seam derivatives and actual triangle
quality still require the generated-mesh evidence.

No new concrete P1/P2 defect identified in the displacement transport:

- The base shell selects one source face and barycentric weights per mid
  vertex, from the torso-supported enclosure faces. Physical surface, rim,
  waist, and medial aliases inherit the corresponding source sample through
  `source_mid_indices`.
- Each target displacement uses that same face/weights against the paired
  base and morph enclosure positions. Domain validation checks the paired
  enclosure vertex counts and source index bounds. Connectivity does not
  depend on morph coordinates.
- Inner/outer pairs and duplicate seam vertices receive identical deltas.
  Linear blends therefore retain their original gauge vectors and seam
  coincidence, up to floating-point error. Per-target normals are recomputed
  from the actual transported triangles on the existing split render indices.
- The new generated-mesh gauge test exercises this invariant at signed and
  full endpoint weights. Its current fixture is a smooth peascod with one
  synthetic morph; the exported fluted presets and multi-channel blends still
  need the separate audits.

This representation intentionally retains world-space gauge vectors rather
than refitting and reorienting every wall at each endpoint. It does not prove
constant thickness perpendicular to a changed local surface, absence of folds,
or body clearance across arbitrary blends. Independent correspondences on the
two plates can also change lap separation. Nearest-surface correspondence is
fixed from the base body and is not a proof that every extreme body retains
the same appropriate anatomical correspondence. All source morphs are
transported, so skeletal residuals and runtime posing must not be claimed
validated by the static identity-only audit.

Documentation cleanup remains: `ArmorMorph::direct_positions` describes
independently evaluated endpoint positions although this path constructs them
as base plus the transferred delta. Update that wording without weakening
the export consistency checks. Current prose describing per-morph refitting
and the old candidate 20 checkpoint also needs to reflect the final method
and exact audit results. Historical reviewer reports should remain unchanged.

The earlier code-25 chart finding is resolved in source. Its failed morph
evidence remains historical evidence; source review and the reported prototype
26 improvement do not substitute for the pending candidate 27 full audits.

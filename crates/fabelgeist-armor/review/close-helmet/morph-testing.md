# Automated close-helmet morph testing

Run from the repository root with Blender and the installed neutral body asset:

```powershell
blender --background --python-exit-code 1 --python scripts/check_close_helmet_assets.py -- assets/equipment/procedural/close_helmet--worn.glb assets/animations/biped/unarmed/base.glb target/helmet-review/morph-sweep.json
```

The explicit Python exit code makes a failed assertion fail the command,
including in automation. The JSON report records the exact asset/body hashes,
every weight vector, failure location, intersection counts, penetration depths,
and random seed. Use `--only NAME` to reproduce one reported configuration.

## Configurations and checks

The sweep contains 280 configurations: neutral; each of the 45 identity channels
at both game-range limits (+/-0.35); all 57 positive basis endpoints; all 45
negative identity unit weights; all-positive, all-negative, and alternating game
weights; those three mixtures with each skeletal residual; 32 seeded random game
identities; and 16 seeded corner combinations. The seed is 25397.

190 configurations use runtime-range inputs. The other 90 are identity
basis/stress cases at +/-1, outside the game's generated identity range. They
remain in the report as diagnostics; failures there do not make the runtime gate
fail. Skeletal residual cases exercise reference-space mesh integrity only;
this tool does not apply their bone translations or certify their final fit.

For every configuration the test checks intersections between the skull, bevor,
and visor; intersections within each physical shell and between the three nape
lames and bowl; and degenerate triangles. The comb is intentionally embedded in
the bowl, so that one named construction join is counted separately. Physical
shell ordering and the comb's narrow sagittal extent are asserted before that
exclusion is used. Shared-vertex triangle pairs are treated as topological
neighbors. Preflight also checks physical closure/winding, seam correspondence,
finite morph attributes, skin bindings, and baseline weights.

232 configurations also receive body checks against the correspondingly morphed
body: actual triangle intersection counts plus nearest-surface signed distances
at every helmet vertex, triangle centroid, and edge midpoint. More than 1 mm of
sampled penetration fails the runtime gate. Shallower contacts remain visible in
the report. These finite samples and nearest-normal signs are diagnostics, not
continuous containment proofs.

Body clearance is omitted for the 48 configurations with nonzero skeletal
residual weights: those equipment residuals require matching bone translations,
while the body's corresponding exported morph deltas are zero. Applying only the
residual and comparing with an unposed body would produce misleading results.
Plate-intersection checks still run for those 48 configurations. This sweep
does not certify animated or every skeletal pose. Use a checker that applies
the joint proportion basis together with the residuals, such as
`scripts/check_underlayer_assets.py`, when assessing skeletal fit.

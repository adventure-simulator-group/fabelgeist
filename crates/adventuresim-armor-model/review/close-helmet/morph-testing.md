# Automated close-helmet morph testing

Run from the repository root with Blender and the installed neutral body asset:

```powershell
blender --background --python-exit-code 1 --python scripts/check_close_helmet_assets.py -- assets/equipment/procedural/close_helmet--worn.glb assets/animations/biped/unarmed/base.glb target/helmet-review/morph-sweep.json
```

The explicit Python exit code makes a failed assertion fail the command, including
in automation. The JSON report records the exact asset/body hashes, every weight
vector, failure location, intersection counts, penetration depths, and random seed.
Use `--only NAME` to reproduce one reported configuration.

## Configurations and checks

The sweep contains 240 configurations: neutral; each of the 45 identity channels
at both game-range limits (+/-0.35); all 47 positive basis endpoints; all 45
negative identity unit weights; all-positive, all-negative, and alternating game
weights; those three mixtures with each spine residual; 32 seeded random game
identities; and 16 seeded corner combinations. The seed is 25397.

150 configurations are within the runtime test scope. The other 90 are identity
basis/stress cases at +/-1, outside the game's generated identity range. They
remain in the report as diagnostics; failures there do not make the runtime gate
fail. The two spine endpoints are runtime cases, not out-of-range stress tests.

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

Body clearance is omitted for the eight configurations with nonzero skeletal
residual weights: those equipment residuals require the matching skeletal pose,
while the body's corresponding exported morph deltas are zero. Applying only
the residual and comparing with an unposed body would produce misleading results.
Plate-intersection checks still run for those eight configurations. This sweep
does not certify animated or every skeletal pose.

## Defects found by the expanded test

The previously installed candidate 33 had 248 nonadjacent triangle intersections
within its first nape lame even in neutral. The earlier component-only check did
not inspect surfaces within the skull component. Candidate 34 starts the first
lame at its root and uses a continuous sweep derivative, removing that fold.

Candidate 34 then failed eight sampled game-range combinations at the outer
nape corners, with shoulder penetration up to approximately 2.8 mm. Candidate 35
adds posterior sweep at those corners, leaving the head and visor unchanged.
The candidate 34 failure report is retained locally as regression evidence at
`target/helmet-review/export-34/morph-sweep.json`. The summary below and the
README record the final measured results and installed hash.

## Current measured result

Candidate 35: all 150 runtime cases pass;
all 240 cases have zero unintended component and self-intersections. Four
runtime cases have shoulder contacts no deeper than 0.285 mm. Of the 90
out-of-range diagnostics, identity 0 at +1 and identity 20 at -1 fail body
clearance (1.48 mm and 2.69 mm respectively). These results use the stated
1 mm tolerance, not a claim of zero body contact everywhere.

The full candidate 35 report remains local at
`target/helmet-review/export-35/morph-sweep.json`. The command above regenerates
the raw report, including every weight vector, using the committed checker and
installed asset. [Provenance](checks/provenance-35.json) retains the input hashes.

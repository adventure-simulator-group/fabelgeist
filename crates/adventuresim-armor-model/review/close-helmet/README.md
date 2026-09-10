# Close-helmet review evidence

Candidate 35 is installed after the expanded automated morph audit found and
fixed two nape defects missed by the earlier checks. The head and visor retain
candidate 33's reviewed coarse shape. [Morph testing](morph-testing.md) documents
the 240-configuration sweep, limits, and reproducible command. The older visual
and runtime reviews below apply to candidate 33; candidate 35's nape corrections
were assessed automatically, as requested.

## Reference and acceptance scope

The current target is The Metropolitan Museum of Art's
[Close Helmet, collection object 25397](https://www.metmuseum.org/art/collection/search/25397),
German, ca. 1500, accession 04.3.243. The museum's photographs substantiate the
modeled design's principal silhouette, proportions, and plate arrangement.
Museum reference views are [quarter](https://collectionapi.metmuseum.org/api/collection/v1/iiif/25397/1483089/main-image),
[front](https://collectionapi.metmuseum.org/api/collection/v1/iiif/25397/1483085/main-image),
[side](https://collectionapi.metmuseum.org/api/collection/v1/iiif/25397/1483086/main-image),
[rear](https://collectionapi.metmuseum.org/api/collection/v1/iiif/25397/1483087/main-image), and
[opposite side](https://collectionapi.metmuseum.org/api/collection/v1/iiif/25397/1483088/main-image). Reports identify their original
local paths under `historical-shape-sources/`; [sources.json](references/sources.json)
retains the museum image URLs.

Acceptance concerns this photographed design adapted to the shown wearer:
rounded enclosing skull, relatively straight cheeks and supported jaw, a visor
whose lower boundary rises toward the temples, an inward nape transition, and
an outward descending neck defense. Category resemblance alone is insufficient.
Rims, rivets, decoration, materials, and minor opening/surface details are outside
this coarse milestone. Review images are local, ignored artifacts rather than
repository assets. Raw check reports, per-frame manifests, and readiness logs
are also local artifacts; Git retains review decisions, result summaries,
provenance hashes, and reproduction inputs. Reports preserve which images were inspected. Keep generated
renders, matched bare-body views, and sections under `target/helmet-review/`;
the committed scripts and design recipes reproduce new review images.

## Review decisions and superseded evidence

The [coarse-width comparison](reports/coarse-width-reference.md) rejected
candidate 28's broad ear band and abrupt narrowing into the jaw. It required
comparison of width through the whole head rather than uniform shrinking.
[Candidate 30 broad](reports/broad-30-coarse.md) and
[known-failure](reports/regression-30-coarse.md) reviews passed that coarse
relationship, with stated reservations about the rounder face and angular rear
fan. Both critics had reviewed earlier candidates.

A [fresh candidate 32 review](reports/fresh-32-coarse.md) then failed the
skull-to-nape contour and questioned the dome/face balance. Candidate 33 gained
an independent nape-waist measurement and posterior blend. The
[candidate 33 follow-up](reports/fresh-33-coarse.md) passed after opening all
current worn/bare views, sections, and five reference photographs. Despite its
filename, that reviewer was returning from candidate 32, not fresh and unprimed.
The coordinating agent agrees that the narrower ear envelope, straighter jaw,
and restored inward nape transition meet the current coarse scope. The rounder
dome, relatively shorter lower face, and angular neck sweep remain recorded
non-blocking differences, supported by the wearer's anatomy in the matched views.

The [candidate 32 code review](reports/code-32.md) found a P2: adjustable
breathing rows could cross eye openings and reach a panic-capable constrained
triangulator. [Candidate 33 code review](reports/code-33.md) confirms that
pairwise containment, minimum-web, and segment-crossing checks reject the
conflict before triangulation; no further concrete P1/P2 was identified in that
bounded follow-up.

All earlier reports remain exact historical evidence and are superseded for
current acceptance: [broad 02](reports/broad-02.md),
[broad 18](reports/broad-18.md), [regression 18](reports/regression-18.md),
[regression 20](reports/regression-20.md),
[code review for 20](reports/code-review-final.md),
[historical search](reports/historical-shape-match.md), and
[earlier export/style review](reports/final-export-style-review.md).
Candidate 02's PASS was erroneous; the user rejected its wide bottom and visor
standoff. Prompts were corrected to require anatomical placement, proportions,
plate seating, and construction-scale separation. The subsequent 18/20 passes
also predated the explicit same-real-shape requirement and do not establish
candidate 33 acceptance. The earlier Met 26428 stepped-visor target is superseded
by Met 25397 for this milestone. Exact old report wording is retained, not
rewritten into a later verdict.

## Geometry and fitting contract

The assembly exports three component meshes: skull, bevor, and shield-shaped
visor. The skull component includes three overlapping nape lames. The
experimental rim was removed. Shared hinge metadata for the visor/bevor is
stored in reference-body coordinates and retained in glTF node extras; it does
not implement articulation or visor animation.

Independent skull, temple, jaw, lower-neck, and nape-waist measurements establish
anatomical bounds. Temple clearance defaults to 5 mm independently of the 10 mm
cranial reserve. Jaw/neck widths default to 840/800 permille of the temple
envelope as minimum style widths; wider measured anatomy wins. The posterior
blend is independent of the ear-width hold. For a fixed design, fitting changes
coordinates while keeping connectivity, component ranges, and morph
correspondence fixed. Adding/removing nape lames through a design change is a
topology change, not a wearer-dependent operation.

Sight span/gap/bridge and breathing count, rows, size, span, location, rotation,
rounding, and side are adjustable. Aperture lengths expressed as `Millimeters`
refer to the authored reference surface; fitting scales and curves their actual
physical dimensions. Pairwise hole and plate-boundary clearance checks operate
in that domain. The openings are through holes with return walls. Plate gauge
is limited to 1–4 mm. Layer offsets follow the carrier normal, while visor
`ShellExtrusion::InPlane` returns preserve horizontal sight rays and gauge along
the source vertex normal. Hard-normal vertex duplicates meet at coincident
physical seams. These local contracts do not guarantee continuous thickness or
clearance on every fitted surface.

## Installed artifact and automated morph checks

The installed candidate 35 asset is `assets/equipment/procedural/close_helmet--worn.glb`,
SHA-256 `07e812891fc2d668d8dacdb2a7862d4ce7b2104a52dd8fe611aa0afbfdf96305`. It retains three component meshes,
13,044 triangles, and 47 ordered morph channels. The export is retained under
`target/helmet-review/export-35/`.

The current sweep tests 240 configurations; [morph testing](morph-testing.md)
records the configuration recipe and reproduction command, and
[provenance](checks/provenance-35.json) records the asset and body hashes. All 150
runtime-range configurations pass the existing 1 mm penetration threshold.
There are no unintended component or self-intersections in any of the 240 cases.
Four runtime cases have shallow shoulder contacts, with maximum sampled depth
0.285 mm. Two out-of-range unit-weight diagnostics fail body clearance: identity
0 at +1 reaches approximately 1.48 mm into the shoulder; identity 20 at -1 reaches
approximately 2.69 mm into the throat. These are explicit limitations, not runtime
passes. Game-generated identity coefficients are bounded to +/-0.35.

Body checks run on 232 configurations. The eight nonzero skeletal-residual
configurations receive plate checks but omit body clearance because they require
the corresponding skeletal pose. Exact hashes, weights, failure locations,
triangle contacts, and distances are in the locally generated report at
`target/helmet-review/export-35/morph-sweep.json`. Raw reports are not committed. This does not establish
continuous or all-pose clearance. The intentional embedded comb/bowl join is
reported separately from unintended intersections.

The previous neutral self-intersection and candidate 34 shoulder-clearance
failures are summarized in [morph testing](morph-testing.md). Their raw reports
remain local regression evidence. The first nape lame now starts at its root with a continuous sweep;
its outer corners sweep farther backward to clear shoulders. The test failed on
the old geometry and passed the supported range after these corrections.

All 36 armor-model tests, formatting, and full lint pass. Logs are
`target/helmet-review/tests-35.log`, `fmt-35.log`, and `lint-35.log`.
Previous candidate 33 audits, design illustrations and runtime captures remain
historical evidence for that version; do not attribute them to the current hash.

## Reproduce the export and checks

Run from the repository root with pinned MHR assets, the canonical body GLB at
`assets/animations/biped/unarmed/base.glb`, Python/NumPy, and Blender on `PATH`.
Use a fresh empty staging directory. These commands reproduce an export and
checks; they do not install or overwrite runtime assets.

```powershell
cargo run --manifest-path crates/adventuresim-character-creator/Cargo.toml -- --generate-equipment --equipment-item close_helmet --lod 1 --recipe assets_src/characters/mhr_base.json --armor-designs crates/adventuresim-armor-model/review/close-helmet/designs/default.json --equipment-output target/helmet-review/reproduce35
python scripts/check_parametric_armor_assets.py target/helmet-review/reproduce35 --allow-partial
blender --background --python-exit-code 1 --python scripts/check_close_helmet_assets.py -- target/helmet-review/reproduce35/close_helmet--worn.glb assets/animations/biped/unarmed/base.glb target/helmet-review/reproduce35/plate-checks.json
python scripts/export_armor_morph_review.py target/helmet-review/reproduce35 target/helmet-review/reproduce35-bodies
foreach ($blend in @('neutral', 'positive', 'negative', 'mixed')) {
    blender --background --python scripts/check_armor_review.py -- "target/helmet-review/reproduce35-bodies/$blend"
}
Get-FileHash -Algorithm SHA256 target/helmet-review/reproduce35/close_helmet--worn.glb
cargo test -p adventuresim-armor-model
just fmt-check
just lint
```

`--allow-partial` is intentional for this single-item export. The morph-review
script reads the explicitly supplied staging directory. Render any resulting
body directory with `scripts/render_armor_review.py` through Blender; compare
new hashes and reports with the current provenance before equating a rerun with
the installed candidate.

## Candidate 33 native runtime evidence (before the nape fixes)

The candidate 33 hash `16128175cb230e0e4ec547ab0dadccf9a680437019e079f069a3e5deff11d595` was captured through the gameplay equipment loader,
rigid head skin, and synchronized wearer morphs. [Runtime provenance](runtime/provenance.json)
records the viewer binary hash and asset root. Exact fixture, 45 identity and
skeletal body parameters are retained in [idle](runtime/idle/) and
[guard](runtime/guard/). Readiness logs, per-frame manifests, and sampled
front/side screenshots remain local artifacts under `target/helmet-review/idle/`
and `target/helmet-review/guard/`.

The [independent runtime review](reports/runtime-33-review.md) passed the visible
coarse helmet in all twelve sampled views. Root inspection agrees. The raised
shoulder hides much of the nape interface in turned guard poses; clearance in
that occluded region remains unverified visually.

The raised-guard stationary-turn harness passed. The ordinary-camera-pitch
capture produced all 65 frames and complete artifacts, but the overall harness
failed its hand/chest animation-jitter metric (11 unacceptable incidents).
This remains a recorded animation-harness failure; it is not a helmet-specific
numerical collision test. The full validation verdicts remain in the local
runtime reports; the results above summarize both scenarios. The guard capture contains 128 frames. Visual inspection
of the helmet is bounded to the recorded sampled views and does not establish
all-pose clearance or animated visor actuation.

The default, wide-slot, two-row, and asymmetric design examples also passed
candidate 33 parameter checks: closed physical
walls, zero component intersections, and sampled wearer penetration within the
1 mm diagnostic threshold. Their locally generated front renders show the requested opening
controls; they are parameter demonstrations, not separately certified museum
reproductions.

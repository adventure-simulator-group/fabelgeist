# Breastplate profiles and fluting

The [museum photographs](references.md) guide rounded, centrally ridged, low
peascod, and fluted rounded front plates. These recipes simplify the front/back
shells and their integral waist flanges. They do not recreate separate plackarts,
articulated gussets, lance rests, or separate fauld lames from the references.

## Editable recipes

- [Rounded](designs/rounded.json): upper belly fullness and a shorter, contracted waist.
- [Central ridge](designs/tapul.json): medial ridge and lower chest projection.
- [Peascod](designs/peascod.json): fullness at the waist with a descending central point.
- [Fluted](designs/fluted.json): rounded shell with 16 flutes and a modest fan.

The creator exposes the same parameters in its breastplate panel and accepts
these complete JSON files through `--breastplate-design`. Profile and fluting
are independent: a preset is an editable starting point, not a separate mesh
implementation. A separate plackart would require separate construction.

`profile.projection_height` locates chest fullness between waist and neckline.
`waist_projection` independently controls projection at the flange seam.
`upper_chest_recession` independently straightens the upper slope of ridged
profiles without moving the waist or neckline. Projection,
medial ridge, and waist-point drop use millimetres. `waist_point_width` controls
how far the pointed lower boundary extends towards the flanks. Fullness controls the lateral
spread of the projected front. `fluting.width` is the fraction of each flute's
pitch occupied by relief; count and width can change independently. `spread`
and `lower_spread` control total pattern breadth and fan, while `start`, `end`,
and `fade` leave smooth margins. Fluting is confined to the front shell.

Detail sampling follows flute centers and edges. The fan uses lateral planes
on the fitted carrier, while preserving side boundaries, attachments, and the
medial crease. Fitting first establishes a
smooth carrier; refinement preserves that carrier and interpolates its extrusion
vectors. Relief displaces both inner and outer surfaces outwards, leaving closed
physical walls. Gauge is measured in the carrier extrusion direction, not
perpendicular to every flute slope. Topology is fixed across wearer morphs for a
recipe; changing the flute recipe can change vertex count and connectivity and
therefore requires regenerating its morph targets.

## Reproduce

Run from the repository root with Blender and pinned MHR assets available. Use a
new output directory per candidate; keep all generated artifacts under `target`.

```powershell
cargo run --manifest-path crates/adventuresim-character-creator/Cargo.toml -- --breastplate-design crates/adventuresim-armor-model/review/breastplate/designs/fluted.json --armor-review-dir target/breastplate-review/candidate
blender --background --python-exit-code 1 --python scripts/render_armor_review.py -- target/breastplate-review/candidate target/breastplate-review/candidate/renders breastplate --bare-body
blender --background --python-exit-code 1 --python scripts/check_breastplate_assets.py -- target/breastplate-review/candidate/breastplate--worn.json target/breastplate-review/candidate/body.json target/breastplate-review/candidate/check.json
cargo run --manifest-path crates/adventuresim-character-creator/Cargo.toml -- --generate-equipment --equipment-item breastplate --breastplate-design crates/adventuresim-armor-model/review/breastplate/designs/fluted.json --equipment-output target/breastplate-review/export
blender --background --python-exit-code 1 --python scripts/check_breastplate_assets.py -- target/breastplate-review/export/breastplate--worn.glb assets/animations/biped/unarmed/base.glb target/breastplate-review/export/morph-checks.json
```

The GLB audit checks 110 static identity configurations: neutral, 45 individual
channels at both +/-0.35 bounds, three aggregate mixtures, and 16 seeded corner
mixtures (seed 1640). It checks closure/winding, export attributes and morph seams,
nonadjacent triangle intersections, and body clearance sampled at all vertices,
edge midpoints and face centroids, with a 1 mm penetration tolerance. Reports
retain exact input hashes, weights, and failures locally. Skeletal poses and
animation are outside this static identity audit; finite nearest-surface samples
are not a continuous containment proof.

## Fitting and morphs

The carrier uses a separate upper armscye boundary and aligns lateral lap width
without pulling the back's depth contour forward to the front arm cutaway.
Independent positive-endpoint refitting was replaced with fixed body
correspondence: identity displacements transfer through the smooth coarse
carrier, and detailed flute vertices interpolate that same displacement.
Inner/outer walls and shading aliases share displacement, retaining their gauge
vectors and closed seams. This avoids accumulating nonlinear fitting offsets
across signed identity combinations. It does not prove continuous containment,
constant surface-normal gauge, or skeletal-animation safety.

This change adds authoring controls and recipes. Existing catalog GLBs have not
been replaced; installed game assets, skeletal poses, and animation have not
been validated for these new recipes. Parameter limits constrain inputs, not a
claim that every combination throughout the Cartesian product is certified.

Review reports, images, exports, binaries, and raw diagnostics remain local under
`target/breastplate-review`. Recipes and source links are retained here.

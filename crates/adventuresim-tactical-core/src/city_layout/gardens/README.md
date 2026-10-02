# Property gardens

An accepted `CityGarden` belongs to one occupied front building and its lot.
The compiler resolves the actual building recipe before siting beds, connected
working lanes and a common-hazel specimen. Merchant rear-range compounds keep
their separate courts and gates. Garden soil patches derive from accepted beds;
an anonymous soil patch does not create a garden or a planting entitlement.

The six-metre rear reservation, two beds, lane widths, planting frequency and
specimen scale are authored layout choices. They do not model household food
capacity, crop yield or historical plot-size distributions. An Erfurt record
of 1 March 1534 (7A-100) documents garden property relationships and shared well
and roof-water obligations; it supports ownership context rather than those
procedural dimensions. Fuchs's 1543 chapter 151 discusses wild and cultivated
hazel. The current common-hazel mesh is not a reconstruction of the historical
cultivated variety or an orchard tree.

- [Erfurt archival finding aid](https://www.archive-in-thueringen.de/de/findbuch/view/bestand/27377/systematik/28365)
- [Fuchs, chapter 151](https://vergil.uni-tuebingen.de/publikationen/FuchsKraeuter/capitel/151.html)

## Geometry and terrain authority

Each plant has a stable identity, specimen, orientation, position and positive
scale. The canonical envelope in `content/tactical/garden-hazel-envelope.json`
measures every vertex of the deterministic production branch and leaf meshes.
Core siting transforms that hull and adds the shader's wind displacement in
world metres. Nominal botanical height and crown radius are not clearance
bounds. Beds, lanes, other plants and all resolved building projections remain
clear of the complete accepted specimen. Clients use the supplied pose without
sampling a new position, changing scale or discarding an accepted plant.

The whole garden plot participates in playable-property partitioning and
terrain grading. Touching property pads share one terrace. A property extending
onto vista terrain requires a coplanar supporting vista surface. Its terrace
and the playable-edge intervals sampled by vista stitching use that elevation.
Unsupported non-level vista footprints, including LOD morphs, fail generation.
Final validation checks the same stitched triangles used by presentation.
Distant
plants use their owner's validated base elevation; playable plants use the
core-generated pad elevation.

Cultivated bed triangles replace the packed-yard triangles beneath them.
Terrain grading does not change the substrate or ground-cover ownership of
its supporting margins. Exact street, building and yard regions travel with
`SceneGround`; queries apply those boundaries before natural grid samples.
Presentation preserves those boundaries instead of warping them with natural
ground-cover noise.

Working ground is reserved from incidental furniture. Urban bare ground has
zero cover density and suppresses wild shrubs; vista grass uses the existing
street and yard mask. Accepted garden plants use the production branch and
leaf materials independently of wild understory sampling.

## Regeneration and review

After changing the common-hazel geometry, export and review its new envelope:

```powershell
$env:FABELGEIST_UPDATE_GARDEN_SPECIMEN = '1'
cargo test -p adventuresim-tactical-client --bin tactical-scene-viewer managed_hazel_matches_shared_envelope --offline
Remove-Item Env:FABELGEIST_UPDATE_GARDEN_SPECIMEN
cargo test -p adventuresim-tactical-client --bin tactical-scene-viewer managed_hazel_matches_shared_envelope --offline
```

Regenerate scene fixtures and the prepared city layout after layout changes.
Run the garden core tests and native review. The capture gate checks accepted
plant identity, pose, mesh count and GPU texture residency.

```text
cargo run -p adventuresim-tactical-core --bin generate-scene-fixtures --offline
cargo run -p adventuresim-tactical-client --example generate-art-demo-city --offline
cargo test -p adventuresim-tactical-core garden --offline
python scripts/capture_garden_review.py --output target/garden-review
```

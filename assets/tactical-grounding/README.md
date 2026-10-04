# Geographic support regression fixtures

`goslar-1238.json` binds the generated merchant property 1238, its front
building 1238 and rear building 17622 to exact programmes, horizontal placements,
court thresholds and geographic observations. Source digests, scene revision,
time and seed are embedded in the document. The four geographic triangles are
the complete covering cells of the ungraded production 50 metre near vista,
with its original diagonal and elevations relative to the settlement datum.
They describe modern source relief, not a surveyed 1544 parcel.

The original half-metre landing candidate remains a comparison case. It passes
isolated support traversal but fails entry through the actual rear doorway.
`doorway_solution` records the longer landings and selected separate floor
levels that pass geographic approach, both building entries and return with
the production controller and static building collision. Engineering bounds
and their calibration are recorded separately from geographic observations.
Neither candidate establishes structural retaining-wall capacity or drainage.
The complete generated property remains required even though modern inventory
geometry places part of it across a mapped medieval wall.

`goslar-965.json` retains the same provenance contract for front building 965
and rear building 17349. Its eight source triangles cover the complete property
and street approach. A level-ground control proves that the unchanged street
door is traversable. On geographic relief, support continues through the front
setback and reserves a level doorway landing. Its four-metre external apron
admits a bounded stair flight; a one-metre declaration fails with an exact
stair-going shortfall. The geometric and controller regressions also cover the
source sliver left by independently rounded plot/apron boundaries. The owning
clipper shares their declared join without merging unrelated properties.

Run the owning regression suite independently of the retired strategic UI:

```sh
cargo test -p adventuresim-tactical-core --lib city_layout::grounding
```

These tests accept a bounded property support experiment. They do not accept
the production city generator, surrounding LOD stitching or all city access.

## Geographic attribution

The extracted generated terrain values retain the source terms and attribution
in [the map and terrain data notice](../../MAP_DATA_LICENSE.md), including
Copernicus DEM GLO-30. Fabelgeist resamples and triangulates the source; the
providers do not endorse these modifications. The fixtures redistribute extracted
compiled scene triangles, not an original source raster. Project contributions
to these generated data follow the notice's CC BY-SA 4.0 licence boundary.

`neighboring-property-support.json` freezes generated compound 519 (members
519 and 16903) beside home 520 at population 6,500, seed 42. It is an algorithmic
fixture on declared gentle source relief, not a historical geographic survey.
Rectangular floor bounds overlap by 0.00937 m² at an empty corner. Exact floor
cross-sections and their convex envelope are disjoint. The named regression
`population_6500_seed_42_neighboring_members_keep_exact_support_without_empty_corner_ownership`
checks complete contacts, unchanged programmes and positions, stable members,
source preservation and deterministic property composition.

`goslar-1036-clipped-source.json` freezes one source triangle and all intersecting
property clipping outlines from the complete generated city. A clipping
intersection was lifted before its horizontal coordinates were rounded for
presentation and collision. That changed a seven-micrometre triangle's normal
from the source plane into an apparent grade of 0.854. The clipper now evaluates
height at the represented horizontal point, retaining the same cuts and
property identities. The regression checks that every emitted vertex remains
on its geographic plane and that this gentle source does not gain a steep face.

The ignored controller test
`production_garden_routes_allow_entry_tending_and_return` accepts a frozen
production export through `FABELGEIST_TERRAIN_SCENE` and writes observations
to `FABELGEIST_GARDEN_REPORT`. It installs the exact terrain and nearby fixed
building, enclosure, physical obstacle and tactical outdoor furniture colliders,
with closed gates, then traverses each garden
from street to tending lanes and back. `FABELGEIST_GARDEN_PROPERTY` restricts
a reproduction to an exact property. This check complements complete working
polygon comparisons in the dispatcher's `compare-city-terrain` example;
centreline traversal alone does not prove every corridor's support.

`goslar-2-stair-contact.json` contains 200 unmodified six-vertex foundation
cells intersecting the failed stair contact near scene coordinates
(26.833, -31.933). It was extracted from the production Goslar export at
minute 340320, settlement seed 3918113949425128608, generation 66. Property
and building 2 retain the inn programme and the merchant-house geometry;
opening 80 connects to the rear approach. Its terrain derives from the same
attributed modern GLO-30 input described above. The small fixture reproduces
a controller shape cast reporting a steep corner normal on a gentle physical
bearing. The regression enters and returns over the retained cells with the
production controller. Complete city and entrance acceptance remains separate.

`kassel-1-access-obstacle.json` retains the accepted support for property and
building 1 and the source rock at cell (76, 53). The rock's centre and its
0.75-metre collision radius are outside the bare approach, but obstruct the
pedestrian cylinder at the approach endpoint. The physical reservation includes
the existing cylinder radius and contact skin. Scenery intersecting that envelope
is excluded after geographic preparation is bound; source relief, property
surfaces and floors remain unchanged. The regression checks the exact outline,
the required clearance, a retained distant obstacle and order independence.
Full-width imported entry and return checks install the server's actual nearby
obstacle and tactical furniture geometry. Interiors remain explicitly unfurnished;
vista furniture deliberately has no physics.

`carpenter-passage-access.json` records property and building 9 of the required
900-resident, seed-47 fixture at minute 340320. An outdoor furniture candidate
blocked the front workplace passage: its internal route had been reserved, but
its accepted exterior approach had not. The regression reserves that exact
validated approach before candidate selection, with the existing furniture
shoulder clearance. It preserves the programme, placement and support surface.
The full controller fixture checks entry and return at all usable-width tracks.

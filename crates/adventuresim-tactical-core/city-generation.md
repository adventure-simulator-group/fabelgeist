# Generated city geometry

The production frontage planner reserves service properties, including their
courts and rear ranges, before packing dwellings along block edges. It completes
nearer blocks before developing peripheral ones. Lanes, secondary streets and
primary streets have distinct widths; their current 4/6/8 metre widths are
procedural design parameters, not measurements recovered from historical views.
Buildings retain their full recipe dimensions. Density comes from parcel and
block organization, rather than scaling houses down.

Distant, close and playable buildings use the same occupied programme. LOD
changes mesh detail and collision activation, while placement, footprint, roof
and storey programme remain canonical. Prosperity changes exterior finish
without selecting a different building mass. Shared construction kits, GPU
instances, bounded workers and immutable asset caching remain in use.

The authored architectural ground floor is plan Y=0. Shared placement seats
that datum at the building's graded elevation. Recentring collision and mesh
geometry uses the collision bounds' centre; the lower collision bound does
not establish a second floor datum. Slabs and footings extending below Y=0
remain buried. Door thresholds, interior furniture and distant exteriors use
the same reference.

Generated homes have settlement-scoped property identities. The native
producer derives a home catalogue from the same lots used by the scene
compiler. Imported scene documents carry that catalogue and reject a home
whose building ID, transform or footprint disagrees with the scene. The
[residence authority contract](../adventuresim-stdb-module/residences.md)
describes registration, legal holdings and occupancy. These records do not
establish an NPC's observed position in a tactical session.

## Historical references and limits

The [Rammelsberg museum's discussion of Sincken's 1574 Goslar view](https://blog.rammelsberg.de/2019/04/wunsch-und-wirklichkeit-landschaft-aus-der-perspektive-ihrer-nutzer/)
provides context for roofscape, landmark hierarchy and the mining landscape.
The [Kassel Stadtmuseum's 1547 plan](https://www.kassel.de/einrichtungen/stadtmuseum/besuch/guide.php)
and [Städel's 1521 Augsburg view](https://www.staedelmuseum.de/de/digital/holbeins-augsburg)
provide comparative frontage, courtyard and precinct evidence. Pictorial
compression and colouring do not establish street dimensions, population
counts or material percentages. No historical scan is vendored.

The central German market-town grammar is one regional grammar. It does not
establish a reconstruction of surveyed Goslar parcels or justify using this
grammar for every settlement. The existing imported scene builder grades the
urban vista to one elevation; property elevation and collision-valid entry
anchors remain further work. Captures use the imported settlement position and
terrain without an invented mountain backdrop or exaggerated relief.

## Geographic height evidence

The terrain pack compiler reads Copernicus GLO-30 raster values directly and
rounds them to integer metres. It does not remove buildings or vegetation.
[Copernicus identifies GLO-30 as a modern digital surface model](https://dataspace.copernicus.eu/explore-data/data-collections/copernicus-contributing-missions/collections-description/COP-DEM).
Its nominal 30 metre resolution describes the source product; it does not
prescribe the scene grid or establish surveyed ground beneath a building.
The imported vista currently samples at 50, 250 and 1,000 metres, preserving
neighbourhood peaks before city grading. Source, resampled, repaired, graded
and presented heights are distinct measurements.

[LGLN's DGM1 catalogue](https://numis.niedersachsen.de/trefferanzeige?docuuid=740e33da-3310-4173-bae1-d30c31124b3a)
and its [coverage service](https://opendata.geoservices.lgln.niedersachsen.de/dgm_wcs?service=WCS&version=2.0.1&request=GetCapabilities)
provide an independent contemporary ground model for bounded diagnostic
comparisons in Niedersachsen. Record the requested bounds, returned coordinate
system, sampling and digest. It does not establish historical street or garden
levels. Different vertical datums also require care when comparing absolute
heights; footprint relief can be compared without treating a fixed offset as
a slope.

## Independent capture and verification

Export an imported city through the production dispatcher:

```sh
cargo run -p adventuresim-tactical-server-dispatcher \
  --example export-city-scene -- \
  --world WORLD.json --settlement viabundus-2337 \
  --terrain-manifest TERRAIN.json --terrain-pack TERRAIN.pack \
  --absolute-minute 340320 --output target/goslar.json \
  --property-catalog target/goslar-homes.json \
  --terrain-stages target/goslar-terrain-stages.json
```

The exporter requires current world provenance and a final terrain pack. It
rejects a scene window outside that pack. Operator names and IDs are explicit
capture fixtures, not persistent resident presence. Review the same document
with `tactical-scene-viewer --scene-input target/goslar.json --profile
city-review` for street, neighbourhood, aerial and skyline views.

The optional terrain-stage export records fixed source transects and the
production vista samples before city grading. `inspect-city-grounding` reports
architectural datums, lower collision geometry, structural nodes, footprint
corners, external thresholds and facade/shell bounds for a bounded area. It
clips playable terrain triangles against rotated footprints, including interior
vertices and boundary intersections. It follows the playable collider's
diagonal; vista triangulation uses a different diagonal. The report explicitly
marks footprints that are not completely covered by playable terrain. The
vista inspection follows rendered cell subdivision and vertex morphing and
reports clipped area, without substituting another representation for missing
coverage.

Limit the report to exact IDs with `--building-id 1238,17622`. Add
`--property-id 1238 --terrain-stages target/goslar-terrain-stages.json` to compare
a common property plane with separate building floors and a stepped court. The
comparison reserves landings and checks endpoint elevation feasibility at the
existing terrain repair grade. Routes cross the court, so separate floor levels
require a terraced court surface with a level central landing. The existing
horizontal court reservation does not imply a single level plane. This is a
decision experiment, not final grading, retaining-wall capacity, gate-sweep or
collision acceptance. The report must not be used to accept an unsupported
required fixture. The stage export must identify the exact scene digest and
terrain source. A mismatched export is rejected.

Support diagnostics intersect fixed collision cuboids with architectural Y=0;
they distinguish buried solids, ground contact and upper projections. Gate
posts use the same dimensions as runtime enclosures. The diagnostic compares
actual contact polygons with the fixed post, since distance to the hinge line
does not measure the available support corridor. A contact envelope alone is
not proof that every point inside it is a structural bearing.

Run the native catalogue/partition checks and layout tests with:

```sh
cargo test -p adventuresim-tactical-server-dispatcher --lib
cargo test -p adventuresim-tactical-core --lib city_layout
cargo test -p adventuresim-tactical-core --lib scene_input
```

Record matched images, frontage coverage, developed footprint and cold/warm
preparation separately in ignored `target/` output. A smaller footprint alone
is not historical or visual acceptance. The retired strategic city interface
is not needed for generation, catalogue registration or these checks.

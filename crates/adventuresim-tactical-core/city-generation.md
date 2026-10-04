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

## Independent capture and verification

Export an imported city through the production dispatcher:

```sh
cargo run -p adventuresim-tactical-server-dispatcher \
  --example export-city-scene -- \
  --world WORLD.json --settlement viabundus-2337 \
  --terrain-manifest TERRAIN.json --terrain-pack TERRAIN.pack \
  --absolute-minute 340320 --output target/goslar.json \
  --property-catalog target/goslar-homes.json
```

The exporter requires current world provenance and a final terrain pack. It
rejects a scene window outside that pack. Operator names and IDs are explicit
capture fixtures, not persistent resident presence. Review the same document
with `tactical-scene-viewer --scene-input target/goslar.json --profile
city-review` for street, neighbourhood, aerial and skyline views.

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

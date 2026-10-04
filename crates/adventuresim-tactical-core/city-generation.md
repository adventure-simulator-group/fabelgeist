# Generated city geometry

The production frontage planner reserves service properties, including their
courts and rear ranges, before packing dwellings along block edges. It completes
nearer blocks before developing peripheral ones. Lanes, secondary streets and
primary streets have distinct widths; their current 4/6/8 metre widths are
procedural design parameters, not measurements recovered from historical views.
Buildings retain their full recipe dimensions. Density comes from parcel and
block organization, rather than scaling houses down.

After programme selection, the compiler seats each single-building bearing
inside its existing plot using the actual floor hull. A nominal programme width
can omit wall and floor thickness; the floor may therefore extend beyond a plot
when its collision centre is placed at the nominal lot centre. Seating applies
the smallest in-plot translation without changing programme, dimensions or
rotation. Garden lanes reconnect around the measured envelope while retaining
their owned working ground, plant identities and existing route widths. An
insufficient plot is rejected with the exact member and measured local bounds.

The compiler then measures the complete detailed render and collision envelopes.
It solves property translations along the original street frontages within their
blocks. When the authored setback cannot seat a complete block, packing may
reclaim its unused legal street setback, checking all four existing road and
block boundaries. It does not enlarge the block or move the street. This
preserves selected identities, programmes, dimensions, orientation, membership
and capacity. A translation carries the property's buildings, court, enclosure,
garden and access route; the outer route hook stays on its original street.
Private land, bearing hulls and elevated building projections are separate
packing constraints. Empty corners between a roof envelope and garden do not
acquire occupancy.

Joint frontage constraints include adjacent and opposing rows and corner
properties, garden beds, tending corridors and the actual planting hulls with
their existing leaf-wind clearance. They use the same route transformation as
the published garden. Private ground and elevated projections remain separate:
a roof may overhang empty ground, but cannot block an accepted garden.

The cheap authored candidates precede a deterministic continuous linear solve
within one block. Stable row-order candidates and separating-plane branches
share a 100,000-state operational cap; each row candidate receives at most 128
linear solves. The pinned Apache-2.0 `microlp` 0.6.0 dependency has no native
solver requirement. An independent check of the rounded world geometry remains
authoritative. An exhausted search or numerical failure is an unresolved
placement, not proof of physical infeasibility. It reports the exact block and
member IDs rather than discarding houses or substituting programmes. Plot
containment uses double-precision comparisons of stored poses with a 1 mm
coordinate roundoff bound. That engineering bound accommodates translated
coincident edges in f32 world coordinates; it does not enlarge legal property or
grading limits. Points beyond the bound remain outside the plot.

Distant, close and playable buildings use the same occupied programme. LOD
changes mesh detail and collision activation, while placement, footprint, roof
and storey programme remain canonical. Prosperity changes exterior finish
without selecting a different building mass. Shared construction kits, GPU
instances, bounded workers and immutable asset caching remain in use.

The authored architectural ground floor is plan Y=0. Shared placement seats that
datum at the building's graded elevation. Recentring collision and mesh geometry
uses the collision bounds' centre; the lower collision bound does not establish
a second floor datum. Slabs and footings extending below Y=0 remain buried. Door
thresholds, interior furniture and distant exteriors use the same reference.

Both playable and distant scene placements carry `base_elevation_metres`, the
scene elevation of architectural Y=0. Promotion copies the exact programme,
identity, horizontal placement, orientation and floor datum. Generated buildings
read that placement instead of retaining a second, mutable pad elevation. Scene
schema 27 and generation version 70 require floor datums and accepted property
support directly; previous formats are rejected. A changed floor invalidates the
scene product while equal programmes retain the same façade recipe keys.

Production geographic preparation retains complete building, property, garden,
street and yard reservations before grading. It prepares fine terrain,
playability repairs and existing stitched vista rings without building pads or
whole-city levelling. The city producer selects floors and bounded support
against those exact triangles, and publishes their compact projection atomically
with the placements. The original playable and vista samples remain unchanged.

An occupied scene requires accepted support. Source and placement digests bind
its plans to exact geographic triangles, programmes, identities, horizontal
poses and floor datums. Runtime generation reconstructs and installs that
support before furniture and collision preparation. Missing or stale support
fails; loading never selects a replacement floor or expands a grading boundary.
Empty geographic scenes carry no property projection.

Geographic microrelief uses the pinned software exponential for road ruts and
tree-root mounds. A platform exponential can differ by one floating-point unit,
invalidating the exact source digest even when the visible shape agrees. The
source binding keeps exact triangle bits across native and browser preparation;
it does not round terrain or accept an approximate digest.

Every compound owns its complete plot and bound passage. A single building owns
its actual bearing outline and bounded doorway approaches, leaving courts and
gardens on the geographic surface. Level courts are considered before stepped
terraces. Different owners cannot merge support through proximity. Compilation
rejects overlapping owners, absent source coverage, excessive cut/fill or
invalid access with exact property/member identities, locations, measured
constraints, permitted bounds and attempted treatment. A required fixture
remains failed when it cannot satisfy those constraints.

The production engineering policy bounds smooth grading to 0.65, cut/fill to 6
metres, coordinate contact error to 1 millimetre and street approaches to 4
metres. Stair bounds are a 0.19 metre riser, 0.25 metre going, 1 metre clear
width, 1.05 metre floor landing and 0.5 metre court landing. Foundation
embedment is 0.2 metres. These are engineering constraints, not historical
measurements or substitutes for actor/collision acceptance.

Garden plant roots retain explicit plant IDs and soil elevations in
`SceneGarden`. `SceneGarden::project` uses the complete physical terrain rather
than the house floor, rejects missing soil for an exact plant and publishes the
complete root set together. Distant gardens wait for their matching scene
terrain before invoking the same projection, independent of message arrival
order. This root contract does not certify route grades, discontinuities or
working space.

Doorway landings that extend into their own bearing join the existing floor;
they do not contribute duplicate coplanar support triangles. The same-property
surface union preserves the floor, doorway pose, grading boundaries and source
terrain outside those boundaries.

Enclosure presentation and collision consume exact property-supported masonry
cells and the accepted gate datum. A house floor does not position a gate or
level a wall. See the accepted enclosure contract below. Complete access and
ordinary presentation acceptance remain separate from compiling those cells.

Upper town and merchant dwelling variations are explicit catalogue programmes.
Their seeds determine storey heights, roof pitches and room growth. A structural
repair must not choose another variation because it admits an earlier candidate
in a recipe search. The city palette compiles those exact programmes and rejects
an invalid programme without substituting a different house.

Generated homes have settlement-scoped property identities. The native producer
derives a home catalogue from accepted compiled building placements. The scene
producer reuses that compiled layout rather than compiling the city a second
time. Imported scene documents carry that catalogue and reject a home whose
building ID, transform or footprint disagrees with the scene. The [residence
authority contract](../adventuresim-stdb-module/residences.md) describes
registration, legal holdings and occupancy. These records do not establish an
NPC's observed position in a tactical session.

## Historical references and limits

The [Rammelsberg museum's discussion of Sincken's 1574 Goslar
view](https://blog.rammelsberg.de/2019/04/wunsch-und-wirklichkeit-landschaft-aus-der-perspektive-ihrer-nutzer/)
provides context for roofscape, landmark hierarchy and the mining landscape. The
[Kassel Stadtmuseum's 1547
plan](https://www.kassel.de/einrichtungen/stadtmuseum/besuch/guide.php) and
[Städel's 1521 Augsburg
view](https://www.staedelmuseum.de/de/digital/holbeins-augsburg) provide
comparative frontage, courtyard and precinct evidence. Pictorial compression and
colouring do not establish street dimensions, population counts or material
percentages. No historical scan is vendored.

The central German market-town grammar is one regional grammar. It does not
establish a reconstruction of surveyed Goslar parcels or justify using this
grammar for every settlement. The imported scene builder retains the geographic
vista and selects bounded property support instead of levelling an urban
rectangle. Collision-valid entries and enclosure/garden presentation remain
separate acceptance obligations. Captures use the imported settlement position
and terrain without an invented mountain backdrop or exaggerated relief.

## Geographic height evidence

The terrain pack compiler reads Copernicus GLO-30 raster values directly and
rounds them to integer metres. It does not remove buildings or vegetation.
[Copernicus identifies GLO-30 as a modern digital surface
model](https://dataspace.copernicus.eu/explore-data/data-collections/copernicus-contributing-missions/collections-description/COP-DEM).
Its nominal 30 metre resolution describes the source product; it does not
prescribe the scene grid or establish surveyed ground beneath a building. The
imported vista currently samples at 50, 250 and 1,000 metres, preserving
neighbourhood peaks before city grading. Source, resampled, repaired, graded and
presented heights are distinct measurements.

[LGLN's DGM1
catalogue](https://numis.niedersachsen.de/trefferanzeige?docuuid=740e33da-3310-4173-bae1-d30c31124b3a)
and its [coverage
service](https://opendata.geoservices.lgln.niedersachsen.de/dgm_wcs?service=WCS&version=2.0.1&request=GetCapabilities)
provide an independent contemporary ground model for bounded diagnostic
comparisons in Niedersachsen. Record the requested bounds, returned coordinate
system, sampling and digest. It does not establish historical street or garden
levels. Different vertical datums also require care when comparing absolute
heights; footprint relief can be compared without treating a fixed offset as a
slope.

Complete-context geographic preparation retains buildings, property
reservations, gardens, streets and yards while preparing the repaired fine
surface. Obstacle clearance uses horizontal reservations independently of member
floors. No property pad levels that source surface.
`TacticalSceneInput::prepare_geographic_terrain` returns this ungraded surface
and its original asset context for bounded support selection.
`GeographicSurface::measure_region` records complete triangle-intersection
coverage and maximum grade; it cannot certify traversal over retaining edges.

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
marks footprints that are not completely covered by playable terrain. The vista
inspection follows rendered cell subdivision and vertex morphing and reports
clipped area, without substituting another representation for missing coverage.

Limit the report to exact IDs with `--building-id 1238,17622`. Add
`--property-id 1238 --terrain-stages target/goslar-terrain-stages.json` to
compare a common property plane with separate building floors and a stepped
court. The comparison reserves landings and checks endpoint elevations against
the existing terrain repair grade. The support comparison separately compiles a
level court with narrow stair flights between independently supported buildings.
Terrace edges follow complete measured bearings, which can extend beyond the
nominal court edge. Source elevations and chosen floor elevations are distinct.
Stair width and endpoint landing length have separate bounds. Widening a flight
must not silently shorten its run. The compiler reserves complete landings at
both route endpoints and checks their corners against the property reservation.
It rejects insufficient run or clearance instead of clipping stairs, changing a
floor or enlarging the reservation. Diagnostics identify the property and member
IDs, constraint, boundary, location, measured and permitted values, units,
shortfall and attempted treatment.

The construction report integrates cut and fill over source/support triangle
intersections. Retaining face areas and provisional masonry thickness provide
rough comparative quantities, with perimeter sampling and exclusions stated in
the payload. They do not establish structural wall sizing or historical costs.
These decision experiments do not establish final retaining-wall collision,
neighbouring boundary, drainage or cart-access acceptance. The production
producer installs the selected bounded building support; its diagnostic report
must not accept an unsupported required fixture. The stage export must identify
the exact scene digest and terrain source. A mismatched export is rejected.

`BoundedSettlementTerrain` composes disjoint owned properties against one
source. It rejects duplicate identities, split member authority and overlapping
grading regions before clipping. Natural triangles that do not intersect a
declared region retain their original topology and elevations. Each foundation
remains a closed convex prism. Exposed cuts also carry vertical faces from the
owned surface to unchanged natural ground. Those faces are present in rendering
and collision but are excluded from walkable floor queries. Internal joins
between a plot and its doorway apron do not acquire cut faces. Collision uses
the prism's known faces directly rather than reconstructing a hull, which can
reject thin source-intersection cells. A shared query index references the same
natural and bearing triangles; cached products carry that index without
rebuilding it on warm reuse.

`SceneTerrain::with_property_surface` installs the accepted topology for
queries, rendering and collision. It does not reconstruct the surface from a
heightfield with another diagonal. Both terrain render levels retain the owned
geometry. Queries at retaining edges preserve distinct elevations; actors and
thresholds can select support below an explicit vertical ceiling. Refinement and
sample rewrites are rejected after installation. The server, capture viewer and
art viewer install the resulting colliders on independent static bodies attached
to the transient scene. A restored scene regenerates those bodies from immutable
geometry. Scene schema version 27 and terrain generation version 70 invalidate
older generated products.

`GroundedCitySceneLayout::support_projection` encodes complete accepted plans
without expanded foundation cells. Source and placement digests bind the plans
to exact geographic triangles and occupied programmes, transforms and floors.
`CityGroundingProjection::reconstruct` rejects changed bindings, malformed
indices, invalid regions and support outside declared boundaries or grade limits
before publishing geometry. Round-trip reconstruction must reproduce the
accepted closed surface exactly. The versioned production scene document carries
this projection. Expanded generated geometry uses bounded runtime transport and
renderer partitions at the original playable and vista-ring boundaries. Verify
exact geometry reconstruction and payload bounds at those boundaries.

Accepted city compilation retains a lightweight recipe memo through playable
partitioning. Support planning reuses the exact occupied programme, including
its authored variation seed. Reconstruction of an occupied
programme never searches for a different valid recipe. Facade meshes remain in
the existing presentation caches; this memo stores only programmes, collision
geometry, thresholds and measured envelopes.

`validate-city-support --world WORLD.json --scene-input target/city.json
--terrain-stages target/city-stages.json --policy-fixture POLICY.json --output
target/support-report.json --surface-output target/city-support.json`
reconstructs the exact production layout and checks both compounds and single
properties. It rejects an input whose programmes, membership, streets or
horizontal placements differ from that reconstruction. Accepted composed
geometry can be emitted for independent inspection. Its report includes
compilation and query timings, serialized bytes and unchanged source-vertex
checks. `--support-projection-output` writes compact plans only after exact
reconstruction passes. A rejected reconstruction exits with failure and writes a
separate `.rejected.json` diagnostic beside that requested path. These native
component measurements do not establish full producer, browser worker, renderer
or complete-city acceptance. Gardens and enclosures require their corresponding
support handoff and complete movement checks.

Access acceptance must exercise `AdventureSimulatorPhysicsPlugin` with the
production humanoid collider and authored movement configuration. Individual
risers below the maximum step height do not prove traversal: body clearance,
contact skin, tread width and the approach from rest also affect the step
calculation. A geometric support comparison must remain rejected when that
movement check fails. Diagnostic changes to controller tolerances are isolated
experiments; they are not accepted production settings.

The [Goslar support fixture](../../assets/tactical-grounding/README.md) compares
a one-metre clear stair width with half-metre and 1.05 metre endpoint landings.
The shorter landing admits an isolated surface goal but fails entry through the
rear building's actual collision geometry: the actor's rear radius still
overlaps upper treads when its head reaches the doorway. The longer landing
accounts for the measured wall stand-off, humanoid radius and unchanged contact
skin. The unobstructed court end has a separate half-metre landing bound.
Complete source/support intersections also constrain a shared vertical change of
front, court and rear floors. This retains flight rises and horizontal geometry
while seating the selected construction within its displacement bound. Goslar
property 1238 selects floors at approximately 21.043, 20.062 and 19.057 metres
relative to the settlement datum. A containment query at a rounded footprint
corner is insufficient evidence: the actual clipped triangle must satisfy the
cut/fill bound at every source intersection.

Street access reserves a level landing before the doorway. A supplied external
apron ends at the plot boundary; its support continues through the existing
front setback to the measured bearing edge. Compare a smooth ramp within the
0.65 triangle-grade bound with a flight within the supplied 0.19 metre riser and
0.25 metre going bounds. The mean rise/run of discrete stairs is distinct from
the grade of their walkable tread triangles. The actor's movement limits remain
unchanged. Property 1238 uses a two-metre external apron; property 965 requires
the declared four-metre apron and a stair approach. A shorter declared
reservation rejects with the exact required run and available run. A compiler
never enlarges that external reservation.

Source clipping welds the explicitly bound apron to the same front half-plane as
its property. Independently rounded rectangles otherwise leave a thin source
triangle at the join, which can block the full actor despite passing point
samples. This weld applies only to the declared join of one property; it does
not merge neighbouring terraces or choose their elevations. These engineering
bounds and selected floors are not historical measurements or structural
retaining-wall design.

Finite foundation cells intersect the complete support triangles with source
terrain triangles, splitting where the two planes cross. Their closed bottoms
extend below both surfaces by a separately supplied architectural embedment.
Missing or overlapping source coverage and excessive cut or fill reject with
exact property/member identities and measured shortfalls. Natural triangles are
clipped only inside the declared plot, existing street approach and exact
street-door apron. Remaining triangle planes retain their geographic elevations.
Clipped heights are evaluated at represented horizontal coordinates, so narrow
float pieces do not acquire a different plane. Edge queries admit only the float
rounding cell, not the architectural contact margin; they retain separate soil
and floor datums at retaining boundaries. Collision uses solid convex foundation
cells and a separate natural-ground body, avoiding nested composite colliders.
These data share spatial queries and collision; production scene generation
installs the accepted support. Presentation partitions the accepted triangles at
the original playable and vista-ring rectangles, retaining the existing material
palettes and culling chunks. Verify support and access through ordinary captures
and actor collision tests. Keep performance measurements separate from
diagnostic render layers.

`CitySceneLayout::plan_compound_support` binds complete occupied programmes to
their exact member IDs, measured ground contacts and uniquely oriented external
doors. It plans without changing source terrain or placements. The selector
compares a common level court first; complete cut/fill failure permits
comparison with separate floors and a level court joined by discrete stair
flights. Flight reach reserves separate floor and court landings and respects
both grade and discrete riser/going bounds. Complete source intersections
validate the selected solution before it can be installed. The production
dispatcher uses this selector and preserves unchanged vista samples. No missing
member or ambiguous door is substituted.

The retained `bevy_ahoy` controller corrects its downward stair-support query
within the existing contact envelope. Its platform velocity uses rigid-body
linear and angular displacement directly, keeping stationary terrain exactly
stationary at geographic offsets. The production movement regressions cover
an ordinary low step from rest, both directions of the proposed court stairs,
and rejection of excessive risers and insufficient headroom. Movement limits
remain unchanged. See [controller
provenance](../../vendor/bevy_ahoy/UPSTREAM.md). The geographic property
regression also uses the exact shared compiler for static building collision. It
traverses the street approach, court, both open doorways and return with the
terrain-following fixed enclosure and an open gate installed. The server gate
regression separately verifies the authoritative hinge and passage controller.
Operable closures remain separate dynamic bodies. These checks do not establish
surrounding-property or complete production-city access acceptance.

Support diagnostics intersect fixed collision cuboids with architectural Y=0;
they distinguish buried solids, ground contact and upper projections. Gate posts
use the same dimensions as runtime enclosures. The diagnostic compares actual
contact polygons with the fixed post, since distance to the hinge line does not
measure the available support corridor. A contact envelope alone is not proof
that every point inside it is a structural bearing.

## Single properties and street approaches

Single properties retain their nominal packing reservation separately from the
measured ground-bearing footprint. Fixed collision solids contribute their
actual cross-sections at the architectural floor datum, including buried slabs
that meet that datum. A convex envelope fills the occupied floor between those
contacts. The rectangular bounds remain a broad phase; their empty corners do
not acquire support ownership. Tilted members contribute their floor section,
not their upper projections. Frames may extend beyond nominal dimensions; only
the measured floor envelope and explicit doorway approaches receive support.
Composition rejects positive-area intersections between exact physical owner
outlines before clipping the natural source once. An overlap diagnostic retains
both property identities, member IDs, broad bounds, location and area. Courts
and gardens outside those regions retain their geographic surface.

The single-property selector intersects the complete bearing's cut/fill bounds
with the attainable floor intervals of every architectural ground entrance. The
selected floor therefore serves all approaches. Existing private court run can
contribute to the approach; the external apron remains bounded by its supplied
maximum. Contact-sized separation at the exterior boundary prevents opposing
street approaches from overlapping through coordinate rounding.

Door approaches begin at the exact architectural threshold and overlap its floor
envelope. Their complete width must fit the union of the measured floor, private
reservation and declared near street regions. A clear approach can straddle
adjoining street segments. Coverage is proved against their union by exact
convex subtraction, including the entire approach width. A real gap remains
rejected. Contact tolerance classifies numerical slivers; it does not create a
new street or merge property elevations. Compound selection prefers its near
street half when support and access fit. Steep cases may use more of their
supplied external reservation, and final composition still rejects any conflict
with another owner.

Ground entrances retain exact opening or authored workplace-passage identities.
A passage purpose distinguishes architectural ground circulation, outdoor
routes, upper circulation and service clearances. An open workshop does not need
an operable door leaf to have a ground entrance; an upper route or kiln flue
does not become a ground doorway. Natural-terrain outdoor routes require
separate validation against the final surface. They do not establish a common
architectural floor or prove usable access merely by carrying an identity.

## Terrain source and representation boundaries

A support calculation must identify the triangles it uses. Nominal source
resolution does not make a vista grid interchangeable with tactical ground. The
imported sampler preserves peaks for distant grids; tactical samples retain
local source elevations and procedural detail. Tactical preparation also repairs
steep samples and adds finer surface detail. The playable heightfield and vista
mesh use different subdivision diagonals. Equal corner samples alone therefore
do not prove equal surfaces inside a footprint.

`GeographicSurface::from_presented_scene` retains the exact fine playable
triangles and appends the existing vista rings. Ring clipping, boundary
subdivision, height morphs and playable-edge stitching use the shared
`vista_surface` policy. Each ring excludes the preceding rectangle; fine ground
is not resampled onto a distant grid. Offset, malformed and nonexpanding rings
reject rather than selecting a different source. Owned grading must not be fed
back as ungraded source terrain.

Playable heightmap repair constrains the gradient of each triangle used by
rendering and collision. Separate cardinal-edge limits are insufficient: two
axes at grade 0.65 produce a diagonal face above the character motor's walkable
slope. The shared `scene::grade` calculation first bounds source-cell jumps,
then projects each three-height triangle onto the permitted gradient disk.
Each projection preserves its mean and minimizes the local height correction;
this is not a claim of a globally optimal terrain solution. Already valid
heightmaps retain their original bits within their floating-point roundoff.
Alternating deterministic sweeps have a finite work limit. Failure returns the
sample coordinate, measured grade, permitted grade and sweep count, and leaves
input samples unchanged. The correction applies only to the prepared playable
sample domain; it does not resample or level the surrounding vista rings.
Source digests bind the resulting surface before owned property grading.

`GeographicSurface::compare_in_outline` overlays both triangulations and the
complete convex bearing footprint. It reports signed elevation extrema at all
intersection controls, together with required and covered area. Missing coverage
is not agreement. Callers must check coverage as well as their explicit
elevation tolerance; point samples and rectangular corner checks cannot
establish complete support.

The `compare-city-terrain` diagnostic checks unchanged production building IDs
against their recorded terrain-stage provenance. It compares raw playable
samples, fine terrain prepared with complete scene context, near vista triangles
and installed bounded support. An optional prepared-terrain capture can be
passed to `validate-city-support` with `--prepared-terrain` for a bounded source
comparison. It is ignored evidence, not a second production scene format.
Complete source preparation retains all original assets and reservations;
triangle comparisons do not certify actor clearance, gates or rendering.

Boundary support searches use the same exact half-plane interval predicate after
a conservative source query. Its broad-phase margin accounts for the incenter
and inradius of every source triangle: offset half-planes extend acute corners
farther than the contact distance. Canonical candidate order preserves the
original boundary stations and foundation geometry. The index does not alter
grading limits, source topology or generated product identity.

## Atomic producer handoff

`SelectedCityGrounding::select` first validates the complete scene roster. Every
building has one exact compound or single-property support owner. Missing
members, duplicate physical IDs, duplicate owners and unbound buildings are
rejected before floor selection. Diagnostics use canonical owner order.
Selection retains its input layout and geographic source as immutable snapshots,
including the accepted programme memo and complete property reservations.

`SelectedCityGrounding::compile` composes all owners before applying any
selected floor. Successful compilation returns a `GroundedCitySceneLayout`
containing the seated placements, accepted support plans and compiled terrain.
Both playable and distant placements receive their exact member floor.
Programmes, horizontal transforms, property membership, business bindings,
streets and gardens remain unchanged. A failed composition returns its exact
diagnostic and publishes no partially seated layout. Installing these
projections together is the producer's responsibility; enclosure, garden, final
route and rendering acceptance remain separate checks.

The `validate-city-support` example exercises this handoff against fixed
imported inputs. `--grounded-placements-output` writes a typed diagnostic
snapshot with exact numeric representations, and `--surface-output` writes the
corresponding expanded topology. The example reports support-plan and
expanded-topology byte counts separately. These are local inspection products,
not a scene-input format or evidence of complete renderer/server acceptance.
Production dispatch and decoding require the corresponding compact accepted
projection directly; there is no rectangular levelling or unbound occupied-scene
fallback.

## Accepted enclosure geometry

`GeneratedBoundary::project` binds the enclosure to its exact accepted property
foundation and both member identities. It rejects missing or different owners;
no front-house floor or neighbouring terrace supplies an enclosure datum. The
gate centre must have one support elevation within the contact bound. Missing or
ambiguous gate support rejects the complete projection with the exact property,
member IDs, element, location, measurement, permitted bound and shortfall.

Every nominal wall, cap and gate-post footprint is intersected with all of that
owner's physical support triangles. Coverage is checked over each complete
footprint, including triangle intersections. Wall bodies and caps follow the
accepted terraces. Wall bases embed below their local soil by the architectural
foundation allowance; cap overhangs retain their nominal dimensions. Gate-post
heads stay at the gate landing's architectural height. Where a post crosses a
step, its masonry plinth reaches the lower supporting soil. This projection
changes no grading region, natural triangle, gate width, hinge, opening
identity, building programme or horizontal enclosure geometry.

The resulting closed finite cells carry the exact vertices shared by collision
and rendering. `SceneBoundary` carries those cells, and its parent elevation is
the accepted gate datum. The existing spatial and material batches retain fixed
walls and caps together. Playable, distant and capture views use the same shared
projection. A distant scene waits for terrain with the exact scene digest;
missing terrain never becomes a guessed enclosure elevation. The projection is
prepared once per scene, without facade generation or a mutable terrain cache.

The enclosure regressions cover the hinge post across two terrace levels,
complete wall support, unchanged owner identities, deterministic source order
and explicit rejection of missing wall bearings. The controller regressions
include the fixed walls, posts and open gate in both directions of geographic
entry and court traversal. Complete gate sweeps, all city routes and ordinary
renderer captures remain separate acceptance checks. These local masonry and
contact values are engineering choices, not historical measurements or a claim
of structural retaining-wall strength.

Street and yard material overlays retain double precision through street,
cultivated-bed and traffic-tile intersections. Their vertices are quantized only
when the completed polygon becomes renderer geometry; its height is evaluated on
the same physical source plane at the represented horizontal coordinates.
Repeated rounding at intermediate cuts can move a source edge across a retaining
boundary and assign the neighboring floor even when the coordinate error is much
smaller than the contact tolerance. Material lifts keep their existing rendering
purpose and do not change physical terrain or actor support.

## Support calculation cost

Geographic triangles and declared grading outlines have immutable planar indexes
during support preparation. A query conservatively retains every intersecting
bound, including touching edges, then supplies the original canonical triangle
or outline order to exact clipping. The index changes neither the geographic
grid nor the support geometry, floor selection, coverage checks or rejection
limits. It is transient preparation data; it adds no serialized fields or
generated-product identities. Unrelated source triangles remain unchanged.
Runtime surface queries retain their separate index over the final natural and
foundation geometry.

Authoritative fine-surface refinement retains a job-local memo table for exact
seeded noise-lattice controls. The table holds at most 65,536 controls; clearing
it recomputes the original draws. Seed framing, draw conversion, interpolation
and accumulation order stay unchanged. The table expires after refinement and
adds no serialized data. Executable revisions continue to identify immutable
browser products independently of their byte-identical physical content.

## Courtyard evidence and site suitability

The 1901 provincial inventory, [*Stadt Goslar*, pp. 327 and
330–331](https://digital.ub.uni-paderborn.de/ihd/content/titleinfo/8171462),
documents Frankenberger Straße 11: a courtyard passage, raised rooms reached by
short stairs and a cellar entrance. The plan and description support considering
discrete levels instead of treating every court as an inclined plane. The
inventory also identifies later alterations and uncertain reused fabric. It does
not supply a calibrated 1544 courtyard level or stair geometry.

The [Denkmalatlas inner-wall
record](https://denkmalatlas.niedersachsen.de/viewer/metadata/41207273/1/-/)
dates the surviving wall south of the Kaiserpfalz to approximately 1400–1450.
Its current mapped geometry is useful for checking whether a procedural
reservation crosses a documented boundary. It is not a surveyed 1544 parcel
plan. Nearby [Liebfrauenberg
5](https://denkmalatlas.niedersachsen.de/viewer/metadata/36533614/1/-/) is a
later house with a stone base and yard access; [Liebfrauenberg
5a](https://denkmalatlas.niedersachsen.de/viewer/metadata/36569828/1/-/)
incorporates older wall and tower fabric in seventeenth-century construction.
These records cannot be assigned to a generated property merely because it is
near their modern addresses.

Retain the exact generated reservation when testing support. Record a
questionable historical site separately; do not move, discard or relabel a
required fixture to obtain acceptance. Terrain-aware siting, regional foundation
practice, resource costs and successive alterations remain separate generation
requirements. The current access segments specify pedestrian reservations; they
do not define a cart's dimensions, loads or traversable grades.

Run the native catalogue/partition checks and layout tests with:

```sh
cargo test -p adventuresim-tactical-server-dispatcher --lib
cargo test -p adventuresim-tactical-core --lib city_layout
cargo test -p adventuresim-tactical-core --lib scene_input
```

Record matched images, frontage coverage, developed footprint and cold/warm
preparation separately in ignored `target/` output. A smaller footprint alone is
not historical or visual acceptance. The retired strategic city interface is not
needed for generation, catalogue registration or these checks.

Rejected support records the attempted single-building or compound treatment,
not only the failed measurement. A rejected required positive fixture remains
failed; typed diagnostics are acceptance only for intentionally invalid cases.

The frozen neighboring-property fixture reproduces compound 519 and home 520
from population 6,500, seed 42. Their rectangular envelopes overlap, while their
actual floor contacts and convex floor envelope are disjoint. The regression
checks complete bearing support, unchanged placement and programmes, stable
member identity, iteration independence and preserved terrain at the formerly
owned empty corner. It does not prove full-city presentation or access.

Accepted terrain vertex and index arrays use the shared bulk geometry transport
in binary worker and replication products. Human-readable scene and diagnostic
documents retain their array representation. Binary foundations encode each
six-vertex prism once and restore its declared solid and bearing indices
exactly. Encoding rejects incomplete cells or indices that differ from that
topology; scalar bits, property identity, members and cut faces remain intact.
Query indexes stay serialized, so warm reuse does not rebuild their acceleration
structures. Executable revisions identify immutable worker-cache products, so a
changed encoding cannot read an older cached product.

Browser persistence starts after generated assets become ready. Its deferred
queue takes ownership of worker buffers only after Wasm receives them. A
rejected queue insertion retains caller ownership. Pending backing buffers and
UTF-16 job text are bounded together to 512 MiB, separately from the stored
compressed product budget. At most 512 pending records and the existing 128 MiB
product limit bound individual jobs. Two write lanes perform compression and
storage; cache settlement has its own measured duration and does not gate asset
readiness. Replacing or removing a pending job preserves its final intended
bytes. The storage format, executable identity, checksum checks and
optional-storage failure behavior remain unchanged. Warm reuse is measured after
cache settlement; resident facades and the bounded scene retention window remain
independent.

Street clipping consumes the accepted triangle stream directly, applying the
same upward-face eligibility test as its mesh adapter. It avoids constructing
renderer normals, UVs and indices merely to discard buried and vertical faces.
The full render and collision geometry remain unchanged.

Prepared scene assets use validated input digests within the same bounded
three-scene retention window as landscape products. Preparing another view
reuses resident immutable scene data. Installation receives a separate generated
scene before venue promotion and interior furnishing; changed input, eviction or
explicit residency clearing requests preparation again. Actor positions, damage,
HP and enemies remain in the transient Bevy simulation.

Physical outdoor scenery must leave accepted support and access clear. After
source-bound support reconstruction, generation excludes obstacle envelopes
intersecting its exact outlines with the existing pedestrian cylinder and
contact skin clearance. This includes approach endpoint caps; checking only a
scenery centre or the bare support polygon is insufficient. Exclusion does not
enlarge grading, resample geographic terrain or choose new floors. Furniture
preparation consumes the remaining physical obstacles. Distant furniture remains
non-physical. The `kassel-1-access-obstacle.json` regression records the
imported counterexample.

Furniture reservations consume the validated single-property doorway approach
regions, including workplace passages that have no door assembly. The accepted
region can include a property setback as well as its bounded street apron. Its
existing furniture shoulder clearance protects the complete approach before
candidate placement; the reservation does not extend grading or change floors.
Accepted surfaces reject unselected treatments and member-arity mismatches.
The `carpenter-passage-access.json` regression records the missing exterior
reservation and retains its exact property, member and programme bindings.

Ordinary rendering omits exactly coincident, oppositely oriented interior side
quads between foundation cells belonging to the same property. Vertex matching
uses their bits, without a welding tolerance. Different owners, stepped levels,
unmatched faces and ambiguous duplicates remain visible. Closed collision
cells, bearing triangles, buried bottoms, source triangles and exposed retaining
faces remain unchanged. Collision diagnostics retain the complete solids.

Vista grass in an occupied scene queries the accepted physical surface directly,
using the same slope limit as playable grass. It cannot restitch raw vista
height samples over a graded foundation or over the already stitched natural
surface. A missing physical surface rejects the tuft. Unoccupied sampled scenes
continue to use their presented heightfield. This changes generated grass
products; the executable revision invalidates their immutable cache identity.

Ownership validation indexes the complete declared clipping outlines before
checking exact intersections. The bounds include entry aprons beyond the plot;
plot-only bounds could miss conflicting access surfaces. Candidate owners retain
canonical order, so the first conflict and its exact diagnostic remain stable.
The index neither joins properties nor changes grading regions or geometry.
Runtime support queries reuse logical implications between nested ownership and
contact margins; their tolerances, candidates and shared-edge order are
unchanged.

Scene worker transfer carries exact occupied placements and the remaining scene
assets. It omits occupied construction plans and collision recipes that venue
jobs have already prepared. The receiver requires the unchanged placement list
and each complete programme's prepared recipe before reconstructing the scene;
it rejects missing recipes, changed bindings and embedded duplicate geometry.
The reconstructed scene retains exact plans, collision, support, furniture and
physical identity. Its worker format changes directly, with executable revisions
identifying new immutable cache products.

The production capture viewer checks declared vista levels and the world-space
extent of their rendered chunk bounds. It respects the configured LOD cap and
rejects missing, extra, moved or truncated rings. Synthetic terrain fixtures can
declare shorter vistas than imported geographic scenes. A universal ring count
or fifty-kilometre requirement would validate a fixture assumption rather than
the declared scene. Per-view capture records report physical pixel dimensions;
the window's logical dimensions can differ when the host display is scaled.

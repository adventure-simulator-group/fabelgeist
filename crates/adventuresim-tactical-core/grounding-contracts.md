# Geographic grounding contracts

Core owns the conversion from admitted architectural geometry to static scene
support. Strategic property ownership, household membership and business
operation remain separate from the identity of a physical scene building.
Tactical movement, health and damage remain transient server state.

`SceneBuildingId` follows a physical building through placement, property
membership, support diagnostics and dispatcher projections. `PropertyMembers`
rejects repeated identities and retains the producer's order. Local
`CityPropertyId`, persistent residence identity, business and operator identity,
`BuildingEntranceId` and opening identity keep separate owners.

`CityPlotBounds` and `CityAccessSegment` accept checked scene points,
dimensions, orientation and widths. Their fields are private; decoding uses the
same construction paths. Read-only native getters serve the polygon clipping and
packing kernels in scene east/north metres. Checked edits reconstruct the record
before replacing it.

## Support admission and queries

`SupportLimits`, `CourtStairLimits` and `FoundationEmbedment` admit their
engineering leaves before use. JSON and postcard decoding use the same
constructors. Reflection treats these owners as opaque. Existing grade,
cut/fill, contact and stair limits remain unchanged.

`SupportQuery` contains a finite `ScenePlanPoint` and either
`SupportCeiling::Bounded(SupportElevation)` or `SupportCeiling::Unbounded`. An
unbounded query requires no infinite position. Bounded queries preserve contact
tolerances and retaining-edge selection, including negative support elevations.
Architectural, collision-centre, floor, ground, plot and gate conversions use
the shared geometry leaves and explicit core datums. Enclosure support vertices
use `Position<GateRelative>`.

Foundation and boundary prism admission checks finite vertices, paired plan
coordinates, ordered top/base heights, topology and indices. Query indices also
check their bounds and source references. Immutable admitted buffers provide
rendering and physics ports. Exact contact degeneracies and represented thin
prisms do not acquire a new epsilon. A cell without plan area remains admitted
contact geometry without a query bearing or solid collider. The existing query
area threshold still governs bearings. Property collider failures retain the
property and members; cell failures also identify the offending cell.

An enclosure binds its exact property, ordered front/rear members, descriptor
geometry and gate elevation. Decoding rejects a different front role, descriptor
member list or gate datum before rendering or collision. Garden root admission
similarly requires exact plant membership in descriptor order.

## Diagnostics and deterministic producers

Support violations carry distinct metre, square-metre or count measurements.
Minimum deficits are `max(required - actual, 0)`; maximum excesses are
`max(actual - permitted, 0)`; exact violations report absolute discrepancy.
Missing required entrances report one missing binding. Duplicate entrances
retain the offending `BuildingEntranceId`. Access-grade controls report rise and
permitted rise in metres. Error locations distinguish checked scene points from
rejected numerical attempts, which cannot become geometry.

Geographic geometry, occupied placement bindings and imported source packages
use separate digest owners. Each retains its established hash inputs and
framing. Scene, terrain and noise producers carry canonical `Seed`; native
lattice coordinates, interpolation, little-endian framing and cache eviction
retain their existing arithmetic. Source and placement mismatch errors retain
all affected property, member and location identities.

Scene schema 28 requires exact floor-bearing and ordered property binding
records. Compound floor bindings identify the authored constant-floor cell
partitions; approaches and stair flights retain independent levels. Internal
partition edges do not expand the contact allowance. Contact bounds are
envelopes rather than complete structural bearing polygons. Single buildings
retain their exact architectural bearing footprints. Access passages retain
their independent levels. Admission checks affine heights and polygon-union
coverage with the existing contact allowance. Placement floor datums must match
their bindings exactly. The generation version is 71. Existing numeric
representations stay unchanged. Transient browser products use the executable
and cache-format digest as their existing cache authority. A different
executable receives a different product cache; no compatibility decoding path is
provided.

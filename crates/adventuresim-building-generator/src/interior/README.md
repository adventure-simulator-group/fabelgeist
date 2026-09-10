# Interior furnishing rules

The solver consumes accepted room cells, authoritative wall collision and openings,
existing process equipment, and stair landings. Furniture footprints and usable
faces come from the mesh-independent `interior_spec` catalogue. Models are never
compiled to decide whether a layout works.

Counts are area-based targets, not promises to fill every available square metre.
Each rejected target is reported in `unmet_budgets`. A building with no furniture,
a disconnected room, or an inaccessible stair is an error. An accepted placement
must preserve access to every earlier placement and every room.

| Room | Target budget and placement |
| --- | --- |
| Bedroom | One bed per 10 m², up to six; one chest per 18 m², up to three; perimeter positions |
| Kitchen | One workbench per 15 m², up to two; cupboards and one kneading trough against walls |
| Pantry | One shelf per 8 m², up to six, plus chests |
| Storage | One trade-specific container per 8 m², up to twelve; shallow rows with accessible fronts; perimeter shelving |
| Shop | One three-module counter run per 50 m², up to two; shelves on walls and up to two freestanding display counters |
| Workshop / kiln / vat / milling floor | One trade-specific workstation per 12 m², up to eight; one supporting rack per 20 m², up to four |
| Stable stalls | Hay racks per 14 m², up to eight; troughs per 18 m², up to six |
| Hospital ward | One bed per 12 m², up to sixteen, with both long sides accessible; washstands and cupboards |
| Schoolroom | Teacher's desk with chair; table and two-bench groups per 12 m² |
| Counting room | Desk and chair groups per 18 m², up to five; cabinets per 22 m² |
| Guardroom / armoury / tower chamber | Trade workstations, weapon racks and chests |
| Nave / chapel | Church benches per 7 m², up to 48; one lectern |
| Chancel | One altar and one cupboard; synagogue programmes use a lectern |
| Sacristy | Cupboards and one desk |
| Hall / common room | Table and two-bench groups per 18 m², cupboards; inns also request a counter run; bathhouses request tubs |
| Entrance / passage / stair hall / gallery | No furnishing; circulation space remains available |

All 66 building uses select an explicit trade kit:

| Building uses | Primary furniture / supporting furniture / storage |
| --- | --- |
| Dwelling, inn, rectory, manor, executioner's house | Dining table / cupboard / chest |
| Parish church, cathedral, chapel, monastery, synagogue | Church bench / lectern / cupboard |
| General shop, market hall, herbalist, apothecary, bookshop, fishmonger | Workbench / shelving / crate |
| Smithy, smelter, assay house | Workbench / tool rack / crate |
| Weaponsmith | Workbench / weapon rack / crate |
| Armorer | Workbench / armour stand / crate |
| Tailor, weaver, printing house, paper mill | Cutting table / shelving / crate |
| Bakehouse | Kneading trough / workbench / grain bin |
| Brewery, malthouse | Cask rack / workbench / grain bin |
| Butcher | Butcher's block / workbench / crate |
| Stable, barn | Feed trough / hay rack / grain bin |
| Granary, horse mill, water mill, windmill | Grain bin / workbench / grain bin |
| Cooper, carpenter, wheelwright, cobbler, stonecutter, sawmill | Workbench / tool rack / crate |
| Tannery, dyer, fulling mill, ropemaker | Drying rack / workbench / crate |
| Chandler, potter, salt works, brickworks, glassworks | Workbench / drying rack / crate |
| Timber yard, warehouse, woad store | Workbench / shelving / crate |
| Town hall, weigh house, guildhall, mint, customs house | Writing desk / cupboard / chest |
| Hospital | Ward bed / washstand / cupboard |
| Bathhouse | Tub / bench / chest |
| School, university | Writing desk / bench / shelving |
| Guardhouse, prison, castle, arsenal | Bunk bed / armour stand / weapon rack |

Counter runs are accepted atomically: matching left end, middle and right
end modules sit exactly one module width apart. Both customer and staff faces
must remain accessible. Their clear strips can overlap other circulation space,
but cannot contain other furniture. Tables with two benches and desks with chairs
are also accepted as whole groups. Church seating faces the chancel. Hospital ward beds share the room axis,
with opposing headings towards the side walls rather than independent rotations. The building
seed selects a compact or broad preference; broad groups can fall back to compact
dimensions when the larger furniture cannot fit.

The 34 model families are dining table, bench, chair, stool, bed, bunk bed,
storage chest, cupboard, shelving, writing desk, lectern, church bench, altar,
ward bed, bath tub, washstand, workbench, cutting table, tool rack, weapon rack,
armour stand, grain bin, storage crate, counter middle, counter left end,
counter right end, counter corner, display counter, drying rack, kneading trough,
butcher's block, cask rack, hay rack, and feed trough. Stool and counter corner
are available model families; generated compositions currently use benches,
chairs, and straight counter runs.

Access uses a 25 cm cardinal lattice and a swept 60 cm square person footprint.
This is the standard routing clearance, not a promise for every character size.
Exact doorway and stair landing points join the lattice through checked cardinal
segments so narrow valid approaches do not depend on lattice alignment.
Every edge checks its complete swept rectangle, preventing diagonal corner cuts
and passage through thin walls between sample points. Each required face gets a
path from the same ground-floor entrance. Layout validation rebuilds those paths
from placements and architecture rather than trusting serialized proof paths.
Door reservations contain the leaf swing and approach on both sides. Existing
workplace passages and stair footprints also remain empty of furniture.
The entrance threshold is checked against architectural collision before joining
the supported inside approach. Furniture must have floor support across its
footprint and clear architectural solids over its full height, including tall
cupboards above the routing agent's head. Mesh origins use the actual floor top.

Timber upper floors reach the end of their stair flight, and jetty beams bear
below the floor deck through a rim sill and post continuations. Spiral circulation
uses resolved landing slabs and occupied floors; the well blocks planar travel.
Keep room allocation reserves the actual spiral well and landing approaches as a
stair hall. Cathedral rooms and continuous paving follow the resolved church
envelope, including the west entrance, transept, choir, and apse. The paving
finishes at the doorway sill datum, with its slab below the occupied floor.
Hall-house frames use high knee braces to keep the central aisle open; their
reach stays above any partition top plate crossed by the bay.
Post rows and cross-ties share bay stations that avoid door approaches. Floor
support follows each slab's actual rotation, and tilted architectural members are
clipped to the relevant vertical interval before checking furniture or headroom.

Physical floor slabs are required on every occupied level, including the ground
floor. Room cells alone never provide support. Courtyard-castle and gatehouse
prototype interiors remain unsupported until their occupied decks are authored;
the solver rejects these missing-floor plans instead of placing floating objects.

Inn counters and complete table-and-two-bench groups retain their full customer,
staff, and seating access. A narrow ground common room may accommodate either
ensemble but cannot necessarily accommodate both; its unmet dining budget remains
explicit, and a wider upper common room can provide the dining and service area.

# Museum weapon studies

These recipes distinguish photographed construction from inferred dimensions.
Select a study in the weapon modeler's preset selector. Its controls edit the
same canonical recipe used by native generation and GLB export. Studies are
separate from the general authoring presets and gameplay chassis registry.

## Cleveland 1916.1589 seven-flanged mace

Select `cma-1916-1589` or export it with the standard command below. The
[recipe](cma-1916.1589.json) studies the complete structural silhouette of
[Cleveland's seven-flanged mace](https://www.clevelandart.org/art/1916.1589),
dated about 1540–50. It covers the compact `flanged-mace` browser family and
`flanged_mace` gameplay family. The elongated `gothic-flanged-mace` has a
separate KHM A 297 study below. Acceptance does not resize either ordinary
preset to this specimen.

The museum publishes an overall length of 645 mm, mass of 1.6 kg, and
"Head: 11.4 cm" without specifying the measurement axis. The
[full photograph](https://openaccess-cdn.clevelandart.org/1916.1589/1916.1589_print.jpg)
and [alternate photograph](https://openaccess-cdn.clevelandart.org/1916.1589/1916.1589_alt0_print.jpg)
are CC0, courtesy of the Cleveland Museum of Art; Gift of Mr. and Mrs. John L.
Severance. They show essentially the same pose, not independent reverse views.

Photographic proportions support a roughly 137 mm flange body, 44 mm breadth
at its ends, and 114 mm maximum cusp diameter. That diameter's agreement with
the ambiguous museum head measurement does not establish its axis. The axial
intervals are 18 mm butt, 180 mm grip, 14 mm grip band, 267 mm exposed haft,
9 mm head band, 137 mm flange body and 20 mm crown, totaling 645 mm. Seven
equivalent concave flanges have cusps above their midpoints. A tapered faceted
haft connects the long grip to the narrow head core.

Flange thickness of 2.5 mm, a 9 mm core circumradius, the regular seven-sided
core and solid steel internal construction are assumptions. The resulting
calculated mass is approximately 2.143 kg, separately from the museum's 1.6 kg.
Density and unmeasured thicknesses are not adjusted to force agreement. Hidden
voids, internal fasteners and unpictured reverse details remain unknown.
Structural scope includes flange contours and count, shaft/grip proportions,
transition bands and terminal finials. Chasing, decorative butt fluting,
gilding, patina and other surface decoration are not reproduced.

### Flange receiving faces

The shared mace core is an explicitly faceted lathe. Its polygon side count
defaults to the flange count; an authored `segments` value must be a multiple
of that count. Those receiving flats retain their phase and shape at every
LOD. `coreProfile` supplies axial height and circumradius stations, with
strictly increasing heights. Its endpoints cover the complete head and crown
interval. Without that profile, the core retains the standard tapered law.

Each flange's inner contour follows the actual core-face apothem at all
profile stations. The entire finite plate thickness must fit on that face.
The inner and outer contour station union proves separation between their
piecewise-linear boundaries; an oversized plate or a protruding core is
rejected. Core and flange solids share receiving surfaces without overlapping
material volumes. `rootRadius` and `shoulderRadius` remain the outside flange
landmarks. The arbitrary `flangeRootScale` parameter is removed.

Ordinary mace mass and inertia consequently change: their previous core/plate
overlap was counted as material twice. The gameplay mace uses six core faces
so its authored upper flange contour retains a positive plate width. The
museum controls expose flange count, thickness, outer radii, cusp height,
concavity, core radius, grip length and exposed haft length. Editing axial head
length in the recipe also requires updating its absolute core-profile heights.

From the repository root, export the study with the standard modeler workflow:

```sh
npm --prefix tools/weapon-modeler run export -- --preset met-14-25-394 --lod high --skinned ../../target/met-14.25.394.glb
```

## Met 14.25.394 spear

[The Metropolitan Museum of Art's object record](https://www.metmuseum.org/art/collection/search/26796)
identifies a Western European spear from the fifteenth century. It reports an
overall length of 233 cm, head length of 50.2 cm, width of 9.5 cm, and whole-object
mass of 2,579.8 g. The recipe is in [met-14.25.394.json](met-14.25.394.json).

Reference photographs: [first face](https://collectionapi.metmuseum.org/api/collection/v1/iiif/26796/1425684/main-image)
and [alternate face](https://collectionapi.metmuseum.org/api/collection/v1/iiif/26796/1425711/main-image).
Both images are Public Domain, courtesy of The Metropolitan Museum of Art.

The two published head photographs support a long pointed blade, a low rounded
belly, a central ridge, a flared socket, and bilateral stops with squared ends
and curved roots. The reported width is interpreted as the stops' outside span.
Scaling the photographs against the reported head length suggests a blade
breadth of 45–52 mm, neck breadth of 28–34 mm, and socket rim breadth of 41–47 mm.
The stop centers lie approximately 64–77 mm above the lower metal rim.

The full shaft and butt are outside these photographs. Their dimensions, the
blade depth, receiving tenon, socket wall and blind cavity are manufacturing
assumptions. The complete recipe uses the reported overall length, but its
integrated material mass is not adjusted to match the museum's whole-object
mass. Calculated and reported masses must be presented separately.

## Socketed leaf construction

`spear.length` measures only the blade, from local `y=0` to the point. An optional
`socket` extends below the blade to `y=-socket.length`. Its `neckLength` occupies
the upper portion of that socket interval, where the circular exterior changes
continuously into the blade section. Named `bladeBase`, `socketRim`, and `tip`
frames identify these landmarks. Generic `base` is the lower socket rim.
The neck uses convex round-to-diamond sections, preserving the central ridge.
Tangent combinations that collapse a section axis are rejected before sampling.

`baseRadius` is the authored exterior radius. `wall` determines the bore radius
at the open rim. `boreTipRadius` and `cavityDepth` define the blind tapered bore;
the cavity terminates below the neck. `insertionDepth` measures the actual
penetration of the receiving shaft independently of the cavity depth. A
`shaft-top` mount places the rim that distance below the shaft's upper end.
The complete inserted shaft profile must clear the coarsest rendered bore.
An optional shaft `tenon` turns down the final portion of its total length.

`shoulderRoundness` interpolates from the angular leaf to smooth shoulder
tangents, retaining the root width, maximum width, belly station and point.
The socket's optional `stops` are part of the same exterior solid. Their
`centerHeight` is measured from the socket rim. `span` measures both outer ends,
`endHeight` sets the square ends, and `rootHeight` sets the underside's vertical
extent. `orientation` rotates the stops around the socket axis relative to the
blade width plane. Omitting `stops` produces an uninterrupted socket.
`rootBlend` supplies a local fillet at the plate/socket junction while retaining
the squared end planes. Curve sampling follows chord and deviation budgets at
every display level. Small fillets refine their local angular neighborhood;
features below the selected detail's error budget approach the sharp junction.

Shaft `wrappings` are flat spiral strips with an axial start, total length,
pitch, width, thickness, phase, material and handedness. The crossed pattern
raises one strip over the other locally; each otherwise follows the same
tapered polygonal surface as the rendered shaft. Wrapping must stay outside a
receiving socket and the turned tenon. The study's wrapping dimensions and
leather thickness are inferred from the visible crossed bands.
The optional `underlay` is a separate hollow covering whose thickness and
material determine the strips' receiving surface. In this study it extends to
the metal rim. The upper strip ends meet that rim; the lower ends lie against
the covering. Stitching or other small fastening details are not represented.

## Cleveland 1921.1253 hand-and-a-half sword

[The Cleveland Museum of Art's object record](https://www.clevelandart.org/art/1921.1253)
identifies a South German sword of about 1500. It reports an overall length of
117.5 cm, blade length of 90.2 cm, quillon span of 26.4 cm, grip length of 21 cm,
and mass of 1.34 kg. The recipe is [cma-1921.1253.json](cma-1921.1253.json).

Reference photographs: [complete sword](https://openaccess-cdn.clevelandart.org/1921.1253/1921.1253_print.jpg)
and [hilt detail](https://openaccess-cdn.clevelandart.org/1921.1253/1921.1253_alt0_print.jpg).
The images are CC0 under the museum's [Open Access policy](https://www.clevelandart.org/open-access).
Credit: Gift of Mr. and Mrs. John L. Severance.

The photographs support a broad, slowly tapering blade with a short rounded
point, a fuller ending near the blade's middle, a waisted grip with two
swellings, flat quillons, stepped terminals and a spirally fluted pommel.
Blade width, fuller width and termination, grip widths and pommel proportions
are estimates from the photographs. Fuller depth, distal thickness, grip
depth, wooden core and cover thickness are manufacturing assumptions. A
matching reverse fuller is an explicit assumption because the sources do not
establish both broad faces independently.

The complete blade starts at the guard's forward face. The recipe uses a
50 mm pommel interval, 210 mm grip, 13 mm central guard interval and 902 mm
blade, totaling 1,175 mm. The guard's 264 mm measurement includes both
terminal extensions; its centerline span is smaller. These assembly planes
are authored explicitly. Calculated material mass is reported separately
from the museum's mass; dimensions and density are not adjusted to force a
mass match.

Export this study through the same command, selecting `cma-1921-1253`.

## Recessed blade, point and grip controls

Both `sectionBlade` and `loftedBlade` use one transverse section and closed
loft implementation. `fullered` describes the asymmetric swept V-floor
section with a ridged reverse. `recessed` requires an explicit `fuller` and
uses beveled flat broad faces. Diamond, hexagonal and lenticular sections
have distinct geometry; a fuller specification is rejected for those modes.

Each `fuller.grooves` entry has `start`, `end`, `entryLength` and `exitLength`,
which are metres from the
complete blade base, including any ricasso. The interval lies beyond the
ricasso and ends no later than the blade tip. The museum recipe retains a distal
ungrooved face.
Both transition lengths are positive and fit inside the interval. Mouth
width and recess depth are independent dimensions. The floor-width and
edge-bevel ratios lie strictly between zero and one. Front means local +Z;
`faces` selects front, back or both. Near each closure, depth vanishes faster
than mouth width so the groove meets the broad face with a vanishing slope.
The complete interval must retain positive metal and local face clearance.

Tessellation samples the complete analytic section, retaining all feature
planes. Its surface tolerance is 0.1 mm at Low, 0.04 mm at Medium and
0.016 mm at High, divided between chord approximation and groove-tail
simplification. A tail whose transverse strips fall below the float32
transport separation becomes a flat broad face through explicit closure
fans. This can shorten the rendered groove slightly; its authored endpoint
and continuous clearance semantics remain unchanged. Reduction must fit the
remaining position budget and an additional normal-angle budget of
0.05/0.02/0.008 radians at Low/Medium/High. The normal check includes axial
and transverse slopes and compares interior samples against the analytic
surface, accounting separately for the existing broad-face chord error.
Unresolvable features that exceed these budgets are rejected. The transport
floor uses the supported 20 m assembly-frame scale; it does not guarantee
arbitrary additional world transforms.

The optional point `start` is normalized within the working blade after its
ricasso. `roundness=0` produces a pointed limiting curve; positive values
produce a rounded terminal tangent. Body taper controls the earlier blade.
The point joins a nonincreasing body width with the same slope, and its
thickness envelope preserves the incoming thickness slope. Blade length and
tip frame remain fixed. Local curvature sampling applies at every LOD.

`profileGrip` stations specify normalized axial position and complete visible
width/depth. Shape-preserving cubics pass through ordered stations without
overshooting their dimensions. Component material is the core material;
`cover.material` is independently authored. Cover thickness is a transverse
polygon-normal inset, not a constant three-dimensional offset on a sloping
grip. Core and cover share identical sampled interfaces and partition the
visible volume. The cover's end annuli leave the core faces exposed.
Authored grip ends are never resized by the oval-grip pommel-seat heuristic.
The sword study assumes a 10 mm depth at the guard-side grip end and uses
`guard.blockBevel=0` to provide a matching full-depth flat seating face.
The default beveled block has a smaller footprint at its lowest plane; its
nominal thickness alone does not describe that contact face.

A profile quillon terminal uses axial offset/radius stations from the actual
arm-end plane, with positive offsets along the outgoing tangent. Repeated
axial stations create annular steps; strictly increasing runs may use smooth
or linear interpolation. Its base covers the receiving arm section and
stays outside that plane. All guard parts inherit the component material.

## London Museum 80.157 rondel dagger

Select `london-80-157` in the modeler's museum studies or the export command.
The recipe is `london-80.157.json`.

- [Museum record](https://www.londonmuseum.org.uk/collections/v/object-29506/dagger-rondel-dagger/):
  fifteenth-century iron and wood dagger, 354 mm overall, 254 mm blade.
- [First face photograph](https://collections.londonmuseum.net/download/985/768/download_2024_10_04_15_36_0020.jpg)
  and [opposing face photograph](https://collections.londonmuseum.net/download/985/771/download_2024_10_04_15_36_0021.jpg):
  copyright London Museum, licensed CC BY-NC 4.0. Photographs are reference
  evidence, not included textures or redistributed repository assets.

The record labels its 30 mm width as overall. That conflicts with the photo
proportions: the rondels appear wider than the blade's roughly 30 mm heel.
The recipe therefore retains this ambiguity and treats its 44 mm guard,
46 mm pommel and 30 mm heel as photo estimates, not relabeled measurements.

The 100 mm hilt comprises a 14 mm pommel rondel, 76 mm exposed wood grip
and 10 mm guard rondel. Their contact planes meet without axial overlap.
Each rondel has a recessed circumferential band with explicit 1 mm bevel
shoulders. Solid steel approximates the recorded iron; hidden layering is
unknown. The plain oval wood grip has an assumed 22 mm end depth. The
asymmetric blade assumes a wedge section, 5 mm heel thickness and a 0.6 mm
body edge thickness. Neither thickness nor mass is published in the museum
record. Calculated mass is a model result, not a fitted historical target.

### Generic blade points and heel placement

The generic `blade` uses a single section loft for capped and pointed ends.
The following thickness law describes the default `edged` section.
Without `point`, `tipWidth` specifies the finite terminal width ratio and
the body retains its distal thickness. With `point`, `tipWidth` still
describes the underlying body law; `point.start` selects where the shared
point curve replaces that law. The start is normalized over the complete
generic blade, which has no ricasso. `point.roundness` ranges from a sharp
limiting curve at zero to a rounded terminal tangent at one.

The point joins a positive, nonincreasing body half-width with matching
slope. If `q` is the point half-width divided by its width at the join,
the envelope `q * (2 - q)` scales both ridge and edge thickness. This
preserves their incoming axial derivatives and closes the entire section
at one exact tip vertex. The terminal fan has no cap face or overlapping
pieces. The body thickness must remain at least its fixed edge thickness;
invalid thin or increasing-width joins are rejected, not clamped.

`singleEdge` ranges from -1 to 1. The endpoints place the full-thickness
spine at one outline edge; zero places the ridge centrally. A straight
spine remains straight through the point when curvature is zero and the
asymmetry is extreme. These are geometric modes, not historical claims
about the unmeasured reverse section.

The blade's local base and origin retain their original coordinate datum.
Use `attach.at: "heel-center"` to seat the actual heel center on a parent
frame. This anchor follows width and asymmetry, including rotated blades;
it is also exposed as the component's `heelCenter` frame. The study needs
no fixed lateral attachment offset, so ordinary editor changes preserve
heel centering. This anchor is rejected on shapes without that definition.

## KHM A 297 elongated Gothic mace

Select `khm-a297` or export it through the standard command. The separate
[recipe](khm-a297.json) covers the browser `gothic-flanged-mace` endpoint.
It does not add another gameplay family: `flanged_mace` already has the
compact Cleveland study.

The [museum record](https://www.khm.at/kunstwerke/streitkolben-372692)
identifies a German mace of about 1520, inventory A 297. It gives 535 mm
length, 75 mm width and 0.85 kg mass, with an iron head and shaft, wood grip
and possibly linen cord. Its [photograph](https://www.khm.at/pics/372692/HJRK_A_297_202303_1.jpg)
is credited to Kunsthistorisches Museum, Hofjagd- und Ruestkammer. The image
is private local reference evidence and is not redistributed in this project.
Only one photographed pose is available; reverse details and cross-sections
remain assumptions.

The visible structural study includes the stepped elongated flanges and
edge lugs, transverse band, open crenellated crown, twisted beveled haft,
broad grip guard, tightly wound cord with a coarser binding, exposed wood,
butt disc and loop. Fine chasing, engraved rope ticks, individual cord
fibres, patina and wear are outside the structural scope.

Photo estimates allocate 15 mm to the loop below its butt disc, 3 mm to
that disc, 105 mm to the grip, 6 mm to the guard, 208 mm to the exposed haft,
15 mm to the head collar, 166 mm to the flange interval and 17 mm to the
crown. The head band occupies 4 mm within the flange interval; the flange
solids end at its actual faces. The photographed cusp is about 0.76 of the
way up the complete flange interval. These subdivisions are estimates,
not additional museum measurements.

Six flanges and four crown teeth are hypotheses, not published counts.
A solid steel approximation, hidden wood core, flange and crown thickness,
haft section, compressed cord crests and internal band continuity are
explicit construction assumptions. The 16 mm beveled haft section has a
maximum breadth of about 17.9 mm. Its twist and the coarse binding's
handedness follow the visible photograph. Calculated High-detail mass is about
0.962 kg versus the museum's 0.85 kg; it is not calibrated to that value.

## Stepped flanges, notched rims and supported cord

A mace can specify `flangeProfile` stations with normalized axial `at` and
absolute `radius` in metres. Stations proceed from zero to one. Equal axial
positions form a radial step; every incident radius must remain outside the
receiving core face. The continuous cusped construction remains available
when the profile is omitted. An explicit profile controls the flange outline;
its length, core, flange count and thickness retain their usual meanings.

Separate flange bodies can terminate against the faces of an intervening
solid band. Each body and band occupies its own axial interval. A surrounding
ring must not be placed through an uninterrupted flange or counted twice in
material volume.

Hollow sockets accept `crenellations` with `count`, `depth` in metres and
`toothFraction`. Notches descend from the terminal plane while retaining a
positive closed lower ring. The mesh includes the bore, notch floors, side
walls and annular upper faces. Repeated notches do not cap the opening.

Swept members accept a `beveled` section: a rectangular section with each
corner cut back by one quarter of its width and depth. It retains eight
faces and has area 7/8 of the bounding rectangle. Twisted sweeps limit both
axial chord and corner travel between rings. Symmetric face subdivision
avoids the handedness-dependent material wedge of a fixed diagonal.
At Medium, the corner-travel budget is 0.6 mm, scaled with display detail;
rotation between rings never exceeds 15 degrees. These are sampling budgets,
not a claim that every point of the analytic surface is within that distance.

Shaft wrappings may use `section: {"kind": "rounded", "crestFraction": ...}`.
This models compressed cord with a finite flat underside, rounded shoulders
and a finite flat crest. The crest fraction lies strictly between zero and
one. Thickness follows the receiving face normal in each transverse section.
The original flat strip measures thickness radially. Both constructions use the
actual shaft facets and retain every angular face boundary.

A single-handed winding requires width smaller than pitch. Crossed strips
retain the stricter four-width spacing required by their crossing lifts.
An `onWrapping` index selects an earlier rounded, single-handed wrapping as
the receiving layer. Its crest supports the upper strip across the gaps;
there is no hidden continuous sleeve. Support cannot reference itself or a
later layer, extend outside the receiving interval, or have no finite crest
contact. Each layer contributes only its own material volume. Flattened
crest proportions, cord thickness and hidden core materials are construction
assumptions when the museum has not measured them.

## Met 29.158.674 Saxon war hammer

Select `met-29-158-674`. The [recipe](met-29.158.674.json) covers browser
`reiter-war-hammer` and gameplay `war_hammer`. Long-pole Lucerne hammers and
pollaxes remain separate coverage entries. Ordinary presets retain their
authored dimensions.

The [Met record](https://www.metmuseum.org/art/collection/search/34089)
identifies a Saxon war hammer of the mid-16th century, made of steel and
silver. Published measurements are 572 mm overall, 156 mm width and
1.134 kg. Its separate "L. of head 5 in. (12.7 cm)" has no specified axis;
the recipe does not reinterpret it as the complete horizontal span.
The opposing [front](https://images.metmuseum.org/CRDImages/aa/original/DP160169.jpg)
and [back](https://images.metmuseum.org/CRDImages/aa/original/DP160170.jpg)
photographs are public domain according to the
[collection API](https://collectionapi.metmuseum.org/public/collection/v1/objects/34089).
Credit: The Metropolitan Museum of Art, Bashford Dean Memorial Collection,
Funds from various donors, 1929.

The structural scope includes the square-section curved beak, baluster poll,
head button, slender steel shaft, one-sided belt hook, disk guard, collars,
dense silver-wire grip courses, domed silver pommel and bottom button.
The hook's blunt cap approximates the source's small rounded tongue.
Floral etching, border dots, microscopic wire ply, patina and wear are outside
scope. Surface color indicates material, not the photographed lighting or age.

Photo estimates allocate 10 mm to the bottom button, 26 mm to the pommel,
98 mm to the grip, 5 mm to guard and collar, 23 mm to the shaft ferrule,
380 mm to the exposed shaft, 20 mm to the head block and 10 mm to its button.
The poll projects 34 mm beyond the block and the beak 102 mm, giving the
156 mm span. The 13 mm beak drop and 52 mm pommel diameter are photo estimates.
Head-block half-width locates both working ends on actual receiving faces;
half-depth plus half hook thickness locates the hook's inner mounting face.
The hook stays straight through that contact interval before bending outward.

The unmeasured core, internal voids, joints, fasteners and alloy composition
remain unknown. Bodies are modeled as solid, including a steel grip core.
The silver pommel includes its lower button; other fittings use steel.
Calculated mass is approximately 1.533 kg versus the published 1.134 kg.
Density and hidden thicknesses are not adjusted to make those values agree.

### Diamond sections and silver

Generic `blade` accepts `section: "diamond"`. Each transverse section is a
four-vertex rhombus, with depth proportional to its current width throughout
the body and point. Equal authored width and thickness give a square section;
unequal diagonals give a rhombus. Curvature translates its centerline without
changing that ratio. Diamond sections require `singleEdge` to be zero.
The existing point curve controls both diagonals and terminates at one vertex.
Diamond sections do not use the cutting section's finite edge thickness floor.
Omitting `section`, or choosing `edged`, retains the cutting section described
above. These modes do not expand generic-blade scabbard eligibility.

The shared `silver` material uses 10,500 kg/m³, the room-temperature elemental
density listed by the [Royal Society of Chemistry](https://periodic-table.rsc.org/element/47/).
This is an explicit approximation for unidentified historical silver alloys.
It affects material mass and appearance while preserving the authored geometry.
Weapon schema/generator identities are 15/18; holder identities are 9/9 because
holders contain material fields and embedded weapon recipes.


## KHM A287 Katzbalger

Select `khm-a287`. The [recipe](khm-a287.json) covers browser and gameplay
`katzbalger`. The scabbard and accessory knives are outside this study.
The [museum record](https://www.khm.at/kunstwerke/landsknechtsschwert-mit-scheide-372681)
identifies Ulrich von Schellenberg's sword, around 1515. Published dimensions
are 884 mm length, 110 mm breadth, 165 mm depth and 1.4 kg mass. The record
identifies forged iron, cast fire-gilded brass, and a wooden grip covered with
fire-gilded brass sheet. Measurement axes are unspecified; assigning breadth
and depth to the transverse guard remains an interpretation.

Opposing whole-object photographs and hilt details appear in the record.
The [front detail](https://www.khm.at/pics/372681/HJRK_A_287_201504_5.jpg)
and [reverse detail](https://www.khm.at/pics/372681/HJRK_A_287_201504_6.jpg)
show the four short blade grooves, calyx grip and S-shaped guard. Photographs
are credited to Kunsthistorisches Museum, Hofjagd- und Ruestkammer. They are
private local reference evidence and are not redistributed in the repository.

Photo estimates allocate 747 mm to the blade and 137 mm to the hilt, including
an 85 mm upper covered grip, 10 mm exposed wood band, 32 mm lower sleeve and
10 mm guard root. Blade width is estimated at 44 mm, maximum calyx breadth at
70 mm, guard bar diameter at 10 mm and terminal bulbs at 18 mm. The inner
pair of fullers ends at 44 mm; the outer pair at 37 mm. These are estimates,
not additional published measurements. The crown rises above the broad wings
and terminates in a small central peen.

Blade thickness, groove depth, grip depth, sheet and end-cap thickness, hidden
tang, joints, fasteners and internal voids remain assumptions. Steel represents
the forged iron blade. Brass represents the cast guard and sheet over the wood
core; gilding is not assigned solid gold's mass. Engraving, rope ticks, patina,
wear and microscopic gilding are outside the structural scope. Material
volumes come from disjoint solids, without calibration to the museum mass.
Calculated mass is approximately 1.329 kg versus the published 1.4 kg.

### Multiple grooves and covered profile bodies

`fuller` contains a shared `bevelWidthRatio` and one to eight `grooves`.
Each groove has its own dimensions, face selection and axial interval.
`lateralPosition` is a signed fraction of the current broad-face half-width;
zero is the blade axis and plus/minus one are the bevel boundaries. The center
follows blade taper. Active grooves on the same face must be disjoint.
Grooves on opposite faces may overlap if their local trapezoidal cuts retain
positive metal. A maximum-depth sum alone does not establish a wall breach.
Continuous interval bounds include groove transitions and terminal points.
Grooves sharing a lateral line may occupy separate axial intervals. Inactive
landmarks lie on the actual neighboring surface, avoiding false ridges.
The existing float32 strip, position and normal budgets still apply.

`profileBody` describes a profiled oval core and optional material cover.
`profileGrip` shares this construction while retaining its 38 by 28 mm
anatomical limits. A wide pommel body does not acquire grip semantics.
`cover.endCap`, when present, is a positive axial thickness smaller than body
length. The core ends at that exact plane and the cover closes over it; the
core and cover share their complete boundary. Without an end cap, both core
faces remain exposed through the cover's annular ends.

Optional `radialSegments` specifies shared circumferential sampling for joined
profile components. Each LOD scales that count and rounds to quarter symmetry,
independently of the body's maximum width. Matching endpoint dimensions,
phase and cover thickness then give matching core and cover footprints.
Without an explicit count, sampling follows the existing radial error rule.
Use shared counts on matching ends; axial contact alone does not establish
matching polygonal footprints.

## Metropolitan Museum 14.25.1096 arming sword

Select `met-14-25-1096`. The [recipe](met-14.25.1096.json) covers gameplay
`arming_sword`; it does not establish Reitschwert or longsword coverage.
The [museum record](https://www.metmuseum.org/art/collection/search/27458)
identifies a Western European, possibly Italian sword, around 1450. Published
dimensions are 943 mm overall, 791 mm blade and 140 mm width, with a mass of
1.134 kg. Materials are steel, copper, wood and leather. The museum explicitly
identifies the grip as a modern restoration. This study reproduces the
displayed object rather than claiming an intact medieval organic grip.

The [first whole photograph](https://images.metmuseum.org/CRDImages/aa/original/LC-14_25_1096-002.jpg),
[opposing photograph](https://images.metmuseum.org/CRDImages/aa/original/LC-14_25_1096-003.jpg)
and [hilt detail](https://images.metmuseum.org/CRDImages/aa/original/SC-LC-23755.jpg)
show the beveled wheel, tapered ribbed grip, bowed guard with a central point,
short opposing fullers and rounded blade point. These public-domain images
are CC0 under the museum's [Open Access policy](https://www.metmuseum.org/hubs/open-access).
Credit: Gift of William H. Riggs, 1913. No exact axial or edge-on photograph
is available. The published width is interpreted as the guard span.

Photo estimates place blade root width at 44 mm, wheel diameter at 59 mm,
wheel face at 45 mm, grip length at about 87 mm and grip crest widths at
29 to 38 mm. Nineteen rounded courses represent the observed transverse
relief. Fuller mouth width is 12 mm, ending 220 mm from the blade heel.
The guard sweeps 19 mm toward the blade and has a 7 mm central shoulder.
These are estimates, not additional museum measurements.

Blade thickness of 5.5 mm, fuller depth of 0.8 mm, wheel depth of 24 mm with
an 18 mm rim, guard depth of 24 mm, grip depths of 17 to 22 mm and a 1 mm
leather cover are unmeasured assumptions. The rounded ribs are modeled in a
covered wooden envelope; the restored grip's actual internal support is
unknown. Neither cord turns nor a helical seam are asserted. The blade's
constant 7 mm receiving heel occupies a matching open guard mortise, with
the fuller beginning beyond that engagement. Hidden tang, peen, fasteners,
internal voids, copper inlays, corrosion and wear remain outside the scope.
Calculated mass is approximately 1.512 kg versus the published 1.134 kg.
Neither material density nor hidden thickness is fitted to the museum mass.

### Finite wheel seats, guard mortises and covered ribs

`wheelPommel` is one transverse beveled wheel with a flat receiving chord.
`diameter`, `faceDiameter`, `thickness` and `rimThickness` define its turned
profile; `seatHeight` locates the cut plane relative to the wheel center.
The `base` frame is the opposite rim and `top` is the receiving chord. A
grip must fit the actual chord footprint, including its beveled depth.
Plane classification resolves floating-point rotation and interpolation
roundoff before cutting; it does not enlarge geometric audit tolerances.
The cut depth and its separation from existing sampled vertices must meet
the shared manufacturing minimum, except for classified roundoff. A real cut
that leaves unresolved triangles or loses closed float32 topology is rejected.

`mortisedGuard` describes a single bowed, chamfered solid. `width` is its
overall span, `height` is the central bar height, and `sweep` raises the arms
toward the blade. `terminalScale` expands arm height toward the ends.
`shoulderHeight` adds the central blade-facing point. `edgeBevel` defines
the chamfer and `thickness` the outer depth. Its `base` frame receives the
grip, and `top` locates the mortise floor rather than the raised arm ends.
The open mortise removes a constant beveled blade section, specified by
`mortise.width`, `mortise.thickness` and `mortise.bevelWidthRatio`.
The mounted blade must have the matching section and pose, a constant
ricasso through the shoulder, and grooves starting beyond that engagement.
Distal curvature and belly remain available beyond the constant ricasso.
Study controls keep those dimensions linked. Manual mismatches are rejected.

Optional `ribs` on `profileGrip` and `profileBody` cut rounded transverse
troughs inward from the authored crest envelope. `count` specifies complete
courses and `depth` is the radial crest-to-trough difference. Both ends
remain at crest height. The core and cover share the same sampled boundary;
ribs are not overlapping rings. Counts are bounded at 64, depth cannot exceed
one quarter of pitch, and trough sections must preserve core and cover
clearance. Grip limits apply to crest dimensions. Existing construction
budgets can reject a costly combination of rib and radial sampling counts.

## Cleveland Museum of Art 1916.686 estoc

Select `cma-1916-686`. The [recipe](cma-1916.686.json) covers browser `estoc`.
There is no gameplay estoc entry; other sword categories remain separate.
The [museum record](https://www.clevelandart.org/art/1916.686) identifies an
early sixteenth-century German sword of steel, wood and leather. Published
dimensions are 1566 mm overall, 1253 mm blade, 262 mm quillons and 300 mm
under the label "grip". The reported mass is 1.6 kg. The museum explicitly
describes the blade as three-sided.

The [whole photograph](https://openaccess-cdn.clevelandart.org/1916.686/1916.686_print.jpg),
[alternate whole photograph](https://openaccess-cdn.clevelandart.org/1916.686/1916.686_alt0_print.jpg)
and three hilt details
([first](https://openaccess-cdn.clevelandart.org/1916.686/1916.686_alt1_print.jpg),
[second](https://openaccess-cdn.clevelandart.org/1916.686/1916.686_alt2_print.jpg),
[third](https://openaccess-cdn.clevelandart.org/1916.686/1916.686_alt3_print.jpg))
show the trumpet pommel, three separated rib bands and thin curled quillons.
The first two hilt details have closely related projections. These photographs
are CC0 under the museum's [Open Access policy](https://www.clevelandart.org/open-access).
Credit: Gift of Mr. and Mrs. John L. Severance.

The published grip label has ambiguous endpoints: overall minus blade leaves
only 313 mm for the complete hilt. The displayed fit uses 250 mm of exposed
leather, a 51 mm pommel and a 12 mm guard. It retains the original 300 mm
museum field without treating it as an additional exposed leather length.
Photo estimates include a 51 mm pommel cap, 19 mm neck, 28 mm maximum grip
breadth, 27 mm blade heel, 32 mm guard block and 10 mm round quillons.
Seven, six and nine rounded courses represent the three visible rib groups.

Blade section height of 16 mm, guard depth of 24 mm, grip depths of 16 to
20 mm and a 1 mm cover are unmeasured assumptions. The pommel's rotational
symmetry and the blade's exact section orientation are also assumptions.
Different photographs suggest that opposing quillon curls lie in a plane
transverse to the blade axis; their precise spatial path is not uniquely
recoverable. The fit uses a nominal 16 mm bend radius and short cut ends.
Calculated mass is about 1.400 kg versus the reported 1.6 kg. Neither density
nor hidden thickness is calibrated to that reported mass.

Six contiguous covered profile intervals represent one grip with localized
ribbed regions. Matching end footprints partition the material without
overlapping rings or bands. They do not assert six historical grip pieces
or a known cord arrangement. The existing guard assembly describes a beveled
central member and two curled members meeting complete side seats. Grip and
blade meet opposite horizontal faces. Tang, peen, internal mortise, adhesive
and fasteners are not reconstructed. Corrosion, pitting, torn leather and
small surface irregularities remain outside structural replication.

### Three-sided generic blades

`blade.section: "triangular"` defines a three-sided section about its area
centroid. `width` is the base breadth and `thickness` is the full transverse
height. The base lies at minus one third of that height and the ridge at
plus two thirds; receiving surfaces must cover this asymmetric footprint.
Both dimensions scale with the authored blade-width envelope. Curvature
moves the centroid along the authored axis. A point closes every section
vertex at one tip, while omission of a point retains the finite end cap.
An asymmetric cutting-edge offset is rejected for this thrusting section.
Existing edged and diamond sections retain their geometry, and this option
does not expand generic-blade scabbard eligibility.

## Metropolitan Museum 96.5.23 halberd

Select `met-96-5-23`. The [recipe](met-96.5.23.json) covers browser
`halberd-1540` and gameplay `halberd`. It does not establish pollaxe,
Lucerne hammer, hand axe or pike coverage. The
[museum record](https://www.metmuseum.org/art/collection/search/25021)
identifies a probably German halberd of 1525–50, made of steel and wood.
Published dimensions are 1613 mm overall, 508 mm head and 241 mm width;
the reported mass is 2.251 kg. The head length is interpreted as apex to
the lower axe corner. The museum does not specify those endpoints.

The [first face](https://images.metmuseum.org/CRDImages/aa/original/96.5.23_003oct2014.jpg)
and [opposing face](https://images.metmuseum.org/CRDImages/aa/original/96.5.23_004oct2014.jpg)
show the complete head and part of the upper haft. Both preserve the paired
upper axe scallops, deep lower notch, notched beak, central ridge, narrowing
langets and round fastener heads. These public-domain photographs are CC0
under the [Met Open Access policy](https://www.metmuseum.org/hubs/open-access).
Credit: John Stoneacre Ellis Collection, Gift of Mrs. Ellis and Augustus
Van Horne Ellis, 1896.

Photo estimates place the upper axe reach at 130 mm, opposing beak reach
at 111 mm, cutting-edge height at 212 mm and upper axe corner 296 mm below
the apex. The visible haft and broad langets are about 28 mm across,
narrowing to 14 mm langet tails; rivet heads are modeled at 9 mm diameter.
The wood end is reconstructed 450 mm below the apex. Its receiving plane
therefore lies 58 mm above the lower axe corner, which defines the plate's
local origin. This dimensional datum accounts for the mounting offset.

Neither photograph shows the lower haft, butt or langet ends. The square
28 mm wooden section, 340 mm complete langets, 3 mm langet thickness,
0.6 mm plate edges, 34-to-11 mm ridge depths and 1.5 mm rivet crowns are
unmeasured assumptions. No claim is made about an original or replaced
haft. The ridge's exact transverse shape and thickness are not established
by the opposing oblique photographs. Density and hidden thickness are not
calibrated to the published mass. Calculated mass is approximately 1.938 kg
versus the museum's 2.251 kg.

The head is one closed forged blank. Its broad central seat meets the wood
end and both langets. The langets have complete flat receiving surfaces
against the wood, and the rivet crowns have flat undersides against the
langets. These disjoint regions do not assert historical weld seams,
internal rivet shanks, drilled holes or a particular hidden fastening
method. Pitting, corrosion, wear and small stamped marks are excluded.

### Contoured forged plates

`contouredPlate` describes a closed planar boundary lifted into a solid
with a variable central ridge. `width` and `length` scale the boundary's
dimensionless coordinates. `start` gives its first point; `boundary`
contains `line` spans with a `to` endpoint or `cubic` spans with two
`controls` and a `to` endpoint. The final span must return to `start`.
Transverse coordinates lie between minus one and one; axial coordinates
lie between zero and one and must reach both ends. Self-intersecting or
degenerate outlines are rejected. Holes are not part of this construction.

Each increasing `thickness` station gives an axial `at` fraction, full
`edge` and `ridge` depths, `flatHalfWidth` and `ridgeHalfWidth`. The crest
is flat inside the first width, slopes to the edge depth at the second,
then remains at edge depth. Station values interpolate axially. A zero
flat width produces a sharp central ridge. Positive stations must retain
the shared manufacturing minimum between the flat and outer ridge widths.
Only a unique terminal outline point may have zero depths and widths;
that point closes the entire section. Interior zero sections are rejected.

The shared curve sampler preserves cusps and bounds curved-edge deviation.
Cubic spans are split at their station and ridge intersections before
sampling. The scalar cubic's derivative extrema isolate crossings and
tangencies, including coincident structural intersections. This avoids
creating tiny incidental edges beside a sampled curve's ridge crossing.
The planar region is then partitioned at every station and moving ridge
edge. Adjacent cells share each cut vertex. Each cell is retriangulated
before refinement, retaining every boundary turn and shared junction;
arithmetic collinear points left by incidental diagonals are removed.
Flat sections omit inactive ridge partitions. Real cut features below the
manufacturing minimum are rejected; only arithmetic roundoff is classified
at an existing cut plane. No geometric audit tolerance changes.

Surface refinement preserves those partition edges. Within each sloping
cell, half-thickness is a quadratic numerator over a positive affine
denominator. Twice the largest edge-midpoint interpolation error bounds
the entire triangle, including the continuous limit at a unique point.
The resulting maximum surface-deviation budgets are 125, 50 and 20 microns
at Low, Medium and High. Sampling and allocation have explicit budgets;
an unresolved or excessive construction is rejected. Constant flat plates,
variable crests, asymmetric outlines and true points use the same primitive.

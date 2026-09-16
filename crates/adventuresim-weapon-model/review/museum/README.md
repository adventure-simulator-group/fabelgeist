# Museum weapon studies

These recipes distinguish photographed construction from inferred dimensions.
Select a study in the weapon modeler's preset selector. Its controls edit the
same canonical recipe used by native generation and GLB export. Studies are
separate from the general authoring presets and gameplay chassis registry.

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

Fuller `start`, `end`, `entryLength` and `exitLength` are metres from the
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

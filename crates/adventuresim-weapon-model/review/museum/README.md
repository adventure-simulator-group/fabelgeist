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

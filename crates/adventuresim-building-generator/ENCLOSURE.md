# Building enclosure contracts

Exterior enclosure is a separate guarantee from structural support and
individual mesh closure. Two closed, supported solids can leave a passage
between them.

## Wall corners

The source storey perimeter determines which walls must meet. Offset wall
lines intersect at shared corners before wall bodies or timber bays are
created. Timber face offsets have their own intersections, so adjacent
facades reference one corner post. Jetty brackets terminate on actual lower
posts or masonry bearing solids. Opening jamb widths and bearing allowances
use the resolved wall span, including editor-added openings in corner bays.

The audit infers required corners from source wall adjacency, independently
of generated junction bonds. It intersects resolved cuboids, infill prisms, and splayed or arched
masonry with vertical sections through each corner and unions their elevation
intervals. Missing material at any height fails the contract. Declared
apertures retain their intentional opening intervals.

## Gables

Gable profiles are clipped against the resolved roof underside and the wall
top. Roof thickness is measured normal to the slope; its vertical effect at a
fixed horizontal position is thickness divided by the normal's vertical
component. Primary gable trusses use the same covering planes.

The gable audit independently clips a required section from roof planes and
wall spans, then subtracts the union of enclosure polygons. A missing region
larger than the numerical area tolerance is an error. The enclosure must also
sit within the wall bearing plane's slab depth. Pitch edits reclip the gables.

These checks cover straight storey-wall corners and primary gables. Existing
specialized contracts still govern towers, roof children, half-hips, and
intentional open structures. Individual mesh topology and load-path audits
remain necessary.

## Regression checks

Run the focused tests with:

```console
cargo test -p adventuresim-building-generator --lib enclosure_
```

The regressions include the committed Fachwerk scene, deliberate missing
corner material, lowered and narrowed gables, and rays through final detail,
facade, and shell meshes. The full building-generator suite checks interaction
with openings, circulation, roof editing, and other building archetypes.

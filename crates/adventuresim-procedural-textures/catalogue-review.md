# Catalogue approach review

The catalogue contains 24 recipes, including six presets of the shared leaf
generator. This review considers the construction of each material, its useful
feature scales, pigment, surface response and authoring controls. More texture
noise is not an acceptance criterion: structure should explain the appearance.

## Technique references

[SideFX's organic-texture tutorial](https://www.sidefx.com/tutorials/how-to-create-organic-textures/)
demonstrates faceting, layered detail and periodic shape stamps. Here these
principles become finite local features with recipe-specific sizes and depths:
pores in clay, application marks in plaster and fracture faces in stone. The
tutorial's AO-driven color ramps are deliberately excluded. Pigment remains
independent of lighting and cavity shading.

[Worley Noise documentation](https://www.sidefx.com/docs/houdini/nodes/cop/worleynoise.html)
explains anisotropic size, jitter, distance metrics and periodicity constraints.
The new stamp evaluator bounds its search support and wraps site identities.
Smooth bowls and angular planes have distinct profiles. It does not draw a
continuous Voronoi crack network across every material.

[Procedural Herringbone with VEX and MaterialX](https://www.sidefx.com/tutorials/houdini-tutorial-procedural-herringbone-pattern-using-vex-materialx/)
separates unit layout, local UVs and per-unit variation. Planks and shingles use
their existing construction layouts and independent local wood coordinates.
Growth bands deform around branch intersections; the same band boundary drives
the dark pigment mask and relief. Footprint integration antialiases both.

These are analytic Rust adaptations, not ports of the Houdini networks or claims
of physical simulation. The existing pinned leaf equations are a separate,
explicit reference adaptation described in [the leaf model](src/leaf/README.md).

## Decisions for every recipe

| Recipe | Approach and decision |
| --- | --- |
| White oak leaf | Retain the shared lobed blade and hierarchical venation. Its lobes, petiole, posture and tissue relief already have structural controls. |
| Dry white oak leaf | Retain the same botanical shape family with separate dry pigment and relief data; drying is not a separate shape program. |
| Hazel leaf | Retain the shared broad blade, basal notch and toothed margin preset; anatomy, not surface noise, distinguishes it. |
| Blackthorn leaf | Retain the compact toothed blade preset and independent front/back response. |
| Hawthorn leaf | Retain the shared lobing and venation parameters; preserve the relationship between organs and veins. |
| Beech leaf | Retain the restrained blade profile, secondary veins and corrugation. |
| Oak bark | Retain the connected furrow/plate system, branch handoffs and tapered checks. Its structural hierarchy is the quality reference for this review. |
| Forest soil | Retain cohesive clods, detached crumbs, pores and compaction/moisture-dependent relief. Packed height/AO is intentional; soil shading belongs to the terrain material. |
| Forest litter | Retain stratified intact, curled, torn, skeletal and humified debris, contact masks and semantic distance filtering. This already models layered objects rather than generic noise. |
| Rock | Add angular overlapping facets and sparse dissolution cavities to the broad geological body. Couple cavity AO to the actual pits. Retain the small mineral palette and normal-variance-aware mip response. |
| Lime plaster | Replace repeating sinusoidal application bands with finite, angled float strokes. Retain sparse pulls, mineral aggregate and millimetric relief. |
| Hewn oak | Retain deformed asymmetric growth bands, branch knots, vessels, rays and blended adze facets. The existing pigment boundaries already follow the relief field. |
| Wattle and daub | Replace broad periodic tool waves with finite application marks. Keep embedded fibres, aggregate, shrink checks and localized exposed wattle. Remove continuous pigment clouds and painted cavity darkness. |
| Handmade brick | Keep the running bond, bowed edges, chips and independent brick/mortar palette. Add distinct fired-clay pores and lime aggregate; the earlier color/normal improvements had not supplied these details. |
| Rubble masonry | Keep irregular gravity-laid courses and interlocking units. Replace soft face undulation with fracture cuts; add pores and mortar grains. Separate unit colors from mortar and remove height-derived pigment shading. |
| Dressed stone | Retain planar ashlar, varied bevels, clustered edge fractures, corner cuts, grouped cavities and trowelled aggregate mortar. These already follow the multi-scale reference approach. |
| Clay roof tile | Retain the beaver-tail overlap and manufacturing deformation. Add pores and finite dragged inclusions; keep kiln colors fixed per unit and antialias exposed lower-course boundaries. |
| Slate roof | Retain split terraces, broken ledges and edge delamination from the preceding review. Broad cleft faces remain planar. |
| Timber shingle | Keep the riven layout, tail variation, checks and overlap. Add knot-aware growth structure with separate grain/pigment controls; weathering affects surface response rather than painted shading. |
| Plank floor | Keep board widths, joist-aligned butt joints, sparse nails and cupping. Replace broad sine pigment waves with filtered growth bands and knot flow. Joint and nail materials have independent colors. |
| Lead sheet | Retain quiet sheet deformation and add sparse soft dents. Represent the default continuous opaque patina as a dielectric with one pigment and finish. Seams and object-edge wear remain geometry responsibilities. |
| Ironwork | Retain overlapping die impressions, scale losses, pits and the coupled bare/oxide finish from the preceding review. |
| Window glass | Add seeded anisotropic draw undulation to explicit finite striations and lenses. Separate broad roughness from bubble relief. Preserve the optical-normal and transmission contract. |
| Crenellation mask | Retain the coverage-preserving architectural silhouette. Fix Texture Studio to use the RGBA alpha channel for coverage and preserve its masonry pigment; it previously treated the red pigment channel as opacity and hid the whole mask. Close crowns remain geometry. |

## Authoring and limits

New stamp groups expose cell counts, radii, clustered density, jitter, size variation,
direction, edge width, roundness and depth. Stamp dimensions are fractions of a
cell; cell counts span the declared physical tile. Depth is a fraction of the
recipe's height range, except plaster's existing physical relief multipliers.
Cut-board grain and palettes are independent for floors and shingles. Obsolete
controls are removed from the final schema rather than retained as inert fields.

The added full-quality detail uses 1024-pixel bakes for rock, handmade brick,
clay roof tiles and timber shingles. Draft and medium remain capped at 128 and
256 pixels. Albedo boundary filtering and new palette mips average in linear
light. Existing shared normal and ARM mip behavior is unchanged.

The review does not add arbitrary rust, moss, runoff or edge wear to reusable
tiles: those effects need placement and exposure inputs from the consuming
scene. Leaf architecture constraints can suppress incompatible morph controls.
Glass's per-texel thickness remains inspectable data; the current renderer uses
nominal scalar thickness. A material-only preview cannot establish how every
recipe looks on every game asset, projection or viewing distance.

Native captures, recipe exports, timings and test logs belong under
`target/catalogue-review/`. Use the shared Texture Studio renderer to compare
lighting, relief, tiling and 3D projections before accepting a material change.

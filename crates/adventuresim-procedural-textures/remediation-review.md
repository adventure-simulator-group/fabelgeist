# Procedural material remediation

This pass follows the 24-recipe artistic audit of the previous catalogue. Its goal is material-specific structure with quiet, categorical pigment. The techniques are adaptations to a deterministic live Rust/WASM generator; no Houdini network is run at runtime.

| Recipes | Change |
|---|---|
| Rock | Joined irregular triangular fracture planes with shared corner heights, selective secondary spalls, and fewer uniform pores. |
| Rubble masonry | Warped courses, varied corner cuts, stronger stone-local fracture faces, and physical mortar-to-stone rollover independent of pigment filtering. |
| Dressed stone | Stronger shallow tool marks; physical bevel profile controls height independently of coverage. |
| Handmade brick | Independent physical clay rollover, varied cupping/twist and quieter repeated end damage. |
| Lime plaster | Broad worked areas locally suppress aggregate instead of uniformly roughening the face. |
| Wattle and daub | Pixel-footprint filtering of narrow fibre/fissure profiles and a one-texel derivative filter to remove zipper aliasing. |
| Hewn oak | Stronger broad hewing facets, evaluated on a beam. |
| Plank floor | Independent grain coordinates and identity for each butt segment; open cathedral figure replaces closed targets. |
| Timber shingle | Quieter growth ridges and less continuous wandering grain; split checks remain separate; filtered lap derivatives remove dotted edges. |
| Clay roof tile | Independent physical lip rollover, stronger per-tile toe asymmetry, sparse varied pores, and filtered lip derivatives. |
| Slate roof | Broader quiet cleavage intervals, less uniform face ripple, and continuous sampled lap normals. |
| Lead sheet | Folded-sheet and strip-reflection review; finish remains independently controlled. |
| Ironwork | Broader overlapping hammer impressions and fewer dominant isolated pits, evaluated on a beam with reflections. |
| Window glass | Finite framed pane, adjustable reflection illumination and backdrop separation. |
| Crenellation mask | Repeated eight-pitch crown view; the intentional silhouette is retained. |
| White oak and dry white oak leaves | Shared rounded lobe/secondary hierarchy; tapering vein relief; subdued tissue texture. Dry leaves add stronger curl and broad folds. |
| Hazel leaf | Continuous rounded base instead of squared basal shoulders; thinner vein hierarchy. |
| Blackthorn leaf | Finer veins and relief; retained species outline. |
| Hawthorn leaf | Fewer coherent primary lobes with matching secondary destinations and sweeps. |
| Beech leaf | European-beech game preset with a finer eight-pair vein hierarchy. |
| Oak bark | Quieter plate relief and shoulders, irregular breakup replacing sine bands, interpolated parallax intersections, and higher height precision. |
| Forest soil | Angular aggregate distances with a linear/rounded profile blend, and higher height precision. |
| Forest litter | Stable broad placement patches, folded-height overlap ordering, constant pigment per leaf, and removal of duplicated full-perimeter height rims. |

Bark and soil use RG for 16-bit normalized height and B for AO. Both the tactical shaders and Studio decode that format; mip levels average decoded values before packing. This addresses quantized raking slopes without hiding them under additional surface noise. Existing recipe documents are not migrated.

## Reference techniques

- [Moeen Sayed: Tile-Based Textures](https://www.sidefx.com/tutorials/how-to-create-tile-based-textures/): local coordinates and masks per construction unit.
- [Moeen Sayed: Organic Textures](https://www.sidefx.com/tutorials/how-to-create-organic-textures/): shaped scatter and layered faceting.
- [SideFX PolyBevel](https://www.sidefx.com/docs/houdini/nodes/sop/polybevel.html) and [SDF to Mono](https://www.sidefx.com/docs/houdini/nodes/cop/sdftomono.html): distinguish physical edge shape from coverage antialiasing.
- [Amy Liu: Ruins Terrain Toolset](https://amyliu.dev/projects/Houdini_Ruins_Terrain_Toolset/): mask-directed repeated fracture passes. This is a geometry reference, not a finished rock-wall shader.
- [Artur J. Zarek: Copernicus generators](https://www.ajz3d.com/blog/sbs-grunges-and-generators-in-cops/): controlled knot placement and distortion. The board-domain correction is specific to our implementation.
- [Michael Ekker: Fallen Leaves](https://www.michaelekker.com/projects/3de8Bo) and [Ole Groenbaek: Texture Scattering Tool](https://olegroenbaek.artstation.com/blog/mazp/houdini-texture-scattering-tool): bending, accumulation, layered placement and baking. The live generator uses analytic folded stamps rather than their offline simulation workflows.
- [Tyler Bay: Shading Theory with Karma](https://www.sidefx.com/tutorials/shading-theory-with-karma/): use appropriate geometry, reflections and backgrounds when judging metal and glass.

## Validation evidence

Native captures and the full documents used to generate them are stored locally under `target/remediation/`. Generated PNGs are intentionally excluded from source control. The gallery covers all 24 recipes, with raking closeups and material-specific forms. Exact capture documents accompany the images. Independent reviewers assessed six mineral/earth, nine crafted, and nine organic recipes.

Final validation passed 159 procedural-texture tests (10 visual-export tests ignored), all 9 Studio tests, and Clippy with warnings denied for the texture, material and Studio crates across all targets. The normal-orientation oracle uses independently exported heights with the matching derivative footprint and retains its strict signed agreement check through Bevy's actual tangent frame. Formatting and whitespace checks passed. The static WASM build completed and the local browser rendered the catalogue, new forms and reflection lighting.

Repository-wide `just lint` passed generated-binding verification but failed the Rust quality gate on existing armor-model and character-creator debt. It reported no texture/material/Studio violations. The branch includes main's repair of a missing scope header in `rust-quality.toml`; no quality ceilings were raised.


## Review outcome and remaining limits

The reviewers accepted the revised masonry planes and physical edge profiles, independent plank segments and open grain figures, worked plaster, finer leaf hierarchy, and dry-leaf folds. Normal-channel isolation established the source of the roof/daub zipper artifacts. One-texel filtering of the derivative input resolves their dotted rhythm; extreme closeups are softer, while physical height and categorical pigment are unchanged.

This is an incremental catalogue remediation, not a claim that every recipe has reached the reference artists' quality. Bark retains its existing large groove graph; soil still has soft aggregate masses; litter still has conspicuous curled ribbons and some cutout-like layering. Dressed-stone nicks, slate cleavage lines, and iron pit stamps have room for further variation. Lead's quiet patina does not provide strong reflection detail in these static captures, and the glass setup demonstrates tint/transmission more clearly than optical deformation. The new forms and environment controls make those limitations reviewable without forcing noisy albedo or a shiny patina.

A separate code review checked packed height/AO consumers and reflection-map layout. It identified overlapping pinned crown strips; spacing and camera framing now account for the wider mesh, with a nonintersection regression test. The far-glass backdrop uses one enlarged repeating quad so it continues to cover the pane during orbiting.

Existing saved recipe documents are intentionally not migrated. New controls are listed in the catalogue inspector, including physical rollover width, fracture planes, board segmentation, smoothing masks and organic placement/fold controls.

# Paint recipes and appearance estimates

The palette records a pigment preparation and binder for each tincture. These
choices drive unlit color and paint roughness in the native editor, browser,
SVG, texture maps and GLB. They leave heraldic identity, geometry, painted
modeling coverage and scene lighting independent.

This is an ingredient catalog with estimated appearances. It does not
calculate color from ingredient quantities or optical measurements. Ground,
film thickness, brush relief and a finishing coat remain separate controls.
The paint is represented as an opaque dielectric; transparent pigment layers,
particle scattering, drying and deterioration are not simulated.

## Evidence

Faltermeier and Meyer, *Notes on the Restoration of the Behaim Shields*,
1995, p. 54, identify azurite, vermilion and lead-tin yellow in the original
paint on German shield 25.26.1. They describe tempera, probably egg. The catalog
therefore marks these selections as `ConservationInference`: pigment evidence
is stronger than the binder identification. This does not reconstruct an
exact workshop formula. The report's unusual term "lead-tin white" is not
silently reinterpreted as lead white; our lead-white entry has a separate
source. [Conservation report](https://resources.metmuseum.org/resources/metpublications/pdf/Appendix_Notes_on_the_Restoration_of_the_Behaim_Shields_The_Metropolitan_Museum_Journal_v_30_1995.pdf).

Cennino Cennini's *Il libro dell'arte*, c. 1400, supplies comparative Italian
workshop instructions. We use Daniel V. Thompson Jr.'s 1933 translation,
*The Craftsman's Handbook*, originally Yale University Press, subsequently
reprinted by Dover. These entries are `WorkshopManual`, not direct evidence
for German heraldic practice in 1544.

| Paint choice | Binder | Source location |
| --- | --- | --- |
| Lead-tin yellow; vermilion; azurite | Egg yolk, inferred | Behaim conservation report, p. 54 |
| Yellow ochre; red ochre; lead white | Egg yolk | Cennini XLV, XXXVIII, LIX; panel method CXLV |
| Lampblack; vine black | Egg yolk | Cennini XXXVII; panel method CXLV |
| Malachite | Egg yolk | Cennini LII |
| Orpiment | Animal-glue size | Cennini XLVII, shields and lances |
| Orpiment and indigo | Animal-glue size | Cennini LIII, shields and lances |
| Indigo and lead white | Animal-glue size | Cennini CXLIV, panel or shield |
| Lac, natural ultramarine and lead white | Egg yolk | Cennini CXLV, violet panel paint |

Cennini gives two parts orpiment to one indigo for the green. These historical
parts are recorded without assuming grams, volume or a calibrated optical
mixing ratio. Other proportions remain unspecified.
[Pigment chapters](https://noteaccess.com/Texts/Cennini/2.htm),
[painting chapters](https://noteaccess.com/Texts/Cennini/6TM.htm).

## Rendering and authoring

Each recipe supplies authored base, shadow and highlight sRGB swatches and a
starting roughness. None is a sampled museum color or a measurement of a
reconstructed paint. The tempera and size roughness values distinguish the
display presets; they are not universal physical constants for those binders.
Preparation, concentration and finishing can change a real paint's appearance.

In particular, the shadow and highlight swatches are artistic interpretations.
Their separate mixtures have not been reconstructed. They do not claim that
every selected ingredient combination supports the same historical modeling
method. Flat painting is available by setting both modeling coverages to zero.

Under **Paint recipes**, select a preparation for each tincture and expand
**Recipe details** for its ingredients, binder and source. Suggested colors do
not restrict deliberate alternative tincture mappings. **Open measured paint
mixer** provides separate recipes for base, shadow and highlight using the
[measured reference stocks](MEASURED_PAINT.md). Catalog appearances remain
estimates. Metal leaf retains its separate conductor response; the selected
paint supplies flat artwork and any painted modeling over that leaf.

`surface.palette` stores seven typed paint selections, ordered as Or, Argent,
Gules, Azure, Sable, Vert and Purpure. For example:

```json
{"kind": "Recipe", "recipe": "LampblackTempera"}
```

The complete document is the editable source. Bundles additionally include
`paint-recipes.json`, with ingredient, binder, source and appearance records;
standalone GLBs include the same records in `extras.paintRecipes`. These
describe the palette, including unused colors. They are not quantities consumed
or production costs. Those require the separate
[gameplay integration work](../GAMEPLAY_INTEGRATION.md).

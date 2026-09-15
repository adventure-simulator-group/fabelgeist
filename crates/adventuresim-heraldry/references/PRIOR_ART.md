# Existing heraldry tools

This review separates useful engineering from historical evidence. It examines
[Armoria](https://github.com/Azgaar/Armoria/tree/9351aa69ebc9ae52852311dea3f74cec28e67f6e)
at revision `9351aa69ebc9ae52852311dea3f74cec28e67f6e` and
[Fantasy Map Generator](https://github.com/Azgaar/Fantasy-Map-Generator/tree/a7289d3e21bcd0ab3dc73d87b29c9ad8c32a2f94)
at revision `a7289d3e21bcd0ab3dc73d87b29c9ad8c32a2f94`, inspected on
15 September 2026. No code or artwork was incorporated through this review.

## Charge preparation and composition

Armoria prepares existing SVG artwork for reuse, with author, source and
license metadata. Primary, secondary and tertiary regions can be recolored
independently. Its [contributor template](https://github.com/Azgaar/Armoria/blob/9351aa69ebc9ae52852311dea3f74cec28e67f6e/public/charges/template.svg)
and [charge renderer](https://github.com/Azgaar/Armoria/blob/9351aa69ebc9ae52852311dea3f74cec28e67f6e/src/components/object/Charge.svelte#L46-L62)
provide a useful model for preparing a small, verified charge collection.
Our adaptation must also retain interior drawing, painted tones and grouping
needed for coherent deformation. The lion is the current worked example.

Its [generator](https://github.com/Azgaar/Armoria/blob/9351aa69ebc9ae52852311dea3f74cec28e67f6e/src/scripts/generator.js#L15-L181)
coordinates divisions, ordinaries, charge counts, placement and tincture
contrast. [Shield-specific anchors](https://github.com/Azgaar/Armoria/blob/9351aa69ebc9ae52852311dea3f74cec28e67f6e/src/data/shields.ts#L13-L53)
help arrangements fit different outlines. These suggest a future constrained
sampler over our existing `Arms` structure. The probability tables are
authored choices, not measurements of historical frequency. Our sampler
should retain scoped deterministic randomness rather than copying Armoria's
replacement of global `Math.random`.

Armoria also defines [repeat tiles](https://github.com/Azgaar/Armoria/blob/9351aa69ebc9ae52852311dea3f74cec28e67f6e/src/data/dataModel.js#L141-L173)
for vair, potent, semy, scales, fretty and other patterns. Consult these and
existing Houdini/Shadertoy techniques before implementing non-trivial
patterns. Record the chosen technique's provenance and check its license.
Historical use and physical construction require separate evidence.

## Related identities and material prices

Fantasy Map Generator accepts parent arms, kinship and dominion probabilities,
and a settlement or culture type. [Kinship](https://github.com/Azgaar/Fantasy-Map-Generator/blob/a7289d3e21bcd0ab3dc73d87b29c9ad8c32a2f94/src/generators/emblems-generator.ts#L36-L108)
can retain colors, divisions, ordinaries and charges;
[dominion](https://github.com/Azgaar/Fantasy-Map-Generator/blob/a7289d3e21bcd0ab3dc73d87b29c9ad8c32a2f94/src/generators/emblems-generator.ts#L222-L250)
can add a canton derived from the parent's arms.
[Settlement integration](https://github.com/Azgaar/Fantasy-Map-Generator/blob/a7289d3e21bcd0ab3dc73d87b29c9ad8c32a2f94/src/generators/burgs-generator.ts#L374-L387)
and [motif tables](https://github.com/Azgaar/Fantasy-Map-Generator/blob/a7289d3e21bcd0ab3dc73d87b29c9ad8c32a2f94/src/data/emblems/typeMapping.ts)
connect emblems to world data. This suggests making houses, towns and patrons
visibly related through explicit game relationships. Their particular
probabilities and canton rule are fantasy heuristics, not rules for Germany
in 1544; historical differencing needs its own study.

Its [market calculations](https://github.com/Azgaar/Fantasy-Map-Generator/blob/a7289d3e21bcd0ab3dc73d87b29c9ad8c32a2f94/src/generators/markets-generator.ts#L205-L245)
derive raw-material prices from supply and demand, then manufactured prices
from local ingredient costs plus value added. This is useful for a future
interface between our recipes and the game economy. That implementation
averages production recipes; it does not find the cheapest paint mixture for
a target color or provide historical pigment prices.

The inspected [Armoria renderer](https://github.com/Azgaar/Armoria/blob/9351aa69ebc9ae52852311dea3f74cec28e67f6e/src/components/object/Shield.svelte#L210-L218)
and [FMG renderer](https://github.com/Azgaar/Fantasy-Map-Generator/blob/a7289d3e21bcd0ab3dc73d87b29c9ad8c32a2f94/src/renderers/emblems/renderer.ts#L263-L268)
use SVG palette colors and display overlays. They do not replace our separate
paint, substrate, relief and physical illumination models.

## Code licenses and artwork licenses

Armoria's [license statement](https://github.com/Azgaar/Armoria/blob/9351aa69ebc9ae52852311dea3f74cec28e67f6e/README.md#L48-L54)
identifies MIT code and separate artwork licenses. Fantasy Map Generator also
has an [MIT software license](https://github.com/Azgaar/Fantasy-Map-Generator/blob/a7289d3e21bcd0ab3dc73d87b29c9ad8c32a2f94/LICENSE).
Neither statement substitutes for checking a third-party image's provenance
and individual license before reuse.

Armoria's [rampant lion](https://github.com/Azgaar/Armoria/blob/9351aa69ebc9ae52852311dea3f74cec28e67f6e/public/charges/lionRampant.svg#L1-L3)
is labeled WappenWiki CC BY-NC-SA 3.0. The
[NonCommercial condition](https://creativecommons.org/licenses/by-nc-sa/3.0/)
excludes commercial use under that license. Our Commons lion instead uses
[CC BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/), which permits
commercial reuse subject to attribution and applicable share-alike terms.
Preserve the source and modification notices accompanying adapted artwork.

There are individual leads worth checking: Armoria's
[great helm](https://github.com/Azgaar/Armoria/blob/9351aa69ebc9ae52852311dea3f74cec28e67f6e/public/charges/helmetGreat.svg#L1-L3)
credits Paladium and a Commons source under BY-SA 4.0; its
[salmon](https://github.com/Azgaar/Armoria/blob/9351aa69ebc9ae52852311dea3f74cec28e67f6e/public/charges/salmon.svg#L1-L3)
credits Pierre Huttin under BY-SA 3.0; its
[Burgundy cross](https://github.com/Azgaar/Armoria/blob/9351aa69ebc9ae52852311dea3f74cec28e67f6e/public/charges/crossBurgundy.svg#L1-L3)
is labeled Azgaar CC0. These are discovery leads, not an approved import list.
Verify originals and follow the [source record requirements](ATTRIBUTION.md).

# Measured paint mixing

The studio offers ingredient controls and a constrained color picker over the
same `StockMix` recipes. The picker shows only supported paint families; it
does not fill the space between them with invented colors. A requested RGB
color is a search target, never the saved ingredient recipe. Base, shadow and
highlight each save an independent mixture. The existing sourced catalog
presets retain their estimated appearances, including the default German lion.

This is an empirical specimen-color model. It does not track an actual mixed
batch, consume paint during application, or calculate light transport through
ordered coats. Paint thickness currently affects surface relief without
determining hiding power. The [physical paint contract](../PHYSICAL_PAINT.md)
sets out the replacement model and its calibration requirements.

## Measurement basis

The included subset comes from Anna Sofia Reichert, Ana Belén
López-Baldomero, Francisco Moronta-Montero, Ana López-Montes, Eva María Valero,
and Carolina Cardell's [2025 supplementary dataset, version 3](https://doi.org/10.6084/m9.figshare.28639103.v3),
licensed [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/). Their
[methodology paper](https://doi.org/10.1007/s00216-025-05948-3) accompanies it.
These are modern reconstructions of historical materials, not samples taken
from a sixteenth-century shield.

All selected stocks use gum Arabic on parchment. The binder solution is
20 g gum in 100 mL water. The following preparation quantities do not specify
finished-stock yield or density:

| Stock | Pigment g | Gum solution mL | Measured white-stock shares |
| --- | ---: | ---: | --- |
| Lead white | 1.1 | 2.75 | Pure stock |
| Lead-tin yellow | 0.5 | 1.0 | 0%, 50%, 100% |
| Yellow ochre | 0.2 | 1.0 | Pure stock only |
| Azurite, extra fine | 1.0 | 0.5 | 0%, 50%, 100% |
| Cinnabar | 1.1 | 2.75 | 0%, 50%, 100% |
| Grape-seed black | 1.1 | 2.75 | 0%, 50%, 100% |

A midpoint combines 0.25 mL colored stock with 0.25 mL white stock. These are
**volumes of prepared paint**, not equal pigment masses. Grape-seed black is
not the catalog's vine-twig black or lampblack; cinnabar is not silently
substituted for every historical vermilion preparation. Egg glair samples in
the original dataset are not egg-yolk tempera and are not used here.

The [vendored data](data/measured-paint-data.json) retains specimen identities,
preparation text, source paths, SHA-256 checksums, reflectance means and spatial
standard deviations. Published means agree with means independently recomputed
from the selected 400-pixel source cubes within 2.3 × 10⁻⁷ reflectance. Spatial
spread within a specimen is not uncertainty across independently made batches.

## Forward model and limits

Each tint family interpolates reflectance piecewise through its actual
0%, 50% and 100% white measurements. Ochre and white are pure-only choices.
No other pigment pair, binder, layer order or extrapolation is supported.
The measured midpoint matters: this is not an RGB blend between two endpoints.
Interior interpolation is an empirical estimate, not an independently
validated mixing law or a reconstruction of absorption/scattering coefficients.

The model integrates reflectance under D65 with the CIE 1931 2° observer over
400–780 nm in 5 nm steps, normalized to a perfect diffuser's Y. This omits the
ends of the standard observer range. It converts XYZ to sRGB, clips to the
display gamut, then quantizes to eight bits. Color matching uses CIELAB of
those **display swatches** under D65, not spectral XYZ or a guarantee under
other lights. Both ΔE76 error and tolerance round to 10⁻⁶ before comparison.
The scene renderer still lights the resulting material separately.

The [CIE observer](https://doi.org/10.25039/CIE.DS.xvudnb9b) and
[D65 illuminant](https://doi.org/10.25039/CIE.DS.hjfjmt59) tables are by the
International Commission on Illumination, licensed
[CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/). The adapted
400–780 nm tables retain that license, separately from the pigment data.
The data metadata records the selection and truncation. Numerical color
conversions follow standard sRGB/CIELAB definitions; see the
[W3C color conversion reference](https://www.w3.org/TR/css-color-4/#color-conversion-code).

The black specimen is visibly gray in this preparation. Lead-tin yellow's
50% white sample is slightly darker than its pure sample. Neither result is
corrected to fit an expectation about pigment names. The authors' separate
CIELAB spreadsheet differs from straightforward integration of their mean
spectra by approximately 1.6–7 ΔE76 for selected pure stocks. That discrepancy
is unresolved; the implementation does not claim established predictive
accuracy.

No controlled thickness, opacity, dry-film yield, or shield-ground calibration
is supplied. Applying these colors to a shield is a reference-paint study;
the substrate transfer and gum-paint roughness remain estimates. The measured
set is a small available calibration, not the boundary of all colors possible
in Germany in 1544. Cloth dyeing requires a separate fiber/mordant/process
model; these paint stocks cannot serve as dye concentrations.

## Deterministic recipe search and cost

`SearchRequest::solve` checks 4,002 recipes: four families at 0–99.9% white
in 0.1% steps, plus pure white and pure ochre. It excludes unavailable stocks
only when their required quantity is nonzero. It excludes batches above an
optional budget. Among recipes within tolerance, it chooses lowest cost,
then lowest color error, then stable stock/share order. If none matches, it
explicitly returns the nearest feasible recipe, with cost as the secondary
tie-breaker. If none is feasible, it returns no result.

This is a complete finite search with deterministic tie-breaking. It proves
optimality only for the supported quantized recipes and supplied costs,
not for arbitrary continuous mixtures or all historical materials.

The workshop prices each prepared stock in illustrative cost units per mL.
All prices start equal; users supply their own availability and price scenario.
A batch volume is stored in microliters. Cost is calculated exactly in integer
millionths of a cost unit from component volume times price, plus a fixed
preparation cost. The binder is already in the prepared stock. Costs are not
historical currency, calibrated rarity, dry-pigment grams, or an automatic
bill for an entire shield. Each tone's quote describes the selected batch;
unused palette entries do not become consumption.

The editor keeps the workshop scenario for the session. Applying saves only
the three ingredient recipes in the document. The CLI saves a reproducible
request and result, including prices, availability, quantities and calibration
identity. Future gameplay integration must account for actual applied area,
waste, layers, labor, local supply and repairs; see
[GAMEPLAY_INTEGRATION.md](../GAMEPLAY_INTEGRATION.md).

## Interface and export

Open **Paint recipes → Open measured paint mixer** for a tincture. Select a
tone, adjust the stock/white slider or click a reachable-color strip, and
review its batch. To match a target, set the requested color and tolerance,
then choose **Find least-cost recipe**. An out-of-tolerance result requires
explicitly choosing the nearest color. **Apply all three paint recipes**
updates the tincture, including the initial nearest-color proposals for any
other tones. Opening or closing the mixer does not alter the document.

A saved selection has this shape:

```json
{
  "kind": "Mixed",
  "paint": {
    "base": {"stock": "Azurite", "white_permille": 250},
    "shadow": {"stock": "Azurite", "white_permille": 0},
    "highlight": {"stock": "Azurite", "white_permille": 750}
  }
}
```

For a repeatable cost search:

```sh
cargo run -p adventuresim-heraldry-studio --bin heraldry-lab -- mix \
  > target/heraldry/mix-request.json
# Edit target_srgb, tolerance and workshop in that file.
cargo run -p adventuresim-heraldry-studio --bin heraldry-lab -- \
  mix target/heraldry/mix-request.json > target/heraldry/mix-result.json
```

The same portable function is exported to WASM as `solve_heraldry_paint`.
Mixed-paint bundles include `paint-calibration.json`. Their recipe manifests
and standalone GLB extras retain source credits, license links and calibration
hashes. The calibration also participates in the bake stamp. The data's
attribution and share-alike notices travel with the adapted tables; they do
not relicense the generator's code.

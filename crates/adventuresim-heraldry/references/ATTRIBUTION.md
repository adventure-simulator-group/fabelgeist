# German lion source and adaptation

[*Lion Rampant Or (16th century German)*](https://commons.wikimedia.org/wiki/File:Lion_Rampant_Or_(16th_century_German).svg)
is by Tom Lemmens (Tom-L), uploaded 28 September 2013, after Rinaldum's
[*Héraldique meuble lion rampant 02*](https://commons.wikimedia.org/wiki/File:H%C3%A9raldique_meuble_lion_rampant_02.svg),
dated 8 December 2009. We use both contributions under
[Creative Commons Attribution-ShareAlike 3.0 Unported](https://creativecommons.org/licenses/by-sa/3.0/).
Rinaldum also offers GFDL; this adaptation uses the CC BY-SA license.

`German_Lion_1530.svg` is the unmodified Tom-L download. Adapted lion artwork
remains under CC BY-SA 3.0; this notice does not relicense the software.

- Source canvas: 326.7587 × 367.24664 units.
- Size: 60,067 bytes.
- SHA-256:
  `206a2186210fd5957c6dd9749dd6d0ffedcb82ebff7eb5eee126d26b5a2ca672`
- Original contributor pages:
  [Tom-L](https://commons.wikimedia.org/wiki/User:Tom-L),
  [Rinaldum](https://commons.wikimedia.org/wiki/User:Rinaldum).

## Construction inspected

The SVG contains 79 paths, one line and two groups. It uses solid fills and
strokes, without gradients. The main body, mane, legs and tail share compound
contours; the raised foreleg is in a separate group. This structure differs
from a drawing with one independently movable path per body part.

The loader resolves SVG transforms and path commands through `usvg`. The
parametric attachment chart applies one continuous spatial deformation to
fills, outlines and painted marks. Controls alter body width, spine arch,
head, paws, limbs, mane and tail without detaching interior lines. Neutral
proportions preserve the source geometry. A clipped copy of the tail supports
the two-tail variant; facing and composition also transform its clip.

The palette identifies these source roles:

| Source color | Role in the adaptation |
| --- | --- |
| `#ffdb43` | Main charge tincture. |
| `#dbba2e` | Full silhouette plus painted shadow underpainting. |
| `#ffea80` | Painted highlights. |
| `#9e7800` | Interior drawing and outlines. |
| `#e94545` | Armed claws and langued tongue, separately editable. |
| `#ffffff` | Teeth and eye white. |
| `#000000` | Pupil. |

The dark underpainting also carries the complete silhouette. Flat paint
therefore recolors it to the main tincture instead of deleting it. Shadow and
highlight coverage can be adjusted independently. The Modeled preset applies
restrained values for both. Source strokes remain in both treatments; no extra
mane hatching is invented.

Fabelgeist contributors modify proportions, tinctures, painted tonal strength,
line widths, tail count, crowns and composition. These changes are applied at
runtime; the vendored source remains intact. Shadow and highlight pigments
follow counterchanged tinctures. Their coverage is baked into color and
material response independently of camera or light direction. Over leaf,
painted marks preserve its relief and partially cover its metallic response.

## Historical basis and limits

The drawing is based on the Bavarian State Library's
[*Sammelband mehrerer Wappenbücher*, BSB Cod.icon. 391](https://www.digitale-sammlungen.de/en/view/bsb00007681),
catalogued as southern Germany (Augsburg?), c. 1530. Rinaldum's Commons source
link identifies fol. 177r (scan 355); the SVG's original document name instead
identifies fol. 286v (scan 574). Both pages were checked.

- [Fol. 177r, official scan](https://api.digitale-sammlungen.de/iiif/image/v2/bsb00007681_00355/full/full/0/default.jpg):
  a line-drawn lion uses hatching to describe shoulder and hip form.
- [Fol. 286v, official scan](https://api.digitale-sammlungen.de/iiif/image/v2/bsb00007681_00574/full/full/0/default.jpg):
  a black lion has lighter painted modeling across the mane, shoulder and
  limbs, alongside internal line work.

This supports pictorial tonal modeling near the setting's place and date.
It does not establish how a corresponding physical shield was decorated.
The [painted shading study](PAINTED_SHADING.md) examines independent material
evidence and the limits of translating illustrations into physical finishes.
Tom-L's gold palette and specific highlight shapes are a modern interpretation;
our pigment mixtures and coverage strengths are also authored, not a
reconstruction of the manuscript's materials or colors.

Credit and the modification notice accompany lion exports in `ATTRIBUTION.txt`,
SVG metadata and GLB copyright, including nested or quartered lion charges.

## Adding other source artwork

Search Wikimedia Commons before authoring new designs. Existing non-geometric
SVGs with suitable licenses may be adapted; newly invented non-geometric
SVGs and text generation remain deferred. Inspect the actual vector structure,
including compound contours, interior strokes, accents and painted tones,
before choosing a source. A silhouette preview is insufficient.

Commons is the preferred discovery source for period designs. Follow its
historical references to the museum, library or archive record and inspect
the underlying image. Armoria and other collections may supply candidates
when the same evidence can be recovered for the individual design.

Keep digital attribution separate from historical provenance. For a design
presented as a period reference, record the historical model's date and place,
institution, accession number or manuscript shelfmark, and page or folio where
available. Link the catalogue record and image, and note attribution or dating
uncertainty. Record the modern SVG's creator and upload date separately. The
age of a coat of arms or blazon does not establish the age of a particular
drawing style. A modern redraw can preserve a documented period model while
adding colors, shading or details; identify those adaptations explicitly.
When the model cannot be traced, keep the asset as a discovery lead rather
than labeling its drawing period-verified.

For each adopted asset, retain the original file and record its title, creator
and adaptation chain, source page, selected license and version, file hash,
historical basis, and our modifications. Preserve meaningful paint regions
when normalizing artwork. Credits and applicable license notices must follow
derived exports as they do for the lion.

A project's software license does not describe every bundled image. Check the
individual artwork and its original source: Armoria, for example, distributes
MIT code alongside CC0, CC BY-SA, CC BY-NC-SA and other charge assets. Do not
import its collection wholesale or treat a NonCommercial asset as equivalent
to our BY-SA lion. See the [prior-art review](PRIOR_ART.md) for pinned examples
and license references.

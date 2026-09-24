# Heraldic drawing and painted construction references

The recipes study charge structure from Central European prints and armorials
made before 1544. Drawings inform shape, line work and painted modeling;
technical examination of objects informs physical construction.
The recipes are parametric interpretations, not facsimiles or authenticated
reconstructions. Their tincture palettes are authored choices where a print
does not establish color.

| Recipe | Source | Construction studied |
| --- | --- | --- |
| `german-lion` | [*Sammelband mehrerer Wappenbücher*, c. 1530, BSB Cod.icon. 391](https://www.digitale-sammlungen.de/en/view/bsb00007681) | German rampant lion, internal line work and painted tonal modeling; modern Tom-L/Rinaldum vector adaptation. |
| `durer-lion` | [Albrecht Dürer, *Coat of Arms with a Lion and a Cock*, c. 1502, The Met](https://www.metmuseum.org/art/collection/search/391113) | Rampant pose, flexed spine, curled mane, articulated paws and looped tail. |
| `woensam-lions` | [Anton Woensam, coat of arms, 1530, British Museum, 1900,1019.79](https://www.britishmuseum.org/collection/object/P_1900-1019-79) | Opposed rampant lions with contrasting tinctures, narrower bodies and reaching forelegs. |
| `wernigerode-eagle` | [Tom-L/Heralder single eagle](https://commons.wikimedia.org/wiki/File:Arms_of_the_King_of_the_Romans_(c.1433-1486).svg) | Modern single-headed variation in the Wernigerode double eagle's style. |
| `wernigerode-double-eagle` | [Wernigerode armorial, c. 1475–1500, BSB Cod.icon. 308 n](https://www.digitale-sammlungen.de/en/view/bsb00043104) | Traced and symmetrized modern vector adaptation; red halos replace the manuscript's head crowns. |

Only the shield charges are in scope. Helmets, crests, supporters, mantling,
lettering, and frames surrounding the source shields are not generated. The
lion family is a profile interpretation; it does not reproduce the turned head
and every engraved fur line in Dürer's print. Spine, limb, mane and tail
controls deform the same sourced lion across its recipes. The quartered
example combines lions and lozenges; the counterchanged example uses a lion.

The [eagle source record](references/EAGLES.md) documents the two Commons
files, inspected construction, exact hashes, license and adaptations.

## Wikimedia Commons drawing and period paint

The lion uses [Tom-L's *Lion Rampant Or (16th century German)*](https://commons.wikimedia.org/wiki/File:Lion_Rampant_Or_(16th_century_German).svg),
after [Rinaldum's unshaded original](https://commons.wikimedia.org/wiki/File:H%C3%A9raldique_meuble_lion_rampant_02.svg).
Both SVGs were downloaded and their construction inspected. We use CC BY-SA
3.0, with credits and modifications accompanying exported artwork. The
[construction record](references/ATTRIBUTION.md) identifies the exact download,
its compound contours, source paint roles and parametric adaptation.

The underlying armorial is catalogued as southern Germany (Augsburg?), c. 1530.
Rinaldum's Commons source link points to fol. 177r; its SVG document name names
fol. 286v. The official scans provide direct visual evidence:

- [Fol. 177r (scan 355)](https://api.digitale-sammlungen.de/iiif/image/v2/bsb00007681_00355/full/full/0/default.jpg)
  includes a line-drawn rampant lion with hatching on its shoulder and hip.
- [Fol. 286v (scan 574)](https://api.digitale-sammlungen.de/iiif/image/v2/bsb00007681_00574/full/full/0/default.jpg)
  shows lighter painted modeling on a black lion's mane, shoulder and limbs.

These pages establish pictorial modeling near the setting's place and date;
they do not establish the manufacture of a corresponding physical shield.
The [painted shading study](references/PAINTED_SHADING.md) distinguishes this
evidence from surviving painted objects, real relief, and later repainting.
Tom-L's precise gold coloring and highlight shapes are a modern interpretation.
Flat retains the source's interior lines with flat tinctures. Modeled adds
restrained painted tones with separate shadow and highlight coverage controls.
Asymmetry is independent. Neither paint choice claims a uniquely authentic
historical finish.
These tones belong to the painted material, independent of the renderer's
lighting. Over leaf, partial pigment coverage preserves the underlying relief.

Existing non-geometric SVGs with suitable licenses may extend the charge
collection. Search Commons first, inspect construction and provenance, and
retain the source's attribution chain. Newly invented non-geometric SVGs and
text generation remain deferred. The [source record](references/ATTRIBUTION.md)
describes the required attribution and adaptation notes.

## Procedural tools and asset workflows

[Armoria and Fantasy Map Generator](references/PRIOR_ART.md) are engineering
references for imported charge assets, constrained composition, repeated
patterns, related heraldic identities, and local material prices. Their
generation weights and drawing styles do not establish historical practice.
The review pins inspected source revisions and distinguishes MIT code from
individual artwork licenses; no upstream code or charge collection has been
incorporated through that review.

Before implementing non-trivial patterns, consult existing work in Houdini
and Shadertoy communities and record the selected technique and its source.
Technical reuse still requires separate evidence for a pattern's historical
application and material construction.

## Historical interpretation and economics

G. W. Eve, *Heraldry as Art: An Account of Its Development and Practice,
Chiefly in England*. London: B. T. Batsford, 1907.
[Full text, Project Gutenberg](https://www.gutenberg.org/files/69298/69298-h/69298-h.htm).
Chapter II, p. 29, conjectures that ordinary working shields generally used
flat painting and that physical relief was exceptional partly because of
cost and difficulty of repair. Chapter VIII, p. 164 onward, discusses painting
methods and materials. This is a historical interpretation,
not contemporary testimony or quantitative evidence for German practice in
1544. The [painted shading study](references/PAINTED_SHADING.md) evaluates
material evidence without treating exceptional objects as a frequency survey.

The [gameplay integration requirements](GAMEPLAY_INTEGRATION.md) adopt the
material and labor principle as a design requirement: production and repair
constraints should influence player choices. Painted modeling and physical
relief require separate cost models. Numerical costs and prevalence remain
to be researched and calibrated.

## Paint recipes

The [paint catalog](references/PAINT_RECIPES.md) records the evidence for each
pigment and binder selection, separately from its estimated rendering values.
Alongside the Behaim conservation report below, it uses Cennino Cennini,
*Il libro dell'arte*, c. 1400, translated by Daniel V. Thompson Jr. as
*The Craftsman's Handbook* (Yale University Press, 1933; Dover reprint).
[Pigment chapters XXXVII–LIX](https://noteaccess.com/Texts/Cennini/2.htm) and
[painting chapters CXLIV–CXLV](https://noteaccess.com/Texts/Cennini/6TM.htm)
provide comparative Italian workshop evidence. These recipes do not establish
their frequency on German heraldic shields in 1544.

## Physical surface evidence

[Christel Faltermeier and Rudolf Meyer, *Appendix: Notes on the Restoration of
the Behaim Shields*, Metropolitan Museum Journal 30 (1995), pp. 53–60](https://resources.metmuseum.org/resources/metpublications/pdf/Appendix_Notes_on_the_Restoration_of_the_Behaim_Shields_The_Metropolitan_Museum_Journal_v_30_1995.pdf)
provides the technical evidence: wood supports, canvas or skin coverings,
prepared grounds, tempera, silver leaf, and resinous glazes. Page 56 describes
oil-adhered silver covered with yellow glaze to resemble gold. Page 58 records
raised wax/resin letters covered with silver and yellow glaze. Note 2 on
pp. 59–60 distinguishes burnished water gilding, unburnished oil gilding, and
raised mordant work. This is the conservation appendix, separate from Helmut
Nickel's accompanying historical article. Later overpainting and present
weathering are outside the intact finish modeled here.

[The National Gallery's technical account of the Wilton Diptych](https://www.nationalgallery.org.uk/paintings/catalogues/national-gallery-2024/the-wilton-diptych)
documents burnished water gilding over bole and contrasting matte mordant
gilding on its armorial decoration. Its mordant contains protein; mordant is a
broader adhesive category, not a synonym for oil. Our `MordantGilding` choice
specifically models the raised wax/resin application described for the Behaim
shields. The English diptych is comparative evidence, not a Nuremberg recipe.

[National Gallery Technical Bulletin 18, *Methods and Materials of Northern
European Painting, 1400–1550* (1997)](https://www.nationalgallery.org.uk/media/15643/methods_and_materials1997.pdf)
also discusses German water gilding and the early sixteenth-century
*Liber illuministarum*: prepared chalk/glue grounds, a polished iron-oxide
layer, size, leaf, and burnishing. This establishes the method within the
region and period informing the modeler.

[Qing Wu et al., *Does substrate colour affect the visual appearance of gilded
medieval sculptures? Part I*, Heritage Science 8, 118 (2020)](https://www.nature.com/articles/s40494-020-00463-3)
examines modern reconstruction samples. Burnishing and substrate texture
affect appearance, while substrate color is not perceptible through intact
gold at least 100 nm thick. Accordingly, the generator changes roughness and
relief when burnishing; it does not tint intact gold with an imagined bole
color or darken base color to imitate a reflection.

## Rendering interpretation and limits

Water gilding smooths prepared relief and lowers perceptual roughness. Oil
gilding retains a softer reflection and fine adhesive texture. Raised mordant
adds local buildup following the gilded shape. Yellow-glazed silver filters
silver reflectance and adds a neutral dielectric highlight through a coating
mask. Uncoated leaf has no square-by-square color or roughness modulation.
Neither a 70 mm sheet grid nor conspicuous joints are inferred from the sources.
Gold defaults to uncoated; a whole-face clear coating remains an explicit
artist control.

The linear RGB conductor values follow the
[PlayCanvas material table](https://developer.playcanvas.com/user-manual/graphics/physical-rendering/physical-materials/):
gold `[1.0, 0.766, 0.336]`, silver `[0.972, 0.960, 0.915]`. These are general
rendering reference values, not assays of historical leaf. The flat artwork
and pigment treatments use the selected paint palette.

Thicknesses, roughness coefficients, glaze absorption, brush pitch, and relief
amplitudes are authored controls and approximations, not measurements extracted
from museum objects. In particular, yellow glaze uses a fixed RGB absorption
profile and a doubled optical path through a neutral clearcoat approximation;
it is not a spectral or angle-dependent coating simulation. The height map
describes visible relative relief rather than a full stratigraphic model.
Microscopic features are footprint-filtered; unresolved texture is represented
by the technique's roughness. Normal slopes use physical units. Light and
reflections belong to the renderer.

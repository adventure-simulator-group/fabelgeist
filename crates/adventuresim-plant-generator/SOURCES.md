# Procedural plant source references

## Scope and interpretation

The flower presets represent a small, extensible sample of plants appropriate
to central Germany in 1544. They are not an exhaustive regional flora or a
reconstruction of a particular historical site. Modern German occurrence and
native status support ecological plausibility; contemporary herbals provide
additional period evidence. Neither establishes presence at every location.

Botanical sources guide recognizable structure, scale, habitat, and season.
Preset dimensions, colors, organ counts, and deformation amplitudes are authored
approximations for rendering, not measurements copied from a botanical survey.
Random variation should preserve the structural traits that identify a preset.

The procedural references below describe prior approaches. They do not establish
that this crate implements their algorithms. The chosen mesh construction,
tessellation, and parameter organization are independent implementation choices.
No source code or licensed Houdini assets are copied or bundled from these
references; downloadable scene files remain external study material.

## Flower generation

- [Danny Laursen: Blooming Flowers, SideFX, 2024](https://www.sidefx.com/tutorials/blooming-flowers-part-one-procedural-petal-model-and-rig/).
  The tutorial develops a procedural petal, then expands it into an
  art-directable flower setup. It also covers orientation, rigging, and Vellum.
  The transferable design idea is to reuse one configurable organ surface.
  A runtime cloth or growth simulation is not required by that design idea.
- [Ijiri, Owada, Okabe, and Igarashi: Floral Diagrams and Inflorescences, 2005](https://takashiijiri.com/projects/ProjTakaFlower/index.html).
  This SIGGRAPH paper separates component geometry from the structure of a
  flower and from arrangements of multiple flowers. It supports keeping organ
  shape, floral layout, and flowering-stem layout independently configurable.
  Its interactive sketch editor is not part of this crate.
- [Prusinkiewicz and Hanan: A Collision-based Model of Spiral Phyllotaxis, 1992](https://algorithmicbotany.org/papers/phyllo.sig92.html).
  The paper constructs spiral arrangements by detecting and eliminating
  collisions while optimizing organ packing. It is a reference for dense
  floral centers. A deterministic spiral placement is a rendering
  approximation, not a reproduction of the collision-based model.
- [Jayelinda Suridge: Modelling by Numbers: Part Two B, 2013](https://www.gamedeveloper.com/business/modelling-by-numbers-part-two-b).
  This original procedural geometry tutorial demonstrates curved stems,
  reusable petal and sepal surfaces, radial repetition, and adjustable mesh
  resolution. It provides a practical precedent for composing multiple plant
  forms from shared geometry operations.

These sources motivate a shared parametric system rather than a mesh-building
function per species. The crate distinguishes free floral organs, ray-and-disk
heads, and fused bell corollas. These are rendering categories: wood-anemone
showy organs are tepals, and a daisy head contains many individual florets.
The generic parameter name `petals` does not change those botanical
distinctions.

## Flower presets and botanical references

[FloraWeb](https://www.floraweb.de/) is the German Federal Agency for Nature
Conservation's information service for wild ferns and flowering plants. The
linked species accounts provide morphology, flowering months, habitat, and
German floristic status. Their distribution maps describe modern records, not
a map of Germany in 1544.

| Preset | Botanical reference | Traits relevant to the model |
| --- | --- | --- |
| Common daisy, *Bellis perennis* | [FloraWeb, 814](https://www.floraweb.de/php/artenhome.php?suchnr=814) | Basal leaf rosette; solitary 10–30 mm head; white rays and yellow disk; fresh meadows and pasture. |
| Meadow buttercup, *Ranunculus acris* | [FloraWeb, 4690](https://www.floraweb.de/php/artenhome.php?suchnr=4690) | Native German buttercup reference for the yellow open-flower preset. |
| Wood anemone, *Anemone nemorosa* | [FloraWeb, 20424](https://www.floraweb.de/php/artenhome.php?suchnr=20424) | Native German woodland-flower reference; the showy organs are tepals. |
| Corn poppy, *Papaver rhoeas* | [FloraWeb, 4115](https://www.floraweb.de/php/artenhome.php?name-use-id=4115) | Four broad red petals, commonly with dark basal markings; arable fields and disturbed ground. |
| Nettle-leaved bellflower, *Campanula trachelium* | [FloraWeb, 1083](https://www.floraweb.de/php/artenhome.php?name-use-id=1083) | Fused 30–40 mm bell corollas; flowers along a stem; mixed woodland, edges, and hedges. |

The shared shapes simplify fine features such as hair, leaf dissection,
reproductive structures, and natural variation in flower orientation. Species
names identify the reference and intended appearance; they do not claim that
the generated meshes are suitable for botanical identification.

## Evidence for the 1544 setting

[The University of Tübingen's edition of Fuchs's 1543 Kräuterbuch](https://vergil.uni-tuebingen.de/publikationen/FuchsKraeuter/capitel/inhaltpflanzenlateinisch.html)
provides a modern Latin cross-index to the period text and illustrations.
Relevant entries include common daisy (chapter 53, illustration 80), wood
anemone (57, 90), corn poppy (195, 291), and nettle-leaved bellflower
(164, 242).
This is direct evidence that these plants were described in a contemporary
German herbal, with modern species identifications supplied by its editors.

[Chapter 53, on daisies](https://vergil.uni-tuebingen.de/publikationen/FuchsKraeuter/capitel/053.html)
distinguishes wild daisies from cultivated forms and discusses oxeye daisies
in meadows. It supports the use of wild forms while keeping garden varieties
as a separate future authoring choice.

The meadow-buttercup preset is supported here by its modern native status and
ecology, not by a verified species-level identification in Fuchs. Extending the
catalog should use the same distinction between direct historical evidence and
an inference from long-established regional occurrence. Inclusion in a herbal
alone is insufficient grounds for scattering a plant into every wild habitat.

## Fungi generation

- [Jayelinda Suridge: Modelling by Numbers: Part Two B, 2013](https://www.gamedeveloper.com/business/modelling-by-numbers-part-two-b).
  The mushroom example combines a curved stem with a cap generated from a
  cubic Bezier profile. A related profile defines the underside, and segment
  counts control mesh resolution. This is a direct precedent for a shared
  cap-and-stipe parameter system rather than a fixed hemispherical cap.
- [Konstantin Magnus: Procedural Mushroom, 2022](https://procegen.konstantinmagnus.de/procedural-mushroom).
  This Houdini example wraps a grid onto a displaced torus, remeshes it, and
  connects boundary samples with shortest surface paths before converting
  curves to volumes. It suggests generating underside detail in relation to
  the cap surface. Its downloadable scene remains external reference material.
- [Tim Zarki: Gills](https://www.zarki.net/work/gill).
  The artist describes profile ramps, soft-body deformation, space-colonization
  gills, and conversion through VDBs to a mesh with vertex colors. The useful
  principles are profile control and correlated variation across surfaces.
- [Junichiro Horikawa: Algorithmic Live, Mushroom Gills, SideFX, 2021](https://www.sidefx.com/tutorials/houdini-algorithmic-live-mushroom-gills/).
  This dedicated procedural gill tutorial supplies further artist prior art
  for the underside as a separately controlled structure.
- [Kara Johnson: Procedural Mushroom Generator](https://vimeo.com/886541907).
  The demonstrated Houdini tool combines an editable curve with configurable
  cap and stem properties, accessories, and asymmetric noise. It is another
  example of multiple mushroom forms sharing one set of controls.

These references motivate shared radial profiles and independently configured
undersides. They do not prescribe the crate's exact equations, mesh topology,
or tessellation limits. A bounded direct mesh implementation need not reproduce
the tutorials' simulations, VDB operations, or surface-path algorithms. The
original implementations and downloadable assets are not copied or bundled.

## Fungal morphology references

The following accounts describe the exact species used by the presets. DGfM
is the German mycological society. Dieter Gewalt's Fundkorb portraits provide
original field photographs, descriptions, and observations from Germany.
These are visual and ecological references; authored mesh dimensions, colors,
and detail budgets remain approximations rather than measured reconstructions.

### Fly agaric: Amanita muscaria

[DGfM: Pilz des Jahres 2022, Fliegenpilz](https://www.dgfm-ev.de/de/ueber-uns/news/pilz-des-jahres-2022-fliegenpilz)
describes the red cap with white veil remnants, white stipe with a hanging
ring, and warty belts around the swollen base. It identifies the species as a
gilled mushroom and describes its association with deciduous and coniferous
trees. Its photographs distinguish the cap patches from pigment spots and
show the basal remains of the universal veil.

### Porcini: Boletus edulis

[Dieter Gewalt: Boletus edulis, Gemeiner Steinpilz](https://fundkorb.de/pilze/boletus-edulis-gemeiner-steinpilz)
explicitly distinguishes this species from the summer, bronze, and pine
boletes. It describes an ochre-brown cap, often slightly glossy and paler at
the rim, and a pale stipe with fine white reticulation toward the top. Young
stipes can be stout; older ones may be more slender. The tube layer beneath
the cap changes from white through yellow to olive as it matures.

The preset's underside must read as a pore-bearing tube surface, not radial
gills. The account records both spruce and broadleaved woodland, specifically
including beech. The bronze bolete, *Boletus aereus*, is not a substitute
morphological reference for this preset.

### Chanterelle: Cantharellus cibarius

[Dieter Gewalt: Cantharellus cibarius, Pfifferling](https://fundkorb.de/pilze/cantharellus-cibarius-pfifferling)
includes original Rhein-Main observations and an underside comparison with
the false chanterelle. Chanterelle ridges are fleshy folds continuous with
the cap, unlike thin blade-shaped gills. The account also distinguishes
*C. cibarius* from the thinner, more orange *C. friesii* and pale *C. pallens*.

[DGfM's illustrated chanterelle comparison](https://www.dgfm-ev.de/jugend-und-nachwuchs/pilze-schule-kiga/klasse-1-bis-3?name=Raetsel3-Pfifferling-und-Raukopf.pdf&reattachment=51cd3a749dc32af70b0d7450bb662223)
independently emphasizes that *C. cibarius* has ridges rather than papery
gills, and lacks a veil. The generator should preserve a stout, fleshy form
and blunt underside ridges rather than turning every chanterelle into a thin
trumpet or an orange gilled mushroom.

### Common puffball: Lycoperdon perlatum

[Dieter Gewalt: Lycoperdon perlatum, Flaschenstaubling](https://fundkorb.de/pilze/lycoperdon-perlatum-flaschenst%C3%A4ubling)
includes original photographs from Zeppelinheim and describes fruiting bodies
about 3–8 cm high, with a rounded upper body and a narrower sterile base.
Young bodies are white, becoming brown as spores mature. Pointed surface
warts can rub off, leaving a fine reticulate pattern. A small opening forms
at the apex at maturity.

The reference form is therefore a closed fruiting body with surface ornament,
not a cap suspended over exposed gills or pores. The mature apical opening is
distinct from the many underside pores of a bolete.

## Fungi and the 1544 scope

These four presets are a representative sample of ground-fruiting fungi,
not a complete central German fungal inventory. Their suitability for 1544
is inferred from established regional occurrence and ecology. No direct
species-level record from 1544 is established by these references, and the
flower evidence from Fuchs must not be extended to fungi without verification.

DGfM documents the fly agaric's widespread German occurrence. The exact-species
Fundkorb accounts supply German observations for porcini, chanterelle, and
common puffball. The German Red List accounts for
[Cantharellus cibarius](https://www.rote-liste-zentrum.de/detailseite/?species_uuid=e4df6d5e-21eb-409a-bfd7-cbe499084d2c)
and
[Lycoperdon perlatum](https://www.rote-liste-zentrum.de/detailseite/?species_uuid=7cd4a101-b19d-49ce-89f7-eeb5f7cd862f)
classify their establishment status as indigenous or ancient introduction.
These sources support historical plausibility without establishing exact
locations or abundance in the game's period.

Modern records involving introduced hosts, such as red-oak plantations in the
chanterelle account, are not historical planting references. Seasonal windows
and habitat weights are conservative placement choices, separate from organ
geometry. They approximate opportunities for fruiting and do not simulate
mycelial growth or guarantee fruiting on every suitable date.

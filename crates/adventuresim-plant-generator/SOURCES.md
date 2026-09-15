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

# Armor references and validation

Recipe families and style controls are described in the
[crate README](../README.md).
The
[creator README](../../adventuresim-character-creator/README.md#parametric-armor-authoring)
documents design overrides, filtered exports, GLB audits, static renders, and
native armor capture fixtures.

## Historical shape references


Reference examples include the Met's
[morion](https://www.metmuseum.org/art/collection/search/27150),
[barbute](https://www.metmuseum.org/art/collection/search/27960) and
[greaves](https://www.metmuseum.org/art/collection/search/22970), Cleveland's
[kettle hat](https://www.clevelandart.org/art/1916.1919) and
[open burgonet](https://www.clevelandart.org/art/1916.1642), Fitzwilliam's
[cuisses and poleyns](https://data.fitzmuseum.cam.ac.uk/id/object/17761) and
[mitten gauntlets](https://data.fitzmuseum.cam.ac.uk/id/object/17694), the
[Visby coif study](https://www.djurfeldt.com/patrik/mailcoif.html), and
[Leicestershire's working leather boot](https://leicestershirecollections.org.uk/archaeology/medieval-coal-mining).
The arming-cap reference was a museum reconstruction, not an original-period
survival. The references support construction families rather than a claim
that the combined outfits reproduce one surviving historical harness.

## Item-specific guides

- [Breastplate profiles, recipes, and references](breastplate/README.md).
- [Close-helmet construction and export checks](close-helmet/README.md).
- [Close-helmet morph testing](close-helmet/morph-testing.md).

## Reproduce validation

Keep generated renders and review reports under `target/armor-review/`. Record
the asset hashes, body recipe, design parameters, and camera settings with each
local capture so a later run can reproduce it.

The boot-layering checker tests both legs against the supported default mail
and padded chausses. Run from the repository root with Blender on `PATH`:

```powershell
blender --background --python scripts/check_boot_layering.py -- assets/equipment/procedural target/armor-review/boot-layering.json
```

Custom garment dimensions and animated poses need their own fit review.
Numerical geometry checks, visual fit, and animation validation cover different
properties; static renders do not establish runtime acceptance.

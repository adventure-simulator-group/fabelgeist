# Close-helmet construction and validation

[The morph-testing guide](morph-testing.md) describes the configuration sweep,
checks, and their limits.

## Historical shape reference

The current target is The Metropolitan Museum of Art's
[Close Helmet, collection object 25397](https://www.metmuseum.org/art/collection/search/25397),
German, ca. 1500, accession 04.3.243. The museum's photographs substantiate the
modeled design's principal silhouette, proportions, and plate arrangement.
Museum reference views are
[quarter](https://collectionapi.metmuseum.org/api/collection/v1/iiif/25397/1483089/main-image),
[front](https://collectionapi.metmuseum.org/api/collection/v1/iiif/25397/1483085/main-image),
[side](https://collectionapi.metmuseum.org/api/collection/v1/iiif/25397/1483086/main-image),
[rear](https://collectionapi.metmuseum.org/api/collection/v1/iiif/25397/1483087/main-image),
and
[opposite side](https://collectionapi.metmuseum.org/api/collection/v1/iiif/25397/1483088/main-image).
Source image URLs are also retained in
[the source manifest](references/sources.json).

Acceptance concerns this photographed design adapted to the shown wearer:
rounded enclosing skull, relatively straight cheeks and supported jaw, a visor
whose lower boundary rises toward the temples, an inward nape transition, and
an outward descending neck defense. Category resemblance alone is insufficient.
Rims, rivets, decoration, materials, and minor opening/surface details are outside
this coarse milestone.

Keep generated renders, matched bare-body views, sections, and review reports
under
`target/helmet-review/`.

## Geometry and fitting contract

The assembly exports three component meshes: skull, bevor, and shield-shaped
visor. The skull component includes three overlapping nape lames. The
experimental rim was removed. Shared hinge metadata for the visor/bevor is
stored in reference-body coordinates and retained in glTF node extras; it does
not implement articulation or visor animation.

Independent skull, temple, jaw, lower-neck, and nape-waist measurements
establish anatomical bounds. Temple clearance defaults to 5 mm independently of
the 10 mm cranial reserve. Jaw/neck widths default to 840/800 permille of the
temple envelope as minimum style widths; wider measured anatomy wins. The
posterior blend is independent of the ear-width hold. For a fixed design,
fitting changes coordinates while keeping connectivity, component ranges, and
morph correspondence fixed. Adding/removing nape lames through a design change
is a topology change, not a wearer-dependent operation.

Sight span/gap/bridge and breathing count, rows, size, span, location, rotation,
rounding, and side are adjustable. Aperture lengths expressed as `Millimeters`
refer to the authored reference surface; fitting scales and curves their actual
physical dimensions. Pairwise hole and plate-boundary clearance checks operate
in that domain. The openings are through holes with return walls. Plate gauge
is limited to 1–4 mm. Layer offsets follow the carrier normal, while visor
`ShellExtrusion::InPlane` returns preserve horizontal sight rays and gauge along
the source vertex normal. Hard-normal vertex duplicates meet at coincident
physical seams. These local contracts do not guarantee continuous thickness or
clearance on every fitted surface.

## Reproduce the export and checks

Run from the repository root with pinned MHR assets, the canonical body GLB at
`assets/animations/biped/unarmed/base.glb`, Python/NumPy, and Blender on `PATH`.
Use a fresh empty staging directory. These commands reproduce an export and
checks; they do not install or overwrite runtime assets.

```powershell
cargo run --manifest-path crates/adventuresim-character-creator/Cargo.toml -- --generate-equipment --equipment-item close_helmet --lod 1 --recipe assets_src/characters/mhr_base.json --armor-designs crates/adventuresim-armor-model/review/close-helmet/designs/default.json --equipment-output target/helmet-review/reproduce
python scripts/check_parametric_armor_assets.py target/helmet-review/reproduce --allow-partial
blender --background --python-exit-code 1 --python scripts/check_close_helmet_assets.py -- target/helmet-review/reproduce/close_helmet--worn.glb assets/animations/biped/unarmed/base.glb target/helmet-review/reproduce/plate-checks.json
python scripts/export_armor_morph_review.py target/helmet-review/reproduce target/helmet-review/reproduce-bodies
foreach ($blend in @('neutral', 'positive', 'negative', 'mixed')) {
    blender --background --python scripts/check_armor_review.py -- "target/helmet-review/reproduce-bodies/$blend"
}
Get-FileHash -Algorithm SHA256 target/helmet-review/reproduce/close_helmet--worn.glb
cargo test -p adventuresim-armor-model
just fmt-check
just lint
```

`--allow-partial` is intentional for this single-item export. The morph-review
script reads the explicitly supplied staging directory. Render any resulting
body directory with `scripts/render_armor_review.py` through Blender; compare
the generated asset hashes with the installed asset before comparing results.

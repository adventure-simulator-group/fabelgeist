# Museum clothing displays

`german1544.glb` displays the shipped German wool gown on its canonical wearer.
The article has 7,956 triangles, including hidden construction; the wearer adds
4,918 triangles. The saved generated pose is static. The display does not
establish animation clearance or cloth simulation.

The gown is an independently plausible design for a German setting around
1544, rather than a copy of a surviving garment or the earlier ornate Cranach
reconstruction. See the [construction guide](../../../crates/fabelgeist-armor/review/textiles/german1544.md)
for historical comparisons and intentionally deferred features. No reference
photograph is distributed with this exhibit.

## Regeneration

From the repository root, use a native body-review export in canonical Y-up
metres. Its positions, normals and topology must match the body used for fitting.

```sh
cargo run --manifest-path crates/adventuresim-character-creator/Cargo.toml \
  --no-default-features --example textile_outfit_review -- \
  BODY.json crates/fabelgeist-armor/review/textiles/german1544-outfit.json \
  target/museum-gown
python scripts/export_art_demo_clothing.py target/museum-gown \
  assets/art-demo/clothing/german1544.glb
```

The packager requires NumPy, as do the other equipment GLB tools. It preserves
positions, normals, triangles, native fabric coordinates and both embedded PNGs.
It converts scalar sRGB colors to glTF linear factors, retains fabric wrapping,
and omits animation, skin and morph data from this static display. It rejects
articles above 10,000 triangles. The adjacent `.sources.json` records the full
recipe, generator version, input hashes, source revision and output hash.

The exhibit is available at `/art-demo#german-dress`, under **Clothing**. Its
catalog entry shares the retained model-loading, camera, and navigation path
with the armor displays. Run `just showcase` to build and serve the collection.

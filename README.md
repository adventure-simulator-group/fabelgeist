<p align="center">
  <img src="wiki/assets/fabelgeist-logo.png" alt="Fabelgeist" width="960">
</p>

# Fabelgeist

*Fabelgeist* is a browser-based historical fantasy role-playing game set in
16th-century Germany.

*   [Project home and wiki](https://fabelgeist.com)
*   [Project repository](https://github.com/adventure-simulator-group/fabelgeist)
*   [Contribute on GitHub](https://github.com/adventure-simulator-group/fabelgeist/issues)
*   [AGPLv3 license](LICENSE)

The standalone [Texture Studio](crates/adventuresim-texture-studio/README.md) edits procedural materials in Bevy and builds as a static WebGPU website.

The [interior furnishing rules](crates/adventuresim-building-generator/src/interior/README.md)
describe furniture budgets and entrance-to-furniture access validation. Capture
the model catalog and furnished rooms through the production renderer with fresh
output directories:

```sh
python scripts/capture_interior_furniture.py --output target/reviews/interior-catalog
python scripts/capture_furnished_rooms.py --scene-input assets/tactical-scenes/interior-furniture-rooms.json --output target/reviews/interior-rooms
```

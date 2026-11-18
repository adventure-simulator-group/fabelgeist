# Woven pattern tiles

Each PNG here is one repeat of a figured weave, used by the `Woven` textile
fill (`TextileFill::Woven`, `WovenPattern`). A tile is a two-channel PNG: the grey
channel is the share of main figure at each sample, the alpha channel the share
of a second ink, and where both are zero lies the ground cloth. A layer draws
one ink, chosen by its `WovenTone`, so a second layer gives the weave its
second colour. It repeats edge to edge on both axes. Its first row is the end of the repeat nearest the
garment's proximal edge, so a motif that stands upright in the tile stands
upright on a skirt.

Tiles are derived from pictures of historic textiles, not drawn by hand or by
a model. `scripts/textile_pattern_tile.py mask` does the work offline; the game
only decodes the finished tile. The source pictures are not distributed here.

## Tiles

### `italian-pomegranate.png`

Pomegranates among flowers in a pointed-oval lattice of ribboned stems, 512 by
960 samples, two inks. It is a half-drop pattern: alternate columns of
ovals sit half a repeat lower.

- Source: Friedrich Fischbach (1839-1908), *Die wichtigsten Webe-Ornamente bis
  zum 19. Jahrhundert*, Mainz, Mainzer Verlags-Anstalt und Druckerei, 1902,
  plate 133, "Italienische und Flandrische Gewebe. 15.
  Jahrhundert", the large middle panel.
- Scan: Getty Research Institute, on the Internet Archive as
  [`gri_33125010015051`](https://archive.org/details/gri_33125010015051), leaf
  `n52`. The archive marks the book as not in copyright; its author died in
  1908.
- Scan file: `https://archive.org/download/gri_33125010015051/page/n52.jpg`,
  2180 by 3000 pixels, SHA-256
  `00dcefe6e5d2066348fc9a7a8c9f53d3ae3fecc1425fc64c8ff04ffe8e43da98`.
- The plate is Fischbach's nineteenth-century drawing of the silk, not a
  photograph of it, and is itself a regularised copy. The tile is evidence of
  the kind of pattern, not of one surviving cloth.

To make it again from that scan:

```sh
python scripts/textile_pattern_tile.py mask fischbach-n52.jpg \
  crates/fabelgeist-armor/patterns/italian-pomegranate.png \
  --box 376 992 1700 1888 --turns 1 --mirror --tones 3 --width 512 --speck 1
```

## Making a tile

Choose a flat, evenly lit picture that shows at least one whole repeat, in the
public domain or under CC0. CC BY is usable with a credit in
`THIRD_PARTY_NOTICES.md`; share-alike and non-commercial terms are not.
Drawn plates from nineteenth-century pattern books need the least cleaning.
Photographs of worn cloth need a larger `--speck` and often a hand-given
`--repeat`.

`mask` works in these steps:

1. Crop to `--box` and turn by `--turns` clockwise quarter turns so the cloth
   stands upright.
2. Reduce colour to one value per pixel along the direction in which the
   picture's colours differ most, then remove slow changes of lighting. Pass
   `--dark-figure` when the figure is darker than its ground.
3. Find the repeat from the picture's correlation with itself. A half-drop
   pattern is found on its diagonal, so the picture need not be as tall as a
   whole upright repeat. `--repeat DOWN ACROSS`, with `--half-drop` where it
   applies, replaces the measurement.
4. Build one repeat from every place in the picture that the repeat makes
   equal, preferring the middle of the picture.
5. With `--mirror`, centre the pattern's left-right mirror axis and repeat the
   left half on the right.
6. Assign each sample to ground or figure. With `--tones 3` the figure is
   split by colour, the more saturated part being the accent ink. Drop features
   smaller than `--speck` and average each ink down to `--width`.

`--sheet` writes a sheet of several repeats for checking the joins.

Then add a variant to `WovenPattern` in `src/textile_woven.rs` with the tile's
size, and list the tile above with its source, licence and command.

## Tracing lines

`scripts/textile_pattern_tile.py trace` turns a small drawn motif into
needlework stroke paths. It separates the figure from the ground, thins it to
lines one sample wide, splits those at their junctions and simplifies each to
at most `--points` points. `--follow edges` traces the outlines of filled
shapes in place of the middle lines of drawn ones. It writes the paths as
JSON, each a list of `across` and `along` positions in thousandths of the
crop; a recipe gives them a region, repeat counts and a stroke style.

A fabric holds 64 strokes of 32 points, which is enough for a border scroll
and far too few for a whole silk repeat; use a tile for those.

The scroll on the guards of `review/textiles/pomegranate-outfit.json` was
traced from the lower right panel of the same plate:

```sh
python scripts/textile_pattern_tile.py trace fischbach-n52.jpg scroll.json \
  --box 1335 1922 1580 2148 --size 320 --smoothing 1.6 --strokes 40 \
  --shortest 10 --tolerance 1.2
```

# Gown of pomegranate silk

`pomegranate-outfit.json` is a crimson gown of figured silk with gold
pomegranates, of the kind worn in Italy around 1500. It exists to show the
`Woven` textile fill: a repeat cut from a picture of a historic textile and
tiled over the cloth. It is a design of that kind, not a reconstruction of one
garment.

The recipe is fitted to `assets_src/characters/jane_fabelgeist.json`, a slim
female test character. Its repeat counts were chosen on her.

## The design

- A white, finely pleated chemise fills a wide, low, rounded neckline. It has
  no collar.
- The bodice, skirt and sleeves are crimson velvet figured in gold thread with
  the `ItalianPomegranate` pattern: pomegranates among flowers in a lattice of
  pointed ovals.
- A guard of dark velvet 30 mm wide edges the neckline. A scroll is couched on
  it in gold cord between two gold edge cords.
- The same scroll runs on a broad dark guard above the hem.
- The sleeves are close and plain, bound at intervals by narrow bands of gold
  cloth.
- The skirt falls to the floor as a soft cone of rounded pleats.

## Where the ornament comes from

Both ornaments come from one plate of Friedrich Fischbach's *Die wichtigsten
Webe-Ornamente bis zum 19. Jahrhundert* (1902), plate 133, fifteenth-century
Italian and Flemish silks. The book is in the public domain. The
[pattern tile record](../../patterns/README.md) gives the scan, the commands
and what the tools do.

- The silk is the plate's large middle panel, reduced to a tile of two inks (cream figure and orange accent) by
  `scripts/textile_pattern_tile.py mask`.
- The scroll is traced from the plate's lower right panel by
  `scripts/textile_pattern_tile.py trace`. The recipe uses the 26 longest of
  the traced lines as `Cord` strokes.

No part of either ornament was drawn by hand.

## Construction

The outfit is four separately fitted articles, authored inner to outer: a
partlet without a collar for the chemise front, a sleeveless gown with a
`neckline_trim`, and two sleeves.

The sleeves are `PuffAndSlash` articles with five courses, none slashed and
none with added fullness, so each is a plain tube. A sleeve needs several
courses to follow a bent arm: a single course the length of the arm has too
few rings and lets the elbow through. The course at the shoulder keeps a
little fullness and a wider band, which the seam fit at the sleeve head
expects.

A fitted-sleeve gown was tried first and rejected. Fitted sleeves keep the
bodice's chart, in which a whole arm takes up a tenth of the width, and a
tiled pattern smears along the arm. A sleeve article has its own chart, round
the arm and along it.

### Repeat counts

Each `Woven` layer uses `space: Garment` and counts repeats across and along
its article's chart. The tile is 512 by 960, a little over half as wide as it
is high, so counts should give repeats of about that shape on the cloth.

| Cloth | Across | Along | One repeat on Jane |
| --- | --- | --- | --- |
| Bodice | 3 | 1 | about 39 cm wide |
| Skirt | 6 | 2 | about 50 cm wide at the hem |
| Sleeve | 2 | 2 | about 15 by 25 cm |

## Known limits

- The top few centimetres of a skirt's chart are stretched where the skirt
  leaves the body, so a pattern smears there. The recipe covers that strip with
  a dark velvet waist trim edged in gold cord. The cause is not fixed.
- A skirt's chart is as wide at the waist as at the hem, so the pattern
  narrows towards the waist; it is not gathered into pleats there.
- The bodice chart stretches over the top of the shoulder, and the pattern
  with it. The pattern does not run on from bodice to skirt or from bodice to
  sleeve; each has its own chart.
- Where a layer's finish is metallic thread the figure stands slightly above
  the ground in the normal map. The ground's roughness and the figure's
  differ, which gives the silk its sheen contrast, but the weave itself, a twill
  ground and fine hatching, is not drawn.
- A tile holds two inks. A weave with three separate colours needs a second
  tile.
- The traced scroll keeps the plate's spirals but not its broad bands,
  lozenges or palmettes, and a stroke repeats at most 32 times, so the hem
  scroll is coarser than the neck scroll.
- Not addressed by the pattern system: the skirt's hooped outline, the
  chemise's straight neck edge and flat back, close seamless sleeves with
  plain flat bands, guards that end square at the shoulder, and no closure.

## Fitting and rendering

```sh
cargo run --manifest-path crates/adventuresim-character-creator/Cargo.toml \
  --no-default-features --example textile_outfit_review -- BODY.json \
  crates/fabelgeist-armor/review/textiles/pomegranate-outfit.json OUTPUT
blender --background --python scripts/render_textile_outfit_review.py -- \
  OUTPUT OUTPUT/render
```

On Jane the articles have 1,740, 11,242, 5,632 and 5,632 triangles, and no
edge of one article crosses a face of another.

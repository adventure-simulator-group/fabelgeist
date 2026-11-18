# Saxon court gown, in the manner of Cranach

`saxon-court-outfit.json` is a red velvet court gown of the kind Lucas Cranach
the Elder painted on Saxon ladies in the 1520s. It is a design in that manner,
not a reconstruction of one garment or one painting. Its inventory follows
Cranach's *Portrait of Princess Sibylle of Cleve*, dated 1526, Klassik Stiftung
Weimar, Cranach Digital Archive DE_KSW_G12 ([archive
record](https://lucascranach.org/en/DE_KSW_G12/)): a pleated chemise with a
jewelled standing collar, a low-necked bodice with a girdle of two gold bands,
banded sleeves with slashed puffs, flared cuffs and a pleated skirt.

It departs from that portrait where the dress system cannot do the painted
thing well. The portrait's mantle of linked gold rings is left out; the
neckline is edged with a guard of gold set with pearls instead. The sleeve head
is plain and not a slashed puff. The bands carry one regular leafy scroll and
not freehand foliage. The guards above the hem are an addition in the manner of
other Cranach portraits; the Sibylle portrait does not show the hem. Colours
are chosen by eye.

The recipe is fitted to `assets_src/characters/jane_fabelgeist.json`, a slim
female test character. Its neckline width, band widths and sleeve proportions
were chosen on her.

## The design

- A white, finely pleated chemise fills the chest and rises to a standing
  collar band: a couched ground crossed by a lattice of gold thread, with
  pearls strung along the lattice and bunched at its crossings, and small gold
  scrolls in the lozenges.
- The bodice is red velvet with a wide, low, rounded neckline. A gold guard 26
  mm wide edges the neckline: a couched ground between two bright cords, with
  a row of pearls set in small lozenges of gold thread.
- Under the bust sits a girdle of two broad gold bands with a narrower strip
  of velvet between them.
- Each sleeve is plain velvet from the shoulder to the elbow, two slashed
  puffs at the elbow, a plain forearm and a small slashed puff at the wrist,
  bound by bands of gold cloth. The slashes show a blue-grey silk lining. A
  gold cuff flares over the back of the hand.
- The skirt falls to the floor as a soft cone of rounded pleats, with two
  plain gold guards above the hem.
- Every gold band has a fine lengthwise grain. The girdle, sleeve bands and
  cuffs also carry a running leafy scroll drawn in dark silk.

## Construction

The outfit is four separately fitted articles, authored inner to outer.

1. A partlet with a tapered standing collar stands in for the chemise front.
   Its pleating is a `TextilePleatRelief`.
2. A sleeveless gown supplies the bodice and pleated skirt. Its
   `neckline_trim` lays a raised band on the face cloth along the neckline.
3. Two puff-and-slash sleeves, one per arm, are fitted over both.

A sleeveless gown accepts lower layers; the fitted-sleeve gown still does not.
Building the sleeves as arm articles avoids that restriction, and each article
has its own 15,000-triangle ceiling.

### Sleeve courses

`PuffAndSlashPattern.courses` lists up to eight courses from the wrist to the
shoulder. Each course has its own axial share, fullness, optional slashes and
band width. Zero fullness gives a fitted section. Bands between courses are a
separate `ConstrictionBand` component that uses `band_fabric`. The lining tube
under the panes supplies the cloth seen through the slashes, so no chemise
sleeve is modelled. Lining shows only through slashes, so it ends under the
band that closes the last slashed course; plain courses above it are a single
cloth.

Each course's `PuffSlashes` shapes its openings. A slash closes toward its ends
and gapes most at `widest`: 500 is the middle, and a lower value moves the
widest point toward the wrist, which makes a teardrop. `slant` leans each slash
around the limb from one end to the other, as a share of one repeat. This
recipe's courses run, from the wrist, a small puff, a plain forearm, two puffs
at the elbow and two plain courses up to the shoulder. Slashed courses have
eight slashes, each half of its repeat wide, widest near the middle and slanted
by 350. `length` is 880, so the slashes run nearly from band to band.

`puff_roundness` shapes the swell. Below 1000 a puff leaves its bands steeply
and stands over them like a drum; this recipe uses 900. A pane has six rows
along its length, so its bulge and its cut edges read as curves.

`PuffAndSlashPattern.lining_fullness` is the share of each course's fullness
that the lining takes. At 1000 the lining swells with the panes and lies flush
in the slashes. This recipe uses 960, which sets it a little under the pane
edges. At 700 the panes stood off the lining like straps. The lining fabric
also carries a `TextilePleatRelief` of fine lengthwise folds, so the silk seen
through a slash is shaded like gathered cloth.

`proximal_edge` chooses the cloth of the fitted ring that closes the top
course: `Band` for a band like the others, or `Seam` for the outer fabric, as
where a sleeve head is set into an armhole. This recipe uses `Seam`.

A sleeve is fitted to convex outlines of the arm and of whatever is worn under
it. A convex outline bridges the hollow between the arm and the body, so a
sleeve head fitted to it alone stands off the bodice like a cap. Over the last
50 mm before a `Seam` edge the fit therefore finds the cloth beneath directly
and draws the sleeve onto it, fully at the seam. Over an opening in that cloth
it lies level with the cloth beside the opening. Under the arm, where the
sleeve lies beside the body and not on it, the head keeps its outline. The
sleeves' `clearance` and `thickness` are 1 mm each, so the seam edge stands
about 3 mm above the bodice.

`proximal_inset` sets how far the sleeve head sits below the shoulder joint, in
limb radii; this recipe uses 0.4. `underarm_drop` sets how much lower the head
ends under the arm than over the shoulder, also in limb radii. A sleeve head is
cut on a curve: nearly level over the shoulder, dipping under the arm. With no
drop its top is level, and a head set high on the shoulder runs into the body
under the arm. The drop takes its length from the proximal course alone, and
that course's fullness falls with the square of the length it keeps. This
recipe uses 0.8 radii. A slashed head would gape under the arm where the course
is shortened, which is one reason this recipe's head is plain.

`BodiceDesign.armhole_edge` places the upper edge of each armhole relative to
the shoulder joint. `Inside` keeps it toward the neck; `Beyond` carries the
cloth out over the arm. This recipe uses `Beyond(45)`, so the sleeve head
overlaps the bodice by enough that no gap opens between them at the back of the
shoulder.

### Settling

Fitting an article over the ones beneath seats its vertices outside their
faces, which samples the surfaces at vertices only. Where a sleeve leaves an
armhole, its triangles can still pass through the rim between samples.
`layer_settle::settle_layers` in the character creator tests the real surfaces:
every edge of an article against the outward faces beneath it, and every edge
of those faces against the article. Where one passes through the other, the
article's cloth moves a short step outward, and the cloth within 25 mm moves
with it, so a lining stays under the cloth over it. Rigid parts do not move. It
runs on the base body and on each morph and returns the number of crossings
left. The outfit review example and the creator's own loadout fitting both call
it; the example prints the count.

### Skirt gathers

The skirt has two kinds of fold. `SkirtPleats.count` and `depth` model the
broad falls as geometry, and their depth adds to the skirt's width. The fine
gathering at the waist, `waist_count` and `waist_depth`, is relief on the cloth
and does not widen it.

The waist relief knows how far the cloth narrows to the waist. The fitted
surface supplies `TextilePleatLayout::Waist.waist_width`, the width at the
waist as a share of the skirt's mean width; here it is about 0.54. A gather of
the same depth is narrower at the waist, so its sides are steeper there.

The skirt's mesh has three densities around the body: 96, 128 or 192 columns.
The least that gives each modelled fold eight columns is used, so sixteen folds
take 128 columns. This recipe uses 16 rounded pleats 40 mm deep and 24 waist
gathers 30 mm deep. `fitted_fraction` is 0.4: the waist opens into the hanging
skirt over two fifths of its length, which gives a cone. At 0.07, with pleats
55 mm deep, the skirt stood out over the hips like a shelf and its folds
flattened over the seat.

### Cuff

`PuffAndSlashDesign.cuff` adds a ring below the distal band. It keeps its
authored length along the forearm axis and opens from the wrist section to the
convex section of the unposed hand at its free edge, plus the authored flare.
It is not fitted to the fingers and does not establish posed hand clearance.

### Neckline band

`GownDesign.neckline_trim` requires a closed bodice. Stations are placed about
every 20 mm along the reference neckline and across the band, so a wider band
gets more rows and follows a curved chest. Each station is a frozen source on
the face cloth, so every refit rebuilds the same band. Its chart runs once
around the neckline; one unit across is the front width. Whatever is put on
after the gown covers the band.

The band takes `front_width` where it hangs below the front of the opening and
`shoulder_width` up the sides, over the shoulders and at the back. The blend
follows the direction in which the band leaves the edge, so a rounded corner
widens gradually. This recipe uses 26 mm throughout.

Each column of stations is walked along the cloth from the edge in short steps.
Its direction is averaged over about 40 mm of edge to each side, so one cut
vertex cannot kink the band. Boundary edges shorter than 4 mm are merged first.
At a sharp neckline corner the band is mitred: each side runs straight to the
corner's diagonal and keeps its own chart there, so ornament is cut at the
mitre instead of being stretched around the corner. This recipe uses
`NecklineShape::Rounded` with a squareness of 700, which the band still turns
without a mitre.

### Ornament

All ornament uses the general textile types; none is specific to this gown.

- **Materials.** Everything drawn on a cloth is named for what it is made of,
  and the name fixes how it takes the light. A cloth and each of its layers
  has a `TextileFinish`: linen, wool, silk, velvet or cloth of metallic
  thread. A cord, a row of stitches and a braid each have a `TextileThread`:
  linen, wool, silk, metal thread (gilt strip wound on silk) or drawn wire.
  Pearls are nacre. A recipe cannot set a roughness or a metalness of its own.
  With the colour and relief maps a third map is baked that holds the
  roughness and metalness of whatever shows at each texel, so a pearl on gold
  cloth is not metal and a dark silk line on it is not gold.
- **Lift.** Every path point has a `lift`: the height of the thread's middle
  line above the cloth there. Where threads cross, the one that stands higher
  shows. Threads of equal height join, and the later stroke supplies the
  colour. This recipe leaves every lift at nought.
- **Paths that leave their tile.** A path point may lie up to one tile beyond
  its own tile. Its repeats then overlap and join without a seam.
- **Curved courses.** `TextileStroke.course` is `Straight`, with a corner at
  each point, or `Curved`, one smooth curve through every point.
- **Variation between repeats.** `TextileStroke.variation` turns each repeat
  about the middle of its path and shifts it, by amounts fixed by the repeat's
  tile and a seed. Strokes that share a seed, a tiling and a path middle move
  together, so a motif of several strokes stays in one piece. Around a closed
  chart the last repeat still meets the first.
- **Even charts.** A band fitted round the body is charted by angle, which
  stretches ornament where the body stands farther from the axis. After
  fitting, the chart of every component that carries needlework is respaced so
  that equal steps span equal lengths of cloth. Only the cloth that carries
  ornament is measured: a bodice is not as wide at its armholes as at its
  girdle. Pearls on the collar stay round at the throat and leaves keep their
  size round the waist.
- **Regions measured where they lie.** Cloth is not equally wide everywhere in
  its chart, least of all round an armhole: on this bodice a chart unit spans
  about 1.2 m on the chest but 0.93 m on average. After fitting, every region
  that carries ornament is measured on the cloth, and its strokes are sized by
  that. A recipe's repeat counts still have to suit the region's real size.
- **Gold bands.** The girdle, the sleeve bands, the cuffs and the hem guards
  are a yellow gold ground. One fine `Cord` line of metal thread, repeated
  across each band's width, gives the cloth a faint grain that runs along the
  band. The girdle and the hem guards are regions of the bodice and skirt
  fabrics, not separate components. The hem guards carry the grain and a
  bright cord along each edge, and no scroll: a scroll on the skirt was broken
  at every pleat.
- **Running scroll.** Each band carries a waving stem in dark silk. A leaf
  with three lobes to a side springs from the stem before each crest and fills
  the hollow beneath it, drawn in outline with a midrib, and a tendril curls
  off the outer side of each crest. One wave is one repeat, about 2.2 times as
  long as the band is high, so the stem runs on unbroken from repeat to repeat
  and the leaves reach from edge to edge. It is seven curved `Cord` strokes
  per band, and line thickness follows the band's height. The cuff carries
  three rows of it so that its leaves match those of the bands beside it. The
  built-in woven `Scrollwork` motif was tried first; it is cut off at every
  repeat when one row of it fills a band.
- **Neckline guard.** Two bright cords along the edges, two zigzags of gold
  thread that cross into small lozenges, and a pearl in each lozenge.
- **Collar.** The collar tapers to the neck within 10 mm of its seam and has
  no outer neckband, so it lies on the neck. A couched ground under a lozenge
  lattice of gold thread. Small pearls are strung along the lattice lines,
  four-pearl clusters sit on the crossings, single pearls run along both edges
  and small gold scrolls fill the lozenges. Clusters and scrolls vary from one
  repeat to the next. The pearls are `Pearls` strokes in the colour and normal
  maps, not geometry.

## What was taken out

This recipe began as a reconstruction of the Sibylle portrait. Five independent
critics in a row failed it, chiefly on the chain mantle, and the features built
for it that did not hold up were removed from the dress system:

- **The chain mantle and outfit overlays.** The mantle was needlework on a
  raised sheet that lay over the sleeve heads. It could not read as separate
  rings, and separate ring meshes for that many links would need about twice
  the triangle ceiling. The overlay role that let a trim lie over later
  articles, and the pass that pressed cloth under it and lifted it clear, went
  with it.
- **Grown foliage.** A generator grew stems, lobed leaves and tendrils into a
  band from a dozen settings. Critics still read the result as oak-like and
  repeating. Bands now use authored strokes.
- **Closing slashes under the arm.** It depended on one measured position in
  the sleeve's chart, checked on one wearer.
- **Pleat spring and rolled pleats.** A spring above about 0.2 made a shelf at
  the hips, and rolled pleats could not be told from rounded ones at eight
  mesh columns to a fold.
- **Velvet sheen.** It was written to glTF as `KHR_materials_sheen`, which the
  game's renderer does not read.

## Evidence and limits

A fabric can be judged on its own before it is put on an article. The
`textile_swatch` example of `fabelgeist-armor` rasterises one
`TextileAppearance` in JSON to a PNG at the size its needlework chart gives.

Regenerate the review renders with:

```sh
cargo run --manifest-path crates/adventuresim-character-creator/Cargo.toml \
  --no-default-features --example textile_outfit_review -- \
  BODY.json crates/fabelgeist-armor/review/textiles/saxon-court-outfit.json \
  target/saxon-court
blender --background --python scripts/render_textile_outfit_review.py -- \
  target/saxon-court target/saxon-court/render
```

`BODY.json` is a native body-review export of the wearer with one texture
coordinate per vertex. The article counts were 2,400, 11,218, 10,624 and 10,624
triangles.

An independent critic who had seen no earlier state compares body-included
renders with the portrait and with the stated design choices. Reviewer reports
are kept locally under `target/`. Known limits:

- The sleeve head is a separate cloth laid on the bodice, so its seam is a
  fine raised edge and not one surface. Putting the gown on over the sleeves
  made the bodice balloon over the sleeve heads.
- The skirt's pleats are shallow and faceted: sixteen folds have eight mesh
  columns each, and more columns would pass the triangle ceiling. They are
  faint just under the waistband.
- Puffs are firm and regular, and the lining lies in the slashes instead of
  swelling out of them.
- The scroll repeats exactly along each band, and every band carries the same
  scroll at its own size.
- The collar is a cone that stands a little off the neck at its seam, its
  ornament repeats exactly, and its pearls are flat. Lowering its seam made it
  flare into wings.
- The chemise pleat lines kink where the partlet's mesh turns. The partlet
  covers the chest and neck only; the chemise is not a complete garment. It
  closes at the back with a slit 1 mm wide.
- The gold bands are colour and relief on the cloth and have no edge of their
  own. A neckline guard with less than 2 mm of relief let the velvet show
  through it.
- The cuff is a thin flared ring that stands free of the hand.
- One wearer was fitted, with a much heavier and a slighter body as morph
  targets. On all three no edge of a sleeve passes through the gown. Morph
  blends and posed clearance were not exercised.
- The roughness and metalness map is used by the creator's preview, the
  exported glTF and the review render. The material values are typical ones
  for each named material.
- The renders are an external static render, not the game's renderer.

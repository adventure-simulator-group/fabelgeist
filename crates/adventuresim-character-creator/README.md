# Character creator

Native, non-authoritative character design tool backed by `fabelgeist-mhr`. It
loads Meta's Momentum Human Rig assets locally, exposes its 45 identity
coefficients and 72 expression coefficients, and previews the generated mesh in
Bevy.

Install the pinned upstream assets into the ignored local authoring cache, then
start the creator from the repository root:

```powershell
just init-mhr-assets
just character-creator
```

The importer verifies Meta's MHR v1.0.1 release by size and SHA-256 and installs
the FBX rigs and model definition under `target/mhr-assets/v1.0.1/assets`. That
default cache is about 50 MB after extraction. Run
`just init-mhr-lod4-correctives` only when comparing the optional LOD 4
pose-corrective network; installing all supported corrective bases requires
`scripts/init_mhr_assets.py --all-correctives` for LODs 4–6. Override the
location with `--assets` or `MHR_ASSETS` when needed. The
downloaded archive and extracted source assets are not committed; deliberately
exported game and Cascadeur artifacts are tracked separately.

The default project is the canonical zero-coefficient MHR body. **Save recipe**
writes the current parameters to the selected recipe path (by default,
`assets_src/characters/mhr_base.json`). **Export rigged GLB** writes to
`assets_src/biped/unarmed/base.glb` by default. The export is a zero-animation
T-pose containing MHR's 127 joints plus the three Fabelgeist animation
attachments, four normalized skinning influences, and inverse bind matrices
for the saved body. Use `just export-mhr-base <staging-path>` to export the canonical
body without opening the studio, then prepare its runtime copy as described
below.

The zero-weight attachment joints follow MHR's side-prefix naming convention:
`l_weapon` is parented to `l_wrist`, `r_weapon` to `r_wrist`, and `c_camera` to
`c_head`. Each weapon joint is positioned halfway from its wrist toward the
corresponding `*_middle1` knuckle, placing it in the generated palm. The camera
joint is positioned at the midpoint of the generated eye joints. Their rotations
inherit the wrist or head without mirrored negative scale.

The left panel has five tabs. **Character** edits skeletal proportions
(**Build**), identity shape (**Body**, **Head** and **Hands**),
**Expression** and **Hair** (see below); double-click a slider to return it
to neutral. **Inventory**
holds everything the character owns (see below). **Armory** reshapes the
catalog's parametric equipment. **Wardrobe** drapes clothes once and saves
them for any body. **Output** loads the body model, saves and loads recipes,
plays the animation preview, exports the rigged GLB and saves catalog designs.

Outside the armory, the character's name is edited above the viewport, and a
bar below it frames a **Full body** or **Portrait** shot, turns the view,
randomizes or resets the appearance and saves the recipe. The camera orbits
the framed subject: drag to orbit, right-drag to pan and use the mouse wheel
to zoom. The default is MHR LOD 4 with pose correctives disabled. Only LODs
4–6 are available in the UI, CLI, and GLB exporter. LOD 4 has 2,461 vertices
and 4,918 triangles before clothing hides body faces. The **Pose-corrective
model** checkbox reloads the selected LOD with or without MHR's corrective
network for direct comparison. Recipes contain model
coordinates, not authoritative character state, and must be regenerated and
validated when connected to game creation.

## Hair

A recipe's `hair` is a hairstyle and its pigment, or `null` for a bald head.
The **Hair** page offers the 24 hairstyles of `fabelgeist-hair`, a strand
hair groomer after DiffLocks (Rosu et al., CVPR 2025): crops, bobs, long and
layered cuts, ponytails, buns, braids, afros, dreadlocks and more. Its
**Show hair** and **Show scalp** switches draw either alone, to look at the
painted scalp without the hair over it or the hair without the scalp; they
are the studio's, not the recipe's. Its colour is a pigment as Blender's Principled Hair takes it: **Melanin** runs
from white through blond and brown to black, **Redness** from ashen to red,
and a dye tints the strands on top, linear RGB, white for none. The page
offers the hair crate's eight named natural colours and ten dyes as
swatches, any dye from a colour picker, and the two sliders, which keep a
dye. A dye tints the hair's own colour, so it barely shows on dark hair;
each dye swatch also lightens the hair, as bleaching before dyeing does.

The hairstyle's `shape` varies its preset; its default grows the preset as
it is. The **Cut**, **Arrangement** and **Growth** menus swap in another of
the hair crate's 16 cuts, 10 arrangements (natural, parted, swept, back,
forward, down, up, ponytail, bun, braid) or 7 growth patterns (whorls,
cowlick, radial, forward, backward, down), or keep the preset's own. The
sliders scale or shift the preset's values: **Length** scales every length
of the cut by 0.3 to 2, keeping the hemline as far from the crown as the hair
is long; **Volume** adds -1 to 4 cm of lift at the crown; **Part** moves the
part up to 6 cm either side; **Curl** and **Clumping** add -1 to 1 to how
much the strands curl and gather into locks, each kept within 0 to 1;
**Waves** adds -10 to 20 mm of waves either side; and **Frizz** scales the
stray strands' wander and their share by 0 to 4. Whether hair drapes on the
body follows the arrangement and length the shape gives: tied, styled or
short hair does not. A changed shape regrows the hair once its slider is let
go; choosing another hairstyle starts it from its preset's own shape. The
recipe stores the shape strictly, rejecting a value outside its range.

A hairstyle is grown once, on a worker thread, on the hair crate's mannequin
bust, by the hair crate's GPU groomer on the studio's own render device: the
scalp cut and the whole groom grown in compute kernels, a twentieth to an eighth
of a second, ten times faster than every core of the CPU, and the strands read
back. The kernels compile once, on the first hairstyle grown (about half a
second); an export opens a device of its own for them. The hair is then placed
on each generated body on the same thread (two to four seconds). Placing
stretches the bust's head to the character's width across the ears, its depth
from the back of the skull to the tip of the nose and its crown, and turns it to
face where the eyes look. The bust is only fitted to the head's extent, so
wherever the character's skull is fuller its strands would lie under the skin:
each strand is moved whole along its root's normal -- the bust scalp's, smooth
-- until the root sits 0.5 mm off the skin. Pushing each root to the nearest
point of the coarse body instead would move neighbouring roots apart along
different facets' normals and part the hair along their edges, at the nape most.

The hair is then draped on the character's own body. What a body does to a
strand is how its equilibrium under gravity (the hair crate's `Settle`)
against the body differs from its equilibrium against nothing; that is found
on the bust, as groomed, and on the character, as carried, and the
character's minus the bust's is added to the strand. Where the two bodies
hold the hair alike it is nothing, and where the character's shoulders stand
higher or wider it is how long hair falls over them, in front of them or
down the back, rather than hanging through them. A tight coil -- a bun's, a
braid's -- answers a body unsteadily, so the drape takes hold only below the
head, fading in from the ears to the jaw. The solve knows nothing of what
ties a lock, and would slide a ponytail's tail off a thicker neck strand by
strand, so a strand is draped only as far as it sinks into the body as
carried: from 3 cm deep, fully from 5 cm, as long hair runs through a
shoulder; a tail resting against the neck is left to the last shrinkwrap.
About two thousand guides are draped and every strand follows the one whose
root is nearest, so hair parts cleanly in front of and behind a shoulder. A
last shrinkwrap keeps every strand out of the body, keeping its length,
before the levels of detail are prepared against that body. Changing only the
colour or the level of detail reuses the placed hair.

**Level of detail** shows any of the hair crate's forms: its strands at the
groom's own point count and at 24, 12 and 6 points each, drawn as ribbons,
or its fine and coarse cut of hair cards (strips along the locks, each
showing a painted bundle of strands from a shared atlas). Both are drawn
with the standard material extended by `hair_preview/strand_shading.wgsl`,
shaded as the hair crate shades them: each point's normal faces the eye,
square to the strands' direction, turned a little towards the card's or
ribbon's face, and the highlights lean towards the pigment's hue. Strands
pointing at the eye, as an afro's do at its crown and hairline, have no
such direction, and their card is seen edge-on: their normal turns to the
eye instead, or they would catch the environment's reflection at a grazing
angle and glint a cold grey. The baked sky visibility, which darkens the
hair's colour, fades that reflection too, so hair deep in the groom is not
dark and glinting at once. A card's
atlas coverage becomes a share of each pixel's four samples, which ones
hashed from the pixel and the card: the hardware's alpha to coverage would
give every card of one coverage the same samples, so stacked half-covering
cards would never add up, and the scalp showed through. Shadows and depth
cut the coverage at half. **Strands drawn** draws a share of a strand level, each ribbon
wider by the inverse of the share so the head stays covered; the preview
draws at most a million and a half ribbon vertices. The level is the
studio's choice, not the recipe's: a game picks it from the head's size on
screen.

The scalp under the hair is painted with the hair lying on it and shaded as the
hair is, so no level shows lit skin between its strands, even between an afro's
open curls, and the hair painted on the scalp looks like the hair drawn over it.
The textures are baked once per hairstyle, on the bust it grew on (about a
seventh to a quarter of a second): each strand's first 8 mm off the skin is
drawn, in steps of at most 2 mm, into textures 2048 texels square mapped afresh
to the scalp. The hair crate maps its scalp by the angle from the crown onto a
disk inside the square, a texel at the crown covering a few times the skin of
one at the hairline, the corners unused and much of the disk below the hairline.
The scalp's texture instead takes Lambert's equal-area projection about the axis
of the narrowest cone from the cranium's centre that holds the scalp, out to
just past its edge, spread over the whole square by Shirley and Chiu's
concentric mapping, which keeps areas too: every texel covers about 0.13 mm of
skin a side, the crown as finely as the rest, near a hair's own width, and the
texture has no seam. The topmost strand over each texel shows, from 25% of the
hair's colour where it meets the skin to 90% at 8 mm off it, and a second
texture keeps the way it runs across the texture; strands rooted within about 4
mm of each other share a tone, so locks show where single strands are far finer
than a pixel. Scalp no strand lies on is at 35%, in the shadow of hair standing
further off it -- up to 25 mm, judged over blocks of eight texels -- as between
an afro's coils. Where no hair lies over it either, it is bare, as at a parting
or a whorl's eye, and how bare the scalp is within about 0.8 mm, measured twice
over so the nearest counts most, turns the texture clear, from a tenth bare to
three quarters, so the body's own skin shows lit through the cap. A strand's own
texel clears only half as far: hair thins into a parting over a couple of
millimetres strand by strand, the last strands' ends lying over the skin, while
a texel or two between coils shows no skin. So the gaps between cards show
strands running the groom's way, its parting and its whorl, rather than a flat
patch, and the textures are filtered anisotropically to keep them where the eye
meets the scalp obliquely. The cap is drawn with the hair's own surface -- its
colour, roughness and highlights leaning to its hue -- extended by
`hair_preview/scalp_shading.wgsl`: each point's normal is the strand normal the
cards and ribbons are lit by (`hair_preview/strand.wgsl`), facing the eye square
to the topmost strand, which the cap's tangents turn from the texture onto the
head, and the strands' shade fades the environment's reflection as the baked
occlusion does on cards. Where no strand runs, the cap is lit by the scalp's own
smooth normals. The body has only a few dozen vertices over the skull, too few
to keep a parting narrow, so the bust's own scalp, cut in four-millimetre cells,
is carried onto the character as a cap laid 0.4 mm over its skin and painted
there. Laid by its corners, a cap triangle spanning a ridge or corner of the
coarse body would dip under the skin and let it show through in specks along the
body's edges, so each triangle is sampled across and lifted until all of it
clears the skin. The cap's edge runs in the cells' steps, so it is blended over
the skin, fading out from 5 mm above the hair crate's hairline over the next 6
mm -- the hairline's own smooth curve, above the steps -- and over a row of
cells along the rest of its edge. Only the cap is painted: the body's skin, the
neck and the shoulders hair falls over, keeps its colour, and the cap comes and
goes with the hair.

Ribbons and cards are straight between their rows, and over a rounded
shoulder or skull a straight segment between two rows just outside the skin
passes under it, the coarse levels' long segments most. Near the skin, rows
are added until no segment is longer than a centimetre, and each row is
moved whole 0.6 mm off the skin -- just over the scalp cap -- more for a
wide card by as much as it sags across a curve as tight as the neck. A row's
lift spreads to its neighbours, tapering over four rows, so a strip bends
off the skin rather than in a sawtooth. The hair crate turns a card to face
away from the head's centre, right beside the head; lying on a shoulder such
a card would stand on its edge, a fin, so within 3 cm of the skin each row
turns to lie flat on it.

To look at hair without the studio's window, an ignored test grows a
hairstyle on the default body and draws every level of detail as the studio
draws them, close over the parting, from above, from the front, from behind
and over the shoulder, printing how long placing takes, how long each level
takes to build and how much of it lies under the skin:

```powershell
$env:MHR_ASSETS = "target/mhr-assets/v1.0.1/assets"; $env:HAIR_PRESET = "long"
cargo test --release --manifest-path crates/adventuresim-character-creator/Cargo.toml --bin adventuresim-character-creator hair_renders -- --ignored --nocapture
```

It writes `target/hair-render/<preset>-<level>-<view>.png`, and the baked
scalp texture as `<preset>-scalp-texture.png`, and the way its strands run as
`<preset>-scalp-strands.png`. With
`HAIR_ON_BUST=1` it draws the hair on the hair crate's own bust instead, as
it grew there, to `bust-<preset>-<level>-<view>.png`: whether a fault lies in
placing hair on a character or in drawing it. `HAIR_SHAPE` varies the preset
as the Hair page does, taking the shape's fields that differ from its default
-- `HAIR_SHAPE='{"length": 0.55, "curl": 0.6}'`, say -- and names the images
`<preset>-shaped-<level>-<view>.png`. `HAIR_PIGMENT` colours it as a recipe's
pigment does -- say
`HAIR_PIGMENT='{"melanin": 0.1, "redness": 0.05, "dye": [0.1, 0.4, 1]}'` --
and names the images `<preset>-coloured-...`. `HAIR_SHOW=scalp` or
`HAIR_SHOW=hair` draws the scalp or the hair alone, as the Hair page's
switches do, to `<preset>-scalp-only-...` or `<preset>-hair-only-...`.

The hair is skinned wholly to `c_head`, so it follows the walk preview, and
the rigged GLB exports its fine cards as a `hair` shell on the same joint,
following the head through every body morph channel. The export carries
neither the scalp cap nor its paint.

## Inventory

A recipe's `inventory` lists every article the character owns. Each has a
stable ID, whether it is worn, and one of three kinds of article:

- a catalog item in one of its placements, such as the left vambrace, with an
  optional design of its own; without one it is built from the catalog default.
  A plate-steel item also carries its decoration, an optional engraving and an
  optional trim, and its construction: solid, lamellar or scale (see
  [Lamellar and scale](#lamellar-and-scale));
- a draped garment: a name, a sewn pattern or fitted coif, its fabric, layer
  and drape settings;
- a settled garment: a copy of a garment saved in the wardrobe, with its
  settled drape, fitted to the wearer without simulating (see
  [Wardrobe](#wardrobe)).

Only worn articles appear on the body and in exports; the rest are carried.
Worn articles must fit together under the catalog's equipment rules, checked
through the same equipment graph the game uses. Each fills body cells, a
location in a layer: clothing, padding, mail, plate and so on. Two articles
cannot fill the same cell, except that articulated plates on one limb may share
it when their fit zones differ. Draped cloth takes the layer of its cut, or
mail when its fabric is chainmail, and fills the cells its cut covers (see
[Garments](#garments)). Attached articles, such as mail voiders, hang from an attachment
point on a worn support such as the arming doublet, within that point's
capacity and on the matching side.

In the **Inventory** tab, worn articles are grouped by layer and carried
articles are listed below them. The checkbox wears or takes off an article.
Wearing one takes off whatever fills its place, and taking one off also takes
off whatever hangs from it. **Acquire** adds and wears **New cloth**, garments
saved in the **Wardrobe** or searchable catalog items, and catalog items with
left and right placements can add both. Selecting an article shows its editor:
shape controls for parametric catalog items and the garment editor for cloth
(see [Garments](#garments)). Editing a catalog item's shape gives it its own design. **Use
catalog shape** discards that design, and **Make catalog default** copies it to
the catalog defaults, which **Save all catalog designs** in the **Output** tab
writes. A plate-steel article's **Decoration** chooses a decoration saved
from the armory, or **Plain**; choosing one copies its engraving and trim into
the article, where they can be edited further, so a recipe never depends on the
library. Its **Construction** builds it of small plates (see
[Lamellar and scale](#lamellar-and-scale)). Draped garments drape from the innermost layer out; within one layer,
the inventory order sets draping order.

Edits rebuild only what they change. The body is kept while its shape stands,
each worn piece's fit while its design and the body stand, and its
construction, trim and lacing while their settings stand too; worn cloth is
draped again only when the body, the garments or the fit of the plate over them
changed. A dragged control applies its value when it is let go.

Recipes use schema version 13. Recipes from older versions are not read.

## Lamellar and scale

A plate-steel piece can be built of small plates instead of one solid plate.
**Lamellar** lays narrow, tall lames in rows; **Scale** lays round-footed
scales in staggered rows. Choosing either starts from its usual plate, which
the **Small plates** controls then reshape: width, height, thickness, gap,
edge bevel, rounded foot, row overlap, row stagger, hole pairs and hole radius.
Rows run from the piece's top down, each covering the top of the row below;
each plate tilts so that its top tucks under the row above, so rows clear each
other however many there are. Plates keep their size on the fitted piece, and
closed pieces such as greaves take as many plates around as their girth needs.
A plate that would reach past the piece's edge, such as at a diagonal cut, is
left out.

**Laced** threads cord through the plates' holes, with its own radius, colour
and roughness. Every row is bound by a running cord through each pair of holes,
across each plate's face and behind it to the next. Scale holes sit near the
top, where the row above hides most of the cord. Lamellar lames are also hung
from the row above by cords that run down over the upper lame's face, turn
under its foot and pass up behind it into the lames below, so the lacing shows
over every row. The cord is its own mesh, previewed and exported as a separate
non-metal surface.

A trim runs along the rim of each plate. It is kept to 15% of the plate's
narrower side, whatever width the decoration asks, and its ornament shrinks
with it, so each plate keeps a face of its own metal.

The plates and cord are laid over the fitted piece's outer surface, so they
follow the wearer and every body morph with the same topology, skinned like the
surface beneath them. Breastplates, cuirasses, faulds, tassets and all limb,
hand and foot defences can take small plates. Helmets, gorgets, brigandines
and jacks of plates cannot yet, and a piece built of small plates still carries
its fluting in the surface its plates follow. Small plates multiply a piece's
geometry, and every vertex carries every body morph, so an exported lamellar
outfit is several times larger than a solid one.

## Armory

The **Armory** tab is for improving the catalog's parametric equipment. It fits
every parametric catalog item to the character's body and hangs the pieces on a
wall, in rows from head to feet, each with its name underneath. Pieces that
failed to fit are named in red, and their errors show on hover and in the
editor. **Both sides** shows the left and right placements of paired pieces, and
**Selected only** hides every other piece. Clicking a name on the wall or in the
list selects that piece and frames it. **◀ ▶**, or the arrow keys over the
viewport, step through the pieces. **On body** shows the selected piece where it
is worn, in every placement, and **See-through body** shows where it clears or
cuts into the body.

The selected piece's shape controls edit its **catalog default** directly, and
only that piece is fitted again. **Revert** restores the default the armory
opened with, and **Save catalog designs** writes all defaults to the paths set
on the **Output** tab. Worn articles without their own design use the edited
default once you leave the armory. **Refit all** fits every piece to the current
body again; this also happens automatically on entering the armory after the
body changed. Catalog clothing without a parametric design is not shown.

A selected plate-steel piece also shows a **Construction** card, which
previews the piece built of lamellar lames or scales on that piece only; each
inventory article chooses its own construction. It also shows a **Decoration**
card for designing an engraving and trim together, previewed on that piece
only. **Start from** loads
a saved decoration to edit. **Save to library** stores the decoration under its
name, replacing one of the same name, and **Delete** removes the named one.
Both write the decoration library at once; by default it is
`assets_src/equipment/decorations.json`, set with `--decorations` or on the
**Output** tab, where **Reload library** reads it again. A library that does not
exist yet is empty.

## Wardrobe

The **Wardrobe** tab drapes a garment on the character's bare body and saves
it once it has settled, so it can be worn on any body without draping again.

The **Design** card holds the same garment editor as the inventory (see
[Garments](#garments)). The garment drapes as soon as the tab opens and again
after every pattern, fabric or drape change, from its first changed stage;
**Drape again** simulates every stage from the placed panels. Once settling has
finished for the design as it stands, **Save to wardrobe** saves it under its
name, replacing a garment of the same name.

Saving binds every cloth vertex to the nearest point of the body's surface: a
body triangle, a position on it, and the offset from that point along the
surface's tangent, bitangent and smooth normal. Wearing a saved garment
evaluates those bindings on the wearer, so the cloth follows the body's size,
slope and proportions while keeping its folds and its ease. Where neighbouring
cloth was bound to body parts that moved apart, such as a hem between thighs
on wider hips, edges stretched past a quarter beyond their settled length are
pulled back together. The cloth is then kept outside the wearer and the
garments beneath it, as after draping (see [Drape stages](#drape-stages)), and
fitted under worn plate like a draped garment. Nothing is simulated, so
a saved garment fits a new body in well under a second. The bindings name the
body mesh's triangles, so a garment is only worn on the level of detail it was
saved on; the default is LOD 1.

**Saved garments** lists the wardrobe. Selecting one fits it to the current
body. **Add to inventory** adds and wears a copy of it, **Edit** loads its
settings into the design to drape and save again, and **Delete** removes it.
The inventory can also add saved garments under **Acquire**. A worn copy
belongs to the recipe, so a recipe never depends on the wardrobe; in the
inventory it keeps its drape, and only its name, layer and chainmail appearance
can be edited. Its per-vertex bindings are stored as compact base64 strings.

The wardrobe is saved at once to `assets_src/equipment/wardrobe.json` by
default, set with `--wardrobe` or on the **Output** tab, where **Reload
wardrobe** reads it again. A wardrobe that does not exist yet is empty.

The GPU test `a_settled_garment_fits_other_bodies_without_draping` drapes a
shape (`SETTLED_TEST_SHAPE`, the tunic by default) on the canonical body,
saves it, fits it to three random bodies and exports one of them from the
inventory:

```powershell
$env:MHR_ASSETS = "target/mhr-assets/v1.0.1/assets"
cargo test --manifest-path crates/adventuresim-character-creator/Cargo.toml a_settled_garment -- --ignored --nocapture
```

In every tab, drag orbits the view, right- or middle-drag pans, and the wheel
zooms.

The preview reads each LOD's authored `ByVertice/Direct` normals from its MHR
FBX. It stores those normals in local rest-surface frames and reconstructs the
frames from the final generated vertices, so authored shading follows identity,
expression, skinning, and optional pose-corrective displacement. Triangle-only
normal reconstruction is retained internally only to define those frames; it is
not sent to Bevy as the character's shading normal.

## Garments

**New cloth** adds a cotton shirt to reshape. Every setting is in its editor,
and a garment is sewn to the wearer's measurements from a GarmentCode pattern,
the design space of the GarmentCodeData dataset:

- **Name** labels it in the inventory and the export.
- **Sewn pattern** or **Fitted coif** chooses how it is made. The coif is a
  mail hood with neck, breast and back flaps, fitted around the head and
  sharing the catalog mail coif's controls.
- **Body** is none, a **Straight tunic** or a **Fitted bodice**. The straight
  tunic has a **Length** measured down from the shoulder in neck-to-waist
  lengths, where 1 reaches the waist and about 2.5 the knee, plus ease and hem
  flare. The fitted bodice is cut at the waist. Either can have **Sleeves**,
  with a length and cuff width, and a **Standing collar** with a height. A
  sleeve's length is a share of the arm from the shoulder joint: 0.9 ends at
  the wrist on a straight tunic and 0.8 on a fitted bodice. Longer sleeves
  reach over the hand, which pierces a cuff narrower than itself.
- **Legs** is none, **Trousers** or a **Skirt**, each with a length and hem
  flare. Trousers also have ease. Their hem flare stops just below straight,
  because narrower legs slide down the wearer while settling. A trouser length
  of 0.8 ends at the ankle; longer legs reach over the foot.
- **Fabric** sets the cloth's weight and drape. **Layer** is clothing, padding
  or outerwear. Padding sits between clothing and mail, and outerwear goes
  over mail and plate. Chainmail is always worn as mail.

A garment must keep a body or legs. It fills the body cells it covers: the
chest and stomach for a body, the arms for sleeves and the legs for a lower
garment. A long tunic over trousers therefore fits in one layer, while two
garments on the torso in the same layer displace each other.

**Start from a shape** fills in the pattern and layer from a medieval wardrobe:
shirt, fitted shirt, tunic, doublet, gambeson, trousers, hose, braies, skirt,
dress, kirtle, surcoat and houppelande. Everything stays editable afterwards.

## Draped chainmail

Choose **Chainmail** as a garment's fabric to make it mail armor. A mail shirt
is the straight tunic with sleeves, as hauberks were cut; its **Length**
reaches anywhere from the waist to the knee, and changing it re-drapes from
sewing. A **Fitted coif** in chainmail is fitted around the head and chest like
the catalog mail coif, then settles as chainmail instead of being sewn from
flat panels. Shirt and coif are both in the mail layer, so keep the shirt
before the coif in the inventory to layer the coif over it.

Selecting **Chainmail** shows its ring controls: outer ring diameter, wire
diameter, row spacing, ring tilt, steel color and roughness. The wire and row
spacing limits follow the ring so every link keeps an opening for its four
neighbours. These are appearance only: edits update the shown garment and the
export without re-draping. The mail's mass comes from its fabric preset.

Mail uses the sewing panels' material coordinates in metres, with separate
vertices at UV seams and joined vertices for animation physics. The preview
material and the export scale them so one texture repeat covers one ring across
and two rows up. The creator generates the repeating maps from the weave:
cutout color, tangent-space normals from the tilted round wire, and ambient
occlusion. Steel uses full metallic response.

### Drape stages

Draping places the pattern panels, sews them without gravity, and settles the
sewn garment under gravity. **Drape stages** exposes each stage's steps,
substeps, constraint iterations, gravity, damping and self-collision. Swept
contacts read the cloth back from the GPU for continuous crossing checks; their
interval, iterations and body inclusion are the main cost of a step. Previews
default to every step and start from the placed panels.

A settled garment hangs from its highest supports, so a loose cut stands off
the chest, belly and back. **Body fit** then draws it onto the body it
dresses: each value closes that share of the gap down to the fabric's
clearance, 0.7 by default, without stretching any cloth edge. Only cloth over
the trunk and arms is drawn in, plus the legs for trousers and the head for a
fitted coif; a hem hanging past the thighs or a sleeve over the hand follows
only as the drawn cloth pulls it. Finally the cloth is kept outside the wearer
and inner garments: its vertices are pushed out to clearance, and where a body
feature smaller than a cloth triangle, such as a thumb, passes between its
vertices, that triangle is lifted over it.

Each drape keeps its completed stages, including after a failure or
cancellation. Changing a stage re-runs from that stage: a body fit change
repeats only the fast host fit, and a settling change resumes from the sewn
garment. Body, pattern, fabric, resolution or sewing changes start again,
as does **Drape again**.

Draping resolves swept vertex/triangle and edge/edge contacts after each GPU
substep. This host projection is vendored from Prism's `shell` library and
includes body triangle interiors and excludes joined seam copies. Fixed body
bounds are cached between substeps. It prevents crossings missed by particle
spheres; it does not infer layer order for already intersecting starting meshes.

Clothing, padding and mail lie under worn rigid plate. Wherever a plate covers
them, the last fit presses them in towards the wearer, clear of the plate's
inner face, each garment far enough in for those pressed over it. The press
spreads to the cloth around it, so cloth tucks under a plate's rim. The plate is
met as fitted, before any small plates are laid on it, so changing a piece's
construction or decoration does not re-drape. Outerwear is not pressed. Fit
completion and export check the emitted cloth against itself, the wearer and
inner garments.

Drape problems never block the studio. Fit problems, such as a garment
intersecting itself, are reported in the status
line while the garment is still shown, animated and exported. A garment that
cannot be draped at all stops the drape there; the garments already draped
remain usable, and export leaves the rest out and says so. Animation and export
wait only while a drape is running. Press **Drape again** to retry.

Render an exported chainmail outfit through Bevy, with an asset/material check
before capture:

```powershell
cargo run --manifest-path crates/adventuresim-character-creator/Cargo.toml --example garment_preview -- outfit.glb outfit.png
```

## Animation integration

The exported base establishes MHR's stable bone names and hierarchy as the
animation-pack contract. The preview keeps body identity separate from
animation. Prism's retargeting pipeline establishes the intended boundary:
import a clip into an engine skeleton, retarget model-space deltas through
semantic rig profiles, then encode the resulting MHR joint pose into MHR's 204
model parameters. Identity remains this recipe's 45 coefficients, so one
retargeted clip works for every generated body. The creator currently shows a
neutral pose; clip playback should reuse Prism's `Retargeter`, `MhrRig`, and
`MhrPoseEncoder`, including its T-pose reference and hinge correction, rather
than copying local rotations.

## Identity morphs in game

Character and procedural equipment exports contain 45 named identity morphs,
`mhr_identity_00` through `mhr_identity_44`, with position deltas in metres and
normal deltas. Zero weights reproduce the saved recipe; weight 1 adds one MHR
identity coefficient relative to that recipe. Expression coefficients remain
baked into its neutral face. Morphs deform the surface independently of skeletal
proportions. Neither form of appearance variation changes tactical physics or
combat stats.

Every primitive of a clothed character uses the same ordered channels. Fitted
clothing retains its base trim triangles when refitted for each sample.
Parametric armor uses the armor generator's corresponding body samples,
including its anatomical joint landmarks. Each recipe retains its authored
vertex and index correspondence across morph samples. Export fails if fitting
changes either.

Breastplate identity targets transfer body displacement through fixed
correspondence on the smooth carrier. Refined flute vertices interpolate that
coarse displacement, and corresponding inner/outer wall vertices share it. This
avoids accumulating independent nonlinear fitting corrections in signed identity
blends. It preserves carrier gauge vectors, not exact normal thickness under
arbitrary deformation; installed skeletal animation needs its own checks.


The tactical client derives bounded cosmetic weights from each persistent
character ID, consistently across clients and reconnects. Equipment uses its
current wearer's weights, updates when transferred, and returns to zero weights
when dropped. Mesh assets remain shared; weights belong to each instance.

## Skeletal proportions

Recipes also store nine absolute MHR skeletal coefficients in
`proportions`, ordered as hip width, shoulder width, upper arm length, lower arm
length, upper leg length, lower leg length, spine length, neck length, and foot
length. The creator's **Skeletal proportions** controls use the pinned model's
limits. **Neutral** resets both surface and skeleton; **Randomize body** varies
both. Zero is MHR's reference skeleton.

Exports store `adventuresim_proportions` in joint-node extras: the recipe's
reference coefficients and local joint translation deltas in metres per unit
coefficient. These nine controls drive translations, including bone-length
offsets; hand scaling, asymmetric lengths, and pose correctives are not part of
this skeletal contract. Export validates the mapping against the MHR model.

In game, character IDs determine stable skeletal coefficients independently of
the surface morph seed. The pose buffer samples shared reference motion and adds
instance-owned joint offsets before terrain and limb IK. A neutral-foot height
correction raises or lowers the pelvis for different leg lengths. Neither the
shared animation cache nor inverse bind matrices are modified. Equipment uses
the wearer's joint entities, so skeletal deformation applies once through
skinning, in addition to its surface morphs; transfers follow the new skeleton
and dropped equipment returns to its exported shape.

Twelve additional equipment targets refit the shell at the spine, neck,
upper-arm, upper-leg, and lower-leg length limits and the hip-width limits.
The channels use the `mhr_skeletal_` prefix, with `spine`, `neck`, `upper_arm`,
`upper_leg`, and `lower_leg` `short`/`long` pairs and a
`hip_narrow`/`hip_wide` pair. Their
position deltas subtract the movement already supplied by skinning, preventing
double deformation. Body primitives carry zero deltas for these channels. The
breastplate fits these endpoints directly to their bodies and validates
that vertex and component connectivity match the reference; its 45 identity
targets retain their reference correspondence. The client interpolates from the
exported reference to either endpoint. Cadence and
distance-to-phase curves are remeasured from each character's retargeted foot
trajectories when its proportions change.

Use the canonical zero-proportion body as the runtime animation reference.
Regenerate body and equipment from the same recipe. For the canonical game body:

```powershell
just export-mhr-base target/morph-assets/base.glb
just generate-procedural-equipment target/morph-assets/equipment
python scripts/prepare_rig_base.py target/morph-assets/base.glb assets/animations/biped/unarmed/base.glb
```

Inspect the staged equipment before copying its GLBs, shared PNGs and manifest into
`assets/equipment/procedural`. The base preparation step preserves morph data.

## Parametric armor authoring

All armor catalog entries have authored parametric recipes. Preview, character
export, equipment export and the armory use the same recipe dispatch, fit and
material. Catalog loading rejects armor without a recipe. The part geometry
lives in `fabelgeist-armor`; the creator owns the MHR landmarks, the
fitting passes, and the transfer of UVs, skinning and morph targets.

Armor is generated and fitted entirely on the GPU. For each piece the creator
uploads the wearer and every morph sample once (`device_equipment`), then
records the piece against each of them (`parametric_equipment`):

- The part frame of the piece's region (`device_frames`; the head and foot
  frames have their own passes) is oriented by rig landmarks and sized by the
  skin those landmarks own. It stays on the device.
- The family's carriers are evaluated in that frame, and its fitter moves them
  onto measured body sections: clearance stations for long plates
  (`device_clearance`), foot sections and boot layering for footwear, cages for
  garments and gorgets, and measured sections for the close helmet and the
  coif.
- The shells are thickened, and each vertex takes its UV and skin weights from
  the nearest body vertex.

Everything is read back after the last realization is recorded, so morph
targets share the base topology by construction. The vambrace and the
breastplate fit directly to the selected forearm and torso skin
(`device_bracer`, `device_torso`). Underlayers are cut from the body on the host
by a frozen plan, using part frames fitted on the device, and every offset,
layer and attribute is then evaluated on the device (`device_underlayer`). The
fitted cloth coif starts from the coif carrier fitted on the device, before its
shell is thickened.

Every rigid armor piece is shaded with the same parametric metal as the plate
armor builder: a base color and roughness plus a tiling map of the surface's
finish, baked on the GPU into normal and roughness maps. The finish is the
gentle undulation left by planishing, which makes reflections wobble; a dense
polishing grain that streaks highlights; uneven gloss from handling; and fine
scratches. The builder's **Metal and scratches** section edits its own metal,
whose default is polished steel. Catalog plate steel takes its color and
roughness from its catalog material, polished, rough or oxidized steel, with
the default finish. Its body-surface UVs are rescaled to the builder's density
of four texture repeats per metre, so the finish is the same size on a helmet,
a vambrace and a lamella. The maps are baked at 1024 texels per repeat, a
quarter millimetre each. Mail keeps its ring weave.

Metal shows its surroundings rather than a color of its own, so the studio is
lit by an environment as well as its spotlight: a dim room with a key soft box
on the spotlight's side, a fill opposite, an overhead strip and a rim light
behind. The environment is generated procedurally and filtered on the GPU. It
also gives the body and cloth their indirect light.

Any plate-steel piece can carry an **engraving**: a tiling relief cut into its
metal, either a procedural ornament or an image. An ornament is a motif (a
wave, zigzag, guilloche of interlaced waves, rope, beads or vine) repeated
across the cell, with optional fillet lines along both of its long sides. The
line width, the number of repeats and each motif's proportions are adjustable,
and the device draws the ornament at the bake's resolution. An image is either a
grayscale height map, where white is the untouched surface and black the floor
of a cut of the chosen depth, or a tangent-space normal map in the glTF
convention. An ornament is always cut as a height map. The engraving repeats a chosen
number of times per metal tile, may be turned on the surface, and roughens the
floor of its cuts. Its slopes add to the scratches in the baked normal map for
both preview and export; a height map also gives the preview a parallax depth
map, which glTF does not carry. The image path is stored in the recipe and read
relative to the working directory; an ornament is stored by its parameters. A
catalog steel article edits its own engraving in the inventory, or takes one
from a saved decoration.

A catalog steel article can also carry a **trim**: a band along every edge of
every plate, finished with its own metal, such as gilt, bluing or bright steel
on a darker plate. The band reaches a chosen width in from each edge and covers
the narrow edge walls, so it wraps the plate's thickness. Every generator
records which face of the plate each triangle lies on. The band is cut out of
the fitted mesh along its exact border, and the cut vertices carry the piece's
skin weights and morph targets. The trim metal's engraving is the ornament.
Adding one sizes its cell to the band's width, and **Fit cell to band width**
restores that after the width changes. Along the band, one engraving cell
repeats every cell's length, starting at the edge and running inward. Each closed edge is stretched slightly so that it
holds a whole number of repeats and its ornament closes on itself. Where a
plate is narrower than two band widths, the bands from opposite edges meet, and
where the band turns a sharp corner, its inner border follows the mesh to
within one triangle. Previews and character exports shade each band as a
separate primitive named after its piece or component with a `.trim` suffix.
Catalog equipment assets carry no trim.

The authored helmet, limb and garment defaults live in
[`assets_src/equipment/armor-designs.json`](../../assets_src/equipment/armor-designs.json).
The paired torso and vambrace defaults live beside it in
`breastplate-design.json` and `vambrace-design.json`. The creator embeds these
authored inputs and validates every recipe before use. Edit the catalog, rebuild
the creator, then regenerate equipment assets to change the game's default
shapes. Invalid entries fail explicitly; there is no generated default fallback.

Use `--write-armor-designs target/armor-designs.json` to write the editable
helmet, limb and garment defaults. Pass `--armor-designs` with that file to
preview or export overrides. The saved document has two required maps:
`defaults` maps catalog item IDs to shared recipes; `placements` maps item IDs
to recipes keyed by `left`, `right`, or `worn`. An explicit placement recipe
takes precedence over the shared item default. Without a saved item default,
the authored catalog recipe applies. Item IDs, placements, construction
families and parameters are validated when loading or saving. The authored
catalog is a separate input format; generate an editable document with
`--write-armor-designs` rather than passing the raw catalog to `--armor-designs`.

For example, edit `defaults.spaulder` for both shoulders, then copy that
recipe to `placements.spaulder.left` and change it for an asymmetric pair. A
newly acquired article starts from its placement's recipe, and the armory then
edits that article's own design. Equipment export and review output select the
same placement recipe.

Use `--bracer-design` for
the vambrace and `--breastplate-design` for the paired torso plates; these are
separate recipe files, outside the catalog override map. Together these are
the catalog defaults that newly acquired inventory items start from. The
**Output** tab's **Save all catalog designs** button writes the catalog,
vambrace and breastplate recipes to the three displayed paths. Each path must be distinct and its parent directory
must exist. Pass all three files back through their corresponding options to
reproduce the saved set in preview or export.

Measurements use millimetres and ratios use permille. The sallet's
`opening_width` is an angular exception: it is the face-opening half-angle in
milliradians. The serialized design contributes to the asset's design hash and
generator version. Generate current defaults before editing; recipe files must
include the required fields of the current schema.

The [museum armor authoring guide](../fabelgeist-armor/review/museum/README.md)
describes anime torso courses, wrapping tassets, independent pauldron wings,
joint extensions, besagews, buffes and bellows visors, with primary historical
references and construction limits.

### Device construction and remaining work

The device constructs puff-and-slash clothing, anime breastplate courses,
pauldrons, and wrapped tassets. Runtime generation fits only the current wearer;
the studio can still request explicit morph realizations.

Wrapped tassets retain separate thigh carriers, medial trimming, shaped hems,
and sloped suspension. Convex triangle-band sections support the final shaped
height. Upper courses seat over completed lower equipment; a waist assembly
fits them at its fauld's suspension height before generating the mesh. The
inner-boundary solve runs once per plate row and is shared by its columns.

The pauldron port is not ready for production promotion. Its generated plates
are closed and body-fitted, but the current chest/arm binding separates the cap
from the arm lames in raised-guard poses. Individual plate identities are kept
for a constrained equipment rig. Breastplate courses currently move together
as one rigid chest assembly. These bindings do not establish collision-free
articulation.

Fastenings, dense bake sources (`--armor-bake-source`), runtime levels of
detail and generated fluting normal maps remain outside device construction.

Metal recipes expose construction-specific shape controls. Helmet crowns have
fullness, ridge height and optional fluting; sallets add face-opening width and
sweep, tail shape, and separate visor side-panel depth. Limb plates expose
section shape and edge flare, greaves add ankle extension, and joint cops add
wing shape and notch depth. Tassets have width, separation, inner cutaway and
rounded or pointed hems; gorgets have collar height, independent front/rear
depth and width, independent front/rear hem flatness, rear sweep and separate
neck clearance. Gorget fluting follows the front bib and leaves the shoulder
return plain. These controls shape fitted carrier surfaces, with physical
padding clearance and metal gauge kept separate.

A close helmet's `neck_length` is its requested maximum extension. Exact body
and plate sections shorten the fitted hem when a compressed neck leaves less
room above the breastplate. Intermediate neck rows enclose the local support
while retaining the authored flare; they add reserve only where that existing
section does not provide the required clearance. Fitting preserves connectivity
and keeps the visor separate.

Solid helmet plates, including separate visors and buffes, use rigid head
skinning so jaw and neck deformation cannot bend them. Their separate component
meshes and hinge metadata remain intact. Arming caps and mail coifs retain
body-derived flexible skinning.

Joint cops and their separate distal metal courses attach to the anatomical
lower arm or lower leg. Shortening a limb therefore preserves the courses'
overlaps instead of shearing them through nearest-skin twist and foot weights.
The component meshes remain separate for future articulation.

Rerebraces blend along the upper-arm axis between its root and distal helper.
Both anchors belong to the upper arm, so elbow flexion cannot bend the enclosing
plate. Arm-length changes scale its formed sections coherently.

Leather boot shafts preserve their fitted elliptical sections with consistent
angular meridians across body morphs. Their feet retain anatomical fitting, and
the sole includes the requested clearance beneath the foot as well as at its
sides. Shaft fitting includes the supported default leg garments.

The shared `PlateFluting` recipe applies to metal limb and garment plates,
vambraces, helmet crowns and close-helmet visors. Set the appropriate `fluting`
field to `null` for a plain surface (`crown.fluting` or `visor_fluting` on
helmets). Its fields are `count` (2–64), `width` (350–850 permille of pitch),
`depth` (1–4 mm), `spread` (400–850), `lower_spread` (500–1000), `start`, `end`,
and `fade` (100–250). The pattern runs from lower to upper plate coordinates;
start/end must remain within 50–950 and leave room for both fades. Lower spread
controls the fan at the bottom of the pattern. Mail and textile recipes do not
accept metal fluting.

The breastplate editor provides Rounded, Central ridge, Peascod, and Fluted
starting points. Its `profile` controls projection, upper-chest recession, the
height of that projection, lateral fullness, medial ridge, and waist-point
drop/width. Waist projection is independent of chest fullness; the flange
follows the waist point without a discontinuity when chest projection height
changes. Opening, length, waist width, clearance, and flange controls remain
independent.

The breastplate uses the shared flute recipe with a torso-specific distribution.
Set `fluting` to `null` for a plain plate, or provide that recipe. Count (2–24),
width (350–850 permille of pitch), depth (1–4 mm), spread, lower spread,
start/end heights, and end taper are independent controls. Width is a proportion
of spacing, not an absolute millimetre width: increasing count at fixed spread
makes the flutes closer and physically narrower. Changing width at fixed count
changes the flute/land ratio. Spread and width scale with the fitted wearer;
flute relief depth remains in millimetres. Both surfaces carry the relief; plate
gauge follows the smooth carrier's extrusion direction, rather than the local
flute normal. Unknown fields and invalid fade intervals are rejected.

[Example recipes and historical references](../fabelgeist-armor/review/breastplate/README.md)
provide editable starting points. Each recipe has one front plate and one back
plate; a separate plackart or articulated waist plate requires a different
construction recipe. The upper armscye is a smooth boundary of the shell, and
the back returns seat over the front at the lower flanks.

For reproducible body-visible review, run the creator with `--armor-review-dir
target/armor-review/candidate`, then:

```powershell
blender --background --python scripts/render_armor_review.py -- target/armor-review/candidate target/armor-review/candidate/renders
python scripts/armor_review_boards.py target/armor-review/candidate/renders
blender --background --python scripts/check_armor_review.py -- target/armor-review/candidate
python scripts/assemble_armor_review.py target/armor-review/candidate
blender --background --python scripts/render_armor_review.py -- target/armor-review/candidate/assemblies target/armor-review/candidate/assemblies/renders
```

These exports include the actual body, triangles, runtime vertex normals and
design parameters. The four-view boards preserve those normals and use back-face
culling. The mesh checker reports closed-edge winding, triangle area, material
volume and sampled body distances; those local distance signs are diagnostics,
not a proof of continuous clearance. Assembled views combine unchanged parts to
expose interface problems. Static review does not replace inspection of
installed equipment under runtime animation.

`--armor-review-selection` chooses what the review exports besides the body:
`catalog` (the default) fits every placement of every parametric catalog item,
`recipe` only the catalog articles the recipe wears, and `body` nothing more.

Add `--profile` to a CLI export to print JSON timing events to standard error.

Studio and review generation fit equipment as a character instance. This path
builds only the current body, parallelizes independent equipment layers, and
does not reserve underlayer folds for hypothetical identities. Character GLB
export instead builds reusable equipment with all requested identity morphs.
Regenerate instance equipment whenever its wearer's body shape changes.

Filtered equipment exports accept comma-separated IDs with `--equipment-item`
and require an empty staging directory. Run `python
scripts/check_parametric_armor_assets.py STAGING_DIRECTORY` to audit actual GLB
winding, skin weights, all 57 morph targets and representative blends. These
attribute and topology checks do not establish body clearance; skeletal fit
targets require their corresponding bone translations for a fit assessment. Use
`--allow-partial` only for a deliberately filtered export.

Individual equipment GLBs reference shared `texture-<BLAKE3>.png` files beside
them. Install those PNGs from the staging directory together with the GLBs;
their content-addressed names let the runtime reuse each image across pieces.
Base color remains sRGB, while normal and ambient-occlusion maps remain linear
through their glTF material roles. Assembled character exports embed their maps
and remain standalone files.

For static review of the exported identity morphs, run
`python scripts/export_armor_morph_review.py STAGING_DIRECTORY OUTPUT_DIRECTORY`.
This evaluates the actual body and armor GLBs at neutral, positive, negative and
mixed identity blends. It does not refit substitute geometry. Use the rendering
and mesh-check commands above on each resulting directory. Skeletal proportions
and animation still require the gameplay renderer.

The native `animation-viewer` supports `--armor-harness` with `plate`,
`plate-tassets`, `plate-underlayers`, `underlayers`, `mail`, `padded` and
`close-helmet`, plus `--hidden` for
automated captures. `plate-tassets` replaces the fauld with tassets because
those defenses share a rigid-armor catalog slot. It uses the shared gameplay
equipment visual plugin and waits for every installed GLB, material, skin and
morph component; unresolved assets fail the capture. Rebuild it after updating
the equipment manifest, then capture idle, walking and raised-guard scenarios.

## Equipment material UVs

`just generate-procedural-equipment DIRECTORY` exports native LOD4 body
geometry with the device-built armor. Runtime body exports support LODs 4–6.
Structural openings and plate boundaries remain explicit.

The tactical client generates armor and clothing at runtime from its base rig
through the creator library, fitting armor on the armor device. The device
reads results back synchronously, so the web build does not generate runtime
equipment.

Set `BLENDER_BIN` to the Blender executable when it is not on PATH. Run
`python scripts/finish_equipment.py DIRECTORY --source-directory SOURCE_DIRECTORY`
after geometry export. Direct creator exports contain construction UVs until
this finishing step runs. Regenerate after changing geometry parameters.

The material atlas occupies `TEXCOORD_0`; existing anatomical coordinates move
losslessly to `TEXCOORD_1`, with their domain and channel recorded explicitly.
Socket coordinates are independent and unchanged. UV0 also gives runtime blood
decals a nonoverlapping surface on each plate. Sharp rims and transitions
between front, side, and rim-facing
zones separate plate faces from edge walls. Concealed rear
meridians and arm undersides provide cuts through curved panels. Blender's
angle-based solver unwraps those charts and packs them with a 0.004 UV margin.
Each independently articulated component has its own atlas. Layouts need not
remain identical between parameter configurations. Body-conforming textured
mail and padding keep their existing material coordinates.

Seam splits copy all original vertex and morph attributes without changing
triangle order, the rig, or component hinges. Tangents use the material UV
channel. Material textures select glTF `texCoord: 0` for these atlases.
Compare source and finished exports with
`python scripts/check_armor_uvs.py ORIGINAL_DIRECTORY FINISHED_DIRECTORY` to
check correspondence, nondegenerate charts, overlap, and tangent frames.

`python scripts/finish_equipment.py DIRECTORY --stage bake --source-directory
SOURCE_DIRECTORY` bakes an already unwrapped export. Normal maps project the
matching dense recipe geometry, including fluting, onto the native LOD surface.
Each component projects only from its matching source component. The export
retains its native shading normals except where tangent-frame conditioning is
required; positions and skin weights are unchanged. AO comes from Cycles rays
against the actual plates within the item. Maps are separate linear glTF normal
and occlusion channels, both using UV0; the unlit albedo is unchanged.

The default bake uses 1024-square images, 32 AO samples, and two-pixel gutters.
`scripts/bake_armor.py` exposes resolution and sample controls for offline work.
Unused AO atlas space is white to avoid dark mip bleeding.
Texture filenames are content-addressed. Regenerate geometry before changing
an already baked atlas.
Compare source and finished exports using `scripts/check_armor_bakes.py`.

Each LOD needs its own unwrap and bake against the dense source. Cycles previews
ray trace ambient occlusion rather than
multiplying the exported AO map into albedo; runtime glTF uses the separate AO
channel for ambient lighting.

## Plate edge finishes

`assets_src/equipment/armor-finishes.json` selects texture-only trim for metal
plates. Generation applies this after UV unwrapping and normal/AO baking. Use
`python scripts/finish_equipment.py DIRECTORY --stage trim` on a bake, or pass
`--finish-recipe`
to `scripts/finish_equipment.py` for a different finish document. Python needs
NumPy and Pillow. Regenerate before changing an already applied finish.

The document has `defaults` and per-item `items` overrides. Patterns are `none`,
`plain`, `double`, `chevron`, `scallop`, and `vine`. `width_mm` accepts 1–30 mm;
`repeats` accepts 1–128 repetitions around each closed rim. `color` is an sRGB
`#RRGGBB` value; `metallic` and `roughness` accept 0–1. The vine is a stylized
ornament, not an exact historical engraving reproduction.

Optional `bands` add straight stripes across a piece, using the same patterns
and material controls. Each band requires `axis` (`x`, `y`, or `z`) and
`position` (0–1 across the complete metal piece's bounds). The axis is the
stripe's width direction: an `x` band at `0.5` centers a vertical stripe across
the piece's width.
Band widths accept 1–100 mm. `phase_axis` controls the perpendicular direction
of pattern repetition; it defaults to `y` for `x`/`z` bands and `x` for `y`
bands. Style fields inherit from the rim recipe unless overridden. Bands paint
over rim trim in list order and remain continuous across atlas seams and plate
courses. A band covers the whole selected coordinate slab, including front and
back surfaces; it is not a component-specific engraving path.

For example, add this array to a symmetric torso plate's finish recipe for a
plain 36 mm gold stripe centered on its medial plane:

```json
"bands": [
  {
    "axis": "x",
    "position": 0.5,
    "width_mm": 36,
    "pattern": "plain",
    "color": "#B79D5A"
  }
]
```

Generators record the outer-sheet boundaries before closing plate returns.
Compaction preserves the applicable boundaries for each helmet component, and
glTF records them in reference-body metres. Finishing measures distance to those
segments within the same connected plate and carries pattern phase around each
rim. UV seams never become decorative boundaries. Widths are measured on the
reference body; the texture follows the existing UVs when the wearer morphs.

Finishes write independent base-color and metallic/roughness maps in UV0. They
leave geometry, skinning, morphs, normal maps, and AO unchanged. The base-color
map contains only the unlit steel and selected trim colors. Mail keeps its own
material system. `scripts/check_armor_trim.py SOURCE FINISHED` checks these
contracts on exported assets.

## Body-conforming underlayers

`arming_doublet`, `padded_chausses`, `mail_voiders`, `mail_brayette`,
`mail_knee_voider`, and `mail_standard` use the `Underlayer` recipe. The doublet
and hose occupy the padding layer. Voiders attach to the
doublet's `mail_voiders` attachment point in the flexible-armor layer. Removing
plate reveals the same clothing and mail; exposed cloth never changes material
automatically. The doublet now covers the torso and sleeves, so separate quilted
sleeves conflict with it. Captive plate or helmet linings are not independent
equipment articles. The brayette independently covers the groin, seat, and
upper thighs. Knee voiders attach to the matching left or right padded hose;
removing that hose also removes its attached mail. The optional mail standard
protects the neck. The plate fixture uses its gorget instead of the standard.

The mesh follows source body triangles with outward standoff. Shading normals
are corrected against incident face planes before offsetting. Include volumes
and Boolean subtraction boxes split crossed triangles, interpolate their source
UVs and skin weights, and close the cut edge with an inner surface. Uncut source
triangles retain their connectivity. A frozen barycentric cut plan gives every
morph the same vertices, UVs and triangle indices. Canonical UV seams remain
split for texturing and physically joined at the garment surface.

Offset directions also satisfy the sampled unposed proportion shapes. The
shared layer envelope accounts for both local folds and nearby opposing body
surfaces, including the space between the thighs. Bone-translation checks use
the same reference offset vectors that the exported mesh retains. These fitting
constraints do not repair a self-intersecting source body.

Recipe distances are millimetres. `clearance` accepts 1–10 and `thickness` 1–8.
They describe the uncompressed stack: tight concave body creases reduce both
distances together using a shared body field. Doublet `length` accepts 700–1200
permille; hose length accepts 700–1000. `sleeve_length` accepts 500–1000 and
`patch_width` 35–100 mm. `cuts` contains boxes with `minimum` and `maximum`
three-coordinate points in the reference body's metre space. Cuts follow the
body after morphing rather than remaining stationary in world space.

Mail uses shared color/alpha, tangent normal, and ambient-occlusion atlases in
the body's canonical UVs. Albedo contains a uniform unlit steel color;
ring relief and recess shading belong to the normal and AO maps. Normal and AO
channels are linear data; color is sRGB. AO is bound to the native material
and glTF occlusion channel, never multiplied into albedo. A construction unwrap
carries the weave around curved panels;
the atlas is baked back into the original UV layout. Separate torus rings
provide the weave bake, including overlap and apertures. Production meshes
remain lightweight skinned surfaces, not individual animated links. Regenerate
a carrier covering every mail family at maximum width and length before
rebuilding an atlas:

```powershell
blender --background --python scripts/unwrap_underlayer_charts.py -- MAIL_REVIEW_DIRECTORY target/mail/charts.json
blender --background --python scripts/bake_mail_weave.py -- target/mail/weave
python scripts/bake_underlayer_materials.py target/mail/charts.json target/mail/weave assets_src/equipment/materials
blender --background --python-exit-code 1 --python scripts/check_underlayer_assets.py -- STAGING_DIRECTORY assets/animations/biped/unarmed/base.glb target/mail/intersections.json
blender --background --python-exit-code 1 --python scripts/check_underlayer_plate_interfaces.py -- STAGING_DIRECTORY assets/equipment/procedural target/mail/plate-interfaces.json
```

The intersection checker tests exported triangles against the body and adjacent
underlayers, all identity basis targets, signed runtime identity bounds,
representative blends, and skeletal proportion endpoints. It applies skeletal
fit residuals together with bone translations. It supplements the common GLB
topology audit and native animation fixture; sampled tests do not establish a
continuous guarantee over all possible bodies and motions.

Repeat `--only CONFIGURATION_NAME` to run a focused set of unposed cases. The
report records source-body self-intersections separately; source defects never
exempt a garment from its intersection checks.

Use `--pose-trace PATH/global-bone-transforms.jsonl` to audit eight sampled
frames from an actual animation-viewer capture. Keep its `armor-readiness.json`
and `body-proportions.json` beside the trace: they supply the captured wearer
configuration. Both proportion and pose audits use the four normalized skin
influences consumed by Bevy. Exports merge joint
influences, retain the four strongest, and renormalize them before writing
`JOINTS_0` and `WEIGHTS_0`. The runtime consumes those four coefficients
directly; the checker does not divide by a homogeneous weight sum. Linear skinning
can
produce intersecting folds, including folds already present in the source body;
the underlayers do not provide cloth collision resolution. Keep a failing
intersection report distinct from a successful asset-loading capture.

The neutral plate fixture reserves space for these defaults through its cuirass,
gorget, fauld, and spaulder clearance parameters. Cuirass section fitting
preserves the requested front and back clearance after seating its returns.
Increasing garment thickness still requires checking the assembled kit; changing
an underlayer invalidates dependent runtime fits. The shared layer plan orders
generation from inner to outer surfaces, honors explicit `layers_over`
declarations, and rejects contradictory or cyclic outfits. Puffed garments seat
against the completed lower surfaces; each remaining plate fitter must consume
those surfaces before assembled-kit clearance can be claimed. Carried and dropped
items retain the physical fit of their last wearer. Studio morph fits require the
matching lower-surface realization rather than substituting its neutral shape.

Articulated breastplates resample the fitted front and rear torso into separate
closed horizontal courses on the GPU. Chevron slopes, overlap, and lap lift are
construction parameters, independent of decorative fluting. The
`animation-viewer --armor-harness anime` fixture exercises this path with runtime
wearer fitting and no equipment morph targets. Course attachment and overlap
under torso motion require body-visible review; mesh closure alone is not a
clearance or articulation guarantee.
The current animation binding carries the cuirass as one rigid chest assembly,
preserving the course overlaps. Independent course rotation or sliding requires
an equipment rig with constrained lap pivots; assigning separate body-spine
joints opens gaps and is not used.

Construction references are the Philadelphia Museum of Art's
[arming doublet, 1977-167-240, c. 1550–1650](https://www.philamuseum.org/objects/71390),
and the Met's German sixteenth-century
[armpit defense, 27.183.35](https://www.metmuseum.org/art/collection/search/34856).
The latter supplies the mail link dimensions: 6.2 mm outside and 4.0 mm inside
diameter. The Met's
[half sleeve, 29.158.186](https://www.metmuseum.org/art/collection/search/23246)
describes attached armpit and elbow gussets. These support separate textile
foundations and localized mail, not a universal requirement for a full mail
shirt beneath plate. The fitted hose is a simplified game garment; it is not a
reconstruction of a photographed surviving padded hose. Period arming dress
varied, including ordinary hose with local knee padding. Closures, stitching,
and individual lacing cords are simplified in these meshes.

The brayette's broad hip and seat wrap with a perineal bridge follows the
[Met's German sixteenth-century brayette, 27.183.14](https://www.metmuseum.org/art/collection/search/34839).
The optional standing collar follows
[Met 27.183.8](https://www.metmuseum.org/art/collection/search/34834).
Longitudinal knee strips adapt the surviving mail in Bavarian Army Museum
A 6147, illustrated in
[Christopher Retsch's catalogue, pp. 190-211](https://d-nb.info/139208119X/34).
That surviving hose contains sewn-in plates; it is evidence for the mail strip
arrangement, not a claim that the game's padded hose replicates that garment.

## Full pauldrons

The device builds a shared shoulder saddle, seats it against the wearer's body
and completed lower plates, then cuts its overlapping closed courses. The
runtime builds only the current wearer's fit, with no equipment morph targets.
`animation-viewer --armor-harness pauldron` exercises both shoulders over a
cuirass. Rigid attachment is separate from fitting: the cap and lames move as one
rigid upper-arm assembly. This is not a constrained armor rig;
posed overlap and body clearance remain acceptance concerns.

`pauldron` is a separate catalog choice from the smaller `spaulder`. A formed
shoulder plate has independent front and rear wing reach and drop, proximal
neck lames, and a narrowing stack of upper-arm lames. These are closed plate
shells with authored physical rims and optional fluting. Material baking and
texture trim are separate authoring capabilities.

The construction follows the broad wings and articulated upper-arm coverage
of the Met's [Italian pauldrons, ca. 1560, 14.25.827a-d](https://www.metmuseum.org/art/collection/search/22301).
The rear view of [Henry VIII's armor, ca. 1544, 32.130.7a-l](https://www.metmuseum.org/art/collection/search/23936)
supplies the relationship of full rear wings to the backplate and wearer.
These references support the construction family; the default is not an
exact reconstruction of either object.

The selected breastplate and gorget geometry constrain the wings in preview,
review export, and equipment export. `plate_clearance` sets separation from
those surfaces; `arm_allowance` reserves room for the rerebrace. Padding
clearance and plate gauge remain separate controls. Changing a supporting
recipe refits the wings; the resulting assembly still requires checking. The
device projects against completed lower-layer triangles and smooths its shared
clearance field before cutting plates. Each plate receives one rigid owner;
metal no longer deforms through a blend of chest and arm skin weights. This
preserves plate shape but requires a constrained equipment rig to keep the
moving plates overlapped. The current binding does not meet that posed gate.

The plate animation-viewer fixture uses full pauldrons. For automated unposed
body, self, and neighboring-piece intersection checks:

```sh
blender --background --python-exit-code 1 --python scripts/check_armor_clearance.py -- \
  assets/equipment/procedural assets/animations/biped/unarmed/base.glb \
  target/pauldron-clearance.json --item pauldron--left --item pauldron--right \
  --neighbor cuirass--worn --neighbor gorget--worn
```

The default audit covers 189 sampled identity and skeletal configurations.
Use repeated `--only` arguments for focused checks, such as `--only neutral`.
Reports distinguish intersections from sampled signed distances and make no
continuous or posed collision guarantee.

A saved review body also supports a construction sweep without loading MHR:

```sh
cargo build --manifest-path crates/adventuresim-character-creator/Cargo.toml \
  --example armor_fit_review
python scripts/export_pauldron_variants.py target/review/body.json \
  target/pauldron-variants \
  --fitter crates/adventuresim-character-creator/target/debug/examples/armor_fit_review
blender --background --python-exit-code 1 \
  --python scripts/check_armor_construction.py -- \
  target/pauldron-variants target/pauldron-variants/check.json
```

The sweep covers both sides at 23 representative parameter settings, including
thin and thick walls with seven lames, minimal crown height, wing limits, and
sparse/dense fluting. This shoulder construction supports 1-3 mm sheet stock;
heavier stock requires a wider bend treatment at the wing returns. It checks
each piece against the body independently;
the exported-asset audit above checks the selected neighboring torso pieces.
These samples do not establish every combination of controls.

## Plate fastenings

`assets_src/equipment/armor-fasteners.json` authors leather retention straps,
metal buckle frames, tongues, and rivet heads independently of plate shape.
The studio's fastening controls and `--fastener-designs PATH` feed the same
geometry in preview, character export, and equipment export. Controls include
strap width, gauge, count, height, spacing, arc, buckle position, leather color,
lining allowance, and underarm drop. Dimensions use millimetres; fit does not
scale the leather gauge with the wearer.
An arc without room for the selected buckle and return fold is rejected for
that wearer instead of generating a reversed strip.

Closures follow a taut cross-section around the supporting body and plates.
Knee and foot closures additionally account for the selected greave recipe.
Shoulder and elbow closures account for the upper-arm plate. A small assembly
allowance keeps closures clear of separately attached neighboring plates.
Descending shoulder bands follow cross-sections at each height along the arm.
Both endpoints must land on their supporting plate.
Leather and metal retain separate material components through UV unwrapping,
normal/AO baking, skinning, and all 57 morph targets. Texture trim applies to
the plate's authored rims, not to leather or buckle edges.

The tasset item includes its fauld: the two occupy one waist equipment slot.
Its independent fauld and tasset controls preserve both component identities.
One to three short buckled hangers support each panel. This attachment layout
is based on the three upper buckles described for Henry VIII's 1544 armor,
Met 32.130.7a-l, in
[Blair and Pyhrr's construction study](https://resources.metmuseum.org/resources/metpublications/pdf/Wilton_Montmorency_Armor_Italian_Armor_for_Henry_VIII_The_Metropolitan_Museum_Journal_v_38_2003.pdf).
Straps model attachment and retention; they are not a leather tension simulation
or an articulated hinge/slide solver.
The tasset tops fit below the fauld hem. Their attachments blend from the
pelvis to the primary leg joints, preserving the medial gap as hip width
changes. Each leather wall shares its mate's attachment field.

Audit exported closures against the unposed body, their own plates, and
declared neighboring plates with:

```sh
blender --background --python-exit-code 1 \
  --python scripts/check_fastener_assets.py -- \
  target/fastener-assets target/body.glb target/fastener-clearance.json
```

The default sweep uses 191 sampled identity and skeletal configurations.
`--only neutral` narrows the bodies; `--item couter--left` narrows the reported
closures while retaining neighboring assets for contact checks. Leather must
have closed, consistently wound walls and no self-intersections. Contact
between the metal tongue and frame is intentional. These checks do not claim
collision-free movement in posed animations or every continuous parameter blend.

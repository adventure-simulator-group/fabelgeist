# Wernigerode eagle sources and adaptation

The Studio uses two Wikimedia Commons drawings by Tom Lemmens (Tom-L) and
Heralder under [CC BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/):

| Studio recipe | Modern SVG | Drawing date |
| --- | --- | --- |
| `wernigerode-eagle` | [*Arms of the King of the Romans (c.1433-1486)*](https://commons.wikimedia.org/wiki/File:Arms_of_the_King_of_the_Romans_(c.1433-1486).svg) | 6 January 2014 |
| `wernigerode-double-eagle` | [*Arms of the Holy Roman Emperor (c.1433-c.1450)*](https://commons.wikimedia.org/wiki/File:Arms_of_the_Holy_Roman_Emperor_(c.1433-c.1450).svg) | 15 October 2013 |

The double eagle traces the *Wernigeroder (Schaffhausensches) Wappenbuch*,
southern Germany, c. 1475–1500, Bayerische Staatsbibliothek, Cod.icon. 308 n.
The [library record](https://www.digitale-sammlungen.de/en/view/bsb00043104)
and [scan 23](https://api.digitale-sammlungen.de/iiif/image/v2/bsb00043104_00023/full/1200,/0/default.jpg)
identify the historical model. The modern artists made it symmetrical and
adjusted its proportions to fit their shield. The scan has small crowns on
the heads; the downloaded redraw has red halos, which this adaptation retains.

The single eagle is a modern variation in the same style, not a separate
tracing of an attested single-headed specimen. The dates in the SVG titles
describe the arms represented, not the date of the drawings. Neither Studio
recipe is a facsimile or evidence of an actual shield's paint construction.

## Pinned files

`eagles/single.original.svg` and `eagles/double.original.svg` are untouched
downloads. The double download is the Commons revision of 30 December 2023,
which restores the revision of 17 December 2013.

| Original | Bytes | SHA-256 |
| --- | --- | --- |
| `single.original.svg` | 22,090 | `c277aa3b748291a6680889e4f7a20ff1f183ea28ebce346e858ccbdd5964c2b2` |
| `double.original.svg` | 36,879 | `644e05e78ed0f2af886ea6c6254b53ce64822f894fa02e30c3874ed887402ef0` |

`python3 crates/adventuresim-heraldry/references/eagles/extract.py` reproduces
the runtime `single.svg` and `double.svg`. It verifies the original hashes,
removes the shield path, identifies the tongue's three paint regions and adds
credit metadata. It expands the double's mirrored wing with unique IDs, so
SVG readers preserve the second tongue's identity. It preserves all contours
and transforms of the eagle, including
the double eagle's halos. The source shield does not determine the Studio's
display object; both recipes use the same broad 450 × 450 mm support.

## Construction and controls

The single original contains 33 paths, seven groups, one rectangle and four
`use` instances. The double contains 31 paths, nine groups, two circles, one
rectangle and four `use` instances. Both construct claws from repeated groups
and reflect a wing group for the opposite side. The double reflects the head
with that group; the single has a separate head. Neither uses raster images,
filters or gradients. Their gray and red modeling consists of solid shapes.

The shared SVG loader resolves instances, compound contours and transforms.
Source strokes become closed outlines before width, asymmetry, reflection
and placement are applied. Eagle heads select the corresponding sourced
drawing. Width, height, position, rotation, facing, tinctures and paint coverage
remain editable. Lion anatomy controls apply only to lions; there is no
procedural replacement anatomy or invented feather generator.

| Source paint | Runtime interpretation |
| --- | --- |
| `#1a1a1a` | Opaque body plus optional shadow underpainting. |
| `#2d2d2d` | Main body tincture, above the shadow underpainting. |
| `#4d4d4c` | Optional body highlights, including their source strokes. |
| `#aa0000`, `#cc0000`, `#db1212` | Armed or tongue base, shadows and highlights. |
| `#780000`, `#7a0000` | Dark accent outlines, retained in Flat. |
| `#000000`, `#111111` | Black outlines, interior drawing and pupils. |
| `#ffffff`, `#ffcc00` | Eye white and gold iris. |

Tongue identification is explicit because it shares its source palette with
the beak and claws. Beak, legs, claws and the double eagle's halos use the
armed tincture. The tongue uses the langued tincture. Counterchanging affects
body paint and its modeled tones; it preserves these separate accents.

Flat retains the full silhouette and fixed line work. The dark source shapes
also form the base silhouette, so simply deleting them would make holes.
Modeled adds adjustable shadow and highlight coverage. Highlight-colored
strokes belong to that paint layer and disappear with it. Other outlines
remain. Painted tones are baked independently of illumination; over leaf they
add pigment coverage through the existing material model. Their shapes come
from the modern SVGs, and their strengths and mixtures are authored choices.
The [painted shading study](PAINTED_SHADING.md) explains the evidence limits.

## Attribution

The selected license covers both originals and adapted eagle artwork. See
[single credit](eagles/single.credit.txt) and
[double credit](eagles/double.credit.txt) for complete reusable notices.
Fabelgeist contributors remove the shield, identify tongue paths, recolor,
separate painted tones, change line widths and asymmetry, reflect and compose
the drawing, and apply physical material rendering. Attribution follows each
used source into `ATTRIBUTION.txt`, SVG metadata and GLB copyright, including
mixed lions and eagles, quarters and inescutcheons. Repeated uses receive one
notice per source. This artwork license does not relicense the software.

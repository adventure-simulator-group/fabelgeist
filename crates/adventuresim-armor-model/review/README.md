# Parametric armor review

This evidence covers all 33 armor catalog entries and the leather boots: 48
placement meshes. The three ordinary clothing assets remain outside the armor
generator. The images below render the installed GLBs with their exported
normals and back-face culling, on the actual body.

The target is simple period-appropriate construction and proportions, adapted
to the wearer. Decoration, straps, rivets, functional hinges, individual mail
rings and exact replicas are outside this mesh scope. Recipe families and
style controls are described in the [crate README](../README.md).

## Installed outfits

![Plate outfit, neutral body](outfits/neutral-plate_harness--worn-board.jpg)

![Mail outfit, neutral body](outfits/neutral-mail_harness--worn-board.jpg)

![Padded outfit, neutral body](outfits/neutral-padded_harness--worn-board.jpg)

The [parts directory](parts/) contains front, side, three-quarter and rear
boards for every placement. The [outfits directory](outfits/) also includes
positive, negative and alternating identity blends. These are evaluated from
the exported morph targets, without generating substitute fitted meshes.

## Static checks

- All 48 GLBs pass closed physical-edge winding, finite attribute, skin-weight,
  fixed correspondence and triangle-area checks at all 47 morph endpoints and
  three combined identity blends. Coincident UV/hard-normal seams are welded
  at one micrometre for the physical-edge check.
- Neutral and three combined identity configurations each pass all 48 meshes:
  no sampled vertex, face centroid or edge midpoint lies more than 1 mm inside
  the body. These bounded nearest-surface diagnostics do not prove continuous
  collision clearance or every possible parameter combination.
- [Static audit](static-audit.json) records installed asset hashes, body
  coefficients and per-piece measurements. The retained breastplate and
  vambraces correctly retain generator version 8; new exports use version 9.
- [Independent regression review](static-regression-review.md) records the
  final four-body visual assessment and its limits. Broad historical reviewers
  separately assessed the helmet, limb and garment families during iteration.

## Historical shape references

Reference examples include the Met's [morion](https://www.metmuseum.org/art/collection/search/27150),
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

## Reproduction

The [creator README](../../adventuresim-character-creator/README.md#parametric-armor-authoring)
documents typed design overrides, filtered exports, GLB audits, static renders
and native armor capture fixtures. Use the installed asset manifest and the
same body coefficients when comparing a new candidate with this evidence.

Runtime capture validation is recorded separately from the static checks.

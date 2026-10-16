# Layered Cranach gown target

Use Anne von Wiese's documented 2024 outfit as the construction target. It is
a modern impression of Saxon court dress, not an extant gown. Match the maker's
chosen construction without presenting every detail as established period
practice. Her [construction account and dressing sequence](https://annevonwiese.com/16th-c-cranach-style-gown-of-2024/)
identify the individual garments.

The white sleeve puffs and white waist opening belong to different pieces.
The waist has a backed linen plastron attached beneath a decorated chest insert
called a *brustfleck*. Black cords lace the outer bodice over that insert. The
outer bodice is sewn to the skirt through a linen waistband. Its sleeves use
longitudinal strips and cross bands, rather than cuts through a complete tube.

## Inner garments

The [Frauenhemd construction notes](https://annevonwiese.com/16th-c-frauenhemd-chemise-with-counted-thead-shirring/)
describe a complete, low-necked chemise. Its rectangular body panels reach below
the knee. Long, full sleeves have underarm gussets, gathered shoulders, shirred
cuffs, cuff slits, buttons and loops. Repeated rhombus shirring ornaments the
front and back; chevron shirring ornaments the cuffs. Preserve the body, back,
gussets and cuff backing when the outer gown obscures them.

The dressing sequence shows a black supportive underdress over the chemise.
The maker's [underdress notes](https://annevonwiese.com/16th-c-under-layers-a-couple-underdresses-and-an-unterhemd-aka-camicia-or-chemise/)
describe fitted, layered bodices with shoulder straps and front lacing. Keep
this garment separate from the gold outer bodice. The sequence, rather than
the white alternative in that article, determines the reference color.

The [goller or partlet notes](https://annevonwiese.com/16th-c-fine-linen-goller-partlet/)
describe a separate sheer shoulder and chest covering. Its sides are open and
tie beneath the bust. It has a back opening, a separately pleated standing
collar, metallic collar trim, an inner neckband and hooks and eyes. Its smooth
chest must remain distinct from both the chemise and plastron. The
[rear photograph](https://i0.wp.com/annevonwiese.com/wp-content/uploads/2024/10/img_7725-1.png?ssl=1)
shows the concealed opening, bound edges and side ties.

## Concealed outer construction

The [inside bodice photograph](https://i0.wp.com/annevonwiese.com/wp-content/uploads/2025/02/img_8678-1.png?ssl=1)
shows black lining and a separately stitched stiffening layer. The
[armhole photograph](https://i0.wp.com/annevonwiese.com/wp-content/uploads/2025/02/img_8720-1.png?ssl=1)
shows the inner armhole finish and four white support tapes. These tapes are
separate from the chemise. Retain the sleeve-band lining, fastening hardware,
insert backing and attachment points when assembling the outfit.

## Generator contract

Reference-specific proportions, fabrics, ornament and assembly choices belong
in recipes. Shared generators express construction: fitted cloth patches,
openings, sewn panels, gathered fabric, facings, bands, ties and closures.
Do not put the maker's name or photographic coordinates into fitting code.

`BodiceDesign` describes fitted sleeveless cloth. A frozen Boolean cut retains
body-triangle correspondence across refits. Face fabric, full lining and full
or local interlining are independently thickened components with their own parametric
textiles. Armhole reach and depth are independent of the neckline. An opening
cuts every layer. This primitive does not itself represent a complete chemise,
an open-sided partlet, a paned sleeve or an assembled gown.

`PartletDesign` supplies separate front and back coverage with open flanks,
shoulder bridges and a rear slit. It can sew an evenly pleated standing collar
to the actual open neck chain before thickening. Neckline rise and anterior
slope control its seam; collar taper controls the upper circumference. Final
collar rows enclose horizontal body-ray intersections. This is a sampled fitting
constraint, not continuous clearance or posed-collision certification.

Optional folded bindings finish the free panel edges. A collar leaves its sewn
neck seam unbound; without a collar, that neckline also receives binding. Inner
neckband and outer collar trim are independent sheets, including their hidden
surfaces. Each has its own cloth gauge and appearance. The rear opening remains
open through these bands. Optional cloth ties connect the front and rear hem
corners at each flank. Their attachment bands, two bow loops, straight tails
and knot wrap remain separate thickened tapes. Width, tail length, bow span,
gauge and fabric are editable. The rest shape uses even, symmetric curves;
layer separation inside the knot follows the cloth gauge. Hook-and-eye closures
are not yet represented.

The ordered article recipe in `cranach-partlet.json` can be passed directly to
the character creator's `textile_outfit_review` example. It includes the white
neckband and gold trim; the catalog's `linen_partlet` omits the decorative trim.
These recipes describe a construction basis rather than a completed outfit.

All textile appearances carry explicit opacity. Base cloth can be translucent
while parametric ornament stays opaque. Preview and portable export use the
same straight-alpha pixels and back-face policy. `MetallicThread` supplies a
shared metal-thread approximation for trim. Directional thread sheen and woven
transmission are not simulated.

`GownDesign` sews this full upper face into the hanging skirt before shell
thickening. Both sides of the waist share physical vertices; their later
material copies retain the same positions, normals, skin and morph endpoints.
The bodice and skirt have separate fabric charts. Hidden upper lining and
stiffening remain independent sheets. The current join has a shared cloth
gauge and an uncut band below the bodice opening. It does not yet model the
maker's opening linen waistband, skirt slit or front fastenings. Those details
remain required for the complete reconstruction.

Keep authored folds evenly repeated and hems level. Cloth simulation will add
irregularity later. Do not bake the photographs' accidental wrinkles into the
rest construction. Deferred skinning and clipping polish does not waive finite
geometry, stable correspondence, real openings or complete hidden pieces.

Inspect each garment alone, the assembly after each dressing step, the back,
and exposed linings and supports. Include the wearer in front, side and
three-quarter views. Native mesh integrity, fitting scope, artistic reference
fidelity and installed runtime behavior are separate acceptance claims.

## Native outfit review export

`textile_outfit_review` exports an ordered JSON array of articles through the
shared part fitter. Each article supplies `design` (a `ParametricDesign`) and
`placement` (`worn`, `left` or `right`). Put inner garments first. The exporter
retains every article and component; it does not hide covered geometry.

```sh
cargo run --manifest-path crates/adventuresim-character-creator/Cargo.toml \
  --no-default-features --example textile_outfit_review -- \
  BODY.json OUTFIT.json OUTPUT --morph-body SECOND_BODY.json
```

The body files must share connectivity, skin and rig correspondence. Outputs
include the original body and outfit inputs, one native mesh per article,
component roles, direct morph endpoints and separate parametric color maps.
Use the outputs for whole-outfit and layer-isolation views. A successful export
alone does not establish reference fidelity or runtime visual acceptance.

# Layered Cranach gown target

Use Anne von Wiese's documented 2024 outfit as the construction target. It is a
modern impression of Saxon court dress, not an extant gown. Match the maker's
chosen construction without presenting every detail as established period
practice. Her [construction account and dressing
sequence](https://annevonwiese.com/16th-c-cranach-style-gown-of-2024/) identify
the individual garments.

The white sleeve puffs and white waist opening belong to different pieces. The
waist has a backed linen plastron attached beneath a decorated chest insert
called a *brustfleck*. Black cords lace the outer bodice over that insert. The
outer bodice is sewn to the skirt through a linen waistband. Its sleeves use
longitudinal strips and cross bands, rather than cuts through a complete tube.

## Inner garments

The [Frauenhemd construction
notes](https://annevonwiese.com/16th-c-frauenhemd-chemise-with-counted-thead-shirring/)
describe a complete, low-necked chemise. Its rectangular body panels reach below
the knee. Long, full sleeves have underarm gussets, gathered shoulders, shirred
cuffs, cuff slits, buttons and loops. Repeated rhombus shirring ornaments the
front and back; chevron shirring ornaments the cuffs. Preserve the body, back,
gussets and cuff backing when the outer gown obscures them.

The dressing sequence shows a black supportive underdress over the chemise. The
maker's [underdress
notes](https://annevonwiese.com/16th-c-under-layers-a-couple-underdresses-and-an-unterhemd-aka-camicia-or-chemise/)
describe fitted, layered bodices with shoulder straps and front lacing. Keep
this garment separate from the gold outer bodice. The sequence, rather than the
white alternative in that article, determines the reference color.

The [goller or partlet
notes](https://annevonwiese.com/16th-c-fine-linen-goller-partlet/) describe a
separate sheer shoulder and chest covering. Its sides are open and tie beneath
the bust. It has a back opening, a separately pleated standing collar, metallic
collar trim, an inner neckband and hooks and eyes. Its smooth chest must remain
distinct from both the chemise and plastron. The [rear
photograph](https://i0.wp.com/annevonwiese.com/wp-content/uploads/2024/10/img_7725-1.png?ssl=1)
shows the concealed opening, bound edges and side ties.

## Concealed outer construction

The [inside bodice
photograph](https://i0.wp.com/annevonwiese.com/wp-content/uploads/2025/02/img_8678-1.png?ssl=1)
shows black lining and a separately stitched stiffening layer. The [armhole
photograph](https://i0.wp.com/annevonwiese.com/wp-content/uploads/2025/02/img_8720-1.png?ssl=1)
shows the inner armhole finish and four white support tapes. These tapes are
separate from the chemise. Retain the sleeve-band lining, fastening hardware,
insert backing and attachment points when assembling the outfit.

## Generator contract

Reference-specific proportions, fabrics, ornament and assembly choices belong in
recipes. Shared generators express construction: fitted cloth patches, openings,
sewn panels, gathered fabric, facings, bands, ties and closures. Do not put the
maker's name or photographic coordinates into fitting code.

`BodiceDesign` describes fitted sleeveless cloth. A frozen Boolean cut retains
body-triangle correspondence across refits. Face fabric, full lining and full or
local interlining are independently thickened components with their own
parametric textiles. Armhole reach and depth are independent of the neckline. An
opening cuts every layer. This primitive does not itself represent a complete
chemise, an open-sided partlet, a paned sleeve or an assembled gown.

`PartletDesign` supplies separate front and back coverage with open flanks,
shoulder bridges and a rear slit. It can sew an evenly pleated standing collar
to the actual open neck chain before thickening. Neckline rise and anterior
slope control its seam. An optional anterior quadratic curve lowers its front
without lowering the nape or creating a sharp junction at the sides. Reach sets
the forward distance from the neck anchor where the additional drop is measured;
past that distance the curve continues along its final tangent. The rear slit
keeps its authored vertical length below the posterior seam. Collar taper
controls the upper circumference. Final collar rows enclose horizontal body-ray
intersections. This is a sampled fitting constraint, not continuous clearance or
posed-collision certification.

Optional folded bindings finish the free panel edges. A collar leaves its sewn
neck seam unbound; without a collar, that neckline also receives binding. Inner
neckband and outer collar trim are independent sheets, including their hidden
surfaces. Each has its own cloth gauge and appearance. The gathered collar also
has its own textile appearance, so it can remain white above sheer panels. The
rear opening remains open through these bands. Optional cloth ties connect the
front and rear hem corners at each flank. Their attachment bands, two bow loops,
straight tails and knot wrap remain separate thickened tapes. Width, tail
length, bow span, gauge and fabric are editable. The rest shape uses even,
symmetric curves; layer separation inside the knot follows the cloth gauge.
Hook-and-eye closures are not yet represented.

The ordered article recipe in `cranach-partlet.json` can be passed directly to
the character creator's `textile_outfit_review` example. It includes the white
neckband and gold trim; the catalog's `linen_partlet` omits the decorative trim.
The reference uses evenly repeated rounded gathers in 100 micrometre cloth. The
recipe adapts the standing collar to this wearer with a 35 mm height. Supported
collar counts reach 192. Fluted and rounded-gather profiles are separate
choices. Their physical height fields become tangent-space normal maps, using
the same height-to-normal principle as armor fluting. Fine gathers do not add
vertices or serrate the silhouette. Rounded gathers use elliptical mouths and
small valley turns that release above the sewn edge. The same template repeats
without random sag or heights. A separate free-edge flare changes the carrier
silhouette above its supporting bands. It leaves the bands and sewn neckline
fixed. Gauge validation rejects an excessively tight turn when the unbound cloth
height is too short. Relief repeats are independent of colored motif limits.

Collar sections offer two constructions. `Fitted` encloses horizontal body
sections at each row. `Oval` gives a smooth tailor-rest stand with an editable
depth-to-width ratio and circumference relative to the sewn seam. Equal arc
coordinates keep its normal-mapped gathers evenly spaced. Its seat uses a smooth two-harmonic
neck curve fitted above sampled upward-facing body crossings. The lower
transition follows the shoulder surface before reaching the stand. This is an
unposed construction query; it does not establish collision-free animation. The
`taper_height` controls this transition. It uses panel fabric on an oval stand
and collar fabric above it. Exact rows at tape edges preserve the joins.

Side openings round into the shoulder bridges. The optional `front_curve` lowers
the anterior neckline with a quadratic tangent envelope while retaining the
posterior height. Cut validation rejects disconnected or branching neck
boundaries. Not every parameter combination fits every wearer.

Collar-band charts follow the sewn seam and band rows. Their ornament does not
slide or stretch with the body's vertical neckline slope. Optional logical
charts on cloth layers survive shell thickening and refit. Other cloth layers
retain their geometric chart unless a construction authors one.

Ringed floral medallions expose petal count, ring count, thread density and
stroke width through the shared textile motif system. These approximate the
source trim's ornament; they do not certify its weave. Metallic-thread ornament
also supplies a shared tangent-space normal map from its raised thread mask.
Preview and portable export carry the same pixels. This does not simulate
directional yarn sheen. Wider strokes can merge adjacent thread rings. Raster
dimensions follow per-axis motif density within a fixed texel budget, so narrow
trims do not inherit a tall skirt texture's filtering behavior.

Runtime armor and clothing normally stay below 10,000 triangles per complete
article, including hidden cloth, bands, and ties. Count native exported triangles;
dense bake sources have a separate purpose. Fine collar and waist gathering,
threadwork, and weave use shared surface maps. Broad hanging falls, puff volume,
slash openings, and construction layers remain geometry. The dense skirt fitting
hull is independent of its render grid. Cloth-chart seams are split before
measuring physical relief dimensions, so rear wrapping cannot distort slope
units. Supports and puff panels use their own
coarse structural sampling. Puff linings share the outer cloth's compression
profile and sample its band and pane boundaries. Wide panes gain angular samples
for their volume; this does not certify continuous layer clearance.

Cloned bodice panels retain every cut-boundary vertex. Interior edges are sampled
once on the reference wearer toward a 500-face target, retaining exact source
points and freezing connectivity across morphs. A link condition preserves local
topology; reference triangles must retain orientation. Accumulated source-plane
error is limited to 3 mm RMS. This is neither a maximum surface-distance bound
nor a morph-clearance guarantee. Exported article budgets and morph endpoints
need separate validation; highly combined construction can still exceed the
normal budget.

These recipes describe a construction basis rather than a completed outfit.

All textile appearances carry explicit opacity. Base cloth can be translucent
while parametric ornament stays opaque. Preview and portable export use the same
straight-alpha pixels and back-face policy. `MetallicThread` supplies a shared
metal-thread approximation for trim. Directional thread sheen and physical woven
transmission are not simulated. A `Weave` fill represents two crossing yarn
families over the base cloth. Warp and weft counts and yarn width are
independent of decorative repeats and apply through the garment chart. Gaps
retain base opacity. Analytical pixel coverage preserves fine yarns within the
raster budget; partially covered pixels composite in linear light. The same
pixels appear in preview and portable export.

`GownDesign` sews this full upper face into the hanging skirt before shell
thickening. Both sides of the waist share physical vertices; their later
material copies retain the same positions, normals, skin and morph endpoints.
The bodice and skirt have separate fabric charts. Hidden upper lining and
stiffening remain independent sheets. The current join has a shared cloth gauge
and an uncut band below the bodice opening. It does not yet model the maker's
opening linen waistband, skirt slit or front fastenings. Those details remain
required for the complete reconstruction.

`GownBodyFit::Loose` releases an unlined, closed upper from the bound neckline.
Its radial ease and fullness-release height are independent of the suspension
height. The fitted upper retains body-source correspondence around the neckline
and armholes. Once the cloth is fully released, a straight fall continues the
sewing boundary's own columns to the hem. The lower sheet does not retain an
artificial waist seam or a second set of circumferential samples. Other skirt
profiles retain the shaped transition. Morph endpoints reuse the same source
points, sewing loops and connectivity.

Optional `SewnSleeves` share the upper carrier's armhole vertices before
thickening. Their reach, full circumference, cuff circumference and cuff length
are independent controls. Sleeves have their own fabric and construction chart;
the sleeve side of each armhole receives sleeve coordinates after material
partitioning. Surface-based skin transfer keeps them out of the skirt's leg
blend. This does not certify posed clearance or arm-only animation ownership.

`TextilePleatLayout::Shirred` draws counted-thread diamond or chevron shirring
within a bounded interval of a fabric chart. Repeat count, depth, stitch rows,
band extent and retained longitudinal gathers are parametric. `Uniform` supplies
regular longitudinal relief without a shirring band. These fields use the
shared physical-height-to-normal pipeline. Fine folds and stitch relief add no
triangles. Their chart dimensions are measured from the fitted material surface
for preview and portable export.

`cranach-chemise.json` is the construction recipe for the separate white
underlayer. Its current carrier includes the loose body and long full sleeves;
it is not the completed historical reconstruction. An optional `ClothTape`
wraps the neckline with its own width, gauge and fabric. It follows the actual
cloth face, including loose cloth. The reference fit freezes its seating
triangles and barycentric weights; morphs evaluate those same source points.
Partlet bindings use this shared tape fitter too.

`ClothFrill` adds a free hem beyond the gathered sleeve band. Its length and
flare are independent. Optional broad `ClothPleats` affect the free edge's
silhouette; fine gathering stays in the fabric normal map. `ClothSlit` cuts a
real tapered opening from the sleeve hem through the frill into the gathered
band. Length, hem gap and rotation are independent. Sleeve rotation zero is
projected world up around the canonical arm axis. Boolean cuts are made on the
reference sewing pattern and preserve their source correspondence during
refitting. Rotation wraps at the pattern seam.

The chemise still needs underarm gussets, rolled slit hems, cuff backing and
cuff closures. Its current sleeve relief places a chevron shirring band above
the free frill. The reference's hidden construction remains a separate
reconstruction requirement; the broad carrier does not imply those pieces.

Keep authored folds evenly repeated and hems level. Cloth simulation will add
irregularity later. Do not bake the photographs' accidental wrinkles into the
rest construction. Deferred skinning and clipping polish does not waive finite
geometry, stable correspondence, real openings or complete hidden pieces.

Inspect each garment alone, the assembly after each dressing step, the back, and
exposed linings and supports. Include the wearer in front, side and
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
component roles, direct morph endpoints and separate parametric color and
tangent-normal maps. Use the outputs for whole-outfit and layer-isolation views.
A successful export alone does not establish reference fidelity or runtime
visual acceptance.

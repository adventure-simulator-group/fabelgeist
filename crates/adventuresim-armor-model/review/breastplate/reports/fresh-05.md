# Fresh broad visual review — candidate 05

Independent visual review, 2026-09-09. I read the crate review guide and reference list, then actually viewed the museum photographs and every supplied current worn render before reading source, candidate measurements, implementation notes, earlier candidates, or other reviews. I did not read those other materials or edit geometry.

## Evidence actually viewed

- Rounded: Met 22741, Greenwich garniture, 1527, three-quarter photograph `22741/60375/main-image`.
- Medial ridge: Cleveland 1916.1647, Nuremberg, circa 1550, frontal object photograph on its museum collection page. The direct image URL was blocked by the browser; the object page displayed the photograph successfully.
- Peascod: Worcester 2014.736, Milan, 1575–1600, isolated front photograph `50645/preview`; Met 23939, George Clifford's garniture, 1586, three-quarter photograph `23939/64763/main-image`.
- Fluted: Cleveland 1916.1640.a, circa 1510–30, frontal object photograph on its collection page; Met 24807, Nuremberg armor, circa 1520 and later, three-quarter photograph `24807/65320/main-image`.
- All sixteen current worn images: `target/breastplate-review/05/{rounded,tapul,peascod,fluted}/renders/breastplate--worn-{front,side,quarter,rear}.png`.
- Matched bare-body images: rounded front, side, quarter, rear; tapul front and side; peascod front and side. The fluted worn cameras and body visibly match rounded, so I used the rounded bare-body views for that shared anatomical assessment.

Scope: simplified front and back shells with integral flange. Missing straps, lance rests, articulated gussets, plackarts, faulds, decoration, and exact historical plate arrangements do not count as failures. The judgments concern this photographed reference adaptation on this displayed wearer, not exact pixel matching.

## Rounded — FAIL, medium confidence

The front reads correctly as one broad, smooth enclosing volume. Front and quarter show no anatomical pectoral cups; side shows a continuous convex chest/belly returning inward, and the small lower flare is a coherent integral flange. The neckline clears the neck, and the arm opening visibly leaves the upper arm free in this pose. The overall front family is credible relative to Met 22741.

Consequential mismatches, ranked:

1. **Back shell seating is visibly loose relative to the wearer.** In `rounded/side`, the back stands away from the shoulder-blade/lumbar contour over a substantial vertical span, and its lower flare projects well beyond the bare body's corresponding contour. `rounded/quarter` also exposes a long strip of daylight between front and rear side boundaries. This reads as a detached rear casing rather than closely cooperating cuirass halves. Missing straps are excluded; the concern is the scale of the separation between the shells and wearer. The image does not establish a clearance value or prove intersection.
2. **The lower torso is longer and straighter than the reference's compact waist return.** `rounded/front` and `rounded/quarter` maintain nearly parallel lower sides until the flange. Against the matched bare front/rear views, the flange sits below the visible inward waist transition, while the rear panel also continues straight down past that transition. The reference has a more clearly gathered, compact breast-to-waist relationship. Confidence in the exact anatomical waist level is moderate because this body has a broad waist region, but the extended lower wall is visible in multiple views.

Reservation: the mounted rounded reference hides some plate boundaries and does not provide a bare wearer or rear view. It supports the front volume much better than an exact rear reconstruction. A side section or clear arm-raised lateral view would resolve the magnitude of the seating concern.

## Tapul / restrained medial ridge — FAIL, medium-high confidence

Front and quarter show a broad upper shell narrowing towards the lower edge; side establishes a lower-chest projection followed by an inward waist return. The arm and neck openings remain clear in the displayed pose. Rear shape and side separation follow the same general construction as rounded.

Consequential mismatches, ranked:

1. **The defining medial ridge dissolves into a localized rounded bulge.** `tapul/front` has a broad smooth highlight around the lower chest rather than Cleveland 1916.1647's uninterrupted central fold joining two bowed halves. `tapul/quarter` similarly reads as a localized rounded projection, and `tapul/side` confirms its concentration at the lower chest. This is more than an uncertain highlight in one view: the organizing surface flow is different across the front and quarter views.
2. **The bottom boundary lacks the reference's shallow central drop.** In `tapul/front` the flange is almost level, with a slight irregular-looking upward change towards the center, rather than the photographed shallow downward center. This weakens the connection between the ridge and lower edge.
3. **Rear seating and the open lateral separation remain consequential.** `tapul/side`, compared with its bare side, and `tapul/quarter` show the same detached rear-casing relationship described for rounded.

Reservation: the museum front photograph cannot validate the intended side depth. I am not rejecting this for failing to be a sharper, more projecting historical tapul. A verified side reference is needed before certifying a stronger tapul profile; the present failure is primarily the visible frontal medial-ridge organization.

## Peascod — FAIL, high confidence for front shape

The preset clearly moves the projection lower than tapul. Front and quarter show a central drop, and side shows a lower projecting belly. The upper shell remains broad and the neck/arm openings clear in the displayed pose. It communicates a peascod direction, but the reference's defining convergence is not yet sufficiently expressed.

Consequential mismatches, ranked:

1. **The lower front is a broad rounded apron rather than a shell converging into a low central point.** `peascod/front` shows a shallow, wide U-shaped drop and broadly rounded center surface; Worcester's front narrows markedly into a rounded but distinctly concentrated point, with the lower edges climbing towards the flanks. `peascod/quarter` confirms the broad nose and weak convergence. This is not a demand for a needle or separate spike.
2. **The center ridge is too diffuse to organize the long front.** `peascod/front` and `peascod/quarter` show a smooth central light band rather than the continuous fold linking the chest to the point in Worcester and the worn Clifford reference. Together with the broad lower boundary, it produces a rounded hanging belly rather than the references' tapered, ridged volume.
3. **Rear seating and lateral shell separation remain visible.** `peascod/side`, compared with its matched bare side, and `peascod/quarter` show a rear wall and flange standing away from the wearer, with a substantial open separation from the front shell.

Reservation: the exact amount of forward projection cannot be matched quantitatively from these photographs. The bare front does support a lower central extension relative to the flanks; the problem is its shape and surface organization, not merely whether it extends below the waist.

## Fluted rounded — FAIL, medium-high confidence

The underlying front remains a single rounded volume, and the channels are integrated surface relief rather than obvious separate rods. There are smooth margins at the neck and flange. Rear and opening behavior match rounded in the displayed pose.

Consequential mismatches, ranked:

1. **The flute field has a different dominant flow and extent from the primary reference.** `fluted/front` and `fluted/quarter` show long channels occupying most of the front height, with outer channels bowing strongly around the chest and turning back inward near their upper ends. Cleveland 1916.1640.a has a shorter bounded belly band, a substantially broader smooth upper chest, and a gentler fan. The modeled pattern reads as elongated curved ribs rather than that reference's compact band of channels. The supplemental Met armor supports finer, longer fluting, but does not establish this particular bowed field as a match to the primary target.
2. **Rear seating remains loose.** `fluted/side` and `fluted/quarter`, assessed against the matched rounded bare body, show the same rear-wall stand-off and lateral separation as rounded.
3. **The lower front retains rounded's long, nearly straight waist wall.** `fluted/front` and `fluted/quarter` have a weaker compact belly-to-waist return than the primary fluted object, independent of its excluded attached fauld.

Reservation: no parameter sweep was supplied. This verdict concerns the shown flute preset; it does not establish whether count, width, depth, fan, or extent controls can produce a closer result.

## Limits

These are visual acceptance judgments for the shown static candidate. I do not certify numerical mesh integrity, clearance, thickness, topology, all-body fitting, morph behavior, parameter range, or installed runtime behavior from these images. The most useful additional evidence for the shared seating question is an unobscured lateral view or a body/shell section at the shoulder blade, waist, and side seam; it should supplement, not replace, the current worn views.

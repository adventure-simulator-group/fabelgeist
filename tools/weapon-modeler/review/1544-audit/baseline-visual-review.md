# Baseline visual inspection

The independent reviewer actually inspected all 44 exported Rust recipes and all
42 browser recipes as labeled front/oblique mesh plates. Images are generated
from saved positions and triangle indices, not an illustrative re-creation.
The reproducible script is render-mesh-review.py. Baseline images are under
output/weapon-audit/before/reviewer-rust and reviewer-browser (six sheets each).
Per-polearm head plates supplement the full-length views. Sheet normalization is
per item; printed axial dimensions are needed for comparisons of total length.

## Findings requiring correction

- Rust dussack and runtime falchion have a knuckle bow mounted around the blade
  above the guard. Rust reitschwert/rapier repeat this. The hand is below the
  guard; its protecting bow must span the grip. Browser equivalents correctly
  protect the grip, confirming an implementation divergence rather than an
  intentional historical form.
- Katzbalger silhouettes in both implementations narrow continuously toward a
  long acute point, reading like a small thrusting sword. For the intended broad
  infantry cutting-sword default, preserve more width through the blade and use
  a shorter point transition. A broad tip is a family choice, not a claim that
  all historical Katzbalgers lacked useful points.
- Both glaives have a very broad basal shoulder, a protruding lower shelf, and
  a strongly narrowing spear-like outline. Make the edge/spine flow continuous
  and single-edged in appearance, with a restrained curve and a short point
  transition, rather than merely reducing the thickness of the same silhouette.
- The halberd's convex, near-vertical cutting edge and dropped beard read as a
  large hand axe. A modest oblique edge and shallower beard better match the
  German/Swiss first-half-sixteenth-century default described by Wallace A952.
  Convex axe edges remain valid for hand axes and other historical variants.
- Rust knife, utility knife, baselard and misericorde have massive barrel-like
  pommels; the misericorde and arming sword also have visibly excessive grips.
  These confirm the analytical inherited-furniture defects.
- The club is a plain large cylinder placed abruptly atop a thinner shaft.
  Replace the step with a tapered wooden swelling appropriate to a club.
- Polearm sockets and hammer polls are visually overbuilt; both mace defaults
  are too elongated. The Gothic Rust mace is 1.37 m long, visibly a long staff
  when its label implies a one-handed mace.

## Reviewed without a new visual proportion blocker

Browser bows, arrows, crossbows, bolts, quivers, firearms, lead ball and pouch
have coherent proportions at the reviewed defaults. The bow string/limb spaces,
crossbow tiller/prod relationship, bolt butt and fletching, and early swept
firearm stock silhouettes remain recognizable. Browser source-anchored mass
checks are close: steel crossbow 2.885 kg versus 3 kg source, Peck pistol 2.451 kg
versus 2.55 kg, matchlock 6.329 kg versus 6.15 kg, bolt quiver 0.480 kg versus
0.448 kg. These are consistency checks, not exact reconstruction certification.

All seven shield silhouettes were inspected. Buckler boss and rim, round shield
curvature, heater/kite outline, pavise central ridge and Roman shield curvature
are legible. Date and regional labels still require correction as described in
historical-assessment.md; a coherent model can nevertheless be outside 1544.

The old Rust Flat-blade depth caveat remains essential. Visually thin curved
blades can coexist with incorrect heuristic weights. Material-volume integration
and corrected thickness semantics must precede final mass acceptance.

Status: baseline review complete; final corrected meshes and numerical results
still require independent acceptance.

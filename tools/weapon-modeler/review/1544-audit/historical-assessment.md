# Weapon proportion review: Germany in 1544

Independent artistic/historical reviewer, 8 September 2026. Baseline: `f2b3fb27`.
This is implementation review evidence, not wiki prose. The review covers the
21 Rust presets, all 23 runtime melee catalog designs, and all 42 browser
presets, including ranged weapons, ammunition, carriers, and shields.

## Evidence and confidence

Museum catalog dimensions and dates below are observations. Proposed thickness,
grip, mass, and control intervals are reviewer engineering judgments constrained
by those observations, not claimed caliper measurements of those objects. A
historical family has substantial variation; a surviving head on a shortened or
replacement haft cannot establish original battlefield length. Labels must
distinguish a plausible family reconstruction from an exact object replica.

| Reference | Observations relevant to this review |
| --- | --- |
| [KHM A 1421, German sword, c.1540](https://www.khm.at/kunstwerke/schwert-371801) | Overall 1238 mm, maximum width 220 mm, 1.45 kg. A close regional/date anchor for the ordinary longsword. |
| [Met 29.158.707, German Katzbalger, c.1520](https://www.metmuseum.org/art/collection/search/33991) | Overall 813 mm, blade 682 mm, guard width 140 mm, 1.077 kg. The difference between blade and overall is only 131 mm. |
| [Musée de l'Armée J 11318, German two-handed sword, first half 16th century](https://musee-lorrain.nancy.fr/les-collections/catalogues-numeriques/la-lorraine-pour-horizon/une-premiere-lorraine-francaise/equipement-de-deux-lansquenets-imperiaux) | Overall 1790 mm, width 360 mm, 3.3 kg. The accompanying museum interpretation describes pikes exceeding five metres. |
| [Met 14.25.961, Munich two-handed sword, c.1540](https://www.metmuseum.org/art/collection/search/27350) | Overall 1929 mm, blade 1384 mm, width 409 mm, 5.103 kg. Heavy examples exist; do not impose the weight of a small longsword on every two-hander. |
| [Schwarzburg Armoury Oss 307, German Großmesser, c.1500](https://combatarchaeology.org/schwarzburg-armoury-grosmesser/) | Research by the armoury's weapons researcher gives 1085 mm overall, 867 mm blade, 195 mm quillons, 1.225 kg. It also identifies later conversion of the tip/back into a tool; do not reproduce that alteration as the original combat form. |
| [Wallace A952, German halberd, c.1500](https://wallacelive.wallacecollection.org/eMP/eMuseumPlus?module=collection&objectId=61449&service=ExternalInterface) | Head 412 mm, maximum width 240 mm, catalog mass 2.01 kg; oblique cutting edge, small fluke, diamond-section spike. Museum describes this as typical German/Swiss infantry form through the first half of the sixteenth century. |
| [Met 14.25.153, Swiss/German halberd, 1500–1525](https://www.metmuseum.org/art/collection/search/26192) | Overall 2691 mm, head 609 mm, width 257 mm, 2.66 kg. Supports a long, relatively light polearm with a forged plate head. |
| [Styrian Armoury: pike](https://www.museum-joanneum.at/en/styrian-armoury/discover/collection/staff-weapons/pike) | Original pikes reached five metres; surviving examples were often shortened by about half in the late seventeenth century. |
| [Met 14.25.1342, German war hammer, mid-sixteenth century](https://www.metmuseum.org/art/collection/search/33853) | Overall 502 mm, 0.992 kg; steel haft sheath around a wooden core, compact hammer and beak. |
| [Royal Armouries: mace VIII.123, Northern Europe, 1475–1500](https://royalarmouries.org/objects-and-stories/stories/the-hundred-years-war-1337-1453) | Approximately 500 mm and 850 g. A retained earlier mace is plausible; excessive modern-fantasy length is not. |
| [Saint Louis Art Museum, mace 231:1923, 1500–1550](https://www.slam.org/accessibility/age-of-armor-treasures-from-the-higgins-armory-collection-at-the-worcester-art-museum/) | Probably Italian, 3 lb 2 oz (about 1.42 kg). Regional comparator, not proof of German manufacture. |
| [Met 14.25.11, French military fork, sixteenth century](https://www.metmuseum.org/art/collection/search/25841) | Overall 2394 mm, head 540 mm, width 90 mm, 1.786 kg. Adjacent-region dimensional comparison only. |
| [Worcester 2018.3, Nuremberg-area rondel dagger, c.1400–1450](https://worcester.emuseum.com/objects/55157/rondel-dagger) | Overall 349 mm, blade 210 mm, rondel diameter 45 mm, 316 g; robust diamond section. Earlier retained type, not an exact 1544 fashion statement. |
| [British Museum 1881,0802.129, English fifteenth-century rondel dagger](https://www.britishmuseum.org/collection/object/H_1881-0802-129) | Overall 553 mm, blade 443 mm, 366 g. Long daggers can remain light; length alone is not evidence of a defect. |
| [Met 14.25.1572a, crossbow](https://www.metmuseum.org/art/collection/search/33738) and [museum research catalog, p.35](https://resources.metmuseum.org/resources/metpublications/pdf/A_Deadly_Art_European_Crossbows_1250_1850.pdf) | Overall 737 mm, width 624 mm, 3 kg. Late fifteenth/early sixteenth century and later alterations; engraved 1584 probably nineteenth century. Present snap lock and possibly steel prod are later changes. |
| [Met 29.158.646a–l, quiver and crossbow bolts](https://www.metmuseum.org/art/collection/search/34069) | Probably early sixteenth-century quiver: 446 mm tall, 290 mm bottom width, 448 g. Bolts mostly 368–416 mm; recorded examples 68–75 g. |
| [Met 14.25.1425, Peter Peck pistol, c.1540–1545](https://www.metmuseum.org/art/collection/search/22387) | Overall 492 mm; dated within the requested setting. Existing recipe identifies the 254/194 mm stacked barrels and 11.7 mm bore. |
| [Met 28.100.6, German matchlock, sixteenth century](https://www.metmuseum.org/art/collection/search/34811) | Overall 1603 mm, barrel 1216 mm, bore 17.7 mm, 6.15 kg. The broad dating does not establish exact availability in 1544. |

## Blocking analytical findings

1. Rust physical properties use approximate shape-volume multipliers and place
   every component's mass at its axial midpoint. Blade taper, ricasso, curvature,
   and profile changes can alter the visible solid without affecting mass or
   balance appropriately. Lateral offsets are omitted from inertia. Derive
   volume, first moments, and second moments from the same canonical solids as
   rendering; verify parameter sensitivity and conservation, rather than adding
   per-catalog weight or balance overrides.
2. The Rust cutting sword constructor supplies 12 mm thickness by default;
   several curved swords author 9–13 mm but the Flat mesh scales this depth by 0.45. Broad polearm heads are 18–28 mm plates.
   The inconsistent thickness semantics and oversized polls produce excessive mass and rotational
   inertia. Cutting blades need realistic distal taper and generally 5–8 mm
   forte thickness. Stiff thrusting estocs/daggers can retain deeper sections.
3. Rust `arming_sword` inherits the longsword's 300 mm grip and 310 mm guard;
   `misericorde` inherits the estoc's 260 mm grip; `bauernwehr` inherits a 215 mm
   messer grip. These are family-construction defects, not just stat tuning.
4. The 3350 mm pike shaft produces only about 3.6 m overall. Set the military
   pike near 4.8–5.2 m overall and derive its reach and inertia from that actual
   length. Keep the ordinary spear a separate, shorter design.
5. The Rust cavalry hammer has a solid steel shaft of 18 mm radius, whereas the
   close museum comparator uses a steel sheath around wood. Correct both the
   material construction and scale. Maces also need shortened shafts/heads and
   physically credible steel sections.

## Complete melee coverage and proposed acceptance envelopes

These envelopes guide the named default; they are not a certification of every
freeform slider combination. Mass envelopes are whole-weapon checks, not inputs
to the physics calculation. Dimensions are millimetres.

| Recipe(s), including runtime mapping | Assessment and remediation target |
| --- | --- |
| `halberd-1540`, `halberd` | Keep roughly 2.1–2.6 m overall; axe plate 6–8, spike 8–12, shaft radius 17–19; whole mass about 1.8–3.0 kg. Prefer an oblique, near-straight German cutting edge over the enlarged bearded-handaxe default. |
| `lucerne-hammer` | Length about 1.9–2.3 m is credible. Reduce 70 mm hammer face/depth toward 35–45 and spike thickness toward 8–12; retain distinct beak and top spike. |
| `pollaxe` | Roughly 1.6–2.0 m is credible. Axe thickness 8–12; hammer face 35–45, depth 30–40; total roughly 2–3.5 kg. |
| `kriegsspiess`, `military_pike` | Increase overall to about 4.8–5.2 m; keep the small 200–300 mm spearhead, not a giant blade. |
| `short-spear`, `spear` | Overall about 1.9–2.3 m is credible; reduce 85 mm-wide, 22 mm-thick Rust head to a slender 40–65 mm leaf with 6–10 mm ridge; retain taper. |
| `hunting_spear` | A broader leaf than the military spear is plausible, roughly 55–75 wide, 6–10 thick. The current head customization applies correctly; retain a broader hunting leaf. |
| `partisan` | Preserve symmetric lateral basal wings and long point; reduce 22 mm plate to 6–9; about 2–2.5 m overall and 1.8–3 kg. Regional family comparison rather than exact German specimen. |
| `glaive` | Blade length 540 is reasonable; reduce 18 mm section to 5–8, width toward 70–100, and excessive sickle-like curvature if seen in profile. |
| `hooked-bill` | Preserve recognizably forward hook/point; reduce 20 mm plate to 6–10; roughly 1.8–3.5 kg. Adjacent European type, not the defining German infantry weapon. |
| `military-fork` | Narrow 130 mm spread toward 90–110 and 22 mm tines toward 8–12; long tapered functional tines, not shovel-sized prongs. Roughly 1.5–2.5 kg. |
| `landsknecht-longsword`, `longsword` | Target blade about 900–1020, width 45–55, forte thickness 5–7, grip 200–260, guard 220–280. About 1.3–1.9 kg; the original 65 mm blade/12 mm depth/310 mm guard is overbuilt. |
| `zweihander` | The roughly 1.76 m length is appropriate. Reduce 72 mm blade width toward 50–65 and 13 mm depth toward 7–10; preserve long grip, ricasso and lugs. A 2.5–4 kg functional default is reasonable; heavier historical examples exist. |
| `katzbalger` | Blade 650–700, width 45–60, thickness 5–7; grip about 100–115. Keep the 140 mm S/figure-eight guard; target roughly 0.9–1.4 kg. |
| `grosse-messer`, `messer` | Preserve knife construction, nagel and single edge; reduce thickness toward 5–7 and avoid oversized brass pommels. Ordinary messer grip 110–150, larger variant 150–200; roughly 0.9–1.5 kg. |
| `kriegsmesser` | Large knife sword can retain 1000–1100 blade and two-handed grip about 250–320; reduce 13 mm thickness toward 7–9 and width toward 55–65. About 1.6–2.6 kg. |
| `dussack`, `falchion` | Curved cutting blade needs 4–7 mm section and 45–65 width, with consistent authored/actual thickness semantics. The enclosed-hilt model is a comparative early saber family; do not equate it with every wooden/leather fencing dussack in later manuals. |
| `estoc` | A deeper thrusting section is intentional; 8–12 mm depth with narrow 20–30 mm blade, strong distal taper, overall roughly 1.2–1.4 m. Do not impose cutting-sword flatness. |
| `rondel-dagger`, `rondel_dagger` | 380 mm blade can be retained as a long dagger, but reduce width toward 20–28 and depth toward 6–10; grip about 90–115 and rondels roughly 45–55 diameter. About 0.3–0.6 kg. |
| `reitschwert-1540`, `rapier` | Keep an early cut-and-thrust side-sword interpretation, blade roughly 850–1000, width 30–45, depth 5–7; grip 100–130. Reduce heavy 48 mm-deep bosses. Roughly 1–1.6 kg; no late cup-hilt morphology. |
| `reiter-war-hammer`, `war_hammer` | Overall about 500–650, narrow wood core/sheath, compact 25–40 mm poll, 0.8–1.5 kg. Current Rust solid steel haft is unacceptable. |
| `hand-axe`, `hand_axe` | Haft 500–700 is plausible; reach 120–150 and 8–12 mm root/wedge section instead of 180 reach/28 slab. Convex cutting edge and beard remain suitable here; about 0.8–1.6 kg. |
| `flanged-mace`, `flanged_mace` | Shorten about 0.5–0.7 m overall; compact 90–140 head and narrow steel haft. Roughly 0.8–1.5 kg. |
| `gothic-flanged-mace` | Preserve concave flange silhouette but shorten the excessive 780 shaft plus 250 head assembly. Roughly 0.6–0.8 m overall, 120–180 head, 0.9–1.7 kg. |
| `arming_sword` | Give it its own one-hand grip about 100–130 and guard 180–230; blade 750–850, 45–55 wide, 5–7 thick; about 1–1.5 kg. |
| `baselard` | Retained older dagger family is possible. Preserve recognizable transverse hilt, not an enlarged rondel pommel; 300–430 blade, 90–115 grip, 5–7 section, 0.3–0.7 kg. |
| `bauernwehr` | Single-handed utility/defensive knife; 350–480 blade, 100–130 grip, 35–50 width, 4–6 section. Keep simple scale construction and modest nagel, not a two-hand sword hilt. |
| `knife`, `utility_knife` | Blade lengths 210/135 are credible. Use simple knife furniture, 90–115 grip, 2–4 mm section; remove massive sword-derived pommels. About 0.15–0.35 kg. |
| `misericorde` | 300–360 narrow thrusting blade, 90–115 grip, modest guard/pommel; 6–9 mm ridge plausible. The original 260 grip is a clear error. |
| `club` | Wood is correct. 760 mm overall plausible, but 96 mm-diameter uniform cylinder head is visually artificial; taper/swelling should read as a wooden club. Check whole mass around 0.7–1.5 kg. |
| `walking_staff` | 1850 mm is credible; 44 mm diameter is stout. Reduce toward 30–38 mm diameter if ordinary travel staff; mass should follow wood volume, not a combat-stat override. |

## Browser ranged, ammunition, carriers and shield coverage

| Recipe | Assessment |
| --- | --- |
| `german-self-bow-1544` | 1.92 m self bow is a plausible regional family reconstruction. No exact German 1544 specimen was established. Check tapered working limbs, string brace clearance, and avoid claiming draw weight from visual dimensions alone without material elasticity. |
| `composite-recurve-bow-1544` | 1.42 m reflex-recurve construction is plausible as a Central/Eastern European or Ottoman comparison/import. Keep that provenance distinction visible; it is not a typical German infantry bow. |
| `flight-arrow-1544` | 760 mm length and 9 mm shaft diameter are plausible substantial arrow proportions. A heavy war-style arrow should not be presented as an optimized light flight arrow without evidence. |
| `arrow-quiver-1544` | 600 mm leather carrier is plausible family equipment; no exact specimen claim. Verify open usable mouth and wall-derived mass. |
| `german-cranequin-crossbow-1544` | 612 mm tiller plus stirrup can match 737 mm source overall, with 624 mm prod. Retain reconstructed nut lock but correct the source dating/alteration claim; do not reproduce a later snap lock as securely 1544. |
| `central-composite-arbalest` | 760 mm tiller and 720 mm prod are plausible comparative dimensions; retain composite layers and appropriate spanning fittings. Exact draw weight requires elastic/material parameters. |
| `light-target-crossbow-comparative` | Already correctly labeled undated comparative study. 560/460 mm scale is not evidence of a German 1544 object. |
| `crossbow-bolt-1544` | 420 mm is near the long end of the source group; 12 mm shaft plausible. Compare derived mass with 68–75 g source examples; flat butt rather than arrow nock is appropriate. |
| `bolt-quiver-1544` | Source-based 446 mm height/290 mm bottom width are correct. Check 448 g comparator using layered physical materials; an open carrier depiction is acceptable, but source also has a lid. |
| `peter-peck-double-wheellock-pistol-1545` | Strong date and dimension anchor; 492 mm overall and 254/194 mm barrels should remain. Keep stacked barrels and early swept stock; inspect paired locks and bore alignment. |
| `german-matchlock-arquebus-16c` | 1603/1216 mm and 17.7 mm bore match source. Broad sixteenth-century attribution only; derived mass should be checked against 6.15 kg, not silently overridden to it. |
| `single-wheellock-pistol-study` | Correctly labeled comparative; 470 mm overall plausible. No exact-object certification. |
| `lead-round-ball` | Radius-only lead sphere correctly determines mass. 11.4 mm diameter suits the 11.7 mm pistol bore with positive clearance; verify sphere volume, not a hand-authored projectile weight. |
| `small-arms-ball-pouch` | 150 x 130 x 55 mm leather carrier plausible; no exact specimen. Material wall thickness must determine mass. |
| `buckler` | 360 mm diameter steel buckler is plausible for the setting; check grip clearance and hollow boss. |
| `targe` | Label Scottish/other regional comparative study, not a generic German 1544 shield. |
| `round-shield` | A round shield is not intrinsically anachronistic; the large timber version remains a generic family reconstruction with lower confidence. |
| `heater-shield` | Label retained medieval/comparative shape; do not present as normal German infantry equipment in 1544. |
| `pavise` | 950 x 480 mm pavise is plausible retained protection; inspect central ridge, curvature and standing support. |
| `kite-shield` | Earlier medieval comparative study; exclude from claims of ordinary 1544 German equipment. |
| `roman-tower-shield` | Ancient comparative study; explicitly outside 1544. |

## Visual acceptance required

Analytical acceptance does not substitute for image inspection. Final review must
inspect labeled full-length and head/hilt detail views plus edge-on or oblique
views of both actual Rust and browser meshes. Check axe cutting-edge inclination,
beard and socket junction; blade taper and edge section; fork tine shape; mace
flange concavity; guard/ring orientation; hilt scale; ranged construction and
shield curvature. Attach the captured corpus and final disposition separately.

Initial status: analytical findings issued; visual acceptance pending images. Read baseline-interpretation.md alongside this document: old authored Flat-blade thickness is scaled by the mesh, so authored depth alone is not an actual thickness measurement. baseline-runtime-metrics.csv records the original heuristic gameplay outputs.


## Limits of historically grounded gameplay inference

Weapon mass, dimensions, centre of mass and rotational inertia are physical
properties of the recipe's geometry and materials. They must not be overridden
by catalog constants. The ability to present a cutting edge, a thrusting point,
or a blunt striking surface can be inferred from recipe components and their
shape parameters. Weapon control or precision factors should respond to mass
distribution and grip geometry if the combat model uses those factors.

Museum dimensions do not establish a numerical penetration or damage score.
Penetration depends on edge/point geometry, material properties, impact speed,
and the target. A game can combine recipe-derived weapon factors with these
contextual inputs without claiming to simulate every metallurgical detail.
Actor skill, technique selection and situational accuracy are properties of
the actor and combat event, not dimensions of the weapon. Keeping those separate
is consistent with the request to derive weapon-intrinsic gameplay statistics.

## Final review disposition

The analytical recommendations above were followed by implementation changes and actual before/after mesh inspection. See `after-review.md` for the final independent disposition covering 44 Rust recipes, 42 browser presets and 20 composer combinations. It supersedes the initial pending status in this document. `before-after-physical-metrics.csv` contains all 106 numerical records, and `accepted-evidence-manifest.json` identifies the final reviewed exports and image sheets.

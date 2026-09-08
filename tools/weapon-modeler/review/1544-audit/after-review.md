# Final independent historical and artistic review

**Disposition: no remaining blocking proportion or silhouette findings within the reviewed scope.** The 44 Rust recipes and 42 browser presets pass as the historically informed family reconstructions or explicitly identified comparative studies described in `historical-assessment.md`. The 20 named composer combinations pass as construction studies, not as certified 1544 German weapons. Technical test acceptance is separate and belongs to the implementing agent.

## Coverage and actual visual inspection

| Collection | Count | Analytical evidence | Visual evidence actually inspected |
| --- | ---: | --- | --- |
| Rust authoring presets | 21 | All dimensions, generated extents, mass, grip-relative centre of mass and inertia | Before and after front/oblique plates |
| Rust gameplay catalog | 23 | Every catalog design, including distinct short-weapon hilts | Before and after front/oblique plates |
| Browser presets | 42 | All melee, ranged, ammunition, carrier and shield recipes | Before and after front/oblique plates |
| Browser named composer combinations | 20 | Both haft modules with every head assembly; mass and handling differences retained | Final browser sheets 06–08 |

The original 86 baseline recipes were inspected across six Rust and six browser sheets, with halberd and glaive head detail views. All corresponding after plates were inspected. After further remediation, the four changed browser meshes (`glaive`, `landsknecht-longsword`, `grosse-messer`, `reitschwert-1540`) were independently re-rendered and inspected again. Final Rust halberd, glaive and arming-sword changes were reinspected on final sheets 01 and 03 plus halberd/glaive head details. The final expanded browser sheets 06–08 were inspected for all 20 composer combinations.

The images render the actual exported triangles, including oblique views to expose plate and hilt depth. They are not concept art. Normalizing each weapon to its panel helps inspect silhouette, while actual axial lengths and derived masses in captions prevent misleading scale comparisons. `render-mesh-review.py` reproduces the plates. `accepted-evidence-manifest.json` identifies the accepted full JSON exports and final sheet files by SHA-256.

## Resolved problems and physical interpretation

The oversized baseline shafts, blunt plate-like cutting heads, short pike, excessive sword/dagger furniture, incorrect hand-bow placement, symmetric spear-like glaive, and halberd edge/backfluke shape were corrected. The final halberd has a modest tapered rear fluke and oblique cutting edge; the glaive has a continuous asymmetric blade with a narrowed root and acute point. Short weapons have one-hand grip proportions, while the two-hand families retain longer hilts.

The authored blade-thickness semantics were corrected before selecting actual section thickness. The old Rust Flat section used only 45% of its authored depth and Fullered 90%; the original parameter alone therefore was not a physical thickness measurement. The current physical model derives mass and distribution from generated solids and materials instead of the old family volume multipliers. The final report preserves that distinction: baseline Rust masses are superseded gameplay heuristics, not measurements of the baseline meshes.

Representative final Rust outcomes are a 4.991 m / 3.533 kg pike, 0.813 m / 1.133 kg katzbalger, 1.093 m / 1.166 kg grosse-messer, 0.497 m / 0.899 kg war hammer and 0.509 m / 0.862 kg flanged mace. These now occupy credible scales relative to the museum comparators. The 1.760 m / 2.662 kg two-hander is a plausible fighting interpretation; a heavier original is not automatically wrong, as surviving period examples also exceed 5 kg.

The last two browser blockers were found by combining image inspection and component masses. `grosse-messer` originally remained 2.370 kg despite a corrected 0.520 kg blade because its oversized steel tang, guard and brass pommel dominated the mass. Those dimensions were reduced; the accepted result is 1.379 kg with centre of mass 0.096 m from the recipe grip. `reitschwert-1540` still had a 10 mm blade and 0.798 kg pommel, giving 2.274 kg overall. Its corrected result is 1.224 kg with centre of mass 0.123 m from grip, with credible furniture visible in the revised plate. No catalog mass override was requested to achieve these outcomes.

`before-after-physical-metrics.csv` contains every reviewed row, including actual mesh length, mass, centre of mass, inertia and grip-to-tip distance. Grip conventions differ between the browser and runtime: raw balance or inertia numbers should not be compared as if they use an identical holding point. Within each generator, mass distribution now meaningfully differentiates point-heavy polearms, compact sidearms and pommel-balanced swords.

## Composer and provenance boundaries

The composer exposes a 1.82 m wooden shaft of 18 mm radius and a 0.46 m steel shaft of 7 mm radius. The wood combinations span 1.898–2.361 m and 1.955–3.525 kg. The steel mace is 0.630 m and 1.373 kg. These are plausible module scales, and the final images show coherent heads and haft attachments.

A short steel haft does not historically validate every attached polearm head. The short steel halberd (0.769 m / 2.721 kg), fork (0.850 m / 2.174 kg), bill (0.840 m / 2.600 kg), glaive (1.000 m / 2.416 kg) and partisan (0.880 m / 2.374 kg), together with unusual long-shaft mace combinations, remain construction studies. Their forward centres of mass and high inertia are expected consequences of those combinations. They must be explicitly presented as studies, without a period-authenticity claim. The baseline export did not contain the 20 composer combinations; their baseline CSV cells are intentionally blank.

Ancient Roman, kite and older heater studies are distinguished from normal German equipment in 1544. The generic strapped round targe is not a German tournament shield. Ranged reconstructions retain appropriate provenance, including the altered historical crossbow source and its reconstructed nut lock. Imported/comparative families are not represented as locally typical just because their dimensions are plausible.

## Limits of acceptance

The source anchors and justified ranges are in `historical-assessment.md`. These are family interpretations, not exact-object replicas or a historical certification of arbitrary control values. Museum survivors vary by date, intended use, alteration and restoration; no single surviving weapon defines a universal category maximum. Heavier polearm variants are not forced to match one specimen's weight.

Material densities and simplified component construction remain modeling assumptions. Summed component solids are not a manufactured assembly with every hidden joinery cut or tang represented; local overlaps or omitted hidden structure can affect absolute mass. That limits the precision of comparison with weighed originals, while retaining the required causal relationship between authored dimensions/materials and physical handling. The review does not establish numerical damage or penetration from museum measurements: those require impact, target and material behavior. Actor skill and combat circumstances are separate from intrinsic weapon properties.

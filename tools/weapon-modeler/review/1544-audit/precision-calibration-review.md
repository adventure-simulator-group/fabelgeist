# Independent review of the consolidated precision calibration

This is a bounded recommendation for the user's requested single dimensionless precision/pointiness scalar. The numerical anchors are game-design targets supplied by the user, not historical measurements. There is no proposed intrinsic accuracy rating and no proposed serialized weapon damage type.

## Simple geometry-derived calibration

Use actual authored working dimensions in millimetres. For pointed blade-like components, a useful baseline is:

`precision = 2550 / (working_width_mm² + working_thickness_mm²)`

This is a concentration proxy. It deliberately uses the transverse dimensions of the working blade rather than weapon name, mass, hilt furniture or catalog ID. The numerator normalizes a 50 × 7 mm broad blade to approximately 1. A finite ordinary range such as 0.05–4 is sufficient for these current recipes; if 4 is intended as an approximate anchor rather than a maximum, retain a wider safety bound instead of flattening all narrower points to exactly 4.

| Geometry | Result before clamping |
| --- | ---: |
| Current misericorde, 24 × 9 mm | 3.88 |
| Narrow sword example, 35 × 7 mm | 2.00 |
| Broad sword, 50 × 7 mm | 1.00 |
| Current arming sword, 54 × 7 mm | 0.86 |
| Current rondel dagger, 26 × 7 mm | 3.52 |
| Current estoc, 28 × 10 mm | 2.88 |
| Current riding sword, 42 × 7 mm | 1.41 |

The current arming sword is broad. Calling it an arming sword must not force its score to 2. Conversely, narrowing the same recipe to about 35 mm should approach 2 without changing its catalog identity.

A modest bounded taper adjustment can distinguish a broad, abruptly pointed katzbalger from a continuously tapering blade. It must use the same profile/taper meaning as generated geometry, not a separate lookup based on the name. Keep any such adjustment bounded so that blade samples, vanishing tip caps and tiny cosmetic features cannot dominate the score. The simplest implementation can begin with transverse dimensions and document the omitted tip-shape nuance.

Other existing geometric primitives can use similarly explicit proxy contact areas:

- Axe: `800 / (edge_height_mm × root_thickness_mm)`. The current 160 × 10 mm hand-axe gives 0.50; the 270 × 7 mm halberd axe gives 0.42. These are working-head dimensions, not haft dimensions.
- Broad mace head: `920 / (head_length_mm × head_diameter_mm)`. The current 115 mm long, 80 mm diameter flanged head gives 0.10. Its cusp radius determines diameter; do not use the grip radius.
- Spear and beak points: apply the pointed-component expression to their working transverse section, optionally incorporating a stable geometric point-taper measure. A narrower pick should become more concentrated without an explicit piercing flag.
- Polls and blunt cylinders: use the projected striking-face/head dimensions to form a proxy area. Face dimensions should affect the result; neither the haft length nor a stored family score should substitute for them. Keep the calibration on the same documented abstract scale.

The normalizers above are calibrated game constants. They are not claims that a sword has a literal contact area equal to width squared, or that every axe engages its entire edge simultaneously.

## Aggregation and interpretive pitfalls

Only working head/blade geometry should contribute. Guard points, pommel facets, rivets and decorative spikes must not accidentally make a weapon precise. Do not sum the values of a halberd's axe, spike and beak: they are alternative contacts, not simultaneous multiplying effects.

If the model supports the actual contacting component, derive its precision for that contact. If the requested clean API retains exactly one weapon-wide scalar, choosing the most concentrated usable working surface is a simple defensible convention, but describe it as the weapon's available concentration. It cannot accurately represent both a hammer poll and its narrow pick in every attack. A mass-weighted average also has a limitation: adding a heavy poll would then make an unchanged pick geometrically less pointed. Neither approximation should be presented as exact surface physics.

Current Rust `Flat` and `Diamond` blade sections have the same generated diamond cross section. Assigning different precision merely from those enum values would introduce an unsupported hidden distinction. Fullers can change mass and inertia; they should not automatically create a pointiness bonus unrelated to the working point.

Actual infinitesimal tip-angle/contact-area measurements do not necessarily reproduce the requested anchors: a long continuously tapered sword can have a more acute geometric tip than a short dagger. Existing recipe meshes also use finite tip caps and tessellation. Sampling the last triangle therefore risks both an unintended ordering and resolution-dependent gameplay. The proposed proxy is an explicit design abstraction for the requested anchor ordering.

Material response, impact speed and the target remain contextual inputs to damage and penetration. Precision should not grant extra kinetic energy. Use the same scalar consistently through the shared contact/damage and penetration resolution; avoid multiplying it into both stages in a way that unintentionally squares the intended advantage. The user-specified 4/2/1/0.5/0.1 scale warrants a direct calibration check of final outcomes, not just the intermediate scalar.

Handling accuracy should be computed from the existing physical balance, reach and rotational inertia plus actor/situational state. Shorter reach and a centre of mass closer to the hand should improve controllability, as requested. Do not hide a second intrinsic accuracy score in the weapon catalog. The direction and reference point of any existing normalized `balance` value must be checked before using it: a larger number is not inherently better balance.

## Bounded acceptance checks

Verify the requested approximate anchors using explicit geometric examples, including a 35 mm narrow sword separate from the current broad arming-sword default. Check that changing only catalog identity leaves precision unchanged, wider/thicker working geometry does not improve concentration, guard/haft furniture does not change precision, and changing tessellation does not change it. Check that mass/balance/reach still affect handling and impact through their own physical roles. These are distinct contracts, not a new weapon-accuracy field or explicit damage-type system.

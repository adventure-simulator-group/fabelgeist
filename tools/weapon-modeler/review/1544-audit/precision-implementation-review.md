# Bounded independent review of consolidated precision

Reviewed `combat/weapon_contact.rs`, its configuration, catalog hydration, handling accuracy, contact classification and the shared armor-energy resolver. The prior accepted weapon geometry was unchanged. This review concerns conceptual behavior and calibration, not compilation or technical test acceptance.

## Finding resolved during review

The original bare-staff fallback searched only `ComponentRole::Structure`, while `walking_staff` consists of a `ComponentRole::Grip` cylinder. It therefore returned zero concentration despite having a physical striking surface. The implementation was changed to accept Structure or Grip when no working head is present. Reinspection confirms the final correction: the fallback accepts Structure or Grip and uses the same broad side-contact footprint convention as mace and club, `length × diameter`. The current 1850 mm long, 44 mm diameter staff produces `920 / (1850 × 44) = 0.0113022`, below mace 0.100 and club 0.057. Its concentrated fraction is approximately 1.12%, rather than the earlier end-face proxy's 32.2%. Its positive-lever contact follows the non-distal-head IntendedSurface path, so the derived scalar reaches contact resolution. This broad footprint remains a calibrated concentration proxy, not a claim that the whole shaft contacts the target simultaneously.

## Calibration and remaining interpretation choices

The requested anchor values are reproduced without catalog-ID ratings: current misericorde 3.881, explicit narrow-sword geometry 35 × 7 mm 2.002, broad katzbalger/longsword 1.000, hand axe 0.500 and flanged mace 0.100. The current 54 mm-wide arming sword yields 0.860, correctly reflecting a broad rather than narrow example. The independent 44-row numerical sweep is in `precision-generated-results.csv`.

The following are calibration choices to check in outcome tests, rather than claims that museum measurements establish a unique score:

- Utility knife 3.685 nearly matches misericorde 3.881 although its Cleaver profile and taper differ. The current simple width/thickness proxy intentionally omits point-profile nuance. A modest geometry-based taper correction is possible if the resulting knife behavior is undesirable; an identity override is not justified.
- Military fork 7.083 uses one tine's transverse dimensions and exceeds the approximate bodkin/misericorde anchor. This represents an available single-tine contact. It does not model simultaneous two-tine load sharing or the entire fork entering a narrow opening. Four was specified as an approximate anchor, not necessarily a hard cap, so exceeding it is not by itself a defect.
- War hammer 2.602 selects its pick over its poll's 0.898. Halberd 1.805 selects its beak over the spike's 1.500 and axe edge's 0.423. Choosing the maximum is documented and avoids summing alternative surfaces, but all primary strikes inherit this best-available-surface value. A single weapon-wide scalar cannot also distinguish which face contacted the target.

## Handling and shared resolver

Handling accuracy contains no precision or catalog identity. The equipment balance coefficient is radius of gyration divided by grip-to-tip reach, with lower values easier to redirect. Consequently the implemented length/balance product equals `(reach + radius_of_gyration) / reference_length`. Shortening the same weapon or bringing its mass distribution inward improves aim in the requested direction. Actor skill and situational contact quality remain separate inputs. Geometry-based gap access uses concentration after aim has been computed; it does not introduce a stored intrinsic accuracy rating.

The energy partition conserves incident energy. It uses concentrated energy `E × P / (1 + P)` and effective resistance `R / P`, so penetration starts when:

`E > R × (1 + P) / P²`

| Precision | Incident-energy threshold as a multiple of resistance |
| ---: | ---: |
| 4 | 0.3125 |
| 2 | 0.75 |
| 1 | 2 |
| 0.5 | 6 |
| 0.1 | 110 |

Thus the 4 versus 0.1 anchors produce a 352-fold threshold contrast. This is the deliberate consequence of the same P affecting both concentration and resistance. It is not an energy-conservation defect or a remaining review blocker. The table documents the final chosen calibration so that outcome tests can preserve it explicitly.

No remaining conceptual blocker was identified after the staff-role and footprint corrections. The final model retains the deliberate nonlinear penetration threshold and the documented single-best-surface approximation. The utility knife remains the clearest tip-profile limitation: its 3.685 score follows narrow transverse dimensions without distinguishing its Cleaver profile and taper from a purpose-built thrusting dagger. This is a limitation of the chosen simple proxy, not an identity-based exception to add. No implementation was edited by this reviewer.

use serde::{Deserialize, Serialize};
use fabelgeist_geometry::{GeometryResult, SignedLength};

use crate::combat_style::MeleeAttackStyle;
use crate::item_catalog_schema::EquipmentMaterial;

/// Authored weapon geometry and the actual spatial state of one committed
/// attack at its contact instant. Distances are measured along the attacker's
/// line to the target surface, in metres.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeleeContactAtTimeFacts {
    pub scheduled_measure_metres: SignedLength,
    pub actual_measure_metres: SignedLength,
    /// Authored preferred striking measure; scheduling never redefines it.
    pub ideal_measure_metres: SignedLength,
    pub effective_reach_metres: SignedLength,
    pub grip_to_tip_metres: SignedLength,
    pub total_length_metres: SignedLength,
    pub striking_head_length_metres: SignedLength,
    pub distal_headed: bool,
    pub attack_style: MeleeAttackStyle,
    pub body_material: Option<EquipmentMaterial>,
    pub striking_material: Option<EquipmentMaterial>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeleeContactClassification {
    IntendedSurface,
    Haft,
    Pommel,
    InvalidatedMiss,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeleeContactInvalidationCause {
    OutsideReach,
    InvalidGeometry,
}

/// Radius of the authoritative tactical humanoid collision cylinder.
pub const HUMANOID_COLLISION_RADIUS_METRES: f32 = 0.4;
/// Shoulder-to-hand reach of the reference humanoid used when autoresolve has
/// no per-instance tactical dimensions.
pub const HUMANOID_REFERENCE_ARM_REACH_METRES: f32 = 0.526_801;

/// Two authoritative humanoid collision radii. Autoresolve therefore cannot
/// project the combatants' centers through one another.
pub const HUMANOID_MELEE_MINIMUM_CENTER_SEPARATION_METRES: f32 =
    HUMANOID_COLLISION_RADIUS_METRES * 2.0;

#[must_use]
pub fn has_distal_striking_surface(
    grip_to_tip_metres: SignedLength,
    striking_head_length_metres: SignedLength,
    body_material: Option<EquipmentMaterial>,
    striking_material: Option<EquipmentMaterial>,
) -> bool {
    grip_to_tip_metres.metres() > 0.0
        && striking_head_length_metres.metres() > 0.0
        && striking_head_length_metres.metres() < grip_to_tip_metres.metres() * 0.5
        && body_material != striking_material
}

/// Preferred surface measure for a melee weapon. Distal-headed weapons seek
/// the center of their authored striking band; continuous blades and other
/// weapons retain the authored fallback reach fraction.
pub fn preferred_melee_striking_measure(
    effective_reach_metres: SignedLength,
    grip_to_tip_metres: SignedLength,
    striking_head_length_metres: SignedLength,
    distal_headed: bool,
    fallback_reach_fraction: f32,
) -> GeometryResult<SignedLength> {
    // Complete local-axis striking-band calculation, retaining the original
    // native max/min/clamp and f32 operation order before admitting its output.
    let reach = effective_reach_metres.metres().max(0.0);
    if !distal_headed || grip_to_tip_metres.metres() <= 0.0 {
        return SignedLength::from_metres(reach * fallback_reach_fraction.clamp(0.0, 1.0));
    }
    let grip = grip_to_tip_metres.metres().min(reach);
    let grip_origin = (reach - grip).max(0.0);
    let head_start = grip_origin + (grip - striking_head_length_metres.metres().clamp(0.0, grip)).max(0.0);
    SignedLength::from_metres((head_start + reach) * 0.5)
}

/// Physical revalidation of a scheduled strike against the target's current
/// measure. `energy_fraction` is dimensionless: for shortened rotational
/// contacts it is the square of actual over intended lever arm, following
/// `E = 1/2 I omega^2` and local contact speed `v = omega r`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeleeContactAtTime {
    pub classification: MeleeContactClassification,
    pub scheduled_measure_metres: SignedLength,
    pub actual_measure_metres: SignedLength,
    pub ideal_measure_metres: SignedLength,
    pub lever_arm_metres: SignedLength,
    pub energy_fraction: f32,
    /// Continuous accuracy contribution from deviation from the committed
    /// ideal measure. This never independently adjudicates a hit in reach.
    pub measure_accuracy_multiplier: f32,
    pub invalidation_cause: Option<MeleeContactInvalidationCause>,
    pub contact_material: Option<EquipmentMaterial>,
}

impl MeleeContactAtTime {
    pub const fn intended(measure_metres: SignedLength) -> Self {
        Self {
            classification: MeleeContactClassification::IntendedSurface,
            scheduled_measure_metres: measure_metres,
            actual_measure_metres: measure_metres,
            ideal_measure_metres: measure_metres,
            lever_arm_metres: SignedLength::ZERO,
            energy_fraction: 1.0,
            measure_accuracy_multiplier: 1.0,
            invalidation_cause: None,
            contact_material: None,
        }
    }

    fn invalid(facts: MeleeContactAtTimeFacts, cause: MeleeContactInvalidationCause) -> Self {
        Self {
            classification: MeleeContactClassification::InvalidatedMiss,
            scheduled_measure_metres: facts.scheduled_measure_metres,
            actual_measure_metres: facts.actual_measure_metres,
            ideal_measure_metres: facts.ideal_measure_metres,
            lever_arm_metres: SignedLength::ZERO,
            energy_fraction: 0.0,
            measure_accuracy_multiplier: 0.0,
            invalidation_cause: Some(cause),
            contact_material: None,
        }
    }
}

/// Classifies the physical surface and lever available at contact measure.
/// This never adjudicates an in-reach hit: a polearm contacted inside its
/// distal band can meet shaft or butt, but cannot deliver unchanged head-edge
/// energy intended for outer measure.
pub fn resolve_melee_contact_at_time(facts: MeleeContactAtTimeFacts) -> GeometryResult<MeleeContactAtTime> {
    if let Some(cause) = invalid_contact_cause(facts) {
        return Ok(MeleeContactAtTime::invalid(facts, cause));
    }
    let measure_accuracy_multiplier = measure_accuracy_multiplier(facts);
    if facts.grip_to_tip_metres.metres() <= 0.0 {
        return Ok(unarmed_contact(facts, measure_accuracy_multiplier));
    }

    let grip_origin_measure = (facts.effective_reach_metres.metres() - facts.grip_to_tip_metres.metres()).max(0.0);
    let lever_arm = SignedLength::from_metres(facts.actual_measure_metres.metres() - grip_origin_measure)?;
    let butt_length = SignedLength::from_metres((facts.total_length_metres.metres() - facts.grip_to_tip_metres.metres()).max(0.0))?;
    if lever_arm.metres() <= 0.0 {
        return close_contact(facts, lever_arm, butt_length, measure_accuracy_multiplier);
    }

    resolve_positive_lever_contact(facts, lever_arm, measure_accuracy_multiplier)
}

fn invalid_contact_cause(facts: MeleeContactAtTimeFacts) -> Option<MeleeContactInvalidationCause> {
    let values = [
        facts.scheduled_measure_metres,
        facts.actual_measure_metres,
        facts.ideal_measure_metres,
        facts.effective_reach_metres,
        facts.grip_to_tip_metres,
        facts.total_length_metres,
        facts.striking_head_length_metres,
    ];
    if values
        .into_iter()
        .any(|value| value.metres() < 0.0)
        || facts.effective_reach_metres.metres() <= 0.0
    {
        return Some(MeleeContactInvalidationCause::InvalidGeometry);
    }
    if facts.actual_measure_metres.metres() > facts.effective_reach_metres.metres() {
        return Some(MeleeContactInvalidationCause::OutsideReach);
    }
    None
}

fn unarmed_contact(
    facts: MeleeContactAtTimeFacts,
    measure_accuracy_multiplier: f32,
) -> MeleeContactAtTime {
    MeleeContactAtTime {
        classification: MeleeContactClassification::IntendedSurface,
        scheduled_measure_metres: facts.scheduled_measure_metres,
        actual_measure_metres: facts.actual_measure_metres,
        ideal_measure_metres: facts.ideal_measure_metres,
        lever_arm_metres: SignedLength::ZERO,
        energy_fraction: 1.0,
        measure_accuracy_multiplier,
        invalidation_cause: None,
        contact_material: facts.striking_material.or(facts.body_material),
    }
}

fn close_contact(
    facts: MeleeContactAtTimeFacts,
    lever_arm: SignedLength,
    butt_length: SignedLength,
    measure_accuracy_multiplier: f32,
) -> GeometryResult<MeleeContactAtTime> {
    if -lever_arm.metres() <= butt_length.metres() && facts.attack_style == MeleeAttackStyle::Swing {
        let fraction = (-lever_arm.metres() / facts.grip_to_tip_metres.metres()).clamp(0.0, 1.0);
        return Ok(MeleeContactAtTime {
            classification: MeleeContactClassification::Pommel,
            scheduled_measure_metres: facts.scheduled_measure_metres,
            actual_measure_metres: facts.actual_measure_metres,
            ideal_measure_metres: facts.ideal_measure_metres,
            lever_arm_metres: lever_arm.negated(),
            energy_fraction: fraction * fraction,
            measure_accuracy_multiplier,
            invalidation_cause: None,
            contact_material: facts.body_material,
        });
    }
    let close_contact_lever = butt_length
        .max(facts.grip_to_tip_metres.scaled_by(0.1)?)
        .min(facts.grip_to_tip_metres);
    let close_lever = facts
        .actual_measure_metres
        .max(close_contact_lever)
        .min(facts.grip_to_tip_metres);
    Ok(MeleeContactAtTime {
        classification: MeleeContactClassification::Haft,
        scheduled_measure_metres: facts.scheduled_measure_metres,
        actual_measure_metres: facts.actual_measure_metres,
        ideal_measure_metres: facts.ideal_measure_metres,
        lever_arm_metres: close_lever,
        energy_fraction: (close_lever.metres() / facts.grip_to_tip_metres.metres())
            .clamp(0.0, 1.0)
            .powi(2),
        measure_accuracy_multiplier,
        invalidation_cause: None,
        contact_material: facts.body_material,
    })
}

fn resolve_positive_lever_contact(
    facts: MeleeContactAtTimeFacts,
    lever_arm: SignedLength,
    measure_accuracy_multiplier: f32,
) -> GeometryResult<MeleeContactAtTime> {
    let head_length = if facts.distal_headed {
        facts
            .striking_head_length_metres.metres()
            .clamp(0.0, facts.grip_to_tip_metres.metres())
    } else {
        // A sword/knife has a continuous striking blade from guard to tip;
        // closeness shortens lever arm but does not turn steel blade into haft.
        facts.grip_to_tip_metres.metres()
    };
    let head_begins_at = facts.grip_to_tip_metres.metres() - head_length;
    if lever_arm.metres() >= head_begins_at {
        return Ok(MeleeContactAtTime {
            classification: MeleeContactClassification::IntendedSurface,
            scheduled_measure_metres: facts.scheduled_measure_metres,
            actual_measure_metres: facts.actual_measure_metres,
            ideal_measure_metres: facts.ideal_measure_metres,
            lever_arm_metres: lever_arm.min(facts.grip_to_tip_metres),
            energy_fraction: (lever_arm.metres() / facts.grip_to_tip_metres.metres())
                .clamp(0.0, 1.0)
                .powi(2),
            measure_accuracy_multiplier,
            invalidation_cause: None,
            contact_material: facts.striking_material,
        });
    }

    let lever_fraction = (lever_arm.metres() / facts.grip_to_tip_metres.metres()).clamp(0.0, 1.0);
    Ok(MeleeContactAtTime {
        classification: MeleeContactClassification::Haft,
        scheduled_measure_metres: facts.scheduled_measure_metres,
        actual_measure_metres: facts.actual_measure_metres,
        ideal_measure_metres: facts.ideal_measure_metres,
        lever_arm_metres: lever_arm,
        energy_fraction: lever_fraction * lever_fraction,
        measure_accuracy_multiplier,
        invalidation_cause: None,
        contact_material: facts.body_material,
    })
}

fn measure_accuracy_multiplier(facts: MeleeContactAtTimeFacts) -> f32 {
    let ideal = facts
        .ideal_measure_metres.metres()
        .clamp(0.0, facts.effective_reach_metres.metres());
    let deviation = (facts.actual_measure_metres.metres() - ideal).abs();
    let style_tolerance = match facts.attack_style {
        MeleeAttackStyle::Swing => 0.30,
        MeleeAttackStyle::Stab => 0.18,
    };
    let striking_tolerance = if facts.distal_headed {
        facts.striking_head_length_metres.metres() * 0.5
    } else {
        facts.grip_to_tip_metres.metres() * style_tolerance
    };
    let tolerance = striking_tolerance.max(facts.effective_reach_metres.metres() * 0.08);
    (tolerance / (tolerance + deviation)).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn halberd(actual_measure_metres: f32) -> MeleeContactAtTimeFacts {
        MeleeContactAtTimeFacts {
            scheduled_measure_metres: SignedLength::from_metres(2.0).unwrap(),
            actual_measure_metres: SignedLength::from_metres(actual_measure_metres).unwrap(),
            ideal_measure_metres: SignedLength::from_metres(1.92).unwrap(),
            effective_reach_metres: SignedLength::from_metres(2.0).unwrap(),
            grip_to_tip_metres: SignedLength::from_metres(1.9).unwrap(),
            total_length_metres: SignedLength::from_metres(2.1).unwrap(),
            striking_head_length_metres: SignedLength::from_metres(0.16).unwrap(),
            distal_headed: true,
            attack_style: MeleeAttackStyle::Swing,
            body_material: Some(EquipmentMaterial::Hardwood),
            striking_material: Some(EquipmentMaterial::RoughSteel),
        }
    }

    fn arming_sword(actual_measure_metres: f32) -> MeleeContactAtTimeFacts {
        MeleeContactAtTimeFacts {
            scheduled_measure_metres: SignedLength::from_metres(1.25).unwrap(),
            actual_measure_metres: SignedLength::from_metres(actual_measure_metres).unwrap(),
            ideal_measure_metres: SignedLength::from_metres(0.875).unwrap(),
            effective_reach_metres: SignedLength::from_metres(1.25).unwrap(),
            grip_to_tip_metres: SignedLength::from_metres(1.0).unwrap(),
            total_length_metres: SignedLength::from_metres(1.2).unwrap(),
            striking_head_length_metres: SignedLength::from_metres(1.0).unwrap(),
            distal_headed: false,
            attack_style: MeleeAttackStyle::Swing,
            body_material: Some(EquipmentMaterial::RoughSteel),
            striking_material: Some(EquipmentMaterial::RoughSteel),
        }
    }

    fn war_hammer(actual_measure_metres: f32) -> MeleeContactAtTimeFacts {
        MeleeContactAtTimeFacts {
            scheduled_measure_metres: SignedLength::from_metres(0.8).unwrap(),
            actual_measure_metres: SignedLength::from_metres(actual_measure_metres).unwrap(),
            ideal_measure_metres: SignedLength::from_metres(0.72).unwrap(),
            effective_reach_metres: SignedLength::from_metres(0.8).unwrap(),
            grip_to_tip_metres: SignedLength::from_metres(0.56).unwrap(),
            total_length_metres: SignedLength::from_metres(0.70).unwrap(),
            striking_head_length_metres: SignedLength::from_metres(0.16).unwrap(),
            distal_headed: true,
            attack_style: MeleeAttackStyle::Swing,
            body_material: Some(EquipmentMaterial::Hardwood),
            striking_material: Some(EquipmentMaterial::RoughSteel),
        }
    }

    #[test]
    fn exact_polearm_closure_revalidates_as_lower_energy_haft_contact() {
        let contact = resolve_melee_contact_at_time(halberd(1.25)).unwrap();
        assert_eq!(contact.classification, MeleeContactClassification::Haft);
        assert!((contact.lever_arm_metres.metres() - 1.15).abs() < 0.001);
        assert!((contact.energy_fraction - (1.15_f32 / 1.9).powi(2)).abs() < 0.001);
        assert!(contact.energy_fraction < 0.4);
        assert_eq!(contact.contact_material, Some(EquipmentMaterial::Hardwood));
    }

    #[test]
    fn polearm_contact_energy_is_monotonic_inside_the_head() {
        let inner = resolve_melee_contact_at_time(halberd(0.8)).unwrap();
        let middle = resolve_melee_contact_at_time(halberd(1.25)).unwrap();
        let outer = resolve_melee_contact_at_time(halberd(1.7)).unwrap();
        assert!(inner.energy_fraction < middle.energy_fraction);
        assert!(middle.energy_fraction < outer.energy_fraction);
        assert!(outer.energy_fraction <= 1.0);
    }

    #[test]
    fn head_to_haft_boundary_conserves_a_continuous_rotational_energy_budget() {
        let boundary = (2.0 - 1.9) + (1.9 - 0.16);
        let just_inside = resolve_melee_contact_at_time(halberd(boundary - 0.000_01)).unwrap();
        let at_head = resolve_melee_contact_at_time(halberd(boundary)).unwrap();
        assert_eq!(just_inside.classification, MeleeContactClassification::Haft);
        assert_eq!(
            at_head.classification,
            MeleeContactClassification::IntendedSurface
        );
        assert!((just_inside.energy_fraction - at_head.energy_fraction).abs() < 0.000_1);
        assert!(at_head.energy_fraction <= 1.0);
    }

    #[test]
    fn sword_at_body_contact_cannot_deliver_full_tip_energy() {
        let contact = resolve_melee_contact_at_time(arming_sword(
            HUMANOID_MELEE_MINIMUM_CENTER_SEPARATION_METRES,
        )).unwrap();
        assert_eq!(
            contact.classification,
            MeleeContactClassification::IntendedSurface
        );
        assert!((contact.lever_arm_metres.metres() - 0.55).abs() < 0.001);
        assert!((contact.energy_fraction - 0.55_f32.powi(2)).abs() < 0.001);
        assert!(contact.energy_fraction < 0.31);
    }

    #[test]
    fn hammer_inside_its_head_band_is_a_haft_not_a_full_head_contact() {
        let contact = resolve_melee_contact_at_time(war_hammer(0.5)).unwrap();
        assert_eq!(contact.classification, MeleeContactClassification::Haft);
        assert_eq!(contact.contact_material, Some(EquipmentMaterial::Hardwood));
        assert!(contact.energy_fraction < 0.25);
    }

    #[test]
    fn hammer_head_to_haft_transition_is_energy_continuous() {
        let boundary = (0.8 - 0.56) + (0.56 - 0.16);
        let haft = resolve_melee_contact_at_time(war_hammer(boundary - 0.000_1)).unwrap();
        let head = resolve_melee_contact_at_time(war_hammer(boundary + 0.000_1)).unwrap();
        assert_eq!(haft.classification, MeleeContactClassification::Haft);
        assert_eq!(
            head.classification,
            MeleeContactClassification::IntendedSurface
        );
        assert!((haft.energy_fraction - head.energy_fraction).abs() < 0.001);
        assert_eq!(haft.contact_material, Some(EquipmentMaterial::Hardwood));
        assert_eq!(head.contact_material, Some(EquipmentMaterial::RoughSteel));
    }

    #[test]
    fn hammer_butt_can_only_make_a_low_energy_pommel_contact() {
        let contact = resolve_melee_contact_at_time(war_hammer(0.2)).unwrap();
        assert_eq!(contact.classification, MeleeContactClassification::Pommel);
        assert!(contact.energy_fraction < 0.01);
        assert_eq!(contact.contact_material, Some(EquipmentMaterial::Hardwood));
    }

    #[test]
    fn only_outside_reach_is_a_spatial_miss() {
        let outside = resolve_melee_contact_at_time(halberd(2.01)).unwrap();
        assert_eq!(
            outside.classification,
            MeleeContactClassification::InvalidatedMiss
        );
        assert_eq!(
            outside.invalidation_cause,
            Some(MeleeContactInvalidationCause::OutsideReach)
        );
        let inside = resolve_melee_contact_at_time(halberd(0.0)).unwrap();
        assert_ne!(
            inside.classification,
            MeleeContactClassification::InvalidatedMiss
        );
        assert!(inside.energy_fraction > 0.0);
    }

    #[test]
    fn distal_preferred_measure_is_head_band_center_not_generic_fraction() {
        let preferred = preferred_melee_striking_measure(SignedLength::from_metres(2.0).unwrap(), SignedLength::from_metres(1.9).unwrap(), SignedLength::from_metres(0.16).unwrap(), true, 0.7).unwrap().metres();
        assert!((preferred - 1.92).abs() < 1.0e-6);
        assert!(preferred > 2.0 * 0.7);
        assert_eq!(
            preferred_melee_striking_measure(SignedLength::from_metres(1.25).unwrap(), SignedLength::from_metres(1.0).unwrap(), SignedLength::from_metres(1.0).unwrap(), false, 0.7).unwrap().metres(),
            0.875
        );
    }

    #[test]
    fn scheduled_distance_does_not_redefine_authored_ideal_measure() {
        let first = resolve_melee_contact_at_time(halberd(1.5)).unwrap();
        let mut rescheduled = halberd(1.5);
        rescheduled.scheduled_measure_metres = SignedLength::from_metres(0.25).unwrap();
        let second = resolve_melee_contact_at_time(rescheduled).unwrap();
        assert_eq!(first.ideal_measure_metres, second.ideal_measure_metres);
        assert_eq!(
            first.measure_accuracy_multiplier,
            second.measure_accuracy_multiplier
        );
    }

    #[test]
    fn measure_accuracy_is_continuous_nonzero_and_best_at_authored_measure() {
        let ideal = resolve_melee_contact_at_time(halberd(1.92)).unwrap();
        let near = resolve_melee_contact_at_time(halberd(1.8)).unwrap();
        let clinch = resolve_melee_contact_at_time(halberd(0.0)).unwrap();
        assert_eq!(ideal.measure_accuracy_multiplier, 1.0);
        assert!(near.measure_accuracy_multiplier < ideal.measure_accuracy_multiplier);
        assert!(clinch.measure_accuracy_multiplier > 0.0);
        assert!(clinch.energy_fraction > 0.0);
    }
}

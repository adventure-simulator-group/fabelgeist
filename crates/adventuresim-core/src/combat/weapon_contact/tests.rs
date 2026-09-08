use super::*;
use crate::autoresolve::{CombatEquipment, CombatWeapon};
use crate::combat::EMBEDDED_COMBAT_RESOLUTION_PARAMETERS;
use adventuresim_weapon_model::{ComponentShape, Millimeters, default_design};

#[test]
fn handling_improves_with_shorter_lever_and_better_balance_only() {
    let config = EMBEDDED_COMBAT_RESOLUTION_PARAMETERS.contact;
    let accuracy = |length, balance, precision| {
        config.handling_accuracy(&CombatEquipment {
            weapon: Some(CombatWeapon {
                grip_to_tip_m: length,
                balance,
                precision,
                ..Default::default()
            }),
            ..Default::default()
        })
    };
    assert!(accuracy(0.5, 0.5, 1.0) > accuracy(2.0, 0.5, 1.0));
    assert!(accuracy(1.0, 0.1, 1.0) > accuracy(1.0, 0.9, 1.0));
    assert_eq!(accuracy(1.0, 0.5, 0.1), accuracy(1.0, 0.5, 4.0));
}

#[test]
fn working_sections_match_reference_weapons_and_respond_to_thickness() {
    let config = EMBEDDED_COMBAT_RESOLUTION_PARAMETERS.contact;
    for (id, expected) in [
        ("misericorde", 4.0),
        ("longsword", 1.0),
        ("hand_axe", 0.5),
        ("flanged_mace", 0.1),
    ] {
        let value = config
            .precision_for_design(&default_design(id).unwrap())
            .value();
        assert!((value / expected - 1.0).abs() < 0.05, "{id}: {value}");
    }
    let mut narrow = default_design("arming_sword").unwrap();
    let blade = narrow
        .components
        .iter_mut()
        .find_map(|component| match &mut component.shape {
            ComponentShape::Blade(blade) => Some(blade),
            _ => None,
        })
        .unwrap();
    blade.width = Millimeters(35);
    blade.thickness = Millimeters(7);
    let thin = config.precision_for_design(&narrow).value();
    assert!((thin - 2.0).abs() < 0.02);
    if let ComponentShape::Blade(blade) = &mut narrow
        .components
        .iter_mut()
        .find(|component| matches!(component.shape, ComponentShape::Blade(_)))
        .unwrap()
        .shape
    {
        blade.thickness = Millimeters(14);
    }
    assert!(config.precision_for_design(&narrow).value() < thin);
}

#[test]
fn every_recipe_has_finite_contact_and_furniture_does_not_add_precision() {
    let config = EMBEDDED_COMBAT_RESOLUTION_PARAMETERS.contact;
    for id in adventuresim_weapon_model::MELEE_CATALOG_IDS {
        let design = default_design(id).unwrap();
        let precision = config.precision_for_design(&design).value();
        assert!(
            precision.is_finite() && precision > 0.0,
            "{id}: {precision}"
        );
        let mut duplicate = design.clone();
        duplicate.components.extend(design.components);
        assert_eq!(config.precision_for_design(&duplicate).value(), precision);
    }
    assert!(
        config
            .precision_for_design(&default_design("walking_staff").unwrap())
            .value()
            < 0.1
    );
}

#[test]
fn precision_changes_concentration_and_resistance_monotonically() {
    let mut previous_fraction = 0.0;
    let mut previous_resistance = f32::INFINITY;
    for value in [0.1, 0.5, 1.0, 2.0, 4.0] {
        let precision = ContactPrecision::new(value);
        assert!(precision.concentrated_fraction() > previous_fraction);
        assert!(precision.resistance(100.0) < previous_resistance);
        previous_fraction = precision.concentrated_fraction();
        previous_resistance = precision.resistance(100.0);
    }
}

#[test]
fn condition_scales_custom_geometry_once() {
    let base = crate::item_catalog::weapon_precision("arming_sword").unwrap();
    let geometry =
        crate::equipment::ParametricWeaponCombatGeometry::new(1.0, 1.0, 0.8, 0.7, 0.2, 0.4, 2.0)
            .unwrap();
    assert!((geometry.conditioned_precision("arming_sword", base * 0.5) - 1.0).abs() < 0.0001);
}

//! Observable admission and authority contracts shared by native and browser clients.

use adventuresim_tactical_core::{combat_config::TacticalCombatConfig, control::*};
use bevy::{ecs::entity::MapEntities, prelude::*};

#[test]
fn switching_away_and_back_never_readmits_old_direct_input() {
    let mut control = CharacterControl::default();
    let original = control.generation;
    assert!(control.accepts_direct(original));
    control.switch(ControlMode::Indirect);
    assert!(!control.accepts_direct(original));
    control.switch(ControlMode::Direct);
    assert!(!control.accepts_direct(original));
    assert!(control.accepts_direct(control.generation));
}

#[test]
fn decoding_rejects_nonfinite_destinations() {
    assert!(serde_json::from_str::<NavigationDestination>(r#"{"position":[1,1e100,3]}"#).is_err());
}

#[test]
fn order_maps_only_its_attack_target() {
    let from = Entity::from_bits(5);
    let to = Entity::from_bits(9);
    let mut order = CharacterOrder {
        goal: CharacterGoal::Attack { target: from },
        ..default()
    };
    order.map_entities(&mut (from, to));
    assert_eq!(order.goal, CharacterGoal::Attack { target: to });
}

#[test]
fn control_configuration_rejects_invalid_lengths() {
    let config = TacticalCombatConfig::default();
    let mut value = serde_json::to_value(&config).unwrap();
    value["ai"]["control"]["arrival_radius_metres"] = (-1).into();
    assert!(serde_json::from_value::<TacticalCombatConfig>(value).is_err());
}

#[test]
fn control_configuration_rejects_nonpositive_timeouts_and_sensitivity() {
    let config = TacticalCombatConfig::default();
    for field in [
        "no_progress_seconds",
        "threat_memory_seconds",
        "mesh_rebuild_interval_seconds",
        "orbit_mouse_degrees_per_pixel",
    ] {
        for invalid in [0, -1] {
            let mut value = serde_json::to_value(&config).unwrap();
            value["ai"]["control"][field] = invalid.into();
            assert!(
                serde_json::from_value::<TacticalCombatConfig>(value).is_err(),
                "{field} accepted {invalid}"
            );
        }
    }
    for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(PositiveControlDuration::try_from(invalid).is_err());
        assert!(OrbitSensitivity::try_from(invalid).is_err());
    }
}

#[test]
fn authored_configuration_round_trips_with_control_contracts() {
    let config = TacticalCombatConfig::default();
    config.validate().unwrap();
    let decoded: TacticalCombatConfig =
        serde_json::from_slice(&serde_json::to_vec(&config).unwrap()).unwrap();
    assert_eq!(decoded, config);
}

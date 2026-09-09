use adventuresim_core::{
    equipment::{EquipmentGraph, EquipmentGraphPlacement},
    item_catalog::{
        self, EquipmentChannel, EquipmentFitZone, EquipmentLocation, OccupancyRequirement,
    },
};

#[test]
fn complete_plate_harness_coexists_and_duplicate_piece_is_rejected() {
    let mut graph = EquipmentGraph::default();
    let mut next_id = 1;
    for item in [
        "morion",
        "gorget",
        "cuirass",
        "fauld",
        "spaulder",
        "rerebrace",
        "couter",
        "vambrace",
        "mitten_gauntlet",
        "cuisse",
        "poleyn",
        "greave",
        "sabaton",
    ] {
        for placement in &item_catalog::definition(item)
            .unwrap()
            .equipment
            .as_ref()
            .unwrap()
            .placements
        {
            let body = EquipmentGraphPlacement {
                body: placement.occupancy.clone(),
                parents: vec![],
            };
            graph
                .equip(next_id, body.clone())
                .unwrap_or_else(|error| panic!("{item}/{}: {error}", placement.id));
            assert_eq!(
                graph.equip(next_id + 100, body),
                Err("body occupancy conflict"),
                "duplicate {item}/{}",
                placement.id
            );
            next_id += 1;
        }
    }
    assert_eq!(graph.nodes.len(), 22);
}

#[test]
fn whole_limb_and_same_fit_zone_conflict_but_adjacent_plates_and_padding_coexist() {
    let forearm = OccupancyRequirement {
        location: EquipmentLocation::LeftArm,
        channel: EquipmentChannel::RigidArmor,
        order: 0,
        fit_zone: Some(EquipmentFitZone::Forearm),
    };
    assert!(forearm.conflicts_with(OccupancyRequirement {
        order: 9,
        ..forearm
    }));
    let whole_arm = OccupancyRequirement {
        fit_zone: None,
        ..forearm
    };
    assert!(forearm.conflicts_with(whole_arm));
    assert!(whole_arm.conflicts_with(forearm));
    assert!(!forearm.conflicts_with(OccupancyRequirement {
        fit_zone: Some(EquipmentFitZone::Elbow),
        ..forearm
    }));
    assert!(!forearm.conflicts_with(OccupancyRequirement {
        channel: EquipmentChannel::Padding,
        ..whole_arm
    }));
    assert!(!forearm.conflicts_with(OccupancyRequirement {
        location: EquipmentLocation::RightArm,
        ..forearm
    }));
}

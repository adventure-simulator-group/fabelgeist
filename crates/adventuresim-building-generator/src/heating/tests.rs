use super::*;
use bevy::math::Vec3;
fn fixture(archetype: BuildingArchetype, seed: u64) -> BuildingPlan {
    generate(&BuildingProgram::fixture(archetype, seed)).unwrap()
}
#[test]
fn grounded_programme_preserves_roof_members_and_reaches_real_rooms() {
    for archetype in [
        BuildingArchetype::FachwerkCottage,
        BuildingArchetype::HallHouse,
    ] {
        for seed in [42, 47, 101, u64::MAX] {
            let heated = fixture(archetype, seed);
            let mut program = BuildingProgram::fixture(archetype, seed);
            program.domestic_heating = None;
            let unheated = generate(&program).unwrap();
            assert_eq!(
                serde_json::to_value(&heated.timber_frame).unwrap(),
                serde_json::to_value(&unheated.timber_frame).unwrap()
            );
            for member in &heated.timber_frame.as_ref().unwrap().members {
                let solid = |p: &BuildingPlan| {
                    serde_json::to_value(
                        p.resolved_geometry
                            .solids
                            .iter()
                            .find(|s| s.id == member.solid)
                            .unwrap(),
                    )
                    .unwrap()
                };
                assert_eq!(solid(&heated), solid(&unheated));
            }
            assert!(heated.domestic_heating.is_some());
            assert!(
                unheated
                    .resolved_geometry
                    .solids
                    .iter()
                    .all(|s| s.role != SolidRole::DomesticHeating)
            );
            assert!(audit_plan(&heated).is_empty());
        }
    }
}
#[test]
fn smoke_and_weather_mutations_fail_closed() {
    let original = fixture(BuildingArchetype::FachwerkCottage, 42);
    let mut missing_route = original.clone();
    missing_route
        .domestic_heating
        .as_mut()
        .unwrap()
        .passages
        .pop();
    assert!(
        audit_plan(&missing_route)
            .iter()
            .any(|i| i.code == "incomplete_domestic_smoke_route")
    );
    let mut blocked = original.clone();
    let h = blocked.domestic_heating.as_ref().unwrap();
    let bore_id = h
        .passages
        .iter()
        .find(|p| p.kind == HeatingPassageKind::FlueBore)
        .unwrap()
        .void;
    let bore = blocked
        .resolved_geometry
        .voids
        .iter()
        .find(|v| v.id == bore_id)
        .unwrap()
        .bounds;
    let mut plug = blocked
        .resolved_geometry
        .solids
        .iter()
        .find(|s| s.id == h.parts[0].solid)
        .unwrap()
        .clone();
    plug.id = ResolvedItemId(u64::MAX - 1);
    plug.centre = (bore.min + bore.max) * 0.5;
    plug.size = Vec3::new(0.1, 0.1, 0.1);
    blocked.resolved_geometry.solids.push(plug);
    assert!(
        audit_plan(&blocked)
            .iter()
            .any(|i| i.code == "blocked_domestic_smoke_route")
    );
    let mut roof_blocked = original.clone();
    let h = roof_blocked.domestic_heating.as_ref().unwrap();
    let face = roof_blocked
        .roof_assemblies
        .iter_mut()
        .flat_map(|r| &mut r.faces)
        .find(|f| f.id == h.roof.face)
        .unwrap();
    face.cutouts.remove(h.roof.cutout_index);
    assert!(
        audit_plan(&roof_blocked)
            .iter()
            .any(|i| i.code == "invalid_heating_roof_penetration")
    );
    let mut orphan = original.clone();
    orphan.domestic_heating = None;
    assert!(
        audit_plan(&orphan)
            .iter()
            .any(|i| i.code == "unowned_domestic_heating")
    );
}
#[test]
fn collision_and_both_lods_retain_the_canonical_stack() {
    let plan = fixture(BuildingArchetype::FachwerkCottage, 42);
    let collision = compile_building_collision(&plan);
    for part in &plan.domestic_heating.as_ref().unwrap().parts {
        assert!(collision.cuboids.iter().any(|c| c.source == part.solid));
        if !matches!(
            part.kind,
            HeatingPartKind::Flue
                | HeatingPartKind::RoofFlashing
                | HeatingPartKind::RoofUpstand
                | HeatingPartKind::RoofCounterFlashing
        ) {
            continue;
        }
        let solid = plan
            .resolved_geometry
            .solids
            .iter()
            .find(|s| s.id == part.solid)
            .unwrap();
        let detail = compile_solid_detail(&plan, solid);
        for level in [BuildingLodLevel::Facade, BuildingLodLevel::Shell] {
            let lod = compile_building_lod(&plan, level);
            for vertex in detail.meshes.iter().flat_map(|m| &m.vertices) {
                assert!(
                    lod.meshes
                        .iter()
                        .flat_map(|m| &m.vertices)
                        .any(|v| v.position.distance(vertex.position) < 0.001)
                );
            }
        }
    }
}

#[test]
fn folded_weathering_requires_every_return_and_downstream_lap() {
    let original = fixture(BuildingArchetype::FachwerkCottage, 42);
    for part in original
        .domestic_heating
        .as_ref()
        .unwrap()
        .parts
        .iter()
        .filter(|p| {
            matches!(
                p.kind,
                HeatingPartKind::RoofUpstand | HeatingPartKind::RoofCounterFlashing
            )
        })
    {
        let mut missing = original.clone();
        missing
            .resolved_geometry
            .solids
            .retain(|s| s.id != part.solid);
        missing
            .domestic_heating
            .as_mut()
            .unwrap()
            .parts
            .retain(|p| p.solid != part.solid);
        assert!(
            audit(&missing)
                .iter()
                .any(|i| i.code == "invalid_heating_roof_penetration")
        );
        let mut shifted = original.clone();
        shifted
            .resolved_geometry
            .solids
            .iter_mut()
            .find(|s| s.id == part.solid)
            .unwrap()
            .centre
            .y += 0.05;
        assert!(
            audit(&shifted)
                .iter()
                .any(|i| i.code == "invalid_heating_roof_penetration")
        );
    }
    let mut inverted = original.clone();
    let heating = inverted.domestic_heating.as_ref().unwrap();
    let face = inverted
        .roof_assemblies
        .iter()
        .flat_map(|r| &r.faces)
        .find(|f| f.id == heating.roof.face)
        .unwrap();
    let normal = face.plane.normal.normalize();
    for id in &heating.roof.flashing {
        let sheet = inverted
            .resolved_geometry
            .solids
            .iter_mut()
            .find(|s| s.id == *id)
            .unwrap();
        let distance = (face.plane.normal.dot(sheet.centre) + face.plane.constant)
            / face.plane.normal.length();
        sheet.centre -= normal * distance * 2.0;
    }
    assert!(
        audit(&inverted)
            .iter()
            .any(|i| i.code == "invalid_heating_roof_penetration")
    );
}
#[test]
fn roof_pitch_change_is_atomic_and_upper_kitchens_are_explicitly_unsupported() {
    let mut plan = fixture(BuildingArchetype::FachwerkCottage, 42);
    let id = plan.domestic_heating.as_ref().unwrap().roof.roof;
    let pitch = plan
        .roof_assemblies
        .iter()
        .find(|r| r.id == id)
        .unwrap()
        .faces[0]
        .pitch_degrees;
    let before = serde_json::to_value(&plan).unwrap();
    set_roof_pitch(&mut plan, id, pitch).unwrap();
    assert!(matches!(
        set_roof_pitch(&mut plan, id, pitch + 1.0),
        Err(RoofEditError::TopologyEvent)
    ));
    assert_eq!(before, serde_json::to_value(&plan).unwrap());
    let mut upper = BuildingProgram::fixture(BuildingArchetype::TownHouse, 42);
    upper.domestic_heating = Some(DomesticHeatingProgramme::HearthAndRearFedStove);
    assert!(matches!(
        generate(&upper),
        Err(GenerationError::InvalidDomesticHeating)
    ));
}

#[test]
fn missing_appliance_material_and_displaced_weathering_are_rejected() {
    let original = fixture(BuildingArchetype::FachwerkCottage, 42);
    for part in original
        .domestic_heating
        .as_ref()
        .unwrap()
        .parts
        .iter()
        .filter(|p| {
            matches!(
                p.kind,
                HeatingPartKind::TiledStove | HeatingPartKind::FireWall | HeatingPartKind::Hearth
            )
        })
    {
        let mut missing = original.clone();
        missing
            .domestic_heating
            .as_mut()
            .unwrap()
            .parts
            .retain(|p| p.solid != part.solid);
        missing
            .resolved_geometry
            .solids
            .retain(|s| s.id != part.solid);
        for wall in &mut missing.wall_assemblies {
            wall.host_solids.retain(|id| *id != part.solid);
        }
        assert!(
            audit(&missing).iter().any(|i| matches!(
                i.code,
                "open_domestic_appliance" | "incomplete_domestic_fire_wall"
            )),
            "missing {:?}",
            part.kind
        );
    }
    for displacement in [Vec3::X * 0.1, Vec3::Y * 0.12] {
        let mut shifted = original.clone();
        let id = shifted.domestic_heating.as_ref().unwrap().roof.flashing[0];
        shifted
            .resolved_geometry
            .solids
            .iter_mut()
            .find(|s| s.id == id)
            .unwrap()
            .centre += displacement;
        let issues = audit(&shifted);
        assert!(
            issues
                .iter()
                .any(|i| i.code == "invalid_heating_roof_penetration")
        );
        assert!(issues.iter().any(|i| i.code == "detached_heating_bearing"));
    }
    let mut enlarged = original.clone();
    let h = enlarged.domestic_heating.as_ref().unwrap();
    let cut = &mut enlarged
        .roof_assemblies
        .iter_mut()
        .flat_map(|r| &mut r.faces)
        .find(|f| f.id == h.roof.face)
        .unwrap()
        .cutouts[h.roof.cutout_index];
    cut[0].x -= 0.1;
    cut[1].x -= 0.1;
    assert!(
        audit(&enlarged)
            .iter()
            .any(|i| i.code == "invalid_heating_roof_penetration")
    );
}

#[test]
fn settlement_height_and_roof_variation_preserves_the_heating_core() {
    for archetype in [
        BuildingArchetype::FachwerkCottage,
        BuildingArchetype::HallHouse,
    ] {
        for seed in [0, 1, 2, 17, 42, 47, 101, u64::MAX] {
            let program = BuildingProgram::settlement(
                archetype,
                Some(adventuresim_world_schema::settlement_buildings::BuildingUse::Dwelling),
                seed,
            );
            let mut unheated = program.clone();
            unheated.domestic_heating = None;
            let baseline = generate(&unheated);
            let result = generate(&program);
            if baseline.is_err() {
                let (
                    Err(GenerationError::StructuralContract { issues: before, .. }),
                    Err(GenerationError::StructuralContract { issues: after, .. }),
                ) = (baseline, &result)
                else {
                    panic!("baseline failure changed: {result:?}");
                };
                assert_eq!(
                    before, *after,
                    "heating changed an already invalid recipe: {archetype:?}/{seed}"
                );
                continue;
            }
            assert!(result.is_ok(), "{archetype:?}/{seed}: {result:?}");
        }
    }
}

#[test]
fn rotated_masonry_and_tilted_flashing_do_not_count_as_sealed_material() {
    let original = fixture(BuildingArchetype::FachwerkCottage, 42);
    for kind in [HeatingPartKind::TiledStove, HeatingPartKind::RoofFlashing] {
        let mut changed = original.clone();
        let id = changed
            .domestic_heating
            .as_ref()
            .unwrap()
            .parts
            .iter()
            .find(|p| p.kind == kind)
            .unwrap()
            .solid;
        changed
            .resolved_geometry
            .solids
            .iter_mut()
            .find(|s| s.id == id)
            .unwrap()
            .crossfall_radians += 0.1;
        assert!(audit(&changed).iter().any(|i| matches!(
            i.code,
            "invalid_domestic_appliance_shape" | "invalid_heating_roof_penetration"
        )));
    }
}

#[test]
fn penetration_updates_drainage_stations_at_the_cut_boundary() {
    let mut plan = fixture(BuildingArchetype::FachwerkCottage, 42);
    let face = plan.domestic_heating.as_ref().unwrap().roof.face;
    let network = plan
        .resolved_geometry
        .roof_drainage_networks
        .iter_mut()
        .find(|n| n.face == face)
        .unwrap();
    let sample = network.samples[0];
    network.samples = [0.5, -0.001, -0.01]
        .map(|x| RoofDrainageSample {
            surface_point: Vec3::new(x, 0.0, 0.5),
            ..sample
        })
        .to_vec();
    let cut = [Vec3::ZERO, Vec3::X, Vec3::X + Vec3::Z, Vec3::Z];
    roof::exclude_cut_samples(&mut plan.resolved_geometry, face, &cut);
    let network = plan
        .resolved_geometry
        .roof_drainage_networks
        .iter()
        .find(|n| n.face == face)
        .unwrap();
    assert_eq!(network.samples.len(), 1);
    assert_eq!(network.samples[0].surface_point.x, -0.01);
}

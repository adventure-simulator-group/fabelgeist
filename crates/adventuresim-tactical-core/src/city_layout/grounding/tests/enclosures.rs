//! Actual stepped enclosure bearings and rejection preserve exact owner identity.
use super::*;
use crate::city_layout::PropertySide;
use crate::city_layout::grounding::{
    BoundarySupportConstraint, BoundarySupportElement, BoundarySupportMesh,
};
use avian3d::prelude::*;
use bevy::math::Quat;

fn project(
    fixture: &Fixture,
) -> (
    BoundarySupportMesh,
    SupportElevation,
    PropertyFoundationMesh,
) {
    let source = fixture.source();
    let plan = fixture.selected_plan(&source);
    let foundation = plan
        .foundations(&source, FoundationEmbedment::from_metres(0.2).unwrap())
        .unwrap();
    let crate::city_layout::grounding::BoundarySupportProjection {
        mesh,
        gate_elevation: gate,
    } = BoundarySupportMesh::project(
        &fixture.property,
        &foundation,
        SupportLimits::new(
            crate::city_layout::grounding::SupportGrade::from_ratio(0.65).unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(6.0)
                .unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.001)
                .unwrap(),
        ),
        FoundationEmbedment::from_metres(0.2).unwrap(),
    )
    .unwrap();
    assert!((gate.metres() - plan.gate_elevation().metres()).abs() < 0.001);
    (mesh, gate, foundation)
}

#[test]
fn goslar_1238_enclosure_follows_all_terraces_and_anchors_the_gate_above_the_front_floor() {
    let fixture = Fixture::load();
    let (mesh, gate, foundation) = project(&fixture);
    let source = fixture.source();
    let plan = fixture.selected_plan(&source);
    assert!(gate.metres() - plan.member_support()[0].elevation.metres() > 2.3);
    let post_head = gate.metres() + fixture.property.boundary.gate.height_metres + 0.15;
    let mut post_soil = Vec::new();
    let collider = mesh.collider().unwrap().into_solid().unwrap();
    for cell in &mesh.cells {
        let centre = cell.native_positions()[..3].iter().copied().sum::<Vec3>() / 3.0;
        assert!(
            collider
                .cast_ray(
                    Vec3::Y * gate.metres(),
                    Rotation::default(),
                    centre + Vec3::Y * (gate.metres() + 0.01),
                    -Vec3::Y,
                    0.02,
                    false
                )
                .is_some()
        );
        if cell.element() == BoundarySupportElement::GatePost(PropertySide::Left) {
            for top in &cell.native_positions()[..3] {
                assert!((top.y + gate.metres() - post_head).abs() < 0.001);
            }
            post_soil.extend(
                cell.native_positions()[3..]
                    .iter()
                    .map(|p| p.y + gate.metres() + 0.2),
            );
        }
        for point in &cell.native_positions()[3..] {
            let world = *point + Vec3::Y * gate.metres();
            if matches!(cell.element(), BoundarySupportElement::WallCap(_)) {
                continue;
            }
            let support = foundation
                .support_triangles
                .iter()
                .filter_map(|indices| {
                    let triangle = super::super::foundations::GroundTriangle::new(
                        indices.map(|i| foundation.positions[i as usize]),
                    )?;
                    triangle
                        .contains(world.xz(), 0.001)
                        .then(|| triangle.height_at(world.xz()))
                })
                .any(|h| (world.y + 0.2 - h).abs() < 0.001);
            assert!(
                support,
                "unbound masonry bearing {:?} at {world:?}",
                cell.element()
            );
        }
    }
    let low = post_soil.iter().copied().fold(f32::INFINITY, f32::min);
    let high = post_soil.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    assert!(
        high - low > 2.3,
        "the complete hinge-post footprint must straddle both levels"
    );
}

#[test]
fn enclosure_projection_rejects_missing_wall_bearings_with_exact_property_and_member_ids() {
    let fixture = Fixture::load();
    let (_, _, mut foundation) = project(&fixture);
    foundation.support_triangles.retain(|indices| {
        indices.iter().any(|i| {
            fixture
                .property
                .boundary
                .gate
                .centre_metres
                .distance(foundation.positions[*i as usize].xz())
                < 2.0
        })
    });
    let error = BoundarySupportMesh::project(
        &fixture.property,
        &foundation,
        SupportLimits::new(
            crate::city_layout::grounding::SupportGrade::from_ratio(0.65).unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(6.0)
                .unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.001)
                .unwrap(),
        ),
        FoundationEmbedment::from_metres(0.2).unwrap(),
    )
    .unwrap_err();
    assert_eq!(error.property_id, CityPropertyId(1238));
    assert_eq!(
        error.member_building_ids,
        [1238, 17622].map(crate::scene_input::SceneBuildingId)
    );
    assert_eq!(error.constraint, BoundarySupportConstraint::Coverage);
    assert!(matches!(error.element, BoundarySupportElement::Wall(_)));
    assert!(error.violation.discrepancy_value() > 0.0);
    assert!(error.location_metres.attempted_metres().is_finite());
}

#[test]
fn enclosure_projection_ignores_other_property_support_and_is_repeatable() {
    let fixture = Fixture::load_965();
    let (mesh, gate, mut foundation) = project(&fixture);
    let repeat = BoundarySupportMesh::project(
        &fixture.property,
        &foundation,
        SupportLimits::new(
            crate::city_layout::grounding::SupportGrade::from_ratio(0.65).unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(6.0)
                .unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.001)
                .unwrap(),
        ),
        FoundationEmbedment::from_metres(0.2).unwrap(),
    )
    .unwrap();
    assert_eq!(
        repeat,
        crate::city_layout::grounding::BoundarySupportProjection {
            mesh,
            gate_elevation: gate
        }
    );
    foundation.support_triangles.reverse();
    let shuffled = BoundarySupportMesh::project(
        &fixture.property,
        &foundation,
        SupportLimits::new(
            crate::city_layout::grounding::SupportGrade::from_ratio(0.65).unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(6.0)
                .unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.001)
                .unwrap(),
        ),
        FoundationEmbedment::from_metres(0.2).unwrap(),
    )
    .unwrap();
    assert_eq!(shuffled, repeat);
    foundation.property_id = CityPropertyId(1238);
    let error = BoundarySupportMesh::project(
        &fixture.property,
        &foundation,
        SupportLimits::new(
            crate::city_layout::grounding::SupportGrade::from_ratio(0.65).unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(6.0)
                .unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.001)
                .unwrap(),
        ),
        FoundationEmbedment::from_metres(0.2).unwrap(),
    )
    .unwrap_err();
    assert_eq!(error.constraint, BoundarySupportConstraint::OwnerBinding);
    assert_eq!(error.property_id, CityPropertyId(965));
}

#[test]
fn decoded_enclosures_reject_foreign_owner_roles_elements_and_gate_datums() {
    use crate::scene_input::{GeneratedBoundary, SceneBoundary};
    let fixture = Fixture::load();
    let (mesh, gate, _) = project(&fixture);
    let scene = serde_json::json!({
        "property_id": fixture.property.id,
        "front_building_id": fixture.property.front_building_id,
        "boundary": fixture.property.boundary,
        "fixed_support": mesh,
    });
    let canonical: SceneBoundary = serde_json::from_value(scene.clone()).unwrap();
    assert_eq!(
        postcard::from_bytes::<SceneBoundary>(&postcard::to_allocvec(&canonical).unwrap()).unwrap(),
        canonical
    );
    let mut wrong_front = scene.clone();
    wrong_front["front_building_id"] = serde_json::json!(fixture.property.rear_building_id);
    assert!(serde_json::from_value::<SceneBoundary>(wrong_front).is_err());
    let mut wrong_owner = scene.clone();
    wrong_owner["property_id"] = serde_json::json!(fixture.property.id.0 + 1);
    assert!(serde_json::from_value::<SceneBoundary>(wrong_owner).is_err());
    let mut wrong_elements = scene.clone();
    let original = wrong_elements["fixed_support"]["binding"]["elements"][0].clone();
    let foreign = serde_json::to_value(BoundarySupportElement::Wall(9999)).unwrap();
    wrong_elements["fixed_support"]["binding"]["elements"][0] = foreign.clone();
    for cell in wrong_elements["fixed_support"]["cells"]
        .as_array_mut()
        .unwrap()
    {
        if cell["element"] == original {
            cell["element"] = foreign.clone();
        }
    }
    assert!(serde_json::from_value::<SceneBoundary>(wrong_elements).is_err());
    let valid_generated = serde_json::json!({"scene": scene, "elevation_metres": gate});
    let generated: GeneratedBoundary = serde_json::from_value(valid_generated.clone()).unwrap();
    let roundtrip: GeneratedBoundary =
        postcard::from_bytes(&postcard::to_allocvec(&generated).unwrap()).unwrap();
    assert_eq!(roundtrip.scene(), generated.scene());
    let mut wrong_datum = valid_generated;
    wrong_datum["elevation_metres"] = serde_json::json!(gate.metres() + 1.0);
    assert!(serde_json::from_value::<GeneratedBoundary>(wrong_datum).is_err());
    #[derive(serde::Serialize)]
    struct BoundaryWire<'a> {
        scene: &'a SceneBoundary,
        elevation_metres: SupportElevation,
    }
    let wrong_datum = BoundaryWire {
        scene: &canonical,
        elevation_metres: SupportElevation::from_metres(gate.metres() + 1.0).unwrap(),
    };
    assert!(
        postcard::from_bytes::<GeneratedBoundary>(&postcard::to_allocvec(&wrong_datum).unwrap())
            .is_err()
    );
}

#[test]
fn boundary_physics_retains_both_windings_thin_prisms_and_contact_cells() {
    let (original, _, _) = project(&Fixture::load());
    for width in [1.0_f32, 0.000_000_01, 0.0] {
        for reversed in [false, true] {
            let mut points = [
                Vec3::new(0.0, -2.0, 0.0),
                Vec3::new(width, -2.0, 0.0),
                Vec3::new(0.0, -2.0, width),
                Vec3::new(0.0, -2.2, 0.0),
                Vec3::new(width, -2.2, 0.0),
                Vec3::new(0.0, -2.2, width),
            ];
            if reversed {
                points.swap(1, 2);
                points.swap(4, 5);
            }
            let mut wire = serde_json::to_value(&original).unwrap();
            for cell in wire["cells"].as_array_mut().unwrap() {
                cell["positions_metres"] = serde_json::to_value(points).unwrap();
            }
            let mesh: BoundarySupportMesh = serde_json::from_value(wire).unwrap();
            let collision = mesh.collider().unwrap();
            if width == 0.0 {
                assert!(matches!(collision, SupportCollision::ContactOnly));
            } else {
                let collider = collision.into_solid().unwrap();
                assert!(collider.contains_point(
                    Vec3::ZERO,
                    Quat::IDENTITY,
                    Vec3::new(width * 0.2, -2.1, width * 0.2)
                ));
                let hit = collider
                    .cast_ray(
                        Vec3::ZERO,
                        Quat::IDENTITY,
                        Vec3::new(width * 0.2, -1.0, width * 0.2),
                        Vec3::NEG_Y,
                        2.0,
                        false,
                    )
                    .unwrap();
                assert!((hit.0 - 1.0).abs() < 0.00001);
                assert!(hit.1.y > 0.99);
            }
        }
    }
}

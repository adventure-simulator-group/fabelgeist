//! Actual stepped enclosure bearings and rejection preserve exact owner identity.
use super::*;
use crate::city_layout::PropertySide;
use crate::city_layout::grounding::{
    BoundarySupportConstraint, BoundarySupportElement, BoundarySupportMesh,
};
use avian3d::prelude::*;

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
    let (mesh, gate) = BoundarySupportMesh::project(
        &fixture.property,
        &foundation,
        SupportLimits::new(0.65, 6.0, 0.001).unwrap(),
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
    let collider = mesh.collider();
    for cell in &mesh.cells {
        let centre = cell.positions_metres[..3].iter().copied().sum::<Vec3>() / 3.0;
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
        if cell.element == BoundarySupportElement::GatePost(PropertySide::Left) {
            for top in &cell.positions_metres[..3] {
                assert!((top.y + gate.metres() - post_head).abs() < 0.001);
            }
            post_soil.extend(
                cell.positions_metres[3..]
                    .iter()
                    .map(|p| p.y + gate.metres() + 0.2),
            );
        }
        for point in &cell.positions_metres[3..] {
            let world = *point + Vec3::Y * gate.metres();
            if matches!(cell.element, BoundarySupportElement::WallCap(_)) {
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
                cell.element
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
        SupportLimits::new(0.65, 6.0, 0.001).unwrap(),
        FoundationEmbedment::from_metres(0.2).unwrap(),
    )
    .unwrap_err();
    assert_eq!(error.property_id, CityPropertyId(1238));
    assert_eq!(error.member_building_ids, [1238, 17622]);
    assert_eq!(error.constraint, BoundarySupportConstraint::Coverage);
    assert!(matches!(error.element, BoundarySupportElement::Wall(_)));
    assert!(error.shortfall > 0.0);
    assert!(error.location_metres.is_finite());
}

#[test]
fn enclosure_projection_ignores_other_property_support_and_is_repeatable() {
    let fixture = Fixture::load_965();
    let (mesh, gate, mut foundation) = project(&fixture);
    let repeat = BoundarySupportMesh::project(
        &fixture.property,
        &foundation,
        SupportLimits::new(0.65, 6.0, 0.001).unwrap(),
        FoundationEmbedment::from_metres(0.2).unwrap(),
    )
    .unwrap();
    assert_eq!(repeat, (mesh, gate));
    foundation.support_triangles.reverse();
    let shuffled = BoundarySupportMesh::project(
        &fixture.property,
        &foundation,
        SupportLimits::new(0.65, 6.0, 0.001).unwrap(),
        FoundationEmbedment::from_metres(0.2).unwrap(),
    )
    .unwrap();
    assert_eq!(shuffled, repeat);
    foundation.property_id = CityPropertyId(1238);
    let error = BoundarySupportMesh::project(
        &fixture.property,
        &foundation,
        SupportLimits::new(0.65, 6.0, 0.001).unwrap(),
        FoundationEmbedment::from_metres(0.2).unwrap(),
    )
    .unwrap_err();
    assert_eq!(error.constraint, BoundarySupportConstraint::OwnerBinding);
    assert_eq!(error.property_id, CityPropertyId(965));
}

use super::*;
use bevy::math::{Vec2, Vec3};
use serde_json::Value;

pub(super) fn fixture() -> (CitySceneLayout, GeographicSurface, CompoundGradingPolicy) {
    let value: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/tactical-grounding/goslar-1238.json"
    )))
    .unwrap();
    let front: DistantBuildingPlacement =
        serde_json::from_value(value["front_distant_placement"].clone()).unwrap();
    let rear = serde_json::from_value(value["rear_distant_placement"].clone()).unwrap();
    let layout = CitySceneLayout {
        playable: vec![front.into()],
        distant: vec![rear],
        compounds: vec![serde_json::from_value(value["compound"].clone()).unwrap()],
        ..Default::default()
    };
    let source = GeographicSurface::from_triangles(
        serde_json::from_value::<Vec<[Vec3; 3]>>(value["geographic_triangles"].clone()).unwrap(),
    )
    .unwrap();
    let policy = CompoundGradingPolicy {
        limits: SupportLimits::new(0.65, 6.0, 0.001).unwrap(),
        stairs: CourtStairLimits::new(0.19, 0.25, 1.0, 1.05, 0.5).unwrap(),
        embedment: FoundationEmbedment::from_metres(0.2).unwrap(),
        street_apron: StreetApronDimensions::from_metres(Vec2::new(1.0, 2.0)).unwrap(),
    };
    (layout, source, policy)
}

fn placements(layout: &CitySceneLayout) -> BTreeMap<u64, TacticalBuildingPlacement> {
    layout
        .playable
        .iter()
        .cloned()
        .chain(
            layout
                .distant
                .iter()
                .copied()
                .map(TacticalBuildingPlacement::from),
        )
        .map(|p| (p.id, p))
        .collect()
}

#[test]
fn accepted_support_seats_both_detail_levels_without_changing_physical_identity() {
    let (layout, source, policy) = fixture();
    let original = layout.clone();
    let grounded = SelectedCityGrounding::select(&layout, &source, policy)
        .unwrap()
        .compile()
        .unwrap();
    assert_eq!(layout, original);
    assert_eq!(grounded.layout().compounds, layout.compounds);
    assert_eq!(
        grounded.layout().single_properties,
        layout.single_properties
    );
    assert_eq!(grounded.layout().gardens, layout.gardens);
    assert_eq!(grounded.layout().businesses, layout.businesses);
    assert_eq!(grounded.layout().streets, layout.streets);
    assert_eq!(grounded.layout().yards, layout.yards);
    let before = placements(&layout);
    let after = placements(grounded.layout());
    assert_eq!(before.len(), after.len());
    for (id, p) in after {
        let mut expected = before[&id].clone();
        expected.base_elevation_metres = p.base_elevation_metres;
        assert_eq!(
            p, expected,
            "programme, horizontal transform or identity changed"
        );
        let expected_floor = if id == 1238 { 21.042906 } else { 19.056694 };
        assert!((p.base_elevation_metres - expected_floor).abs() < 0.001);
    }
    assert_eq!(
        grounded.terrain().foundations[0].member_building_ids,
        [1238, 17622]
    );
    for member in layout.plan_compound_support(&source, policy).unwrap()[0].member_support() {
        assert!(
            grounded
                .terrain()
                .elevations_at(member.contact.centre_metres)
                .iter()
                .any(|h| (h.metres() - member.elevation.metres()).abs()
                    < policy.limits.contact_tolerance_metres())
        );
    }
}

#[test]
fn promotion_and_input_order_preserve_the_accepted_floors_and_terrain() {
    let (mut layout, source, policy) = fixture();
    let first = SelectedCityGrounding::select(&layout, &source, policy)
        .unwrap()
        .compile()
        .unwrap();
    layout.playable.extend(
        layout
            .distant
            .drain(..)
            .map(TacticalBuildingPlacement::from),
    );
    layout.playable.reverse();
    let mut triangles: Vec<_> = source.triangles().collect();
    triangles.reverse();
    let other_source = GeographicSurface::from_triangles(triangles).unwrap();
    let second = SelectedCityGrounding::select(&layout, &other_source, policy)
        .unwrap()
        .compile()
        .unwrap();
    assert_eq!(placements(first.layout()), placements(second.layout()));
    assert_eq!(first.terrain(), second.terrain());
}

#[test]
fn missing_or_duplicate_physical_bindings_are_rejected_before_floor_selection() {
    let (mut layout, source, policy) = fixture();
    layout.compounds.clear();
    let original = layout.clone();
    assert!(matches!(
        SelectedCityGrounding::select(&layout, &source, policy),
        Err(CityGroundingError::UnboundBuilding { building: 1238 })
    ));
    assert_eq!(layout, original);
    layout.playable.push(layout.playable[0].clone());
    assert!(matches!(
        SelectedCityGrounding::select(&layout, &source, policy),
        Err(CityGroundingError::DuplicateBuilding { building: 1238 })
    ));
}

#[test]
fn ownership_rejection_publishes_no_partial_floor_projection() {
    let (mut layout, source, policy) = fixture();
    let mut other = layout.compounds[0].clone();
    other.id = CityPropertyId(1239);
    other.front_building_id = 1239;
    other.rear_building_id = 17623;
    layout.compounds.push(other);
    let mut front = layout.playable[0].clone();
    front.id = 1239;
    layout.playable.push(front);
    let mut rear = layout.distant[0];
    rear.id = 17623;
    layout.distant.push(rear);
    let original = layout.clone();
    let selected = SelectedCityGrounding::select(&layout, &source, policy).unwrap();
    assert!(matches!(
        selected.compile(),
        Err(SettlementSupportError::OwnershipOverlap {
            first: CityPropertyId(1238),
            second: CityPropertyId(1239),
            ..
        })
    ));
    assert_eq!(layout, original);
}

#[test]
fn duplicate_ownership_diagnostics_are_independent_of_property_iteration_order() {
    let (mut layout, source, policy) = fixture();
    let mut other = layout.compounds[0].clone();
    other.id = CityPropertyId(1239);
    layout.compounds.push(other);
    for _ in 0..2 {
        assert!(matches!(
            SelectedCityGrounding::select(&layout, &source, policy),
            Err(CityGroundingError::Terrain(
                SettlementSupportError::DuplicateMember {
                    building: 1238,
                    first: CityPropertyId(1238),
                    second: CityPropertyId(1239)
                }
            ))
        ));
        layout.compounds.reverse();
    }
}

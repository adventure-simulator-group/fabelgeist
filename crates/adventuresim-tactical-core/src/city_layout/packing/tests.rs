use super::*;
use intervals::PackingClearance;

fn bounds(centre: Vec2, dimensions: Vec2, yaw: f32) -> CityPlotBounds {
    CityPlotBounds {
        centre_metres: centre,
        dimensions_metres: dimensions,
        orientation: BuildingOrientation::from_radians(yaw).unwrap(),
    }
}

#[test]
fn rotated_translation_intervals_reject_real_overlap_and_retain_the_clear_slot() {
    let orientation = BuildingOrientation::from_radians(0.37).unwrap();
    let tangent = orientation.local_to_world(Vec2::X);
    let first = bounds(Vec2::ZERO, Vec2::new(9.216, 15.216), 0.37);
    let second = bounds(tangent * 9.01, first.dimensions_metres, 0.37);
    assert!(first.intersects(second));
    let forbidden = FrontageInterval::overlap_displacements(
        first,
        second,
        tangent,
        PackingClearance::BuildingBody,
        DVec2::ZERO,
    )
    .unwrap();
    let available = FrontageInterval {
        minimum_metres: -1.0,
        maximum_metres: 0.0,
    }
    .without(forbidden);
    let correction = available
        .iter()
        .map(FrontageInterval::nearest_origin)
        .min_by(|a, b| a.abs().total_cmp(&b.abs()))
        .unwrap();
    assert!(
        !CityPlotBounds {
            centre_metres: first.centre_metres + tangent * correction as f32,
            ..first
        }
        .intersects(second)
    );
    assert!((correction + 0.327).abs() < CityPlotBounds::COORDINATE_TOLERANCE_METRES);
}

#[test]
fn empty_corners_of_combined_roof_and_ground_bounds_do_not_claim_occupancy() {
    use context::ParcelGeometry;
    let first = ParcelGeometry {
        bearings: Vec::new(),
        garden: None,
        reservation: bounds(Vec2::new(1.0, 3.0), Vec2::new(4.0, 8.0), 0.0),
        buildings: vec![bounds(Vec2::ZERO, Vec2::splat(4.0), 0.0)],
    };
    let second = ParcelGeometry {
        bearings: Vec::new(),
        garden: None,
        reservation: bounds(Vec2::new(-2.25, 5.0), Vec2::new(2.0, 3.0), 0.0),
        buildings: vec![bounds(Vec2::new(-2.25, 5.0), Vec2::splat(1.0), 0.0)],
    };
    assert!(bounds(Vec2::new(0.5, 2.5), Vec2::new(5.0, 9.0), 0.0).intersects(second.reservation));
    assert!(
        first
            .forbidden_displacements(&second, Dir2::X, DVec2::ZERO)
            .iter()
            .all(|range| !range.intersects(FrontageInterval {
                minimum_metres: -0.001,
                maximum_metres: 0.001
            }))
    );
}

#[test]
fn translated_rotated_child_edges_remain_contained_but_real_excess_is_rejected() {
    for centre in [
        Vec2::new(-231.813_7, -122.429_76),
        Vec2::new(1500.0, -1900.0),
    ] {
        for yaw in [2.944_197_2, 0.37, -1.7] {
            let mut parent = bounds(centre, Vec2::new(12.5, 18.0), yaw);
            let mut child = bounds(
                centre + parent.orientation.local_to_world(Vec2::new(-1.0, 6.0)),
                Vec2::new(10.5, 6.0),
                yaw,
            );
            for delta in [Vec2::ZERO, Vec2::new(-0.503_295_9, -0.100_658_54)] {
                parent.centre_metres += delta;
                child.centre_metres += delta;
                assert!(
                    child
                        .corners()
                        .into_iter()
                        .all(|point| parent.contains(point))
                );
                let outside =
                    parent.centre_metres + parent.orientation.local_to_world(Vec2::new(0.0, 9.002));
                assert!(
                    !parent.contains(outside),
                    "the coordinate bound must not permit a real boundary violation"
                );
            }
        }
    }
}

pub(super) fn record_garden_failure(
    original: &[gardens::CityGarden],
    translated: &CompiledCityLayout,
    translations: &BTreeMap<CityPropertyId, Vec2>,
    validation: &Result<(), CityCompileError>,
) {
    let Err(CityCompileError::Packing {
        property,
        issue: CityPackingIssue::Garden { .. },
    }) = validation
    else {
        return;
    };
    let Some(path) = std::env::var_os("FABELGEIST_SUPPORT_DIAGNOSTIC_DIR") else {
        return;
    };
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path);
    std::fs::create_dir_all(&path).unwrap();
    std::fs::write(
        path.join(format!("packing-garden-{}.json", property.0)),
        serde_json::to_vec_pretty(&serde_json::json!({
            "original":original.iter().find(|g|g.owner==*property),
            "translated":translated.gardens.iter().find(|g|g.owner==*property),
            "translation":translations[property], "error":format!("{validation:?}"),
        }))
        .unwrap(),
    )
    .unwrap();
}

#[test]
fn measured_roof_projection_cannot_block_garden_working_or_planting_space() {
    use context::ParcelGeometry;
    use gardens::{
        CityGarden, GardenPlantId, GardenPlantPlacement, GardenPlantScale, GardenSpecimen,
    };
    let orientation = BuildingOrientation::from_radians(0.0).unwrap();
    let plant = GardenPlantPlacement {
        id: GardenPlantId(7),
        specimen: GardenSpecimen::CommonHazel,
        centre_metres: Vec2::new(1.0, 4.0),
        orientation,
        scale: GardenPlantScale::new(0.4),
    };
    let garden = CityGarden {
        owner: CityPropertyId(1),
        front_building_id: 1,
        plot: bounds(Vec2::new(0.0, 3.0), Vec2::new(4.0, 8.0), 0.0),
        cultivated_bounds: bounds(Vec2::new(0.0, 4.0), Vec2::new(4.0, 5.0), 0.0),
        beds: vec![bounds(Vec2::new(1.0, 2.0), Vec2::splat(0.5), 0.0)],
        access: vec![CityAccessSegment {
            start_metres: Vec2::new(-1.0, 0.0),
            end_metres: Vec2::new(-1.0, 4.0),
            half_width_metres: 0.2,
        }],
        plants: vec![plant],
    };
    let first = ParcelGeometry {
        reservation: garden.plot,
        buildings: vec![bounds(Vec2::ZERO, Vec2::splat(1.0), 0.0)],
        bearings: Vec::new(),
        garden: Some(garden.clone()),
    };
    let extreme = plant
        .world_hull()
        .into_iter()
        .max_by(|a, b| a.x.total_cmp(&b.x))
        .unwrap();
    for centre in [
        Vec2::new(1.0, 2.0),
        Vec2::new(-1.0, 3.0),
        extreme + Vec2::new(0.02, 0.0),
    ] {
        let body = bounds(centre, Vec2::splat(0.01), 0.0);
        assert!(!garden.clears_building(body));
        let other = ParcelGeometry {
            reservation: bounds(Vec2::new(5.0, 3.0), Vec2::splat(1.0), 0.0),
            buildings: vec![body],
            bearings: Vec::new(),
            garden: None,
        };
        let blocked = |a: &ParcelGeometry, b: &ParcelGeometry| !a.clears(b);
        assert!(
            blocked(&first, &other),
            "the solver must protect the accepted garden"
        );
        assert!(
            blocked(&other, &first),
            "pair ordering must preserve garden protection"
        );
        let mut empty = first.clone();
        empty.garden = None;
        assert!(
            !blocked(&empty, &other),
            "an elevated envelope may overhang unoccupied ground"
        );
    }
}

#[test]
fn property_translation_keeps_garden_street_hook_and_member_offsets() {
    use gardens::CityGarden;
    for yaw in [0.0, 0.37, -1.7] {
        let plot = bounds(Vec2::new(50.0, -30.0), Vec2::new(12.0, 20.0), yaw);
        let tangent = plot.orientation.local_to_world(Vec2::X);
        let point = |local| plot.centre_metres + plot.orientation.local_to_world(local);
        let mut garden = CityGarden {
            owner: CityPropertyId(4),
            front_building_id: 8,
            plot,
            cultivated_bounds: bounds(point(Vec2::new(0.0, 4.0)), Vec2::new(8.0, 8.0), yaw),
            beds: vec![bounds(point(Vec2::new(2.0, 4.0)), Vec2::splat(2.0), yaw)],
            access: vec![
                CityAccessSegment {
                    start_metres: point(Vec2::new(-3.0, -12.0)),
                    end_metres: point(Vec2::new(-3.0, 0.0)),
                    half_width_metres: 0.4,
                },
                CityAccessSegment {
                    start_metres: point(Vec2::new(-3.0, 0.0)),
                    end_metres: point(Vec2::new(-3.0, 7.0)),
                    half_width_metres: 0.4,
                },
            ],
            plants: Vec::new(),
        };
        let original = garden.clone();
        let delta = tangent * 1.2 - tangent.perp() * 2.0;
        garden.translate(delta, tangent);
        assert_eq!(garden.owner, original.owner);
        assert_eq!(garden.front_building_id, original.front_building_id);
        assert_eq!(
            garden.plot.dimensions_metres,
            original.plot.dimensions_metres
        );
        assert_eq!(
            garden.beds[0].centre_metres,
            original.beds[0].centre_metres + delta
        );
        assert_eq!(garden.access[0].end_metres, garden.access[1].start_metres);
        let hook_delta = garden.access[0].start_metres - original.access[0].start_metres;
        assert!(
            hook_delta.dot(tangent.perp()).abs()
                < CityPlotBounds::COORDINATE_TOLERANCE_METRES as f32
        );
        assert!(
            (hook_delta.dot(tangent) - 1.2).abs()
                < CityPlotBounds::COORDINATE_TOLERANCE_METRES as f32
        );
        assert!(garden.clearance_geometry().is_ok());
    }
}

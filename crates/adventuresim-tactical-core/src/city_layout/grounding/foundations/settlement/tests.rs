use super::*;
use crate::city_layout::grounding::tests::Fixture;

fn fixtures() -> (Vec<CompoundSupportPlan>, GeographicSurface) {
    let fixtures = [Fixture::load_965(), Fixture::load()];
    let sources: Vec<_> = fixtures.iter().map(Fixture::source).collect();
    let plans = fixtures
        .iter()
        .zip(&sources)
        .map(|(fixture, source)| fixture.selected_plan(source))
        .collect();
    let source =
        GeographicSurface::from_triangles(sources.iter().flat_map(|s| s.triangles())).unwrap();
    (plans, source)
}

#[test]
fn goslar_properties_share_one_clipped_source_and_iteration_independent_support() {
    let (mut plans, source) = fixtures();
    let embedment = FoundationEmbedment::from_metres(0.2).unwrap();
    let forward = BoundedSettlementTerrain::compile(
        &plans
            .iter()
            .map(CompoundSupportPlan::support_surface)
            .collect::<Vec<_>>(),
        &source,
        embedment,
    )
    .unwrap();
    plans.reverse();
    let reverse = BoundedSettlementTerrain::compile(
        &plans
            .iter()
            .map(CompoundSupportPlan::support_surface)
            .collect::<Vec<_>>(),
        &source,
        embedment,
    )
    .unwrap();
    assert_eq!(forward, reverse);
    assert_eq!(forward.foundations.len(), 2);
    assert_eq!(forward.colliders().len(), 3);
    for plan in plans {
        for member in plan.member_support() {
            let point = member.contact.centre_metres;
            let heights: Vec<_> = forward.elevations_at(point).iter().collect();
            assert_eq!(
                heights.len(),
                1,
                "member {}: {heights:?}",
                member.building_id
            );
            assert!((heights[0].metres() - member.elevation.metres()).abs() < 0.001);
            let hit = forward.highest_surface_at(point).unwrap();
            let normal = *hit.normal;
            assert!((normal - Vec3::Y).length() < 0.001);
        }
    }
    // A complete source triangle far from either declared support region is
    // retained bit for bit, without LOD-spacing-dependent grading padding.
    let untouched = source
        .triangles()
        .find(|triangle| triangle.iter().all(|p| p.z == -400.0 || p.x == -100.0))
        .unwrap();
    assert!(forward.natural_triangles.contains(&untouched));
}

#[test]
fn repeated_property_and_member_authority_are_rejected_before_clipping() {
    let (plans, source) = fixtures();
    let embedment = FoundationEmbedment::from_metres(0.2).unwrap();
    let repeated = [plans[0].clone(), plans[0].clone()];
    assert!(matches!(
        BoundedSettlementTerrain::compile(&repeated.iter().map(CompoundSupportPlan::support_surface).collect::<Vec<_>>(), &source, embedment),
        Err(SettlementSupportError::DuplicateProperty { property }) if property == plans[0].property_id()
    ));
    let mut other = plans[1].clone();
    other.levels.front.building_id = plans[0].levels.front.building_id;
    other.property.front_building_id = plans[0].property.front_building_id;
    assert!(matches!(
        BoundedSettlementTerrain::compile(&[plans[0].clone(), other].iter().map(CompoundSupportPlan::support_surface).collect::<Vec<_>>(), &source, embedment),
        Err(SettlementSupportError::DuplicateMember { building, first, second })
            if building == plans[0].levels.front.building_id
                && first == plans[0].property_id() && second == plans[1].property_id()
    ));
}

#[test]
fn nearby_properties_are_not_merged_and_overlap_names_both_owners() {
    let fixture = Fixture::load_965();
    let source = fixture.source();
    let plan = fixture.selected_plan(&source);
    let mut overlapping = plan.clone();
    overlapping.property.id = CityPropertyId(966);
    overlapping.levels.front.building_id = 966;
    overlapping.levels.rear.building_id = 17350;
    overlapping.property.front_building_id = 966;
    overlapping.property.rear_building_id = 17350;
    let error = BoundedSettlementTerrain::compile(
        &[plan.clone(), overlapping]
            .iter()
            .map(CompoundSupportPlan::support_surface)
            .collect::<Vec<_>>(),
        &source,
        FoundationEmbedment::from_metres(0.2).unwrap(),
    )
    .unwrap_err();
    assert!(matches!(error,
        SettlementSupportError::OwnershipOverlap {
            first: CityPropertyId(965), second: CityPropertyId(966),
            first_members, second_members,
            area_square_metres, ..
        } if first_members == [965, 17349] && second_members == [966, 17350] && area_square_metres > 500.0
    ));
    let touching = CityPlotBounds {
        centre_metres: plan.reservation().centre_metres
            + plan.reservation().orientation.local_to_world(Vec2::X)
                * plan.reservation().dimensions_metres.x,
        ..plan.reservation()
    };
    // Rectangle rotation is represented in f32; use an axis-aligned exact
    // edge control to distinguish a shared boundary from overlapping area.
    let first = CityPlotBounds {
        centre_metres: Vec2::ZERO,
        dimensions_metres: Vec2::splat(2.0),
        orientation: crate::scene_input::BuildingOrientation::IDENTITY,
    };
    assert_eq!(
        overlap(
            first,
            CityPlotBounds {
                centre_metres: Vec2::X * 2.0,
                ..first
            },
            &first.corners().map(Vec2::as_dvec2),
            &CityPlotBounds {
                centre_metres: Vec2::X * 2.0,
                ..first
            }
            .corners()
            .map(Vec2::as_dvec2)
        ),
        None
    );
    assert!(!plan.reservation().contains(touching.centre_metres));
}

#[test]
fn retaining_edge_queries_keep_both_bound_levels_without_averaging() {
    let fixture = Fixture::load_965();
    let source = fixture.source();
    let plan = fixture.selected_plan(&source);
    let terrain = BoundedSettlementTerrain::compile(
        &(std::slice::from_ref(&plan))
            .iter()
            .map(CompoundSupportPlan::support_surface)
            .collect::<Vec<_>>(),
        &source,
        FoundationEmbedment::from_metres(0.2).unwrap(),
    )
    .unwrap();
    let local = Vec2::new(plan.split_frontage_metres, -5.0);
    let edge =
        plan.reservation().centre_metres + plan.reservation().orientation.local_to_world(local);
    let heights: Vec<_> = terrain
        .elevations_at(edge)
        .iter()
        .map(SupportElevation::metres)
        .collect();
    assert!(heights.len() >= 2, "retaining edge {edge:?}: {heights:?}");
    assert!(heights.last().unwrap() - heights[0] > 0.1);
    let selected = terrain
        .surface_below(Vec3::new(edge.x, heights[0], edge.y))
        .unwrap();
    assert!((selected.elevation.metres() - heights[0]).abs() < 0.001);
    assert!(
        (terrain.highest_surface_at(edge).unwrap().elevation.metres() - heights.last().unwrap())
            .abs()
            < 0.001
    );
}

#[test]
fn runtime_terrain_retains_the_accepted_source_diagonal_and_roundtrips_owned_geometry() {
    use crate::scene::SceneTerrain;
    let fixture = Fixture::load_965();
    let source = fixture.source();
    // Embed this recorded 100 m source window in an authored flat exterior.
    // The metadata grid deliberately has a different subdivision diagonal.
    // Installation must retain the already accepted source/support topology.
    let side = 41;
    let spacing = 50.0;
    let heights = (0..side * side)
        .map(|index| {
            let point = (Vec2::new((index % side) as f32, (index / side) as f32)
                - Vec2::splat((side - 1) as f32 * 0.5))
                * spacing;
            source
                .elevation_at(point)
                .map_or(20.0, SupportElevation::metres)
        })
        .collect();
    let sampled = SceneTerrain::from_heightmap(side, side, spacing, heights).unwrap();
    let plan = fixture.selected_plan(&source);
    let accepted = BoundedSettlementTerrain::compile(
        &(std::slice::from_ref(&plan))
            .iter()
            .map(CompoundSupportPlan::support_surface)
            .collect::<Vec<_>>(),
        &source,
        FoundationEmbedment::from_metres(0.2).unwrap(),
    )
    .unwrap();
    let mut bound = sampled.clone().with_property_surface(accepted);
    let encoded = postcard::to_allocvec(&bound).unwrap();
    let decoded: SceneTerrain = postcard::from_bytes(&encoded).unwrap();
    assert_eq!(bound, decoded);
    assert_eq!(bound.colliders().len(), 2);
    let fine = bound.mesh_components_with_stride_filtered(1, |_| true);
    let coarse = bound.mesh_components_with_stride_filtered(5, |_| true);
    assert_eq!(fine, coarse, "terrain LOD changed an accepted foundation");
    for member in plan.member_support() {
        let point = member.contact.centre_metres;
        let expected = member.elevation.metres();
        let rendered = fine
            .1
            .as_chunks::<3>()
            .0
            .iter()
            .filter_map(|indices| {
                let triangle =
                    GroundTriangle::new(indices.map(|i| Vec3::from_array(fine.0[i as usize])))?;
                triangle
                    .contains(point, 0.001)
                    .then(|| triangle.height_at(point))
            })
            .max_by(f32::total_cmp)
            .unwrap();
        assert!(
            (rendered - expected).abs() < 0.001,
            "rendered foundation {rendered}, floor {expected}"
        );
        assert!((bound.height_at(point).unwrap() - expected).abs() < 0.001);
        assert!((bound.coarse_height_at(point).unwrap() - expected).abs() < 0.001);
        let mut hits = bound
            .colliders()
            .iter()
            .filter_map(|collider| {
                collider.cast_ray(
                    Vec3::ZERO,
                    avian3d::prelude::Rotation::default(),
                    Vec3::new(point.x, expected + 10.0, point.y),
                    Vec3::NEG_Y,
                    20.0,
                    false,
                )
            })
            .map(|(distance, _)| expected + 10.0 - distance)
            .collect::<Vec<_>>();
        hits.sort_by(f32::total_cmp);
        assert!((hits.last().unwrap() - expected).abs() < 0.001, "{hits:?}");
        assert!(hits.iter().all(|height| *height <= expected + 0.001));
    }
    let point = Vec2::new(-100.0, -300.0);
    assert_eq!(bound.height_at(point), sampled.height_at(point));
    // Rewriting or refining an already selected surface must not turn steps
    // into a heightfield or invalidate its support contract.
    assert!(!bound.rewrite_heights(|_, _| 0.0));
    assert_eq!(
        bound.constrain_max_grade(0.65),
        Err(crate::scene::TerrainGradeError::OwnedSurface)
    );
    assert!(bound.refined(1.0, |_, h| h).is_none());
    assert_eq!(bound, decoded);
}

#[test]
fn unowned_source_query_does_not_extrapolate_a_nearby_graded_floor() {
    let fixture = Fixture::load_965();
    let source = fixture.source();
    let plan = fixture.selected_plan(&source);
    let surface = plan.support_surface();
    let terrain = BoundedSettlementTerrain::compile(
        std::slice::from_ref(&surface),
        &source,
        FoundationEmbedment::from_metres(0.2).unwrap(),
    )
    .unwrap();
    // The street boundary has explicit source-matching elevations. The side
    // boundary may have a different floor: a contact-sized exterior point must
    // retain its source height, rather than extrapolating that floor outward.
    let plot = plan.reservation();
    let point = plot.centre_metres
        + plot
            .orientation
            .local_to_world(Vec2::new(-plot.dimensions_metres.x * 0.5 - 0.0005, 0.0));
    assert!(!surface.contains(point));
    let expected = source.elevation_at(point).unwrap().metres();
    let observed = terrain
        .highest_surface_at(point)
        .unwrap()
        .elevation
        .metres();
    assert!(
        (observed - expected).abs() < 0.001,
        "source {expected}, observed {observed}"
    );
}

#[test]
fn ownership_broad_phase_includes_aprons_beyond_disjoint_rotated_plots() {
    let fixture = Fixture::load_965();
    let source = fixture.source();
    let first = fixture.selected_plan(&source).support_surface();
    assert!(
        first.support_regions().len() > 1,
        "fixture must have an external approach"
    );
    let plot = first.support_regions()[0];
    let offset = plot
        .orientation
        .local_to_world(Vec2::new(0.0, -plot.dimensions_metres.y - 2.0));
    let mut second = first.clone();
    second.mesh.property_id = CityPropertyId(966);
    second.mesh.member_building_ids = vec![966, 17350];
    for point in &mut second.mesh.positions {
        point.x += offset.x;
        point.z += offset.y;
    }
    for region in &mut second.regions {
        region.centre_metres += offset;
    }
    for point in second.clipping_outlines.iter_mut().flatten() {
        *point += offset.as_dvec2();
    }
    assert!(
        !plot.intersects(second.support_regions()[0]),
        "plots must be disjoint"
    );
    let expected = first
        .support_regions()
        .iter()
        .zip(&first.clipping_outlines)
        .find_map(|(a, ao)| {
            second
                .support_regions()
                .iter()
                .zip(&second.clipping_outlines)
                .find_map(|(b, bo)| {
                    overlap(*a, *b, ao, bo).map(|(location, area)| (*a, *b, location, area))
                })
        })
        .expect("fixture must expose an apron conflict");
    let error = validate_owners(&[&first, &second]).unwrap_err();
    assert!(matches!(error, SettlementSupportError::OwnershipOverlap{
        first:CityPropertyId(965), second:CityPropertyId(966), first_region, second_region,
        location_metres, area_square_metres, ..
    }if (first_region,second_region,location_metres,area_square_metres)==expected));
}

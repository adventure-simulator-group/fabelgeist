use super::*;

fn surface() -> GeographicSurface {
    let point = |x: usize, z: usize| {
        let y = if x == 12 && z == 12 {
            4.0
        } else {
            x as f32 * 0.03 + z as f32 * 0.02
        };
        Vec3::new(x as f32, y, z as f32)
    };
    GeographicSurface::from_triangles((0..24).flat_map(|z| {
        (0..24).flat_map(move |x| {
            let [a, b, c, d] = [
                point(x, z),
                point(x + 1, z),
                point(x + 1, z + 1),
                point(x, z + 1),
            ];
            [[a, b, c], [a, c, d]]
        })
    }))
    .unwrap()
}

#[test]
fn indexed_source_retains_complete_rotated_intersections_and_interior_peak() {
    let source = surface();
    for angle in [0.0_f32, 0.23, 0.87, 1.5] {
        let rotation = bevy::math::Mat2::from_angle(angle);
        let outline = [
            Vec2::new(-2.0, -1.0),
            Vec2::new(2.0, -1.0),
            Vec2::new(2.0, 1.0),
            Vec2::new(-2.0, 1.0),
        ]
        .map(|p| Vec2::splat(12.0) + rotation * p);
        let mut controls = Vec::new();
        for i in 1..outline.len() - 1 {
            let support = GroundTriangle::new(
                [outline[0], outline[i], outline[i + 1]].map(|p| Vec3::new(p.x, 0.0, p.y)),
            )
            .unwrap();
            let exact: Vec<_> = source
                .triangles
                .iter()
                .filter_map(|triangle| {
                    let polygon = support.intersection(triangle);
                    (!polygon.is_empty()).then_some((triangle.points(), polygon))
                })
                .collect();
            let indexed: Vec<_> = source
                .intersecting(&support)
                .filter_map(|triangle| {
                    let polygon = support.intersection(triangle);
                    (!polygon.is_empty()).then_some((triangle.points(), polygon))
                })
                .collect();
            assert_eq!(indexed, exact);
            assert!(source.intersecting(&support).count() < source.triangles.len() / 10);
            controls.extend(exact.into_iter().flat_map(|(points, polygon)| {
                let triangle = GroundTriangle::new(points).unwrap();
                polygon.into_iter().map(move |p| GeographicHeightControl {
                    point: crate::scene_coordinates::ScenePlanPoint::from_metres(p).unwrap(),
                    elevation: SupportElevation(triangle.height_at(p)),
                })
            }));
        }
        let range = source
            .height_range_in_outline(
                &crate::scene_coordinates::ScenePlanPolygon::from_ordered_vertices(
                    (outline)
                        .iter()
                        .copied()
                        .map(crate::scene_coordinates::ScenePlanPoint::try_from)
                        .collect::<Result<Vec<_>, _>>()
                        .unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(
            range.minimum,
            *controls
                .iter()
                .min_by(|a, b| a.elevation.metres().total_cmp(&b.elevation.metres()))
                .unwrap()
        );
        assert_eq!(
            range.maximum,
            *controls
                .iter()
                .max_by(|a, b| a.elevation.metres().total_cmp(&b.elevation.metres()))
                .unwrap()
        );
        assert_eq!(range.maximum.elevation.metres(), 4.0);
    }
}

#[test]
fn indexed_sampling_retains_canonical_edges_and_reports_missing_source() {
    let source = surface();
    for point in [
        Vec2::ZERO,
        Vec2::splat(12.0),
        Vec2::new(12.3, 12.8),
        Vec2::splat(24.0),
        Vec2::splat(-0.001),
    ] {
        let exact = source
            .triangles
            .iter()
            .find(|triangle| triangle.contains(point, 0.0))
            .map(|triangle| SupportElevation(triangle.height_at(point)));
        assert_eq!(
            source.elevation_at(crate::scene_coordinates::ScenePlanPoint::try_from(point).unwrap()),
            exact
        );
    }
    let mut reordered: Vec<_> = source.triangles().collect();
    reordered.reverse();
    for triangle in &mut reordered {
        triangle.swap(0, 2);
    }
    let other = GeographicSurface::from_triangles(reordered).unwrap();
    assert_eq!(
        source.triangles().collect::<Vec<_>>(),
        other.triangles().collect::<Vec<_>>()
    );
    assert_eq!(
        source.elevation_at(
            crate::scene_coordinates::ScenePlanPoint::try_from(Vec2::splat(12.0)).unwrap()
        ),
        other.elevation_at(
            crate::scene_coordinates::ScenePlanPoint::try_from(Vec2::splat(12.0)).unwrap()
        )
    );
}

#[test]
fn complete_surface_comparison_detects_diagonal_peak_and_missing_coverage() {
    // Identical corner samples do not imply identical presented triangles.
    let [a, b, c, d] = [
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(2.0, 4.0, 2.0),
        Vec3::new(0.0, 0.0, 2.0),
    ];
    let first = GeographicSurface::from_triangles([[a, b, c], [a, c, d]]).unwrap();
    let second = GeographicSurface::from_triangles([[a, b, d], [b, c, d]]).unwrap();
    let outline = [a, b, c, d].map(|p| p.xz());
    let comparison = first
        .compare_in_outline(
            &second,
            &crate::scene_coordinates::ScenePlanPolygon::from_ordered_vertices(
                (outline)
                    .iter()
                    .copied()
                    .map(crate::scene_coordinates::ScenePlanPoint::try_from)
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(comparison.maximum.position_metres.metres(), Vec2::ONE);
    assert_eq!(comparison.maximum.difference_metres.metres(), 2.0);
    assert_eq!(comparison.minimum.difference_metres.metres(), 0.0);
    assert_eq!(comparison.covered_area_square_metres.square_metres(), 4.0);
    let reordered = GeographicSurface::from_triangles([[d, c, a], [c, b, a]]).unwrap();
    assert_eq!(
        serde_json::to_value(comparison).unwrap(),
        serde_json::to_value(
            reordered
                .compare_in_outline(
                    &second,
                    &crate::scene_coordinates::ScenePlanPolygon::from_ordered_vertices(
                        (outline)
                            .iter()
                            .copied()
                            .map(crate::scene_coordinates::ScenePlanPoint::try_from)
                            .collect::<Result<Vec<_>, _>>()
                            .unwrap()
                    )
                    .unwrap()
                )
                .unwrap()
        )
        .unwrap(),
    );
    let partial = GeographicSurface::from_triangles([[a, b, d]]).unwrap();
    let comparison = first
        .compare_in_outline(
            &partial,
            &crate::scene_coordinates::ScenePlanPolygon::from_ordered_vertices(
                (outline)
                    .iter()
                    .copied()
                    .map(crate::scene_coordinates::ScenePlanPoint::try_from)
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(comparison.required_area_square_metres.square_metres(), 4.0);
    assert_eq!(comparison.covered_area_square_metres.square_metres(), 2.0);
    assert!(
        crate::scene_coordinates::ScenePlanPolygon::from_ordered_vertices(vec![
            crate::scene_coordinates::ScenePlanPoint::ORIGIN
        ])
        .is_err()
    );
}

#[test]
fn complete_surface_comparison_accounts_for_rotated_footprint_intersections() {
    let first = surface();
    let mut points: Vec<_> = first.triangles().collect();
    for triangle in &mut points {
        for vertex in triangle {
            vertex.y += 0.5;
        }
    }
    let second = GeographicSurface::from_triangles(points).unwrap();
    let rotation = bevy::math::Mat2::from_angle(0.37);
    let outline = [
        Vec2::new(-2.0, -1.0),
        Vec2::new(2.0, -1.0),
        Vec2::new(2.0, 1.0),
        Vec2::new(-2.0, 1.0),
    ]
    .map(|p| Vec2::splat(12.0) + rotation * p);
    let comparison = first
        .compare_in_outline(
            &second,
            &crate::scene_coordinates::ScenePlanPolygon::from_ordered_vertices(
                (outline)
                    .iter()
                    .copied()
                    .map(crate::scene_coordinates::ScenePlanPoint::try_from)
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    assert!(
        (comparison.covered_area_square_metres.square_metres()
            - comparison.required_area_square_metres.square_metres())
        .abs()
            < 1e-8
    );
    assert!((comparison.minimum.difference_metres.metres() + 0.5).abs() < 1e-6);
    assert!((comparison.maximum.difference_metres.metres() + 0.5).abs() < 1e-6);
}

#[test]
fn boundary_query_preserves_tolerated_segment_sections_and_acute_corner_extensions() {
    use crate::city_layout::grounding::boundaries::segment_interval;
    use bevy::math::DVec2;
    let ordinary = surface();
    let acute = GeographicSurface::from_triangles([
        [
            Vec3::ZERO,
            Vec3::new(1000.0, 0.0, 0.0),
            Vec3::new(1000.0, 0.0, 0.01),
        ],
        [
            Vec3::new(2000.0, 0.0, 0.0),
            Vec3::new(2001.0, 0.0, 0.0),
            Vec3::new(2001.0, 0.0, 1.0),
        ],
    ])
    .unwrap();
    for (source, segments) in [
        (
            &ordinary,
            vec![
                [DVec2::new(-0.0008, 1.0), DVec2::new(-0.0008, 2.0)],
                [DVec2::new(11.7, 11.3), DVec2::new(12.4, 12.1)],
                [DVec2::new(-1.0, 12.0), DVec2::new(25.0, 12.0)],
            ],
        ),
        (
            &acute,
            vec![[DVec2::new(-50.0, -0.0005), DVec2::new(-49.0, -0.0005)]],
        ),
    ] {
        for segment in segments {
            let sections = |triangles: Vec<&GroundTriangle>| {
                triangles
                    .into_iter()
                    .filter_map(|triangle| {
                        segment_interval(
                            segment,
                            &triangle.points().map(|p| p.xz().as_dvec2()),
                            0.001,
                        )
                        .map(|interval| (triangle.points(), interval))
                    })
                    .collect::<Vec<_>>()
            };
            let exact = sections(source.triangles.iter().collect());
            let indexed = sections(
                source
                    .intersecting_boundary_segment(segment, 0.001)
                    .unwrap()
                    .collect(),
            );
            assert!(!exact.is_empty());
            assert_eq!(indexed, exact);
        }
    }
    let local = [DVec2::splat(12.0), DVec2::new(12.5, 12.3)];
    assert!(
        ordinary
            .intersecting_boundary_segment(local, 0.001)
            .unwrap()
            .count()
            < ordinary.triangles.len() / 10
    );
}

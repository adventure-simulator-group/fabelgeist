use super::*;

#[test]
fn goslar_1036_clipped_source_preserves_the_represented_geographic_plane() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/tactical-grounding/goslar-1036-clipped-source.json"
    )))
    .unwrap();
    let source = GroundTriangle::new(
        serde_json::from_value(fixture["source_triangle_metres"].clone()).unwrap(),
    )
    .unwrap();
    let cuts = SourceCutRegions::from_outlines(
        fixture["clipping_regions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|region| serde_json::from_value(region["outline_metres"].clone()).unwrap())
            .collect(),
    );
    let pieces = source.outside_regions(&cuts);
    assert!(pieces.len() > 1);
    for triangle in &pieces {
        for point in triangle.points() {
            assert_eq!(
                point.y,
                source.height_at(point.xz()),
                "clipped source must use the elevation of its represented XY coordinate"
            );
        }
        let [a, b, c] = triangle.points().map(Vec3::as_dvec3);
        let normal = (b - a).cross(c - a);
        let grade = normal.xz().length() / normal.y.abs();
        assert!(
            grade < 0.1,
            "natural grade became {grade} at {:?}",
            triangle.points()
        );
    }
    // Every source cut still uses its original horizontal geometry and order.
    let repeat = source.outside_regions(&cuts);
    assert_eq!(
        pieces
            .iter()
            .map(GroundTriangle::points)
            .collect::<Vec<_>>(),
        repeat
            .iter()
            .map(GroundTriangle::points)
            .collect::<Vec<_>>()
    );
}

#[test]
fn represented_edge_queries_do_not_extend_ownership_by_a_contact_margin() {
    let triangle = GroundTriangle::new([
        Vec3::new(-5.229, 0.0, 64.825),
        Vec3::new(-4.0, 0.0, 64.825),
        Vec3::new(-4.0, 0.0, 65.0),
    ])
    .unwrap();
    assert!(triangle.contains(
        Vec2::new(-5.229, 64.825),
        GroundTriangle::represented_point_tolerance(Vec2::new(-5.229, 64.825)),
    ));
    let outside = Vec2::new(-5.229 - 0.0005, 64.825);
    assert!(triangle.contains(outside, 0.001));
    assert!(!triangle.contains(
        outside,
        GroundTriangle::represented_point_tolerance(outside)
    ));
}

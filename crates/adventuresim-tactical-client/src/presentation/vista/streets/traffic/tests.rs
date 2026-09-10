use super::*;

fn road(start: Vec2, end: Vec2, half_width: f32) -> CityStreetPatch {
    CityStreetPatch::Corridor {
        start_metres: start,
        end_metres: end,
        half_width_metres: half_width,
        surface: CityStreetSurface::Fieldstone,
    }
}

#[test]
fn interior_crossings_produce_curved_wheels_inside_the_visible_street_union() {
    let streets = [
        road(Vec2::new(-24.0, 0.0), Vec2::new(24.0, 0.0), 4.0),
        road(Vec2::new(0.0, -24.0), Vec2::new(0.0, 24.0), 4.0),
    ];
    let network = TrafficNetwork::new(&streets);
    let diagonal = network
        .strokes
        .iter()
        .filter(|s| {
            let tangent = (s.end - s.start).normalize();
            tangent.x.abs() > 0.25 && tangent.y.abs() > 0.25
        })
        .count();
    assert!(
        diagonal > 40,
        "interior crossing must carry curved wheel marks: {diagonal}"
    );
    for stroke in &network.strokes {
        for point in [stroke.start, stroke.start.lerp(stroke.end, 0.5), stroke.end] {
            assert!(
                network.clearance(point) >= 0.0,
                "wheel outside street at {point}"
            );
        }
    }
}

#[test]
fn t_junctions_turn_but_disconnected_parallel_roads_do_not() {
    let main = road(Vec2::new(-20.0, 0.0), Vec2::new(20.0, 0.0), 3.0);
    let branch = road(Vec2::ZERO, Vec2::new(0.0, 20.0), 3.0);
    let network = TrafficNetwork::new(&[main, branch]);
    assert!(network.strokes.iter().any(|s| {
        let d = (s.end - s.start).normalize();
        d.x.abs() > 0.3 && d.y.abs() > 0.3
    }));
    let parallel = road(Vec2::new(-20.0, 8.0), Vec2::new(20.0, 8.0), 2.0);
    let separate = TrafficNetwork::new(&[main, parallel]);
    assert!(
        separate
            .strokes
            .iter()
            .all(|s| (s.end - s.start).normalize().y.abs() < 0.1)
    );
}

#[test]
fn masks_are_order_independent_and_share_identical_filter_gutters() {
    let streets = [
        road(Vec2::new(40.0, 12.0), Vec2::new(90.0, 12.0), 4.0),
        road(Vec2::new(64.0, -20.0), Vec2::new(64.0, 40.0), 4.0),
    ];
    let network = TrafficNetwork::new(&streets);
    let reversed = streets
        .into_iter()
        .rev()
        .map(|street| match street {
            CityStreetPatch::Corridor {
                start_metres,
                end_metres,
                half_width_metres,
                ..
            } => road(end_metres, start_metres, half_width_metres),
            _ => unreachable!(),
        })
        .collect::<Vec<_>>();
    let left = network.pixels(TrafficTile(0, 0));
    assert_eq!(
        left,
        TrafficNetwork::new(&reversed).pixels(TrafficTile(0, 0))
    );
    let right = network.pixels(TrafficTile(1, 0));
    for y in 0..TILE_PIXELS {
        for gutter in 0..FILTER_GUTTER_PIXELS * 2 {
            let a = (y * TILE_PIXELS + TILE_INTERIOR_PIXELS + gutter) * 4;
            let b = (y * TILE_PIXELS + gutter) * 4;
            assert_eq!(&left[a..a + 4], &right[b..b + 4], "tile seam at row {y}");
        }
    }
}

#[test]
fn mud_fills_the_carriageway_and_wheels_use_multiple_gauges() {
    let network = TrafficNetwork::new(&[road(Vec2::new(2.0, 16.0), Vec2::new(62.0, 16.0), 4.0)]);
    let pixels = network.pixels(TrafficTile(0, 0));
    let at = |point: Vec2| {
        let pixel = (point * TEXELS_PER_METRE).floor().as_uvec2()
            + UVec2::splat(FILTER_GUTTER_PIXELS as u32);
        &pixels[(pixel.y as usize * TILE_PIXELS + pixel.x as usize) * 4..][..4]
    };
    for y in [14.0, 15.0, 16.0, 17.0, 18.0] {
        assert!(at(Vec2::new(32.0, y))[0] > 150);
    }
    assert!(at(Vec2::new(32.0, 20.0))[0] < 30);
    let lanes = network
        .strokes
        .iter()
        .filter(|s| (s.start.x - 32.0).abs() < 0.7)
        .map(|s| (s.start.y * 10.0).round() as i32)
        .collect::<std::collections::BTreeSet<_>>();
    assert!(
        lanes.len() >= 8,
        "expected overlapping widths, found {lanes:?}"
    );
    let narrow = TrafficNetwork::new(&[road(Vec2::ZERO, Vec2::new(20.0, 0.0), 1.0)]);
    assert!(!narrow.strokes.is_empty());
    assert!(
        narrow
            .strokes
            .iter()
            .all(|s| s.start.is_finite() && s.end.is_finite())
    );
}

#[test]
fn shared_endpoint_junctions_have_no_internal_shoulders_or_track_gaps() {
    let centre = Vec2::splat(16.0);
    let streets = [Vec2::X, -Vec2::X, Vec2::Y, -Vec2::Y]
        .map(|direction| road(centre, centre + direction * 14.0, 3.0));
    let network = TrafficNetwork::new(&streets);
    let pixels = network.pixels(TrafficTile(0, 0));
    for y in 60..=68 {
        for x in 60..=68 {
            let index = ((y + FILTER_GUTTER_PIXELS) * TILE_PIXELS + x + FILTER_GUTTER_PIXELS) * 4;
            assert_eq!(
                pixels[index + 2],
                255,
                "internal rectangle edge must not become a shoulder"
            );
            assert!(
                pixels[index] > 180,
                "traffic mud must cross shared endpoints"
            );
        }
    }
    assert!(
        network
            .strokes
            .iter()
            .any(|s| s.start.distance(centre) < 1.0)
    );
}

#[test]
fn angled_connections_and_duplicate_widths_are_supported_deterministically() {
    let a = road(Vec2::ZERO, Vec2::new(24.0, 0.0), 3.0);
    let b = road(Vec2::ZERO, Vec2::new(-23.0, 6.0), 3.0);
    let narrow = road(Vec2::ZERO, Vec2::new(24.0, 0.0), 1.5);
    let network = TrafficNetwork::new(&[a, b, narrow]);
    assert_eq!(
        network.pixels(TrafficTile(0, 0)),
        TrafficNetwork::new(&[narrow, b, a]).pixels(TrafficTile(0, 0))
    );
    assert!(
        network.strokes.iter().any(|s| {
            let direction = (s.end - s.start).normalize();
            direction.y.abs() > 0.07 && direction.y.abs() < 0.17
        }),
        "near-straight arms still need a smooth heading transition"
    );
}

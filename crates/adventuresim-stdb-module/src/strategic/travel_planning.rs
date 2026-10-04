use adventuresim_world_schema::coordinates::{UnboundedCoordinateE7, Wgs84CoordinateE7};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct StrategicPositionE7 {
    longitude_e7: i32,
    latitude_e7: i32,
}

/// Encodes the hybrid strategic-location wire representation. Geographic
/// positions are validated as WGS84; explicitly abstract positions preserve
/// their unbounded planar coordinate convention.
fn encode_position_e7(
    longitude: f64,
    latitude: f64,
    coordinates_are_geographic: bool,
) -> Option<StrategicPositionE7> {
    if coordinates_are_geographic {
        let coordinate = Wgs84CoordinateE7::from_longitude_latitude_degrees(longitude, latitude)?;
        Some(StrategicPositionE7 {
            longitude_e7: coordinate.longitude().get(),
            latitude_e7: coordinate.latitude().get(),
        })
    } else {
        let longitude = UnboundedCoordinateE7::from_coordinate_units(longitude)?;
        let latitude = UnboundedCoordinateE7::from_coordinate_units(latitude)?;
        Some(StrategicPositionE7 {
            longitude_e7: longitude.raw(),
            latitude_e7: latitude.raw(),
        })
    }
}

/// Decodes the hybrid strategic-location wire representation. Invalid WGS84
/// values fail closed instead of entering distance or route calculations.
fn decode_position_e7(
    longitude_e7: i32,
    latitude_e7: i32,
    coordinates_are_geographic: bool,
) -> Option<(f64, f64)> {
    if coordinates_are_geographic {
        Wgs84CoordinateE7::new(latitude_e7, longitude_e7)
            .map(|coordinate| coordinate.longitude_latitude_degrees())
    } else {
        Some((
            UnboundedCoordinateE7::from_raw(longitude_e7).coordinate_units(),
            UnboundedCoordinateE7::from_raw(latitude_e7).coordinate_units(),
        ))
    }
}

fn wgs84_route_coordinate(point: &JourneyRoutePoint) -> Option<Wgs84CoordinateE7> {
    Wgs84CoordinateE7::new(point.latitude_e7, point.longitude_e7)
}

fn travel_neighbors(ctx: &ReducerContext, node: u64) -> Vec<(u64, u32)> {
    let mut neighbors: Vec<_> = ctx
        .db
        .travel_edge()
        .from_node_id()
        .filter(&node)
        .map(|edge| (edge.to_node_id, edge.length_m))
        .collect();
    neighbors.extend(
        ctx.db
            .travel_edge()
            .to_node_id()
            .filter(&node)
            .map(|edge| (edge.from_node_id, edge.length_m)),
    );
    neighbors
}

/// Returns the next settlements reached from a source. Paths end at the first
/// settlement encountered, so journeys cannot skip intermediate settlements.
fn connected_settlement_distances(ctx: &ReducerContext, source_node_id: u64) -> HashMap<u64, u64> {
    let settlement_nodes: HashSet<u64> = ctx
        .db
        .settlement()
        .iter()
        .filter_map(|settlement| settlement.source_node_id)
        .collect();
    let mut distances = HashMap::from([(source_node_id, 0_u64)]);
    let mut pending = BinaryHeap::from([std::cmp::Reverse((0_u64, source_node_id))]);
    let mut destinations = HashMap::new();

    while let Some(std::cmp::Reverse((distance, node))) = pending.pop() {
        if distances.get(&node).is_some_and(|known| *known != distance) {
            continue;
        }
        if node != source_node_id && settlement_nodes.contains(&node) {
            destinations.insert(node, distance);
            continue;
        }
        for (neighbor, length_m) in travel_neighbors(ctx, node) {
            let next_distance = distance.saturating_add(u64::from(length_m));
            if distances
                .get(&neighbor)
                .is_none_or(|known| next_distance < *known)
            {
                distances.insert(neighbor, next_distance);
                pending.push(std::cmp::Reverse((next_distance, neighbor)));
            }
        }
    }
    destinations
}

fn journey_minutes(distance_m: u64) -> u64 {
    distance_m
        .saturating_mul(MINUTES_PER_HOUR)
        .div_ceil(
            adventuresim_core::strategic_time::OVERLAND_WALKING_SPEED_KM_PER_HOUR
                * METERS_PER_KILOMETER,
        )
        .max(1)
}

fn quest_journey_minutes(distance_m: u64) -> u64 {
    journey_minutes(distance_m).saturating_mul(QUEST_TRAVEL_SPEED_DIVISOR)
}

fn straight_line_distance_m(
    from_x: f64,
    from_y: f64,
    to_x: f64,
    to_y: f64,
    geographic: bool,
) -> u64 {
    if geographic {
        let earth_radius_m = 6_371_000.0_f64;
        let lat1 = from_y.to_radians();
        let lat2 = to_y.to_radians();
        let delta_lat = (to_y - from_y).to_radians();
        let delta_lon = (to_x - from_x).to_radians();
        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1.cos() * lat2.cos() * (delta_lon / 2.0).sin().powi(2);
        if !a.is_finite() {
            return u64::MAX;
        }
        let a = a.clamp(0.0, 1.0);
        let distance_m = earth_radius_m * 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        if distance_m.is_finite() {
            distance_m.round() as u64
        } else {
            u64::MAX
        }
    } else {
        (((from_x - to_x).powi(2) + (from_y - to_y).powi(2)).sqrt() * METERS_PER_KILOMETER as f64)
            .round() as u64
    }
}

/// Canonical travel distance between signed E7 coordinates, in meters.
/// Geographic mode uses great-circle distance. Abstract mode uses the same
/// Euclidean coordinate-units-as-kilometers convention as strategic travel.
/// Invalid geographic latitude/longitude values fail closed.
pub(crate) fn coordinate_distance_e7_m(
    from_longitude_e7: i32,
    from_latitude_e7: i32,
    to_longitude_e7: i32,
    to_latitude_e7: i32,
    coordinates_are_geographic: bool,
) -> Option<u64> {
    let from = decode_position_e7(
        from_longitude_e7,
        from_latitude_e7,
        coordinates_are_geographic,
    )?;
    let to = decode_position_e7(to_longitude_e7, to_latitude_e7, coordinates_are_geographic)?;
    Some(straight_line_distance_m(
        from.0,
        from.1,
        to.0,
        to.1,
        coordinates_are_geographic,
    ))
}

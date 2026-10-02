//! Compare one level courtyard with stepped terraces and a central landing.
//! This is a decision experiment, not runtime grading or acceptance geometry.
use adventuresim_tactical_core::city_layout::{CityAccessSegment, CityCompound};
use bevy::math::Vec2;
use serde_json::{Value, json};

/// Reserves a landing at each route end equal to the route's half-width.
/// Existing motor limits still need verification against generated triangles.
fn ramp_length(route: &CityAccessSegment) -> f32 {
    (route.start_metres.distance(route.end_metres) - route.half_width_metres * 2.0).max(0.0)
}

pub(super) fn compare(
    compound: &CityCompound,
    front_threshold: Vec2,
    rear_threshold: Vec2,
    front_floor: f32,
    terrain_height: impl Fn(Vec2) -> Option<f32>,
    maximum_grade: f32,
) -> Option<Value> {
    let route_to = |threshold: Vec2| {
        let mut matches = compound
            .access
            .iter()
            .filter(|route| route.ends_at(threshold));
        let route = matches.next()?;
        matches.next().is_none().then_some(route)
    };
    let front_route = route_to(front_threshold)?;
    let rear_route = route_to(rear_threshold)?;
    if ramp_length(front_route) <= 0.0 || ramp_length(rear_route) <= 0.0 {
        return None;
    }
    let court_source = terrain_height(compound.court.centre_metres)?;
    let rear_source = terrain_height(rear_threshold)?;
    let front_reach = ramp_length(front_route) * maximum_grade;
    let rear_reach = ramp_length(rear_route) * maximum_grade;
    let court = court_source.clamp(front_floor - front_reach, front_floor + front_reach);
    let rear = rear_source.clamp(court - rear_reach, court + rear_reach);
    let candidate = |name: &str, levels: [f32; 3]| {
        json!({
            "name": name,
            "property_id": compound.id,
            "member_building_ids": [compound.front_building_id,compound.rear_building_id],
            "front_court_rear_elevations_m": levels,
            "front_route":front_route,
            "rear_route":rear_route,
            "front_effective_ramp_length_m":ramp_length(front_route),
            "rear_effective_ramp_length_m":ramp_length(rear_route),
            "front_ramp_grade":(levels[0]-levels[1]).abs()/ramp_length(front_route),
            "rear_ramp_grade":(levels[2]-levels[1]).abs()/ramp_length(rear_route),
            "central_landing_elevation_m":levels[1],
            "entire_court_is_level":levels[0] == levels[1] && levels[1] == levels[2],
            "court_requires_terraced_surface":levels[0] != levels[1] || levels[1] != levels[2],
            "verification_scope":"Endpoint level feasibility with explicit landings. No support, retaining-wall, gate-sweep or complete terrain-triangle acceptance is asserted.",
        })
    };
    Some(json!({
        "maximum_candidate_grade": maximum_grade,
        "source_court_elevation_m":court_source,
        "source_rear_threshold_elevation_m":rear_source,
        "candidates":[candidate("level_courtyard_and_both_buildings",[front_floor;3]),
            candidate("stepped_court_with_level_central_landing",[front_floor,court,rear])],
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Fixture {
        compound: CityCompound,
        front_threshold: Vec2,
        rear_threshold: Vec2,
        front_floor_m: f32,
        court_source_m: f32,
        rear_source_m: f32,
    }

    impl Fixture {
        fn load() -> Self {
            let value: Value =
                serde_json::from_str(include_str!("fixtures/goslar-1238.json")).unwrap();
            Self {
                compound: serde_json::from_value(value["compound"].clone()).unwrap(),
                front_threshold: serde_json::from_value(value["front_threshold"].clone()).unwrap(),
                rear_threshold: serde_json::from_value(value["rear_threshold"].clone()).unwrap(),
                front_floor_m: value["front_floor_m"].as_f64().unwrap() as f32,
                court_source_m: value["court_source_m"].as_f64().unwrap() as f32,
                rear_source_m: value["rear_source_m"].as_f64().unwrap() as f32,
            }
        }
    }

    #[test]
    fn goslar_1238_requires_a_stepped_court_and_reserves_both_landings() {
        let fixture = Fixture::load();
        let height = |point: Vec2| {
            Some(if point == fixture.compound.court.centre_metres {
                fixture.court_source_m
            } else {
                fixture.rear_source_m
            })
        };
        let run = |property: &CityCompound| {
            compare(
                property,
                fixture.front_threshold,
                fixture.rear_threshold,
                fixture.front_floor_m,
                height,
                0.65,
            )
            .unwrap()
        };
        let report = run(&fixture.compound);
        let terraced = &report["candidates"][1];
        let levels = terraced["front_court_rear_elevations_m"]
            .as_array()
            .unwrap();
        let [front, court, rear] = std::array::from_fn::<_, 3, _>(|i| levels[i].as_f64().unwrap());
        assert!(front > court && court > rear);
        assert!(
            rear < front - 2.0,
            "rear range does not inherit front floor"
        );
        assert_eq!(report["candidates"][0]["entire_court_is_level"], true);
        assert_eq!(terraced["entire_court_is_level"], false);
        assert_eq!(terraced["court_requires_terraced_surface"], true);
        for route in [&fixture.compound.access[2], &fixture.compound.access[4]] {
            assert!(
                fixture
                    .compound
                    .court
                    .contains((route.start_metres + route.end_metres) * 0.5),
                "an access ramp crosses the reserved court rather than bypassing it"
            );
        }
        for key in ["front_ramp_grade", "rear_ramp_grade"] {
            assert!(terraced[key].as_f64().unwrap() <= 0.650_001);
        }
        let route_length = fixture.compound.access[2]
            .start_metres
            .distance(fixture.compound.access[2].end_metres);
        assert!(
            terraced["front_effective_ramp_length_m"].as_f64().unwrap()
                < f64::from(route_length) - 0.79
        );
        let mut reversed = fixture.compound.clone();
        reversed.access.reverse();
        assert_eq!(report, run(&reversed));
        assert_eq!(terraced["member_building_ids"], json!([1238, 17622]));
    }

    #[test]
    fn a_nearby_route_cannot_replace_a_missing_threshold_binding() {
        let fixture = Fixture::load();
        assert!(
            compare(
                &fixture.compound,
                fixture.front_threshold + Vec2::X,
                fixture.rear_threshold,
                fixture.front_floor_m,
                |_| Some(0.0),
                0.65
            )
            .is_none()
        );
    }
}

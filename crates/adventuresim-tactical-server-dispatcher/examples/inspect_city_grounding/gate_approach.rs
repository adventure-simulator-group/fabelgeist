//! Test the street-to-gate constraint before accepting interior route endpoints.
use adventuresim_tactical_core::city_layout::{CityAccessSegment, CityCompound};
use bevy::math::Vec2;
use serde_json::{Value, json};
#[path = "gate_approach/profile.rs"]
mod profile;

pub(super) fn inspect(
    compound: &CityCompound,
    common_floor: f32,
    court_floor: f32,
    terrain_height: impl Fn(Vec2) -> Option<f32>,
    maximum_grade: f32,
) -> Result<Option<Value>, adventuresim_building_generator::DoorError> {
    let gate_point = compound.boundary.gate.centre_metres;
    // Native endpoint-grade and height-query kernel consumes scene metres.
    let gate = gate_point.metres();
    let mut crossings = compound
        .access
        .iter()
        .filter(|route| route.contains_centreline(gate_point));
    let Some(route) = crossings.next() else {
        return Ok(None);
    };
    if crossings.next().is_some() {
        return Ok(None);
    }
    let length = route.start_metres().distance(gate);
    let effective_run = (length - route.half_width_metres() * 2.0).max(0.0);
    let Some(street_height) = terrain_height(route.start_metres()) else {
        return Ok(None);
    };
    let Some(natural_gate_height) = terrain_height(gate) else {
        return Ok(None);
    };
    let maximum_rise = effective_run * maximum_grade;
    let report = |name, gate_floor: f32| {
        let rise = (street_height - gate_floor).abs();
        json!({
            "name":name,"gate_floor_m":gate_floor,
            "required_rise_m":rise,"permitted_rise_m":maximum_rise,
            "shortfall_m":(rise-maximum_rise).max(0.0),
            "grade":if effective_run > 0.0 {Some(rise/effective_run)} else {None},
            "passes_endpoint_grade":effective_run > 0.0 && rise <= maximum_rise,
            "difference_from_front_floor_m":gate_floor-common_floor,
        })
    };
    Ok(Some(json!({
        "property_id":compound.id,
        "member_building_ids":[compound.front_building_id,compound.rear_building_id],
        "street_endpoint":route.start_metres(),"gate_centre":gate,
        "street_elevation_m":street_height,"natural_gate_elevation_m":natural_gate_height,
        "approach_length_m":length,"effective_run_m":effective_run,
        "landing_length_each_m":route.half_width_metres(),
        "maximum_candidate_grade":maximum_grade,
        "candidates":[report("gate_at_common_front_floor",common_floor),
            report("separate_gate_landing_at_source_height",natural_gate_height)],
        "independent_gate_to_court_profile":profile::inspect(compound, route,
            street_height, natural_gate_height, court_floor, maximum_grade)?,
        "constraint":"The unchanged street endpoint must reach a level gate landing. Independent gate elevation requires coherent post support, retaining geometry and an access route to the court.",
        "verification_scope":"Endpoint constraint only. Does not accept complete ramps, gate sweeps, supporting geometry or generation.",
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn goslar_1238_interior_endpoint_feasibility_does_not_accept_the_gate() {
        let fixture: Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/tactical-grounding/goslar-1238.json"
        )))
        .unwrap();
        let compound: CityCompound = serde_json::from_value(fixture["compound"].clone()).unwrap();
        let floor = fixture["front_floor_m"].as_f64().unwrap() as f32;
        let street = fixture["street_approach_source_m"].as_f64().unwrap() as f32;
        let gate = fixture["gate_source_m"].as_f64().unwrap() as f32;
        let height = |point| {
            Some(if point == compound.access[0].start_metres() {
                street
            } else {
                gate
            })
        };
        let court = fixture["court_candidate_m"].as_f64().unwrap() as f32;
        let report = inspect(&compound, floor, court, height, 0.65)
            .unwrap()
            .unwrap();
        assert_eq!(report["candidates"][0]["passes_endpoint_grade"], false);
        assert!(report["candidates"][0]["shortfall_m"].as_f64().unwrap() > 2.0);
        assert_eq!(report["candidates"][1]["passes_endpoint_grade"], true);
        assert!(
            report["candidates"][1]["difference_from_front_floor_m"]
                .as_f64()
                .unwrap()
                > 2.1
        );
        let passage = &report["independent_gate_to_court_profile"];
        assert_eq!(passage["passes_endpoint_grade"], true);
        assert!(passage["grade"].as_f64().unwrap() < 0.22);
        assert!(passage["gate_to_court_effective_run_m"].as_f64().unwrap() > 16.8);
        let mut reversed = compound.clone();
        reversed.access.reverse();
        assert_eq!(
            report,
            inspect(&reversed, floor, court, height, 0.65)
                .unwrap()
                .unwrap()
        );
    }
}

//! Bounded construction comparison; this does not publish runtime foundations.
use adventuresim_tactical_core::city_layout::grounding::{
    CompoundSupportLevels, CompoundSupportPlan, CourtStairLimits, CourtTreatment, MemberSupport,
    SupportElevation, SupportLimits,
};
use adventuresim_tactical_core::city_layout::{CityCompound, CityPlotBounds};
use bevy::math::{Vec2, Vec3, Vec3Swizzles};
use serde_json::{Value, json};
#[path = "solutions/quantities.rs"]
mod quantities;

const COMPARISON_CONTACT_TOLERANCE_METRES: f32 = 0.001;
const PROVISIONAL_COMPARISON_CUT_FILL_METRES: f32 = 6.0;
const COMPARISON_MAXIMUM_RISER_METRES: f32 = 0.19;
const COMPARISON_MINIMUM_GOING_METRES: f32 = 0.25;
const COMPARISON_STAIR_CLEAR_WIDTH_METRES: f32 = 1.0;
const COMPARISON_LANDING_RUN_METRES: f32 = 0.5;

pub(super) fn stair_limits() -> CourtStairLimits {
    CourtStairLimits::new(
        COMPARISON_MAXIMUM_RISER_METRES,
        COMPARISON_MINIMUM_GOING_METRES,
        COMPARISON_STAIR_CLEAR_WIDTH_METRES,
        COMPARISON_LANDING_RUN_METRES,
        COMPARISON_LANDING_RUN_METRES,
    )
    .expect("authored comparison stair dimensions are valid")
}

pub(super) fn compare(
    property: &CityCompound,
    members: [(CityPlotBounds, Vec2); 2],
    proposed_levels_metres: [f32; 3],
    geographic: &[[Vec3; 3]],
    height: impl Fn(Vec2) -> Option<f32>,
    maximum_grade: f32,
) -> Option<Value> {
    let elevation = SupportElevation::from_metres;
    let mut gate_routes = property
        .access
        .iter()
        .filter(|r| r.contains_centreline(property.boundary.gate.centre_metres));
    let route = gate_routes
        .next()
        .filter(|_| gate_routes.next().is_none())?;
    let levels = CompoundSupportLevels {
        front: MemberSupport {
            building_id: property.front_building_id,
            contact: members[0].0,
            court_threshold_metres: members[0].1,
            elevation: elevation(proposed_levels_metres[0])?,
        },
        rear: MemberSupport {
            building_id: property.rear_building_id,
            contact: members[1].0,
            court_threshold_metres: members[1].1,
            elevation: elevation(proposed_levels_metres[2])?,
        },
        court: elevation(proposed_levels_metres[1])?,
        gate: elevation(height(property.boundary.gate.centre_metres)?)?,
        street: elevation(height(route.start_metres)?)?,
    };
    let limits = SupportLimits::new(
        maximum_grade,
        PROVISIONAL_COMPARISON_CUT_FILL_METRES,
        COMPARISON_CONTACT_TOLERANCE_METRES,
    )?;
    let stairs = stair_limits();
    let thickness = property
        .boundary
        .walls
        .iter()
        .map(|w| w.thickness_metres)
        .max_by(f32::total_cmp)?;
    let candidates=[CourtTreatment::Level,CourtTreatment::Terraced(stairs)].map(|treatment| {
        match CompoundSupportPlan::compile(property,levels,limits,treatment) {
            Ok(plan)=>json!({"treatment":treatment,
                "member_support":plan.member_support(),"court_elevation":plan.court_elevation(),
                "gate_elevation":plan.gate_elevation(),"stair_flights":plan.stair_flights().collect::<Vec<_>>(),
                "mesh_maximum_grade":plan.mesh().maximum_grade(),
                "quantities":quantities::measure(&plan,geographic,&height,thickness),"mesh":plan.mesh(),
            }),
            Err(error)=>json!({"treatment":treatment,"support_rejection":error}),
        }
    });
    Some(
        json!({"property_id":property.id,"member_building_ids":[property.front_building_id,property.rear_building_id],
            "chosen_level_inputs":levels,"comparison_limits":limits,"candidates":candidates,
            "geographic_triangles":geographic,
            "access_scope":"Pedestrian support surfaces. Cart/service traversal, neighbouring boundary geometry, drainage and retaining-wall structural capacity are not accepted by this comparison.",
            "verification_scope":"Bounded construction decision experiment. Six-metre cut/fill and stair dimensions are explicit provisional modelling assumptions, not calibrated runtime limits or historical measurements. No production terrain is changed, no required positive fixture is accepted, and no retaining-wall collision capacity is asserted.",
        }),
    )
}

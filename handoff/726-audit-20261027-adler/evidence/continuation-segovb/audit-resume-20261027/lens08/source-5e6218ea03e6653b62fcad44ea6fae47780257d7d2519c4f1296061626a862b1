//! Bounded construction comparison; this does not publish runtime foundations.
use adventuresim_tactical_core::city_layout::grounding::{
    CompoundSupportLevels, CompoundSupportPlan, CourtStairLimits, CourtTreatment, MemberSupport,
    SupportElevation, SupportLimits,
};
use adventuresim_tactical_core::city_layout::{CityCompound, CityPlotBounds};
use adventuresim_tactical_core::scene_coordinates::ScenePlanPoint;
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

const fn positive_comparison_length(
    value: f32,
) -> adventuresim_building_generator::spatial_geometry::PositiveLength {
    match adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(value) {
        Ok(length) => length,
        Err(_) => panic!("authored comparison lengths must be positive"),
    }
}
const COMPARISON_STAIRS: CourtStairLimits = CourtStairLimits::new(
    positive_comparison_length(COMPARISON_MAXIMUM_RISER_METRES),
    positive_comparison_length(COMPARISON_MINIMUM_GOING_METRES),
    positive_comparison_length(COMPARISON_STAIR_CLEAR_WIDTH_METRES),
    positive_comparison_length(COMPARISON_LANDING_RUN_METRES),
    positive_comparison_length(COMPARISON_LANDING_RUN_METRES),
);
pub(super) fn stair_limits() -> CourtStairLimits {
    COMPARISON_STAIRS
}

#[derive(Debug, thiserror::Error)]
pub(super) enum ComparisonInputError {
    #[error("non-finite comparison threshold {attempted_metres:?}")]
    NonFiniteThreshold { attempted_metres: Vec2 },
    #[error("comparison requires three finite front/court/rear elevations")]
    InvalidProposedLevels,
}

pub(super) struct ComparisonMember {
    pub bearing: CityPlotBounds,
    pub threshold: ScenePlanPoint,
}
impl ComparisonMember {
    pub fn from_measurement(
        bearing: CityPlotBounds,
        threshold_metres: Vec2,
    ) -> Result<Self, ComparisonInputError> {
        Ok(Self {
            bearing,
            threshold: ScenePlanPoint::from_metres(threshold_metres).ok_or(
                ComparisonInputError::NonFiniteThreshold {
                    attempted_metres: threshold_metres,
                },
            )?,
        })
    }
}
pub(super) struct ComparisonMembers {
    pub front: ComparisonMember,
    pub rear: ComparisonMember,
}
pub(super) struct ProposedCompoundLevels {
    pub front: SupportElevation,
    pub court: SupportElevation,
    pub rear: SupportElevation,
}

impl ProposedCompoundLevels {
    /// Adapt the diagnostic producer's front/court/rear JSON array explicitly.
    pub fn from_capture(value: &Value) -> Result<Self, ComparisonInputError> {
        Self::decode_capture(value).ok_or(ComparisonInputError::InvalidProposedLevels)
    }
    fn decode_capture(value: &Value) -> Option<Self> {
        let values: &[Value; 3] = value.as_array()?.as_slice().try_into().ok()?;
        let elevation = |value: &Value| SupportElevation::from_metres(value.as_f64()? as f32);
        Some(Self {
            front: elevation(&values[0])?,
            court: elevation(&values[1])?,
            rear: elevation(&values[2])?,
        })
    }
}

pub(super) fn compare(
    property: &CityCompound,
    members: ComparisonMembers,
    proposed: ProposedCompoundLevels,
    geographic: &[[Vec3; 3]],
    height: impl Fn(Vec2) -> Option<f32>,
    maximum_grade: f32,
) -> Option<Value> {
    let elevation = SupportElevation::from_metres;
    let gate = ScenePlanPoint::from_metres(property.boundary.gate.centre_metres)?;
    let mut gate_routes = property
        .access
        .iter()
        .filter(|r| r.contains_centreline(gate));
    let route = gate_routes
        .next()
        .filter(|_| gate_routes.next().is_none())?;
    let levels = CompoundSupportLevels {
        front: MemberSupport {
            building_id: property.front_building_id,
            contact: members.front.bearing,
            court_threshold_metres: members.front.threshold,
            elevation: proposed.front,
        },
        rear: MemberSupport {
            building_id: property.rear_building_id,
            contact: members.rear.bearing,
            court_threshold_metres: members.rear.threshold,
            elevation: proposed.rear,
        },
        court: proposed.court,
        gate: elevation(height(property.boundary.gate.centre_metres)?)?,
        street: elevation(height(route.start_metres())?)?,
    };
    let limits = SupportLimits::new(
        adventuresim_tactical_core::city_layout::grounding::SupportGrade::from_ratio(
            maximum_grade,
        )?,
        adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
            PROVISIONAL_COMPARISON_CUT_FILL_METRES,
        )
        .ok()?,
        adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
            COMPARISON_CONTACT_TOLERANCE_METRES,
        )
        .ok()?,
    );
    let stairs = stair_limits();
    let thickness = property
        .boundary
        .walls
        .iter()
        .map(|w| w.thickness_metres)
        .max_by(f32::total_cmp)?;
    let candidates=[CourtTreatment::Level,CourtTreatment::Terraced(stairs)].map(|treatment| {
        match CompoundSupportPlan::compile(property,levels,limits,treatment) {
            Ok(plan)=>{
                let mesh = match plan.mesh() {
                    Ok(mesh) => mesh,
                    Err(error) => return json!({"treatment":treatment,"support_rejection":error}),
                };
                match quantities::measure(&plan,geographic,&height,thickness) { Ok(quantities)=>json!({"treatment":treatment,
                "member_support":plan.member_support(),"court_elevation":plan.court_elevation(),
                "gate_elevation":plan.gate_elevation(),"stair_flights":plan.stair_flights().collect::<Vec<_>>(),
                "mesh_maximum_grade":mesh.maximum_grade(),
                "quantities":quantities,"mesh":mesh,
            }), Err(error)=>json!({"treatment":treatment,"property_id":property.id,"quantity_rejection":error}) }},
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

#[cfg(test)]
mod input_tests {
    use super::*;
    #[test]
    fn proposed_levels_reject_malformed_or_unrepresentable_capture_values() {
        assert!(ProposedCompoundLevels::from_capture(&json!([1, 2])).is_err());
        assert!(ProposedCompoundLevels::from_capture(&json!([1, 2, 3, 4])).is_err());
        assert!(ProposedCompoundLevels::from_capture(&json!([1, 2, 1e100])).is_err());
        let levels = ProposedCompoundLevels::from_capture(&json!([-1, 2, 3])).unwrap();
        assert_eq!(levels.front.metres(), -1.0);
        assert_eq!(levels.court.metres(), 2.0);
        assert_eq!(levels.rear.metres(), 3.0);
    }
}

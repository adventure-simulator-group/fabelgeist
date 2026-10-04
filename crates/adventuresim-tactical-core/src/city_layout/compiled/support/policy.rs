//! Engineering bounds for production settlement grading, independent of history.
use super::*;

const MAXIMUM_CUT_FILL_METRES: f32 = 6.0;
const MAXIMUM_RISER_METRES: f32 = 0.19;
const MINIMUM_GOING_METRES: f32 = 0.25;
const MINIMUM_CLEAR_STAIR_WIDTH_METRES: f32 = 1.0;
const MINIMUM_FLOOR_LANDING_RUN_METRES: f32 = 1.05;
const MINIMUM_COURT_LANDING_RUN_METRES: f32 = 0.5;
const FOUNDATION_EMBEDMENT_METRES: f32 = 0.2;
const DOORWAY_APPROACH_WIDTH_METRES: f32 = 1.0;
const MAXIMUM_STREET_APPROACH_RUN_METRES: f32 = 4.0;

impl CompoundGradingPolicy {
    /// Fixed engineering constraints used by generated settlements. Source
    /// relief outside each private reservation and bound approach is untouched.
    /// These bounds do not certify actor clearance or historical construction.
    pub fn bounded_settlement() -> Self {
        Self {
            limits: SupportLimits::new(
                crate::scene_input::MAX_PLAYABLE_GRADE,
                MAXIMUM_CUT_FILL_METRES,
                CityPlotBounds::COORDINATE_TOLERANCE_METRES as f32,
            )
            .expect("positive production support limits"),
            stairs: CourtStairLimits::new(
                MAXIMUM_RISER_METRES,
                MINIMUM_GOING_METRES,
                MINIMUM_CLEAR_STAIR_WIDTH_METRES,
                MINIMUM_FLOOR_LANDING_RUN_METRES,
                MINIMUM_COURT_LANDING_RUN_METRES,
            )
            .expect("positive production stair limits"),
            embedment: FoundationEmbedment::from_metres(FOUNDATION_EMBEDMENT_METRES)
                .expect("positive foundation embedment"),
            street_apron: StreetApronDimensions::from_metres(Vec2::new(
                DOORWAY_APPROACH_WIDTH_METRES,
                MAXIMUM_STREET_APPROACH_RUN_METRES,
            ))
            .expect("positive doorway approach dimensions"),
        }
    }
}

//! Engineering bounds for production settlement grading, independent of history.
use super::*;
use crate::city_layout::grounding::SupportGrade;
use adventuresim_building_generator::spatial_geometry::PositiveLength;

// Admit authored constants at compile time. An invalid engineering literal is
// a build failure, never a runtime fallback or a newly selected terrain policy.
const fn positive_metres(value: f32) -> PositiveLength {
    match PositiveLength::from_metres(value) {
        Ok(length) => length,
        Err(_) => panic!("production engineering lengths must be positive"),
    }
}
const SETTLEMENT_LIMITS: SupportLimits = SupportLimits::new(
    match SupportGrade::from_ratio(crate::scene::TerrainGradeLimit::PLAYABLE.ratio()) {
        Some(grade) => grade,
        None => panic!("production support grade must be positive"),
    },
    positive_metres(6.0),
    positive_metres(CityPlotBounds::COORDINATE_TOLERANCE_METRES as f32),
);
const SETTLEMENT_STAIRS: CourtStairLimits = CourtStairLimits::new(
    positive_metres(0.19),
    positive_metres(0.25),
    positive_metres(1.0),
    positive_metres(1.05),
    positive_metres(0.5),
);
const SETTLEMENT_EMBEDMENT: FoundationEmbedment = FoundationEmbedment::new(positive_metres(0.2));
const STREET_APPROACH: StreetApronDimensions =
    StreetApronDimensions::new(positive_metres(1.0), positive_metres(4.0));

impl CompoundGradingPolicy {
    /// Fixed engineering constraints used by generated settlements. Source
    /// relief outside each private reservation and bound approach is untouched.
    /// These bounds do not certify actor clearance or historical construction.
    pub const fn bounded_settlement() -> Self {
        Self {
            limits: SETTLEMENT_LIMITS,
            stairs: SETTLEMENT_STAIRS,
            embedment: SETTLEMENT_EMBEDMENT,
            street_apron: STREET_APPROACH,
        }
    }
}

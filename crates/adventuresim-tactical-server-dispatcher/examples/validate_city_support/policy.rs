//! Explicit engineering bounds supplied by the fixed acceptance fixture.
use super::*;
pub(super) fn read(
    path: &std::path::Path,
) -> Result<CompoundGradingPolicy, Box<dyn std::error::Error>> {
    let fixture: Value = serde_json::from_slice(&std::fs::read(path)?)?;
    let chosen = &fixture["doorway_solution"];
    let number = |key| {
        chosen[key]
            .as_f64()
            .map(|n| n as f32)
            .ok_or("missing engineering policy bound")
    };
    let policy = CompoundGradingPolicy {
        limits: SupportLimits::new(
            number("maximum_grade")?,
            number("maximum_displacement_m")?,
            number("contact_tolerance_m")?,
        )
        .ok_or("invalid support bounds")?,
        stairs: CourtStairLimits::new(
            number("maximum_riser_m")?,
            number("minimum_going_m")?,
            number("clear_stair_width_m")?,
            number("endpoint_landing_run_m")?,
            number("court_landing_run_m")?,
        )
        .ok_or("invalid stair bounds")?,
        embedment: FoundationEmbedment::from_metres(number("foundation_embedment_m")?)
            .ok_or("invalid embedment")?,
        street_apron: StreetApronDimensions::from_metres(serde_json::from_value::<Vec2>(
            chosen["street_entry_apron"]["dimensions_metres"].clone(),
        )?)
        .ok_or("invalid apron bounds")?,
    };
    Ok(policy)
}

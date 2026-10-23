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
            adventuresim_tactical_core::city_layout::grounding::SupportGrade::from_ratio(number(
                "maximum_grade",
            )?)
            .ok_or("invalid support grade")?,
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                number("maximum_displacement_m")?,
            )?,
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                number("contact_tolerance_m")?,
            )?,
        ),
        stairs: CourtStairLimits::new(
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                number("maximum_riser_m")?,
            )?,
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                number("minimum_going_m")?,
            )?,
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                number("clear_stair_width_m")?,
            )?,
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                number("endpoint_landing_run_m")?,
            )?,
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                number("court_landing_run_m")?,
            )?,
        ),
        embedment: FoundationEmbedment::from_metres(number("foundation_embedment_m")?)
            .ok_or("invalid embedment")?,
        street_apron: StreetApronDimensions::from_metres(serde_json::from_value::<Vec2>(
            chosen["street_entry_apron"]["dimensions_metres"].clone(),
        )?)
        .ok_or("invalid apron bounds")?,
    };
    Ok(policy)
}

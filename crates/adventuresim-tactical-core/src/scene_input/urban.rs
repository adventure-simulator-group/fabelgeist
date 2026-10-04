//! Bounded urban descriptors and property membership.
use super::*;
use crate::city_layout::{MAX_CITY_STREET_PATCHES, MAX_CITY_YARD_PATCHES};
use crate::scene_input::SceneValidationError;
pub(super) fn validate(input: &TacticalSceneInput) -> Result<(), SceneInputError> {
    if input.streets.len() > MAX_CITY_STREET_PATCHES
        || input.streets.iter().any(|street| !street.is_valid())
    {
        return invalid(SceneValidationError::Streets);
    }
    if input.yards.len() > MAX_CITY_YARD_PATCHES || input.yards.iter().any(|yard| !yard.is_valid())
    {
        return invalid(SceneValidationError::Yards);
    }
    buildings::validate_building_placements(&input.buildings)?;
    buildings::validate_distant_building_placements(&input.distant_buildings)?;
    compounds::validate(input)?;
    gardens::validate(input)?;
    parishes::validate(input)?;
    Ok(())
}

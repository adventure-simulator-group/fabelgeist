//! Native vector and direction adapters for the canonical geometry contracts.

use bevy::math::InvalidDirectionError;
use fabelgeist_geometry::{
    CoordinateAxis, DirectionError, GeometryError, GeometryResult as Result, GeometryRole,
};

pub(super) fn finite(value: bevy::math::Vec3, role: GeometryRole) -> Result<()> {
    for (axis, component) in [CoordinateAxis::X, CoordinateAxis::Y, CoordinateAxis::Z]
        .into_iter()
        .zip(value.to_array())
    {
        if !component.is_finite() {
            return Err(GeometryError::NonFinite { role, axis });
        }
    }
    Ok(())
}

/// Translate Bevy's rejection once where native direction admission occurs.
pub(super) fn direction_error(source: InvalidDirectionError) -> GeometryError {
    let source = match source {
        InvalidDirectionError::Zero => DirectionError::Zero,
        InvalidDirectionError::Infinite => DirectionError::Infinite,
        InvalidDirectionError::NaN => DirectionError::NaN,
    };
    GeometryError::Direction { source }
}

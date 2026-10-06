use bevy::math::InvalidDirectionError;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoordinateAxis {
    X,
    Y,
    Z,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeometryRole {
    Position,
    Displacement,
    Elevation,
    CuboidDimensions,
    LeafDimensions,
    PlanDimensions,
    PlanExtents,
    Angle,
    BoundsCentre,
    BoundsExtent,
    Direction,
    SweepRadius,
    FloorWidth,
    FloorSpan,
    FloorStation,
    ClearanceVolume,
    PositiveLength,
}

#[derive(Debug, thiserror::Error, PartialEq, serde::Serialize)]
pub enum GeometryError {
    #[error("spatial index partition is empty")]
    EmptySpatialPartition,
    #[error("rotation must be finite and normalized")]
    InvalidRotation,
    #[error("invalid rigid frame projection")]
    InvalidProjection,
    #[error("{role:?} has a nonfinite {axis:?} component")]
    NonFinite {
        role: GeometryRole,
        axis: CoordinateAxis,
    },
    #[error("{role:?} has a negative {axis:?} extent")]
    NegativeExtent {
        role: GeometryRole,
        axis: CoordinateAxis,
    },
    #[error("{role:?} requires a positive {axis:?} extent")]
    ZeroExtent {
        role: GeometryRole,
        axis: CoordinateAxis,
    },
    #[error("bounds are reversed on {axis:?}")]
    ReversedBounds { axis: CoordinateAxis },
    #[error("invalid normalized direction: {source}")]
    Direction {
        #[source]
        #[serde(serialize_with = "serialize_direction_error")]
        source: InvalidDirectionError,
    },
    #[error("decoded direction is not normalized")]
    UnnormalizedDirection,
}

impl Clone for GeometryError {
    fn clone(&self) -> Self {
        match self {
            Self::EmptySpatialPartition => Self::EmptySpatialPartition,
            Self::InvalidRotation => Self::InvalidRotation,
            Self::InvalidProjection => Self::InvalidProjection,
            Self::NonFinite { role, axis } => Self::NonFinite {
                role: *role,
                axis: *axis,
            },
            Self::NegativeExtent { role, axis } => Self::NegativeExtent {
                role: *role,
                axis: *axis,
            },
            Self::ZeroExtent { role, axis } => Self::ZeroExtent {
                role: *role,
                axis: *axis,
            },
            Self::ReversedBounds { axis } => Self::ReversedBounds { axis: *axis },
            Self::UnnormalizedDirection => Self::UnnormalizedDirection,
            Self::Direction { source } => Self::Direction {
                source: match source {
                    InvalidDirectionError::Zero => InvalidDirectionError::Zero,
                    InvalidDirectionError::Infinite => InvalidDirectionError::Infinite,
                    InvalidDirectionError::NaN => InvalidDirectionError::NaN,
                },
            },
        }
    }
}
impl Eq for GeometryError {}

pub(super) fn finite(value: bevy::math::Vec3, role: GeometryRole) -> Result<(), GeometryError> {
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

fn serialize_direction_error<S: serde::Serializer>(
    source: &InvalidDirectionError,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(match source {
        InvalidDirectionError::Zero => "zero",
        InvalidDirectionError::Infinite => "infinite",
        InvalidDirectionError::NaN => "nan",
    })
}

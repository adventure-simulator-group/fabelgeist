//! Physical inventory containment failures.
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContainmentError {
    ZeroExteriorVolume,
    MeasuredVolumeExceedsExterior,
    DuplicateObject,
    UnknownContainedObject,
    VolumeOverflow,
    SelfContainment,
    UnknownDestination,
    NotAContainer,
    UnknownChild,
    WouldCycle,
    ExistingCycle,
    DepthExceeded,
    TraversalBoundExceeded,
    CapacityExceeded,
    UnknownObject,
}
impl fmt::Display for ContainmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ZeroExteriorVolume => {
                "Every physical object must have a positive exterior volume"
            }
            Self::MeasuredVolumeExceedsExterior => {
                "Measured volume exceeds the object's authored exterior volume"
            }
            Self::DuplicateObject => "Duplicate inventory object identity",
            Self::UnknownContainedObject => "Unknown contained object",
            Self::VolumeOverflow => "Container volume overflow",
            Self::SelfContainment => "A container cannot contain itself",
            Self::UnknownDestination => "Unknown destination container",
            Self::NotAContainer => "Destination is not a container",
            Self::UnknownChild => "Unknown child object",
            Self::WouldCycle => "Container nesting would create a cycle",
            Self::ExistingCycle => "Existing containment cycle",
            Self::DepthExceeded => "Container nesting exceeds the maximum depth",
            Self::TraversalBoundExceeded => "Containment traversal bound exceeded",
            Self::CapacityExceeded => "Container capacity exceeded",
            Self::UnknownObject => "Unknown inventory object",
        })
    }
}
impl std::error::Error for ContainmentError {}

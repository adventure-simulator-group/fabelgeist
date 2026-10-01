//! Design options the device builders do not construct yet. A design that
//! uses one is refused, never built as if the option were absent.

use crate::{GarmentArmorDesign, GarmentPlateShape, GenerateError, LimbArmorDesign};

/// Refuse the option `unsupported` names.
pub(crate) fn on_device(unsupported: Option<&'static str>) -> Result<(), GenerateError> {
    unsupported.map_or(Ok(()), |option| Err(GenerateError::NotOnDevice(option)))
}

impl LimbArmorDesign {
    /// The first option of this design the device cannot build yet.
    pub fn device_unsupported(&self) -> Option<&'static str> {
        match self {
            Self::Pauldron(_) => Some("a pauldron"),
            _ => None,
        }
    }
}

impl GarmentArmorDesign {
    /// The first option of this design the device cannot build yet.
    pub fn device_unsupported(&self) -> Option<&'static str> {
        matches!(self.plate_shape, GarmentPlateShape::WrappedTassets(_))
            .then_some("wrapped tassets")
    }
}

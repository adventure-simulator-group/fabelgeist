//! Design options the device builders do not construct yet. A design that
//! uses one is refused, never built as if the option were absent.

use crate::{
    BreastplateConstruction, BreastplateDesign, GarmentArmorDesign, GarmentPlateShape,
    GenerateError, HelmetDesign, LimbArmorDesign, Permille,
};

/// Refuse the option `unsupported` names.
pub(crate) fn on_device(unsupported: Option<&'static str>) -> Result<(), GenerateError> {
    unsupported.map_or(Ok(()), |option| Err(GenerateError::NotOnDevice(option)))
}

impl HelmetDesign {
    /// The first option of this design the device cannot build yet.
    pub fn device_unsupported(&self) -> Option<&'static str> {
        match self {
            Self::Burgonet(d) if d.buffe.is_some() => Some("a burgonet's buffe"),
            Self::Burgonet(d) if d.peak_rise.0 != 0 => Some("a burgonet's peak rise"),
            Self::CloseHelmet(d) if d.bellows.is_some() => Some("a bellows visor"),
            _ => None,
        }
    }
}

impl LimbArmorDesign {
    /// The first option of this design the device cannot build yet.
    pub fn device_unsupported(&self) -> Option<&'static str> {
        match self {
            Self::Pauldron(_) => Some("a pauldron"),
            Self::Spaulder(d) if d.besagew.is_some() => Some("a spaulder's besagew"),
            Self::Spaulder(d) if d.crown_coverage != Permille(1000) => {
                Some("a spaulder's crown coverage")
            }
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

impl BreastplateDesign {
    /// The first option of this design the device cannot build yet.
    pub fn device_unsupported(&self) -> Option<&'static str> {
        matches!(self.construction, BreastplateConstruction::Anime(_))
            .then_some("an anime breastplate")
    }
}

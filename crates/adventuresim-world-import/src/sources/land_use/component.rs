//! Closed roles for the HYDE input components.

pub(super) const COMPONENTS: [HydeComponent; 3] = [
    HydeComponent::Cropland,
    HydeComponent::Grazing,
    HydeComponent::Urban,
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum HydeComponent {
    Cropland,
    Grazing,
    Urban,
}

impl HydeComponent {
    pub(super) const fn filename(self) -> &'static str {
        match self {
            Self::Cropland => "cropland.nc",
            Self::Grazing => "grazing_land.nc",
            Self::Urban => "urban_area.nc",
        }
    }

    pub(super) const fn variable(self) -> &'static str {
        match self {
            Self::Cropland => "cropland",
            Self::Grazing => "grazing_land",
            Self::Urban => "urban_area",
        }
    }
}

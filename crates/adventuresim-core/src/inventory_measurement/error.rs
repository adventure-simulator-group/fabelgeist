//! Admission and arithmetic failures for measured inventory quantities.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeasurementError {
    ZeroCapacity,
    NonContainerHasTare,
    MeasuredRowIsNotSingleton,
    BulkLotMustBeMeasuredSingleton,
    MissingInstanceBasis,
    UnexpectedInstanceBasis,
    AmountExceedsCapacity,
    FractionExceedsWhole,
    InvalidFractionScale,
    InvalidPricingFactor,
    Overflow,
}

impl std::fmt::Display for MeasurementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::ZeroCapacity => "Measured inventory capacity must be nonzero",
            Self::NonContainerHasTare => "Only a container measurement may include tare",
            Self::MeasuredRowIsNotSingleton => {
                "Measured inventory state requires a quantity-one row"
            }
            Self::BulkLotMustBeMeasuredSingleton => {
                "A bulk material lot requires a measured quantity-one row"
            }
            Self::MissingInstanceBasis => "Measured inventory state has no instance basis",
            Self::UnexpectedInstanceBasis => {
                "Unmeasured inventory has an unexpected instance basis"
            }
            Self::AmountExceedsCapacity => "Measured inventory amount exceeds capacity",
            Self::FractionExceedsWhole => "A consumable row fraction cannot exceed one whole",
            Self::InvalidFractionScale => {
                "Consumable fraction scale must be finite and between zero and one"
            }
            Self::InvalidPricingFactor => "Inventory pricing factor must be finite and nonnegative",
            Self::Overflow => "Inventory measurement arithmetic overflow",
        })
    }
}

impl std::error::Error for MeasurementError {}

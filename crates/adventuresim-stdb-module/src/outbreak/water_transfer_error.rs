//! Admission and conservation failures of private water contribution transfer.

use adventuresim_core::{
    material::{MaterialError, Microliters},
    physical_object::PhysicalObjectId,
};

#[derive(Debug)]
pub(crate) enum WaterContributionTransferError {
    ExceedsPublicVolume {
        container: PhysicalObjectId,
        source_total: Microliters,
        moved: Microliters,
    },
    InvalidMaterialLot {
        stored_lot_id: u64,
        source: MaterialError,
    },
}

impl std::fmt::Display for WaterContributionTransferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExceedsPublicVolume { .. } => {
                f.write_str("Water material transfer exceeds public volume")
            }
            Self::InvalidMaterialLot { source, .. } => source.fmt(f),
        }
    }
}

impl std::error::Error for WaterContributionTransferError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidMaterialLot { source, .. } => Some(source),
            Self::ExceedsPublicVolume { .. } => None,
        }
    }
}

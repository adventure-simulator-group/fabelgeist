use super::layout::{
    CorrectiveBasisDimensions, CorrectiveDimension, CorrectiveNetworkDimensions,
    CorrectiveRigTopology, RejectedCorrectiveShape,
};
use super::sparse::{SparseActivationCoordinate, SparseActivationLayout, SparseCoordinateAxis};
use super::{CorrectiveArchiveRole, CorrectiveArrayRole};
use fabelgeist_numpy_storage::{NpzArrayError, ZipReadError};

#[derive(Debug)]
pub enum CorrectiveDecodeError {
    Archive {
        role: CorrectiveArchiveRole,
        source: Box<ZipReadError>,
    },
    Array {
        role: CorrectiveArrayRole,
        source: Box<NpzArrayError>,
    },
    BasisShape(RejectedCorrectiveShape),
    BasisVertices {
        basis: CorrectiveBasisDimensions,
        rig: CorrectiveRigTopology,
    },
    SkeletonWithoutPose(CorrectiveRigTopology),
    DimensionOverflow(CorrectiveDimension),
    BasisComponents {
        basis: CorrectiveBasisDimensions,
        network: CorrectiveNetworkDimensions,
    },
    SparseLayout(SparseActivationLayout),
    SparseCoordinateEncoding {
        coordinate: SparseActivationCoordinate,
        axis: SparseCoordinateAxis,
        source: std::num::TryFromIntError,
    },
    SparseCoordinate {
        coordinate: SparseActivationCoordinate,
        network: CorrectiveNetworkDimensions,
    },
}
impl std::fmt::Display for CorrectiveDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Archive { role, source } => write!(f, "{role:?} corrective archive: {source}"),
            Self::Array { role, source } => write!(f, "{role:?} corrective array: {source}"),
            Self::BasisShape(shape) => write!(
                f,
                "corrective basis has shape {shape}, expected [components, vertices, 3]"
            ),
            Self::BasisVertices { basis, rig } => write!(
                f,
                "corrective basis vertex mismatch: {basis}; rig has {rig}"
            ),
            Self::SkeletonWithoutPose(rig) => write!(
                f,
                "corrective rig must include the two skipped joints: {rig}"
            ),
            Self::DimensionOverflow(dimension) => {
                write!(f, "corrective {dimension:?} layout overflows storage")
            }
            Self::BasisComponents { basis, network } => write!(
                f,
                "corrective basis component mismatch: {basis}; network has {network}"
            ),
            Self::SparseLayout(layout) => write!(
                f,
                "sparse activation needs two coordinates per weight: {layout}"
            ),
            Self::SparseCoordinateEncoding {
                coordinate,
                axis,
                source,
            } => {
                write!(
                    f,
                    "sparse activation {coordinate} has an invalid {axis:?}: {source}"
                )
            }
            Self::SparseCoordinate {
                coordinate,
                network,
            } => write!(f, "sparse activation {coordinate} is outside {network}"),
        }
    }
}
impl std::error::Error for CorrectiveDecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Archive { source, .. } => Some(source.as_ref()),
            Self::Array { source, .. } => Some(source.as_ref()),
            Self::SparseCoordinateEncoding { source, .. } => Some(source),
            _ => None,
        }
    }
}

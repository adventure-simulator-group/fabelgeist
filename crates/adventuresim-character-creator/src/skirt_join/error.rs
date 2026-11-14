//! Stable sewing failures, retaining causes from mesh and cut operations.
use fabelgeist_rig::RigJointLookupError;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum SkirtJoinError {
    #[error("dress has no upper section")]
    MissingUpperSection,
    #[error("dress upper opening must be above its waist")]
    OpeningBelowWaist,
    #[error("dress upper patch must have a waist and upper opening")]
    InvalidOpeningCount,
    #[error("sewn upper has no waist around the pelvis axis")]
    MissingWaist,
    #[error("dress waist is empty")]
    EmptyWaist,
    #[error("dress waist is not star-shaped around pelvis")]
    NonStarShapedWaist,
    #[error(transparent)]
    JointLookup(#[from] RigJointLookupError),
    #[error(transparent)]
    Design(#[from] fabelgeist_armor::GenerateError),
    #[error("dress source point count exceeds the native layout")]
    PointCount(#[from] std::num::TryFromIntError),
    #[error(transparent)]
    MaterialChart(#[from] crate::gown_charts::SewnChartError),
    #[error("cutting the sewn upper surface failed")]
    SurfaceCut(#[source] anyhow::Error),
    #[error("partitioning the sewn materials failed")]
    MaterialPartition(#[source] anyhow::Error),
    #[error("normalizing the sewn material charts failed")]
    ChartNormalization(#[source] anyhow::Error),
}

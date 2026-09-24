//! Stable terminal-shoot identity, independent of branch traversal.
use super::*;

pub(in crate::presentation) fn shoot_identity(shoot: &TreeBranchSegment) -> u64 {
    streams::SHOOT_IDENTITY
        .seed(
            0,
            &[
                u64::from(shoot.primary_group),
                u64::from(shoot.secondary_group),
                u64::from(shoot.start.x.to_bits()),
                u64::from(shoot.start.y.to_bits()),
                u64::from(shoot.start.z.to_bits()),
                u64::from(shoot.end.x.to_bits()),
                u64::from(shoot.end.y.to_bits()),
                u64::from(shoot.end.z.to_bits()),
            ],
        )
        .to_u64()
}

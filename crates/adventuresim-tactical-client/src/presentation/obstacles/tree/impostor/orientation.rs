//! Aggregate crown and macro-cluster orientation.
use super::*;

pub(super) fn subcluster_phase(seed: u64, key: u16) -> f32 {
    fabelgeist_determinism::StreamId::new("visual.obstacles.tree.impostor.subcluster-phase")
        .rng(seed, &[u64::from(key)])
        .inclusive_unit_f32()
        * core::f32::consts::FRAC_PI_2
}

pub(super) fn crown_group_right(branches: &[TreeBranchSegment], group: u8) -> Vec3 {
    branches
        .iter()
        .filter(|branch| branch.depth == 1 && branch.primary_group == group)
        .max_by(|left, right| {
            left.end
                .xz()
                .length_squared()
                .total_cmp(&right.end.xz().length_squared())
        })
        .map(|branch| {
            let horizontal = branch.end.xz().normalize_or_zero();
            if horizontal.length_squared() > 0.25 {
                Vec3::new(horizontal.x, 0.0, horizontal.y)
            } else {
                Vec3::X
            }
        })
        .unwrap_or(Vec3::X)
}

pub(super) fn lod1_macro_cluster_axis(
    branches: &[TreeBranchSegment],
    primary_group: u8,
    secondary_group_range: (u16, u16),
) -> Vec3 {
    let (first, last) = secondary_group_range;
    let axis = branches
        .iter()
        .filter(|branch| {
            branch.depth == 2
                && branch.primary_group == primary_group
                && (first..=last).contains(&branch.secondary_group)
        })
        .map(|branch| {
            let direction = branch.end - branch.start;
            direction.normalize_or_zero() * direction.length()
        })
        .sum::<Vec3>();
    if axis.length_squared() > 0.01 {
        axis.normalize()
    } else {
        crown_group_right(branches, primary_group)
    }
}

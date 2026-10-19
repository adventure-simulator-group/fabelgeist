//! Exact collider-centred numerical source records, without Rust memory layout.
use super::super::{TreeBranchSegment, TreeLeaf, impostor_assets::bounded};
use anyhow::{Result, ensure};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub(super) const MAX_SOURCE_BRANCHES: usize = 32_768;
pub(super) const MAX_SOURCE_LEAVES: usize = 131_072;

#[derive(Serialize, Deserialize)]
pub(super) struct Branch {
    start: [f32; 3],
    end: [f32; 3],
    start_radius: f32,
    end_radius: f32,
    depth: u8,
    primary_group: u8,
    secondary_group: u16,
    is_limb_tip: bool,
}
#[derive(Serialize, Deserialize)]
pub(super) struct Leaf {
    petiole_start: [f32; 3],
    center: [f32; 3],
    right: [f32; 3],
    up: [f32; 3],
    length: f32,
    width: f32,
    primary_group: u8,
    secondary_group: u16,
    shoot_id: u64,
    leaf_ordinal: u8,
    shade: f32,
    torsion: f32,
}
#[derive(Serialize, Deserialize)]
pub(super) struct Source {
    #[serde(deserialize_with = "bounded::sequence::<_, _, MAX_SOURCE_BRANCHES>")]
    branches: Vec<Branch>,
    #[serde(deserialize_with = "bounded::sequence::<_, _, MAX_SOURCE_LEAVES>")]
    leaves: Vec<Leaf>,
}
impl Source {
    pub(super) fn from_records(branches: &[TreeBranchSegment], leaves: &[TreeLeaf]) -> Self {
        Self {
            branches: branches.iter().map(Branch::from).collect(),
            leaves: leaves.iter().map(Leaf::from).collect(),
        }
    }
    pub(super) fn validate(&self) -> Result<()> {
        ensure!(
            !self.branches.is_empty()
                && self.branches.len() <= MAX_SOURCE_BRANCHES
                && self.leaves.len() <= MAX_SOURCE_LEAVES,
            "source record limits"
        );
        for branch in &self.branches {
            ensure!(
                branch
                    .start
                    .iter()
                    .chain(&branch.end)
                    .chain([&branch.start_radius, &branch.end_radius])
                    .all(|value| value.is_finite()),
                "nonfinite source branch"
            );
            ensure!(
                branch.start_radius > 0.0
                    && branch.end_radius >= 0.0
                    && branch.start != branch.end
                    && (branch.primary_group < super::super::TREE_PRIMARY_GROUP_COUNT
                        || (branch.depth == 0
                            && branch.primary_group == u8::MAX
                            && branch.secondary_group == u16::MAX)),
                "source branch shape/group"
            );
        }
        for leaf in &self.leaves {
            ensure!(
                leaf.petiole_start
                    .iter()
                    .chain(&leaf.center)
                    .chain(&leaf.right)
                    .chain(&leaf.up)
                    .chain([&leaf.length, &leaf.width, &leaf.shade, &leaf.torsion])
                    .all(|value| value.is_finite()),
                "nonfinite source leaf"
            );
            ensure!(
                leaf.length > 0.0
                    && leaf.width > 0.0
                    && leaf.primary_group < super::super::TREE_PRIMARY_GROUP_COUNT
                    && leaf.shade >= 0.0,
                "source leaf shape/group"
            );
        }
        Ok(())
    }
    pub(super) fn maximum_storage_bytes() -> usize {
        MAX_SOURCE_BRANCHES * std::mem::size_of::<Branch>()
            + MAX_SOURCE_LEAVES * std::mem::size_of::<Leaf>()
    }

    pub(super) fn runtime(self) -> (Vec<TreeBranchSegment>, Vec<TreeLeaf>) {
        (
            self.branches
                .into_iter()
                .map(TreeBranchSegment::from)
                .collect(),
            self.leaves.into_iter().map(TreeLeaf::from).collect(),
        )
    }
}
impl From<&TreeBranchSegment> for Branch {
    fn from(value: &TreeBranchSegment) -> Self {
        Self {
            start: value.start.to_array(),
            end: value.end.to_array(),
            start_radius: value.start_radius,
            end_radius: value.end_radius,
            depth: value.depth,
            primary_group: value.primary_group,
            secondary_group: value.secondary_group,
            is_limb_tip: value.is_limb_tip,
        }
    }
}
impl From<Branch> for TreeBranchSegment {
    fn from(value: Branch) -> Self {
        Self {
            start: Vec3::from_array(value.start),
            end: Vec3::from_array(value.end),
            start_radius: value.start_radius,
            end_radius: value.end_radius,
            depth: value.depth,
            primary_group: value.primary_group,
            secondary_group: value.secondary_group,
            is_limb_tip: value.is_limb_tip,
        }
    }
}
impl From<&TreeLeaf> for Leaf {
    fn from(value: &TreeLeaf) -> Self {
        Self {
            petiole_start: value.petiole_start.to_array(),
            center: value.center.to_array(),
            right: value.right.to_array(),
            up: value.up.to_array(),
            length: value.length,
            width: value.width,
            primary_group: value.primary_group,
            secondary_group: value.secondary_group,
            shoot_id: value.shoot_id,
            leaf_ordinal: value.leaf_ordinal,
            shade: value.shade,
            torsion: value.torsion,
        }
    }
}
impl From<Leaf> for TreeLeaf {
    fn from(value: Leaf) -> Self {
        Self {
            petiole_start: Vec3::from_array(value.petiole_start),
            center: Vec3::from_array(value.center),
            right: Vec3::from_array(value.right),
            up: Vec3::from_array(value.up),
            length: value.length,
            width: value.width,
            primary_group: value.primary_group,
            secondary_group: value.secondary_group,
            shoot_id: value.shoot_id,
            leaf_ordinal: value.leaf_ordinal,
            shade: value.shade,
            torsion: value.torsion,
        }
    }
}

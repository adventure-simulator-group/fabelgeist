//! Offline matching uses botanical shape, independently of conservative cull boxes.

use bevy::math::Vec3Swizzles;
use bevy::prelude::Vec3;
use fabelgeist_botany::{
    identity::PlantId,
    viewer::sim::{BranchInstance, LeafInstance},
};
use fabelgeist_determinism::{DeterministicRng, Seed};
use fabelgeist_math::units::{Metres, Ratio};

use super::{Result, TreeYaw};

/// Degenerate seedlings still need finite logarithms and scale denominators.
const MINIMUM_EXTENT: f32 = 0.01;
/// Complexity differences influence selection without overwhelming dimensions.
const COMPLEXITY_WEIGHT: f32 = 0.5;
/// Crown occupancy is expressed as fractions of the full tree height.
const CROWN_WEIGHT: f32 = 2.0;
/// Preserve an asymmetric crown's direction; symmetric crowns admit free yaw.
const DIRECTIONAL_CROWN_FRACTION: f32 = 0.15;

#[derive(Clone, Copy)]
pub(super) struct TreeForm {
    pub height: Metres,
    pub width: Metres,
    branching: Ratio,
    foliage: Ratio,
    crown_base: Ratio,
    crown_center: Ratio,
    heading: Option<TreeYaw>,
}

impl TreeForm {
    /// GPU records are a native metre-space boundary. This numerical reduction
    /// measures shape, not shader bounds: inflated cull margins must not decide
    /// how a replacement crown is scaled or which sparse/dense form it uses.
    pub fn measure<'a>(
        root: [Metres; 3],
        branches: impl Iterator<Item = &'a BranchInstance>,
        leaves: impl Iterator<Item = &'a LeafInstance>,
    ) -> Self {
        let root = Vec3::from_array(root.map(Metres::metres));
        let mut height = MINIMUM_EXTENT;
        let mut radius = MINIMUM_EXTENT;
        let mut branch_count = 0_usize;
        for branch in branches {
            branch_count += 1;
            for end in [branch.a, branch.b] {
                let point = Vec3::from_slice(&end[..3]) - root;
                height = height.max(point.y + end[3]);
                radius = radius.max(point.xz().length() + end[3]);
            }
        }
        let mut leaf_count = 0_usize;
        let mut crown_base = f32::INFINITY;
        let mut crown_center = Vec3::ZERO;
        for leaf in leaves {
            leaf_count += 1;
            let point = Vec3::from_array(leaf.center) - root;
            height = height.max(point.y + leaf.size);
            radius = radius.max(point.xz().length() + leaf.size);
            crown_base = crown_base.min(point.y - leaf.size);
            crown_center += point;
        }
        if leaf_count > 0 {
            crown_center /= leaf_count as f32;
        } else {
            crown_base = 0.0;
        }
        let heading = (crown_center.xz().length() > radius * DIRECTIONAL_CROWN_FRACTION)
            .then_some(TreeYaw(crown_center.x.atan2(crown_center.z)));
        Self {
            height: Metres::from_metres(height),
            width: Metres::from_metres(radius * 2.0),
            branching: Ratio::from_ratio((branch_count as f32).ln_1p()),
            foliage: Ratio::from_ratio((leaf_count as f32).ln_1p()),
            crown_base: Ratio::from_ratio(crown_base / height),
            crown_center: Ratio::from_ratio(crown_center.y / height),
            heading,
        }
    }

    pub fn distance(self, other: Self) -> f32 {
        // Dimensionless comparison kernel; counts use log space so seedlings
        // and mature trees receive similar relative-complexity sensitivity.
        let dimensions = (self.height.metres().ln() - other.height.metres().ln()).powi(2)
            + (self.width.metres().ln() - other.width.metres().ln()).powi(2);
        let complexity = (self.branching.ratio() - other.branching.ratio()).powi(2)
            + (self.foliage.ratio() - other.foliage.ratio()).powi(2);
        let crown = (self.crown_base.ratio() - other.crown_base.ratio()).powi(2)
            + (self.crown_center.ratio() - other.crown_center.ratio()).powi(2);
        dimensions + COMPLEXITY_WEIGHT * complexity + CROWN_WEIGHT * crown
    }

    pub fn yaw_from(self, model: Self, identity: PlantId) -> Result<TreeYaw> {
        if let (Some(source), Some(model)) = (self.heading, model.heading) {
            return TreeYaw::from_radians(source.radians() - model.radians());
        }
        let mut random = DeterministicRng::new(Seed::from_u64(identity.word()));
        TreeYaw::from_radians(random.unit_f32() * std::f32::consts::TAU)
    }
}

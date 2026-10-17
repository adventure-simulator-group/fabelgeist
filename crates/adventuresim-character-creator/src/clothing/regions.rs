//! Anatomical garment regions mapped to the body skeleton.
use super::Region;
use fabelgeist_rig::{RigJointMembership, RigJointName, RigJointOrdinal};
use std::collections::HashSet;

pub(super) struct RegionRig {
    pub(super) proximal: RigJointName,
    pub(super) distal: RigJointName,
    region: Region,
}

pub(super) fn region_rig(region: Region) -> RegionRig {
    let (proximal, distal) = match region {
        Region::Head => (RigJointName::C_NECK, RigJointName::C_HEAD),
        Region::Neck => (RigJointName::C_SPINE3, RigJointName::C_NECK),
        Region::Chest => (RigJointName::C_SPINE2, RigJointName::C_NECK),
        Region::LeftAxilla => (RigJointName::L_CLAVICLE, RigJointName::L_UPARM),
        Region::RightAxilla => (RigJointName::R_CLAVICLE, RigJointName::R_UPARM),
        Region::Stomach => (RigJointName::C_SPINE0, RigJointName::C_SPINE2),
        Region::Groin => (RigJointName::C_SPINE1, RigJointName::C_SPINE0),
        Region::LeftUpperArm => (RigJointName::L_UPARM, RigJointName::L_LOWARM),
        Region::LeftForearm => (RigJointName::L_LOWARM, RigJointName::L_WRIST),
        Region::RightUpperArm => (RigJointName::R_UPARM, RigJointName::R_LOWARM),
        Region::RightForearm => (RigJointName::R_LOWARM, RigJointName::R_WRIST),
        Region::LeftThigh => (RigJointName::L_UPLEG, RigJointName::L_LOWLEG),
        Region::LeftLowerLeg => (RigJointName::L_LOWLEG, RigJointName::L_FOOT),
        Region::RightThigh => (RigJointName::R_UPLEG, RigJointName::R_LOWLEG),
        Region::RightLowerLeg => (RigJointName::R_LOWLEG, RigJointName::R_FOOT),
    };
    RegionRig {
        proximal,
        distal,
        region,
    }
}

impl RegionRig {
    pub(super) fn skin_joints(rigs: &[Self], names: &[RigJointName]) -> HashSet<RigJointOrdinal> {
        names
            .iter()
            .enumerate()
            .filter_map(
                |(slot, name): (usize, &RigJointName)| -> Option<RigJointOrdinal> {
                    rigs.iter()
                        .any(|rig: &Self| -> bool {
                            rig.membership(name) == RigJointMembership::Included
                        })
                        .then_some(RigJointOrdinal::from(slot))
                },
            )
            .collect()
    }

    pub(super) fn membership(&self, name: &RigJointName) -> RigJointMembership {
        let included = match self.region {
            Region::Head => {
                [
                    RigJointName::C_HEAD,
                    RigJointName::C_JAW,
                    RigJointName::L_EYE,
                    RigJointName::R_EYE,
                ]
                .contains(name)
                    || name.family_prefix(&RigJointName::C_TONGUE) == RigJointMembership::Included
            }
            Region::Neck => name.skin_family(&RigJointName::C_NECK) == RigJointMembership::Included,
            Region::Chest => [
                RigJointName::C_SPINE2,
                RigJointName::C_SPINE3,
                RigJointName::L_CLAVICLE,
                RigJointName::R_CLAVICLE,
            ]
            .contains(name),
            Region::LeftAxilla => name == &RigJointName::L_CLAVICLE,
            Region::RightAxilla => name == &RigJointName::R_CLAVICLE,
            Region::Stomach => [RigJointName::C_SPINE0, RigJointName::C_SPINE1].contains(name),
            Region::Groin => [
                RigJointName::C_SPINE0,
                RigJointName::L_UPLEG,
                RigJointName::R_UPLEG,
            ]
            .contains(name),
            Region::LeftUpperArm
            | Region::LeftForearm
            | Region::RightUpperArm
            | Region::RightForearm
            | Region::LeftThigh
            | Region::LeftLowerLeg
            | Region::RightThigh
            | Region::RightLowerLeg => {
                name.skin_family(&self.proximal) == RigJointMembership::Included
            }
        };
        RigJointMembership::from(included)
    }
}

/// Waist support includes thigh twists, but deliberately excludes spine twists.
/// Socket export and cloth fitting share this exact skin-selection policy.
pub(crate) fn waist_surface_joint(name: &RigJointName) -> RigJointMembership {
    RigJointMembership::from(
        [
            RigJointName::ROOT,
            RigJointName::C_SPINE0,
            RigJointName::C_SPINE1,
        ]
        .contains(name)
            || name.skin_family(&RigJointName::L_UPLEG) == RigJointMembership::Included
            || name.skin_family(&RigJointName::R_UPLEG) == RigJointMembership::Included,
    )
}

/// Ordered-rig slots admitted into the waist support selection.
pub(super) fn waist_joints(names: &[RigJointName]) -> HashSet<RigJointOrdinal> {
    names
        .iter()
        .enumerate()
        .filter_map(
            |(slot, name): (usize, &RigJointName)| -> Option<RigJointOrdinal> {
                (waist_surface_joint(name) == RigJointMembership::Included)
                    .then_some(RigJointOrdinal::from(slot))
            },
        )
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skin_queries_preserve_head_prefix_axilla_exactness_and_waist_twist_exclusions() {
        assert_eq!(
            region_rig(Region::Head).membership(&RigJointName::from("c_tongue_helper")),
            RigJointMembership::Included
        );
        assert_eq!(
            region_rig(Region::LeftAxilla).membership(&RigJointName::from("l_clavicle_twist")),
            RigJointMembership::Excluded
        );
        assert_eq!(
            region_rig(Region::LeftUpperArm).membership(&RigJointName::from("l_uparm_twisted")),
            RigJointMembership::Included
        );
        assert_eq!(
            waist_surface_joint(&RigJointName::from("l_upleg_twist2_proc")),
            RigJointMembership::Included
        );
        assert_eq!(
            waist_surface_joint(&RigJointName::from("c_spine0_twist")),
            RigJointMembership::Excluded
        );
    }
}

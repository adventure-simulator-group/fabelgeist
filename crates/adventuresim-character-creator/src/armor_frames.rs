//! The wearer armor is fitted to: its body, rig and anatomical regions.

use anyhow::Result;
use fabelgeist_rig::{RigJointMembership, RigJointName, RigJointPart, RigSide};

#[derive(Clone, Copy, Debug)]
pub enum Side {
    Left,
    Right,
}

impl Side {
    pub fn from_placement(id: &str) -> Result<Self> {
        match id {
            "left" => Ok(Self::Left),
            "right" => Ok(Self::Right),
            _ => anyhow::bail!("paired armor requires a left or right placement"),
        }
    }
    /// Which way this side's local x points: +1 on the left, -1 on the right.
    pub(crate) fn sign(self) -> f32 {
        match self {
            Self::Left => 1.0,
            Self::Right => -1.0,
        }
    }

    /// Construct the anatomical label on this equipment side.
    pub(crate) fn joint(self, part: RigJointPart) -> RigJointName {
        RigJointName::sided(
            match self {
                Self::Left => RigSide::Left,
                Self::Right => RigSide::Right,
            },
            part,
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub enum FitRegion {
    Head,
    Neck,
    Torso,
    Hips,
    UpperArm(Side),
    Forearm(Side),
    WholeArm(Side),
    Shoulder(Side),
    Elbow(Side),
    Thigh(Side),
    LowerLeg(Side),
    WholeLeg(Side),
    Knee(Side),
    Hand(Side),
    Foot(Side),
}

impl FitRegion {
    /// The joints whose skin makes up the region: any of these, or their
    /// twist joints, owning enough of a vertex's weight puts it in the region.
    pub fn owners(self) -> Vec<RigJointName> {
        use FitRegion::*;
        let central = |names: &[RigJointName]| -> Vec<RigJointName> { names.to_vec() };
        let sided = |side: Side, names: &[RigJointPart]| -> Vec<RigJointName> {
            names
                .iter()
                .map(|part: &RigJointPart| -> RigJointName { side.joint(*part) })
                .collect()
        };
        match self {
            Head => central(&[RigJointName::C_HEAD, RigJointName::C_JAW]),
            Neck => central(&[RigJointName::C_NECK, RigJointName::C_SPINE3]),
            Torso => central(&[
                RigJointName::C_SPINE0,
                RigJointName::C_SPINE1,
                RigJointName::C_SPINE2,
                RigJointName::C_SPINE3,
                RigJointName::L_CLAVICLE,
                RigJointName::R_CLAVICLE,
            ]),
            Hips => central(&[
                RigJointName::ROOT,
                RigJointName::C_SPINE0,
                RigJointName::C_SPINE1,
                RigJointName::L_UPLEG,
                RigJointName::R_UPLEG,
            ]),
            UpperArm(s) | Shoulder(s) => sided(s, &[RigJointPart::Uparm]),
            Forearm(s) => sided(s, &[RigJointPart::Lowarm]),
            WholeArm(s) | Elbow(s) => sided(s, &[RigJointPart::Uparm, RigJointPart::Lowarm]),
            Thigh(s) => sided(s, &[RigJointPart::Upleg]),
            LowerLeg(s) | Knee(s) => sided(s, &[RigJointPart::Lowleg]),
            WholeLeg(s) => sided(s, &[RigJointPart::Upleg, RigJointPart::Lowleg]),
            Hand(s) => sided(s, HAND_SKIN_JOINTS),
            Foot(s) => sided(
                s,
                &[
                    RigJointPart::Foot,
                    RigJointPart::Talocrural,
                    RigJointPart::Subtalar,
                    RigJointPart::Transversetarsal,
                    RigJointPart::Ball,
                ],
            ),
        }
    }

    /// The joints whose skin supports a part fitted to the region. The
    /// shoulder cap spans the deltoid and the clavicular transition: its
    /// frame stays anchored by the upper arm, while its support also takes the
    /// proximal surface that does not shorten with that bone.
    pub fn support_owners(self) -> Vec<RigJointName> {
        let mut owners = self.owners();
        if let FitRegion::Shoulder(side) = self {
            owners.push(side.joint(RigJointPart::Clavicle));
        }
        owners
    }
}

pub struct Wearer<'a> {
    pub faces: &'a [[u32; 3]],
    pub positions: &'a [[f32; 3]],
    pub normals: &'a [[f32; 3]],
    pub joint_indices: &'a [[u32; 8]],
    pub joint_weights: &'a [[f32; 8]],
    pub joint_names: &'a [RigJointName],
    pub joints: &'a [[f32; 8]],
}

/// Skin supports a region when at least this much of its weight belongs to
/// the region's joints.
pub(crate) const SKIN_SUPPORT_THRESHOLD: f32 = 0.30;
/// [`SKIN_SUPPORT_THRESHOLD`] as a WGSL constant of the same name.
pub(crate) fn skin_support_wgsl() -> String {
    format!("const SKIN_SUPPORT_THRESHOLD: f32 = {SKIN_SUPPORT_THRESHOLD:?};\n")
}

/// A frame's half extents never fall below this, however thin its skin.
pub(crate) const MINIMUM_REGION_RADIUS_M: f32 = 0.018;
// The glove body covers every finger segment, including terminal skin weights.
// The thumb has its own frame and remains outside this envelope.
pub(crate) const HAND_SKIN_JOINTS: &[RigJointPart] = &[
    RigJointPart::Wrist,
    RigJointPart::Index1,
    RigJointPart::Index2,
    RigJointPart::Index3,
    RigJointPart::IndexNull,
    RigJointPart::Middle1,
    RigJointPart::Middle2,
    RigJointPart::Middle3,
    RigJointPart::MiddleNull,
    RigJointPart::Ring1,
    RigJointPart::Ring2,
    RigJointPart::Ring3,
    RigJointPart::RingNull,
    RigJointPart::Pinky0,
    RigJointPart::Pinky1,
    RigJointPart::Pinky2,
    RigJointPart::Pinky3,
    RigJointPart::PinkyNull,
];

impl Wearer<'_> {
    /// The body vertices whose skin belongs mostly to `region`'s joints.
    pub fn support_indices(&self, region: FitRegion) -> Result<Vec<usize>> {
        let owners = region.support_owners();
        let owned = self
            .joint_names
            .iter()
            .map(|name: &RigJointName| -> RigJointMembership {
                RigJointMembership::from(owners.iter().any(|owner: &RigJointName| -> bool {
                    name.skin_family(owner) == RigJointMembership::Included
                }))
            })
            .collect::<Vec<_>>();
        Ok(self
            .joint_indices
            .iter()
            .zip(self.joint_weights)
            .enumerate()
            .filter_map(|(i, (indices, weights))| {
                let weight: f32 = indices
                    .iter()
                    .zip(weights)
                    .filter(|(j, _)| owned[**j as usize] == RigJointMembership::Included)
                    .map(|(_, w)| w)
                    .sum();
                (weight >= SKIN_SUPPORT_THRESHOLD).then_some(i)
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::{FitRegion, Side, Wearer};
    use fabelgeist_rig::RigJointName;

    #[test]
    fn shoulder_support_unites_clavicle_and_arm_without_crossing_body_regions() {
        let names = [
            RigJointName::L_UPARM,
            RigJointName::L_LOWARM,
            RigJointName::L_UPARM_TWIST0_PROC,
            RigJointName::L_CLAVICLE,
            RigJointName::R_CLAVICLE,
            RigJointName::C_SPINE3,
        ];
        let positions = [[0.0; 3]; 6];
        let joints = [[0.0; 8]; 6];
        let indices = [
            [2; 8],
            [3; 8],
            [4; 8],
            [5; 8],
            [2, 3, 5, 5, 5, 5, 5, 5],
            [2, 3, 5, 5, 5, 5, 5, 5],
        ];
        let mut weights = [[0.125; 8]; 6];
        weights[4] = [0.2, 0.2, 0.6, 0.0, 0.0, 0.0, 0.0, 0.0];
        weights[5] = [0.1, 0.1, 0.8, 0.0, 0.0, 0.0, 0.0, 0.0];
        let wearer = Wearer {
            faces: &[],
            positions: &positions,
            normals: &positions,
            joint_indices: &indices,
            joint_weights: &weights,
            joint_names: &names,
            joints: &joints,
        };
        assert_eq!(
            wearer
                .support_indices(FitRegion::Shoulder(Side::Left))
                .unwrap(),
            [0, 1, 4]
        );
        assert_eq!(
            wearer
                .support_indices(FitRegion::UpperArm(Side::Left))
                .unwrap(),
            [0]
        );
    }

    #[test]
    fn hand_envelope_includes_skin_owned_by_finger_terminal_joints() {
        let names = [
            RigJointName::L_WRIST,
            RigJointName::L_MIDDLE_NULL,
            RigJointName::L_PINKY_NULL,
            RigJointName::L_FOOT,
        ];
        let positions = [[0.0; 3]; 3];
        let indices = [[1; 8], [2; 8], [3; 8]];
        let weights = [[0.125; 8]; 3];
        let joints = [[0.0; 8]; 4];
        let wearer = Wearer {
            faces: &[],
            positions: &positions,
            normals: &positions,
            joint_indices: &indices,
            joint_weights: &weights,
            joint_names: &names,
            joints: &joints,
        };
        assert_eq!(
            wearer.support_indices(FitRegion::Hand(Side::Left)).unwrap(),
            [0, 1]
        );
    }
}

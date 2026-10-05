//! Which rig joints orient and size each region's frame.

use fabelgeist_rig::{
    RigJointLookupError, RigJointMembership, RigJointName, RigJointOrdinal, RigJointPart,
};

use super::Side;
use crate::armor_frames::{FitRegion, Wearer};

/// How a region's frame is oriented and finished.
#[derive(Clone, Copy)]
pub(super) enum Rule {
    Limb = 0,
    Shoulder = 1,
    Foot = 2,
    Hand = 3,
    Head = 4,
    Elbow = 5,
    Thumb = 6,
    Knee = 7,
}

/// Landmark slots the orientation kernel reads, by rule.
#[derive(Clone, Copy)]
pub(super) enum FrameLandmark {
    Joint(RigJointOrdinal),
    Unused,
}
impl From<RigJointOrdinal> for FrameLandmark {
    fn from(joint: RigJointOrdinal) -> Self {
        Self::Joint(joint)
    }
}
impl FrameLandmark {
    /// The fixed six-word orientation-kernel layout; unused slots encode zero.
    pub(super) fn device_word(self) -> u32 {
        match self {
            Self::Joint(joint) => usize::from(joint) as u32,
            Self::Unused => 0,
        }
    }
}

pub(super) struct Landmarks {
    pub rule: Rule,
    /// Joint indices; meaning depends on the rule.
    pub joints: [FrameLandmark; 6],
    pub side: f32,
    pub owners: Vec<RigJointName>,
}

impl Wearer<'_> {
    fn joint_index(
        &self,
        name: &RigJointName,
    ) -> std::result::Result<RigJointOrdinal, RigJointLookupError> {
        name.require_in(self.joint_names)
    }

    /// Which joints own skin for a set of owner names.
    pub(crate) fn owned_joints(&self, owners: &[RigJointName]) -> Vec<RigJointMembership> {
        self.joint_names
            .iter()
            .map(|name: &RigJointName| -> RigJointMembership {
                RigJointMembership::from(owners.iter().any(|owner: &RigJointName| -> bool {
                    name.skin_family(owner) == RigJointMembership::Included
                }))
            })
            .collect()
    }

    pub(super) fn device_landmarks(
        &self,
        region: FitRegion,
    ) -> std::result::Result<Landmarks, RigJointLookupError> {
        use FitRegion::*;
        use RigJointPart::{Foot as FootPart, Lowarm, Lowleg, Uparm, Upleg, Wrist};
        let unused = FrameLandmark::Unused;
        let (rule, joints, side) = match region {
            Head => return self.head_landmarks(),
            Neck => (
                Rule::Limb,
                self.named(&RigJointName::C_NECK, &RigJointName::C_SPINE3)?,
                1.0,
            ),
            Torso => (
                Rule::Limb,
                self.named(&RigJointName::C_NECK, &RigJointName::C_SPINE0)?,
                1.0,
            ),
            Hips => (
                Rule::Limb,
                self.named(&RigJointName::C_SPINE1, &RigJointName::ROOT)?,
                1.0,
            ),
            UpperArm(s) => (Rule::Limb, self.limb(s, Uparm, Lowarm, unused)?, s.sign()),
            Shoulder(s) => {
                let uparm = self.side_joint(s, Uparm)?;
                (
                    Rule::Shoulder,
                    self.limb(s, Uparm, Lowarm, uparm.into())?,
                    s.sign(),
                )
            }
            Forearm(s) => (Rule::Limb, self.limb(s, Lowarm, Wrist, unused)?, s.sign()),
            WholeArm(s) => (Rule::Limb, self.limb(s, Uparm, Wrist, unused)?, s.sign()),
            Thigh(s) => (Rule::Limb, self.limb(s, Upleg, Lowleg, unused)?, s.sign()),
            LowerLeg(s) => (
                Rule::Limb,
                self.limb(s, Lowleg, FootPart, unused)?,
                s.sign(),
            ),
            Knee(s) => {
                let lowleg = self.side_joint(s, Lowleg)?;
                (
                    Rule::Knee,
                    self.limb(s, Lowleg, FootPart, lowleg.into())?,
                    s.sign(),
                )
            }
            WholeLeg(s) => (Rule::Limb, self.limb(s, Upleg, FootPart, unused)?, s.sign()),
            Elbow(s) => {
                let joints = [
                    self.side_joint(s, Lowarm)?.into(),
                    self.side_joint(s, Uparm)?.into(),
                    self.side_joint(s, Wrist)?.into(),
                    unused,
                    unused,
                    unused,
                ];
                (Rule::Elbow, joints, s.sign())
            }
            Hand(s) => return self.hand_landmarks(s),
            Foot(s) => return self.foot_landmarks(s),
        };
        Ok(Landmarks {
            rule,
            joints,
            side,
            owners: region.owners(),
        })
    }

    fn head_landmarks(&self) -> std::result::Result<Landmarks, RigJointLookupError> {
        let [left, right, head] = self.eyes()?;
        Ok(Landmarks {
            rule: Rule::Head,
            joints: [
                head.into(),
                self.joint_index(&RigJointName::C_JAW_NULL)?.into(),
                left.into(),
                right.into(),
                head.into(),
                FrameLandmark::Unused,
            ],
            side: 1.0,
            owners: FitRegion::Head.owners(),
        })
    }

    fn hand_landmarks(&self, side: Side) -> std::result::Result<Landmarks, RigJointLookupError> {
        let mut joints = self.limb(
            side,
            RigJointPart::Wrist,
            RigJointPart::MiddleNull,
            FrameLandmark::Unused,
        )?;
        joints[2] = self.side_joint(side, RigJointPart::Pinky1)?.into();
        joints[3] = self.side_joint(side, RigJointPart::Index1)?.into();
        Ok(Landmarks {
            rule: Rule::Hand,
            joints,
            side: side.sign(),
            owners: FitRegion::Hand(side).owners(),
        })
    }

    fn foot_landmarks(&self, side: Side) -> std::result::Result<Landmarks, RigJointLookupError> {
        let mut joints = [FrameLandmark::Unused; 6];
        joints[0] = self.side_joint(side, RigJointPart::Foot)?.into();
        joints[1] = self.side_joint(side, RigJointPart::Ball)?.into();
        Ok(Landmarks {
            rule: Rule::Foot,
            joints,
            side: side.sign(),
            owners: FitRegion::Foot(side).owners(),
        })
    }

    fn side_joint(
        &self,
        side: Side,
        name: RigJointPart,
    ) -> std::result::Result<RigJointOrdinal, RigJointLookupError> {
        self.joint_index(&side.joint(name))
    }

    /// The eyes and the head, which face every frame forward.
    fn eyes(&self) -> std::result::Result<[RigJointOrdinal; 3], RigJointLookupError> {
        Ok([
            self.joint_index(&RigJointName::L_EYE)?,
            self.joint_index(&RigJointName::R_EYE)?,
            self.joint_index(&RigJointName::C_HEAD)?,
        ])
    }

    /// A central span between two named joints, faced by the eyes.
    fn named(
        &self,
        proximal: &RigJointName,
        distal: &RigJointName,
    ) -> std::result::Result<[FrameLandmark; 6], RigJointLookupError> {
        let [l, r, h] = self.eyes()?;
        Ok([
            self.joint_index(proximal)?.into(),
            self.joint_index(distal)?.into(),
            l.into(),
            r.into(),
            h.into(),
            FrameLandmark::Unused,
        ])
    }

    /// A limb span on one side, faced by the eyes, with one extra joint slot.
    fn limb(
        &self,
        side: Side,
        proximal: RigJointPart,
        distal: RigJointPart,
        extra: FrameLandmark,
    ) -> std::result::Result<[FrameLandmark; 6], RigJointLookupError> {
        let [l, r, h] = self.eyes()?;
        Ok([
            self.side_joint(side, proximal)?.into(),
            self.side_joint(side, distal)?.into(),
            l.into(),
            r.into(),
            h.into(),
            extra,
        ])
    }

    pub(super) fn thumb_landmarks(
        &self,
        side: Side,
    ) -> std::result::Result<Landmarks, RigJointLookupError> {
        Ok(Landmarks {
            rule: Rule::Thumb,
            joints: [
                self.side_joint(side, RigJointPart::Thumb1)?.into(),
                self.side_joint(side, RigJointPart::ThumbNull)?.into(),
                self.side_joint(side, RigJointPart::Thumb3)?.into(),
                FrameLandmark::Unused,
                FrameLandmark::Unused,
                FrameLandmark::Unused,
            ],
            side: side.sign(),
            // Every thumb joint, matched by prefix below.
            owners: Vec::new(),
        })
    }
}

#[cfg(test)]
mod packing_tests {
    use super::*;

    #[test]
    fn extracted_frame_rules_keep_joint_words_and_unused_padding() {
        let names = [
            "root",
            "c_jaw_null",
            "l_eye",
            "r_eye",
            "c_head",
            "l_wrist",
            "l_middle_null",
            "l_pinky1",
            "l_index1",
            "l_foot",
            "l_ball",
            "l_lowarm",
            "l_uparm",
            "l_thumb1",
            "l_thumb_null",
            "l_thumb3",
        ]
        .map(RigJointName::from);
        let wearer = Wearer {
            faces: &[],
            positions: &[],
            normals: &[],
            joint_indices: &[],
            joint_weights: &[],
            joint_names: &names,
            joints: &[],
        };
        let head = wearer.device_landmarks(FitRegion::Head).unwrap();
        let hand = wearer
            .device_landmarks(FitRegion::Hand(Side::Left))
            .unwrap();
        let foot = wearer
            .device_landmarks(FitRegion::Foot(Side::Left))
            .unwrap();
        let shoulder = wearer
            .device_landmarks(FitRegion::Shoulder(Side::Left))
            .unwrap();
        let thumb = wearer.thumb_landmarks(Side::Left).unwrap();
        assert_eq!(
            head.joints.map(FrameLandmark::device_word),
            [4, 1, 2, 3, 4, 0]
        );
        assert_eq!(
            hand.joints.map(FrameLandmark::device_word),
            [5, 6, 7, 8, 4, 0]
        );
        assert_eq!(
            foot.joints.map(FrameLandmark::device_word),
            [9, 10, 0, 0, 0, 0]
        );
        assert_eq!(
            shoulder.joints.map(FrameLandmark::device_word),
            [12, 11, 2, 3, 4, 12]
        );
        assert_eq!(
            thumb.joints.map(FrameLandmark::device_word),
            [13, 14, 15, 0, 0, 0]
        );
        assert_eq!(
            [
                head.rule as u32,
                hand.rule as u32,
                foot.rule as u32,
                shoulder.rule as u32,
                thumb.rule as u32
            ],
            [4, 3, 2, 1, 6]
        );
        let skin =
            ["l_uparm", "l_uparm_twisted", "r_uparm", "l_uparm_extra"].map(RigJointName::from);
        let wearer = Wearer {
            joint_names: &skin,
            ..wearer
        };
        assert_eq!(
            wearer
                .owned_joints(&[RigJointName::L_UPARM])
                .into_iter()
                .map(u32::from)
                .collect::<Vec<_>>(),
            [1, 1, 0, 0]
        );
    }
}

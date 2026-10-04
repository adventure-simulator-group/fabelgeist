//! Which rig joints orient and size each region's frame.

use anyhow::{Context, Result};

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
pub(super) struct Landmarks {
    pub rule: Rule,
    /// Joint indices; meaning depends on the rule.
    pub joints: [u32; 6],
    pub side: f32,
    pub owners: Vec<String>,
}

impl Wearer<'_> {
    fn joint_index(&self, name: &str) -> Result<u32> {
        self.joint_names
            .iter()
            .position(|n| n == name)
            .map(|i| i as u32)
            .with_context(|| format!("missing armor landmark {name}"))
    }

    /// Which joints own skin for a set of owner names.
    pub(crate) fn owned_joints(&self, owners: &[String]) -> Vec<u32> {
        self.joint_names
            .iter()
            .map(|name| {
                u32::from(
                    owners
                        .iter()
                        .any(|owner| name == owner || name.starts_with(&format!("{owner}_twist"))),
                )
            })
            .collect()
    }

    pub(super) fn device_landmarks(&self, region: FitRegion) -> Result<Landmarks> {
        use FitRegion::*;
        let (rule, joints, side) = match region {
            Head => {
                let [l, r, h] = self.eyes()?;
                (
                    Rule::Head,
                    [h, self.joint_index("c_jaw_null")?, l, r, h, 0],
                    1.0,
                )
            }
            Neck => (Rule::Limb, self.named("c_neck", "c_spine3")?, 1.0),
            Torso => (Rule::Limb, self.named("c_neck", "c_spine0")?, 1.0),
            Hips => (Rule::Limb, self.named("c_spine1", "root")?, 1.0),
            UpperArm(s) => (Rule::Limb, self.limb(s, "uparm", "lowarm", 0)?, s.sign()),
            Shoulder(s) => {
                let uparm = self.side_joint(s, "uparm")?;
                (
                    Rule::Shoulder,
                    self.limb(s, "uparm", "lowarm", uparm)?,
                    s.sign(),
                )
            }
            Forearm(s) => (Rule::Limb, self.limb(s, "lowarm", "wrist", 0)?, s.sign()),
            WholeArm(s) => (Rule::Limb, self.limb(s, "uparm", "wrist", 0)?, s.sign()),
            Thigh(s) => (Rule::Limb, self.limb(s, "upleg", "lowleg", 0)?, s.sign()),
            LowerLeg(s) => (Rule::Limb, self.limb(s, "lowleg", "foot", 0)?, s.sign()),
            Knee(s) => {
                let lowleg = self.side_joint(s, "lowleg")?;
                (
                    Rule::Knee,
                    self.limb(s, "lowleg", "foot", lowleg)?,
                    s.sign(),
                )
            }
            WholeLeg(s) => (Rule::Limb, self.limb(s, "upleg", "foot", 0)?, s.sign()),
            Elbow(s) => {
                let joints = [
                    self.side_joint(s, "lowarm")?,
                    self.side_joint(s, "uparm")?,
                    self.side_joint(s, "wrist")?,
                    0,
                    0,
                    0,
                ];
                (Rule::Elbow, joints, s.sign())
            }
            Hand(s) => {
                let mut joints = self.limb(s, "wrist", "middle_null", 0)?;
                joints[2] = self.side_joint(s, "pinky1")?;
                joints[3] = self.side_joint(s, "index1")?;
                (Rule::Hand, joints, s.sign())
            }
            Foot(s) => {
                let joints = [
                    self.side_joint(s, "foot")?,
                    self.side_joint(s, "ball")?,
                    0,
                    0,
                    0,
                    0,
                ];
                (Rule::Foot, joints, s.sign())
            }
        };
        Ok(Landmarks {
            rule,
            joints,
            side,
            owners: region.owners(),
        })
    }

    fn side_joint(&self, side: Side, name: &str) -> Result<u32> {
        self.joint_index(&format!("{}_{name}", side.prefix()))
    }

    /// The eyes and the head, which face every frame forward.
    fn eyes(&self) -> Result<[u32; 3]> {
        Ok([
            self.joint_index("l_eye")?,
            self.joint_index("r_eye")?,
            self.joint_index("c_head")?,
        ])
    }

    /// A central span between two named joints, faced by the eyes.
    fn named(&self, proximal: &str, distal: &str) -> Result<[u32; 6]> {
        let [l, r, h] = self.eyes()?;
        Ok([
            self.joint_index(proximal)?,
            self.joint_index(distal)?,
            l,
            r,
            h,
            0,
        ])
    }

    /// A limb span on one side, faced by the eyes, with one extra joint slot.
    fn limb(&self, side: Side, proximal: &str, distal: &str, extra: u32) -> Result<[u32; 6]> {
        let [l, r, h] = self.eyes()?;
        Ok([
            self.side_joint(side, proximal)?,
            self.side_joint(side, distal)?,
            l,
            r,
            h,
            extra,
        ])
    }

    pub(super) fn thumb_landmarks(&self, side: Side) -> Result<Landmarks> {
        let p = side.prefix();
        Ok(Landmarks {
            rule: Rule::Thumb,
            joints: [
                self.joint_index(&format!("{p}_thumb1"))?,
                self.joint_index(&format!("{p}_thumb_null"))?,
                self.joint_index(&format!("{p}_thumb3"))?,
                0,
                0,
                0,
            ],
            side: side.sign(),
            // Every thumb joint, matched by prefix below.
            owners: Vec::new(),
        })
    }
}

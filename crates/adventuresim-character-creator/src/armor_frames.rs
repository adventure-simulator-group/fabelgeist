//! The wearer armor is fitted to: its body, rig and anatomical regions.

use anyhow::Result;

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

    /// The rig's joint name prefix for this side.
    pub(crate) fn prefix(self) -> &'static str {
        match self {
            Self::Left => "l",
            Self::Right => "r",
        }
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
    pub fn owners(self) -> Vec<String> {
        use FitRegion::*;
        let central = |names: &[&str]| names.iter().map(|n| n.to_string()).collect();
        let sided = |side: Side, names: &[&str]| {
            names
                .iter()
                .map(|n| format!("{}_{n}", side.prefix()))
                .collect()
        };
        match self {
            Head => central(&["c_head", "c_jaw"]),
            Neck => central(&["c_neck", "c_spine3"]),
            Torso => central(&[
                "c_spine0",
                "c_spine1",
                "c_spine2",
                "c_spine3",
                "l_clavicle",
                "r_clavicle",
            ]),
            Hips => central(&["root", "c_spine0", "c_spine1", "l_upleg", "r_upleg"]),
            UpperArm(s) | Shoulder(s) => sided(s, &["uparm"]),
            Forearm(s) => sided(s, &["lowarm"]),
            WholeArm(s) | Elbow(s) => sided(s, &["uparm", "lowarm"]),
            Thigh(s) => sided(s, &["upleg"]),
            LowerLeg(s) | Knee(s) => sided(s, &["lowleg"]),
            WholeLeg(s) => sided(s, &["upleg", "lowleg"]),
            Hand(s) => sided(s, HAND_SKIN_JOINTS),
            Foot(s) => sided(
                s,
                &["foot", "talocrural", "subtalar", "transversetarsal", "ball"],
            ),
        }
    }

    /// The joints whose skin supports a part fitted to the region. The
    /// shoulder cap spans the deltoid and the clavicular transition: its
    /// frame stays anchored by the upper arm, while its support also takes the
    /// proximal surface that does not shorten with that bone.
    pub fn support_owners(self) -> Vec<String> {
        let mut owners = self.owners();
        if let FitRegion::Shoulder(side) = self {
            owners.push(format!("{}_clavicle", side.prefix()));
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
    pub joint_names: &'a [String],
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
pub(crate) const HAND_SKIN_JOINTS: &[&str] = &[
    "wrist",
    "index1",
    "index2",
    "index3",
    "index_null",
    "middle1",
    "middle2",
    "middle3",
    "middle_null",
    "ring1",
    "ring2",
    "ring3",
    "ring_null",
    "pinky0",
    "pinky1",
    "pinky2",
    "pinky3",
    "pinky_null",
];

impl Wearer<'_> {
    /// The body vertices whose skin belongs mostly to `region`'s joints.
    pub fn support_indices(&self, region: FitRegion) -> Result<Vec<usize>> {
        let owners = region.support_owners();
        let owned = self
            .joint_names
            .iter()
            .map(|name| {
                owners
                    .iter()
                    .any(|owner| name == owner || name.starts_with(&format!("{owner}_twist")))
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
                    .filter(|(j, _)| owned[**j as usize])
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

    #[test]
    fn shoulder_support_unites_clavicle_and_arm_without_crossing_body_regions() {
        let names = [
            "l_uparm",
            "l_lowarm",
            "l_uparm_twist0_proc",
            "l_clavicle",
            "r_clavicle",
            "c_spine3",
        ]
        .map(String::from);
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
        let names = ["l_wrist", "l_middle_null", "l_pinky_null", "l_foot"].map(String::from);
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

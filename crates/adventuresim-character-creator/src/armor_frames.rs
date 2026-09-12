//! Semantic rig anchors and body envelopes for independently authored armor.

use adventuresim_armor_model::parametric::PartFrame;
use anyhow::{Context, Result, ensure};

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
    fn prefix(self) -> &'static str {
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

pub struct Wearer<'a> {
    pub faces: &'a [[u32; 3]],
    pub positions: &'a [[f32; 3]],
    pub normals: &'a [[f32; 3]],
    pub joint_indices: &'a [[u32; 8]],
    pub joint_weights: &'a [[f32; 8]],
    pub joint_names: &'a [String],
    pub joints: &'a [[f32; 8]],
}

const SKIN_SUPPORT_THRESHOLD: f32 = 0.30;
const MINIMUM_REGION_RADIUS_M: f32 = 0.018;
// The glove body covers every finger segment, including terminal skin weights.
// The thumb has its own frame and remains outside this envelope.
const HAND_SKIN_JOINTS: &[&str] = &[
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
    pub fn support_indices(&self, region: FitRegion) -> Result<Vec<usize>> {
        let (_, _, mut owners) = self.landmarks(region)?;
        // The shoulder cap spans the deltoid and the clavicular transition.
        // Its frame remains anchored by the upper arm, while its support also
        // includes the proximal surface that does not shorten with that bone.
        if let FitRegion::Shoulder(side) = region {
            owners.push(format!("{}_clavicle", side.prefix()));
        }
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
    fn joint(&self, name: &str) -> Result<[f32; 3]> {
        let i = self
            .joint_names
            .iter()
            .position(|n| n == name)
            .with_context(|| format!("missing armor landmark {name}"))?;
        Ok([self.joints[i][0], self.joints[i][1], self.joints[i][2]])
    }

    pub fn frame(&self, region: FitRegion) -> Result<PartFrame> {
        if let FitRegion::Elbow(side) = region {
            return self.elbow_frame(side);
        }
        let (proximal, distal, owners) = self.landmarks(region)?;
        let axial = normalized(subtract(proximal, distal))?;
        let eyes = midpoint(self.joint("l_eye")?, self.joint("r_eye")?);
        let facing = subtract(eyes, self.joint("c_head")?);
        let front = match region {
            FitRegion::Shoulder(_) => {
                let up = [0.0, 1.0, 0.0];
                normalized(std::array::from_fn(|i| up[i] - axial[i] * dot(up, axial)))?
            }
            FitRegion::Foot(side) => {
                let foot = subtract(
                    self.joint(&format!("{}_ball", side.prefix()))?,
                    self.joint(&format!("{}_foot", side.prefix()))?,
                );
                normalized([foot[0], 0.0, foot[2]])?
            }
            FitRegion::Hand(side) => {
                let pinky = self.joint(&format!("{}_pinky1", side.prefix()))?;
                let index = self.joint(&format!("{}_index1", side.prefix()))?;
                let away = subtract(pinky, index);
                let across = normalized(std::array::from_fn(|i| {
                    away[i] - axial[i] * dot(away, axial)
                }))?;
                cross(across, axial)
            }
            _ => normalized([facing[0], 0.0, facing[2]])?,
        };
        let across = normalized(cross(axial, front))?;
        let mut anterior = cross(across, axial);
        if matches!(region, FitRegion::Hand(Side::Right)) {
            anterior = anterior.map(|v| -v);
        }
        let mut frame = PartFrame {
            origin: midpoint(proximal, distal),
            axes: [across, axial, anterior],
            half_extents: [1.0; 3],
        };
        let mut lo = [f32::INFINITY; 3];
        let mut hi = [f32::NEG_INFINITY; 3];
        let length = distance(proximal, distal);
        let owned = self
            .joint_names
            .iter()
            .map(|name| {
                owners
                    .iter()
                    .any(|owner| name == owner || name.starts_with(&format!("{owner}_twist")))
            })
            .collect::<Vec<_>>();
        for (i, p) in self.positions.iter().enumerate() {
            let weight: f32 = self.joint_indices[i]
                .iter()
                .zip(self.joint_weights[i])
                .filter(|(j, _)| owned[**j as usize])
                .map(|(_, w)| w)
                .sum();
            if weight < SKIN_SUPPORT_THRESHOLD {
                continue;
            }
            let local: [f32; 3] = frame.axes.map(|a| dot(subtract(*p, frame.origin), a));
            if local[1].abs() > length * 0.55 {
                continue;
            }
            for axis in 0..3 {
                lo[axis] = lo[axis].min(local[axis]);
                hi[axis] = hi[axis].max(local[axis]);
            }
        }
        ensure!(
            lo.iter().chain(&hi).all(|v| v.is_finite()),
            "no anatomical envelope for {region:?}"
        );
        let center = [(lo[0] + hi[0]) * 0.5, 0.0, (lo[2] + hi[2]) * 0.5];
        frame.origin = frame.point(center);
        frame.half_extents = [(hi[0] - lo[0]) * 0.5, length * 0.5, (hi[2] - lo[2]) * 0.5]
            .map(|v| v.max(MINIMUM_REGION_RADIUS_M));
        self.specialize(region, &mut frame)?;
        frame.validate()?;
        Ok(frame)
    }

    fn landmarks(&self, region: FitRegion) -> Result<([f32; 3], [f32; 3], Vec<String>)> {
        use FitRegion::*;
        let named =
            |a: &str, b: &str, owners: Vec<String>| Ok((self.joint(a)?, self.joint(b)?, owners));
        let limb = |s: Side, a: &str, b: &str, owners: &[&str]| {
            let prefix = s.prefix();
            named(
                &format!("{prefix}_{a}"),
                &format!("{prefix}_{b}"),
                owners.iter().map(|n| format!("{prefix}_{n}")).collect(),
            )
        };
        match region {
            Head => {
                let head = self.joint("c_head")?;
                let chin = self.joint("c_jaw_null")?;
                let top = self
                    .positions
                    .iter()
                    .map(|p| p[1])
                    .fold(f32::NEG_INFINITY, f32::max);
                Ok((
                    [head[0], top, head[2]],
                    [head[0], chin[1], head[2]],
                    vec!["c_head".into(), "c_jaw".into()],
                ))
            }
            Neck => named(
                "c_neck",
                "c_spine3",
                vec!["c_neck".into(), "c_spine3".into()],
            ),
            Torso => named(
                "c_neck",
                "c_spine0",
                vec![
                    "c_spine0".into(),
                    "c_spine1".into(),
                    "c_spine2".into(),
                    "c_spine3".into(),
                    "l_clavicle".into(),
                    "r_clavicle".into(),
                ],
            ),
            Hips => named(
                "c_spine1",
                "root",
                vec![
                    "root".into(),
                    "c_spine0".into(),
                    "c_spine1".into(),
                    "l_upleg".into(),
                    "r_upleg".into(),
                ],
            ),
            UpperArm(s) | Shoulder(s) => limb(s, "uparm", "lowarm", &["uparm"]),
            Forearm(s) => limb(s, "lowarm", "wrist", &["lowarm"]),
            Elbow(s) => limb(s, "lowarm", "wrist", &["uparm", "lowarm"]),
            WholeArm(s) => limb(s, "uparm", "wrist", &["uparm", "lowarm"]),
            Thigh(s) => limb(s, "upleg", "lowleg", &["upleg"]),
            LowerLeg(s) | Knee(s) => limb(s, "lowleg", "foot", &["lowleg"]),
            WholeLeg(s) => limb(s, "upleg", "foot", &["upleg", "lowleg"]),
            Hand(s) => limb(s, "wrist", "middle_null", HAND_SKIN_JOINTS),
            Foot(s) => {
                let prefix = s.prefix();
                let ankle = self.joint(&format!("{prefix}_foot"))?;
                let floor = self
                    .positions
                    .iter()
                    .filter(|p| {
                        if matches!(s, Side::Left) {
                            p[0] > 0.0
                        } else {
                            p[0] < 0.0
                        }
                    })
                    .map(|p| p[1])
                    .fold(f32::INFINITY, f32::min);
                Ok((
                    [ankle[0], ankle[1] + 0.025, ankle[2]],
                    [ankle[0], floor, ankle[2]],
                    ["foot", "talocrural", "subtalar", "transversetarsal", "ball"]
                        .iter()
                        .map(|n| format!("{prefix}_{n}"))
                        .collect(),
                ))
            }
        }
    }

    fn specialize(&self, region: FitRegion, frame: &mut PartFrame) -> Result<()> {
        match region {
            FitRegion::Shoulder(s) => {
                frame.origin = self.joint(&format!("{}_uparm", s.prefix()))?;
                frame.half_extents[1] *= 0.48;
            }
            FitRegion::Knee(s) => {
                frame.origin = self.joint(&format!("{}_lowleg", s.prefix()))?;
                frame.half_extents[1] = frame.half_extents[0] * 0.85;
                // A cop's side fan must point away from the body on both limbs.
                let outward = if matches!(s, Side::Left) { 1.0 } else { -1.0 };
                if frame.axes[0][0] * outward < 0.0 {
                    frame.axes[0] = frame.axes[0].map(|v| -v);
                }
            }
            _ => {}
        }
        Ok(())
    }
}

#[path = "elbow_frame.rs"]
mod elbow_frame;

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    (0..3).map(|i| a[i] * b[i]).sum()
}
fn subtract(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
fn midpoint(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| (a[i] + b[i]) * 0.5)
}
fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = subtract(a, b);
    dot(d, d).sqrt()
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn normalized(a: [f32; 3]) -> Result<[f32; 3]> {
    let l = dot(a, a).sqrt();
    ensure!(l > 1e-6, "coincident armor landmarks");
    Ok(a.map(|v| v / l))
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

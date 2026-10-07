//! Where the mannequin bust's head lies on a character.
//!
//! The bust's head is matched to the character's by its extent: as wide as
//! the character's head across the ears, as deep from the back of the skull
//! to the tip of the nose, its crown at the character's crown, and turned to
//! face where the eyes look. Height scales with the mean of width and depth,
//! which keeps the hair's fall from stretching with a long face.

use fabelgeist_hair::Head;

/// The joint whose skin carries the head, and the eyes that face it.
const HEAD_JOINT: &str = "c_head";
const EYE_JOINTS: [&str; 2] = ["l_eye", "r_eye"];
/// Vertices carried at least this much by the head joint belong to the head.
const HEAD_SKIN_SHARE: f32 = 0.5;

/// The skinned body the hair is placed on, in model space (metres, +Y up).
pub struct HeadSkin<'a> {
    pub positions: &'a [[f32; 3]],
    pub joint_indices: &'a [[u32; 8]],
    pub joint_weights: &'a [[f32; 8]],
    pub joint_names: &'a [String],
    /// Each joint's rest transform: translation, rotation quaternion, scale.
    pub global_joint_states: &'a [[f32; 8]],
}

/// Why a head could not be found on a body.
#[derive(Debug, thiserror::Error)]
pub enum HeadPlacementError {
    #[error("the skeleton has no `{0}` joint")]
    MissingJoint(&'static str),
    #[error("no vertices are skinned to the head")]
    NoHeadVertices,
    #[error("the eyes do not look away from the head joint")]
    NoFacing,
}

/// The affine map from the bust's head frame to a character's model space.
#[derive(Clone, Copy, Debug)]
pub struct HeadPlacement {
    origin: [f32; 3],
    /// The bust's +X, +Y and +Z in model space, each scaled.
    axes: [[f32; 3]; 3],
    /// The scale along each bust axis, for carrying normals.
    scale: [f32; 3],
    /// The index of the head joint, which the hair is skinned to.
    pub head_joint: u32,
}

impl HeadPlacement {
    pub fn fit(skin: &HeadSkin<'_>, bust: &Head) -> Result<Self, HeadPlacementError> {
        let joint = |name: &'static str| {
            skin.joint_names
                .iter()
                .position(|joint| joint == name)
                .ok_or(HeadPlacementError::MissingJoint(name))
        };
        let head = joint(HEAD_JOINT)?;
        let [left, right] = [joint(EYE_JOINTS[0])?, joint(EYE_JOINTS[1])?];
        let at = |index: usize| {
            let state = skin.global_joint_states[index];
            [state[0], state[1], state[2]]
        };
        let eyes = scale(add(at(left), at(right)), 0.5);
        let mut forward = sub(eyes, at(head));
        forward[1] = 0.0;
        let forward = normalize(forward).ok_or(HeadPlacementError::NoFacing)?;
        let up = [0.0, 1.0, 0.0];
        let side = cross(up, forward);

        let extent = HeadExtent::of(skin, head as u32, [side, up, forward])
            .ok_or(HeadPlacementError::NoHeadVertices)?;
        let bust_extent = HeadExtent::of_bust(bust);
        let width = extent.side[1] - extent.side[0];
        let depth = extent.forward[1] - extent.forward[0];
        let scale_side = width / (bust_extent.side[1] - bust_extent.side[0]);
        let scale_forward = depth / (bust_extent.forward[1] - bust_extent.forward[0]);
        let scale_up = 0.5 * (scale_side + scale_forward);
        let factors = [scale_side, scale_up, scale_forward];

        // Model-space coordinates, along each axis, of the bust's origin.
        let centre = |range: [f32; 2]| 0.5 * (range[0] + range[1]);
        let origin_side = centre(extent.side) - scale_side * centre(bust_extent.side);
        let origin_forward = centre(extent.forward) - scale_forward * centre(bust_extent.forward);
        let origin_up = extent.crown - scale_up * bust_extent.crown;
        let origin = add(
            add(scale(side, origin_side), scale(up, origin_up)),
            scale(forward, origin_forward),
        );
        Ok(Self {
            origin,
            axes: [
                scale(side, scale_side),
                scale(up, scale_up),
                scale(forward, scale_forward),
            ],
            scale: factors,
            head_joint: head as u32,
        })
    }

    /// A point of the bust's frame, in model space.
    pub fn point(&self, p: [f32; 3]) -> [f32; 3] {
        let [x, y, z] = self.axes;
        add(
            add(self.origin, scale(x, p[0])),
            add(scale(y, p[1]), scale(z, p[2])),
        )
    }

    /// A normal of the bust's frame, in model space: carried by the inverse
    /// transpose, so it stays square to the stretched surface.
    pub fn normal(&self, n: [f32; 3]) -> [f32; 3] {
        let [x, y, z] = self.axes;
        let [sx, sy, sz] = self.scale;
        let carried = add(
            scale(x, n[0] / (sx * sx)),
            add(scale(y, n[1] / (sy * sy)), scale(z, n[2] / (sz * sz))),
        );
        normalize(carried).unwrap_or(n)
    }

    /// A model-space point, back in the bust's frame.
    pub fn bust_point(&self, p: [f32; 3]) -> [f32; 3] {
        let offset = sub(p, self.origin);
        let [sx, sy, sz] = self.scale;
        let [x, y, z] = self.axes;
        [
            dot(offset, x) / (sx * sx),
            dot(offset, y) / (sy * sy),
            dot(offset, z) / (sz * sz),
        ]
    }

    /// A model-space normal, back in the bust's frame: the inverse of
    /// [`Self::normal`].
    pub fn bust_normal(&self, n: [f32; 3]) -> [f32; 3] {
        let [x, y, z] = self.axes;
        normalize([dot(n, x), dot(n, y), dot(n, z)]).unwrap_or(n)
    }
}

/// A head's extent along the side and forward axes, and its crown's height.
struct HeadExtent {
    side: [f32; 2],
    forward: [f32; 2],
    crown: f32,
}

impl HeadExtent {
    fn of(skin: &HeadSkin<'_>, head: u32, [side, up, forward]: [[f32; 3]; 3]) -> Option<Self> {
        let mut extent: Option<Self> = None;
        for (index, position) in skin.positions.iter().enumerate() {
            let share: f32 = skin.joint_indices[index]
                .iter()
                .zip(&skin.joint_weights[index])
                .filter(|(joint, _)| **joint == head)
                .map(|(_, weight)| weight)
                .sum();
            if share < HEAD_SKIN_SHARE {
                continue;
            }
            let [s, u, f] = [
                dot(*position, side),
                dot(*position, up),
                dot(*position, forward),
            ];
            let grown = extent.get_or_insert(Self {
                side: [s, s],
                forward: [f, f],
                crown: u,
            });
            grown.side = [grown.side[0].min(s), grown.side[1].max(s)];
            grown.forward = [grown.forward[0].min(f), grown.forward[1].max(f)];
            grown.crown = grown.crown.max(u);
        }
        extent
    }

    /// The bust's head: across its ears, from the back of its skull to the
    /// tip of its nose, and up to its crown.
    fn of_bust(bust: &Head) -> Self {
        let [left, right] = bust.ears;
        let cranium = bust.cranium;
        let nose = bust.nose;
        Self {
            side: [left.center.x - left.radii.x, right.center.x + right.radii.x],
            forward: [
                cranium.center.z - cranium.radii.z,
                nose.center.z + nose.radii.z,
            ],
            crown: cranium.center.y + cranium.radii.y,
        }
    }
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(a: [f32; 3]) -> Option<[f32; 3]> {
    let length = dot(a, a).sqrt();
    (length > f32::EPSILON).then(|| scale(a, 1.0 / length))
}

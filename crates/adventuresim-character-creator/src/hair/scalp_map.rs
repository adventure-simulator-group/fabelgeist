//! Where on the scalp's texture each point of the scalp lies.
//!
//! The hair crate maps its scalp by the angle from the crown, out to past
//! the nape, onto a disk inside the square texture: a texel near the crown
//! covers a few times the skin one near the hairline does, a fifth of the
//! square is the corners outside the disk, and much of the disk lies below
//! the hairline. The scalp's texture is mapped afresh, fitted to the scalp:
//!
//! * by Lambert's equal-area projection about the scalp's middle -- the
//!   axis of the narrowest cone from the cranium's centre that holds the
//!   whole scalp, leaning back from straight up, as the scalp reaches lower
//!   at the nape than at the brow -- out to its edge, so every texel covers
//!   about as much skin: the crown, where partings and whorls lie, as
//!   finely as anywhere;
//! * the disk spread over the whole square by Shirley and Chiu's concentric
//!   mapping, which keeps areas too, so no texel is wasted on the corners.
//!
//! Both are continuous: the texture has no seam.

use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI, TAU};

use fabelgeist_hair::{Head, Surface};
use fabelgeist_math::Vec3;

/// How far past the scalp's edge the map reaches, radians from its axis:
/// so the scalp's edge lies inside the texture, not on its rim.
const RIM_MARGIN: f32 = 0.02;
/// Steps taken leaning the axis towards the scalp's furthest point, each a
/// smaller share of the way: the narrowest cone holding the scalp, near
/// enough.
const AXIS_STEPS: usize = 64;

/// The scalp's texture map, fitted to one bust's scalp.
#[derive(Clone, Copy, Debug)]
pub(super) struct ScalpMap {
    /// The point the map is projected about: the cranium's centre.
    pub(crate) center: Vec3,
    /// The map's axis, and the directions square to it the texture's sides
    /// run along: towards the face, and towards the head's side.
    pub(crate) axis: Vec3,
    pub(crate) forward: Vec3,
    pub(crate) side: Vec3,
    /// The sine of half the angle from the axis the map reaches.
    pub(crate) rim: f32,
    /// About how much skin a texel's side covers, metres, at `size` texels
    /// a side.
    texel_per_side: f32,
}

impl ScalpMap {
    /// The map for `head`'s `scalp`.
    pub(super) fn fit(head: &Head, scalp: &Surface) -> Self {
        let center = head.cranium.center;
        let directions: Vec<Vec3> = scalp
            .positions
            .iter()
            .filter_map(|&p| unit(p - center))
            .collect();
        let up = Vec3::new(0.0, 1.0, 0.0);
        let mean = directions
            .iter()
            .fold(Vec3::new(0.0, 0.0, 0.0), |sum, &d| sum + d);
        let mut axis = unit(mean).unwrap_or(up);
        for step in 1..=AXIS_STEPS {
            let furthest = directions
                .iter()
                .copied()
                .min_by(|a, b| a.dot(axis).total_cmp(&b.dot(axis)));
            let Some(furthest) = furthest else {
                break;
            };
            axis = unit(axis + (furthest - axis) * (1.0 / (step + 1) as f32)).unwrap_or(axis);
        }
        let widest = directions
            .iter()
            .map(|d| angle(*d, axis))
            .fold(0.0f32, f32::max);
        let face = Vec3::new(0.0, 0.0, 1.0);
        let forward = unit(face - axis * face.dot(axis)).unwrap_or(face);
        let side = axis.cross(forward);
        let radius = scalp
            .positions
            .iter()
            .map(|&p| (p - center).length())
            .sum::<f32>()
            / scalp.positions.len().max(1) as f32;
        let rim = ((widest + RIM_MARGIN).min(PI) * 0.5).sin();
        // A sphere's cap out to the rim, the area the square spans.
        let area = 4.0 * PI * radius * radius * rim * rim;
        Self {
            center,
            axis,
            forward,
            side,
            rim,
            texel_per_side: area.sqrt(),
        }
    }

    /// About how much skin a texel's side covers, metres, in a texture
    /// `size` texels a side.
    pub(super) fn texel(&self, size: usize) -> f32 {
        self.texel_per_side / size as f32
    }

    /// Where `p`, in the bust's frame, lies in the texture, 0 to 1 each way,
    /// if within the map.
    pub(super) fn uv(&self, p: Vec3) -> Option<[f32; 2]> {
        let direction = unit(p - self.center)?;
        let reach = (angle(direction, self.axis) * 0.5).sin() / self.rim;
        if reach > 1.0 {
            return None;
        }
        let azimuth = direction.dot(self.side).atan2(direction.dot(self.forward));
        let [a, b] = square(reach, azimuth);
        Some([(a + 1.0) * 0.5, (b + 1.0) * 0.5])
    }
}

/// `v` at unit length, if it has a direction.
fn unit(v: Vec3) -> Option<Vec3> {
    let length = v.length();
    (length > f32::EPSILON).then(|| v * (1.0 / length))
}

/// The angle between unit directions `a` and `b`, radians.
fn angle(a: Vec3, b: Vec3) -> f32 {
    a.dot(b).clamp(-1.0, 1.0).acos()
}

/// A point of the unit disk, `reach` from its centre at `azimuth` radians,
/// on the square from -1 to 1 each way: Shirley and Chiu's concentric
/// mapping, inverted.
fn square(reach: f32, azimuth: f32) -> [f32; 2] {
    // Into the quarter-turn the square's first side starts at.
    let azimuth = (azimuth + FRAC_PI_4).rem_euclid(TAU) - FRAC_PI_4;
    let along = |from: f32| (azimuth - from) / FRAC_PI_4 * reach;
    if azimuth < FRAC_PI_4 {
        [reach, along(0.0)]
    } else if azimuth < 3.0 * FRAC_PI_4 {
        [-along(FRAC_PI_2), reach]
    } else if azimuth < 5.0 * FRAC_PI_4 {
        [-reach, -along(PI)]
    } else {
        [along(3.0 * FRAC_PI_2), -reach]
    }
}

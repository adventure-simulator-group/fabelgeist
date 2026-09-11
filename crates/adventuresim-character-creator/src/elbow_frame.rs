//! Joint-centered cop placement in the anatomical elbow bend plane.

use super::*;

const JOINT_BAND_M: f32 = 0.045;
const COP_HEIGHT_TO_WIDTH: f32 = 0.85;

impl Wearer<'_> {
    pub(super) fn elbow_frame(&self, side: Side) -> Result<PartFrame> {
        let origin = self.joint(&format!("{}_lowarm", side.prefix()))?;
        let upper = normalized(subtract(
            self.joint(&format!("{}_uparm", side.prefix()))?,
            origin,
        ))?;
        let lower = normalized(subtract(
            self.joint(&format!("{}_wrist", side.prefix()))?,
            origin,
        ))?;
        let axial = normalized(subtract(upper, lower))?;
        let posterior = normalized(std::array::from_fn(|i| -upper[i] - lower[i]))
            .context("elbow bend plane requires non-collinear arm landmarks")?;
        let mut outward = normalized(cross(axial, posterior))?;
        let side_sign = if matches!(side, Side::Left) {
            1.0
        } else {
            -1.0
        };
        if outward[0] * side_sign < 0.0 {
            outward = outward.map(|v| -v);
        }
        let mut frame = PartFrame {
            origin,
            axes: [outward, axial, posterior],
            half_extents: [0.0; 3],
        };
        for index in self.support_indices(FitRegion::Elbow(side))? {
            let local = frame
                .axes
                .map(|axis| dot(subtract(self.positions[index], origin), axis));
            if local[1].abs() > JOINT_BAND_M {
                continue;
            }
            for axis in [0, 2] {
                frame.half_extents[axis] = frame.half_extents[axis].max(local[axis].abs());
            }
        }
        ensure!(
            frame.half_extents[0] > 0.0 && frame.half_extents[2] > 0.0,
            "no anatomical envelope at elbow"
        );
        frame.half_extents[1] = frame.half_extents[0] * COP_HEIGHT_TO_WIDTH;
        frame.validate()?;
        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Arms {
        positions: Vec<[f32; 3]>,
        indices: Vec<[u32; 8]>,
        weights: Vec<[f32; 8]>,
        names: Vec<String>,
        joints: Vec<[f32; 8]>,
    }

    impl Arms {
        fn bent() -> Self {
            let mut arms = Self {
                positions: Vec::new(),
                indices: Vec::new(),
                weights: Vec::new(),
                names: Vec::new(),
                joints: Vec::new(),
            };
            for (prefix, x, width, depth) in [("l", 0.3, 0.04, 0.03), ("r", -0.3, 0.06, 0.05)] {
                let first_joint = arms.joints.len() as u32;
                // Unequal bone lengths must not skew the bend-plane bisector.
                for (name, y, z) in [
                    ("uparm", 1.5, 0.1),
                    ("lowarm", 1.2, 0.0),
                    ("wrist", 0.6, 0.2),
                ] {
                    arms.names.push(format!("{prefix}_{name}"));
                    arms.joints.push([x, y, z, 0.0, 0.0, 0.0, 1.0, 1.0]);
                }
                for (vertex, offset) in [
                    [width, 0.0, 0.0],
                    [-width, 0.0, 0.0],
                    [0.0, 0.0, depth],
                    [0.0, 0.0, -depth],
                    // Owned skin farther down the limb is not elbow width.
                    [0.5, -0.2, 0.5],
                ]
                .into_iter()
                .enumerate()
                {
                    arms.positions
                        .push([x + offset[0], 1.2 + offset[1], offset[2]]);
                    arms.indices.push([first_joint + vertex as u32 % 2; 8]);
                    arms.weights.push([0.125; 8]);
                }
            }
            arms
        }

        fn wearer(&self) -> Wearer<'_> {
            Wearer {
                faces: &[],
                positions: &self.positions,
                normals: &self.positions,
                joint_indices: &self.indices,
                joint_weights: &self.weights,
                joint_names: &self.names,
                joints: &self.joints,
            }
        }
    }

    fn assert_vector(actual: [f32; 3], expected: [f32; 3]) {
        assert!(
            distance(actual, expected) < 1e-5,
            "{actual:?} != {expected:?}"
        );
    }

    #[test]
    fn both_elbows_face_away_from_bend_and_use_only_local_arm_support() {
        let arms = Arms::bent();
        for (side, x, width, depth) in [
            (Side::Left, 0.3_f32, 0.04, 0.03),
            (Side::Right, -0.3_f32, 0.06, 0.05),
        ] {
            let frame = arms.wearer().frame(FitRegion::Elbow(side)).unwrap();
            assert_vector(frame.origin, [x, 1.2, 0.0]);
            assert_vector(frame.axes[0], [x.signum(), 0.0, 0.0]);
            assert_vector(frame.axes[1], [0.0, 1.0, 0.0]);
            assert_vector(frame.axes[2], [0.0, 0.0, -1.0]);
            assert!((frame.half_extents[0] - width).abs() < 1e-5);
            assert!((frame.half_extents[2] - depth).abs() < 1e-5);
        }
    }

    #[test]
    fn elbow_orientation_follows_the_arm_bend_plane_when_arms_are_raised() {
        let mut arms = Arms::bent();
        let (sin, cos) = 0.6_f32.sin_cos();
        let rotate = |p: [f32; 3]| [p[0], cos * p[1] - sin * p[2], sin * p[1] + cos * p[2]];
        for position in &mut arms.positions {
            *position = rotate(*position);
        }
        for joint in &mut arms.joints {
            let position = rotate([joint[0], joint[1], joint[2]]);
            joint[..3].copy_from_slice(&position);
        }
        for side in [Side::Left, Side::Right] {
            let frame = arms.wearer().frame(FitRegion::Elbow(side)).unwrap();
            assert_vector(frame.axes[1], rotate([0.0, 1.0, 0.0]));
            assert_vector(frame.axes[2], rotate([0.0, 0.0, -1.0]));
        }
    }

    #[test]
    fn collinear_arm_landmarks_are_rejected_instead_of_inventing_a_bend_plane() {
        for side in [Side::Left, Side::Right] {
            for wrist_y in [0.6, 1.5] {
                let mut arms = Arms::bent();
                let first = if matches!(side, Side::Left) { 0 } else { 3 };
                arms.joints[first][2] = 0.0;
                arms.joints[first + 2][1] = wrist_y;
                arms.joints[first + 2][2] = 0.0;
                assert!(arms.wearer().frame(FitRegion::Elbow(side)).is_err());
            }
        }
    }
}

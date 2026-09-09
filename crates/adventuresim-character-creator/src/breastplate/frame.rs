//! Resolve anatomical rig axes and the torso width domain for each body pose.

use super::*;

#[derive(Clone, Copy)]
pub(super) struct TorsoFrame {
    pub(super) bottom: [f32; 3],
    pub(super) vertical_axis: [f32; 3],
    pub(super) vertical_extent: f32,
    pub(super) lateral_axis: [f32; 3],
    pub(super) front_axis: [f32; 3],
    pub(super) half_width: f32,
    pub(super) anchors: TorsoUpperRigAnchors,
}
impl TorsoFrame {
    pub(super) fn new(
        input: &TorsoSurfaceInput<'_>,
        positions: &[[f32; 3]],
        normals: &[[f32; 3]],
        states: &[[f32; 8]],
    ) -> Result<Self, String> {
        let joint = |name: &str| {
            input
                .joint_names
                .iter()
                .position(|candidate| candidate == name)
                .map(|index| {
                    let state = states[index];
                    [state[0], state[1], state[2]]
                })
                .ok_or_else(|| format!("MHR rig is missing {name}"))
        };
        let bottom = joint("c_spine0")?;
        let neck = joint("c_neck")?;
        let left_clavicle = joint("l_clavicle")?;
        let right_clavicle = joint("r_clavicle")?;
        let left_shoulder = joint("l_uparm")?;
        let right_shoulder = joint("r_uparm")?;
        let head = joint("c_head")?;
        let eyes = scale(add(joint("l_eye")?, joint("r_eye")?), 0.5);
        let vertical_axis = normalized(sub(neck, bottom))?;
        let vertical_extent = length(sub(neck, bottom));
        let lateral_axis = normalized(sub(left_clavicle, right_clavicle))?;
        let eye_direction = sub(eyes, head);
        let front_axis = normalized(sub(
            eye_direction,
            scale(vertical_axis, dot(eye_direction, vertical_axis)),
        ))?;
        let support_joints = input.support_joints();
        let raw_coordinates = |position: [f32; 3]| {
            let relative = sub(position, bottom);
            [
                dot(relative, lateral_axis),
                dot(relative, vertical_axis) / vertical_extent,
            ]
        };
        let mut front_widths = positions
            .iter()
            .copied()
            .enumerate()
            .filter(|(vertex, position)| {
                let vertical = raw_coordinates(*position)[1];
                (0.05..=0.95).contains(&vertical)
                    && dot(normals[*vertex], front_axis) > 0.02
                    && input.joint_weight(*vertex, &support_joints) >= 0.2
            })
            .map(|(_, position)| raw_coordinates(position)[0].abs())
            .collect::<Vec<_>>();
        front_widths.sort_by(|a, b| a.total_cmp(b));
        let half_width = *front_widths
            .get(front_widths.len() * 9 / 10)
            .ok_or_else(|| "front torso has no width samples".to_owned())?;
        if half_width <= 1e-4 {
            return Err("front torso width is degenerate".into());
        }
        Ok(Self {
            bottom,
            vertical_axis,
            vertical_extent,
            lateral_axis,
            front_axis,
            half_width,
            anchors: TorsoUpperRigAnchors {
                neck_base: neck,
                clavicles: [left_clavicle, right_clavicle],
                shoulders: [left_shoulder, right_shoulder],
            },
        })
    }
    pub(super) fn raw_coordinates(&self, position: [f32; 3]) -> [f32; 2] {
        let relative = sub(position, self.bottom);
        [
            dot(relative, self.lateral_axis),
            dot(relative, self.vertical_axis) / self.vertical_extent,
        ]
    }
    pub(super) fn coordinates(&self, position: [f32; 3]) -> [f32; 2] {
        let [lateral, vertical] = self.raw_coordinates(position);
        [lateral / self.half_width, vertical]
    }
}

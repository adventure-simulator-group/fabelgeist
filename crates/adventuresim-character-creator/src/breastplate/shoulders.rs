//! Select and fair a continuous shoulder contour with stable morph branches.

use super::*;

pub(super) fn shoulder_faces(input: &TorsoSurfaceInput<'_>, frame: TorsoFrame) -> Vec<[u32; 3]> {
    let shoulder_joints = input
        .joint_names
        .iter()
        .enumerate()
        .filter_map(|(index, name)| {
            matches!(
                name.as_str(),
                "root"
                    | "c_spine0"
                    | "c_spine1"
                    | "c_spine2"
                    | "c_spine3"
                    | "c_neck"
                    | "l_clavicle"
                    | "r_clavicle"
                    | "l_uparm"
                    | "r_uparm"
            )
            .then_some(index)
        })
        .collect::<BTreeSet<_>>();
    let forbidden_limb_joints = input
        .joint_names
        .iter()
        .enumerate()
        .filter_map(|(index, name)| {
            ((name.contains("head")
                || name.contains("eye")
                || name.contains("jaw")
                || name.contains("loarm")
                || name.contains("hand")
                || name.contains("upleg")
                || name.contains("loleg")
                || name.contains("foot"))
                && !matches!(name.as_str(), "l_uparm" | "r_uparm"))
            .then_some(index)
        })
        .collect::<BTreeSet<_>>();
    let joint_set_weight = |vertex: usize, joints: &BTreeSet<usize>| {
        input.joint_indices[vertex]
            .iter()
            .zip(input.joint_weights[vertex])
            .filter(|(joint, _)| joints.contains(&(**joint as usize)))
            .map(|(_, weight)| weight)
            .sum::<f32>()
    };
    input
        .faces
        .iter()
        .copied()
        .filter(|face| {
            let supported = face
                .iter()
                .map(|vertex| joint_set_weight(*vertex as usize, &shoulder_joints))
                .sum::<f32>()
                / 3.0;
            let forbidden = face
                .iter()
                .map(|vertex| joint_set_weight(*vertex as usize, &forbidden_limb_joints))
                .sum::<f32>()
                / 3.0;
            let within_shoulder_region = face.iter().any(|vertex| {
                let [lateral, vertical] = frame.raw_coordinates(input.positions[*vertex as usize]);
                lateral.abs() <= frame.half_width * 1.35 && (0.68..=1.18).contains(&vertical)
            });
            supported >= 0.015 && forbidden <= 0.45 && within_shoulder_region
        })
        .collect::<Vec<_>>()
}
pub(super) struct ShoulderContour<'a> {
    pub(super) frame: TorsoFrame,
    pub(super) faces: &'a [[u32; 3]],
}
impl ShoulderContour<'_> {
    fn guide(&self, signed: f32) -> [f32; 3] {
        let [left_clavicle, right_clavicle] = self.frame.anchors.clavicles;
        let [left_shoulder, right_shoulder] = self.frame.anchors.shoulders;
        let neck = self.frame.anchors.neck_base;
        let absolute = signed.abs();
        let (clavicle, shoulder) = if signed < 0.0 {
            (right_clavicle, right_shoulder)
        } else {
            (left_clavicle, left_shoulder)
        };
        let shoulder_center = add(clavicle, scale(sub(shoulder, clavicle), 0.65));
        if absolute <= 0.36 {
            let t = smoothstep(absolute / 0.36);
            add(neck, scale(sub(clavicle, neck), t))
        } else {
            let t = smoothstep((absolute - 0.36) / 0.64);
            add(clavicle, scale(sub(shoulder_center, clavicle), t))
        }
    }
    fn station(
        &self,
        positions: &[[f32; 3]],
        normals: &[[f32; 3]],
        sample: usize,
        query: [f32; 3],
        branch_signature: Option<&[f32]>,
    ) -> Result<Vec<(f32, f32, TorsoShoulderSample)>, String> {
        let TorsoFrame {
            bottom,
            lateral_axis,
            front_axis,
            vertical_axis,
            vertical_extent,
            ..
        } = self.frame;
        let query_lateral = dot(sub(query, bottom), lateral_axis);
        let query_depth = dot(sub(query, bottom), front_axis);
        let mut candidates = Vec::<(f32, f32, TorsoShoulderSample)>::new();
        for face in self.faces {
            for crossing in triangle_lateral_crossings(
                positions,
                normals,
                *face,
                bottom,
                lateral_axis,
                query_lateral,
            )? {
                let relative = sub(crossing.position, bottom);
                let depth = dot(relative, front_axis);
                let vertical = dot(relative, vertical_axis) / vertical_extent;
                if (0.68..=1.18).contains(&vertical) && (depth - query_depth).abs() <= 0.18 {
                    candidates.push((vertical, depth, crossing));
                }
            }
        }
        if candidates.is_empty() {
            return Err(format!(
                "shoulder lateral plane sample {sample} misses body contour"
            ));
        }
        candidates.sort_by(|left, right| left.1.total_cmp(&right.1));
        let mut unique = Vec::<(f32, f32, TorsoShoulderSample)>::new();
        for candidate in candidates {
            if let Some(existing) = unique
                .last_mut()
                .filter(|existing| (existing.1 - candidate.1).abs() < 0.001)
            {
                if candidate.0 > existing.0 {
                    *existing = candidate;
                }
            } else {
                unique.push(candidate);
            }
        }
        let frontmost_depth = unique
            .iter()
            .map(|(_, depth, _)| *depth)
            .reduce(f32::max)
            .ok_or_else(|| format!("shoulder sample {sample} has no top contour"))?;
        let anterior_floor = (query_depth - 0.005).max(frontmost_depth - 0.060);
        unique.retain(|(_, depth, _)| *depth >= anterior_floor);
        let pareto = unique
            .iter()
            .copied()
            .filter(|candidate| {
                !unique.iter().any(|other| {
                    other.0 >= candidate.0 - 1e-5
                        && other.1 >= candidate.1 - 1e-5
                        && (other.0 > candidate.0 + 1e-5 || other.1 > candidate.1 + 1e-5)
                })
            })
            .collect::<Vec<_>>();
        let corridor_highest = pareto
            .iter()
            .map(|(height, _, _)| *height)
            .reduce(f32::max)
            .ok_or_else(|| format!("shoulder sample {sample} has no top contour"))?;
        let semantic_offset = branch_signature.map_or(0.0, |signature| signature[sample]);
        let station = pareto
            .into_iter()
            .map(|(height, depth, crossing)| {
                let local_cost = 8.0 * (corridor_highest - height)
                    + 0.25 * (depth - query_depth).abs()
                    + 4.0 * (query_depth - depth).max(0.0)
                    + branch_signature.map_or(0.0, |_| {
                        3.0 * ((depth - query_depth) - semantic_offset).abs()
                    });
                (depth, local_cost, crossing)
            })
            .collect::<Vec<_>>();
        if station.is_empty() {
            return Err(format!("shoulder sample {sample} has no top contour"));
        }
        Ok(station)
    }
    fn solve(
        &self,
        stations: &[Vec<(f32, f32, TorsoShoulderSample)>],
    ) -> Result<Vec<usize>, String> {
        let TorsoFrame {
            vertical_axis,
            front_axis,
            ..
        } = self.frame;
        let mut states = Vec::<(usize, usize, f32, Vec<usize>)>::new();
        for first in 0..stations[0].len() {
            for second in 0..stations[1].len() {
                let first_position = stations[0][first].2.position;
                let second_position = stations[1][second].2.position;
                let pair_delta = sub(second_position, first_position);
                if dot(pair_delta, vertical_axis).abs() > 0.04
                    || dot(pair_delta, front_axis).abs() > 0.04
                {
                    continue;
                }
                let depth_step = (stations[1][second].0 - stations[0][first].0).abs();
                states.push((
                    first,
                    second,
                    stations[0][first].1 + stations[1][second].1 + 25.0 * depth_step,
                    vec![first, second],
                ));
            }
        }
        for sample in 2..stations.len() {
            let mut slots = vec![
                None::<(f32, Vec<usize>)>;
                stations[sample - 1].len() * stations[sample].len()
            ];
            for (before, previous, cost, path) in &states {
                let before_depth = stations[sample - 2][*before].0;
                let previous_depth = stations[sample - 1][*previous].0;
                for next in 0..stations[sample].len() {
                    let next_depth = stations[sample][next].0;
                    let previous_position = stations[sample - 1][*previous].2.position;
                    let next_position = stations[sample][next].2.position;
                    let pair_delta = sub(next_position, previous_position);
                    if dot(pair_delta, vertical_axis).abs() > 0.04
                        || dot(pair_delta, front_axis).abs() > 0.04
                    {
                        continue;
                    }
                    let first_difference = (next_depth - previous_depth).abs();
                    let second_difference =
                        (next_depth - 2.0 * previous_depth + before_depth).abs();
                    let next_cost = *cost
                        + stations[sample][next].1
                        + 25.0 * first_difference
                        + 80.0 * second_difference;
                    let slot = &mut slots[*previous * stations[sample].len() + next];
                    if slot.as_ref().is_none_or(|(best, _)| next_cost < *best) {
                        let mut next_path = path.clone();
                        next_path.push(next);
                        *slot = Some((next_cost, next_path));
                    }
                }
            }
            states = slots
                .into_iter()
                .enumerate()
                .filter_map(|(index, state)| {
                    state.map(|(cost, path)| {
                        (
                            index / stations[sample].len(),
                            index % stations[sample].len(),
                            cost,
                            path,
                        )
                    })
                })
                .collect();
        }
        let path = states
            .into_iter()
            .min_by(|left, right| left.2.total_cmp(&right.2))
            .ok_or_else(|| "shoulder contour path solve is empty".to_owned())?
            .3;
        Ok(path)
    }
    fn fair(
        &self,
        raw_envelope: &[TorsoShoulderSample],
    ) -> Result<Vec<TorsoShoulderSample>, String> {
        let TorsoFrame {
            bottom,
            lateral_axis,
            ..
        } = self.frame;
        let semantic_anchors = [0, 20, 32, 44, TORSO_SHOULDER_ENVELOPE_SAMPLES - 1];
        let mut envelope = raw_envelope.to_vec();
        for _ in 0..64 {
            let previous = envelope.clone();
            for sample in 1..TORSO_SHOULDER_ENVELOPE_SAMPLES - 1 {
                if semantic_anchors.contains(&sample) {
                    continue;
                }
                let mut position = add(
                    scale(previous[sample].position, 0.50),
                    scale(
                        add(previous[sample - 1].position, previous[sample + 1].position),
                        0.25,
                    ),
                );
                let raw_lateral = dot(sub(raw_envelope[sample].position, bottom), lateral_axis);
                let fair_lateral = dot(sub(position, bottom), lateral_axis);
                position = add(position, scale(lateral_axis, raw_lateral - fair_lateral));
                let displacement = sub(position, raw_envelope[sample].position);
                let displacement_length = length(displacement);
                if displacement_length > 0.012 {
                    position = add(
                        raw_envelope[sample].position,
                        scale(displacement, 0.012 / displacement_length),
                    );
                }
                envelope[sample].position = position;
                envelope[sample].normal = normalized(add(
                    scale(previous[sample].normal, 0.50),
                    scale(
                        add(previous[sample - 1].normal, previous[sample + 1].normal),
                        0.25,
                    ),
                ))?;
            }
        }
        Ok(envelope)
    }
    fn validate(
        &self,
        envelope: &[TorsoShoulderSample],
        raw_envelope: &[TorsoShoulderSample],
    ) -> Result<(), String> {
        let TorsoFrame {
            bottom,
            lateral_axis,
            vertical_axis,
            front_axis,
            ..
        } = self.frame;
        for (sample, (fair, raw)) in envelope.iter().zip(raw_envelope).enumerate() {
            let corridor_distance = length(sub(fair.position, raw.position));
            if corridor_distance > 0.012_1 {
                return Err(format!(
                    "shoulder contour leaves body corridor at sample {sample}: {corridor_distance}"
                ));
            }
        }
        for (sample, pair) in envelope.windows(2).enumerate() {
            let first = sub(pair[0].position, bottom);
            let second = sub(pair[1].position, bottom);
            let lateral_step = dot(second, lateral_axis) - dot(first, lateral_axis);
            let height_step = (dot(second, vertical_axis) - dot(first, vertical_axis)).abs();
            let depth_step = (dot(second, front_axis) - dot(first, front_axis)).abs();
            if lateral_step <= 1e-6 || height_step > 0.04 || depth_step > 0.04 {
                return Err(format!(
                    "shoulder contour discontinuity at sample {sample}: lateral {lateral_step}, height {height_step}, depth {depth_step}"
                ));
            }
        }
        for (sample, points) in envelope.windows(3).enumerate() {
            let second_difference = add(
                sub(points[0].position, scale(points[1].position, 2.0)),
                points[2].position,
            );
            if length(second_difference) > 0.02 {
                return Err(format!(
                    "shoulder contour bends too sharply at sample {}: {}",
                    sample + 1,
                    length(second_difference)
                ));
            }
        }
        Ok(())
    }
    pub(super) fn envelope(
        &self,
        positions: &[[f32; 3]],
        normals: &[[f32; 3]],
        branch_signature: Option<&[f32]>,
    ) -> Result<(Vec<TorsoShoulderSample>, Vec<f32>), String> {
        let mut stations = Vec::new();
        let mut query_depths = Vec::new();
        for sample in 0..TORSO_SHOULDER_ENVELOPE_SAMPLES {
            let signed = sample as f32 / (TORSO_SHOULDER_ENVELOPE_SAMPLES - 1) as f32 * 2.0 - 1.0;
            let query = self.guide(signed);
            query_depths.push(dot(sub(query, self.frame.bottom), self.frame.front_axis));
            stations.push(self.station(positions, normals, sample, query, branch_signature)?);
        }
        let path = self.solve(&stations)?;
        let raw_envelope = path
            .iter()
            .enumerate()
            .map(|(sample, choice)| stations[sample][*choice].2)
            .collect::<Vec<_>>();
        let envelope = self.fair(&raw_envelope)?;
        self.validate(&envelope, &raw_envelope)?;
        let signature = path
            .iter()
            .enumerate()
            .map(|(sample, choice)| stations[sample][*choice].0 - query_depths[sample])
            .collect();
        Ok((envelope, signature))
    }
}

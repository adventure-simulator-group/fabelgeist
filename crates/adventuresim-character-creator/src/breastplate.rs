//! Maps the front torso into a stable semantic parameter domain for armor.

use std::collections::{BTreeMap, BTreeSet};

use adventuresim_armor_model::{
    SurfaceMorph, TORSO_SHOULDER_ENVELOPE_SAMPLES, TorsoClearanceMesh, TorsoCoronalAnchor,
    TorsoShoulderSample, TorsoSurface, TorsoUpperRigAnchors, TorsoVertex,
};

use crate::bracer::ForearmMorphSample;

pub struct TorsoSurfaceInput<'a> {
    pub domain: &'a str,
    pub positions: &'a [[f32; 3]],
    pub normals: &'a [[f32; 3]],
    pub faces: &'a [[u32; 3]],
    pub texcoords: &'a [[f32; 2]],
    pub texcoord_faces: &'a [[u32; 3]],
    pub joint_indices: &'a [[u32; 8]],
    pub joint_weights: &'a [[f32; 8]],
    pub joint_names: &'a [String],
    pub global_joint_states: &'a [[f32; 8]],
    pub morphs: &'a [ForearmMorphSample],
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
fn length(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}
fn smoothstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}
fn normalized(a: [f32; 3]) -> Result<[f32; 3], String> {
    let length = length(a);
    (length > 1e-8)
        .then(|| scale(a, length.recip()))
        .ok_or_else(|| "torso landmarks coincide".to_owned())
}

fn triangle_lateral_crossings(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    face: [u32; 3],
    origin: [f32; 3],
    lateral_axis: [f32; 3],
    target_lateral: f32,
) -> Result<Vec<TorsoShoulderSample>, String> {
    let vertices = face.map(|index| positions[index as usize]);
    let vertex_normals = face.map(|index| normals[index as usize]);
    let distances =
        vertices.map(|position| dot(sub(position, origin), lateral_axis) - target_lateral);
    let mut crossings = Vec::with_capacity(2);
    for (first, second) in [(0, 1), (1, 2), (2, 0)] {
        let first_distance = distances[first];
        let second_distance = distances[second];
        let epsilon = 1e-7;
        if (first_distance > epsilon && second_distance > epsilon)
            || (first_distance < -epsilon && second_distance < -epsilon)
        {
            continue;
        }
        if first_distance.abs() <= epsilon && second_distance.abs() <= epsilon {
            for corner in [first, second] {
                let candidate = TorsoShoulderSample {
                    position: vertices[corner],
                    normal: normalized(vertex_normals[corner])?,
                };
                if crossings.iter().all(|existing: &TorsoShoulderSample| {
                    length(sub(existing.position, candidate.position)) > 1e-6
                }) {
                    crossings.push(candidate);
                }
            }
            continue;
        }
        let denominator = first_distance - second_distance;
        let fraction = if denominator.abs() <= epsilon {
            0.0
        } else {
            (first_distance / denominator).clamp(0.0, 1.0)
        };
        let candidate = TorsoShoulderSample {
            position: add(
                vertices[first],
                scale(sub(vertices[second], vertices[first]), fraction),
            ),
            normal: normalized(add(
                vertex_normals[first],
                scale(sub(vertex_normals[second], vertex_normals[first]), fraction),
            ))?,
        };
        if crossings.iter().all(|existing: &TorsoShoulderSample| {
            length(sub(existing.position, candidate.position)) > 1e-6
        }) {
            crossings.push(candidate);
        }
    }
    Ok(crossings)
}

#[derive(Clone, Copy)]
struct SectionFrame {
    plane_origin: [f32; 3],
    plane_normal: [f32; 3],
    lateral_origin: [f32; 3],
    lateral_axis: [f32; 3],
    half_band: f32,
    depth_axis: [f32; 3],
}

fn section_depths(positions: &[[f32; 3]], faces: &[[u32; 3]], frame: SectionFrame) -> Vec<f32> {
    let mut depths = Vec::new();
    for face in faces {
        let vertices = face.map(|index| positions[index as usize]);
        let distances =
            vertices.map(|position| dot(sub(position, frame.plane_origin), frame.plane_normal));
        for (first, second) in [(0, 1), (1, 2), (2, 0)] {
            let first_distance = distances[first];
            let second_distance = distances[second];
            let epsilon = 1e-7;
            if first_distance.abs() <= epsilon && second_distance.abs() <= epsilon {
                continue;
            }
            if (first_distance > epsilon && second_distance > epsilon)
                || (first_distance < -epsilon && second_distance < -epsilon)
            {
                continue;
            }
            let denominator = first_distance - second_distance;
            let fraction = if denominator.abs() <= epsilon {
                0.0
            } else {
                (first_distance / denominator).clamp(0.0, 1.0)
            };
            let crossing = add(
                vertices[first],
                scale(sub(vertices[second], vertices[first]), fraction),
            );
            if dot(sub(crossing, frame.lateral_origin), frame.lateral_axis).abs() <= frame.half_band
            {
                depths.push(dot(crossing, frame.depth_axis));
            }
        }
    }
    depths
}

fn section_midpoint(mut depths: Vec<f32>) -> Result<f32, String> {
    if depths.len() < 4 {
        return Err("torso section has too few central sagittal crossings".into());
    }
    depths.sort_by(f32::total_cmp);
    let trim = if depths.len() >= 50 {
        depths.len() / 100
    } else {
        0
    };
    Ok((depths[trim] + depths[depths.len() - 1 - trim]) * 0.5)
}

pub fn build_front_torso_surface(input: TorsoSurfaceInput<'_>) -> Result<TorsoSurface, String> {
    let count = input.positions.len();
    if input.domain.is_empty()
        || input.normals.len() != count
        || input.joint_indices.len() != count
        || input.joint_weights.len() != count
        || input.faces.len() != input.texcoord_faces.len()
        || input.joint_names.len() != input.global_joint_states.len()
        || input.morphs.iter().any(|morph| {
            morph.positions.len() != count
                || morph.normals.len() != count
                || morph.global_joint_states.len() != input.joint_names.len()
        })
    {
        return Err("front torso surface inputs are inconsistent".into());
    }
    let joint = |name: &str| {
        input
            .joint_names
            .iter()
            .position(|candidate| candidate == name)
            .map(|index| {
                let state = input.global_joint_states[index];
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
    let support_joints = input
        .joint_names
        .iter()
        .enumerate()
        .filter_map(|(index, name)| {
            matches!(
                name.as_str(),
                "c_spine0"
                    | "root"
                    | "c_spine1"
                    | "c_spine2"
                    | "c_spine3"
                    | "c_neck"
                    | "l_clavicle"
                    | "r_clavicle"
                    | "l_uparm"
                    | "r_uparm"
                    | "l_upleg"
                    | "r_upleg"
            )
            .then_some(index)
        })
        .collect::<BTreeSet<_>>();
    let support_weight = |vertex: usize| {
        input.joint_indices[vertex]
            .iter()
            .zip(input.joint_weights[vertex])
            .filter(|(joint, _)| support_joints.contains(&(**joint as usize)))
            .map(|(_, weight)| weight)
            .sum::<f32>()
    };
    let raw_coordinates = |position: [f32; 3]| {
        let relative = sub(position, bottom);
        [
            dot(relative, lateral_axis),
            dot(relative, vertical_axis) / vertical_extent,
        ]
    };
    let mut front_widths = input
        .positions
        .iter()
        .copied()
        .enumerate()
        .filter(|(vertex, position)| {
            let vertical = raw_coordinates(*position)[1];
            (0.05..=0.95).contains(&vertical)
                && dot(input.normals[*vertex], front_axis) > 0.02
                && support_weight(*vertex) >= 0.2
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
    let coordinates = |position: [f32; 3]| {
        let [lateral, vertical] = raw_coordinates(position);
        [lateral / half_width, vertical]
    };
    let morph_frames = input
        .morphs
        .iter()
        .map(|morph| {
            let joint = |name: &str| {
                let index = input
                    .joint_names
                    .iter()
                    .position(|candidate| candidate == name)
                    .expect("base rig validation guarantees torso joints");
                let state = morph.global_joint_states[index];
                [state[0], state[1], state[2]]
            };
            let bottom = joint("c_spine0");
            let neck = joint("c_neck");
            let head = joint("c_head");
            let eyes = scale(add(joint("l_eye"), joint("r_eye")), 0.5);
            let vertical_axis = normalized(sub(neck, bottom))?;
            let vertical_extent = length(sub(neck, bottom));
            let lateral_axis = normalized(sub(joint("l_clavicle"), joint("r_clavicle")))?;
            let eye_direction = sub(eyes, head);
            let front_axis = normalized(sub(
                eye_direction,
                scale(vertical_axis, dot(eye_direction, vertical_axis)),
            ))?;
            let raw = |position: [f32; 3]| {
                let relative = sub(position, bottom);
                [
                    dot(relative, lateral_axis),
                    dot(relative, vertical_axis) / vertical_extent,
                ]
            };
            let mut widths = morph
                .positions
                .iter()
                .copied()
                .enumerate()
                .filter(|(vertex, position)| {
                    let vertical = raw(*position)[1];
                    (0.05..=0.95).contains(&vertical)
                        && dot(morph.normals[*vertex], front_axis) > 0.02
                        && support_weight(*vertex) >= 0.2
                })
                .map(|(_, position)| raw(position)[0].abs())
                .collect::<Vec<_>>();
            widths.sort_by(f32::total_cmp);
            let half_width = *widths
                .get(widths.len() * 9 / 10)
                .ok_or_else(|| "morph front torso has no width samples".to_owned())?;
            if half_width <= 1e-4 {
                return Err("morph front torso width is degenerate".to_owned());
            }
            let semantic = morph
                .positions
                .iter()
                .map(|position| {
                    let [lateral, vertical] = raw(*position);
                    [lateral / half_width, vertical]
                })
                .collect::<Vec<_>>();
            Ok((
                bottom,
                vertical_axis,
                vertical_extent,
                lateral_axis,
                front_axis,
                half_width,
                semantic,
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let torso_joints = input
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
            )
            .then_some(index)
        })
        .collect::<BTreeSet<_>>();
    let limb_joints = input
        .joint_names
        .iter()
        .enumerate()
        .filter_map(|(index, name)| {
            (name.contains("uparm")
                || name.contains("loarm")
                || name.contains("hand")
                || name.contains("upleg")
                || name.contains("loleg")
                || name.contains("foot"))
            .then_some(index)
        })
        .collect::<BTreeSet<_>>();
    let torso_weight = |vertex: usize| {
        input.joint_indices[vertex]
            .iter()
            .zip(input.joint_weights[vertex])
            .filter(|(joint, _)| torso_joints.contains(&(**joint as usize)))
            .map(|(_, weight)| weight)
            .sum::<f32>()
    };
    let limb_weight = |vertex: usize| {
        input.joint_indices[vertex]
            .iter()
            .zip(input.joint_weights[vertex])
            .filter(|(joint, _)| limb_joints.contains(&(**joint as usize)))
            .map(|(_, weight)| weight)
            .sum::<f32>()
    };
    const CORONAL_LEVELS: [f32; 4] = [0.20, 0.45, 0.70, 0.92];
    let section_faces = input
        .faces
        .iter()
        .copied()
        .filter(|face| {
            let torso = face
                .iter()
                .map(|vertex| torso_weight(*vertex as usize))
                .sum::<f32>()
                / 3.0;
            let limb = face
                .iter()
                .map(|vertex| limb_weight(*vertex as usize))
                .sum::<f32>()
                / 3.0;
            torso >= 0.08 && limb <= 0.35
        })
        .collect::<Vec<_>>();
    if section_faces.is_empty() {
        return Err("torso section face selection is empty".into());
    }
    let midpoint_depths = |positions: &[[f32; 3]],
                           section_bottom: [f32; 3],
                           section_vertical: [f32; 3],
                           section_extent: f32,
                           section_lateral: [f32; 3],
                           section_front: [f32; 3],
                           section_half_width: f32| {
        CORONAL_LEVELS
            .iter()
            .map(|level| {
                let plane_origin = add(
                    section_bottom,
                    scale(section_vertical, level * section_extent),
                );
                section_midpoint(section_depths(
                    positions,
                    &section_faces,
                    SectionFrame {
                        plane_origin,
                        plane_normal: section_vertical,
                        lateral_origin: section_bottom,
                        lateral_axis: section_lateral,
                        half_band: section_half_width * 0.25,
                        depth_axis: section_front,
                    },
                ))
                .map_err(|error| format!("torso coronal level {level}: {error}"))
            })
            .collect::<Result<Vec<_>, String>>()
    };
    let base_coronal_depths = midpoint_depths(
        input.positions,
        bottom,
        vertical_axis,
        vertical_extent,
        lateral_axis,
        front_axis,
        half_width,
    )?;
    let coronal_anchors = CORONAL_LEVELS
        .into_iter()
        .zip(base_coronal_depths)
        .map(|(vertical, depth)| TorsoCoronalAnchor { vertical, depth })
        .collect::<Vec<_>>();
    let morph_coronal_depths = input
        .morphs
        .iter()
        .zip(&morph_frames)
        .map(|(morph, frame)| {
            midpoint_depths(
                &morph.positions,
                frame.0,
                frame.1,
                frame.2,
                frame.3,
                frame.4,
                frame.5,
            )
        })
        .collect::<Result<Vec<_>, String>>()?;
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
    let shoulder_faces = input
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
                let [lateral, vertical] = raw_coordinates(input.positions[*vertex as usize]);
                lateral.abs() <= half_width * 1.35 && (0.68..=1.18).contains(&vertical)
            });
            supported >= 0.015 && forbidden <= 0.45 && within_shoulder_region
        })
        .collect::<Vec<_>>();
    let guide = |signed: f32| {
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
    };
    let shoulder_envelope_for = |positions: &[[f32; 3]],
                                 normals: &[[f32; 3]],
                                 branch_signature: Option<&[f32]>| {
        let mut stations = Vec::<Vec<(f32, f32, TorsoShoulderSample)>>::new();
        let mut query_depths = Vec::with_capacity(TORSO_SHOULDER_ENVELOPE_SAMPLES);
        for sample in 0..TORSO_SHOULDER_ENVELOPE_SAMPLES {
            let signed = sample as f32 / (TORSO_SHOULDER_ENVELOPE_SAMPLES - 1) as f32 * 2.0 - 1.0;
            let query = guide(signed);
            let query_lateral = dot(sub(query, bottom), lateral_axis);
            let query_depth = dot(sub(query, bottom), front_axis);
            query_depths.push(query_depth);
            let mut candidates = Vec::<(f32, f32, TorsoShoulderSample)>::new();
            for face in &shoulder_faces {
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
            stations.push(station);
        }
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
        let raw_envelope = path
            .iter()
            .enumerate()
            .map(|(sample, choice)| stations[sample][*choice].2)
            .collect::<Vec<_>>();
        let semantic_anchors = [0, 20, 32, 44, TORSO_SHOULDER_ENVELOPE_SAMPLES - 1];
        let mut envelope = raw_envelope.clone();
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
        for (sample, (fair, raw)) in envelope.iter().zip(&raw_envelope).enumerate() {
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
        let signature = path
            .iter()
            .enumerate()
            .map(|(sample, choice)| stations[sample][*choice].0 - query_depths[sample])
            .collect::<Vec<_>>();
        Ok((envelope, signature))
    };
    let (shoulder_envelope, shoulder_branch_signature) =
        shoulder_envelope_for(input.positions, input.normals, None)?;
    let morph_shoulder_envelopes = input
        .morphs
        .iter()
        .map(|morph| {
            shoulder_envelope_for(
                &morph.positions,
                &morph.normals,
                Some(&shoulder_branch_signature),
            )
            .map(|(envelope, _)| envelope)
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut clearance_source = BTreeMap::<u32, u32>::new();
    let mut clearance_body_vertices = Vec::<usize>::new();
    let clearance_faces = shoulder_faces
        .iter()
        .map(|body_face| {
            body_face.map(|body_vertex| {
                *clearance_source.entry(body_vertex).or_insert_with(|| {
                    clearance_body_vertices.push(body_vertex as usize);
                    (clearance_body_vertices.len() - 1) as u32
                })
            })
        })
        .collect::<Vec<_>>();
    let clearance_samples = |positions: &[[f32; 3]], normals: &[[f32; 3]]| {
        clearance_body_vertices
            .iter()
            .map(|vertex| TorsoShoulderSample {
                position: positions[*vertex],
                normal: normals[*vertex],
            })
            .collect::<Vec<_>>()
    };
    let clearance_mesh = TorsoClearanceMesh {
        vertices: clearance_samples(input.positions, input.normals),
        faces: clearance_faces,
        morph_vertices: input
            .morphs
            .iter()
            .map(|morph| clearance_samples(&morph.positions, &morph.normals))
            .collect(),
    };
    let upper_rig_anchors = TorsoUpperRigAnchors {
        neck_base: neck,
        clavicles: [left_clavicle, right_clavicle],
        shoulders: [left_shoulder, right_shoulder],
    };
    // Boundary landmarks are semantic garment-domain inputs.  Resolve them
    // from every fitted rig directly: inferring their motion from a nearby
    // body-envelope sample collapses neck/armscye widths on signed morphs and
    // turns a smooth authored opening into a serrated neutral-space offset.
    let morph_upper_rig_anchors = input
        .morphs
        .iter()
        .map(|morph| {
            let position = |name: &str| {
                let index = input
                    .joint_names
                    .iter()
                    .position(|candidate| candidate == name)
                    .expect("base rig validation guarantees garment joints");
                let state = morph.global_joint_states[index];
                [state[0], state[1], state[2]]
            };
            TorsoUpperRigAnchors {
                neck_base: position("c_neck"),
                clavicles: [position("l_clavicle"), position("r_clavicle")],
                shoulders: [position("l_uparm"), position("r_uparm")],
            }
        })
        .collect();
    let supported = |vertex: usize| {
        let [lateral, vertical] = coordinates(input.positions[vertex]);
        lateral.abs() <= 1.35
            && (-0.30..=1.08).contains(&vertical)
            && dot(input.normals[vertex], front_axis) > 0.02
            && support_weight(vertex) >= 0.2
    };
    let selected = input
        .faces
        .iter()
        .copied()
        .zip(input.texcoord_faces.iter().copied())
        .filter(|(face, _)| {
            face.iter()
                .filter(|vertex| supported(**vertex as usize))
                .count()
                >= 2
        })
        .collect::<Vec<_>>();
    if selected.is_empty() {
        return Err("front torso selection is empty".into());
    }
    let mut split = BTreeMap::<(u32, u32), u32>::new();
    let mut source = Vec::<(usize, usize)>::new();
    let mut faces = Vec::with_capacity(selected.len());
    for (body_face, uv_face) in selected {
        let mut face = [0; 3];
        for corner in 0..3 {
            let key = (body_face[corner], uv_face[corner]);
            face[corner] = *split.entry(key).or_insert_with(|| {
                source.push((key.0 as usize, key.1 as usize));
                (source.len() - 1) as u32
            });
        }
        faces.push(face);
    }
    let vertices = source
        .iter()
        .map(|(body, uv)| {
            let [lateral, vertical] = coordinates(input.positions[*body]);
            Ok(TorsoVertex {
                uv: *input
                    .texcoords
                    .get(*uv)
                    .ok_or_else(|| "torso UV references a missing coordinate".to_owned())?,
                lateral,
                vertical,
                position: input.positions[*body],
                normal: input.normals[*body],
                joint_indices: input.joint_indices[*body],
                joint_weights: input.joint_weights[*body],
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let morphs = input
        .morphs
        .iter()
        .map(|morph| SurfaceMorph {
            name: morph.name.clone(),
            positions: source
                .iter()
                .map(|(body, _)| morph.positions[*body])
                .collect(),
            normals: source
                .iter()
                .map(|(body, _)| morph.normals[*body])
                .collect(),
        })
        .collect();
    let morph_fronts = morph_frames.iter().map(|frame| frame.4).collect();
    let morph_semantic_coordinates = morph_frames
        .iter()
        .map(|frame| source.iter().map(|(body, _)| frame.6[*body]).collect())
        .collect();
    Ok(TorsoSurface {
        domain: input.domain.to_owned(),
        front: front_axis,
        morph_fronts,
        upper_rig_anchors,
        morph_upper_rig_anchors,
        morph_semantic_coordinates,
        vertices,
        faces,
        coronal_anchors,
        morph_coronal_depths,
        shoulder_envelope,
        morph_shoulder_envelopes,
        clearance_mesh,
        morphs,
    })
}

#[cfg(test)]
mod tests {
    use super::{SectionFrame, section_depths, section_midpoint, triangle_lateral_crossings};

    const FACES: [[u32; 3]; 4] = [[0, 1, 2], [1, 3, 2], [4, 6, 5], [5, 6, 7]];

    fn section_positions(depth_delta: f32) -> Vec<[f32; 3]> {
        vec![
            [-0.1, -1.0, -0.2 + depth_delta],
            [0.1, -1.0, -0.2 + depth_delta],
            [-0.1, 1.0, -0.2 + depth_delta],
            [0.1, 1.0, -0.2 + depth_delta],
            [-0.1, -1.0, 0.4 + depth_delta],
            [0.1, -1.0, 0.4 + depth_delta],
            [-0.1, 1.0, 0.4 + depth_delta],
            [0.1, 1.0, 0.4 + depth_delta],
        ]
    }

    fn midpoint(positions: &[[f32; 3]], height: f32) -> f32 {
        section_midpoint(section_depths(
            positions,
            &FACES,
            SectionFrame {
                plane_origin: [0.0, height, 0.0],
                plane_normal: [0.0, 1.0, 0.0],
                lateral_origin: [0.0; 3],
                lateral_axis: [1.0, 0.0, 0.0],
                half_band: 0.2,
                depth_axis: [0.0, 0.0, 1.0],
            },
        ))
        .unwrap()
    }

    #[test]
    fn triangle_sections_are_exact_and_stable_between_vertices() {
        let positions = section_positions(0.0);
        assert!((midpoint(&positions, 0.0) - 0.1).abs() < 1e-6);
        assert!((midpoint(&positions, -0.0005) - midpoint(&positions, 0.0005)).abs() < 1e-6);
    }

    #[test]
    fn morph_sections_use_corresponding_morph_positions() {
        let base = section_positions(0.0);
        let morph = section_positions(0.035);
        assert!((midpoint(&morph, 0.0) - midpoint(&base, 0.0) - 0.035).abs() < 1e-6);
    }

    #[test]
    fn lateral_plane_intersects_triangle_without_fixed_depth_query() {
        let positions = [[-1.0, 0.0, -0.4], [1.0, 0.2, 0.4], [0.0, 1.0, 0.8]];
        let normals = [[0.0, 1.0, 0.0]; 3];
        let crossings = triangle_lateral_crossings(
            &positions,
            &normals,
            [0, 1, 2],
            [0.0; 3],
            [1.0, 0.0, 0.0],
            0.25,
        )
        .unwrap();
        assert_eq!(crossings.len(), 2);
        assert!(
            crossings
                .iter()
                .all(|crossing| (crossing.position[0] - 0.25).abs() < 1e-6)
        );
        assert!(crossings.iter().any(|crossing| crossing.position[2] > 0.6));
    }
}

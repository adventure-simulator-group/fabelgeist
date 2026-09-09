//! Intersect body triangles with anatomical planes and measure coronal depth.

use super::*;

pub(super) fn triangle_lateral_crossings(
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

pub(super) const CORONAL_LEVELS: [f32; 4] = [0.20, 0.45, 0.70, 0.92];
pub(super) fn section_faces(input: &TorsoSurfaceInput<'_>) -> Result<Vec<[u32; 3]>, String> {
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
    Ok(section_faces)
}
pub(super) fn coronal_depths(
    positions: &[[f32; 3]],
    section_faces: &[[u32; 3]],
    frame: TorsoFrame,
) -> Result<Vec<f32>, String> {
    let TorsoFrame {
        bottom,
        vertical_axis,
        vertical_extent,
        lateral_axis,
        front_axis,
        half_width,
        ..
    } = frame;
    CORONAL_LEVELS
        .iter()
        .map(|level| {
            let plane_origin = add(bottom, scale(vertical_axis, level * vertical_extent));
            section_midpoint(section_depths(
                positions,
                section_faces,
                SectionFrame {
                    plane_origin,
                    plane_normal: vertical_axis,
                    lateral_origin: bottom,
                    lateral_axis,
                    half_band: half_width * 0.25,
                    depth_axis: front_axis,
                },
            ))
            .map_err(|error| format!("torso coronal level {level}: {error}"))
        })
        .collect::<Result<Vec<_>, String>>()
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

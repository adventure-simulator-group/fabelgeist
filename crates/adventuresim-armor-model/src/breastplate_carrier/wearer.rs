//! Measure the wearer in the anatomical carrier frame.

use super::*;

pub(super) fn valid_surface(surface: &TorsoSurface) -> bool {
    let count = surface.vertices.len();
    !surface.domain.is_empty()
        && count >= 3
        && !surface.faces.is_empty()
        && surface
            .faces
            .iter()
            .flatten()
            .all(|i| (*i as usize) < count)
        && surface.morphs.len() == surface.morph_fronts.len()
        && surface.morphs.len() == surface.morph_semantic_coordinates.len()
        && surface.morphs.len() == surface.morph_upper_rig_anchors.len()
        && surface.morphs.len() == surface.morph_coronal_depths.len()
        && surface
            .morphs
            .iter()
            .all(|m| m.positions.len() == count && m.normals.len() == count)
        && surface
            .morph_semantic_coordinates
            .iter()
            .all(|v| v.len() == count)
        && surface.coronal_anchors.len() >= 2
        && surface
            .morph_coronal_depths
            .iter()
            .all(|v| v.len() == surface.coronal_anchors.len())
        && surface
            .clearance_mesh
            .has_corresponding_domains(surface.morphs.len())
}

impl<'a> Wearer<'a> {
    pub(super) fn new(
        pose: CarrierPose<'a>,
        clearance: &'a TorsoClearancePose,
        source_faces: &'a [[u32; 3]],
        torso_faces: &'a [[u32; 3]],
    ) -> Result<Self, GenerateError> {
        let CarrierPose {
            positions,
            semantic,
            front,
            anchors,
            coronal_depths,
            shoulder_envelope,
        } = pose;
        let frame = Frame::from_front(front)?;
        let neck = local(anchors.neck_base, frame);
        let shoulders = anchors.shoulders.map(|point| local(point, frame));
        let shoulder_half_width = shoulders
            .iter()
            .map(|point| (point[0] - neck[0]).abs())
            .sum::<f32>()
            * 0.5;
        let count = positions.len() as f32;
        let mean_level = semantic.iter().map(|p| p[1]).sum::<f32>() / count;
        let mean_height = positions.iter().map(|p| local(*p, frame)[1]).sum::<f32>() / count;
        let variance = semantic
            .iter()
            .map(|p| (p[1] - mean_level).powi(2))
            .sum::<f32>();
        let height_slope = semantic
            .iter()
            .zip(positions)
            .map(|(uv, p)| (uv[1] - mean_level) * (local(*p, frame)[1] - mean_height))
            .sum::<f32>()
            / variance.max(1e-8);
        let coronal_origin = coronal_depths.iter().sum::<f32>() / coronal_depths.len() as f32;
        let center_front = positions
            .iter()
            .zip(semantic)
            .filter(|(_, uv)| uv[0].abs() < 0.12 && (0.25..=0.75).contains(&uv[1]))
            .map(|(point, _)| local(*point, frame)[2])
            .fold(f32::NEG_INFINITY, f32::max);
        if !center_front.is_finite() {
            return Err(GenerateError::InvalidSurface);
        }
        let top_height = shoulder_envelope
            .iter()
            .map(|sample| local(sample.position, frame)[1])
            .fold(neck[1], f32::max);
        let fit_bottom = neck[1] - 0.36 * (height_slope.abs() / REFERENCE_SEMANTIC_HEIGHT);
        let fit_top = neck[1] - 0.10 * (height_slope.abs() / REFERENCE_SEMANTIC_HEIGHT);
        let mut torso_min = [f32::INFINITY; 3];
        let mut torso_max = [f32::NEG_INFINITY; 3];
        for index in torso_faces.iter().flatten() {
            let point = local(
                clearance.enclosure_vertices[*index as usize].position,
                frame,
            );
            if (fit_bottom..=fit_top).contains(&point[1]) {
                for axis in 0..3 {
                    torso_min[axis] = torso_min[axis].min(point[axis]);
                    torso_max[axis] = torso_max[axis].max(point[axis]);
                }
            }
        }
        if report_fit() {
            eprintln!(
                "breastplate wearer frame={:?} neck_y={} top_y={} shoulder_half={} semantic_height={} front_radius={} coronal={}",
                [frame.lateral, frame.vertical, frame.front],
                neck[1],
                top_height,
                shoulder_half_width,
                height_slope,
                center_front - coronal_origin,
                coronal_origin
            );
            eprintln!(
                "breastplate torso_fit_bounds y={fit_bottom}..{fit_top} min={torso_min:?} max={torso_max:?}"
            );
        }
        Ok(Wearer {
            frame,
            anchors,
            clearance,
            source_faces,
            torso_faces,
            x_scale: (torso_max[0].abs().max(torso_min[0].abs()) / REFERENCE_TORSO_HALF_WIDTH)
                .clamp(0.65, 1.55),
            shoulder_x_scale: (shoulder_half_width / REFERENCE_SHOULDER_HALF_WIDTH)
                .clamp(0.65, 1.55),
            y_scale: (height_slope.abs() / REFERENCE_SEMANTIC_HEIGHT).clamp(0.70, 1.45),
            z_scale: (((torso_max[2] - torso_min[2]) * 0.5) / REFERENCE_SECTION_RADIUS)
                .clamp(0.65, 1.80),
            lateral_origin: neck[0],
            coronal_origin: (torso_min[2] + torso_max[2]) * 0.5,
        })
    }
}

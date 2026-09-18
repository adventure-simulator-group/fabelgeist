//! Body-section fitting for flexible ringed limb garments.

use adventuresim_armor_model::{
    PartFrame, PartMesh, PuffAndSlashDesign, PuffAndSlashKind, generate_puff_and_slash,
};
use anyhow::{Result, ensure};

use crate::{
    armor_frames::{FitRegion, Wearer},
    armor_layer::ArmorLayerSurface,
    plate_section::PlateSection,
};

const SECTION_COUNT: usize = 25;
const SECTION_HALF_WIDTH_M: f32 = 0.014;
const FULLNESS_TAPER_COURSES: f32 = 0.8;
const LAYER_SUPPORT_ENVELOPE_SCALE: f32 = 1.8;
const SLEEVE_PROXIMAL_INSET_RADII: f32 = 1.5;

pub(crate) fn fit(
    design: &PuffAndSlashDesign,
    wearer: &Wearer<'_>,
    region: FitRegion,
    layers: &[ArmorLayerSurface<'_>],
) -> Result<PartMesh> {
    let frame = wearer.frame(region)?;
    let mesh = generate_puff_and_slash(design, &frame)?;
    let support = wearer.support_indices(region)?;
    ensure!(
        !support.is_empty(),
        "puff-and-slash garment has no limb support"
    );
    let mut owned = vec![false; wearer.positions.len()];
    for index in support {
        owned[index] = true;
    }
    let proximal_inset = if design.kind == PuffAndSlashKind::Sleeve {
        frame.half_extents[0].max(frame.half_extents[2]) * SLEEVE_PROXIMAL_INSET_RADII
    } else {
        0.0
    };
    let [low, high] = axial_span(design, &frame, proximal_inset);
    let mut triangles = wearer
        .faces
        .iter()
        .filter(|face| face.iter().any(|index| owned[*index as usize]))
        .map(|face| face.map(|index| local(&frame, wearer.positions[index as usize])))
        .collect::<Vec<_>>();
    triangles.extend(layer_triangles(
        layers,
        wearer,
        region,
        &frame,
        [low, high],
    )?);
    let sections = (0..SECTION_COUNT)
        .map(|index| {
            let y = low + (high - low) * index as f32 / (SECTION_COUNT - 1) as f32;
            PlateSection::measured_surface(&triangles, y, SECTION_HALF_WIDTH_M)
                .ok_or_else(|| anyhow::anyhow!("limb section has no body surface support at {y} m"))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(mesh.refit_surfaces(|positions, _| {
        fit_carrier(positions, &frame, [low, high], &sections, design);
    })?)
}

fn layer_triangles(
    layers: &[ArmorLayerSurface<'_>],
    wearer: &Wearer<'_>,
    region: FitRegion,
    frame: &PartFrame,
    span: [f32; 2],
) -> Result<Vec<[[f32; 3]; 3]>> {
    let radial_limit =
        frame.half_extents[0].max(frame.half_extents[2]) * LAYER_SUPPORT_ENVELOPE_SCALE;
    let mut triangles = Vec::new();
    for layer in layers {
        ensure!(
            layer.joint_indices.len() == layer.positions.len()
                && layer.joint_weights.len() == layer.positions.len(),
            "puff-and-slash lower layer has incomplete skin ownership"
        );
        let mut owned = vec![false; layer.positions.len()];
        for index in wearer.support_indices_for(region, layer.joint_indices, layer.joint_weights)? {
            owned[index] = true;
        }
        triangles.extend(
            layer
                .faces
                .iter()
                .filter(|face| face.iter().all(|index| owned[*index as usize]))
                .filter_map(|face| {
                    let triangle = face.map(|index| local(frame, layer.positions[index as usize]));
                    triangle
                        .iter()
                        .any(|point| {
                            (span[0] - SECTION_HALF_WIDTH_M..=span[1] + SECTION_HALF_WIDTH_M)
                                .contains(&point[1])
                                && point[0].hypot(point[2]) <= radial_limit
                        })
                        .then_some(triangle)
                }),
        );
    }
    Ok(triangles)
}

fn fit_carrier(
    positions: &mut [[f32; 3]],
    frame: &PartFrame,
    span: [f32; 2],
    sections: &[PlateSection],
    design: &PuffAndSlashDesign,
) {
    let course_length = (span[1] - span[0]) / f32::from(design.puff_count);
    let authored_span = axial_span(design, frame, 0.0);
    let base_allowance = design.clearance.metres() + 2.0 * design.thickness.metres();
    for point in positions {
        let mut p = local(frame, *point);
        let axial =
            ((p[1] - authored_span[0]) / (authored_span[1] - authored_span[0])).clamp(0.0, 1.0);
        p[1] = span[0] + (span[1] - span[0]) * axial;
        let station = ((p[1] - span[0]) / (span[1] - span[0]) * (SECTION_COUNT - 1) as f32)
            .clamp(0.0, (SECTION_COUNT - 1) as f32);
        let first = (station.floor() as usize).min(SECTION_COUNT - 2);
        let t = station - first as f32;
        let blend = t * t * (3.0 - 2.0 * t);
        let a = &sections[first];
        let b = &sections[first + 1];
        let center: [f32; 2] =
            std::array::from_fn(|axis| a.center[axis] + (b.center[axis] - a.center[axis]) * blend);
        let radial = [p[0], p[2]];
        let distance = radial[0].hypot(radial[1]);
        if distance <= f32::EPSILON {
            continue;
        }
        let direction = radial.map(|value| value / distance);
        let ellipse_radius = ((direction[0] / frame.half_extents[0]).powi(2)
            + (direction[1] / frame.half_extents[2]).powi(2))
        .sqrt()
        .recip();
        let mut authored_allowance = distance - ellipse_radius;
        if design.kind == PuffAndSlashKind::Sleeve && design.proximal_position.0 == 1_000 {
            let fullness_transition =
                ((span[1] - p[1]) / (course_length * FULLNESS_TAPER_COURSES)).clamp(0.0, 1.0);
            authored_allowance = base_allowance
                + (authored_allowance - base_allowance).max(0.0) * fullness_transition;
        }
        let body_radius = a.radius(direction) * (1.0 - blend) + b.radius(direction) * blend;
        let fitted_radius = body_radius + authored_allowance;
        p[0] = center[0] + direction[0] * fitted_radius;
        p[2] = center[1] + direction[1] * fitted_radius;
        *point = frame.point(p);
    }
}

fn axial_span(design: &PuffAndSlashDesign, frame: &PartFrame, proximal_inset: f32) -> [f32; 2] {
    let full_length = 2.0 * frame.half_extents[1] - proximal_inset;
    let garment_length = full_length * design.length.unit();
    let margin = full_length - garment_length;
    let low = -frame.half_extents[1] + margin * design.proximal_position.unit();
    [low, low + garment_length]
}

fn local(frame: &PartFrame, point: [f32; 3]) -> [f32; 3] {
    frame.axes.map(|axis| {
        (0..3)
            .map(|coordinate| (point[coordinate] - frame.origin[coordinate]) * axis[coordinate])
            .sum()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::armor_frames::Side;

    #[test]
    fn layer_sections_ignore_the_torso_branch_at_an_armhole() {
        let names = ["l_uparm", "l_lowarm", "l_wrist", "c_spine3"].map(String::from);
        let joints = [[0.0; 8]; 4];
        let wearer = Wearer {
            detail: adventuresim_armor_model::ArmorDetail::BakeSource,
            faces: &[],
            positions: &[],
            normals: &[],
            joint_indices: &[],
            joint_weights: &[],
            joint_names: &names,
            joints: &joints,
        };
        let frame = PartFrame {
            detail: adventuresim_armor_model::ArmorDetail::BakeSource,
            origin: [0.0; 3],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            half_extents: [0.1, 0.5, 0.1],
        };
        let positions = [
            [0.07, -0.01, -0.01],
            [0.07, 0.01, -0.01],
            [0.07, 0.0, 0.01],
            [-0.17, -0.01, -0.01],
            [-0.17, 0.01, -0.01],
            [-0.17, 0.0, 0.01],
        ];
        let faces = [[0, 1, 2], [3, 4, 5]];
        let joint_indices = [[0; 8], [0; 8], [0; 8], [3; 8], [3; 8], [3; 8]];
        let joint_weights = [[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; 6];
        let layer = ArmorLayerSurface {
            relief: adventuresim_armor_model::Millimeters(0),
            positions: &positions,
            faces: &faces,
            joint_indices: &joint_indices,
            joint_weights: &joint_weights,
        };
        assert_eq!(
            layer_triangles(
                &[layer],
                &wearer,
                FitRegion::WholeArm(Side::Left),
                &frame,
                [-0.5, 0.5],
            )
            .unwrap(),
            [[positions[0], positions[1], positions[2]]]
        );
    }

    #[test]
    fn sleeve_carrier_terminates_below_the_shoulder_joint() {
        let frame = PartFrame {
            detail: adventuresim_armor_model::ArmorDetail::BakeSource,
            origin: [0.0; 3],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            half_extents: [0.1, 0.5, 0.08],
        };
        let design = PuffAndSlashDesign {
            length: adventuresim_armor_model::Permille(1_000),
            proximal_position: adventuresim_armor_model::Permille(1_000),
            ..Default::default()
        };
        let fitted_span = axial_span(
            &design,
            &frame,
            frame.half_extents[0] * SLEEVE_PROXIMAL_INSET_RADII,
        );
        let section_points = (0..16)
            .map(|column| {
                let angle = column as f32 * std::f32::consts::TAU / 16.0;
                [0.07 * angle.cos(), 0.0, 0.06 * angle.sin()]
            })
            .collect::<Vec<_>>();
        let section = PlateSection::measured(&section_points, 0.0, 0.01);
        let sections = vec![section; SECTION_COUNT];
        let authored_high = axial_span(&design, &frame, 0.0)[1];
        let mut points = [[0.1, authored_high, 0.0]];
        fit_carrier(&mut points, &frame, fitted_span, &sections, &design);
        let fitted = local(&frame, points[0]);
        assert!((fitted[1] - fitted_span[1]).abs() < 1e-6);
        assert!(frame.half_extents[1] - fitted[1] >= 0.149_99);
    }
}

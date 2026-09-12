//! A thigh plate's medial edge stays on its anatomical side of the pelvis.
//! The canonical unposed sagittal plane follows root X. Only the existing
//! proximal trim changes; every candidate is refitted and fully thickened.
use super::{joint, local, trim_proximal};
use crate::{
    armor_clearance::{self, PlateFit},
    armor_frames::{FitRegion, Side, Wearer},
};
use adventuresim_armor_model::{CuisseDesign, PartFrame, PartMesh};
use anyhow::{Result, ensure};

const TRIM_PRECISION_M: f32 = 0.000001;

pub(super) fn fit(
    mesh: &PartMesh,
    design: &CuisseDesign,
    wearer: &Wearer<'_>,
    region: FitRegion,
    frame: &PartFrame,
) -> Result<PartMesh> {
    let FitRegion::Thigh(side) = region else {
        anyhow::bail!("cuisse requires thigh landmarks");
    };
    let outward = if matches!(side, Side::Left) {
        1.0
    } else {
        -1.0
    };
    let sagittal_x = joint(wearer, "root")?[0];
    // The paired closed shells retain one gauge of space between them.
    let separation = design.gauge.thickness.metres() * 0.5;
    let stays_on_side = |mesh: &PartMesh| {
        mesh.positions
            .iter()
            .all(|point| outward * (point[0] - sagittal_x) >= separation)
    };
    let fitted = |trim| {
        armor_clearance::fit(
            &trim_proximal(mesh, frame, region, trim)?,
            wearer,
            region,
            design.gauge.clearance,
            design.gauge.thickness,
            PlateFit::Cuisse(design),
        )
    };
    let original = fitted(0.0)?;
    if stays_on_side(&original) {
        return Ok(original);
    }
    let heights = mesh.positions.iter().map(|point| local(frame, *point)[1]);
    let low = heights.clone().fold(f32::INFINITY, f32::min);
    let high = heights.fold(f32::NEG_INFINITY, f32::max);
    let mut passing_trim = high - low;
    let mut accepted = fitted(passing_trim)?;
    ensure!(
        stays_on_side(&accepted),
        "cuisse cannot retain its noncollapsed medial span within the ipsilateral thigh region"
    );
    let mut failing_trim = 0.0;
    while passing_trim - failing_trim > TRIM_PRECISION_M {
        let candidate_trim = (failing_trim + passing_trim) * 0.5;
        let candidate = fitted(candidate_trim)?;
        if stays_on_side(&candidate) {
            passing_trim = candidate_trim;
            accepted = candidate;
        } else {
            failing_trim = candidate_trim;
        }
    }
    Ok(accepted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_armor_model::{LimbArmorDesign, Millimeters, generate_limb_armor};

    #[test]
    fn narrow_hips_trim_only_as_needed_and_keep_bilateral_shells_separate() {
        let names = [
            "root", "l_upleg", "l_lowleg", "r_upleg", "r_lowleg", "c_head", "l_eye", "r_eye",
        ]
        .map(String::from);
        let mut previous_indices = None;
        for hip in [0.095, 0.063, 0.060] {
            let mut joints = [[0.0; 8]; 8];
            for (index, point) in [
                [0.0, 1.0, 0.0],
                [hip, 0.9, 0.0],
                [hip, 0.5, 0.0],
                [-hip, 0.9, 0.0],
                [-hip, 0.5, 0.0],
                [0.0, 1.6, 0.0],
                [0.03, 1.65, 0.1],
                [-0.03, 1.65, 0.1],
            ]
            .iter()
            .enumerate()
            {
                joints[index][..3].copy_from_slice(point);
            }
            let mut positions = Vec::new();
            let mut faces = Vec::new();
            let mut skin = Vec::new();
            for (sign, owner) in [(1.0, 1), (-1.0, 3)] {
                let start = positions.len() as u32;
                for row in 0..=16 {
                    let t = row as f32 / 16.0;
                    for column in 0..48 {
                        let angle = column as f32 / 48.0 * std::f32::consts::TAU;
                        let radius = 0.036 + t * 0.014;
                        positions.push([
                            sign * hip + radius * angle.sin(),
                            0.5 + t * 0.4,
                            radius * angle.cos(),
                        ]);
                        skin.push([owner; 8]);
                    }
                }
                for row in 0..16_u32 {
                    for column in 0..48_u32 {
                        let a = start + row * 48 + column;
                        let b = start + row * 48 + (column + 1) % 48;
                        faces.extend([[a, b, b + 48], [a, b + 48, a + 48]]);
                    }
                }
            }
            let weights = vec![[0.125; 8]; positions.len()];
            let wearer = Wearer {
                positions: &positions,
                faces: &faces,
                normals: &[],
                joint_indices: &skin,
                joint_weights: &weights,
                joint_names: &names,
                joints: &joints,
            };
            let design = CuisseDesign {
                gauge: adventuresim_armor_model::PlateGauge {
                    clearance: Millimeters(12),
                    ..Default::default()
                },
                ..Default::default()
            };
            for (side, sign) in [(Side::Left, 1.0), (Side::Right, -1.0)] {
                let region = FitRegion::Thigh(side);
                let frame = wearer.frame(region).unwrap();
                let mesh =
                    generate_limb_armor(&LimbArmorDesign::Cuisse(design.clone()), &frame).unwrap();
                let result = fit(&mesh, &design, &wearer, region, &frame);
                if hip < 0.062 {
                    // With this lining/gauge the parallel thighs leave no
                    // bilateral room within the retained proximal span.
                    assert!(result.is_err());
                    continue;
                }
                let fitted = result.unwrap();
                fitted.normals().unwrap();
                assert!(
                    fitted
                        .positions
                        .iter()
                        .all(|p| sign * p[0] >= design.gauge.thickness.metres() * 0.5)
                );
                if let Some(indices) = &previous_indices {
                    assert_eq!(&fitted.indices, indices);
                }
                previous_indices = Some(fitted.indices.clone());
                let unchanged = armor_clearance::fit(
                    &trim_proximal(&mesh, &frame, region, 0.0).unwrap(),
                    &wearer,
                    region,
                    design.gauge.clearance,
                    design.gauge.thickness,
                    PlateFit::Cuisse(&design),
                )
                .unwrap();
                if hip > 0.09 {
                    assert_eq!(fitted.positions, unchanged.positions);
                } else {
                    assert!(
                        unchanged.positions.iter().any(|p| sign * p[0] < 0.0),
                        "fixture must exercise the original bilateral overlap"
                    );
                }
            }
        }
    }
}

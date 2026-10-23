//! Anatomical identities of a closed torso's four physical sewing boundaries.
use crate::armor_frames::Wearer;
use crate::surface_cut::{SurfaceCut, dot, interpolate};
use anyhow::{Context, Result, ensure};
use fabelgeist_armor::PartFrame;
use fabelgeist_rig::RigJointName;

/// Native frozen cut-point ordinals and canonical metre positions.
pub(crate) struct TorsoBoundaries {
    pub positions: Vec<[f32; 3]>,
    pub waist: Vec<u32>,
    pub neckline: Vec<u32>,
    pub armholes: [ArmholeBoundary; 2],
}

/// An anatomically named sewing loop at the native cut-topology boundary.
pub(crate) struct ArmholeBoundary {
    pub points: Vec<u32>,
}

impl TorsoBoundaries {
    pub fn new(cut: &SurfaceCut, body: &Wearer<'_>, head: &PartFrame) -> Result<Self> {
        let positions = cut
            .points
            .iter()
            .map(|p| {
                interpolate(
                    body.faces[p.triangle].map(|v| body.positions[v as usize]),
                    p.weights,
                )
            })
            .collect::<Vec<_>>();
        let mut borders = cut.sewing_boundary_loops(&positions)?;
        ensure!(
            borders.len() == 4,
            "sewn torso requires a waist, neckline and two armholes"
        );
        let mut armhole = |shoulder: RigJointName| -> Result<Vec<u32>> {
            let index = shoulder.require_in(body.joint_names)?;
            let state = body
                .joints
                .get(index)
                .context("missing shoulder for sewing boundary")?;
            let offset = std::array::from_fn(|axis| state[axis] - head.origin[axis]);
            let side = dot(offset, head.axes[0]).signum();
            let index = borders
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| {
                    (side * mean_projection(a, &positions, head.axes[0]))
                        .total_cmp(&(side * mean_projection(b, &positions, head.axes[0])))
                })
                .map(|(i, _)| i)
                .context("missing armhole sewing boundary")?;
            Ok(borders.remove(index))
        };
        let right_armhole = armhole(RigJointName::R_UPARM)?;
        let left_armhole = armhole(RigJointName::L_UPARM)?;
        // Only the central loops remain. A deep neckline cannot be confused
        // with a shallow armhole merely because its mean height is lower.
        borders.sort_by(|a, b| {
            mean_projection(a, &positions, [0., 1., 0.]).total_cmp(&mean_projection(
                b,
                &positions,
                [0., 1., 0.],
            ))
        });
        let neckline = borders.pop().context("missing neckline boundary")?;
        let waist = borders.pop().context("missing waist boundary")?;
        Ok(Self {
            positions,
            waist,
            neckline,
            armholes: [
                ArmholeBoundary {
                    points: right_armhole,
                },
                ArmholeBoundary {
                    points: left_armhole,
                },
            ],
        })
    }
}

fn mean_projection(boundary: &[u32], positions: &[[f32; 3]], axis: [f32; 3]) -> f32 {
    boundary
        .iter()
        .map(|&i| dot(positions[i as usize], axis))
        .sum::<f32>()
        / boundary.len() as f32
}

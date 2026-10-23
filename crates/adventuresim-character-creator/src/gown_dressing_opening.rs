//! Bridge physical bodice opening ownership into the frozen skirt cut chart.
use crate::{
    armor_frames::Wearer,
    bodice_opening_edges::BodiceOpeningEdges,
    surface_cut::{SurfaceCut, dot, interpolate},
};
use anyhow::{Context, Result};
use fabelgeist_armor::gpu::{OpeningChartPoint, SkirtOpeningSource};
use fabelgeist_armor::{DressingOpeningSide, GownDesign, PartFrame};
use fabelgeist_rig::RigJointName;
pub(crate) fn prepare(
    design: &GownDesign,
    cut: &SurfaceCut,
    body: &Wearer<'_>,
    hips: &PartFrame,
    head: &PartFrame,
) -> Result<Option<SkirtOpeningSource>> {
    let Some(slit) = &design.dressing_slit else {
        return Ok(None);
    };
    let edges = BodiceOpeningEdges::new(cut, body, head, design)?;
    // ClothTorsoFrame lateral points from the anatomical right shoulder to left.
    let stations = match slit.side {
        DressingOpeningSide::Right => &edges.opening.left,
        DressingOpeningSide::Left => &edges.opening.right,
    };
    let start = stations
        .first()
        .context("dressing slit has no bodice lip")?
        .vertex;
    let chart = |point: [f32; 3]| {
        let relative = std::array::from_fn(|i| point[i] - hips.origin[i]);
        OpeningChartPoint {
            angle: dot(relative, head.axes[0])
                .atan2(dot(relative, head.axes[2]))
                .rem_euclid(std::f32::consts::TAU),
            y: point[1],
        }
    };
    let points = cut
        .points
        .iter()
        .map(|p| {
            chart(interpolate(
                body.faces[p.triangle].map(|v| body.positions[v as usize]),
                p.weights,
            ))
        })
        .collect::<Vec<_>>();
    let root = RigJointName::ROOT.require_in(body.joint_names)?;
    let foot = RigJointName::L_FOOT.require_in(body.joint_names)?;
    let waist = body.joints[root][1] + design.bodice.hem_rise.metres();
    let point = edges.positions[start];
    let radius = ((point[0] - hips.origin[0]).powi(2) + (point[2] - hips.origin[2]).powi(2)).sqrt();
    Ok(Some(SkirtOpeningSource {
        axis_angle: points[start].angle,
        top_y: points[start].y,
        points,
        start_body_vertex: u32::try_from(start)?,
        waist_y: waist,
        hem_y: body.joints[root][1]
            - (body.joints[root][1] - body.joints[foot][1]) * design.skirt.length.unit(),
        tip_y: waist - slit.length.metres(),
        radius,
        gap: slit.gap,
        overlap: slit.overlap,
    }))
}

impl crate::gown_plan::GownPlan {
    pub(crate) fn finalize_dressing_opening(&mut self, design: &GownDesign) -> Result<()> {
        // Gussets change carrier connectivity and source-table offsets. Their
        // preliminary reference fit must use the ordinary unslit carrier.
        if let Some(source) = self.dressing_source.take() {
            self.join.patch.dressing_opening =
                Some(fabelgeist_armor::gpu::SkirtDressingOpening::new(
                    &design.skirt,
                    &self.join.patch,
                    *source,
                )?);
        }
        Ok(())
    }
}

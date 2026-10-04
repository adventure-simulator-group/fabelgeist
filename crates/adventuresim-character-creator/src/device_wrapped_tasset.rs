//! Wrapped thigh carriers measured in canonical unposed body coordinates.
use anyhow::{Result, ensure};
use fabelgeist_armor::gpu::wrapped_tassets::{
    self, HULL_START, HULL_WORDS, LAYER_STATIONS, MEASURED_SECTIONS, MEASURED_WORDS,
};
use fabelgeist_armor::{GarmentArmorDesign, GarmentPlateShape, WrappedTassetDesign};
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::Buffer;

use crate::armor_frames::{FitRegion, Side, Wearer};
use crate::armor_layer::ArmorLayerSurface;
use crate::device_frames::DeviceWearer;
use crate::device_garment_kernel::{Grid, Word, atomic, dispatch, read, read_u32, write};
use crate::device_piece::{DeviceCheck, DeviceRecording};

const SECTION_HALF_WIDTH_M: f32 = 0.025;
const SUSPENSION_RISE_M: f32 = 0.015;
const SAMPLES_PER_TRIANGLE: u32 = 9;

struct Support {
    points: Vec<[f32; 3]>,
    triangles: Vec<[[f32; 3]; 3]>,
    thigh_end: u32,
    waist_end: u32,
    metadata: Vec<f32>,
    nodes: u32,
    layer_nodes: u32,
}

impl Support {
    fn new(
        wearer: &Wearer<'_>,
        design: &GarmentArmorDesign,
        shape: &WrappedTassetDesign,
        side: Side,
        layers: &[ArmorLayerSurface<'_>],
        suspension: Option<f32>,
    ) -> Result<Self> {
        let joint = |name: &str| -> Result<[f32; 8]> {
            let index = wearer
                .joint_names
                .iter()
                .position(|n| n == name)
                .ok_or_else(|| anyhow::anyhow!("missing tasset landmark {name}"))?;
            Ok(wearer.joints[index])
        };
        let hip = joint(&format!("{}_upleg", side.prefix()))?;
        let knee = joint(&format!("{}_lowleg", side.prefix()))?;
        let root = joint("root")?;
        let top = suspension.unwrap_or(root[1] + SUSPENSION_RISE_M);
        let bottom = top
            + (hip[1] + (knee[1] - hip[1]) * shape.knee_reach.unit() - top) * design.length.unit();
        ensure!(
            top.is_finite() && bottom.is_finite() && top > bottom,
            "invalid tasset suspension span"
        );
        let points_for = |region| {
            wearer.support_indices(region).map(|indices| {
                indices
                    .into_iter()
                    .map(|i| wearer.positions[i])
                    .collect::<Vec<_>>()
            })
        };
        let mut points = points_for(FitRegion::Thigh(side))?;
        let thigh_end = points.len() as u32;
        points.extend(points_for(FitRegion::Hips)?);
        points.extend(points_for(FitRegion::Torso)?);
        let waist_end = points.len() as u32;
        let triangles = Self::triangles(wearer, side)?;
        let low = triangles
            .iter()
            .flatten()
            .map(|p| p[1])
            .fold(f32::INFINITY, f32::min);
        let high = triangles
            .iter()
            .flatten()
            .map(|p| p[1])
            .fold(f32::NEG_INFINITY, f32::max);
        ensure!(
            low.is_finite()
                && high.is_finite()
                && high > low
                && thigh_end >= 3
                && waist_end > thigh_end + 2,
            "insufficient tasset body support"
        );
        let nodes = ((high - low) / SECTION_HALF_WIDTH_M).ceil() as u32 + 1;
        let mut metadata = vec![0.0; HULL_START as usize];
        // Identity frame, with finite positive extents for the chart machinery.
        for word in [3, 7, 11, 12, 13, 14] {
            metadata[word] = 1.0;
        }
        metadata[16..21].copy_from_slice(&[bottom, top, low, high, nodes as f32]);
        metadata[27..32].copy_from_slice(&[
            root[2],
            design.clearance.metres(),
            design.wall_thickness.metres(),
            design.flare.unit(),
            f32::from(design.lame_count),
        ]);
        let layer_nodes = Self::layers(
            &mut points,
            &mut metadata,
            layers,
            thigh_end,
            waist_end,
            shape,
            design,
        );
        let inner_start = HULL_START + (nodes + layer_nodes) * HULL_WORDS;
        metadata[45] = inner_start as f32;
        metadata.resize(
            (inner_start + u32::from(design.lame_count) * (wrapped_tassets::COURSE_ROWS + 1))
                as usize,
            0.0,
        );
        Ok(Self {
            points,
            triangles,
            thigh_end,
            waist_end,
            metadata,
            nodes,
            layer_nodes,
        })
    }

    fn triangles(wearer: &Wearer<'_>, side: Side) -> Result<Vec<[[f32; 3]; 3]>> {
        let mut owned = vec![false; wearer.positions.len()];
        for region in [
            FitRegion::Thigh(side),
            FitRegion::LowerLeg(side),
            FitRegion::Hips,
            FitRegion::Torso,
        ] {
            for index in wearer.support_indices(region)? {
                owned[index] = true;
            }
        }
        let triangles = wearer
            .faces
            .iter()
            .filter(|face| face.iter().any(|&i| owned[i as usize]))
            .map(|face| face.map(|i| wearer.positions[i as usize]))
            .collect::<Vec<_>>();
        Ok(triangles)
    }

    fn layers(
        points: &mut Vec<[f32; 3]>,
        metadata: &mut [f32],
        layers: &[ArmorLayerSurface<'_>],
        thigh: u32,
        waist: u32,
        shape: &WrappedTassetDesign,
        design: &GarmentArmorDesign,
    ) -> u32 {
        if layers.iter().all(|layer| layer.positions.is_empty()) {
            return 0;
        }
        points.extend(
            layers
                .iter()
                .flat_map(|layer| layer.positions.iter().copied()),
        );
        let low = points[waist as usize..]
            .iter()
            .map(|p| p[1])
            .fold(f32::INFINITY, f32::min);
        let lateral = points[waist as usize..]
            .iter()
            .map(|p| p[0].abs())
            .fold(0.0, f32::max);
        let transition = (metadata[17] - metadata[16]) / f32::from(design.lame_count);
        let high = (metadata[17] + shape.upper_edge_slope.unit() * lateral).max(low + transition);
        metadata[23..26].copy_from_slice(&[low, high, LAYER_STATIONS as f32]);
        points.extend_from_within(thigh as usize..waist as usize);
        LAYER_STATIONS
    }
}

struct DeviceSupport {
    fit: Buffer,
    status: Buffer,
}

impl DeviceWearer<'_> {
    pub(crate) fn record_wrapped_tassets(
        &self,
        batch: &mut KernelBatch,
        design: &GarmentArmorDesign,
        layers: &[ArmorLayerSurface<'_>],
        suspension: Option<f32>,
    ) -> Result<DeviceRecording> {
        design.validate()?;
        let GarmentPlateShape::WrappedTassets(shape) = design.plate_shape else {
            anyhow::bail!("wrapped tasset shape required");
        };
        let left_support = Support::new(self.host, design, &shape, Side::Left, layers, suspension)?;
        let right_support =
            Support::new(self.host, design, &shape, Side::Right, layers, suspension)?;
        let bounds = [&left_support, &right_support]
            .map(|support| support.metadata[18]..=support.metadata[19]);
        let left = self.record_tasset_support(batch, &left_support)?;
        let right = self.record_tasset_support(batch, &right_support)?;
        let part = wrapped_tassets::record_wrapped_tassets(
            self.gpu,
            batch,
            design,
            [&left.fit, &right.fit],
            [&left.status, &right.status],
        )?;
        dispatch(
            self,
            batch,
            COPY_STATUS,
            &[
                read_u32("left", &left.status),
                read_u32("right", &right.status),
                atomic("status", part.status()),
            ],
            &[],
            Grid::Singles(1),
        )?;
        let checks = vec![DeviceCheck::Mesh(Box::new(move |part| {
            ensure!(part.components.len() == bounds.len(), "missing tasset side");
            for (component, span) in part.components.iter().zip(&bounds) {
                validate_shell_support(&part.positions[component.vertices.clone()], span)?;
            }
            Ok(())
        }))];
        Ok(DeviceRecording {
            part,
            frames: Vec::new(),
            checks,
        })
    }

    fn record_tasset_support(
        &self,
        batch: &mut KernelBatch,
        support: &Support,
    ) -> Result<DeviceSupport> {
        let gpu = self.gpu;
        let fit = gpu.upload(&support.metadata)?;
        let status = gpu.scratch(4, "tasset support status")?;
        let points = gpu.upload(&support.points)?;
        let triangles = gpu.upload(&support.triangles)?;
        let capacity = (support.triangles.len() as u32 * SAMPLES_PER_TRIANGLE)
            .max(support.points.len() as u32)
            + 1;
        let count = MEASURED_SECTIONS + support.nodes + support.layer_nodes;
        let bytes = u64::from(capacity) * u64::from(count) * 8;
        let samples = gpu.scratch(bytes, "tasset section samples")?;
        let hulls = gpu.scratch(bytes, "tasset section hulls")?;
        let measured = gpu.scratch(
            u64::from(MEASURED_SECTIONS * MEASURED_WORDS) * 4,
            "tasset shape sections",
        )?;
        let source = format!(
            "{}\n{}\n{}",
            wrapped_tassets::layout(),
            include_str!("device_section_hull.wgsl"),
            include_str!("device_tasset_sections.wgsl")
        );
        dispatch(
            self,
            batch,
            &source,
            &[
                read("points", &points),
                read("triangles", &triangles),
                write("samples", &samples),
                write("hulls", &hulls),
                write("fit", &fit),
                write("measured", &measured),
                atomic("status", &status),
            ],
            &[
                Word::U("capacity", capacity),
                Word::U("thigh", support.thigh_end),
                Word::U("waist", support.waist_end),
                Word::U("point_count", support.points.len() as u32),
                Word::U("faces", support.triangles.len() as u32),
                Word::U("nodes", support.nodes),
            ],
            Grid::Singles(count),
        )?;
        dispatch(
            self,
            batch,
            &format!(
                "{}\n{}",
                wrapped_tassets::layout(),
                include_str!("device_tasset_controls.wgsl")
            ),
            &[
                read("measured", &measured),
                write("fit", &fit),
                atomic("status", &status),
            ],
            &[],
            Grid::Singles(1),
        )?;
        Ok(DeviceSupport { fit, status })
    }
}
const COPY_STATUS: &str = r#"
@compute @workgroup_size(1)
fn main() { atomicOr(&status[0],left[0] | right[0]); }
"#;

/// Check after gauge, fluting and lap lift have moved the final shell vertices.
fn validate_shell_support(
    positions: &[[f32; 3]],
    span: &std::ops::RangeInclusive<f32>,
) -> Result<()> {
    if let Some(point) = positions.iter().find(|point| !span.contains(&point[1])) {
        anyhow::bail!(
            "finished tasset shell height {} exceeds measured body support {:?}",
            point[1],
            span
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_extrusion_must_remain_inside_measured_support() {
        let span = 0.5..=1.0;
        assert!(validate_shell_support(&[[0.1, 0.999, 0.1]], &span).is_ok());
        assert!(validate_shell_support(&[[0.1, 1.001, 0.1]], &span).is_err());
        assert!(validate_shell_support(&[[0.1, 0.499, 0.1]], &span).is_err());
    }
}

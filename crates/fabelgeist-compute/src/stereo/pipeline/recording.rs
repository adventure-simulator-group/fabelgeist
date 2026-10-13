//! Recording depth, temporal filtering, and per-eye output stages.

use super::super::rig::Grid;
use super::{Matrix4, Stage, StereoDepth, invert_rigid, mat4_rotation, mat4_rows, vec4};
use crate::kernel::KernelBatch;
use anyhow::Result;
use fabelgeist_gpu::data::gpu::parameters::{PassParameter, PassParameters};
use fabelgeist_gpu::prelude::WorkgroupGrid;

impl StereoDepth {
    fn image_groups(&self) -> WorkgroupGrid {
        let (w, h) = self.rectified.size();
        WorkgroupGrid::from([w.div_ceil(16), h.div_ceil(16), 1])
    }

    fn sized(&self) -> PassParameters {
        let (w, h) = self.rectified.size();
        let mut parameters = PassParameters::new();
        parameters.insert("width", PassParameter::Unsigned(w));
        parameters.insert("height", PassParameter::Unsigned(h));
        parameters
    }

    fn with_grid(&self, parameters: &mut PassParameters) {
        let (kind, grid) = self.rectified.grid_uniforms();
        parameters.insert("grid_kind", PassParameter::Unsigned(kind));
        parameters.insert("grid", vec4(grid));
        parameters.insert(
            "geometry",
            PassParameter::Unsigned(self.rectified.rig.geometry.index()),
        );
        parameters.insert(
            "baseline",
            PassParameter::Number(self.rectified.baseline() as f64),
        );
    }

    /// Pixels of disparity, turned into the inverse metres the depth
    /// consumers compare ranges in.
    fn inverse_tolerance(&self, pixels: f32) -> f32 {
        pixels * self.rectified.disparity_step() / self.rectified.baseline()
    }

    pub(super) fn record(
        &self,
        stage: Stage,
        batch: &mut KernelBatch,
        motion: &Matrix4,
    ) -> Result<()> {
        let groups = self.image_groups();
        match stage {
            Stage::Depth => self.record_depth(batch, groups),
            Stage::Temporal => self.record_temporal(batch, motion, groups),
            Stage::Unrectify => self.record_unrectify(batch),
        }
    }

    fn record_depth(&self, batch: &mut KernelBatch, groups: WorkgroupGrid) -> Result<()> {
        let mut p = self.sized();
        self.with_grid(&mut p);
        p.insert(
            "disparity_step",
            PassParameter::Number(self.rectified.disparity_step() as f64),
        );
        p.insert(
            "max_range",
            PassParameter::Number(self.settings.max_range as f64),
        );
        p.insert(
            "min_disparity",
            PassParameter::Number(self.settings.min_disparity as f64),
        );
        p.insert("disparity", PassParameter::Buffer(self.disparity.clone()));
        p.insert("confidence", PassParameter::Buffer(self.confidence.clone()));
        p.insert("points", PassParameter::Buffer(self.points.clone()));
        p.insert("distance", PassParameter::Buffer(self.distance.clone()));
        batch.dispatch(&self.kernels.depth, &p, groups)?;
        Ok(())
    }

    fn record_temporal(
        &self,
        batch: &mut KernelBatch,
        motion: &Matrix4,
        groups: WorkgroupGrid,
    ) -> Result<()> {
        let Some(temporal) = self.temporal else {
            return Ok(());
        };
        let (read, write) = (self.front, 1 - self.front);
        let mut p = self.sized();
        self.with_grid(&mut p);
        p.insert(
            "previous_from_current",
            PassParameter::Mat4(mat4_rows(&invert_rigid(motion))),
        );
        p.insert(
            "current_from_previous",
            PassParameter::Mat4(mat4_rows(motion)),
        );
        // Disparity noise is even in inverse range, so the tolerance is
        // quoted in pixels and turned into inverse metres here.
        let inverse_tolerance = self.inverse_tolerance(temporal.tolerance);
        p.insert(
            "inverse_tolerance",
            PassParameter::Number(inverse_tolerance as f64),
        );
        p.insert("decay", PassParameter::Number(temporal.decay as f64));
        p.insert("max_age", PassParameter::Number(temporal.max_age as f64));
        p.insert("reset", PassParameter::Unsigned((self.frames == 0) as u32));
        p.insert("points", PassParameter::Buffer(self.points.clone()));
        p.insert("history", PassParameter::Buffer(self.history[read].clone()));
        p.insert("history_age", PassParameter::Buffer(self.age[read].clone()));
        p.insert(
            "next_history",
            PassParameter::Buffer(self.history[write].clone()),
        );
        p.insert("next_age", PassParameter::Buffer(self.age[write].clone()));
        p.insert(
            "filtered_distance",
            PassParameter::Buffer(self.filtered_distance.clone()),
        );
        batch.dispatch(&self.kernels.temporal, &p, groups)?;
        Ok(())
    }

    fn record_unrectify(&self, batch: &mut KernelBatch) -> Result<()> {
        let (w, h) = self.rectified.size();
        let Some(views) = &self.views else {
            return Ok(());
        };
        // What this frame produced: the temporal stage has written
        // the history buffer it did not read, and has not swapped yet.
        let points = if self.temporal.is_some() {
            &self.history[1 - self.front]
        } else {
            &self.points
        };
        let wraps = matches!(
            self.rectified.rectification.grid,
            Grid::Panorama { longitude, .. }
                if longitude[1] - longitude[0] > std::f32::consts::TAU - 1e-3
        );
        let tolerance = self.temporal.map_or(1.5, |t| t.tolerance);
        for eye in 0..2 {
            let (vw, vh) = views.sizes[eye];
            let (kind, a, b, c) = self.rectified.rig.eye(eye).projection.uniforms();
            let mut p = PassParameters::new();
            self.with_grid(&mut p);
            p.insert("width", PassParameter::Unsigned(vw));
            p.insert("height", PassParameter::Unsigned(vh));
            p.insert("grid_width", PassParameter::Unsigned(w));
            p.insert("grid_height", PassParameter::Unsigned(h));
            p.insert("projection_kind", PassParameter::Unsigned(kind));
            p.insert("eye", PassParameter::Unsigned(eye as u32));
            p.insert("projection_a", vec4(a));
            p.insert("projection_b", vec4(b));
            p.insert("projection_c", vec4(c));
            p.insert(
                "eye_from_grid",
                PassParameter::Mat4(mat4_rotation(&self.rectified.eye_from_grid(eye))),
            );
            p.insert(
                "inverse_tolerance",
                PassParameter::Number(self.inverse_tolerance(tolerance) as f64),
            );
            p.insert(
                "disparities",
                PassParameter::Unsigned(self.settings.max_disparity),
            );
            p.insert("wraps", PassParameter::Unsigned(wraps as u32));
            let (backdrop_range, backdrop_inverse, backdrop_confidence) = views
                .backdrop
                .map_or((0.0, 0.0, 0.0), |(range, pixels, confidence)| {
                    (range, self.inverse_tolerance(pixels), confidence)
                });
            p.insert(
                "backdrop_range",
                PassParameter::Number(backdrop_range as f64),
            );
            p.insert(
                "backdrop_inverse",
                PassParameter::Number(backdrop_inverse as f64),
            );
            p.insert(
                "backdrop_confidence",
                PassParameter::Number(backdrop_confidence as f64),
            );
            p.insert("points", PassParameter::Buffer(points.clone()));
            p.insert(
                "view_points",
                PassParameter::Buffer(views.points[eye].clone()),
            );
            p.insert(
                "view_distance",
                PassParameter::Buffer(views.distance[eye].clone()),
            );
            batch.dispatch(
                &self.kernels.unrectify,
                &p,
                ([vw.div_ceil(16), vh.div_ceil(16), 1]).into(),
            )?;
        }
        Ok(())
    }
}

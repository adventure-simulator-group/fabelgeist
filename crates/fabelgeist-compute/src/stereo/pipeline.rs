//! The stages after a match, one kernel each, recorded into one submit a frame.

use super::kernels::Kernels;
use super::rig::{Grid, Matrix3, Rectified};
use crate::kernel::KernelBatch;
use anyhow::{Result, ensure};
use fabelgeist_gpu::data::gpu::buffer::{Buffer, BufferByteLength, BufferDefinition};
use fabelgeist_gpu::data::gpu::parameters::{PassParameter, PassParameters};
use fabelgeist_gpu::data::matrix::Mat4;
use fabelgeist_gpu::data::vector::Vec4;
use fabelgeist_gpu::globals::WgpuContext;
use fabelgeist_gpu::prelude::WorkgroupGrid;
use fabelgeist_gpu::prelude::{BufferLabel, BufferUpload};

/// A 4x4 rigid transform, by rows.
pub type Matrix4 = [[f32; 4]; 4];

/// No motion at all: a rig on a tripod.
pub const STILL: Matrix4 = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// How disparity becomes distance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TriangulationSettings {
    /// The widest disparity the matcher reports, in pixels of the grid: how
    /// far along a row the right eye's view searches for its surface.
    pub max_disparity: u32,
    /// Where a disparity too small to tell from infinity is put, in metres.
    pub max_range: f32,
    pub min_disparity: f32,
}

impl Default for TriangulationSettings {
    fn default() -> Self {
        Self {
            max_disparity: 192,
            max_range: 1000.0,
            min_disparity: 0.05,
        }
    }
}

/// How the temporal filter trusts its history.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TemporalSettings {
    /// How far, in pixels of disparity, the history may be from the current
    /// frame and still be the same surface.
    pub tolerance: f32,
    /// What a frame without support takes off the history's confidence.
    pub decay: f32,
    /// The most frames of history one frame is weighed against.
    pub max_age: f32,
}

impl Default for TemporalSettings {
    fn default() -> Self {
        Self {
            tolerance: 1.5,
            decay: 0.85,
            max_age: 8.0,
        }
    }
}

/// A stage of the pipeline, in the order they run.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Stage {
    Depth,
    Temporal,
    Unrectify,
}

impl Stage {
    pub const ALL: [Self; 3] = [Self::Depth, Self::Temporal, Self::Unrectify];

    pub fn label(self) -> &'static str {
        match self {
            Self::Depth => "triangulate",
            Self::Temporal => "temporal",
            Self::Unrectify => "unrectify",
        }
    }
}

/// Depth resampled onto each eye's own picture.
struct Views {
    sizes: [(u32, u32); 2],
    points: [Buffer; 2],
    distance: [Buffer; 2],
    /// Metres, the pixels of disparity below which a point goes on it, and the
    /// confidence below which one does.
    backdrop: Option<(f32, f32, f32)>,
}

/// Depth from a stereo pair's disparity, on the card.
///
/// Matching is somebody else's: [`upload`](Self::upload) the disparity a
/// matcher found on the rectified grid, then [`run`](Self::run) it. Everything
/// is sized once, when this is made, so a frame allocates nothing.
pub struct StereoDepth {
    rectified: Rectified,
    settings: TriangulationSettings,
    temporal: Option<TemporalSettings>,
    kernels: Kernels,
    disparity: Buffer,
    confidence: Buffer,
    points: Buffer,
    distance: Buffer,
    history: [Buffer; 2],
    age: [Buffer; 2],
    filtered_distance: Buffer,
    views: Option<Views>,
    /// Which history buffer holds the latest frame.
    front: usize,
    frames: u64,
}

fn storage(
    context: &WgpuContext,
    label: BufferLabel,
    bytes: BufferByteLength,
) -> std::result::Result<Buffer, fabelgeist_gpu::prelude::BufferCreationError> {
    Buffer::new(
        context,
        bytes.max(4u64.into()),
        BufferDefinition::storage().with_label(label),
    )
}

fn mat4_rows(rows: &Matrix4) -> Mat4 {
    let mut columns = [[0.0; 4]; 4];
    for (r, row) in rows.iter().enumerate() {
        for (c, value) in row.iter().enumerate() {
            columns[c][r] = *value;
        }
    }
    Mat4 { columns }
}

fn mat4_rotation(m: &Matrix3) -> Mat4 {
    let mut rows = STILL;
    for r in 0..3 {
        for c in 0..3 {
            rows[r][c] = m[r][c];
        }
    }
    mat4_rows(&rows)
}

/// The inverse of a rigid transform.
pub fn invert_rigid(m: &Matrix4) -> Matrix4 {
    let mut out = STILL;
    for r in 0..3 {
        for c in 0..3 {
            out[r][c] = m[c][r];
        }
    }
    for row in &mut out[..3] {
        row[3] = -(0..3).map(|c| row[c] * m[c][3]).sum::<f32>();
    }
    out
}

fn vec4(v: [f32; 4]) -> PassParameter {
    PassParameter::Vec4(Vec4::new(v[0], v[1], v[2], v[3]))
}

impl StereoDepth {
    pub fn new(
        context: &WgpuContext,
        rectified: Rectified,
        settings: TriangulationSettings,
        temporal: Option<TemporalSettings>,
    ) -> Result<Self> {
        ensure!(
            settings.max_disparity >= 1,
            "a matcher that reports no disparity measures nothing"
        );
        let (w, h) = rectified.size();
        let pixels = w as u64 * h as u64;
        let float = pixels * 4;
        let make = |label: BufferLabel,
                    bytes: BufferByteLength|
         -> std::result::Result<
            Buffer,
            fabelgeist_gpu::prelude::BufferCreationError,
        > { storage(context, label, bytes) };
        Ok(Self {
            kernels: Kernels::new(context)?,
            disparity: make(("stereo disparity").into(), (float).into())?,
            confidence: make(("stereo confidence").into(), (float).into())?,
            points: make(("stereo points").into(), (pixels * 16).into())?,
            distance: make(("stereo distance").into(), (float).into())?,
            history: [
                make(("stereo history a").into(), (pixels * 16).into())?,
                make(("stereo history b").into(), (pixels * 16).into())?,
            ],
            age: [
                make(("stereo age a").into(), (float).into())?,
                make(("stereo age b").into(), (float).into())?,
            ],
            filtered_distance: make(("stereo filtered distance").into(), (float).into())?,
            views: None,
            front: 0,
            frames: 0,
            rectified,
            settings,
            temporal,
        })
    }

    /// Also put the depth back on each eye's own picture every frame:
    /// `sizes[eye]` pixels covering that eye's region, in its own projection.
    pub fn with_views(mut self, context: &WgpuContext, sizes: [(u32, u32); 2]) -> Result<Self> {
        for (w, h) in sizes {
            ensure!(
                w >= 1 && h >= 1 && (w as u64 * h as u64) < (u32::MAX / 16) as u64,
                "an eye's view of {w}x{h} cannot be made"
            );
        }
        let pixels = sizes.map(|(w, h)| w as u64 * h as u64);
        let make = |label: BufferLabel,
                    bytes: BufferByteLength|
         -> std::result::Result<
            Buffer,
            fabelgeist_gpu::prelude::BufferCreationError,
        > { storage(context, label, bytes) };
        self.views = Some(Views {
            sizes,
            points: [
                make(("stereo view points left").into(), (pixels[0] * 16).into())?,
                make(("stereo view points right").into(), (pixels[1] * 16).into())?,
            ],
            distance: [
                make(("stereo view distance left").into(), (pixels[0] * 4).into())?,
                make(
                    ("stereo view distance right").into(),
                    (pixels[1] * 4).into(),
                )?,
            ],
            backdrop: None,
        });
        Ok(self)
    }

    /// Give every pixel of the views a point: where the pair has no
    /// trustworthy depth -- rejected, outside the grid, under `min_disparity`
    /// pixels of disparity, which a short baseline cannot tell from infinity,
    /// or matched with less than `min_confidence` -- the pixel is put `range`
    /// metres out along its own ray, with a confidence of 0.001 that says so.
    /// Its distance stays what was measured, or zero.
    ///
    /// For showing a clip rather than measuring it: the measurable foreground
    /// stands in front of the whole picture instead of in front of holes, and
    /// seen from a little to the side, a far surface stays a picture rather
    /// than smearing along the rays its depth is uncertain along.
    pub fn with_backdrop(mut self, range: f32, min_disparity: f32, min_confidence: f32) -> Self {
        if let Some(views) = self.views.as_mut() {
            views.backdrop = Some((range, min_disparity, min_confidence));
        }
        self
    }

    pub fn rectified(&self) -> &Rectified {
        &self.rectified
    }

    pub fn settings(&self) -> &TriangulationSettings {
        &self.settings
    }

    /// Bytes held on the card.
    pub fn memory(&self) -> BufferByteLength {
        let buffers = [
            &self.disparity,
            &self.confidence,
            &self.points,
            &self.distance,
            &self.history[0],
            &self.history[1],
            &self.age[0],
            &self.age[1],
            &self.filtered_distance,
        ];
        let mut bytes = BufferByteLength::default();
        for buffer in buffers {
            bytes = bytes + buffer.length();
        }
        if let Some(views) = &self.views {
            for buffer in views.points.iter().chain(&views.distance) {
                bytes = bytes + buffer.length();
            }
        }
        bytes
    }

    /// Hand over one frame's match, row-major over the rectified grid:
    /// `disparity` in pixels, the left pixel at `x` seeing what the right one
    /// sees at `x - d`, and negative where there is none; `confidence` above
    /// zero wherever there is one.
    pub fn upload(
        &mut self,
        context: &WgpuContext,
        disparity: &[f32],
        confidence: &[f32],
    ) -> Result<()> {
        let (w, h) = self.rectified.size();
        let pixels = w as usize * h as usize;
        ensure!(
            disparity.len() == pixels && confidence.len() == pixels,
            "a {w}x{h} grid takes {pixels} disparities and confidences, not {} and {}",
            disparity.len(),
            confidence.len()
        );
        self.disparity
            .write(context, BufferUpload::from_elements(disparity));
        self.confidence
            .write(context, BufferUpload::from_elements(confidence));
        Ok(())
    }

    fn image_groups(&self) -> WorkgroupGrid {
        let (w, h) = self.rectified.size();
        WorkgroupGrid::from([w.div_ceil(16), h.div_ceil(16), 1])
    }

    fn sized(&self) -> PassParameters {
        let (w, h) = self.rectified.size();
        PassParameters::from([("width".into(), w.into()), ("height".into(), h.into())])
    }

    fn with_grid(&self, parameters: &mut PassParameters) {
        let (kind, grid) = self.rectified.grid_uniforms();
        parameters.overlay(PassParameters::from([
            ("grid_kind".into(), (kind).into()),
            ("grid".into(), vec4(grid)),
            (
                "geometry".into(),
                (self.rectified.rig.geometry.index()).into(),
            ),
            ("baseline".into(), (self.rectified.baseline() as f64).into()),
        ]));
    }

    /// Pixels of disparity, turned into the inverse metres the depth
    /// consumers compare ranges in.
    fn inverse_tolerance(&self, pixels: f32) -> f32 {
        pixels * self.rectified.disparity_step() / self.rectified.baseline()
    }

    fn record(&self, stage: Stage, batch: &mut KernelBatch, motion: &Matrix4) -> Result<()> {
        let groups = self.image_groups();
        let s = &self.settings;
        let k = &self.kernels;
        match stage {
            Stage::Depth => {
                let mut p = self.sized();
                self.with_grid(&mut p);
                p.overlay(PassParameters::from([
                    (
                        "disparity_step".into(),
                        (self.rectified.disparity_step() as f64).into(),
                    ),
                    ("max_range".into(), (s.max_range as f64).into()),
                    ("min_disparity".into(), (s.min_disparity as f64).into()),
                    ("disparity".into(), (self.disparity.clone()).into()),
                    ("confidence".into(), (self.confidence.clone()).into()),
                    ("points".into(), (self.points.clone()).into()),
                    ("distance".into(), (self.distance.clone()).into()),
                ]));
                batch.dispatch(&k.depth, &p, groups)?;
            }
            Stage::Temporal => {
                let Some(temporal) = self.temporal else {
                    return Ok(());
                };
                let (read, write) = (self.front, 1 - self.front);
                let mut p = self.sized();
                self.with_grid(&mut p);
                p.overlay(PassParameters::from([
                    (
                        "previous_from_current".into(),
                        (mat4_rows(&invert_rigid(motion))).into(),
                    ),
                    ("current_from_previous".into(), (mat4_rows(motion)).into()),
                ]));
                // Disparity noise is even in inverse range, so the tolerance is
                // quoted in pixels and turned into inverse metres here.
                let inverse_tolerance = self.inverse_tolerance(temporal.tolerance);
                p.overlay(PassParameters::from([
                    (
                        "inverse_tolerance".into(),
                        (inverse_tolerance as f64).into(),
                    ),
                    ("decay".into(), (temporal.decay as f64).into()),
                    ("max_age".into(), (temporal.max_age as f64).into()),
                    ("reset".into(), ((self.frames == 0) as u32).into()),
                    ("points".into(), (self.points.clone()).into()),
                    ("history".into(), (self.history[read].clone()).into()),
                    ("history_age".into(), (self.age[read].clone()).into()),
                    ("next_history".into(), (self.history[write].clone()).into()),
                    ("next_age".into(), (self.age[write].clone()).into()),
                    (
                        "filtered_distance".into(),
                        (self.filtered_distance.clone()).into(),
                    ),
                ]));
                batch.dispatch(&k.temporal, &p, groups)?;
            }
            Stage::Unrectify => self.record_unrectified_views(batch)?,
        }
        Ok(())
    }

    fn record_unrectified_views(
        &self,
        batch: &mut KernelBatch,
    ) -> std::result::Result<(), crate::KernelDispatchError> {
        let (w, h) = self.rectified.size();
        let s = &self.settings;
        let k = &self.kernels;
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
            p.overlay(PassParameters::from([
                ("width".into(), (vw).into()),
                ("height".into(), (vh).into()),
                ("grid_width".into(), (w).into()),
                ("grid_height".into(), (h).into()),
                ("projection_kind".into(), (kind).into()),
                ("eye".into(), (eye as u32).into()),
                ("projection_a".into(), vec4(a)),
                ("projection_b".into(), vec4(b)),
                ("projection_c".into(), vec4(c)),
                (
                    "eye_from_grid".into(),
                    (mat4_rotation(&self.rectified.eye_from_grid(eye))).into(),
                ),
                (
                    "inverse_tolerance".into(),
                    (self.inverse_tolerance(tolerance) as f64).into(),
                ),
                ("disparities".into(), (s.max_disparity).into()),
                ("wraps".into(), (wraps as u32).into()),
            ]));
            let (backdrop_range, backdrop_inverse, backdrop_confidence) = views
                .backdrop
                .map_or((0.0, 0.0, 0.0), |(range, pixels, confidence)| {
                    (range, self.inverse_tolerance(pixels), confidence)
                });
            p.overlay(PassParameters::from([
                ("backdrop_range".into(), (backdrop_range as f64).into()),
                ("backdrop_inverse".into(), (backdrop_inverse as f64).into()),
                (
                    "backdrop_confidence".into(),
                    (backdrop_confidence as f64).into(),
                ),
                ("points".into(), (points.clone()).into()),
                ("view_points".into(), (views.points[eye].clone()).into()),
                ("view_distance".into(), (views.distance[eye].clone()).into()),
            ]));
            batch.dispatch(
                &k.unrectify,
                &p,
                [vw.div_ceil(16), vh.div_ceil(16), 1].into(),
            )?;
        }

        Ok(())
    }

    /// Run every stage over the uploaded disparity, in one submit.
    ///
    /// `motion` takes the previous frame's grid to this one's; None is a rig
    /// that has not moved. Nothing waits for the card: read an output, or
    /// wait on the queue, to know it is done.
    pub fn run(&mut self, context: &WgpuContext, motion: Option<Matrix4>) -> Result<()> {
        let motion = motion.unwrap_or(STILL);
        let mut batch = KernelBatch::labelled(context, ("stereo depth").into());
        for stage in Stage::ALL {
            self.record(stage, &mut batch, &motion)?;
        }
        batch.submit();
        if self.temporal.is_some() {
            self.front = 1 - self.front;
        }
        self.frames += 1;
        Ok(())
    }

    /// Forget the history, as after a cut or a seek.
    pub fn reset_history(&mut self) {
        self.frames = 0;
    }

    pub fn frames(&self) -> u64 {
        self.frames
    }

    /// The disparity last uploaded. Negative: none.
    pub fn disparity(&self) -> &Buffer {
        &self.disparity
    }

    pub fn confidence(&self) -> &Buffer {
        &self.confidence
    }

    /// `vec4(x, y, z, confidence)` in the grid frame; confidence zero where
    /// there is no point.
    pub fn points(&self) -> &Buffer {
        &self.points
    }

    /// Metres along each pixel's ray; zero where there is no point.
    pub fn distance(&self) -> &Buffer {
        &self.distance
    }

    /// The points after the temporal filter, or before it when there is none.
    pub fn filtered_points(&self) -> &Buffer {
        if self.temporal.is_some() {
            &self.history[self.front]
        } else {
            &self.points
        }
    }

    pub fn filtered_distance(&self) -> &Buffer {
        if self.temporal.is_some() {
            &self.filtered_distance
        } else {
            &self.distance
        }
    }

    /// How many pixels `eye`'s own view has, if views were asked for.
    pub fn view_size(&self, eye: usize) -> Option<(u32, u32)> {
        self.views.as_ref().map(|views| views.sizes[eye.min(1)])
    }

    /// `vec4(x, y, z, confidence)` for each pixel of `eye`'s own picture, in
    /// that eye's camera and measured from its own centre; confidence zero
    /// where there is no depth.
    pub fn view_points(&self, eye: usize) -> Option<&Buffer> {
        self.views.as_ref().map(|views| &views.points[eye.min(1)])
    }

    /// Metres from `eye`'s centre along each pixel of its own picture; zero
    /// where there is no depth.
    pub fn view_distance(&self, eye: usize) -> Option<&Buffer> {
        self.views.as_ref().map(|views| &views.distance[eye.min(1)])
    }
}

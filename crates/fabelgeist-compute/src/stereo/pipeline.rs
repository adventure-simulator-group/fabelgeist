//! The stages after a match, one kernel each, recorded into one submit a frame.

use super::rig::{Matrix3, Rectified};
use crate::kernel::{Kernel, KernelBatch};
use anyhow::{Result, anyhow, ensure};
use fabelgeist_gpu::data::gpu::buffer::{Buffer, BufferDefinition};
use fabelgeist_gpu::data::gpu::parameters::PassParameter;
use fabelgeist_gpu::data::matrix::Mat4;
use fabelgeist_gpu::data::vector::Vec4;
use fabelgeist_gpu::globals::WgpuContext;
use fabelgeist_gpu::prelude::BufferUpload;

mod recording;

/// A 4x4 rigid transform, by rows.
pub type Matrix4 = [[f32; 4]; 4];

/// No motion at all: a rig on a tripod.
pub const STILL: Matrix4 = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

const COMMON: &str = include_str!("common.wgsl");

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

struct Kernels {
    depth: Kernel,
    temporal: Kernel,
    unrectify: Kernel,
}

impl Kernels {
    fn new(context: &WgpuContext) -> Result<Self> {
        let with_rays = |source: &str| format!("{COMMON}\n{source}");
        let compile = |name: &str, code: String| {
            Kernel::new(context, code)
                .map_err(|error| anyhow!("compiling the {name} kernel: {error:#}"))
        };
        Ok(Self {
            depth: compile("depth", with_rays(include_str!("depth.wgsl")))?,
            temporal: compile("temporal", with_rays(include_str!("temporal.wgsl")))?,
            unrectify: compile("unrectify", with_rays(include_str!("unrectify.wgsl")))?,
        })
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

fn storage(context: &WgpuContext, label: &str, bytes: u64) -> Result<Buffer> {
    Buffer::new(
        context,
        (bytes.max(4)).into(),
        BufferDefinition::storage().with_label((label).into()),
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
        let make = |label: &str, bytes: u64| storage(context, label, bytes);
        Ok(Self {
            kernels: Kernels::new(context)?,
            disparity: make("stereo disparity", float)?,
            confidence: make("stereo confidence", float)?,
            points: make("stereo points", pixels * 16)?,
            distance: make("stereo distance", float)?,
            history: [
                make("stereo history a", pixels * 16)?,
                make("stereo history b", pixels * 16)?,
            ],
            age: [make("stereo age a", float)?, make("stereo age b", float)?],
            filtered_distance: make("stereo filtered distance", float)?,
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
        let make = |label: &str, bytes: u64| storage(context, label, bytes);
        self.views = Some(Views {
            sizes,
            points: [
                make("stereo view points left", pixels[0] * 16)?,
                make("stereo view points right", pixels[1] * 16)?,
            ],
            distance: [
                make("stereo view distance left", pixels[0] * 4)?,
                make("stereo view distance right", pixels[1] * 4)?,
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
    pub fn memory(&self) -> u64 {
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
        let views: u64 = self
            .views
            .iter()
            .flat_map(|views| views.points.iter().chain(&views.distance))
            .map(|buffer| u64::from(buffer.size))
            .sum();
        buffers
            .iter()
            .map(|buffer| u64::from(buffer.size))
            .sum::<u64>()
            + views
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
            .write(context, BufferUpload::from_elements(disparity))?;
        self.confidence
            .write(context, BufferUpload::from_elements(confidence))?;
        Ok(())
    }

    /// Run every stage over the uploaded disparity, in one submit.
    ///
    /// `motion` takes the previous frame's grid to this one's; None is a rig
    /// that has not moved. Nothing waits for the card: read an output, or
    /// wait on the queue, to know it is done.
    pub fn run(&mut self, context: &WgpuContext, motion: Option<Matrix4>) -> Result<()> {
        let motion = motion.unwrap_or(STILL);
        let mut batch = KernelBatch::labelled(context, "stereo depth");
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

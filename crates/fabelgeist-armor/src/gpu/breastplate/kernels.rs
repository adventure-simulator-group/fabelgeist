//! Compiling and dispatching the breastplate's kernels.
//!
//! Every kernel shares one parameter block and one prelude: the layout
//! constants, the exactly rounded arithmetic of [`host_float`] with the
//! `unit` normalization, and -- for a kernel that binds the plate's design
//! and wearer -- the authored shape and the fit's profile evaluation.

use fabelgeist_gpu::prelude::PassParameterName;
use std::sync::Arc;

use fabelgeist_compute::{Kernel, KernelBatch, host_float};
use fabelgeist_gpu::prelude::{Buffer, PassParameters, ShaderSource};

use super::carrier_wgsl::COARSE_WORDS;
use super::finish_wgsl::{CARRIER_WORDS, UPPER_RIM_ROWS};
use super::fit::ITERATIONS;
use super::fit_wgsl::{self, CENTER_WORDS, PREPARED_WORDS, RESIDUAL_WORDS};
use super::shape_wgsl;
use super::skin_wgsl::{MORPH_WORDS, SAMPLE_WORDS, SKIN_WORDS};
use super::topology::{SKIRT_SAMPLES, U_SAMPLES, V_SAMPLES};
use crate::GenerateError;
use crate::gpu::anatomy::SURFACE_HEADER;
use crate::gpu::{ArmorGpu, wgsl};

/// The pass parameters every breastplate kernel takes.
#[derive(Clone, Copy, Default)]
pub(super) struct Params {
    pub count: u32,
    pub width: u32,
    pub rear: bool,
    pub side: u32,
    pub torso_count: u32,
    pub iteration: u32,
    pub front_count: u32,
    pub extra: u32,
}

impl Params {
    pub(super) fn counted(count: u32) -> Self {
        Self {
            count,
            ..Self::default()
        }
    }

    fn parameters(self) -> PassParameters {
        let mut parameters = PassParameters::new();
        parameters.insert("count".into(), (self.count).into());
        parameters.insert("width".into(), (self.width).into());
        parameters.insert("rear".into(), (u32::from(self.rear)).into());
        parameters.insert("side".into(), (self.side).into());
        parameters.insert("torso_count".into(), (self.torso_count).into());
        parameters.insert("iteration".into(), (self.iteration).into());
        parameters.insert("front_count".into(), (self.front_count).into());
        parameters.insert("extra".into(), (self.extra).into());
        parameters.insert("zero".into(), (0u32).into());
        parameters
    }
}

/// A kernel's buffers of points, and whether it writes each.
pub(super) type Points<'a> = &'a [(&'a str, bool)];

/// Compile a breastplate kernel: its entry after the shared prelude, with
/// the shape when it binds `plate` and `fail` when it binds `status`.
fn kernel(gpu: &ArmorGpu, entry: &str, points: Points) -> Result<Arc<Kernel>, GenerateError> {
    let shape = entry.contains("var<storage, read> plate")
        || entry.contains("var<storage, read_write> plate");
    let status = entry.contains("var<storage, read_write> status");
    let accessors = points
        .iter()
        .map(|(name, writes)| {
            if *writes {
                wgsl::points(name)
            } else {
                wgsl::read_points(name)
            }
        })
        .collect::<String>();
    let source = format!(
        r#"
struct Params {{
    count: u32,
    width: u32,
    rear: u32,
    side: u32,
    torso_count: u32,
    iteration: u32,
    front_count: u32,
    extra: u32,
    // Always zero; `host_zero` reads it.
    zero: u32,
}};
const HEADER: u32 = {header}u;
const U_SAMPLES: u32 = {u_samples}u;
const V_SAMPLES: u32 = {v_samples}u;
const SKIRT_SAMPLES: u32 = {skirt_samples}u;
const CENTER_WORDS: u32 = {center_words}u;
const PREPARED_WORDS: u32 = {prepared_words}u;
const RESIDUAL_WORDS: u32 = {residual_words}u;
const SAMPLE_WORDS: u32 = {sample_words}u;
const CARRIER_WORDS: u32 = {carrier_words}u;
const MORPH_WORDS: u32 = {morph_words}u;
const SKIN_WORDS: u32 = {skin_words}u;
const COARSE_WORDS: u32 = {coarse_words}u;
const UPPER_RIM_ROWS: u32 = {upper_rim_rows}u;
const LAST_ITERATION: u32 = {last_iteration}u;
// Padding reserve for interpolation and negative identity-morph blends.
const FIT_SURFACE_MARGIN: f32 = 0.006;
const MAX_FIT_CORRECTION: f32 = 0.060;
{math}
{ordered}
{status_code}
{zero_hook}
{host_float}
{unit}
{shape_code}
{fit_common}
{accessors}
{entry}
"#,
        header = SURFACE_HEADER,
        u_samples = U_SAMPLES,
        v_samples = V_SAMPLES,
        skirt_samples = SKIRT_SAMPLES,
        center_words = CENTER_WORDS,
        prepared_words = PREPARED_WORDS,
        residual_words = RESIDUAL_WORDS,
        sample_words = SAMPLE_WORDS,
        carrier_words = CARRIER_WORDS,
        morph_words = MORPH_WORDS,
        skin_words = SKIN_WORDS,
        coarse_words = COARSE_WORDS,
        upper_rim_rows = UPPER_RIM_ROWS,
        last_iteration = ITERATIONS - 1,
        math = wgsl::MATH,
        ordered = wgsl::ORDERED_FLOAT,
        status_code = if status { wgsl::STATUS } else { "" },
        zero_hook = host_float::PARAMS_ZERO_HOOK,
        host_float = host_float::wgsl(),
        unit = shape_wgsl::UNIT,
        shape_code = if shape { shape_wgsl::SHAPE } else { "" },
        fit_common = if shape { fit_wgsl::FIT_COMMON } else { "" },
    );
    gpu.cache()
        .get(gpu.context(), &ShaderSource::from(source))
        .map_err(crate::GenerateError::from)
}

/// Record one dispatch of a breastplate kernel over `items` invocations,
/// or `items` workgroups of a single-invocation kernel.
pub(super) fn dispatch(
    gpu: &ArmorGpu,
    batch: &mut KernelBatch,
    entry: &str,
    points: Points,
    params: Params,
    buffers: &[(&str, &Buffer)],
    items: fabelgeist_gpu::prelude::InvocationCount,
) -> Result<(), GenerateError> {
    let mut parameters = params.parameters();
    for (name, buffer) in buffers {
        parameters.insert(PassParameterName::from(*name), ((*buffer).clone()).into());
    }
    let kernel = kernel(gpu, entry, points)?;
    batch
        .dispatch_items(&kernel, &parameters, items)
        .map_err(GenerateError::from)?;
    Ok(())
}

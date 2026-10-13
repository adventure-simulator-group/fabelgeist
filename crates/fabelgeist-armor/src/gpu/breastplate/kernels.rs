//! Compiling and dispatching the breastplate's kernels.
//!
//! Every kernel shares one parameter block and one prelude: the layout
//! constants, the exactly rounded arithmetic of [`host_float`] with the
//! `unit` normalization, and -- for a kernel that binds the plate's design
//! and wearer -- the authored shape and local torso support.

use std::sync::Arc;

use fabelgeist_compute::{Kernel, KernelBatch, host_float};
use fabelgeist_gpu::prelude::{Buffer, PassParameters};

use super::carrier_wgsl::COARSE_WORDS;
use super::finish_wgsl::CARRIER_WORDS;
use super::fit::RadialFit;
use super::fit_wgsl::{self, CENTER_WORDS};
use super::shape_wgsl;
use super::skin_wgsl::{MORPH_WORDS, SAMPLE_WORDS, SKIN_WORDS};
use super::topology::{SKIRT_SAMPLES, U_SAMPLES, V_SAMPLES};
use crate::GenerateError;
use crate::gpu::anatomy::SURFACE_HEADER;
use crate::gpu::{ArmorGpu, device_error, wgsl};

/// The pass parameters every breastplate kernel takes.
#[derive(Clone, Copy, Default)]
pub(super) struct Params {
    pub count: u32,
    pub width: u32,
    pub rear: bool,
    pub side: u32,
    pub torso_count: u32,
    pub radial_fit: RadialFit,
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
        parameters.insert("count", self.count);
        parameters.insert("width", self.width);
        parameters.insert("rear", u32::from(self.rear));
        parameters.insert("side", self.side);
        parameters.insert("torso_count", self.torso_count);
        parameters.insert("radial_fit", self.radial_fit as u32);
        parameters.insert("front_count", self.front_count);
        parameters.insert("extra", self.extra);
        parameters.insert("zero", 0u32);
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
    radial_fit: u32,
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
const SAMPLE_WORDS: u32 = {sample_words}u;
const CARRIER_WORDS: u32 = {carrier_words}u;
const MORPH_WORDS: u32 = {morph_words}u;
const SKIN_WORDS: u32 = {skin_words}u;
const COARSE_WORDS: u32 = {coarse_words}u;
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
{fit_neighbors}
{accessors}
{entry}
"#,
        header = SURFACE_HEADER,
        u_samples = U_SAMPLES,
        v_samples = V_SAMPLES,
        skirt_samples = SKIRT_SAMPLES,
        center_words = CENTER_WORDS,
        sample_words = SAMPLE_WORDS,
        carrier_words = CARRIER_WORDS,
        morph_words = MORPH_WORDS,
        skin_words = SKIN_WORDS,
        coarse_words = COARSE_WORDS,
        math = wgsl::MATH,
        ordered = wgsl::ORDERED_FLOAT,
        status_code = if status { wgsl::STATUS } else { "" },
        zero_hook = host_float::PARAMS_ZERO_HOOK,
        host_float = host_float::wgsl(),
        unit = shape_wgsl::UNIT,
        shape_code = if shape { shape_wgsl::SHAPE } else { "" },
        fit_common = if shape { fit_wgsl::FIT_COMMON } else { "" },
        fit_neighbors = fit_wgsl::NEIGHBORS,
    );
    gpu.cache()
        .get(gpu.context(), &source)
        .map_err(device_error)
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
        parameters.insert(*name, (*buffer).clone());
    }
    let kernel = kernel(gpu, entry, points)?;
    batch
        .dispatch_items(&kernel, &parameters, items)
        .map_err(device_error)?;
    Ok(())
}

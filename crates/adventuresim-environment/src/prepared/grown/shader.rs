//! Compose the same geometry libraries used by drawing, with a checked ABI.

use std::num::NonZeroU32;

use anyhow::{Context, Result, bail, ensure};
use fabelgeist_botany::viewer::sim::{BranchInstance, LeafInstance};
use fabelgeist_gpu::prelude::{ComputePipeline, ComputeShader, WgpuContext};
use naga_oil::compose::{ComposableModuleDescriptor, Composer, NagaModuleDescriptor};

use super::gpu::{ExpandedVertex, Params};
use crate::render::ForestPart;

const FOREST: &str = include_str!("../../render/shaders/forest_common.wgsl");
const BRANCH_GEOMETRY: &str = include_str!("../../render/shaders/branch_geometry.wgsl");
const BRANCH_SURFACE: &str = include_str!("../../render/shaders/branch_surface.wgsl");
const LEAF_GEOMETRY: &str = include_str!("../../render/shaders/leaf_geometry.wgsl");
const COMMON: &str = include_str!("shaders/common.wgsl");
const BRANCH: &str = include_str!("shaders/branch.wgsl");
const LEAF: &str = include_str!("shaders/leaf.wgsl");

pub(super) struct Kernel {
    pub pipeline: ComputePipeline,
    pub width: NonZeroU32,
}

impl Kernel {
    pub fn new(context: &WgpuContext, role: ForestPart) -> Result<Self> {
        let mut composer = Composer::default();
        for (source, path) in [
            (FOREST, "forest_common.wgsl"),
            (BRANCH_GEOMETRY, "branch_geometry.wgsl"),
            (BRANCH_SURFACE, "branch_surface.wgsl"),
            (LEAF_GEOMETRY, "leaf_geometry.wgsl"),
            (COMMON, "grown_expansion.wgsl"),
        ] {
            if let Err(error) = composer.add_composable_module(ComposableModuleDescriptor {
                source,
                file_path: path,
                ..Default::default()
            }) {
                bail!("{}", error.emit_to_string(&composer));
            }
        }
        let source = match role {
            ForestPart::Branches => BRANCH,
            ForestPart::Leaves => LEAF,
        };
        let module = composer
            .make_naga_module(NagaModuleDescriptor {
                source,
                file_path: "grown_compute.wgsl",
                ..Default::default()
            })
            .map_err(|error| anyhow::anyhow!(error.emit_to_string(&composer)))?;
        let width = validate_interface(&module, role)?;
        let limits = context.device.limits();
        ensure!(
            width.get() <= limits.max_compute_workgroup_size_x
                && width.get() <= limits.max_compute_invocations_per_workgroup,
            "grown workgroup exceeds device limits"
        );
        let info = wgpu::naga::valid::Validator::new(
            wgpu::naga::valid::ValidationFlags::all(),
            wgpu::naga::valid::Capabilities::all(),
        )
        .validate(&module)?;
        let code = wgpu::naga::back::wgsl::write_string(
            &module,
            &info,
            wgpu::naga::back::wgsl::WriterFlags::empty(),
        )?;
        let shader = ComputeShader::new(context, code)?;
        let pipeline = ComputePipeline::new(context, shader)?;
        ensure!(
            pipeline.bind_group_layouts.len() == 1,
            "unexpected grown bind group count"
        );
        Ok(Self { pipeline, width })
    }
}

fn validate_interface(module: &wgpu::naga::Module, role: ForestPart) -> Result<NonZeroU32> {
    let [entry] = module.entry_points.as_slice() else {
        bail!("grown kernel needs one entry point");
    };
    ensure!(
        entry.stage == wgpu::naga::ShaderStage::Compute && entry.workgroup_size[1..] == [1, 1],
        "invalid grown dispatch shape"
    );
    let width = NonZeroU32::new(entry.workgroup_size[0]).context("zero grown workgroup width")?;
    ensure!(
        module.global_variables.len() == 3,
        "unexpected grown global binding count"
    );
    let input_stride = match role {
        ForestPart::Branches => size_of::<BranchInstance>(),
        ForestPart::Leaves => size_of::<LeafInstance>(),
    };
    for (binding, expected) in [(0, input_stride), (2, size_of::<ExpandedVertex>())] {
        let variable = binding_variable(module, binding)?;
        let wgpu::naga::TypeInner::Array { stride, .. } = module.types[variable.ty].inner else {
            bail!("grown storage binding is not an array");
        };
        ensure!(
            usize::try_from(stride)? == expected,
            "grown storage stride mismatch"
        );
    }
    let uniform = binding_variable(module, 1)?;
    let wgpu::naga::TypeInner::Struct { ref members, span } = module.types[uniform.ty].inner else {
        bail!("grown params are not a struct");
    };
    ensure!(
        usize::try_from(span)? == size_of::<Params>()
            && members.iter().map(|member| member.offset).eq([
                u32::try_from(std::mem::offset_of!(Params, shape))?,
                u32::try_from(std::mem::offset_of!(Params, span))?,
            ]),
        "grown uniform layout mismatch"
    );
    Ok(width)
}

fn binding_variable(
    module: &wgpu::naga::Module,
    binding: u32,
) -> Result<&wgpu::naga::GlobalVariable> {
    module
        .global_variables
        .iter()
        .map(|(_, variable)| variable)
        .find(|variable| {
            variable
                .binding
                .as_ref()
                .is_some_and(|slot| slot.group == 0 && slot.binding == binding)
        })
        .context("missing grown kernel binding")
}

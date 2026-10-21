//! Record only after the original ordinary DrawMaterial tuple succeeds.
use super::*;
use bevy::{
    asset::UntypedAssetId,
    ecs::system::{SystemParamItem, lifetimeless::SRes},
    pbr::{PreparedMaterial, RenderMaterialInstances, RenderMeshInstances},
    render::{
        erased_render_asset::ErasedRenderAssets,
        mesh::{RenderMesh, allocator::MeshAllocator},
        render_asset::RenderAssets,
        render_phase::{
            CachedRenderPipelinePhaseItem, PhaseItem, PhaseItemExtraIndex, RenderCommand,
            RenderCommandResult, TrackedRenderPass,
        },
        render_resource::CachedRenderPipelineId,
        view::ExtractedView,
    },
};
use bindings::MeshBinding;
use std::ops::Range;
use types::{ColorPhase, RenderFrameId};

const MAX_COLOR_COMMANDS: usize = 4096;

pub(super) struct Record;

pub(super) trait ObservedPhase: PhaseItem + CachedRenderPipelinePhaseItem {
    const PHASE: ColorPhase;
}

#[derive(Clone, Debug)]
pub(super) struct Receipt {
    pub phase: ColorPhase,
    pub frame: RenderFrameId,
    pub camera: RetainedViewEntity,
    pub pipeline: CachedRenderPipelineId,
    pub representative: Entity,
    pub material: UntypedAssetId,
    pub material_group: u32,
    pub binding: MeshBinding,
    pub dispatch: Dispatch,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Dispatch {
    Direct {
        instances: Range<u32>,
    },
    DynamicUniform {
        instances: Range<u32>,
        offset: u32,
    },
    Indirect {
        parameters: Range<u32>,
        count: CommandCount,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CommandCount {
    Range,
    BatchSet(u32),
}

impl ObservedPhase for Opaque3d {
    const PHASE: ColorPhase = ColorPhase::Opaque;
}
impl ObservedPhase for AlphaMask3d {
    const PHASE: ColorPhase = ColorPhase::AlphaMask;
}

impl<P: ObservedPhase> RenderCommand<P> for Record {
    type Param = (
        SRes<Frame>,
        SRes<RenderMeshInstances>,
        SRes<RenderMaterialInstances>,
        SRes<ErasedRenderAssets<PreparedMaterial>>,
        SRes<RenderAssets<RenderMesh>>,
        SRes<MeshAllocator>,
    );
    type ViewQuery = &'static ExtractedView;
    type ItemQuery = ();

    fn render<'w>(
        item: &P,
        view: &'w ExtractedView,
        _entity: Option<()>,
        params: SystemParamItem<'w, '_, Self::Param>,
        _pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let (frame, instances, materials, prepared, meshes, allocator) = params;
        let frame = frame.into_inner();
        let Some(pose) = &frame.pose else {
            return RenderCommandResult::Success;
        };
        if pose.camera != view.retained_view_entity {
            return RenderCommandResult::Success;
        }
        let instances = instances.into_inner();
        let materials = materials.into_inner();
        let prepared = prepared.into_inner();
        let meshes = meshes.into_inner();
        let allocator = allocator.into_inner();
        let Some(mesh_id) = instances.mesh_asset_id(item.main_entity()) else {
            pose.fail("successful draw has no actual mesh");
            return RenderCommandResult::Success;
        };
        let Some(mesh) = meshes.get(mesh_id) else {
            pose.fail("successful draw has no prepared mesh");
            return RenderCommandResult::Success;
        };
        let Some(binding) = MeshBinding::read(mesh_id, mesh, allocator) else {
            pose.fail("successful draw has no allocation");
            return RenderCommandResult::Success;
        };
        let Some(material) = materials.instances.get(&item.main_entity()) else {
            pose.fail("successful draw has no material instance");
            return RenderCommandResult::Success;
        };
        let Some(actual_material) = prepared.get(material.asset_id) else {
            pose.fail("successful draw has no prepared material");
            return RenderCommandResult::Success;
        };
        let dispatch = match item.extra_index() {
            PhaseItemExtraIndex::None => Dispatch::Direct {
                instances: item.batch_range().clone(),
            },
            PhaseItemExtraIndex::DynamicOffset(offset) => Dispatch::DynamicUniform {
                instances: item.batch_range().clone(),
                offset,
            },
            PhaseItemExtraIndex::IndirectParametersIndex {
                range,
                batch_set_index,
            } => Dispatch::Indirect {
                parameters: range,
                count: match batch_set_index {
                    None => CommandCount::Range,
                    Some(index) => CommandCount::BatchSet(u32::from(index)),
                },
            },
        };
        let receipt = Receipt {
            phase: P::PHASE,
            frame: frame.serial,
            camera: pose.camera,
            pipeline: item.cached_pipeline(),
            representative: item.main_entity().id(),
            material: material.asset_id,
            material_group: actual_material.binding.group.0,
            binding,
            dispatch,
        };
        if let Ok(mut commands) = frame.commands.lock() {
            if commands.len() >= MAX_COLOR_COMMANDS {
                pose.fail("color command fixture budget");
            } else {
                commands.push(receipt);
            }
        } else {
            pose.fail("color command lock");
        }
        RenderCommandResult::Success
    }
}

impl Dispatch {
    fn same_held_shape(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Indirect { parameters, count },
                Self::Indirect {
                    parameters: other_parameters,
                    count: other_count,
                },
            ) => {
                // Bevy rebuilds these addresses per frame. Their positive span
                // and count source remain held; exact addresses stay in receipts.
                parameters
                    .end
                    .checked_sub(parameters.start)
                    .is_some_and(|commands| {
                        commands > 0
                            && Some(commands)
                                == other_parameters.end.checked_sub(other_parameters.start)
                    })
                    && matches!(
                        (count, other_count),
                        (CommandCount::Range, CommandCount::Range)
                            | (CommandCount::BatchSet(_), CommandCount::BatchSet(_))
                    )
            }
            _ => self == other,
        }
    }
}

impl Receipt {
    /// Hold semantic draw identity, not frame-local indirect-buffer addresses.
    pub(super) fn same_batch(&self, other: &Self) -> bool {
        self.phase == other.phase
            && self.camera == other.camera
            && self.pipeline == other.pipeline
            && self.representative == other.representative
            && self.material == other.material
            && self.material_group == other.material_group
            && self.binding == other.binding
            && self.dispatch.same_held_shape(&other.dispatch)
    }

    pub(super) fn compatible(&self, member: &bindings::RenderMember) -> bool {
        self.pipeline == member.pipeline
            && bindings::material_type(self.material) == member.material_type()
            && self.material_group == member.material_group
            && self.binding.slabs == member.binding.slabs
            && self.binding.vertex_buffer == member.binding.vertex_buffer
            && self.binding.index_buffer == member.binding.index_buffer
            && self.binding.geometry.indexed() == member.binding.geometry.indexed()
    }

    pub(super) fn report(&self) -> serde_json::Value {
        let dispatch = match &self.dispatch {
            Dispatch::Direct { instances } => {
                serde_json::json!({"kind":"direct", "instances":instances})
            }
            Dispatch::DynamicUniform { instances, offset } => {
                serde_json::json!({"kind":"dynamic_uniform", "instances":instances, "offset":offset})
            }
            Dispatch::Indirect { parameters, count } => {
                serde_json::json!({"kind":"indirect", "parameters":parameters,
                "count":match count { CommandCount::Range => serde_json::json!({"kind":"range"}),
                    CommandCount::BatchSet(index) => serde_json::json!({"kind":"batch_set", "index":index}) }})
            }
        };
        serde_json::json!({"phase":self.phase, "frame":self.frame, "camera":format!("{:?}",self.camera),
            "pipeline":format!("{:?}",self.pipeline), "representative_bits":self.representative.to_bits(),
            "material":format!("{:?}",self.material), "material_group":self.material_group,
            "binding":self.binding.report(), "dispatch":dispatch})
    }
}

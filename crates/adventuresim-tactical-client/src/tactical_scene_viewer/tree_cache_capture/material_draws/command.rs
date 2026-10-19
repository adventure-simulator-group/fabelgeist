//! Successful singleton draws for the owned tree and actual visible terrain.
use super::*;

pub(super) struct Record;

impl<P: PhaseItem> RenderCommand<P> for Record {
    type Param = (
        SRes<Frame>,
        SRes<RenderMeshInstances>,
        SRes<RenderMaterialInstances>,
        SRes<ErasedRenderAssets<PreparedMaterial>>,
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
        let (frame, meshes, materials, prepared) = params;
        let Some(pose) = &frame.into_inner().0 else {
            return RenderCommandResult::Success;
        };
        if pose.camera != view.retained_view_entity {
            return RenderCommandResult::Success;
        }
        let entity = item.main_entity().id();
        let owned = pose.expected.iter().find(|member| member.entity == entity);
        let terrain = pose
            .background
            .iter()
            .find(|member| member.entity == entity);
        let expected_material = owned
            .map(|member| member.material)
            .or_else(|| terrain.map(|member| member.material));
        let Some(expected_mesh) = owned
            .map(|member| member.mesh)
            .or_else(|| terrain.map(|member| member.mesh))
        else {
            return RenderCommandResult::Success;
        };
        let mesh = meshes.into_inner().mesh_asset_id(item.main_entity());
        let material = materials
            .into_inner()
            .instances
            .get(&item.main_entity())
            .map(|instance| instance.asset_id);
        let Some(actual_material) = material else {
            pose.invalid.store(true, Ordering::Release);
            return RenderCommandResult::Success;
        };
        let Some(actual) = prepared.into_inner().get(actual_material) else {
            pose.invalid.store(true, Ordering::Release);
            return RenderCommandResult::Success;
        };
        if mesh != Some(expected_mesh)
            || material != expected_material
            || item.batch_range().len() != 1
        {
            pose.invalid.store(true, Ordering::Release);
            return RenderCommandResult::Success;
        }
        let path = match item.extra_index() {
            PhaseItemExtraIndex::None => SingleDrawPath::Direct,
            PhaseItemExtraIndex::DynamicOffset(offset) => {
                SingleDrawPath::DynamicUniformOffset { offset }
            }
            PhaseItemExtraIndex::IndirectParametersIndex {
                range,
                batch_set_index: None,
            } if range.len() == 1 => SingleDrawPath::Indirect {
                parameter: range.start,
            },
            _ => {
                pose.invalid.store(true, Ordering::Release);
                return RenderCommandResult::Success;
            }
        };
        let receipt = BindingReceipt {
            mesh: expected_mesh,
            material: actual_material,
            group: actual.binding.group.0,
            slot: actual.binding.slot.0,
            path,
        };
        if let Ok(bindings) = pose.bindings.lock() {
            if bindings.as_ref().is_some_and(|bindings| {
                !bindings
                    .get(&entity)
                    .is_some_and(|binding| binding.same_binding(&receipt))
            }) {
                pose.invalid.store(true, Ordering::Release);
            }
        } else {
            pose.invalid.store(true, Ordering::Release);
        }
        if let Ok(mut drawn) = pose.drawn.lock() {
            drawn.insert(entity, receipt);
        } else {
            pose.invalid.store(true, Ordering::Release);
        }
        RenderCommandResult::Success
    }
}

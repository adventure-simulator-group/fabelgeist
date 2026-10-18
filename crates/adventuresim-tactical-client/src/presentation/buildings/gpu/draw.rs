use super::*;
use bevy::{
    core_pipeline::{
        core_3d::{AlphaMask3d, Opaque3d},
        prepass::{AlphaMask3dPrepass, Opaque3dPrepass},
    },
    ecs::{
        query::QueryItem,
        system::{SystemParamItem, lifetimeless::SRes},
    },
    material::labels::DrawFunctionLabel,
    pbr::*,
    render::{
        Render, RenderApp, RenderSystems, erased_render_asset::ErasedRenderAssets, render_phase::*,
        view::ExtractedView,
    },
};

type CityDraw = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshViewBindingArrayBindGroup<1>,
    SetMaterialBindGroup<3>,
    DrawCity,
);
type CityShadowDraw = (
    SetItemPipeline,
    SetPrepassViewBindGroup<0>,
    SetPrepassViewEmptyBindGroup<1>,
    SetMaterialBindGroup<3>,
    DrawCity,
);

pub(super) fn install(app: &mut App) {
    app.sub_app_mut(RenderApp)
        .add_render_command::<Opaque3d, CityDraw>()
        .add_render_command::<AlphaMask3d, CityDraw>()
        .add_render_command::<Shadow, CityShadowDraw>()
        .add_render_command::<Opaque3dPrepass, CityShadowDraw>()
        .add_render_command::<AlphaMask3dPrepass, CityShadowDraw>()
        .add_systems(
            Render,
            replace_draws
                .after(RenderSystems::PrepareAssets)
                .before(RenderSystems::PrepareMeshes),
        );
}

fn replace_draws(
    mut materials: ResMut<ErasedRenderAssets<PreparedMaterial>>,
    opaque: Res<DrawFunctions<Opaque3d>>,
    alpha: Res<DrawFunctions<AlphaMask3d>>,
    shadow: Res<DrawFunctions<Shadow>>,
    prepass: Res<DrawFunctions<Opaque3dPrepass>>,
    mask_prepass: Res<DrawFunctions<AlphaMask3dPrepass>>,
) {
    for (id, material) in materials.iter_mut() {
        if id.type_id() != std::any::TypeId::of::<material::CityMaterial>() {
            continue;
        }
        let Some(properties) = Arc::get_mut(&mut material.properties) else {
            continue;
        };
        properties.draw_functions.clear();
        properties.add_draw_function(MainPassOpaqueDrawFunction, opaque.read().id::<CityDraw>());
        properties.add_draw_function(MainPassAlphaMaskDrawFunction, alpha.read().id::<CityDraw>());
        properties.add_draw_function(ShadowsDrawFunction, shadow.read().id::<CityShadowDraw>());
        properties.add_draw_function(
            ShadowsDepthOnlyDrawFunction,
            shadow.read().id::<CityShadowDraw>(),
        );
        properties.add_draw_function(
            PrepassOpaqueDrawFunction,
            prepass.read().id::<CityShadowDraw>(),
        );
        properties.add_draw_function(
            PrepassAlphaMaskDrawFunction,
            mask_prepass.read().id::<CityShadowDraw>(),
        );
    }
}

struct DrawCity;
impl<P: PhaseItem> RenderCommand<P> for DrawCity {
    type Param = (
        SRes<CityGpuScenes>,
        SRes<compute::CityViews>,
        SRes<scratch::Scratch>,
        SRes<RenderMaterialInstances>,
    );
    type ViewQuery = (Entity, &'static ExtractedView);
    type ItemQuery = ();

    fn render<'w>(
        item: &P,
        (entity, _): QueryItem<'w, '_, Self::ViewQuery>,
        _: Option<()>,
        (scenes, views, scratch, materials): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let (scenes, views, scratch, materials) = (
            scenes.into_inner(),
            views.into_inner(),
            scratch.into_inner(),
            materials.into_inner(),
        );
        let Some(instance) = materials.instances.get(&item.main_entity()) else {
            return RenderCommandResult::Skip;
        };
        let Some((owner, batch)) = PresentationOwner::ALL.into_iter().find_map(|owner| {
            scenes
                .owners
                .get(owner)
                .batches
                .iter()
                .position(|batch| batch.material.id().untyped() == instance.asset_id)
                .map(|batch| (owner, batch))
        }) else {
            return RenderCommandResult::Skip;
        };
        let views = views.owners.get(owner);
        let scratch = scratch.owners.get(owner);
        let Some(slot) = views.slots.get(&entity) else {
            return RenderCommandResult::Skip;
        };
        let Some(batch) = scratch.batches.get(batch) else {
            return RenderCommandResult::Skip;
        };
        pass.set_bind_group(2, &batch.geometry, &[]);
        pass.set_vertex_buffer(0, batch.indirect.slice(..));
        pass.draw_indirect(&batch.indirect, u64::from(*slot) * 16);
        RenderCommandResult::Success
    }
}

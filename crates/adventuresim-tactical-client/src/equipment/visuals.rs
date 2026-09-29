//! Gameplay equipment rendering shared by the live client and native captures.
use super::*;

pub(crate) struct EquipmentVisualPlugin;

impl Plugin for EquipmentVisualPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(OutlinePlugin::JUMP_FLOOD)
            .init_resource::<WeaponMeshCache>()
            .add_systems(
                Update,
                (
                    spawn_item_placeholders,
                    request_procedural_equipment_models,
                    resolve_procedural_equipment_models,
                    sync_procedural_equipment_skins,
                    morphs::sync_equipment_morphs,
                    render_binding::sync_render_bindings,
                    update_item_placeholders,
                )
                    .chain(),
            );
        // Runtime equipment is fitted on the armor device, which the web
        // build lacks.
        #[cfg(not(target_family = "wasm"))]
        app.init_resource::<RuntimeEquipmentBodyCache>()
            .add_systems(
                Update,
                generate_runtime_equipment_models
                    .after(spawn_item_placeholders)
                    .before(request_procedural_equipment_models),
            );
    }
}

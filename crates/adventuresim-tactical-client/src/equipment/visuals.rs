//! Gameplay equipment rendering shared by the live client and native captures.
use super::*;

pub(crate) struct EquipmentVisualPlugin;

impl Plugin for EquipmentVisualPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(OutlinePlugin::JUMP_FLOOD)
            .init_resource::<WeaponMeshCache>()
            .init_resource::<RuntimeEquipmentBodyCache>()
            .add_systems(
                Update,
                (
                    spawn_item_placeholders,
                    generate_runtime_equipment_models,
                    request_procedural_equipment_models,
                    resolve_procedural_equipment_models,
                    sync_procedural_equipment_skins,
                    morphs::sync_equipment_morphs,
                    render_binding::sync_render_bindings,
                    update_item_placeholders,
                )
                    .chain(),
            );
    }
}

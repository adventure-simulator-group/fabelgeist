//! Gameplay equipment rendering shared by the live client and native captures.
use super::*;

pub(crate) struct EquipmentVisualPlugin;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct EquipmentVisualSystems;

impl Plugin for EquipmentVisualPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(OutlinePlugin::JUMP_FLOOD)
            .init_resource::<WeaponMeshCache>()
            .add_systems(
                Update,
                (
                    remove_orphan_equipment,
                    spawn_item_placeholders,
                    request_procedural_equipment_models,
                    resolve_procedural_equipment_models,
                    sync_procedural_equipment_skins,
                    morphs::sync_equipment_morphs,
                    render_binding::sync_render_bindings,
                    update_item_placeholders,
                )
                    .chain()
                    .in_set(EquipmentVisualSystems),
            );
        // Runtime equipment is fitted on the armor device, which the web
        // build lacks.
        #[cfg(not(target_family = "wasm"))]
        app.init_resource::<RuntimeEquipmentBodyCache>()
            .add_systems(
                Update,
                (
                    runtime_equipment::prepare_runtime_equipment_body,
                    generate_runtime_equipment_models,
                )
                    .chain()
                    .in_set(EquipmentVisualSystems)
                    .after(spawn_item_placeholders)
                    .before(request_procedural_equipment_models),
            );
    }
}

fn remove_orphan_equipment(
    mut commands: Commands,
    roots: Query<(Entity, &ItemPlaceholder)>,
    items: Query<(), With<TacticalEquipmentPhysical>>,
) {
    for (entity, item) in &roots {
        if !items.contains(item.0) {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retired_items_remove_pending_geometry_before_generation() {
        let mut app = App::new();
        app.add_systems(Update, remove_orphan_equipment);
        let item = app
            .world_mut()
            .spawn(TacticalEquipmentPhysical {
                dimensions_m: Vec3::ONE,
                grip_to_tip_m: 0.0,
                striking_head_length_m: 0.0,
                anchor_offset_m: Vec3::ZERO,
            })
            .id();
        let root = app.world_mut().spawn(ItemPlaceholder(item)).id();
        app.update();
        assert!(app.world().get_entity(root).is_ok());
        app.world_mut().despawn(item);
        app.update();
        assert!(app.world().get_entity(root).is_err());
    }
}

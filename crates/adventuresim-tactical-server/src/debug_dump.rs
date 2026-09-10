//! Explicit transient world snapshot allowlist.
use super::*;

/// Serializes only the "core" reflectable character/level components (see
/// [`on_player_added`](player_projection::on_player_added) and
/// [`on_scene_terrain_added`] for the corresponding derive-the-rest hooks)
/// on every entity to a `.scn.ron` file under `world_dumps/`.
///
/// Deliberately an *allowlist*, not "every reflected component/resource on
/// every entity" - `reflect_auto_register` registers third-party types for
/// all sorts of unrelated reasons (BRP inspection, entity mapping, engine
/// bookkeeping), and any of them landing in the dump breaks the *entire*
/// dump if it lacks full reflection-based serialization support, not just
/// that one field. Both `Time<Real>` (a resource `.extract_resources()`
/// pulled in from `TimePlugin`) and `aeronet_io::Session` (a component on
/// every connected client's entity) hit exactly this - each contains a
/// `bevy_platform::time::Instant` with no `ReflectSerialize` registered.
/// Neither is reachable from a bare `App::new()` (what this file's own
/// tests use), which is why this took two rounds to actually surface.
pub(super) fn on_debug_dump_world_request(
    _request: On<FromClient<DebugDumpWorldRequest>>,
    world: &World,
) {
    let entities: Vec<Entity> = world
        .archetypes()
        .iter()
        .flat_map(|archetype| archetype.entities().iter().map(|entity| entity.id()))
        .collect();
    let registry = world.resource::<AppTypeRegistry>().read();
    // The filter must be set up *before* `extract_entities` - it's applied
    // immediately as entities are extracted, not lazily at `build()`.
    let scene = DynamicWorldBuilder::from_world(world, &registry)
        .deny_all_components()
        .allow_component::<Player>()
        .allow_component::<CharacterId>()
        .allow_component::<Skills>()
        .allow_component::<Limbs>()
        .allow_component::<TacticalAttributes>()
        .allow_component::<Stats>()
        .allow_component::<TacticalCombatState>()
        .allow_component::<TacticalCombatSide>()
        .allow_component::<Transform>()
        .allow_component::<SceneId>()
        .allow_component::<SceneTerrain>()
        .allow_component::<SceneBuilding>()
        .allow_component::<SceneFurniture>()
        .allow_component::<SceneFurnitureGroup>()
        .allow_component::<SceneVistaFurniture>()
        .allow_component::<crate::bot::MissionEnemy>()
        .allow_component::<crate::bot::OffensiveCombatAi>()
        .allow_component::<crate::bot::CombatantBehaviorPackages>()
        .allow_component::<crate::bot::ReactiveDefenseAi>()
        .allow_component::<crate::bot::DefenseChances>()
        .allow_component::<crate::bot::RaisedGuardAi>()
        .allow_component::<crate::bot::AimAtNearestOpponentAi>()
        .allow_component::<crate::bot::RecoverToUprightAi>()
        // Inventory items are separate entities (linked back to their
        // owning character via `ItemOf`), not components on the character
        // itself - without these, a dumped/loaded character's equipment is
        // silently empty. `InventoryItems` (the reverse side of the
        // `ItemOf` relationship) MUST be captured too: scene loading
        // applies components with `RelationshipHookMode::Skip`, so nothing
        // reconstructs the reverse side on load - a dump carries both sides
        // of the relationship verbatim, exactly like bevy's own
        // `ChildOf`/`Children` pair in dynamic scenes.
        .allow_component::<InventoryItems>()
        .allow_component::<ItemOf>()
        .allow_component::<TacticalItemQuantity>()
        .allow_component::<ItemProperties>()
        .allow_component::<WeaponItem>()
        .allow_component::<ShieldItem>()
        .allow_component::<ArmorItem>()
        .allow_component::<EquipmentTopology>()
        .allow_component::<EquipSlot>()
        .extract_entities(entities.into_iter())
        .build();
    let ron = match scene.serialize(&registry) {
        Ok(ron) => ron,
        Err(error) => {
            error!(?error, "Failed to serialize world dump");
            return;
        }
    };
    drop(registry);

    let dir = std::path::Path::new("world_dumps");
    if let Err(error) = std::fs::create_dir_all(dir) {
        error!(?error, "Failed to create world_dumps directory");
        return;
    }
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let path = dir.join(format!("world_dump_{timestamp}.scn.ron"));
    match std::fs::write(&path, ron) {
        Ok(()) => info!(path = %path.display(), "Dumped world state"),
        Err(error) => error!(?error, path = %path.display(), "Failed to write world dump"),
    }
}

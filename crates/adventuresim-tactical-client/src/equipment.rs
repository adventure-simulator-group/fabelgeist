//! Tactical grab input, QWERTY slot HUD, and placeholder item presentation.

use crate::{
    animation::{
        AuthoredBindTransform, BoneRole, HandSide, HeldWeaponConstraint, HumanoidRig, MhrBone,
        authored_bind_global,
    },
    player::ClientPlayer,
    presentation::GrabTargetOutline,
};
use adventuresim_core::equipment_assets::EquipmentAssets;
use adventuresim_tactical_core::prelude::*;
use adventuresim_tactical_netcode::{
    bevy_replicon::prelude::ClientTriggerExt,
    prelude::{EquipmentAction, EquipmentActionRequest, EquipmentHand},
};
use adventuresim_weapon_model::{
    ICON_RENDERER_VERSION, Material as WeaponMaterial, WeaponIconSpec, decode, generate,
    generate_holder_icon, generate_icon,
};
use bevy::{
    asset::RenderAssetUsages,
    camera::visibility::NoFrustumCulling,
    mesh::{
        Indices, PrimitiveTopology,
        skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
    },
    prelude::*,
};
use bevy_mod_outline::{OutlineMode, OutlinePlugin, OutlineVolume};
use grab_world::*;
use icons::*;
use placeholder_update::update_item_placeholders;
use preview::{
    HeldPreviewItem, eligible_slot_depth, hud_layers, ordered_preview_at_location,
    outermost_occupied_depth,
};
pub(crate) use render_binding::ProceduralEquipmentPart;
pub(crate) use runtime_equipment::RuntimeEquipmentBodyCache;
#[cfg(test)]
use runtime_equipment::RuntimeEquipmentPresentation;
use runtime_equipment::generate_runtime_equipment_models;
use skin::sync_procedural_equipment_skins;
#[cfg(target_family = "wasm")]
pub(crate) use slot_hud::SlotHudPress;
use std::{
    collections::{BTreeMap, HashMap},
    sync::LazyLock,
};
pub(crate) use visuals::EquipmentVisualPlugin;

mod grab_world;
mod icons;
mod placeholder_update;
mod placeholder_visual;
mod preview;
#[cfg(any(target_family = "wasm", test))]
pub(crate) mod readiness;
mod render_binding;
pub(crate) mod runtime_equipment;
pub(crate) mod settlement;
mod skin;
mod slot_hud;
mod slot_selection;
pub(crate) mod visuals;

const PICKUP_RANGE_M: f32 = 2.0;

const INVALID_FLASH_SECS: f32 = 0.18;

static PROCEDURAL_EQUIPMENT_ASSETS: LazyLock<EquipmentAssets> = LazyLock::new(|| {
    // This is a committed build input, not gameplay admission. Invalid embedded data is an invariant failure.
    match EquipmentAssets::from_json(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/equipment/procedural/manifest.json"
    ))) {
        Ok(assets) => assets,
        Err(error) => panic!("committed procedural equipment manifest must be valid: {error}"),
    }
});

pub(crate) struct TacticalEquipmentPlugin;

#[derive(Resource, Default)]
struct GrabSession {
    active: Option<EquipmentHand>,
    selection: Option<GrabSelection>,
    repeated_input: Option<(&'static str, usize)>,
    invalid_flash_remaining: f32,
    next_sequence: EquipmentSequence,
}

#[derive(Clone)]
struct CachedWeaponPart {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

#[derive(Clone)]
struct CachedWeapon {
    parts: Vec<CachedWeaponPart>,
    grip: Vec3,
}

#[derive(Resource, Default)]
struct WeaponMeshCache {
    weapons: HashMap<WeaponMeshCacheKey, CachedWeapon>,
    holders: HashMap<WeaponMeshCacheKey, CachedWeapon>,
    materials: HashMap<WeaponMaterial, Handle<StandardMaterial>>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct WeaponMeshCacheKey {
    generator_version: u16,
    design_hash: [u8; 32],
}

/// Equipment overlays, which the game holds back while a scene is loading.
#[derive(SystemSet, Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct EquipmentHudSet;

#[derive(Component)]
pub(crate) struct ItemPlaceholder(pub(crate) Entity);

#[derive(Component)]
pub(crate) struct ProceduralEquipmentResolved;

#[derive(Component)]
pub(crate) struct ProceduralEquipmentFailed;

#[derive(Component)]
struct ItemFallback;

#[derive(Component, Default)]
struct EquipmentAttachmentSockets(BTreeMap<String, Transform>);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GrabSelection {
    Slot {
        location: EquipmentLocation,
        depth: u16,
    },
    Hand(EquipmentHand),
    SceneItem(Entity),
    Door(Entity),
    Window(Entity),
}

impl Plugin for TacticalEquipmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EquipmentVisualPlugin)
            .init_resource::<GrabSession>()
            .init_resource::<WeaponIconCache>()
            .add_systems(
                PreUpdate,
                update_grab_input.after(bevy::input::InputSystems),
            )
            .add_systems(
                Update,
                (
                    update_pickup_outlines,
                    slot_hud::push_slot_hud.in_set(EquipmentHudSet),
                ),
            )
            .add_observer(slot_hud::apply_slot_hud_press);
    }
}

fn procedural_equipment_file(item_id: &str, placement_id: Option<&str>) -> Option<&'static str> {
    // ItemProperties and replicated placement text enter the canonical catalogue here.
    let item =
        adventuresim_core::equipment_assets::CatalogItem::find(item_catalog::catalog(), item_id)?;
    match placement_id {
        Some(id) => PROCEDURAL_EQUIPMENT_ASSETS.file_for(item.placement(id)?),
        None => PROCEDURAL_EQUIPMENT_ASSETS.default_file_for(item),
    }
}

fn key_code(input: &str) -> Option<KeyCode> {
    Some(match input {
        "q" => KeyCode::KeyQ,
        "e" => KeyCode::KeyE,
        "f" => KeyCode::KeyF,
        "x" => KeyCode::KeyX,
        "tab" => KeyCode::Tab,
        "r" => KeyCode::KeyR,
        "g" => KeyCode::KeyG,
        "y" => KeyCode::KeyY,
        "h" => KeyCode::KeyH,
        "1" => KeyCode::Digit1,
        "2" => KeyCode::Digit2,
        "3" => KeyCode::Digit3,
        "4" => KeyCode::Digit4,
        "5" => KeyCode::Digit5,
        "t" => KeyCode::KeyT,
        "`" => KeyCode::Backquote,
        "v" => KeyCode::KeyV,
        "b" => KeyCode::KeyB,
        "z" => KeyCode::KeyZ,
        "c" => KeyCode::KeyC,
        _ => return None,
    })
}

#[allow(clippy::too_many_arguments)]
fn update_grab_input(
    mut commands: Commands,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    player: Single<(Entity, &GlobalTransform), With<ClientPlayer>>,
    item_owners: Query<(Entity, &ItemOf, Option<&EquipSlot>)>,
    topologies: Query<(Entity, &ItemOf, &EquipmentTopology, &ItemProperties)>,
    properties: Query<&ItemProperties>,
    action_states: Query<&EquipmentActionState>,
    world_targets: WorldGrabTargets,
    mut session: ResMut<GrabSession>,
) {
    session.invalid_flash_remaining =
        (session.invalid_flash_remaining - time.delta_secs()).max(0.0);
    let (actor, actor_transform) = *player;
    if session.active.is_none() && !mouse.pressed(MouseButton::Right) {
        session.active = if mouse.just_pressed(MouseButton::Left) {
            Some(EquipmentHand::Right)
        } else if mouse.just_pressed(MouseButton::Middle) {
            Some(EquipmentHand::Left)
        } else {
            None
        };
    }
    let Some(hand) = session.active else { return };
    let held = held_item(actor, hand, &item_owners);
    session.selection = world_grab_selection(
        session.selection,
        held.is_some(),
        world_targets.pointed(actor_transform),
    );
    let held_preview = HeldPreviewItem::from(held.and_then(|entity| properties.get(entity).ok()));
    slot_selection::keyboard_slots(&mut session, &keys, held_preview, |location| {
        ordered_preview_at_location(actor, location, &topologies)
    });
    let released = match hand {
        EquipmentHand::Right => mouse.just_released(MouseButton::Left),
        EquipmentHand::Left => mouse.just_released(MouseButton::Middle),
    };
    if !released {
        return;
    }
    let Ok(action_state) = action_states.get(actor) else {
        warn!(
            ?actor,
            "Equipment action not sent: authoritative revision is unavailable"
        );
        (session.active, session.selection, session.repeated_input) = (None, None, None);
        return;
    };
    let action = match session.selection {
        Some(GrabSelection::Slot { location, depth }) => EquipmentAction::Slot {
            location,
            depth,
            expected_destination: ordered_preview_at_location(actor, location, &topologies)
                .get(depth as usize)
                .map(|target| target.addressed_entity()),
        },
        Some(GrabSelection::Hand(destination)) => EquipmentAction::Hand {
            destination,
            expected_destination: held_item(actor, destination, &item_owners),
        },
        Some(GrabSelection::SceneItem(item)) => EquipmentAction::Pickup { item },
        Some(GrabSelection::Door(door)) => EquipmentAction::OpenDoor { door },
        Some(GrabSelection::Window(window)) => EquipmentAction::ToggleWindow { window },
        None if held.is_some() => EquipmentAction::Drop,
        None => {
            session.active = None;
            session.repeated_input = None;
            return;
        }
    };
    let sequence = session.next_sequence;
    session.next_sequence = sequence.next();
    commands.client_trigger(EquipmentActionRequest {
        actor,
        sequence,
        expected_revision: action_state.revision,
        hand,
        expected_hand_item: held,
        action,
    });
    (session.active, session.selection, session.repeated_input) = (None, None, None);
}

fn weapon_material(
    class: WeaponMaterial,
    cache: &mut WeaponMeshCache,
    materials: &mut Assets<StandardMaterial>,
) -> Handle<StandardMaterial> {
    cache
        .materials
        .entry(class)
        .or_insert_with(|| {
            let base_color = match class {
                WeaponMaterial::Wood => Color::srgb(0.30, 0.18, 0.09),
                WeaponMaterial::Leather => Color::srgb(0.16, 0.09, 0.05),
                WeaponMaterial::DarkLeather => Color::srgb(0.055, 0.045, 0.038),
                WeaponMaterial::Brass => Color::srgb(0.58, 0.42, 0.13),
                WeaponMaterial::Steel => Color::srgb(0.55, 0.58, 0.60),
                WeaponMaterial::DarkSteel => Color::srgb(0.22, 0.24, 0.26),
                material => {
                    let [r, g, b] = material.color().map(|n| n as f32);
                    Color::srgb(r, g, b)
                }
            };
            materials.add(StandardMaterial {
                base_color,
                metallic: if class.is_metal() { 0.82 } else { 0.0 },
                perceptual_roughness: if class.is_metal() { 0.34 } else { 0.76 },
                ..default()
            })
        })
        .clone()
}

fn cached_weapon(
    appearance: &WeaponAppearance,
    cache: &mut WeaponMeshCache,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Option<CachedWeapon> {
    if appearance.recipe.len() > adventuresim_weapon_model::MAX_ENCODED_RECIPE_BYTES {
        return None;
    }
    if appearance.generator_version != adventuresim_weapon_model::GENERATOR_VERSION {
        return None;
    }
    let design = decode(&appearance.recipe).ok()?;
    if adventuresim_weapon_model::design_hash(&design).0 != appearance.design_hash {
        return None;
    }
    let key = WeaponMeshCacheKey {
        generator_version: appearance.generator_version,
        design_hash: appearance.design_hash,
    };
    if let Some(cached) = cache.weapons.get(&key) {
        return Some(cached.clone());
    }
    let generated = generate(&design).ok()?;
    let grip = Vec3::from_array(
        generated
            .anchors
            .iter()
            .find(|anchor| anchor.name == "weapon.grip")?
            .position,
    );
    let parts = generated
        .parts
        .into_iter()
        .map(|part| {
            let mut mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
            );
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, part.positions);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, part.normals);
            mesh.insert_indices(Indices::U32(part.indices));
            CachedWeaponPart {
                mesh: meshes.add(mesh),
                material: weapon_material(part.material, cache, materials),
            }
        })
        .collect();
    let cached = CachedWeapon { parts, grip };
    cache.weapons.insert(key, cached.clone());
    Some(cached)
}

fn cached_holder(
    appearance: &WeaponHolderAppearance,
    cache: &mut WeaponMeshCache,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Option<CachedWeapon> {
    if appearance.recipe.len() > adventuresim_weapon_model::MAX_ENCODED_RECIPE_BYTES
        || appearance.generator_version != adventuresim_weapon_model::HOLDER_GENERATOR_VERSION
    {
        return None;
    }
    let design = adventuresim_weapon_model::decode_holder(&appearance.recipe).ok()?;
    if adventuresim_weapon_model::holder_design_hash(&design).0 != appearance.design_hash {
        return None;
    }
    let key = WeaponMeshCacheKey {
        generator_version: appearance.generator_version,
        design_hash: appearance.design_hash,
    };
    if let Some(cached) = cache.holders.get(&key) {
        return Some(cached.clone());
    }
    let generated = adventuresim_weapon_model::generate_holder(&design).ok()?;
    let parts = generated
        .parts
        .into_iter()
        .map(|part| {
            let mut mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
            );
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, part.positions);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, part.normals);
            mesh.insert_indices(Indices::U32(part.indices));
            CachedWeaponPart {
                mesh: meshes.add(mesh),
                material: weapon_material(part.material, cache, materials),
            }
        })
        .collect();
    let cached = CachedWeapon {
        parts,
        grip: Vec3::from_array(generated.grip),
    };
    cache.holders.insert(key, cached.clone());
    Some(cached)
}

#[expect(
    clippy::type_complexity,
    reason = "the Bevy query selects every appearance change that can invalidate an equipment placeholder"
)]
fn spawn_item_placeholders(
    mut commands: Commands,
    added: Query<
        (
            Entity,
            &TacticalEquipmentPhysical,
            Option<&EquipmentTopology>,
            Option<&ItemProperties>,
            Option<&WeaponAppearance>,
            Option<&WeaponHolderAppearance>,
        ),
        Or<(
            Added<TacticalEquipmentPhysical>,
            Added<EquipmentTopology>,
            Changed<EquipmentTopology>,
            Added<ItemProperties>,
            Changed<ItemProperties>,
            Added<WeaponAppearance>,
            Changed<WeaponAppearance>,
            Added<WeaponHolderAppearance>,
            Changed<WeaponHolderAppearance>,
        )>,
    >,
    existing: Query<(Entity, &ItemPlaceholder)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cache: ResMut<WeaponMeshCache>,
) {
    for (item, physical, topology, properties, appearance, holder_appearance) in &added {
        if !physical.is_valid() {
            continue;
        }
        for (root, placeholder) in &existing {
            if placeholder.0 == item {
                commands.entity(root).despawn();
            }
        }
        commands.entity(item).remove::<EquipmentAttachmentSockets>();
        let mut root_commands = commands.spawn((
            Name::new("Tactical item placeholder"),
            ItemPlaceholder(item),
            Transform::default(),
            // Attachment is resolved on the following update. Keeping the
            // root hidden avoids a one-frame flash at the world origin.
            Visibility::Hidden,
        ));
        if let Some(presentation) = runtime_equipment::presentation(item, properties, topology) {
            root_commands.insert(presentation);
            continue;
        }
        let root = root_commands.id();
        let (generated, part_name) = if let Some(holder) =
            holder_appearance.and_then(|appearance| {
                cached_holder(appearance, &mut cache, &mut meshes, &mut materials)
            }) {
            (Some(holder), "Procedural weapon holder part")
        } else if properties.is_some_and(|properties| {
            matches!(
                properties.id.as_str(),
                "scabbard" | "sword_sheath" | "boot_sheath" | "forearm_holster" | "weapon_loop"
            )
        }) {
            // Holder catalog rows are semantic chassis, not renderable generic
            // boxes. An absent, corrupt, or unsupported holder instance stays
            // visually absent until a valid first-class recipe arrives.
            continue;
        } else {
            (
                appearance.and_then(|appearance| {
                    cached_weapon(appearance, &mut cache, &mut meshes, &mut materials)
                }),
                "Procedural weapon part",
            )
        };
        if let Some(generated) = generated {
            placeholder_visual::spawn_generated(&mut commands, root, item, generated, part_name);
        } else {
            placeholder_visual::spawn_fallback(
                &mut commands,
                root,
                item,
                physical,
                &mut meshes,
                &mut materials,
            );
        }
    }
}

fn semantic_attachment_axis(role: BoneRole) -> Option<(BoneRole, BoneRole)> {
    Some(match role {
        BoneRole::Pelvis => (BoneRole::Pelvis, BoneRole::StomachOne),
        BoneRole::StomachOne => (BoneRole::StomachOne, BoneRole::StomachTwo),
        BoneRole::StomachTwo => (BoneRole::StomachTwo, BoneRole::StomachThree),
        BoneRole::StomachThree => (BoneRole::StomachThree, BoneRole::Chest),
        BoneRole::Chest => (BoneRole::Chest, BoneRole::NeckOne),
        BoneRole::NeckOne => (BoneRole::NeckOne, BoneRole::Head),
        BoneRole::Head => (BoneRole::NeckOne, BoneRole::Head),
        BoneRole::ClavicleLeft => (BoneRole::ClavicleLeft, BoneRole::UpperArmLeft),
        BoneRole::ClavicleRight => (BoneRole::ClavicleRight, BoneRole::UpperArmRight),
        BoneRole::UpperArmLeft => (BoneRole::UpperArmLeft, BoneRole::ForearmLeft),
        BoneRole::UpperArmRight => (BoneRole::UpperArmRight, BoneRole::ForearmRight),
        BoneRole::ForearmLeft | BoneRole::HandLeft => (BoneRole::ForearmLeft, BoneRole::HandLeft),
        BoneRole::ForearmRight | BoneRole::HandRight => {
            (BoneRole::ForearmRight, BoneRole::HandRight)
        }
        BoneRole::ThighLeft => (BoneRole::ThighLeft, BoneRole::ShinLeft),
        BoneRole::ThighRight => (BoneRole::ThighRight, BoneRole::ShinRight),
        BoneRole::ShinLeft => (BoneRole::ShinLeft, BoneRole::FootLeft),
        BoneRole::ShinRight => (BoneRole::ShinRight, BoneRole::FootRight),
        BoneRole::FootLeft | BoneRole::ToeLeft => (BoneRole::FootLeft, BoneRole::ToeLeft),
        BoneRole::FootRight | BoneRole::ToeRight => (BoneRole::FootRight, BoneRole::ToeRight),
        BoneRole::Root | BoneRole::Camera | BoneRole::WeaponLeft | BoneRole::WeaponRight => {
            return None;
        }
    })
}

fn bind_space_attachment_correction(
    bind: GlobalTransform,
    desired_bind_rotation: Quat,
) -> Transform {
    GlobalTransform::from(Transform {
        translation: bind.translation(),
        rotation: desired_bind_rotation,
        scale: Vec3::ONE,
    })
    .reparented_to(&bind)
}

fn equipment_bind_correction(
    role: BoneRole,
    rig: &HumanoidRig,
    bind_nodes: &Query<(&AuthoredBindTransform, Option<&ChildOf>)>,
) -> Option<Transform> {
    let &bone = rig.get(&role)?;
    let bind = authored_bind_global(bone, bind_nodes.get(bone).ok()?.0.owner, bind_nodes)?;
    let desired_rotation = semantic_attachment_axis(role)
        .and_then(|(from_role, to_role)| {
            let &from = rig.get(&from_role)?;
            let &to = rig.get(&to_role)?;
            let owner = bind_nodes.get(from).ok()?.0.owner;
            let from = authored_bind_global(from, owner, bind_nodes)?.translation();
            let to = authored_bind_global(to, owner, bind_nodes)?.translation();
            Some(Quat::from_rotation_arc(
                Vec3::Y,
                (to - from).try_normalize()?,
            ))
        })
        .unwrap_or(Quat::IDENTITY);
    Some(bind_space_attachment_correction(bind, desired_rotation))
}

fn resolve_character_location(
    topology: &EquipmentTopology,
    topologies: &Query<
        (&EquipmentTopology, Option<&EquipmentAttachmentSockets>),
        Without<ItemPlaceholder>,
    >,
) -> Option<EquipmentLocation> {
    let mut topology = topology;
    let mut visited = std::collections::BTreeSet::new();
    for _ in 0..32 {
        if let Some(location) =
            topology
                .occupancies
                .iter()
                .find_map(|occupancy| match occupancy.anchor {
                    TacticalEquipmentAnchor::CharacterLocation(location) => Some(location),
                    TacticalEquipmentAnchor::ItemAttachment { .. } => None,
                })
        {
            return Some(location);
        }
        let parent = topology
            .occupancies
            .iter()
            .find_map(|occupancy| match occupancy.anchor {
                TacticalEquipmentAnchor::ItemAttachment { parent, .. } => Some(parent),
                TacticalEquipmentAnchor::CharacterLocation(_) => None,
            })?;
        if !visited.insert(parent) {
            return None;
        }
        topology = topologies.get(parent).ok()?.0;
    }
    None
}

fn resolve_equipment_attachment_socket(
    topology: &EquipmentTopology,
    topologies: &Query<
        (&EquipmentTopology, Option<&EquipmentAttachmentSockets>),
        Without<ItemPlaceholder>,
    >,
) -> Option<Transform> {
    let mut topology = topology;
    let mut visited = std::collections::BTreeSet::new();
    for _ in 0..32 {
        let (parent, attachment_point_id) =
            topology
                .occupancies
                .iter()
                .find_map(|occupancy| match &occupancy.anchor {
                    TacticalEquipmentAnchor::ItemAttachment {
                        parent,
                        attachment_point_id,
                    } => Some((*parent, attachment_point_id.as_str())),
                    TacticalEquipmentAnchor::CharacterLocation(_) => None,
                })?;
        if !visited.insert(parent) {
            return None;
        }
        let (parent_topology, sockets) = topologies.get(parent).ok()?;
        if let Some(socket) = sockets.and_then(|sockets| sockets.0.get(attachment_point_id)) {
            return Some(*socket);
        }
        topology = parent_topology;
    }
    None
}

fn equipment_location_bone(location: EquipmentLocation) -> BoneRole {
    match location {
        EquipmentLocation::Head | EquipmentLocation::Face => BoneRole::Head,
        EquipmentLocation::Neck => BoneRole::NeckOne,
        EquipmentLocation::Chest | EquipmentLocation::Back => BoneRole::Chest,
        EquipmentLocation::Stomach => BoneRole::StomachTwo,
        EquipmentLocation::LeftShoulder => BoneRole::ClavicleLeft,
        EquipmentLocation::RightShoulder => BoneRole::ClavicleRight,
        EquipmentLocation::LeftArm => BoneRole::UpperArmLeft,
        EquipmentLocation::RightArm => BoneRole::UpperArmRight,
        EquipmentLocation::LeftHand => BoneRole::HandLeft,
        EquipmentLocation::RightHand => BoneRole::HandRight,
        EquipmentLocation::LeftLeg => BoneRole::ThighLeft,
        EquipmentLocation::RightLeg => BoneRole::ThighRight,
        EquipmentLocation::LeftFoot => BoneRole::FootLeft,
        EquipmentLocation::RightFoot => BoneRole::FootRight,
        EquipmentLocation::LeftBelt
        | EquipmentLocation::RightBelt
        | EquipmentLocation::FrontBelt
        | EquipmentLocation::BackBelt
        | EquipmentLocation::BackLeftPocket
        | EquipmentLocation::BackRightPocket => BoneRole::Pelvis,
        EquipmentLocation::LeftPocket => BoneRole::ThighLeft,
        EquipmentLocation::RightPocket => BoneRole::ThighRight,
    }
}

fn holding_side(slot: Option<&EquipSlot>) -> Option<HandSide> {
    match slot {
        Some(EquipSlot::HoldingLeft) => Some(HandSide::Left),
        Some(EquipSlot::HoldingRight) => Some(HandSide::Right),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn procedural_equipment_manifest_selects_exact_and_dropped_variants() {
        assert_eq!(
            procedural_equipment_file("mail_sleeve", Some("right")),
            Some("mail_sleeve--right.glb")
        );
        assert_eq!(
            procedural_equipment_file("mail_sleeve", None),
            Some("mail_sleeve--left.glb")
        );
        assert_eq!(
            procedural_equipment_file("mail_sleeve", Some("left_hand")),
            None
        );
        assert_eq!(
            procedural_equipment_file("belt", Some("worn")),
            Some("belt--worn.glb")
        );
        assert_eq!(
            procedural_equipment_file("belt", None),
            Some("belt--worn.glb")
        );
        assert_eq!(
            procedural_equipment_file("linen_breeches", Some("worn")),
            Some("linen_breeches--worn.glb")
        );
        assert_eq!(
            procedural_equipment_file("leather_boot", Some("right")),
            Some("leather_boot--right.glb")
        );
        assert_eq!(
            procedural_equipment_file("leather_boot", None),
            Some("leather_boot--left.glb")
        );
        assert_eq!(procedural_equipment_file("arming_sword", None), None);

        let asset_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        for entry in PROCEDURAL_EQUIPMENT_ASSETS.entries() {
            let file = &entry.file;
            let portrait = format!("icons/{}.png", file.strip_suffix(".glb").unwrap());
            assert!(
                asset_root
                    .join("equipment/procedural")
                    .join(portrait)
                    .is_file(),
                "manifest asset {file} must ship its portrait"
            );
        }
    }

    #[test]
    fn procedural_equipment_uses_live_rig_while_worn_and_rest_pose_when_dropped() {
        let mut world = World::new();
        let owner = world.spawn_empty().id();
        let root = world
            .spawn((
                MhrBone { owner },
                Name::new("root"),
                crate::animation::AuthoredRigJoint::new("root".into()),
            ))
            .id();
        let spine = world
            .spawn((
                MhrBone { owner },
                Name::new("c_spine0"),
                crate::animation::AuthoredRigJoint::new("c_spine0".into()),
            ))
            .id();
        let item = world.spawn(ItemOf(owner)).id();
        let part = world
            .spawn(ProceduralEquipmentPart {
                item,
                inverse_bindposes: Handle::default(),
                joint_names: vec!["root".into(), "c_spine0".into()].into(),
                rigid_center: Vec3::ZERO,
            })
            .id();

        world
            .run_system_once(sync_procedural_equipment_skins)
            .unwrap();
        assert_eq!(
            world.get::<SkinnedMesh>(part).unwrap().joints,
            [root, spine]
        );

        world.entity_mut(item).insert(TacticalSceneItem);
        world
            .run_system_once(sync_procedural_equipment_skins)
            .unwrap();
        assert!(world.get::<SkinnedMesh>(part).is_none());
    }

    #[test]
    fn generated_armor_starts_with_runtime_presentation() {
        let mut world = World::new();
        world.insert_resource(Assets::<Mesh>::default());
        world.insert_resource(Assets::<StandardMaterial>::default());
        world.init_resource::<WeaponMeshCache>();
        let item = world
            .spawn((
                valid_physical(),
                EquipmentTopology {
                    placement_id: Some("worn".into()),
                    ..default()
                },
                ItemProperties {
                    weight: 2.5,
                    id: "arming_doublet".into(),
                },
            ))
            .id();

        world.run_system_once(spawn_item_placeholders).unwrap();

        let presentation = world
            .query::<(&ItemPlaceholder, &RuntimeEquipmentPresentation)>()
            .iter(&world)
            .find(|(placeholder, _)| placeholder.0 == item)
            .map(|(_, presentation)| presentation)
            .unwrap();
        assert_eq!(presentation.item, item);
        assert_eq!(presentation.item_id, "arming_doublet");
        assert_eq!(presentation.placement_id, "worn");
        assert!(world.query::<&ItemFallback>().iter(&world).next().is_none());
    }

    #[test]
    fn generated_clothing_starts_with_runtime_presentation() {
        let mut world = World::new();
        world.insert_resource(Assets::<Mesh>::default());
        world.insert_resource(Assets::<StandardMaterial>::default());
        world.init_resource::<WeaponMeshCache>();
        let item = world
            .spawn((
                valid_physical(),
                EquipmentTopology {
                    placement_id: Some("worn".into()),
                    ..default()
                },
                ItemProperties {
                    weight: 0.5,
                    id: "linen_tunic".into(),
                },
            ))
            .id();

        world.run_system_once(spawn_item_placeholders).unwrap();

        let presentation = world
            .query::<(&ItemPlaceholder, &RuntimeEquipmentPresentation)>()
            .iter(&world)
            .find(|(placeholder, _)| placeholder.0 == item)
            .map(|(_, presentation)| presentation)
            .unwrap();
        assert_eq!(presentation.item, item);
        assert_eq!(presentation.item_id, "linen_tunic");
        assert_eq!(presentation.placement_id, "worn");
    }

    #[test]
    fn identical_weapon_recipes_share_cached_mesh_handles() {
        let design = adventuresim_weapon_model::default_design("longsword").unwrap();
        let appearance = WeaponAppearance {
            generator_version: adventuresim_weapon_model::GENERATOR_VERSION,
            design_hash: adventuresim_weapon_model::design_hash(&design).0,
            recipe: adventuresim_weapon_model::encode(&design).unwrap(),
        };
        let mut cache = WeaponMeshCache::default();
        let mut meshes = Assets::<Mesh>::default();
        let mut materials = Assets::<StandardMaterial>::default();
        let first = cached_weapon(&appearance, &mut cache, &mut meshes, &mut materials).unwrap();
        let second = cached_weapon(&appearance, &mut cache, &mut meshes, &mut materials).unwrap();
        assert_eq!(first.parts.len(), second.parts.len());
        assert!(
            first
                .parts
                .iter()
                .zip(&second.parts)
                .all(|(left, right)| left.mesh == right.mesh)
        );
        assert_eq!(cache.weapons.len(), 1);
    }

    #[test]
    fn corrupt_weapon_recipe_falls_back_instead_of_panicking() {
        let appearance = WeaponAppearance {
            generator_version: adventuresim_weapon_model::GENERATOR_VERSION,
            design_hash: [7; 32],
            recipe: vec![0xff, 0x00],
        };
        assert!(
            cached_weapon(
                &appearance,
                &mut WeaponMeshCache::default(),
                &mut Assets::<Mesh>::default(),
                &mut Assets::<StandardMaterial>::default(),
            )
            .is_none()
        );
    }

    fn longsword_appearance() -> WeaponAppearance {
        let design = adventuresim_weapon_model::default_design("longsword").unwrap();
        WeaponAppearance {
            generator_version: adventuresim_weapon_model::GENERATOR_VERSION,
            design_hash: adventuresim_weapon_model::design_hash(&design).0,
            recipe: adventuresim_weapon_model::encode(&design).unwrap(),
        }
    }

    #[test]
    fn identical_weapon_recipes_share_one_tactical_icon() {
        let appearance = longsword_appearance();
        let mut cache = WeaponIconCache::default();
        let first = cached_weapon_icon(&appearance, &mut cache).unwrap();
        let second = cached_weapon_icon(&appearance, &mut cache).unwrap();
        assert_eq!(first, second);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn tactical_icon_cache_reauthenticates_recipe_before_a_warm_hit() {
        let valid = longsword_appearance();
        let mut cache = WeaponIconCache::default();
        assert!(cached_weapon_icon(&valid, &mut cache).is_some());

        let mut borrowed_hash = valid;
        let other = adventuresim_weapon_model::default_design("rondel_dagger").unwrap();
        borrowed_hash.recipe = adventuresim_weapon_model::encode(&other).unwrap();
        assert!(cached_weapon_icon(&borrowed_hash, &mut cache).is_none());
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn distinct_weapon_designs_get_distinct_tactical_icons() {
        let longsword = longsword_appearance();
        let rondel = adventuresim_weapon_model::default_design("rondel_dagger").unwrap();
        let rondel = WeaponAppearance {
            generator_version: adventuresim_weapon_model::GENERATOR_VERSION,
            design_hash: adventuresim_weapon_model::design_hash(&rondel).0,
            recipe: adventuresim_weapon_model::encode(&rondel).unwrap(),
        };
        let mut cache = WeaponIconCache::default();
        let longsword = cached_weapon_icon(&longsword, &mut cache).unwrap();
        let rondel = cached_weapon_icon(&rondel, &mut cache).unwrap();
        assert_ne!(longsword, rondel);
        assert_eq!(cache.len(), 2);
    }

    fn longsword_holder_appearance() -> WeaponHolderAppearance {
        let design = adventuresim_weapon_model::default_design("longsword").unwrap();
        let holder = adventuresim_weapon_model::default_holder_design(&design).unwrap();
        WeaponHolderAppearance {
            generator_version: adventuresim_weapon_model::HOLDER_GENERATOR_VERSION,
            design_hash: adventuresim_weapon_model::holder_design_hash(&holder).0,
            recipe: adventuresim_weapon_model::encode_holder(&holder).unwrap(),
        }
    }

    #[test]
    fn identical_holder_recipes_share_one_tactical_icon() {
        let appearance = longsword_holder_appearance();
        let mut cache = WeaponIconCache::default();
        let first = cached_holder_icon(&appearance, &mut cache).unwrap();
        let second = cached_holder_icon(&appearance, &mut cache).unwrap();
        assert_eq!(first, second);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn tactical_holder_icon_cache_reauthenticates_before_a_warm_hit() {
        let valid = longsword_holder_appearance();
        let mut cache = WeaponIconCache::default();
        assert!(cached_holder_icon(&valid, &mut cache).is_some());
        let mut tampered = valid;
        tampered.recipe[5] ^= 0x3c;
        assert!(cached_holder_icon(&tampered, &mut cache).is_none());
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn identical_holder_recipes_share_a_separate_cache() {
        let appearance = longsword_holder_appearance();
        let mut cache = WeaponMeshCache::default();
        let mut meshes = Assets::<Mesh>::default();
        let mut materials = Assets::<StandardMaterial>::default();
        let first = cached_holder(&appearance, &mut cache, &mut meshes, &mut materials).unwrap();
        let second = cached_holder(&appearance, &mut cache, &mut meshes, &mut materials).unwrap();
        assert!(
            first
                .parts
                .iter()
                .zip(&second.parts)
                .all(|(left, right)| left.mesh == right.mesh)
        );
        assert_eq!(cache.holders.len(), 1);
        assert!(cache.weapons.is_empty());
    }

    #[test]
    fn warm_holder_cache_reauthenticates_recipe_and_hash() {
        let valid = longsword_holder_appearance();
        let mut cache = WeaponMeshCache::default();
        let mut meshes = Assets::<Mesh>::default();
        let mut materials = Assets::<StandardMaterial>::default();
        assert!(cached_holder(&valid, &mut cache, &mut meshes, &mut materials).is_some());
        let mut tampered = valid;
        tampered.recipe[5] ^= 0x3c;
        assert!(cached_holder(&tampered, &mut cache, &mut meshes, &mut materials).is_none());
    }

    #[test]
    fn holder_chassis_without_an_instance_never_renders_a_generic_box() {
        let mut world = World::new();
        world.insert_resource(Assets::<Mesh>::default());
        world.insert_resource(Assets::<StandardMaterial>::default());
        world.init_resource::<WeaponMeshCache>();
        world.spawn((
            valid_physical(),
            ItemProperties {
                weight: 0.3,
                id: "scabbard".into(),
            },
        ));
        world.run_system_once(spawn_item_placeholders).unwrap();
        assert_eq!(world.query::<&Mesh3d>().iter(&world).count(), 0);
    }

    #[test]
    fn warm_cache_does_not_accept_a_recipe_with_a_borrowed_hash() {
        let valid = longsword_appearance();
        let mut cache = WeaponMeshCache::default();
        let mut meshes = Assets::<Mesh>::default();
        let mut materials = Assets::<StandardMaterial>::default();
        assert!(cached_weapon(&valid, &mut cache, &mut meshes, &mut materials).is_some());
        let mut tampered = valid;
        tampered.recipe[4] ^= 0x5a;
        assert!(cached_weapon(&tampered, &mut cache, &mut meshes, &mut materials).is_none());
    }

    fn valid_physical() -> TacticalEquipmentPhysical {
        TacticalEquipmentPhysical {
            dimensions_m: Vec3::new(0.25, 1.4, 0.08),
            grip_to_tip_m: 1.15,
            striking_head_length_m: 0.25,
            anchor_offset_m: Vec3::new(0.0, -0.45, 0.0),
        }
    }

    #[test]
    fn either_replication_arrival_order_builds_one_weapon_presentation() {
        for appearance_first in [false, true] {
            let mut world = World::new();
            world.insert_resource(Assets::<Mesh>::default());
            world.insert_resource(Assets::<StandardMaterial>::default());
            world.init_resource::<WeaponMeshCache>();
            let item = if appearance_first {
                world.spawn(longsword_appearance()).id()
            } else {
                world.spawn(valid_physical()).id()
            };
            world.run_system_once(spawn_item_placeholders).unwrap();
            if appearance_first {
                world.entity_mut(item).insert(valid_physical());
            } else {
                world.entity_mut(item).insert(longsword_appearance());
            }
            world.run_system_once(spawn_item_placeholders).unwrap();
            let roots = world
                .query::<&ItemPlaceholder>()
                .iter(&world)
                .filter(|placeholder| placeholder.0 == item)
                .count();
            assert_eq!(roots, 1, "appearance_first={appearance_first}");
        }
    }

    #[test]
    fn either_replication_arrival_order_builds_one_holder_presentation() {
        for appearance_first in [false, true] {
            let mut world = World::new();
            world.insert_resource(Assets::<Mesh>::default());
            world.insert_resource(Assets::<StandardMaterial>::default());
            world.init_resource::<WeaponMeshCache>();
            let item = if appearance_first {
                world.spawn(longsword_holder_appearance()).id()
            } else {
                world.spawn(valid_physical()).id()
            };
            world.run_system_once(spawn_item_placeholders).unwrap();
            if appearance_first {
                world.entity_mut(item).insert(valid_physical());
            } else {
                world.entity_mut(item).insert(longsword_holder_appearance());
            }
            world.run_system_once(spawn_item_placeholders).unwrap();
            let roots = world
                .query::<&ItemPlaceholder>()
                .iter(&world)
                .filter(|placeholder| placeholder.0 == item)
                .count();
            assert_eq!(roots, 1, "appearance_first={appearance_first}");
            assert_eq!(world.resource::<WeaponMeshCache>().holders.len(), 1);
        }
    }

    #[test]
    fn grab_outline_is_visible_only_for_the_selected_world_target() {
        let selected = Entity::from_bits(1);
        let other = Entity::from_bits(2);

        assert!(grab_target_outline_selected(
            selected,
            Some(GrabSelection::SceneItem(selected))
        ));
        assert!(!grab_target_outline_selected(
            other,
            Some(GrabSelection::SceneItem(selected))
        ));
        assert!(grab_target_outline_selected(
            selected,
            Some(GrabSelection::Door(selected))
        ));
        assert!(grab_target_outline_selected(
            selected,
            Some(GrabSelection::Window(selected))
        ));
        assert!(!grab_target_outline_selected(selected, None));
    }

    #[test]
    fn movement_keys_are_not_slot_addresses() {
        for key in ["w", "a", "s", "d"] {
            assert!(
                INPUT_ADDRESS_MAPPINGS
                    .iter()
                    .all(|mapping| mapping.input != key)
            );
        }
    }

    #[test]
    fn empty_hand_tracks_the_pointed_scene_item_for_release() {
        let first = Entity::from_bits(21);
        let second = Entity::from_bits(22);
        let selection = world_grab_selection(None, false, Some(GrabSelection::SceneItem(first)));
        assert_eq!(selection, Some(GrabSelection::SceneItem(first)));
        assert_eq!(
            world_grab_selection(selection, false, Some(GrabSelection::SceneItem(second)),),
            Some(GrabSelection::SceneItem(second))
        );
        assert_eq!(world_grab_selection(selection, false, None), None);
    }

    #[test]
    fn empty_hand_tracks_a_pointed_door_but_an_occupied_hand_does_not() {
        let door = Entity::from_bits(24);
        assert_eq!(
            world_grab_selection(None, false, Some(GrabSelection::Door(door))),
            Some(GrabSelection::Door(door))
        );
        assert_eq!(
            world_grab_selection(None, true, Some(GrabSelection::Door(door))),
            None
        );
    }

    #[test]
    fn empty_hand_tracks_a_pointed_window_but_an_occupied_hand_does_not() {
        let window = Entity::from_bits(25);
        assert_eq!(
            world_grab_selection(None, false, Some(GrabSelection::Window(window))),
            Some(GrabSelection::Window(window))
        );
        assert_eq!(
            world_grab_selection(None, true, Some(GrabSelection::Window(window))),
            None
        );
    }

    #[test]
    fn pointed_scene_item_does_not_override_an_explicit_slot_selection() {
        let selection = Some(GrabSelection::Slot {
            location: EquipmentLocation::LeftBelt,
            depth: 0,
        });
        assert_eq!(
            world_grab_selection(
                selection,
                false,
                Some(GrabSelection::SceneItem(Entity::from_bits(23))),
            ),
            selection
        );
    }

    #[test]
    fn armor_slots_use_body_anchor_instead_of_stale_hand_constraint() {
        assert_eq!(holding_side(Some(&EquipSlot::ArmorChest)), None);
        assert_eq!(
            holding_side(Some(&EquipSlot::HoldingLeft)),
            Some(HandSide::Left)
        );
    }

    #[test]
    fn every_equipment_location_has_a_semantic_bone() {
        for (location, expected) in [
            (EquipmentLocation::Head, BoneRole::Head),
            (EquipmentLocation::Face, BoneRole::Head),
            (EquipmentLocation::Neck, BoneRole::NeckOne),
            (EquipmentLocation::Chest, BoneRole::Chest),
            (EquipmentLocation::Stomach, BoneRole::StomachTwo),
            (EquipmentLocation::Back, BoneRole::Chest),
            (EquipmentLocation::LeftShoulder, BoneRole::ClavicleLeft),
            (EquipmentLocation::RightShoulder, BoneRole::ClavicleRight),
            (EquipmentLocation::LeftArm, BoneRole::UpperArmLeft),
            (EquipmentLocation::RightArm, BoneRole::UpperArmRight),
            (EquipmentLocation::LeftHand, BoneRole::HandLeft),
            (EquipmentLocation::RightHand, BoneRole::HandRight),
            (EquipmentLocation::LeftLeg, BoneRole::ThighLeft),
            (EquipmentLocation::RightLeg, BoneRole::ThighRight),
            (EquipmentLocation::LeftFoot, BoneRole::FootLeft),
            (EquipmentLocation::RightFoot, BoneRole::FootRight),
            (EquipmentLocation::LeftBelt, BoneRole::Pelvis),
            (EquipmentLocation::RightBelt, BoneRole::Pelvis),
            (EquipmentLocation::FrontBelt, BoneRole::Pelvis),
            (EquipmentLocation::BackBelt, BoneRole::Pelvis),
            (EquipmentLocation::LeftPocket, BoneRole::ThighLeft),
            (EquipmentLocation::RightPocket, BoneRole::ThighRight),
            (EquipmentLocation::BackLeftPocket, BoneRole::Pelvis),
            (EquipmentLocation::BackRightPocket, BoneRole::Pelvis),
        ] {
            assert_eq!(equipment_location_bone(location), expected, "{location:?}");
        }
    }

    #[test]
    fn attachment_correction_cancels_mhr_bind_roll_but_keeps_position() {
        let bind = GlobalTransform::from(Transform {
            translation: Vec3::new(0.4, 1.2, -0.3),
            rotation: Quat::from_euler(EulerRot::XYZ, 0.7, -0.4, 1.1),
            scale: Vec3::splat(1.25),
        });
        let desired = Quat::from_rotation_arc(Vec3::Y, Vec3::X);
        let correction = bind_space_attachment_correction(bind, desired);
        let resolved = bind.mul_transform(correction);

        assert!(resolved.translation().abs_diff_eq(bind.translation(), 1e-5));
        assert!(resolved.rotation().dot(desired).abs() > 1.0 - 1e-5);
        assert!(
            resolved
                .to_scale_rotation_translation()
                .0
                .abs_diff_eq(Vec3::ONE, 1e-5)
        );
    }

    #[test]
    fn limb_placeholders_use_semantic_joint_axes_not_local_mhr_y() {
        assert_eq!(
            semantic_attachment_axis(BoneRole::UpperArmLeft),
            Some((BoneRole::UpperArmLeft, BoneRole::ForearmLeft))
        );
        assert_eq!(
            semantic_attachment_axis(BoneRole::ThighRight),
            Some((BoneRole::ThighRight, BoneRole::ShinRight))
        );
        assert_eq!(
            semantic_attachment_axis(BoneRole::FootLeft),
            Some((BoneRole::FootLeft, BoneRole::ToeLeft))
        );
    }

    #[test]
    fn attached_items_resolve_the_body_bone_through_their_parent_chain() {
        fn occupancy(anchor: TacticalEquipmentAnchor) -> EquipmentTopologyOccupancy {
            EquipmentTopologyOccupancy {
                occupancy_id: String::new(),
                anchor,
                channel: EquipmentChannel::Containment,
                order: 0,
                requirement_index: 0,
                capacity_index: 0,
            }
        }

        let mut world = World::new();
        let socket = Transform::from_translation(Vec3::new(-0.31, 1.02, -0.04));
        let belt = world
            .spawn((
                EquipmentTopology {
                    placement_id: Some("worn".into()),
                    occupancies: vec![occupancy(TacticalEquipmentAnchor::CharacterLocation(
                        EquipmentLocation::LeftBelt,
                    ))],
                },
                EquipmentAttachmentSockets(BTreeMap::from([("mount".into(), socket)])),
            ))
            .id();
        let sheath = world
            .spawn(EquipmentTopology {
                placement_id: Some("attached".into()),
                occupancies: vec![occupancy(TacticalEquipmentAnchor::ItemAttachment {
                    parent: belt,
                    attachment_point_id: "mount".into(),
                })],
            })
            .id();
        let weapon = world
            .spawn(EquipmentTopology {
                placement_id: Some("contained".into()),
                occupancies: vec![occupancy(TacticalEquipmentAnchor::ItemAttachment {
                    parent: sheath,
                    attachment_point_id: "blade".into(),
                })],
            })
            .id();

        let location = world
            .run_system_once(
                move |topologies: Query<
                    (&EquipmentTopology, Option<&EquipmentAttachmentSockets>),
                    Without<ItemPlaceholder>,
                >| {
                    let topology = topologies.get(weapon).unwrap().0;
                    (
                        resolve_character_location(topology, &topologies),
                        resolve_equipment_attachment_socket(topology, &topologies),
                    )
                },
            )
            .unwrap();
        assert_eq!(location.0, Some(EquipmentLocation::LeftBelt));
        assert_eq!(location.1, Some(socket));
    }

    #[test]
    fn every_equipment_catalog_icon_is_present_in_tactical_atlas() {
        for definition in item_catalog::catalog()
            .iter()
            .filter(|definition| definition.equipment.is_some())
        {
            assert!(
                EQUIPMENT_ICON_SLUGS.contains(&definition.presentation.icon.as_str()),
                "missing tactical atlas icon {}",
                definition.presentation.icon
            );
        }
    }
}

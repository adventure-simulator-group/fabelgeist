//! Spawn the temporary render roots for replicated tactical equipment.

use super::*;

struct PlaceholderAssets<'a> {
    meshes: &'a mut Assets<Mesh>,
    materials: &'a mut Assets<StandardMaterial>,
}

struct PlaceholderRoot {
    entity: Entity,
    runtime_equipment: bool,
}

enum PlaceholderVisual {
    Generated(CachedWeapon, &'static str),
    Fallback,
}

#[expect(
    clippy::type_complexity,
    reason = "the Bevy query selects every appearance change that can invalidate an equipment placeholder"
)]
pub(super) fn spawn_item_placeholders(
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
    let mut assets = PlaceholderAssets {
        meshes: &mut meshes,
        materials: &mut materials,
    };
    for (item, physical, topology, properties, appearance, holder_appearance) in &added {
        if !physical.is_valid() {
            continue;
        }
        despawn_existing_placeholders(&mut commands, item, &existing);
        commands.entity(item).remove::<EquipmentAttachmentSockets>();
        let root = spawn_placeholder_root(&mut commands, item, topology, properties);
        if root.runtime_equipment {
            continue;
        }
        let Some(visual) = placeholder_visual(
            holder_appearance,
            properties,
            appearance,
            &mut cache,
            &mut assets,
        ) else {
            continue;
        };
        spawn_placeholder_geometry(
            &mut commands,
            root.entity,
            item,
            physical,
            visual,
            &mut assets,
        );
    }
}

fn despawn_existing_placeholders(
    commands: &mut Commands,
    item: Entity,
    existing: &Query<(Entity, &ItemPlaceholder)>,
) {
    for (root, placeholder) in existing {
        if placeholder.0 == item {
            commands.entity(root).despawn();
        }
    }
}

fn spawn_placeholder_root(
    commands: &mut Commands,
    item: Entity,
    topology: Option<&EquipmentTopology>,
    properties: Option<&ItemProperties>,
) -> PlaceholderRoot {
    let runtime_equipment = properties.is_some_and(|properties| {
        adventuresim_character_creator::runtime_equipment::is_runtime_equipment(&properties.id)
    });
    let mut root_commands = commands.spawn((
        Name::new("Tactical item placeholder"),
        ItemPlaceholder(item),
        Transform::default(),
        Visibility::Hidden,
    ));
    if runtime_equipment {
        let properties = properties.expect("runtime equipment requires item properties");
        root_commands.insert(RuntimeEquipmentPresentation {
            item,
            item_id: properties.id.clone(),
            placement_id: topology
                .and_then(|topology| topology.placement_id.clone())
                .unwrap_or_else(|| "worn".into()),
        });
    } else if let Some(file) = properties.and_then(|properties| {
        procedural_equipment_file(
            &properties.id,
            topology.and_then(|topology| topology.placement_id.as_deref()),
        )
    }) {
        root_commands.insert(ProceduralEquipmentPresentation {
            asset_path: procedural_equipment_asset_path(file),
        });
    }
    PlaceholderRoot {
        entity: root_commands.id(),
        runtime_equipment,
    }
}

fn placeholder_visual(
    holder_appearance: Option<&WeaponHolderAppearance>,
    properties: Option<&ItemProperties>,
    appearance: Option<&WeaponAppearance>,
    cache: &mut WeaponMeshCache,
    assets: &mut PlaceholderAssets,
) -> Option<PlaceholderVisual> {
    if let Some(holder) = holder_appearance
        .and_then(|appearance| cached_holder(appearance, cache, assets.meshes, assets.materials))
    {
        return Some(PlaceholderVisual::Generated(
            holder,
            "Procedural weapon holder part",
        ));
    }
    if properties.is_some_and(|properties| {
        matches!(
            properties.id.as_str(),
            "scabbard" | "sword_sheath" | "boot_sheath" | "forearm_holster" | "weapon_loop"
        )
    }) {
        return None;
    }
    Some(
        appearance
            .and_then(|appearance| {
                cached_weapon(appearance, cache, assets.meshes, assets.materials)
            })
            .map_or(PlaceholderVisual::Fallback, |weapon| {
                PlaceholderVisual::Generated(weapon, "Procedural weapon part")
            }),
    )
}

fn spawn_placeholder_geometry(
    commands: &mut Commands,
    root: Entity,
    item: Entity,
    physical: &TacticalEquipmentPhysical,
    visual: PlaceholderVisual,
    assets: &mut PlaceholderAssets,
) {
    match visual {
        PlaceholderVisual::Generated(generated, part_name) => {
            commands.entity(root).with_children(|parent| {
                for part in generated.parts {
                    parent.spawn((
                        Name::new(part_name),
                        Mesh3d(part.mesh),
                        MeshMaterial3d(part.material),
                        Transform::from_translation(-generated.grip),
                        GrabTargetOutline(item),
                        OutlineVolume {
                            visible: false,
                            colour: Color::WHITE,
                            width: 4.0,
                        },
                        OutlineMode::FloodFlat,
                    ));
                }
            });
        }
        PlaceholderVisual::Fallback => {
            commands.entity(root).with_child((
                Name::new("Tactical item fallback"),
                ItemFallback(item),
                Mesh3d(assets.meshes.add(Cuboid::new(
                    physical.dimensions_m.x,
                    physical.dimensions_m.y,
                    physical.dimensions_m.z,
                ))),
                MeshMaterial3d(assets.materials.add(StandardMaterial {
                    base_color: Color::srgb(0.48, 0.34, 0.18),
                    perceptual_roughness: 0.8,
                    ..default()
                })),
                Transform::from_translation(-physical.anchor_offset_m),
                GrabTargetOutline(item),
                OutlineVolume {
                    visible: false,
                    colour: Color::WHITE,
                    width: 4.0,
                },
                OutlineMode::FloodFlat,
            ));
        }
    }
}

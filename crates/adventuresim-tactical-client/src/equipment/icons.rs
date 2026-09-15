//! Equipment portrait caching and HUD image selection.
use super::*;

pub(super) fn procedural_textures(
    contexts: &mut EguiContexts,
    cache: &mut WeaponIconCache,
    images: &mut Assets<Image>,
    weapons: &Query<(Entity, &WeaponAppearance)>,
    holders: &Query<(Entity, &WeaponHolderAppearance)>,
) -> HashMap<Entity, egui::TextureId> {
    let mut textures = HashMap::new();
    for (entity, appearance) in weapons {
        if let Some(handle) = cached_weapon_icon(appearance, cache, images) {
            textures.insert(
                entity,
                contexts.add_image(EguiTextureHandle::Weak(handle.id())),
            );
        }
    }
    for (entity, appearance) in holders {
        if let Some(handle) = cached_holder_icon(appearance, cache, images) {
            textures.insert(
                entity,
                contexts.add_image(EguiTextureHandle::Weak(handle.id())),
            );
        }
    }
    textures
}

impl WeaponIconCache {
    pub(super) fn armor_textures(
        &mut self,
        items: &Query<(
            Entity,
            &ItemOf,
            Option<&EquipSlot>,
            &ItemProperties,
            &EquipmentTopology,
        )>,
        scene_items: &Query<(Entity, &ItemProperties), With<TacticalSceneItem>>,
        assets: &AssetServer,
        contexts: &mut EguiContexts,
    ) -> HashMap<Entity, egui::TextureId> {
        // Exact carried placements win if an entity also retains its scene tag.
        scene_items
            .iter()
            .map(|(entity, item)| (entity, item, None))
            .chain(items.iter().map(|(entity, _, _, item, topology)| {
                (entity, item, topology.placement_id.as_deref())
            }))
            .filter_map(|(entity, item, placement)| {
                self.armor_texture(&item.id, placement, assets, contexts)
                    .map(|texture| (entity, texture))
            })
            .collect()
    }

    fn armor_texture(
        &mut self,
        item: &str,
        placement: Option<&str>,
        assets: &AssetServer,
        contexts: &mut EguiContexts,
    ) -> Option<egui::TextureId> {
        let file = procedural_equipment_file(item, placement)?;
        let stem = file.strip_suffix(".glb")?;
        let path = procedural_equipment_asset_path(&format!("icons/{stem}.png"));
        let handle = self
            .armor
            .entry(path.clone())
            .or_insert_with(|| assets.load(path));
        Some(contexts.add_image(EguiTextureHandle::Weak(handle.id())))
    }
}

pub(super) fn cached_weapon_icon(
    appearance: &WeaponAppearance,
    cache: &mut WeaponIconCache,
    images: &mut Assets<Image>,
) -> Option<Handle<Image>> {
    if appearance.recipe.len() > 16 * 1024
        || appearance.generator_version != adventuresim_weapon_model::GENERATOR_VERSION
    {
        return None;
    }
    let design = decode(&appearance.recipe).ok()?;
    if adventuresim_weapon_model::design_hash(&design).0 != appearance.design_hash {
        return None;
    }
    let key = WeaponIconCacheKey {
        source: IconSource::Weapon,
        generator_version: appearance.generator_version,
        renderer_version: ICON_RENDERER_VERSION,
        design_hash: appearance.design_hash,
        size: TACTICAL_WEAPON_ICON_SIZE,
        supersampling: TACTICAL_WEAPON_ICON_SUPERSAMPLING,
    };
    if let Some(cached) = cache.icons.get(&key) {
        return Some(cached.clone());
    }
    let icon = generate_icon(
        &design,
        WeaponIconSpec {
            size: TACTICAL_WEAPON_ICON_SIZE,
            supersampling: TACTICAL_WEAPON_ICON_SUPERSAMPLING,
        },
    )
    .ok()?;
    let rgba = icon.rgba;
    let handle = images.add(Image::new(
        Extent3d {
            width: u32::from(TACTICAL_WEAPON_ICON_SIZE),
            height: u32::from(TACTICAL_WEAPON_ICON_SIZE),
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    ));
    cache.icons.insert(key, handle.clone());
    Some(handle)
}

pub(super) fn cached_holder_icon(
    appearance: &WeaponHolderAppearance,
    cache: &mut WeaponIconCache,
    images: &mut Assets<Image>,
) -> Option<Handle<Image>> {
    if appearance.recipe.len() > 16 * 1024
        || appearance.generator_version != adventuresim_weapon_model::HOLDER_GENERATOR_VERSION
    {
        return None;
    }
    let design = adventuresim_weapon_model::decode_holder(&appearance.recipe).ok()?;
    if adventuresim_weapon_model::holder_design_hash(&design).0 != appearance.design_hash {
        return None;
    }
    let key = WeaponIconCacheKey {
        source: IconSource::Holder,
        generator_version: appearance.generator_version,
        renderer_version: ICON_RENDERER_VERSION,
        design_hash: appearance.design_hash,
        size: TACTICAL_WEAPON_ICON_SIZE,
        supersampling: TACTICAL_WEAPON_ICON_SUPERSAMPLING,
    };
    if let Some(cached) = cache.icons.get(&key) {
        return Some(cached.clone());
    }
    let icon = generate_holder_icon(
        &design,
        WeaponIconSpec {
            size: TACTICAL_WEAPON_ICON_SIZE,
            supersampling: TACTICAL_WEAPON_ICON_SUPERSAMPLING,
        },
    )
    .ok()?;
    let rgba = icon.rgba;
    let handle = images.add(Image::new(
        Extent3d {
            width: u32::from(TACTICAL_WEAPON_ICON_SIZE),
            height: u32::from(TACTICAL_WEAPON_ICON_SIZE),
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    ));
    cache.icons.insert(key, handle.clone());
    Some(handle)
}

pub(super) fn equipment_icon_image(
    entity: Option<Entity>,
    fallback_slug: &str,
    size: egui::Vec2,
    procedural: &HashMap<Entity, egui::TextureId>,
    atlas: egui::TextureId,
) -> egui::Image<'static> {
    if let Some(texture) = entity.and_then(|entity| procedural.get(&entity)).copied() {
        egui::Image::new((texture, size)).bg_fill(egui::Color32::BLACK)
    } else {
        egui::Image::new((atlas, size)).uv(icon_uv(fallback_slug))
    }
}

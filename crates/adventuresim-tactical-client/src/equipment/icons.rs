//! Equipment icons for the slot HUD: a cell of the catalogue atlas, an
//! authored armour portrait, or a weapon portrait generated from its recipe.
//! The page draws them (`tactical-hud.js`); generated portraits cross to it
//! once each, keyed by design.
use super::*;
use crate::hud;
use serde::Serialize;

pub(super) const TACTICAL_WEAPON_ICON_SIZE: u16 = 64;

const TACTICAL_WEAPON_ICON_SUPERSAMPLING: u8 = 4;

const EQUIPMENT_ICON_COLUMNS: usize = 8;
const EQUIPMENT_ICON_ROWS: usize = 7;

pub(super) const EQUIPMENT_ICON_SLUGS: [&str; 56] = [
    "ancient-sword",
    "arm-bandage",
    "armor-cuisses",
    "armor-vest",
    "barbute",
    "belt-armor",
    "bo",
    "bordered-shield",
    "bow-arrow",
    "bowie-knife",
    "bracer",
    "breastplate",
    "broad-dagger",
    "broadsword",
    "brodie-helmet",
    "chain-mail",
    "chest-armor",
    "crested-helmet",
    "crossbow",
    "daggers",
    "flanged-mace",
    "greaves",
    "halberd",
    "heavy-helm",
    "helmet",
    "knapsack",
    "layered-armor",
    "light-helm",
    "mailed-fist",
    "mail-shirt",
    "metal-skirt",
    "musket",
    "piercing-sword",
    "plain-dagger",
    "pocket-bow",
    "pteruges",
    "relic-blade",
    "rifle",
    "roman-shield",
    "round-shield",
    "saber-slash",
    "shield",
    "shirt",
    "skirt",
    "sleeveless-jacket",
    "spear-hook",
    "spears",
    "stiletto",
    "sword-hilt",
    "templar-shield",
    "trousers",
    "two-handed-sword",
    "visored-helm",
    "warhammer",
    "wood-axe",
    "wood-club",
];

const EQUIPMENT_ICON_ATLAS: &str = "tactical-equipment-icons.png";

/// Generated portraits already sent to the page, by design.
#[derive(Resource, Default)]
pub(super) struct WeaponIconCache {
    icons: HashMap<WeaponIconCacheKey, String>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct WeaponIconCacheKey {
    source: IconSource,
    generator_version: u16,
    renderer_version: u16,
    design_hash: [u8; 32],
    size: u16,
    supersampling: u8,
}

impl WeaponIconCacheKey {
    fn new(source: IconSource, generator_version: u16, design_hash: [u8; 32]) -> Self {
        Self {
            source,
            generator_version,
            renderer_version: ICON_RENDERER_VERSION,
            design_hash,
            size: TACTICAL_WEAPON_ICON_SIZE,
            supersampling: TACTICAL_WEAPON_ICON_SUPERSAMPLING,
        }
    }

    /// The page's name for the portrait.
    fn page_key(&self) -> String {
        let source = match self.source {
            IconSource::Weapon => "weapon",
            IconSource::Holder => "holder",
        };
        let hash: String = self
            .design_hash
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        format!(
            "{source}-{}-{}-{hash}",
            self.generator_version, self.renderer_version
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum IconSource {
    Weapon,
    Holder,
}

/// What the page draws for one item.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub(super) enum IconView {
    /// A cell of the catalogue atlas, a `columns` x `rows` grid image.
    Atlas {
        path: &'static str,
        column: usize,
        row: usize,
        columns: usize,
        rows: usize,
    },
    /// An image under the asset root.
    Asset { path: String },
    /// A portrait sent earlier with `hud::icon`, on a black backing.
    Generated { key: String },
}

impl IconView {
    /// The atlas cell for a catalogue icon slug; unknown slugs show the first.
    pub(super) fn atlas(slug: &str) -> Self {
        let index = EQUIPMENT_ICON_SLUGS
            .iter()
            .position(|candidate| *candidate == slug)
            .unwrap_or(0);
        Self::Atlas {
            path: EQUIPMENT_ICON_ATLAS,
            column: index % EQUIPMENT_ICON_COLUMNS,
            row: index / EQUIPMENT_ICON_COLUMNS,
            columns: EQUIPMENT_ICON_COLUMNS,
            rows: EQUIPMENT_ICON_ROWS,
        }
    }
}

/// Portraits for every weapon and holder in the scene, generating and sending
/// any the page does not have yet.
pub(super) fn procedural_icons(
    cache: &mut WeaponIconCache,
    weapons: &Query<(Entity, &WeaponAppearance)>,
    holders: &Query<(Entity, &WeaponHolderAppearance)>,
) -> HashMap<Entity, IconView> {
    let mut icons = HashMap::new();
    for (entity, appearance) in weapons {
        if let Some(key) = cached_weapon_icon(appearance, cache) {
            icons.insert(entity, IconView::Generated { key });
        }
    }
    for (entity, appearance) in holders {
        if let Some(key) = cached_holder_icon(appearance, cache) {
            icons.insert(entity, IconView::Generated { key });
        }
    }
    icons
}

/// Authored portraits of carried and scene armour pieces.
pub(super) fn armor_icons(
    items: &Query<(
        Entity,
        &ItemOf,
        Option<&EquipSlot>,
        &ItemProperties,
        &EquipmentTopology,
    )>,
    scene_items: &Query<(Entity, &ItemProperties), With<TacticalSceneItem>>,
) -> HashMap<Entity, IconView> {
    // Exact carried placements win if an entity also retains its scene tag.
    scene_items
        .iter()
        .map(|(entity, item)| (entity, item, None))
        .chain(
            items.iter().map(|(entity, _, _, item, topology)| {
                (entity, item, topology.placement_id.as_deref())
            }),
        )
        .filter_map(|(entity, item, placement)| {
            let file = procedural_equipment_file(&item.id, placement)?;
            let stem = file.strip_suffix(".glb")?;
            Some((
                entity,
                IconView::Asset {
                    path: format!("equipment/procedural/icons/{stem}.png"),
                },
            ))
        })
        .collect()
}

pub(super) fn cached_weapon_icon(
    appearance: &WeaponAppearance,
    cache: &mut WeaponIconCache,
) -> Option<String> {
    if appearance.recipe.len() > adventuresim_weapon_model::MAX_ENCODED_RECIPE_BYTES
        || appearance.generator_version != adventuresim_weapon_model::GENERATOR_VERSION
    {
        return None;
    }
    let design = decode(&appearance.recipe).ok()?;
    if adventuresim_weapon_model::design_hash(&design).0 != appearance.design_hash {
        return None;
    }
    let key = WeaponIconCacheKey::new(
        IconSource::Weapon,
        appearance.generator_version,
        appearance.design_hash,
    );
    cache.icon(key, || {
        generate_icon(&design, icon_spec())
            .ok()
            .map(|icon| icon.rgba)
    })
}

pub(super) fn cached_holder_icon(
    appearance: &WeaponHolderAppearance,
    cache: &mut WeaponIconCache,
) -> Option<String> {
    if appearance.recipe.len() > adventuresim_weapon_model::MAX_ENCODED_RECIPE_BYTES
        || appearance.generator_version != adventuresim_weapon_model::HOLDER_GENERATOR_VERSION
    {
        return None;
    }
    let design = adventuresim_weapon_model::decode_holder(&appearance.recipe).ok()?;
    if adventuresim_weapon_model::holder_design_hash(&design).0 != appearance.design_hash {
        return None;
    }
    let key = WeaponIconCacheKey::new(
        IconSource::Holder,
        appearance.generator_version,
        appearance.design_hash,
    );
    cache.icon(key, || {
        generate_holder_icon(&design, icon_spec())
            .ok()
            .map(|icon| icon.rgba)
    })
}

fn icon_spec() -> WeaponIconSpec {
    WeaponIconSpec {
        size: TACTICAL_WEAPON_ICON_SIZE,
        supersampling: TACTICAL_WEAPON_ICON_SUPERSAMPLING,
    }
}

impl WeaponIconCache {
    /// The page key of a portrait, rendering and sending it on first use.
    fn icon(
        &mut self,
        key: WeaponIconCacheKey,
        render: impl FnOnce() -> Option<Vec<u8>>,
    ) -> Option<String> {
        if let Some(cached) = self.icons.get(&key) {
            return Some(cached.clone());
        }
        let rgba = render()?;
        let page_key = key.page_key();
        let size = u32::from(TACTICAL_WEAPON_ICON_SIZE);
        hud::icon(&page_key, size, size, &rgba);
        self.icons.insert(key, page_key.clone());
        Some(page_key)
    }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.icons.len()
    }
}

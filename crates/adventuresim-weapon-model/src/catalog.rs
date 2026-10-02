//! Gameplay chassis registration and authored recipes over one kernel.
use crate::*;
use std::sync::OnceLock;
pub const PRESET_IDS: &[&str] = &[
    "german-self-bow-1544",
    "composite-recurve-bow-1544",
    "flight-arrow-1544",
    "arrow-quiver-1544",
    "german-cranequin-crossbow-1544",
    "central-composite-arbalest",
    "light-target-crossbow-comparative",
    "crossbow-bolt-1544",
    "bolt-quiver-1544",
    "peter-peck-double-wheellock-pistol-1545",
    "german-matchlock-arquebus-16c",
    "single-wheellock-pistol-study",
    "lead-round-ball",
    "small-arms-ball-pouch",
    "halberd-1540",
    "lucerne-hammer",
    "pollaxe",
    "kriegsspiess",
    "short-spear",
    "partisan",
    "glaive",
    "hooked-bill",
    "military-fork",
    "landsknecht-longsword",
    "zweihander",
    "katzbalger",
    "grosse-messer",
    "dussack",
    "estoc",
    "rondel-dagger",
    "reitschwert-1540",
    "reiter-war-hammer",
    "hand-axe",
    "flanged-mace",
    "gothic-flanged-mace",
    "buckler",
    "targe",
    "round-shield",
    "heater-shield",
    "pavise",
    "kite-shield",
    "roman-tower-shield",
];
pub const MELEE_CATALOG_IDS: &[&str] = &[
    "arming_sword",
    "baselard",
    "bauernwehr",
    "club",
    "falchion",
    "flanged_mace",
    "halberd",
    "hand_axe",
    "hunting_spear",
    "katzbalger",
    "knife",
    "kriegsmesser",
    "longsword",
    "messer",
    "military_pike",
    "misericorde",
    "rapier",
    "rondel_dagger",
    "spear",
    "utility_knife",
    "walking_staff",
    "war_hammer",
    "zweihander",
];

fn gameplay() -> &'static [WeaponDesign] {
    static CATALOG: OnceLock<Vec<WeaponDesign>> = OnceLock::new();
    CATALOG.get_or_init(|| {
        serde_json::from_str(include_str!("../catalog/gameplay.json"))
            .expect("authored gameplay recipes")
    })
}
/// Only registered gameplay chassis require per-instance physical inventory.
pub fn default_design(catalog_id: &str) -> Option<WeaponDesign> {
    gameplay()
        .iter()
        .find(|design| design.catalog_id == catalog_id)
        .cloned()
}
pub fn preset_design(id: &str) -> Option<WeaponDesign> {
    let catalog = crate::authoring::authoring_catalog();
    let preset = catalog["presets"]
        .as_array()?
        .iter()
        .find(|entry| entry["id"] == id)?;
    Some(WeaponDesign {
        catalog_id: id.into(),
        recipe: serde_json::from_value(preset["definition"].clone()).ok()?,
    })
}
pub fn recommended_holder(catalog_id: &str) -> Option<WeaponHolderKind> {
    match catalog_id {
        "arming_sword" | "baselard" | "bauernwehr" | "falchion" | "katzbalger" | "knife"
        | "kriegsmesser" | "longsword" | "messer" | "misericorde" | "rapier" | "rondel_dagger"
        | "utility_knife" | "zweihander" => Some(WeaponHolderKind::BladeSheath),
        "club" | "flanged_mace" | "hand_axe" | "war_hammer" => Some(WeaponHolderKind::HaftLoop),
        "halberd" | "hunting_spear" | "military_pike" | "spear" | "walking_staff" => None,
        _ => None,
    }
}

pub fn default_holder_design(weapon: &WeaponDesign) -> Option<WeaponHolderDesign> {
    let kind = recommended_holder(&weapon.catalog_id)?;
    Some(WeaponHolderDesign {
        catalog_id: match kind {
            WeaponHolderKind::BladeSheath => "scabbard",
            WeaponHolderKind::HaftLoop => "weapon_loop",
        }
        .into(),
        kind,
        fitted_weapon: weapon.clone(),
        body_material: Material::DarkLeather,
        fitting_material: Material::Brass,
        clearance: Millimeters(5),
        wall_thickness: Millimeters(2),
        throat_length: Millimeters(12),
        chape_length: Millimeters(20),
        loop_position: Permille(280),
        loop_bar_radius: Millimeters(4),
        hanger_width: Millimeters(42),
        hanger_height: Millimeters(76),
    })
}

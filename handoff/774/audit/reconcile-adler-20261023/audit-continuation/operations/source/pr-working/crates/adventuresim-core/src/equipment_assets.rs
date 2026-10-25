//! Procedural-equipment manifest wire records and catalogue-checked assets.

use crate::item_catalog::{self, EquipmentMaterial};
pub use selection::{CatalogItem, CatalogPlacement, OwnedCatalogPlacement};
use serde::{Deserialize, Serialize};

mod selection;

pub const GLB_EXTENSION: &str = "glb";
pub const EQUIPMENT_MANIFEST_SCHEMA_VERSION: u32 = 1;
pub const EQUIPMENT_MHR_RELEASE: &str = "v1.0.1";

#[derive(Serialize, Deserialize)]
pub struct ProceduralEquipmentManifest {
    pub assets: Vec<ProceduralEquipmentAsset>,
    pub lod: u8,
    pub mhr_release: String,
    pub schema_version: u32,
}

#[derive(Serialize, Deserialize)]
pub struct ProceduralEquipmentAsset {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub armor_design_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub armor_generator_version: Option<u16>,
    pub coverage: f32,
    pub file: String,
    pub item_id: String,
    pub material: Option<EquipmentMaterial>,
    pub morph_targets: usize,
    pub placement_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surface_uv_domain: Option<String>,
    pub triangles: usize,
}

pub struct EquipmentAssetEntry {
    pub selection: CatalogPlacement<'static>,
    pub file: String,
}

pub struct EquipmentAssets {
    entries: Vec<EquipmentAssetEntry>,
}

impl EquipmentAssets {
    pub fn from_json(source: &str) -> Result<Self, serde_json::Error> {
        let manifest: ProceduralEquipmentManifest = serde_json::from_str(source)?;
        let mut entries: Vec<EquipmentAssetEntry> = Vec::new();
        for asset in manifest.assets {
            let item =
                CatalogItem::find(item_catalog::catalog(), &asset.item_id).ok_or_else(|| {
                    <serde_json::Error as serde::de::Error>::custom(format!(
                        "unknown equipment manifest item {}",
                        asset.item_id
                    ))
                })?;
            let selection = item.placement(&asset.placement_id).ok_or_else(|| {
                <serde_json::Error as serde::de::Error>::custom(format!(
                    "unknown placement {} for {}",
                    asset.placement_id,
                    item.definition().id
                ))
            })?;
            if entries.iter().any(|entry| entry.selection == selection) {
                return Err(<serde_json::Error as serde::de::Error>::custom(
                    "procedural equipment manifest keys must be unique",
                ));
            }
            entries.push(EquipmentAssetEntry {
                selection,
                file: asset.file,
            });
        }
        // Preserve the previous nested ordered-map default-placement selection.
        entries.sort_by(|left, right| {
            (
                &left.selection.item().definition().id,
                &left.selection.definition().id,
            )
                .cmp(&(
                    &right.selection.item().definition().id,
                    &right.selection.definition().id,
                ))
        });
        Ok(Self { entries })
    }
    pub fn file_for(&self, selection: CatalogPlacement<'static>) -> Option<&str> {
        self.entries
            .iter()
            .find(|entry| entry.selection == selection)
            .map(|entry| entry.file.as_str())
    }

    /// The first lexical placement, matching the manifest's ordered lookup.
    pub fn default_file_for(&self, item: CatalogItem<'static>) -> Option<&str> {
        self.entries
            .iter()
            .find(|entry| entry.selection.item() == item)
            .map(|entry| entry.file.as_str())
    }

    pub fn entries(&self) -> impl Iterator<Item = &EquipmentAssetEntry> {
        self.entries.iter()
    }
}

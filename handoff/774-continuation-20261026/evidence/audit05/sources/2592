//! Procedural-equipment manifest wire records and catalogue-checked assets.

use crate::item_catalog::{self, EquipmentMaterial, EquipmentPlacement, ItemDefinition};
use serde::{Deserialize, Serialize};

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
    pub item: &'static ItemDefinition,
    pub placement: &'static EquipmentPlacement,
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
            let item = item_catalog::definition(&asset.item_id).ok_or_else(|| {
                <serde_json::Error as serde::de::Error>::custom(format!(
                    "unknown equipment manifest item {}",
                    asset.item_id
                ))
            })?;
            let placement = item
                .equipment
                .as_ref()
                .and_then(|equipment| {
                    equipment
                        .placements
                        .iter()
                        .find(|placement| placement.id == asset.placement_id)
                })
                .ok_or_else(|| {
                    <serde_json::Error as serde::de::Error>::custom(format!(
                        "unknown placement {} for {}",
                        asset.placement_id, item.id
                    ))
                })?;
            if entries.iter().any(|entry| {
                std::ptr::eq(entry.item, item) && std::ptr::eq(entry.placement, placement)
            }) {
                return Err(<serde_json::Error as serde::de::Error>::custom(
                    "procedural equipment manifest keys must be unique",
                ));
            }
            entries.push(EquipmentAssetEntry {
                item,
                placement,
                file: asset.file,
            });
        }
        // Preserve the previous nested ordered-map default-placement selection.
        entries.sort_by(|left, right| {
            (&left.item.id, &left.placement.id).cmp(&(&right.item.id, &right.placement.id))
        });
        Ok(Self { entries })
    }
    pub fn file_for(
        &self,
        item: &ItemDefinition,
        placement: Option<&EquipmentPlacement>,
    ) -> Option<&str> {
        self.entries
            .iter()
            .find(|entry| {
                std::ptr::eq(entry.item, item)
                    && placement.is_none_or(|placement| std::ptr::eq(entry.placement, placement))
            })
            .map(|entry| entry.file.as_str())
    }
    pub fn entries(&self) -> impl Iterator<Item = &EquipmentAssetEntry> {
        self.entries.iter()
    }
}

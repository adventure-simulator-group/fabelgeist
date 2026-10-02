//! Baked portraits follow the same catalog and placement manifest as tactical meshes.

use serde::Deserialize;
use std::{collections::BTreeMap, sync::LazyLock};

#[derive(Deserialize)]
struct Manifest {
    assets: Vec<Asset>,
}
#[derive(Deserialize)]
struct Asset {
    item_id: String,
    placement_id: String,
    file: String,
}

static PORTRAITS: LazyLock<BTreeMap<String, String>> = LazyLock::new(|| {
    let mut manifest: Manifest = serde_json::from_str(include_str!(
        "../../../../assets/equipment/procedural/manifest.json"
    ))
    .expect("procedural equipment manifest must be valid");
    manifest
        .assets
        .sort_by(|a, b| a.placement_id.cmp(&b.placement_id));
    let mut portraits = BTreeMap::new();
    for asset in manifest.assets {
        let stem = asset
            .file
            .strip_suffix(".glb")
            .expect("equipment asset must be GLB");
        portraits
            .entry(asset.item_id)
            .or_insert_with(|| format!("/static/equipment-icons/{stem}.png"));
    }
    portraits
});

pub(super) fn portrait(item_id: &str) -> Option<&'static str> {
    PORTRAITS.get(item_id).map(String::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_catalog_portrait_is_shipped() {
        for url in PORTRAITS.values() {
            let path =
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(url.trim_start_matches('/'));
            assert!(path.is_file(), "missing portrait: {}", path.display());
        }
        assert!(portrait("barbute").is_some());
        assert!(portrait("longsword").is_none());
    }
}

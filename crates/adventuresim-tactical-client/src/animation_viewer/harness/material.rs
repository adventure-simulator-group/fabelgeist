//! Report actual authored and renderer-bound surface maps without reading pixels.
use bevy::prelude::*;
use serde_json::{Value, json};

pub(super) fn textures(material: &StandardMaterial, images: &Assets<Image>) -> Value {
    let image = |handle: &Option<Handle<Image>>| {
        handle.as_ref().map(|handle| {
            let image = images.get(handle);
            json!({
                "asset_id": format!("{:?}", handle.id()),
                "asset_path": handle.path().map(ToString::to_string),
                "loaded": image.is_some(),
                "size": image.map(|image| [image.width(), image.height()]),
                "format": image.map(|image| format!("{:?}", image.texture_descriptor.format)),
                "mip_levels": image.map(|image| image.texture_descriptor.mip_level_count),
                "cpu_bytes": image.and_then(|image| image.data.as_ref()).map(Vec::len),
            })
        })
    };
    json!({
        "base_color": image(&material.base_color_texture),
        "normal": image(&material.normal_map_texture),
        "occlusion": image(&material.occlusion_texture),
        "normal_and_occlusion_distinct": match (&material.normal_map_texture, &material.occlusion_texture) {
            (Some(normal), Some(occlusion)) => Some(normal.id() != occlusion.id()),
            _ => None,
        },
    })
}

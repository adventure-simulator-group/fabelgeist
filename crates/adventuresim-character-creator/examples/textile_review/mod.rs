//! Shared native body, morph and mesh-export protocol for textile reviews.
use adventuresim_character_creator::{
    armor_recipes::ParametricDesign, device_body::DeviceBody, device_fit::Realization,
    review_component::ReviewComponent,
};
use anyhow::Result;
use fabelgeist_armor::GeneratedArmor;
use std::path::{Path, PathBuf};

pub use body::Body;

mod body;

pub struct MorphBodies {
    bodies: Vec<Body>,
    devices: Vec<DeviceBody>,
    names: Vec<String>,
}

impl MorphBodies {
    pub fn load(paths: &[PathBuf]) -> Result<Self> {
        let bodies = paths
            .iter()
            .map(|path| Ok(serde_json::from_slice::<Body>(&std::fs::read(path)?)?))
            .collect::<Result<Vec<_>>>()?;
        let devices = bodies.iter().map(|_| DeviceBody::default()).collect();
        let names = paths
            .iter()
            .map(|path| {
                path.file_stem()
                    .and_then(|v| v.to_str())
                    .map(str::to_owned)
                    .ok_or_else(|| anyhow::anyhow!("invalid morph filename"))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            bodies,
            devices,
            names,
        })
    }

    pub fn names(&self) -> Vec<&str> {
        self.names.iter().map(String::as_str).collect()
    }

    pub fn realizations(&self) -> Vec<(&str, Realization<'_>)> {
        self.bodies
            .iter()
            .zip(&self.devices)
            .zip(&self.names)
            .map(|((body, device), name)| (name.as_str(), body.realization(device)))
            .collect()
    }
}

pub fn write_generated(
    output: &Path,
    name: &str,
    placement: &str,
    design: &ParametricDesign,
    generated: &GeneratedArmor,
) -> Result<()> {
    let components = generated
        .components
        .iter()
        .map(ReviewComponent::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let textile_maps = write_textile_maps(output, name, generated)?;
    std::fs::write(
        output.join(format!("{name}.json")),
        serde_json::to_vec(&serde_json::json!({
            "id": name, "placement": placement, "design": design,
            "positions": generated.positions, "normals": generated.normals,
            "texcoords": generated.texcoords, "textile_maps": textile_maps,
            "indices": generated.indices, "components": components,
            "plate_faces": generated.faces,
            "joint_indices": generated.joint_indices.native_words(), "joint_weights": generated.joint_weights,
            "morphs": generated.morphs.iter().map(|morph| serde_json::json!({
                "name": morph.name, "positions": morph.direct_positions,
                "normal_deltas": morph.normal_deltas,
            })).collect::<Vec<_>>(),
            "generator_version": fabelgeist_armor::GENERATOR_VERSION,
        }))?,
    )?;
    Ok(())
}

fn write_textile_maps(
    output: &std::path::Path,
    article_name: &str,
    armor: &fabelgeist_armor::GeneratedArmor,
) -> Result<Vec<serde_json::Value>> {
    let mut maps = Vec::new();
    for (index, component) in armor.components.iter().enumerate() {
        if let Some(fabric) = component.appearance.textile()
            && let Some(textures) =
                adventuresim_character_creator::textile_surface::textures(fabric)?
        {
            let name = format!("{article_name}-textile-{index}-color.png");
            std::fs::write(output.join(&name), textures.base_color_png)?;
            let normal = format!("{article_name}-textile-{index}-normal.png");
            std::fs::write(output.join(&normal), textures.normal_png)?;
            maps.push(serde_json::json!({"component":index,"base_color":name,"normal":normal, "chart_boundary":fabric.chart_boundary}));
        }
    }
    Ok(maps)
}

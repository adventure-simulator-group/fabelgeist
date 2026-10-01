//! Runtime equipment generated on the armor device from the canonical rig's
//! mesh data, without an intermediate GLB.

use crate::{
    armor_frames::Side,
    armor_recipes::{self, DedicatedGenerator, ParametricDesign},
    bracer::{ForearmSide, ForearmSurfaceInput},
    clothing::{GarmentSpecification, generate_clothing_shells},
    device_body::DeviceBody,
    device_fit::{self, FitBody, Fitted, Realization},
    device_torso::TorsoSurfaceInput,
    device_underlayer::SurfaceDomain,
    underlayer_armor::{self, UnderlayerBody},
};
use anyhow::{Context, Result, bail};
use fabelgeist_armor::{
    ArmorGpu, BracerDesign, BreastplateDesign, GeneratedArmor, parametric_design_hash,
};
use std::sync::LazyLock;

/// One evaluated wearer in its unposed fitting frame. Runtime equipment has no
/// body-shape morph targets; a changed body requires a new fit.
#[derive(Clone)]
pub struct RuntimeBody {
    pub domain: String,
    pub faces: Vec<[u32; 3]>,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    /// One surface coordinate per vertex; `texcoord_faces` index them.
    pub texcoords: Vec<[f32; 2]>,
    pub texcoord_faces: Vec<[u32; 3]>,
    pub joint_indices: Vec<[u32; 8]>,
    pub joint_weights: Vec<[f32; 8]>,
    pub joint_names: Vec<String>,
    pub global_joint_states: Vec<[f32; 8]>,
    /// This body on the armor device, once a piece has uploaded it.
    pub device: DeviceBody,
}

impl RuntimeBody {
    fn validate(&self) -> Result<()> {
        let vertices = self.positions.len();
        if self.domain.trim().is_empty()
            || vertices == 0
            || self.normals.len() != vertices
            || self.texcoords.len() != vertices
            || self.joint_indices.len() != vertices
            || self.joint_weights.len() != vertices
            || self.joint_names.len() != self.global_joint_states.len()
            || self.faces.is_empty()
            || self.texcoord_faces.len() != self.faces.len()
        {
            bail!("runtime equipment body data is inconsistent");
        }
        Ok(())
    }

    fn fit_body(&self) -> FitBody<'_> {
        FitBody {
            faces: &self.faces,
            texcoords: &self.texcoords,
            joint_indices: &self.joint_indices,
            joint_weights: &self.joint_weights,
            joint_names: &self.joint_names,
        }
    }

    fn wearer(&self) -> Realization<'_> {
        Realization {
            positions: &self.positions,
            normals: &self.normals,
            joints: &self.global_joint_states,
            device: &self.device,
        }
    }
}

/// Generate one fitted equipment item: armor on the armor device, clothing
/// offset from the body.
pub async fn generate(
    body: &RuntimeBody,
    item_id: &str,
    placement: &str,
    bracer_design: &BracerDesign,
    breastplate_design: &BreastplateDesign,
    layers: &[&GeneratedArmor],
) -> Result<GeneratedArmor> {
    if is_runtime_armor(item_id) {
        generate_runtime_armor(
            body,
            item_id,
            placement,
            bracer_design,
            breastplate_design,
            layers,
        )
        .await
    } else {
        generate_runtime_clothing(body, item_id, placement)
    }
}

/// Generate one fitted armor piece on the armor device.
///
/// Fits the authored design only to this wearer, without equipment morphs.
pub async fn generate_runtime_armor(
    body: &RuntimeBody,
    item_id: &str,
    placement: &str,
    bracer_design: &BracerDesign,
    breastplate_design: &BreastplateDesign,
    layers: &[&GeneratedArmor],
) -> Result<GeneratedArmor> {
    body.validate()?;
    let gpu = crate::armor_gpu::armor_gpu_async().await?;
    match DedicatedGenerator::for_item(item_id) {
        Some(DedicatedGenerator::Vambrace) => {
            generate_vambrace(gpu, body, placement, bracer_design).await
        }
        Some(DedicatedGenerator::Breastplate) => {
            generate_breastplate(gpu, body, breastplate_design).await
        }
        None => {
            let design = armor_recipes::recipe(item_id)
                .with_context(|| format!("no runtime armor recipe for {item_id}"))?;
            generate_parametric(gpu, body, &design, placement, layers).await
        }
    }
}

async fn generate_vambrace(
    gpu: &ArmorGpu,
    body: &RuntimeBody,
    placement: &str,
    design: &BracerDesign,
) -> Result<GeneratedArmor> {
    let side = match Side::from_placement(placement)? {
        Side::Left => ForearmSide::Left,
        Side::Right => ForearmSide::Right,
    };
    crate::device_bracer::generate_bracer_on_device_async(
        gpu,
        design,
        ForearmSurfaceInput {
            domain: &body.domain,
            side,
            positions: &body.positions,
            normals: &body.normals,
            faces: &body.faces,
            texcoords: &body.texcoords,
            texcoord_faces: &body.texcoord_faces,
            joint_indices: &body.joint_indices,
            joint_weights: &body.joint_weights,
            joint_names: &body.joint_names,
            global_joint_states: &body.global_joint_states,
            morphs: &[],
        },
    )
    .await
}

async fn generate_breastplate(
    gpu: &ArmorGpu,
    body: &RuntimeBody,
    design: &BreastplateDesign,
) -> Result<GeneratedArmor> {
    crate::device_torso::generate_breastplate_on_device_async(
        gpu,
        design,
        TorsoSurfaceInput {
            domain: &body.domain,
            positions: &body.positions,
            normals: &body.normals,
            faces: &body.faces,
            texcoords: &body.texcoords,
            texcoord_faces: &body.texcoord_faces,
            joint_indices: &body.joint_indices,
            joint_weights: &body.joint_weights,
            joint_names: &body.joint_names,
            global_joint_states: &body.global_joint_states,
            morphs: &[],
        },
    )
    .await
}

async fn generate_parametric(
    gpu: &ArmorGpu,
    body: &RuntimeBody,
    design: &ParametricDesign,
    placement: &str,
    layers: &[&GeneratedArmor],
) -> Result<GeneratedArmor> {
    match design {
        ParametricDesign::Underlayer(_) | ParametricDesign::TrunkHose(_) => {
            let wearer = body.wearer().wearer(&body.fit_body());
            let cut = UnderlayerBody {
                wearer: &wearer,
                domain: SurfaceDomain {
                    uv_faces: &body.texcoord_faces,
                    texcoords: &body.texcoords,
                },
                surface_domain: &body.domain,
                proportions: &[],
            };
            match design {
                ParametricDesign::Underlayer(design) => {
                    underlayer_armor::fit_underlayer_async(gpu, design, placement, &cut, &[]).await
                }
                ParametricDesign::TrunkHose(design) => {
                    underlayer_armor::fit_trunk_hose_async(gpu, design, placement, &cut, &[]).await
                }
                _ => unreachable!(),
            }
        }
        _ => {
            let piece = device_fit::fit_recipe_async(
                gpu,
                &body.fit_body(),
                &body.wearer(),
                &[],
                design,
                placement,
                layers,
            )
            .await?;
            device_fit::assemble_recipe(
                design,
                piece,
                &Fitted {
                    placement,
                    morphs: &[],
                    domain: &body.domain,
                    joint_names: &body.joint_names,
                    joints: &body.global_joint_states,
                },
            )
        }
    }
}

/// Generate one fitted clothing item directly from the canonical body mesh.
pub fn generate_runtime_clothing(
    body: &RuntimeBody,
    item_id: &str,
    placement_id: &str,
) -> Result<GeneratedArmor> {
    body.validate()?;
    let item = catalog_item(item_id).with_context(|| format!("unknown clothing item {item_id}"))?;
    let equipment = item
        .equipment
        .as_ref()
        .with_context(|| format!("clothing item {item_id} has no equipment definition"))?;
    let material = equipment
        .material
        .with_context(|| format!("clothing item {item_id} has no material"))?;
    let placement = equipment
        .placements
        .iter()
        .find(|placement| placement.id == placement_id)
        .with_context(|| format!("clothing item {item_id} has no placement {placement_id}"))?;
    let specification = GarmentSpecification::from_catalog(
        format!("{item_id} · {placement_id}"),
        placement,
        material,
    );
    let generated = generate_clothing_shells(
        &[specification],
        &body.positions,
        &body.normals,
        &body.faces,
        &body.joint_indices,
        &body.joint_weights,
        &body.joint_names,
        &body.global_joint_states,
    )
    .map_err(anyhow::Error::msg)?;
    let shell = generated
        .shells
        .into_iter()
        .next()
        .context("clothing generator returned no shell")?;
    let base_positions = shell.positions.clone();
    let base_normals = shell.normals.clone();
    let indices = shell
        .faces
        .iter()
        .flat_map(|face| face.iter().copied())
        .collect::<Vec<_>>();
    let design_hash = parametric_design_hash(
        &serde_json::to_vec(&(item_id, placement_id)).expect("clothing IDs serialize"),
    );
    Ok(GeneratedArmor {
        design_hash,
        surface_domain: body.domain.clone(),
        positions: base_positions,
        normals: base_normals,
        texcoords: body.texcoords.clone(),
        joint_indices: body.joint_indices.clone(),
        joint_weights: body.joint_weights.clone(),
        indices,
        // Offset from the body: a cloth layer, not a thickened plate.
        faces: Vec::new(),
        trim: None,
        grids: Vec::new(),
        morphs: Vec::new(),
        components: Vec::new(),
    })
}

static ITEM_CATALOG: LazyLock<Vec<crate::item_catalog_schema::ItemDefinition>> =
    LazyLock::new(|| {
        let document: crate::item_catalog_schema::ItemCatalogDocument =
            serde_json::from_str(include_str!("../../../content/items/catalog.yaml"))
                .expect("embedded item catalog must be valid");
        document.items
    });

fn catalog_item(item_id: &str) -> Option<&'static crate::item_catalog_schema::ItemDefinition> {
    ITEM_CATALOG.iter().find(|item| item.id == item_id)
}

/// The material assigned to an authored runtime equipment item.
pub fn runtime_equipment_material(
    item_id: &str,
) -> Option<crate::item_catalog_schema::EquipmentMaterial> {
    catalog_item(item_id)
        .and_then(|item| item.equipment.as_ref())
        .and_then(|equipment| equipment.material)
}

/// Whether the item has an authored parametric armor runtime path.
pub fn is_runtime_armor(item_id: &str) -> bool {
    armor_recipes::is_parametric(item_id)
}

/// Whether the item has an authored fitted clothing runtime path.
pub fn is_runtime_clothing(item_id: &str) -> bool {
    catalog_item(item_id).is_some_and(|item| {
        (matches!(item.kind, crate::item_catalog_schema::ItemKind::Clothing)
            || item_id == "leather_belt")
            && !is_runtime_armor(item_id)
    })
}

/// Whether the item should be generated from the loaded body rig.
pub fn is_runtime_equipment(item_id: &str) -> bool {
    is_runtime_armor(item_id) || is_runtime_clothing(item_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_clothing_uses_the_runtime_path() {
        for item_id in ["linen_tunic", "linen_breeches", "leather_belt"] {
            assert!(is_runtime_clothing(item_id), "{item_id}");
            assert!(is_runtime_equipment(item_id), "{item_id}");
        }
        for item_id in ["leather_boot", "vambrace", "breastplate", "cuirass"] {
            assert!(is_runtime_armor(item_id), "{item_id}");
            assert!(!is_runtime_clothing(item_id), "{item_id}");
        }
        assert!(!is_runtime_equipment("arming_sword"));
    }
}

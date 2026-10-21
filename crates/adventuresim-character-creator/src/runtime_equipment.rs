//! Runtime equipment generated on the armor device from the canonical rig's
//! mesh data, without an intermediate GLB.

use crate::{
    armor_frames::{FittingPlacement, Side},
    armor_recipes::{self, DedicatedGenerator, ParametricDesign},
    bracer::{ForearmSide, ForearmSurfaceInput},
    device_body::DeviceBody,
    device_fit::{self, FitBody, Fitted, Realization},
    device_torso::TorsoSurfaceInput,
    device_underlayer::SurfaceDomain,
    underlayer_armor::{self, UnderlayerBody},
};
use adventuresim_core::{
    equipment_assets::{CatalogItem, CatalogPlacement},
    item_catalog, item_references::BELT_ID,
};
use anyhow::{Context, Result, bail};
use clothing::RuntimeClothing;
use fabelgeist_armor::{ArmorGpu, BracerDesign, BreastplateDesign, GeneratedArmor};
use fabelgeist_rig::RigJointName;
use std::sync::LazyLock;
pub use warmup::{WarmupReport, warm_up};

mod clothing;
mod warmup;

static ITEM_CATALOG: LazyLock<Vec<RuntimeCatalogItem>> = LazyLock::new(|| {
    item_catalog::authored_document_items()
        .into_iter()
        .filter_map(|item| CatalogItem::admit(item_catalog::catalog(), item))
        .map(RuntimeCatalogItem::from_catalog)
        .collect()
});

/// A canonical catalogue reference with clothing eligibility classified once.
#[derive(Clone, Copy)]
struct RuntimeCatalogItem {
    definition: CatalogItem<'static>,
    clothing_role: RuntimeClothingRole,
}

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
    pub joint_indices: fabelgeist_rig::PackedRigSkin<8>,
    pub joint_weights: Vec<[f32; 8]>,
    pub joint_names: fabelgeist_rig::RigJointTable<RigJointName>,
    pub global_joint_states: fabelgeist_rig::RigJointTable<[f32; 8]>,
    /// This body on the armor device, once a piece has uploaded it.
    pub device: DeviceBody,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RuntimeClothingRole {
    Fitted,
    Unsupported,
}

impl RuntimeCatalogItem {
    fn from_catalog(item: CatalogItem<'static>) -> Self {
        let fitted = matches!(item.kind, crate::item_catalog_schema::ItemKind::Clothing)
            || item.id == BELT_ID;
        Self {
            definition: item,
            clothing_role: if fitted {
                RuntimeClothingRole::Fitted
            } else {
                RuntimeClothingRole::Unsupported
            },
        }
    }

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
            || self.joint_indices.joint_count() != self.joint_names.joint_count()
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
    selection: CatalogPlacement<'static>,
    bracer_design: &BracerDesign,
    breastplate_design: &BreastplateDesign,
    layers: &[&GeneratedArmor],
) -> Result<GeneratedArmor> {
    if armor_recipes::is_parametric(selection.item()) {
        generate_runtime_armor(
            body,
            selection,
            bracer_design,
            breastplate_design,
            layers,
        )
        .await
    } else {
        body.validate()?;
        catalog_record(selection.item()).with_context(|| {
            format!("unknown clothing item {}", selection.item().id)
        })?;
        RuntimeClothing::new(body, selection).generate()
    }
}

/// Generate one fitted armor piece on the armor device.
///
/// Fits the authored design only to this wearer, without equipment morphs.
pub async fn generate_runtime_armor(
    body: &RuntimeBody,
    selection: CatalogPlacement<'static>,
    bracer_design: &BracerDesign,
    breastplate_design: &BreastplateDesign,
    layers: &[&GeneratedArmor],
) -> Result<GeneratedArmor> {
    body.validate()?;
    let gpu = crate::armor_gpu::armor_gpu_async().await?;
    let item = selection.item();
    match DedicatedGenerator::for_item(item) {
        Some(DedicatedGenerator::Vambrace) => {
            generate_vambrace(gpu, body, selection, bracer_design).await
        }
        Some(DedicatedGenerator::Breastplate) => {
            generate_breastplate(gpu, body, breastplate_design).await
        }
        None => {
            let design = armor_recipes::recipe(item)
                .with_context(|| format!("no runtime armor recipe for {}", item.id))?;
            generate_parametric(gpu, body, &design, selection, layers).await
        }
    }
}

/// The material assigned to an authored runtime equipment item.
pub fn runtime_equipment_material(
    item: CatalogItem<'static>,
) -> Option<crate::item_catalog_schema::EquipmentMaterial> {
    catalog_record(item)
        .and_then(|item| item.definition.definition().equipment.as_ref())
        .and_then(|equipment| equipment.material)
}

/// Whether the item has an authored parametric armor runtime path.
pub fn is_runtime_armor(item_id: &str) -> bool {
    CatalogItem::find(item_catalog::catalog(), item_id).is_some_and(armor_recipes::is_parametric)
}

/// Whether the item has an authored fitted clothing runtime path.
pub fn is_runtime_clothing(item_id: &str) -> bool {
    catalog_item(item_id).is_some_and(|item| {
        item.clothing_role == RuntimeClothingRole::Fitted
            && !armor_recipes::is_parametric(item.definition)
    })
}

/// Whether the item should be generated from the loaded body rig.
pub fn is_runtime_equipment(item_id: &str) -> bool {
    CatalogItem::find(item_catalog::catalog(), item_id).is_some_and(|item| {
        armor_recipes::is_parametric(item)
            || catalog_record(item).is_some_and(|record| record.clothing_role == RuntimeClothingRole::Fitted)
    })
}

async fn generate_vambrace(
    gpu: &ArmorGpu,
    body: &RuntimeBody,
    placement: CatalogPlacement<'_>,
    design: &BracerDesign,
) -> Result<GeneratedArmor> {
    let side = match FittingPlacement::from(placement).side()? {
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
    placement: CatalogPlacement<'_>,
    layers: &[&GeneratedArmor],
) -> Result<GeneratedArmor> {
    let placement = FittingPlacement::from(placement);
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

fn catalog_record(item: CatalogItem<'static>) -> Option<RuntimeCatalogItem> {
    ITEM_CATALOG.iter().find(|record| record.definition == item).copied()
}

fn catalog_item(item_id: &str) -> Option<RuntimeCatalogItem> {
    let item = CatalogItem::find(item_catalog::catalog(), item_id)?;
    catalog_record(item)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_clothing_uses_the_runtime_path() {
        for item_id in ["linen_tunic", "linen_breeches", "belt"] {
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

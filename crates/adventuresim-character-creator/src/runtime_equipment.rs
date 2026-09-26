//! Runtime armor generation from the canonical rig's CPU mesh data.

use crate::{
    armor_design_input::ArmorPlacement,
    armor_frames::Wearer,
    armor_recipes::{self, ParametricDesign},
    bracer::{ForearmMorphSample, ForearmSide, ForearmSurfaceInput, build_forearm_surface},
    breastplate::{TorsoSurfaceInput, build_front_torso_surface},
    clothing::{GarmentSpecification, generate_clothing_shells},
    nearest_vertex::NearestVertices,
};
use adventuresim_armor_model::{ArmorMorph, GeneratedArmor, PartMesh, parametric_design_hash};
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::sync::LazyLock;

#[derive(Clone, Copy, Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum FittedArmorKind {
    Vambrace,
    Breastplate,
    Cuirass,
}

impl FittedArmorKind {
    fn parse(item_id: &str) -> Option<Self> {
        Self::deserialize(
            serde::de::value::StrDeserializer::<serde::de::value::Error>::new(item_id),
        )
        .ok()
    }
}

/// One body realization used to refit an armor piece.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeBodyMorph {
    pub name: String,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub global_joint_states: Vec<[f32; 8]>,
}

/// The body data needed to generate an armor mesh without an intermediate GLB.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeBody {
    pub detail: adventuresim_armor_model::ArmorDetail,
    pub domain: String,
    pub faces: Vec<[u32; 3]>,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub texcoords: Vec<[f32; 2]>,
    pub texcoord_faces: Vec<[u32; 3]>,
    pub joint_indices: Vec<[u32; 8]>,
    pub joint_weights: Vec<[f32; 8]>,
    pub joint_names: Vec<String>,
    pub global_joint_states: Vec<[f32; 8]>,
    pub morphs: Vec<RuntimeBodyMorph>,
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
            || self.morphs.iter().any(|morph| {
                morph.positions.len() != vertices
                    || morph.normals.len() != vertices
                    || morph.global_joint_states.len() != self.joint_names.len()
            })
        {
            bail!("runtime armor body data is inconsistent");
        }
        Ok(())
    }

    fn wearer<'a>(
        &'a self,
        positions: &'a [[f32; 3]],
        normals: &'a [[f32; 3]],
        joints: &'a [[f32; 8]],
    ) -> Wearer<'a> {
        Wearer {
            detail: self.detail,
            faces: &self.faces,
            positions,
            normals,
            joint_indices: &self.joint_indices,
            joint_weights: &self.joint_weights,
            joint_names: &self.joint_names,
            joints,
        }
    }

    fn forearm_morphs(&self) -> Vec<ForearmMorphSample> {
        self.morphs
            .iter()
            .map(|morph| ForearmMorphSample {
                name: morph.name.clone(),
                positions: morph.positions.clone(),
                normals: morph.normals.clone(),
                global_joint_states: morph.global_joint_states.clone(),
            })
            .collect()
    }
}

/// Generate one fitted armor piece directly into renderer-independent data.
///
/// The output contains the same runtime-resolution geometry and morph
/// correspondence as the exported equipment, but no GLB serialization step.
pub fn generate_runtime_armor(
    body: &RuntimeBody,
    item_id: &str,
    placement: &str,
    bracer_design: &adventuresim_armor_model::BracerDesign,
    breastplate_design: &adventuresim_armor_model::BreastplateDesign,
) -> Result<GeneratedArmor> {
    body.validate()?;
    match FittedArmorKind::parse(item_id) {
        Some(FittedArmorKind::Vambrace) => generate_vambrace(body, placement, bracer_design),
        Some(FittedArmorKind::Breastplate | FittedArmorKind::Cuirass) => {
            generate_breastplate(body, breastplate_design)
        }
        None => {
            let design = armor_recipes::recipe(item_id)
                .with_context(|| format!("no runtime armor recipe for {item_id}"))?;
            generate_parametric(body, &design, placement)
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
    let morphs = body
        .morphs
        .iter()
        .map(|morph| {
            let fitted = shell
                .refit(&morph.positions, &morph.normals)
                .map_err(anyhow::Error::msg)?;
            Ok(ArmorMorph {
                name: morph.name.clone(),
                direct_positions: fitted.positions.clone(),
                position_deltas: deltas(&base_positions, &fitted.positions),
                normal_deltas: deltas(&base_normals, &fitted.normals),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(GeneratedArmor {
        construction_faces: Vec::new(),
        plate_edges: Vec::new(),
        components: Vec::new(),
        design_hash,
        surface_domain: body.domain.clone(),
        positions: base_positions,
        normals: base_normals,
        texcoords: body.texcoords.clone(),
        normal_map: None,
        joint_indices: body.joint_indices.clone(),
        joint_weights: body.joint_weights.clone(),
        indices,
        morphs,
    })
}

fn generate_vambrace(
    body: &RuntimeBody,
    placement: &str,
    design: &adventuresim_armor_model::BracerDesign,
) -> Result<GeneratedArmor> {
    let side = match ArmorPlacement::parse(placement) {
        Some(ArmorPlacement::Left) => ForearmSide::Left,
        Some(ArmorPlacement::Right) => ForearmSide::Right,
        _ => bail!("vambrace placement {placement} has no forearm side"),
    };
    let morphs = body.forearm_morphs();
    let surface = build_forearm_surface(ForearmSurfaceInput {
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
        morphs: &morphs,
    })
    .map_err(anyhow::Error::msg)?;
    adventuresim_armor_model::generate_bracer(design, &surface).map_err(anyhow::Error::new)
}

fn generate_breastplate(
    body: &RuntimeBody,
    design: &adventuresim_armor_model::BreastplateDesign,
) -> Result<GeneratedArmor> {
    let morphs = body.forearm_morphs();
    let surface = build_front_torso_surface(TorsoSurfaceInput {
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
        morphs: &morphs,
    })
    .map_err(anyhow::Error::msg)?;
    adventuresim_armor_model::generate_breastplate(design, &surface).map_err(anyhow::Error::new)
}

fn generate_parametric(
    body: &RuntimeBody,
    design: &ParametricDesign,
    placement: &str,
) -> Result<GeneratedArmor> {
    let mesh = armor_recipes::fitted_mesh(
        design,
        placement,
        &body.wearer(&body.positions, &body.normals, &body.global_joint_states),
        &[],
    )?;
    let mut armor = assemble_parametric(body, design, mesh)?;
    for morph in &body.morphs {
        let endpoint = armor_recipes::fitted_mesh(
            design,
            placement,
            &body.wearer(&morph.positions, &morph.normals, &morph.global_joint_states),
            &[],
        )?;
        if endpoint.indices != armor.indices || endpoint.positions.len() != armor.positions.len() {
            bail!("runtime armor morph changed topology for {placement}");
        }
        let normals = endpoint.normals().map_err(anyhow::Error::new)?;
        armor.morphs.push(ArmorMorph {
            name: morph.name.clone(),
            direct_positions: endpoint.positions.clone(),
            position_deltas: deltas(&armor.positions, &endpoint.positions),
            normal_deltas: deltas(&armor.normals, &normals),
        });
    }
    Ok(armor)
}

fn assemble_parametric(
    body: &RuntimeBody,
    design: &ParametricDesign,
    mesh: PartMesh,
) -> Result<GeneratedArmor> {
    let nearest = NearestVertices::new(&body.positions);
    let source = mesh
        .positions
        .iter()
        .map(|position| nearest.nearest(*position))
        .collect::<Vec<_>>();
    let body_texcoords = source
        .iter()
        .map(|index| body.texcoords[*index])
        .collect::<Vec<_>>();
    let (normal_map, texcoords) = mesh
        .runtime_fluting(&body_texcoords)
        .map_err(anyhow::Error::new)?
        .map_or((None, body_texcoords), |(map, texcoords)| {
            (Some(map), texcoords)
        });
    let normals = mesh.normals().map_err(anyhow::Error::new)?;
    let encoded = serde_json::to_vec(design)?;
    Ok(GeneratedArmor {
        construction_faces: mesh.construction_face_ranges(),
        plate_edges: Vec::new(),
        components: mesh.components,
        design_hash: parametric_design_hash(&encoded),
        surface_domain: body.domain.clone(),
        positions: mesh.positions,
        normals,
        texcoords,
        normal_map,
        joint_indices: source
            .iter()
            .map(|index| body.joint_indices[*index])
            .collect(),
        joint_weights: source
            .iter()
            .map(|index| body.joint_weights[*index])
            .collect(),
        indices: mesh.indices,
        morphs: Vec::new(),
    })
}

fn deltas(base: &[[f32; 3]], sample: &[[f32; 3]]) -> Vec<[f32; 3]> {
    base.iter()
        .zip(sample)
        .map(|(base, sample)| std::array::from_fn(|axis| sample[axis] - base[axis]))
        .collect()
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

/// Returns whether the item has an authored parametric armor runtime path.
pub fn is_runtime_armor(item_id: &str) -> bool {
    matches!(item_id, "vambrace" | "breastplate" | "cuirass")
        || armor_recipes::is_parametric(item_id)
}

/// Returns whether the item has an authored fitted clothing runtime path.
pub fn is_runtime_clothing(item_id: &str) -> bool {
    catalog_item(item_id).is_some_and(|item| {
        (matches!(item.kind, crate::item_catalog_schema::ItemKind::Clothing)
            || item_id == "leather_belt")
            && !is_runtime_armor(item_id)
    })
}

/// Returns whether the item should be generated from the loaded body rig.
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
        assert!(is_runtime_armor("leather_boot"));
        assert!(!is_runtime_clothing("leather_boot"));
        assert!(!is_runtime_equipment("arming_sword"));
    }
}

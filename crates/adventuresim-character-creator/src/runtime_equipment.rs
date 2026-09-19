//! Runtime armor generation from the canonical rig's CPU mesh data.

use crate::{
    armor_frames::Wearer,
    armor_recipes::{self, ParametricDesign},
    bracer::{ForearmMorphSample, ForearmSide, ForearmSurfaceInput, build_forearm_surface},
    breastplate::{TorsoSurfaceInput, build_front_torso_surface},
    nearest_vertex::NearestVertices,
};
use adventuresim_armor_model::{ArmorMorph, GeneratedArmor, PartMesh, parametric_design_hash};
use anyhow::{Context, Result, bail};

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
    match item_id {
        "vambrace" => generate_vambrace(body, placement, bracer_design),
        "breastplate" | "cuirass" => generate_breastplate(body, breastplate_design),
        _ => {
            let design = armor_recipes::recipe(item_id)
                .with_context(|| format!("no runtime armor recipe for {item_id}"))?;
            generate_parametric(body, &design, placement)
        }
    }
}

fn generate_vambrace(
    body: &RuntimeBody,
    placement: &str,
    design: &adventuresim_armor_model::BracerDesign,
) -> Result<GeneratedArmor> {
    let side = match placement {
        "left" => ForearmSide::Left,
        "right" => ForearmSide::Right,
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

/// The material assigned to an authored runtime armor item.
pub fn runtime_armor_material(
    item_id: &str,
) -> Option<crate::item_catalog_schema::EquipmentMaterial> {
    static CATALOG: std::sync::LazyLock<Vec<crate::item_catalog_schema::ItemDefinition>> =
        std::sync::LazyLock::new(|| {
            let document: crate::item_catalog_schema::ItemCatalogDocument =
                serde_json::from_str(include_str!("../../../content/items/catalog.yaml"))
                    .expect("embedded item catalog must be valid");
            document.items
        });
    CATALOG
        .iter()
        .find(|item| item.id == item_id)
        .and_then(|item| item.equipment.as_ref())
        .and_then(|equipment| equipment.material)
}

/// Returns whether the item has an authored parametric armor runtime path.
pub fn is_runtime_armor(item_id: &str) -> bool {
    matches!(item_id, "vambrace" | "breastplate" | "cuirass")
        || armor_recipes::is_parametric(item_id)
}

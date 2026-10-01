use super::*;
#[derive(Component)]
pub(crate) struct RuntimeEquipmentPresentation {
    pub(super) item: Entity,
    pub(super) item_id: String,
    pub(super) placement_id: String,
}

mod body;
mod generation;
mod layers;
mod meshes;
mod rig;
mod sockets;
use body::try_build_runtime_body;
use meshes::{CachedPart, equipment_parts, referenced_center};

use adventuresim_character_creator::{
    design_input::{load_bracer_design, load_breastplate_design},
    runtime_equipment::RuntimeBody,
};
use anyhow::{Context, bail};
use bevy::{
    gltf::{Gltf, GltfMesh, GltfNode, GltfSkin},
    mesh::{VertexAttributeValues, morph::MorphAttributes, skinning::SkinnedMeshInverseBindposes},
};
use fabelgeist_armor::GeneratedArmor;

mod morphs;
use generation::{FitKey, FittedBody, PendingFit};
use morphs::BodyShapeKey;
use std::sync::Arc;

#[derive(Resource, Default)]
pub(crate) struct RuntimeEquipmentBodyCache {
    body: Option<body::CanonicalBody>,
    pub(super) bracer_design: Option<fabelgeist_armor::BracerDesign>,
    pub(super) breastplate_design: Option<fabelgeist_armor::BreastplateDesign>,
    pub(super) failed: bool,
    bodies: HashMap<BodyShapeKey, FittedBody>,
    models: HashMap<FitKey, Result<CachedEquipment, String>>,
    pending: Option<PendingFit>,
    use_clock: u64,
    last_used: HashMap<FitKey, u64>,
}

impl RuntimeEquipmentBodyCache {
    /// Start a fresh cache with a validated torso recipe. Existing physical
    /// fits are never reinterpreted when a review fixture chooses a design.
    pub(crate) fn with_breastplate(
        design: fabelgeist_armor::BreastplateDesign,
    ) -> anyhow::Result<Self> {
        fabelgeist_armor::validate_breastplate(&design)?;
        Ok(Self {
            breastplate_design: Some(design),
            ..default()
        })
    }
}

#[expect(clippy::too_many_arguments)]
pub(super) fn prepare_runtime_equipment_body(
    animation: Res<crate::animation::AnimationRuntime>,
    gltfs: Res<Assets<Gltf>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    gltf_nodes: Res<Assets<GltfNode>>,
    gltf_skins: Res<Assets<GltfSkin>>,
    meshes: Res<Assets<Mesh>>,
    inverse_bindposes: Res<Assets<SkinnedMeshInverseBindposes>>,
    mut cache: ResMut<RuntimeEquipmentBodyCache>,
) {
    if cache.failed {
        return;
    }

    if cache.body.is_none() {
        let Some(base_handle) = animation.requested_base() else {
            return;
        };
        let Some(body) = try_build_runtime_body(
            base_handle,
            &gltfs,
            &gltf_meshes,
            &gltf_nodes,
            &gltf_skins,
            &meshes,
            &inverse_bindposes,
        ) else {
            return;
        };

        match body {
            Ok(body) => cache.body = Some(body),
            Err(error) => {
                error!("failed to prepare the runtime armor body: {error:#}");
                cache.failed = true;
                return;
            }
        }
    }

    if cache.bracer_design.is_none() {
        match load_bracer_design(None) {
            Ok(design) => cache.bracer_design = Some(design),
            Err(error) => {
                error!("failed to load the runtime bracer design: {error:#}");
                cache.failed = true;
                return;
            }
        }
    }
    if cache.breastplate_design.is_none() {
        match load_breastplate_design(None) {
            Ok(design) => cache.breastplate_design = Some(design),
            Err(error) => {
                error!("failed to load the runtime breastplate design: {error:#}");
                cache.failed = true;
            }
        }
    }
}

#[derive(Clone)]
struct CachedEquipment {
    generated: Arc<GeneratedArmor>,
    parts: Vec<CachedPart>,
    rigid_center: Vec3,
    sockets: BTreeMap<String, Transform>,
}

pub(super) use generation::generate_runtime_equipment_models;

/// The presentation of an item generated on the armor device, if it is one.
pub(super) fn presentation(
    item: Entity,
    properties: Option<&ItemProperties>,
    topology: Option<&EquipmentTopology>,
) -> Option<RuntimeEquipmentPresentation> {
    let properties = properties.filter(|properties| {
        adventuresim_character_creator::runtime_equipment::is_runtime_equipment(&properties.id)
    })?;
    Some(RuntimeEquipmentPresentation {
        item,
        item_id: properties.id.clone(),
        placement_id: topology
            .and_then(|topology| topology.placement_id.clone())
            .unwrap_or_else(|| "worn".into()),
    })
}

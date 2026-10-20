//! Complete owned census and separately selected camera-visible draw demand.
use super::super::*;
use crate::presentation::GrownSource;
use adventuresim_environment::{
    prepared::{LibraryId, RecordSpan},
    render::ForestPart,
};
use bevy::{asset::UntypedAssetId, render::storage::ShaderBuffer};

#[derive(Clone, Debug, PartialEq)]
pub(in super::super) struct SourceBinding {
    pub(in super::super) role: adventuresim_environment::render::grown::GrownBufferRole,
    pub(in super::super) handle: Handle<ShaderBuffer>,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(in super::super) struct PartOrdinal(pub(in super::super) usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(in super::super) struct PartIdentity {
    pub(in super::super) bank: LibraryId,
    pub(in super::super) source: GrownSource,
    pub(in super::super) encoding: GrownVertexEncoding,
    pub(in super::super) level: Level,
    pub(in super::super) ordinal: PartOrdinal,
    pub(in super::super) role: ForestPart,
    pub(in super::super) records: RecordSpan,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in super::super) struct Member {
    pub entity: Entity,
    pub root: Entity,
    pub part: PartIdentity,
    pub mesh: AssetId<Mesh>,
    pub material: UntypedAssetId,
    pub transform: Mat4,
}

#[derive(Clone, Copy)]
pub(in super::super) struct RootMember {
    pub(in super::super) entity: Entity,
    pub(in super::super) origin: RootOrigin,
    pub(in super::super) frame: Mat4,
}

#[derive(Resource, Default)]
pub(in super::super) struct CurrentInventory {
    pub(in super::super) roots: Vec<RootMember>,
    pub(in super::super) buffers: Vec<SourceBinding>,
    pub members: Vec<Member>,
    pub visible: Vec<Entity>,
    pub(in super::super) demand: DrawDemand,
    pub valid: bool,
    pub background: Vec<background::Member>,
    pub(in super::super) selection: Option<Arc<generated::selection::CheckpointSelection>>,
}

/// Required proof membership is independent of actual native camera visibility.
#[derive(Default)]
pub(in super::super) enum DrawDemand {
    #[default]
    AllOwned,
    CameraVisible,
    FrozenCamera {
        members: Vec<Entity>,
    },
}

#[derive(Clone, Copy, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(in super::super) enum RootOrigin {
    Specimen { ordinal: layout::RootOrdinal },
    Generated { source: GrownSource },
    Vista { source: GrownSource },
}

impl CurrentInventory {
    pub(in super::super) fn requires(&self, entity: Entity) -> bool {
        match &self.demand {
            DrawDemand::AllOwned => true,
            DrawDemand::CameraVisible => self.visible.contains(&entity),
            DrawDemand::FrozenCamera { members } => members.contains(&entity),
        }
    }

    pub(in super::super) fn report(&self) -> serde_json::Value {
        serde_json::json!({"selection":self.selection.as_ref().map(|selection|selection.report()),"whole_inventory_verified":self.valid,"owned_grown_roots":self.roots.len(),"roots":self.roots.iter().map(|root|serde_json::json!({"entity_bits":root.entity.to_bits(),"origin":root.origin,"world_from_root":root.frame.to_cols_array()})).collect::<Vec<_>>(),"members":self.members.iter().map(|member|serde_json::json!({"entity_bits":member.entity.to_bits(),"root_bits":member.root.to_bits(),"part":member.part,"mesh":format!("{:?}",member.mesh),"material":format!("{:?}",member.material),"world_from_mesh":member.transform.to_cols_array()})).collect::<Vec<_>>(),"draw_policy":"ordinary_gpu_membership","limits":"held source/asset identities and actual GPU instance membership; no CPU byte seals, shadow/prepass membership or performance claim"})
    }
}

impl SourceBinding {
    pub(in super::super) fn from_assets(assets: &GrownAssets) -> Vec<Self> {
        assets
            .buffers()
            .map(|buffer| Self {
                role: buffer.role(),
                handle: buffer.handle().clone(),
            })
            .collect()
    }
}

impl CurrentInventory {
    pub(in super::super) fn validate_capacity(&self) -> Result<()> {
        let required = self
            .members
            .iter()
            .filter(|member| self.requires(member.entity))
            .count()
            .checked_add(self.background.len())
            .context("grown proof member count overflow")?;
        ensure!(
            required <= super::MAX_FIXTURE_MEMBERS,
            "complete grown proof census exceeds admitted member capacity: {required}"
        );
        Ok(())
    }
}

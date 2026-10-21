//! The first unavailable native binding, without granting a later-frame retry.

use super::*;
use bevy::render::render_resource::CachedPipelineState;

pub(in super::super) enum Availability<T> {
    Ready(T),
    Missing(MissingBinding),
}

pub(in super::super) enum MissingBinding {
    ViewSpecializations,
    Member {
        member: Box<TaggedMember>,
        resource: MissingMemberResource,
    },
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(in super::super) enum MissingMemberResource {
    MaterialInstance,
    PreparedMaterial,
    Mesh,
    MeshAllocation {
        vertex_slice_present: bool,
        slabs_present: bool,
        indexed: bool,
        index_slice_present: bool,
    },
    EntitySpecialization {
        mesh_queue_data_present: bool,
    },
    CompiledPipeline {
        pipeline: String,
        preparation: PipelinePreparation,
    },
}

#[derive(Serialize)]
#[serde(tag = "phase", rename_all = "snake_case")]
pub(in super::super) enum PipelinePreparation {
    Queued,
    Creating,
    Ready,
    Failed { error: String },
}

impl<T> Availability<T> {
    pub(in super::super) fn as_ready(&self) -> Option<&T> {
        match self {
            Self::Ready(value) => Some(value),
            Self::Missing(_) => None,
        }
    }
}

impl MissingBinding {
    pub(in super::super) fn report(&self) -> serde_json::Value {
        match self {
            Self::ViewSpecializations => serde_json::json!({"kind":"view_specializations"}),
            Self::Member { member, resource } => serde_json::json!({
                "kind":"member", "member":member.report(), "resource":resource,
            }),
        }
    }
}

impl From<&CachedPipelineState> for PipelinePreparation {
    fn from(state: &CachedPipelineState) -> Self {
        match state {
            CachedPipelineState::Queued => Self::Queued,
            CachedPipelineState::Creating(_) => Self::Creating,
            CachedPipelineState::Ok(_) => Self::Ready,
            CachedPipelineState::Err(error) => Self::Failed {
                error: format!("{error:?}"),
            },
        }
    }
}

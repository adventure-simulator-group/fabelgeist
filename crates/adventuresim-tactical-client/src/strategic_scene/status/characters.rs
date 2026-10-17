//! Readiness distinguishes the full roster, requested views and visited rigs.
use super::super::{RetainedScene, SceneModel, protocol::StrategicView};
use crate::animation::{HumanoidRig, pose_buffer::PoseBufferRig};
use bevy::{ecs::system::SystemParam, prelude::*};

#[derive(SystemParam)]
pub(super) struct CharacterReadiness<'w, 's> {
    models: Query<'w, 's, (), (With<SceneModel>, With<HumanoidRig>)>,
    poses: Query<'w, 's, &'static PoseBufferRig, With<SceneModel>>,
}

pub(super) struct CharacterCounts {
    pub(super) roster: usize,
    pub(super) loaded: usize,
    pub(super) posed: usize,
    pub(super) required: usize,
    pub(super) required_posed: usize,
}

impl CharacterReadiness<'_, '_> {
    pub(super) fn counts(
        &self,
        view: Option<&StrategicView>,
        scene: &RetainedScene,
    ) -> CharacterCounts {
        let mut counts = CharacterCounts {
            roster: view.map_or(0, |view| view.people.len()),
            loaded: self.models.iter().count(),
            posed: self
                .poses
                .iter()
                .filter(|pose| pose.has_presented_pose())
                .count(),
            required: 0,
            required_posed: 0,
        };
        if let Some(view) = view {
            for person in view
                .people
                .iter()
                .filter(|person| view.requires_person(person.id))
            {
                counts.required += 1;
                if scene.people.get(&person.id).is_some_and(|person| {
                    self.poses
                        .get(person.entity)
                        .is_ok_and(|pose| pose.has_presented_pose())
                }) {
                    counts.required_posed += 1;
                }
            }
        }
        counts
    }
}

//! Authoritative scene membership for independently replicated physical roots.

use bevy::{ecs::entity::MapEntities, prelude::*};
use serde::{Deserialize, Serialize};

use crate::scene_input::GeneratedObstacle;

/// Assigned by the server's obstacle spawn batch, never inferred by a client.
/// Replication maps the referenced scene into the receiving world's entity ID.
#[derive(
    Component, Serialize, Deserialize, Reflect, Clone, Copy, Debug, PartialEq, Eq, MapEntities,
)]
#[component(immutable)]
#[reflect(Component)]
pub struct SceneObstacleOf(#[entities] pub Entity);

/// Authoritative initial physical population on the exact owning scene entity.
/// An empty local pending queue alone cannot prove delivery is complete.
#[derive(Component, Serialize, Deserialize, Reflect, Clone, Copy, Debug, PartialEq, Eq)]
#[component(immutable)]
#[reflect(Component)]
pub struct SceneObstacleCensus {
    trees: ObstacleCount,
    rocks: ObstacleCount,
}

/// Number of physical roots of one obstacle kind, not render parts or draws.
#[derive(Serialize, Deserialize, Reflect, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObstacleCount(u32);

impl SceneObstacleCensus {
    pub fn from_generated(
        obstacles: &[GeneratedObstacle],
    ) -> Result<Self, std::num::TryFromIntError> {
        let trees = obstacles
            .iter()
            .filter(|obstacle| matches!(obstacle, GeneratedObstacle::Tree { .. }))
            .count();
        Ok(Self {
            trees: ObstacleCount(u32::try_from(trees)?),
            rocks: ObstacleCount(u32::try_from(obstacles.len() - trees)?),
        })
    }

    pub fn trees(self) -> ObstacleCount {
        self.trees
    }
    pub fn rocks(self) -> ObstacleCount {
        self.rocks
    }
}

impl ObstacleCount {
    pub fn roots(self) -> u32 {
        self.0
    }
}

//! Complete terrain preflight before cache or entity mutation.
use super::*;

/// Actual ground positions in area-world metres, bound to one live scene.
pub(crate) struct TreeRootBatch {
    pub(super) destination: Destination,
    pub(super) positions: Vec<Vec3>,
}

pub(super) struct Destination {
    pub(super) scene: Entity,
    pub(super) environment: SceneEnvironment,
    pub(super) terrain: SceneTerrain,
}

impl TreeRootBatch {
    /// Framework boundary: finite XYZ metres with Y sampled from actual terrain.
    pub(crate) fn from_area_world_metres(
        world: &World,
        positions: impl IntoIterator<Item = Vec3>,
    ) -> Result<Self> {
        let destination = Destination::from_world(world)?;
        let batch = Self {
            destination,
            positions: positions.into_iter().collect(),
        };
        batch.verify(world)?;
        Ok(batch)
    }

    pub(super) fn verify(&self, world: &World) -> Result<()> {
        let current = Destination::from_world(world)?;
        ensure!(
            self.destination.scene == current.scene
                && serde_json::to_vec(&self.destination.environment)?
                    == serde_json::to_vec(&current.environment)?,
            "tree root batch scene/environment changed"
        );
        ensure!(!self.positions.is_empty(), "empty tree root batch");
        for root in &self.positions {
            ensure!(
                root.is_finite()
                    && current
                        .terrain
                        .height_at(Vec2::new(root.x, root.z))
                        .is_some_and(|height| height.is_finite() && height == root.y),
                "tree root requires actual destination ground"
            );
        }
        Ok(())
    }
}

impl Destination {
    fn from_world(world: &World) -> Result<Self> {
        let scene = world
            .resource::<ActiveTacticalScene>()
            .entity
            .context("active scene for prepared tree")?;
        Ok(Self {
            scene,
            environment: world
                .get::<SceneEnvironment>(scene)
                .context("active tree environment")?
                .clone(),
            terrain: world
                .get::<SceneTerrain>(scene)
                .context("active tree terrain")?
                .clone(),
        })
    }
}

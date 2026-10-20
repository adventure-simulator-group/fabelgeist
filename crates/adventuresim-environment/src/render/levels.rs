//! Levels of detail: a plant whose
//! radius spans at least [`ForestDetail::full_pixels`] on screen is drawn in
//! full detail, at least [`ForestDetail::middle_pixels`] in middle detail,
//! and smaller in the coarsest. Here a whole chunk takes the level its
//! largest plant would have at the chunk's nearest point to the camera, so
//! no plant is drawn coarser than its own size asks.
//!
//! Changing a chunk's level uploads its instances again, so a chunk keeps
//! its level until its plants are a margin past the threshold it crossed,
//! and a new chunk starts at the level it needs.

use bevy::camera::primitives::Aabb;
use bevy::prelude::*;
use bevy_eidolon::prelude::*;
use fabelgeist_math::units::Metres;

use super::chunks::ForestChunk;
use super::materials::ForestMaterials;
use super::{ForestDetail, ForestViewCamera};

mod projection;
pub use projection::{ForestLodProjection, ProjectedRadiusPixels, source_bounds_radius};

/// How far under its threshold a chunk's plants shrink on screen before
/// the chunk turns coarser, as a share of the threshold.
const HYSTERESIS: f32 = 0.15;

/// How finely a chunk's plants are drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "prepared", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "viewer", derive(clap::ValueEnum))]
pub enum Level {
    Full,
    Middle,
    #[default]
    Coarse,
}

/// Adaptive spatial-batch selection, or one common level for every batch.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "prepared", derive(serde::Serialize, serde::Deserialize))]
pub enum LevelSelection {
    #[default]
    Adaptive,
    Fixed(Level),
}

/// How a level builds its instances: sides around a branch tube and spans
/// along it, and blades a leaf cluster.
pub(crate) struct Shape {
    pub(crate) sides: u32,
    pub(crate) spans: u32,
    pub(crate) blades: u32,
}

impl Level {
    pub(crate) const ALL: [Level; 3] = [Level::Full, Level::Middle, Level::Coarse];

    pub(super) fn index(self) -> usize {
        match self {
            Level::Full => 0,
            Level::Middle => 1,
            Level::Coarse => 2,
        }
    }

    /// The level's shape: in full detail as `detail` asks, coarser
    /// further down.
    pub(crate) fn shape(self, detail: &ForestDetail) -> Shape {
        match self {
            Level::Full => Shape {
                sides: detail.tube_sides.max(3),
                spans: detail.tube_spans.max(1),
                blades: detail.leaf_blades.max(1),
            },
            Level::Middle => Shape {
                sides: 6,
                spans: 1,
                blades: 6,
            },
            Level::Coarse => Shape {
                sides: 4,
                spans: 1,
                blades: 2,
            },
        }
    }

    /// The level of a plant whose radius spans `pixels` on screen, for a
    /// chunk now at `current`.
    pub fn for_projected_radius(
        pixels: ProjectedRadiusPixels,
        current: Level,
        detail: &ForestDetail,
    ) -> Self {
        let pixels = pixels.pixels();
        let keep = 1.0 - HYSTERESIS;
        let full = match current {
            Level::Full => detail.full_pixels * keep,
            Level::Middle | Level::Coarse => detail.full_pixels,
        };
        let middle = match current {
            Level::Full | Level::Middle => detail.middle_pixels * keep,
            Level::Coarse => detail.middle_pixels,
        };
        if pixels >= full {
            Level::Full
        } else if pixels >= middle {
            Level::Middle
        } else {
            Level::Coarse
        }
    }
}

/// Where the camera stands, and how many pixels a metre at a metre's
/// distance spans on its screen.
#[derive(Resource, Clone, Debug, Default)]
pub struct ForestLodView {
    source: Option<ForestViewSource>,
}

#[derive(Clone, Debug)]
struct ForestViewSource {
    camera: ForestLodCamera,
    projection: ForestLodProjection,
}

/// Actual native camera inputs sampled after propagation and camera update.
/// This is selection-source evidence, not a GPU uniform-buffer readback.
#[derive(Clone, Debug)]
pub struct ForestLodCamera {
    pub entity: Entity,
    pub world_from_view: GlobalTransform,
    pub projection: Projection,
    pub physical_viewport: UVec2,
}

impl ForestLodView {
    /// The unique active perspective camera used for this frame's selection.
    /// Missing or ambiguous ownership yields no source and adaptive coarse LOD.
    pub fn camera(&self) -> Option<&ForestLodCamera> {
        self.source.as_ref().map(|source| &source.camera)
    }

    /// The level a chunk around `aabb` whose largest plant has `radius`
    /// needs, now at `current`; the coarsest before a camera is seen.
    pub(super) fn level(
        &self,
        aabb: &Aabb,
        radius: f32,
        current: Level,
        detail: &ForestDetail,
    ) -> Level {
        if let LevelSelection::Fixed(level) = detail.level {
            return level;
        }
        let Some(source) = &self.source else {
            return Level::Coarse;
        };
        // The batcher's native vector kernel supplies world-metre radii.
        let pixels = source.projection.projected_radius(
            source.camera.world_from_view.translation(),
            aabb,
            Metres::from_metres(radius),
        );
        pixels.map_or(Level::Coarse, |pixels| {
            Level::for_projected_radius(pixels, current, detail)
        })
    }
}

/// Follow the unique active forest view, never an unrelated camera.
pub(super) fn follow_camera(
    cameras: Query<(Entity, &Camera, &GlobalTransform, &Projection), With<ForestViewCamera>>,
    mut view: ResMut<ForestLodView>,
) {
    let mut active = cameras.iter().filter(|(_, camera, _, _)| camera.is_active);
    let Some((entity, camera, transform, projection)) = active.next() else {
        *view = ForestLodView::default();
        return;
    };
    if active.next().is_some() {
        *view = ForestLodView::default();
        return;
    }
    let found = (|| {
        let Projection::Perspective(perspective) = projection else {
            return None;
        };
        let physical_viewport = camera.physical_viewport_size()?;
        let screen = ForestLodProjection::from_perspective(perspective, physical_viewport)?;
        Some(ForestLodView {
            source: Some(ForestViewSource {
                camera: ForestLodCamera {
                    entity,
                    world_from_view: *transform,
                    projection: projection.clone(),
                    physical_viewport,
                },
                projection: screen,
            }),
        })
    })();
    *view = found.unwrap_or_default();
}

/// Give each chunk the level its largest plant has on screen, swapping its
/// mesh and material when that changes.
pub(super) fn pick(
    mut commands: Commands,
    detail: Res<ForestDetail>,
    view: Res<ForestLodView>,
    forest: Res<ForestMaterials>,
    mut chunks: Query<(Entity, &mut ForestChunk, &Aabb)>,
    probe: Option<Res<super::benchmark::ForestBenchmark>>,
) {
    for (entity, mut chunk, aabb) in &mut chunks {
        let level = view.level(aabb, chunk.radius(), chunk.level(), &detail);
        if level == chunk.level() {
            continue;
        }
        chunk.set_level(level);
        if let Some(probe) = &probe {
            probe.expect_level(entity, level);
        }
        let mut entity = commands.entity(entity);
        forest.dress(&mut entity, chunk.foliage(), level);
    }
}

impl ForestMaterials {
    /// Give a chunk entity the mesh and material of `level`.
    pub(super) fn dress(&self, entity: &mut EntityCommands, foliage: bool, level: Level) {
        let assets = self.level(level);
        if foliage {
            entity.insert((
                Mesh3d(assets.leaf_mesh.clone()),
                InstancedMeshMaterial(assets.leaves.clone()),
            ));
        } else {
            entity.insert((
                Mesh3d(assets.branch_mesh.clone()),
                InstancedMeshMaterial(assets.branches.clone()),
            ));
        }
    }
}

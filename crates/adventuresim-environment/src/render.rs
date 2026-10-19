//! Botany's ecosystems drawn in Bevy.
//!
//! The simulation hands over snapshots
//! ([`Snapshot`](fabelgeist_botany::viewer::sim::Snapshot)): every branch segment
//! and leaf cluster as an instance record, the terrain and the buildings as
//! meshes. [`ForestPlugin`] draws them: the instances through `bevy_eidolon`
//! (GPU instancing and GPU culling, as the game's grass and shrubs), each
//! branch built as a tube and each cluster as blades in its vertex shader,
//! and lit by Bevy's PBR lights; the
//! terrain and buildings as ordinary meshes with their vertex colours.
//!
//! An application feeds it with [`ForestFeed::show`] and tunes it with
//! [`ForestDetail`].

pub mod address;
pub mod benchmark;
mod chunks;
mod feed;
mod ground;
pub(crate) mod levels;
mod materials;
mod records;
pub(crate) mod shapes;
#[cfg(feature = "prepared")]
pub mod trees;
#[cfg(feature = "prepared")]
pub mod view;

use bevy::asset::embedded_asset;
use bevy::camera::{CameraUpdateSystems, visibility::VisibilitySystems};
use bevy::prelude::*;
use bevy::shader::load_shader_library;
use bevy::transform::TransformSystems;
use bevy_eidolon::{prelude::*, prepass::CullComputeCamera};

pub use feed::ForestFeed;
pub use ground::GroundPart;
pub use levels::{ForestLodCamera, ForestLodView, Level, LevelSelection};
pub use materials::{BranchMaterial, LeafMaterial};

type UnmarkedForestCamera = (
    With<ForestViewCamera>,
    With<Camera3d>,
    Without<CullComputeCamera>,
);

/// The active perspective camera that owns forest LOD and readiness evidence.
/// Mark exactly one active camera, including in applications with other views.
/// Eidolon's global cull buffer requires this to be the only active culling
/// camera. Readiness rejects a different or ambiguous render-world source.
#[derive(Component)]
pub struct ForestViewCamera;

/// Camera sampling and selection use the same propagated frame. Consumers
/// applying a camera pose must do so before transform propagation and camera
/// update; observers read selection only after `Select` and visibility checks.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ForestLodSystems {
    FollowCamera,
    Select,
}

/// Geometry owned by the forest renderer, including its supplied ground.
/// Consumers may assign presentation layers without touching unrelated meshes.
#[derive(Component)]
pub struct ForestGeometry;

/// How finely the forest is built, and whether its foliage is drawn.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "prepared", derive(serde::Serialize, serde::Deserialize))]
pub struct ForestDetail {
    /// Around each branch tube, and along each segment, in full detail.
    pub tube_sides: u32,
    pub tube_spans: u32,
    /// Blades a leaf cluster, in full detail.
    pub leaf_blades: u32,
    /// A plant whose radius spans at least this many pixels on screen is
    /// drawn in full detail, at least `middle_pixels` in middle detail,
    /// and smaller in the coarsest.
    pub full_pixels: f32,
    pub middle_pixels: f32,
    pub leaves: bool,
    /// Override adaptive selection for controlled representation comparisons.
    pub level: LevelSelection,
}

/// The two independently rendered parts of forest geometry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "prepared", derive(serde::Serialize, serde::Deserialize))]
pub enum ForestPart {
    Branches,
    Leaves,
}

impl Default for ForestDetail {
    /// The detail plants are drawn at unless asked otherwise.
    fn default() -> Self {
        Self {
            tube_sides: 10,
            tube_spans: 6,
            leaf_blades: 10,
            full_pixels: 220.0,
            middle_pixels: 40.0,
            leaves: true,
            level: LevelSelection::Adaptive,
        }
    }
}

/// Draws the snapshots fed to [`ForestFeed`].
pub struct ForestPlugin;

impl Plugin for ForestPlugin {
    fn build(&self, app: &mut App) {
        load_shader_library!(app, "render/shaders/forest_common.wgsl");
        load_shader_library!(app, "render/shaders/branch_geometry.wgsl");
        load_shader_library!(app, "render/shaders/branch_surface.wgsl");
        load_shader_library!(app, "render/shaders/leaf_geometry.wgsl");
        load_shader_library!(app, "render/shaders/leaf_surface.wgsl");
        // Bevy discovers conditional imports before pipeline specialization.
        load_shader_library!(app, "render/shaders/tree_pose.wgsl");
        embedded_asset!(app, "render/shaders/branch.wgsl");
        embedded_asset!(app, "render/shaders/leaf.wgsl");
        embedded_asset!(app, "render/shaders/forest_cull.wgsl");
        if !app.is_plugin_added::<InstancedMaterialCorePlugin>() {
            app.add_plugins(InstancedMaterialCorePlugin);
        }
        if !app.is_plugin_added::<GpuComputeCullCorePlugin>() {
            app.add_plugins(GpuComputeCullCorePlugin);
        }
        records::install(app);
        app.add_plugins((
            InstancedMaterialPlugin::<BranchMaterial>::default(),
            GpuCullComputePlugin::<BranchMaterial>::default(),
            InstancedMaterialPlugin::<LeafMaterial>::default(),
            GpuCullComputePlugin::<LeafMaterial>::default(),
        ))
        .init_resource::<ForestFeed>()
        .init_resource::<ForestLodView>()
        .init_resource::<ForestDetail>()
        .configure_sets(
            PostUpdate,
            (
                ForestLodSystems::FollowCamera
                    .after(TransformSystems::Propagate)
                    .after(CameraUpdateSystems),
                ForestLodSystems::Select
                    .after(ForestLodSystems::FollowCamera)
                    .before(VisibilitySystems::VisibilityPropagate)
                    .before(VisibilitySystems::CheckVisibility),
            ),
        )
        .add_systems(Startup, materials::setup)
        .add_systems(
            Update,
            (
                cull_from_cameras,
                materials::follow_detail,
                feed::draw,
                feed::prepare,
                show_leaves,
            )
                .chain(),
        )
        .add_systems(
            PostUpdate,
            (
                levels::follow_camera.in_set(ForestLodSystems::FollowCamera),
                levels::pick.in_set(ForestLodSystems::Select),
            ),
        );
    }
}

/// The forest's explicit view participates in GPU instance culling.
fn cull_from_cameras(mut commands: Commands, cameras: Query<Entity, UnmarkedForestCamera>) {
    for camera in &cameras {
        commands.entity(camera).insert(CullComputeCamera);
    }
}

fn show_leaves(
    detail: Res<ForestDetail>,
    mut chunks: Query<(&chunks::ForestChunk, &mut Visibility)>,
) {
    for (chunk, mut visibility) in &mut chunks {
        let wanted = if chunk.foliage() && !detail.leaves {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        visibility.set_if_neq(wanted);
    }
}

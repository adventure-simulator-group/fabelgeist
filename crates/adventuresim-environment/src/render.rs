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

pub mod benchmark;
mod chunks;
mod feed;
mod ground;
mod levels;
mod materials;
mod records;
mod shapes;

use bevy::asset::embedded_asset;
use bevy::prelude::*;
use bevy::shader::load_shader_library;
use bevy_eidolon::{prelude::*, prepass::CullComputeCamera};

pub use feed::ForestFeed;
pub use ground::GroundPart;
pub use materials::{BranchMaterial, LeafMaterial};

/// How finely the forest is built, and whether its foliage is drawn.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
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
        }
    }
}

/// Draws the snapshots fed to [`ForestFeed`].
pub struct ForestPlugin;

impl Plugin for ForestPlugin {
    fn build(&self, app: &mut App) {
        load_shader_library!(app, "render/shaders/forest_common.wgsl");
        embedded_asset!(app, "render/shaders/branch.wgsl");
        embedded_asset!(app, "render/shaders/leaf.wgsl");
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
        .init_resource::<levels::LevelView>()
        .init_resource::<ForestDetail>()
        .add_systems(Startup, materials::setup)
        .add_systems(
            Update,
            (
                cull_from_cameras,
                levels::follow_camera,
                materials::follow_detail,
                feed::draw,
                feed::prepare,
                levels::pick,
                show_leaves,
            )
                .chain(),
        );
    }
}

/// Every 3D camera culls the forest's instances on the GPU.
fn cull_from_cameras(
    mut commands: Commands,
    cameras: Query<Entity, (With<Camera3d>, Without<CullComputeCamera>)>,
) {
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

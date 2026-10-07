//! Botany's ecosystems drawn in Bevy.
//!
//! The simulation hands over snapshots ([`Snapshot`]): every branch segment
//! and leaf cluster as an instance record, the terrain and the buildings as
//! meshes. [`ForestPlugin`] draws them: the instances through `bevy_eidolon`
//! (GPU instancing and GPU culling, as the game's grass and shrubs), each
//! branch built as a tube and each cluster as blades in its vertex shader,
//! and lit by Bevy's PBR lights; the
//! terrain and buildings as ordinary meshes with their vertex colours.
//!
//! An application feeds it with [`ForestFeed::show`] and tunes it with
//! [`ForestDetail`].

mod chunks;
mod ground;
mod materials;
mod shapes;

use bevy::asset::embedded_asset;
use bevy::prelude::*;
use bevy::shader::load_shader_library;
use bevy_eidolon::{prelude::*, prepass::CullComputeCamera};
use fabelgeist_botany::viewer::sim::Snapshot;

pub use materials::{BranchMaterial, LeafMaterial};

/// The snapshot to draw next, if a new one arrived.
#[derive(Resource, Default)]
pub struct ForestFeed {
    pending: Option<Snapshot>,
}

impl ForestFeed {
    /// Draw `snapshot` from the next frame on.
    pub fn show(&mut self, snapshot: Snapshot) {
        self.pending = Some(snapshot);
    }
}

/// How finely the forest is built, and whether its foliage is drawn.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ForestDetail {
    /// Around each branch tube, and along each segment.
    pub tube_sides: u32,
    pub tube_spans: u32,
    /// Blades a leaf cluster.
    pub leaf_blades: u32,
    pub leaves: bool,
}

impl Default for ForestDetail {
    /// The detail close plants are drawn at unless asked otherwise.
    fn default() -> Self {
        Self {
            tube_sides: 10,
            tube_spans: 6,
            leaf_blades: 10,
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
        app.add_plugins((
            InstancedMaterialPlugin::<BranchMaterial>::default(),
            GpuCullComputePlugin::<BranchMaterial>::default(),
            InstancedMaterialPlugin::<LeafMaterial>::default(),
            GpuCullComputePlugin::<LeafMaterial>::default(),
        ))
        .init_resource::<ForestFeed>()
        .init_resource::<ForestDetail>()
        .add_systems(Startup, materials::setup)
        .add_systems(
            Update,
            (
                cull_from_cameras,
                materials::follow_detail,
                draw_snapshot,
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

/// Take a fed snapshot: its instances into the materials' storage, its
/// chunks as entities, and its terrain and buildings when it brings them.
fn draw_snapshot(
    mut commands: Commands,
    mut feed: ResMut<ForestFeed>,
    forest: Res<materials::ForestMaterials>,
    mut stores: materials::ForestStores,
    mut ground: ground::GroundAssets,
    old: Query<Entity, With<chunks::ForestChunk>>,
) {
    let Some(snapshot) = feed.pending.take() else {
        return;
    };
    forest.store(&snapshot, &mut stores);
    for entity in &old {
        commands.entity(entity).despawn();
    }
    chunks::spawn(&mut commands, &forest, &snapshot);
    ground.show(&mut commands, &snapshot);
}

fn show_leaves(
    detail: Res<ForestDetail>,
    mut chunks: Query<(&chunks::ForestChunk, &mut Visibility)>,
) {
    for (chunk, mut visibility) in &mut chunks {
        let wanted = if chunk.foliage && !detail.leaves {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        visibility.set_if_neq(wanted);
    }
}

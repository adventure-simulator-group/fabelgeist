//! A snapshot's instances, gathered into square chunks of ground. Each chunk
//! is one instanced draw with its own bounds: the GPU culls whole chunks
//! outside the view before it culls their instances.

use std::collections::HashMap;
use std::sync::Arc;

use bevy::camera::primitives::Aabb;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::prelude::*;
use bevy_eidolon::prelude::*;
use fabelgeist_botany::viewer::sim::{BranchInstance, LeafInstance, Snapshot};

use super::materials::ForestMaterials;

/// A chunk's side, metres.
const CHUNK: f32 = 16.0;
/// Farther than this an instance is not drawn, metres: beyond any scene.
const FAR: f32 = 100_000.0;
/// How far a cluster's blades reach from its centre, in cluster sizes: a
/// palm frond is four long.
const CLUSTER_REACH: f32 = 4.0;

/// One chunk's draw, of branches or of foliage.
#[derive(Component)]
pub(super) struct ForestChunk {
    pub(super) foliage: bool,
}

/// An instance's bounding sphere and its index in the snapshot.
struct Placed {
    center: Vec3,
    radius: f32,
    index: u32,
}

impl Placed {
    fn branch(index: usize, branch: &BranchInstance) -> Self {
        let a = Vec3::from_slice(&branch.a[..3]);
        let b = Vec3::from_slice(&branch.b[..3]);
        Self {
            center: (a + b) * 0.5,
            radius: a.distance(b) * 0.5 + branch.a[3].max(branch.b[3]),
            index: index as u32,
        }
    }

    fn leaf(index: usize, leaf: &LeafInstance) -> Self {
        Self {
            center: Vec3::from_array(leaf.center),
            radius: leaf.size * CLUSTER_REACH,
            index: index as u32,
        }
    }
}

/// Spawn a snapshot's branch and foliage chunks.
pub(super) fn spawn(commands: &mut Commands, forest: &ForestMaterials, snapshot: &Snapshot) {
    let branches = snapshot
        .branches
        .iter()
        .enumerate()
        .map(|(index, branch)| Placed::branch(index, branch));
    for (instances, aabb) in gather(branches) {
        commands.spawn((
            chunk(instances, aabb, false),
            Mesh3d(forest.branch_mesh.clone()),
            InstancedMeshMaterial(forest.branches.clone()),
        ));
    }
    let leaves = snapshot
        .leaves
        .iter()
        .enumerate()
        .map(|(index, leaf)| Placed::leaf(index, leaf));
    for (instances, aabb) in gather(leaves) {
        commands.spawn((
            chunk(instances, aabb, true),
            Mesh3d(forest.leaf_mesh.clone()),
            InstancedMeshMaterial(forest.leaves.clone()),
        ));
    }
}

fn chunk(instances: Vec<InstanceData>, aabb: Aabb, foliage: bool) -> impl Bundle {
    (
        ForestChunk { foliage },
        InstanceMaterialData {
            instances: Arc::new(instances),
            color: LinearRgba::WHITE,
            visibility_range: Vec4::new(0.0, 0.0, FAR, FAR),
        },
        aabb,
        // Culling belongs to the GPU: CPU frustum culling would free the
        // retained instance buffers.
        NoFrustumCulling,
        Transform::default(),
        Visibility::Inherited,
    )
}

/// The instances by chunk, each chunk with the box around them.
fn gather(placed: impl Iterator<Item = Placed>) -> Vec<(Vec<InstanceData>, Aabb)> {
    let mut chunks: HashMap<IVec2, (Vec<InstanceData>, Vec3, Vec3)> = HashMap::new();
    for item in placed {
        let cell = (item.center.xz() / CHUNK).floor().as_ivec2();
        let reach = Vec3::splat(item.radius);
        let (instances, low, high) = chunks
            .entry(cell)
            .or_insert_with(|| (Vec::new(), Vec3::INFINITY, Vec3::NEG_INFINITY));
        *low = low.min(item.center - reach);
        *high = high.max(item.center + reach);
        instances.push(InstanceData {
            position: item.center,
            scale: item.radius,
            index: item.index,
            ..default()
        });
    }
    chunks
        .into_values()
        .map(|(instances, low, high)| (instances, Aabb::from_min_max(low, high)))
        .collect()
}

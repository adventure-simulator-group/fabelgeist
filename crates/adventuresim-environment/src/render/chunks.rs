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

/// One chunk's draw, of branches or of foliage. A chunk lives as long as
/// its cell holds instances: a new snapshot replaces its instances in place,
/// since the instancer skips a draw on the frame its entity appears.
#[derive(Component)]
pub(super) struct ForestChunk {
    key: ChunkKey,
}

impl ForestChunk {
    pub(super) fn foliage(&self) -> bool {
        self.key.foliage
    }
}

/// A chunk's cell of ground, and whether it holds foliage or branches.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct ChunkKey {
    cell: IVec2,
    foliage: bool,
}

/// A chunk's instances and the box around them.
struct Gathered {
    instances: Vec<InstanceData>,
    aabb: Aabb,
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

/// The chunks drawn now.
pub(super) type Shown<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static ForestChunk,
        &'static mut InstanceMaterialData,
        &'static mut Aabb,
    ),
>;

/// Show a snapshot's branch and foliage chunks: refill the chunks still
/// holding instances, drop the emptied ones and spawn the new ones.
pub(super) fn show(
    commands: &mut Commands,
    forest: &ForestMaterials,
    snapshot: &Snapshot,
    shown: &mut Shown,
) {
    let branches = snapshot
        .branches
        .iter()
        .enumerate()
        .map(|(index, branch)| Placed::branch(index, branch));
    let leaves = snapshot
        .leaves
        .iter()
        .enumerate()
        .map(|(index, leaf)| Placed::leaf(index, leaf));
    let mut chunks = gather(branches, false);
    chunks.extend(gather(leaves, true));
    for (entity, chunk, mut data, mut aabb) in shown.iter_mut() {
        let Some(gathered) = chunks.remove(&chunk.key) else {
            commands.entity(entity).despawn();
            continue;
        };
        data.instances = Arc::new(gathered.instances);
        *aabb = gathered.aabb;
    }
    for (key, gathered) in chunks {
        let mut chunk = commands.spawn((
            ForestChunk { key },
            InstanceMaterialData {
                instances: Arc::new(gathered.instances),
                color: LinearRgba::WHITE,
                visibility_range: Vec4::new(0.0, 0.0, FAR, FAR),
            },
            gathered.aabb,
            // Culling belongs to the GPU: CPU frustum culling would free the
            // retained instance buffers. Without the opt-in, eidolon still
            // runs its cull pass over the page and appends every instance a
            // second time past the chunk's range, over its neighbours', in a
            // different order each frame: the forest flickers.
            NoFrustumCulling,
            GpuCullCompute,
            Transform::default(),
            Visibility::Inherited,
        ));
        if key.foliage {
            chunk.insert((
                Mesh3d(forest.leaf_mesh.clone()),
                InstancedMeshMaterial(forest.leaves.clone()),
            ));
        } else {
            chunk.insert((
                Mesh3d(forest.branch_mesh.clone()),
                InstancedMeshMaterial(forest.branches.clone()),
            ));
        }
    }
}

/// The instances by chunk, each chunk with the box around them.
fn gather(placed: impl Iterator<Item = Placed>, foliage: bool) -> HashMap<ChunkKey, Gathered> {
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
        .into_iter()
        .map(|(cell, (instances, low, high))| {
            let aabb = Aabb::from_min_max(low, high);
            (ChunkKey { cell, foliage }, Gathered { instances, aabb })
        })
        .collect()
}

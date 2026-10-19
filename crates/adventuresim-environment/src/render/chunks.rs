//! A snapshot's instances, gathered into square chunks of ground. Each chunk
//! is one instanced draw with its own bounds: the GPU culls whole chunks
//! outside the view, then tests each instance's distance. Frustum culling is
//! per chunk, not per branch or leaf.

use std::collections::HashMap;
use std::sync::Arc;

use bevy::camera::primitives::Aabb;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::prelude::*;
use bevy_eidolon::prelude::*;
use fabelgeist_botany::viewer::sim::{BranchInstance, LeafInstance, PlantInstance, Snapshot};

use super::ForestDetail;
use super::ForestPart;
use super::benchmark::{ForestBenchmark, ForestWork};
use super::levels::{Level, LevelView};
use super::materials::ForestMaterials;
use super::records::fingerprint;

/// A chunk's side, metres.
pub(super) const CHUNK: f32 = 16.0;
/// Farther than this an instance is not drawn, metres: beyond any scene.
pub(super) const FAR: f32 = 100_000.0;
/// How far a cluster's blades reach from its centre, in cluster sizes: a
/// palm frond is four long.
const CLUSTER_REACH: f32 = 4.0;
/// The plant radii, metres, that part a cell's plants into chunks by size:
/// a chunk's level of detail follows its largest plant, so shrubs are kept
/// apart from the trees over them.
const SIZE_CLASSES: [f32; 3] = [1.5, 4.0, 10.0];

/// One chunk's draw, of branches or of foliage. A chunk lives as long as
/// its cell holds instances of its plants' size: a new snapshot replaces
/// its instances in place, since the instancer skips a draw on the frame
/// its entity appears.
#[derive(Component)]
pub(super) struct ForestChunk {
    key: ChunkKey,
    /// Its largest plant's radius, metres.
    radius: f32,
    level: Level,
    /// Its instances' fingerprint: unchanged instances are not uploaded
    /// again.
    fingerprint: u64,
}

impl ForestChunk {
    pub(super) fn foliage(&self) -> bool {
        self.key.foliage
    }

    pub(super) fn radius(&self) -> f32 {
        self.radius
    }

    pub(super) fn level(&self) -> Level {
        self.level
    }

    pub(super) fn set_level(&mut self, level: Level) {
        self.level = level;
    }
}

/// A chunk's cell of ground, its plants' size class, and whether it holds
/// foliage or branches.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct ChunkKey {
    cell: IVec2,
    size: usize,
    foliage: bool,
}

/// A chunk's instances, the box around them, its largest plant's radius
/// and its instances' fingerprint.
struct Gathered {
    instances: Vec<InstanceData>,
    aabb: Aabb,
    radius: f32,
    fingerprint: u64,
}

/// An instance's bounding sphere, its index in the snapshot, and its
/// plant's radius.
struct Placed {
    center: Vec3,
    radius: f32,
    index: u32,
    plant: f32,
}

impl Placed {
    fn branch(index: usize, branch: &BranchInstance, plants: &[PlantInstance]) -> Self {
        let a = Vec3::from_slice(&branch.a[..3]);
        let b = Vec3::from_slice(&branch.b[..3]);
        Self {
            center: (a + b) * 0.5,
            radius: a.distance(b) * 0.5 + branch.a[3].max(branch.b[3]),
            index: index as u32,
            plant: plant_radius(plants, branch.plant),
        }
    }

    fn leaf(index: usize, leaf: &LeafInstance, plants: &[PlantInstance]) -> Self {
        Self {
            center: Vec3::from_array(leaf.center),
            radius: leaf.size * CLUSTER_REACH,
            index: index as u32,
            plant: plant_radius(plants, leaf.plant),
        }
    }
}

fn plant_radius(plants: &[PlantInstance], plant: u32) -> f32 {
    plants
        .get(plant as usize)
        .map_or(0.0, |plant| plant.sphere[3])
}

/// The size class of a plant of `radius`.
pub(super) fn size_class(radius: f32) -> usize {
    SIZE_CLASSES
        .iter()
        .take_while(|bound| radius >= **bound)
        .count()
}

/// The chunks drawn now.
pub(super) type Shown<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut ForestChunk,
        &'static mut InstanceMaterialData,
        &'static mut Aabb,
    ),
>;

/// A snapshot's instances, by chunk.
pub(super) struct Chunks(HashMap<ChunkKey, Gathered>);

impl Chunks {
    /// Gather `snapshot`'s branches and leaf clusters into chunks. It visits
    /// every instance, so it runs off the main thread.
    pub(super) fn new(snapshot: &Snapshot) -> Self {
        let plants = &snapshot.plant_bounds;
        let branches = snapshot
            .branches
            .iter()
            .enumerate()
            .map(|(index, branch)| Placed::branch(index, branch, plants));
        let leaves = snapshot
            .leaves
            .iter()
            .enumerate()
            .map(|(index, leaf)| Placed::leaf(index, leaf, plants));
        let mut chunks = gather(branches, false);
        chunks.extend(gather(leaves, true));
        Self(chunks)
    }
}

/// Where new chunks go, and at what level of detail.
pub(super) struct Placement<'a> {
    pub(super) forest: &'a ForestMaterials,
    pub(super) view: &'a LevelView,
    pub(super) detail: &'a ForestDetail,
}

/// Show `chunks`: refill the chunks still holding instances, drop the
/// emptied ones and spawn the new ones at the level they need.
pub(super) fn show(
    commands: &mut Commands,
    placement: &Placement,
    Chunks(mut chunks): Chunks,
    shown: &mut Shown,
    probe: Option<&ForestBenchmark>,
) {
    for (entity, mut chunk, mut data, mut aabb) in shown.iter_mut() {
        let Some(gathered) = chunks.remove(&chunk.key) else {
            commands.entity(entity).despawn();
            continue;
        };
        if gathered.fingerprint == chunk.fingerprint {
            continue;
        }
        data.instances = Arc::new(gathered.instances);
        *aabb = gathered.aabb;
        chunk.radius = gathered.radius;
        chunk.fingerprint = gathered.fingerprint;
    }
    for (key, gathered) in chunks {
        let instance_count = gathered.instances.len();
        let level = placement.view.level(
            &gathered.aabb,
            gathered.radius,
            Level::Coarse,
            placement.detail,
        );
        let mut chunk = commands.spawn((
            super::ForestGeometry,
            ForestChunk {
                key,
                radius: gathered.radius,
                level,
                fingerprint: gathered.fingerprint,
            },
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
        placement.forest.dress(&mut chunk, key.foliage, level);
        if let Some(probe) = probe {
            let part = if key.foliage {
                ForestPart::Leaves
            } else {
                ForestPart::Branches
            };
            probe.expect_chunk(
                chunk.id(),
                instance_count,
                part,
                level,
                ForestWork {
                    records: instance_count,
                    indexed_slots: instance_count,
                },
            );
        }
    }
}

/// A chunk being gathered: its instances, its box's corners and its
/// largest plant's radius.
struct Gathering {
    instances: Vec<InstanceData>,
    low: Vec3,
    high: Vec3,
    radius: f32,
}

/// The instances by chunk, each chunk with the box around them.
fn gather(placed: impl Iterator<Item = Placed>, foliage: bool) -> HashMap<ChunkKey, Gathered> {
    let mut chunks: HashMap<ChunkKey, Gathering> = HashMap::new();
    for item in placed {
        let key = ChunkKey {
            cell: (item.center.xz() / CHUNK).floor().as_ivec2(),
            size: size_class(item.plant),
            foliage,
        };
        let reach = Vec3::splat(item.radius);
        let chunk = chunks.entry(key).or_insert_with(|| Gathering {
            instances: Vec::new(),
            low: Vec3::INFINITY,
            high: Vec3::NEG_INFINITY,
            radius: 0.0,
        });
        chunk.low = chunk.low.min(item.center - reach);
        chunk.high = chunk.high.max(item.center + reach);
        chunk.radius = chunk.radius.max(item.plant);
        chunk.instances.push(InstanceData {
            position: item.center,
            scale: item.radius,
            index: item.index,
            ..default()
        });
    }
    chunks
        .into_iter()
        .map(|(key, chunk)| {
            let gathered = Gathered {
                fingerprint: fingerprint(bytemuck::cast_slice(&chunk.instances)),
                instances: chunk.instances,
                aabb: Aabb::from_min_max(chunk.low, chunk.high),
                radius: chunk.radius,
            };
            (key, gathered)
        })
        .collect()
}

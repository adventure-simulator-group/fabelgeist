//! Trees as octahedral impostors, baked in the browser from the snapshot on
//! screen.
//!
//! Each tree the size of a tree is drawn from [`FRAMES`]² directions over the
//! upper hemisphere into one layer of an atlas ([`bake`]); a tree then
//! needs one quad, reading the three views nearest the eye's direction.
//! Scene trees switch to theirs while they span fewer pixels than
//! [`ImpostorSettings::switch`], their branches and leaves stepping aside in
//! the same frame, the same test on both sides; and woods of their copies,
//! turned and scaled, fill the ground around the scene out to the horizon.
//!
//! A bake belongs to the snapshot it was taken from: plants are numbered
//! anew in every snapshot, so the scene's trees switch only while that
//! snapshot is on screen, and a bake is taken again once the forest has
//! grown on.

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use bevy::asset::RenderAssetUsages;
use bevy::camera::primitives::Aabb;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::image::{ImageSampler, ImageSamplerDescriptor};
use bevy::mesh::{Indices, MeshVertexBufferLayoutRef, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, Extent3d, RenderPipelineDescriptor, SpecializedMeshPipelineError,
    TextureDimension, TextureUsages, TextureViewDescriptor, TextureViewDimension,
};
use bevy::render::{Render, RenderApp, RenderStartup, RenderSystems};
use bevy::shader::ShaderRef;
use bevy_eidolon::prelude::*;
use fabelgeist_botany::viewer::sim::{BranchInstance, LeafInstance, Snapshot};

use super::feed::ForestFeed;
use super::levels::Level;
use super::materials::{CULL_SHADER, ForestMaterials, ForestStores, SpeciesRecord};
use super::records::Bytes;
use super::{ForestDetail, ForestGeometry};

mod bake;

use bake::{BakeJob, BakeModel, BakeQueue};

const IMPOSTOR_SHADER: &str = "embedded://adventuresim_environment/render/shaders/impostor.wgsl";

/// Views along each side of a tree's grid: 64 views in all.
pub const FRAMES: u32 = 8;
/// A view's side, texels.
pub const FRAME_TEXELS: u32 = 64;
/// Mip levels, down to a texel a view.
const MIPS: u32 = FRAME_TEXELS.ilog2() + 1;
/// Plants at least this wide, metres of radius, are trees with impostors:
/// the size class the forest keeps apart from shrubs.
const TREE_RADIUS: f32 = 4.0;
/// The most trees a bake takes, largest first.
const MOST_TREES: usize = 64;
/// How far a cluster's blades reach past its centre, in cluster sizes, to
/// bound a tree.
const CLUSTER_REACH: f32 = 2.0;
/// Room round a tree in its views, as a share of its radius.
const MARGIN: f32 = 1.04;
/// The smallest blade baked, texels.
const SMALLEST_BLADE: f32 = 1.0;
/// Years the forest grows on before its trees are baked again.
const REBAKE_YEARS: f32 = 10.0;
/// The woods' cells, metres: each one draw, culled whole.
const WOODS_CELL: f32 = 128.0;
/// The woods start this far past the scene's edge, metres.
const WOODS_GAP: f32 = 4.0;
/// The woods' trees are this much smaller or larger than the one they copy.
const WOODS_SCALE: [f32; 2] = [0.8, 1.2];
/// Forest floor under the woods, sRGB.
const WOODS_FLOOR: Color = Color::srgb(0.27, 0.29, 0.18);

/// Whether trees are drawn as impostors, and how.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct ImpostorSettings {
    pub enabled: bool,
    /// A scene tree spanning fewer pixels of radius than this is drawn as its
    /// impostor; zero keeps every scene tree whole.
    pub switch: f32,
    /// Trees in the woods round the scene.
    pub woods: u32,
    /// How far the woods reach from the scene's middle, metres.
    pub woods_reach: f32,
    /// Raise to bake again now.
    pub rebake: u32,
}

impl Default for ImpostorSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            switch: 40.0,
            woods: 30_000,
            woods_reach: 1_000.0,
            rebake: 0,
        }
    }
}

impl ImpostorSettings {
    /// The switch the forest's shaders follow: none while off.
    pub(super) fn switch_pixels(&self) -> f32 {
        if self.enabled { self.switch } else { 0.0 }
    }
}

/// What has been baked, for display.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct ImpostorStatus {
    /// Trees in the bake drawn now, and the year it was taken.
    pub trees: usize,
    pub year: Option<f32>,
    /// The atlas's size on the GPU, bytes.
    pub bytes: u64,
    /// A bake under way: trees baked of how many.
    pub baking: Option<(usize, usize)>,
    /// Whether the scene's trees switch now: the snapshot on screen is the
    /// one baked.
    pub scene: bool,
}

/// Trees drawn as impostors.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ImpostorMaterial {
    /// x: views along each side of the grid; y: 1 while scene trees switch;
    /// z: the switch, pixels of radius, as f32 bits.
    #[uniform(0)]
    settings: UVec4,
    #[texture(1, dimension = "2d_array")]
    #[sampler(2)]
    albedo: Handle<Image>,
    #[texture(3, dimension = "2d_array")]
    normal: Handle<Image>,
}

impl InstancedMaterial for ImpostorMaterial {
    fn cull_shader() -> ShaderRef {
        CULL_SHADER.into()
    }
    fn vertex_shader() -> ShaderRef {
        IMPOSTOR_SHADER.into()
    }
    fn fragment_shader() -> ShaderRef {
        IMPOSTOR_SHADER.into()
    }
    fn disable_prepass(&self) -> bool {
        true
    }
    fn double_sided(&self) -> bool {
        true
    }
    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: Self::Data,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        descriptor.multisample.alpha_to_coverage_enabled = descriptor.multisample.count > 1;
        Ok(())
    }
}

pub(super) struct ImpostorPlugin;

impl Plugin for ImpostorPlugin {
    fn build(&self, app: &mut App) {
        let queue = BakeQueue::default();
        app.add_plugins((
            InstancedMaterialPlugin::<ImpostorMaterial>::default(),
            GpuCullComputePlugin::<ImpostorMaterial>::default(),
        ))
        .init_resource::<ImpostorSettings>()
        .init_resource::<ImpostorStatus>()
        .init_resource::<Impostors>()
        .insert_resource(queue.clone())
        .add_systems(Update, (start_bake, finish_bake, mark_plants, show).chain());
        let Some(render) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render
            .insert_resource(queue)
            .add_systems(RenderStartup, bake::init)
            .add_systems(
                Render,
                bake::bake
                    .in_set(RenderSystems::PrepareResources)
                    .run_if(resource_exists::<bake::Baker>),
            );
    }
}

/// One baked tree: the plant it was, the bake's centre and radius, and the
/// ground it stands on.
#[derive(Clone, Copy, Debug)]
struct Tree {
    plant: usize,
    center: Vec3,
    radius: f32,
    ground: f32,
}

/// A bake: its atlas, its trees, and the snapshot they were baked from.
struct Bake {
    snapshot: Arc<Snapshot>,
    trees: Vec<Tree>,
    material: Handle<ImpostorMaterial>,
    bytes: u64,
    progress: Arc<AtomicU32>,
}

#[derive(Resource, Default)]
struct Impostors {
    /// The bake drawn, and one under way.
    current: Option<Bake>,
    pending: Option<Bake>,
    /// The rebake request last served.
    rebake: u32,
    mesh: Option<Handle<Mesh>>,
    /// The entities drawing the bake, and the woods' settings they follow.
    entities: Vec<Entity>,
    woods: Option<(u32, f32)>,
    /// Which snapshot the plants' impostors were last written for, whether
    /// they were marked, and how many bytes were written.
    marked: Option<(usize, bool)>,
    written: usize,
}

/// Bake the snapshot on screen when impostors are wanted and the bake drawn
/// is missing, asked for again, or of a forest since grown on.
fn start_bake(
    settings: Res<ImpostorSettings>,
    feed: Res<ForestFeed>,
    detail: Res<ForestDetail>,
    queue: Res<BakeQueue>,
    mut impostors: ResMut<Impostors>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<ImpostorMaterial>>,
) {
    if !settings.enabled || impostors.pending.is_some() {
        return;
    }
    let Some(snapshot) = feed.drawn() else {
        return;
    };
    let asked = settings.rebake != impostors.rebake;
    let due = match &impostors.current {
        None => true,
        Some(current) => {
            let baked = &current.snapshot;
            !Arc::ptr_eq(baked, snapshot)
                && (baked.scene_generation != snapshot.scene_generation
                    || (snapshot.time - baked.time).abs() >= REBAKE_YEARS)
        }
    };
    if !asked && !due {
        return;
    }
    impostors.rebake = settings.rebake;
    let shape = Level::Middle.shape(&detail);
    let (trees, job) = prepare(snapshot, [shape.sides, shape.spans, shape.blades]);
    let layers = trees.len().max(1) as u32;
    let mut atlas = |format| {
        let side = FRAMES * FRAME_TEXELS;
        let mut image = Image::new_uninit(
            Extent3d {
                width: side,
                height: side,
                depth_or_array_layers: layers,
            },
            TextureDimension::D2,
            format,
            RenderAssetUsages::RENDER_WORLD,
        );
        image.texture_descriptor.mip_level_count = MIPS;
        image.texture_descriptor.usage =
            TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT;
        image.texture_view_descriptor = Some(TextureViewDescriptor {
            dimension: Some(TextureViewDimension::D2Array),
            ..default()
        });
        image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::linear());
        images.add(image)
    };
    let albedo = atlas(bake::ALBEDO_FORMAT);
    let normal = atlas(bake::NORMAL_FORMAT);
    let side = u64::from(FRAMES * FRAME_TEXELS);
    // Four bytes a texel, two atlases, a third more for the mip levels.
    let bytes = side * side * 4 * 2 * u64::from(layers) * 4 / 3;
    let progress = Arc::new(AtomicU32::new(0));
    if !trees.is_empty() {
        queue.push(BakeJob {
            albedo: albedo.id(),
            normal: normal.id(),
            progress: progress.clone(),
            ..job
        });
    }
    impostors.pending = Some(Bake {
        snapshot: snapshot.clone(),
        trees,
        material: materials.add(ImpostorMaterial {
            settings: UVec4::new(FRAMES, 0, settings.switch.to_bits(), 0),
            albedo,
            normal,
        }),
        bytes,
        progress,
    });
}

/// The trees of `snapshot` to bake, largest first, and their records.
fn prepare(snapshot: &Snapshot, [sides, spans, blades]: [u32; 3]) -> (Vec<Tree>, BakeJob) {
    let plants = &snapshot.plant_bounds;
    let mut chosen: Vec<usize> = (0..plants.len())
        .filter(|&plant| plants[plant].sphere[3] >= TREE_RADIUS)
        .collect();
    chosen.sort_by(|&a, &b| plants[b].sphere[3].total_cmp(&plants[a].sphere[3]));
    chosen.truncate(MOST_TREES);
    let mut slot = vec![usize::MAX; plants.len()];
    for (index, &plant) in chosen.iter().enumerate() {
        slot[plant] = index;
    }
    let mut branches: Vec<Vec<BranchInstance>> = vec![Vec::new(); chosen.len()];
    let mut leaves: Vec<Vec<LeafInstance>> = vec![Vec::new(); chosen.len()];
    for branch in &snapshot.branches {
        if let Some(&index) = slot.get(branch.plant as usize)
            && index != usize::MAX
        {
            branches[index].push(*branch);
        }
    }
    for leaf in &snapshot.leaves {
        if let Some(&index) = slot.get(leaf.plant as usize)
            && index != usize::MAX
        {
            leaves[index].push(*leaf);
        }
    }
    let mut trees = Vec::with_capacity(chosen.len());
    let mut models = Vec::with_capacity(chosen.len());
    let (mut first_branch, mut first_leaf) = (0u32, 0u32);
    for (index, &plant) in chosen.iter().enumerate() {
        let [x, y, z, sphere] = plants[plant].sphere;
        let center = Vec3::new(x, y, z);
        let mut reach: f32 = 0.0;
        let mut ground = f32::INFINITY;
        for branch in &branches[index] {
            for ring in [branch.a, branch.b] {
                reach = reach.max(center.distance(Vec3::from_slice(&ring[..3])) + ring[3]);
                ground = ground.min(ring[1]);
            }
        }
        for leaf in &leaves[index] {
            reach =
                reach.max(center.distance(Vec3::from_array(leaf.center)) + leaf.size * CLUSTER_REACH);
        }
        let radius = if reach > 0.0 { reach } else { sphere } * MARGIN;
        let ground = if ground.is_finite() { ground } else { y - sphere };
        trees.push(Tree {
            plant,
            center,
            radius,
            ground,
        });
        let branch_count = branches[index].len() as u32;
        let leaf_count = leaves[index].len() as u32;
        models.push(BakeModel {
            center_radius: [x, y, z, radius],
            records: [first_branch, branch_count, first_leaf, leaf_count],
            shape: [sides, spans, blades, FRAMES],
            texel: [2.0 * radius / FRAME_TEXELS as f32, SMALLEST_BLADE, 0.0, 0.0],
        });
        first_branch += branch_count;
        first_leaf += leaf_count;
    }
    let species: Vec<SpeciesRecord> = snapshot.species.iter().map(SpeciesRecord::from).collect();
    let job = BakeJob {
        albedo: AssetId::default(),
        normal: AssetId::default(),
        frames: FRAMES,
        frame_texels: FRAME_TEXELS,
        mips: MIPS,
        models,
        branches: bytemuck::cast_slice(&branches.concat()).to_vec(),
        leaves: bytemuck::cast_slice(&leaves.concat()).to_vec(),
        species: bytemuck::cast_slice(&species).to_vec(),
        plants: bytemuck::cast_slice(plants).to_vec(),
        progress: Arc::default(),
    };
    (trees, job)
}

/// Draw a bake once every tree in it is baked.
fn finish_bake(
    mut impostors: ResMut<Impostors>,
    mut status: ResMut<ImpostorStatus>,
) {
    let impostors = &mut *impostors;
    let Some(pending) = &impostors.pending else {
        status.baking = None;
        return;
    };
    let done = pending.progress.load(Ordering::Acquire) as usize;
    if done < pending.trees.len() {
        status.baking = Some((done, pending.trees.len()));
        return;
    }
    impostors.current = impostors.pending.take();
    impostors.woods = None;
    impostors.marked = None;
    status.baking = None;
}

/// Mark the plants drawn as impostors in the forest's buffer: the baked
/// trees while their snapshot is on screen, none otherwise.
fn mark_plants(
    settings: Res<ImpostorSettings>,
    feed: Res<ForestFeed>,
    forest: Option<ResMut<ForestMaterials>>,
    mut stores: ForestStores,
    mut impostors: ResMut<Impostors>,
    mut status: ResMut<ImpostorStatus>,
) {
    let Some(mut forest) = forest else {
        return;
    };
    let Some(drawn) = feed.drawn() else {
        return;
    };
    if impostors.current.is_none() && impostors.written == 0 {
        return;
    }
    let current = impostors
        .current
        .as_ref()
        .filter(|current| settings.enabled && Arc::ptr_eq(&current.snapshot, drawn));
    let wanted = (Arc::as_ptr(drawn) as usize, current.is_some());
    if impostors.marked == Some(wanted) {
        return;
    }
    let mut spheres = vec![[0.0f32; 4]; drawn.plant_bounds.len().max(1)];
    if let Some(current) = current {
        for tree in &current.trees {
            spheres[tree.plant] = tree.center.extend(tree.radius).to_array();
        }
    }
    let mut bytes: Vec<u8> = bytemuck::cast_slice(&spheres).to_vec();
    // Clear whatever an earlier, longer write left behind.
    let length = bytes.len().max(impostors.written);
    bytes.resize(length, 0);
    impostors.written = length;
    forest.store_impostors(Bytes::new(&bytes), &mut stores);
    impostors.marked = Some(wanted);
    status.scene = wanted.1;
}

/// Spawn the bake's draws, follow the settings, and report.
fn show(
    mut commands: Commands,
    settings: Res<ImpostorSettings>,
    feed: Res<ForestFeed>,
    mut impostors: ResMut<Impostors>,
    mut status: ResMut<ImpostorStatus>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ImpostorMaterial>>,
    mut floors: ResMut<Assets<StandardMaterial>>,
    mut visibility: Query<&mut Visibility>,
) {
    let impostors = &mut *impostors;
    let Some(current) = &impostors.current else {
        return;
    };
    status.trees = current.trees.len();
    status.year = Some(current.snapshot.time);
    status.bytes = if current.trees.is_empty() { 0 } else { current.bytes };
    if let Some(mut material) = materials.get_mut(&current.material) {
        let wanted = UVec4::new(
            FRAMES,
            u32::from(status.scene),
            settings.switch.to_bits(),
            0,
        );
        if material.settings != wanted {
            material.settings = wanted;
        }
    }
    let woods = (settings.woods, settings.woods_reach);
    if impostors.woods != Some(woods) {
        for entity in impostors.entities.drain(..) {
            commands.entity(entity).despawn();
        }
        impostors.woods = Some(woods);
        if !current.trees.is_empty() {
            let mesh = impostors
                .mesh
                .get_or_insert_with(|| meshes.add(quad()))
                .clone();
            let bounds = feed.drawn().map_or(current.snapshot.bounds, |drawn| drawn.bounds);
            impostors.entities = spawn(
                &mut commands,
                current,
                &mesh,
                bounds,
                woods,
                &mut meshes,
                &mut floors,
            );
        }
    }
    let wanted = if settings.enabled {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for &entity in &impostors.entities {
        if let Ok(mut shown) = visibility.get_mut(entity) {
            shown.set_if_neq(wanted);
        }
    }
}

/// The scene's trees as impostors, in one draw, and the woods round the
/// scene in cells of their own, on a floor of their own.
fn spawn(
    commands: &mut Commands,
    bake: &Bake,
    mesh: &Handle<Mesh>,
    bounds: [[f32; 3]; 2],
    (count, reach): (u32, f32),
    meshes: &mut Assets<Mesh>,
    floors: &mut Assets<StandardMaterial>,
) -> Vec<Entity> {
    let mut entities = Vec::new();
    let scene: Vec<InstanceData> = bake
        .trees
        .iter()
        .enumerate()
        .map(|(layer, tree)| InstanceData {
            position: tree.center,
            scale: tree.radius,
            index: layer as u32,
            seed: 1,
            ..default()
        })
        .collect();
    entities.push(spawn_cell(commands, scene, mesh, &bake.material));

    let [low, high] = bounds.map(Vec3::from_array);
    let middle = (low + high) * 0.5;
    let inner = (high - low).xz().length() * 0.5 + WOODS_GAP;
    let outer = reach.max(inner + 1.0);
    let floor = low.y;
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    let mut cells: std::collections::HashMap<IVec2, Vec<InstanceData>> = Default::default();
    for _ in 0..count {
        // Even over the ring's area.
        let r = (inner * inner + rng.unit() * (outer * outer - inner * inner)).sqrt();
        let angle = rng.unit() * std::f32::consts::TAU;
        let foot = Vec3::new(
            middle.x + r * angle.cos(),
            floor,
            middle.z + r * angle.sin(),
        );
        let layer = (rng.unit() * bake.trees.len() as f32) as usize % bake.trees.len();
        let tree = &bake.trees[layer];
        let scale = WOODS_SCALE[0] + rng.unit() * (WOODS_SCALE[1] - WOODS_SCALE[0]);
        let yaw = rng.unit() * std::f32::consts::TAU;
        let lift = (tree.center.y - tree.ground) * scale;
        let instance = InstanceData {
            position: foot + Vec3::Y * lift,
            scale: tree.radius * scale,
            rotation: yaw,
            index: layer as u32,
            seed: 0,
            ..default()
        };
        let cell = (foot.xz() / WOODS_CELL).floor().as_ivec2();
        cells.entry(cell).or_default().push(instance);
    }
    for instances in cells.into_values() {
        entities.push(spawn_cell(commands, instances, mesh, &bake.material));
    }
    if count > 0 {
        let disc = meshes.add(Circle::new(outer + WOODS_CELL));
        entities.push(
            commands
                .spawn((
                    ForestGeometry,
                    Mesh3d(disc),
                    MeshMaterial3d(floors.add(StandardMaterial {
                        base_color: WOODS_FLOOR,
                        perceptual_roughness: 0.95,
                        ..default()
                    })),
                    Transform::from_translation(Vec3::new(middle.x, floor - 0.05, middle.z))
                        .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                    Visibility::Hidden,
                ))
                .id(),
        );
    }
    entities
}

/// One instanced draw of impostors, culled by its bounds on the GPU.
fn spawn_cell(
    commands: &mut Commands,
    instances: Vec<InstanceData>,
    mesh: &Handle<Mesh>,
    material: &Handle<ImpostorMaterial>,
) -> Entity {
    let mut low = Vec3::INFINITY;
    let mut high = Vec3::NEG_INFINITY;
    for instance in &instances {
        low = low.min(instance.position - Vec3::splat(instance.scale));
        high = high.max(instance.position + Vec3::splat(instance.scale));
    }
    commands
        .spawn((
            ForestGeometry,
            InstanceMaterialData {
                instances: Arc::new(instances),
                color: LinearRgba::WHITE,
                visibility_range: Vec4::new(0.0, 0.0, super::chunks::FAR, super::chunks::FAR),
            },
            Aabb::from_min_max(low, high),
            NoFrustumCulling,
            GpuCullCompute,
            Mesh3d(mesh.clone()),
            InstancedMeshMaterial(material.clone()),
            Transform::default(),
            Visibility::Hidden,
        ))
        .id()
}

/// A quad from corner (-1, -1) to (1, 1), which the impostor shader turns
/// to the eye.
fn quad() -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [-1.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
            [1.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0],
        ],
    )
    .with_inserted_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3]))
}

/// A small fixed-seed generator: the woods stand the same every time.
struct Rng(u64);

impl Rng {
    fn unit(&mut self) -> f32 {
        // SplitMix64.
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^= z >> 31;
        (z >> 40) as f32 / (1u64 << 24) as f32
    }
}

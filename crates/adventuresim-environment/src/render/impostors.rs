//! Trees as octahedral impostors, baked in the browser from the snapshot on
//! screen.
//!
//! Each tree the size of a tree, and each sapling of a species that grows
//! to one, is drawn from [`FRAMES`]² directions over the upper hemisphere
//! into one layer of an atlas ([`bake`]), trees finer than saplings; a tree then
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
/// A tree's view's side, texels: at the switch a tree spans about as many
/// pixels as its view has texels, so it is never drawn magnified.
pub const FRAME_TEXELS: u32 = 128;
/// A sapling's, which is never large on screen.
pub const SAPLING_TEXELS: u32 = 32;
/// Plants at least this wide, metres of radius, are trees with impostors:
/// the size class the forest keeps apart from shrubs.
const TREE_RADIUS: f32 = 4.0;
/// Saplings this wide and wider, of a species with trees, have impostors.
const SAPLING_RADIUS: f32 = 0.1;
/// The most trees and saplings a bake takes, largest first.
const MOST_TREES: usize = 64;
const MOST_SAPLINGS: usize = 192;

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
/// What tree and sapling impostors are tinted, to tell them apart.
const TREE_TINT: Vec3 = Vec3::new(1.0, 0.55, 0.2);
const SAPLING_TINT: Vec3 = Vec3::new(0.2, 0.8, 1.0);

/// Whether trees are drawn as impostors, and how.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct ImpostorSettings {
    pub enabled: bool,
    /// A scene tree spanning fewer pixels of radius than this is drawn as its
    /// impostor; zero keeps every scene tree whole.
    pub switch: f32,
    /// Trees, and saplings under them, in the woods round the scene.
    pub woods: u32,
    pub woods_saplings: u32,
    /// How far the woods reach from the scene's middle, metres.
    pub woods_reach: f32,
    /// The baked leaves' coverage is multiplied by this as they are drawn:
    /// above one, a partly covered texel fills in, and an open one stays
    /// open.
    pub density: f32,
    /// How much the coverage grows each mip level below a view's full size,
    /// to keep a distant crown as full as a near one.
    pub mip_scale: f32,
    /// Each blade is baked this much larger, across and along; applies to
    /// the next bake.
    pub leaf_size: f32,
    /// Tint tree impostors orange and sapling impostors cyan.
    pub tint: bool,
    /// Raise to bake again now.
    pub rebake: u32,
}

impl Default for ImpostorSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            switch: 40.0,
            woods: 20_000,
            woods_saplings: 20_000,
            woods_reach: 1_000.0,
            density: 1.0,
            mip_scale: 0.25,
            leaf_size: 1.0,
            tint: false,
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
    /// Trees and saplings in the bake drawn now, and the year it was taken.
    pub trees: usize,
    pub saplings: usize,
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
    /// z: the switch, pixels of radius, as f32 bits; w: the leaf density, as
    /// f32 bits.
    #[uniform(0)]
    settings: UVec4,
    #[texture(1, dimension = "2d_array")]
    #[sampler(2)]
    albedo: Handle<Image>,
    #[texture(3, dimension = "2d_array")]
    normal: Handle<Image>,
    /// rgb: the set's tint; w: 1 while tinted.
    #[uniform(5)]
    tint: Vec4,
    /// x: the coverage's growth a mip level.
    #[uniform(6)]
    coverage_scale: Vec4,
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

/// A bake: the trees and the saplings, and the snapshot they were baked
/// from.
struct Bake {
    snapshot: Arc<Snapshot>,
    trees: Set,
    saplings: Set,
}

/// Plants baked alike: their atlas, as a material, and how far along.
struct Set {
    trees: Vec<Tree>,
    material: Handle<ImpostorMaterial>,
    bytes: u64,
    progress: Arc<AtomicU32>,
}

impl Bake {
    fn sets(&self) -> [&Set; 2] {
        [&self.trees, &self.saplings]
    }
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
    woods: Option<(u32, u32, f32)>,
    /// The scene's terrain, which the woods' floor leaves a hole for: its
    /// lower and upper corners.
    terrain: Option<[Vec3; 2]>,
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
    let shape = [shape.sides, shape.spans, shape.blades];
    let (trees, saplings) = choose(snapshot);
    let mut jobs = Vec::new();
    let mut set = |chosen: Vec<usize>, texels: u32| {
        let (trees, job) = prepare(snapshot, &chosen, texels, shape, settings.leaf_size);
        let layers = trees.len().max(1) as u32;
        let side = FRAMES * texels;
        let mips = texels.ilog2() + 1;
        let mut atlas = |format| {
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
            image.texture_descriptor.mip_level_count = mips;
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
        let side = u64::from(side);
        // Four bytes a texel, two atlases, a third more for the mip levels.
        let bytes = side * side * 4 * 2 * u64::from(layers) * 4 / 3;
        let progress = Arc::new(AtomicU32::new(0));
        if !trees.is_empty() {
            jobs.push(BakeJob {
                albedo: albedo.id(),
                normal: normal.id(),
                progress: progress.clone(),
                ..job
            });
        }
        Set {
            bytes: if trees.is_empty() { 0 } else { bytes },
            trees,
            material: materials.add(ImpostorMaterial {
                settings: UVec4::new(FRAMES, 0, settings.switch.to_bits(), settings.density.to_bits()),
                albedo,
                normal,
                tint: Vec4::ZERO,
                coverage_scale: Vec4::new(settings.mip_scale, 0.0, 0.0, 0.0),
            }),
            progress,
        }
    };
    let trees = set(trees, FRAME_TEXELS);
    let saplings = set(saplings, SAPLING_TEXELS);
    for (set, tint) in [(&trees, TREE_TINT), (&saplings, SAPLING_TINT)] {
        if let Some(mut material) = materials.get_mut(&set.material) {
            material.tint = tint.extend(0.0);
        }
    }
    queue.push(jobs);
    impostors.pending = Some(Bake {
        snapshot: snapshot.clone(),
        trees,
        saplings,
    });
}

/// The plants of `snapshot` to bake, each largest first: its trees, and the
/// saplings of species some of whose plants are trees.
fn choose(snapshot: &Snapshot) -> (Vec<usize>, Vec<usize>) {
    let plants = &snapshot.plant_bounds;
    let radius = |plant: usize| plants[plant].sphere[3];
    let mut tree_species = vec![false; snapshot.species.len()];
    for plant in plants {
        if plant.sphere[3] >= TREE_RADIUS
            && let Some(species) = tree_species.get_mut(plant.species as usize)
        {
            *species = true;
        }
    }
    let largest_first = |mut chosen: Vec<usize>, most: usize| {
        chosen.sort_by(|&a, &b| radius(b).total_cmp(&radius(a)));
        chosen.truncate(most);
        chosen
    };
    let trees = (0..plants.len()).filter(|&plant| radius(plant) >= TREE_RADIUS).collect();
    let saplings = (0..plants.len())
        .filter(|&plant| {
            let r = radius(plant);
            (SAPLING_RADIUS..TREE_RADIUS).contains(&r)
                && tree_species
                    .get(plants[plant].species as usize)
                    .copied()
                    .unwrap_or(false)
        })
        .collect();
    (
        largest_first(trees, MOST_TREES),
        largest_first(saplings, MOST_SAPLINGS),
    )
}

/// The `chosen` plants of `snapshot` as trees to bake at `texels` a view,
/// and their records.
fn prepare(
    snapshot: &Snapshot,
    chosen: &[usize],
    texels: u32,
    [sides, spans, blades]: [u32; 3],
    leaf_size: f32,
) -> (Vec<Tree>, BakeJob) {
    let plants = &snapshot.plant_bounds;
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
            texel: [2.0 * radius / texels as f32, SMALLEST_BLADE, leaf_size, 0.0],
        });
        first_branch += branch_count;
        first_leaf += leaf_count;
    }
    let species: Vec<SpeciesRecord> = snapshot.species.iter().map(SpeciesRecord::from).collect();
    let job = BakeJob {
        albedo: AssetId::default(),
        normal: AssetId::default(),
        frames: FRAMES,
        frame_texels: texels,
        mips: texels.ilog2() + 1,
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
    let (done, all) = pending.sets().iter().fold((0, 0), |(done, all), set| {
        let baked = set.progress.load(Ordering::Acquire) as usize;
        (done + baked, all + set.trees.len())
    });
    if done < all {
        status.baking = Some((done, all));
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
        for tree in current.sets().iter().flat_map(|set| &set.trees) {
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
    // Only a scene's first snapshot brings its terrain.
    if let Some(terrain) = feed.drawn().and_then(|drawn| drawn.terrain.as_ref()) {
        let mut low = Vec3::INFINITY;
        let mut high = Vec3::NEG_INFINITY;
        for vertex in &terrain.vertices {
            low = low.min(Vec3::from_array(vertex.position));
            high = high.max(Vec3::from_array(vertex.position));
        }
        let corners = low.is_finite().then_some([low, high]);
        if corners != impostors.terrain {
            impostors.terrain = corners;
            impostors.woods = None;
        }
    }
    let Some(current) = &impostors.current else {
        return;
    };
    status.trees = current.trees.trees.len();
    status.saplings = current.saplings.trees.len();
    status.year = Some(current.snapshot.time);
    status.bytes = current.sets().iter().map(|set| set.bytes).sum();
    let wanted = UVec4::new(
        FRAMES,
        u32::from(status.scene),
        settings.switch.to_bits(),
        settings.density.to_bits(),
    );
    let tinted = f32::from(u8::from(settings.tint));
    for set in current.sets() {
        if let Some(mut material) = materials.get_mut(&set.material)
            && (material.settings != wanted
                || material.tint.w != tinted
                || material.coverage_scale.x != settings.mip_scale)
        {
            material.settings = wanted;
            material.tint.w = tinted;
            material.coverage_scale.x = settings.mip_scale;
        }
    }
    let woods = (settings.woods, settings.woods_saplings, settings.woods_reach);
    if impostors.woods != Some(woods) {
        for entity in impostors.entities.drain(..) {
            commands.entity(entity).despawn();
        }
        impostors.woods = Some(woods);
        if current.sets().iter().any(|set| !set.trees.is_empty()) {
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
                impostors.terrain,
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
    terrain: Option<[Vec3; 2]>,
    (trees, saplings, reach): (u32, u32, f32),
    meshes: &mut Assets<Mesh>,
    floors: &mut Assets<StandardMaterial>,
) -> Vec<Entity> {
    let mut entities = Vec::new();
    for set in bake.sets() {
        if set.trees.is_empty() {
            continue;
        }
        let scene: Vec<InstanceData> = set
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
        entities.push(spawn_cell(commands, scene, mesh, &set.material));
    }

    let [low, high] = bounds.map(Vec3::from_array);
    let middle = (low + high) * 0.5;
    let inner = (high - low).xz().length() * 0.5 + WOODS_GAP;
    let outer = reach.max(inner + 1.0);
    let floor = low.y;
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    // Each set's woods by cell.
    let mut cells: [std::collections::HashMap<IVec2, Vec<InstanceData>>; 2] = Default::default();
    let sets = bake.sets();
    // The saplings stand among the trees, or alone when no tree was baked.
    let picks = [
        (0, if sets[0].trees.is_empty() { 0 } else { trees }),
        (1, if sets[1].trees.is_empty() { 0 } else { saplings }),
    ];
    let count = trees + saplings;
    for which in picks
        .into_iter()
        .flat_map(|(which, count)| std::iter::repeat_n(which, count as usize))
    {
        // Even over the ring's area.
        let r = (inner * inner + rng.unit() * (outer * outer - inner * inner)).sqrt();
        let angle = rng.unit() * std::f32::consts::TAU;
        let foot = Vec3::new(
            middle.x + r * angle.cos(),
            floor,
            middle.z + r * angle.sin(),
        );
        let set = sets[which];
        let layer = (rng.unit() * set.trees.len() as f32) as usize % set.trees.len();
        let tree = &set.trees[layer];
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
        cells[which].entry(cell).or_default().push(instance);
    }
    for (set, cells) in sets.iter().zip(cells) {
        for instances in cells.into_values() {
            entities.push(spawn_cell(commands, instances, mesh, &set.material));
        }
    }
    if count > 0 {
        // Round the scene's terrain, never under it: two floors at one
        // height fight over every pixel.
        let [hole_low, hole_high] = terrain.unwrap_or([low, high]);
        let floor_y = terrain.map_or(floor, |[low, _]| low.y);
        let reach = outer + WOODS_CELL;
        let frame = floor_frame(
            hole_low.xz(),
            hole_high.xz(),
            middle.xz() - Vec2::splat(reach),
            middle.xz() + Vec2::splat(reach),
            floor_y,
        );
        entities.push(
            commands
                .spawn((
                    ForestGeometry,
                    Mesh3d(meshes.add(frame)),
                    MeshMaterial3d(floors.add(StandardMaterial {
                        base_color: WOODS_FLOOR,
                        perceptual_roughness: 0.95,
                        ..default()
                    })),
                    // Flat ground shades itself from nothing; casting would
                    // only stripe it.
                    bevy::light::NotShadowCaster,
                    Transform::default(),
                    Visibility::Hidden,
                ))
                .id(),
        );
    }
    entities
}

/// Flat ground at `y` from `outer_low` to `outer_high`, but for the hole from
/// `hole_low` to `hole_high`: four rectangles round it.
fn floor_frame(hole_low: Vec2, hole_high: Vec2, outer_low: Vec2, outer_high: Vec2, y: f32) -> Mesh {
    let (a, b) = (outer_low, outer_high);
    let (h, k) = (hole_low.max(a), hole_high.min(b));
    let rects = [
        [a, Vec2::new(b.x, h.y)],
        [Vec2::new(a.x, k.y), b],
        [Vec2::new(a.x, h.y), Vec2::new(h.x, k.y)],
        [Vec2::new(k.x, h.y), Vec2::new(b.x, k.y)],
    ];
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    for [low, high] in rects {
        let base = positions.len() as u32;
        positions.extend([
            [low.x, y, low.y],
            [high.x, y, low.y],
            [high.x, y, high.y],
            [low.x, y, high.y],
        ]);
        // Counter-clockwise from above.
        indices.extend([0, 2, 1, 0, 3, 2].map(|i| base + i));
    }
    let normals = vec![[0.0, 1.0, 0.0]; positions.len()];
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices))
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

use super::super::{DistantCityBuildingPresentation, PendingCityBuildings};
use super::*;
use bevy::{
    camera::{
        primitives::Aabb,
        visibility::{NoFrustumCulling, RenderLayers},
    },
    render::{batching::NoAutomaticBatching, render_resource::ShaderType},
};
use std::collections::HashMap;

// Low bits encode LOD; facade decals are not part of the shadow silhouette.
const FACADE_OVERLAY_FLAG: u32 = 1 << 8;
const LOD_LEVEL_MASK: u32 = 3;
const COMPONENT_FLAG: u32 = 1 << 9;
const COMPONENT_INDEX_SHIFT: u32 = 10;
const COMPONENT_RECORD: u32 = 2;

mod input;
mod props;
mod ranges;
pub(in crate::presentation::buildings) use input::PendingGpuBuildings;
pub(super) use ranges::DrawRange;
#[cfg(test)]
mod tests;

#[derive(Clone, ShaderType)]
pub(super) struct Building {
    transform: Mat4,
    bounds: Vec4,
    levels: UVec4,
    uv_offset: Vec4,
}

#[derive(Clone)]
struct Part {
    entity: Option<Entity>,
    root: Entity,
    transform: Mat4,
    local_transform: Mat4,
    uv_offset: Vec2,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    level: u32,
    fade: Option<std::ops::Range<f32>>,
}

type MaterialRanges =
    HashMap<(usize, AssetId<StandardMaterial>), (Handle<StandardMaterial>, Vec<UVec4>)>;

struct PackedCity {
    geometry: geometry::Geometry,
    buildings: Vec<Building>,
    batches: MaterialRanges,
}

fn pack(world: &World, parts: impl Iterator<Item = Part> + Clone) -> PackedCity {
    let mut geometry = geometry::Geometry::default();
    let mut meshes = HashMap::new();
    let mut roots = HashMap::new();
    let mut components = HashMap::new();
    let mut buildings: Vec<Building> = Vec::new();
    let mut batches = MaterialRanges::new();
    // All component records share their owner's bounds and LOD choice. A beam
    // must never select a different representation from the surrounding facade.
    let mut bounds: HashMap<Entity, (Vec4, u32)> = HashMap::new();
    for part in parts.clone() {
        let slices = meshes.entry(part.mesh.id()).or_insert_with(|| {
            geometry.insert(
                world
                    .resource::<Assets<Mesh>>()
                    .get(&part.mesh)
                    .expect("resident city mesh"),
            )
        });
        let radius = slices.iter().map(|slice| slice.radius).fold(0.0, f32::max)
            + part.local_transform.w_axis.truncate().length();
        let bound = bounds
            .entry(part.root)
            .or_insert((part.transform.w_axis.truncate().extend(0.0), 0));
        bound.0.w = bound.0.w.max(radius);
        bound.1 |= 1 << (part.level & LOD_LEVEL_MASK);
    }
    for part in parts {
        let slices = &meshes[&part.mesh.id()];
        let building = *roots.entry(part.root).or_insert_with(|| {
            let index = buildings.len();
            buildings.push(Building {
                transform: part.transform,
                bounds: bounds[&part.root].0,
                uv_offset: Vec4::ZERO,
                levels: part.fade.as_ref().map_or(UVec4::ZERO, |fade| {
                    UVec4::new(0, 1, fade.start.to_bits(), fade.end.to_bits())
                }),
            });
            index
        });
        let record = &mut buildings[building];
        record.levels.x = bounds[&part.root].1;
        let component = component_index(&part, &mut buildings, &mut components);
        for slice in slices.iter() {
            let jobs = &mut batches
                .entry((slice.page, part.material.id()))
                .or_insert_with(|| (part.material.clone(), Vec::new()))
                .1;
            jobs.push(UVec4::new(
                building as u32,
                slice.start,
                slice.count,
                part.level | component,
            ));
        }
    }

    PackedCity {
        geometry,
        buildings,
        batches,
    }
}

type ComponentTransforms = HashMap<([u32; 16], [u32; 2]), u32>;

fn component_index(
    part: &Part,
    records: &mut Vec<Building>,
    components: &mut ComponentTransforms,
) -> u32 {
    if part.local_transform == Mat4::IDENTITY && part.uv_offset == Vec2::ZERO {
        return 0;
    }
    *components
        .entry((
            part.local_transform.to_cols_array().map(f32::to_bits),
            part.uv_offset.to_array().map(f32::to_bits),
        ))
        .or_insert_with(|| {
            let index = u32::try_from(records.len()).expect("bounded city component count");
            assert!(index <= u32::MAX >> COMPONENT_INDEX_SHIFT);
            records.push(Building {
                transform: part.local_transform,
                bounds: Vec4::ZERO,
                levels: UVec4::new(0, COMPONENT_RECORD, 0, 0),
                uv_offset: part.uv_offset.extend(0.0).extend(0.0),
            });
            COMPONENT_FLAG | index << COMPONENT_INDEX_SHIFT
        })
}

fn spawn_batch(
    world: &mut World,
    handle: Handle<StandardMaterial>,
    jobs: Vec<UVec4>,
    vertices: &Handle<ShaderBuffer>,
    indices: &Handle<ShaderBuffer>,
    anchor: &Handle<Mesh>,
) -> GpuBatch {
    let authored = world
        .resource::<Assets<StandardMaterial>>()
        .get(&handle)
        .expect("resident city material")
        .clone();
    let grouped = ranges::InstanceRanges::new(jobs);
    let ranges = grouped.ranges.len() as u32;
    let capacity = grouped.capacity;
    let owners = world
        .resource_mut::<Assets<ShaderBuffer>>()
        .add(ShaderBuffer::from(grouped.owners));
    let source = world
        .resource_mut::<Assets<ShaderBuffer>>()
        .add(ShaderBuffer::from(grouped.ranges));
    let material = world
        .resource_mut::<Assets<material::CityMaterial>>()
        .add(material::CityMaterial::from_source(authored));
    world.spawn((
        Name::new("GPU city material batch"),
        DistantCityBuildingPresentation,
        Mesh3d(anchor.clone()),
        MeshMaterial3d(material.clone()),
        Transform::default(),
        NoFrustumCulling,
        NoAutomaticBatching,
        RenderLayers::layer(0),
        Aabb {
            center: Vec3::ZERO.into(),
            half_extents: Vec3::splat(1.0).into(),
        },
    ));
    GpuBatch {
        owners,
        material,
        source,
        vertices: vertices.clone(),
        indices: indices.clone(),
        ranges,
        capacity,
    }
}

fn upload(world: &mut World, packed: PackedCity) -> CityGpuScene {
    let PackedCity {
        geometry,
        buildings,
        batches,
    } = packed;
    let count = buildings.len() as u32;
    let geometry_bytes: usize = geometry
        .pages
        .iter()
        .map(|p| p.vertices.len() * size_of::<Vec4>() + p.indices.len() * size_of::<u32>())
        .sum();
    let unshared_ranges = batches.values().map(|(_, jobs)| jobs.len()).sum::<usize>();
    let clusters: u32 = batches
        .values()
        .flat_map(|(_, jobs)| jobs)
        .map(|job| job.z.div_ceil(VERTICES_PER_CLUSTER))
        .sum();
    let pages: Vec<_> = geometry
        .pages
        .into_iter()
        .map(|page| {
            let mut buffers = world.resource_mut::<Assets<ShaderBuffer>>();
            (
                buffers.add(ShaderBuffer::from(page.vertices)),
                buffers.add(ShaderBuffer::from(page.indices)),
            )
        })
        .collect();
    let buildings = world
        .resource_mut::<Assets<ShaderBuffer>>()
        .add(ShaderBuffer::from(buildings));
    let selection = world
        .resource_mut::<Assets<ShaderBuffer>>()
        .add(ShaderBuffer::with_size(
            count as usize * MAX_CITY_VIEWS * size_of::<u32>(),
            bevy::asset::RenderAssetUsages::RENDER_WORLD,
        ));
    let anchor = world
        .resource_mut::<Assets<Mesh>>()
        .add(Cuboid::new(1.0, 1.0, 1.0));
    let mut gpu_batches = Vec::new();
    for ((page, _), (handle, jobs)) in batches {
        gpu_batches.push(spawn_batch(
            world,
            handle,
            jobs,
            &pages[page].0,
            &pages[page].1,
            &anchor,
        ));
    }
    info!(
        buildings = count,
        batches = gpu_batches.len(),
        clusters,
        unshared_ranges,
        ranges = gpu_batches.iter().map(|batch| batch.ranges).sum::<u32>(),
        geometry_bytes,
        pages = pages.len(),
        "GPU city resident"
    );
    CityGpuScene {
        buildings,
        selection,
        count,
        batches: Arc::new(gpu_batches),
    }
}

/// Pack once after generation and deferred material updates have completed.
pub(super) fn assemble(world: &mut World) {
    if world
        .get_resource::<PendingCityBuildings>()
        .is_some_and(|pending| !pending.finished())
    {
        return;
    }
    let mut pending = std::mem::take(&mut *world.resource_mut::<PendingGpuBuildings>());
    // Only publish alongside a completed city; interactive furniture is never
    // consumed by this static scenery path.
    if pending.is_empty() {
        return;
    }
    pending.parts.extend(props::parts(world));
    READY.store(false, Ordering::Relaxed);
    let packed = pack(world, pending.iter(world.get_resource()));
    let scene = upload(world, packed);
    let prop_roots: std::collections::HashSet<_> = pending
        .parts
        .iter()
        .filter(|part| part.fade.is_some())
        .map(|part| part.root)
        .collect();
    for part in pending.parts {
        if let Some(entity) = part.entity {
            world.entity_mut(entity).despawn();
        }
    }
    // Vista descriptors still retain the recipes and identities. These roots
    // have no physics, animation or remaining render children to update.
    for root in prop_roots {
        world.entity_mut(root).despawn();
    }
    world.insert_resource(scene);
}

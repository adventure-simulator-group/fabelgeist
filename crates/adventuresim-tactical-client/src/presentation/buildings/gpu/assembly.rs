use super::super::{DistantCityBuildingPresentation, PendingCityBuildings};
use super::*;
use bevy::{
    camera::{primitives::Aabb, visibility::NoFrustumCulling},
    render::{batching::NoAutomaticBatching, render_resource::ShaderType},
};
pub(in crate::presentation::buildings) use input::{
    PendingGpuBuildings, PendingGpuCities, PlacementAppearance,
};
use packing::pack;
pub(super) use ranges::DrawRange;
use std::collections::HashMap;

mod input;
mod packing;
mod props;
mod ranges;

// Low bits encode LOD; facade decals are not part of the shadow silhouette.
const FACADE_OVERLAY_FLAG: u32 = 1 << 8;
const LOD_LEVEL_MASK: u32 = 3;
const COMPONENT_FLAG: u32 = 1 << 9;
const COMPONENT_INDEX_SHIFT: u32 = 10;
const COMPONENT_RECORD: u32 = 2;
type AssemblyResult<T> = Result<T, AssemblyError>;

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
    HashMap<(usize, AssetId<StandardMaterial>), (Handle<StandardMaterial>, ranges::InstanceRanges)>;

struct PackedCity {
    geometry: geometry::Geometry,
    buildings: Vec<Building>,
    batches: MaterialRanges,
}
type ComponentTransforms = HashMap<([u32; 16], [u32; 2]), u32>;

#[derive(Debug, thiserror::Error)]
pub(in crate::presentation) enum AssemblyError {
    #[error("City building materials are not resident")]
    MissingMaterials,
    #[error("A city source material is not resident")]
    MissingMaterial,
    #[error("A city source mesh is not resident")]
    MissingMesh,
    #[error("City component storage capacity is exceeded")]
    ComponentCapacity,
}

/// Acknowledges the exact candidate root; another resident city is insufficient.
#[derive(Component)]
pub(in crate::presentation) struct CityAssemblyPublished;

#[derive(Component, Debug)]
pub(in crate::presentation) struct CityAssemblyFailure(pub AssemblyError);

/// Pack once after generation and deferred material updates have completed.
pub(super) fn assemble(world: &mut World) {
    for owner in PresentationOwner::ALL {
        if owner == PresentationOwner::Scene
            && world
                .get_resource::<PendingCityBuildings>()
                .is_some_and(|pending| !pending.finished())
        {
            continue;
        }
        assemble_owner(world, owner);
    }
}

fn component_index(
    part: &Part,
    records: &mut Vec<Building>,
    components: &mut ComponentTransforms,
) -> AssemblyResult<u32> {
    if part.local_transform == Mat4::IDENTITY && part.uv_offset == Vec2::ZERO {
        return Ok(0);
    }
    let key = (
        part.local_transform.to_cols_array().map(f32::to_bits),
        part.uv_offset.to_array().map(f32::to_bits),
    );
    if let Some(index) = components.get(&key) {
        return Ok(*index);
    }
    let index = u32::try_from(records.len()).map_err(|_| AssemblyError::ComponentCapacity)?;
    if index > u32::MAX >> COMPONENT_INDEX_SHIFT {
        return Err(AssemblyError::ComponentCapacity);
    }
    records.push(Building {
        transform: part.local_transform,
        bounds: Vec4::ZERO,
        levels: UVec4::new(0, COMPONENT_RECORD, 0, 0),
        uv_offset: part.uv_offset.extend(0.0).extend(0.0),
    });
    let component = COMPONENT_FLAG | index << COMPONENT_INDEX_SHIFT;
    components.insert(key, component);
    Ok(component)
}

fn spawn_batch(
    world: &mut World,
    owner: PresentationOwner,
    handle: Handle<StandardMaterial>,
    grouped: ranges::InstanceRanges,
    vertices: &Handle<ShaderBuffer>,
    indices: &Handle<ShaderBuffer>,
    anchor: &Handle<Mesh>,
) -> AssemblyResult<GpuBatch> {
    let authored = world
        .resource::<Assets<StandardMaterial>>()
        .get(&handle)
        .ok_or(AssemblyError::MissingMaterial)?
        .clone();
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
    let entity = world
        .spawn((
            Name::new("GPU city material batch"),
            DistantCityBuildingPresentation,
            CityBatchAnchor,
            owner,
            Mesh3d(anchor.clone()),
            MeshMaterial3d(material.clone()),
            Transform::default(),
            NoFrustumCulling,
            NoAutomaticBatching,
            owner.render_layers(),
            Aabb {
                center: Vec3::ZERO.into(),
                half_extents: Vec3::splat(1.0).into(),
            },
        ))
        .id();
    Ok(GpuBatch {
        anchor: entity,
        owners,
        material,
        source,
        vertices: vertices.clone(),
        indices: indices.clone(),
        ranges,
        capacity,
    })
}

fn upload(
    world: &mut World,
    owner: PresentationOwner,
    packed: PackedCity,
    frame: CityFrame,
) -> AssemblyResult<CityGpuScene> {
    let PackedCity {
        geometry,
        buildings,
        batches,
    } = packed;
    if batches.values().any(|(material, _)| {
        world
            .resource::<Assets<StandardMaterial>>()
            .get(material)
            .is_none()
    }) {
        return Err(AssemblyError::MissingMaterial);
    }
    let count = buildings.len() as u32;
    let frame_buffer = world
        .resource_mut::<Assets<ShaderBuffer>>()
        .add(ShaderBuffer::from(frame));
    let geometry_bytes: usize = geometry
        .pages
        .iter()
        .map(|p| p.vertices.len() * size_of::<Vec4>() + p.indices.len() * size_of::<u32>())
        .sum();
    let unshared_ranges = batches
        .values()
        .map(|(_, jobs)| jobs.unshared_ranges())
        .sum::<usize>();
    let clusters: u32 = batches.values().map(|(_, jobs)| jobs.capacity).sum();
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
            owner,
            handle,
            jobs,
            &pages[page].0,
            &pages[page].1,
            &anchor,
        )?);
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
    Ok(CityGpuScene {
        frame,
        frame_buffer,
        buildings,
        selection,
        count,
        batches: Arc::new(gpu_batches),
    })
}

fn assemble_owner(world: &mut World, owner: PresentationOwner) {
    let mut pending = std::mem::take(
        world
            .resource_mut::<PendingGpuCities>()
            .owners
            .get_mut(owner),
    );
    if pending
        .publication
        .as_ref()
        .is_some_and(|publication| world.get_entity(publication.root).is_err())
    {
        return;
    }
    // Only publish alongside a completed city; interactive furniture is never
    // consumed by this static scenery path.
    if pending.is_empty() {
        if let Some(publication) = pending.publication {
            super::reset(world, owner);
            world
                .resource_mut::<CityGpuScenes>()
                .owners
                .get_mut(owner)
                .frame = publication.frame;
            world
                .entity_mut(publication.root)
                .insert(CityAssemblyPublished)
                .remove::<CityAssemblyFailure>();
            READY.get(owner).store(true, Ordering::Relaxed);
        }
        return;
    }
    if owner == PresentationOwner::Scene {
        pending.parts.extend(props::parts(world));
    }
    let frame = pending
        .publication
        .as_ref()
        .map(|publication| publication.frame)
        .unwrap_or(world.resource::<CityGpuScenes>().owners.get(owner).frame);
    let scene = pending
        .groups(world.get_resource())
        .and_then(|groups| pack(world, groups))
        .and_then(|packed| upload(world, owner, packed, frame));
    let scene = match scene {
        Ok(scene) => scene,
        Err(error) => {
            warn!(?owner, %error, "Could not assemble GPU city");
            if let Some(publication) = &pending.publication {
                world
                    .entity_mut(publication.root)
                    .insert(CityAssemblyFailure(error));
            }
            return;
        }
    };
    if let Some(publication) = &pending.publication {
        world
            .entity_mut(publication.root)
            .insert(CityAssemblyPublished)
            .remove::<CityAssemblyFailure>();
    }
    READY.get(owner).store(false, Ordering::Relaxed);
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
    let previous = std::mem::replace(
        world.resource_mut::<CityGpuScenes>().owners.get_mut(owner),
        scene,
    );
    for batch in previous.batches.iter() {
        if let Ok(entity) = world.get_entity_mut(batch.anchor) {
            entity.despawn();
        }
    }
}

#[cfg(test)]
mod tests;

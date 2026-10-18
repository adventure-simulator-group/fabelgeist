//! Pack each prototype once, then reference its shared placement list.
use super::*;

pub(super) fn pack(world: &World, groups: Vec<input::Group>) -> AssemblyResult<PackedCity> {
    let mut geometry = geometry::Geometry::default();
    let mut meshes = HashMap::new();
    let mut components = HashMap::new();
    let mut buildings = Vec::new();
    let mut batches: HashMap<_, (Handle<StandardMaterial>, ranges::RangeGroups)> = HashMap::new();
    for group in groups {
        let mut radius: f32 = 0.0;
        let mut levels = 0;
        for part in &group.parts {
            let slices = match meshes.entry(part.mesh.id()) {
                std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
                std::collections::hash_map::Entry::Vacant(entry) => entry.insert(
                    geometry.insert(
                        world
                            .resource::<Assets<Mesh>>()
                            .get(&part.mesh)
                            .ok_or(AssemblyError::MissingMesh)?,
                    ),
                ),
            };
            radius = radius.max(
                slices.iter().map(|slice| slice.radius).fold(0.0, f32::max)
                    + part.local_transform.w_axis.truncate().length(),
            );
            levels |= 1 << (part.level & LOD_LEVEL_MASK);
        }
        let Some(first) = group.parts.first() else {
            continue;
        };
        let owners: Arc<[u32]> = group
            .placements
            .iter()
            .map(|&transform| {
                let index = buildings.len() as u32;
                buildings.push(Building {
                    transform,
                    bounds: transform.w_axis.truncate().extend(radius),
                    uv_offset: Vec4::ZERO,
                    levels: first
                        .fade
                        .as_ref()
                        .map_or(UVec4::new(levels, 0, 0, 0), |fade| {
                            UVec4::new(levels, 1, fade.start.to_bits(), fade.end.to_bits())
                        }),
                });
                index
            })
            .collect();
        for part in &group.parts {
            let component = component_index(part, &mut buildings, &mut components)?;
            for slice in &meshes[&part.mesh.id()] {
                batches
                    .entry((slice.page, part.material.id()))
                    .or_insert_with(|| (part.material.clone(), ranges::RangeGroups::default()))
                    .1
                    .push(
                        (slice.start, slice.count, part.level | component),
                        owners.clone(),
                    );
            }
        }
    }
    Ok(PackedCity {
        geometry,
        buildings,
        batches: batches
            .into_iter()
            .map(|(key, (handle, groups))| (key, (handle, groups.finish())))
            .collect(),
    })
}

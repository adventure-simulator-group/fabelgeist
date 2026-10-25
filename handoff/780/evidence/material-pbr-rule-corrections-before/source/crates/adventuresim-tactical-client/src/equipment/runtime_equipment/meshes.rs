//! Material partitions of generated equipment, sharing one rigid origin.
use super::*;
use adventuresim_core::equipment_assets::CatalogItem;
use fabelgeist_armor::ComponentAppearance;

struct MaterialPartition<'a> {
    appearance: Option<&'a ComponentAppearance>,
    indices: Vec<u32>,
}

#[derive(Clone)]
pub(super) struct CachedPart {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

/// Partition by authored material, retaining untagged faces in the item's
/// catalog material. Shared vertices are compacted independently per draw.
pub(super) fn equipment_parts(
    armor: &GeneratedArmor,
    item: CatalogItem<'static>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
) -> anyhow::Result<Vec<CachedPart>> {
    let mut groups: Vec<MaterialPartition<'_>> = Vec::new();
    for (triangle, face) in armor.indices.as_chunks::<3>().0.iter().enumerate() {
        let component = armor
            .components
            .iter()
            .find(|component| component.indices.contains(&(triangle * 3)));
        let appearance = component
            .map(|component| &component.appearance)
            .filter(|appearance| !matches!(appearance, ComponentAppearance::Inherit));
        let group = match groups
            .iter()
            .position(|found| found.appearance == appearance)
        {
            Some(index) => index,
            None => {
                groups.push(MaterialPartition {
                    appearance,
                    indices: Vec::new(),
                });
                groups.len() - 1
            }
        };
        groups[group].indices.extend(face);
    }
    groups
        .into_iter()
        .map(|MaterialPartition { appearance, indices }| {
            let mesh = runtime_equipment_mesh(armor, &indices, meshes)?;
            let material = match appearance {
                Some(ComponentAppearance::Textile(fabric)) => {
                    materials.add(adventuresim_character_creator::textile_surface::material(fabric, images)?)
                }
                Some(ComponentAppearance::Surface(surface)) => {
                    materials.add(adventuresim_character_creator::surface_projection::standard_material(*surface))
                }
                Some(ComponentAppearance::Inherit) | None => {
                    runtime_equipment_material(item, materials)?
                }
            };
            Ok(CachedPart { mesh, material })
        })
        .collect()
}

/// All material parts use the same origin when carried or dropped. Unused
/// body vertices in clothing cutouts must not move that origin.
pub(super) fn referenced_center(armor: &GeneratedArmor) -> Vec3 {
    let (min, max) = armor.indices.iter().fold(
        (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
        |(min, max), &index| {
            let p = Vec3::from_array(armor.positions[index as usize]);
            (min.min(p), max.max(p))
        },
    );
    (min + max) * 0.5
}

#[cfg(test)]
#[path = "mesh_tests.rs"]
mod tests;

fn runtime_equipment_mesh(
    armor: &GeneratedArmor,
    indices: &[u32],
    meshes: &mut Assets<Mesh>,
) -> anyhow::Result<Handle<Mesh>> {
    let mut remap = BTreeMap::new();
    let mut vertices = Vec::new();
    let indices = indices
        .iter()
        .map(|&index| {
            *remap.entry(index).or_insert_with(|| {
                vertices.push(index as usize);
                (vertices.len() - 1) as u32
            })
        })
        .collect::<Vec<_>>();
    let skin = vertices
        .iter()
        .map(|&vertex| {
            armor
                .joint_indices
                .weighted_vertex(vertex, armor.joint_weights[vertex])
                .strongest_four()
        })
        .collect::<Vec<_>>();
    let native_joints = skin
        .iter()
        .map(|skin| skin.native_u16())
        .collect::<Result<Vec<_>, _>>()?;
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vertices
            .iter()
            .map(|&v| armor.positions[v])
            .collect::<Vec<_>>(),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        vertices
            .iter()
            .map(|&v| armor.normals[v])
            .collect::<Vec<_>>(),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vertices
            .iter()
            .map(|&v| armor.texcoords[v])
            .collect::<Vec<_>>(),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_JOINT_INDEX,
        VertexAttributeValues::Uint16x4(native_joints),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_JOINT_WEIGHT,
        skin.iter().map(|skin| skin.weights).collect::<Vec<_>>(),
    )
    .with_inserted_indices(Indices::U32(indices));

    Ok(meshes.add(mesh))
}

fn runtime_equipment_material(
    item: CatalogItem<'static>,
    materials: &mut Assets<StandardMaterial>,
) -> anyhow::Result<Handle<StandardMaterial>> {
    let material =
        adventuresim_character_creator::runtime_equipment::runtime_equipment_material(item)
            .with_context(|| format!("runtime armor material missing for {}", item.id))?;
    let appearance = adventuresim_character_creator::equipment_pbr(material);
    Ok(materials.add(
        adventuresim_character_creator::surface_projection::standard_material(appearance),
    ))
}

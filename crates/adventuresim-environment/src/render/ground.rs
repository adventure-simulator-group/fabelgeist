//! The terrain and the buildings, which a scene's first snapshot brings and
//! later ones leave as they are: ordinary meshes coloured by their vertices.

use bevy::asset::RenderAssetUsages;
use bevy::ecs::system::SystemParam;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use std::sync::Arc;

use fabelgeist_botany::viewer::sim::{Snapshot, StructureGeometry, TerrainGeometry};

/// Soil and grass are matte.
const TERRAIN_ROUGHNESS: f32 = 0.95;
/// Stone, plaster, tiles and timber are rough too, a little less.
const STRUCTURE_ROUGHNESS: f32 = 0.85;

/// The terrain and buildings a snapshot brings: only a scene's first.
pub(super) struct Ground {
    terrain: Option<Arc<TerrainGeometry>>,
    structures: Option<Arc<StructureGeometry>>,
}

impl Ground {
    #[cfg(feature = "prepared")]
    pub(super) fn prepared(frame: &crate::prepared::ForestFrame) -> Self {
        Self {
            terrain: frame.terrain.clone(),
            structures: frame.structures.clone(),
        }
    }

    pub(super) fn new(snapshot: &Snapshot) -> Self {
        Self {
            terrain: snapshot.terrain.clone(),
            structures: snapshot.structures.clone(),
        }
    }
}

/// The terrain or a scene's buildings: what a pointer can rest on.
#[derive(Component)]
pub struct GroundPart;

#[derive(SystemParam)]
pub(super) struct GroundAssets<'w, 's> {
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
    shown: Query<'w, 's, Entity, With<GroundPart>>,
}

impl GroundAssets<'_, '_> {
    /// Replace the ground when a snapshot brings it, starting a scene.
    pub(super) fn show(&mut self, commands: &mut Commands, ground: &Ground) {
        let Some(terrain) = &ground.terrain else {
            return;
        };
        for entity in &self.shown {
            commands.entity(entity).despawn();
        }
        let vertices = &terrain.vertices;
        self.spawn(
            commands,
            Surface {
                positions: vertices.iter().map(|vertex| vertex.position).collect(),
                normals: vertices.iter().map(|vertex| vertex.normal).collect(),
                colors: vertices.iter().map(|vertex| linear(vertex.color)).collect(),
                indices: terrain.indices.clone(),
            },
            TERRAIN_ROUGHNESS,
        );
        let Some(structures) = &ground.structures else {
            return;
        };
        if structures.indices.is_empty() {
            return;
        }
        let vertices = &structures.vertices;
        self.spawn(
            commands,
            Surface {
                positions: vertices.iter().map(|vertex| vertex.position).collect(),
                normals: vertices.iter().map(|vertex| vertex.normal).collect(),
                colors: vertices.iter().map(|vertex| linear(vertex.color)).collect(),
                indices: structures.indices.clone(),
            },
            STRUCTURE_ROUGHNESS,
        );
    }

    fn spawn(&mut self, commands: &mut Commands, surface: Surface, roughness: f32) {
        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, surface.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, surface.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, surface.colors)
        .with_inserted_indices(Indices::U32(surface.indices));
        commands.spawn((
            GroundPart,
            Mesh3d(self.meshes.add(mesh)),
            MeshMaterial3d(self.materials.add(StandardMaterial {
                perceptual_roughness: roughness,
                ..default()
            })),
        ));
    }
}

struct Surface {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

/// An sRGB vertex colour as Bevy's linear vertex colour.
fn linear(color: [f32; 3]) -> [f32; 4] {
    let [r, g, b] = color;
    LinearRgba::from(Color::srgb(r, g, b)).to_f32_array()
}

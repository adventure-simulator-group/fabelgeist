//! Independent occupied-building jobs prepare interiors and upload-ready meshes.
use super::*;
use crate::presentation::venues::{Place, PlaceId, PortraitId, select_building};
use adventuresim_building_generator::{
    BuildingLodLevel, BuildingLodMaterial, LodMesh, compile_building_lod,
    compile_static_building_detail, compile_static_building_lod,
    interior::{InteriorLayout, furnish},
};
use adventuresim_tactical_core::scene_input::{GeneratedBuildingRecipe, TacticalBuildingPlacement};
use bevy::{
    asset::RenderAssetUsages,
    math::Vec3,
    mesh::{Indices, Mesh, PrimitiveTopology, VertexAttributeValues},
};

#[derive(Deserialize)]
pub(super) struct VenueRequest {
    places: Vec<Place>,
    people: Vec<Visitor>,
}

#[derive(Deserialize)]
struct Visitor {
    id: PortraitId,
    place: PlaceId,
}

impl VenueRequest {
    pub(super) fn placements(&self, input: &TacticalSceneInput) -> Vec<TacticalBuildingPlacement> {
        let mut placements = input.buildings.clone();
        for place in &self.places {
            let operators = self
                .people
                .iter()
                .filter(|p| p.place == place.id)
                .map(|p| p.id.0);
            let Some(id) = select_building(input, place.kind, operators) else {
                continue;
            };
            if placements.iter().any(|p| p.id == id) {
                continue;
            }
            if let Some(distant) = input.distant_buildings.iter().find(|p| p.id == id) {
                placements.push((*distant).into());
            }
        }
        placements
    }
}

#[derive(Serialize, Deserialize)]
pub(super) struct PreparedVenue {
    pub recipe: GeneratedBuildingRecipe,
    pub interior: InteriorLayout,
    pub geometry: Option<VenueGeometry>,
}

#[derive(Serialize, Deserialize)]
pub(in crate::presentation) struct VenueGeometry {
    pub detail: Vec<PreparedBatch>,
    pub facade: Vec<PreparedBatch>,
    pub shell: Vec<PreparedBatch>,
}

#[derive(Serialize, Deserialize)]
pub(in crate::presentation) struct PreparedBatch {
    pub material: BuildingLodMaterial,
    #[serde(with = "super::packed")]
    positions: Vec<[f32; 3]>,
    #[serde(with = "super::packed")]
    normals: Vec<[f32; 3]>,
    #[serde(with = "super::packed")]
    uvs: Vec<[f32; 2]>,
    #[serde(with = "super::packed")]
    tangents: Vec<[f32; 4]>,
    #[serde(with = "super::packed")]
    indices: Vec<u32>,
}

impl PreparedBatch {
    fn new(batch: &LodMesh, origin: Vec3) -> Self {
        let mut mesh = super::super::recipe_mesh::recipe_mesh(batch, origin);
        let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.remove_attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            unreachable!()
        };
        let Some(VertexAttributeValues::Float32x3(normals)) =
            mesh.remove_attribute(Mesh::ATTRIBUTE_NORMAL)
        else {
            unreachable!()
        };
        let Some(VertexAttributeValues::Float32x2(uvs)) =
            mesh.remove_attribute(Mesh::ATTRIBUTE_UV_0)
        else {
            unreachable!()
        };
        let tangents = match mesh.remove_attribute(Mesh::ATTRIBUTE_TANGENT) {
            Some(VertexAttributeValues::Float32x4(values)) => values,
            None => Vec::new(),
            _ => unreachable!(),
        };
        Self {
            material: batch.material,
            positions,
            normals,
            uvs,
            tangents,
            indices: batch.indices.clone(),
        }
    }

    pub(in crate::presentation) fn into_mesh(self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        if !self.tangents.is_empty() {
            mesh.insert_attribute(Mesh::ATTRIBUTE_TANGENT, self.tangents);
        }
        mesh.insert_indices(Indices::U32(self.indices));
        mesh
    }
}

impl PreparedVenue {
    pub(super) fn generate(
        program: BuildingProgram,
        recipe: Option<GeneratedBuildingRecipe>,
    ) -> Result<Self, String> {
        let recipe = match recipe {
            Some(recipe) if recipe.program == program => recipe,
            Some(_) => return Err("venue recipe does not match its program".into()),
            None => GeneratedBuildingRecipe::generate(program).map_err(|e| e.to_string())?,
        };
        let origin = recipe
            .collision
            .bounds
            .centre()
            .map_err(|error| error.to_string())?
            .metres();
        let batches = |meshes: Vec<LodMesh>| {
            meshes
                .iter()
                .map(|batch| PreparedBatch::new(batch, origin))
                .collect()
        };
        let geometry = VenueGeometry {
            detail: batches(
                compile_static_building_detail(&recipe.plan)
                    .map_err(|error| error.to_string())?
                    .meshes,
            ),
            facade: batches(
                compile_static_building_lod(&recipe.plan, BuildingLodLevel::Facade)
                    .map_err(|error| error.to_string())?
                    .meshes,
            ),
            shell: batches(
                compile_building_lod(&recipe.plan, BuildingLodLevel::Shell)
                    .map_err(|error| error.to_string())?
                    .meshes,
            ),
        };
        let interior = furnish(&recipe.plan, &recipe.program).map_err(|e| e.to_string())?;
        Ok(Self {
            recipe,
            interior,
            geometry: Some(geometry),
        })
    }
}

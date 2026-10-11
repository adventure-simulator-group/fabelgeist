//! Complete canonical ground installation independent of actor scene observers.
use super::*;
use crate::presentation::{
    buildings::CityFrame,
    generation::PreparedCityProduct,
    ownership::PresentationOwner,
    terrain::{
        LANDFORM_PATCH_DEPTH_BIAS, TacticalTerrainMaterial, enable_cliff_surface, terrain_material,
    },
};
use bevy::ecs::system::SystemState;
use meshes::GroundMeshes;

mod meshes;

/// Retains the actual presented triangles for markers and path clipping.
pub(in crate::presentation) struct FocusedGround {
    pub support: streets::GroundSupport,
    pub terrain_materials: Vec<Handle<TacticalTerrainMaterial>>,
    pub paving_materials: Vec<Handle<CityGroundMaterial>>,
    /// Native mesh boundary: canonical city east/north half extents in metres.
    pub half_extent: Vec2,
}

struct GroundChunk {
    mesh: Mesh,
    origin: Vec3,
}

impl FocusedGround {
    pub(in crate::presentation) fn install(
        world: &mut World,
        product: &PreparedCityProduct,
        root: Entity,
        frame: CityFrame,
    ) -> Result<Self> {
        let mut state = SystemState::<(
            Commands,
            ResMut<Assets<Mesh>>,
            ResMut<Assets<Image>>,
            ResMut<Assets<TacticalTerrainMaterial>>,
            ResMut<Assets<TacticalVistaMaterial>>,
            Res<ProceduralTextureAssets>,
            Res<TacticalGraphicsSettings>,
            streets::CityGroundAssets,
        )>::new(world);
        let (
            mut commands,
            mut meshes,
            mut images,
            mut terrain_materials,
            mut vista_materials,
            textures,
            settings,
            mut paving,
        ) = state.get_mut(world)?;
        let input = product.document.input();
        let environment = input.environment_snapshot(input.digest()?);
        let ground = GroundMeshes::new(
            product,
            &environment,
            settings.config.rendering.vista.maximum_lods,
        )?;
        let mut material = terrain_material(
            &product.terrain,
            &environment,
            Some(&product.surface_ground),
            &textures,
            &mut images,
            &settings.config.grass,
        );
        material.extension.set_geographic_frame(frame);
        let mut terrain_handles = Vec::new();
        if let Some(landform) = ground.landform {
            let mut patch_material = material.clone();
            enable_cliff_surface(&mut patch_material, landform.surface);
            patch_material.base.depth_bias = LANDFORM_PATCH_DEPTH_BIAS;
            let handle = terrain_materials.add(patch_material);
            landform
                .chunk
                .spawn(&mut commands, root, &mut meshes, handle.clone());
            terrain_handles.push(handle);
        }
        let terrain_material = terrain_materials.add(material);
        ground
            .fine
            .spawn(&mut commands, root, &mut meshes, terrain_material.clone());
        terrain_handles.push(terrain_material);
        let vista_material = vista_materials.add(vista_material(
            environment.weather,
            grass_terminal_pigment(&environment),
        ));
        for chunk in ground.vista {
            chunk.spawn(&mut commands, root, &mut meshes, vista_material.clone());
        }
        let paving_materials = paving
            .spawn_focused(
                &mut commands,
                &product.ground,
                streets::FocusedPavingPlacement {
                    root,
                    frame,
                    weather: environment.weather,
                },
                &mut meshes,
                &mut images,
            )
            .map_err(|error| error.to_string())?;
        state.apply(world);
        Ok(Self {
            support: ground.support,
            terrain_materials: terrain_handles,
            paving_materials,
            half_extent: ground.half_extent,
        })
    }
}

impl GroundChunk {
    fn spawn<M: bevy::pbr::Material>(
        self,
        commands: &mut Commands,
        root: Entity,
        meshes: &mut Assets<Mesh>,
        material: Handle<M>,
    ) {
        let owner = PresentationOwner::RegionalMap;
        commands.spawn((
            Name::new("Map city canonical ground"),
            owner,
            owner.render_layers(),
            NotShadowCaster,
            Mesh3d(meshes.add(self.mesh)),
            MeshMaterial3d(material),
            Transform::from_translation(self.origin),
            ChildOf(root),
        ));
    }

    fn reflected(self) -> std::result::Result<Self, geographic_ground::GeographicGroundError> {
        Ok(Self {
            mesh: geographic_ground::reflected(self.mesh)?,
            ..self
        })
    }
}

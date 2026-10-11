//! Complete canonical ground installation independent of actor scene observers.
use super::*;
use crate::presentation::{
    buildings::CityFrame,
    generation::PreparedCityProduct,
    ownership::PresentationOwner,
    terrain::{TacticalTerrainMaterial, terrain_material, urban_playable_mesh},
};
use bevy::ecs::system::SystemState;

/// Retains the actual presented triangles for markers and path clipping.
pub(in crate::presentation) struct FocusedGround {
    pub support: streets::GroundSupport,
    pub terrain_material: Handle<TacticalTerrainMaterial>,
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
        let fine = urban_playable_mesh(&product.terrain, input.landform.as_ref());
        let mut support = streets::GroundSupport::default();
        support.add_mesh(&fine, Vec3::ZERO);
        let (chunks, half_extent) = GroundChunk::vista(
            product,
            &environment,
            settings.config.rendering.vista.maximum_lods,
        );
        for chunk in &chunks {
            support.add_mesh(&chunk.mesh, chunk.origin);
        }
        // Reflect every mesh before adding entities. A malformed canonical
        // upload leaves the displayed city untouched.
        let fine = geographic_ground::reflected(fine)?;
        let chunks = chunks
            .into_iter()
            .map(GroundChunk::reflected)
            .collect::<std::result::Result<Vec<_>, geographic_ground::GeographicGroundError>>()?;
        let mut material = terrain_material(
            &product.terrain,
            &environment,
            Some(&product.surface_ground),
            &textures,
            &mut images,
            &settings.config.grass,
        );
        material.extension.set_geographic_frame(frame);
        let terrain_material = terrain_materials.add(material);
        GroundChunk {
            mesh: fine,
            origin: Vec3::ZERO,
        }
        .spawn(&mut commands, root, &mut meshes, terrain_material.clone());
        let vista_material = vista_materials.add(vista_material(
            environment.weather,
            grass_terminal_pigment(&environment),
        ));
        for chunk in chunks {
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
            support,
            terrain_material,
            paving_materials,
            half_extent,
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

    fn vista(
        product: &PreparedCityProduct,
        environment: &SceneEnvironment,
        maximum_lods: usize,
    ) -> (Vec<Self>, Vec2) {
        let input = product.document.input();
        let lods = input
            .vista
            .lods
            .iter()
            .take(maximum_lods)
            .collect::<Vec<_>>();
        let mut inner = Vec2::new(product.terrain.width(), product.terrain.depth()) * 0.5;
        let mut chunks = Vec::new();
        for (index, lod) in lods.iter().copied().enumerate() {
            let origin = Vec3::new(
                lod.origin_east_metres as f32,
                0.0,
                lod.origin_north_metres as f32,
            );
            chunks.extend(
                vista_lod_meshes_with_morph(
                    lod,
                    inner,
                    lods.get(index + 1).copied(),
                    Some(&product.terrain),
                    (index == 0).then_some(environment),
                    environment.weather,
                    input.landform.map(|recipe| recipe.transition_collar()),
                )
                .into_iter()
                .map(|mesh| Self { mesh, origin }),
            );
            inner = Vec2::new(f32::from(lod.width - 1), f32::from(lod.depth - 1))
                * lod.spacing_metres
                * 0.5;
        }
        (chunks, inner)
    }
}

//! Sampled strategic scenery never becomes tactical terrain or ground support.
use super::*;
use adventuresim_tactical_core::{
    city_layout::grounding::SupportElevation,
    scene::{TerrainSupportError, TerrainSupportResult},
    scene_coordinates::ScenePlanPoint,
};
mod colors;

#[derive(Component, Clone)]
pub(crate) struct StrategicBackdrop {
    terrain: SceneTerrain,
    lods: Vec<VistaLod>,
    foreground: Vec<EnvironmentalSample>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum BackdropError {
    #[error("Strategic backdrop height samples are invalid")]
    Heightmap,
}

impl StrategicBackdrop {
    pub(crate) fn new(
        input: &TacticalSceneInput,
        maximum_lods: usize,
    ) -> Result<Self, BackdropError> {
        let terrain = SceneTerrain::from_heightmap(
            usize::from(input.playable.width),
            usize::from(input.playable.depth),
            input.playable.spacing_metres,
            input.playable.heights_metres.clone(),
        )
        .ok_or(BackdropError::Heightmap)?;
        if input.playable.environment.len() != input.playable.heights_metres.len() {
            return Err(BackdropError::Heightmap);
        }
        Ok(Self {
            terrain,
            lods: input
                .vista
                .lods
                .iter()
                .take(maximum_lods)
                .cloned()
                .collect(),
            foreground: input.playable.environment.clone(),
        })
    }

    pub(crate) fn weather_surface(&self) -> &SceneTerrain {
        &self.terrain
    }

    pub(crate) fn required_height_at(
        &self,
        point: ScenePlanPoint,
    ) -> TerrainSupportResult<SupportElevation> {
        self.height_at(point)
            .ok_or(TerrainSupportError::MissingSupport { point })
    }

    pub(crate) fn height_at(&self, point: ScenePlanPoint) -> Option<SupportElevation> {
        if let Some(height) = self.terrain.height_at(point) {
            return Some(height);
        }
        let mut inner = Vec2::new(self.terrain.width(), self.terrain.depth()) * 0.5;
        for (index, lod) in self.lods.iter().enumerate() {
            let height = adventuresim_tactical_core::vista_surface::sampled_vista_triangle_height(
                lod,
                self.lods.get(index + 1),
                (index == 0).then_some(&self.terrain),
                inner,
                point.metres(),
            );
            if height.is_some() {
                return height.and_then(SupportElevation::from_metres);
            }
            inner = Vec2::new(f32::from(lod.width - 1), f32::from(lod.depth - 1))
                * lod.spacing_metres
                * 0.5;
        }
        None
    }
}

pub(crate) fn spawn_strategic_backdrop(
    commands: &mut Commands,
    backdrop: StrategicBackdrop,
    environment: SceneEnvironment,
    root: Entity,
) {
    commands.spawn((environment.clone(), backdrop.clone(), ChildOf(root)));
    commands.queue(move |world: &mut World| {
        let material = world
            .resource_mut::<Assets<TacticalVistaMaterial>>()
            .add(vista_material(
                environment.weather,
                grass_terminal_pigment(&environment),
            ));
        let mut mesh = backdrop.terrain.mesh();
        mesh.remove_attribute(Mesh::ATTRIBUTE_UV_0);
        // A broad sampled surface supplies context; fine paving, vegetation and
        // fault geometry belong to the tactical renderer and focused map city.
        let foreground = backdrop
            .foreground
            .iter()
            .map(|sample| pigment::vista_sample_color(*sample, environment.weather).to_array())
            .collect::<Vec<_>>();
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, foreground.clone());
        spawn_mesh(world, root, mesh, material.clone(), Vec3::ZERO);
        let lods = backdrop.lods.iter().collect::<Vec<_>>();
        let mut inner = Vec2::new(backdrop.terrain.width(), backdrop.terrain.depth()) * 0.5;
        for (index, lod) in lods.iter().copied().enumerate() {
            for mut mesh in natural::sampled_vista_lod_meshes_with_morph(
                lod,
                inner,
                lods.get(index + 1).copied(),
                (index == 0).then_some(&backdrop.terrain),
                None,
                environment.weather,
            ) {
                if index == 0 {
                    backdrop.stitch_colors(&mut mesh, &foreground, lod.spacing_metres);
                }
                let origin = Vec3::new(
                    lod.origin_east_metres as f32,
                    0.0,
                    lod.origin_north_metres as f32,
                );
                spawn_mesh(world, root, mesh, material.clone(), origin);
            }
            inner = Vec2::new(f32::from(lod.width - 1), f32::from(lod.depth - 1))
                * lod.spacing_metres
                * 0.5;
        }
    });
}

fn spawn_mesh(
    world: &mut World,
    root: Entity,
    mesh: Mesh,
    material: Handle<TacticalVistaMaterial>,
    origin: Vec3,
) {
    let mesh = world.resource_mut::<Assets<Mesh>>().add(mesh);
    world.spawn((
        NotShadowCaster,
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(origin),
        ChildOf(root),
    ));
}

//! Cheap room-bounded indirect daylight, shared by architecture and moving PBR surfaces.
mod exposure;
mod field;
mod material;
#[cfg(test)]
mod tests;

use super::{PresentedCelestialLighting, TacticalGameplayCamera};
use adventuresim_building_generator::CELL_SIZE_METRES;
use bevy::{
    prelude::*,
    render::{render_resource::ShaderType, storage::ShaderBuffer},
};
pub(super) use field::InteriorField;
pub(crate) use material::{InteriorMaterial, InteriorMaterialSource};

const MAX_LIT_BUILDINGS: usize = 16;
const FIELD_DISTANCE_METRES: f32 = 100.0;
const DAYLIGHT_IRRADIANCE: f32 = 12_000.0;
const DAYLIGHT_START_ALTITUDE_DEGREES: f32 = -8.0;
const FULL_DAYLIGHT_ALTITUDE_DEGREES: f32 = 12.0;

#[derive(Clone, Copy, Default, Debug, PartialEq, ShaderType)]
struct GpuBuilding {
    local_from_world: Mat4,
    world_min: Vec4,
    world_max: Vec4,
    origin_and_cell: Vec4,
    dimensions_and_offset: UVec4,
    height: Vec4,
}

impl GpuBuilding {
    fn from_field(field: &InteriorField, transform: &GlobalTransform, offset: usize) -> Self {
        let local_size = field.dimensions.as_vec3()
            * Vec3::new(CELL_SIZE_METRES, field.storey_height, CELL_SIZE_METRES);
        let centre = transform.transform_point(field.origin + local_size * 0.5);
        let half = transform.affine().matrix3.abs() * (local_size * 0.5);
        Self {
            local_from_world: transform.to_matrix().inverse(),
            world_min: (centre - half).extend(0.0),
            world_max: (centre + half).extend(0.0),
            origin_and_cell: field.origin.extend(CELL_SIZE_METRES),
            dimensions_and_offset: field.dimensions.extend(offset as u32),
            height: Vec4::new(field.storey_height, 0.0, 0.0, 0.0),
        }
    }
}

#[derive(Clone, Debug, PartialEq, ShaderType)]
struct GpuField {
    daylight: Vec4,
    counts: UVec4,
    buildings: [GpuBuilding; MAX_LIT_BUILDINGS],
    #[shader(size(runtime))]
    samples: Vec<field::LightSample>,
}

impl Default for GpuField {
    fn default() -> Self {
        Self {
            daylight: Vec4::ZERO,
            counts: UVec4::ZERO,
            buildings: [GpuBuilding::default(); MAX_LIT_BUILDINGS],
            samples: vec![field::LightSample::default()],
        }
    }
}

#[derive(Resource)]
pub(crate) struct InteriorLightingGpu {
    pub(crate) buffer: Handle<ShaderBuffer>,
    data: GpuField,
    selection: Vec<(Entity, Mat4)>,
    _shader: Handle<Shader>,
}

pub(super) struct InteriorLightingPlugin;

impl Plugin for InteriorLightingPlugin {
    fn build(&self, app: &mut App) {
        let data = GpuField::default();
        let buffer = app
            .world_mut()
            .resource_mut::<Assets<ShaderBuffer>>()
            .add(ShaderBuffer::from(data.clone()));
        let shader = app
            .world_mut()
            .resource_mut::<Assets<Shader>>()
            .add(Shader::from_wgsl(
                include_str!("../../../../../assets/shaders/tactical_interior_lighting.wgsl"),
                "shaders/tactical_interior_lighting.wgsl",
            ));
        app.insert_resource(InteriorLightingGpu {
            buffer,
            data,
            selection: Vec::new(),
            _shader: shader,
        })
        .init_resource::<material::InteriorMaterials>()
        .init_resource::<exposure::InteriorExposure>()
        .add_plugins(MaterialPlugin::<InteriorMaterial>::default())
        .add_systems(
            PostUpdate,
            (
                upload_field,
                exposure::adapt_exposure,
                material::prepare_materials,
            )
                .after(bevy::transform::TransformSystems::Propagate),
        );
    }
}

fn daylight_response(celestial: &PresentedCelestialLighting) -> Vec4 {
    let Some(snapshot) = &celestial.snapshot else {
        return Vec4::ZERO;
    };
    let day = ((snapshot.sun_altitude_degrees - DAYLIGHT_START_ALTITUDE_DEGREES)
        / (FULL_DAYLIGHT_ALTITUDE_DEGREES - DAYLIGHT_START_ALTITUDE_DEGREES))
        .clamp(0.0, 1.0);
    let day = day * day * (3.0 - 2.0 * day);
    (snapshot.ambient_color * DAYLIGHT_IRRADIANCE * day * snapshot.weather_transmission).extend(day)
}

fn upload_field(
    camera: Option<Single<&GlobalTransform, With<TacticalGameplayCamera>>>,
    fields: Query<(Entity, Ref<InteriorField>, &GlobalTransform)>,
    celestial: Res<PresentedCelestialLighting>,
    mut gpu: ResMut<InteriorLightingGpu>,
    mut buffers: ResMut<Assets<ShaderBuffer>>,
) {
    let mut data = GpuField {
        daylight: daylight_response(&celestial),
        ..default()
    };
    if let Some(camera) = camera {
        let camera_position = camera.translation();
        let mut nearby: Vec<_> = fields
            .iter()
            .filter_map(|(entity, field, transform)| {
                let local = transform
                    .affine()
                    .inverse()
                    .transform_point3(camera_position)
                    - field.origin;
                let size = field.dimensions.as_vec3()
                    * Vec3::new(CELL_SIZE_METRES, field.storey_height, CELL_SIZE_METRES);
                let distance = (local - local.clamp(Vec3::ZERO, size)).length_squared();
                (distance <= FIELD_DISTANCE_METRES * FIELD_DISTANCE_METRES)
                    .then_some((distance, entity, field, transform))
            })
            .collect();
        nearby.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        nearby.truncate(MAX_LIT_BUILDINGS);
        // Stable ordering avoids re-uploading every sample when the camera moves
        // between two buildings that are already resident.
        nearby.sort_by_key(|entry| entry.1);
        let selection: Vec<_> = nearby
            .iter()
            .map(|entry| (entry.1, entry.3.to_matrix()))
            .collect();
        if selection == gpu.selection
            && data.daylight == gpu.data.daylight
            && !nearby.iter().any(|entry| entry.2.is_changed())
        {
            return;
        }
        gpu.selection = selection;
        for (_, _, field, transform) in nearby {
            let index = data.counts.x as usize;
            data.buildings[index] = GpuBuilding::from_field(&field, transform, data.samples.len());
            data.samples.extend_from_slice(&field.samples);
            data.counts.x += 1;
        }
    } else {
        gpu.selection.clear();
    }
    if data != gpu.data {
        if let Some(mut buffer) = buffers.get_mut(&gpu.buffer) {
            buffer.set_data(data.clone());
        }
        gpu.data = data;
    }
}

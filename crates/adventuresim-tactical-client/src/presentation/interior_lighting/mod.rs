//! Cheap room-bounded indirect daylight, shared by architecture and moving PBR surfaces.
mod exposure;
pub(crate) use exposure::{FixedViewExposure, InteriorExposure};
mod field;
mod material;
mod shutters;
#[cfg(test)]
mod tests;

use super::PresentedCelestialLighting;
use adventuresim_building_generator::CELL_SIZE_METRES;
use bevy::{
    prelude::*,
    render::{render_resource::ShaderType, storage::ShaderBuffer},
};
pub(crate) use field::InteriorField;
pub(crate) use material::{InteriorMaterial, InteriorMaterialSource};

const MAX_LIT_BUILDINGS: usize = 16;
// Two MiB of samples keep the GPU allocation (and material bindings) stable
// as nearby buildings enter and leave the daylight field.
const MAX_GPU_SAMPLES: usize = 65_536;
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
    outdoor_irradiance: Vec4,
    counts: UVec4,
    buildings: [GpuBuilding; MAX_LIT_BUILDINGS],
    #[shader(size(runtime))]
    samples: Vec<field::LightSample>,
}

impl Default for GpuField {
    fn default() -> Self {
        Self {
            daylight: Vec4::ZERO,
            outdoor_irradiance: Vec4::ZERO,
            counts: UVec4::ZERO,
            buildings: [GpuBuilding::default(); MAX_LIT_BUILDINGS],
            samples: vec![field::LightSample::default()],
        }
    }
}

impl GpuField {
    fn for_upload(&self) -> Self {
        let mut padded = self.clone();
        padded
            .samples
            .resize(MAX_GPU_SAMPLES, field::LightSample::default());
        padded
    }
}

#[derive(Resource)]
pub(crate) struct InteriorLightingGpu {
    pub(crate) buffer: Handle<ShaderBuffer>,
    data: GpuField,
    selection: Vec<(Entity, Mat4)>,
    _shader: Handle<Shader>,
}

impl InteriorLightingGpu {
    pub(crate) fn daylight(&self) -> Vec4 {
        self.data.daylight
    }

    pub(crate) fn sample_at(&self, position: Vec3) -> Vec3 {
        for building in self.data.buildings.iter().take(self.data.counts.x as usize) {
            let local = building.local_from_world.transform_point3(position);
            let cell = (local - building.origin_and_cell.truncate())
                / Vec3::new(
                    building.origin_and_cell.w,
                    building.height.x,
                    building.origin_and_cell.w,
                );
            let dimensions = building.dimensions_and_offset;
            if cell.cmplt(Vec3::ZERO).any() || cell.cmpge(dimensions.truncate().as_vec3()).any() {
                continue;
            }
            let cell = cell.floor().as_uvec3();
            let index = dimensions.w + (cell.y * dimensions.z + cell.z) * dimensions.x + cell.x;
            let sample = self.data.samples[index as usize];
            if sample.positive.w > 0.0 {
                return sample.positive.truncate() + sample.negative.truncate();
            }
        }
        Vec3::ZERO
    }
}

pub(super) struct InteriorLightingPlugin;

impl Plugin for InteriorLightingPlugin {
    fn build(&self, app: &mut App) {
        let data = GpuField::default();
        let buffer = app
            .world_mut()
            .resource_mut::<Assets<ShaderBuffer>>()
            .add(ShaderBuffer::from(data.for_upload()));
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
                shutters::update_shutter_light.before(upload_field),
                upload_field,
                exposure::adapt_exposure,
                exposure::expose_fixed_views,
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
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    fields: Query<(Entity, Ref<InteriorField>, &GlobalTransform)>,
    celestial: Res<PresentedCelestialLighting>,
    mut gpu: ResMut<InteriorLightingGpu>,
    mut buffers: ResMut<Assets<ShaderBuffer>>,
) {
    let daylight = daylight_response(&celestial);
    let mut data = GpuField {
        daylight,
        outdoor_irradiance: daylight
            * (bevy::light::light_consts::lux::RAW_SUNLIGHT / DAYLIGHT_IRRADIANCE),
        ..default()
    };
    let positions = cameras
        .iter()
        .filter(|(_, camera)| camera.is_active)
        .map(|(transform, _)| transform.translation())
        .collect::<Vec<_>>();
    if !positions.is_empty() {
        let mut nearby: Vec<_> = fields
            .iter()
            .filter_map(|(entity, field, transform)| {
                let size = field.dimensions.as_vec3()
                    * Vec3::new(CELL_SIZE_METRES, field.storey_height, CELL_SIZE_METRES);
                let inverse = transform.affine().inverse();
                let distance = positions
                    .iter()
                    .map(|position| {
                        let local = inverse.transform_point3(*position) - field.origin;
                        (local - local.clamp(Vec3::ZERO, size)).length_squared()
                    })
                    .fold(f32::INFINITY, f32::min);
                (distance <= FIELD_DISTANCE_METRES * FIELD_DISTANCE_METRES)
                    .then_some((distance, entity, field, transform))
            })
            .collect();
        nearby.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        nearby.truncate(MAX_LIT_BUILDINGS);
        let mut sample_count = data.samples.len();
        nearby.retain(|entry| {
            let fits = sample_count + entry.2.samples.len() <= MAX_GPU_SAMPLES;
            if fits {
                sample_count += entry.2.samples.len();
            }
            fits
        });
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
            buffer.set_data(data.for_upload());
        }
        gpu.data = data;
    }
}

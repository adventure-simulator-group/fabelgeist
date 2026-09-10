//! Geometry-derived daylight samples. Each occupied cell belongs to exactly one room.
use adventuresim_building_generator::{BuildingPlan, CELL_SIZE_METRES, OpeningKind};
use bevy::{prelude::*, render::render_resource::ShaderType};

const WINDOW_TRANSMISSION: f32 = 0.65;
const BOUNCE_FRACTION: f32 = 0.28;
const OPENING_DISTANCE_SOFTENING_SQUARED: f32 = 2.25;
const SAMPLE_HEIGHT_FRACTION: f32 = 0.5;
const MAX_SAMPLE_IRRADIANCE: f32 = 1.0;

/// The six ambient-cube lobes, with a room identifier in positive.w (zero is outside).
#[derive(Clone, Copy, Debug, Default, PartialEq, ShaderType)]
pub(super) struct LightSample {
    pub positive: Vec4,
    pub negative: Vec4,
}

#[derive(Component, Clone)]
pub(in crate::presentation) struct InteriorField {
    pub origin: Vec3,
    pub dimensions: UVec3,
    pub storey_height: f32,
    pub(super) samples: Vec<LightSample>,
}

impl InteriorField {
    pub fn from_plan(plan: &BuildingPlan, local_origin: Vec3) -> Self {
        let cells: Vec<_> = plan
            .storeys
            .iter()
            .flat_map(|s| &s.rooms)
            .flat_map(|r| &r.cells)
            .collect();
        let min_x = cells.iter().map(|c| c.x).min().unwrap_or(0);
        let min_z = cells.iter().map(|c| c.z).min().unwrap_or(0);
        let width = cells.iter().map(|c| c.x - min_x + 1).max().unwrap_or(1) as u32;
        let depth = cells.iter().map(|c| c.z - min_z + 1).max().unwrap_or(1) as u32;
        let levels = plan
            .storeys
            .iter()
            .map(|s| u32::from(s.level) + 1)
            .max()
            .unwrap_or(1);
        let dimensions = UVec3::new(width, levels, depth);
        let mut samples = vec![LightSample::default(); (width * depth * levels) as usize];
        for storey in &plan.storeys {
            for room in &storey.rooms {
                for cell in &room.cells {
                    let centre = cell.centre();
                    let point = Vec3::new(
                        centre.x,
                        plan.storey_height_metres * SAMPLE_HEIGHT_FRACTION,
                        centre.y,
                    );
                    let mut positive = Vec3::ZERO;
                    let mut negative = Vec3::ZERO;
                    for opening in &storey.openings {
                        let wall = storey.walls[opening.wall];
                        // Opaque doors contribute no daylight. Glass keeps transmitting when closed.
                        if !wall.exterior()
                            || wall.inside_room != room.id
                            || matches!(opening.kind, OpeningKind::Door)
                        {
                            continue;
                        }
                        let centre = wall.centre();
                        let source = Vec3::new(
                            centre.x,
                            opening.sill_metres + opening.height_metres * 0.5,
                            centre.y,
                        );
                        let delta = source - point;
                        let energy =
                            opening.width_metres * opening.height_metres * WINDOW_TRANSMISSION
                                / (OPENING_DISTANCE_SOFTENING_SQUARED + delta.length_squared());
                        let direction = delta.normalize_or_zero();
                        let bounce = Vec3::splat(energy * BOUNCE_FRACTION);
                        positive += bounce + direction.max(Vec3::ZERO) * energy;
                        negative += bounce + (-direction).max(Vec3::ZERO) * energy;
                    }
                    let index = ((u32::from(storey.level) * depth + (cell.z - min_z) as u32)
                        * width
                        + (cell.x - min_x) as u32) as usize;
                    samples[index] = LightSample {
                        positive: positive
                            .min(Vec3::splat(MAX_SAMPLE_IRRADIANCE))
                            .extend(f32::from(room.id) + 1.0),
                        negative: negative.min(Vec3::splat(MAX_SAMPLE_IRRADIANCE)).extend(0.0),
                    };
                }
            }
        }
        Self {
            origin: Vec3::new(
                f32::from(min_x) * CELL_SIZE_METRES,
                0.0,
                f32::from(min_z) * CELL_SIZE_METRES,
            ) - local_origin,
            dimensions,
            storey_height: plan.storey_height_metres,
            samples,
        }
    }

    pub(super) fn sample(&self, local_position: Vec3) -> Option<LightSample> {
        let position = (local_position - self.origin)
            / Vec3::new(CELL_SIZE_METRES, self.storey_height, CELL_SIZE_METRES);
        if position.cmplt(Vec3::ZERO).any() || position.cmpge(self.dimensions.as_vec3()).any() {
            return None;
        }
        let cell = position.floor().as_uvec3();
        let index = ((cell.y * self.dimensions.z + cell.z) * self.dimensions.x + cell.x) as usize;
        let sample = self.samples[index];
        (sample.positive.w > 0.0).then_some(sample)
    }
}

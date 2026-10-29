//! Shared vista geometry policy for terrain presentation and supported scenery.
use crate::{
    city_layout::grounding::SupportElevation,
    scene::SceneTerrain,
    scene_coordinates::{PlanDisplacement, Scene, ScenePlanPoint},
    scene_input::VistaLod,
};
use adventuresim_building_generator::spatial_geometry::{
    Displacement, GeometryResult, PositiveLength,
};
use bevy::math::{FloatExt, UVec2, Vec2, Vec3};

/// One presented vertex and its east/north finite-difference edges. Consumers
/// retain their normalization policy when producing a support or mesh normal.
pub struct VistaVertexGeometry {
    elevation: SupportElevation,
    east_tangent: Displacement<Scene>,
    north_tangent: Displacement<Scene>,
}

impl VistaVertexGeometry {
    pub fn sample(
        lod: &VistaLod,
        coarser: Option<&VistaLod>,
        terrain: Option<&SceneTerrain>,
        local: PlanDisplacement,
        playable_half_extent: Vec2,
        normal_sample_distance: PositiveLength,
    ) -> Option<Self> {
        let height = native_presented_vista_vertex_height(
            lod,
            coarser,
            terrain,
            local,
            playable_half_extent,
        )?;
        let delta = normal_sample_distance.metres();
        let at = |offset: Vec2| {
            PlanDisplacement::from_metres(local.metres() + offset)
                .and_then(|probe| {
                    native_presented_vista_vertex_height(
                        lod,
                        coarser,
                        terrain,
                        probe,
                        playable_half_extent,
                    )
                })
                .unwrap_or(height)
        };
        Some(Self {
            elevation: SupportElevation::from_metres(height)?,
            east_tangent: Displacement::from_metres(Vec3::new(
                delta * 2.0,
                at(Vec2::X * delta) - at(-Vec2::X * delta),
                0.0,
            ))
            .ok()?,
            north_tangent: Displacement::from_metres(Vec3::new(
                0.0,
                at(Vec2::Y * delta) - at(-Vec2::Y * delta),
                delta * 2.0,
            ))
            .ok()?,
        })
    }

    pub fn elevation(&self) -> SupportElevation {
        self.elevation
    }

    pub fn east_tangent(&self) -> Displacement<Scene> {
        self.east_tangent
    }

    pub fn north_tangent(&self) -> Displacement<Scene> {
        self.north_tangent
    }
}

impl VistaLod {
    /// Structural admission for the native row-major height interpolation.
    /// Environmental columns have their separate mesh/pigment admission.
    pub fn has_height_grid(&self) -> bool {
        self.width >= 2
            && self.depth >= 2
            && self.heights_metres.len() == usize::from(self.width) * usize::from(self.depth)
            && self.spacing_metres.is_finite()
            && self.spacing_metres > 0.0
            && (self.origin_east_metres as f32).is_finite()
            && (self.origin_north_metres as f32).is_finite()
    }

    pub fn scene_point(&self, local: PlanDisplacement) -> GeometryResult<ScenePlanPoint> {
        ScenePlanPoint::try_from(
            local.metres()
                + Vec2::new(
                    self.origin_east_metres as f32,
                    self.origin_north_metres as f32,
                ),
        )
    }

    pub fn local_offset(&self, point: ScenePlanPoint) -> GeometryResult<PlanDisplacement> {
        PlanDisplacement::try_from(
            point.metres()
                - Vec2::new(
                    self.origin_east_metres as f32,
                    self.origin_north_metres as f32,
                ),
        )
    }
}

pub fn presented_vista_vertex_height(
    lod: &VistaLod,
    coarser_lod: Option<&VistaLod>,
    playable_terrain: Option<&SceneTerrain>,
    local: PlanDisplacement,
    playable_half_extent: Vec2,
) -> Option<SupportElevation> {
    SupportElevation::from_metres(native_presented_vista_vertex_height(
        lod,
        coarser_lod,
        playable_terrain,
        local,
        playable_half_extent,
    )?)
}

pub fn subdivide_playable_boundary_rectangle(
    rectangle: [f32; 4],
    playable_half_extent: Vec2,
    terrain: Option<&SceneTerrain>,
) -> Vec<[f32; 4]> {
    let Some(terrain) = terrain else {
        return vec![rectangle];
    };
    let [minimum_x, maximum_x, minimum_z, maximum_z] = rectangle;
    let epsilon = terrain.grid_scale() * 0.01;
    if (minimum_x - playable_half_extent.x).abs() <= epsilon
        || (maximum_x + playable_half_extent.x).abs() <= epsilon
    {
        return split_rectangle_axis(
            rectangle,
            1,
            terrain.depth() * -0.5,
            terrain.grid_scale(),
            terrain.grid_depth(),
        );
    }
    if (minimum_z - playable_half_extent.y).abs() <= epsilon
        || (maximum_z + playable_half_extent.y).abs() <= epsilon
    {
        return split_rectangle_axis(
            rectangle,
            0,
            terrain.width() * -0.5,
            terrain.grid_scale(),
            terrain.grid_width(),
        );
    }
    vec![rectangle]
}

pub fn split_rectangle_axis(
    rectangle: [f32; 4],
    axis: usize,
    terrain_minimum: f32,
    spacing: f32,
    sample_count: usize,
) -> Vec<[f32; 4]> {
    let (minimum, maximum) = if axis == 0 {
        (rectangle[0], rectangle[1])
    } else {
        (rectangle[2], rectangle[3])
    };
    let mut boundaries = vec![minimum, maximum];
    boundaries.extend((0..sample_count).filter_map(|index| {
        let coordinate = terrain_minimum + index as f32 * spacing;
        (coordinate > minimum && coordinate < maximum).then_some(coordinate)
    }));
    boundaries.sort_by(f32::total_cmp);
    boundaries.dedup_by(|left, right| (*left - *right).abs() < spacing * 0.001);
    boundaries
        .windows(2)
        .map(|interval| {
            let mut split = rectangle;
            if axis == 0 {
                split[0] = interval[0];
                split[1] = interval[1];
            } else {
                split[2] = interval[0];
                split[3] = interval[1];
            }
            split
        })
        .collect()
}

pub fn stitch_vista_height_to_playable_edge(
    terrain: &SceneTerrain,
    local: PlanDisplacement,
    playable_half_extent: Vec2,
    transition_width: f32,
    vista_height: SupportElevation,
) -> Option<SupportElevation> {
    SupportElevation::from_metres(native_stitch_vista_height_to_playable_edge(
        terrain,
        local,
        playable_half_extent,
        transition_width,
        vista_height.metres(),
    ))
}

pub fn cell_rectangles_outside_inner_rectangle(
    minimum: Vec2,
    maximum: Vec2,
    inner_half_extent: Vec2,
) -> Vec<[f32; 4]> {
    if inner_half_extent.x <= 0.0
        || inner_half_extent.y <= 0.0
        || maximum.x <= -inner_half_extent.x
        || minimum.x >= inner_half_extent.x
        || maximum.y <= -inner_half_extent.y
        || minimum.y >= inner_half_extent.y
    {
        return vec![[minimum.x, maximum.x, minimum.y, maximum.y]];
    }
    if minimum.x >= -inner_half_extent.x
        && maximum.x <= inner_half_extent.x
        && minimum.y >= -inner_half_extent.y
        && maximum.y <= inner_half_extent.y
    {
        return Vec::new();
    }

    let mut rectangles = Vec::with_capacity(4);
    if minimum.x < -inner_half_extent.x {
        rectangles.push([
            minimum.x,
            maximum.x.min(-inner_half_extent.x),
            minimum.y,
            maximum.y,
        ]);
    }
    if maximum.x > inner_half_extent.x {
        rectangles.push([
            minimum.x.max(inner_half_extent.x),
            maximum.x,
            minimum.y,
            maximum.y,
        ]);
    }
    let middle_minimum_x = minimum.x.max(-inner_half_extent.x);
    let middle_maximum_x = maximum.x.min(inner_half_extent.x);
    if middle_minimum_x < middle_maximum_x {
        if minimum.y < -inner_half_extent.y {
            rectangles.push([
                middle_minimum_x,
                middle_maximum_x,
                minimum.y,
                maximum.y.min(-inner_half_extent.y),
            ]);
        }
        if maximum.y > inner_half_extent.y {
            rectangles.push([
                middle_minimum_x,
                middle_maximum_x,
                minimum.y.max(inner_half_extent.y),
                maximum.y,
            ]);
        }
    }
    rectangles
}

pub fn presented_height_at(
    lod: &VistaLod,
    world: ScenePlanPoint,
    coarser_lod: Option<&VistaLod>,
) -> Option<SupportElevation> {
    SupportElevation::from_metres(native_presented_height_at(lod, world, coarser_lod)?)
}

pub fn lod_transition_weight(lod: &VistaLod, coarser: &VistaLod, world: ScenePlanPoint) -> f32 {
    let world = world.metres();
    let center = Vec2::new(
        lod.origin_east_metres as f32,
        lod.origin_north_metres as f32,
    );
    let half_extent = f32::from(lod.width.saturating_sub(1)) * lod.spacing_metres * 0.5;
    // Begin morphing one coarse sample before the boundary. A one-fine-cell
    // band still exposes the square footprint whenever adjacent LOD spacing
    // grows rapidly (50 m -> 250 m -> 1 km).
    let transition_width = coarser
        .spacing_metres
        .min(half_extent)
        .max(lod.spacing_metres);
    let radius = (world - center).abs().max_element();
    ((radius - (half_extent - transition_width)) / transition_width).clamp(0.0, 1.0)
}

pub fn sample_vista_height(lod: &VistaLod, world: ScenePlanPoint) -> Option<SupportElevation> {
    SupportElevation::from_metres(native_sample_vista_height(lod, world)?)
}

/// Height of the actual clipped vista triangle, including the playable seam.
/// Uses the same rectangle splits and vertex policy as the production mesh.
pub fn vista_triangle_height(
    lod: &VistaLod,
    coarser: Option<&VistaLod>,
    terrain: &SceneTerrain,
    point: ScenePlanPoint,
) -> Option<SupportElevation> {
    sampled_vista_triangle_height(
        lod,
        coarser,
        Some(terrain),
        Vec2::new(terrain.width(), terrain.depth()) * 0.5,
        point,
    )
}

/// Sample the rendered ring in canonical east/north metres. The inner rectangle
/// and optional playable seam are the same inputs used by sampled ring meshing.
pub fn sampled_vista_triangle_height(
    lod: &VistaLod,
    coarser: Option<&VistaLod>,
    terrain: Option<&SceneTerrain>,
    inner_half_extent: Vec2,
    point: ScenePlanPoint,
) -> Option<SupportElevation> {
    if !lod.has_height_grid() {
        return None;
    }
    let local = point.metres()
        - Vec2::new(
            lod.origin_east_metres as f32,
            lod.origin_north_metres as f32,
        );
    let half = inner_half_extent;
    let size = Vec2::new(f32::from(lod.width - 1), f32::from(lod.depth - 1));
    let cell = (local / lod.spacing_metres + size * 0.5).floor();
    if cell.cmplt(Vec2::ZERO).any() || cell.cmpge(size).any() {
        return None;
    }
    let minimum = (cell - size * 0.5) * lod.spacing_metres;
    let maximum = minimum + Vec2::splat(lod.spacing_metres);
    for rectangle in cell_rectangles_outside_inner_rectangle(minimum, maximum, half) {
        for [x0, x1, z0, z1] in subdivide_playable_boundary_rectangle(rectangle, half, terrain) {
            if local.x < x0 || local.x > x1 || local.y < z0 || local.y > z1 {
                continue;
            }
            let vertex = |x, z| {
                native_presented_vista_vertex_height(
                    lod,
                    coarser,
                    terrain,
                    PlanDisplacement::from_metres(Vec2::new(x, z))?,
                    half,
                )
            };
            let a = vertex(x0, z0)?;
            let b = vertex(x1, z0)?;
            let c = vertex(x1, z1)?;
            let d = vertex(x0, z1)?;
            let u = (local.x - x0) / (x1 - x0);
            let v = (local.y - z0) / (z1 - z0);
            return SupportElevation::from_metres(if u >= v {
                a * (1.0 - u) + b * (u - v) + c * v
            } else {
                a * (1.0 - v) + d * (v - u) + c * u
            });
        }
    }
    None
}

// Bilinear, LOD, seam and triangle interpolation are one native calculation.
// Intermediate f32 samples stay unadmitted so a nonfinite coarse sample cannot
// become "absent" and select the valid-own fallback before the completed result.
fn native_presented_vista_vertex_height(
    lod: &VistaLod,
    coarser_lod: Option<&VistaLod>,
    playable_terrain: Option<&SceneTerrain>,
    local: PlanDisplacement,
    playable_half_extent: Vec2,
) -> Option<f32> {
    let world = lod.scene_point(local).ok()?;
    let vista_height = native_presented_height_at(lod, world, coarser_lod)?;
    Some(playable_terrain.map_or(vista_height, |terrain| {
        native_stitch_vista_height_to_playable_edge(
            terrain,
            local,
            playable_half_extent,
            lod.spacing_metres,
            vista_height,
        )
    }))
}

fn native_stitch_vista_height_to_playable_edge(
    terrain: &SceneTerrain,
    local: PlanDisplacement,
    playable_half_extent: Vec2,
    transition_width: f32,
    vista_height: f32,
) -> f32 {
    let local = local.metres();
    let boundary = local.clamp(-playable_half_extent, playable_half_extent);
    let Some(playable_height) =
        ScenePlanPoint::from_metres(boundary).and_then(|point| terrain.height_at(point))
    else {
        return vista_height;
    };
    let outside_distance = (local.abs() - playable_half_extent)
        .max(Vec2::ZERO)
        .max_element();
    let vista_weight = if transition_width > 0.0 {
        (outside_distance / transition_width).clamp(0.0, 1.0)
    } else {
        f32::from(outside_distance > 0.0)
    };
    playable_height.metres().lerp(vista_height, vista_weight)
}

fn native_presented_height_at(
    lod: &VistaLod,
    world: ScenePlanPoint,
    coarser_lod: Option<&VistaLod>,
) -> Option<f32> {
    let own = native_sample_vista_height(lod, world)?;
    let Some(coarser) = coarser_lod else {
        return Some(own);
    };
    if !coarser.has_height_grid() {
        return None;
    }
    let weight = lod_transition_weight(lod, coarser, world);
    if weight <= 0.0 {
        return Some(own);
    }
    Some(
        native_sample_vista_height(coarser, world)
            .map(|height| own.lerp(height, weight))
            .unwrap_or(own),
    )
}

fn native_sample_vista_height(lod: &VistaLod, world: ScenePlanPoint) -> Option<f32> {
    if !lod.has_height_grid() {
        return None;
    }
    let world = world.metres();
    let width = usize::from(lod.width);
    let depth = usize::from(lod.depth);
    let local = world
        - Vec2::new(
            lod.origin_east_metres as f32,
            lod.origin_north_metres as f32,
        );
    let coordinate =
        local / lod.spacing_metres + Vec2::new((width - 1) as f32 * 0.5, (depth - 1) as f32 * 0.5);
    if coordinate.x < 0.0
        || coordinate.y < 0.0
        || coordinate.x > (width - 1) as f32
        || coordinate.y > (depth - 1) as f32
    {
        return None;
    }
    let lower = coordinate.floor().as_uvec2();
    let upper = (lower + UVec2::ONE).min(UVec2::new(width as u32 - 1, depth as u32 - 1));
    let fraction = coordinate.fract();
    let at = |x: u32, z: u32| lod.heights_metres[z as usize * width + x as usize];
    let near = at(lower.x, lower.y).lerp(at(upper.x, lower.y), fraction.x);
    let far = at(lower.x, upper.y).lerp(at(upper.x, upper.y), fraction.x);
    Some(near.lerp(far, fraction.y))
}

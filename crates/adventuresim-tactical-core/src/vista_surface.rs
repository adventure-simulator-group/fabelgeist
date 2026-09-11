//! Shared vista geometry policy for terrain presentation and supported scenery.
use crate::{scene::SceneTerrain, scene_input::VistaLod};
use bevy::math::{FloatExt, UVec2, Vec2};

pub fn presented_vista_vertex_height(
    lod: &VistaLod,
    coarser_lod: Option<&VistaLod>,
    playable_terrain: Option<&SceneTerrain>,
    local: Vec2,
    playable_half_extent: Vec2,
) -> Option<f32> {
    let world = local
        + Vec2::new(
            lod.origin_east_metres as f32,
            lod.origin_north_metres as f32,
        );
    let vista_height = presented_height_at(lod, world, coarser_lod)?;
    Some(playable_terrain.map_or(vista_height, |terrain| {
        stitch_vista_height_to_playable_edge(
            terrain,
            local,
            playable_half_extent,
            lod.spacing_metres,
            vista_height,
        )
    }))
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
    local: Vec2,
    playable_half_extent: Vec2,
    transition_width: f32,
    vista_height: f32,
) -> f32 {
    let boundary = local.clamp(-playable_half_extent, playable_half_extent);
    let Some(playable_height) = terrain.height_at(boundary) else {
        return vista_height;
    };
    let outside_distance = (local.abs() - playable_half_extent)
        .max(Vec2::ZERO)
        .max_element();
    let vista_weight = (outside_distance / transition_width.max(f32::EPSILON)).clamp(0.0, 1.0);
    playable_height.lerp(vista_height, vista_weight)
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
    world: Vec2,
    coarser_lod: Option<&VistaLod>,
) -> Option<f32> {
    let own = sample_vista_height(lod, world)?;
    let Some(coarser) = coarser_lod else {
        return Some(own);
    };
    let weight = lod_transition_weight(lod, coarser, world);
    if weight <= 0.0 {
        return Some(own);
    }
    Some(
        sample_vista_height(coarser, world)
            .map(|height| own.lerp(height, weight))
            .unwrap_or(own),
    )
}

pub fn lod_transition_weight(lod: &VistaLod, coarser: &VistaLod, world: Vec2) -> f32 {
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

pub fn sample_vista_height(lod: &VistaLod, world: Vec2) -> Option<f32> {
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

/// Height of the actual clipped vista triangle, including the playable seam.
/// Uses the same rectangle splits and vertex policy as the production mesh.
pub fn vista_triangle_height(
    lod: &VistaLod,
    coarser: Option<&VistaLod>,
    terrain: &SceneTerrain,
    point: Vec2,
) -> Option<f32> {
    let local = point
        - Vec2::new(
            lod.origin_east_metres as f32,
            lod.origin_north_metres as f32,
        );
    let half = Vec2::new(terrain.width(), terrain.depth()) * 0.5;
    let size = Vec2::new(f32::from(lod.width - 1), f32::from(lod.depth - 1));
    let cell = (local / lod.spacing_metres + size * 0.5).floor();
    if cell.cmplt(Vec2::ZERO).any() || cell.cmpge(size).any() {
        return None;
    }
    let minimum = (cell - size * 0.5) * lod.spacing_metres;
    let maximum = minimum + Vec2::splat(lod.spacing_metres);
    for rectangle in cell_rectangles_outside_inner_rectangle(minimum, maximum, half) {
        for [x0, x1, z0, z1] in
            subdivide_playable_boundary_rectangle(rectangle, half, Some(terrain))
        {
            if local.x < x0 || local.x > x1 || local.y < z0 || local.y > z1 {
                continue;
            }
            let vertex = |x, z| {
                presented_vista_vertex_height(lod, coarser, Some(terrain), Vec2::new(x, z), half)
            };
            let a = vertex(x0, z0)?;
            let b = vertex(x1, z0)?;
            let c = vertex(x1, z1)?;
            let d = vertex(x0, z1)?;
            let u = (local.x - x0) / (x1 - x0);
            let v = (local.y - z0) / (z1 - z0);
            return Some(if u >= v {
                a * (1.0 - u) + b * (u - v) + c * v
            } else {
                a * (1.0 - v) + d * (v - u) + c * u
            });
        }
    }
    None
}

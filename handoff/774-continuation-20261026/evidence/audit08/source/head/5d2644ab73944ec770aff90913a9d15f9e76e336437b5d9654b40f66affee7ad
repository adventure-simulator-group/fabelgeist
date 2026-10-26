//! Woody-shelter rasterization and publication of matching weather textures.

use super::*;
use adventuresim_building_generator::spatial_geometry::{
    Elevation, GeometryResult, PositiveLength,
};
use adventuresim_tactical_core::scene_coordinates::Scene;

pub(in crate::presentation) fn update_weather_occlusion_map(
    active: Res<ActiveTacticalScene>,
    scenes: Query<(
        &SceneEnvironment,
        Option<&SceneTerrain>,
        Option<&StrategicBackdrop>,
    )>,
    trees: Query<(&GlobalTransform, &StreamedTreePresentation)>,
    tree_cache: Res<TreePresentationCache>,
    mut state: ResMut<WeatherOcclusionState>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(scene) = active.entity else {
        return;
    };
    let Some(map) = state
        .scene
        .filter(|selected| *selected == scene)
        .and(state.map.as_ref())
    else {
        return;
    };
    let Ok((environment, terrain, backdrop)) = scenes.get(scene) else {
        return;
    };
    let Some(terrain) = weather_surface(terrain, backdrop) else {
        return;
    };
    if environment.weather.precipitation == Precipitation::Clear {
        return;
    }

    let signature = weather_occlusion_tree_signature(scene, &trees);
    if state.tree_signature == Some(signature) {
        return;
    }
    let replacement = match weather_occlusion_image(terrain, map.range, &trees, &tree_cache) {
        Ok(image) => image,
        Err(error) => {
            warn!(%error, "Weather occlusion raster rejected");
            return;
        }
    };
    let Some(mut image) = images.get_mut(&map.image) else {
        return;
    };
    *image = replacement;
    state.tree_signature = Some(signature);
}

pub(super) fn empty_weather_occlusion_image() -> Image {
    Image::new(
        Extent3d {
            width: WEATHER_OCCLUSION_RESOLUTION,
            height: WEATHER_OCCLUSION_RESOLUTION,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        vec![0; WEATHER_OCCLUSION_RESOLUTION as usize * WEATHER_OCCLUSION_RESOLUTION as usize * 4],
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    )
}

fn weather_occlusion_tree_signature(
    scene: Entity,
    trees: &Query<(&GlobalTransform, &StreamedTreePresentation)>,
) -> u64 {
    let mut records = Vec::new();
    for (transform, presentation) in trees {
        let translation = transform.translation();
        let mut record = [0; 20];
        record[..8].copy_from_slice(&presentation.weather_occlusion_cache_key().to_le_bytes());
        record[8..12].copy_from_slice(&translation.x.to_bits().to_le_bytes());
        record[12..16].copy_from_slice(&translation.y.to_bits().to_le_bytes());
        record[16..].copy_from_slice(&translation.z.to_bits().to_le_bytes());
        records.push(record);
    }
    records.sort_unstable();
    let fields = records.iter().map(<[u8; 20]>::as_slice).collect::<Vec<_>>();
    fabelgeist_determinism::Seed::derive(
        &scene.to_bits().to_le_bytes(),
        StreamId::new("visual.weather.occlusion-signature"),
        &fields,
    )
    .to_u64()
}

fn weather_occlusion_image(
    terrain: &SceneTerrain,
    bounds: SceneElevationRange,
    trees: &Query<(&GlobalTransform, &StreamedTreePresentation)>,
    tree_cache: &TreePresentationCache,
) -> GeometryResult<Image> {
    let pixel_count = WEATHER_OCCLUSION_RESOLUTION as usize * WEATHER_OCCLUSION_RESOLUTION as usize;
    let mut shelter_heights = vec![None; pixel_count];
    for (transform, presentation) in trees {
        let Some(branches) =
            tree_cache.weather_occlusion_branches(presentation.weather_occlusion_cache_key())
        else {
            continue;
        };
        for branch in branches.iter().filter(|branch| branch.depth <= 2) {
            rasterize_weather_branch(&mut shelter_heights, terrain, transform, branch)?;
        }
    }

    let minimum = bounds.minimum().metres() - 2.0;
    let range = PositiveLength::from_metres(bounds.maximum().metres() + 24.0 - minimum)?.metres();
    let mut pixels = vec![0; pixel_count * 4];
    for (index, height) in shelter_heights.into_iter().enumerate() {
        let Some(height) = height else {
            continue;
        };
        let encoded =
            (((height.metres() - minimum) / range).clamp(0.0, 1.0) * 65_535.0).round() as u16;
        pixels[index * 4] = (encoded & 0xff) as u8;
        pixels[index * 4 + 1] = (encoded >> 8) as u8;
        pixels[index * 4 + 3] = 255;
    }
    Ok(Image::new(
        Extent3d {
            width: WEATHER_OCCLUSION_RESOLUTION,
            height: WEATHER_OCCLUSION_RESOLUTION,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    ))
}

fn rasterize_weather_branch(
    shelter_heights: &mut [Option<Elevation<Scene>>],
    terrain: &SceneTerrain,
    transform: &GlobalTransform,
    branch: &TreeBranchSegment,
) -> GeometryResult<()> {
    let start = transform.transform_point(branch.start);
    let end = transform.transform_point(branch.end);
    let scale = transform
        .to_scale_rotation_translation()
        .0
        .abs()
        .max_element();
    let start_radius = branch.start_radius * scale;
    let end_radius = branch.end_radius * scale;
    let cell_width = terrain.width() / WEATHER_OCCLUSION_RESOLUTION as f32;
    let cell_depth = terrain.depth() / WEATHER_OCCLUSION_RESOLUTION as f32;
    let footprint_margin = start_radius.max(end_radius) + cell_width.hypot(cell_depth) * 0.5;
    let minimum = start.xz().min(end.xz()) - Vec2::splat(footprint_margin);
    let maximum = start.xz().max(end.xz()) + Vec2::splat(footprint_margin);
    let to_texel = |value: f32, half_extent: f32, span: f32| {
        (((value + half_extent) / span) * WEATHER_OCCLUSION_RESOLUTION as f32).floor() as i32
    };
    let min_x = to_texel(minimum.x, terrain.width() * 0.5, terrain.width())
        .clamp(0, WEATHER_OCCLUSION_RESOLUTION as i32 - 1);
    let max_x = to_texel(maximum.x, terrain.width() * 0.5, terrain.width())
        .clamp(0, WEATHER_OCCLUSION_RESOLUTION as i32 - 1);
    let min_z = to_texel(minimum.y, terrain.depth() * 0.5, terrain.depth())
        .clamp(0, WEATHER_OCCLUSION_RESOLUTION as i32 - 1);
    let max_z = to_texel(maximum.y, terrain.depth() * 0.5, terrain.depth())
        .clamp(0, WEATHER_OCCLUSION_RESOLUTION as i32 - 1);
    if min_x > max_x || min_z > max_z {
        return Ok(());
    }

    let axis = end.xz() - start.xz();
    let axis_length_squared = axis.length_squared();
    let half_diagonal = cell_width.hypot(cell_depth) * 0.5;
    for z in min_z..=max_z {
        for x in min_x..=max_x {
            let point = Vec2::new(
                (x as f32 + 0.5) * cell_width - terrain.width() * 0.5,
                (z as f32 + 0.5) * cell_depth - terrain.depth() * 0.5,
            );
            let along = if axis_length_squared > 0.000_001 {
                ((point - start.xz()).dot(axis) / axis_length_squared).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let centre = start.xz() + axis * along;
            let radius = start_radius + (end_radius - start_radius) * along;
            if point.distance_squared(centre) > (radius + half_diagonal).powi(2) {
                continue;
            }
            let shelter_height =
                Elevation::from_metres(start.y + (end.y - start.y) * along + radius)?;
            let index = z as usize * WEATHER_OCCLUSION_RESOLUTION as usize + x as usize;
            shelter_heights[index] = Some(
                shelter_heights[index]
                    .map_or(shelter_height, |previous| previous.max(shelter_height)),
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn branch_occlusion_raster_preserves_a_large_limb_footprint() {
        let terrain = SceneTerrain::from_heightmap(11, 11, 1.0, vec![0.0; 121]).unwrap();
        let branch = TreeBranchSegment {
            start: Vec3::new(-3.0, 5.0, 0.0),
            end: Vec3::new(3.0, 5.0, 0.0),
            start_radius: 0.32,
            end_radius: 0.18,
            depth: 1,
            primary_group: 0,
            secondary_group: 0,
            is_limb_tip: false,
        };
        let mut heights = vec![
            None;
            WEATHER_OCCLUSION_RESOLUTION as usize
                * WEATHER_OCCLUSION_RESOLUTION as usize
        ];
        rasterize_weather_branch(&mut heights, &terrain, &GlobalTransform::IDENTITY, &branch)
            .unwrap();
        let covered = heights
            .iter()
            .enumerate()
            .filter_map(|(index, height)| {
                height.is_some().then_some((
                    index % WEATHER_OCCLUSION_RESOLUTION as usize,
                    index / WEATHER_OCCLUSION_RESOLUTION as usize,
                ))
            })
            .collect::<Vec<_>>();
        let width = covered.iter().map(|(x, _)| *x).max().unwrap()
            - covered.iter().map(|(x, _)| *x).min().unwrap()
            + 1;
        let depth = covered.iter().map(|(_, z)| *z).max().unwrap()
            - covered.iter().map(|(_, z)| *z).min().unwrap()
            + 1;
        assert!(covered.len() > 1_000);
        assert!(width > depth * 5);
        assert!(
            heights[256 * WEATHER_OCCLUSION_RESOLUTION as usize + 256]
                .unwrap()
                .metres()
                >= 5.18
        );
    }
}

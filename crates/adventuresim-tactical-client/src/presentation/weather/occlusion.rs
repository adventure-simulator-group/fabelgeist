//! Tree shelter images from an explicit playable cache or a treeless backdrop.

use super::*;

pub(super) enum WeatherTreeSource<'a> {
    Playable(&'a RecipeTreeCache),
    BackdropWithoutTrees,
}

impl WeatherTreeSource<'_> {
    pub(super) fn backdrop_without_trees(
        trees: &Query<(&GlobalTransform, &StreamedTreePresentation)>,
    ) -> anyhow::Result<Self> {
        anyhow::ensure!(
            trees.is_empty(),
            "strategic backdrop has playable tree occluders"
        );
        Ok(Self::BackdropWithoutTrees)
    }
}

pub(super) fn weather_occlusion_tree_signature(
    scene: Entity,
    trees: &Query<(&GlobalTransform, &StreamedTreePresentation)>,
) -> anyhow::Result<u64> {
    let mut records = Vec::new();
    for (transform, presentation) in trees {
        let translation = transform.translation();
        let mut record = [0; 20];
        record[..8].copy_from_slice(&presentation.weather_occlusion_cache_key()?.to_le_bytes());
        record[8..12].copy_from_slice(&translation.x.to_bits().to_le_bytes());
        record[12..16].copy_from_slice(&translation.y.to_bits().to_le_bytes());
        record[16..].copy_from_slice(&translation.z.to_bits().to_le_bytes());
        records.push(record);
    }
    records.sort_unstable();
    let fields = records.iter().map(<[u8; 20]>::as_slice).collect::<Vec<_>>();
    Ok(fabelgeist_determinism::Seed::derive(
        &scene.to_bits().to_le_bytes(),
        StreamId::new("visual.weather.occlusion-signature"),
        &fields,
    )
    .to_u64())
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

pub(super) fn weather_occlusion_image(
    terrain: &SceneTerrain,
    trees: &Query<(&GlobalTransform, &StreamedTreePresentation)>,
    source: WeatherTreeSource<'_>,
) -> anyhow::Result<Image> {
    let WeatherTreeSource::Playable(recipe) = source else {
        return Ok(empty_weather_occlusion_image());
    };
    let pixel_count = WEATHER_OCCLUSION_RESOLUTION as usize * WEATHER_OCCLUSION_RESOLUTION as usize;
    let mut shelter_heights = vec![f32::NEG_INFINITY; pixel_count];
    for (transform, presentation) in trees {
        let Some(branches) =
            recipe.weather_occlusion_branches(presentation.weather_occlusion_cache_key()?)
        else {
            continue;
        };
        for branch in branches.iter().filter(|branch| branch.depth <= 2) {
            rasterize_weather_branch(&mut shelter_heights, terrain, transform, branch);
        }
    }

    let minimum = terrain.minimum_height() - 2.0;
    let range = terrain.maximum_height() + 24.0 - minimum;
    let mut pixels = vec![0; pixel_count * 4];
    for (index, height) in shelter_heights.into_iter().enumerate() {
        if !height.is_finite() {
            continue;
        }
        let encoded = (((height - minimum) / range).clamp(0.0, 1.0) * 65_535.0).round() as u16;
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
    shelter_heights: &mut [f32],
    terrain: &SceneTerrain,
    transform: &GlobalTransform,
    branch: &TreeBranchSegment,
) {
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
        return;
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
            let shelter_height = start.y + (end.y - start.y) * along + radius;
            let index = z as usize * WEATHER_OCCLUSION_RESOLUTION as usize + x as usize;
            shelter_heights[index] = shelter_heights[index].max(shelter_height);
        }
    }
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
            f32::NEG_INFINITY;
            WEATHER_OCCLUSION_RESOLUTION as usize
                * WEATHER_OCCLUSION_RESOLUTION as usize
        ];
        rasterize_weather_branch(&mut heights, &terrain, &GlobalTransform::IDENTITY, &branch);
        let covered = heights
            .iter()
            .enumerate()
            .filter_map(|(index, height)| {
                height.is_finite().then_some((
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
        assert!(heights[256 * WEATHER_OCCLUSION_RESOLUTION as usize + 256] >= 5.18);
    }
}

//! Rasterize natural variation without warping managed property boundaries.
use super::*;
#[cfg(test)]
use adventuresim_tactical_core::terrain_streams;
mod noise;
use noise::GroundMaskNoise;

const GROUND_PRESENTATION_SAMPLES_PER_CELL: usize = 6;
const BROAD_WARP_NOISE_SCALE: f32 = 0.38;
const FINE_WARP_NOISE_SCALE: f32 = 0.93;

pub(super) fn ground_surface_pixel(sample: GroundSurface) -> [u8; 4] {
    let cover = match sample.cover {
        GroundCover::Bare => 0,
        GroundCover::TallGrass => 1,
        GroundCover::LeafLitter => 2,
        GroundCover::LooseStone => 3,
        GroundCover::Reeds => 4,
    };
    let substrate = match sample.substrate {
        GroundSubstrate::Soil => 0,
        GroundSubstrate::Stone => 1,
        GroundSubstrate::Gravel => 2,
        GroundSubstrate::Mud => 3,
        GroundSubstrate::Road => 4,
        GroundSubstrate::Water => 5,
    };
    [
        cover,
        substrate,
        adventuresim_world_schema::UnitBasisPoints::saturating(sample.cover_density_bps)
            .scale_u32_floor(255) as u8,
        sample.cover_height_cm.min(255) as u8,
    ]
}

fn chamfer_distance_to(
    width: usize,
    height: usize,
    sources: impl Fn(usize) -> bool,
    maximum: usize,
) -> Vec<usize> {
    let mut distance = (0..width * height)
        .map(|index| if sources(index) { 0 } else { maximum + 1 })
        .collect::<Vec<_>>();
    for z in 0..height {
        for x in 0..width {
            let index = z * width + x;
            for (dx, dz) in [(-1_isize, 0_isize), (0, -1), (-1, -1), (1, -1)] {
                let nx = x as isize + dx;
                let nz = z as isize + dz;
                if nx >= 0 && nz >= 0 && nx < width as isize && nz < height as isize {
                    distance[index] = distance[index]
                        .min(distance[nz as usize * width + nx as usize].saturating_add(1));
                }
            }
        }
    }
    for z in (0..height).rev() {
        for x in (0..width).rev() {
            let index = z * width + x;
            for (dx, dz) in [(1_isize, 0_isize), (0, 1), (1, 1), (-1, 1)] {
                let nx = x as isize + dx;
                let nz = z as isize + dz;
                if nx >= 0 && nz >= 0 && nx < width as isize && nz < height as isize {
                    distance[index] = distance[index]
                        .min(distance[nz as usize * width + nx as usize].saturating_add(1));
                }
            }
        }
    }
    distance
}

fn encode_canopy_floor_distance(
    ground: &SceneGround,
    width: usize,
    height: usize,
    pixels: &mut [u8],
) {
    let metres_per_pixel = ground.grid_scale() / GROUND_PRESENTATION_SAMPLES_PER_CELL as f32;
    let inner_radius = (2.2 / metres_per_pixel).ceil().max(1.0) as usize;
    let outer_radius = (4.8 / metres_per_pixel).ceil().max(1.0) as usize;
    let litter = pixels
        .as_chunks::<4>()
        .0
        .iter()
        .map(|pixel| pixel[0] == GroundCover::LeafLitter as u8)
        .collect::<Vec<_>>();
    let distance_to_litter =
        chamfer_distance_to(width, height, |index| litter[index], outer_radius);
    let distance_to_other =
        chamfer_distance_to(width, height, |index| !litter[index], inner_radius);
    for index in 0..width * height {
        let encoded = if litter[index] {
            let depth = (distance_to_other[index] as f32 / inner_radius as f32).clamp(0.0, 1.0);
            128.0 + depth * 127.0
        } else {
            let proximity =
                (1.0 - distance_to_litter[index] as f32 / outer_radius as f32).clamp(0.0, 1.0);
            proximity * 127.0
        };
        // Alpha is presentation-only. It carries signed distance from the
        // organic litter boundary instead of duplicating gameplay cover
        // height, which remains authoritative in SceneGround.
        pixels[index * 4 + 3] = encoded.round() as u8;
    }
}

#[cfg(test)]
pub(super) fn ground_mask_noise(seed: u64, point: Vec2) -> f32 {
    let cell = point.floor();
    let local = point - cell;
    let curve = local * local * (Vec2::splat(3.0) - local * 2.0);
    let hash = |offset: Vec2| {
        let coordinate = cell + offset;
        let x = i64::from(coordinate.x as i32) as u64;
        let y = i64::from(coordinate.y as i32) as u64;
        terrain_streams::GROUND_MASK_LATTICE
            .rng(seed, &[x, y])
            .inclusive_unit_f32()
    };
    let bottom = hash(Vec2::ZERO).lerp(hash(Vec2::X), curve.x);
    let top = hash(Vec2::Y).lerp(hash(Vec2::ONE), curve.x);
    bottom.lerp(top, curve.y)
}

pub(super) fn organic_ground_pixels(ground: &SceneGround, seed: u64) -> (u32, u32, Vec<u8>) {
    let source_width = ground.grid_width();
    let source_depth = ground.grid_depth();
    let width = (source_width - 1) * GROUND_PRESENTATION_SAMPLES_PER_CELL + 1;
    let depth = (source_depth - 1) * GROUND_PRESENTATION_SAMPLES_PER_CELL + 1;
    let mut pixels = Vec::with_capacity(width * depth * 4);
    let scale = GROUND_PRESENTATION_SAMPLES_PER_CELL as f32;
    let maximum = Vec2::new((source_width - 1) as f32, (source_depth - 1) as f32);
    let broad_offset = Vec2::new(17.3, -9.1);
    let fine_x_offset = Vec2::new(31.7, 5.9);
    let fine_y_offset = Vec2::new(-7.7, 23.1);
    let broad_x = GroundMaskNoise::new(
        seed ^ 0x2f31_9a87,
        Vec2::ZERO,
        maximum * BROAD_WARP_NOISE_SCALE,
    );
    let broad_y = GroundMaskNoise::new(
        seed ^ 0x91b7_43cd,
        broad_offset,
        maximum * BROAD_WARP_NOISE_SCALE + broad_offset,
    );
    let fine_x = GroundMaskNoise::new(
        seed ^ 0x6d25_e9f1,
        fine_x_offset,
        maximum * FINE_WARP_NOISE_SCALE + fine_x_offset,
    );
    let fine_y = GroundMaskNoise::new(
        seed ^ 0xc4ab_1283,
        fine_y_offset,
        maximum * FINE_WARP_NOISE_SCALE + fine_y_offset,
    );
    for z in 0..depth {
        for x in 0..width {
            let point = Vec2::new(x as f32 / scale, z as f32 / scale);
            let broad_point = point * BROAD_WARP_NOISE_SCALE;
            let fine_point = point * FINE_WARP_NOISE_SCALE;
            let broad_warp = Vec2::new(
                broad_x.sample(broad_point),
                broad_y.sample(broad_point + broad_offset),
            ) * 2.0
                - Vec2::ONE;
            let fine_warp = Vec2::new(
                fine_x.sample(fine_point + fine_x_offset),
                fine_y.sample(fine_point + fine_y_offset),
            ) * 2.0
                - Vec2::ONE;
            let warped = point + broad_warp * 0.78 + fine_warp * 0.22;
            let source_x = (warped.x.round() as isize).clamp(0, source_width as isize - 1) as usize;
            let source_z = (warped.y.round() as isize).clamp(0, source_depth as isize - 1) as usize;
            pixels.extend_from_slice(&ground_surface_pixel(
                ground
                    .urban
                    .surface_at(
                        point * ground.grid_scale()
                            - Vec2::new(ground.width(), ground.depth()) * 0.5,
                    )
                    .unwrap_or(ground.samples()[source_z * source_width + source_x]),
            ));
        }
    }
    encode_canopy_floor_distance(ground, width, depth, &mut pixels);
    (width as u32, depth as u32, pixels)
}

pub(in crate::presentation) fn grass_cover_mask_pixels(
    ground: &SceneGround,
    seed: u64,
) -> (u32, u32, Vec<u8>) {
    let (width, height, ground_pixels) = organic_ground_pixels(ground, seed);
    let mut mask = vec![0_u8; width as usize * height as usize];
    let metres_per_pixel = ground.grid_scale() / GROUND_PRESENTATION_SAMPLES_PER_CELL as f32;
    let radius = (4.8 / metres_per_pixel).ceil().max(1.0) as usize;
    let width_usize = width as usize;
    let height_usize = height as usize;
    // The playable rectangle is a data-authority boundary, not a vegetation
    // boundary. Only authored non-grass pixels seed this distance field.
    let distance = chamfer_distance_to(
        width_usize,
        height_usize,
        |index| ground_pixels[index * 4] != GroundCover::TallGrass as u8,
        radius,
    );
    for z in 0..height as usize {
        for x in 0..width as usize {
            let pixel = (z * width as usize + x) * 4;
            if ground_pixels[pixel] != GroundCover::TallGrass as u8 {
                continue;
            }
            let density = ground_pixels[pixel + 2];
            let feather = (distance[z * width_usize + x] as f32 / radius as f32)
                .clamp(0.0, 1.0)
                .powf(1.28);
            mask[z * width as usize + x] = (f32::from(density) * feather) as u8;
        }
    }
    (width, height, mask)
}

pub(super) fn ground_map_image(ground: Option<&SceneGround>, seed: u64) -> Image {
    let (width, height, pixels) = ground.map_or_else(
        || (1, 1, vec![0, 0, 0, 0]),
        |ground| organic_ground_pixels(ground, seed),
    );
    let mut image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    // The shader rounds the interpolated enum channel back to one exact cover
    // kind before selecting a material. Linear filtering therefore smooths
    // the baked contour itself without producing a visible colour gradient.
    image.sampler = ImageSampler::linear();
    image
}

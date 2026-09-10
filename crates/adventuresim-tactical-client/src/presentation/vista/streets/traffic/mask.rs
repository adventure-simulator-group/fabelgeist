//! Bounded tile rasterization with identical world-space filter gutters.
use super::*;
use bevy::{
    image::ImageSampler,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

pub(in super::super) struct TrafficMask {
    pub(in super::super) image: Handle<Image>,
    pub(in super::super) transform: Vec4,
}

impl TrafficMask {
    pub(in super::super) fn bake(
        network: &TrafficNetwork,
        tile: TrafficTile,
        images: &mut Assets<Image>,
    ) -> Self {
        let pixels = network.pixels(tile);
        let mut image = Image::new(
            Extent3d {
                width: TILE_PIXELS as u32,
                height: TILE_PIXELS as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            pixels,
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::RENDER_WORLD,
        );
        image.sampler = ImageSampler::linear();
        let origin = tile.origin() - Vec2::splat(FILTER_GUTTER_PIXELS as f32 / TEXELS_PER_METRE);
        Self {
            image: images.add(image),
            transform: Vec4::new(
                origin.x,
                origin.y,
                TEXELS_PER_METRE / TILE_PIXELS as f32,
                TEXELS_PER_METRE / TILE_PIXELS as f32,
            ),
        }
    }
}

impl TrafficNetwork {
    pub(super) fn pixels(&self, tile: TrafficTile) -> Vec<u8> {
        let origin = tile.origin() - Vec2::splat(FILTER_GUTTER_PIXELS as f32 / TEXELS_PER_METRE);
        let maximum = origin + Vec2::splat(TILE_PIXELS as f32 / TEXELS_PER_METRE);
        let roads = self
            .roads
            .iter()
            .copied()
            .filter(|road| {
                let padding = Vec2::splat(road.half_width);
                (road.start.min(road.end) - padding).cmple(maximum).all()
                    && (road.start.max(road.end) + padding).cmpge(origin).all()
            })
            .collect::<Vec<_>>();
        let markets = self
            .markets
            .iter()
            .copied()
            .filter(|corners| {
                let min = corners
                    .iter()
                    .copied()
                    .fold(Vec2::splat(f32::INFINITY), Vec2::min);
                let max = corners
                    .iter()
                    .copied()
                    .fold(Vec2::splat(f32::NEG_INFINITY), Vec2::max);
                min.cmple(maximum).all() && max.cmpge(origin).all()
            })
            .collect::<Vec<_>>();
        let mut pixels = vec![0_u8; TILE_PIXELS * TILE_PIXELS * 4];
        for y in 0..TILE_PIXELS {
            for x in 0..TILE_PIXELS {
                let point =
                    origin + (Vec2::new(x as f32, y as f32) + Vec2::splat(0.5)) / TEXELS_PER_METRE;
                let mut mud = 0.0_f32;
                let mut clearance = f32::NEG_INFINITY;
                for road in &roads {
                    let edge = road.clearance(point);
                    clearance = clearance.max(edge);
                    if edge < 0.0 {
                        continue;
                    }
                    let local = road.coordinates(point);
                    // Hooves, displaced soil and many vehicle lanes fill most
                    // of the carriageway, independently of the sharper ruts.
                    mud = mud.max(
                        smooth_band(
                            road.half_width * 0.48,
                            road.half_width * 0.92,
                            local.x.abs(),
                        ) * 0.88,
                    );
                }
                for &corners in &markets {
                    clearance = clearance.max(quad_clearance(corners, point));
                }
                if (0.0..ROAD_SHOULDER_METRES).contains(&clearance) {
                    let at = |p| {
                        roads
                            .iter()
                            .map(|road| road.clearance(p))
                            .chain(markets.iter().map(|&corners| quad_clearance(corners, p)))
                            .fold(f32::NEG_INFINITY, f32::max)
                    };
                    for fraction in [0.25, 0.5, 0.75, 1.0] {
                        let radius = ROAD_SHOULDER_METRES * fraction;
                        if union_support(point, radius, at) {
                            clearance = clearance.max(radius);
                        } else {
                            break;
                        }
                    }
                }
                let index = (y * TILE_PIXELS + x) * 4;
                pixels[index] = (mud * 255.0) as u8;
                pixels[index + 2] =
                    ((1.0 - smooth_band(0.0, ROAD_SHOULDER_METRES, clearance)) * 255.0) as u8;
                pixels[index + 3] = 255;
            }
        }
        if let Some(strokes) = self.tiles.get(&tile) {
            for &index in strokes {
                rasterize(&mut pixels, origin, self.strokes[index]);
            }
        }
        pixels
    }
}

fn rasterize(pixels: &mut [u8], origin: Vec2, stroke: WheelStroke) {
    let radius = stroke.half_width + paths::WHEEL_FEATHER_METRES;
    let min = (((stroke.start.min(stroke.end) - Vec2::splat(radius) - origin) * TEXELS_PER_METRE)
        .floor()
        .as_ivec2())
    .max(IVec2::ZERO);
    let max = (((stroke.start.max(stroke.end) + Vec2::splat(radius) - origin) * TEXELS_PER_METRE)
        .ceil()
        .as_ivec2())
    .min(IVec2::splat(TILE_PIXELS as i32 - 1));
    let delta = stroke.end - stroke.start;
    for y in min.y..=max.y {
        for x in min.x..=max.x {
            let point =
                origin + (Vec2::new(x as f32, y as f32) + Vec2::splat(0.5)) / TEXELS_PER_METRE;
            let t = ((point - stroke.start).dot(delta) / delta.length_squared()).clamp(0.0, 1.0);
            let distance = point.distance(stroke.start + delta * t);
            let value = smooth_band(stroke.half_width * 0.35, radius, distance) * stroke.strength;
            let index = (y as usize * TILE_PIXELS + x as usize) * 4;
            pixels[index + 1] = pixels[index + 1].max((value * 255.0) as u8);
        }
    }
}

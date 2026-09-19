//! Runtime flute relief encoded as a compact tangent-space normal atlas.
use crate::PlateFluting;

const MAP_WIDTH: usize = 512;
const PATTERN_HEIGHT: usize = 128;
const MINIMUM_CHART_EXTENT_M: f32 = 0.05;
// A normal map needs less slope than displaced geometry because it cannot add
// matching silhouette or self-shadowing. This keeps dense historical patterns
// readable instead of turning sub-pixel flutes into black-white moire.
const NORMAL_RELIEF_SCALE: f32 = 0.40;
const FULL_STRENGTH_FLUTE_COUNT: f32 = 24.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FlutingTile {
    Plate {
        pattern: PlateFluting,
        width_mm: u16,
        height_mm: u16,
    },
    Radial {
        pattern: crate::RadialFluting,
        radius_mm: u16,
    },
}

impl FlutingTile {
    pub(crate) fn new(pattern: PlateFluting, width_m: f32, height_m: f32) -> Self {
        let millimeters =
            |metres: f32| (metres.max(MINIMUM_CHART_EXTENT_M) * 100.0).round() as u16 * 10;
        Self::Plate {
            pattern,
            width_mm: millimeters(width_m),
            height_mm: millimeters(height_m),
        }
    }

    pub(crate) fn radial(pattern: crate::RadialFluting, radius_m: f32) -> Self {
        Self::Radial {
            pattern,
            radius_mm: (radius_m.max(MINIMUM_CHART_EXTENT_M) * 1_000.0).round() as u16,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GeneratedNormalMap {
    pub width: u32,
    pub height: u32,
    pub rgba8: Vec<u8>,
}

pub(crate) fn atlas(
    tiles: &[FlutingTile],
    samples: &[(usize, [f32; 2])],
) -> (GeneratedNormalMap, Vec<[f32; 2]>) {
    let height = PATTERN_HEIGHT * tiles.len();
    let mut rgba8 = Vec::with_capacity(MAP_WIDTH * height * 4);
    for tile in tiles {
        for row in 0..PATTERN_HEIGHT {
            let v = (row as f32 + 0.5) / PATTERN_HEIGHT as f32;
            for column in 0..MAP_WIDTH {
                let u = (column as f32 + 0.5) / MAP_WIDTH as f32;
                rgba8.extend(normal_texel(*tile, u, v));
            }
        }
    }
    let texcoords = samples
        .iter()
        .map(|(tile, [u, v])| {
            [
                u.clamp(0.0, 1.0),
                (*tile as f32 + v.clamp(0.0, 1.0)) / tiles.len() as f32,
            ]
        })
        .collect();
    (
        GeneratedNormalMap {
            width: MAP_WIDTH as u32,
            height: height as u32,
            rgba8,
        },
        texcoords,
    )
}

fn normal_texel(tile: FlutingTile, u: f32, v: f32) -> [u8; 4] {
    match tile {
        FlutingTile::Plate {
            pattern,
            width_mm,
            height_mm,
        } => normal_texel_from_relief(
            pattern.count,
            pattern.depth,
            u,
            v,
            f32::from(width_mm) / 1_000.0,
            f32::from(height_mm) / 1_000.0,
            |u, v| pattern.relief(u, v),
        ),
        FlutingTile::Radial { pattern, radius_mm } => {
            let radius_m = f32::from(radius_mm) / 1_000.0;
            normal_texel_from_relief(
                pattern.count,
                pattern.depth,
                u,
                v,
                radius_m * 2.0,
                radius_m * 2.0,
                |u, v| {
                    let [x, y] = [u * 2.0 - 1.0, v * 2.0 - 1.0];
                    let radius = x.mul_add(x, y * y).sqrt();
                    let angle = y.atan2(x) / std::f32::consts::TAU;
                    pattern.relief(angle, radius)
                },
            )
        }
    }
}

fn normal_texel_from_relief(
    count: crate::FluteCount,
    depth: crate::Millimeters,
    u: f32,
    v: f32,
    width_m: f32,
    height_m: f32,
    relief: impl Fn(f32, f32) -> f32,
) -> [u8; 4] {
    let resolvable_pitch = (FULL_STRENGTH_FLUTE_COUNT / f32::from(count.0)).min(1.0);
    // Dense flutes quickly become narrower than a screen pixel. Their averaged
    // normal approaches the carrier normal cubically, while alpha retains the
    // full authored height field for offline or future parallax use.
    let relief_scale = NORMAL_RELIEF_SCALE * resolvable_pitch.powi(3);
    let du = 0.5 / MAP_WIDTH as f32;
    let dv = 0.5 / PATTERN_HEIGHT as f32;
    let lateral_slope =
        (relief(u + du, v) - relief(u - du, v)) / (2.0 * du * width_m) * relief_scale;
    let vertical_slope =
        (relief(u, v + dv) - relief(u, v - dv)) / (2.0 * dv * height_m) * relief_scale;
    let inverse_length = (lateral_slope
        .mul_add(lateral_slope, vertical_slope.mul_add(vertical_slope, 1.0)))
    .sqrt()
    .recip();
    let normal = [
        -lateral_slope * inverse_length,
        -vertical_slope * inverse_length,
        inverse_length,
    ];
    let height = relief(u, v) / depth.metres();
    [
        encode_normal(normal[0]),
        encode_normal(normal[1]),
        encode_normal(normal[2]),
        (height.clamp(0.0, 1.0) * 255.0).round() as u8,
    ]
}

fn encode_normal(value: f32) -> u8 {
    ((value * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0).round() as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_map_is_flat_outside_pattern_and_sloped_at_flute_edges() {
        let pattern = PlateFluting::default();
        let tile = FlutingTile::new(pattern, 0.3, 0.4);
        assert_eq!(normal_texel(tile, 0.0, 0.5), [128, 128, 255, 0]);
        let pitch = pattern.spread.unit() / f32::from(pattern.count.0);
        let edge = (1.0 - pattern.spread.unit()) * 0.5 + pitch * 0.30;
        let sample = normal_texel(tile, edge, 0.5);
        assert_ne!(sample[0], 128);
        assert!(sample[2] < 255);
    }

    #[test]
    fn atlas_remaps_each_pattern_to_its_own_vertical_tile() {
        let first = PlateFluting::default();
        let mut second = first;
        second.count = crate::FluteCount(8);
        let (_, coordinates) = atlas(
            &[
                FlutingTile::new(first, 0.3, 0.4),
                FlutingTile::new(second, 0.3, 0.4),
            ],
            &[(0, [0.25, 0.5]), (1, [0.75, 0.5])],
        );
        assert_eq!(coordinates, [[0.25, 0.25], [0.75, 0.75]]);
    }

    #[test]
    fn atlas_has_complete_pixels_and_bounded_finite_coordinates() {
        let (map, coordinates) = atlas(
            &[FlutingTile::new(PlateFluting::default(), 0.3, 0.4)],
            &[(0, [-1.0, f32::INFINITY]), (0, [2.0, -1.0])],
        );
        assert_eq!(
            map.rgba8.len(),
            map.width as usize * map.height as usize * 4
        );
        assert!(coordinates.iter().flatten().all(|value| value.is_finite()));
        assert!(
            coordinates
                .iter()
                .flatten()
                .all(|value| (0.0..=1.0).contains(value))
        );
    }

    #[test]
    fn authored_parameters_change_the_generated_relief() {
        let base = PlateFluting::default();
        let variants = [
            PlateFluting {
                count: crate::FluteCount(8),
                ..base
            },
            PlateFluting {
                depth: crate::Millimeters(4),
                ..base
            },
            PlateFluting {
                fade: crate::Permille(200),
                ..base
            },
        ];
        let (base_map, _) = atlas(&[FlutingTile::new(base, 0.3, 0.4)], &[]);
        for variant in variants {
            let (map, _) = atlas(&[FlutingTile::new(variant, 0.3, 0.4)], &[]);
            assert_ne!(map.rgba8, base_map.rgba8);
        }
    }

    #[test]
    fn dense_patterns_attenuate_normals_without_erasing_height() {
        let sparse = PlateFluting {
            count: crate::FluteCount(16),
            ..PlateFluting::default()
        };
        let dense = PlateFluting {
            count: crate::FluteCount(64),
            ..sparse
        };
        let normal_energy = |map: &GeneratedNormalMap| {
            map.rgba8
                .as_chunks::<4>()
                .0
                .iter()
                .map(|pixel| i32::from(pixel[0]).abs_diff(128) + i32::from(pixel[1]).abs_diff(128))
                .sum::<u32>()
        };
        let (sparse, _) = atlas(&[FlutingTile::new(sparse, 0.3, 0.4)], &[]);
        let (dense, _) = atlas(&[FlutingTile::new(dense, 0.3, 0.4)], &[]);
        assert!(normal_energy(&dense) < normal_energy(&sparse));
        assert!(sparse.rgba8.iter().skip(3).step_by(4).max().unwrap() > &240);
        assert!(dense.rgba8.iter().skip(3).step_by(4).max().unwrap() > &240);
    }

    #[test]
    fn radial_fluting_fades_to_flat_at_its_borders() {
        let tile = FlutingTile::radial(crate::RadialFluting::default(), 0.058);
        assert_eq!(normal_texel(tile, 0.5, 0.0), [128, 128, 255, 0]);
        let middle = normal_texel(tile, 0.75, 0.52);
        assert_ne!(&middle[..2], &[128, 128]);
        assert!(middle[3] > 0);
    }
}

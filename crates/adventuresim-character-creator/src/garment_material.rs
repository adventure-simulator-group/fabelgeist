//! Parametric chainmail appearance: ring geometry sets the texture tile and
//! surface maps. It never changes the cloth simulation.
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

mod rings;
pub use rings::MailMaps;

/// Air gap between neighbouring rings of the same row.
const ROW_NEIGHBOUR_GAP_M: f32 = 0.0001;
/// A link interlinks four neighbours only if its opening spans two wires.
const MIN_INSIDE_DIAMETER_IN_WIRES: f32 = 2.0;
const MM_PER_M: f32 = 1000.0;

/// Base color factor, metallic and roughness of untextured cloth.
pub const CLOTH_PBR: ([f32; 4], f32, f32) = ([0.52, 0.42, 0.28, 1.0], 0.0, 0.85);

/// European 4-in-1 mail made of round-wire rings.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct MailWeave {
    /// Outside diameter of one link, in metres.
    pub ring_outer_diameter_m: f32,
    /// Diameter of the ring wire, in metres.
    pub wire_diameter_m: f32,
    /// Distance between neighbouring ring rows, in metres. Closer rows pack
    /// more rings into the same area.
    pub row_pitch_m: f32,
    /// How far each row's rings lean out of the cloth surface, in degrees.
    pub ring_tilt_degrees: f32,
    /// Steel base color, sRGB channels in 0..=1.
    pub steel_color_srgb: [f32; 3],
    /// Perceptual roughness of the steel surface.
    pub roughness: f32,
}

impl MailWeave {
    /// Met 27.183.35: 6.2 mm outside and 4.0 mm inside link diameters.
    pub const STANDARD: Self = Self {
        ring_outer_diameter_m: 0.0062,
        wire_diameter_m: 0.0011,
        row_pitch_m: 0.0022,
        ring_tilt_degrees: 34.0,
        steel_color_srgb: [0.71; 3],
        roughness: 0.32,
    };
    pub const RING_OUTER_DIAMETER_M: RangeInclusive<f32> = 0.003..=0.016;
    pub const WIRE_DIAMETER_M: RangeInclusive<f32> = 0.0005..=0.003;
    pub const RING_TILT_DEGREES: RangeInclusive<f32> = 0.0..=60.0;
    pub const ROUGHNESS: RangeInclusive<f32> = 0.05..=1.0;

    /// Thickest wire that still leaves an opening for the neighbouring links.
    pub fn max_wire_diameter_m(&self) -> f32 {
        self.ring_outer_diameter_m / (2.0 + MIN_INSIDE_DIAMETER_IN_WIRES)
    }

    /// Wire diameters valid for this ring: the absolute limits, capped by
    /// [`Self::max_wire_diameter_m`].
    pub fn wire_diameter_range_m(&self) -> RangeInclusive<f32> {
        *Self::WIRE_DIAMETER_M.start()
            ..=self
                .max_wire_diameter_m()
                .min(*Self::WIRE_DIAMETER_M.end())
    }

    /// Rows closer than one wire collide; rows further than half a ring apart
    /// no longer pass through each other's openings.
    pub fn row_pitch_range_m(&self) -> RangeInclusive<f32> {
        self.wire_diameter_m..=self.ring_outer_diameter_m * 0.5
    }

    /// Material-space size of one periodic tile: one ring across, two rows up.
    pub fn repeat_m(&self) -> [f32; 2] {
        [
            self.ring_outer_diameter_m + ROW_NEIGHBOUR_GAP_M,
            self.row_pitch_m * 2.0,
        ]
    }

    pub fn rings_per_m2(&self) -> f32 {
        let [column_pitch, _] = self.repeat_m();
        1.0 / (column_pitch * self.row_pitch_m)
    }

    /// Base color factor, metallic and roughness; color lives in the maps.
    pub fn pbr(&self) -> ([f32; 4], f32, f32) {
        ([1.0; 4], 1.0, self.roughness)
    }

    pub fn validate(&self) -> Result<()> {
        let scalars = [
            self.ring_outer_diameter_m,
            self.wire_diameter_m,
            self.row_pitch_m,
            self.ring_tilt_degrees,
            self.roughness,
        ];
        ensure!(
            scalars
                .iter()
                .chain(&self.steel_color_srgb)
                .all(|value| value.is_finite()),
            "chainmail parameters must be finite"
        );
        let mm = |range: &RangeInclusive<f32>| (range.start() * MM_PER_M, range.end() * MM_PER_M);
        let (outer_min, outer_max) = mm(&Self::RING_OUTER_DIAMETER_M);
        ensure!(
            Self::RING_OUTER_DIAMETER_M.contains(&self.ring_outer_diameter_m),
            "ring outer diameter must be between {outer_min} and {outer_max} mm"
        );
        let (wire_min, wire_max) = mm(&Self::WIRE_DIAMETER_M);
        ensure!(
            Self::WIRE_DIAMETER_M.contains(&self.wire_diameter_m),
            "wire diameter must be between {wire_min} and {wire_max} mm"
        );
        ensure!(
            self.wire_diameter_m <= self.max_wire_diameter_m(),
            "wire is too thick for the ring: at most {:.2} mm leaves an opening for its neighbours",
            self.max_wire_diameter_m() * MM_PER_M
        );
        let (pitch_min, pitch_max) = mm(&self.row_pitch_range_m());
        ensure!(
            self.row_pitch_range_m().contains(&self.row_pitch_m),
            "row spacing must be between {pitch_min:.2} and {pitch_max:.2} mm so rows interlink"
        );
        ensure!(
            Self::RING_TILT_DEGREES.contains(&self.ring_tilt_degrees),
            "ring tilt must be between {} and {} degrees",
            Self::RING_TILT_DEGREES.start(),
            Self::RING_TILT_DEGREES.end()
        );
        ensure!(
            Self::ROUGHNESS.contains(&self.roughness),
            "steel roughness must be between {} and {}",
            Self::ROUGHNESS.start(),
            Self::ROUGHNESS.end()
        );
        ensure!(
            self.steel_color_srgb
                .iter()
                .all(|channel| (0.0..=1.0).contains(channel)),
            "steel color channels must be between 0 and 1"
        );
        Ok(())
    }
}

impl Default for MailWeave {
    fn default() -> Self {
        Self::STANDARD
    }
}

/// Export-ready chainmail: generated maps, texture coordinates scaled to one
/// ring repeat, and the steel finish.
pub struct MailSurface {
    pub maps: MailMaps,
    pub texcoords: Vec<[f32; 2]>,
    weave: MailWeave,
}

impl MailSurface {
    /// `material_m` are the garment's pattern coordinates, in metres.
    pub fn new(weave: &MailWeave, material_m: &[[f32; 2]]) -> Result<Self> {
        let [across, up] = weave.repeat_m();
        Ok(Self {
            maps: MailMaps::new(weave)?,
            texcoords: material_m
                .iter()
                .map(|uv| [uv[0] / across, uv[1] / up])
                .collect(),
            weave: *weave,
        })
    }

    pub fn pbr(&self) -> ([f32; 4], f32, f32) {
        self.weave.pbr()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode(png: &[u8]) -> image::RgbaImage {
        image::load_from_memory(png).unwrap().to_rgba8()
    }

    #[test]
    fn standard_rings_have_open_holes_steel_color_and_curved_normals() {
        let maps = MailMaps::new(&MailWeave::STANDARD).unwrap();
        let textures = maps.textures();
        let color = decode(textures.base_color_png);
        let normal = decode(textures.normal_png);
        assert_eq!(color.dimensions(), normal.dimensions());
        assert!(color.pixels().any(|p| p[3] == 0));
        assert!(color.pixels().any(|p| p[3] == 255));
        let steel = MailWeave::STANDARD
            .steel_color_srgb
            .map(|c| (c * 255.0).round() as u8);
        let wire: Vec<_> = color
            .pixels()
            .zip(normal.pixels())
            .filter(|(color, _)| color[3] == 255)
            .map(|(color, normal)| {
                assert_eq!(color.0[..3], steel);
                normal
            })
            .collect();
        for axis in 0..2 {
            assert!(wire.iter().any(|n| n[axis] < 96));
            assert!(wire.iter().any(|n| n[axis] > 160));
        }
        assert!(wire.iter().all(|n| n[2] >= 127));
    }

    #[test]
    fn ring_chart_wraps_across_tile_edges() {
        let weave = MailWeave::STANDARD;
        let chart = rings::Chart::new(&weave);
        let [tile_x, tile_y] = weave.repeat_m();
        let (mut matched, mut mismatched) = (0, 0);
        const GRID: usize = 40;
        for i in 0..GRID {
            for j in 0..GRID {
                let x = tile_x * i as f32 / GRID as f32;
                let y = tile_y * j as f32 / GRID as f32;
                let base = chart.sample(x, y);
                for shifted in [chart.sample(x + tile_x, y), chart.sample(x, y + tile_y)] {
                    match (&base, &shifted) {
                        (Some(a), Some(b))
                            if a.normal
                                .iter()
                                .zip(b.normal)
                                .all(|(p, q)| (p - q).abs() < 0.01) =>
                        {
                            matched += 1
                        }
                        (None, None) => matched += 1,
                        _ => mismatched += 1,
                    }
                }
            }
        }
        // Only texels grazing a silhouette may differ by rounding.
        assert!(mismatched * 100 < matched, "{mismatched} of {matched}");
    }

    #[test]
    fn ring_geometry_sets_the_texture_tile() {
        let standard = MailWeave::STANDARD;
        let [across, up] = standard.repeat_m();
        assert!((across - 0.0063).abs() < 1e-6 && (up - 0.0044).abs() < 1e-6);
        let larger = MailWeave {
            ring_outer_diameter_m: 0.009,
            row_pitch_m: 0.003,
            ..standard
        };
        assert!(larger.repeat_m()[0] > across);
        let maps = MailMaps::new(&larger).unwrap();
        let color = decode(maps.textures().base_color_png);
        let expected = (color.width() as f32 * larger.repeat_m()[1] / larger.repeat_m()[0]).round();
        assert_eq!(color.height(), expected as u32);
    }

    #[test]
    fn mail_surface_scales_pattern_metres_to_ring_repeats() {
        let weave = MailWeave {
            roughness: 0.6,
            ..MailWeave::STANDARD
        };
        let [across, up] = weave.repeat_m();
        let surface = MailSurface::new(&weave, &[[across * 3.0, up * 2.0]]).unwrap();
        assert!((surface.texcoords[0][0] - 3.0).abs() < 1e-4);
        assert!((surface.texcoords[0][1] - 2.0).abs() < 1e-4);
        assert_eq!(surface.pbr().2, 0.6);
    }

    #[test]
    fn weaves_that_cannot_interlink_are_rejected() {
        let standard = MailWeave::STANDARD;
        assert!(standard.validate().is_ok());
        let thick_wire = MailWeave {
            wire_diameter_m: 0.002,
            ..standard
        };
        assert!(thick_wire.validate().is_err());
        let loose_rows = MailWeave {
            row_pitch_m: 0.004,
            ..standard
        };
        assert!(loose_rows.validate().is_err());
        let invalid = MailWeave {
            roughness: f32::NAN,
            ..standard
        };
        assert!(invalid.validate().is_err());
        assert!(MailMaps::new(&thick_wire).is_err());
    }

    #[test]
    fn dependent_limits_are_valid_for_every_ring() {
        for ring in [
            *MailWeave::RING_OUTER_DIAMETER_M.start(),
            *MailWeave::RING_OUTER_DIAMETER_M.end(),
        ] {
            let ring = MailWeave {
                ring_outer_diameter_m: ring,
                ..MailWeave::STANDARD
            };
            for wire in [*ring.wire_diameter_range_m().start(), *ring.wire_diameter_range_m().end()] {
                let wire = MailWeave {
                    wire_diameter_m: wire,
                    ..ring
                };
                for row_pitch in [*wire.row_pitch_range_m().start(), *wire.row_pitch_range_m().end()] {
                    let weave = MailWeave { row_pitch_m: row_pitch, ..wire };
                    assert!(weave.validate().is_ok(), "{weave:?}");
                }
            }
        }
    }
}

//! Interlinked round-wire rings seen straight down the cloth normal.
//!
//! Rows alternate their tilt out of the cloth plane and odd rows shift by half
//! a ring, forming European 4-in-1 mail. One chart covers exactly one periodic
//! tile, [`MailWeave::repeat_m`], so the maps wrap seamlessly on a garment.
use super::MailWeave;
use crate::export::SurfaceTextures;
use anyhow::Result;
use image::ImageEncoder;

/// Horizontal texel count; the vertical count follows the tile aspect ratio.
const MAP_WIDTH_TEXELS: u32 = 256;
/// A sphere-traced ray has reached the wire within this fraction of its radius.
const HIT_TOLERANCE_IN_WIRE_RADII: f32 = 1e-3;
const TRACE_STEP_LIMIT: usize = 128;
/// Occlusion probes along the surface normal reach this many wire radii.
const OCCLUSION_REACH_IN_WIRE_RADII: f32 = 3.0;
const OCCLUSION_PROBES: u32 = 5;
/// Tangent-space normal of an open aperture, facing straight out of the cloth.
const FLAT_NORMAL_RGBA: [u8; 4] = [128, 128, 255, 255];

/// PNG surface maps for one weave, shared by the live preview and GLB export.
pub struct MailMaps {
    color_png: Vec<u8>,
    normal_png: Vec<u8>,
    occlusion_png: Vec<u8>,
}

impl MailMaps {
    pub fn new(weave: &MailWeave) -> Result<Self> {
        weave.validate()?;
        let chart = Chart::new(weave);
        let [tile_x, tile_y] = weave.repeat_m();
        let width = MAP_WIDTH_TEXELS;
        let height = ((width as f32 * tile_y / tile_x).round() as u32).max(1);
        let steel = weave.steel_color_srgb.map(|c| (c * 255.0).round() as u8);
        let texels = (width * height) as usize;
        let mut color = Vec::with_capacity(texels * 4);
        let mut normal = Vec::with_capacity(texels * 4);
        let mut occlusion = Vec::with_capacity(texels * 4);
        for row in 0..height {
            // Image rows run downwards while the tile's y axis points up.
            let y = tile_y * (1.0 - (row as f32 + 0.5) / height as f32);
            for column in 0..width {
                let x = tile_x * (column as f32 + 0.5) / width as f32;
                match chart.sample(x, y) {
                    Some(hit) => {
                        color.extend([steel[0], steel[1], steel[2], u8::MAX]);
                        normal.extend(hit.normal.map(signed_unit_to_byte));
                        normal.push(u8::MAX);
                        let visibility = (hit.visibility * 255.0).round() as u8;
                        occlusion.extend([visibility, visibility, visibility, u8::MAX]);
                    }
                    None => {
                        color.extend([0; 4]);
                        normal.extend(FLAT_NORMAL_RGBA);
                        occlusion.extend([u8::MAX; 4]);
                    }
                }
            }
        }
        Ok(Self {
            color_png: encode(&color, width, height)?,
            normal_png: encode(&normal, width, height)?,
            occlusion_png: encode(&occlusion, width, height)?,
        })
    }

    /// Color and alpha are sRGB; normal and occlusion are linear data.
    pub fn textures(&self) -> SurfaceTextures<'_> {
        SurfaceTextures {
            base_color_png: &self.color_png,
            normal_png: &self.normal_png,
            occlusion_png: Some(&self.occlusion_png),
            cutout: true,
        }
    }
}

fn signed_unit_to_byte(component: f32) -> u8 {
    ((component * 0.5 + 0.5) * 255.0).round() as u8
}

fn encode(pixels: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    image::codecs::png::PngEncoder::new(&mut bytes).write_image(
        pixels,
        width,
        height,
        image::ExtendedColorType::Rgba8,
    )?;
    Ok(bytes)
}

struct Ring {
    center: [f32; 2],
    cos: f32,
    /// Signed: alternate rows lean in opposite directions.
    sin: f32,
}

pub(super) struct Chart {
    rings: Vec<Ring>,
    centre_radius: f32,
    wire_radius: f32,
    /// Half extents of one tilted ring's bounding box.
    reach: [f32; 3],
}

pub(super) struct Hit {
    pub(super) normal: [f32; 3],
    pub(super) visibility: f32,
}

impl Chart {
    pub(super) fn new(weave: &MailWeave) -> Self {
        let [column_pitch, _] = weave.repeat_m();
        let row_pitch = weave.row_pitch_m;
        let wire_radius = weave.wire_diameter_m * 0.5;
        let centre_radius = weave.ring_outer_diameter_m * 0.5 - wire_radius;
        let (sin, cos) = weave.ring_tilt_degrees.to_radians().sin_cos();
        let outer = centre_radius + wire_radius;
        let reach = [
            outer,
            outer * cos + wire_radius * sin,
            outer * sin + wire_radius * cos,
        ];
        // Cover the tile and a neighbouring tile on each side, so samples near
        // an edge see every ring that wraps across it.
        let rows = (reach[1] / row_pitch).ceil() as i32 + 2;
        let columns = (reach[0] / column_pitch).ceil() as i32 + 1;
        let mut rings = Vec::new();
        for row in -rows..=4 + rows {
            let odd = row.rem_euclid(2) == 1;
            let shift = if odd { 0.5 } else { 0.0 };
            for column in -columns..=2 + columns {
                rings.push(Ring {
                    center: [
                        (column as f32 + shift) * column_pitch,
                        row as f32 * row_pitch,
                    ],
                    cos,
                    sin: if odd { -sin } else { sin },
                });
            }
        }
        Self {
            rings,
            centre_radius,
            wire_radius,
            reach,
        }
    }

    /// The highest wire surface under an orthographic ray from above.
    pub(super) fn sample(&self, x: f32, y: f32) -> Option<Hit> {
        let tolerance = self.wire_radius * HIT_TOLERANCE_IN_WIRE_RADII;
        let mut best: Option<(f32, &Ring)> = None;
        for ring in &self.rings {
            if (x - ring.center[0]).abs() > self.reach[0]
                || (y - ring.center[1]).abs() > self.reach[1]
            {
                continue;
            }
            let mut z = self.reach[2];
            for _ in 0..TRACE_STEP_LIMIT {
                if z < -self.reach[2] {
                    break;
                }
                let distance = self.ring_distance(ring, [x, y, z]);
                if distance <= tolerance {
                    if best.is_none_or(|(height, _)| z > height) {
                        best = Some((z, ring));
                    }
                    break;
                }
                z -= distance;
            }
        }
        let (height, ring) = best?;
        let point = [x, y, height];
        let normal = self.normal(ring, point);
        Some(Hit {
            normal,
            visibility: self.visibility(point, normal),
        })
    }

    /// Coordinates in the ring's own frame, whose centre circle lies in xy.
    fn local(ring: &Ring, point: [f32; 3]) -> [f32; 3] {
        let [dx, dy, dz] = [
            point[0] - ring.center[0],
            point[1] - ring.center[1],
            point[2],
        ];
        [
            dx,
            dy * ring.cos + dz * ring.sin,
            dz * ring.cos - dy * ring.sin,
        ]
    }

    fn ring_distance(&self, ring: &Ring, point: [f32; 3]) -> f32 {
        let [x, y, z] = Self::local(ring, point);
        let radial = (x * x + y * y).sqrt() - self.centre_radius;
        (radial * radial + z * z).sqrt() - self.wire_radius
    }

    fn distance(&self, point: [f32; 3], margin: f32) -> f32 {
        self.rings
            .iter()
            .filter(|ring| {
                (point[0] - ring.center[0]).abs() <= self.reach[0] + margin
                    && (point[1] - ring.center[1]).abs() <= self.reach[1] + margin
            })
            .map(|ring| self.ring_distance(ring, point))
            .fold(f32::INFINITY, f32::min)
    }

    fn normal(&self, ring: &Ring, point: [f32; 3]) -> [f32; 3] {
        let [x, y, z] = Self::local(ring, point);
        let radial = (x * x + y * y).sqrt().max(f32::MIN_POSITIVE);
        let towards_centre = self.centre_radius / radial;
        let [nx, ny, nz] = normalize([x - x * towards_centre, y - y * towards_centre, z]);
        // Rotate out of the ring frame. The chart is seen from +z, so a grazing
        // hit cannot face away from the viewer.
        normalize([
            nx,
            ny * ring.cos - nz * ring.sin,
            (ny * ring.sin + nz * ring.cos).max(0.0),
        ])
    }

    /// Fraction of the surrounding hemisphere left open by neighbouring wire.
    fn visibility(&self, point: [f32; 3], normal: [f32; 3]) -> f32 {
        let reach = self.wire_radius * OCCLUSION_REACH_IN_WIRE_RADII;
        let occlusion: f32 = (1..=OCCLUSION_PROBES)
            .map(|probe| {
                let offset = reach * probe as f32 / OCCLUSION_PROBES as f32;
                let sample = [0, 1, 2].map(|axis| point[axis] + normal[axis] * offset);
                (offset - self.distance(sample, reach).min(offset)) / offset
            })
            .sum::<f32>()
            / OCCLUSION_PROBES as f32;
        (1.0 - occlusion).clamp(0.0, 1.0)
    }
}

fn normalize(vector: [f32; 3]) -> [f32; 3] {
    let length = vector.iter().map(|c| c * c).sum::<f32>().sqrt();
    if length > f32::MIN_POSITIVE {
        vector.map(|c| c / length)
    } else {
        [0.0, 0.0, 1.0]
    }
}

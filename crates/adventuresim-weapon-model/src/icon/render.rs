//! Supersampled orthographic color rasterizer with interpolated normals and depth.

use super::{IconError, Projection, WeaponIconSpec, studio};
use crate::{Material as RecipeMaterial, MeshPart};
use glam::{Vec2, Vec3};

const CAMERA_YAW_DEGREES: f32 = 12.0;
pub(super) const CAMERA_YAW: f32 = CAMERA_YAW_DEGREES.to_radians();
const MIN_TRIANGLE_AREA: f32 = 1.0e-8;

pub(super) fn render(
    parts: &[MeshPart],
    spec: WeaponIconSpec,
    projection: &Projection,
) -> Result<(Vec<u8>, Vec<u8>), IconError> {
    if !(16..=512).contains(&spec.size) || !(1..=8).contains(&spec.supersampling) {
        return Err(IconError::InvalidSpec);
    }
    let width = usize::from(spec.size) * usize::from(spec.supersampling);
    let mut target = Target {
        width,
        colors: vec![Vec3::ZERO; width * width],
        depth: vec![f32::NEG_INFINITY; width * width],
    };
    let view = Vec3::new(-CAMERA_YAW.sin(), 0.0, CAMERA_YAW.cos());
    for part in parts {
        let material = Material::from(part.material);
        for indices in part.indices.as_chunks::<3>().0 {
            let vertices = indices.map(|index| {
                let position = part.positions[index as usize];
                Vertex {
                    screen: Vec2::from_array(projection.point(position)) * width as f32,
                    depth: Vec3::from_array(position).dot(view),
                    normal: Vec3::from_array(part.normals[index as usize]),
                }
            });
            target.triangle(vertices, &material, view);
        }
    }
    Ok(target.resolve(spec))
}

struct Vertex {
    screen: Vec2,
    depth: f32,
    normal: Vec3,
}
struct Material {
    albedo: Vec3,
    metallic: f32,
    roughness: f32,
}

impl From<RecipeMaterial> for Material {
    fn from(value: RecipeMaterial) -> Self {
        let (color, metallic, roughness) = match value {
            RecipeMaterial::Steel => ([0.62, 0.66, 0.7], 1.0, 0.24),
            RecipeMaterial::DarkSteel => ([0.16, 0.19, 0.22], 1.0, 0.36),
            RecipeMaterial::Brass => ([0.72, 0.43, 0.13], 1.0, 0.28),
            RecipeMaterial::Wood => ([0.26, 0.11, 0.035], 0.0, 0.7),
            RecipeMaterial::Leather => ([0.18, 0.065, 0.025], 0.0, 0.65),
            RecipeMaterial::DarkLeather => ([0.045, 0.022, 0.014], 0.0, 0.7),
            material => (
                material.color().map(|n| n as f32),
                if material.is_metal() { 1.0 } else { 0.0 },
                0.5,
            ),
        };
        Self {
            albedo: Vec3::from_array(color),
            metallic,
            roughness,
        }
    }
}

struct Target {
    width: usize,
    colors: Vec<Vec3>,
    depth: Vec<f32>,
}

impl Target {
    fn triangle(&mut self, vertices: [Vertex; 3], material: &Material, view: Vec3) {
        let [a, b, c] = vertices.map(|v| (v.screen, v.depth, v.normal));
        let area = (b.0 - a.0).perp_dot(c.0 - a.0);
        if area.abs() < MIN_TRIANGLE_AREA {
            return;
        }
        let min = a.0.min(b.0).min(c.0).floor().max(Vec2::ZERO);
        let max =
            a.0.max(b.0)
                .max(c.0)
                .ceil()
                .min(Vec2::splat(self.width as f32 - 1.0));
        for y in min.y as usize..=max.y.max(0.0) as usize {
            for x in min.x as usize..=max.x.max(0.0) as usize {
                let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                let weights = Vec3::new(
                    (b.0 - p).perp_dot(c.0 - p),
                    (c.0 - p).perp_dot(a.0 - p),
                    (a.0 - p).perp_dot(b.0 - p),
                ) / area;
                if weights.min_element() < 0.0 {
                    continue;
                }
                let depth = weights.dot(Vec3::new(a.1, b.1, c.1));
                let index = y * self.width + x;
                if depth <= self.depth[index] {
                    continue;
                }
                let normal =
                    (a.2 * weights.x + b.2 * weights.y + c.2 * weights.z).normalize_or_zero();
                if normal == Vec3::ZERO {
                    continue;
                }
                self.depth[index] = depth;
                self.colors[index] = studio::shade(
                    normal,
                    view,
                    material.albedo,
                    material.metallic,
                    material.roughness,
                );
            }
        }
    }

    fn resolve(self, spec: WeaponIconSpec) -> (Vec<u8>, Vec<u8>) {
        let size = usize::from(spec.size);
        let samples = usize::from(spec.supersampling);
        let mut rgba = Vec::with_capacity(size * size * 4);
        let mut coverage = Vec::with_capacity(size * size);
        for y in 0..size {
            for x in 0..size {
                let mut color = Vec3::ZERO;
                let mut covered = 0;
                for sy in 0..samples {
                    for sx in 0..samples {
                        let index = (y * samples + sy) * self.width + x * samples + sx;
                        color += self.colors[index];
                        covered += usize::from(self.depth[index].is_finite());
                    }
                }
                color /= (samples * samples) as f32;
                rgba.extend([
                    studio::srgb(color.x),
                    studio::srgb(color.y),
                    studio::srgb(color.z),
                    255,
                ]);
                coverage.push((covered * 255 / (samples * samples)) as u8);
            }
        }
        (rgba, coverage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearer_surface_occludes_far_surface_regardless_of_submission_order() {
        let render = |reverse: bool| {
            let mut target = Target {
                width: 4,
                colors: vec![Vec3::ZERO; 16],
                depth: vec![f32::NEG_INFINITY; 16],
            };
            let surfaces = if reverse {
                [(1.0, RecipeMaterial::Brass), (0.0, RecipeMaterial::Leather)]
            } else {
                [(0.0, RecipeMaterial::Leather), (1.0, RecipeMaterial::Brass)]
            };
            for (depth, material) in surfaces {
                let vertices =
                    [Vec2::ZERO, Vec2::new(4.0, 0.0), Vec2::new(0.0, 4.0)].map(|screen| Vertex {
                        screen,
                        depth,
                        normal: Vec3::Z,
                    });
                target.triangle(vertices, &Material::from(material), Vec3::Z);
            }
            target.colors
        };
        assert_eq!(render(false), render(true));
        let color = render(false)[0];
        assert!(
            color.x > color.z,
            "the nearer brass surface retains its warm material"
        );
        assert_eq!(
            render(false)[15],
            Vec3::ZERO,
            "uncovered pixels remain black"
        );
    }
}

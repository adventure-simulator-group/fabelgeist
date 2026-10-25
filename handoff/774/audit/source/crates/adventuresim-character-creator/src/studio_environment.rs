//! The photographic studio the preview's metal reflects.
//!
//! A metal has no diffuse color: what it shows is its surroundings, blurred
//! by its roughness. Lit only by a spotlight and a flat ambient term, steel
//! reads as grey plastic with one highlight. The studio is a dim room with
//! the large soft boxes a product photographer would use: a key above and in
//! front on the spotlight's side, a fill on the other side, an overhead strip
//! and a rim behind. Their reflections give polished steel its bright, sharp
//! bands and rough steel its broad sheen. The cubemap is filtered on the GPU
//! into diffuse and specular maps by `GeneratedEnvironmentMapLight`.

use bevy::{
    asset::RenderAssetUsages,
    image::Image,
    math::Vec3,
    render::render_resource::{
        Extent3d, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension,
    },
};

/// Texels along a cube face edge; a power of two, as the filter requires.
const FACE_SIZE: u32 = 256;

/// Scale from the cubemap's relative radiance to cd/m².
pub const STUDIO_ENVIRONMENT_INTENSITY: f32 = 400.0;

/// A soft rectangular light source seen from the subject.
struct SoftBox {
    /// Toward the box's centre.
    toward: [f32; 3],
    /// Toward the box's long side, perpendicular to `toward`.
    up: [f32; 3],
    /// Half extents across and along, as tangents of the angle subtended.
    half: [f32; 2],
    radiance: [f32; 3],
}

/// The studio's lights. The key sits on the spotlight's side, above and in
/// front of the subject, who faces +Z.
const SOFT_BOXES: [SoftBox; 4] = [
    // Key.
    SoftBox {
        toward: [-2.4, 4.2, 3.0],
        up: [0.0, 1.0, 0.0],
        half: [0.45, 0.6],
        radiance: [16.0, 14.8, 13.4],
    },
    // Fill.
    SoftBox {
        toward: [3.5, 1.2, 1.6],
        up: [0.0, 1.0, 0.0],
        half: [0.5, 0.9],
        radiance: [2.0, 2.2, 2.5],
    },
    // Overhead strip.
    SoftBox {
        toward: [0.0, 1.0, 0.15],
        up: [0.0, 0.0, 1.0],
        half: [0.18, 0.9],
        radiance: [5.0, 5.0, 5.0],
    },
    // Rim, behind the subject.
    SoftBox {
        toward: [1.2, 0.8, -3.0],
        up: [0.0, 1.0, 0.0],
        half: [0.12, 0.8],
        radiance: [7.0, 7.0, 7.4],
    },
];

/// Relative radiance of the studio seen along `direction`.
pub fn studio_radiance(direction: Vec3) -> Vec3 {
    let d = direction.normalize();
    // A dim room: darker grey floor, lighter walls toward the ceiling.
    let height = d.y;
    let room = if height >= 0.0 {
        Vec3::new(0.035, 0.037, 0.042).lerp(Vec3::new(0.06, 0.062, 0.068), height)
    } else {
        Vec3::new(0.03, 0.028, 0.026).lerp(Vec3::new(0.012, 0.012, 0.012), -height)
    };
    SOFT_BOXES.iter().fold(room, |sum, light| {
        let toward = Vec3::from(light.toward).normalize();
        let facing = d.dot(toward);
        if facing <= 0.0 {
            return sum;
        }
        let up = Vec3::from(light.up);
        let up = (up - toward * up.dot(toward)).normalize();
        let across = toward.cross(up);
        // Where the ray meets the box's plane, in tangent units.
        let hit = d / facing;
        let [x, y] = [hit.dot(across) / light.half[0], hit.dot(up) / light.half[1]];
        // A diffuser's edge falls off over its outer tenth.
        let edge = |t: f32| ((1.0 - t.abs()) / 0.1).clamp(0.0, 1.0);
        sum + Vec3::from(light.radiance) * edge(x) * edge(y)
    })
}

/// The direction through a cube face texel, in the wgpu face order
/// +X, -X, +Y, -Y, +Z, -Z.
fn face_direction(face: u32, u: f32, v: f32) -> Vec3 {
    match face {
        0 => Vec3::new(1.0, -v, -u),
        1 => Vec3::new(-1.0, -v, u),
        2 => Vec3::new(u, 1.0, v),
        3 => Vec3::new(u, -1.0, -v),
        4 => Vec3::new(u, -v, 1.0),
        _ => Vec3::new(-u, -v, -1.0),
    }
}

/// The studio as an HDR cubemap.
pub fn studio_environment() -> Image {
    let mut data = Vec::with_capacity((FACE_SIZE * FACE_SIZE * 6 * 16) as usize);
    for face in 0..6 {
        for y in 0..FACE_SIZE {
            for x in 0..FACE_SIZE {
                let coordinate = |i: u32| 2.0 * (i as f32 + 0.5) / FACE_SIZE as f32 - 1.0;
                let radiance = studio_radiance(face_direction(face, coordinate(x), coordinate(y)));
                for channel in [radiance.x, radiance.y, radiance.z, 1.0] {
                    data.extend_from_slice(&channel.to_le_bytes());
                }
            }
        }
    }
    let mut image = Image::new(
        Extent3d {
            width: FACE_SIZE,
            height: FACE_SIZE,
            depth_or_array_layers: 6,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba32Float,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..Default::default()
    });
    image
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_key_outshines_the_room_and_the_floor_is_darkest() {
        let key = studio_radiance(Vec3::new(-2.4, 4.2, 3.0));
        let wall = studio_radiance(Vec3::new(-1.0, 0.0, 0.0));
        let floor = studio_radiance(Vec3::NEG_Y);
        assert!(key.x > 20.0 * wall.x);
        assert!(floor.x < wall.x);
    }

    #[test]
    fn every_face_texel_points_out_of_its_face() {
        let axes = [
            Vec3::X,
            Vec3::NEG_X,
            Vec3::Y,
            Vec3::NEG_Y,
            Vec3::Z,
            Vec3::NEG_Z,
        ];
        for (face, axis) in axes.into_iter().enumerate() {
            for (u, v) in [(-0.9, -0.9), (0.9, 0.3), (0.0, 0.0)] {
                assert_eq!(face_direction(face as u32, u, v).dot(axis), 1.0);
            }
        }
    }
}

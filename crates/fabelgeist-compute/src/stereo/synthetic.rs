//! A scene whose depth is known, photographed through any rig.
//!
//! Footage has no ground truth, so a matcher checked only against footage is
//! checked against how its output looks. This renders a textured scene --
//! spheres, a box, a floor and a dome around all of it -- through a
//! [`StereoRig`] exactly as the rig describes, and says for every rectified
//! pixel how far away the surface is and what disparity it should have. A
//! rectifier with an axis the wrong way round, a triangulation with the wrong
//! triangle, or a projection read in the wrong units all show up as a number.
//!
//! The texture is attached to the surfaces, so both eyes see the same marks
//! from their two places; its scale shrinks with distance from the rig, so
//! near and far surfaces carry detail at about the same size in the picture.
//! The right eye is rendered darker and lifted, as a real pair of cameras
//! seldom agrees on exposure.

use super::rig::{Geometry, Grid, Rectified, StereoRig, dot, length, transform};
use std::f32::consts::PI;

#[derive(Clone, Debug)]
pub struct Scene {
    /// Centre, radius, brightness.
    pub spheres: Vec<([f32; 3], f32, f32)>,
    /// Corner, opposite corner, brightness.
    pub boxes: Vec<([f32; 3], [f32; 3], f32)>,
    /// The floor is the plane `y = floor`, below the rig.
    pub floor: f32,
    /// Everything is inside a sphere this big around the rig.
    pub dome: f32,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            spheres: vec![
                ([-0.55, 0.25, 2.0], 0.45, 0.8),
                ([0.9, -0.25, 3.4], 0.7, 0.45),
                ([0.2, 0.3, -2.6], 0.6, 0.7),
                ([2.6, 0.1, 0.6], 0.5, 0.35),
                ([-3.0, 0.4, -1.0], 0.8, 0.6),
                ([0.0, -1.9, 3.0], 0.5, 0.55),
            ],
            boxes: vec![
                ([-1.7, -0.6, 4.4], [-0.4, 1.6, 5.4], 0.3),
                ([1.2, -0.2, -4.2], [2.6, 1.6, -3.0], 0.5),
            ],
            floor: 1.6,
            dome: 9.0,
        }
    }
}

fn hash(x: i32, y: i32, z: i32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x8da6_b343)
        ^ (y as u32).wrapping_mul(0xd816_3841)
        ^ (z as u32).wrapping_mul(0xcb1a_b31f);
    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1_e995);
    h ^= h >> 15;
    (h & 0x00ff_ffff) as f32 / 0x0100_0000 as f32
}

fn noise(p: [f32; 3]) -> f32 {
    let base = p.map(f32::floor);
    let f = [p[0] - base[0], p[1] - base[1], p[2] - base[2]].map(|t| t * t * (3.0 - 2.0 * t));
    let [x, y, z] = base.map(|v| v as i32);
    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let layer = |dz: i32| {
        let row = |dy: i32| lerp(hash(x, y + dy, z + dz), hash(x + 1, y + dy, z + dz), f[0]);
        lerp(row(0), row(1), f[1])
    };
    lerp(layer(0), layer(1), f[2])
}

impl Scene {
    /// How far along a ray the first surface is, and how bright it is there.
    pub fn trace(&self, origin: [f32; 3], direction: [f32; 3]) -> Option<(f32, f32)> {
        let mut best = f32::INFINITY;
        let mut albedo = 0.5;
        let mut take = |t: f32, a: f32| {
            if t > 1e-4 && t < best {
                best = t;
                albedo = a;
            }
        };
        let sphere = |centre: [f32; 3], radius: f32| -> Option<f32> {
            let oc = [
                origin[0] - centre[0],
                origin[1] - centre[1],
                origin[2] - centre[2],
            ];
            let b = dot(oc, direction);
            let c = dot(oc, oc) - radius * radius;
            let disc = b * b - c;
            if disc < 0.0 {
                return None;
            }
            let root = disc.sqrt();
            let near = -b - root;
            Some(if near > 1e-4 { near } else { -b + root })
        };
        for &(centre, radius, a) in &self.spheres {
            if let Some(t) = sphere(centre, radius) {
                take(t, a);
            }
        }
        for &(low, high, a) in &self.boxes {
            let mut enter = f32::NEG_INFINITY;
            let mut leave = f32::INFINITY;
            for axis in 0..3 {
                let inverse = 1.0 / direction[axis];
                let t0 = (low[axis] - origin[axis]) * inverse;
                let t1 = (high[axis] - origin[axis]) * inverse;
                enter = enter.max(t0.min(t1));
                leave = leave.min(t0.max(t1));
            }
            if leave >= enter.max(0.0) {
                take(enter, a);
            }
        }
        if direction[1] > 1e-6 {
            take((self.floor - origin[1]) / direction[1], 0.65);
        }
        if let Some(t) = sphere([0.0; 3], self.dome) {
            take(t, 0.5);
        }
        if !best.is_finite() {
            return None;
        }
        let p = [
            origin[0] + direction[0] * best,
            origin[1] + direction[1] * best,
            origin[2] + direction[2] * best,
        ];
        let reach = length(p).max(0.3);
        let q = p.map(|v| v * 55.0 / reach);
        let fine = [q[0] * 2.3 + 17.0, q[1] * 2.3 + 5.0, q[2] * 2.3 + 29.0];
        let texture = 0.6 * noise(q) + 0.4 * noise(fine);
        Some((best, 12.0 + 230.0 * (0.35 * albedo + 0.65 * texture)))
    }
}

/// Render `eyes` of a rig into one `width` x `height` luma image.
///
/// Both eyes for a packed frame, or one for an eye with an image of its own.
/// Every pixel is four rays averaged, so the texture does not alias into
/// something the two eyes disagree about.
pub fn render(rig: &StereoRig, scene: &Scene, width: u32, height: u32, eyes: &[usize]) -> Vec<u8> {
    let mut out = vec![0u8; width as usize * height as usize];
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let rows_per = (height as usize).div_ceil(threads).max(1);
    std::thread::scope(|scope| {
        for (band, chunk) in out.chunks_mut(rows_per * width as usize).enumerate() {
            scope.spawn(move || {
                for (i, value) in chunk.iter_mut().enumerate() {
                    let x = (i % width as usize) as f32;
                    let y = (band * rows_per + i / width as usize) as f32;
                    let mut sum = 0.0;
                    let mut hits = 0;
                    for (sx, sy) in [(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)] {
                        let uv = [(x + sx) / width as f32, (y + sy) / height as f32];
                        for &eye in eyes {
                            let Some(direction) = rig.eye(eye).direction(uv) else {
                                continue;
                            };
                            let origin = rig.origin(eye, direction);
                            if let Some((_, luma)) = scene.trace(origin, direction) {
                                sum += if eye == 0 { luma } else { luma * 0.88 + 14.0 };
                                hits += 1;
                            }
                            break;
                        }
                    }
                    if hits > 0 {
                        *value = (sum / hits as f32).round().clamp(0.0, 255.0) as u8;
                    }
                }
            });
        }
    });
    out
}

/// The answer, for every pixel of a rectified grid.
#[derive(Clone, Debug)]
pub struct Truth {
    /// Metres along the pixel's ray -- from the left eye for a parallel rig,
    /// and to the point from the rig's centre for omnidirectional stereo.
    /// NaN where there is no answer.
    pub distance: Vec<f32>,
    /// Pixels between the two eyes' rectified pictures of the same point.
    /// NaN where either eye cannot see it.
    pub disparity: Vec<f32>,
}

pub fn truth(rectified: &Rectified, scene: &Scene) -> Truth {
    let (w, h) = rectified.size();
    let rig = rectified.rig;
    let to_rig = rectified.to_rig();
    let mut distance = vec![f32::NAN; (w * h) as usize];
    let mut disparity = vec![f32::NAN; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let index = (y * w + x) as usize;
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            if rectified.source(0, px, py, (1, 1)).is_none() {
                continue;
            }
            let ray = rectified.ray(px, py);
            let direction = transform(&to_rig, ray);
            let origin = rig.origin(0, direction);
            let Some((t, _)) = scene.trace(origin, direction) else {
                continue;
            };
            let point = match rig.geometry {
                Geometry::Parallel { .. } => ray.map(|v| v * t),
                Geometry::Omnidirectional { .. } => [
                    origin[0] + direction[0] * t,
                    origin[1] + direction[1] * t,
                    origin[2] + direction[2] * t,
                ],
            };
            distance[index] = match rig.geometry {
                Geometry::Parallel { .. } => t,
                Geometry::Omnidirectional { .. } => length(point),
            };
            let Some(right) = rectified.project_point(1, point) else {
                continue;
            };
            if rectified.source(1, right[0], right[1], (1, 1)).is_none() {
                continue;
            }
            let mut apart = px - right[0];
            if let Grid::Panorama { longitude, .. } = rectified.rectification.grid {
                if (longitude[1] - longitude[0]) > 1.9 * PI && apart < -(w as f32) * 0.5 {
                    apart += w as f32;
                }
            }
            disparity[index] = apart;
        }
    }
    Truth {
        distance,
        disparity,
    }
}

/// How close a disparity map came.
#[derive(Clone, Copy, Debug, Default)]
pub struct Score {
    /// Pixels with an answer.
    pub truth_pixels: usize,
    /// Of those, how many the matcher kept.
    pub coverage: f32,
    /// Mean and median absolute disparity error, in pixels, over the kept.
    pub mean_error: f32,
    pub median_error: f32,
    /// The fraction of kept pixels more than one and two pixels out.
    pub bad_1: f32,
    pub bad_2: f32,
    /// Median of |range - truth| / truth, over kept pixels nearer than 30 m.
    pub median_range_error: f32,
}

pub fn score(truth: &Truth, disparity: &[f32], distance: &[f32]) -> Score {
    let mut errors = Vec::new();
    let mut ranges = Vec::new();
    let mut truth_pixels = 0;
    for (index, expected) in truth.disparity.iter().enumerate() {
        if !expected.is_finite() {
            continue;
        }
        truth_pixels += 1;
        let found = disparity[index];
        if found < 0.0 {
            continue;
        }
        errors.push((found - expected).abs());
        let real = truth.distance[index];
        if real.is_finite() && real < 30.0 && distance[index] > 0.0 {
            ranges.push((distance[index] - real).abs() / real);
        }
    }
    let median = |values: &mut Vec<f32>| {
        if values.is_empty() {
            return f32::NAN;
        }
        values.sort_by(f32::total_cmp);
        values[values.len() / 2]
    };
    let kept = errors.len().max(1) as f32;
    Score {
        truth_pixels,
        coverage: errors.len() as f32 / truth_pixels.max(1) as f32,
        mean_error: errors.iter().sum::<f32>() / kept,
        bad_1: errors.iter().filter(|e| **e > 1.0).count() as f32 / kept,
        bad_2: errors.iter().filter(|e| **e > 2.0).count() as f32 / kept,
        median_error: median(&mut errors),
        median_range_error: median(&mut ranges),
    }
}

/// The answer for one eye's own picture, `size` pixels over its region:
/// metres from that eye's centre along each pixel's ray.
///
/// NaN where the ray has no picture, leaves the rectified grid, hits nothing,
/// or reaches a surface the left eye cannot see -- depth measured from the
/// left eye has nothing to say about those, so they are not asked of it.
pub fn view_truth(rectified: &Rectified, scene: &Scene, eye: usize, size: (u32, u32)) -> Vec<f32> {
    use super::rig::{normalize, transpose};
    let rig = rectified.rig;
    let this = rig.eye(eye);
    let to_rig = rectified.to_rig();
    let from_rig = transpose(&to_rig);
    let [u, v, rw, rh] = this.region;
    let (w, h) = size;
    let minus = |a: [f32; 3], b: [f32; 3]| [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    let mut out = vec![f32::NAN; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let uv = [
                u + (x as f32 + 0.5) / w as f32 * rw,
                v + (y as f32 + 0.5) / h as f32 * rh,
            ];
            let Some(direction) = this.direction(uv) else {
                continue;
            };
            if rectified.pixel(transform(&from_rig, direction)).is_none() {
                continue;
            }
            let origin = rig.origin(eye, direction);
            let Some((t, _)) = scene.trace(origin, direction) else {
                continue;
            };
            if eye == 1 {
                let point = [
                    origin[0] + direction[0] * t,
                    origin[1] + direction[1] * t,
                    origin[2] + direction[2] * t,
                ];
                let in_grid = match rig.geometry {
                    Geometry::Parallel { .. } => {
                        transform(&from_rig, minus(point, rig.origin(0, direction)))
                    }
                    Geometry::Omnidirectional { .. } => point,
                };
                let Some(left) = rectified.project_point(0, in_grid) else {
                    continue;
                };
                let left_direction = transform(&to_rig, rectified.ray(left[0], left[1]));
                let left_origin = rig.origin(0, left_direction);
                let reach = length(minus(point, left_origin));
                match scene.trace(left_origin, normalize(minus(point, left_origin))) {
                    Some((seen, _)) if seen >= reach * 0.99 - 1e-3 => {}
                    _ => continue,
                }
            }
            out[(y * w + x) as usize] = t;
        }
    }
    out
}

/// How close an eye's own view came.
#[derive(Clone, Copy, Debug, Default)]
pub struct ViewScore {
    /// Pixels with an answer.
    pub truth_pixels: usize,
    /// Of those, how many have depth.
    pub coverage: f32,
    /// Median of |distance - truth| / truth, over pixels with both.
    pub median_relative_error: f32,
    /// The fraction of those more than five per cent out.
    pub bad_5: f32,
}

pub fn score_view(truth: &[f32], distance: &[f32]) -> ViewScore {
    let mut errors = Vec::new();
    let mut truth_pixels = 0;
    for (expected, found) in truth.iter().zip(distance) {
        if !expected.is_finite() {
            continue;
        }
        truth_pixels += 1;
        if *found > 0.0 {
            errors.push((found - expected).abs() / expected);
        }
    }
    let kept = errors.len().max(1) as f32;
    let bad_5 = errors.iter().filter(|e| **e > 0.05).count() as f32 / kept;
    errors.sort_by(f32::total_cmp);
    ViewScore {
        truth_pixels,
        coverage: errors.len() as f32 / truth_pixels.max(1) as f32,
        median_relative_error: errors.get(errors.len() / 2).copied().unwrap_or(f32::NAN),
        bad_5,
    }
}

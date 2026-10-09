//! Where each eye's picture is, how it was projected, and the grid both eyes
//! are resampled onto so that a match is a search along a row.
//!
//! A stereo frame is described in three independent parts, because real files
//! mix them freely:
//!
//! * [`Eye`] -- which part of the frame holds the eye's picture, the
//!   [`Projection`] that picture was taken through, and how the eye's camera
//!   is turned inside the rig. Side-by-side, top-bottom, swapped or two
//!   separate files are all just regions.
//! * [`Geometry`] -- where the two viewpoints are. Two lenses a baseline apart
//!   is [`Geometry::Parallel`]; a 360 video stitched for a head that turns is
//!   [`Geometry::Omnidirectional`], where every direction was seen from its own
//!   pair of points on a circle.
//! * [`Grid`] -- what the rectified picture looks like. A pinhole grid is the
//!   textbook one and keeps `Z = f B / d`; an [`Grid::Epipolar`] grid measures
//!   angles about the baseline and covers a whole hemisphere; a
//!   [`Grid::Panorama`] grid is the one omnidirectional stereo is already
//!   rectified in.
//!
//! Coordinates everywhere are `x` right, `y` down, `z` forward, in metres and
//! radians. The rectified ("grid") frame has its `x` axis along the baseline,
//! and a point in it is measured from the left eye for a parallel rig and from
//! the rig's centre for an omnidirectional one.

use anyhow::{Result, anyhow, ensure};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

/// A 3x3 matrix, by rows.
pub type Matrix3 = [[f32; 3]; 3];

/// The identity rotation.
pub const IDENTITY: Matrix3 = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

pub fn transform(m: &Matrix3, v: [f32; 3]) -> [f32; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

pub fn multiply(a: &Matrix3, b: &Matrix3) -> Matrix3 {
    let mut out = [[0.0; 3]; 3];
    for (row, line) in out.iter_mut().enumerate() {
        for (column, value) in line.iter_mut().enumerate() {
            *value = (0..3).map(|k| a[row][k] * b[k][column]).sum();
        }
    }
    out
}

pub fn transpose(m: &Matrix3) -> Matrix3 {
    [
        [m[0][0], m[1][0], m[2][0]],
        [m[0][1], m[1][1], m[2][1]],
        [m[0][2], m[1][2], m[2][2]],
    ]
}

/// Turning about `x`. Positive tips the view up: forward goes to `-y`.
pub fn rotation_x(angle: f32) -> Matrix3 {
    let (s, c) = angle.sin_cos();
    [[1.0, 0.0, 0.0], [0.0, c, -s], [0.0, s, c]]
}

/// Turning about `y`. Positive turns the view right: forward goes to `+x`.
pub fn rotation_y(angle: f32) -> Matrix3 {
    let (s, c) = angle.sin_cos();
    [[c, 0.0, s], [0.0, 1.0, 0.0], [-s, 0.0, c]]
}

/// Turning about `z`, the optical axis.
pub fn rotation_z(angle: f32) -> Matrix3 {
    let (s, c) = angle.sin_cos();
    [[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]]
}

pub fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub fn length(v: [f32; 3]) -> f32 {
    dot(v, v).sqrt()
}

pub fn normalize(v: [f32; 3]) -> [f32; 3] {
    let l = length(v).max(1e-20);
    [v[0] / l, v[1] / l, v[2] / l]
}

fn scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// A direction from longitude (right of forward) and latitude (above level).
pub fn direction_from_angles(longitude: f32, latitude: f32) -> [f32; 3] {
    [
        latitude.cos() * longitude.sin(),
        -latitude.sin(),
        latitude.cos() * longitude.cos(),
    ]
}

/// How a fisheye lens spends its image circle on angles off its axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FisheyeModel {
    /// `r = f θ`, which most action and VR180 lenses are close to.
    Equidistant,
    /// `r = 2 f sin(θ/2)`, equal area.
    Equisolid,
    /// `r = 2 f tan(θ/2)`, which keeps shapes.
    Stereographic,
    /// `r = f sin θ`, which cannot reach past ninety degrees.
    Orthographic,
}

impl FisheyeModel {
    pub const ALL: [Self; 4] = [
        Self::Equidistant,
        Self::Equisolid,
        Self::Stereographic,
        Self::Orthographic,
    ];

    pub(crate) fn index(self) -> u32 {
        self as u32
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Equidistant => "equidistant",
            Self::Equisolid => "equisolid",
            Self::Stereographic => "stereographic",
            Self::Orthographic => "orthographic",
        }
    }

    /// The radius, in focal lengths, of a ray `theta` off the axis.
    pub fn radius(self, theta: f32) -> f32 {
        match self {
            Self::Equidistant => theta,
            Self::Equisolid => 2.0 * (theta * 0.5).sin(),
            Self::Stereographic => 2.0 * (theta * 0.5).tan(),
            Self::Orthographic => theta.sin(),
        }
    }

    /// The angle off the axis of a ray at `radius` focal lengths.
    pub fn angle(self, radius: f32) -> f32 {
        match self {
            Self::Equidistant => radius,
            Self::Equisolid => 2.0 * (radius * 0.5).clamp(-1.0, 1.0).asin(),
            Self::Stereographic => 2.0 * (radius * 0.5).atan(),
            Self::Orthographic => radius.clamp(-1.0, 1.0).asin(),
        }
    }
}

/// How one eye's picture maps directions to places in its region.
///
/// Every length is a fraction of the eye's own region, so a description holds
/// whatever resolution the file was encoded at.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Projection {
    /// A rectilinear lens, with Brown-Conrady distortion
    /// `[k1, k2, p1, p2, k3]` applied to the normalised image point.
    Pinhole {
        focal: [f32; 2],
        principal: [f32; 2],
        distortion: [f32; 5],
    },
    /// Longitude across, latitude down, over the ranges given as
    /// `[min, max]`. A full sphere is `[-π, π]` by `[-π/2, π/2]`.
    Equirectangular {
        longitude: [f32; 2],
        latitude: [f32; 2],
    },
    /// An image circle of `field_of_view` across, centred at `centre` with
    /// radius `radius` (both fractions of the region, per axis).
    Fisheye {
        model: FisheyeModel,
        field_of_view: f32,
        centre: [f32; 2],
        radius: [f32; 2],
    },
}

impl Projection {
    /// An undistorted pinhole with its principal point in the middle.
    /// `aspect` is the region's width over its height, in pixels.
    pub fn pinhole(horizontal_fov: f32, aspect: f32) -> Self {
        let fx = 0.5 / (horizontal_fov * 0.5).tan();
        Self::Pinhole {
            focal: [fx, fx * aspect],
            principal: [0.5, 0.5],
            distortion: [0.0; 5],
        }
    }

    /// An equirectangular picture centred straight ahead.
    pub fn equirectangular(horizontal: f32, vertical: f32) -> Self {
        Self::Equirectangular {
            longitude: [-horizontal * 0.5, horizontal * 0.5],
            latitude: [-vertical * 0.5, vertical * 0.5],
        }
    }

    /// A fisheye whose image circle fills its region.
    pub fn fisheye(model: FisheyeModel, field_of_view: f32) -> Self {
        Self::Fisheye {
            model,
            field_of_view,
            centre: [0.5, 0.5],
            radius: [0.5, 0.5],
        }
    }

    /// Where a direction in the eye's camera lands in its region, as a
    /// fraction of it, or None where the picture does not cover it.
    pub fn project(&self, d: [f32; 3]) -> Option<[f32; 2]> {
        let uv = match *self {
            Self::Pinhole {
                focal,
                principal,
                distortion: [k1, k2, p1, p2, k3],
            } => {
                if d[2] <= 1e-6 {
                    return None;
                }
                let (x, y) = (d[0] / d[2], d[1] / d[2]);
                let r2 = x * x + y * y;
                let radial = 1.0 + k1 * r2 + k2 * r2 * r2 + k3 * r2 * r2 * r2;
                let xd = x * radial + 2.0 * p1 * x * y + p2 * (r2 + 2.0 * x * x);
                let yd = y * radial + p1 * (r2 + 2.0 * y * y) + 2.0 * p2 * x * y;
                [principal[0] + focal[0] * xd, principal[1] + focal[1] * yd]
            }
            Self::Equirectangular {
                longitude,
                latitude,
            } => {
                let lon = d[0].atan2(d[2]);
                let lon = longitude[0] + ((lon - longitude[0]) / TAU).rem_euclid(1.0) * TAU;
                let lat = (-d[1]).clamp(-1.0, 1.0).asin();
                [
                    (lon - longitude[0]) / (longitude[1] - longitude[0]),
                    (latitude[1] - lat) / (latitude[1] - latitude[0]),
                ]
            }
            Self::Fisheye {
                model,
                field_of_view,
                centre,
                radius,
            } => {
                let half = field_of_view * 0.5;
                let theta = d[2].clamp(-1.0, 1.0).acos();
                // A hair of slack, so the rim of the circle survives rounding.
                if theta > half + 1e-4 {
                    return None;
                }
                let phi = d[1].atan2(d[0]);
                let g = model.radius(theta) / model.radius(half);
                [
                    centre[0] + radius[0] * g * phi.cos(),
                    centre[1] + radius[1] * g * phi.sin(),
                ]
            }
        };
        (uv[0] >= 0.0 && uv[0] <= 1.0 && uv[1] >= 0.0 && uv[1] <= 1.0).then_some(uv)
    }

    /// The direction a place in the region was seen along, in the eye's
    /// camera, or None where the region holds no picture.
    pub fn unproject(&self, uv: [f32; 2]) -> Option<[f32; 3]> {
        match *self {
            Self::Pinhole {
                focal,
                principal,
                distortion: [k1, k2, p1, p2, k3],
            } => {
                let xd = (uv[0] - principal[0]) / focal[0];
                let yd = (uv[1] - principal[1]) / focal[1];
                // Distortion has no closed-form inverse; the fixed point
                // converges in a handful of steps for any lens that is not
                // folding its own picture over.
                let (mut x, mut y) = (xd, yd);
                for _ in 0..30 {
                    let r2 = x * x + y * y;
                    let radial = 1.0 + k1 * r2 + k2 * r2 * r2 + k3 * r2 * r2 * r2;
                    let dx = 2.0 * p1 * x * y + p2 * (r2 + 2.0 * x * x);
                    let dy = p1 * (r2 + 2.0 * y * y) + 2.0 * p2 * x * y;
                    x = (xd - dx) / radial;
                    y = (yd - dy) / radial;
                }
                Some(normalize([x, y, 1.0]))
            }
            Self::Equirectangular {
                longitude,
                latitude,
            } => {
                let lon = longitude[0] + uv[0] * (longitude[1] - longitude[0]);
                let lat = latitude[1] - uv[1] * (latitude[1] - latitude[0]);
                Some(direction_from_angles(lon, lat))
            }
            Self::Fisheye {
                model,
                field_of_view,
                centre,
                radius,
            } => {
                let dx = (uv[0] - centre[0]) / radius[0];
                let dy = (uv[1] - centre[1]) / radius[1];
                let g = (dx * dx + dy * dy).sqrt();
                if g > 1.0 {
                    return None;
                }
                let theta = model.angle(g * model.radius(field_of_view * 0.5));
                let phi = dy.atan2(dx);
                Some([
                    theta.sin() * phi.cos(),
                    theta.sin() * phi.sin(),
                    theta.cos(),
                ])
            }
        }
    }

    /// The projection as the shader reads it: a kind and three rows of numbers.
    pub(crate) fn uniforms(&self) -> (u32, [f32; 4], [f32; 4], [f32; 4]) {
        match *self {
            Self::Pinhole {
                focal,
                principal,
                distortion: [k1, k2, p1, p2, k3],
            } => (
                0,
                [focal[0], focal[1], principal[0], principal[1]],
                [k1, k2, p1, p2],
                [k3, 0.0, 0.0, 0.0],
            ),
            Self::Equirectangular {
                longitude,
                latitude,
            } => (
                1,
                [longitude[0], longitude[1], latitude[0], latitude[1]],
                [0.0; 4],
                [0.0; 4],
            ),
            Self::Fisheye {
                model,
                field_of_view,
                centre,
                radius,
            } => {
                let half = field_of_view * 0.5;
                (
                    2,
                    [centre[0], centre[1], radius[0], radius[1]],
                    [half, model.index() as f32, model.radius(half), 0.0],
                    [0.0; 4],
                )
            }
        }
    }
}

/// One eye of a stereo frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Eye {
    /// Where the eye's picture is in its image, as `[u, v, width, height]`
    /// fractions. The whole image for an eye that has a file of its own.
    pub region: [f32; 4],
    pub projection: Projection,
    /// From the eye's camera to the rig: `d_rig = rotation · d_eye`.
    pub rotation: Matrix3,
}

impl Eye {
    pub fn new(region: [f32; 4], projection: Projection) -> Self {
        Self {
            region,
            projection,
            rotation: IDENTITY,
        }
    }

    pub fn with_rotation(mut self, rotation: Matrix3) -> Self {
        self.rotation = rotation;
        self
    }

    /// The direction, in the rig, that a place in the eye's image was seen
    /// along. `uv` is a fraction of the whole image, not of the region.
    pub fn direction(&self, uv: [f32; 2]) -> Option<[f32; 3]> {
        let [u, v, w, h] = self.region;
        let local = [(uv[0] - u) / w, (uv[1] - v) / h];
        if !(0.0..=1.0).contains(&local[0]) || !(0.0..=1.0).contains(&local[1]) {
            return None;
        }
        self.projection
            .unproject(local)
            .map(|d| transform(&self.rotation, d))
    }

    /// Where in the eye's image a direction in the rig lands, as a fraction of
    /// the whole image.
    pub fn place(&self, direction: [f32; 3]) -> Option<[f32; 2]> {
        let local = self
            .projection
            .project(transform(&transpose(&self.rotation), direction))?;
        let [u, v, w, h] = self.region;
        Some([u + local[0] * w, v + local[1] * h])
    }
}

/// Where the two viewpoints are.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Geometry {
    /// Two centres of projection. `baseline` goes from the left eye to the
    /// right one, in the rig, in metres; the rig's origin is halfway between.
    Parallel { baseline: [f32; 3] },
    /// Omnidirectional stereo: every direction was seen from the two ends of a
    /// diameter of a level circle `ipd` across, turned to face it.
    Omnidirectional { ipd: f32 },
}

impl Geometry {
    pub fn baseline_length(&self) -> f32 {
        match *self {
            Self::Parallel { baseline } => length(baseline),
            Self::Omnidirectional { ipd } => ipd,
        }
    }

    pub(crate) fn index(&self) -> u32 {
        match self {
            Self::Parallel { .. } => 0,
            Self::Omnidirectional { .. } => 1,
        }
    }
}

/// A stereo frame's whole description.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StereoRig {
    pub left: Eye,
    pub right: Eye,
    pub geometry: Geometry,
}

impl StereoRig {
    pub fn eye(&self, index: usize) -> &Eye {
        if index == 0 { &self.left } else { &self.right }
    }

    /// Where, in the rig, a ray seen by `eye` along `direction` starts.
    pub fn origin(&self, eye: usize, direction: [f32; 3]) -> [f32; 3] {
        let side = if eye == 0 { -0.5 } else { 0.5 };
        match self.geometry {
            Geometry::Parallel { baseline } => scale(baseline, side),
            Geometry::Omnidirectional { ipd } => {
                let longitude = direction[0].atan2(direction[2]);
                scale([longitude.cos(), 0.0, -longitude.sin()], side * ipd)
            }
        }
    }
}

/// The picture both eyes are resampled onto.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Grid {
    /// A rectilinear picture, principal point in the middle, square pixels.
    /// Only for a parallel rig, and only for less than half the sphere.
    Pinhole { horizontal_fov: f32 },
    /// Angles about the baseline: across is the angle toward it, down the
    /// rows is the turn about it. Every row is an epipolar plane, so this
    /// rectifies a parallel rig over up to a whole sphere. `horizontal` is
    /// `[left, right]` and `vertical` is `[top, bottom]`, up positive.
    Epipolar {
        horizontal: [f32; 2],
        vertical: [f32; 2],
    },
    /// Longitude and latitude, which is what omnidirectional stereo is
    /// already rectified in. `longitude` is `[left, right]` and `latitude` is
    /// `[top, bottom]`.
    Panorama {
        longitude: [f32; 2],
        latitude: [f32; 2],
    },
}

impl Grid {
    pub fn label(&self) -> String {
        let d = f32::to_degrees;
        match *self {
            Self::Pinhole { horizontal_fov } => format!("pinhole {:.0}°", d(horizontal_fov)),
            Self::Epipolar {
                horizontal,
                vertical,
            } => format!(
                "epipolar {:.0}°x{:.0}°",
                d(horizontal[1] - horizontal[0]),
                d(vertical[0] - vertical[1])
            ),
            Self::Panorama {
                longitude,
                latitude,
            } => format!(
                "panorama {:.0}°x{:.0}°",
                d(longitude[1] - longitude[0]),
                d(latitude[0] - latitude[1])
            ),
        }
    }
}

/// A grid, how many pixels it has, and how far it is tipped about the
/// baseline -- which is the only way a rectified view can turn and keep its
/// rows on epipolar lines.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rectification {
    pub grid: Grid,
    pub width: u32,
    pub height: u32,
    pub pitch: f32,
}

/// A rig and a grid, with the rotations between them solved.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rectified {
    pub rig: StereoRig,
    pub rectification: Rectification,
    /// From the grid frame to the rig.
    to_rig: Matrix3,
}

impl Rectified {
    pub fn new(rig: StereoRig, rectification: Rectification) -> Result<Self> {
        ensure!(
            rectification.width >= 8 && rectification.height >= 8,
            "a rectified grid of {}x{} is too small to match in",
            rectification.width,
            rectification.height
        );
        let to_rig = match (rig.geometry, rectification.grid) {
            (Geometry::Parallel { baseline }, Grid::Pinhole { .. } | Grid::Epipolar { .. }) => {
                ensure!(length(baseline) > 0.0, "the baseline has no length");
                if let Grid::Pinhole { horizontal_fov } = rectification.grid {
                    ensure!(
                        horizontal_fov > 0.0 && horizontal_fov < PI * 0.95,
                        "a pinhole grid cannot see {:.0} degrees across",
                        horizontal_fov.to_degrees()
                    );
                }
                let x = normalize(baseline);
                // Forward is whatever both eyes agree on, with the part along
                // the baseline taken out: that is the one freedom
                // rectification leaves, and splitting it evenly between the
                // eyes keeps the most of both pictures.
                let ahead = normalize(
                    [0, 1]
                        .map(|eye| transform(&rig.eye(eye).rotation, [0.0, 0.0, 1.0]))
                        .into_iter()
                        .fold([0.0; 3], |sum, d| {
                            [sum[0] + d[0], sum[1] + d[1], sum[2] + d[2]]
                        }),
                );
                let off_axis = sub(ahead, scale(x, dot(ahead, x)));
                ensure!(
                    length(off_axis) > 1e-3,
                    "the eyes look along the baseline, so no picture can be rectified"
                );
                let z = normalize(off_axis);
                let y = cross(z, x);
                let rectify = [[x[0], y[0], z[0]], [x[1], y[1], z[1]], [x[2], y[2], z[2]]];
                multiply(&rectify, &rotation_x(rectification.pitch))
            }
            (Geometry::Omnidirectional { ipd }, Grid::Panorama { .. }) => {
                ensure!(ipd > 0.0, "the interpupillary distance has no length");
                ensure!(
                    rectification.pitch == 0.0,
                    "an omnidirectional panorama is level by construction; move its latitude range instead of pitching it"
                );
                IDENTITY
            }
            (geometry, grid) => {
                return Err(anyhow!(
                    "a {} grid does not rectify {} stereo",
                    grid.label(),
                    match geometry {
                        Geometry::Parallel { .. } => "parallel",
                        Geometry::Omnidirectional { .. } => "omnidirectional",
                    }
                ));
            }
        };
        let (w, h) = (rectification.width as f64, rectification.height as f64);
        ensure!(
            w * h * 256.0 < u32::MAX as f64,
            "a rectified grid of {w}x{h} cannot be indexed on the card"
        );
        Ok(Self {
            rig,
            rectification,
            to_rig,
        })
    }

    pub fn size(&self) -> (u32, u32) {
        (self.rectification.width, self.rectification.height)
    }

    pub fn to_rig(&self) -> Matrix3 {
        self.to_rig
    }

    /// Metres between the viewpoints.
    pub fn baseline(&self) -> f32 {
        self.rig.geometry.baseline_length()
    }

    /// The grid as the shader reads it: a kind, and four numbers.
    pub(crate) fn grid_uniforms(&self) -> (u32, [f32; 4]) {
        let (w, h) = (
            self.rectification.width as f32,
            self.rectification.height as f32,
        );
        match self.rectification.grid {
            Grid::Pinhole { horizontal_fov } => {
                let f = w * 0.5 / (horizontal_fov * 0.5).tan();
                (0, [f, w * 0.5, h * 0.5, 0.0])
            }
            Grid::Epipolar {
                horizontal,
                vertical,
            } => (1, [horizontal[0], horizontal[1], vertical[0], vertical[1]]),
            Grid::Panorama {
                longitude,
                latitude,
            } => (2, [longitude[0], longitude[1], latitude[0], latitude[1]]),
        }
    }

    /// How much one pixel of disparity is: a tangent for a pinhole grid, an
    /// angle for the others.
    pub fn disparity_step(&self) -> f32 {
        let (_, g) = self.grid_uniforms();
        match self.rectification.grid {
            Grid::Pinhole { .. } => 1.0 / g[0],
            _ => (g[1] - g[0]) / self.rectification.width as f32,
        }
    }

    /// The ray through a place in the grid, in the grid frame. Pixel centres
    /// are at half-integers.
    pub fn ray(&self, x: f32, y: f32) -> [f32; 3] {
        let (w, h) = (
            self.rectification.width as f32,
            self.rectification.height as f32,
        );
        let (_, g) = self.grid_uniforms();
        match self.rectification.grid {
            Grid::Pinhole { .. } => normalize([(x - g[1]) / g[0], (y - g[2]) / g[0], 1.0]),
            Grid::Epipolar { .. } => {
                let beta = g[0] + (g[1] - g[0]) * x / w;
                let alpha = -(g[2] + (g[3] - g[2]) * y / h);
                [
                    beta.sin(),
                    beta.cos() * alpha.sin(),
                    beta.cos() * alpha.cos(),
                ]
            }
            Grid::Panorama { .. } => {
                direction_from_angles(g[0] + (g[1] - g[0]) * x / w, g[2] + (g[3] - g[2]) * y / h)
            }
        }
    }

    /// Where a direction in the grid frame lands on the grid, if it does.
    pub fn pixel(&self, d: [f32; 3]) -> Option<[f32; 2]> {
        let (w, h) = (
            self.rectification.width as f32,
            self.rectification.height as f32,
        );
        let (_, g) = self.grid_uniforms();
        let p = match self.rectification.grid {
            Grid::Pinhole { .. } => {
                if d[2] <= 1e-6 {
                    return None;
                }
                [g[1] + g[0] * d[0] / d[2], g[2] + g[0] * d[1] / d[2]]
            }
            Grid::Epipolar { .. } => {
                let beta = d[0].clamp(-1.0, 1.0).asin();
                let vertical = -d[1].atan2(d[2]);
                [
                    (beta - g[0]) / (g[1] - g[0]) * w,
                    (vertical - g[2]) / (g[3] - g[2]) * h,
                ]
            }
            Grid::Panorama { .. } => {
                let lon = d[0].atan2(d[2]);
                let lon = g[0] + ((lon - g[0]) / TAU).rem_euclid(1.0) * TAU;
                let lat = (-d[1]).clamp(-1.0, 1.0).asin();
                [
                    (lon - g[0]) / (g[1] - g[0]) * w,
                    (lat - g[2]) / (g[3] - g[2]) * h,
                ]
            }
        };
        (p[0] >= 0.0 && p[0] < w && p[1] >= 0.0 && p[1] < h).then_some(p)
    }

    /// Where a point in the grid frame appears in `eye`'s rectified picture.
    ///
    /// The difference between the two eyes' answers along the row is the
    /// disparity the matcher is looking for, which is how a ground truth for
    /// it is worked out.
    pub fn project_point(&self, eye: usize, point: [f32; 3]) -> Option<[f32; 2]> {
        match self.rig.geometry {
            Geometry::Parallel { .. } => {
                let from = if eye == 0 {
                    point
                } else {
                    sub(point, [self.baseline(), 0.0, 0.0])
                };
                self.pixel(normalize(from))
            }
            Geometry::Omnidirectional { ipd } => {
                let half = ipd * 0.5;
                let level = (point[0] * point[0] + point[2] * point[2]).sqrt();
                if level <= half {
                    return None;
                }
                let turn = (half / level).asin();
                let longitude = point[0].atan2(point[2]) + if eye == 0 { turn } else { -turn };
                let reach = (level * level - half * half).sqrt();
                let latitude = (-point[1]).atan2(reach);
                self.pixel(direction_from_angles(longitude, latitude))
            }
        }
    }

    /// The rotation from the grid frame to `eye`'s camera.
    pub(crate) fn eye_from_grid(&self, eye: usize) -> Matrix3 {
        multiply(&transpose(&self.rig.eye(eye).rotation), &self.to_rig)
    }

    /// Where in `eye`'s image a grid pixel is read from, in that image's
    /// pixels.
    pub fn source(&self, eye: usize, x: f32, y: f32, image: (u32, u32)) -> Option<[f32; 2]> {
        let d = transform(&self.to_rig, self.ray(x, y));
        let uv = self.rig.eye(eye).place(d)?;
        Some([uv[0] * image.0 as f32, uv[1] * image.1 as f32])
    }
}

/// Up to a right angle, as a convenience for the ranges above.
pub const RIGHT_ANGLE: f32 = FRAC_PI_2;

//! A structure: the solids it is made of and the mesh that draws them, built
//! up together a piece at a time so that the two agree.

use crate::mesh::{Material, Mesh};
use crate::solid::{Shape, Solid};

/// What a [`Feature`] is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FeatureKind {
    Tower,
    Keep,
    Wall,
    Gatehouse,
    House,
    Hall,
}

/// A building of a structure, for whatever is placed around it: where it
/// stands and how far it reaches.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Feature {
    pub kind: FeatureKind,
    /// The middle of its foot.
    pub base: [f32; 3],
    /// Half its extent across, and along -- for a wall, from end to end.
    pub half: [f32; 2],
    pub yaw: f32,
    pub height: f32,
    /// A drum rather than a block.
    pub round: bool,
}

impl Feature {
    /// Points around its foot, `out` metres from its faces, `spacing` apart:
    /// where something planted against it would stand.
    pub fn around(&self, out: f32, spacing: f32) -> Vec<[f32; 3]> {
        let [a, b] = [self.half[0] + out, self.half[1] + out];
        let frame = Solid::new(Shape::Box, self.base, [0.0; 3], self.yaw);
        let mut points = Vec::new();
        match self.kind {
            _ if self.round => {
                // Round: around the drum.
                let count = ((std::f32::consts::TAU * a / spacing).round() as usize).max(3);
                for i in 0..count {
                    let t = std::f32::consts::TAU * i as f32 / count as f32;
                    points.push(frame.world([a * t.cos(), 0.0, a * t.sin()]));
                }
            }
            _ => {
                let corners = [[-a, -b], [a, -b], [a, b], [-a, b]];
                for k in 0..4 {
                    let (p, q) = (corners[k], corners[(k + 1) % 4]);
                    let length = ((q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2)).sqrt();
                    let count = ((length / spacing).round() as usize).max(1);
                    for i in 0..count {
                        let t = i as f32 / count as f32;
                        points.push(frame.world([
                            p[0] + (q[0] - p[0]) * t,
                            0.0,
                            p[1] + (q[1] - p[1]) * t,
                        ]));
                    }
                }
            }
        }
        points
    }
}

/// Solids, the mesh drawing them, and the buildings they make up.
#[derive(Clone, Debug, Default)]
pub struct Structure {
    pub solids: Vec<Solid>,
    pub mesh: Mesh,
    pub features: Vec<Feature>,
}

/// Sides of a round solid's mesh.
pub(crate) const ROUND_SIDES: usize = 28;

impl Structure {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn merge(&mut self, other: Structure) {
        self.solids.extend(other.solids);
        self.mesh.append(&other.mesh);
        self.features.extend(other.features);
    }

    /// Signed distance to the union of the solids, visiting every one.
    pub fn distance(&self, p: [f32; 3]) -> f32 {
        self.solids
            .iter()
            .map(|s| s.distance(p))
            .fold(f32::INFINITY, f32::min)
    }

    /// Lower and upper corner of everything in it.
    pub fn bounds(&self) -> [[f32; 3]; 2] {
        let mut lo = [f32::INFINITY; 3];
        let mut hi = [f32::NEG_INFINITY; 3];
        for p in &self.mesh.positions {
            for k in 0..3 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        }
        [lo, hi]
    }

    /// A box, solid and drawn. `skip_bottom` leaves its underside undrawn,
    /// where it stands on something.
    pub fn block(
        &mut self,
        base: [f32; 3],
        half: [f32; 2],
        height: f32,
        yaw: f32,
        material: Material,
        tint: f32,
    ) -> Solid {
        let solid = Solid::new(Shape::Box, base, [half[0], height, half[1]], yaw);
        self.solids.push(solid);
        self.draw_block(&solid, material, tint);
        solid
    }

    /// A box drawn but not solid: trim, too small to matter to anything
    /// growing past it.
    pub fn trim(
        &mut self,
        base: [f32; 3],
        half: [f32; 2],
        height: f32,
        yaw: f32,
        material: Material,
        tint: f32,
    ) {
        let solid = Solid::new(Shape::Box, base, [half[0], height, half[1]], yaw);
        self.draw_block(&solid, material, tint);
    }

    fn draw_block(&mut self, solid: &Solid, material: Material, tint: f32) {
        let [a, h, b] = solid.size;
        let w = |x: f32, y: f32, z: f32| solid.world([x, y, z]);
        let faces = [
            // +x, -x, +z, -z, top, bottom; counter-clockwise from outside.
            [w(a, 0.0, b), w(a, 0.0, -b), w(a, h, -b), w(a, h, b)],
            [w(-a, 0.0, -b), w(-a, 0.0, b), w(-a, h, b), w(-a, h, -b)],
            [w(-a, 0.0, b), w(a, 0.0, b), w(a, h, b), w(-a, h, b)],
            [w(a, 0.0, -b), w(-a, 0.0, -b), w(-a, h, -b), w(a, h, -b)],
            [w(-a, h, b), w(a, h, b), w(a, h, -b), w(-a, h, -b)],
            [w(-a, 0.0, -b), w(a, 0.0, -b), w(a, 0.0, b), w(-a, 0.0, b)],
        ];
        for face in faces {
            self.mesh.polygon(&face, material, tint);
        }
    }

    /// A drum: a vertical cylinder, solid and drawn, with its top capped.
    pub fn drum(
        &mut self,
        base: [f32; 3],
        radius: f32,
        height: f32,
        material: Material,
        tint: f32,
    ) -> Solid {
        let solid = Solid::new(Shape::Cylinder, base, [radius, height, radius], 0.0);
        self.solids.push(solid);
        let ring = |y: f32| -> Vec<([f32; 3], [f32; 3])> {
            (0..=ROUND_SIDES)
                .map(|i| {
                    let t = std::f32::consts::TAU * i as f32 / ROUND_SIDES as f32;
                    let (s, c) = t.sin_cos();
                    (
                        [base[0] + radius * c, base[1] + y, base[2] - radius * s],
                        [c, 0.0, -s],
                    )
                })
                .collect()
        };
        self.mesh
            .smooth_band(&ring(0.0), &ring(height), material, tint);
        let top: Vec<[f32; 3]> = ring(height)
            .into_iter()
            .take(ROUND_SIDES)
            .map(|(p, _)| p)
            .collect();
        self.mesh.polygon(&top, material, tint);
        solid
    }

    /// A conical roof, solid and drawn.
    pub fn cone(
        &mut self,
        base: [f32; 3],
        radius: f32,
        height: f32,
        material: Material,
        tint: f32,
    ) -> Solid {
        let solid = Solid::new(Shape::Cone, base, [radius, height, radius], 0.0);
        self.solids.push(solid);
        let slant = (radius * radius + height * height).sqrt();
        let (nr, ny) = (height / slant, radius / slant);
        let ring = |r: f32, y: f32| -> Vec<([f32; 3], [f32; 3])> {
            (0..=ROUND_SIDES)
                .map(|i| {
                    let t = std::f32::consts::TAU * i as f32 / ROUND_SIDES as f32;
                    let (s, c) = t.sin_cos();
                    (
                        [base[0] + r * c, base[1] + y, base[2] - r * s],
                        [c * nr, ny, -s * nr],
                    )
                })
                .collect()
        };
        // Two bands, so the tip is not one long sliver.
        self.mesh.smooth_band(
            &ring(radius, 0.0),
            &ring(radius * 0.4, height * 0.6),
            material,
            tint,
        );
        self.mesh.smooth_band(
            &ring(radius * 0.4, height * 0.6),
            &ring(0.001, height),
            material,
            tint,
        );
        // The underside, where it overhangs.
        let under: Vec<[f32; 3]> = ring(radius, 0.0)
            .into_iter()
            .take(ROUND_SIDES)
            .map(|(p, _)| p)
            .rev()
            .collect();
        self.mesh.polygon(&under, material, tint * 0.8);
        solid
    }

    /// A gabled roof, solid and drawn: its slopes in `roof`, the triangles
    /// at its ends in `gable`.
    #[allow(clippy::too_many_arguments)]
    pub fn gable(
        &mut self,
        base: [f32; 3],
        half_length: f32,
        half_width: f32,
        height: f32,
        yaw: f32,
        roof: Material,
        gable: Material,
        tint: f32,
    ) -> Solid {
        let solid = Solid::new(Shape::Gable, base, [half_length, height, half_width], yaw);
        self.solids.push(solid);
        let (a, b, h) = (half_length, half_width, height);
        let w = |x: f32, y: f32, z: f32| solid.world([x, y, z]);
        self.mesh.polygon(
            &[w(-a, 0.0, b), w(a, 0.0, b), w(a, h, 0.0), w(-a, h, 0.0)],
            roof,
            tint,
        );
        self.mesh.polygon(
            &[w(a, 0.0, -b), w(-a, 0.0, -b), w(-a, h, 0.0), w(a, h, 0.0)],
            roof,
            tint,
        );
        self.mesh
            .polygon(&[w(a, 0.0, b), w(a, 0.0, -b), w(a, h, 0.0)], gable, tint);
        self.mesh
            .polygon(&[w(-a, 0.0, -b), w(-a, 0.0, b), w(-a, h, 0.0)], gable, tint);
        self.mesh.polygon(
            &[w(-a, 0.0, -b), w(a, 0.0, -b), w(a, 0.0, b), w(-a, 0.0, b)],
            roof,
            tint * 0.7,
        );
        solid
    }

    /// A pyramid roof on a square base, solid and drawn.
    pub fn pyramid(
        &mut self,
        base: [f32; 3],
        half: f32,
        height: f32,
        yaw: f32,
        material: Material,
        tint: f32,
    ) -> Solid {
        let solid = Solid::new(Shape::Pyramid, base, [half, height, half], yaw);
        self.solids.push(solid);
        let w = |x: f32, y: f32, z: f32| solid.world([x, y, z]);
        let apex = w(0.0, height, 0.0);
        let corners = [
            w(half, 0.0, half),
            w(half, 0.0, -half),
            w(-half, 0.0, -half),
            w(-half, 0.0, half),
        ];
        for k in 0..4 {
            self.mesh
                .polygon(&[corners[k], corners[(k + 1) % 4], apex], material, tint);
        }
        self.mesh.polygon(
            &[corners[3], corners[2], corners[1], corners[0]],
            material,
            tint * 0.7,
        );
        solid
    }

    /// A flat rectangle laid on a surface: a window, a door, an arrow slit.
    /// `center` is on the surface, `out` its outward normal (horizontal),
    /// and it stands `lift` proud of it.
    #[allow(clippy::too_many_arguments)]
    pub fn panel(
        &mut self,
        center: [f32; 3],
        out: [f32; 3],
        width: f32,
        height: f32,
        lift: f32,
        material: Material,
        tint: f32,
    ) {
        let across = crate::mesh::normalize([-out[2], 0.0, out[0]]);
        let c = [
            center[0] + out[0] * lift,
            center[1] + out[1] * lift,
            center[2] + out[2] * lift,
        ];
        let (hw, hh) = (width * 0.5, height * 0.5);
        let p = |u: f32, v: f32| [c[0] + across[0] * u, c[1] + v, c[2] + across[2] * u];
        // Counter-clockwise seen from `out`.
        self.mesh.polygon(
            &[p(hw, -hh), p(-hw, -hh), p(-hw, hh), p(hw, hh)],
            material,
            tint,
        );
    }
}

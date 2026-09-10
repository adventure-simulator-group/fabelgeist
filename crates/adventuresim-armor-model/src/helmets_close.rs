//! Independent skull, pivoted chin defense and pierced lifting visor.
use std::f32::consts::{FRAC_PI_2, PI};

use super::{
    CloseHelmetDesign, CloseHelmetProfile,
    geometry::{AROUND, Surface, comb},
};
use crate::{ArmorComponentRole, ArmorHinge, GenerateError, PartMesh};

#[path = "helmets_visor_domain.rs"]
mod domain;
use domain::{HALF_WIDTH_MM, HEIGHT_MM, VisorDomain};

const JAW_ROWS: usize = 12;
const NECK_ROWS: usize = 6;
const BEVOR_COLUMNS: usize = 32;
const PLATE_GAP_M: f32 = 0.0007;
const VISOR_BROW_OVERLAP_M: f32 = 0.030;
const SIDE_WRAP_RADIANS: f32 = PI * 0.55;
pub(super) const NECK_HEM_HEAD_RATIO: f32 = 1.20;
const CHIN_HEAD_RATIO: f32 = 1.03;
const EAR_LOBE_TAPER_START: f32 = 0.40;
const LOWER_FACE_PROJECTION_FRACTION: f32 = 0.25;
const VISOR_SEATING_TRANSITION: f32 = 0.25;
const VISOR_LOWER_EDGE_HEAD_RATIO: f32 = 0.86;
const BEVOR_MOUTH_HEAD_RATIO: f32 = 0.76;
const FACE_PROJECTION_HEAD_RATIO: f32 = 0.35;
const SIGHT_BROW_DROP_HEAD_RATIO: f32 = 0.07;
const BROW_PEAK_M: f32 = 0.025;
const BEVOR_PIVOT_BROW_DROP_M: f32 = 0.008;

pub(super) fn generate(
    radii: [f32; 3],
    _brow: f32,
    half_height: f32,
    d: &CloseHelmetDesign,
) -> Result<PartMesh, GenerateError> {
    generate_fitted(
        radii[1],
        half_height,
        d,
        &CloseHelmetProfile::authored(radii, d),
    )
}

pub(super) fn generate_fitted(
    crown: f32,
    half_height: f32,
    d: &CloseHelmetDesign,
    profile: &CloseHelmetProfile,
) -> Result<PartMesh, GenerateError> {
    let carrier = Carrier {
        crown,
        half_height,
        brow: half_height * super::shapes::BROW_HEIGHT,
        d,
        profile,
    };
    let height = carrier.visor_height();
    let hinge = ArmorHinge {
        origin: [
            0.0,
            carrier.brow + VISOR_BROW_OVERLAP_M - height * 0.20,
            profile.skull_center() + profile.skull_depth() * SIDE_WRAP_RADIANS.cos(),
        ],
        axis: [1.0, 0.0, 0.0],
    };
    let mut mesh = carrier
        .skull()?
        .with_component(ArmorComponentRole::Skull, None);
    mesh.append(
        carrier
            .bevor()?
            .with_component(ArmorComponentRole::Bevor, Some(hinge)),
    );
    mesh.append(
        carrier
            .visor()?
            .with_component(ArmorComponentRole::Visor, Some(hinge)),
    );
    Ok(mesh)
}

struct Carrier<'a> {
    crown: f32,
    half_height: f32,
    brow: f32,
    d: &'a CloseHelmetDesign,
    profile: &'a CloseHelmetProfile,
}

impl Carrier<'_> {
    fn jaw_height(&self, angle: f32) -> f32 {
        -self.half_height * CHIN_HEAD_RATIO
            + self.d.back_edge_lift.metres() * (1.0 - angle.cos().max(0.0).powi(2))
    }

    fn hem_height(&self, angle: f32) -> f32 {
        -self.half_height * NECK_HEM_HEAD_RATIO
            + self.d.back_edge_lift.metres() * (1.0 - angle.cos().max(0.0).powi(2))
    }

    fn row_height(&self, top: f32, angle: f32, row: usize) -> f32 {
        let jaw = self.jaw_height(angle);
        if row <= JAW_ROWS {
            top + (jaw - top) * row as f32 / JAW_ROWS as f32
        } else {
            jaw + (self.hem_height(angle) - jaw) * (row - JAW_ROWS) as f32 / NECK_ROWS as f32
        }
    }

    fn point(&self, angle: f32, y: f32, layer: f32) -> [f32; 3] {
        const NORMAL_SAMPLE_ANGLE: f32 = 0.001;
        const NORMAL_SAMPLE_HEIGHT_M: f32 = 0.0001;
        let p = self.base_point(angle, y);
        if layer == 0.0 {
            return p;
        }
        let across: [f32; 3] = std::array::from_fn(|i| {
            self.base_point(angle + NORMAL_SAMPLE_ANGLE, y)[i]
                - self.base_point(angle - NORMAL_SAMPLE_ANGLE, y)[i]
        });
        let up: [f32; 3] = std::array::from_fn(|i| {
            self.base_point(angle, y + NORMAL_SAMPLE_HEIGHT_M)[i]
                - self.base_point(angle, y - NORMAL_SAMPLE_HEIGHT_M)[i]
        });
        let normal = [
            across[1] * up[2] - across[2] * up[1],
            across[2] * up[0] - across[0] * up[2],
            across[0] * up[1] - across[1] * up[0],
        ];
        let length = normal.iter().map(|v| v * v).sum::<f32>().sqrt();
        let offset = (self.d.fit.wall_thickness.metres() + PLATE_GAP_M) * layer;
        std::array::from_fn(|i| p[i] + normal[i] / length * offset)
    }

    fn base_point(&self, angle: f32, y: f32) -> [f32; 3] {
        if y > self.brow {
            let height = ((y - self.brow) / (self.crown - self.brow)).clamp(0.0, 1.0);
            let radius = (1.0 - height * height).sqrt();
            let width = self.profile.temple_half_width
                + (self.profile.skull_half_width - self.profile.temple_half_width) * smooth(height);
            return [
                width * radius * angle.sin(),
                y,
                self.profile.skull_center() + self.profile.skull_depth() * radius * angle.cos(),
            ];
        }
        let jaw = self.jaw_height(angle);
        let t = (self.brow - y) / (self.brow - jaw);
        let jaw_blend = smooth((t - EAR_LOBE_TAPER_START) / (1.0 - EAR_LOBE_TAPER_START));
        let neck_blend = smooth((jaw - y) / (jaw - self.hem_height(angle)));
        let [x, z] = self
            .profile
            .section(self.d, angle, jaw_blend, neck_blend, smooth(t));
        let lip = self.d.throat_flare.metres() * neck_blend.powi(4) * 0.5;
        // Both overlapping plates share the lower face projection. The visor's
        // exit must lead into a receding chin, rather than ending behind it.
        let mouth = -self.half_height * FACE_PROJECTION_HEAD_RATIO;
        let face_blend = if y >= mouth {
            smooth((self.brow - y) / (self.brow - mouth))
        } else {
            ((jaw - y) / (jaw - mouth)).clamp(0.0, 1.0)
        };
        let face_projection =
            self.d.visor_projection.metres() * LOWER_FACE_PROJECTION_FRACTION * face_blend;
        [
            x + lip * angle.sin(),
            y,
            z + lip * angle.cos()
                + face_projection * angle.cos().max(0.0).powi(2)
                + self.d.chin_projection.metres()
                    * angle.cos().max(0.0).powi(2)
                    * jaw_blend
                    * (1.0 - neck_blend),
        ]
    }

    fn skull(&self) -> Result<PartMesh, GenerateError> {
        let mut surface = Surface::default();
        let radii = [
            self.profile.temple_half_width,
            self.crown,
            self.profile.skull_depth(),
        ];
        let rim = surface.dome(radii, self.brow);
        for p in &mut surface.positions {
            let crown_blend = smooth((p[1] - self.brow) / (self.crown - self.brow));
            p[0] *= 1.0
                + (self.profile.skull_half_width / self.profile.temple_half_width - 1.0)
                    * crown_blend;
            p[2] += self.profile.skull_center();
        }
        let first = AROUND / 4;
        let last = AROUND - first;
        let mut previous = rim[first..=last].to_vec();
        for row in 1..=JAW_ROWS + NECK_ROWS {
            let ring = (first..=last)
                .map(|i| {
                    let angle = i as f32 / AROUND as f32 * PI * 2.0;
                    surface.vertex(self.point(angle, self.row_height(self.brow, angle, row), 0.0))
                })
                .collect::<Vec<_>>();
            surface.connect(&previous, &ring, false);
            previous = ring;
        }
        let mut mesh = surface.shell(self.d.fit.wall_thickness.metres())?;
        if self.d.comb_height.0 > 0 {
            let mut crest = comb(
                radii,
                self.brow,
                self.d.comb_height.metres(),
                self.d.fit.wall_thickness.metres(),
            )?;
            for p in &mut crest.positions {
                p[2] += self.profile.skull_center();
            }
            mesh.append(crest);
        }
        if self.d.nape_length.0 > 0 {
            mesh.append(self.nape()?);
        }
        Ok(mesh)
    }

    fn nape(&self) -> Result<PartMesh, GenerateError> {
        const LAMES: usize = 3;
        const LAME_ROWS: usize = 4;
        const LAME_COLUMNS: usize = 24;
        const OVERLAP: f32 = 0.06;
        let mut tail = PartMesh::default();
        for lame in 0..LAMES {
            let start = if lame == 0 {
                0.0
            } else {
                lame as f32 / LAMES as f32 - OVERLAP
            };
            let end = (lame + 1) as f32 / LAMES as f32;
            let mut surface = Surface::default();
            let mut previous = Vec::new();
            for row in 0..=LAME_ROWS {
                let t = start + (end - start) * row as f32 / LAME_ROWS as f32;
                let ring = (0..=LAME_COLUMNS)
                    .map(|column| {
                        let u = column as f32 / LAME_COLUMNS as f32 * 2.0 - 1.0;
                        surface.vertex(self.nape_point(u, t, (LAMES - lame) as f32))
                    })
                    .collect::<Vec<_>>();
                if !previous.is_empty() {
                    surface.connect(&previous, &ring, false);
                }
                previous = ring;
            }
            tail.append(surface.shell(self.d.fit.wall_thickness.metres())?);
        }
        Ok(tail)
    }

    fn nape_base(&self, u: f32, t: f32) -> [f32; 3] {
        const NAPE_HALF_WRAP: f32 = PI * 7.0 / 18.0;
        const NAPE_ROOT_RISE_M: f32 = 0.030;
        const NAPE_CORNER_SWEEP: f32 = 0.20;
        let angle = PI + NAPE_HALF_WRAP * u;
        let root = self.hem_height(angle) + NAPE_ROOT_RISE_M * (1.0 - u * u);
        let mut p = self.point(angle, root, 0.0);
        let sweep = self.d.nape_flare.metres() * t;
        p[0] -= sweep * u * 0.9;
        p[1] -= self.d.nape_length.metres() * t;
        p[2] -= sweep * (1.0 + NAPE_CORNER_SWEEP * u * u);
        p
    }

    fn nape_point(&self, u: f32, t: f32, layer: f32) -> [f32; 3] {
        const SAMPLE: f32 = 0.001;
        let across: [f32; 3] = std::array::from_fn(|i| {
            self.nape_base(u + SAMPLE, t)[i] - self.nape_base(u - SAMPLE, t)[i]
        });
        let up: [f32; 3] = std::array::from_fn(|i| {
            self.nape_base(u, t - SAMPLE)[i] - self.nape_base(u, t + SAMPLE)[i]
        });
        let n = [
            across[1] * up[2] - across[2] * up[1],
            across[2] * up[0] - across[0] * up[2],
            across[0] * up[1] - across[1] * up[0],
        ];
        let length = n.iter().map(|v| v * v).sum::<f32>().sqrt();
        let offset = (self.d.fit.wall_thickness.metres() + PLATE_GAP_M) * layer;
        let p = self.nape_base(u, t);
        std::array::from_fn(|i| p[i] + n[i] / length * offset)
    }

    fn bevor(&self) -> Result<PartMesh, GenerateError> {
        let mut surface = Surface::default();
        let mut previous = Vec::new();
        for row in 0..=JAW_ROWS + NECK_ROWS {
            let ring = (0..=BEVOR_COLUMNS)
                .map(|column| {
                    let angle =
                        (column as f32 / BEVOR_COLUMNS as f32 * 2.0 - 1.0) * SIDE_WRAP_RADIANS;
                    let front = angle.cos().max(0.0);
                    let top = self.brow
                        - BEVOR_PIVOT_BROW_DROP_M
                        - (self.half_height * BEVOR_MOUTH_HEAD_RATIO + self.brow
                            - BEVOR_PIVOT_BROW_DROP_M)
                            * front.powi(2);
                    surface.vertex(self.point(angle, self.row_height(top, angle, row), 1.0))
                })
                .collect::<Vec<_>>();
            if !previous.is_empty() {
                surface.connect(&previous, &ring, false);
            }
            previous = ring;
        }
        surface.shell(self.d.fit.wall_thickness.metres())
    }

    fn visor_height(&self) -> f32 {
        self.brow + VISOR_BROW_OVERLAP_M + self.half_height * VISOR_LOWER_EDGE_HEAD_RATIO
    }

    fn visor_y(&self, angle: f32, t: f32) -> f32 {
        let sight_t = domain::SIGHT_CENTER_MM as f32 / HEIGHT_MM;
        let eye = self.brow - self.half_height * SIGHT_BROW_DROP_HEAD_RATIO;
        if t <= sight_t {
            let top = self.brow + VISOR_BROW_OVERLAP_M + BROW_PEAK_M * angle.cos().max(0.0).powi(2);
            top + (eye - top) * t / sight_t
        } else {
            let bottom = -self.half_height * VISOR_LOWER_EDGE_HEAD_RATIO;
            eye + (bottom - eye) * (t - sight_t) / (1.0 - sight_t)
        }
    }

    fn visor(&self) -> Result<PartMesh, GenerateError> {
        let domain = VisorDomain::new(self.d)?;
        let positions = domain
            .points
            .iter()
            .map(|p| {
                let angle = p[0] as f32 / HALF_WIDTH_MM * SIDE_WRAP_RADIANS;
                let t = p[1] as f32 / HEIGHT_MM;
                let ridge = self.d.ridge_height.unit();
                let ramp = if t <= ridge {
                    t / ridge
                } else {
                    (1.0 - t) / (1.0 - ridge)
                };
                let rounded = (ramp.clamp(0.0, 1.0) * FRAC_PI_2).sin();
                let projection = self.d.visor_projection.metres()
                    * (rounded + (ramp - rounded) * self.d.ridge_sharpness.unit());
                let y = self.visor_y(angle, t);
                let mut point = self.point(angle, y, 2.0);
                point[2] += projection
                    * smooth((1.0 - t) / VISOR_SEATING_TRANSITION)
                    * angle.cos().max(0.0).powi(2);
                point
            })
            .collect();
        PartMesh::from_surface(
            positions,
            domain.indices,
            self.d.fit.wall_thickness.metres(),
            crate::BoundaryNormals::Separate,
            crate::ShellExtrusion::InPlane {
                normal: [0.0, 1.0, 0.0],
            },
        )
    }
}

fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Millimeters;

    #[test]
    fn sights_open_into_the_head_cavity_instead_of_facing_another_plate() {
        for scale in [0.75, 1.0, 1.3] {
            for gauge in [1, 2, 4] {
                let mut design = CloseHelmetDesign::default();
                design.fit.wall_thickness = Millimeters(gauge);
                let gap = design.fit.clearance.metres() + design.fit.wall_thickness.metres();
                let height = 0.115 * scale;
                let radii = [0.085 * scale + gap, height + gap, 0.105 * scale + gap];
                let profile = CloseHelmetProfile::authored(radii, &design);
                let carrier = Carrier {
                    crown: radii[1],
                    half_height: height,
                    brow: height * super::super::shapes::BROW_HEIGHT,
                    d: &design,
                    profile: &profile,
                };
                let mesh = generate_fitted(radii[1], height, &design, &profile)
                    .unwrap_or_else(|e| panic!("scale {scale}, gauge {gauge}: {e:?}"));
                let y = carrier.visor_y(0.0, domain::SIGHT_CENTER_MM as f32 / HEIGHT_MM);
                for x in [-30.0, 30.0] {
                    let p = carrier.point(x / HALF_WIDTH_MM * SIDE_WRAP_RADIANS, y, 2.0);
                    assert!(
                        !blocks_sight(&mesh, p, profile.skull_center()),
                        "sight blocked at scale {scale}, gauge {gauge}"
                    );
                }
                assert!(
                    blocks_sight(&mesh, carrier.point(0.0, y, 2.0), profile.skull_center()),
                    "central bridge must still block the same ray test"
                );
            }
        }
    }

    // Intersect a forward ray with actual triangles projected onto the XY plane.
    fn blocks_sight(mesh: &PartMesh, p: [f32; 3], cavity_depth: f32) -> bool {
        mesh.indices.as_chunks::<3>().0.iter().any(|triangle| {
            let [a, b, c] = triangle.map(|i| mesh.positions[i as usize]);
            let area = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
            if area.abs() < 1e-10 {
                return false;
            }
            let u = ((p[0] - a[0]) * (c[1] - a[1]) - (p[1] - a[1]) * (c[0] - a[0])) / area;
            let v = ((b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0])) / area;
            u >= 0.0
                && v >= 0.0
                && u + v <= 1.0
                && a[2] + u * (b[2] - a[2]) + v * (c[2] - a[2]) >= cavity_depth
        })
    }
}

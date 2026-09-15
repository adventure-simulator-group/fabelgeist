//! Curved shield panels and their shared fitting dimensions.
use super::*;
mod fittings;
mod panels;
#[cfg(test)]
mod tests;
use std::f64::consts::{PI, TAU};

pub(super) enum Panel<'a> {
    Round(&'a RoundShieldParameters),
    Shaped(&'a ShapedShieldParameters),
}
pub(super) struct Shield<'a> {
    panel: Panel<'a>,
    thickness: f64,
    rim_radius: f64,
    boss_radius: f64,
    boss_height: f64,
    strapped: bool,
    angle: f64,
    mirrored: bool,
    grip_length: f64,
    grip_radius: f64,
    spacing: f64,
    clearance: f64,
    strap_width: f64,
    strap_thickness: f64,
    rim_material: Material,
    boss_material: Material,
    grip_material: Material,
    strap_material: Material,
}
impl<'a> Shield<'a> {
    pub(super) fn from_shape(shape: &'a Shape) -> Option<Self> {
        macro_rules! shield {
            ($p:ident,$panel:expr,$strapped:expr) => {
                Self {
                    panel: $panel,
                    thickness: $p.thickness.get(),
                    rim_radius: $p.rim_radius.map_or(0.0, Metres::get),
                    boss_radius: $p.boss_radius.map_or(0.0, Metres::get),
                    boss_height: $p.boss_height.map_or(0.0, Metres::get),
                    strapped: $strapped,
                    angle: $p.fitting_angle.map_or(0.0, Degrees::get),
                    mirrored: $p.mirrored.unwrap_or(false),
                    grip_length: $p.grip_length.map_or(0.0, Metres::get),
                    grip_radius: $p.grip_radius.map_or(0.0, Metres::get),
                    spacing: $p.fitting_spacing.map_or(0.0, Metres::get),
                    clearance: $p.fitting_clearance.map_or(0.0, Metres::get),
                    strap_width: $p.strap_width.map_or(0.0, Metres::get),
                    strap_thickness: $p.strap_thickness.map_or(0.0, Metres::get),
                    rim_material: $p.rim_material.unwrap_or(Material::DarkSteel),
                    boss_material: $p.boss_material.unwrap_or(Material::Steel),
                    grip_material: $p.grip_material.unwrap_or(Material::Wood),
                    strap_material: $p.strap_material.unwrap_or(Material::Leather),
                }
            };
        }
        Some(match shape {
            Shape::RoundShield(p) => shield!(
                p,
                Panel::Round(p),
                p.fitting_mode == RoundShieldFittingMode::GripAndStrap
            ),
            Shape::ShapedShield(p) => shield!(
                p,
                Panel::Shaped(p),
                p.fitting_mode == ShapedShieldFittingMode::GripAndStrap
            ),
            _ => return None,
        })
    }
    fn curve(&self, [x, y]: PlanarPoint) -> f64 {
        match self.panel {
            Panel::Round(p) => {
                let radial = (x.hypot(y) / p.radius.get()).min(1.0);
                let limit = p.center_radius.get() / p.radius.get();
                let t = if limit > 0.0 { radial / limit } else { 1.0 };
                p.outer_curve.get() * (1.0 - radial * radial)
                    + if t < 1.0 {
                        p.center_curve.get() * (1.0 - t * t).powi(2)
                    } else {
                        0.0
                    }
            }
            Panel::Shaped(p) => {
                let horizontal = (2.0 * x / p.width.get()).abs().min(1.0);
                let distance = (x / (p.center_width.get() / 2.0)).powi(2)
                    + (y / (p.center_height.get() / 2.0)).powi(2);
                p.cylindrical_curve.get() * (1.0 - horizontal * horizontal)
                    + if distance < 1.0 {
                        p.center_curve.get() * (1.0 - distance).powi(2)
                    } else {
                        0.0
                    }
            }
        }
    }
    fn surface(&self, point: PlanarPoint, front: bool) -> f64 {
        self.curve(point) + self.thickness * if front { 0.5 } else { -0.5 }
    }
    fn aperture(&self) -> f64 {
        if matches!(self.panel, Panel::Round(_)) && !self.strapped && self.boss_height > 0.0 {
            self.boss_radius
                .mul_add(0.72, 0.0)
                .min(self.grip_length * 0.4)
        } else {
            0.0
        }
    }
    pub(super) fn grip(&self) -> Result<Point, String> {
        let layout = self.layout(Detail::High)?;
        let (body, _) = self.body(Detail::High)?;
        self.handle_center(&body, &layout)
    }
    fn part(
        &self,
        solid: Solid,
        material: Material,
        label: &str,
        r: &ResolvedComponent,
        role: output::ShieldRole,
    ) -> PartSource {
        let mut part = PartSource::new(solid, material, label, &r.id);
        part.shield_role = Some(role);
        part
    }
    pub(super) fn construct(
        &self,
        r: &ResolvedComponent,
        detail: Detail,
    ) -> Result<Vec<PartSource>, String> {
        let (body, outline) = self.body(detail)?;
        let mut parts = self.fittings(r, &body, detail)?;
        parts.insert(
            0,
            self.part(
                body,
                r.component.material.unwrap_or(Material::Wood),
                &r.label,
                r,
                output::ShieldRole::Body,
            ),
        );
        let mut at = 1;
        if self.rim_radius > 0.0 {
            parts.insert(
                at,
                self.part(
                    self.rim(&outline, detail),
                    self.rim_material,
                    "shield rim",
                    r,
                    output::ShieldRole::Rim,
                ),
            );
            at += 1;
        }
        if self.boss_height > 0.0 && self.boss_radius > 0.0 {
            parts.insert(
                at,
                self.part(
                    self.boss(detail),
                    self.boss_material,
                    "shield boss",
                    r,
                    output::ShieldRole::Boss,
                ),
            );
        }
        Ok(parts)
    }
    fn rim(&self, outline: &[PlanarPoint], detail: Detail) -> Solid {
        let fraction =
            self.rim_radius / outline.iter().map(|p| p[0].hypot(p[1])).fold(0.0, f64::max);
        let sections = detail.radial(self.rim_radius, 12);
        let point = |i: usize, j: usize| {
            let [x, y] = outline[i];
            let angle = j as f64 / sections as f64 * TAU;
            let scale = 1.0 + fraction * angle.cos();
            [
                x * scale,
                y * scale,
                self.curve([x, y]) + self.rim_radius * angle.sin(),
            ]
        };
        let mut solid = Solid::default();
        for i in 0..outline.len() {
            for j in 0..sections {
                solid.quad(
                    point(i, j),
                    point((i + 1) % outline.len(), j),
                    point((i + 1) % outline.len(), (j + 1) % sections),
                    point(i, (j + 1) % sections),
                    1,
                );
            }
        }
        solid.positive()
    }
    fn boss(&self, detail: Detail) -> Solid {
        let segments = detail.radial(self.boss_radius, 32);
        let rows = detail.samples(12, 6);
        let wall = 0.0015_f64.min(self.boss_height * 0.25);
        let point = |row: usize, segment: usize, inner: bool| {
            let latitude = row as f64 / rows as f64 * PI / 2.0;
            let angle = segment as f64 / segments as f64 * TAU;
            let radius = if row == rows {
                0.0
            } else {
                self.boss_radius * latitude.cos()
            };
            let x = radius * angle.cos();
            let y = radius * angle.sin();
            [
                x,
                y,
                self.surface([x, y], true) + self.boss_height * latitude.sin()
                    - 0.0005
                    - if inner { wall } else { 0.0 },
            ]
        };
        let mut solid = Solid::default();
        for inner in [false, true] {
            for row in 0..rows {
                for segment in 0..segments {
                    let next = (segment + 1) % segments;
                    let [a, b, c, d] = [
                        point(row, segment, inner),
                        point(row, next, inner),
                        point(row + 1, next, inner),
                        point(row + 1, segment, inner),
                    ];
                    if inner {
                        solid.triangle(a, c, b, 2);
                        if row < rows - 1 {
                            solid.triangle(a, d, c, 2);
                        }
                    } else {
                        solid.triangle(a, b, c, 1);
                        if row < rows - 1 {
                            solid.triangle(a, c, d, 1);
                        }
                    }
                }
            }
        }
        for segment in 0..segments {
            let next = (segment + 1) % segments;
            solid.quad(
                point(0, segment, false),
                point(0, segment, true),
                point(0, next, true),
                point(0, next, false),
                0,
            );
        }
        solid.positive()
    }
}

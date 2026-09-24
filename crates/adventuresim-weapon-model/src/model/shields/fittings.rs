//! Rear fittings seated on the emitted shield surface.
use super::*;

pub(super) struct Layout {
    pub(super) grip: PlanarPoint,
    strap: PlanarPoint,
    position: PlanarPoint,
    axis: PlanarPoint,
    scale: f64,
}
fn endpoint(center: PlanarPoint, axis: PlanarPoint, distance: f64) -> PlanarPoint {
    [
        center[0] + axis[0] * distance,
        center[1] + axis[1] * distance,
    ]
}
fn back_surface(solid: &Solid, [x, y]: PlanarPoint) -> Result<f64, String> {
    for &[a, b, c] in &solid.faces {
        let [a, b, c] = [solid.positions[a], solid.positions[b], solid.positions[c]];
        let denominator = (b[1] - c[1]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[1] - c[1]);
        if denominator >= 0.0 {
            continue;
        }
        let u = ((b[1] - c[1]) * (x - c[0]) + (c[0] - b[0]) * (y - c[1])) / denominator;
        let v = ((c[1] - a[1]) * (x - c[0]) + (a[0] - c[0]) * (y - c[1])) / denominator;
        let w = 1.0 - u - v;
        if u >= -1e-10 && v >= -1e-10 && w >= -1e-10 {
            return Ok(u * a[2] + v * b[2] + w * c[2]);
        }
    }
    Err("shield fitting lies outside panel surface".into())
}
impl Shield<'_> {
    pub(super) fn layout(&self, detail: Detail) -> Result<Layout, String> {
        let angle = self.angle.to_radians();
        let position = [angle.cos(), angle.sin()];
        let axis = [-angle.sin(), angle.cos()];
        let spacing = if self.strapped {
            self.spacing * if self.mirrored { -1.0 } else { 1.0 }
        } else {
            0.0
        };
        let mut grip = position.map(|v| v * (-spacing / 2.0));
        let mut strap = position.map(|v| v * spacing / 2.0);
        let mut scale = 1.0_f64;
        if matches!(self.panel, Panel::Shaped(_)) {
            let mut outline = self.outline(detail);
            if signed_area(&outline) < 0.0 {
                outline.reverse();
            }
            let centers = if self.strapped {
                vec![grip, strap]
            } else {
                vec![grip]
            };
            let anchors: Vec<_> = centers
                .into_iter()
                .flat_map(|center| {
                    [-1.0, 1.0].map(|side| endpoint(center, axis, side * self.grip_length / 2.0))
                })
                .collect();
            let margin = (self.grip_radius * 1.25).max(if self.strapped {
                self.strap_width / 2.0_f64.sqrt() + self.strap_thickness
            } else {
                0.0
            });
            for i in 0..outline.len() {
                let a = outline[i];
                let b = outline[(i + 1) % outline.len()];
                let dx = b[0] - a[0];
                let dy = b[1] - a[1];
                let size = dx.hypot(dy);
                let normal = [dy / size, -dx / size];
                let room = normal[0] * a[0] + normal[1] * a[1];
                for q in &anchors {
                    let extent = normal[0] * q[0] + normal[1] * q[1] + margin;
                    if extent > 0.0 {
                        scale = scale.min(room * 0.98 / extent);
                    }
                }
            }
            if scale <= 0.0 {
                return Err("shield fitting layout has no interior clearance".into());
            }
            grip = grip.map(|v| v * scale);
            strap = strap.map(|v| v * scale);
        }
        Ok(Layout {
            grip,
            strap,
            position,
            axis,
            scale,
        })
    }
    pub(super) fn handle_center(&self, body: &Solid, layout: &Layout) -> Result<Point, String> {
        let half = self.grip_length * layout.scale / 2.0;
        let a = endpoint(layout.grip, layout.axis, -half);
        let b = endpoint(layout.grip, layout.axis, half);
        Ok([
            layout.grip[0],
            layout.grip[1],
            back_surface(body, a)?.min(back_surface(body, b)?) - self.clearance,
        ])
    }
    pub(super) fn fittings(
        &self,
        r: &ResolvedComponent,
        body: &Solid,
        detail: Detail,
    ) -> Result<Vec<PartSource>, String> {
        let layout = self.layout(Detail::High)?;
        let length = self.grip_length * layout.scale;
        let radius = self.grip_radius * layout.scale;
        let half = length / 2.0;
        let ends = [
            endpoint(layout.grip, layout.axis, -half),
            endpoint(layout.grip, layout.axis, half),
        ];
        let backs = [back_surface(body, ends[0])?, back_surface(body, ends[1])?];
        let lifted = self.grip()?[2];
        let point = |xy: PlanarPoint, z: f64| [xy[0], xy[1], z];
        let path = [
            point(ends[0], backs[0] - radius * 1.5),
            point(endpoint(layout.grip, layout.axis, -half * 0.76), lifted),
            point(endpoint(layout.grip, layout.axis, half * 0.76), lifted),
            point(ends[1], backs[1] - radius * 1.5),
        ];
        let grip = Solid::sweep(
            &path,
            &Sweep {
                width: radius * 2.0,
                depth: radius * 2.0,
                ring_scales: Some(bend_scales(&path, radius, false)),
                ..Sweep::default()
            },
            detail,
        )?;
        let mut parts = vec![self.part(
            grip,
            self.grip_material,
            "shield handle",
            r,
            output::ShieldRole::Fitting,
        )];
        let penetration = 0.0003_f64.min(self.thickness * 0.15);
        let aperture =
            if matches!(self.panel, Panel::Round(_)) && !self.strapped && self.boss_height > 0.0 {
                (self.boss_radius * 0.72).min(length * 0.4)
            } else {
                0.0
            };
        for i in 0..2 {
            let bottom = backs[i] - radius * 2.0;
            let top = backs[i] + penetration;
            let foot_radius = (radius * 0.75).min((half - aperture) * 0.45);
            let solid = Solid::lathe(
                &[[0.0, foot_radius], [top - bottom, foot_radius]],
                12,
                1.0,
                false,
                detail,
            )?
            .transform([90.0, 0.0, 0.0], point(ends[i], bottom));
            parts.push(self.part(
                solid,
                self.grip_material,
                "shield handle foot",
                r,
                output::ShieldRole::Fitting,
            ));
        }
        if self.strapped {
            parts.extend(self.strap(r, body, &layout, length, detail)?);
        }
        Ok(parts)
    }
    fn strap(
        &self,
        r: &ResolvedComponent,
        body: &Solid,
        layout: &Layout,
        length: f64,
        detail: Detail,
    ) -> Result<Vec<PartSource>, String> {
        let width = self.strap_width * layout.scale;
        let thickness = self.strap_thickness * layout.scale;
        let ends = [
            endpoint(layout.strap, layout.axis, -length * 0.46),
            endpoint(layout.strap, layout.axis, length * 0.46),
        ];
        let z =
            back_surface(body, ends[0])?.min(back_surface(body, ends[1])?) - self.clearance * 0.72;
        let rotation = [0.0, 0.0, self.angle];
        let solid = Solid::cuboid([width, length * 0.92, thickness], detail)?
            .transform(rotation, [layout.strap[0], layout.strap[1], z]);
        let mut parts = vec![self.part(
            solid,
            self.strap_material,
            "forearm strap",
            r,
            output::ShieldRole::Fitting,
        )];
        for end in ends {
            let mut back = f64::INFINITY;
            for across in [-1.0, 1.0] {
                for along in [-1.0, 1.0] {
                    back = back.min(back_surface(
                        body,
                        endpoint(
                            endpoint(end, layout.position, across * width / 2.0),
                            layout.axis,
                            along * width / 2.0,
                        ),
                    )?);
                }
            }
            let bottom = z - thickness / 2.0;
            let top = back + (thickness / 2.0).min(self.thickness * 0.2);
            let solid = Solid::cuboid([width, width, top - bottom], detail)?
                .transform(rotation, [end[0], end[1], (bottom + top) / 2.0]);
            parts.push(self.part(
                solid,
                self.strap_material,
                "strap attachment",
                r,
                output::ShieldRole::Fitting,
            ));
        }
        Ok(parts)
    }
}

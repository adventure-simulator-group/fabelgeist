//! Spatial support queries against terrain, masonry and typed bark references.
use super::*;

impl Supports<'_> {
    /// A bark segment as the plant has it now: its ends and their radii.
    pub fn segment(&self, s: &BarkSegment) -> Option<(Vec3, Vec3, f32, f32)> {
        let plant = self.plants.get(s.plant.index())?;
        let module = plant.modules.get(s.module.index())?;
        let prototype = self.prototypes.get(module.prototype);
        let n = s.node as usize;
        if n == 0 || n >= prototype.node_count() || module.node_offset.is_none() {
            return None;
        }
        let offset = module.node_offset?;
        let offset = offset.index();
        let b = plant.nodes.get(offset + n)?;
        let a = plant
            .nodes
            .get(offset + prototype.nodes[n].parent as usize)?
            .position;
        let end = if b.end_diameter > 0.0 {
            b.end_diameter.min(b.diameter)
        } else {
            b.diameter * 0.5
        };
        Some((a, b.position, b.diameter * 0.5, end * 0.5))
    }

    fn ground(&self, p: Vec3) -> Contact {
        let h = self.terrain.height_at(p.x, p.z);
        let e = 0.25;
        let dx = self.terrain.height_at(p.x + e, p.z) - self.terrain.height_at(p.x - e, p.z);
        let dz = self.terrain.height_at(p.x, p.z + e) - self.terrain.height_at(p.x, p.z - e);
        let normal = Vec3::new(-dx / (2.0 * e), 1.0, -dz / (2.0 * e)).normalize();
        let distance = (p.y - h) * normal.y;
        Contact {
            point: p - normal * distance,
            normal,
            distance,
            kind: Support::Ground,
            diameter: f32::INFINITY,
            axis: Vec3::new(0.0, 0.0, 0.0),
        }
    }

    /// The nearest surface within `range` of `p` that `accept` takes.
    pub fn nearest(
        &self,
        p: Vec3,
        range: f32,
        accept: impl Fn(&Contact) -> bool,
    ) -> Option<Contact> {
        let mut best: Option<Contact> = None;
        let mut offer = |c: Contact| {
            if c.distance < range && best.is_none_or(|b| c.distance < b.distance) && accept(&c) {
                best = Some(c);
            }
        };
        offer(self.ground(p));
        if let Some(obstacles) = self.obstacles {
            let distance = obstacles.distance(p);
            if distance < range
                && let Some(normal) = obstacles.normal(p)
            {
                offer(Contact {
                    point: p - normal * distance,
                    normal,
                    distance,
                    kind: Support::Masonry,
                    diameter: f32::INFINITY,
                    axis: Vec3::new(0.0, 0.0, 0.0),
                });
            }
        }
        let cells = ((range + self.bark.reach) / BARK_CELL).ceil() as i32;
        self.bark.grid.for_each_near(p.to_array(), cells, |i| {
            let Some((a, b, ra, rb)) = self.segment(&self.bark.segments[i as usize]) else {
                return;
            };
            let ab = b - a;
            let length2 = ab.dot(ab).max(1e-12);
            let t = ((p - a).dot(ab) / length2).clamp(0.0, 1.0);
            let c = a + ab * t;
            let r = ra + (rb - ra) * t;
            let v = p - c;
            let d = v.length();
            if d < 1e-6 {
                return;
            }
            let normal = v / d;
            offer(Contact {
                point: c + normal * r,
                normal,
                distance: d - r,
                kind: Support::Bark { segment: i },
                diameter: 2.0 * r,
                axis: ab / length2.sqrt(),
            });
        });
        best
    }

    /// Where a hold on bark is now, `lift` out from the surface, and the
    /// surface's normal there; `None` if the branch is gone.
    pub fn bark_point(&self, hold: &BarkHold, lift: f32) -> Option<(Vec3, Vec3)> {
        let plant = self
            .plants
            .get(hold.plant_index.index())
            .filter(|p| p.id == hold.plant)?;
        plant
            .modules
            .get(hold.module_index.index())
            .filter(|m| m.id == hold.module)?;
        let segment = BarkSegment {
            plant: hold.plant_index,
            module: hold.module_index,
            node: hold.node,
        };
        let (a, b, ra, rb) = self.segment(&segment)?;
        let axis = (b - a).normalize();
        let (f1, f2) = basis(axis);
        let out = f1 * hold.angle.cos() + f2 * hold.angle.sin();
        let r = ra + (rb - ra) * hold.t;
        Some((a + (b - a) * hold.t + out * (r + lift), out))
    }
}

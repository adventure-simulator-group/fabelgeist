//! Exact fixed-solid intersections with the architectural ground datum.
use super::*;
use bevy::math::Quat;

impl CollisionCuboid {
    /// Ordered plan-space cross-section at architectural Y=0. Empty means the
    /// solid does not meet the datum. A touching edge or point remains present;
    /// callers must distinguish those contacts from an area-bearing surface.
    pub fn ground_contact_polygon(self) -> Vec<Vec2> {
        let bounds = self.bounds();
        if bounds.min.y > 0.0 || bounds.max.y < 0.0 {
            return Vec::new();
        }
        let rotation = Quat::from_rotation_y(self.yaw_radians)
            * Quat::from_rotation_x(self.crossfall_radians)
            * Quat::from_rotation_z(self.longfall_radians);
        let corners = std::array::from_fn::<_, 8, _>(|i| {
            let sign = Vec3::new(
                if i & 1 == 0 { -1.0 } else { 1.0 },
                if i & 2 == 0 { -1.0 } else { 1.0 },
                if i & 4 == 0 { -1.0 } else { 1.0 },
            );
            self.centre + rotation * (self.size * sign * 0.5)
        });
        let mut points = Vec::new();
        for (i, a) in corners.iter().enumerate() {
            if a.y == 0.0 {
                points.push(Vec2::new(a.x, a.z));
            }
            for axis in [1, 2, 4] {
                if i & axis != 0 {
                    continue;
                }
                let b = corners[i | axis];
                if (a.y < 0.0 && b.y > 0.0) || (a.y > 0.0 && b.y < 0.0) {
                    let point = *a + (b - *a) * (-a.y / (b.y - a.y));
                    points.push(Vec2::new(point.x, point.z));
                }
            }
        }
        points.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
        points.dedup();
        if !points.is_empty() {
            let centre = points.iter().copied().sum::<Vec2>() / points.len() as f32;
            points.sort_by(|a, b| {
                let a = *a - centre;
                let b = *b - centre;
                a.y.atan2(a.x).total_cmp(&b.y.atan2(b.x))
            });
        }
        points
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid() -> CollisionCuboid {
        CollisionCuboid {
            source: ResolvedItemId(1),
            centre: Vec3::ZERO,
            size: Vec3::splat(2.0),
            yaw_radians: 0.0,
            crossfall_radians: 0.0,
            longfall_radians: 0.0,
        }
    }

    #[test]
    fn pitched_solid_contact_is_smaller_than_its_projected_envelope() {
        let solid = CollisionCuboid {
            centre: Vec3::Y,
            crossfall_radians: core::f32::consts::FRAC_PI_4,
            ..solid()
        };
        let polygon = solid.ground_contact_polygon();
        assert_eq!(polygon.len(), 4);
        let min = polygon.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
        let max = polygon
            .iter()
            .map(|p| p.y)
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(max - min < 1.0, "the floor cuts only the lower tip");
        assert!(solid.bounds().max.z - solid.bounds().min.z > 2.0);
        let collision = BuildingCollision {
            bounds: solid.bounds(),
            cuboids: vec![solid],
        };
        let contact = collision.ground_floor_contact_bounds().unwrap();
        let expected_half_depth = 2.0_f32.sqrt() - 1.0;
        assert!((contact.min.z + expected_half_depth).abs() < 0.000001);
        assert!((contact.max.z - expected_half_depth).abs() < 0.000001);
        assert_eq!(contact.min.y, 0.0);
        assert_eq!(contact.max.y, 0.0);
        assert_eq!(contact.min.x, -1.0);
        assert_eq!(contact.max.x, 1.0);
    }

    #[test]
    fn buried_slab_keeps_its_top_contact_and_upper_projection_has_none() {
        let slab = CollisionCuboid {
            centre: Vec3::new(0.0, -0.5, 0.0),
            size: Vec3::new(2.0, 1.0, 3.0),
            yaw_radians: 0.73,
            ..solid()
        };
        assert_eq!(slab.ground_contact_polygon().len(), 4);
        let upper = CollisionCuboid {
            centre: Vec3::Y * 4.0,
            ..slab
        };
        assert!(upper.ground_contact_polygon().is_empty());
    }
}

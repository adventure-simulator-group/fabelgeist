//! Void, reveal, closure and measured attachment metadata for roof windows.
use super::*;

impl RectangularRoofWindow {
    pub(super) fn void(&self, geometry: &mut ResolvedGeometry) -> ResolvedItemId {
        let Self {
            opening_id,
            owner,
            origin,
            tangent,
            outward,
            base,
            thickness,
            opening_width,
            clear_height,
            sill_height,
            ..
        } = self.clone();
        let sill_elevation = base + sill_height;
        let head_bottom = base + sill_height + clear_height;
        let exterior_depth_sign = if tangent.x.abs() > 0.5 {
            if outward.y >= 0.0 { 1 } else { -1 }
        } else if outward.x <= 0.0 {
            1
        } else {
            -1
        };
        let void_half = tangent.abs() * (opening_width * 0.5) + outward.abs() * (thickness * 0.5);
        wall_void(
            geometry,
            owner,
            10,
            ResolvedBounds {
                min: Vec3::new(
                    origin.x - void_half.x,
                    sill_elevation,
                    origin.y - void_half.y,
                ),
                max: Vec3::new(origin.x + void_half.x, head_bottom, origin.y + void_half.y),
            },
            opening_id,
            opening_width,
            opening_width,
            clear_height,
            clear_height,
            exterior_depth_sign,
        )
    }
    pub(super) fn reveals(&self, geometry: &mut ResolvedGeometry) -> Vec<ResolvedItemId> {
        let Self {
            owner,
            origin,
            tangent,
            outward,
            base,
            thickness,
            opening_width,
            clear_height,
            sill_height,
            ..
        } = self.clone();
        let sill_elevation = base + sill_height;
        let head_bottom = base + sill_height + clear_height;
        let void_half = tangent.abs() * (opening_width * 0.5) + outward.abs() * (thickness * 0.5);
        let reveal_depth = outward.abs() * (thickness * 0.5);
        let side_half = tangent.abs() * 0.008;
        let mut reveal_surfaces = Vec::new();
        for (slot, side, role) in [
            (10_u64, -1.0_f32, SurfaceRole::LeftJambReveal),
            (11, 1.0, SurfaceRole::RightJambReveal),
        ] {
            let plan = origin + tangent * side * opening_width * 0.5;
            reveal_surfaces.push(wall_surface(
                geometry,
                owner,
                slot,
                ResolvedBounds {
                    min: Vec3::new(
                        plan.x - reveal_depth.x - side_half.x,
                        sill_elevation,
                        plan.y - reveal_depth.y - side_half.y,
                    ),
                    max: Vec3::new(
                        plan.x + reveal_depth.x + side_half.x,
                        head_bottom,
                        plan.y + reveal_depth.y + side_half.y,
                    ),
                },
                role,
            ));
        }
        reveal_surfaces.push(wall_shaped_surface(
            geometry,
            owner,
            12,
            ResolvedBounds {
                min: Vec3::new(
                    origin.x - void_half.x,
                    sill_elevation,
                    origin.y - void_half.y,
                ),
                max: Vec3::new(
                    origin.x + void_half.x,
                    sill_elevation + 0.015,
                    origin.y + void_half.y,
                ),
            },
            SurfaceRole::WeatherSill,
            crate::ResolvedSurfaceShape::WeatherSill {
                interior_elevation_metres: sill_elevation,
                exterior_elevation_metres: sill_elevation - 0.035,
                drip_depth_metres: 0.025,
            },
        ));
        reveal_surfaces.push(wall_surface(
            geometry,
            owner,
            13,
            ResolvedBounds {
                min: Vec3::new(
                    origin.x - void_half.x,
                    head_bottom - 0.015,
                    origin.y - void_half.y,
                ),
                max: Vec3::new(origin.x + void_half.x, head_bottom, origin.y + void_half.y),
            },
            SurfaceRole::Intrados,
        ));
        reveal_surfaces.extend(self.mouths(geometry));
        reveal_surfaces
    }
    pub(super) fn mouths(&self, geometry: &mut ResolvedGeometry) -> Vec<ResolvedItemId> {
        let Self {
            owner,
            origin,
            tangent,
            outward,
            base,
            thickness,
            opening_width,
            clear_height,
            sill_height,
            ..
        } = self.clone();
        let sill_elevation = base + sill_height;
        let head_bottom = base + sill_height + clear_height;
        let mut reveal_surfaces = Vec::new();
        for (slot, sign, role) in [
            (14_u64, 1.0_f32, SurfaceRole::ExteriorThroat),
            (15, -1.0, SurfaceRole::InteriorMouth),
        ] {
            let plan = origin + outward * thickness * 0.5 * sign;
            let half = tangent.abs() * opening_width * 0.5 + outward.abs() * 0.006;
            reveal_surfaces.push(wall_surface(
                geometry,
                owner,
                slot,
                ResolvedBounds {
                    min: Vec3::new(plan.x - half.x, sill_elevation, plan.y - half.y),
                    max: Vec3::new(plan.x + half.x, head_bottom, plan.y + half.y),
                },
                role,
            ));
        }
        reveal_surfaces
    }
    pub(super) fn closures(
        &self,
        nodes: &WindowNodes,
        geometry: &mut ResolvedGeometry,
    ) -> Vec<ResolvedItemId> {
        let Self {
            owner,
            origin,
            outward,
            base,
            opening_width,
            clear_height,
            sill_height,
            ..
        } = self.clone();
        let sill_elevation = base + sill_height;
        let head_node = nodes.head_node;
        let closure = fixed_window_closure_policy();
        let mut closure_solids = Vec::new();
        for (layer_index, layer) in closure.layers.iter().copied().enumerate() {
            let role = if layer == crate::ClosureKind::LeadedGlazing {
                SolidRole::LeadedGlazing
            } else {
                SolidRole::OpeningClosure
            };
            let plan = origin - outward * (0.065 + layer_index as f32 * 0.035);
            closure_solids.push(wall_solid(
                geometry,
                owner,
                20 + layer_index as u64,
                Vec3::new(plan.x, sill_elevation + clear_height * 0.5, plan.y),
                self.local_size(
                    (opening_width * 0.92 - 0.10).max(0.04),
                    (clear_height * 0.92 - 0.10).max(0.04),
                    0.025,
                ),
                role,
                crate::ResolvedSolidShape::Cuboid,
                head_node,
            ));
        }
        closure_solids
    }
    pub(super) fn bearings(
        &self,
        nodes: &WindowNodes,
        geometry: &mut ResolvedGeometry,
    ) -> ([ResolvedItemId; 2], ResolvedItemId) {
        let Self {
            owner,
            origin,
            tangent,
            base,
            opening_width,
            clear_height,
            sill_height,
            head_height,
            ..
        } = self.clone();
        let head_bottom = base + sill_height + clear_height;
        let head_node = nodes.head_node;
        let spandrel_node = nodes.spandrel_node;
        let bearing_ids = [
            ResolvedItemId((4_u64 << 60) | (u64::from(owner.0) << 32) | 60),
            ResolvedItemId((4_u64 << 60) | (u64::from(owner.0) << 32) | 61),
        ];
        for (side, id) in [-1.0_f32, 1.0].into_iter().zip(bearing_ids) {
            let plan = origin + tangent * side * (opening_width * 0.5 + 0.03);
            geometry.support_interfaces.push(SupportInterface {
                id,
                owner,
                node: head_node,
                bounds: ResolvedBounds {
                    min: Vec3::new(plan.x - 0.08, head_bottom, plan.y - 0.08),
                    max: Vec3::new(plan.x + 0.08, head_bottom + 0.08, plan.y + 0.08),
                },
            });
        }
        let wall_above_interface = ResolvedItemId((4_u64 << 60) | (u64::from(owner.0) << 32) | 62);
        geometry.support_interfaces.push(SupportInterface {
            id: wall_above_interface,
            owner,
            node: spandrel_node,
            bounds: ResolvedBounds {
                min: Vec3::new(
                    origin.x - 0.08,
                    head_bottom + head_height - 0.025,
                    origin.y - 0.08,
                ),
                max: Vec3::new(
                    origin.x + 0.08,
                    head_bottom + head_height + 0.025,
                    origin.y + 0.08,
                ),
            },
        });
        (bearing_ids, wall_above_interface)
    }
}

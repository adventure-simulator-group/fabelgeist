//! Collision contact and height queries are different observations at cut edges.
use super::*;
use adventuresim_tactical_core::avian3d;
use adventuresim_tactical_core::city_layout::CompoundGradingPolicy;
use bevy::math::{Quat, Vec3};

const RAY_START_CONTACT_BOUNDS: f32 = 2.0;
const RAY_RANGE_CONTACT_BOUNDS: f32 = 3.0;

pub(super) struct CornerSupportProbe<'a> {
    terrain: &'a SceneTerrain,
    colliders: Vec<Collider>,
    contact_bound_metres: f32,
}

impl<'a> CornerSupportProbe<'a> {
    pub(super) fn new(terrain: &'a SceneTerrain) -> Self {
        Self {
            terrain,
            colliders: terrain.colliders(),
            contact_bound_metres: CompoundGradingPolicy::bounded_settlement()
                .limits
                .contact_tolerance_metres(),
        }
    }

    pub(super) fn measure(&self, outline: &[Vec2], floor: f32) -> Value {
        let probes: Vec<_> = outline.iter().map(|point| {
            let bearing = Vec3::new(point.x, floor, point.y);
            let start = bearing + Vec3::Y * (self.contact_bound_metres * RAY_START_CONTACT_BOUNDS);
            let max_ray = self.contact_bound_metres * RAY_RANGE_CONTACT_BOUNDS;
            let hit = self.colliders.iter().filter_map(|shape|
                shape.cast_ray(Vec3::ZERO, Quat::IDENTITY, start, -Vec3::Y, max_ray, false))
                .min_by(|a, b| a.0.total_cmp(&b.0));
            let nearest = self.colliders.iter().map(|shape|
                shape.project_point(Vec3::ZERO, Quat::IDENTITY, bearing, false).0)
                .min_by(|a, b| a.distance_squared(bearing).total_cmp(&b.distance_squared(bearing)))
                .expect("occupied terrain has physical support colliders");
            let cylinder = Collider::cylinder(adventuresim_core::combat::HUMANOID_COLLISION_RADIUS_METRES, 1.9);
            let bottom = -cylinder.aabb(Vec3::ZERO, Rotation::default()).min.y;
            let pose = avian3d::parry::math::Pose::from_translation(start + Vec3::Y * bottom);
            let cylinder_hit = self.colliders.iter().filter_map(|shape|
                avian3d::parry::query::cast_shapes(&pose, -Vec3::Y, cylinder.shape_scaled().as_ref(),
                    &avian3d::parry::math::Pose::IDENTITY, Vec3::ZERO, shape.shape_scaled().as_ref(),
                    avian3d::parry::query::ShapeCastOptions::with_max_time_of_impact(max_ray))
                    .expect("production cylinder/terrain pair supports shape casts"))
                .min_by(|a,b| a.time_of_impact.total_cmp(&b.time_of_impact));
            let below = self.terrain.surface_below(bearing + Vec3::Y * self.contact_bound_metres);
            json!({"bearing_metres":bearing,"ray_support_height_metres":hit.map(|hit| start.y - hit.0),
                "ray_support_within_bound":hit.is_some_and(|hit| (start.y-hit.0-floor).abs()<=self.contact_bound_metres),
                "cylinder_support_hit":cylinder_hit.map(|hit| json!({"foot_elevation_metres":start.y-hit.time_of_impact,"time_of_impact":hit.time_of_impact,"normal":hit.normal2})),
                "nearest_solid_metres":nearest,"nearest_solid_distance_metres":nearest.distance(bearing),
                "nearest_point_query_within_bound":nearest.distance(bearing)<=self.contact_bound_metres,
                "height_query_below_floor_metres":below.map(|sample| sample.elevation.metres()),
                "all_support_elevations_metres":self.terrain.support_elevations_at(*point).iter().map(|h|h.metres()).collect::<Vec<_>>()})
        }).collect();
        json!({"colliders_materialized":self.colliders.len(),"contact_bound_metres":self.contact_bound_metres,
            "scope":"Exact bearing vertices, no point perturbation. Nearest-point, exact rays, production-radius cylinder casts and unbound height queries are distinct observations. Cylinder corner tests include adjacent soil and do not prove occupied access. Full footprint support and soil intrusion require the separate triangle-intersection experiment.","probes":probes})
    }
}

pub(super) fn measure_enclosures(
    input: &TacticalSceneInput,
    terrain: &SceneTerrain,
) -> Result<Vec<Value>, adventuresim_tactical_core::scene_input::SceneInputError> {
    let mut enclosures = Vec::new();
    for compound in &input.compounds {
        let boundary = GeneratedBoundary::project(compound, terrain)?;
        let _collider = boundary.scene.fixed_support.collider();
        enclosures.push(json!({"property_id":compound.id,"member_building_ids":[compound.front_building_id,compound.rear_building_id],
            "gate_elevation_metres":boundary.elevation_metres,"cells":boundary.scene.fixed_support.cells.len(),
            "triangles":boundary.scene.fixed_support.cells.len()*8,
            "minimum_masonry_base_metres":boundary.scene.fixed_support.cells.iter().flat_map(|c|c.positions_metres[3..].iter())
                .map(|p|p.y+boundary.elevation_metres).fold(f32::INFINITY,f32::min)}));
    }
    Ok(enclosures)
}

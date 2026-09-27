//! Reusable timber geometry. Placement and physical dimensions remain exact.
use super::*;
use bevy::math::Mat4;

/// A timber member's metric geometry, independent of its position in a building.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimberComponent {
    pub size_metres: Vec3,
    pub interior: bool,
}

impl TimberComponent {
    pub fn meshes(self) -> Vec<LodMesh> {
        let mut detail = BuildingDetail { meshes: Vec::new() };
        append_cuboid_faces(
            &mut detail,
            if self.interior {
                BuildingLodMaterial::InteriorTimber
            } else {
                BuildingLodMaterial::Timber
            },
            Vec3::ZERO,
            self.size_metres,
            Quat::IDENTITY,
            None,
        );
        detail.meshes
    }
}

#[derive(Clone, Debug)]
pub struct TimberInstance {
    pub source: crate::ResolvedItemId,
    pub component: TimberComponent,
    pub transform: Mat4,
    pub uv_offset: Vec2,
    pub facade: bool,
}

/// Expanded surfaces and instanced members are disjoint render products of the
/// same tactical plan. Collision and structural assemblies are unchanged.
pub struct BuildingKit<'a> {
    plan: &'a BuildingPlan,
    excluded: BTreeSet<crate::ResolvedItemId>,
    facade_excluded: BTreeSet<crate::ResolvedItemId>,
    pub instances: Vec<TimberInstance>,
}

impl<'a> BuildingKit<'a> {
    pub fn new(plan: &'a BuildingPlan) -> Self {
        let facade_solids = crate::lod::component_solids(plan);
        let mut kit = Self {
            plan,
            excluded: BTreeSet::new(),
            facade_excluded: BTreeSet::new(),
            instances: Vec::new(),
        };
        for solid in &plan.resolved_geometry.solids {
            if !matches!(solid.shape, ResolvedSolidShape::Cuboid)
                || !(cuboids::is_fachwerk_member_role(solid.role)
                    || matches!(
                        solid.role,
                        SolidRole::RoofFraming
                            | SolidRole::BeamJoist
                            | SolidRole::FrameJoist
                            | SolidRole::FrameGirder
                            | SolidRole::RoofPlate
                    ))
            {
                continue;
            }
            let wall = wall_for_solid(plan, solid);
            // Gable compilers select individual faces; they retain their own
            // geometry until the component representation supports face masks.
            if wall.is_some_and(|wall| matches!(wall.source, crate::WallSourceId::RoofGable { .. }))
                || solid.role == SolidRole::FrameGableMember
            {
                continue;
            }
            let material = material_for_solid(plan, solid);
            if !matches!(
                material,
                BuildingLodMaterial::Timber | BuildingLodMaterial::InteriorTimber
            ) {
                continue;
            }
            let yaw = if solid.role == SolidRole::RoofFraming {
                -solid.yaw_radians
            } else {
                solid.yaw_radians
            };
            let rotation = Quat::from_rotation_y(yaw)
                * Quat::from_rotation_x(solid.crossfall_radians)
                * Quat::from_rotation_z(solid.longfall_radians);
            let (centre, size_metres) = cuboids::render_cuboid_placement(
                solid,
                wall,
                cuboids::is_fachwerk_member_role(solid.role),
                rotation,
            );
            let facade = facade_solids.contains(&solid.id);
            kit.excluded.insert(solid.id);
            if facade {
                kit.facade_excluded.insert(solid.id);
            }
            kit.instances.push(TimberInstance {
                source: solid.id,
                component: TimberComponent {
                    size_metres,
                    interior: material == BuildingLodMaterial::InteriorTimber,
                },
                transform: Mat4::from_rotation_translation(rotation, centre),
                uv_offset: crate::member_uv::phase(centre),
                facade,
            });
        }
        kit
    }

    pub fn detail(&self) -> BuildingDetail {
        compile_detail(self.plan, &self.excluded)
    }

    pub fn facade(&self) -> crate::BuildingLod {
        crate::lod::compile_components(self.plan, &self.facade_excluded)
    }
}

#[cfg(test)]
mod tests;

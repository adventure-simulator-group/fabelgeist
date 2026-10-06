//! Architectural window bars and casements, with leaf mesh compilation.

use crate::CollisionResult;
use crate::spatial_geometry::GeometryResult;
use std::collections::BTreeMap;

use bevy::math::{Quat, Vec2, Vec3};
use serde::{Deserialize, Serialize};

use crate::{
    BuildingPlan, ClosureKind, ClosureState, OpeningAssemblyId, OpeningUse, ResolvedItemId,
    ResolvedSolid, SolidRole,
};

const CASEMENT_OPEN_ANGLE_RADIANS: f32 = 80.0 * core::f32::consts::PI / 180.0;
const MINIMUM_LEAF_THICKNESS_METRES: f32 = 0.025;
const SWING_DIRECTION_PROBE_RADIANS: f32 = 0.01;
const THREE_BAR_MINIMUM_WIDTH_METRES: f32 = 0.9;
mod leaf;
pub use leaf::compile_window_leaf;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WindowLeafKind {
    LeadedGlass,
    TimberShutter,
}

impl WindowLeafKind {
    pub const fn material(self) -> crate::BuildingLodMaterial {
        match self {
            Self::LeadedGlass => crate::BuildingLodMaterial::Glass,
            Self::TimberShutter => crate::BuildingLodMaterial::InteriorTimber,
        }
    }
}

#[cfg(test)]
mod contract_tests;
mod spec;
use crate::spatial_geometry::{
    Architectural, CuboidDimensions, LeafDimensions, PlanDirection, Position, Radians,
};
pub use spec::{
    WindowBarPresence, WindowBarSpec, WindowError, WindowErrorCause, WindowResult, WindowSpec,
};

/// Compile fixed bars for every opening declaring an iron-bar closure layer.
///
/// Fixed bars compile independently of the opening's operable state. Invalid
/// frame or bar geometry returns [`crate::CollisionError`] with the bar's
/// source identity, which encodes the opening identity and bar ordinal.
pub fn compile_window_bars(plan: &BuildingPlan) -> CollisionResult<Vec<WindowBarSpec>> {
    plan.opening_assemblies
        .iter()
        .filter(|opening| opening.closure.layers.contains(&ClosureKind::IronBars))
        .flat_map(|opening| {
            let width = opening.profile.interior_width_metres();
            let height = opening.profile.clear_height_metres();
            let count = if width >= THREE_BAR_MINIMUM_WIDTH_METRES {
                3
            } else {
                2
            };
            (0..count).map(move |index| {
                let fraction = (index + 1) as f32 / (count + 1) as f32;
                let offset = (fraction - 0.5) * width;
                let plan_position = opening.frame.origin + opening.frame.tangent * offset;
                let source = ResolvedItemId((7_u64 << 60) | (opening.id.0 << 8) | index as u64);
                let admit = || -> GeometryResult<WindowBarSpec> {
                    // Bars retain the authored frame without normalizing set-out arithmetic.
                    PlanDirection::<Architectural>::from_normalized(opening.frame.tangent)?;
                    PlanDirection::<Architectural>::from_normalized(opening.frame.outward)?;
                    Ok(WindowBarSpec {
                        opening: opening.id,
                        source,
                        centre: Position::from_metres(Vec3::new(
                            plan_position.x,
                            opening.sill_elevation_metres + height * 0.5,
                            plan_position.y,
                        ))?,
                        size_metres: CuboidDimensions::from_metres(Vec3::new(
                            0.035, height, 0.035,
                        ))?,
                        yaw_radians: Radians::new(
                            -opening.frame.tangent.y.atan2(opening.frame.tangent.x),
                        )?,
                    })
                };
                admit().map_err(|cause| crate::CollisionError {
                    source_id: source,
                    cause,
                })
            })
        })
        .collect()
}

/// Compile operable exterior windows with an inside-room binding.
///
/// Ineligible openings are excluded. A missing supported closure or invalid
/// selected leaf geometry returns [`WindowError`] with the opening identity and
/// available closure-source identities.
pub fn compile_operable_windows(
    plan: &BuildingPlan,
) -> WindowResult<Vec<WindowSpec<Architectural>>> {
    let solids = plan
        .resolved_geometry
        .solids
        .iter()
        .map(|solid| (solid.id, solid))
        .collect::<BTreeMap<_, _>>();
    plan.opening_assemblies
        .iter()
        .filter(|opening| {
            opening.use_kind == OpeningUse::Window
                && opening.closure.state == ClosureState::Operable
                && opening.frame.inside_room.is_some()
                && opening.frame.outside_room.is_none()
        })
        .map(|opening| {
            let solid = opening
                .closure_solids
                .iter()
                .find_map(|id| {
                    solids.get(id).copied().filter(|solid| {
                        matches!(
                            solid.role,
                            SolidRole::LeadedGlazing | SolidRole::OpeningClosure
                        )
                    })
                })
                .ok_or_else(|| WindowError {
                    opening: opening.id,
                    source_id: opening.closure_solids.first().copied(),
                    cause: WindowErrorCause::MissingClosure {
                        sources: opening.closure_solids.clone(),
                    },
                })?;
            let admit = || {
                WindowSpec::from_solid(
                    opening.id,
                    PlanDirection::from_vector(opening.frame.tangent)?,
                    PlanDirection::from_vector(opening.frame.outward)?,
                    if opening.closure.layers.contains(&ClosureKind::IronBars) {
                        WindowBarPresence::Present
                    } else {
                        WindowBarPresence::Absent
                    },
                    solid,
                )
            };
            admit().map_err(|cause| WindowError {
                opening: opening.id,
                source_id: Some(solid.id),
                cause: cause.into(),
            })
        })
        .collect()
}

impl WindowSpec<Architectural> {
    fn from_solid(
        opening: OpeningAssemblyId,
        tangent: PlanDirection<Architectural>,
        outward: PlanDirection<Architectural>,
        bars: WindowBarPresence,
        solid: &ResolvedSolid,
    ) -> GeometryResult<Self> {
        let tangent_owner = tangent;
        let outward_owner = outward;
        let tangent = tangent_owner.vector();
        let outward = outward_owner.vector();
        let width =
            tangent.x.abs() * solid.size.metres().x + tangent.y.abs() * solid.size.metres().z;
        let thickness =
            outward.x.abs() * solid.size.metres().x + outward.y.abs() * solid.size.metres().z;
        let hinge_centre =
            solid.centre.metres() - Vec3::new(tangent.x, 0.0, tangent.y) * width * 0.5;
        let positive_swing = Quat::from_rotation_y(SWING_DIRECTION_PROBE_RADIANS)
            * Vec3::new(tangent.x, 0.0, tangent.y);
        let enters_room = Vec2::new(positive_swing.x, positive_swing.z).dot(-outward) > 0.0;
        Ok(Self {
            leaf: if solid.role == SolidRole::LeadedGlazing {
                WindowLeafKind::LeadedGlass
            } else {
                WindowLeafKind::TimberShutter
            },
            opening,
            source: solid.id,
            closed_centre: solid.centre,
            hinge_centre: Position::from_metres(hinge_centre)?,
            size_metres: LeafDimensions::from_metres(Vec3::new(
                width,
                solid.size.metres().y,
                thickness.max(MINIMUM_LEAF_THICKNESS_METRES),
            ))?,
            closed_yaw_radians: Radians::new(-tangent.y.atan2(tangent.x))?,
            tangent: tangent_owner,
            outward: outward_owner,
            open_angle_radians: Radians::new(if enters_room {
                CASEMENT_OPEN_ANGLE_RADIANS
            } else {
                -CASEMENT_OPEN_ANGLE_RADIANS
            })?,
            bars,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BuildingArchetype, BuildingProgram, generate};

    #[test]
    fn civilian_seed_matrix_contains_operable_fixed_and_barred_windows() {
        let mut operable = 0;
        let mut fixed = 0;
        let mut barred = 0;
        for seed in 0..20 {
            let plan = generate(&BuildingProgram::fixture(
                BuildingArchetype::TownHouse,
                seed,
            ))
            .expect("town house");
            let windows = compile_operable_windows(&plan).unwrap();
            operable += windows.len();
            barred += windows
                .iter()
                .filter(|window| window.bars == WindowBarPresence::Present)
                .count();
            fixed += plan
                .opening_assemblies
                .iter()
                .filter(|opening| {
                    opening.use_kind == OpeningUse::Window
                        && opening.closure.state == ClosureState::Closed
                })
                .count();
        }
        assert!(operable > 0);
        assert!(fixed > 0);
        assert!(barred > 0);
    }

    #[test]
    fn operable_windows_swing_toward_the_inside() {
        let plan = generate(&BuildingProgram::fixture(BuildingArchetype::TownHouse, 42)).unwrap();
        for window in compile_operable_windows(&plan).unwrap() {
            let closed_arm = Vec3::new(window.tangent.vector().x, 0.0, window.tangent.vector().y);
            let open_arm = Quat::from_rotation_y(window.open_angle_radians.radians()) * closed_arm;
            assert!(Vec2::new(open_arm.x, open_arm.z).dot(-window.outward.vector()) > 0.0);
        }
    }
}

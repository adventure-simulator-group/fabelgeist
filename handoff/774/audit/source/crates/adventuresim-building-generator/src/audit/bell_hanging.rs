//! A bell must have a continuous physical connection to its fixed bearing rails.
use super::*;

const CONTACT_TOLERANCE_METRES: f32 = 0.001;
const MINIMUM_CONTACT_AREA_SQUARE_METRES: f32 = 0.000_001;
// Authored modest ringing envelope; it is not an unrestricted full-circle bell.
const RINGING_CLEARANCE_RADIANS: crate::spatial_geometry::Radians =
    match crate::spatial_geometry::Radians::new(std::f32::consts::FRAC_PI_6) {
        Ok(angle) => angle,
        Err(_) => unreachable!(),
    };

pub(super) fn audit(plan: &BuildingPlan, issues: &mut Vec<AuditIssue>) {
    for bell in plan
        .resolved_geometry
        .solids
        .iter()
        .filter(|s| s.role == SolidRole::ChurchBell)
    {
        let parts = plan
            .resolved_geometry
            .solids
            .iter()
            .filter(|s| {
                s.owner == bell.owner
                    && matches!(
                        s.role,
                        SolidRole::ChurchBellFrame
                            | SolidRole::ChurchBellFitting
                            | SolidRole::ChurchBellAxle
                            | SolidRole::ChurchBellHeadstock
                            | SolidRole::ChurchBellBearing
                            | SolidRole::ChurchBellCrown
                    )
            })
            .collect::<Vec<_>>();
        let mut reached = BTreeSet::from([bell.id]);
        let mut frontier = vec![bell];
        while let Some(part) = frontier.pop() {
            for candidate in &parts {
                if !reached.contains(&candidate.id) && contacts(part, candidate) {
                    reached.insert(candidate.id);
                    frontier.push(candidate);
                }
            }
        }
        let rails = parts
            .iter()
            .filter(|part| {
                part.role == SolidRole::ChurchBellFrame
                    && part.size.metres().z > part.size.metres().x
                    && plan.resolved_geometry.solids.iter().any(|support| {
                        !parts.iter().any(|part| part.id == support.id)
                            && support.id != bell.id
                            && contacts(part, support)
                    })
            })
            .count();
        if rails < 2 || parts.is_empty() || parts.iter().any(|s| !reached.contains(&s.id)) {
            issues.push(issue(
                "disconnected_bell_suspension",
                format!(
                    "bell {} lacks a continuous crown, headstock and two-rail bearing connection",
                    bell.id.0
                ),
            ));
        }
        let clear = match super::bell_swing::is_clear(plan, bell, RINGING_CLEARANCE_RADIANS) {
            Ok(clear) => clear,
            Err(error) => {
                issues.push(issue("invalid_bell_collision", error.to_string()));
                continue;
            }
        };
        if !matches!(clear, super::bell_swing::BellSwingAssessment::Clear) {
            issues.push(issue(
                "blocked_bell_swing",
                format!(
                    "bell {} cannot traverse its authored ringing envelope: {clear:?}",
                    bell.id.0
                ),
            ));
        }
    }
}

pub(super) fn moving(role: SolidRole) -> bool {
    matches!(
        role,
        SolidRole::ChurchBell
            | SolidRole::ChurchBellAxle
            | SolidRole::ChurchBellCrown
            | SolidRole::ChurchBellHeadstock
            | SolidRole::ChurchBellFitting
    )
}

fn contacts(a: &ResolvedSolid, b: &ResolvedSolid) -> bool {
    let overlap = (a.centre.metres() + a.size.metres() * 0.5)
        .min(b.centre.metres() + b.size.metres() * 0.5)
        - (a.centre.metres() - a.size.metres() * 0.5)
            .max(b.centre.metres() - b.size.metres() * 0.5);
    let mut extents = overlap.to_array();
    extents.sort_by(f32::total_cmp);
    extents[0] >= -CONTACT_TOLERANCE_METRES
        && extents[1] * extents[2] > MINIMUM_CONTACT_AREA_SQUARE_METRES
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn churches_reject_a_detached_crown_even_with_unchanged_support_ids() {
        for archetype in [
            BuildingArchetype::Cathedral,
            BuildingArchetype::ParishChurch,
        ] {
            let mut plan = crate::generate(&crate::BuildingProgram::fixture(
                archetype,
                fabelgeist_determinism::Seed::from_u64(42),
            ))
            .unwrap();
            let mut issues = Vec::new();
            audit(&plan, &mut issues);
            assert!(issues.is_empty(), "{archetype:?}: {issues:?}");
            for part in &mut plan.resolved_geometry.solids {
                if part.role == SolidRole::ChurchBellCrown {
                    {
                        let mut native_geometry = part.centre.metres();
                        native_geometry.y += 0.7;
                        part.centre =
                            crate::spatial_geometry::Position::from_metres(native_geometry)
                                .unwrap();
                    };
                }
            }
            audit(&plan, &mut issues);
            assert!(
                issues
                    .iter()
                    .any(|issue| issue.code == "disconnected_bell_suspension")
            );
        }
    }

    #[test]
    fn journals_and_bell_clear_fixed_bearings_through_the_authored_swing() {
        use adventuresim_world_schema::settlement_buildings::BuildingUse;
        for usage in [BuildingUse::Chapel, BuildingUse::ParishChurch] {
            let plan = crate::generate(&crate::BuildingProgram::settlement(
                BuildingArchetype::ParishChurch,
                Some(usage),
                fabelgeist_determinism::Seed::from_u64(42),
            ))
            .unwrap();
            let bell = plan
                .resolved_geometry
                .solids
                .iter()
                .find(|s| s.role == SolidRole::ChurchBell)
                .unwrap();
            assert_eq!(
                super::bell_swing::is_clear(&plan, bell, RINGING_CLEARANCE_RADIANS).unwrap(),
                super::bell_swing::BellSwingAssessment::Clear
            );
            let collision = crate::compile_building_collision(&plan).unwrap();
            for axle in plan
                .resolved_geometry
                .solids
                .iter()
                .filter(|s| s.role == SolidRole::ChurchBellAxle)
            {
                let parts = collision
                    .cuboids
                    .iter()
                    .filter(|part| part.source == axle.id)
                    .collect::<Vec<_>>();
                assert!(!parts.is_empty());
                for part in parts {
                    let rotation = Quat::from_euler(
                        bevy::math::EulerRot::YXZ,
                        part.yaw_radians.radians(),
                        part.crossfall_radians.radians(),
                        part.longfall_radians.radians(),
                    );
                    for y in [-1.0, 1.0] {
                        for z in [-1.0, 1.0] {
                            let point = rotation
                                * Vec3::new(
                                    0.0,
                                    y * part.size.metres().y * 0.5,
                                    z * part.size.metres().z * 0.5,
                                );
                            assert!(
                                Vec2::new(point.y, point.z).length()
                                    <= axle.size.metres().y * 0.5 + 0.00001
                            );
                        }
                    }
                }
            }
        }
    }
}

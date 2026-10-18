//! Authored wall frames must describe an orthogonal unit basis, cardinal except for apse walls.
use super::*;
use crate::WallSourceId;
pub(super) fn valid(wall: &crate::WallAssembly) -> bool {
    !((wall.frame.tangent.length() - 1.0).abs() > 0.001
        || (wall.frame.outward.length() - 1.0).abs() > 0.001
        || wall.frame.tangent.dot(wall.frame.outward).abs() > 0.001
        || (!matches!(wall.source, WallSourceId::ChurchApse { .. })
            && !matches!(
                (wall.frame.tangent, wall.frame.outward),
                (
                    Vec2 {
                        x: -1.0 | 0.0 | 1.0,
                        y: -1.0 | 0.0 | 1.0
                    },
                    Vec2 {
                        x: -1.0 | 0.0 | 1.0,
                        y: -1.0 | 0.0 | 1.0
                    }
                )
            )))
}

pub(super) fn audit_radial(
    plan: &BuildingPlan,
    wall: &crate::WallAssembly,
    solids: &std::collections::HashMap<ResolvedItemId, &ResolvedSolid>,
    issues: &mut Vec<AuditIssue>,
) {
    match wall.source {
        crate::WallSourceId::RoundTower { tower_index } => {
            let tower = plan.towers.get(tower_index);
            let radial = wall.radial_frame;
            let shell = wall.host_solids.first().and_then(|id| solids.get(id));
            let valid = tower.is_some_and(|tower| {
                radial.is_some_and(|radial| {
                    radial.centre.distance(tower.centre_metres()) <= 0.001
                        && radial.reference_outward.length_squared() > 0.99
                }) && shell.is_some_and(|shell| {
                    matches!(
                        shell.shape,
                        crate::ResolvedSolidShape::RoundTowerShell {
                            outer_radius_metres,
                            inner_radius_metres,
                            chord_interfaces,
                        } if (outer_radius_metres - tower.radius_metres()).abs() <= 0.001
                            && (outer_radius_metres - inner_radius_metres
                                - wall.thickness_metres).abs() <= 0.001
                            && chord_interfaces
                                == [tower.chord_interface, tower.secondary_chord_interface]
                    )
                })
            });
            if !valid {
                issues.push(issue(
                    "invalid_round_wall_authority",
                    format!("round wall {} drifts from its grid tower shell", wall.id.0),
                ));
            }
        }
        WallSourceId::StoreyWall { .. }
        | WallSourceId::CurtainWall { .. }
        | WallSourceId::WorkplaceWall { .. }
        | WallSourceId::ArtilleryCurtain { .. }
        | WallSourceId::SquareTowerFace { .. }
        | WallSourceId::ChurchClerestory { .. }
        | WallSourceId::RoofGable { .. }
        | WallSourceId::RoofChildFront { .. }
        | WallSourceId::ChurchExterior { .. }
        | WallSourceId::ChurchArcade { .. }
        | WallSourceId::ChurchCrossing { .. }
        | WallSourceId::ChurchApse { .. }
        | WallSourceId::ChurchTowerFace { .. } => {
            if wall.radial_frame.is_some() {
                issues.push(issue(
                    "invalid_wall_authority",
                    format!("linear wall {} declares a radial frame", wall.id.0),
                ));
            }
        }
        WallSourceId::ArtilleryRondel { .. } => {}
    }
}

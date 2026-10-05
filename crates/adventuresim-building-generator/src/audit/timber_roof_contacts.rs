//! Physical roof envelope and cross-authority contact diagnostics for a timber frame.
use super::*;
pub(super) fn audit(
    plan: &BuildingPlan,
    issues: &mut Vec<AuditIssue>,
) -> Result<(), crate::GenerationError> {
    let roof_envelope_intrusions = timber_roof_envelope_intrusions(plan);
    if !roof_envelope_intrusions.is_empty() {
        issues.push(issue(
            "timber_intrudes_through_roof",
            format!(
                "roof-construction members leave the authoritative roof envelope: {:?}",
                roof_envelope_intrusions
            ),
        ));
    }
    let exposed_child_supports = exposed_roof_child_support_posts(plan);
    if !exposed_child_supports.is_empty() {
        issues.push(issue(
            "exposed_roof_child_support",
            format!(
                "roof children contain freestanding generic support posts outside their declared curb/front/cheek authority: {:?}",
                exposed_child_supports
            ),
        ));
    }
    let oversized_child_flashings = oversized_child_roof_flashings(plan);
    if !oversized_child_flashings.is_empty() {
        issues.push(issue(
            "invalid_child_roof_flashing_profile",
            format!(
                "child-roof flashing rises above the seated civilian seam profile: {:?}",
                oversized_child_flashings
            ),
        ));
    }
    let invalid_child_drainage = invalid_attached_child_drainage(plan);
    if !invalid_child_drainage.is_empty() {
        issues.push(issue(
            "invalid_child_roof_drainage",
            format!(
                "attached civilian roof drains bypass the containing parent weather face: {:?}",
                invalid_child_drainage
            ),
        ));
    }
    let invalid_child_curb = invalid_dormer_trimmer_envelope(plan);
    if !invalid_child_curb.is_empty() {
        issues.push(issue(
            "invalid_dormer_trimmer_envelope",
            format!(
                "dormer trimmers project outside the exact front-to-rear child enclosure: {:?}",
                invalid_child_curb
            ),
        ));
    }
    let oversized_child_gutters = oversized_attached_child_gutters(plan);
    if !oversized_child_gutters.is_empty() {
        issues.push(issue(
            "invalid_child_roof_drainage_profile",
            format!(
                "attached child roof uses a full-building gutter profile: {:?}",
                oversized_child_gutters
            ),
        ));
    }
    let unseated_dormers = unseated_gabled_dormer_roofs(plan);
    if !unseated_dormers.is_empty() {
        issues.push(issue(
            "unseated_dormer_roof",
            format!(
                "gabled dormer retains a free rear verge or misses the parent weather plane: {:?}",
                unseated_dormers
            ),
        ));
    }
    let coplanar_openings = coplanar_timber_opening_faces(plan);
    if !coplanar_openings.is_empty() {
        issues.push(issue(
            "coplanar_timber_opening_face",
            format!(
                "timber-wall opening solids reach the exposed frame plane: {:?}",
                coplanar_openings
            ),
        ));
    }
    audit_intersections(plan, issues)
}
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
struct ContactRoles {
    left: Option<SolidRole>,
    right: Option<SolidRole>,
}
impl ContactRoles {
    fn label(self) -> String {
        let label = |role: Option<SolidRole>| {
            role.map_or_else(|| "missing".to_owned(), |role| format!("{role:?}"))
        };
        format!("{} x {}", label(self.left), label(self.right))
    }
}
fn audit_intersections(
    plan: &BuildingPlan,
    issues: &mut Vec<AuditIssue>,
) -> Result<(), crate::GenerationError> {
    let undeclared_intersections = undeclared_timber_intersections(plan)?;
    if !undeclared_intersections.is_empty() {
        let mut role_counts = std::collections::BTreeMap::<ContactRoles, usize>::new();
        for (left, right) in &undeclared_intersections {
            let roles = [left, right].map(|id| {
                plan.resolved_geometry
                    .solids
                    .iter()
                    .find(|solid| solid.id == *id)
                    .map(|solid| solid.role)
            });
            *role_counts
                .entry(ContactRoles {
                    left: roles[0],
                    right: roles[1],
                })
                .or_default() += 1;
        }
        let mut seen_roles = std::collections::HashSet::new();
        let sample = undeclared_intersections
            .iter()
            .filter(|(left, right)| {
                let key = [left, right]
                    .into_iter()
                    .filter_map(|id| {
                        plan.resolved_geometry
                            .solids
                            .iter()
                            .find(|solid| solid.id == *id)
                    })
                    .map(|solid| solid.role)
                    .collect::<Vec<_>>();
                seen_roles.insert(key)
            })
            .take(20)
            .map(|(left, right)| {
                let describe = |id: ResolvedItemId| {
                    plan.resolved_geometry
                        .solids
                        .iter()
                        .find(|solid| solid.id == id)
                        .map(|solid| (id, solid.role, solid.owner, solid.centre, solid.size))
                };
                (describe(*left), describe(*right))
            })
            .collect::<Vec<_>>();
        // Sorting display labels belongs to the diagnostic adapter. Role
        // grouping and sample selection above use the closed SolidRole owner.
        let role_counts = role_counts
            .into_iter()
            .map(|(roles, count)| (roles.label(), count))
            .collect::<BTreeMap<_, _>>();
        issues.push(issue(
            "undeclared_timber_intersection",
            format!(
                "{} timber pairs overlap without an exact typed joint or bearing interface; roles: {:?}; first pairs (id, role, owner): {:?}",
                undeclared_intersections.len(), role_counts, sample
            ),
        ));
    }
    Ok(())
}

pub(super) fn child_front_join(
    plan: &BuildingPlan,
    frame: &crate::TimberFrameAssembly,
    member: &crate::TimberFrameMember,
    b: &ResolvedSolid,
) -> bool {
    matches!(
        member.role,
        crate::TimberMemberRole::GableTie
            | crate::TimberMemberRole::GablePost
            | crate::TimberMemberRole::WallPlate
            | crate::TimberMemberRole::Sill
            | crate::TimberMemberRole::Rafter
            | crate::TimberMemberRole::Collar
            | crate::TimberMemberRole::Purlin
    ) && frame.bays.iter().any(|bay| {
        bay.member_ids.contains(&member.id)
            && bay.wall.is_some_and(|wall_id| {
                plan.wall_assemblies
                    .iter()
                    .find(|wall| wall.id == wall_id)
                    .is_some_and(|wall| {
                        matches!(
                            wall.source,
                            crate::WallSourceId::RoofChildFront { roof }
                                if plan.roof_assemblies.iter().any(|assembly| {
                                    (assembly.id == roof && assembly.owner == b.owner)
                                        || assembly.children.iter().any(|child| {
                                            child.child == roof
                                                && child.flashing_ids.contains(&b.id)
                                        })
                                })
                        )
                    })
            })
    })
}

use super::*;
use crate::{
    BuildingLodLevel, compile_building_collision, compile_building_detail, compile_building_lod,
};

fn fixture(archetype: BuildingArchetype, seed: u64) -> BuildingPlan {
    crate::generate(&BuildingProgram::fixture(archetype, seed)).unwrap()
}

fn openings(plan: &BuildingPlan) -> impl Iterator<Item = &crate::OpeningAssembly> {
    plan.opening_assemblies
        .iter()
        .filter(|opening| matches!(opening.host_source, crate::WallSourceId::RoofGable { .. }))
}

#[test]
fn eligible_gables_have_supported_apertures() {
    for archetype in [
        BuildingArchetype::TownHouse,
        BuildingArchetype::FachwerkMerchantHouse,
    ] {
        for seed in [42, 47, 101] {
            let plan = fixture(archetype, seed);
            assert_eq!(openings(&plan).count(), 2);
            for opening in openings(&plan) {
                assert_eq!(opening.profile.exterior_width_metres(), 0.70);
                assert_eq!(opening.profile.clear_height_metres(), 1.00);
                assert!(opening.frame.inside_room.is_none());
                assert_eq!(opening.closure.state, crate::ClosureState::Closed);
                let crate::OpeningHeadKind::TimberFrameMember { member } = opening.head_kind else {
                    panic!("missing original rail lintel");
                };
                assert!(member.0 < 1_000_000, "lintel must retain its original ID");
            }
        }
    }
}

#[test]
fn aperture_pitch_edits_are_atomic_and_unchanged_pitch_is_a_noop() {
    let mut plan = fixture(BuildingArchetype::FachwerkMerchantHouse, 42);
    let crate::WallSourceId::RoofGable { roof, .. } = openings(&plan).next().unwrap().host_source
    else {
        unreachable!()
    };
    let pitch = plan
        .roof_assemblies
        .iter()
        .find(|r| r.id == roof)
        .unwrap()
        .faces[0]
        .pitch_degrees;
    let before = serde_json::to_vec(&plan).unwrap();
    assert_eq!(set_roof_pitch(&mut plan, roof, pitch), Ok(()));
    assert_eq!(serde_json::to_vec(&plan).unwrap(), before);
    assert_eq!(
        set_roof_pitch(&mut plan, roof, pitch - 1.0),
        Err(RoofEditError::TopologyEvent)
    );
    assert_eq!(serde_json::to_vec(&plan).unwrap(), before);
}

fn ray_hit(triangle: [Vec3; 3], start: Vec3, direction: Vec3) -> bool {
    let edge = triangle[1] - triangle[0];
    let side = triangle[2] - triangle[0];
    let p = direction.cross(side);
    let determinant = edge.dot(p);
    if determinant.abs() < 0.000001 {
        return false;
    }
    let offset = start - triangle[0];
    let u = offset.dot(p) / determinant;
    let q = offset.cross(edge);
    let v = direction.dot(q) / determinant;
    let t = side.dot(q) / determinant;
    u >= -0.00001 && v >= -0.00001 && u + v <= 1.00001 && (0.0..=1.0).contains(&t)
}

fn meshes_hit(meshes: &[crate::LodMesh], point: Vec3, outward: Vec3) -> bool {
    meshes.iter().any(|mesh| {
        mesh.indices.as_chunks::<3>().0.iter().any(|indices| {
            ray_hit(
                indices.map(|i| mesh.vertices[i as usize].position),
                point + outward * 0.5,
                -outward,
            )
        })
    })
}

#[test]
fn detail_facade_shell_and_collision_share_clear_aperture_and_fixed_glass() {
    for archetype in [
        BuildingArchetype::TownHouse,
        BuildingArchetype::FachwerkMerchantHouse,
    ] {
        let plan = fixture(archetype, 42);
        let closure_ids = openings(&plan)
            .flat_map(|o| o.closure_solids.iter().copied())
            .collect::<BTreeSet<_>>();
        let mut open = plan.clone();
        open.resolved_geometry
            .solids
            .retain(|solid| !closure_ids.contains(&solid.id));
        let closed_meshes = [
            compile_building_detail(&plan).meshes,
            compile_building_lod(&plan, BuildingLodLevel::Facade).meshes,
            compile_building_lod(&plan, BuildingLodLevel::Shell).meshes,
        ];
        let open_meshes = [
            compile_building_detail(&open).meshes,
            compile_building_lod(&open, BuildingLodLevel::Facade).meshes,
            compile_building_lod(&open, BuildingLodLevel::Shell).meshes,
        ];
        let collision = compile_building_collision(&plan);
        let open_collision = compile_building_collision(&open);
        for opening in openings(&plan) {
            let frame = opening.frame;
            let outward = Vec3::new(frame.outward.x, 0.0, frame.outward.y);
            let point = |x: f32, y: f32| {
                let p = frame.origin + frame.tangent * x;
                Vec3::new(p.x, opening.sill_elevation_metres + y, p.y)
            };
            for x in [-0.8, 0.8] {
                let start = point(x, 0.5) - outward * 0.5;
                assert!(
                    closed_meshes[0].iter().any(|mesh| {
                        mesh.material == crate::BuildingLodMaterial::InteriorPlaster
                            && mesh.indices.as_chunks::<3>().0.iter().any(|indices| {
                                let vertices = indices.map(|i| mesh.vertices[i as usize]);
                                let triangle = vertices.map(|v| v.position);
                                vertices[0].normal.dot(-outward) > 0.999
                                    && (triangle[1] - triangle[0])
                                        .cross(triangle[2] - triangle[0])
                                        .dot(-outward)
                                        > 0.0
                                    && ray_hit(triangle, start, outward)
                            })
                    }),
                    "{archetype:?} missing front-facing interior plaster beside the bay at {x}"
                );
            }
            for x in [-0.349, 0.0, 0.349] {
                for y in [0.001, 0.5, 0.999] {
                    let sample = point(x, y);
                    for (index, meshes) in open_meshes.iter().enumerate() {
                        assert!(
                            !meshes_hit(meshes, sample, outward),
                            "{archetype:?} level {index} blocked aperture {x},{y}"
                        );
                        assert!(
                            meshes_hit(&closed_meshes[index], sample, outward),
                            "level {index} missing glass {x},{y}"
                        );
                    }
                    let probe = crate::CollisionCuboid {
                        source: ResolvedItemId(0),
                        centre: sample,
                        size: Vec3::new(outward.x.abs() + 0.001, 0.001, outward.z.abs() + 0.001),
                        yaw_radians: 0.0,
                        crossfall_radians: 0.0,
                        longfall_radians: 0.0,
                    };
                    assert!(!open_collision.cuboids.iter().any(|c| c.intersects(probe)));
                    assert!(
                        collision
                            .cuboids
                            .iter()
                            .any(|c| closure_ids.contains(&c.source) && c.intersects(probe))
                    );
                }
            }
            for sample in [
                point(-0.45, 0.5),
                point(0.45, 0.5),
                point(0.0, -0.05),
                point(0.0, 1.05),
            ] {
                let probe = crate::CollisionCuboid {
                    source: ResolvedItemId(0),
                    centre: sample,
                    size: Vec3::new(outward.x.abs() + 0.001, 0.001, outward.z.abs() + 0.001),
                    yaw_radians: 0.0,
                    crossfall_radians: 0.0,
                    longfall_radians: 0.0,
                };
                assert!(
                    open_collision.cuboids.iter().any(|c| c.intersects(probe)),
                    "missing surrounding collision"
                );
                for meshes in &open_meshes {
                    assert!(
                        meshes_hit(meshes, sample, outward),
                        "missing surrounding material"
                    );
                }
            }
        }
    }
}

#[test]
fn residual_enclosures_are_closed_and_gable_audit_rejects_mutations() {
    let plan = fixture(BuildingArchetype::TownHouse, 42);
    for roof in &plan.roof_assemblies {
        for face in &roof.enclosure_faces {
            let triangles = crate::tessellate_roof_enclosure(face, &plan.wall_assemblies);
            let positions = triangles
                .iter()
                .flat_map(|t| t.positions.map(|p| p.to_array()))
                .collect::<Vec<_>>();
            let indices = (0..positions.len() as u32).collect::<Vec<_>>();
            assert!(crate::audit_triangle_mesh(&positions, &indices).passes_closed_solid());
        }
    }
    let opening = openings(&plan).next().unwrap().clone();
    let mut missing_jamb = plan.clone();
    missing_jamb
        .resolved_geometry
        .solids
        .retain(|s| s.id != opening.jamb_solids[0]);
    let mut shifted_void = plan.clone();
    let void = shifted_void
        .resolved_geometry
        .voids
        .iter_mut()
        .find(|v| v.id == opening.void_id)
        .unwrap();
    void.bounds.min.x += 0.15;
    void.bounds.max.x += 0.15;
    let mut oversized = plan.clone();
    oversized
        .wall_assemblies
        .iter_mut()
        .find(|w| w.id == opening.host_wall)
        .unwrap()
        .length_metres += 0.4;
    let mut unowned = plan.clone();
    unowned
        .wall_assemblies
        .iter_mut()
        .find(|w| w.id == opening.host_wall)
        .unwrap()
        .source = crate::WallSourceId::RoofGable {
        roof: RoofAssemblyId(999),
        enclosure: ResolvedItemId(999),
    };
    let mut displaced = plan.clone();
    displaced
        .resolved_geometry
        .solids
        .iter_mut()
        .find(|s| s.id == opening.jamb_solids[0])
        .unwrap()
        .centre
        .z += 0.5;
    let mut rotated_glass = plan.clone();
    rotated_glass
        .resolved_geometry
        .solids
        .iter_mut()
        .find(|s| s.id == opening.closure_solids[0])
        .unwrap()
        .longfall_radians = 0.2;
    for changed in [
        missing_jamb,
        shifted_void,
        oversized,
        unowned,
        displaced,
        rotated_glass,
    ] {
        let issues = crate::audit_plan(&changed);
        assert!(
            issues.iter().any(|i| i.code == "roof_gable_enclosure_gap"),
            "mutation escaped gable coverage: {issues:?}"
        );
    }
}

#[test]
fn both_ridge_axes_preserve_original_members_and_omit_blocked_bays() {
    use bevy::math::Quat;
    let plan = fixture(BuildingArchetype::TownHouse, 42);
    for angle in [0.0, std::f32::consts::FRAC_PI_2] {
        let rotation = Quat::from_rotation_y(angle);
        let mut original = plan
            .timber_frame
            .as_ref()
            .unwrap()
            .members
            .iter()
            .filter(|m| m.id.0 < 1_000_000)
            .cloned()
            .collect::<Vec<_>>();
        for member in &mut original {
            member.start = rotation * member.start;
            member.end = rotation * member.end;
        }
        let ids = original.iter().map(|m| m.solid).collect::<BTreeSet<_>>();
        let mut geometry = ResolvedGeometry {
            solids: plan
                .resolved_geometry
                .solids
                .iter()
                .filter(|s| ids.contains(&s.id))
                .cloned()
                .collect(),
            ..Default::default()
        };
        for solid in &mut geometry.solids {
            solid.centre = rotation * solid.centre;
            solid.yaw_radians += angle;
        }
        let original_solids = serde_json::to_vec(&geometry.solids).unwrap();
        let mut roofs = plan.roof_assemblies.clone();
        for roof in &mut roofs {
            for face in &mut roof.enclosure_faces {
                face.inset_walls.clear();
                for p in &mut face.polygon {
                    *p = rotation * *p;
                }
            }
        }
        let mut builder = TimberFrameBuilder::new(
            &mut geometry,
            GeometryOwnerId(82_000),
            crate::StructuralTimberMaterial::Fir,
        );
        builder.members = original.clone();
        let face = &roofs[0].enclosure_faces[0];
        let candidate = Candidate::find(&builder, face).expect("unblocked bay");
        let point = candidate.frame.origin
            + candidate.frame.tangent * (CLEAR_WIDTH_METRES + JAMB_WIDTH_METRES) * 0.5
            + candidate.frame.outward * (BAY_DEPTH_METRES * 0.5 + 0.02);
        let mut strip = builder.geometry.solids[0].clone();
        strip.id = ResolvedItemId(999);
        strip.centre = Vec3::new(point.x, candidate.base + 0.5, point.y);
        strip.size = Vec3::splat(0.01);
        strip.yaw_radians = 0.0;
        strip.crossfall_radians = 0.0;
        strip.longfall_radians = 0.0;
        builder.geometry.solids.push(strip);
        let mut obstruction = builder.members[0].clone();
        obstruction.id = crate::TimberMemberId(999_999);
        obstruction.solid = ResolvedItemId(999);
        builder.members.push(obstruction);
        assert!(
            !candidate.fits(&builder, face),
            "exterior jamb strip must be clear"
        );
        builder.members.pop();
        builder.geometry.solids.pop();
        let saved_members = serde_json::to_vec(&builder.members).unwrap();
        let saved_counters = (
            builder.next_member,
            builder.next_node,
            builder.next_joint,
            builder.next_interface,
        );
        let (mut walls, mut apertures, mut bays) = (Vec::new(), Vec::new(), Vec::new());
        resolve(
            &BuildingProgram::fixture(BuildingArchetype::TownHouse, 42),
            &mut builder,
            &mut roofs,
            &mut walls,
            &mut apertures,
            &mut bays,
        );
        assert_eq!(apertures.len(), 2, "axis rotation {angle}");
        assert_eq!(
            serde_json::to_vec(&builder.members[..original.len()]).unwrap(),
            saved_members
        );
        assert_eq!(
            serde_json::to_vec(&builder.geometry.solids[..ids.len()]).unwrap(),
            original_solids
        );
        assert_eq!(
            (
                builder.next_member,
                builder.next_node,
                builder.next_joint,
                builder.next_interface
            ),
            saved_counters
        );
        for opening in &apertures {
            let bounds = builder
                .geometry
                .voids
                .iter()
                .find(|v| v.id == opening.void_id)
                .unwrap()
                .bounds;
            for member in &builder.members {
                let solid = builder
                    .geometry
                    .solids
                    .iter()
                    .find(|s| s.id == member.solid)
                    .unwrap();
                assert!(
                    !crate::solid_overlap::overlaps_bounds(solid, (bounds.min, bounds.max), 0.001),
                    "frame intrudes at rotation {angle}"
                );
            }
        }
        let face = &roofs[0].enclosure_faces[0];
        let blocked = ResolvedSolid {
            id: ResolvedItemId(999),
            owner: builder.owner,
            centre: Vec3::new(4.5, 9.0, 0.0),
            size: Vec3::splat(100.0),
            yaw_radians: 0.0,
            crossfall_radians: 0.0,
            longfall_radians: 0.0,
            role: SolidRole::FramePost,
            shape: crate::ResolvedSolidShape::Cuboid,
            supported_by: Vec::new(),
        };
        builder.geometry.solids.push(blocked);
        let mut blocker = builder.members[0].clone();
        blocker.solid = ResolvedItemId(999);
        blocker.id = crate::TimberMemberId(999_999);
        builder.members.push(blocker);
        assert!(Candidate::find(&builder, face).is_none());
    }
}

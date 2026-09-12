use super::*;
use std::collections::BTreeMap;

fn closed(mesh: &PartMesh) {
    mesh.normals().unwrap();
    let mut weld = BTreeMap::new();
    let ids = mesh
        .positions
        .iter()
        .map(|p| {
            let next = weld.len();
            *weld
                .entry(p.map(|v| (v * 1e6).round() as i64))
                .or_insert(next)
        })
        .collect::<Vec<_>>();
    let mut edges = BTreeMap::new();
    for face in mesh.indices.as_chunks::<3>().0 {
        for i in 0..3 {
            let a = ids[face[i] as usize];
            let b = ids[face[(i + 1) % 3] as usize];
            assert_ne!(a, b);
            *edges.entry((a, b)).or_insert(0) += 1;
        }
    }
    for (&(a, b), &count) in &edges {
        assert_eq!(count, 1);
        assert_eq!(edges.get(&(b, a)), Some(&1));
    }
}

#[test]
fn eccentric_support_preserves_closure_correspondence_and_closed_walls() {
    let mut prior = None;
    for radius in [0.045, 0.08, 0.15] {
        let plate = mesh::cuboid([radius, 0.2, radius * 0.7]);
        let section = section::ClosureSection::new(
            &[],
            &[],
            &plate.positions,
            plate.indices.as_chunks::<3>().0,
            0.0,
            0.04,
            0.006,
        )
        .unwrap();
        let (leather, metal) = mesh::closure(&section, 0.0, &StrapDesign::default()).unwrap();
        closed(&leather);
        closed(&metal);
        if let Some(indices) = &prior {
            assert_eq!(&leather.indices, indices);
        }
        prior = Some(leather.indices);
    }
}

#[test]
fn suspension_is_bilateral_and_requires_both_attachment_plates() {
    let recipes = catalog::load(None).unwrap();
    let catalog::FastenerRecipe::TassetSuspension(d) = &recipes["tassets"] else {
        panic!()
    };
    let mut fauld = mesh::cuboid([0.25, 0.10, 0.15]);
    for p in &mut fauld.positions {
        p[1] += 0.90;
    }
    let mut tassets = mesh::cuboid([0.25, 0.15, 0.16]);
    for p in &mut tassets.positions {
        p[1] += 0.67;
    }
    let result = d.generate(&tassets, &fauld).unwrap();
    result.normals().unwrap();
    assert_eq!(result.components.len(), 2);
    assert!(result.positions.iter().any(|p| p[0] < -0.15));
    assert!(result.positions.iter().any(|p| p[0] > 0.15));
    assert!(d.generate(&tassets, &PartMesh::new()).is_err());
}

#[test]
fn authored_recipes_validate_and_reject_unusable_strap_parameters() {
    for recipe in catalog::load(None).unwrap().values() {
        recipe.validate().unwrap();
    }
    for d in [
        StrapDesign {
            width: Millimeters(8),
            thickness: Millimeters(4),
            ..Default::default()
        },
        StrapDesign {
            count: 3,
            spacing: Permille(800),
            ..Default::default()
        },
        StrapDesign {
            start_angle: Milliradians(u16::MAX),
            ..Default::default()
        },
    ] {
        assert!(d.validate().is_err());
    }
}

#[test]
fn a_small_wearer_rejects_an_arc_without_room_for_the_buckle() {
    let plate = mesh::cuboid([0.045, 0.2, 0.03]);
    let section = section::ClosureSection::new(
        &[],
        &[],
        &plate.positions,
        plate.indices.as_chunks::<3>().0,
        0.0,
        0.04,
        0.006,
    )
    .unwrap();
    let d = StrapDesign {
        start_angle: Milliradians(1000),
        end_angle: Milliradians(1500),
        buckle_position: Permille(500),
        ..Default::default()
    };
    d.validate().unwrap();
    assert!(mesh::closure(&section, 0.0, &d).is_err());
}

fn tapered_support(half: bool) -> PartMesh {
    let mut mesh = PartMesh::new();
    let segments = 64_u32;
    for (y, radius) in [(0.0, 0.04), (0.1, 0.10)] {
        for i in 0..=segments {
            let angle = i as f32 / segments as f32
                * if half {
                    std::f32::consts::PI
                } else {
                    std::f32::consts::TAU
                };
            mesh.positions
                .push([radius * angle.sin(), y, radius * angle.cos()]);
        }
    }
    for i in 0..segments {
        let top = i + segments + 1;
        mesh.indices.extend([i, i + 1, top + 1, i, top + 1, top]);
    }
    mesh
}

#[test]
fn descending_shoulder_band_narrows_below_the_deltoid_and_encloses_the_arm() {
    let arm = tapered_support(false);
    let design = StrapDesign {
        underarm_drop: Millimeters(60),
        width: Millimeters(14),
        start_angle: Milliradians(3050),
        end_angle: Milliradians(6230),
        ..Default::default()
    };
    let mut section = section::ClosureSection::new(
        &arm.positions,
        arm.indices.as_chunks::<3>().0,
        &[],
        &[],
        0.07,
        0.08,
        0.006,
    )
    .unwrap();
    section
        .follow_underarm(
            section::SupportSurfaces {
                body: &arm.positions,
                body_faces: arm.indices.as_chunks::<3>().0,
                plate: &[],
                plate_faces: &[],
            },
            0.1,
            &design,
        )
        .unwrap();
    let top = section.radius(design.start_angle.radians()).unwrap();
    let bottom = section
        .radius((design.start_angle.radians() + design.end_angle.radians()) * 0.5)
        .unwrap();
    assert!(
        top - bottom > 0.02,
        "strap retains the broad deltoid radius"
    );
    assert!(bottom > 0.064, "strap crosses the supporting tapered arm");
}

#[test]
fn shoulder_straps_require_both_endpoints_to_land_on_their_own_plate() {
    let plate = tapered_support(true);
    let frame = PartFrame {
        origin: [0.0; 3],
        axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        half_extents: [1.0; 3],
    };
    let region = FitRegion::UpperArm(crate::armor_frames::Side::Left);
    let mut design = StrapDesign {
        underarm_drop: Millimeters(60),
        height: Permille(500),
        start_angle: Milliradians(1000),
        end_angle: Milliradians(2000),
        ..Default::default()
    };
    assert!(support::local_mesh(&plate, None, &frame, region, &design).is_ok());
    design.end_angle = Milliradians(4700);
    assert!(
        support::local_mesh(
            &plate,
            Some(&tapered_support(false)),
            &frame,
            region,
            &design
        )
        .is_err()
    );
}

#[test]
fn tension_path_endpoints_remain_supported_across_rotated_hulls() {
    let arm = tapered_support(false);
    for turn in 0..32 {
        let design = StrapDesign {
            underarm_drop: Millimeters(60),
            start_angle: Milliradians(200 + turn * 85),
            end_angle: Milliradians(3000 + turn * 85),
            ..Default::default()
        };
        let mut section = section::ClosureSection::new(
            &arm.positions,
            arm.indices.as_chunks::<3>().0,
            &[],
            &[],
            0.07,
            0.08,
            0.006,
        )
        .unwrap();
        section
            .follow_underarm(
                section::SupportSurfaces {
                    body: &arm.positions,
                    body_faces: arm.indices.as_chunks::<3>().0,
                    plate: &[],
                    plate_faces: &[],
                },
                0.1,
                &design,
            )
            .unwrap();
        for angle in [design.start_angle.radians(), design.end_angle.radians()] {
            assert!(section.radius(angle).unwrap() > 0.09);
        }
        mesh::closure(&section, 0.1, &design).unwrap();
    }
}

#[test]
fn seam_crossing_arcs_keep_endpoint_and_underarm_height_correspondence() {
    let design = StrapDesign {
        start_angle: Milliradians(2750),
        end_angle: Milliradians(6600),
        ..Default::default()
    };
    design.validate().unwrap();
    let start = design.start_angle.radians();
    let span = design.end_angle.radians() - start;
    for fraction in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let angle = start + span * fraction;
        for turn in [-1.0, 0.0, 1.0] {
            assert!(
                (design.arc_fraction(angle + turn * std::f32::consts::TAU) - fraction).abs() < 1e-6
            );
        }
    }
    assert_eq!(design.arc_fraction(start - 0.01), 0.0);
    assert_eq!(design.arc_fraction(design.end_angle.radians() + 0.01), 1.0);
    let before = design.arc_fraction(std::f32::consts::TAU - 0.0001);
    let after = design.arc_fraction(0.0001);
    assert!(
        after > before && after - before < 0.0001,
        "the measured polar seam must not jump to the opposite strap endpoint"
    );
    for (start, end) in [(6284, 7000), (2750, 9200), (2750, 3100), (2750, 2700)] {
        assert!(
            StrapDesign {
                start_angle: Milliradians(start),
                end_angle: Milliradians(end),
                ..Default::default()
            }
            .validate()
            .is_err()
        );
    }
}

#[test]
fn suspended_besagew_does_not_supply_band_support_or_main_plate_anchors() {
    let plate = tapered_support(true).with_component(ArmorComponentRole::Plate, None);
    let frame = PartFrame {
        origin: [0.0; 3],
        axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        half_extents: [1.0; 3],
    };
    let region = FitRegion::UpperArm(crate::armor_frames::Side::Left);
    let mut design = StrapDesign {
        underarm_drop: Millimeters(60),
        height: Permille(500),
        start_angle: Milliradians(1000),
        end_angle: Milliradians(2000),
        ..Default::default()
    };
    let mut distant = tapered_support(false).with_component(ArmorComponentRole::Besagew, None);
    for point in &mut distant.positions {
        point[2] += 0.2;
    }
    let mut assembly = plate.clone();
    assembly.append(distant);
    let obstacles = support::local_mesh(&assembly, None, &frame, region, &design).unwrap();
    assert_eq!(obstacles.indices, plate.indices);
    assert_eq!(
        obstacles.positions, assembly.positions,
        "support filtering must not move the independently suspended disc"
    );
    for height in [0.02, 0.05, 0.08] {
        let own = section::ClosureSection::new(
            &[],
            &[],
            &plate.positions,
            plate.indices.as_chunks::<3>().0,
            height,
            design.width.metres(),
            0.0,
        )
        .unwrap();
        let layered = section::ClosureSection::new(
            &[],
            &[],
            &obstacles.positions,
            obstacles.indices.as_chunks::<3>().0,
            height,
            design.width.metres(),
            0.0,
        )
        .unwrap();
        for angle in [1.0, 1.5, 2.0] {
            assert_eq!(own.radius(angle).unwrap(), layered.radius(angle).unwrap());
        }
    }
    design.end_angle = Milliradians(4700);
    let mut misleading = plate;
    misleading.append(tapered_support(false).with_component(ArmorComponentRole::Besagew, None));
    assert!(
        support::local_mesh(&misleading, None, &frame, region, &design).is_err(),
        "another component cannot supply the missing shoulder attachment"
    );
}

#[test]
fn seam_crossing_shoulder_route_builds_closed_leather_and_hardware() {
    let arm = tapered_support(false);
    let design = StrapDesign {
        width: Millimeters(14),
        underarm_drop: Millimeters(60),
        start_angle: Milliradians(2750),
        end_angle: Milliradians(6600),
        ..Default::default()
    };
    let mut section = section::ClosureSection::new(
        &arm.positions,
        arm.indices.as_chunks::<3>().0,
        &[],
        &[],
        0.07,
        0.08,
        0.006,
    )
    .unwrap();
    section
        .follow_underarm(
            section::SupportSurfaces {
                body: &arm.positions,
                body_faces: arm.indices.as_chunks::<3>().0,
                plate: &[],
                plate_faces: &[],
            },
            0.1,
            &design,
        )
        .unwrap();
    let (leather, hardware) = mesh::closure(&section, 0.1, &design).unwrap();
    closed(&leather);
    closed(&hardware);
}

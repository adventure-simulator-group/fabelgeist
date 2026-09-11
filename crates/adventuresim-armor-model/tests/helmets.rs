use std::collections::BTreeMap;

use adventuresim_armor_model::{
    BarbuteDesign, BurgonetDesign, CloseHelmetDesign, CoifDesign, CoifDrapeProfile, HelmetDesign,
    HelmetKind, KettleHatDesign, Millimeters, MorionDesign, Permille, SalletDesign,
    VisoredSalletDesign, generate_coif_with_drape, generate_helmet,
    parametric::{PartFrame, PartMesh},
};

const KINDS: [HelmetKind; 9] = [
    HelmetKind::Morion,
    HelmetKind::KettleHat,
    HelmetKind::Barbute,
    HelmetKind::Burgonet,
    HelmetKind::Sallet,
    HelmetKind::VisoredSallet,
    HelmetKind::CloseHelmet,
    HelmetKind::ArmingCap,
    HelmetKind::MailCoif,
];

fn frame(scale: f32) -> PartFrame {
    PartFrame {
        origin: [0.0, 1.65, 0.0],
        axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        half_extents: [0.085 * scale, 0.115 * scale, 0.105 * scale],
    }
}

#[test]
fn temple_fan_crowns_preserve_closed_walls_and_lower_plate_connections() {
    use adventuresim_armor_model::{FluteCount, HelmetCrown, PlateFluting};
    for count in [2, 7, 24] {
        let crown = HelmetCrown {
            fluting: Some(PlateFluting {
                count: FluteCount(count),
                ..Default::default()
            }),
            ..Default::default()
        };
        for design in [
            HelmetDesign::Morion(MorionDesign {
                crown,
                ..Default::default()
            }),
            HelmetDesign::KettleHat(KettleHatDesign {
                crown,
                ..Default::default()
            }),
            HelmetDesign::Barbute(BarbuteDesign {
                crown,
                ..Default::default()
            }),
            HelmetDesign::Burgonet(BurgonetDesign {
                crown,
                ..Default::default()
            }),
            HelmetDesign::Sallet(SalletDesign {
                crown,
                ..Default::default()
            }),
            HelmetDesign::CloseHelmet(CloseHelmetDesign {
                crown,
                visor_fluting: crown.fluting,
                ..Default::default()
            }),
        ] {
            let small = generate_helmet(&design, &frame(0.8)).unwrap();
            let large = generate_helmet(&design, &frame(1.2)).unwrap();
            assert_eq!(
                small.indices, large.indices,
                "body dimensions changed fluted topology"
            );
            assert_closed_solid(&small);
            assert_closed_solid(&large);
        }
    }
}

#[test]
fn close_helmet_keeps_independent_plate_partitions() {
    let mesh = generate_helmet(
        &HelmetDesign::CloseHelmet(CloseHelmetDesign {
            comb_height: Millimeters(0),
            ..Default::default()
        }),
        &frame(1.0),
    )
    .unwrap();
    assert_eq!(mesh.components.len(), 3);
    let mut vertex_end = 0;
    let mut index_end = 0;
    for part in &mesh.components {
        assert_eq!(part.vertices.start, vertex_end);
        assert_eq!(part.indices.start, index_end);
        assert!(
            mesh.indices[part.indices.clone()]
                .iter()
                .all(|i| part.vertices.contains(&(*i as usize)))
        );
        vertex_end = part.vertices.end;
        index_end = part.indices.end;
    }
    assert_eq!(vertex_end, mesh.positions.len());
    assert_eq!(index_end, mesh.indices.len());
}

#[test]
fn anatomical_coif_drape_changes_flaps_without_changing_head_or_connectivity() {
    let design = CoifDesign::default();
    let head = frame(1.0);
    let a = CoifDrapeProfile::from_head(&design, &head);
    let mut b = a;
    for section in &mut b.back.sections {
        section.edge_depth -= 0.015;
    }
    let mesh_a = generate_coif_with_drape(&design, &head, &a).unwrap();
    let mesh_b = generate_coif_with_drape(&design, &head, &b).unwrap();
    assert_eq!(mesh_a.indices, mesh_b.indices);
    assert_closed_solid(&mesh_b);
    assert_ne!(mesh_a.positions, mesh_b.positions);
    for (a, b) in mesh_a
        .positions
        .iter()
        .zip(&mesh_b.positions)
        .filter(|(p, _)| p[1] > head.origin[1])
    {
        assert_eq!(a, b);
    }
    b.back.sections[1].height = b.back.sections[0].height;
    assert!(generate_coif_with_drape(&design, &head, &b).is_err());
}

#[test]
fn hanging_mail_bridges_a_neck_hollow_instead_of_reproducing_it() {
    let design = CoifDesign::default();
    let head = frame(1.0);
    let mut profile = CoifDrapeProfile::from_head(&design, &head);
    profile.neck.front_depth = 0.08;
    for (i, section) in profile.front.sections.iter_mut().enumerate() {
        let z = if i == 0 || i == 4 { 0.08 } else { 0.015 };
        section.center_depth = z;
        section.edge_depth = z;
    }
    let mesh = generate_coif_with_drape(&design, &head, &profile).unwrap();
    assert_closed_solid(&mesh);
    let hanging_center = mesh
        .positions
        .iter()
        .filter(|p| p[0].abs() < 0.001 && p[1] < 1.49 && p[2] > 0.0)
        .collect::<Vec<_>>();
    assert!(!hanging_center.is_empty());
    assert!(
        hanging_center.iter().all(|p| p[2] > 0.07),
        "gravity-supported front cloth must bridge the hollow"
    );
}

fn assert_closed_solid(mesh: &PartMesh) {
    mesh.normals()
        .expect("finite, nondegenerate triangles and normals");
    let mut edges = BTreeMap::<(u32, u32), Vec<(u32, u32)>>::new();
    let mut volume = 0.0_f64;
    let mut welded = BTreeMap::new();
    let mapping: Vec<_> = mesh
        .positions
        .iter()
        .map(|p| {
            let next = welded.len() as u32;
            *welded
                .entry(p.map(|v| (f64::from(v) * 1e7).round() as i64))
                .or_insert(next)
        })
        .collect();
    for triangle in mesh.indices.as_chunks::<3>().0 {
        let physical = triangle.map(|i| mapping[i as usize]);
        for (a, b) in [
            (physical[0], physical[1]),
            (physical[1], physical[2]),
            (physical[2], physical[0]),
        ] {
            edges.entry((a.min(b), a.max(b))).or_default().push((a, b));
        }
        let [a, b, c] = [triangle[0], triangle[1], triangle[2]]
            .map(|i| mesh.positions[i as usize].map(f64::from));
        volume += (a[0] * (b[1] * c[2] - b[2] * c[1])
            + a[1] * (b[2] * c[0] - b[0] * c[2])
            + a[2] * (b[0] * c[1] - b[1] * c[0]))
            / 6.0;
    }
    for incident in edges.values() {
        assert_eq!(incident.len(), 2, "every solid edge has two faces");
        assert_eq!(
            incident[0],
            (incident[1].1, incident[1].0),
            "consistent winding at each edge"
        );
    }
    assert!(
        volume > 1e-7,
        "outward winding gives positive volume, got {volume}"
    );
}

#[test]
fn every_catalog_helmet_is_a_closed_solid_on_representative_heads() {
    for scale in [0.75, 1.0, 1.3] {
        for kind in KINDS {
            let design = HelmetDesign::catalog(kind);
            let mesh =
                generate_helmet(&design, &frame(scale)).unwrap_or_else(|e| panic!("{kind:?}: {e}"));
            assert_closed_solid(&mesh);
            let again = generate_helmet(&design, &frame(scale)).unwrap();
            assert_eq!(mesh.positions, again.positions);
            assert_eq!(mesh.indices, again.indices);
        }
    }
}

#[test]
fn extreme_style_controls_preserve_solid_topology() {
    let designs = [
        HelmetDesign::Morion(MorionDesign {
            brim_width: Millimeters(65),
            brim_sweep: Millimeters(60),
            comb_height: Millimeters(85),
            ..Default::default()
        }),
        HelmetDesign::KettleHat(KettleHatDesign {
            brim_width: Millimeters(85),
            brim_drop: Millimeters(45),
            ..Default::default()
        }),
        HelmetDesign::Barbute(BarbuteDesign {
            eye_opening: adventuresim_armor_model::Milliradians(850),
            mouth_opening: adventuresim_armor_model::Milliradians(120),
            cheek_depth: Permille(1050),
            ..Default::default()
        }),
        HelmetDesign::Barbute(BarbuteDesign {
            eye_opening: adventuresim_armor_model::Milliradians(500),
            mouth_opening: adventuresim_armor_model::Milliradians(400),
            cheek_depth: Permille(750),
            ..Default::default()
        }),
        HelmetDesign::Burgonet(BurgonetDesign {
            comb_height: Millimeters(0),
            cheek_depth: Permille(1000),
            neck_flare: Millimeters(40),
            ..Default::default()
        }),
        HelmetDesign::Sallet(SalletDesign {
            tail_length: Millimeters(110),
            tail_drop: Millimeters(45),
            brow_projection: Millimeters(20),
            ..Default::default()
        }),
        HelmetDesign::VisoredSallet(VisoredSalletDesign {
            visor_projection: Millimeters(50),
            sight_gap: Millimeters(5),
            ..Default::default()
        }),
        HelmetDesign::CloseHelmet(CloseHelmetDesign {
            comb_height: Millimeters(40),
            throat_flare: Millimeters(15),
            visor_projection: Millimeters(50),
            sight_gap: Millimeters(5),
            back_edge_lift: Millimeters(45),
            ..Default::default()
        }),
        HelmetDesign::MailCoif(CoifDesign {
            front_flap_length: Millimeters(150),
            back_flap_length: Millimeters(170),
            flap_width: Permille(1200),
            ..Default::default()
        }),
    ];
    for design in designs {
        assert_closed_solid(&generate_helmet(&design, &frame(0.75)).unwrap());
        assert_closed_solid(&generate_helmet(&design, &frame(1.3)).unwrap());
    }
}

#[test]
fn coif_flap_controls_preserve_the_fitted_skull() {
    let narrow = HelmetDesign::MailCoif(CoifDesign::default());
    let wide = HelmetDesign::MailCoif(CoifDesign {
        back_flap_length: Millimeters(170),
        flap_width: Permille(1200),
        ..Default::default()
    });
    let a = generate_helmet(&narrow, &frame(1.0)).unwrap();
    let b = generate_helmet(&wide, &frame(1.0)).unwrap();
    assert_eq!(a.indices, b.indices);
    for (a, b) in a
        .positions
        .iter()
        .zip(&b.positions)
        .filter(|(p, _)| p[1] > 1.7)
    {
        assert_eq!(a, b);
    }
    assert_ne!(a.positions, b.positions);
}

#[test]
fn neck_coverage_extends_the_enclosure_without_moving_the_head() {
    let head = frame(1.0);
    let generate = |coverage| {
        generate_helmet(
            &HelmetDesign::MailCoif(CoifDesign {
                neck_coverage: Permille(coverage),
                ..Default::default()
            }),
            &head,
        )
        .unwrap()
    };
    let short = generate(800);
    let long = generate(1100);
    assert_closed_solid(&short);
    assert_closed_solid(&long);
    assert_eq!(short.indices, long.indices);
    let bottom = |mesh: &PartMesh| {
        mesh.positions
            .iter()
            .map(|p| p[1])
            .fold(f32::INFINITY, f32::min)
    };
    assert!(bottom(&long) < bottom(&short));
    for (a, b) in short
        .positions
        .iter()
        .zip(&long.positions)
        .filter(|(p, _)| p[1] > head.origin[1])
    {
        assert_eq!(a, b);
    }
}

#[test]
fn reflected_placement_preserves_outward_winding() {
    let mut reflected = frame(1.0);
    reflected.axes[0][0] = -1.0;
    for kind in KINDS {
        assert_closed_solid(&generate_helmet(&HelmetDesign::catalog(kind), &reflected).unwrap());
    }
}

#[test]
fn bad_style_and_invalid_anatomical_frames_are_rejected() {
    let invalid = HelmetDesign::Barbute(BarbuteDesign {
        mouth_opening: adventuresim_armor_model::Milliradians(0),
        ..Default::default()
    });
    assert!(generate_helmet(&invalid, &frame(1.0)).is_err());
    let mut invalid_frame = frame(1.0);
    invalid_frame.half_extents[1] = f32::NAN;
    assert!(generate_helmet(&HelmetDesign::catalog(HelmetKind::Morion), &invalid_frame).is_err());
}

#[test]
fn mouth_control_changes_face_opening_without_changing_skull() {
    let closed = HelmetDesign::Barbute(BarbuteDesign::default());
    let open = HelmetDesign::Barbute(BarbuteDesign {
        mouth_opening: adventuresim_armor_model::Milliradians(400),
        ..Default::default()
    });
    let a = generate_helmet(&closed, &frame(1.0)).unwrap();
    let b = generate_helmet(&open, &frame(1.0)).unwrap();
    assert_eq!(a.indices, b.indices);
    for (a, b) in a
        .positions
        .iter()
        .zip(&b.positions)
        .filter(|(p, _)| p[1] > 1.7)
    {
        assert_eq!(a, b);
    }
    assert_ne!(a.positions, b.positions);
}

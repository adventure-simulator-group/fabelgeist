mod common;

use common::{assert_closed_solid, bounds, frame, gpu, reflected};
use fabelgeist_armor::{
    BarbuteDesign, BuiltPart, BurgonetDesign, CoifDesign, FluteCount, GenerateError, HelmetCrown,
    HelmetDesign, HelmetKind, KettleHatDesign, Millimeters, Milliradians, MorionDesign, PartFrame,
    Permille, PlateFluting, SalletDesign, VisoredSalletDesign,
    gpu::{
        COIF_DRAPE_SECTIONS, CoifDrapeProfile, CoifDrapeSection, CoifFlapDrape, CoifNeckDrape,
        generate_helmet_on, record_coif,
    },
};

/// Every helmet family built from the head frame alone.
const FRAMED_KINDS: [HelmetKind; 7] = [
    HelmetKind::Morion,
    HelmetKind::KettleHat,
    HelmetKind::Barbute,
    HelmetKind::Burgonet,
    HelmetKind::Sallet,
    HelmetKind::VisoredSallet,
    HelmetKind::ArmingCap,
];

fn head(scale: f32) -> PartFrame {
    frame(
        [0.0, 1.65, 0.0],
        [0.085 * scale, 0.115 * scale, 0.105 * scale],
    )
}

fn helmet(design: &HelmetDesign, head: &PartFrame) -> BuiltPart {
    generate_helmet_on(gpu(), design, head).unwrap_or_else(|e| panic!("{design:?}: {e}"))
}

#[test]
fn every_catalog_helmet_is_a_deterministic_closed_solid_on_representative_heads() {
    for scale in [0.75, 1.0, 1.3] {
        for kind in FRAMED_KINDS {
            let design = HelmetDesign::catalog(kind);
            let mesh = helmet(&design, &head(scale));
            assert_closed_solid(&mesh, &format!("{kind:?} at {scale}"));
            let again = helmet(&design, &head(scale));
            assert_eq!(mesh.positions, again.positions);
            assert_eq!(mesh.indices, again.indices);
        }
    }
}

#[test]
fn helmets_fitted_by_measured_sections_are_not_built_from_a_frame_alone() {
    for kind in [HelmetKind::CloseHelmet, HelmetKind::MailCoif] {
        assert!(generate_helmet_on(gpu(), &HelmetDesign::catalog(kind), &head(1.0)).is_err());
    }
}

#[test]
fn helmets_grow_with_the_head_and_follow_its_placement() {
    for kind in FRAMED_KINDS {
        let design = HelmetDesign::catalog(kind);
        let small = helmet(&design, &head(0.75));
        let large = helmet(&design, &head(1.3));
        assert_eq!(small.indices, large.indices, "{kind:?}");
        let [small_low, small_high] = bounds(&small.positions);
        let [large_low, large_high] = bounds(&large.positions);
        for axis in 0..3 {
            assert!(
                large_high[axis] - large_low[axis] > (small_high[axis] - small_low[axis]) * 1.3,
                "{kind:?} did not grow along axis {axis}"
            );
        }

        let offset = [0.2, -0.1, 0.3];
        let moved = PartFrame {
            origin: std::array::from_fn(|i| head(1.0).origin[i] + offset[i]),
            ..head(1.0)
        };
        let here = helmet(&design, &head(1.0));
        let there = helmet(&design, &moved);
        for (p, q) in here.positions.iter().zip(&there.positions) {
            for axis in 0..3 {
                assert!((q[axis] - p[axis] - offset[axis]).abs() < 1e-5, "{kind:?}");
            }
        }
    }
}

#[test]
fn temple_fan_crowns_keep_closed_walls_and_body_independent_topology() {
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
        ] {
            let small = helmet(&design, &head(0.8));
            let large = helmet(&design, &head(1.2));
            assert_eq!(
                small.indices, large.indices,
                "body dimensions changed fluted topology"
            );
            assert_closed_solid(&small, &format!("{design:?}, small"));
            assert_closed_solid(&large, &format!("{design:?}, large"));
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
            eye_opening: Milliradians(850),
            mouth_opening: Milliradians(120),
            cheek_depth: Permille(1050),
            ..Default::default()
        }),
        HelmetDesign::Barbute(BarbuteDesign {
            eye_opening: Milliradians(500),
            mouth_opening: Milliradians(400),
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
    ];
    for design in designs {
        for scale in [0.75, 1.3] {
            assert_closed_solid(
                &helmet(&design, &head(scale)),
                &format!("{design:?} at {scale}"),
            );
        }
    }
}

#[test]
fn reflected_placement_preserves_outward_winding() {
    for kind in FRAMED_KINDS {
        let design = HelmetDesign::catalog(kind);
        assert_closed_solid(
            &helmet(&design, &reflected(head(1.0))),
            &format!("{kind:?}"),
        );
    }
}

#[test]
fn bad_style_and_invalid_anatomical_frames_are_rejected() {
    let invalid = HelmetDesign::Barbute(BarbuteDesign {
        mouth_opening: Milliradians(0),
        ..Default::default()
    });
    assert!(invalid.validate().is_err());
    assert!(generate_helmet_on(gpu(), &invalid, &head(1.0)).is_err());
    let morion = HelmetDesign::catalog(HelmetKind::Morion);
    let mut not_finite = head(1.0);
    not_finite.half_extents[1] = f32::NAN;
    let mut sheared = head(1.0);
    sheared.axes[2] = [0.0, 0.6, 0.8];
    let mut flat = head(1.0);
    flat.half_extents[0] = -0.08;
    for bad in [not_finite, sheared, flat] {
        assert!(generate_helmet_on(gpu(), &morion, &bad).is_err(), "{bad:?}");
    }
}

#[test]
fn mouth_control_changes_face_opening_without_changing_skull() {
    let closed = HelmetDesign::Barbute(BarbuteDesign::default());
    let open = HelmetDesign::Barbute(BarbuteDesign {
        mouth_opening: Milliradians(400),
        ..Default::default()
    });
    let a = helmet(&closed, &head(1.0));
    let b = helmet(&open, &head(1.0));
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

/// A head-proportioned drape, as a preview without a wearer hangs one: the
/// neck boundary below the jaw, each flap hanging straight from it.
fn drape(design: &CoifDesign, head: &PartFrame) -> CoifDrapeProfile {
    let gap = design.fit.clearance.metres() + design.fit.wall_thickness.metres();
    let h = head.half_extents[1];
    let depth = head.half_extents[2] + gap;
    let front_height = -h * (1.0 + 0.65 * design.neck_coverage.unit());
    let neck = CoifNeckDrape {
        front_height,
        side_height: front_height + h * 0.34,
        back_height: front_height + h * 0.22,
        half_width: (head.half_extents[0] + gap) * 1.5,
        center_depth: -depth * 0.36,
        front_depth: depth * 0.38,
        back_depth: -depth * 1.32,
    };
    // Where each flap leaves the neck: a twelfth of a turn either side.
    let flap_corner = (std::f32::consts::TAU / 12.0).cos().powi(2);
    let flap = |front: bool| {
        let (bottom, length, depth) = if front {
            (
                neck.front_height,
                design.front_flap_length.metres(),
                neck.front_depth,
            )
        } else {
            (
                neck.back_height,
                design.back_flap_length.metres(),
                neck.back_depth,
            )
        };
        let top = neck.side_height + (bottom - neck.side_height) * flap_corner;
        let outward = if front { 1.0 } else { -1.0 };
        CoifFlapDrape {
            sections: std::array::from_fn(|i| {
                let t = i as f32 / (COIF_DRAPE_SECTIONS - 1) as f32;
                let z = depth + outward * length * (1.0 - t) * 0.25;
                CoifDrapeSection {
                    height: bottom - length + (top - bottom + length) * t,
                    center_depth: z,
                    edge_depth: z,
                }
            }),
        }
    };
    CoifDrapeProfile {
        neck,
        front: flap(true),
        back: flap(false),
    }
}

fn coif(
    design: &CoifDesign,
    head: &PartFrame,
    drape: &CoifDrapeProfile,
) -> Result<BuiltPart, GenerateError> {
    let fit = gpu().upload(fabelgeist_gpu::prelude::BufferUpload::from_elements(
        &drape.fit_words(head.half_extents),
    ))?;
    gpu().build_in(&[*head], |batch, frames| {
        record_coif(gpu(), batch, design, &fit, frames[0])
    })
}

fn preview_coif(design: &CoifDesign, head: &PartFrame) -> BuiltPart {
    coif(design, head, &drape(design, head)).unwrap()
}

#[test]
fn coifs_are_closed_solids_on_representative_heads() {
    let designs = [
        CoifDesign::default(),
        CoifDesign {
            front_flap_length: Millimeters(150),
            back_flap_length: Millimeters(170),
            flap_width: Permille(1200),
            ..Default::default()
        },
    ];
    for design in designs {
        for placed in [head(0.75), head(1.3)] {
            let mesh = preview_coif(&design, &placed);
            assert_closed_solid(&mesh, &format!("{design:?} on {placed:?}"));
            assert!(mesh.components.is_empty());
        }
    }
}

#[test]
fn a_coif_is_not_placed_inside_out_by_a_reflected_head() {
    // A coif is thickened in its own frame and then placed; placement does
    // not rewind triangles, so a reflecting head frame is refused.
    let design = CoifDesign::default();
    let mirrored = reflected(head(1.0));
    assert!(coif(&design, &mirrored, &drape(&design, &mirrored)).is_err());
}

#[test]
fn anatomical_coif_drape_changes_flaps_without_changing_head_or_connectivity() {
    let design = CoifDesign::default();
    let head = head(1.0);
    let a = drape(&design, &head);
    let mut b = a;
    for section in &mut b.back.sections {
        section.edge_depth -= 0.015;
    }
    let mesh_a = coif(&design, &head, &a).unwrap();
    let mesh_b = coif(&design, &head, &b).unwrap();
    assert_eq!(mesh_a.indices, mesh_b.indices);
    assert_closed_solid(&mesh_b, "coif on a deeper back");
    assert_ne!(mesh_a.positions, mesh_b.positions);
    for (a, b) in mesh_a
        .positions
        .iter()
        .zip(&mesh_b.positions)
        .filter(|(p, _)| p[1] > head.origin[1])
    {
        assert_eq!(a, b);
    }
}

#[test]
fn hanging_mail_bridges_a_neck_hollow_instead_of_reproducing_it() {
    let design = CoifDesign::default();
    let head = head(1.0);
    let mut profile = drape(&design, &head);
    profile.neck.front_depth = 0.08;
    for (i, section) in profile.front.sections.iter_mut().enumerate() {
        let z = if i == 0 || i == 4 { 0.08 } else { 0.015 };
        section.center_depth = z;
        section.edge_depth = z;
    }
    let mesh = coif(&design, &head, &profile).unwrap();
    assert_closed_solid(&mesh, "coif over a hollow");
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

#[test]
fn coif_flap_controls_preserve_the_fitted_skull() {
    let head = head(1.0);
    let narrow = CoifDesign::default();
    let wide = CoifDesign {
        back_flap_length: Millimeters(170),
        flap_width: Permille(1200),
        ..Default::default()
    };
    // One drape for both, so only the flap controls differ.
    let profile = drape(&narrow, &head);
    let a = coif(&narrow, &head, &profile).unwrap();
    let b = coif(&wide, &head, &profile).unwrap();
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
fn a_lower_neck_boundary_extends_the_enclosure_without_moving_the_head() {
    let design = CoifDesign::default();
    let head = head(1.0);
    let short = drape(
        &CoifDesign {
            neck_coverage: Permille(800),
            ..design
        },
        &head,
    );
    let long = drape(
        &CoifDesign {
            neck_coverage: Permille(1100),
            ..design
        },
        &head,
    );
    let short = coif(&design, &head, &short).unwrap();
    let long = coif(&design, &head, &long).unwrap();
    assert_closed_solid(&short, "short neck");
    assert_closed_solid(&long, "long neck");
    assert_eq!(short.indices, long.indices);
    assert!(bounds(&long.positions)[0][1] < bounds(&short.positions)[0][1]);
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
fn invalid_coif_designs_and_head_frames_are_rejected() {
    let head = head(1.0);
    let design = CoifDesign::default();
    let profile = drape(&design, &head);
    let invalid = CoifDesign {
        flap_width: Permille(0),
        ..design
    };
    assert!(HelmetDesign::MailCoif(invalid).validate().is_err());
    assert!(coif(&invalid, &head, &profile).is_err());
    let mut skewed = head;
    skewed.axes[0] = skewed.axes[2];
    assert!(coif(&design, &skewed, &profile).is_err());
}

#[test]
fn a_burgonet_peak_rise_lifts_the_peak_front_without_changing_topology() {
    let head = head(1.0);
    let build = |rise| {
        helmet(
            &HelmetDesign::Burgonet(BurgonetDesign {
                peak_rise: Millimeters(rise),
                ..BurgonetDesign::default()
            }),
            &head,
        )
    };
    let flat = build(0);
    let raised = build(20);
    assert_closed_solid(&raised, "raised peak");
    assert_eq!(flat.indices, raised.indices);
    // The peak's front tip is the frontmost point; it rises by the full 20 mm.
    let front = |part: &BuiltPart| {
        *part
            .positions
            .iter()
            .max_by(|a, b| a[2].total_cmp(&b[2]))
            .unwrap()
    };
    let lift = front(&raised)[1] - front(&flat)[1];
    assert!((lift - 0.020).abs() < 0.003, "lift {lift}");
}

#[test]
fn buffes_are_closed_solids_apart_from_the_skull_in_every_construction() {
    use fabelgeist_armor::{ArmorComponentRole, BuffeCourses, BuffeDesign, VisorBreaths};
    let breaths = VisorBreaths::buffe();
    for (name, buffe) in [
        ("plain", BuffeDesign::default()),
        (
            "pierced",
            BuffeDesign {
                breaths: Some(breaths),
                ..BuffeDesign::default()
            },
        ),
        (
            "coursed",
            BuffeDesign {
                courses: Some(BuffeCourses::default()),
                ..BuffeDesign::default()
            },
        ),
        (
            "coursed and pierced",
            BuffeDesign {
                courses: Some(BuffeCourses::default()),
                breaths: Some(breaths),
                chin_width: Permille(500),
                ridge_sharpness: Permille(1000),
                ..BuffeDesign::default()
            },
        ),
    ] {
        let design = HelmetDesign::Burgonet(BurgonetDesign {
            buffe: Some(buffe),
            ..BurgonetDesign::default()
        });
        design.validate().unwrap_or_else(|e| panic!("{name}: {e}"));
        for scale in [0.8, 1.0, 1.25] {
            let context = format!("{name} buffe at {scale}");
            let part = helmet(&design, &head(scale));
            assert_closed_solid(&part, &context);
            let roles = part
                .components
                .iter()
                .map(|component| component.role)
                .collect::<Vec<_>>();
            assert_eq!(
                roles,
                [ArmorComponentRole::Skull, ArmorComponentRole::Buffe],
                "{context}"
            );
        }
    }
}

#[test]
fn a_buffe_covers_the_lower_face_below_the_peak() {
    use fabelgeist_armor::{ArmorComponentRole, BuffeDesign};
    let head = head(1.0);
    let part = helmet(
        &HelmetDesign::Burgonet(BurgonetDesign {
            buffe: Some(BuffeDesign::default()),
            ..BurgonetDesign::default()
        }),
        &head,
    );
    let buffe = part
        .components
        .iter()
        .find(|component| component.role == ArmorComponentRole::Buffe)
        .unwrap();
    let [low, high] = bounds(&part.positions[buffe.vertices.clone()]);
    // In front of the face, from below the chin to under the brow.
    assert!(high[2] > head.origin[2] + head.half_extents[2], "{high:?}");
    assert!(low[1] < head.origin[1] - head.half_extents[1], "{low:?}");
    assert!(
        high[1] < head.origin[1] + 0.2 * head.half_extents[1],
        "{high:?}"
    );
}

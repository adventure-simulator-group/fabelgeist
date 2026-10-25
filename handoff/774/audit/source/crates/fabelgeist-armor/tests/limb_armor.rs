mod common;

use common::{assert_closed_solid, bounds, frame, gpu, largest_difference, reflected, scaled};
use fabelgeist_armor::{
    BootDesign, BuiltPart, CuisseDesign, FootArmorDesign, GauntletDesign, GenerateError,
    GreaveDesign, JointCupDesign, LimbArmorDesign, Millimeters, PartFrame, Permille,
    RerebraceDesign, SpaulderDesign, gpu::generate_limb_armor_on, record_extremity_armor,
};

/// A thumb frame beside the hand, its long axis across the hand's.
fn thumb(hand: &PartFrame) -> PartFrame {
    PartFrame {
        origin: hand.point([
            0.9 * hand.half_extents[0],
            -0.2 * hand.half_extents[1],
            0.3 * hand.half_extents[2],
        ]),
        axes: [[0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, -1.0]],
        half_extents: hand.half_extents.map(|v| v * 0.3),
    }
}

/// Build any limb design: a mitten gets `thumb(hand)` unless `frames`
/// supplies a thumb frame of its own.
fn build(design: &LimbArmorDesign, frames: &[PartFrame]) -> Result<BuiltPart, GenerateError> {
    match design {
        LimbArmorDesign::MittenGauntlet(_)
        | LimbArmorDesign::Sabaton(_)
        | LimbArmorDesign::LeatherBoot(_) => {
            let mut frames = frames.to_vec();
            if matches!(design, LimbArmorDesign::MittenGauntlet(_)) && frames.len() == 1 {
                frames.push(thumb(&frames[0]));
            }
            gpu().build_in(&frames, |batch, buffers| {
                record_extremity_armor(gpu(), batch, design, buffers)
            })
        }
        _ => generate_limb_armor_on(gpu(), design, &frames[0]),
    }
}

fn families() -> Vec<(LimbArmorDesign, PartFrame)> {
    let at = |half_extents| frame([0.1, 0.9, -0.05], half_extents);
    vec![
        (
            LimbArmorDesign::Greave(GreaveDesign::default()),
            at([0.065, 0.18, 0.06]),
        ),
        (
            LimbArmorDesign::Cuisse(CuisseDesign::default()),
            at([0.085, 0.18, 0.08]),
        ),
        (
            LimbArmorDesign::Rerebrace(RerebraceDesign::default()),
            at([0.045, 0.13, 0.05]),
        ),
        (
            LimbArmorDesign::Poleyn(JointCupDesign::poleyn()),
            at([0.06, 0.055, 0.055]),
        ),
        (
            LimbArmorDesign::Couter(JointCupDesign::couter()),
            at([0.045, 0.045, 0.04]),
        ),
        (
            LimbArmorDesign::Spaulder(SpaulderDesign::default()),
            at([0.065, 0.09, 0.07]),
        ),
        (
            LimbArmorDesign::MittenGauntlet(GauntletDesign::default()),
            at([0.042, 0.09, 0.018]),
        ),
        (
            LimbArmorDesign::Sabaton(FootArmorDesign::default()),
            at([0.045, 0.04, 0.13]),
        ),
        (
            LimbArmorDesign::LeatherBoot(BootDesign::default()),
            at([0.045, 0.04, 0.13]),
        ),
    ]
}

#[test]
fn every_family_is_a_closed_outward_solid_on_small_large_and_reflected_limbs() {
    for (design, base) in families() {
        for scale in [0.72, 1.0, 1.4] {
            let fit = scaled(base, scale);
            let context = format!("{design:?} at {scale}");
            let mesh = build(&design, &[fit]).unwrap_or_else(|e| panic!("{context}: {e}"));
            assert_closed_solid(&mesh, &context);
            let mirrored = build(&design, &[reflected(fit)]).unwrap();
            assert_closed_solid(&mirrored, &format!("{context}, reflected"));
            assert_eq!(mirrored.positions.len(), mesh.positions.len());
        }
    }
}

#[test]
fn generation_is_deterministic_and_topology_depends_only_on_the_design() {
    for (design, base) in families() {
        let reference = build(&design, &[base]).unwrap();
        let again = build(&design, &[base]).unwrap();
        assert_eq!(reference.positions, again.positions, "{design:?}");
        assert_eq!(reference.indices, again.indices, "{design:?}");
        for scale in [0.72, 1.4] {
            let other = build(&design, &[scaled(base, scale)]).unwrap();
            assert_eq!(other.indices, reference.indices, "{design:?} at {scale}");
            assert_ne!(
                other.positions, reference.positions,
                "{design:?} at {scale}"
            );
        }
    }
}

#[test]
fn reflection_mirrors_the_plate_across_the_frame() {
    // Mirrored in the world plane through the hand or foot, as a left limb
    // mirrors a right one.
    let mirror = |frame: PartFrame, plane: f32| PartFrame {
        origin: [
            2.0 * plane - frame.origin[0],
            frame.origin[1],
            frame.origin[2],
        ],
        axes: frame.axes.map(|a| [-a[0], a[1], a[2]]),
        ..frame
    };
    for (design, base) in families() {
        let frames = [base, thumb(&base)];
        let frames = &frames[..if matches!(design, LimbArmorDesign::MittenGauntlet(_)) {
            2
        } else {
            1
        }];
        let mesh = build(&design, frames).unwrap();
        let reflected_frames = frames
            .iter()
            .map(|f| mirror(*f, base.origin[0]))
            .collect::<Vec<_>>();
        let mirrored = build(&design, &reflected_frames).unwrap();
        assert_closed_solid(&mirrored, &format!("{design:?}, mirrored"));
        let expected = mesh
            .positions
            .iter()
            .map(|p| [2.0 * base.origin[0] - p[0], p[1], p[2]])
            .collect::<Vec<_>>();
        let mirrored_bounds = bounds(&mirrored.positions);
        let expected_bounds = bounds(&expected);
        for side in 0..2 {
            for axis in 0..3 {
                assert!(
                    (mirrored_bounds[side][axis] - expected_bounds[side][axis]).abs() < 1e-5,
                    "{design:?}: reflected bounds {mirrored_bounds:?} vs {expected_bounds:?}"
                );
            }
        }
    }
}

#[test]
fn sabaton_toe_controls_widen_and_lengthen_the_toe_box() {
    let fit = frame([0.0; 3], [0.045, 0.04, 0.13]);
    let rounded = build(
        &LimbArmorDesign::Sabaton(FootArmorDesign::default()),
        &[fit],
    )
    .unwrap();
    let broad = build(
        &LimbArmorDesign::Sabaton(FootArmorDesign {
            toe_width: Permille(1400),
            toe_extension: Millimeters(40),
            lame_count: 8,
            ..Default::default()
        }),
        &[fit],
    )
    .unwrap();
    let [_, rounded_high] = bounds(&rounded.positions);
    let [_, broad_high] = bounds(&broad.positions);
    assert!(broad_high[0] > rounded_high[0] + 0.01, "toe box not wider");
    assert!(broad_high[2] > rounded_high[2] + 0.025, "toe not longer");
    assert!(
        broad.indices.len() > rounded.indices.len(),
        "lames not added"
    );
    assert_closed_solid(&broad, "broad sabaton");
}

#[test]
fn boot_shaft_height_raises_its_top_by_the_same_distance() {
    let fit = frame([0.0; 3], [0.045, 0.04, 0.13]);
    let top = |height| {
        let boot = build(
            &LimbArmorDesign::LeatherBoot(BootDesign {
                shaft_height: Millimeters(height),
                ..Default::default()
            }),
            &[fit],
        )
        .unwrap();
        bounds(&boot.positions)[1][1]
    };
    let rise = top(300) - top(80);
    assert!((rise - 0.22).abs() < 0.001, "shaft rose {rise} m");
}

#[test]
fn greave_ankle_taper_narrows_only_the_lower_leg() {
    let fit = frame([0.0; 3], [0.065, 0.18, 0.06]);
    let greave = |taper| {
        build(
            &LimbArmorDesign::Greave(GreaveDesign {
                ankle_taper: Permille(taper),
                ..Default::default()
            }),
            &[fit],
        )
        .unwrap()
    };
    let (narrow, wide) = (greave(450), greave(850));
    assert_eq!(narrow.indices, wide.indices);
    let width_between = |mesh: &BuiltPart, low: f32, high: f32| {
        let band = mesh
            .positions
            .iter()
            .filter(|p| (low..high).contains(&p[1]))
            .copied()
            .collect::<Vec<_>>();
        common::extent(&band, 0)
    };
    let ankle = |mesh: &BuiltPart| width_between(mesh, -0.16, -0.12);
    let knee = |mesh: &BuiltPart| width_between(mesh, 0.12, 0.16);
    assert!(
        ankle(&narrow) < ankle(&wide) - 0.005,
        "ankle {} vs {}",
        ankle(&narrow),
        ankle(&wide)
    );
    assert!((knee(&narrow) - knee(&wide)).abs() < 0.002, "knee moved");
}

#[test]
fn parameter_extremes_preserve_shell_integrity() {
    let designs = [
        LimbArmorDesign::Greave(GreaveDesign {
            length: Permille(650),
            ankle_taper: Permille(450),
            shin_ridge: Millimeters(15),
            ..Default::default()
        }),
        LimbArmorDesign::Cuisse(CuisseDesign {
            wrap: Permille(900),
            knee_taper: Permille(550),
            ..Default::default()
        }),
        LimbArmorDesign::Rerebrace(RerebraceDesign {
            wrap: Permille(950),
            distal_taper: Permille(1100),
            ..Default::default()
        }),
        LimbArmorDesign::Poleyn(JointCupDesign {
            dome: Permille(1100),
            wing: Permille(650),
            length: Permille(650),
            ..Default::default()
        }),
        LimbArmorDesign::Spaulder(SpaulderDesign {
            crown: Permille(1350),
            lame_count: 6,
            ..Default::default()
        }),
        LimbArmorDesign::MittenGauntlet(GauntletDesign {
            cuff_length: Permille(650),
            cuff_flare: Permille(1600),
            finger_lames: 6,
            ..Default::default()
        }),
        LimbArmorDesign::Sabaton(FootArmorDesign {
            toe_width: Permille(1450),
            toe_extension: Millimeters(50),
            ankle_cutaway: Millimeters(30),
            lame_count: 8,
            ..Default::default()
        }),
        LimbArmorDesign::LeatherBoot(BootDesign {
            shaft_height: Millimeters(350),
            shaft_flare: Permille(1500),
            ..Default::default()
        }),
    ];
    for design in designs {
        let mesh = build(&design, &[frame([0.0; 3], [0.05, 0.08, 0.08])]).unwrap();
        assert_closed_solid(&mesh, &format!("{design:?}"));
    }
}

#[test]
fn invalid_designs_and_frames_fail_at_the_boundary() {
    let fit = frame([0.0; 3], [0.05, 0.08, 0.08]);
    let invalid = LimbArmorDesign::Spaulder(SpaulderDesign {
        lame_count: 0,
        ..Default::default()
    });
    assert!(invalid.validate().is_err());
    assert!(build(&invalid, &[fit]).is_err());
    let invalid = LimbArmorDesign::Sabaton(FootArmorDesign {
        ankle_cutaway: Millimeters(31),
        ..Default::default()
    });
    assert!(build(&invalid, &[fit]).is_err());

    let greave = LimbArmorDesign::Greave(GreaveDesign::default());
    let mut not_finite = fit;
    not_finite.half_extents[0] = f32::NAN;
    let mut collapsed = fit;
    collapsed.half_extents[2] = 0.0;
    let mut skewed = fit;
    skewed.axes[1] = skewed.axes[0];
    for bad in [not_finite, collapsed, skewed] {
        assert!(build(&greave, &[bad]).is_err(), "{bad:?}");
        assert!(
            build(
                &LimbArmorDesign::Sabaton(FootArmorDesign::default()),
                &[bad]
            )
            .is_err(),
            "{bad:?}"
        );
    }
}

#[test]
fn extremity_builders_reject_limb_plates_and_limb_builders_reject_extremities() {
    let fit = frame([0.0; 3], [0.05, 0.08, 0.08]);
    let greave = LimbArmorDesign::Greave(GreaveDesign::default());
    let result = gpu().build_in(&[fit], |batch, buffers| {
        record_extremity_armor(gpu(), batch, &greave, buffers)
    });
    assert!(result.is_err());
    let boot = LimbArmorDesign::LeatherBoot(BootDesign::default());
    assert!(generate_limb_armor_on(gpu(), &boot, &fit).is_err());
}

#[test]
fn the_thumb_is_placed_by_its_own_frame() {
    let design = LimbArmorDesign::MittenGauntlet(GauntletDesign::default());
    let hand = frame([0.3, 0.9, 0.1], [0.042, 0.09, 0.018]);
    let near = thumb(&hand);
    let offset = [0.02, -0.01, 0.015];
    let moved = PartFrame {
        origin: std::array::from_fn(|i| near.origin[i] + offset[i]),
        ..near
    };
    let a = build(&design, &[hand, near]).unwrap();
    let b = build(&design, &[hand, moved]).unwrap();
    assert_eq!(a.indices, b.indices);
    let (mut still, mut shifted) = (0, 0);
    for (p, q) in a.positions.iter().zip(&b.positions) {
        if p == q {
            still += 1;
        } else {
            for axis in 0..3 {
                assert!(
                    (q[axis] - p[axis] - offset[axis]).abs() < 1e-5,
                    "a thumb vertex did not follow its frame rigidly: {p:?} -> {q:?}"
                );
            }
            shifted += 1;
        }
    }
    assert!(still > shifted, "the hand plates moved with the thumb");
    assert!(shifted > 0, "no thumb shell");
    // The thumb lies along its own long axis, here the hand's width.
    let thumb_points = a
        .positions
        .iter()
        .zip(&b.positions)
        .filter(|(p, q)| p != q)
        .map(|(p, _)| *p)
        .collect::<Vec<_>>();
    assert!(common::extent(&thumb_points, 0) > common::extent(&thumb_points, 1));
    assert!(largest_difference(&a.positions, &b.positions) > 0.01);
}

#[test]
fn sabaton_rejects_ankle_trim_that_consumes_the_instep_span() {
    let design = LimbArmorDesign::Sabaton(FootArmorDesign {
        ankle_cutaway: Millimeters(30),
        ..Default::default()
    });
    design.validate().unwrap();
    for half_length in [0.030, 0.030 / 0.70] {
        let short_foot = frame([0.0; 3], [0.040, 0.030, half_length]);
        assert!(
            build(&design, &[short_foot]).is_err(),
            "half length {half_length}"
        );
    }
    let compatible = build(&design, &[frame([0.0; 3], [0.040, 0.030, 0.060])]).unwrap();
    assert_closed_solid(&compatible, "trimmed sabaton");
}

#[test]
fn joint_cup_constructions_are_closed_solids_with_plate_and_extension_components() {
    use fabelgeist_armor::{
        ArmorComponentRole, JointCupConstruction, JointExtension, JointFluteOrientation,
        PlateFluting,
    };
    let fit = frame([0.1, 0.9, -0.05], [0.06, 0.055, 0.055]);
    let raised = JointCupDesign {
        construction: JointCupConstruction::RaisedCop,
        wing: Permille(1200),
        medial_wrap: Permille(400),
        lateral_wrap: Permille(900),
        ..JointCupDesign::poleyn()
    };
    let extended = JointCupDesign {
        distal_extension: Some(JointExtension::default()),
        ..JointCupDesign::poleyn()
    };
    let transverse = JointCupDesign {
        fluting: Some(PlateFluting::default()),
        flute_orientation: JointFluteOrientation::Transverse,
        ..JointCupDesign::couter()
    };
    for (name, design, extension_lames) in [
        ("raised cop", raised, 0),
        ("extended poleyn", extended, 3),
        ("transverse flutes", transverse, 0),
    ] {
        let limb = LimbArmorDesign::Poleyn(design);
        limb.validate().unwrap_or_else(|e| panic!("{name}: {e}"));
        let part = build(&limb, &[fit]).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_closed_solid(&part, name);
        let roles = part
            .components
            .iter()
            .map(|component| component.role)
            .collect::<Vec<_>>();
        let expected = if extension_lames > 0 {
            vec![
                ArmorComponentRole::Plate,
                ArmorComponentRole::JointExtension,
            ]
        } else {
            vec![ArmorComponentRole::Plate]
        };
        assert_eq!(roles, expected, "{name}");
        let covered = part
            .components
            .iter()
            .map(|component| component.vertices.len())
            .sum::<usize>();
        assert_eq!(covered, part.positions.len(), "{name}");
    }
}

#[test]
fn a_joint_extension_hangs_below_the_cup_and_its_plates_lap_outward() {
    use fabelgeist_armor::{ArmorComponentRole, JointExtension};
    let fit = frame([0.0; 3], [0.06, 0.055, 0.055]);
    let design = JointCupDesign {
        distal_extension: Some(JointExtension::default()),
        ..JointCupDesign::poleyn()
    };
    let part = build(&LimbArmorDesign::Poleyn(design), &[fit]).unwrap();
    let range = |role| {
        part.components
            .iter()
            .find(|component| component.role == role)
            .unwrap()
            .vertices
            .clone()
    };
    let lowest = |vertices: std::ops::Range<usize>| {
        part.positions[vertices]
            .iter()
            .map(|p| p[1])
            .fold(f32::INFINITY, f32::min)
    };
    let cup = lowest(range(ArmorComponentRole::Plate));
    let extension = lowest(range(ArmorComponentRole::JointExtension));
    // The default extension is 130 mm long below the cup's distal edge.
    assert!(extension < cup - 0.1, "extension {extension} vs cup {cup}");
}

#[test]
fn spaulder_crown_coverage_and_besagew_are_closed_solids() {
    use fabelgeist_armor::{ArmorComponentRole, BesagewDesign, RadialFluting};
    let fit = frame([0.1, 0.9, -0.05], [0.065, 0.09, 0.07]);
    let fluted = BesagewDesign {
        fluting: Some(RadialFluting::default()),
        ..BesagewDesign::default()
    };
    for (name, coverage, besagew) in [
        ("short crown", 600, None),
        ("besagew", 1000, Some(BesagewDesign::default())),
        ("fluted besagew on a short crown", 700, Some(fluted)),
    ] {
        let design = LimbArmorDesign::Spaulder(SpaulderDesign {
            crown_coverage: Permille(coverage),
            besagew: besagew.clone(),
            ..SpaulderDesign::default()
        });
        design.validate().unwrap_or_else(|e| panic!("{name}: {e}"));
        let part = build(&design, &[fit]).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_closed_solid(&part, name);
        let roles = part
            .components
            .iter()
            .map(|component| component.role)
            .collect::<Vec<_>>();
        if besagew.is_some() {
            assert_eq!(
                roles,
                [ArmorComponentRole::Plate, ArmorComponentRole::Besagew],
                "{name}"
            );
        } else {
            assert!(roles.is_empty(), "{name}");
        }
    }
}

#[test]
fn shorter_crown_coverage_lowers_the_crown_toward_the_neck() {
    let fit = frame([0.0; 3], [0.065, 0.09, 0.07]);
    let top = |coverage| {
        let design = LimbArmorDesign::Spaulder(SpaulderDesign {
            crown_coverage: Permille(coverage),
            ..SpaulderDesign::default()
        });
        bounds(&build(&design, &[fit]).unwrap().positions)[1][1]
    };
    assert!(top(500) < top(1000) - 0.02);
}

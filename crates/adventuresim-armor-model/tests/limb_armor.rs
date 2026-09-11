use std::collections::BTreeMap;

use adventuresim_armor_model::{
    BootDesign, CuisseDesign, FootArmorDesign, GauntletDesign, GreaveDesign, JointCupDesign,
    LimbArmorDesign, Millimeters, PartFrame, PartMesh, Permille, RerebraceDesign, SpaulderDesign,
    generate_gauntlet_thumb, generate_limb_armor,
};

fn frame(extents: [f32; 3]) -> PartFrame {
    PartFrame {
        origin: [0.0; 3],
        axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        half_extents: extents,
    }
}

#[test]
fn sabaton_rejects_ankle_trim_that_consumes_the_instep_span() {
    let foot = FootArmorDesign {
        ankle_cutaway: Millimeters(30),
        ..Default::default()
    };
    let design = LimbArmorDesign::Sabaton(foot);
    design.validate().unwrap();
    for half_length in [0.030, 0.030 / 0.70] {
        let fit = frame([0.040, 0.030, half_length]);
        fit.validate().unwrap();
        let result = generate_limb_armor(&design, &fit);
        assert!(
            matches!(
                result,
                Err(adventuresim_armor_model::GenerateError::SabatonTrimExceedsFoot { .. })
            ),
            "half length {half_length}: {result:?}"
        );
    }
    let compatible = frame([0.040, 0.030, 0.060]);
    generate_limb_armor(&design, &compatible)
        .unwrap()
        .normals()
        .unwrap();
}

fn families() -> Vec<(LimbArmorDesign, PartFrame)> {
    vec![
        (
            LimbArmorDesign::Greave(GreaveDesign::default()),
            frame([0.065, 0.18, 0.06]),
        ),
        (
            LimbArmorDesign::Cuisse(CuisseDesign::default()),
            frame([0.085, 0.18, 0.08]),
        ),
        (
            LimbArmorDesign::Rerebrace(RerebraceDesign::default()),
            frame([0.045, 0.13, 0.05]),
        ),
        (
            LimbArmorDesign::Poleyn(JointCupDesign::poleyn()),
            frame([0.06, 0.055, 0.055]),
        ),
        (
            LimbArmorDesign::Couter(JointCupDesign::couter()),
            frame([0.045, 0.045, 0.04]),
        ),
        (
            LimbArmorDesign::Spaulder(SpaulderDesign::default()),
            frame([0.065, 0.09, 0.07]),
        ),
        (
            LimbArmorDesign::MittenGauntlet(GauntletDesign::default()),
            frame([0.042, 0.09, 0.018]),
        ),
        (
            LimbArmorDesign::Sabaton(FootArmorDesign::default()),
            frame([0.045, 0.04, 0.13]),
        ),
        (
            LimbArmorDesign::LeatherBoot(BootDesign::default()),
            frame([0.045, 0.04, 0.13]),
        ),
    ]
}

fn check_closed_winding(mesh: &PartMesh) {
    assert!(mesh.normals().is_ok());
    let mut edges = BTreeMap::<(u32, u32), Vec<(u32, u32)>>::new();
    let mut signed_volume = 0.0_f64;
    for face in mesh.indices.as_chunks::<3>().0 {
        for (a, b) in [(face[0], face[1]), (face[1], face[2]), (face[2], face[0])] {
            edges.entry((a.min(b), a.max(b))).or_default().push((a, b));
        }
        let [a, b, c] =
            [face[0], face[1], face[2]].map(|i| mesh.positions[i as usize].map(f64::from));
        signed_volume += (a[0] * (b[1] * c[2] - b[2] * c[1])
            + a[1] * (b[2] * c[0] - b[0] * c[2])
            + a[2] * (b[0] * c[1] - b[1] * c[0]))
            / 6.0;
    }
    assert!(
        edges
            .values()
            .all(|uses| uses.len() == 2 && uses[0] == (uses[1].1, uses[1].0))
    );
    assert!(
        signed_volume > 0.0,
        "outward surfaces must enclose positive wall volume"
    );
}

#[test]
fn every_family_has_closed_walls_and_consistent_winding_at_body_extremes() {
    for (design, base) in families() {
        for scale in [0.72, 1.0, 1.4] {
            let mut fit = base;
            fit.half_extents = fit.half_extents.map(|v| v * scale);
            let mesh =
                generate_limb_armor(&design, &fit).unwrap_or_else(|e| panic!("{design:?}: {e}"));
            check_closed_winding(&mesh);
            let again = generate_limb_armor(&design, &fit).unwrap();
            assert_eq!(mesh.positions, again.positions);
            assert_eq!(mesh.indices, again.indices);
            fit.axes[0][0] = -1.0;
            check_closed_winding(&generate_limb_armor(&design, &fit).unwrap());
        }
    }
}

#[test]
fn design_controls_change_the_intended_volumes() {
    let fit = frame([0.045, 0.04, 0.13]);
    let rounded =
        generate_limb_armor(&LimbArmorDesign::Sabaton(FootArmorDesign::default()), &fit).unwrap();
    let broad = generate_limb_armor(
        &LimbArmorDesign::Sabaton(FootArmorDesign {
            toe_width: Permille(1400),
            toe_extension: Millimeters(40),
            lame_count: 8,
            ..Default::default()
        }),
        &fit,
    )
    .unwrap();
    let bounds = |mesh: &PartMesh, axis: usize| {
        mesh.positions
            .iter()
            .map(|p| p[axis])
            .fold(f32::NEG_INFINITY, f32::max)
    };
    assert!(bounds(&broad, 0) > bounds(&rounded, 0) + 0.01);
    assert!(bounds(&broad, 2) > bounds(&rounded, 2) + 0.025);
    assert!(broad.indices.len() > rounded.indices.len());
    check_closed_winding(&broad);
    let boot = |height| {
        generate_limb_armor(
            &LimbArmorDesign::LeatherBoot(BootDesign {
                shaft_height: Millimeters(height),
                ..Default::default()
            }),
            &fit,
        )
        .unwrap()
    };
    assert!((bounds(&boot(300), 1) - bounds(&boot(80), 1) - 0.22).abs() < 0.001);
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
        check_closed_winding(&generate_limb_armor(&design, &frame([0.05, 0.08, 0.08])).unwrap());
    }
}

#[test]
fn invalid_design_and_fit_fail_at_the_boundary() {
    let fit = frame([0.05, 0.08, 0.08]);
    let invalid = LimbArmorDesign::Spaulder(SpaulderDesign {
        lame_count: 0,
        ..Default::default()
    });
    assert!(generate_limb_armor(&invalid, &fit).is_err());
    let mut invalid_fit = fit;
    invalid_fit.half_extents[0] = f32::NAN;
    assert!(
        generate_limb_armor(
            &LimbArmorDesign::Greave(GreaveDesign::default()),
            &invalid_fit
        )
        .is_err()
    );
}

#[test]
fn separate_thumb_uses_its_anatomical_axis_and_closed_tip() {
    let design = GauntletDesign::default();
    for scale in [0.72, 1.0, 1.4] {
        let mut fit = frame([0.011 * scale, 0.046 * scale, 0.012 * scale]);
        fit.origin = [0.1, 0.2, 0.3];
        fit.axes = [[0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]];
        let mesh = generate_gauntlet_thumb(&design, &fit).unwrap();
        check_closed_winding(&mesh);
        assert!(
            mesh.positions
                .iter()
                .any(|p| p[0] < fit.origin[0] - fit.half_extents[1])
        );
        assert!(mesh.positions.iter().all(|p| p[2] >= fit.origin[2] - 0.01));
    }
}

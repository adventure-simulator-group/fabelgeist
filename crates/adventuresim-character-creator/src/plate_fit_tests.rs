use super::*;
use crate::armor_frames::Side;
use adventuresim_armor_model::{
    CuisseDesign, GreaveDesign, LimbArmorDesign, Permille, RerebraceDesign, generate_limb_armor,
};

fn fitted(design: &LimbArmorDesign) -> PartMesh {
    let frame = PartFrame {
        origin: [0.0; 3],
        axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        half_extents: [0.06, 0.20, 0.045],
    };
    let (style, region) = match design {
        LimbArmorDesign::Greave(d) => (PlateFit::Greave(d), FitRegion::LowerLeg(Side::Left)),
        LimbArmorDesign::Cuisse(d) => (PlateFit::Cuisse(d), FitRegion::Thigh(Side::Left)),
        LimbArmorDesign::Rerebrace(d) => (PlateFit::Rerebrace(d), FitRegion::UpperArm(Side::Left)),
        _ => unreachable!(),
    };
    let points = (0..128)
        .map(|i| {
            let angle = i as f32 / 128.0 * std::f32::consts::TAU;
            [0.040 * angle.sin(), 0.0, 0.030 * angle.cos()]
        })
        .collect::<Vec<_>>();
    let sections = (0..STATIONS)
        .map(|_| Section::measured(&points, 0.0, 0.024))
        .collect::<Vec<_>>();
    let mesh = generate_limb_armor(design, &frame)
        .unwrap()
        .refit_surfaces(|p| fit_carrier(p, &frame, region, &sections, 0.012, style))
        .unwrap();
    mesh.normals().unwrap();
    mesh
}

fn assert_changes_fitted_shape(a: LimbArmorDesign, b: LimbArmorDesign) {
    let a = fitted(&a);
    let b = fitted(&b);
    assert_eq!(
        a.indices, b.indices,
        "style edits must retain correspondence"
    );
    let movement = a
        .positions
        .iter()
        .zip(&b.positions)
        .map(|(a, b)| {
            a.iter()
                .zip(b)
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f32>()
                .sqrt()
        })
        .fold(0.0, f32::max);
    assert!(
        movement > 0.001,
        "fitted style control has no meaningful effect: {movement} m"
    );
}

#[test]
fn long_plate_style_controls_survive_anatomical_fitting() {
    let greave = GreaveDesign::default();
    for variant in [
        GreaveDesign {
            ankle_taper: Permille(850),
            ..greave.clone()
        },
        GreaveDesign {
            knee_taper: Permille(1050),
            ..greave.clone()
        },
        GreaveDesign {
            calf_height: Permille(450),
            ..greave.clone()
        },
    ] {
        assert_changes_fitted_shape(
            LimbArmorDesign::Greave(greave.clone()),
            LimbArmorDesign::Greave(variant),
        );
    }
    let cuisse = CuisseDesign::default();
    assert_changes_fitted_shape(
        LimbArmorDesign::Cuisse(cuisse.clone()),
        LimbArmorDesign::Cuisse(CuisseDesign {
            knee_taper: Permille(1000),
            ..cuisse
        }),
    );
    let arm = RerebraceDesign::default();
    for variant in [
        RerebraceDesign {
            section_depth: Permille(1200),
            ..arm.clone()
        },
        RerebraceDesign {
            distal_taper: Permille(1100),
            ..arm.clone()
        },
    ] {
        assert_changes_fitted_shape(
            LimbArmorDesign::Rerebrace(arm.clone()),
            LimbArmorDesign::Rerebrace(variant),
        );
    }
}

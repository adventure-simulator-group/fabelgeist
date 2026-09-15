use super::*;
use crate::model::profile_tests::endpoint;

#[test]
fn top_bottom_modes_are_simple_and_independent() {
    for top in [
        ShapedShieldTopShape::Flat,
        ShapedShieldTopShape::Rounded,
        ShapedShieldTopShape::SinglePeak,
        ShapedShieldTopShape::DoublePeak,
    ] {
        for bottom in [
            ShapedShieldBottomShape::Flat,
            ShapedShieldBottomShape::Rounded,
            ShapedShieldBottomShape::Point,
        ] {
            let mut recipe = endpoint("heater-shield", None);
            let Shape::ShapedShield(p) = &mut recipe.components[0].shape else {
                panic!()
            };
            p.top_depth = Metres::new(if top == ShapedShieldTopShape::Flat {
                0.0
            } else {
                0.1
            })
            .unwrap();
            p.bottom_depth = Metres::new(if bottom == ShapedShieldBottomShape::Flat {
                0.0
            } else {
                0.16
            })
            .unwrap();
            p.top_shape = top.clone();
            p.bottom_shape = bottom;
            for detail in [Detail::Low, Detail::Medium, Detail::High] {
                let shield = Shield::from_shape(&recipe.components[0].shape).unwrap();
                let outline = shield.outline(detail);
                assert!(outline.len() >= 12);
                assert!(Region::triangulate(&outline, false).is_ok());
            }
        }
    }
}

#[test]
fn kite_taper_and_roman_rounded_corners_shape_the_silhouette() {
    for detail in [Detail::Low, Detail::Medium, Detail::High] {
        let kite = endpoint("kite-shield", None);
        let outline = Shield::from_shape(&kite.components[0].shape)
            .unwrap()
            .outline(detail);
        assert!(outline[outline.len() / 2][0].abs() < outline[0][0].abs() * 0.55);
        let roman = endpoint("roman-tower-shield", None);
        let Shape::ShapedShield(p) = &roman.components[0].shape else {
            panic!()
        };
        let outline = Shield::from_shape(&roman.components[0].shape)
            .unwrap()
            .outline(detail);
        assert!(outline[0][1] < p.height.get() / 2.0 - p.corner_radius.get() * 0.9);
        assert!(outline[(outline.len() / 2 - 1) / 2][1] > p.height.get() / 2.0 - 1e-9);
    }
}

#[test]
fn fittings_mirror_and_rotate_and_only_center_grips_open_the_body() {
    let mut round = endpoint("round-shield", None);
    let Shape::RoundShield(p) = &mut round.components[0].shape else {
        panic!()
    };
    p.mirrored = Some(false);
    p.fitting_angle = Some(Degrees::new(0.0).unwrap());
    let layout = |recipe: &Recipe| {
        Shield::from_shape(&recipe.components[0].shape)
            .unwrap()
            .layout(Detail::Medium)
            .unwrap()
    };
    let right = layout(&round);
    let Shape::RoundShield(p) = &mut round.components[0].shape else {
        panic!()
    };
    p.mirrored = Some(true);
    let left = layout(&round);
    assert_eq!(right.grip[0], -left.grip[0]);
    let Shape::RoundShield(p) = &mut round.components[0].shape else {
        panic!()
    };
    p.fitting_angle = Some(Degrees::new(90.0).unwrap());
    let turned = layout(&round);
    assert!((turned.grip[0] + left.grip[1]).abs() < 1e-9);
    assert!((turned.grip[1] - left.grip[0]).abs() < 1e-9);
    let mut buckler = endpoint("buckler", None);
    assert!(
        Shield::from_shape(&buckler.components[0].shape)
            .unwrap()
            .aperture()
            > 0.045
    );
    let Shape::RoundShield(p) = &mut buckler.components[0].shape else {
        panic!()
    };
    p.fitting_mode = RoundShieldFittingMode::GripAndStrap;
    assert_eq!(
        Shield::from_shape(&buckler.components[0].shape)
            .unwrap()
            .aperture(),
        0.0
    );
}

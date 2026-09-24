use adventuresim_heraldry::{
    artwork::{Artwork, PaintRole},
    bake::{Baked, Resolution},
    document::*,
    export, presets, provenance,
};

const RECIPES: [&str; 2] = ["wernigerode-eagle", "wernigerode-double-eagle"];

#[test]
fn both_sources_fit_the_broad_shield_and_retain_distinct_heads() {
    let mut pictures = Vec::new();
    for recipe in RECIPES {
        let d = presets::preset(recipe).unwrap();
        assert_eq!(d.surface.width, Millimeters(450.0));
        assert_eq!(d.surface.height, Millimeters(450.0));
        let art = Artwork::compose(&d).unwrap();
        for point in art.shapes.iter().skip(1).flat_map(|s| s.path.flattened(12)) {
            assert!((30.0..=970.0).contains(&point[0]), "{recipe}: {point:?}");
            assert!((15.0..=910.0).contains(&point[1]), "{recipe}: {point:?}");
        }
        // The source's large yellow shield must not survive as a second field.
        let gold: Vec<_> = art
            .shapes
            .iter()
            .skip(1)
            .filter(|s| s.tincture == Tincture::Or)
            .collect();
        assert_eq!(gold.len(), if recipe == RECIPES[0] { 1 } else { 2 });
        assert!(gold.iter().all(|s| s.role == PaintRole::Accent)); // irises
        pictures.push(Baked::generate(&d, Resolution::Draft).unwrap().flat);
    }
    assert_ne!(pictures[0], pictures[1]);
}

#[test]
fn flat_retains_lines_and_accents_while_painted_tones_ignore_lighting() {
    for recipe in RECIPES {
        let mut d = presets::preset(recipe).unwrap();
        let modeled = Artwork::compose(&d).unwrap();
        let bake = Baked::generate(&d, Resolution::Draft).unwrap();
        d.view.light = Degrees(140.0);
        d.view.yaw = Degrees(20.0);
        assert_eq!(
            bake.to_bytes(),
            Baked::generate(&d, Resolution::Draft).unwrap().to_bytes()
        );
        d.drawing.painted_modeling = PaintedModeling::FLAT;
        let flat = Artwork::compose(&d).unwrap();
        let fixed_paint = |a: &Artwork| {
            a.shapes
                .iter()
                .filter(|s| s.opacity == Ratio(1.0))
                .map(|s| (s.path.svg(), s.role, s.tincture))
                .collect::<Vec<_>>()
        };
        assert_eq!(fixed_paint(&modeled), fixed_paint(&flat));
        assert!(flat.shapes.iter().all(|s| !matches!(
            s.role,
            PaintRole::Shadow | PaintRole::Highlight | PaintRole::AccentHighlight
        )));
        let flat_bake = Baked::generate(&d, Resolution::Draft).unwrap();
        assert_ne!(bake.flat, flat_bake.flat);
        for modeling in [
            PaintedModeling {
                shadows: Ratio(0.7),
                highlights: Ratio(0.0),
            },
            PaintedModeling {
                shadows: Ratio(0.0),
                highlights: Ratio(0.7),
            },
        ] {
            d.drawing.painted_modeling = modeling;
            assert_ne!(
                flat_bake.flat,
                Baked::generate(&d, Resolution::Draft).unwrap().flat
            );
        }
    }
}

#[test]
fn counterchanging_keeps_tongues_and_armed_parts_independent() {
    for (index, recipe) in RECIPES.into_iter().enumerate() {
        let mut d = presets::preset(recipe).unwrap();
        d.arms.field = Field::Divided {
            division: Division::Pale,
            boundary: Boundary::Straight,
            tinctures: [Tincture::Azure, Tincture::Argent],
        };
        let charge = &mut d.arms.charges[0];
        charge.color = Coloring::Counterchanged {
            tinctures: [Tincture::Azure, Tincture::Argent],
        };
        charge.langued = Tincture::Vert;
        charge.armed = Tincture::Purpure;
        let art = Artwork::compose(&d).unwrap();
        let tongues: Vec<_> = art
            .shapes
            .iter()
            .filter(|s| s.tincture == Tincture::Vert)
            .collect();
        // Each source head has one tongue base, painted highlight and outline.
        assert_eq!(tongues.len(), (index + 1) * 3);
        assert!(tongues.iter().all(|s| !s.role.is_charge()));
        for role in [PaintRole::Charge, PaintRole::Shadow, PaintRole::Highlight] {
            for tincture in [Tincture::Azure, Tincture::Argent] {
                assert!(
                    art.shapes
                        .iter()
                        .any(|s| s.role == role && s.tincture == tincture)
                );
            }
        }
        assert!(
            art.shapes
                .iter()
                .any(|s| s.tincture == Tincture::Purpure && s.role == PaintRole::AccentHighlight)
        );
    }
}

#[test]
fn facing_reflects_every_sourced_contour_including_paint_and_strokes() {
    for recipe in RECIPES {
        let mut d = presets::preset(recipe).unwrap();
        d.drawing.asymmetry = Ratio(0.6);
        let left = Artwork::compose(&d).unwrap();
        let ChargeKind::Eagle { facing, .. } = &mut d.arms.charges[0].shape else {
            unreachable!()
        };
        *facing = Facing::Sinister;
        let right = Artwork::compose(&d).unwrap();
        assert_eq!(left.shapes.len(), right.shapes.len());
        for (a, b) in left.shapes.iter().skip(1).zip(right.shapes.iter().skip(1)) {
            for (a, b) in a.path.flattened(8).iter().zip(b.path.flattened(8)) {
                assert!((a[0] + b[0] - 1000.0).abs() < 0.001);
                assert!((a[1] - b[1]).abs() < 0.001);
            }
        }
    }
}

#[test]
fn mixed_nested_sources_receive_complete_deduplicated_export_credit() {
    let mut d = presets::preset(RECIPES[0]).unwrap();
    let single_credit = provenance::attribution(&d).unwrap();
    assert!(single_credit.contains("King of the Romans"));
    assert!(!single_credit.contains("Holy Roman Emperor"));
    let single = d.arms.clone();
    let double = presets::preset(RECIPES[1]).unwrap().arms;
    d.arms = ArmsDesign::plain(Tincture::Or);
    d.arms.field = Field::Quarterly {
        quarters: Box::new([single.clone(), double.clone(), single, double]),
    };
    d.arms.inescutcheon = Some(Box::new(ArmsDesign::lion()));
    let bake = Baked::generate(&d, Resolution::Draft).unwrap();
    let bundle = export::bundle(&d, &bake, Resolution::Draft).unwrap();
    for name in ["ATTRIBUTION.txt", "arms.svg", "display.glb"] {
        let bytes = &bundle.iter().find(|file| file.name == name).unwrap().bytes;
        let text = String::from_utf8_lossy(bytes);
        for title in [
            "Single eagle artwork",
            "Double eagle artwork",
            "Lion artwork",
        ] {
            assert_eq!(text.matches(title).count(), 1, "{name}: {title}");
        }
        for required in ["Heralder", "Rinaldum", "CC BY-SA 3.0", "shield removal"] {
            assert!(text.contains(required), "{name}: {required}");
        }
    }
}

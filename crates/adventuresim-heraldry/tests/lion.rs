use adventuresim_heraldry::{
    artwork::{Artwork, PaintRole},
    bake::{Baked, Resolution},
    document::*,
    presets,
};

fn bake(d: &Document) -> Baked {
    Baked::generate(d, Resolution::Draft).unwrap()
}

#[test]
fn painted_modeling_preserves_linework_and_is_independent_of_lighting() {
    let mut d = presets::preset("german-lion").unwrap();
    let period_art = Artwork::compose(&d).unwrap();
    let period = bake(&d);
    for role in [PaintRole::Shadow, PaintRole::Highlight] {
        assert!(period_art.shapes.iter().any(|s| s.role == role));
    }
    d.view.light = Degrees(130.0);
    d.view.yaw = Degrees(30.0);
    d.view.exposure = 1.0;
    assert_eq!(period.to_bytes(), bake(&d).to_bytes());
    d.drawing.painted_modeling = PaintedModeling::FLAT;
    let clean_art = Artwork::compose(&d).unwrap();
    assert!(
        clean_art
            .shapes
            .iter()
            .all(|s| !matches!(s.role, PaintRole::Shadow | PaintRole::Highlight))
    );
    let structure = |a: &Artwork| {
        a.shapes
            .iter()
            .filter(|s| !matches!(s.role, PaintRole::Shadow | PaintRole::Highlight))
            .map(|s| (s.path.svg(), s.tincture, s.role, s.opacity))
            .collect::<Vec<_>>()
    };
    assert_eq!(structure(&period_art), structure(&clean_art));
    assert!(
        clean_art
            .shapes
            .iter()
            .any(|s| { s.role == PaintRole::Accent && s.tincture == Tincture::Sable })
    );
    let clean = bake(&d);
    assert_ne!(period.flat, clean.flat);
    assert_ne!(period.albedo, clean.albedo);
    assert_eq!(
        period
            .flat
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| p[3])
            .collect::<Vec<_>>(),
        clean
            .flat
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| p[3])
            .collect::<Vec<_>>()
    );
}

#[test]
fn shadow_and_highlight_paint_can_be_used_independently() {
    let mut d = presets::preset("german-lion").unwrap();
    d.drawing.painted_modeling = PaintedModeling::FLAT;
    let flat = bake(&d);
    d.drawing.painted_modeling.shadows = Ratio(0.6);
    let shadow = bake(&d);
    assert_ne!(flat.flat, shadow.flat);
    assert!(
        Artwork::compose(&d)
            .unwrap()
            .shapes
            .iter()
            .all(|s| s.role != PaintRole::Highlight)
    );
    d.drawing.painted_modeling = PaintedModeling {
        shadows: Ratio(0.0),
        highlights: Ratio(0.4),
    };
    let highlight = bake(&d);
    assert_ne!(flat.flat, highlight.flat);
    assert_ne!(shadow.flat, highlight.flat);
    assert!(
        Artwork::compose(&d)
            .unwrap()
            .shapes
            .iter()
            .all(|s| s.role != PaintRole::Shadow)
    );
}

#[test]
fn painted_modeling_rejects_invalid_coverage() {
    for coverage in [-0.01, 1.01, f32::NAN, f32::INFINITY] {
        for shadows in [false, true] {
            let mut d = presets::preset("german-lion").unwrap();
            if shadows {
                d.drawing.painted_modeling.shadows = Ratio(coverage);
            } else {
                d.drawing.painted_modeling.highlights = Ratio(coverage);
            }
            assert!(d.validate().is_err());
        }
    }
}

#[test]
fn modeling_pigment_covers_leaf_without_cutting_through_its_relief() {
    let mut d = presets::preset("german-lion").unwrap();
    d.surface.gold = MetalFinish::MordantGilding {
        relief: Millimeters(0.08),
    };
    let period = bake(&d);
    d.drawing.painted_modeling = PaintedModeling::FLAT;
    let clean = bake(&d);
    let mut overpainted = 0;
    for (i, (p, c)) in period
        .orm
        .as_chunks::<4>()
        .0
        .iter()
        .zip(clean.orm.as_chunks::<4>().0.iter())
        .enumerate()
    {
        if c[2] == 255 && p[2] < 240 {
            overpainted += 1;
            assert!(period.height[i] >= clean.height[i] - 0.001);
        }
    }
    assert!(overpainted > 20, "painted modeling must cover some leaf");
}

#[test]
fn two_tails_and_their_clips_follow_facing_and_composition() {
    let mut d = presets::preset("german-lion").unwrap();
    d.surface.shape = DisplayShape::Panel;
    d.arms.charges[0].center = [0.5, 0.5];
    let one = bake(&d);
    d.arms.charges[0].shape = ChargeKind::Lion {
        tails: LionTails::Two,
        facing: Facing::Dexter,
    };
    let dexter = bake(&d);
    let dexter_art = Artwork::compose(&d).unwrap();
    assert_ne!(one.flat, dexter.flat);
    d.arms.charges[0].shape = ChargeKind::Lion {
        tails: LionTails::Two,
        facing: Facing::Sinister,
    };
    let sinister = bake(&d);
    let sinister_art = Artwork::compose(&d).unwrap();
    for (a, b) in dexter_art
        .shapes
        .iter()
        .skip(1)
        .zip(sinister_art.shapes.iter().skip(1))
    {
        // The final clip is the unchanged panel. Earlier clips belong to the
        // drawing and must reflect with it, as must every painted contour.
        let local = a.clips.len() - 1;
        assert_eq!(a.clips.len(), b.clips.len());
        for (a, b) in std::iter::once(&a.path)
            .chain(&a.clips[..local])
            .zip(std::iter::once(&b.path).chain(&b.clips[..local]))
        {
            for (a, b) in a.flattened(8).iter().zip(b.flattened(8)) {
                assert!((1000.0 - a[0] - b[0]).abs() < 0.001);
                assert!((a[1] - b[1]).abs() < 0.001);
            }
        }
    }
    let n = dexter.size as usize;
    let mut error = 0u64;
    for y in 0..n {
        for x in 0..n {
            let a = &dexter.flat[(y * n + x) * 4..][..4];
            let b = &sinister.flat[(y * n + n - 1 - x) * 4..][..4];
            error += a
                .iter()
                .zip(b)
                .map(|(a, b)| u64::from(a.abs_diff(*b)))
                .sum::<u64>();
        }
    }
    // Raster edge coverage differs slightly with curve direction. Compare
    // image error as well as the stricter vector invariant above.
    assert!(error as f64 / ((n * n * 4) as f64) < 0.25);
}

#[test]
fn counterchanging_recolors_painted_tones_and_preserves_accents() {
    let mut d = presets::preset("german-lion").unwrap();
    d.arms.field = Field::Divided {
        division: Division::Pale,
        boundary: Boundary::Straight,
        tinctures: [Tincture::Azure, Tincture::Argent],
    };
    d.arms.charges[0].color = Coloring::Counterchanged {
        tinctures: [Tincture::Azure, Tincture::Argent],
    };
    let art = Artwork::compose(&d).unwrap();
    for role in [PaintRole::Shadow, PaintRole::Highlight] {
        for tincture in [Tincture::Azure, Tincture::Argent] {
            assert!(
                art.shapes
                    .iter()
                    .any(|s| s.role == role && s.tincture == tincture)
            );
        }
        assert!(
            art.shapes
                .iter()
                .all(|s| s.role != role || s.tincture != Tincture::Or)
        );
    }
    assert!(
        art.shapes
            .iter()
            .any(|s| s.role == PaintRole::Accent && s.tincture == Tincture::Gules)
    );
}

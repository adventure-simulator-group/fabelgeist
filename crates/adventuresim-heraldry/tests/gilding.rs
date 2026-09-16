use adventuresim_heraldry::{
    bake::{Baked, Resolution},
    document::*,
};

fn panel() -> Document {
    let mut d = Document {
        arms: ArmsDesign::plain(Tincture::Or),
        ..Default::default()
    };
    d.surface.shape = DisplayShape::Panel;
    d.surface.gold = MetalFinish::BURNISHED;
    d
}
fn bake(d: &Document) -> Baked {
    Baked::generate(d, Resolution::Draft).unwrap()
}

#[test]
fn intact_leaf_has_no_sheet_color_grid_and_application_changes_reflections() {
    let mut d = panel();
    let water = bake(&d);
    let reflectance = &water.albedo[..4];
    assert!(
        water
            .albedo
            .as_chunks::<4>()
            .0
            .iter()
            .all(|p| p == reflectance)
    );
    for finish in [MetalFinish::OilGilding, MetalFinish::RAISED_MORDANT] {
        d.surface.gold = finish;
        let applied = bake(&d);
        assert_eq!(water.flat, applied.flat);
        assert_eq!(water.albedo, applied.albedo);
        assert_ne!(water.orm, applied.orm);
        assert_ne!(water.height, applied.height);
        assert!(applied.coat.as_chunks::<4>().0.iter().all(|p| p[0] == 0));
    }
    d.surface.gold = MetalFinish::BURNISHED;
    d.surface.seed.0 += 1;
    let varied = bake(&d);
    assert_eq!(water.albedo, varied.albedo);
    assert_ne!(water.height, varied.height);
    d.surface.palette[Tincture::Or] = adventuresim_heraldry::paint::Paint::Recipe {
        recipe: adventuresim_heraldry::paint::PaintRecipe::YellowOchreTempera,
    };
    let palette = bake(&d);
    assert_ne!(varied.flat, palette.flat);
    assert_eq!(varied.albedo, palette.albedo);
}

#[test]
fn burnishing_smooths_the_surface_without_darkening_the_metal() {
    let mut d = panel();
    d.surface.gold = MetalFinish::WaterGilding {
        burnish: Ratio(0.0),
    };
    let unburnished = bake(&d);
    d.surface.gold = MetalFinish::WaterGilding {
        burnish: Ratio(1.0),
    };
    let burnished = bake(&d);
    assert_eq!(unburnished.flat, burnished.flat);
    assert_eq!(unburnished.albedo, burnished.albedo);
    assert!(
        unburnished
            .orm
            .as_chunks::<4>()
            .0
            .iter()
            .zip(burnished.orm.as_chunks::<4>().0)
            .all(|(a, b)| a[1] > b[1] && a[2] == b[2])
    );
    let amplitude = |b: &Baked| b.height.iter().map(|h| h.abs()).sum::<f32>();
    assert!(amplitude(&burnished) < amplitude(&unburnished));
}

#[test]
fn silver_glaze_filters_reflectance_and_coats_only_its_decoration() {
    let mut d = panel();
    d.arms.field = Field::Divided {
        division: Division::Pale,
        boundary: Boundary::Straight,
        tinctures: [Tincture::Or, Tincture::Azure],
    };
    d.surface.gold = MetalFinish::YellowGlazedSilver { depth: Ratio(0.0) };
    let clear = bake(&d);
    d.surface.gold = MetalFinish::YellowGlazedSilver { depth: Ratio(1.0) };
    let yellow = bake(&d);
    let gold = (64 * 128 + 32) * 4;
    let paint = (64 * 128 + 96) * 4;
    assert_eq!(clear.flat, yellow.flat);
    assert_eq!(clear.orm, yellow.orm);
    assert_eq!(clear.coat, yellow.coat);
    assert_eq!(yellow.coat[gold], 255);
    assert_eq!(yellow.coat[paint], 0);
    assert_eq!(yellow.orm[gold + 2], 255);
    assert_eq!(yellow.orm[paint + 2], 0);
    assert_eq!(
        clear.albedo[paint..paint + 4],
        yellow.albedo[paint..paint + 4]
    );
    for c in 0..3 {
        assert!(clear.albedo[gold + c] > yellow.albedo[gold + c]);
    }
    assert!(yellow.albedo[gold] > yellow.albedo[gold + 2]);
    d.surface.glaze_roughness = Ratio(0.1);
    let glossy = bake(&d);
    assert_ne!(yellow.coat[gold + 1], glossy.coat[gold + 1]);
    assert_eq!(yellow.orm, glossy.orm);
    assert_eq!(yellow.albedo, glossy.albedo);
}

#[test]
fn raised_mordant_follows_the_gilded_shape_and_keeps_field_paint_unchanged() {
    let mut d = panel();
    d.arms = ArmsDesign::plain(Tincture::Azure);
    d.arms
        .charges
        .push(Charge::new(ChargeKind::Roundel, Tincture::Or));
    d.surface.gold = MetalFinish::MordantGilding {
        relief: Millimeters(0.01),
    };
    let thin = bake(&d);
    d.surface.gold = MetalFinish::MordantGilding {
        relief: Millimeters(0.08),
    };
    let raised = bake(&d);
    assert_eq!(thin.flat, raised.flat);
    assert_eq!(thin.albedo, raised.albedo);
    assert_eq!(thin.height[5 * 128 + 5], raised.height[5 * 128 + 5]);
    assert!(raised.height[64 * 128 + 64] - thin.height[64 * 128 + 64] > 0.06);
    assert_ne!(thin.normal, raised.normal);
}

#[test]
fn unsupported_gilding_combinations_and_obsolete_sheet_controls_are_rejected() {
    let mut d = panel();
    d.surface.silver = MetalFinish::GLAZED_SILVER;
    assert!(d.validate().is_err());
    d.surface.silver = MetalFinish::BURNISHED;
    d.surface.gold = MetalFinish::WaterGilding {
        burnish: Ratio(1.1),
    };
    assert!(d.validate().is_err());
    let d = panel();
    let mut json = serde_json::to_value(&d).unwrap();
    json["surface"]["leaf_size"] = 70.into();
    assert!(Document::from_json(&json.to_string()).is_err());
    let mut json = serde_json::to_value(&d).unwrap();
    json["surface"]["gold"] = "Leaf".into();
    assert!(Document::from_json(&json.to_string()).is_err());
}

use adventuresim_heraldry::{
    bake::{Baked, Resolution},
    document::{Document, Tincture},
    export,
    paint::{Paint, mixing::*},
};

#[test]
fn measured_knots_survive_and_unsupported_mixtures_fail_at_input() {
    let data: serde_json::Value = serde_json::from_str(CALIBRATION_JSON).unwrap();
    assert_eq!(data["wavelengths_nm"].as_array().unwrap().len(), 77);
    for (stock, record) in PaintStock::ALL
        .into_iter()
        .zip(data["stocks"].as_array().unwrap())
    {
        for knot in record["knots"].as_array().unwrap() {
            let mix =
                StockMix::new(stock, knot["white_permille"].as_u64().unwrap() as u16).unwrap();
            let measured: Vec<f64> = serde_json::from_value(knot["reflectance"].clone()).unwrap();
            assert!(
                mix.reflectance()
                    .iter()
                    .zip(&measured)
                    .all(|(a, b)| (a - b).abs() < 1e-14)
            );
            assert!(mix.measured_knot());
        }
        if stock.supports_tints() {
            let a = StockMix::new(stock, 0).unwrap().reflectance();
            let b = StockMix::new(stock, 500).unwrap().reflectance();
            let middle = StockMix::new(stock, 250).unwrap().reflectance();
            assert!(
                middle
                    .iter()
                    .zip(a.iter().zip(b))
                    .all(|(m, (a, b))| (m - (a + b) / 2.0).abs() < 1e-14)
            );
            assert_eq!(
                StockMix::new(stock, 1000).unwrap().color(),
                StockMix::pure(PaintStock::LeadWhite).color()
            );
        }
    }
    for input in [
        r#"{"stock":"YellowOchre","white_permille":100}"#,
        r#"{"stock":"Azurite","white_permille":1001}"#,
        r#"{"stock":"Azurite","white_permille":0,"rgb":[0,0,0]}"#,
    ] {
        assert!(serde_json::from_str::<StockMix>(input).is_err());
    }
    // These are measured reference paints, including the pale black specimen.
    for (stock, expected) in [
        (PaintStock::LeadWhite, [240, 235, 221]),
        (PaintStock::Azurite, [63, 99, 155]),
        (PaintStock::GrapeSeedBlack, [137, 138, 138]),
    ] {
        assert!(
            StockMix::pure(stock)
                .color()
                .into_iter()
                .zip(expected)
                .all(|(a, b)| a.abs_diff(b) <= 1)
        );
    }
}

#[test]
fn quotes_use_prepared_stock_volume_and_ignore_zero_quantity_stock() {
    let mix = StockMix::new(PaintStock::Azurite, 250).unwrap();
    let mut workshop = Workshop {
        batch_microliters: BatchVolume(20_000),
        setup_cost: SetupCost(7),
        ..Default::default()
    };
    workshop.prices_per_ml[PaintStock::Azurite.index()] = StockPrice(8);
    workshop.prices_per_ml[PaintStock::LeadWhite.index()] = StockPrice(2);
    let quote = workshop.quote(mix).unwrap().unwrap();
    assert_eq!(quote.stock_milliliters, [5.0, 0.0, 0.0, 15.0, 0.0, 0.0]);
    assert_eq!(quote.cost_micro_units, 137_000_000);
    workshop.available[PaintStock::Azurite.index()] = false;
    assert!(workshop.quote(mix).unwrap().is_none());
    assert!(
        workshop
            .quote(StockMix::new(PaintStock::Azurite, 1000).unwrap())
            .unwrap()
            .is_some()
    );
    workshop.available[PaintStock::Azurite.index()] = true;
    workshop.budget_units = Some(136);
    assert!(workshop.quote(mix).unwrap().is_none());
    workshop.budget_units = Some(137);
    assert!(workshop.quote(mix).unwrap().is_some());
    workshop.batch_microliters = BatchVolume(0);
    assert!(workshop.quote(mix).is_err());
}

#[test]
fn solver_distinguishes_cheapest_match_nearest_and_no_available_recipe() {
    let mut request = SearchRequest {
        target_srgb: [0, 255, 0],
        tolerance: ColorTolerance(200.0),
        ..Default::default()
    };
    request.workshop.prices_per_ml = [StockPrice(10); 6];
    request.workshop.prices_per_ml[PaintStock::YellowOchre.index()] = StockPrice(1);
    let cheap = request.solve().unwrap().unwrap();
    assert_eq!(cheap.mix, StockMix::pure(PaintStock::YellowOchre));
    assert_eq!(cheap.status, MatchStatus::WithinTolerance);
    assert_eq!(cheap, request.solve().unwrap().unwrap());
    request.tolerance = ColorTolerance(0.0);
    assert_eq!(
        request.solve().unwrap().unwrap().status,
        MatchStatus::NearestOutsideTolerance
    );
    request.workshop.available = [false; 6];
    assert!(request.solve().unwrap().is_none());
    request.workshop.available[PaintStock::LeadWhite.index()] = true;
    let white = request.solve().unwrap().unwrap();
    assert_eq!(white.mix, StockMix::pure(PaintStock::LeadWhite));
    request.target_srgb = white.preview_srgb;
    assert_eq!(
        request.solve().unwrap().unwrap().status,
        MatchStatus::WithinTolerance
    );
    request.workshop.budget_units = Some(0);
    assert!(request.solve().unwrap().is_none());
    request.tolerance = ColorTolerance(f64::NAN);
    assert!(request.solve().is_err());
}

#[test]
fn changing_prices_changes_recipe_without_changing_target_or_allowed_color_error() {
    let target = StockMix::new(PaintStock::Azurite, 500).unwrap().color();
    let mut request = SearchRequest {
        target_srgb: target,
        tolerance: ColorTolerance(5.0),
        ..Default::default()
    };
    request.workshop.available = [true, false, false, true, false, false];
    request.workshop.prices_per_ml[0] = StockPrice(100);
    let blue_cheaper = request.solve().unwrap().unwrap();
    request.workshop.prices_per_ml[0] = StockPrice(1);
    request.workshop.prices_per_ml[3] = StockPrice(100);
    let white_cheaper = request.solve().unwrap().unwrap();
    assert!(blue_cheaper.mix.white_permille() < white_cheaper.mix.white_permille());
    for result in [blue_cheaper, white_cheaper] {
        assert!(result.delta_e76 <= 5.0);
        assert_eq!(result.status, MatchStatus::WithinTolerance);
    }
}

#[test]
fn independent_tone_recipes_roundtrip_and_export_calibration_without_changing_relief() {
    let mut document = Document::default();
    let before = Baked::generate(&document, Resolution::Draft).unwrap();
    let paint = MixedPaint {
        base: StockMix::pure(PaintStock::LeadTinYellow),
        shadow: StockMix::pure(PaintStock::YellowOchre),
        highlight: StockMix::new(PaintStock::LeadTinYellow, 750).unwrap(),
    };
    document.surface.palette[Tincture::Or] = Paint::Mixed { paint };
    assert_eq!(
        Document::from_json(&document.to_json().unwrap()).unwrap(),
        document
    );
    let baked = Baked::generate(&document, Resolution::Draft).unwrap();
    assert_eq!(before.height, baked.height);
    assert_ne!(before.flat, baked.flat);
    assert!(!before.matches(&document, Resolution::Draft));
    let files = export::bundle(&document, &baked, Resolution::Draft).unwrap();
    let calibration = files
        .iter()
        .find(|f| f.name == "paint-calibration.json")
        .unwrap();
    assert_eq!(calibration.bytes, CALIBRATION_JSON.as_bytes());
    let manifest: serde_json::Value = serde_json::from_slice(
        &files
            .iter()
            .find(|f| f.name == "paint-recipes.json")
            .unwrap()
            .bytes,
    )
    .unwrap();
    assert_eq!(manifest["palette"][0]["definition"]["id"], calibration_id());
    let mut invalid = serde_json::to_value(document).unwrap();
    invalid["surface"]["palette"][0] =
        serde_json::json!({"kind":"Custom","appearance":{"base":[0,0,0]}});
    assert!(Document::from_json(&invalid.to_string()).is_err());
}

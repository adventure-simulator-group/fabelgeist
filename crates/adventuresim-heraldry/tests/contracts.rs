use adventuresim_heraldry::{
    artwork::Artwork,
    bake::{Baked, Resolution, TextureKind, mips},
    document::*,
    export,
    geometry::Mesh,
    presets,
};

#[test]
fn presets_roundtrip_and_reproduce_across_worker_transport() {
    for name in presets::PRESETS {
        let d = presets::preset(name).unwrap();
        let restored = Document::from_json(&d.to_json().unwrap()).unwrap();
        assert_eq!(d, restored);
        let a = Baked::generate(&d, Resolution::Draft).unwrap();
        let b = Baked::generate(&restored, Resolution::Draft).unwrap();
        assert_eq!(a.to_bytes(), b.to_bytes());
        assert_eq!(
            a.to_bytes(),
            Baked::from_bytes(&a.to_bytes()).unwrap().to_bytes()
        );
        assert!(
            a.flat
                .as_chunks::<4>()
                .0
                .iter()
                .filter(|p| p[3] > 0)
                .count()
                > 1000
        );
        assert!(a.height.iter().all(|h| h.is_finite()));
    }
}
#[test]
fn viewing_is_independent_and_stale_exports_are_rejected() {
    let mut d = Document::default();
    let a = Baked::generate(&d, Resolution::Draft).unwrap();
    d.view.light = Degrees(145.0);
    d.view.yaw = Degrees(40.0);
    d.view.exposure = 2.0;
    assert!(a.matches(&d, Resolution::Draft));
    assert_eq!(
        a.to_bytes(),
        Baked::generate(&d, Resolution::Draft).unwrap().to_bytes()
    );
    d.surface.seed.0 += 1;
    assert!(export::bundle(&d, &a, Resolution::Draft).is_err());
    let b = Baked::generate(&d, Resolution::Draft).unwrap();
    assert_eq!(a.flat, b.flat);
    assert_ne!(a.albedo, b.albedo);
}
#[test]
fn rejects_unknown_fields_nonfinite_values_and_unbounded_compositions() {
    let d = Document::default();
    let mut json = serde_json::to_value(&d).unwrap();
    json["drawing"]["feather_rng"] = serde_json::json!(1);
    assert!(Document::from_json(&json.to_string()).is_err());
    let mut d = d;
    d.surface.width.0 = f32::NAN;
    assert!(d.validate().is_err());
    d.surface = PaintedSurface::default();
    for _ in 0..7 {
        let mut a = ArmsDesign::plain(Tincture::Or);
        a.inescutcheon = Some(Box::new(d.arms));
        d.arms = a;
    }
    assert!(d.validate().is_err());
    assert!(Baked::from_bytes(&[0; 68]).is_err());
}
#[test]
fn counterchanging_follows_fields_and_preserves_accent_tinctures() {
    let mut d = presets::preset("counterchanged").unwrap();
    d.surface.shape = DisplayShape::Panel;
    d.arms.charges[0] = Charge::new(ChargeKind::Roundel, Tincture::Or);
    d.arms.charges[0].color = Coloring::Counterchanged {
        tinctures: [Tincture::Azure, Tincture::Argent],
    };
    let b = Baked::generate(&d, Resolution::Draft).unwrap();
    let pixel = |x: usize, y: usize| &b.flat[(y * 128 + x) * 4..(y * 128 + x) * 4 + 3];
    for (x, t) in [(46, Tincture::Argent), (82, Tincture::Azure)] {
        for (a, b) in pixel(x, 60)
            .iter()
            .zip(d.surface.palette[t].appearance().colors.base)
        {
            assert!(a.abs_diff(b) < 4);
        }
    }
    let d = presets::preset("counterchanged").unwrap();
    let art = Artwork::compose(&d).unwrap();
    assert!(art.shapes.iter().any(|s| s.tincture == Tincture::Gules));
}
#[test]
fn heraldic_advice_does_not_reject_legal_exceptions() {
    let mut d = Document::default();
    d.arms.field = Field::Solid {
        tincture: Tincture::Azure,
    };
    assert!(!d.advice().is_empty());
    d.validate().unwrap();
}
#[test]
fn pigment_and_leaf_have_distinct_physical_response_without_changing_identity() {
    let mut d = Document::default();
    let a = Baked::generate(&d, Resolution::Draft).unwrap();
    d.surface.gold = MetalFinish::Pigment;
    let b = Baked::generate(&d, Resolution::Draft).unwrap();
    assert_eq!(a.flat, b.flat);
    assert!(a.orm.as_chunks::<4>().0.iter().any(|p| p[2] == 255));
    assert!(b.orm.as_chunks::<4>().0.iter().all(|p| p[2] == 0));
    assert_ne!(a.orm, b.orm);
}
#[test]
fn linear_color_mips_and_normal_mips_preserve_their_contracts() {
    let pixels = [
        0, 0, 0, 255, 255, 255, 255, 255, 0, 0, 0, 255, 255, 255, 255, 255,
    ];
    let m = mips(&pixels, 2, TextureKind::Color);
    assert!((m[1].rgba[0] as i16 - 188).abs() <= 1);
    let n = [
        128, 128, 255, 255, 128, 128, 255, 255, 128, 128, 255, 255, 128, 128, 255, 255,
    ];
    assert_eq!(
        mips(&n, 2, TextureKind::Normal)[1].rgba,
        [128, 128, 255, 255]
    );
    let transparent = [255, 0, 0, 255, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0];
    assert_eq!(
        &mips(&transparent, 2, TextureKind::Color)[1].rgba[..3],
        &[255, 0, 0]
    );
}
#[test]
fn closed_geometry_is_scaled_in_metres_with_outward_triangle_winding() {
    for shape in [DisplayShape::Panel, DisplayShape::Shield] {
        let mut d = Document::default();
        d.surface.shape = shape;
        let mesh = Mesh::generate(&d).unwrap();
        let mut edges = std::collections::BTreeMap::new();
        let key = |p: [f32; 3]| p.map(|v| (v * 1_000_000.0).round() as i32);
        for indices in [&mesh.front, &mesh.support] {
            for t in indices.as_chunks::<3>().0.iter() {
                let a = mesh.positions[t[0] as usize];
                let b = mesh.positions[t[1] as usize];
                let c = mesh.positions[t[2] as usize];
                let u = std::array::from_fn::<_, 3, _>(|i| b[i] - a[i]);
                let v = std::array::from_fn::<_, 3, _>(|i| c[i] - a[i]);
                let cross = [
                    u[1] * v[2] - u[2] * v[1],
                    u[2] * v[0] - u[0] * v[2],
                    u[0] * v[1] - u[1] * v[0],
                ];
                let n = mesh.normals[t[0] as usize];
                assert!(cross.iter().zip(n).map(|(a, b)| a * b).sum::<f32>() > 0.0);
                for (a, b) in [(a, b), (b, c), (c, a)] {
                    let mut edge = [key(a), key(b)];
                    edge.sort();
                    *edges.entry(edge).or_insert(0) += 1;
                }
            }
        }
        assert!(edges.values().all(|count| *count == 2));
        d.surface.width.0 *= 2.0;
        d.surface.height.0 *= 2.0;
        d.surface.curvature.0 *= 2.0;
        d.surface.thickness.0 *= 2.0;
        let scaled = Mesh::generate(&d).unwrap();
        for (a, b) in mesh.positions.iter().zip(scaled.positions) {
            for (a, b) in a.iter().zip(b) {
                assert!((a * 2.0 - b).abs() < 0.00001);
            }
        }
    }
}
#[test]
fn exported_glb_loads_with_matching_vertices_textures_and_clearcoat() {
    let mut d = Document::default();
    d.surface.gold = MetalFinish::GLAZED_SILVER;
    let b = Baked::generate(&d, Resolution::Draft).unwrap();
    let bytes = export::glb(&d, &b).unwrap();
    let loaded = gltf::Gltf::from_slice(&bytes).unwrap();
    let blob = loaded.blob.as_ref().unwrap();
    assert_eq!(loaded.document.images().count(), 4);
    assert_eq!(loaded.document.materials().count(), 2);
    let primitive = loaded
        .document
        .meshes()
        .next()
        .unwrap()
        .primitives()
        .next()
        .unwrap();
    let reader = primitive.reader(|_| Some(blob));
    let positions: Vec<_> = reader.read_positions().unwrap().collect();
    assert_eq!(positions, Mesh::generate(&d).unwrap().positions);
    let material = primitive.material();
    let coat = material.extension_value("KHR_materials_clearcoat").unwrap();
    assert_eq!(coat["clearcoatFactor"], 1.0);
    assert_eq!(coat["clearcoatRoughnessFactor"], 1.0);
    assert_eq!(coat["clearcoatTexture"]["index"], 3);
    assert_eq!(coat["clearcoatRoughnessTexture"]["index"], 3);
    for (image, expected) in loaded
        .document
        .images()
        .zip([&b.albedo, &b.normal, &b.orm, &b.coat])
    {
        let gltf::image::Source::View { view, .. } = image.source() else {
            panic!("external texture")
        };
        let png = &blob[view.offset()..view.offset() + view.length()];
        let decoded = image::load_from_memory(png).unwrap().to_rgba8();
        assert_eq!(decoded.width(), 128);
        assert_eq!(decoded.as_raw(), expected);
    }
}
#[test]
fn construction_controls_change_reusable_anatomy_at_valid_extremes() {
    for name in ["imperial-eagle", "durer-lion"] {
        for value in [0.5, 1.5] {
            let mut d = presets::preset(name).unwrap();
            let before = Artwork::compose(&d).unwrap().svg(&d.surface.palette);
            d.drawing.eagle.wing_span = Ratio(value);
            d.drawing.eagle.neck_length = Ratio(value);
            d.drawing.lion.foreleg_reach = Ratio(value);
            d.drawing.lion.spine_arch = Ratio(value);
            d.drawing.lion.tail_curl = Ratio(value);
            let art = Artwork::compose(&d).unwrap();
            let after = art.svg(&d.surface.palette);
            assert_ne!(before, after);
            assert!(!after.contains("NaN"));
            let b = Baked::generate(&d, Resolution::Draft).unwrap();
            assert!(b.height.iter().all(|v| v.is_finite()));
        }
    }
}

#[test]
fn lion_controls_change_anatomy_and_attribution_survives_compound_exports() {
    let mut d = Document {
        arms: ArmsDesign::lion(),
        ..Default::default()
    };
    let original = Baked::generate(&d, Resolution::Draft).unwrap();
    type Edit = fn(&mut LionDrawing);
    let edits: [Edit; 8] = [
        |l| l.body_width = Ratio(1.4),
        |l| l.spine_arch = Ratio(1.4),
        |l| l.head_size = Ratio(1.4),
        |l| l.foreleg_reach = Ratio(1.4),
        |l| l.hindleg_spread = Ratio(1.4),
        |l| l.mane_fullness = Ratio(1.4),
        |l| l.tail_curl = Ratio(1.4),
        |l| l.paw_size = Ratio(1.4),
    ];
    for edit in edits {
        let mut variant = d.clone();
        edit(&mut variant.drawing.lion);
        assert_ne!(
            original.flat,
            Baked::generate(&variant, Resolution::Draft).unwrap().flat
        );
    }
    d.drawing.painted_modeling = PaintedModeling::FLAT;
    d.arms = ArmsDesign::plain(Tincture::Or);
    d.arms.inescutcheon = Some(Box::new(ArmsDesign::lion()));
    let bake = Baked::generate(&d, Resolution::Draft).unwrap();
    let bundle = export::bundle(&d, &bake, Resolution::Draft).unwrap();
    let credit = bundle.iter().find(|f| f.name == "ATTRIBUTION.txt").unwrap();
    assert!(String::from_utf8_lossy(&credit.bytes).contains("Tom-L"));
    assert!(String::from_utf8_lossy(&credit.bytes).contains("Rinaldum"));
    let svg = Artwork::compose(&d).unwrap().svg(&d.surface.palette);
    assert!(svg.contains("CC BY-SA 3.0"));
    let glb = export::glb(&d, &bake).unwrap();
    assert!(String::from_utf8_lossy(&glb).contains("Rinaldum"));
    d.arms.inescutcheon = None;
    assert!(adventuresim_heraldry::provenance::attribution(&d).is_none());
    assert!(export::glb(&d, &bake).is_err());
}

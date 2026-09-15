//! Construction layout assertions retained from the browser authoring tests.
use super::profile_tests::endpoint;
use super::*;

#[test]
fn long_tiller_with_rearward_nut_has_ordered_stock_stations() {
    let mut recipe = endpoint("central-composite-arbalest", None);
    let Shape::Crossbow(p) = &mut recipe.components[0].shape else {
        panic!()
    };
    p.length = Metres::new(0.92).unwrap();
    p.nut_position = Metres::new(0.30).unwrap();
    let layout = crossbows::StockLayout::new(p);
    assert!(layout.rear.windows(2).all(|s| s[1].y > s[0].y));
    assert!(layout.fore.windows(2).all(|s| s[1].y > s[0].y));
    assert!(layout.rear.last().unwrap().y < layout.fore[0].y);
    assert!(generate_model(&recipe, Detail::Medium).is_ok());
}

#[test]
fn composite_laminations_meet_through_the_full_taper() {
    let recipe = endpoint("composite-recurve-bow-1544", None);
    let Shape::ArcheryBow(p) = &recipe.components[0].shape else {
        panic!()
    };
    let layers = archery::composite_layers(p).unwrap();
    for progress in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let scale = 1.0 + (p.tip_scale.get() - 1.0) * progress;
        let intervals = layers.map(|(_, width, center, _)| {
            [
                (center - width / 2.0) * scale,
                (center + width / 2.0) * scale,
            ]
        });
        assert!((intervals[1][1] - intervals[0][0]).abs() < 1e-12);
        assert!((intervals[0][1] - intervals[2][0]).abs() < 1e-12);
        assert!((intervals[1][0] + p.limb_depth.get() * scale / 2.0).abs() < 1e-12);
        assert!((intervals[2][1] - p.limb_depth.get() * scale / 2.0).abs() < 1e-12);
    }
}

#[test]
fn quiver_strap_anchors_follow_the_taper_with_material_overlap() {
    for extreme in [Some("min"), Some("max")] {
        let recipe = endpoint("arrow-quiver-1544", extreme);
        let Shape::ArrowQuiver(p) = &recipe.components[0].shape else {
            panic!()
        };
        let profile = archery::quiver_profile(p);
        let path = archery::quiver_strap(p, &profile);
        for anchor in [path[0], path[2]] {
            let overlap = archery::profile_radius(&profile, anchor[1]) - anchor[0];
            assert!(overlap > 0.0 && (overlap - p.strap_thickness.get() * 0.35).abs() < 1e-9);
        }
        assert!(generate_model(&recipe, Detail::Medium).is_ok());
    }
}

#[test]
fn bow_tip_loops_encircle_nocks_and_include_control_span_attachments() {
    for id in ["german-self-bow-1544", "composite-recurve-bow-1544"] {
        let recipe = endpoint(id, None);
        let Shape::ArcheryBow(p) = &recipe.components[0].shape else {
            panic!()
        };
        for upper in [false, true] {
            for detail in [Detail::Low, Detail::Medium, Detail::High] {
                let limb = archery::limb(p, upper, detail);
                let (attachment, points) = archery::tip_loop(p, &limb, detail);
                let tangent = normalize(sub(*limb.last().unwrap(), limb[limb.len() - 2]));
                let normal = normalize(cross([0.0, 0.0, 1.0], tangent));
                let binormal = normalize(cross(tangent, normal));
                assert!(dot(tangent, normal).abs() < 1e-9 && dot(tangent, binormal).abs() < 1e-9);
                assert!(points.iter().any(|q| magnitude(sub(*q, attachment)) < 1e-8));
                let range = |axis| {
                    let values: Vec<_> = points.iter().map(|q| dot(*q, axis)).collect();
                    values.iter().copied().fold(f64::NEG_INFINITY, f64::max)
                        - values.iter().copied().fold(f64::INFINITY, f64::min)
                };
                assert!(range(normal) / 2.0 > p.limb_depth.get() * p.tip_scale.get() * 0.54);
                assert!(range(binormal) / 2.0 > p.limb_width.get() * p.tip_scale.get() * 0.54);
            }
        }
    }
}

#[test]
fn crossbow_tip_loops_encircle_the_prod_and_include_string_attachments() {
    for id in [
        "german-cranequin-crossbow-1544",
        "central-composite-arbalest",
        "light-target-crossbow-comparative",
    ] {
        let recipe = endpoint(id, None);
        let Shape::Crossbow(p) = &recipe.components[0].shape else {
            panic!()
        };
        for right in [false, true] {
            for detail in [Detail::Low, Detail::Medium, Detail::High] {
                let prod = crossbows::prod(p, detail);
                let (attachment, points) = crossbows::tip_loop(p, &prod, right, detail);
                let tip = prod[if right { prod.len() - 1 } else { 0 }];
                assert!(
                    magnitude(sub(attachment, tip))
                        > p.prod_depth.get() * p.prod_tip_scale.get() / 2.0
                );
                assert!(points.iter().any(|q| magnitude(sub(*q, attachment)) < 1e-8));
            }
        }
    }
}

#[test]
fn firearm_stock_stations_retain_distinct_pistol_and_matchlock_profiles() {
    let catalog = crate::authoring::authoring_catalog();
    let firearms: Vec<_> = catalog["presets"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| serde_json::from_value::<Recipe>(v["definition"].clone()).ok())
        .flat_map(|r| r.components.into_iter().map(|c| c.shape))
        .filter_map(|s| {
            if let Shape::Firearm(p) = s {
                Some(p)
            } else {
                None
            }
        })
        .collect();
    let peck = firearms
        .iter()
        .find(|p| p.stock_style == FirearmStockStyle::Pistol)
        .unwrap();
    let musket = firearms
        .iter()
        .find(|p| p.stock_style != FirearmStockStyle::Pistol)
        .unwrap();
    let pistol = firearms::stations(peck);
    let matchlock = firearms::stations(musket);
    assert_eq!(pistol.len(), 8);
    assert_eq!(matchlock.len(), 9);
    assert!(pistol[1].width > pistol[0].width);
    assert!(pistol[1..6].windows(2).all(|s| s[1].top >= s[0].top));
    assert!(matchlock[1].width > musket.butt_width.get());
    assert!(matchlock[4].bottom < -musket.stock_depth.get() * 0.85);
    assert!((musket.length.get() - musket.barrel_length.get() - 0.387).abs() < 1e-12);
    assert_ne!(
        pistol
            .iter()
            .map(|s| (s.y / peck.length.get(), s.width / peck.butt_width.get()))
            .collect::<Vec<_>>(),
        matchlock
            .iter()
            .map(|s| (s.y / musket.length.get(), s.width / musket.butt_width.get()))
            .collect::<Vec<_>>()
    );
}

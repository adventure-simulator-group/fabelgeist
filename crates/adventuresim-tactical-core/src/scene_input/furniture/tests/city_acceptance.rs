use super::*;

#[test]
fn real_city_places_market_vendors_behind_its_wide_street_reservations() {
    let input = TacticalSceneInput::load(std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/tactical-scenes/massive-city.json"
    )))
    .unwrap();
    let scene = input.generate().unwrap();
    assert_vendor_stock_variety(&scene.furniture);
    let vendors: Vec<_> = scene
        .furniture
        .groups
        .iter()
        .filter(|group| group.kind == FurnitureGroupKind::Vendor)
        .collect();
    println!(
        "massive-city: {} groups, {} vendor groups",
        scene.furniture.groups.len(),
        vendors.len()
    );
    for kind in FurnitureKind::ALL {
        println!(
            "{kind:?}: {}",
            scene
                .furniture
                .instances
                .iter()
                .filter(|instance| instance.scene.key.kind == kind)
                .count()
        );
    }
    for vendor in &vendors {
        println!("vendor centre: {:?}", vendor.footprint.centre_metres);
    }
    report_terrain_delta("massive-city", &input, &scene.terrain);
    assert!(
        !vendors.is_empty(),
        "real market lost all vendors to wider perimeter roads"
    );
    for group in &scene.furniture.groups {
        assert!(
            scene
                .furniture
                .reserved_routes
                .iter()
                .all(|route| !route.intersects(group.footprint)),
            "accepted group obstructs an actual street or approach"
        );
    }
    for vendor in vendors {
        let FurnitureAnchor::Market { patch_index } = vendor.anchor else {
            panic!("vendor is not anchored to market");
        };
        assert!(
            vendor
                .footprint
                .corners()
                .into_iter()
                .all(|point| input.streets[patch_index as usize].contains(point))
        );
    }
}

pub(super) fn report_terrain_delta(name: &str, input: &TacticalSceneInput, terrain: &SceneTerrain) {
    let spacing_metres = 0.5;
    let mut max_abs = 0.0_f32;
    let mut max_positive = 0.0_f32;
    let mut max_street_abs = 0.0_f32;
    for z in 0..=(terrain.depth() / spacing_metres) as usize {
        for x in 0..=(terrain.width() / spacing_metres) as usize {
            let point = Vec2::new(
                x as f32 * spacing_metres - terrain.width() * 0.5,
                z as f32 * spacing_metres - terrain.depth() * 0.5,
            );
            let (Some(fine), Some(coarse)) =
                (terrain.height_at(point), terrain.coarse_height_at(point))
            else {
                continue;
            };
            let delta = fine - coarse;
            max_abs = max_abs.max(delta.abs());
            max_positive = max_positive.max(delta);
            if input.streets.iter().any(|patch| patch.contains(point)) {
                max_street_abs = max_street_abs.max(delta.abs());
            }
        }
    }
    println!(
        "{name} terrain fine/coarse at 0.5m: abs={max_abs:.5}m positive={max_positive:.5}m streets_abs={max_street_abs:.5}m"
    );
}

pub(super) fn assert_vendor_stock_variety(layout: &FurnitureLayout) {
    for stock in [FurnitureKind::Barrel, FurnitureKind::CargoStack] {
        let count = layout
            .groups
            .iter()
            .filter(|group| {
                group.kind == FurnitureGroupKind::Vendor
                    && layout.instances.iter().any(|instance| {
                        instance.scene.group_id == group.id && instance.scene.key.kind == stock
                    })
            })
            .count();
        println!("vendor groups with {stock:?}: {count}");
        assert!(
            count >= 2,
            "market must retain repeated examples of both side-stock compositions"
        );
    }
}

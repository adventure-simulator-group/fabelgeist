use super::*;
use crate::{BuildingProgram, generate, settlement_archetype};

#[test]
fn brands_are_stable_per_lot_and_only_public_shops_receive_them() {
    let first = ShopName::for_establishment(EstablishmentId(15), BuildingUse::Inn).unwrap();
    assert_eq!(first.text(), "Ursula Klein’s Tavern");
    assert_eq!(
        first,
        ShopName::for_establishment(EstablishmentId(15), BuildingUse::Inn).unwrap()
    );
    assert_ne!(
        first,
        ShopName::for_establishment(EstablishmentId(16), BuildingUse::Inn).unwrap()
    );
    for usage in [
        BuildingUse::Dwelling,
        BuildingUse::Barn,
        BuildingUse::Cathedral,
        BuildingUse::MarketHall,
    ] {
        assert!(ShopSign::for_establishment(EstablishmentId(15), usage).is_none());
    }
}

#[test]
fn storefront_sites_clear_entrances_and_keep_both_mounts_above_pedestrians() {
    for usage in [
        BuildingUse::Inn,
        BuildingUse::GeneralShop,
        BuildingUse::Smithy,
        BuildingUse::Bakehouse,
        BuildingUse::Stable,
    ] {
        let program =
            BuildingProgram::validated_settlement(settlement_archetype(usage), usage, 42, None)
                .unwrap();
        let plan = generate(&program).unwrap();
        let site =
            SignSite::for_plan(&plan).unwrap_or_else(|| panic!("{usage:?} has no sign site"));
        assert!(site.mounting.is_supported(&plan, site.outward));
        let mut floating = site.mounting;
        floating.contact += site.outward * 0.1;
        assert!(
            !floating.is_supported(&plan, site.outward),
            "partial or absent plate support must fail"
        );
        let support = plan
            .resolved_geometry
            .solids
            .iter()
            .find(|solid| solid.id == site.mounting.support)
            .unwrap();
        let mut partial = site.mounting;
        partial.contact.y = support.centre.y + super::site::solid_extent(support).y;
        assert!(
            !partial.is_supported(&plan, site.outward),
            "a plate half hanging off its support must fail"
        );
        for mount in [SignMount::Wall, SignMount::Projecting] {
            assert!(
                site.supports(&plan, mount),
                "{usage:?} {mount:?} intersects the building"
            );
            let board = site.board(mount);
            assert!(board.centre.y - board.size.y * 0.5 >= SIGN_PEDESTRIAN_CLEARANCE_METRES);
            assert!(
                (board.centre - site.attachment).dot(site.outward)
                    + if mount == SignMount::Projecting {
                        board.size.x * 0.5
                    } else {
                        0.03
                    }
                    <= SIGN_MAX_PROJECTION_METRES
            );
        }
    }
}

#[cfg(feature = "sign-render")]
#[test]
fn both_fonts_paint_long_german_names_without_clipping_or_missing_glyphs() {
    for font in [SignFont::GrenzeGotisch, SignFont::UnifrakturCook] {
        let sign = ShopSign {
            name: ShopName {
                proprietor: "Margarete Großmüller’s".to_owned(),
                trade: "Apothecary".to_owned(),
            },
            mount: SignMount::Projecting,
            font,
            finish: SignFinish::PalePaint,
        };
        let texture = SignTexture::paint(&sign, Vec2::new(1.35, 0.62));
        assert_eq!(texture.lines.len(), 2);
        let paint = sign.finish.colors().0;
        let mut ink_pixels = 0;
        for y in 30..texture.height - 30 {
            for x in 30..texture.width - 30 {
                let offset = ((y * texture.width + x) * 4) as usize;
                if texture.rgba[offset..offset + 3] != paint {
                    assert!(x >= 50 && x < texture.width - 50);
                    assert!(y >= 50 && y < texture.height - 50);
                    ink_pixels += 1;
                }
            }
        }
        assert!(ink_pixels > 1000, "{font:?} lettering disappeared");
        assert!(
            texture
                .rgba
                .as_chunks::<4>()
                .0
                .iter()
                .all(|pixel| pixel[3] == 255)
        );
    }
}

use adventuresim_weapon_model::*;
#[test]
fn procedural_icons_obey_focus_orientation_and_clipping_contracts() {
    for id in MELEE_CATALOG_IDS {
        let design = default_design(id).unwrap();
        let icon = generate_icon(
            &design,
            WeaponIconSpec {
                size: 96,
                supersampling: 4,
            },
        )
        .unwrap_or_else(|error| panic!("{id}: {error}"));
        let expected = icon_layout(&design);
        let expected_from_catalog_family = if matches!(
            *id,
            "arming_sword"
                | "baselard"
                | "bauernwehr"
                | "falchion"
                | "katzbalger"
                | "knife"
                | "kriegsmesser"
                | "longsword"
                | "messer"
                | "misericorde"
                | "rapier"
                | "rondel_dagger"
                | "utility_knife"
                | "zweihander"
        ) {
            WeaponIconLayout::HiltFocus
        } else {
            WeaponIconLayout::HeadFocus
        };
        assert_eq!(expected, expected_from_catalog_family, "{id} family");
        assert_eq!(icon.layout, expected, "{id}");
        assert_eq!(icon.framing_anchor, [0.5, 0.5], "{id} semantic anchor");
        assert!(
            (1.0..=2.0).contains(&icon.head_zoom),
            "{id} invalid head zoom {}",
            icon.head_zoom
        );
        assert_eq!(icon.rgba.len(), 96 * 96 * 4, "{id}");
        assert!(
            icon.rgba
                .as_chunks::<4>()
                .0
                .iter()
                .all(|pixel| pixel[3] == 255),
            "{id} must have an opaque background"
        );
        let colors = icon
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .map(|pixel| &pixel[..3])
            .collect::<std::collections::HashSet<_>>();
        assert!(
            colors.len() > 16,
            "{id} must preserve lighting and material variation"
        );
        let occupied = icon
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|pixel| pixel[..3].iter().any(|value| *value > 0))
            .count();
        assert!(occupied > 96, "{id} icon is effectively empty");
        assert!(
            occupied < 96 * 96 / 2,
            "{id} icon consumes the whole square"
        );
        match expected {
            WeaponIconLayout::HiltFocus => {
                assert!(icon.focus_bounds.min[0] >= 0.01, "{id} hilt clipped left");
                assert!(icon.focus_bounds.min[1] >= 0.01, "{id} hilt clipped top");
                assert!(icon.focus_bounds.max[0] <= 0.99, "{id} hilt clipped right");
                assert!(icon.focus_bounds.max[1] <= 0.99, "{id} hilt clipped bottom");
                if *id != "utility_knife" {
                    assert!(
                        icon.occupied_bounds.max[1] >= 0.97,
                        "{id} blade did not clip bottom"
                    );
                }
                assert!(
                    icon.occupied_bounds.min[0] < 0.25,
                    "{id} blade does not extend toward lower-left"
                );
            }
            WeaponIconLayout::HeadFocus => {
                assert!(icon.focus_bounds.min[0] >= 0.01, "{id} head clipped left");
                assert!(icon.focus_bounds.min[1] >= 0.01, "{id} head clipped top");
                assert!(icon.focus_bounds.max[0] <= 0.99, "{id} head clipped right");
                assert!(icon.focus_bounds.max[1] <= 0.99, "{id} head clipped bottom");
                assert!(
                    icon.occupied_bounds.max[1] >= 0.97,
                    "{id} shaft did not clip bottom"
                );
                assert!(
                    icon.occupied_bounds.max[0] >= 0.97,
                    "{id} shaft does not extend toward lower-right"
                );
                if icon.head_zoom < 1.99 {
                    assert!(
                        icon.focus_bounds.min[0] <= 0.15 + 1.0 / 96.0
                            || icon.focus_bounds.min[1] <= 0.15 + 1.0 / 96.0,
                        "{id} head did not approach the inset corner: {:?}",
                        icon.focus_bounds
                    );
                }
                if matches!(*id, "war_hammer" | "walking_staff") {
                    assert!(
                        icon.head_zoom > 1.5,
                        "{id} exception did not materially enlarge the head"
                    );
                }
            }
        }
        let repeated = generate_icon(
            &design,
            WeaponIconSpec {
                size: 96,
                supersampling: 4,
            },
        )
        .unwrap();
        assert_eq!(icon.rgba, repeated.rgba, "{id} icon is not deterministic");
        let png = icon.encode_png().unwrap();
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n", "{id}");
        assert_eq!(png[25], 6, "{id}: color portrait PNG must carry RGBA");
    }
}

#[test]
fn procedural_holder_icons_are_fitted_mirrored_and_deterministic() {
    let spec = WeaponIconSpec {
        size: 96,
        supersampling: 4,
    };
    let sword = default_design("longsword").unwrap();
    let sheath = default_holder_design(&sword).unwrap();
    let icon = generate_holder_icon(&sheath, spec).unwrap();
    assert_eq!(icon.layout, WeaponIconLayout::HiltFocus);
    assert!(icon.mirrored);
    assert_eq!(icon.framing_anchor, [0.24, 0.24]);
    assert!(icon.focus_bounds.min[0] >= 0.01);
    assert!(icon.focus_bounds.min[1] >= 0.01);
    assert!(icon.focus_bounds.max[0] <= 0.99);
    assert!(icon.focus_bounds.max[1] <= 0.99);
    assert!(
        icon.occupied_bounds.max[0] >= 0.97 && icon.occupied_bounds.max[1] >= 0.97,
        "scabbard body must exit toward the lower-right: {:?}",
        icon.occupied_bounds
    );
    assert_eq!(
        icon.rgba,
        generate_holder_icon(&sheath, spec).unwrap().rgba,
        "holder icon is not deterministic"
    );

    let hammer = default_design("war_hammer").unwrap();
    let haft_loop = default_holder_design(&hammer).unwrap();
    let loop_icon = generate_holder_icon(&haft_loop, spec).unwrap();
    assert_eq!(loop_icon.layout, WeaponIconLayout::HeadFocus);
    assert!(!loop_icon.mirrored);
    assert_eq!(loop_icon.framing_anchor, [0.5, 0.5]);
    assert!(loop_icon.occupied_bounds.min[0] >= 0.01);
    assert!(loop_icon.occupied_bounds.min[1] >= 0.01);
    assert!(loop_icon.occupied_bounds.max[0] <= 0.99);
    assert!(loop_icon.occupied_bounds.max[1] <= 0.99);
    assert_ne!(icon.rgba, loop_icon.rgba);

    for id in MELEE_CATALOG_IDS {
        let weapon = default_design(id).unwrap();
        let Some(holder) = default_holder_design(&weapon) else {
            continue;
        };
        generate_holder_icon(
            &holder,
            WeaponIconSpec {
                size: 32,
                supersampling: 2,
            },
        )
        .unwrap_or_else(|error| panic!("{id} holder icon: {error}"));
    }
}

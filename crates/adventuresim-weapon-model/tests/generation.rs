use adventuresim_weapon_model::recipe::{Metres, Ratio, Shape};
use adventuresim_weapon_model::*;
use std::collections::HashMap;

fn signed_volume(positions: &[[f32; 3]], indices: &[u32]) -> f32 {
    indices
        .as_chunks::<3>()
        .0
        .iter()
        .map(|triangle| {
            let [a, b, c] = [
                positions[triangle[0] as usize],
                positions[triangle[1] as usize],
                positions[triangle[2] as usize],
            ];
            (a[0] * (b[1] * c[2] - b[2] * c[1]) - a[1] * (b[0] * c[2] - b[2] * c[0])
                + a[2] * (b[0] * c[1] - b[1] * c[0]))
                / 6.0
        })
        .sum()
}

fn assert_closed_and_outward(design: &WeaponDesign) {
    let generated = generate(design).expect("generated weapon");
    assert_parts_closed_and_outward(&generated.parts);
}

fn assert_parts_closed_and_outward(parts: &[adventuresim_weapon_model::MeshPart]) {
    for part in parts {
        assert_eq!(
            part.positions.len(),
            part.normals.len(),
            "{}",
            part.component_id
        );
        assert_eq!(part.indices.len() % 3, 0);
        assert!(
            signed_volume(&part.positions, &part.indices) > 1e-10,
            "{} winding",
            part.component_id
        );
        assert!(
            part.positions
                .iter()
                .flatten()
                .all(|value| value.is_finite())
        );
        assert!(part.normals.iter().flatten().all(|value| value.is_finite()));
        assert!(part.normals.iter().all(|normal| {
            ((normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt() - 1.0)
                .abs()
                < 1e-4
        }));
        let vertex_key = |index: u32| part.positions[index as usize].map(f32::to_bits);
        let mut edges = HashMap::<([u32; 3], [u32; 3]), u8>::new();
        for triangle in part.indices.as_chunks::<3>().0 {
            for [a, b] in [
                [triangle[0], triangle[1]],
                [triangle[1], triangle[2]],
                [triangle[2], triangle[0]],
            ] {
                let a = vertex_key(a);
                let b = vertex_key(b);
                *edges
                    .entry(if a < b { (a, b) } else { (b, a) })
                    .or_default() += 1;
            }
        }
        assert!(
            edges.values().all(|incidence| *incidence == 2),
            "{} is not a closed indexed solid",
            part.component_id
        );
        for triangle in part.indices.as_chunks::<3>().0 {
            let [a, b, c] = [
                part.positions[triangle[0] as usize],
                part.positions[triangle[1] as usize],
                part.positions[triangle[2] as usize],
            ];
            let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let face = [
                ab[1] * ac[2] - ab[2] * ac[1],
                ab[2] * ac[0] - ab[0] * ac[2],
                ab[0] * ac[1] - ab[1] * ac[0],
            ];
            assert!(
                face.iter().map(|n| n * n).sum::<f32>() > 0.0,
                "{} degenerate face",
                part.component_id
            );
        }
    }
}

#[test]
fn every_non_polearm_has_a_fitted_closed_holder() {
    let mut blade_sheaths = 0;
    let mut haft_loops = 0;
    let mut polearms = 0;
    for id in MELEE_CATALOG_IDS {
        let design = default_design(id).unwrap();
        let generated = default_holder_design(&design);
        assert_eq!(
            generated.as_ref().map(|holder| holder.kind),
            recommended_holder(id)
        );
        match generated {
            Some(holder_design) => {
                let holder = generate_holder(&holder_design).unwrap();
                assert_eq!(holder.design_hash, holder_design_hash(&holder_design));
                assert!(!holder.parts.is_empty(), "{id}");
                assert_parts_closed_and_outward(&holder.parts);
                assert!(holder.bounds.max[1] > holder.bounds.min[1], "{id}");
                match holder.kind {
                    WeaponHolderKind::BladeSheath => blade_sheaths += 1,
                    WeaponHolderKind::HaftLoop => haft_loops += 1,
                }
            }
            None => polearms += 1,
        }
    }
    assert_eq!((blade_sheaths, haft_loops, polearms), (14, 4, 5));
}

#[test]
fn sheath_tracks_blade_dimensions_and_leaves_the_hilt_exposed() {
    let design = default_design("longsword").unwrap();
    let weapon = generate(&design).unwrap();
    let holder = generate_holder(&default_holder_design(&design).unwrap()).unwrap();
    assert_eq!(holder.kind, WeaponHolderKind::BladeSheath);
    let blade = weapon
        .parts
        .iter()
        .find(|part| part.component_id == "blade")
        .unwrap();
    assert!(holder.bounds.min[0] < blade.bounds.min[0]);
    assert!(holder.bounds.max[0] > blade.bounds.max[0]);
    assert!(holder.bounds.min[2] < blade.bounds.min[2]);
    assert!(holder.bounds.max[2] > blade.bounds.max[2]);
    assert!(holder.bounds.min[1] > holder.grip[1]);

    let mut longer = design.clone();
    let blade = longer
        .recipe
        .components
        .iter_mut()
        .find(|part| part.id.as_deref() == Some("blade"))
        .unwrap();
    match &mut blade.shape {
        Shape::LoftedBlade(spec) => spec.length = Metres::new(spec.length.get() + 0.12).unwrap(),
        _ => panic!("longsword blade changed shape"),
    }
    let longer_holder = generate_holder(&default_holder_design(&longer).unwrap()).unwrap();
    assert!(longer_holder.bounds.max[1] > holder.bounds.max[1] + 0.10);
}

#[test]
fn haft_loop_tracks_the_grip_instead_of_the_head() {
    let design = default_design("flanged_mace").unwrap();
    let holder = generate_holder(&default_holder_design(&design).unwrap()).unwrap();
    assert_eq!(holder.kind, WeaponHolderKind::HaftLoop);
    assert!(holder.bounds.min[1] < holder.grip[1]);
    assert!(holder.bounds.max[1] > holder.grip[1]);

    let mut wider = design.clone();
    let grip = wider
        .recipe
        .components
        .iter_mut()
        .find(|part| part.role == Some(ComponentRole::Grip))
        .unwrap();
    match &mut grip.shape {
        Shape::Shaft(spec) => spec.radius = Metres::new(spec.radius.get() + 0.002).unwrap(),
        _ => panic!("mace grip changed shape"),
    }
    let wider_holder = generate_holder(&default_holder_design(&wider).unwrap()).unwrap();
    assert!(wider_holder.bounds.max[0] > holder.bounds.max[0] + 0.0015);
}

#[test]
fn holder_templates_encode_independent_per_instance_parameters() {
    let weapon = default_design("longsword").unwrap();
    let first = default_holder_design(&weapon).unwrap();
    validate_holder(&first).unwrap();
    let bytes = encode_holder(&first).unwrap();
    assert_eq!(decode_holder(&bytes).unwrap(), first);

    let mut second = first.clone();
    second.clearance = Millimeters(first.clearance.0 + 3);
    second.chape_length = Millimeters(first.chape_length.0 + 8);
    assert_ne!(holder_design_hash(&first), holder_design_hash(&second));
    assert_ne!(
        encode_holder(&first).unwrap(),
        encode_holder(&second).unwrap()
    );
    let first_mesh = generate_holder(&first).unwrap();
    let second_mesh = generate_holder(&second).unwrap();
    assert!(second_mesh.bounds.max[0] > first_mesh.bounds.max[0]);

    let mut invalid = first;
    invalid.clearance = Millimeters(0);
    assert!(validate_holder(&invalid).is_err());
    assert!(encode_holder(&invalid).is_err());
    assert!(default_holder_design(&default_design("halberd").unwrap()).is_none());

    let mut hostile = default_holder_design(&weapon).unwrap();
    hostile.hanger_height = Millimeters(u32::MAX);
    let result = std::panic::catch_unwind(|| generate_holder(&hostile));
    assert!(matches!(result, Ok(Err(_))));
}

#[test]
fn holder_parameter_extremes_stay_valid_closed_and_distinct() {
    for id in MELEE_CATALOG_IDS {
        let weapon = default_design(id).unwrap();
        let Some(default) = default_holder_design(&weapon) else {
            continue;
        };
        let mut minimum = default.clone();
        minimum.clearance = Millimeters(2);
        minimum.throat_length = Millimeters(4);
        minimum.chape_length = Millimeters(6);
        minimum.loop_position = Permille(0);
        minimum.loop_bar_radius = Millimeters(2);
        minimum.hanger_width = Millimeters(20);
        minimum.hanger_height = Millimeters(30);
        let mut maximum = default;
        maximum.clearance = Millimeters(20);
        maximum.throat_length = Millimeters(40);
        maximum.chape_length = Millimeters(60);
        maximum.loop_position = Permille(1_000);
        maximum.loop_bar_radius = Millimeters(12);
        maximum.hanger_width = Millimeters(120);
        maximum.hanger_height = Millimeters(180);
        for design in [&minimum, &maximum] {
            validate_holder(design).unwrap_or_else(|errors| panic!("{id}: {errors:?}"));
            let holder = generate_holder(design).unwrap();
            assert_parts_closed_and_outward(&holder.parts);
            assert!(holder.derived.mass_kg.is_finite() && holder.derived.mass_kg > 0.0);
            assert!(holder.derived.length_m.is_finite() && holder.derived.length_m > 0.0);
        }
        assert_ne!(
            holder_design_hash(&minimum),
            holder_design_hash(&maximum),
            "{id}"
        );
    }
}

#[test]
fn every_existing_melee_catalog_id_has_a_valid_deterministic_solid() {
    assert_eq!(MELEE_CATALOG_IDS.len(), 23);
    for id in MELEE_CATALOG_IDS {
        let design = default_design(id).unwrap_or_else(|| panic!("missing {id}"));
        assert_eq!(design.catalog_id, *id);
        validate(&design).unwrap_or_else(|errors| panic!("{id}: {errors:?}"));
        assert_closed_and_outward(&design);
        let generated = generate(&design).unwrap();
        assert!(generated.derived.mass_kg > 0.0, "{id}");
        assert!(generated.derived.length_m > 0.0, "{id}");
        assert!(generated.derived.grip_to_tip_m > 0.0, "{id}");
        assert!(
            generated
                .anchors
                .iter()
                .any(|anchor| anchor.name == "weapon.grip")
        );
        assert!(
            generated
                .anchors
                .iter()
                .any(|anchor| anchor.name == "weapon.tip")
        );
        assert_eq!(generated.design_hash, design_hash(&design));
    }
    assert!(default_design("bow").is_none());
}

#[test]
fn every_melee_catalog_entry_has_distinct_geometry() {
    let mut geometries = std::collections::HashMap::new();
    for id in MELEE_CATALOG_IDS {
        let mut design = default_design(id).unwrap();
        design.catalog_id = "geometry".into();
        let geometry = encode(&design).unwrap();
        assert_eq!(
            geometries.insert(geometry, *id),
            None,
            "{id} reuses another catalog geometry"
        );
    }

    for id in ["baselard", "knife", "utility_knife", "misericorde"] {
        let design = default_design(id).unwrap();
        assert!(
            design.recipe.components.iter().all(|component| {
                !component
                    .id
                    .as_deref()
                    .is_some_and(|id| id.contains("rondel"))
            }),
            "{id} retained rondel furniture"
        );
    }
    let zweihander = default_design("zweihander").unwrap();
    let blade = zweihander
        .recipe
        .components
        .iter()
        .find(|component| component.id.as_deref() == Some("blade"))
        .unwrap();
    assert!(matches!(&blade.shape, Shape::LoftedBlade(spec) if spec.ricasso.get() >= 0.2));
    assert!(
        zweihander
            .recipe
            .components
            .iter()
            .any(|component| component.id.as_deref() == Some("parrying-lugs"))
    );
}

#[test]
fn json_transport_and_design_hash_are_canonical_and_sensitive() {
    let design = default_design("longsword").unwrap();
    let first = encode(&design).unwrap();
    let second = encode(&design).unwrap();
    assert_eq!(first, second);
    assert_eq!(decode(&first).unwrap(), design);
    assert_eq!(design_hash(&design), design_hash(&decode(&first).unwrap()));
    let mut changed = design.clone();
    match &mut changed.recipe.components.last_mut().unwrap().shape {
        Shape::LoftedBlade(blade) => {
            blade.length = Metres::new(blade.length.get() + 0.001).unwrap()
        }
        _ => panic!("expected blade"),
    }
    assert_ne!(design_hash(&design), design_hash(&changed));
    assert_ne!(encode(&design).unwrap(), encode(&changed).unwrap());
}

#[test]
fn recipe_derivation_is_deterministic_and_mesh_independent() {
    for id in PRESET_IDS {
        let design = preset_design(id).unwrap();
        let first = derive_properties(&design).unwrap();
        assert_eq!(first, derive_properties(&design).unwrap(), "{id}");
        assert!(first.mass_kg > 0.0, "{id}");
        assert!(first.length_m > 0.0, "{id}");
        assert!(first.grip_to_tip_m > 0.0, "{id}");
        assert!(first.center_of_mass_from_grip_m.is_finite(), "{id}");
        assert!(
            first.moment_of_inertia_kg_m2.is_finite() && first.moment_of_inertia_kg_m2 > 0.0,
            "{id}"
        );
        assert!(first.balance.is_finite() && first.balance > 0.0, "{id}");
    }
}

#[test]
fn realistic_longsword_pommel_shifts_mass_and_handling_without_dominating_weight() {
    let baseline = default_design("longsword").unwrap();
    let baseline_properties = derive_properties(&baseline).unwrap();
    assert!((1.0..3.0).contains(&baseline_properties.mass_kg));
    assert!(baseline_properties.center_of_mass_from_grip_m > 0.0);

    let mut heavier = baseline.clone();
    let Shape::Pommel(pommel) = &mut heavier.recipe.components[1].shape else {
        panic!("longsword pommel changed shape")
    };
    for point in pommel.profile.as_mut().unwrap() {
        point[1] = Metres::new(point[1].get() * 1.08).unwrap();
    }
    let heavier_properties = derive_properties(&heavier).unwrap();
    assert!(heavier_properties.mass_kg > baseline_properties.mass_kg);
    assert!(
        heavier_properties.center_of_mass_from_grip_m
            < baseline_properties.center_of_mass_from_grip_m
    );
    assert!(heavier_properties.balance < baseline_properties.balance);
}

#[test]
fn curved_bill_and_gothic_flange_profiles_are_dense_and_parameter_sensitive() {
    let bill = preset_design("hooked-bill").unwrap();
    let generated = generate(&bill).unwrap();
    let hook = generated
        .parts
        .iter()
        .find(|part| part.component_id == "continuous bill and hook")
        .unwrap();
    assert!(
        hook.positions.len() >= 60,
        "continuous hook sampling regressed"
    );
    assert!(hook.bounds.max[0] - hook.bounds.min[0] > 0.18);
    let mut changed = bill.clone();
    let Shape::Bill(spec) = &mut changed
        .recipe
        .components
        .iter_mut()
        .find(|component| matches!(component.shape, Shape::Bill(_)))
        .unwrap()
        .shape
    else {
        panic!("bill")
    };
    spec.hook_curvature = Some(Ratio::new(spec.hook_curvature.unwrap().get() + 0.09).unwrap());
    assert_ne!(
        generate(&bill).unwrap().parts,
        generate(&changed).unwrap().parts
    );

    let mace = preset_design("gothic-flanged-mace").unwrap();
    let mut changed = mace.clone();
    let Shape::Mace(spec) = &mut changed.recipe.components.last_mut().unwrap().shape else {
        panic!("gothic mace")
    };
    spec.concavity = Some(Ratio::new(spec.concavity.unwrap().get() - 0.15).unwrap());
    assert_ne!(
        generate(&mace).unwrap().parts,
        generate(&changed).unwrap().parts
    );
}

#[test]
fn katzbalger_fan_has_sampled_mushroom_dome_and_narrow_neck() {
    let generated = generate(&preset_design("katzbalger").unwrap()).unwrap();
    let fan = generated
        .parts
        .iter()
        .find(|part| part.component_id == "pommel")
        .unwrap();
    assert!(
        fan.positions.len() >= 100,
        "fan silhouette sampling regressed"
    );
    assert!(((fan.bounds.max[0] - fan.bounds.min[0]) - 0.055).abs() < 0.001);
    assert!(((fan.bounds.max[1] - fan.bounds.min[1]) - 0.031).abs() < 0.001);
    let base_y = fan.bounds.min[1];
    let neck = fan
        .positions
        .iter()
        .filter(|point| point[1] - base_y < 0.001)
        .map(|point| point[0].abs())
        .fold(0.0_f32, f32::max);
    assert!(neck <= 0.011, "fan neck widened into a diamond: {neck}");
    assert!(
        fan.positions
            .iter()
            .filter(|point| point[1] - base_y > 0.018)
            .count()
            >= 48,
        "domed cap undersampled"
    );
}

#[test]
fn tapered_cylinders_affect_mesh_and_derived_mass() {
    let tapered = default_design("walking_staff").unwrap();
    let tapered_generated = generate(&tapered).unwrap();
    let tapered_mass = derive_properties(&tapered).unwrap().mass_kg;

    let mut straight = tapered.clone();
    let Shape::Shaft(shaft) = &mut straight.recipe.components[0].shape else {
        panic!()
    };
    shaft.bottom_scale = Some(Ratio::new(1.0).unwrap());
    shaft.top_scale = Some(Ratio::new(1.0).unwrap());
    validate(&straight).unwrap();
    let straight_generated = generate(&straight).unwrap();
    let straight_mass = derive_properties(&straight).unwrap().mass_kg;

    let tapered_part = &tapered_generated.parts[0];
    let straight_part = &straight_generated.parts[0];
    let tapered_bottom = tapered_part
        .positions
        .iter()
        .filter(|point| (point[1] - tapered_part.bounds.min[1]).abs() < 1e-6)
        .map(|point| point[0].hypot(point[2]))
        .fold(0.0_f32, f32::max);
    let tapered_top = tapered_part
        .positions
        .iter()
        .filter(|point| (point[1] - tapered_part.bounds.max[1]).abs() < 1e-6)
        .map(|point| point[0].hypot(point[2]))
        .fold(0.0_f32, f32::max);
    assert!(
        tapered_top > tapered_bottom,
        "shaft taper was not generated"
    );
    assert!(straight_mass > tapered_mass, "frustum volume was ignored");
    assert!(
        straight_part.bounds.max[0] > tapered_part.bounds.max[0],
        "scale did not affect mesh bounds"
    );
}

#[test]
fn shape_aware_normals_smooth_walls_but_split_caps_and_blade_creases() {
    fn groups(part: &adventuresim_weapon_model::MeshPart) -> HashMap<[u32; 3], Vec<[f32; 3]>> {
        let mut groups = HashMap::<[u32; 3], Vec<[f32; 3]>>::new();
        for (position, normal) in part.positions.iter().zip(&part.normals) {
            groups
                .entry(position.map(f32::to_bits))
                .or_default()
                .push(*normal);
        }
        groups
    }
    fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
        a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
    }

    let staff = generate(&default_design("walking_staff").unwrap()).unwrap();
    let shaft = &staff.parts[0];
    let shaft_groups = groups(shaft);
    assert!(
        shaft_groups.values().any(|normals| {
            normals.iter().any(|normal| normal[1].abs() > 0.99)
                && normals.iter().any(|normal| normal[1].abs() < 0.2)
        }),
        "cylinder cap seam was smoothed"
    );
    for (position, normal) in shaft.positions.iter().zip(&shaft.normals) {
        if (position[1] - shaft.bounds.min[1]).abs() > 1e-5
            && (position[1] - shaft.bounds.max[1]).abs() > 1e-5
        {
            let radial = [position[0], 0.0, position[2]];
            let magnitude = radial[0].hypot(radial[2]);
            assert!(magnitude > 0.0);
            assert!(
                dot(*normal, [radial[0] / magnitude, 0.0, radial[2] / magnitude]) > 0.9,
                "curved shaft wall was not radially smooth: {normal:?}"
            );
        }
    }

    let bill = generate(&preset_design("hooked-bill").unwrap()).unwrap();
    let bill = bill
        .parts
        .iter()
        .find(|part| part.component_id == "continuous bill and hook")
        .unwrap();
    assert!(
        groups(bill).values().any(|normals| {
            normals
                .iter()
                .enumerate()
                .any(|(index, a)| normals[index + 1..].iter().any(|b| dot(*a, *b) < 0.25))
        }),
        "blade front/side crease was globally averaged"
    );
}
#[test]
fn mace_assemblies_seat_the_head_in_the_shaft_and_sleeve() {
    for id in ["flanged-mace", "gothic-flanged-mace"] {
        let generated = generate(&preset_design(id).unwrap()).unwrap();
        let head = generated
            .parts
            .iter()
            .find(|p| p.component_id == "longitudinal flanged head")
            .unwrap();
        let shaft = generated
            .parts
            .iter()
            .find(|p| p.component_id == "shaft")
            .unwrap();
        let sleeve = generated
            .parts
            .iter()
            .find(|p| p.component_id == "narrow head sleeve")
            .unwrap();
        assert!(head.bounds.min[1] < shaft.bounds.max[1]);
        assert!((head.bounds.min[1] - sleeve.bounds.max[1]).abs() < 1e-6);
    }
}

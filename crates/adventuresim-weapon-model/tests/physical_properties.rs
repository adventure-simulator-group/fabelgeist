//! Physical regressions for recipe authority and the museum-grounded 1544 audit.
use adventuresim_weapon_model::*;

fn mesh_mass(weapon: &GeneratedWeapon) -> f64 {
    weapon
        .parts
        .iter()
        .map(|part| {
            part.indices
                .as_chunks::<3>()
                .0
                .iter()
                .map(|triangle| {
                    let [a, b, c] =
                        triangle.map(|index| part.positions[index as usize].map(f64::from));
                    (a[0] * (b[1] * c[2] - b[2] * c[1]) - a[1] * (b[0] * c[2] - b[2] * c[0])
                        + a[2] * (b[0] * c[1] - b[1] * c[0]))
                        / 6.0
                })
                .sum::<f64>()
                * f64::from(part.material.density_kg_m3())
        })
        .sum()
}

#[test]
fn every_recipe_conserves_the_material_volume_visible_in_its_mesh() {
    for design in PRESET_IDS
        .iter()
        .map(|id| preset_design(id).unwrap())
        .chain(
            MELEE_CATALOG_IDS
                .iter()
                .map(|id| default_design(id).unwrap()),
        )
    {
        let mesh = generate(&design).unwrap();
        assert!(
            (mesh_mass(&mesh) - f64::from(mesh.derived.mass_kg)).abs() < 0.0001,
            "{}",
            design.catalog_id
        );
        assert!((mesh.bounds.max[1] - mesh.bounds.min[1] - mesh.derived.length_m).abs() < 0.00001);
    }
}

#[test]
fn a_regular_polygon_rod_matches_analytic_mass_centroid_and_inertia() {
    let design = WeaponDesign {
        catalog_id: "rod-fixture".into(),
        components: vec![ComponentDesign {
            id: "grip".into(),
            role: ComponentRole::Grip,
            attachment: Attachment::Root,
            offset: OffsetMm::default(),
            material: MaterialClass::Wood,
            shape: ComponentShape::Cylinder(CylinderSpec {
                length: Millimeters(1000),
                radius: Millimeters(10),
                bottom_scale: Permille(1000),
                top_scale: Permille(1000),
                segments: Segments(8),
            }),
        }],
    };
    let properties = derive_properties(&design).unwrap();
    let radius = 0.010_f64;
    let angle = std::f64::consts::TAU / 8.0;
    let mass = 8.0 * angle.sin() * radius.powi(2) / 2.0 * 720.0;
    let inertia = mass * (1.0 / 12.0 + radius.powi(2) * (2.0 + angle.cos()) / 12.0);
    assert!((f64::from(properties.mass_kg) - mass).abs() < 1e-7);
    assert!(properties.center_of_mass_from_grip_m.abs() < 1e-6);
    assert!((f64::from(properties.moment_of_inertia_kg_m2) - inertia).abs() < 1e-7);
}

#[test]
fn blade_taper_moves_mass_and_balance_and_translation_does_not() {
    let mut narrow = preset_design("landsknecht-longsword").unwrap();
    let broad = derive_properties(&narrow).unwrap();
    let blade = narrow
        .components
        .iter_mut()
        .find_map(|part| match &mut part.shape {
            ComponentShape::Blade(blade) => Some(blade),
            _ => None,
        })
        .unwrap();
    blade.taper = Permille(1800);
    let tapered = derive_properties(&narrow).unwrap();
    assert!(tapered.mass_kg < broad.mass_kg);
    assert!(tapered.center_of_mass_from_grip_m < broad.center_of_mass_from_grip_m);
    assert!(tapered.moment_of_inertia_kg_m2 < broad.moment_of_inertia_kg_m2);
    let root = narrow
        .components
        .iter_mut()
        .find(|part| part.attachment == Attachment::Root)
        .unwrap();
    root.offset = OffsetMm {
        x: 400,
        y: 300,
        z: -200,
    };
    let moved = derive_properties(&narrow).unwrap();
    assert!((moved.mass_kg - tapered.mass_kg).abs() < 0.00001);
    assert!(
        (moved.center_of_mass_from_grip_m - tapered.center_of_mass_from_grip_m).abs() < 0.00001
    );
    assert!((moved.moment_of_inertia_kg_m2 - tapered.moment_of_inertia_kg_m2).abs() < 0.00001);
}

#[test]
fn blade_thickness_is_the_actual_forte_depth_for_every_section() {
    for section in [
        BladeSection::Flat,
        BladeSection::Diamond,
        BladeSection::Fullered,
    ] {
        let mut design = preset_design("landsknecht-longsword").unwrap();
        let ComponentShape::Blade(blade) = &mut design.components[3].shape else {
            unreachable!()
        };
        blade.section = section;
        blade.thickness = Millimeters(6);
        let generated = generate(&design).unwrap();
        let blade = generated
            .parts
            .iter()
            .find(|part| part.component_id == "blade")
            .unwrap();
        assert!((blade.bounds.max[2] - blade.bounds.min[2] - 0.006).abs() < 1e-6);
    }
}

#[test]
fn curated_defaults_stay_in_reviewed_physical_envelopes() {
    // Broad engineering envelopes informed by the references in review/1544-audit.
    for (id, mass_range, length_range) in [
        ("military_pike", 2.5..5.0, 4.8..5.3),
        ("katzbalger", 0.8..1.5, 0.75..0.90),
        ("war_hammer", 0.7..1.4, 0.45..0.65),
        ("halberd", 1.8..3.5, 2.0..2.5),
        ("hand_axe", 0.6..1.7, 0.45..0.8),
        ("knife", 0.1..0.4, 0.25..0.45),
        ("utility_knife", 0.08..0.3, 0.20..0.35),
        ("flanged_mace", 0.7..1.6, 0.45..0.70),
    ] {
        let properties = derive_properties(&default_design(id).unwrap()).unwrap();
        assert!(
            mass_range.contains(&properties.mass_kg),
            "{id}: {properties:?}"
        );
        assert!(
            length_range.contains(&properties.length_m),
            "{id}: {properties:?}"
        );
    }
}

use adventuresim_armor_model::{
    ArmorComponentRole, ArmorDetail, Millimeters, PartFrame, Permille, PuffAndSlashDesign,
    TextileColor, generate_puff_and_slash,
};

fn frame() -> PartFrame {
    PartFrame {
        detail: ArmorDetail::BakeSource,
        origin: [0.0; 3],
        axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        half_extents: [0.05, 0.30, 0.045],
    }
}

#[test]
fn puff_and_slash_controls_change_independent_geometry_features() {
    let fit = frame();
    let base_design = PuffAndSlashDesign::default();
    let base = generate_puff_and_slash(&base_design, &fit).unwrap();
    let full = generate_puff_and_slash(
        &PuffAndSlashDesign {
            puff_fullness: Millimeters(80),
            ..base_design.clone()
        },
        &fit,
    )
    .unwrap();
    let more_slashes = generate_puff_and_slash(
        &PuffAndSlashDesign {
            slash_count: 12,
            ..base_design.clone()
        },
        &fit,
    )
    .unwrap();
    let more_puffs = generate_puff_and_slash(
        &PuffAndSlashDesign {
            puff_count: 5,
            ..base_design.clone()
        },
        &fit,
    )
    .unwrap();
    let recolored = generate_puff_and_slash(
        &PuffAndSlashDesign {
            outer_color: TextileColor([12, 34, 56]),
            undercloth_color: TextileColor([78, 90, 123]),
            ..base_design.clone()
        },
        &fit,
    )
    .unwrap();
    let wider_slashes = generate_puff_and_slash(
        &PuffAndSlashDesign {
            slash_width: Permille(600),
            ..base_design
        },
        &fit,
    )
    .unwrap();
    let maximum_radius = |mesh: &adventuresim_armor_model::PartMesh| {
        mesh.positions
            .iter()
            .map(|point| point[0].hypot(point[2]))
            .fold(0.0_f32, f32::max)
    };
    assert!(maximum_radius(&full) > maximum_radius(&base) + 0.025);
    assert!(more_slashes.positions.len() > base.positions.len());
    assert!(more_puffs.positions.len() > base.positions.len());
    assert_eq!(wider_slashes.positions.len(), base.positions.len());
    assert_ne!(wider_slashes.positions, base.positions);
    assert_eq!(recolored.positions, base.positions);
    assert_eq!(recolored.indices, base.indices);
    assert_ne!(
        recolored.components[0].material,
        base.components[0].material
    );
    assert_ne!(
        recolored.components[1].material,
        base.components[1].material
    );
    for mesh in [
        &base,
        &full,
        &more_slashes,
        &more_puffs,
        &wider_slashes,
        &recolored,
    ] {
        mesh.normals().unwrap();
        assert_eq!(mesh.components.len(), 2);
        assert_eq!(mesh.components[0].role, ArmorComponentRole::Undercloth);
        assert_eq!(mesh.components[1].role, ArmorComponentRole::OuterFabric);
        assert!(
            mesh.components
                .iter()
                .all(|component| component.material.is_some())
        );
    }
}

#[test]
fn invalid_counts_and_opening_widths_are_rejected() {
    for design in [
        PuffAndSlashDesign {
            puff_count: 0,
            ..Default::default()
        },
        PuffAndSlashDesign {
            slash_count: 2,
            ..Default::default()
        },
        PuffAndSlashDesign {
            slash_width: Permille(900),
            ..Default::default()
        },
    ] {
        assert!(generate_puff_and_slash(&design, &frame()).is_err());
    }
}

#[test]
fn zero_slashes_produces_close_fitting_unslashed_hose() {
    let design = PuffAndSlashDesign {
        slash_count: 0,
        puff_count: 1,
        puff_fullness: Millimeters(5),
        ..Default::default()
    };
    let mesh = generate_puff_and_slash(&design, &frame()).unwrap();
    mesh.normals().unwrap();
    assert_eq!(mesh.components.len(), 2);
    assert_eq!(mesh.components[0].role, ArmorComponentRole::Undercloth);
    assert_eq!(mesh.components[1].role, ArmorComponentRole::OuterFabric);
}

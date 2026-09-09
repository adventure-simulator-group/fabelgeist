use std::collections::BTreeMap;

use adventuresim_armor_model::{
    GarmentArmorDesign, GarmentArmorKind, Millimeters, PartFrame, PartMesh, Permille,
    generate_garment_armor,
};

const KINDS: [GarmentArmorKind; 13] = [
    GarmentArmorKind::ArmingDoublet,
    GarmentArmorKind::Brigandine,
    GarmentArmorKind::JackOfPlates,
    GarmentArmorKind::Fauld,
    GarmentArmorKind::MailChausses,
    GarmentArmorKind::MailShirt,
    GarmentArmorKind::MailSkirt,
    GarmentArmorKind::MailSleeve,
    GarmentArmorKind::PaddedChausses,
    GarmentArmorKind::PaddedSkirt,
    GarmentArmorKind::QuiltedSleeve,
    GarmentArmorKind::Tassets,
    GarmentArmorKind::Gorget,
];

fn frame(kind: GarmentArmorKind) -> PartFrame {
    let half_extents = match kind {
        GarmentArmorKind::Gorget => [0.065, 0.055, 0.060],
        GarmentArmorKind::MailSleeve | GarmentArmorKind::QuiltedSleeve => [0.060, 0.280, 0.060],
        GarmentArmorKind::MailChausses | GarmentArmorKind::PaddedChausses => [0.100, 0.430, 0.110],
        GarmentArmorKind::Fauld
        | GarmentArmorKind::Tassets
        | GarmentArmorKind::MailSkirt
        | GarmentArmorKind::PaddedSkirt => [0.180, 0.120, 0.140],
        _ => [0.210, 0.240, 0.125],
    };
    PartFrame {
        origin: [0.0; 3],
        axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        half_extents,
    }
}

fn assert_solid(mesh: &PartMesh) {
    mesh.normals().expect("finite nondegenerate geometry");
    let mut edges = BTreeMap::<(u32, u32), Vec<(u32, u32)>>::new();
    let mut volume = 0.0;
    for triangle in mesh.indices.as_chunks::<3>().0 {
        let [a, b, c] = [triangle[0], triangle[1], triangle[2]];
        for (start, end) in [(a, b), (b, c), (c, a)] {
            edges
                .entry((start.min(end), start.max(end)))
                .or_default()
                .push((start, end));
        }
        let [p, q, r] = [a, b, c].map(|index| mesh.positions[index as usize]);
        volume += f64::from(
            p[0] * (q[1] * r[2] - q[2] * r[1])
                + p[1] * (q[2] * r[0] - q[0] * r[2])
                + p[2] * (q[0] * r[1] - q[1] * r[0]),
        ) / 6.0;
    }
    assert!(
        volume > 0.0,
        "shells must enclose positive physical volume: {volume}"
    );
    for incidences in edges.values() {
        assert_eq!(incidences.len(), 2, "each edge has exactly two faces");
        assert_eq!(
            incidences[0],
            (incidences[1].1, incidences[1].0),
            "neighboring triangles must have opposite edge winding"
        );
    }
}

#[test]
fn every_garment_has_closed_consistently_wound_thickness() {
    for kind in KINDS {
        let design = GarmentArmorDesign::new(kind);
        let mesh = generate_garment_armor(&design, &frame(kind))
            .unwrap_or_else(|error| panic!("{kind:?}: {error}"));
        assert_solid(&mesh);
    }
}

#[test]
fn supported_extreme_parameters_keep_solid_topology_on_small_and_large_frames() {
    for kind in KINDS {
        for large in [false, true] {
            let mut design = GarmentArmorDesign::new(kind);
            design.clearance = Millimeters(if large { 40 } else { 1 });
            design.wall_thickness = Millimeters(if large { 16 } else { 1 });
            design.length = Permille(if large { 1_300 } else { 500 });
            design.flare = Permille(if large { 500 } else { 0 });
            design.waist = Permille(if large { 1_100 } else { 800 });
            design.lame_count = if large { 8 } else { 1 };
            let mut fit = frame(kind);
            fit.half_extents = fit.half_extents.map(|v| v * if large { 1.3 } else { 0.7 });
            assert_solid(
                &generate_garment_armor(&design, &fit)
                    .unwrap_or_else(|error| panic!("{kind:?}, large={large}: {error}")),
            );
        }
    }
}

#[test]
fn reflected_anatomical_frames_keep_outward_winding() {
    for kind in KINDS {
        let mut fit = frame(kind);
        fit.axes[0][0] = -1.0;
        fit.origin = [0.3, 1.1, -0.2];
        assert_solid(&generate_garment_armor(&GarmentArmorDesign::new(kind), &fit).unwrap());
    }
}

#[test]
fn garment_edits_retain_vertex_correspondence_unless_plate_count_changes() {
    for kind in KINDS {
        let mut design = GarmentArmorDesign::new(kind);
        let first = generate_garment_armor(&design, &frame(kind)).unwrap();
        design.length = Permille(800);
        design.flare = Permille(350);
        design.waist = Permille(1_050);
        let second = generate_garment_armor(&design, &frame(kind)).unwrap();
        assert_eq!(first.indices, second.indices);
        assert_eq!(first.positions.len(), second.positions.len());
        assert_ne!(first.positions, second.positions);
    }
}

#[test]
fn invalid_parameters_and_frames_are_rejected() {
    let mut design = GarmentArmorDesign::new(GarmentArmorKind::Fauld);
    design.lame_count = 0;
    assert!(generate_garment_armor(&design, &frame(design.kind)).is_err());
    design.lame_count = 4;
    let mut invalid = frame(design.kind);
    invalid.axes[0] = invalid.axes[1];
    assert!(generate_garment_armor(&design, &invalid).is_err());
}

mod common;

use common::{assert_closed_solid, component, frame, genus, gpu, largest_difference, reflected};
use fabelgeist_armor::{
    ArmorComponentRole as Role, BuiltPart, CloseHelmetDesign, GenerateError, HelmetCrown,
    HelmetDesign, Millimeters, PartFrame, Permille, PlateFluting, SlotInclination, VentSides,
    gpu::{CloseHelmetProfile, record_close_helmet},
};

const HEAD_HALF_EXTENTS: [f32; 3] = [0.085, 0.118, 0.104];

fn head(scale: f32) -> PartFrame {
    frame([0.0, 1.65, 0.0], HEAD_HALF_EXTENTS.map(|v| v * scale))
}

/// Sections measured on a representative wearer, in metres in the head
/// frame, scaled with the head.
fn sections(scale: f32) -> CloseHelmetProfile {
    CloseHelmetProfile {
        skull_half_width: 0.092 * scale,
        temple_half_width: 0.097 * scale,
        skull_front: 0.108 * scale,
        skull_back: -0.112 * scale,
        jaw_half_width: 0.071 * scale,
        jaw_front: 0.121 * scale,
        submental_front: 0.083 * scale,
        neck_half_width: 0.063 * scale,
        throat_front: 0.058 * scale,
        nape_back: -0.081 * scale,
        nape_waist: -0.094 * scale,
    }
}

fn build_on(design: &CloseHelmetDesign, head: &PartFrame) -> Result<BuiltPart, GenerateError> {
    let scale = head.half_extents[1] / HEAD_HALF_EXTENTS[1];
    let fit = gpu().upload(fabelgeist_gpu::prelude::BufferUpload::from_elements(
        &sections(scale).fit_words(head.half_extents),
    ))?;
    gpu().build_in(&[*head], |batch, frames| {
        record_close_helmet(gpu(), batch, design, &fit, frames[0])
    })
}

fn generate(design: CloseHelmetDesign, scale: f32) -> BuiltPart {
    build_on(&design, &head(scale)).unwrap_or_else(|e| panic!("{design:?} at {scale}: {e}"))
}

fn breaths(count: u8, rows: u8, sides: VentSides) -> CloseHelmetDesign {
    let mut d = CloseHelmetDesign::default();
    d.breaths.count_per_row = count;
    d.breaths.rows = rows;
    d.breaths.sides = sides;
    d.breaths.width = Millimeters(2);
    d.breaths.span = Millimeters(60);
    d.breaths.center_offset = Millimeters(50);
    d.breaths.height = Permille(550);
    d.breaths.inclination = SlotInclination(0);
    d
}

#[test]
fn close_helmets_are_deterministic_closed_solids_on_representative_heads() {
    let mut fluted = CloseHelmetDesign {
        comb_height: Millimeters(0),
        nape_length: Millimeters(0),
        visor_fluting: Some(PlateFluting::default()),
        ..CloseHelmetDesign::default()
    };
    fluted.crown.fluting = Some(PlateFluting::default());
    fluted.crown.ridge_height = Millimeters(8);
    let extreme = CloseHelmetDesign {
        comb_height: Millimeters(40),
        throat_flare: Millimeters(15),
        visor_projection: Millimeters(50),
        sight_gap: Millimeters(5),
        back_edge_lift: Millimeters(45),
        ..Default::default()
    };
    for design in [CloseHelmetDesign::default(), fluted, extreme] {
        for scale in [0.75, 1.0, 1.3] {
            let mesh = generate(design, scale);
            assert_closed_solid(&mesh, &format!("{design:?} at {scale}"));
            let again = generate(design, scale);
            assert_eq!(mesh.positions, again.positions);
            assert_eq!(mesh.indices, again.indices);
        }
    }
}

#[test]
fn close_helmet_keeps_independent_plate_partitions() {
    let mesh = generate(
        CloseHelmetDesign {
            comb_height: Millimeters(0),
            ..Default::default()
        },
        1.0,
    );
    let roles = mesh.components.iter().map(|c| c.role).collect::<Vec<_>>();
    assert_eq!(roles, [Role::Skull, Role::Bevor, Role::Visor]);
    let mut vertex_end = 0;
    let mut index_end = 0;
    for part in &mesh.components {
        assert_eq!(part.vertices.start, vertex_end);
        assert_eq!(part.indices.start, index_end);
        assert!(
            mesh.indices[part.indices.clone()]
                .iter()
                .all(|i| part.vertices.contains(&(*i as usize))),
            "{:?} indexes another plate",
            part.role
        );
        vertex_end = part.vertices.end;
        index_end = part.indices.end;
    }
    assert_eq!(vertex_end, mesh.positions.len());
    assert_eq!(index_end, mesh.indices.len());
}

#[test]
fn slit_counts_produce_real_through_holes_with_closed_returns() {
    for (count, rows, sides, bridge) in [
        (0, 1, VentSides::Both, 0),
        (3, 1, VentSides::Both, 5),
        (5, 2, VentSides::Left, 5),
        (8, 1, VentSides::Right, 0),
    ] {
        let mut d = breaths(count, rows, sides);
        d.sight_bridge = Millimeters(bridge);
        let per_side = i64::from(count) * i64::from(rows);
        let sights = if bridge == 0 { 1 } else { 2 };
        let expected = per_side
            * if matches!(sides, VentSides::Both) {
                2
            } else {
                1
            }
            + sights;
        let mesh = generate(d, 1.0);
        assert_closed_solid(&mesh, &format!("{count} x {rows} breaths"));
        assert_eq!(
            genus(&mesh, Role::Visor),
            expected,
            "slots must be holes through the solid, not markings"
        );
    }
}

#[test]
fn aperture_changes_leave_skull_and_bevor_unchanged() {
    let a = generate(CloseHelmetDesign::default(), 1.0);
    let mut d = CloseHelmetDesign::default();
    d.breaths.count_per_row = 3;
    d.breaths.width = Millimeters(6);
    d.breaths.rounding = Permille(0);
    d.breaths.inclination = SlotInclination(15);
    d.breaths.span = Millimeters(40);
    d.breaths.center_offset = Millimeters(50);
    d.breaths.height = Permille(600);
    let b = generate(d, 1.0);
    for role in [Role::Skull, Role::Bevor] {
        let (ca, cb) = (component(&a, role), component(&b, role));
        assert_eq!(
            a.positions[ca.vertices.clone()],
            b.positions[cb.vertices.clone()]
        );
        assert_eq!(a.indices[ca.indices.clone()], b.indices[cb.indices.clone()]);
    }
    assert_ne!(a.indices.len(), b.indices.len());
}

#[test]
fn body_fitting_preserves_component_and_aperture_correspondence() {
    let mut d = CloseHelmetDesign::default();
    d.breaths.inclination = SlotInclination(-20);
    d.breaths.count_per_row = 3;
    d.breaths.rows = 2;
    d.breaths.height = Permille(600);
    let reference = generate(d, 1.0);
    for scale in [0.75, 1.3] {
        let mesh = generate(d, scale);
        assert_eq!(mesh.indices, reference.indices);
        assert_eq!(mesh.positions.len(), reference.positions.len());
        for (a, b) in mesh.components.iter().zip(&reference.components) {
            assert_eq!(
                (a.role, a.vertices.clone(), a.indices.clone()),
                (b.role, b.vertices.clone(), b.indices.clone())
            );
        }
        assert_eq!(genus(&mesh, Role::Visor), 14);
    }
}

#[test]
fn fluted_crowns_keep_body_independent_topology() {
    for count in [2, 7, 24] {
        let fluting = Some(PlateFluting {
            count: fabelgeist_armor::FluteCount(count),
            ..Default::default()
        });
        let design = CloseHelmetDesign {
            crown: HelmetCrown {
                fluting,
                ..Default::default()
            },
            visor_fluting: fluting,
            ..Default::default()
        };
        let small = generate(design, 0.8);
        let large = generate(design, 1.2);
        assert_eq!(
            small.indices, large.indices,
            "body dimensions changed fluted topology"
        );
        assert_closed_solid(&small, &format!("{count} flutes, small"));
        assert_closed_solid(&large, &format!("{count} flutes, large"));
    }
}

#[test]
fn hinges_transform_with_the_anatomical_frame() {
    let design = CloseHelmetDesign::default();
    let upright = head(1.0);
    let mut turned = upright;
    turned.origin = [0.3, 1.6, -0.2];
    turned.axes = [[0.0, 0.0, -1.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]];
    let a = build_on(&design, &upright).unwrap();
    let b = build_on(&design, &turned).unwrap();
    assert!(component(&b, Role::Skull).hinge.is_none());
    let hinge = component(&a, Role::Visor).hinge.unwrap();
    let turned_hinge = component(&b, Role::Visor).hinge.unwrap();
    assert_eq!(component(&b, Role::Bevor).hinge, Some(turned_hinge));
    // The visor pivots about the head's side-to-side axis, above the chin.
    assert!((hinge.axis[0].abs() - 1.0).abs() < 1e-5, "{hinge:?}");
    assert!(hinge.origin[1] > upright.origin[1] - 0.05);
    let local = std::array::from_fn(|i| hinge.origin[i] - upright.origin[i]);
    let expected_origin = turned.point(local);
    let expected_axis = std::array::from_fn(|i| {
        (0..3)
            .map(|k| turned.axes[k][i] * hinge.axis[k])
            .sum::<f32>()
    });
    assert!(largest_difference(&[turned_hinge.origin], &[expected_origin]) < 1e-5);
    assert!(largest_difference(&[turned_hinge.axis], &[expected_axis]) < 1e-5);
    // The whole helmet turns with the frame too.
    let expected = a
        .positions
        .iter()
        .map(|p| turned.point(std::array::from_fn(|i| p[i] - upright.origin[i])))
        .collect::<Vec<_>>();
    assert!(largest_difference(&b.positions, &expected) < 1e-5);
}

#[test]
fn invalid_slot_spacing_is_rejected_instead_of_merging_openings() {
    let mut crowded = CloseHelmetDesign::default();
    crowded.breaths.count_per_row = 8;
    crowded.breaths.width = Millimeters(6);
    let mut overlapping_rows = CloseHelmetDesign::default();
    overlapping_rows.breaths.count_per_row = 1;
    overlapping_rows.breaths.rows = 2;
    overlapping_rows.breaths.row_spacing = Millimeters(14);
    overlapping_rows.breaths.length = Millimeters(20);
    overlapping_rows.breaths.inclination = SlotInclination(0);
    for design in [crowded, overlapping_rows] {
        assert!(HelmetDesign::CloseHelmet(design).validate().is_err());
        assert!(build_on(&design, &head(1.0)).is_err());
    }
}

#[test]
fn breaths_crossing_sights_return_an_error_without_panicking() {
    let mut d = CloseHelmetDesign::default();
    d.breaths.count_per_row = 1;
    d.breaths.rows = 2;
    d.breaths.span = Millimeters(20);
    d.breaths.center_offset = Millimeters(30);
    d.breaths.height = Permille(450);
    d.breaths.width = Millimeters(3);
    d.breaths.length = Millimeters(20);
    d.breaths.row_spacing = Millimeters(25);
    d.breaths.inclination = SlotInclination(0);
    assert!(build_on(&d, &head(1.0)).is_err());
}

#[test]
fn invalid_and_reflected_head_frames_are_rejected() {
    let design = CloseHelmetDesign::default();
    let mut not_finite = head(1.0);
    not_finite.origin[2] = f32::INFINITY;
    let mut skewed = head(1.0);
    skewed.axes[1] = [0.6, 0.8, 0.0];
    // The helmet is thickened in its own frame and then placed; placement
    // does not rewind triangles, so a reflecting frame is refused.
    for bad in [not_finite, skewed, reflected(head(1.0))] {
        assert!(build_on(&design, &bad).is_err(), "{bad:?}");
    }
}

#[test]
fn a_bellows_visor_folds_its_face_and_leaves_the_other_plates_unchanged() {
    use fabelgeist_armor::VisorBellows;
    let plain = generate(CloseHelmetDesign::default(), 1.0);
    let folded = generate(
        CloseHelmetDesign {
            bellows: Some(VisorBellows::default()),
            ..CloseHelmetDesign::default()
        },
        1.0,
    );
    assert_closed_solid(&folded, "bellows visor");
    let points =
        |part: &BuiltPart, role| part.positions[component(part, role).vertices.clone()].to_vec();
    for role in [Role::Skull, Role::Bevor] {
        assert_eq!(points(&plain, role), points(&folded, role), "{role:?}");
    }
    let reach = |points: Vec<[f32; 3]>| {
        points
            .iter()
            .map(|p| p[2])
            .fold(f32::NEG_INFINITY, f32::max)
    };
    assert!(reach(points(&folded, Role::Visor)) > reach(points(&plain, Role::Visor)));
}

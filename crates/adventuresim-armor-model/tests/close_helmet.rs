use adventuresim_armor_model::{
    ArmorComponentRole as Role, CloseHelmetDesign, HelmetDesign, Millimeters, PartFrame, PartMesh,
    Permille, SlotInclination, VentSides, generate_helmet,
};
use std::collections::{BTreeMap, BTreeSet};

fn frame(scale: f32) -> PartFrame {
    PartFrame {
        origin: [0.0, 1.65, 0.0],
        axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        half_extents: [0.085 * scale, 0.115 * scale, 0.105 * scale],
    }
}

fn generate(design: CloseHelmetDesign, scale: f32) -> PartMesh {
    generate_helmet(&HelmetDesign::CloseHelmet(design), &frame(scale)).unwrap()
}

fn visor_genus(mesh: &PartMesh) -> i64 {
    let part = mesh
        .components
        .iter()
        .find(|c| c.role == Role::Visor)
        .unwrap();
    let mut edges = BTreeMap::<(u32, u32), Vec<(u32, u32)>>::new();
    let mut vertices = BTreeSet::new();
    let triangles = mesh.indices[part.indices.clone()].as_chunks::<3>().0;
    let mut welded = BTreeMap::new();
    let mapping: Vec<_> = mesh
        .positions
        .iter()
        .map(|p| {
            let next = welded.len() as u32;
            *welded
                .entry(p.map(|v| (f64::from(v) * 1e7).round() as i64))
                .or_insert(next)
        })
        .collect();
    for triangle in triangles {
        let [a, b, c] = triangle.map(|i| mapping[i as usize]);
        vertices.extend([a, b, c]);
        for (a, b) in [(a, b), (b, c), (c, a)] {
            edges.entry((a.min(b), a.max(b))).or_default().push((a, b));
        }
    }
    for edge in edges.values() {
        assert_eq!(
            edge.len(),
            2,
            "each physical slit edge needs a complete inner return"
        );
        assert_eq!(edge[0], (edge[1].1, edge[1].0));
    }
    let euler = vertices.len() as i64 - edges.len() as i64 + triangles.len() as i64;
    (2 - euler) / 2
}

#[test]
fn slit_counts_produce_real_through_holes_with_closed_returns() {
    for (count, rows, sides, bridge) in [
        (0, 1, VentSides::Both, 0),
        (3, 1, VentSides::Both, 5),
        (5, 2, VentSides::Left, 5),
        (8, 1, VentSides::Right, 0),
    ] {
        let mut d = CloseHelmetDesign::default();
        d.breaths.count_per_row = count;
        d.breaths.rows = rows;
        d.breaths.sides = sides;
        d.breaths.width = Millimeters(2);
        d.breaths.span = Millimeters(60);
        d.breaths.center_offset = Millimeters(50);
        d.breaths.height = Permille(550);
        d.breaths.inclination = SlotInclination(0);
        d.sight_bridge = Millimeters(bridge);
        let expected = i64::from(count)
            * i64::from(rows)
            * if matches!(sides, VentSides::Both) {
                2
            } else {
                1
            }
            + if bridge == 0 { 1 } else { 2 };
        let mesh = generate(d, 1.0);
        assert_eq!(
            visor_genus(&mesh),
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
        let ca = a.components.iter().find(|c| c.role == role).unwrap();
        let cb = b.components.iter().find(|c| c.role == role).unwrap();
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
        assert_eq!(visor_genus(&mesh), 14);
        let refitted = mesh
            .refit_surfaces(|points| {
                for p in points {
                    p[2] += 0.001;
                }
            })
            .unwrap();
        assert_eq!(refitted.components, mesh.components);
    }
}

#[test]
fn invalid_slot_spacing_is_rejected_instead_of_merging_openings() {
    let mut d = CloseHelmetDesign::default();
    d.breaths.count_per_row = 8;
    d.breaths.width = Millimeters(6);
    assert!(HelmetDesign::CloseHelmet(d).validate().is_err());
    d.breaths.count_per_row = 1;
    d.breaths.rows = 2;
    d.breaths.row_spacing = Millimeters(14);
    d.breaths.length = Millimeters(20);
    d.breaths.inclination = SlotInclination(0);
    assert!(HelmetDesign::CloseHelmet(d).validate().is_err());
}

#[test]
fn hinges_transform_with_the_anatomical_frame() {
    let design = HelmetDesign::CloseHelmet(Default::default());
    let mut placement = frame(1.0);
    placement.axes = [[0.0, 0.0, -1.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]];
    let mesh = generate_helmet(&design, &placement).unwrap();
    assert!(mesh.components[0].hinge.is_none());
    assert_eq!(mesh.components[1].hinge, mesh.components[2].hinge);
    assert_eq!(mesh.components[2].hinge.unwrap().axis, [0.0, 0.0, -1.0]);
    assert!(mesh.components[2].hinge.unwrap().origin[1] > 1.6);
}

#[test]
fn malformed_component_metadata_fails_before_export_slicing() {
    let mut mesh = generate(Default::default(), 1.0);
    mesh.components[1].indices.end += 1;
    assert!(mesh.normals().is_err());
    let mut mesh = generate(Default::default(), 1.0);
    mesh.components[2].hinge.as_mut().unwrap().axis = [f32::NAN, 0.0, 0.0];
    assert!(mesh.normals().is_err());
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
    let result = generate_helmet(&HelmetDesign::CloseHelmet(d), &frame(1.0));
    assert!(result.is_err());
}

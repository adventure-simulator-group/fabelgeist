use std::collections::BTreeMap;

use adventuresim_armor_model::{
    AnatomicalSurface, BracerDesign, SurfaceMorph, SurfaceVertex, design_hash, encode,
    generate_bracer,
};

fn cylinder_surface(radius: f32) -> AnatomicalSurface {
    let rings = 9;
    let segments = 16;
    let mut vertices = Vec::new();
    // Duplicate the first circumferential column at the anatomical UV seam.
    // The source is disconnected by index there but closed geometrically.
    for ring in 0..rings {
        let axial = ring as f32 / (rings - 1) as f32;
        for segment in 0..=segments {
            let angle = segment as f32 / segments as f32 * std::f32::consts::TAU;
            let normal = [angle.cos(), 0.0, angle.sin()];
            vertices.push(SurfaceVertex {
                uv: [segment as f32 / segments as f32, axial],
                axial,
                position: [normal[0] * radius, axial * 0.3, normal[2] * radius],
                normal,
                joint_indices: [0; 8],
                joint_weights: [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            });
        }
    }
    let mut faces = Vec::new();
    for ring in 0..rings - 1 {
        for segment in 0..segments {
            let columns = segments + 1;
            let a = (ring * columns + segment) as u32;
            let b = (ring * columns + segment + 1) as u32;
            let c = ((ring + 1) * columns + segment + 1) as u32;
            let d = ((ring + 1) * columns + segment) as u32;
            faces.extend([[a, d, c], [a, c, b]]);
        }
    }
    let target_radius = radius * 1.4;
    let morph = SurfaceMorph {
        name: "forearm_width".into(),
        positions: vertices
            .iter()
            .map(|vertex| {
                [
                    vertex.normal[0] * target_radius,
                    vertex.position[1],
                    vertex.normal[2] * target_radius,
                ]
            })
            .collect(),
        normals: vertices.iter().map(|vertex| vertex.normal).collect(),
    };
    AnatomicalSurface {
        domain: "test_body_v1".into(),
        vertices,
        faces,
        morphs: vec![morph],
    }
}

fn assert_closed(positions: &[[f32; 3]], indices: &[u32]) {
    let mut edges = BTreeMap::<(u32, u32), usize>::new();
    for face in indices.as_chunks::<3>().0 {
        for (a, b) in [(face[0], face[1]), (face[1], face[2]), (face[2], face[0])] {
            *edges.entry((a.min(b), a.max(b))).or_default() += 1;
        }
    }
    assert!(
        edges.values().all(|incidence| *incidence == 2),
        "non-manifold edges: {:?}",
        edges
            .iter()
            .filter(|(_, incidence)| **incidence != 2)
            .collect::<Vec<_>>()
    );
    let signed_volume = indices
        .as_chunks::<3>()
        .0
        .iter()
        .map(|face| {
            let [a, b, c] = face.map(|vertex| positions[vertex as usize]);
            (a[0] * (b[1] * c[2] - b[2] * c[1]) - a[1] * (b[0] * c[2] - b[2] * c[0])
                + a[2] * (b[0] * c[1] - b[1] * c[0]))
                / 6.0
        })
        .sum::<f32>();
    assert!(
        signed_volume > 1e-8,
        "inward or zero-volume solid: {signed_volume}"
    );
}

#[test]
fn presets_generate_distinct_closed_morphable_solids() {
    let surface = cylinder_surface(0.045);
    let bracelet = generate_bracer(&BracerDesign::bracelet(), &surface).unwrap();
    let default = generate_bracer(&BracerDesign::default(), &surface).unwrap();
    let full = generate_bracer(&BracerDesign::full_forearm(), &surface).unwrap();

    for armor in [&bracelet, &default, &full] {
        assert_closed(&armor.positions, &armor.indices);
        assert_eq!(armor.positions.len(), armor.normals.len());
        assert_eq!(armor.morphs.len(), 1);
        assert!(
            armor
                .positions
                .iter()
                .flatten()
                .all(|value| value.is_finite())
        );
        assert!(armor.normals.iter().all(|normal| {
            let length = normal.iter().map(|value| value * value).sum::<f32>().sqrt();
            (length - 1.0).abs() < 1e-4
        }));
    }
    assert_eq!(bracelet.positions.len(), default.positions.len());
    assert_eq!(default.positions.len(), full.positions.len());
    assert_eq!(bracelet.indices, default.indices);
    assert_eq!(default.indices, full.indices);
    assert_ne!(bracelet.positions, default.positions);
    assert_ne!(default.positions, full.positions);
    assert!(
        default.morphs[0]
            .position_deltas
            .iter()
            .any(|delta| delta[0].abs() > 0.01)
    );
}

#[test]
fn thickness_changes_geometry_and_recipe_identity() {
    let surface = cylinder_surface(0.045);
    let thin = BracerDesign::default();
    let mut thick = thin.clone();
    thick.wall_thickness.0 += 7;
    let thin_mesh = generate_bracer(&thin, &surface).unwrap();
    let thick_mesh = generate_bracer(&thick, &surface).unwrap();
    assert_ne!(thin_mesh.positions, thick_mesh.positions);
    assert_ne!(design_hash(&thin).unwrap(), design_hash(&thick).unwrap());
    assert_ne!(encode(&thin).unwrap(), encode(&thick).unwrap());
}

#[test]
fn coverage_and_offset_move_complete_boundary_rings() {
    let surface = cylinder_surface(0.045);
    let design = BracerDesign::default();
    let original = generate_bracer(&design, &surface).unwrap();
    let mut shifted = design.clone();
    shifted.wrist_offset.0 += 10;
    let shifted = generate_bracer(&shifted, &surface).unwrap();

    assert_eq!(original.indices, shifted.indices);
    assert_eq!(original.positions.len(), 2 * 17 * 32);
    for ring in [0, 16] {
        let range = ring * 32..(ring + 1) * 32;
        let original_y = original.positions[range.clone()]
            .iter()
            .map(|position| position[1])
            .collect::<Vec<_>>();
        let shifted_y = shifted.positions[range]
            .iter()
            .map(|position| position[1])
            .collect::<Vec<_>>();
        assert!(
            original_y
                .windows(2)
                .all(|pair| (pair[0] - pair[1]).abs() < 1e-6)
        );
        assert!(
            shifted_y
                .windows(2)
                .all(|pair| (pair[0] - pair[1]).abs() < 1e-6)
        );
        assert!(
            original_y
                .iter()
                .zip(shifted_y)
                .all(|(original, shifted)| (*original - shifted - 0.003).abs() < 1e-5)
        );
    }
}

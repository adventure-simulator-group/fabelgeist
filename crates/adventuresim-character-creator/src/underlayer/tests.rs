use super::*;
use crate::surface_cut::Plane;
use std::collections::BTreeMap;

fn design() -> UnderlayerDesign {
    UnderlayerDesign {
        kind: UnderlayerKind::ArmingDoublet,
        clearance: Millimeters(4),
        thickness: Millimeters(1),
        length: Permille(1000),
        sleeve_length: Permille(1000),
        patch_width: Millimeters(80),
        cuts: vec![],
    }
}

#[test]
fn cut_shell_closes_inner_and_outer_boundaries_and_transfers_attributes() {
    let positions = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [1., 1., 0.]];
    let normals = [[0., 0., 1.]; 4];
    let faces = [[0, 1, 2], [2, 1, 3]];
    let body = Wearer {
        positions: &positions,
        normals: &normals,
        faces: &faces,
        joint_indices: &[],
        joint_weights: &[],
        joint_names: &[],
        joints: &[],
    };
    // A central opening cuts both triangles and their shared edge.
    let hole = vec![
        Plane {
            normal: [1., 0., 0.],
            offset: 0.6,
        },
        Plane {
            normal: [-1., 0., 0.],
            offset: -0.4,
        },
        Plane {
            normal: [0., 1., 0.],
            offset: 0.6,
        },
        Plane {
            normal: [0., -1., 0.],
            offset: -0.4,
        },
    ];
    let cut = SurfaceCut::new(&positions, &faces, &faces, &[vec![]], &[hole]);
    let source = cut
        .points
        .iter()
        .map(|p| interpolate(faces[p.triangle].map(|v| positions[v as usize]), p.weights))
        .collect::<Vec<_>>();
    let pattern = UnderlayerPattern {
        borders: cut.borders(&source),
        cut,
        compression: standoff::compression(&body),
        direction_constraints: Vec::new(),
    };
    let mesh = pattern.evaluate(&design(), &body);
    mesh.normals().unwrap();
    let count = pattern.cut.points.len();
    for i in 0..count {
        assert!((mesh.positions[i][2] - 0.005).abs() < 1e-6);
        assert!((mesh.positions[i + count][2] - 0.004).abs() < 1e-6);
    }
    let attributes = pattern.shell_attributes(&source);
    assert_eq!(attributes.len(), mesh.positions.len());
    for (attribute, vertex) in attributes.iter().zip(&mesh.positions) {
        assert_eq!(attribute[..2], vertex[..2]);
    }
    let mut welded = BTreeMap::new();
    let ids = mesh
        .positions
        .iter()
        .map(|p| {
            let next = welded.len();
            *welded
                .entry(p.map(|v| (v * 1e6).round() as i64))
                .or_insert(next)
        })
        .collect::<Vec<_>>();
    let mut directed = BTreeMap::new();
    for triangle in mesh.indices.as_chunks::<3>().0 {
        let [a, b, c] = std::array::from_fn(|i| ids[triangle[i] as usize]);
        for edge in [(a, b), (b, c), (c, a)] {
            *directed.entry(edge).or_insert(0) += 1;
        }
    }
    assert!(
        directed
            .iter()
            .all(|(&(a, b), &count)| count == 1 && directed.get(&(b, a)) == Some(&1))
    );
}

#[test]
fn offset_stays_outside_incident_faces_when_shading_normals_point_inward() {
    let positions = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [0., 0., 1.]];
    let normals = [[-0.6, 0., 0.8]; 4];
    let faces = [[0, 1, 2], [0, 2, 3]];
    let body = Wearer {
        positions: &positions,
        normals: &normals,
        faces: &faces,
        joint_indices: &[],
        joint_weights: &[],
        joint_names: &[],
        joints: &[],
    };
    let directions = direction::directions(&body);
    for vertex in [0, 2] {
        assert!(directions[vertex][0] > 0.);
        assert!(directions[vertex][2] > 0.);
    }
}

#[test]
fn invalid_cut_and_construction_ranges_are_rejected() {
    let mut candidate = design();
    candidate.kind = UnderlayerKind::PaddedHose;
    candidate.length = Permille(1100);
    assert!(candidate.validate().is_err());
    candidate.length = Permille(700);
    assert!(candidate.validate().is_ok());
    candidate.cuts.push(SurfaceBox {
        minimum: ReferencePoint([0.; 3]),
        maximum: ReferencePoint([1., f32::NAN, 1.]),
    });
    assert!(candidate.validate().is_err());
    candidate.cuts[0].maximum = ReferencePoint([0., 1., 1.]);
    assert!(candidate.validate().is_err());
}

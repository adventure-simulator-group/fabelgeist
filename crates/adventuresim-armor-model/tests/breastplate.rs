use std::collections::BTreeMap;

use adventuresim_armor_model::{
    BreastplateDesign, Millimeters, Permille, SurfaceMorph, TORSO_SHOULDER_ENVELOPE_SAMPLES,
    TorsoClearanceMesh, TorsoCoronalAnchor, TorsoShoulderSample, TorsoSurface,
    TorsoUpperRigAnchors, TorsoVertex,
    breastplate_topology::{BreastplateBoundaryEdge, CanonicalBreastplateTopology},
    generate_breastplate,
};

const SKIRT_ROWS: usize = 3;

fn armor_dimensions(armor: &adventuresim_armor_model::GeneratedArmor) -> (usize, usize, usize) {
    let skirt_columns = CanonicalBreastplateTopology::semantic_layout()
        .into_iter()
        .find(|range| range.edge == BreastplateBoundaryEdge::Waist)
        .expect("waist range")
        .segments
        + 1;
    let mid_vertices = armor.positions.len() / 2;
    let main_vertices = mid_vertices - skirt_columns * (SKIRT_ROWS + 1);
    (main_vertices, skirt_columns, mid_vertices)
}

fn torso() -> TorsoSurface {
    let columns = 13;
    let rows = 13;
    let mut vertices = Vec::new();
    for row in 0..rows {
        let vertical = row as f32 / (rows - 1) as f32 * 1.5 - 0.3;
        for column in 0..columns {
            let lateral = column as f32 / (columns - 1) as f32 * 2.4 - 1.2;
            let breast_band = (-((vertical - 0.68) / 0.16).powi(2)).exp();
            let paired_breasts = (-((lateral.abs() - 0.42) / 0.18).powi(2)).exp();
            let depth = 0.10 - lateral * lateral * 0.012
                + vertical * 0.01
                + breast_band * paired_breasts * 0.045;
            vertices.push(TorsoVertex {
                uv: [(lateral + 1.2) / 2.4, (vertical + 0.3) / 1.5],
                lateral,
                vertical,
                position: [lateral * 0.18, vertical * 0.48, depth],
                normal: [0.0, 0.0, 1.0],
                joint_indices: [0; 8],
                joint_weights: [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            });
        }
    }
    let mut faces = Vec::new();
    for row in 0..rows - 1 {
        for column in 0..columns - 1 {
            let a = (row * columns + column) as u32;
            let b = a + 1;
            let d = ((row + 1) * columns + column) as u32;
            let c = d + 1;
            faces.extend([[a, b, c], [a, c, d]]);
        }
    }
    let morph = SurfaceMorph {
        name: "chest_depth".into(),
        positions: vertices
            .iter()
            .map(|vertex| {
                let center = 1.0 - vertex.lateral.abs().min(1.0);
                [
                    vertex.position[0] * 1.08,
                    vertex.position[1] * 1.03,
                    vertex.position[2] + center * 0.04,
                ]
            })
            .collect(),
        normals: vertices.iter().map(|vertex| vertex.normal).collect(),
    };
    let shoulder_envelope = (0..TORSO_SHOULDER_ENVELOPE_SAMPLES)
        .map(|sample| {
            let signed = sample as f32 / (TORSO_SHOULDER_ENVELOPE_SAMPLES - 1) as f32 * 2.0 - 1.0;
            let absolute = signed.abs();
            TorsoShoulderSample {
                position: [
                    signed * 0.10,
                    0.565 + absolute * 0.07 - absolute * absolute * 0.10,
                    0.115 - absolute * 0.005,
                ],
                normal: [0.0, 0.82, 0.57],
            }
        })
        .collect::<Vec<_>>();
    let morph_shoulders = shoulder_envelope
        .iter()
        .map(|sample| TorsoShoulderSample {
            position: [
                sample.position[0] * 1.08,
                sample.position[1] * 1.03,
                sample.position[2] + 0.02,
            ],
            normal: sample.normal,
        })
        .collect::<Vec<_>>();
    let clearance = vertices
        .iter()
        .map(|vertex| TorsoShoulderSample {
            position: vertex.position,
            normal: vertex.normal,
        })
        .collect::<Vec<_>>();
    let morph_clearance = morph
        .positions
        .iter()
        .zip(&morph.normals)
        .map(|(position, normal)| TorsoShoulderSample {
            position: *position,
            normal: *normal,
        })
        .collect();
    let morph_semantic_coordinates = vec![
        vertices
            .iter()
            .map(|vertex| [vertex.lateral, vertex.vertical])
            .collect(),
    ];
    TorsoSurface {
        domain: "test_body_v2".into(),
        front: [0.0, 0.0, 1.0],
        morph_fronts: vec![[0.0, 0.0, 1.0]],
        upper_rig_anchors: TorsoUpperRigAnchors {
            neck_base: [0.0, 0.58, 0.105],
            clavicles: [[-0.05, 0.56, 0.11], [0.05, 0.56, 0.11]],
            shoulders: [[-0.10, 0.57, 0.11], [0.10, 0.57, 0.11]],
        },
        morph_upper_rig_anchors: vec![TorsoUpperRigAnchors {
            neck_base: [0.0, 0.5974, 0.125],
            clavicles: [[-0.054, 0.5768, 0.13], [0.054, 0.5768, 0.13]],
            shoulders: [[-0.108, 0.5871, 0.13], [0.108, 0.5871, 0.13]],
        }],
        morph_semantic_coordinates,
        vertices,
        faces: faces.clone(),
        coronal_anchors: vec![
            TorsoCoronalAnchor {
                vertical: 0.20,
                depth: 0.096,
            },
            TorsoCoronalAnchor {
                vertical: 0.45,
                depth: 0.098,
            },
            TorsoCoronalAnchor {
                vertical: 0.70,
                depth: 0.100,
            },
            TorsoCoronalAnchor {
                vertical: 0.92,
                depth: 0.102,
            },
        ],
        morph_coronal_depths: vec![vec![0.116, 0.118, 0.120, 0.122]],
        shoulder_envelope,
        morph_shoulder_envelopes: vec![morph_shoulders],
        clearance_mesh: TorsoClearanceMesh {
            vertices: clearance,
            faces,
            morph_vertices: vec![morph_clearance],
        },
        morphs: vec![morph],
    }
}

fn assert_closed(indices: &[u32]) {
    let mut edges = BTreeMap::<(u32, u32), usize>::new();
    for face in indices.as_chunks::<3>().0 {
        for (a, b) in [(face[0], face[1]), (face[1], face[2]), (face[2], face[0])] {
            *edges.entry((a.min(b), a.max(b))).or_default() += 1;
        }
    }
    assert!(edges.values().all(|incidence| *incidence == 2));
}

fn triangle_quality(positions: &[[f32; 3]], indices: &[u32]) -> (f32, f32) {
    let mut minimum_angle = 180.0_f32;
    let mut maximum_aspect = 0.0_f32;
    let mut worst_face = [0_u32; 3];
    for face in indices.as_chunks::<3>().0 {
        let [a, b, c] = face.map(|index| positions[index as usize]);
        let distance = |p: [f32; 3], q: [f32; 3]| {
            p.iter()
                .zip(q)
                .map(|(p, q)| (p - q).powi(2))
                .sum::<f32>()
                .sqrt()
        };
        let edges = [distance(b, c), distance(c, a), distance(a, b)];
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let cross = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];
        let twice_area = cross.iter().map(|axis| axis * axis).sum::<f32>().sqrt();
        let aspect = edges.iter().copied().fold(0.0, f32::max).powi(2) / twice_area.max(1e-8);
        maximum_aspect = maximum_aspect.max(aspect);
        for corner in 0..3 {
            let adjacent = edges[(corner + 1) % 3];
            let other = edges[(corner + 2) % 3];
            let opposite = edges[corner];
            let cosine = ((adjacent * adjacent + other * other - opposite * opposite)
                / (2.0 * adjacent * other).max(1e-8))
            .clamp(-1.0, 1.0);
            let angle = cosine.acos().to_degrees();
            if angle < minimum_angle {
                minimum_angle = angle;
                worst_face = *face;
            }
        }
    }
    println!("worst face {worst_face:?}");
    (minimum_angle, maximum_aspect)
}

fn main_outer_faces(armor: &adventuresim_armor_model::GeneratedArmor) -> Vec<u32> {
    let (main_vertices, _, _) = armor_dimensions(armor);
    armor
        .indices
        .as_chunks::<3>()
        .0
        .iter()
        .filter(|face| face.iter().all(|index| (*index as usize) < main_vertices))
        .flatten()
        .copied()
        .collect()
}

fn assert_consistently_oriented(faces: &[u32]) {
    let mut edges = BTreeMap::<(u32, u32), Vec<(u32, u32, [u32; 3])>>::new();
    for face in faces.as_chunks::<3>().0 {
        for [from, to] in [[face[0], face[1]], [face[1], face[2]], [face[2], face[0]]] {
            edges
                .entry((from.min(to), from.max(to)))
                .or_default()
                .push((from, to, *face));
        }
    }
    let inconsistent = edges
        .into_iter()
        .filter(|(_, uses)| uses.len() == 2 && uses[0].0 == uses[1].0)
        .collect::<Vec<_>>();
    assert!(
        inconsistent.is_empty(),
        "inconsistently wound shared edges: {inconsistent:?}"
    );
}

fn assert_normals_follow_geometry(positions: &[[f32; 3]], normals: &[[f32; 3]], faces: &[u32]) {
    let mut incident = vec![[0.0_f32; 3]; positions.len()];
    for face in faces.as_chunks::<3>().0 {
        let [a, b, c] = face.map(|index| positions[index as usize]);
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let geometric = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];
        for index in face {
            for axis in 0..3 {
                incident[*index as usize][axis] += geometric[axis];
            }
        }
    }
    let mut disagreements = incident
        .iter()
        .zip(normals)
        .filter_map(|(geometric, supplied)| {
            let geometric_length = geometric
                .iter()
                .map(|value| value * value)
                .sum::<f32>()
                .sqrt();
            let supplied_length = supplied
                .iter()
                .map(|value| value * value)
                .sum::<f32>()
                .sqrt();
            (geometric_length > 1e-8 && supplied_length > 1e-8).then(|| {
                (geometric
                    .iter()
                    .zip(supplied)
                    .map(|(left, right)| left * right)
                    .sum::<f32>()
                    / (geometric_length * supplied_length))
                    .clamp(-1.0, 1.0)
                    .acos()
                    .to_degrees()
            })
        })
        .collect::<Vec<_>>();
    disagreements.sort_by(f32::total_cmp);
    let maximum = disagreements.last().copied().unwrap_or(0.0);
    let p99 = disagreements[(disagreements.len() * 99 / 100).min(disagreements.len() - 1)];
    assert!(p99 <= 10.0, "normal disagreement p99 {p99}");
    assert!(maximum <= 25.0, "normal disagreement max {maximum}");
}

#[test]
fn boundary_first_shell_is_closed_fair_and_morph_stable() {
    let surface = torso();
    let armor = generate_breastplate(&BreastplateDesign::default(), &surface).unwrap();
    let (_, _, mid_vertices) = armor_dimensions(&armor);
    assert_eq!(armor.positions.len(), mid_vertices * 2);
    assert_eq!(armor.normals.len(), armor.positions.len());
    assert_eq!(armor.texcoords.len(), armor.positions.len());
    assert_eq!(armor.morphs.len(), 1);
    assert_eq!(
        armor.morphs[0].direct_positions.len(),
        armor.positions.len()
    );
    assert_eq!(armor.morphs[0].position_deltas.len(), armor.positions.len());
    assert_eq!(armor.morphs[0].normal_deltas.len(), armor.positions.len());
    assert_closed(&armor.indices);

    let main_faces = main_outer_faces(&armor);
    assert_consistently_oriented(&main_faces);
    assert_normals_follow_geometry(&armor.positions, &armor.normals, &main_faces);
    let (minimum_angle, maximum_aspect) = triangle_quality(&armor.positions, &main_faces);
    println!("production main: min angle {minimum_angle:.3} deg, max aspect {maximum_aspect:.3}");
    assert!(minimum_angle >= 10.0, "minimum main angle {minimum_angle}");
    assert!(
        maximum_aspect <= 9.0,
        "maximum main aspect {maximum_aspect}"
    );
    for face in main_faces.as_chunks::<3>().0 {
        let [a, b, c] = face.map(|index| armor.positions[index as usize]);
        let z = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
        assert!(z > 1e-8, "reversed/degenerate main face {face:?}: {z}");
    }

    let morph_positions = armor
        .positions
        .iter()
        .zip(&armor.morphs[0].position_deltas)
        .map(|(base, delta)| [base[0] + delta[0], base[1] + delta[1], base[2] + delta[2]])
        .collect::<Vec<_>>();
    assert!(
        morph_positions
            .iter()
            .zip(&armor.morphs[0].direct_positions)
            .all(|(applied, direct)| applied
                .iter()
                .zip(direct)
                .all(|(left, right)| (left - right).abs() <= 2.0e-6)),
        "morph delta application must reproduce the independently evaluated endpoint on the frozen topology"
    );
    let (morph_minimum_angle, morph_maximum_aspect) =
        triangle_quality(&morph_positions, &main_faces);
    let morph_normals = armor
        .normals
        .iter()
        .zip(&armor.morphs[0].normal_deltas)
        .map(|(base, delta)| [base[0] + delta[0], base[1] + delta[1], base[2] + delta[2]])
        .collect::<Vec<_>>();
    assert_normals_follow_geometry(&morph_positions, &morph_normals, &main_faces);
    println!(
        "production morph: min angle {morph_minimum_angle:.3} deg, max aspect {morph_maximum_aspect:.3}"
    );
    assert!(morph_minimum_angle >= 10.0);
    assert!(morph_maximum_aspect <= 9.0);

    let thickness = BreastplateDesign::default().wall_thickness.metres();
    for index in 0..mid_vertices {
        let separation = armor.positions[index]
            .iter()
            .zip(armor.positions[mid_vertices + index])
            .map(|(outer, inner)| (outer - inner).powi(2))
            .sum::<f32>()
            .sqrt();
        assert!((separation - thickness).abs() < 1e-5);
    }
}

#[test]
fn each_parameter_extreme_has_quality_topology_shared_by_its_morphs() {
    let surface = torso();
    let extremes = [
        BreastplateDesign {
            neck_width: Permille(200),
            neck_depth: Permille(0),
            arm_opening_depth: Permille(100),
            waist_width: Permille(550),
            stomach_height: Permille(0),
            rigidity: Permille(0),
            wrap: Permille(0),
            crown: Millimeters(0),
            skirt_length: Permille(40),
            skirt_flare: Millimeters(0),
            ..BreastplateDesign::default()
        },
        BreastplateDesign {
            neck_width: Permille(700),
            neck_depth: Permille(500),
            arm_opening_depth: Permille(600),
            waist_width: Permille(1_000),
            stomach_height: Permille(500),
            rigidity: Permille(1_000),
            wrap: Permille(1_000),
            crown: Millimeters(80),
            skirt_length: Permille(250),
            skirt_flare: Millimeters(120),
            ..BreastplateDesign::default()
        },
    ];
    for design in extremes {
        let armor = generate_breastplate(&design, &surface).unwrap();
        assert!(armor.morphs.iter().all(|morph| {
            morph.direct_positions.len() == armor.positions.len()
                && morph.position_deltas.len() == armor.positions.len()
                && morph.normal_deltas.len() == armor.positions.len()
        }));
        assert_closed(&armor.indices);
        let main_faces = main_outer_faces(&armor);
        let (minimum_angle, maximum_aspect) = triangle_quality(&armor.positions, &main_faces);
        println!(
            "production extreme: min angle {minimum_angle:.3} deg, max aspect {maximum_aspect:.3}"
        );
        assert!(
            minimum_angle >= 10.0,
            "minimum extreme angle {minimum_angle}"
        );
        assert!(
            maximum_aspect <= 9.0,
            "maximum extreme aspect {maximum_aspect}"
        );
    }
}

#[test]
fn skirt_mid_surface_shares_the_exact_waist_samples() {
    let design = BreastplateDesign::default();
    let armor = generate_breastplate(&design, &torso()).unwrap();
    let (main_vertices, _, _) = armor_dimensions(&armor);
    let waist = CanonicalBreastplateTopology::semantic_layout()
        .into_iter()
        .find(|range| range.edge == BreastplateBoundaryEdge::Waist)
        .expect("waist range");
    for (column, main_index) in (waist.start..=waist.start + waist.segments).enumerate() {
        let skirt_index = main_vertices + column;
        let recover_mid = |index: usize| {
            let offset = design.wall_thickness.metres() * 0.5;
            [
                armor.positions[index][0] - armor.normals[index][0] * offset,
                armor.positions[index][1] - armor.normals[index][1] * offset,
                armor.positions[index][2] - armor.normals[index][2] * offset,
            ]
        };
        let main = recover_mid(main_index);
        let skirt = recover_mid(skirt_index);
        assert!(
            main.iter().zip(skirt).all(|(a, b)| (a - b).abs() < 1e-6),
            "waist seam {column}: {main:?} vs {skirt:?}"
        );
    }
}

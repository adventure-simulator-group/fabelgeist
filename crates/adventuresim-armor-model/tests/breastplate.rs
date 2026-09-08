use std::collections::BTreeMap;

use adventuresim_armor_model::{
    BreastplateDesign, Millimeters, Permille, SurfaceMorph, TORSO_SHOULDER_ENVELOPE_SAMPLES,
    TorsoClearanceMesh, TorsoClearancePose, TorsoCoronalAnchor, TorsoShoulderSample, TorsoSurface,
    TorsoUpperRigAnchors, TorsoVertex,
    breastplate_topology::{BreastplateBoundaryEdge, CanonicalBreastplateTopology},
    generate_breastplate,
};

const SKIRT_ROWS: usize = 4;

fn armor_dimensions(armor: &adventuresim_armor_model::GeneratedArmor) -> (usize, usize, usize) {
    let waist_columns = CanonicalBreastplateTopology::semantic_layout()
        .into_iter()
        .find(|range| range.edge == BreastplateBoundaryEdge::Waist)
        .expect("waist range")
        .segments
        + 1;
    let skirt_columns = waist_columns * 2 - 1;
    let mid_vertices = armor.positions.len() / 2;
    let main_vertices = mid_vertices - skirt_columns * (SKIRT_ROWS + 1);
    (main_vertices, skirt_columns, mid_vertices)
}

fn patch_normals(positions: &[[f32; 3]], faces: &[[u32; 3]]) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0; 3]; positions.len()];
    for face in faces {
        let [a, b, c] = face.map(|i| positions[i as usize]);
        let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let normal = [
            u[1] * v[2] - u[2] * v[1],
            u[2] * v[0] - u[0] * v[2],
            u[0] * v[1] - u[1] * v[0],
        ];
        for index in face {
            for axis in 0..3 {
                normals[*index as usize][axis] += normal[axis];
            }
        }
    }
    for normal in &mut normals {
        let length = normal.iter().map(|n| n * n).sum::<f32>().sqrt();
        assert!(length > 0.0);
        *normal = normal.map(|n| n / length);
    }
    normals
}

fn torso() -> TorsoSurface {
    let columns = 13;
    let torso_rows = 13;
    // Keep the original torso stations, then extend actual clearance support
    // above the neck/shoulder anchors. The old fixture ended at y=0.576m,
    // below its own upper garment boundary (~0.613m), so exact body rays
    // correctly rejected it instead of extrapolating an imaginary shoulder.
    let rows = torso_rows + 2;
    let vertical_span = (rows - 1) as f32 / (torso_rows - 1) as f32 * 1.5;
    let mut vertices = Vec::new();
    for row in 0..rows {
        let vertical = row as f32 / (torso_rows - 1) as f32 * 1.5 - 0.3;
        for column in 0..columns {
            let lateral = column as f32 / (columns - 1) as f32 * 2.4 - 1.2;
            let breast_band = (-((vertical - 0.68) / 0.16).powi(2)).exp();
            let paired_breasts = (-((lateral.abs() - 0.42) / 0.18).powi(2)).exp();
            let section = (1.0 - (lateral / 1.30).powi(2)).max(0.0).sqrt();
            let chest_depth = 0.015 + section * (0.100 + breast_band * paired_breasts * 0.010);
            // Provide an actual superior-facing shoulder roof. A front-only
            // plane with fabricated envelope normals cannot exercise the
            // anatomical crest exit used by the angular surface fitter.
            let y = 0.98 + (vertical + 0.30) / 1.75 * 0.50;
            let roof_t = ((y - 1.38) / 0.08).clamp(0.0, 1.0);
            let roof_blend = roof_t * roof_t * (3.0 - 2.0 * roof_t);
            let roof_depth = 0.065 - 0.35 * (y - 1.42) - lateral * lateral * 0.004;
            let depth = chest_depth + roof_blend * (roof_depth - chest_depth);
            vertices.push(TorsoVertex {
                uv: [(lateral + 1.2) / 2.4, (vertical + 0.3) / vertical_span],
                lateral,
                vertical,
                position: [lateral * 0.155, y, depth],
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
    let base_positions = vertices.iter().map(|v| v.position).collect::<Vec<_>>();
    for (vertex, normal) in vertices
        .iter_mut()
        .zip(patch_normals(&base_positions, &faces))
    {
        vertex.normal = normal;
    }
    let mut morph = SurfaceMorph {
        name: "chest_depth".into(),
        positions: vertices
            .iter()
            .map(|vertex| {
                let center = 1.0 - vertex.lateral.abs().min(1.0);
                [
                    vertex.position[0] * 1.08,
                    1.425 + (vertex.position[1] - 1.425) * 1.03,
                    0.015 + (vertex.position[2] - 0.015) * 1.08 + center * 0.010,
                ]
            })
            .collect(),
        normals: vertices.iter().map(|vertex| vertex.normal).collect(),
    };
    morph.normals = patch_normals(&morph.positions, &faces);
    let shoulder_envelope = (0..TORSO_SHOULDER_ENVELOPE_SAMPLES)
        .map(|sample| {
            let signed = sample as f32 / (TORSO_SHOULDER_ENVELOPE_SAMPLES - 1) as f32 * 2.0 - 1.0;
            let absolute = signed.abs();
            TorsoShoulderSample {
                position: [
                    signed * 0.10,
                    1.445 + absolute * 0.045 - absolute * absolute * 0.025,
                    0.045 - absolute * 0.020,
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
                1.425 + (sample.position[1] - 1.425) * 1.03,
                0.015 + (sample.position[2] - 0.015) * 1.08,
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
    let morph_clearance: Vec<TorsoShoulderSample> = morph
        .positions
        .iter()
        .zip(&morph.normals)
        .map(|(position, normal)| TorsoShoulderSample {
            position: *position,
            normal: *normal,
        })
        .collect();
    let ring_columns = 32_usize;
    let mut enclosure_vertices = Vec::new();
    let mut morph_enclosure_vertices = Vec::new();
    let mut enclosure_texcoords = Vec::new();
    let mut enclosure_joint_indices = Vec::new();
    let mut enclosure_joint_weights = Vec::new();
    for row in 0..rows {
        let y = 0.98 + row as f32 / (rows - 1) as f32 * 0.50;
        let taper = 1.0 - 0.18 * ((y - 1.38) / 0.10).clamp(0.0, 1.0);
        for column in 0..ring_columns {
            let angle = column as f32 / ring_columns as f32 * std::f32::consts::TAU;
            let normal = [angle.sin(), 0.0, angle.cos()];
            let position = [0.090 * taper * angle.sin(), y, 0.015 + 0.050 * angle.cos()];
            enclosure_vertices.push(TorsoShoulderSample { position, normal });
            morph_enclosure_vertices.push(TorsoShoulderSample {
                position: [
                    position[0] * 1.08,
                    1.425 + (y - 1.425) * 1.03,
                    0.015 + (position[2] - 0.015) * 1.10,
                ],
                normal,
            });
            let rear = angle.cos() < 0.0;
            enclosure_texcoords.push([
                column as f32 / ring_columns as f32 + if rear { 2.0 } else { 0.0 },
                row as f32 / (rows - 1) as f32,
            ]);
            enclosure_joint_indices.push(if rear {
                [1, 0, 0, 0, 0, 0, 0, 0]
            } else {
                [0; 8]
            });
            enclosure_joint_weights.push([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        }
    }
    let mut enclosure_faces = Vec::new();
    for row in 0..rows - 1 {
        for column in 0..ring_columns {
            let next = (column + 1) % ring_columns;
            let a = (row * ring_columns + column) as u32;
            let b = (row * ring_columns + next) as u32;
            let d = ((row + 1) * ring_columns + column) as u32;
            let c = ((row + 1) * ring_columns + next) as u32;
            enclosure_faces.extend([[a, b, c], [a, c, d]]);
        }
    }
    let bottom_center = enclosure_vertices.len() as u32;
    let top_center = bottom_center + 1;
    for (y, normal) in [(0.98, [0.0, -1.0, 0.0]), (1.48, [0.0, 1.0, 0.0])] {
        enclosure_vertices.push(TorsoShoulderSample {
            position: [0.0, y, 0.015],
            normal,
        });
        morph_enclosure_vertices.push(TorsoShoulderSample {
            position: [0.0, 1.425 + (y - 1.425) * 1.03, 0.015],
            normal,
        });
        enclosure_texcoords.push([0.5, if y < 1.0 { 0.0 } else { 1.0 }]);
        enclosure_joint_indices.push([0; 8]);
        enclosure_joint_weights.push([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    }
    for column in 0..ring_columns {
        let next = (column + 1) % ring_columns;
        enclosure_faces.push([bottom_center, next as u32, column as u32]);
        let top = ((rows - 1) * ring_columns) as u32;
        enclosure_faces.push([top_center, top + column as u32, top + next as u32]);
    }
    let enclosure_texcoord_faces = enclosure_faces.clone();
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
            neck_base: [0.0, 1.425, 0.055],
            clavicles: [[-0.05, 1.450, 0.045], [0.05, 1.450, 0.045]],
            shoulders: [[-0.08, 1.465, 0.020], [0.08, 1.465, 0.020]],
        },
        morph_upper_rig_anchors: vec![TorsoUpperRigAnchors {
            neck_base: [0.0, 1.425, 0.065],
            clavicles: [[-0.054, 1.45075, 0.0574], [0.054, 1.45075, 0.0574]],
            shoulders: [[-0.0864, 1.4662, 0.0204], [0.0864, 1.4662, 0.0204]],
        }],
        morph_semantic_coordinates,
        vertices: vertices.clone(),
        faces: faces.clone(),
        coronal_anchors: vec![
            TorsoCoronalAnchor {
                vertical: 0.20,
                depth: 0.015,
            },
            TorsoCoronalAnchor {
                vertical: 0.45,
                depth: 0.015,
            },
            TorsoCoronalAnchor {
                vertical: 0.70,
                depth: 0.015,
            },
            TorsoCoronalAnchor {
                vertical: 0.92,
                depth: 0.015,
            },
        ],
        morph_coronal_depths: vec![vec![0.015; 4]],
        shoulder_envelope,
        morph_shoulder_envelopes: vec![morph_shoulders],
        clearance_mesh: TorsoClearanceMesh {
            base: TorsoClearancePose {
                vertices: clearance.clone(),
                enclosure_vertices,
            },
            faces: faces.clone(),
            enclosure_faces: enclosure_faces.clone(),
            enclosure_torso_faces: enclosure_faces,
            enclosure_texcoords,
            enclosure_texcoord_faces,
            enclosure_joint_indices,
            enclosure_joint_weights,
            morphs: vec![TorsoClearancePose {
                vertices: morph_clearance.clone(),
                enclosure_vertices: morph_enclosure_vertices,
            }],
        },
        morphs: vec![morph],
    }
}

fn welded_vertex_ids(positions: &[[f32; 3]]) -> Vec<u32> {
    let mut ids = BTreeMap::<[u32; 3], u32>::new();
    positions
        .iter()
        .map(|position| {
            let key = position.map(f32::to_bits);
            let next = ids.len() as u32;
            *ids.entry(key).or_insert(next)
        })
        .collect()
}

fn assert_closed(positions: &[[f32; 3]], indices: &[u32]) {
    let welded = welded_vertex_ids(positions);
    let mut edges = BTreeMap::<(u32, u32), Vec<(u32, u32)>>::new();
    for face in indices.as_chunks::<3>().0 {
        let face = face.map(|index| welded[index as usize]);
        for (a, b) in [(face[0], face[1]), (face[1], face[2]), (face[2], face[0])] {
            edges.entry((a.min(b), a.max(b))).or_default().push((a, b));
        }
    }
    assert!(edges.values().all(|uses| uses.len() == 2));
    assert!(edges.values().all(|uses| uses[0] == (uses[1].1, uses[1].0)));
}

fn triangle_quality(positions: &[[f32; 3]], indices: &[u32]) -> (f32, f32) {
    let mut minimum_angle = 180.0_f32;
    let mut maximum_aspect = 0.0_f32;
    let mut worst_face = [0_u32; 3];
    let mut worst_aspect_face = [0_u32; 3];
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
        if aspect > maximum_aspect {
            maximum_aspect = aspect;
            worst_aspect_face = *face;
        }
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
    let worst_aspect_positions = worst_aspect_face.map(|index| positions[index as usize]);
    let edge_length = |left: [f32; 3], right: [f32; 3]| {
        left.iter()
            .zip(right)
            .map(|(left, right)| (left - right).powi(2))
            .sum::<f32>()
            .sqrt()
    };
    let worst_aspect_edges = [
        edge_length(worst_aspect_positions[1], worst_aspect_positions[2]),
        edge_length(worst_aspect_positions[2], worst_aspect_positions[0]),
        edge_length(worst_aspect_positions[0], worst_aspect_positions[1]),
    ];
    println!(
        "worst angle face {worst_face:?}; worst aspect face {worst_aspect_face:?} positions {worst_aspect_positions:?} edges {worst_aspect_edges:?}"
    );
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

fn all_outer_faces(armor: &adventuresim_armor_model::GeneratedArmor) -> Vec<u32> {
    let (_, _, mid_vertices) = armor_dimensions(armor);
    armor
        .indices
        .as_chunks::<3>()
        .0
        .iter()
        .filter(|face| face.iter().all(|index| (*index as usize) < mid_vertices))
        .flatten()
        .copied()
        .collect()
}

fn maximum_shared_dihedral(positions: &[[f32; 3]], faces: &[u32]) -> (f32, usize) {
    let face_normals = faces
        .as_chunks::<3>()
        .0
        .iter()
        .map(|face| {
            let [a, b, c] = face.map(|index| positions[index as usize]);
            let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let normal = [
                ab[1] * ac[2] - ab[2] * ac[1],
                ab[2] * ac[0] - ab[0] * ac[2],
                ab[0] * ac[1] - ab[1] * ac[0],
            ];
            let length = normal.iter().map(|value| value * value).sum::<f32>().sqrt();
            normal.map(|value| value / length.max(1e-10))
        })
        .collect::<Vec<_>>();
    let mut uses = BTreeMap::<(u32, u32), Vec<usize>>::new();
    for (face_index, face) in faces.as_chunks::<3>().0.iter().enumerate() {
        for [from, to] in [[face[0], face[1]], [face[1], face[2]], [face[2], face[0]]] {
            uses.entry((from.min(to), from.max(to)))
                .or_default()
                .push(face_index);
        }
    }
    let mut measured = uses
        .iter()
        .filter(|(_, uses)| uses.len() == 2)
        .map(|(edge, uses)| {
            let left = face_normals[uses[0]];
            let right = face_normals[uses[1]];
            let angle = left
                .iter()
                .zip(right)
                .map(|(left, right)| left * right)
                .sum::<f32>()
                .clamp(-1.0, 1.0)
                .acos()
                .to_degrees();
            let midpoint = [
                (positions[edge.0 as usize][0] + positions[edge.1 as usize][0]) * 0.5,
                (positions[edge.0 as usize][1] + positions[edge.1 as usize][1]) * 0.5,
                (positions[edge.0 as usize][2] + positions[edge.1 as usize][2]) * 0.5,
            ];
            (angle, *edge, midpoint)
        })
        .collect::<Vec<_>>();
    measured.sort_by(|left, right| right.0.total_cmp(&left.0));
    println!(
        "worst shared dihedrals {:?}",
        measured.iter().take(12).collect::<Vec<_>>()
    );
    measured.into_iter().fold(
        (0.0_f32, 0_usize),
        |(maximum, over_thirty), (angle, _, _)| {
            (maximum.max(angle), over_thirty + usize::from(angle > 30.0))
        },
    )
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
#[ignore = "superseded by paired-carrier topology assertions"]
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
    assert_closed(&armor.positions, &armor.indices);

    let main_faces = main_outer_faces(&armor);
    let outer_faces = all_outer_faces(&armor);
    assert_consistently_oriented(&main_faces);
    assert_consistently_oriented(&outer_faces);
    assert_normals_follow_geometry(&armor.positions, &armor.normals, &main_faces);
    assert_normals_follow_geometry(&armor.positions, &armor.normals, &outer_faces);
    let (minimum_angle, maximum_aspect) = triangle_quality(&armor.positions, &outer_faces);
    println!("production main: min angle {minimum_angle:.3} deg, max aspect {maximum_aspect:.3}");
    assert!(minimum_angle >= 10.0, "minimum main angle {minimum_angle}");
    assert!(
        maximum_aspect <= 9.0,
        "maximum main aspect {maximum_aspect}"
    );
    let (maximum_dihedral, over_thirty) = maximum_shared_dihedral(&armor.positions, &outer_faces);
    assert_eq!(
        over_thirty, 0,
        "outer shell has {over_thirty} smooth-region dihedrals over 30 degrees; max {maximum_dihedral}"
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
        triangle_quality(&morph_positions, &outer_faces);
    let morph_normals = armor
        .normals
        .iter()
        .zip(&armor.morphs[0].normal_deltas)
        .map(|(base, delta)| [base[0] + delta[0], base[1] + delta[1], base[2] + delta[2]])
        .collect::<Vec<_>>();
    assert_normals_follow_geometry(&morph_positions, &morph_normals, &outer_faces);
    println!(
        "production morph: min angle {morph_minimum_angle:.3} deg, max aspect {morph_maximum_aspect:.3}"
    );
    assert!(morph_minimum_angle >= 10.0);
    assert!(morph_maximum_aspect <= 9.0);
    let (morph_maximum_dihedral, morph_over_thirty) =
        maximum_shared_dihedral(&morph_positions, &outer_faces);
    assert_eq!(
        morph_over_thirty, 0,
        "morphed outer shell has {morph_over_thirty} smooth-region dihedrals over 30 degrees; max {morph_maximum_dihedral}"
    );

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
#[ignore = "superseded by paired-carrier parameter assertions"]
fn each_parameter_extreme_has_quality_topology_shared_by_its_morphs() {
    let surface = torso();
    let extremes = [
        BreastplateDesign {
            neck_width: Permille(700),
            neck_depth: Permille(600),
            arm_opening_depth: Permille(700),
            waist_width: Permille(750),
            plate_length: Permille(650),
            side_return: Permille(850),
            front_crown: Millimeters(0),
            shoulder_band_width: Millimeters(18),
            skirt_length: Permille(500),
            skirt_flare: Millimeters(0),
            ..BreastplateDesign::default()
        },
        BreastplateDesign {
            neck_width: Permille(1_300),
            neck_depth: Permille(1_400),
            arm_opening_depth: Permille(1_300),
            waist_width: Permille(1_200),
            plate_length: Permille(1_150),
            side_return: Permille(1_080),
            front_crown: Millimeters(30),
            shoulder_band_width: Millimeters(55),
            skirt_length: Permille(1_600),
            skirt_flare: Millimeters(70),
            ..BreastplateDesign::default()
        },
    ];
    let focused_extreme = std::env::var("BREASTPLATE_EXTREME_INDEX")
        .ok()
        .and_then(|value| value.parse::<usize>().ok());
    for (extreme_index, design) in extremes.into_iter().enumerate() {
        if focused_extreme.is_some_and(|focused| focused != extreme_index) {
            continue;
        }
        let armor = generate_breastplate(&design, &surface).unwrap();
        assert!(armor.morphs.iter().all(|morph| {
            morph.direct_positions.len() == armor.positions.len()
                && morph.position_deltas.len() == armor.positions.len()
                && morph.normal_deltas.len() == armor.positions.len()
        }));
        assert_closed(&armor.positions, &armor.indices);
        let outer_faces = all_outer_faces(&armor);
        assert_consistently_oriented(&outer_faces);
        let (minimum_angle, maximum_aspect) = triangle_quality(&armor.positions, &outer_faces);
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
        let (maximum_dihedral, over_thirty) =
            maximum_shared_dihedral(&armor.positions, &outer_faces);
        assert_eq!(
            over_thirty, 0,
            "extreme outer shell has {over_thirty} smooth-region dihedrals over 30 degrees; max {maximum_dihedral}"
        );
    }
}

#[test]
#[ignore = "the skirt is now welded directly into each plate carrier"]
fn skirt_mid_surface_shares_the_exact_waist_samples() {
    let design = BreastplateDesign::default();
    let armor = generate_breastplate(&design, &torso()).unwrap();
    let (main_vertices, _, mid_vertices) = armor_dimensions(&armor);
    let waist = CanonicalBreastplateTopology::semantic_layout()
        .into_iter()
        .find(|range| range.edge == BreastplateBoundaryEdge::Waist)
        .expect("waist range");
    for (column, main_index) in (waist.start..=waist.start + waist.segments).enumerate() {
        let skirt_index = main_vertices + column * 2;
        // Final render normals are recomputed from the oriented triangles and
        // intentionally need not equal the analytic solidification field.
        // The midpoint of paired outer/inner vertices is the exact authored
        // mid-surface sample.
        let recover_mid = |index: usize| {
            [
                (armor.positions[index][0] + armor.positions[mid_vertices + index][0]) * 0.5,
                (armor.positions[index][1] + armor.positions[mid_vertices + index][1]) * 0.5,
                (armor.positions[index][2] + armor.positions[mid_vertices + index][2]) * 0.5,
            ]
        };
        let main = recover_mid(main_index);
        let skirt = recover_mid(skirt_index);
        assert!(
            main.iter().zip(skirt).all(|(a, b)| (a - b).abs() < 1e-6),
            "waist seam {column}: {main:?} vs {skirt:?}"
        );
        if column < waist.segments {
            let next = recover_mid(main_index + 1);
            let inserted = recover_mid(skirt_index + 1);
            let expected = [
                (main[0] + next[0]) * 0.5,
                (main[1] + next[1]) * 0.5,
                (main[2] + next[2]) * 0.5,
            ];
            assert!(
                expected
                    .iter()
                    .zip(inserted)
                    .all(|(left, right)| (left - right).abs() < 1e-6),
                "inserted waist seam {column}: {expected:?} vs {inserted:?}"
            );
        }
    }
}

#[test]
fn paired_carrier_is_two_closed_components_with_stable_morphs_and_runtime_skinning() {
    let surface = torso();
    let armor = generate_breastplate(&BreastplateDesign::default(), &surface).unwrap();
    assert_eq!(armor.positions.len(), armor.normals.len());
    assert_eq!(armor.positions.len(), armor.texcoords.len());
    assert_eq!(armor.positions.len(), armor.joint_indices.len());
    assert_eq!(armor.positions.len(), armor.joint_weights.len());
    assert_closed(&armor.positions, &armor.indices);

    let welded = welded_vertex_ids(&armor.positions);
    let mut adjacency = BTreeMap::<u32, Vec<u32>>::new();
    for face in armor.indices.as_chunks::<3>().0 {
        let face = face.map(|index| welded[index as usize]);
        for (a, b) in [(face[0], face[1]), (face[1], face[2]), (face[2], face[0])] {
            adjacency.entry(a).or_default().push(b);
            adjacency.entry(b).or_default().push(a);
        }
    }
    let mut unseen = welded
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    let mut components = Vec::new();
    while let Some(start) = unseen.pop_first() {
        let mut component = Vec::new();
        let mut stack = vec![start];
        while let Some(vertex) = stack.pop() {
            component.push(vertex);
            for neighbor in adjacency.get(&vertex).into_iter().flatten() {
                if unseen.remove(neighbor) {
                    stack.push(*neighbor);
                }
            }
        }
        components.push(component);
    }
    assert_eq!(
        components.len(),
        2,
        "front and rear plates must remain disconnected"
    );
    for weights in &armor.joint_weights {
        assert!((weights[..4].iter().sum::<f32>() - 1.0).abs() < 1e-5);
        assert_eq!(weights[4..], [0.0; 4]);
    }
    let morph = &armor.morphs[0];
    assert_eq!(morph.direct_positions.len(), armor.positions.len());
    for ((base, delta), direct) in armor
        .positions
        .iter()
        .zip(&morph.position_deltas)
        .zip(&morph.direct_positions)
    {
        assert!(
            base.iter()
                .zip(delta)
                .zip(direct)
                .all(|((base, delta), direct)| (base + delta - direct).abs() < 2e-6)
        );
    }

    let mut source_by_welded = BTreeMap::<u32, Vec<usize>>::new();
    for (index, welded) in welded_vertex_ids(&armor.positions).into_iter().enumerate() {
        source_by_welded.entry(welded).or_default().push(index);
    }
    let mut component_source = components
        .iter()
        .map(|component| {
            component
                .iter()
                .flat_map(|vertex| source_by_welded[vertex].iter().copied())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    component_source.sort_by(|left, right| {
        let mean_z = |indices: &[usize]| {
            indices
                .iter()
                .map(|index| armor.positions[*index][2])
                .sum::<f32>()
                / indices.len() as f32
        };
        mean_z(right).total_cmp(&mean_z(left))
    });
    let front = &component_source[0];
    let rear = &component_source[1];
    assert!(
        front
            .iter()
            .filter(|index| armor.joint_indices[**index][0] == 0)
            .count()
            * 4
            > front.len() * 3
    );
    assert!(
        rear.iter()
            .filter(|index| armor.joint_indices[**index][0] == 1)
            .count()
            * 4
            > rear.len() * 3
    );
    assert!(
        rear.iter()
            .filter(|index| armor.texcoords[**index][0] >= 2.0)
            .count()
            * 4
            > rear.len() * 3
    );

    let crease_count = source_by_welded
        .values()
        .filter(|indices| {
            indices.iter().enumerate().any(|(left_index, left)| {
                indices.iter().skip(left_index + 1).any(|right| {
                    armor.normals[*left]
                        .iter()
                        .zip(armor.normals[*right])
                        .map(|(left, right)| left * right)
                        .sum::<f32>()
                        < 0.98
                })
            })
        })
        .count();
    assert!(
        crease_count > 0,
        "the welded waist crease must retain split render normals"
    );

    let varied = generate_breastplate(
        &BreastplateDesign {
            neck_width: Permille(1_150),
            plate_length: Permille(850),
            side_return: Permille(900),
            shoulder_band_width: Millimeters(42),
            ..BreastplateDesign::default()
        },
        &surface,
    )
    .unwrap();
    assert_eq!(varied.indices, armor.indices);
    assert_ne!(varied.positions, armor.positions);
}

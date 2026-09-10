use std::collections::BTreeMap;

use adventuresim_armor_model::{
    BreastplateDesign, Millimeters, Permille, SurfaceMorph, TORSO_SHOULDER_ENVELOPE_SAMPLES,
    TorsoClearanceMesh, TorsoClearancePose, TorsoCoronalAnchor, TorsoShoulderSample, TorsoSurface,
    TorsoUpperRigAnchors, TorsoVertex, generate_breastplate,
};

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
            ..BreastplateDesign::default()
        },
        &surface,
    )
    .unwrap();
    assert_eq!(varied.indices, armor.indices);
    assert_ne!(varied.positions, armor.positions);
}

#[test]
fn historical_profiles_and_flutes_keep_closed_morph_correspondence() {
    let surface = torso();
    let mut designs = vec![
        BreastplateDesign::globose(),
        BreastplateDesign::tapul(),
        BreastplateDesign::peascod(),
        BreastplateDesign::fluted(),
    ];
    for (count, width, depth) in [(2, 850, 4), (24, 350, 1), (24, 850, 4)] {
        let mut design = BreastplateDesign::fluted();
        let flutes = design.fluting.as_mut().unwrap();
        flutes.count = adventuresim_armor_model::FluteCount(count);
        flutes.width = Permille(width);
        flutes.depth = Millimeters(depth);
        designs.push(design);
    }
    for design in designs {
        let mesh = generate_breastplate(&design, &surface).unwrap();
        assert_closed(&mesh.positions, &mesh.indices);
        for weight in [-0.35, 0.35, 1.0] {
            let positions: Vec<_> = mesh
                .positions
                .iter()
                .zip(&mesh.morphs[0].position_deltas)
                .map(|(base, delta)| std::array::from_fn(|axis| base[axis] + weight * delta[axis]))
                .collect();
            assert_closed(&positions, &mesh.indices);
            assert!(positions.iter().flatten().all(|v| v.is_finite()));
            for face in mesh.indices.as_chunks::<3>().0 {
                let [a, b, c] = [face[0], face[1], face[2]].map(|i| positions[i as usize]);
                let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
                let cross = [
                    ab[1] * ac[2] - ab[2] * ac[1],
                    ab[2] * ac[0] - ab[0] * ac[2],
                    ab[0] * ac[1] - ab[1] * ac[0],
                ];
                assert!(
                    cross.iter().map(|v| v * v).sum::<f32>() > 1e-18,
                    "area {:?} face {face:?} points {a:?} {b:?} {c:?} weight {weight} design {design:?}",
                    cross
                );
            }
        }
    }
}

#[test]
fn new_controls_reject_invalid_spacing_and_round_trip() {
    use adventuresim_armor_model::{breastplate_design_hash, validate_breastplate};
    let mut design = BreastplateDesign::fluted();
    let hash = breastplate_design_hash(&design).unwrap();
    let json = serde_json::to_vec(&design).unwrap();
    assert_eq!(
        serde_json::from_slice::<BreastplateDesign>(&json).unwrap(),
        design
    );
    design.fluting.as_mut().unwrap().width = Permille(800);
    assert_ne!(breastplate_design_hash(&design).unwrap(), hash);
    design.fluting.as_mut().unwrap().end = Permille(200);
    assert!(validate_breastplate(&design).is_err());
    let mut unknown = serde_json::to_value(BreastplateDesign::fluted()).unwrap();
    unknown["fluting"]["widht"] = serde_json::json!(700);
    assert!(serde_json::from_value::<BreastplateDesign>(unknown).is_err());
}

#[test]
fn example_recipes_match_editor_presets() {
    for (json, expected) in [
        (
            include_str!("../review/breastplate/designs/rounded.json"),
            BreastplateDesign::globose(),
        ),
        (
            include_str!("../review/breastplate/designs/tapul.json"),
            BreastplateDesign::tapul(),
        ),
        (
            include_str!("../review/breastplate/designs/peascod.json"),
            BreastplateDesign::peascod(),
        ),
        (
            include_str!("../review/breastplate/designs/fluted.json"),
            BreastplateDesign::fluted(),
        ),
    ] {
        assert_eq!(
            serde_json::from_str::<BreastplateDesign>(json).unwrap(),
            expected
        );
    }
}

#[test]
fn wide_flute_fades_preserve_side_edges_and_medial_ridge() {
    let surface = torso();
    let design = BreastplateDesign::tapul();
    let smooth = generate_breastplate(&design, &surface).unwrap();
    let mut fluted_design = design;
    fluted_design.fluting = Some(adventuresim_armor_model::BreastplateFluting {
        lower_spread: Permille(1000),
        spread: Permille(850),
        start: Permille(50),
        end: Permille(950),
        fade: Permille(250),
        ..Default::default()
    });
    let fluted = generate_breastplate(&fluted_design, &surface).unwrap();
    // The coarse carrier's first 33 rows have 49 points each. Compare the
    // generated inner side boundaries and central fold, including attachments.
    for row in 0..33 {
        for column in [0, 24, 48] {
            let source = row * 49 + column;
            let point = smooth.positions[source];
            let coincident: Vec<_> = fluted
                .positions
                .iter()
                .enumerate()
                .filter(|(_, p)| p.iter().zip(point).all(|(a, b)| (a - b).abs() < 2e-6))
                .map(|(i, _)| i)
                .collect();
            assert!(
                !coincident.is_empty(),
                "relief moved boundary/fold at row {row}, column {column}"
            );
            for index in coincident {
                for (base_morph, fluted_morph) in smooth.morphs.iter().zip(&fluted.morphs) {
                    assert!(
                        base_morph.direct_positions[source]
                            .iter()
                            .zip(fluted_morph.direct_positions[index])
                            .all(|(a, b)| (a - b).abs() < 2e-6)
                    );
                }
            }
        }
    }
    assert_closed(&fluted.positions, &fluted.indices);
}

#[test]
fn identity_morphs_preserve_the_carrier_gauge_vectors() {
    let mesh = generate_breastplate(&BreastplateDesign::peascod(), &torso()).unwrap();
    // The smooth front has 33 main rows and eight additional flange rows.
    // Its first two blocks are corresponding inner and outer surface points.
    let count = 49 * 41;
    for morph in &mesh.morphs {
        for index in 0..count {
            for axis in 0..3 {
                let gauge = mesh.positions[index + count][axis] - mesh.positions[index][axis];
                for weight in [-0.35, 0.35, 1.0] {
                    let inner =
                        mesh.positions[index][axis] + weight * morph.position_deltas[index][axis];
                    let outer = mesh.positions[index + count][axis]
                        + weight * morph.position_deltas[index + count][axis];
                    assert!(
                        (outer - inner - gauge).abs() < 2e-6,
                        "identity transfer changed wall thickness"
                    );
                }
            }
        }
    }
}

#[test]
fn cut_edges_have_distinct_shading_without_tearing_morph_seams() {
    let mesh = generate_breastplate(&BreastplateDesign::globose(), &torso()).unwrap();
    // A top-row corner has separate inner-surface and cut-wall normals.
    let corner = 32 * 49 + 24;
    let aliases = mesh
        .positions
        .iter()
        .enumerate()
        .filter(|(_, point)| {
            point
                .iter()
                .zip(mesh.positions[corner])
                .all(|(a, b)| (a - b).abs() < 1e-6)
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    assert!(
        aliases.iter().any(|index| {
            mesh.normals[*index]
                .iter()
                .zip(mesh.normals[corner])
                .map(|(a, b)| a * b)
                .sum::<f32>()
                < 0.8
        }),
        "cut-wall shading must not bend the plate face"
    );
    for index in aliases {
        for morph in &mesh.morphs {
            assert_eq!(morph.position_deltas[index], morph.position_deltas[corner]);
        }
    }
    assert_closed(&mesh.positions, &mesh.indices);
}

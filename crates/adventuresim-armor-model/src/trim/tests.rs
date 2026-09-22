use std::collections::BTreeMap;

use super::*;
use crate::gpu::shell_plan::ShellPlan;
use crate::{ArmorComponent, ArmorComponentRole, ArmorMorph, BoundaryNormals, PlateFace};

/// Plate side, metres.
const SIDE: f32 = 0.2;
const THICKNESS: f32 = 0.002;
/// Carrier cells across the plate: coarse, so the border cuts triangles.
const CELLS: u32 = 7;
const BAND: TrimBand = TrimBand {
    width: 0.03,
    period: 0.05,
};

/// A square plate thickened like a generated one, offset along x, with its
/// face roles and one morph lifting it a centimetre.
fn plate(boundary: BoundaryNormals, x: f32) -> GeneratedArmor {
    let row = CELLS + 1;
    let carrier = (0..CELLS)
        .flat_map(|j| {
            (0..CELLS).flat_map(move |i| {
                let [a, b, c, d] = [
                    j * row + i,
                    j * row + i + 1,
                    (j + 1) * row + i + 1,
                    (j + 1) * row + i,
                ];
                [a, b, c, a, c, d]
            })
        })
        .collect::<Vec<_>>();
    let plan = ShellPlan::new(row * row, &carrier, boundary).unwrap();
    let positions = plan
        .sources
        .iter()
        .map(|source| {
            let packed = source.packed();
            let carrier = packed & !crate::gpu::shell_plan::INNER_BIT;
            let depth = if packed == carrier { 0.0 } else { -THICKNESS };
            let cell = SIDE / CELLS as f32;
            [
                x + (carrier % row) as f32 * cell,
                (carrier / row) as f32 * cell,
                depth,
            ]
        })
        .collect::<Vec<_>>();
    let count = positions.len();
    GeneratedArmor {
        components: Vec::new(),
        design_hash: [0; 32],
        surface_domain: "test".into(),
        normals: vec![[0.0, 0.0, 1.0]; count],
        texcoords: positions.iter().map(|p| [p[0], p[1]]).collect(),
        joint_indices: positions
            .iter()
            .map(|p| [u32::from(p[0] - x > SIDE / 2.0), 0, 0, 0, 0, 0, 0, 0])
            .collect(),
        joint_weights: vec![[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; count],
        morphs: vec![ArmorMorph {
            name: "lift".into(),
            direct_positions: positions
                .iter()
                .map(|p| [p[0], p[1], p[2] + 0.01])
                .collect(),
            position_deltas: vec![[0.0, 0.0, 0.01]; count],
            normal_deltas: vec![[0.0; 3]; count],
        }],
        positions,
        indices: plan.indices.clone(),
        faces: plan.faces().collect(),
        trim: None,
    }
}

/// Distance from a point on the plate at `x` to its outline.
fn inset(p: [f32; 3], x: f32) -> f32 {
    let (u, v) = (p[0] - x, p[1]);
    u.min(SIDE - u).min(v).min(SIDE - v)
}

fn triangles(armor: &GeneratedArmor, range: Range<usize>) -> Vec<([u32; 3], PlateFace)> {
    range
        .step_by(3)
        .map(|at| {
            (
                std::array::from_fn(|k| armor.indices[at + k]),
                armor.faces[at / 3],
            )
        })
        .collect()
}

fn area(armor: &GeneratedArmor, corners: [u32; 3]) -> f32 {
    let [a, b, c] = corners.map(|v| armor.positions[v as usize]);
    let (e, f) = (edges::sub(b, a), edges::sub(c, a));
    let n = [
        e[1] * f[2] - e[2] * f[1],
        e[2] * f[0] - e[0] * f[2],
        e[0] * f[1] - e[1] * f[0],
    ];
    0.5 * (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt()
}

#[test]
fn the_band_follows_every_edge_at_its_width() {
    for boundary in [BoundaryNormals::Smooth, BoundaryNormals::Separate] {
        let armor = plate(boundary, 0.0).trimmed(BAND).unwrap();
        let [surface] = armor.surfaces().try_into().unwrap();
        let mut band_area = 0.0;
        for (corners, face) in triangles(&armor, surface.trim.clone()) {
            match face {
                PlateFace::Outer => {
                    band_area += area(&armor, corners);
                    for v in corners {
                        let p = armor.positions[v as usize];
                        assert!(inset(p, 0.0) <= BAND.width + 1e-5, "{p:?} is off the band");
                    }
                }
                PlateFace::Edge => {}
                PlateFace::Inner => panic!("the band is on the outer face"),
            }
        }
        for (corners, face) in triangles(&armor, surface.plate) {
            if face == PlateFace::Outer {
                for v in corners {
                    let p = armor.positions[v as usize];
                    assert!(inset(p, 0.0) >= BAND.width - 1e-5, "{p:?} is in the band");
                }
            }
            assert_ne!(face, PlateFace::Edge, "edge walls are trimmed");
        }
        // Exact along the straight runs; where the band turns a corner a
        // triangle whose corners all lie in it may reach past the border, by
        // at most one carrier cell at each of the four corners.
        let inside = SIDE - 2.0 * BAND.width;
        let exact = SIDE * SIDE - inside * inside;
        let cell = SIDE / CELLS as f32;
        assert!(
            band_area > exact - 1e-6 && band_area < exact + 4.0 * cell * cell,
            "{band_area} against {exact}"
        );
    }
}

#[test]
fn the_trimmed_plate_is_still_closed() {
    let armor = plate(BoundaryNormals::Separate, 0.0).trimmed(BAND).unwrap();
    let mut ids = BTreeMap::new();
    let welded = armor
        .positions
        .iter()
        .map(|p| {
            let next = ids.len();
            *ids.entry(p.map(|v| (v * 1e6).round() as i64))
                .or_insert(next)
        })
        .collect::<Vec<_>>();
    let mut uses = BTreeMap::<(usize, usize), i32>::new();
    for t in armor.indices.as_chunks::<3>().0 {
        let [a, b, c] = t.map(|v| welded[v as usize]);
        for (s, e) in [(a, b), (b, c), (c, a)] {
            *uses.entry((s, e)).or_default() += 1;
            *uses.entry((e, s)).or_default() -= 1;
        }
    }
    assert!(uses.values().all(|u| *u == 0));
}

#[test]
fn coordinates_run_on_along_a_closed_rim() {
    let armor = plate(BoundaryNormals::Smooth, 0.0).trimmed(BAND).unwrap();
    let trim = armor.trim.as_ref().unwrap();
    // 0.8 m round holds exactly sixteen 5 cm periods.
    let closure = 16.0 * BAND.period;
    let mut widest = 0.0f32;
    for (corners, face) in triangles(&armor, trim.bands[0].clone()) {
        let along = corners.map(|v| trim.coordinates[v as usize][0]);
        let low = along.iter().copied().fold(f32::MAX, f32::min);
        let high = along.iter().copied().fold(f32::MIN, f32::max);
        widest = widest.max(high - low);
        assert!(high <= closure + SIDE, "a copy runs at most one rim on");
        if face == PlateFace::Outer {
            for v in corners {
                assert!(trim.coordinates[v as usize][1] <= BAND.width + 1e-5);
            }
        }
    }
    // Neighbouring corners on a rim are at most a corner's reach apart.
    assert!(widest < SIDE, "a band triangle spans {widest} m of rim");
}

#[test]
fn cut_vertices_carry_skin_and_morphs() {
    let armor = plate(BoundaryNormals::Smooth, 0.0).trimmed(BAND).unwrap();
    for (index, p) in armor.positions.iter().enumerate() {
        let morph = &armor.morphs[0];
        assert!((morph.direct_positions[index][2] - p[2] - 0.01).abs() < 1e-6);
        assert!((morph.position_deltas[index][2] - 0.01).abs() < 1e-6);
        let total: f32 = armor.joint_weights[index].iter().sum();
        assert!((total - 1.0).abs() < 1e-5);
    }
}

#[test]
fn components_keep_their_own_vertices() {
    let mut first = plate(BoundaryNormals::Smooth, 0.0);
    let second = plate(BoundaryNormals::Smooth, 1.0);
    let (vertices, indices) = (first.positions.len(), first.indices.len());
    first.components = vec![
        ArmorComponent {
            role: ArmorComponentRole::Skull,
            vertices: 0..vertices,
            indices: 0..indices,
            hinge: None,
        },
        ArmorComponent {
            role: ArmorComponentRole::Visor,
            vertices: vertices..2 * vertices,
            indices: indices..2 * indices,
            hinge: None,
        },
    ];
    first.positions.extend(&second.positions);
    first.normals.extend(&second.normals);
    first.texcoords.extend(&second.texcoords);
    first.joint_indices.extend(&second.joint_indices);
    first.joint_weights.extend(&second.joint_weights);
    first
        .indices
        .extend(second.indices.iter().map(|i| i + vertices as u32));
    first.faces.extend(&second.faces);
    for (morph, other) in first.morphs.iter_mut().zip(&second.morphs) {
        morph.direct_positions.extend(&other.direct_positions);
        morph.position_deltas.extend(&other.position_deltas);
        morph.normal_deltas.extend(&other.normal_deltas);
    }
    let armor = first.trimmed(BAND).unwrap();
    let surfaces = armor.surfaces();
    assert_eq!(surfaces.len(), 2);
    for (surface, x) in surfaces.iter().zip([0.0, 1.0]) {
        let component = &armor.components[surface.component.unwrap()];
        assert_eq!(component.indices, surface.plate.start..surface.trim.end);
        assert!(!surface.trim.is_empty());
        for index in &armor.indices[component.indices.clone()] {
            assert!(component.vertices.contains(&(*index as usize)));
            let p = armor.positions[*index as usize];
            assert!(p[0] >= x - 1e-6 && p[0] <= x + SIDE + 1e-6);
        }
    }
    assert_eq!(armor.components[1].vertices.end, armor.positions.len());
}

#[test]
fn a_piece_without_plate_faces_cannot_be_trimmed() {
    let mut armor = plate(BoundaryNormals::Smooth, 0.0);
    armor.faces.clear();
    assert_eq!(armor.trimmed(BAND), Err(TrimError::NoPlateFaces));
    let armor = plate(BoundaryNormals::Smooth, 0.0);
    let band = TrimBand { width: 0.0, ..BAND };
    assert_eq!(armor.trimmed(band), Err(TrimError::InvalidBand));
}

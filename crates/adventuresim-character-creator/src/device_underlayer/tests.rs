use std::collections::BTreeMap;

use super::*;
use crate::surface_cut::{Plane, SurfaceCut};
use crate::underlayer::UnderlayerKind;
use fabelgeist_armor::{Millimeters, Permille};

const GRID: u32 = 24;
const SPAN_M: f32 = 0.2;
/// How far shading normals lean off their faces.
const TILT: f32 = 1.5;

/// Two wavy sheets facing each other across a narrow gap, the lower one
/// split by a seam of duplicated vertices, with shading normals tilted off
/// their faces so that directions need projecting and prisms fold.
struct Fixture {
    positions: Vec<[f32; 3]>,
    faces: Vec<[u32; 3]>,
    joint_indices: Vec<[u32; 8]>,
    joint_weights: Vec<[f32; 8]>,
}

impl Fixture {
    fn new() -> Self {
        let lower = |x: f32, y: f32| 0.006 * (90.0 * x).sin() * (25.0 * y).cos();
        let upper = |x: f32, y: f32| 0.014 + 0.004 * (20.0 * x + 1.0).sin() + 0.002 * y;
        let mut positions = Vec::new();
        let mut faces = Vec::new();
        for (height, facing_down) in [(&lower as &dyn Fn(f32, f32) -> f32, false), (&upper, true)] {
            let first = positions.len() as u32;
            for row in 0..GRID {
                for column in 0..GRID {
                    let (x, y) = (
                        column as f32 / (GRID - 1) as f32 * SPAN_M,
                        row as f32 / (GRID - 1) as f32 * SPAN_M,
                    );
                    positions.push([x, y, height(x, y)]);
                }
            }
            // The lower sheet's middle column is duplicated, and the
            // triangles right of it use the copies, as a UV seam does.
            let seam = GRID / 2;
            let copies = positions.len() as u32;
            if !facing_down {
                for row in 0..GRID {
                    positions.push(positions[(first + row * GRID + seam) as usize]);
                }
            }
            for row in 0..GRID - 1 {
                for column in 0..GRID - 1 {
                    let at = |r: u32, c: u32| {
                        if !facing_down && c == seam && column >= seam {
                            copies + r
                        } else {
                            first + r * GRID + c
                        }
                    };
                    let [a, b, c, d] = [
                        at(row, column),
                        at(row, column + 1),
                        at(row + 1, column),
                        at(row + 1, column + 1),
                    ];
                    if facing_down {
                        faces.extend([[a, c, b], [b, c, d]]);
                    } else {
                        faces.extend([[a, b, c], [b, d, c]]);
                    }
                }
            }
        }
        Self {
            joint_indices: vec![[0; 8]; positions.len()],
            joint_weights: vec![[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; positions.len()],
            positions,
            faces,
        }
    }

    /// Area-weighted vertex normals, each tilted its own way.
    fn normals(&self, positions: &[[f32; 3]]) -> Vec<[f32; 3]> {
        let mut sums = vec![[0.0f32; 3]; positions.len()];
        for face in &self.faces {
            let [a, b, c] = face.map(|v| positions[v as usize]);
            let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let n = [
                u[1] * v[2] - u[2] * v[1],
                u[2] * v[0] - u[0] * v[2],
                u[0] * v[1] - u[1] * v[0],
            ];
            for &i in face {
                for axis in 0..3 {
                    sums[i as usize][axis] += n[axis];
                }
            }
        }
        sums.into_iter()
            .enumerate()
            .map(|(i, n)| {
                let length = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
                // Uneven tilts shear the offset prisms until some fold.
                let (u, v) = ((i as f32 * 12.9898).sin(), (i as f32 * 78.233).cos());
                let tilted = [
                    n[0] / length + TILT * u,
                    n[1] / length + TILT * v,
                    n[2] / length,
                ];
                let length =
                    (tilted[0] * tilted[0] + tilted[1] * tilted[1] + tilted[2] * tilted[2]).sqrt();
                tilted.map(|v| v / length)
            })
            .collect()
    }

    fn moved(&self, amplitude: f32, frequency: f32) -> Vec<[f32; 3]> {
        self.positions
            .iter()
            .map(|p| {
                [
                    p[0] * (1.0 + amplitude),
                    p[1],
                    p[2] + amplitude * (frequency * p[0]).sin() * 0.05,
                ]
            })
            .collect()
    }
}

fn design() -> UnderlayerDesign {
    UnderlayerDesign {
        kind: UnderlayerKind::ArmingDoublet,
        clearance: Millimeters(4),
        thickness: Millimeters(3),
        length: Permille(1000),
        sleeve_length: Permille(1000),
        patch_width: Millimeters(80),
        cuts: vec![],
    }
}

fn wearer<'a>(
    fixture: &'a Fixture,
    positions: &'a [[f32; 3]],
    normals: &'a [[f32; 3]],
) -> Wearer<'a> {
    Wearer {
        positions,
        normals,
        faces: &fixture.faces,
        joint_indices: &fixture.joint_indices,
        joint_weights: &fixture.joint_weights,
        joint_names: &[],
        joints: &[],
    }
}

/// The lower sheet's middle, less a hole.
fn cut(fixture: &Fixture) -> SurfaceCut {
    let plane = |normal: [f32; 3], offset: f32| Plane { normal, offset };
    let include = vec![
        plane([1.0, 0.0, 0.0], 0.17),
        plane([-1.0, 0.0, 0.0], -0.03),
        plane([0.0, 1.0, 0.0], 0.16),
        plane([0.0, -1.0, 0.0], -0.02),
        plane([0.0, 0.0, 1.0], 0.009),
    ];
    let hole = vec![
        plane([1.0, 0.0, 0.0], 0.121),
        plane([-1.0, 0.0, 0.0], -0.083),
        plane([0.0, 1.0, 0.0], 0.117),
        plane([0.0, -1.0, 0.0], -0.071),
    ];
    SurfaceCut::new(
        &fixture.positions,
        &fixture.faces,
        &fixture.faces,
        &[include],
        &[hole],
    )
}

/// Every edge of the shell, its coincident vertices welded, is used once in
/// each direction.
fn assert_closed(positions: &[[f32; 3]], indices: &[u32]) {
    let mut welded = BTreeMap::new();
    let ids = positions
        .iter()
        .map(|p| {
            let next = welded.len();
            *welded
                .entry(p.map(|v| (v * 1e6).round() as i64))
                .or_insert(next)
        })
        .collect::<Vec<_>>();
    let mut directed = BTreeMap::new();
    for triangle in indices.as_chunks::<3>().0 {
        let [a, b, c] = std::array::from_fn(|i| ids[triangle[i] as usize]);
        for edge in [(a, b), (b, c), (c, a)] {
            *directed.entry(edge).or_insert(0) += 1;
        }
    }
    assert!(
        directed
            .iter()
            .all(|(&(a, b), &count)| count == 1 && directed.get(&(b, a)) == Some(&1)),
        "the shell is not closed"
    );
}

#[test]
fn a_cut_shell_is_closed_offset_by_its_layers_and_carries_surface_coordinates() {
    let gpu = ArmorGpu::open().unwrap();
    let positions = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [1., 1., 0.]];
    let normals = [[0., 0., 1.]; 4];
    let faces = [[0, 1, 2], [2, 1, 3]];
    let texcoords = positions.map(|p| [p[0], p[1]]);
    let body = Wearer {
        positions: &positions,
        normals: &normals,
        faces: &faces,
        joint_indices: &[[0; 8]; 4],
        joint_weights: &[[1., 0., 0., 0., 0., 0., 0., 0.]; 4],
        joint_names: &[],
        joints: &[],
    };
    let plane = |normal: [f32; 3], offset: f32| Plane { normal, offset };
    // A central opening cuts both triangles and their shared edge.
    let hole = vec![
        plane([1., 0., 0.], 0.6),
        plane([-1., 0., 0.], -0.4),
        plane([0., 1., 0.], 0.6),
        plane([0., -1., 0.], -0.4),
    ];
    let cut = SurfaceCut::new(&positions, &faces, &faces, &[vec![]], &[hole]);
    let plan = CutPlan::from_cut(&cut, &body, &faces);
    let design = UnderlayerDesign {
        thickness: Millimeters(1),
        ..design()
    };
    let fitted = Fit {
        gpu: &gpu,
        design: &design,
        body: &body,
        plan: &plan,
        proportions: &[],
        morphs: &[],
        domain: Some(SurfaceDomain {
            uv_faces: &faces,
            texcoords: &texcoords,
        }),
    }
    .run()
    .unwrap();
    let shell = &fitted.base.positions;
    assert_closed(shell, &fitted.indices);
    let count = plan.point_count as usize;
    let (outer, inner) = (
        design.clearance.metres() + design.thickness.metres(),
        design.clearance.metres(),
    );
    for i in 0..count {
        assert!((shell[i][2] - outer).abs() < 1e-6, "{:?}", shell[i]);
        assert!(
            (shell[i + count][2] - inner).abs() < 1e-6,
            "{:?}",
            shell[i + count]
        );
    }
    assert_eq!(fitted.texcoords.len(), shell.len());
    for (texcoord, vertex) in fitted.texcoords.iter().zip(shell) {
        assert!((texcoord[0] - vertex[0]).abs() < 1e-6 && (texcoord[1] - vertex[1]).abs() < 1e-6);
    }
    assert!(
        fitted
            .joint_weights
            .iter()
            .all(|w| (w[0] - 1.0).abs() < 1e-6)
    );
}

#[test]
fn a_frozen_envelope_keeps_the_layers_below_the_facing_sheet_on_every_sample() {
    let gpu = ArmorGpu::open().unwrap();
    let fixture = Fixture::new();
    let normals = fixture.normals(&fixture.positions);
    let body = wearer(&fixture, &fixture.positions, &normals);
    let proportion = fixture.moved(0.08, 40.0);
    let morphed = fixture.moved(-0.05, 70.0);
    let morph_normals = fixture.normals(&morphed);
    let design = design();
    let cut = cut(&fixture);
    let plan = CutPlan::from_cut(&cut, &body, &fixture.faces);
    let fitted = Fit {
        gpu: &gpu,
        design: &design,
        body: &body,
        plan: &plan,
        proportions: &[BodyShape {
            positions: &proportion,
            normals: &normals,
        }],
        morphs: &[BodyShape {
            positions: &morphed,
            normals: &morph_normals,
        }],
        domain: None,
    }
    .run()
    .unwrap();
    assert_eq!(fitted.endpoints.len(), 1);
    let base = &fitted.base.positions;
    assert_eq!(fitted.endpoints[0].positions.len(), base.len());
    assert_closed(base, &fitted.indices);
    // The unconstrained layers would reach 7 mm above the lower sheet,
    // through the upper one where they face each other.
    let upper = |p: [f32; 3]| 0.014 + 0.004 * (20.0 * p[0] + 1.0).sin() + 0.002 * p[1];
    for p in base {
        assert!(p.iter().all(|v| v.is_finite()));
        assert!(p[2] < upper(*p), "{p:?} passes the facing sheet");
    }
    let normals = &fitted.endpoints[0].normals;
    assert!(normals.iter().flatten().all(|v| v.is_finite()));
}

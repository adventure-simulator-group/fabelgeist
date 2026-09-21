//! Plate armor geometry built on the device.

use super::{designs, gpu};
use crate::{Armor, ArmorMesh, ArmorPart, Construction};

type Point = [f64; 3];

fn point(p: [f32; 3]) -> Point {
    p.map(f64::from)
}

fn sub(a: Point, b: Point) -> Point {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: Point, b: Point) -> Point {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: Point, b: Point) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn length(a: Point) -> f64 {
    dot(a, a).sqrt()
}

/// Each face's corners and doubled area vector.
fn faces(m: &ArmorMesh) -> impl Iterator<Item = ([Point; 3], Point)> + '_ {
    m.faces.iter().map(|f| {
        let p = f.map(|i| point(m.positions[i as usize]));
        (p, cross(sub(p[1], p[0]), sub(p[2], p[0])))
    })
}

/// Surface area, and the length of the summed area vectors, which is zero
/// for a closed surface.
fn closure(m: &ArmorMesh) -> (f64, f64) {
    let (area, sum) = faces(m).fold((0.0, [0.0; 3]), |(area, sum), (_, n)| {
        (
            area + length(n) / 2.0,
            [sum[0] + n[0], sum[1] + n[1], sum[2] + n[2]],
        )
    });
    (area, length(sum) / 2.0)
}

/// Enclosed volume, positive when the faces wind outward.
fn volume(m: &ArmorMesh) -> f64 {
    let origin = point(m.positions[0]);
    faces(m)
        .map(|(p, _)| {
            let [a, b, c] = p.map(|p| sub(p, origin));
            dot(a, cross(b, c)) / 6.0
        })
        .sum()
}

fn bounds(parts: &[ArmorPart]) -> [Point; 2] {
    parts.iter().flat_map(|part| &part.mesh.positions).fold(
        [[f64::INFINITY; 3], [f64::NEG_INFINITY; 3]],
        |[lo, hi], p| {
            let p = point(*p);
            [
                [0, 1, 2].map(|k| lo[k].min(p[k])),
                [0, 1, 2].map(|k| hi[k].max(p[k])),
            ]
        },
    )
}

/// The largest opening of a part, as a fraction of its area.
const CRACKS: f64 = 5e-3;

/// A part's volume over half its area, in plate thicknesses. Straight chords
/// between a tile's mapped corners cut into or bulge past the curved surface,
/// and front and back are triangulated differently.
const THICKNESS: std::ops::RangeInclusive<f64> = 0.25..=1.25;

/// The cosine below which a face turns against its corners' normals. A tile
/// straddling a sharp ridge has faces nearly edge-on to the smooth normals
/// of its corners, so only a clearly opposed face is wound wrongly.
const OPPOSED: f64 = 0.5;

fn is_fauld(part: &ArmorPart) -> bool {
    part.name.starts_with("Fauld layer")
}

/// Every part is a closed, outward-wound solid about as thick as the plate,
/// with finite attributes, unit normals and in-range indices.
#[test]
fn every_part_is_a_closed_outward_plate() {
    for (name, a) in designs() {
        let parts = gpu().build(&a).unwrap();
        for part in &parts {
            let m = &part.mesh;
            let label = format!("{name}: {}", part.name);
            assert!(!m.faces.is_empty(), "{label}: empty");
            assert_eq!(m.positions.len(), m.normals.len(), "{label}");
            assert_eq!(m.positions.len(), m.uvs.len(), "{label}");
            assert!(
                m.faces
                    .iter()
                    .flatten()
                    .all(|i| (*i as usize) < m.positions.len()),
                "{label}: index out of range"
            );
            assert!(
                m.positions.iter().flatten().all(|v| v.is_finite()),
                "{label}"
            );
            assert!(m.uvs.iter().flatten().all(|v| v.is_finite()), "{label}");
            for n in &m.normals {
                let n = length(point(*n));
                assert!((n - 1.0).abs() < 0.01, "{label}: normal length {n}");
            }
            // A tile follows the surface by its corners, so a vertex its
            // rivet holes split into an edge leaves a hairline crack.
            let (area, open) = closure(m);
            assert!(open < area * CRACKS, "{label}: open by {open} of {area} m²");
            let volume = volume(m);
            let thickness = 2.0 * volume / area / f64::from(a.thickness);
            assert!(
                THICKNESS.contains(&thickness),
                "{label}: {thickness} plate thicknesses"
            );
            // Each face turns the way its corners' normals do.
            let against = m
                .faces
                .iter()
                .zip(faces(m))
                .filter(|(f, (_, face))| {
                    let normal = f.iter().fold([0.0; 3], |sum, i| {
                        let n = point(m.normals[*i as usize]);
                        [sum[0] + n[0], sum[1] + n[1], sum[2] + n[2]]
                    });
                    dot(normal, *face) < -OPPOSED * length(normal) * length(*face)
                })
                .count();
            assert_eq!(against, 0, "{label}: faces wound against their normals");
        }
    }
}

/// A solid breastplate is one part and a tiled one a part per tile; every
/// fauld layer is its own part, in order, below the breastplate.
#[test]
fn parts_follow_the_construction() {
    for (name, a) in designs() {
        let parts = gpu().build(&a).unwrap();
        let (fauld, breastplate): (Vec<_>, Vec<_>) = parts.iter().partition(|p| is_fauld(p));
        let layers = (1..=a.fauld.layer_count).map(|i| format!("Fauld layer {i}"));
        assert!(
            fauld.iter().map(|p| p.name.clone()).eq(layers),
            "{name}: fauld layers"
        );
        if a.construction == Construction::Solid {
            assert_eq!(breastplate.len(), 1, "{name}");
            assert_eq!(breastplate[0].name, "Breastplate", "{name}");
        } else {
            assert!(breastplate.len() > 1, "{name}: one tile");
            assert!(breastplate.iter().all(|p| p.name.starts_with("Plate ")));
        }
        let mut top = f64::INFINITY;
        for layer in fauld {
            let [lo, _] = bounds(std::slice::from_ref(layer));
            assert!(lo[1] < top, "{name}: {} does not descend", layer.name);
            top = lo[1];
        }
    }
}

/// Rivet holes remove each hole's twelve-sided prism from every tile. The
/// armor is made as flat and the plate as thick as supported, so that the
/// tiles' chords barely leave the surface.
#[test]
fn rivet_holes_remove_their_volume() {
    let mut a = Armor {
        construction: Construction::Lamellar,
        width: 0.8,
        depth: 0.03,
        waist: 1.0,
        neck: 0.0,
        arm_cut: 0.0,
        thickness: 0.012,
        ridge: 0.0,
        center_point: 0.0,
        ..Armor::default()
    };
    a.fauld.layer_count = 0;
    a.plate.roundness = 0.0;
    a.plate.bevel = 0.0;
    a.plate.hole_pairs = 2;
    a.plate.hole_radius = 0.005;
    let radius = f64::from(a.plate.hole_radius);
    let pierced = gpu().build(&a).unwrap();
    a.plate.hole_radius = 0.0;
    let solid = gpu().build(&a).unwrap();
    assert_eq!(pierced.len(), solid.len());
    let sides = 12.0;
    let prism = sides / 2.0 * radius * radius * (std::f64::consts::TAU / sides).sin();
    let expected = 4.0 * prism * f64::from(a.thickness);
    for (pierced, solid) in pierced.iter().zip(&solid) {
        let removed = volume(&solid.mesh) - volume(&pierced.mesh);
        assert!(
            (removed - expected).abs() < expected * 0.1,
            "{}: removed {removed}, expected {expected}",
            pierced.name
        );
        assert!(pierced.mesh.faces.len() > solid.mesh.faces.len());
    }
}

/// The breastplate spans the design's width and height from its
/// translation, and the ridge and translation move it as they say.
#[test]
fn shape_controls_move_the_surface() {
    let mut a = Armor::default();
    a.fauld.layer_count = 0;
    a.ridge = 0.0;
    let flat = gpu().build(&a).unwrap();
    let [lo, hi] = bounds(&flat);
    let t = a.translation.map(f64::from);
    let tolerance = 1e-4;
    let span = hi[0] - lo[0];
    let width = f64::from(a.width);
    assert!(span > width * f64::from(a.waist) && span < width + tolerance);
    assert!((hi[1] - t[1] - f64::from(a.height)).abs() < tolerance);
    assert!((t[1] - lo[1] - f64::from(a.center_point)).abs() < tolerance);
    assert!(((hi[0] + lo[0]) / 2.0 - t[0]).abs() < tolerance);
    a.ridge = 0.1;
    let ridged = bounds(&gpu().build(&a).unwrap());
    assert!(ridged[1][2] - hi[2] > 0.09, "the ridge does not rise");
    a.ridge = 0.0;
    let shift = [0.2, -0.3, 0.1];
    a.translation = [0, 1, 2].map(|k| a.translation[k] + shift[k]);
    let moved = bounds(&gpu().build(&a).unwrap());
    for k in 0..3 {
        let s = f64::from(shift[k]);
        assert!((moved[0][k] - lo[k] - s).abs() < tolerance);
        assert!((moved[1][k] - hi[k] - s).abs() < tolerance);
    }
}

#[test]
fn builds_are_deterministic() {
    for (name, a) in designs() {
        let first = gpu().build(&a).unwrap();
        let second = gpu().build(&a).unwrap();
        assert_eq!(first.len(), second.len(), "{name}");
        for (x, y) in first.iter().zip(&second) {
            assert_eq!(x.name, y.name, "{name}");
            assert_eq!(x.mesh.faces, y.mesh.faces, "{name}: {}", x.name);
            assert_eq!(x.mesh.positions, y.mesh.positions, "{name}: {}", x.name);
            assert_eq!(x.mesh.normals, y.mesh.normals, "{name}: {}", x.name);
            assert_eq!(x.mesh.uvs, y.mesh.uvs, "{name}: {}", x.name);
        }
    }
}

#[test]
fn invalid_armor_is_rejected() {
    let a = Armor {
        width: f32::NAN,
        ..Armor::default()
    };
    assert!(gpu().build(&a).is_err());
    let mut a = Armor::default();
    a.fauld.layer_count = 13;
    assert!(gpu().build(&a).is_err());
    let mut a = Armor::default();
    a.plate.hole_radius = a.plate.width / 4.0;
    assert!(gpu().build(&a).is_err());
}

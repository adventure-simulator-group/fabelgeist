//! The device and the mesh measurements shared by the armor tests.
#![allow(
    dead_code,
    reason = "each test crate uses its own subset of these helpers"
)]

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use fabelgeist_armor::{ArmorComponentRole, ArmorGpu, BuiltPart, PartFrame};

/// One device for every test in a test crate.
pub fn gpu() -> &'static ArmorGpu {
    static GPU: OnceLock<ArmorGpu> = OnceLock::new();
    GPU.get_or_init(|| ArmorGpu::open().expect("open the armor device"))
}

/// An unrotated frame at `origin`.
pub fn frame(origin: [f32; 3], half_extents: [f32; 3]) -> PartFrame {
    PartFrame {
        origin,
        axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        half_extents,
    }
}

/// `frame` mirrored across its own sagittal plane, as a left limb is.
pub fn reflected(mut frame: PartFrame) -> PartFrame {
    frame.axes[0] = frame.axes[0].map(|v| -v);
    frame
}

pub fn scaled(mut frame: PartFrame, scale: f32) -> PartFrame {
    frame.half_extents = frame.half_extents.map(|v| v * scale);
    frame
}

/// Axis-aligned bounds of `points`: lowest corner, then highest.
pub fn bounds<'a>(points: impl IntoIterator<Item = &'a [f32; 3]>) -> [[f32; 3]; 2] {
    points.into_iter().fold(
        [[f32::INFINITY; 3], [f32::NEG_INFINITY; 3]],
        |[low, high], p| {
            [
                std::array::from_fn(|i| low[i].min(p[i])),
                std::array::from_fn(|i| high[i].max(p[i])),
            ]
        },
    )
}

pub fn extent(points: &[[f32; 3]], axis: usize) -> f32 {
    let [low, high] = bounds(points);
    high[axis] - low[axis]
}

pub fn largest_difference(a: &[[f32; 3]], b: &[[f32; 3]]) -> f32 {
    assert_eq!(a.len(), b.len());
    a.iter()
        .zip(b)
        .flat_map(|(a, b)| (0..3).map(move |i| (a[i] - b[i]).abs()))
        .fold(0.0, f32::max)
}

/// Physical vertex ids: the device duplicates a wall vertex wherever a
/// boundary keeps separate normals, and the copies are bitwise equal.
fn welded(positions: &[[f32; 3]]) -> Vec<u32> {
    let mut ids = BTreeMap::<[u32; 3], u32>::new();
    positions
        .iter()
        .map(|p| {
            let next = ids.len() as u32;
            *ids.entry(p.map(f32::to_bits)).or_insert(next)
        })
        .collect()
}

/// The part's closed shells: each a set of triangle indices joined by
/// shared physical vertices.
pub fn shells(part: &BuiltPart) -> Vec<Vec<usize>> {
    let ids = welded(&part.positions);
    let triangles = part.indices.as_chunks::<3>().0;
    let mut parent = (0..part.positions.len()).collect::<Vec<_>>();
    fn root(parent: &mut [usize], mut i: usize) -> usize {
        while parent[i] != i {
            parent[i] = parent[parent[i]];
            i = parent[i];
        }
        i
    }
    for triangle in triangles {
        let [a, b, c] = triangle.map(|i| ids[i as usize] as usize);
        for (x, y) in [(a, b), (b, c)] {
            let (x, y) = (root(&mut parent, x), root(&mut parent, y));
            parent[x] = y;
        }
    }
    let mut groups = BTreeMap::<usize, Vec<usize>>::new();
    for (t, triangle) in triangles.iter().enumerate() {
        let r = root(&mut parent, ids[triangle[0] as usize] as usize);
        groups.entry(r).or_default().push(t);
    }
    groups.into_values().collect()
}

/// Signed volume enclosed by some of the part's triangles, measured about
/// their first vertex so distant placement does not cost precision.
pub fn volume(part: &BuiltPart, triangles: &[usize]) -> f64 {
    let at = |i: u32| part.positions[i as usize].map(f64::from);
    let Some(first) = triangles.first() else {
        return 0.0;
    };
    let o = at(part.indices[first * 3]);
    triangles
        .iter()
        .map(|t| {
            let [a, b, c] = std::array::from_fn(|k| {
                let p = at(part.indices[t * 3 + k]);
                [p[0] - o[0], p[1] - o[1], p[2] - o[2]]
            });
            (a[0] * (b[1] * c[2] - b[2] * c[1])
                + a[1] * (b[2] * c[0] - b[0] * c[2])
                + a[2] * (b[0] * c[1] - b[1] * c[0]))
                / 6.0
        })
        .sum()
}

/// Total area of some of the part's triangles.
pub fn area(part: &BuiltPart, triangles: &[usize]) -> f64 {
    triangles
        .iter()
        .map(|t| {
            let [a, b, c] =
                std::array::from_fn(|k| part.positions[part.indices[t * 3 + k] as usize]);
            let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]].map(f64::from);
            let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]].map(f64::from);
            let n = [
                u[1] * v[2] - u[2] * v[1],
                u[2] * v[0] - u[0] * v[2],
                u[0] * v[1] - u[1] * v[0],
            ];
            (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt() / 2.0
        })
        .sum()
}

/// Every value finite, every index in range, one unit normal per vertex,
/// every physical edge shared by exactly two oppositely wound triangles, and
/// every shell enclosing positive volume: an outward, watertight solid.
pub fn assert_closed_solid(part: &BuiltPart, context: &str) {
    assert!(!part.indices.is_empty(), "{context}: empty part");
    assert_eq!(part.indices.len() % 3, 0, "{context}: partial triangle");
    assert_eq!(part.normals.len(), part.positions.len(), "{context}");
    assert!(
        part.positions.iter().flatten().all(|v| v.is_finite()),
        "{context}: non-finite position"
    );
    assert!(
        part.normals
            .iter()
            .all(|n| ((n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt() - 1.0).abs() < 1e-3),
        "{context}: normal is not unit length"
    );
    assert!(
        part.indices
            .iter()
            .all(|&i| (i as usize) < part.positions.len()),
        "{context}: index out of range"
    );
    let ids = welded(&part.positions);
    let mut edges = BTreeMap::<(u32, u32), Vec<(u32, u32)>>::new();
    for triangle in part.indices.as_chunks::<3>().0 {
        let [a, b, c] = triangle.map(|i| ids[i as usize]);
        assert!(a != b && b != c && c != a, "{context}: collapsed triangle");
        for (start, end) in [(a, b), (b, c), (c, a)] {
            edges
                .entry((start.min(end), start.max(end)))
                .or_default()
                .push((start, end));
        }
    }
    for uses in edges.values() {
        assert_eq!(uses.len(), 2, "{context}: edge without exactly two faces");
        assert_eq!(
            uses[0],
            (uses[1].1, uses[1].0),
            "{context}: neighbouring faces wound inconsistently"
        );
    }
    for shell in shells(part) {
        let enclosed = volume(part, &shell);
        assert!(
            enclosed > 1e-9,
            "{context}: a shell encloses {enclosed} m³; outward walls enclose positive volume"
        );
    }
}

/// The genus of a closed component: how many holes pass through it.
pub fn genus(part: &BuiltPart, role: ArmorComponentRole) -> i64 {
    let component = component(part, role);
    let ids = welded(&part.positions);
    let triangles = part.indices[component.indices.clone()].as_chunks::<3>().0;
    let mut vertices = BTreeSet::new();
    let mut edges = BTreeSet::new();
    for triangle in triangles {
        let [a, b, c] = triangle.map(|i| ids[i as usize]);
        vertices.extend([a, b, c]);
        for (x, y) in [(a, b), (b, c), (c, a)] {
            edges.insert((x.min(y), x.max(y)));
        }
    }
    let euler = vertices.len() as i64 - edges.len() as i64 + triangles.len() as i64;
    (2 - euler) / 2
}

pub fn component(part: &BuiltPart, role: ArmorComponentRole) -> &fabelgeist_armor::ArmorComponent {
    part.components
        .iter()
        .find(|c| c.role == role)
        .unwrap_or_else(|| panic!("no {role:?} component"))
}

/// The mean distance between a thin shell's walls: twice its volume over
/// its surface area, which ignores the small area of its cut edges.
pub fn mean_gauge(part: &BuiltPart) -> f64 {
    let all = (0..part.indices.len() / 3).collect::<Vec<_>>();
    2.0 * volume(part, &all) / area(part, &all)
}

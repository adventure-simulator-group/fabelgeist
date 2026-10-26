//! Closed geometry must have coherent exterior winding and one fan per vertex.
use fabelgeist_armor::GeneratedArmor;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn assert_closed_oriented_manifold(armor: &GeneratedArmor) {
    let mut ids = BTreeMap::new();
    let mut points = Vec::new();
    let vertices = armor
        .positions
        .iter()
        .map(|&p| {
            let key = p.map(|v| if v == 0.0 { 0 } else { v.to_bits() });
            *ids.entry(key).or_insert_with(|| {
                points.push(p.map(f64::from));
                points.len() - 1
            })
        })
        .collect::<Vec<_>>();
    let faces = armor
        .indices
        .as_chunks::<3>()
        .0
        .iter()
        .map(|f| f.map(|i| vertices[i as usize]))
        .collect::<Vec<_>>();
    let mut edges = BTreeMap::<_, Vec<_>>::new();
    let mut incident = vec![Vec::new(); points.len()];
    for (id, face) in faces.iter().enumerate() {
        assert!(face[0] != face[1] && face[1] != face[2] && face[2] != face[0]);
        for &vertex in face {
            incident[vertex].push(id);
        }
        for i in 0..3 {
            let (a, b) = (face[i], face[(i + 1) % 3]);
            edges
                .entry([a.min(b), a.max(b)])
                .or_default()
                .push((a, b, id));
        }
    }
    let mut neighbors = vec![Vec::new(); faces.len()];
    for (edge, uses) in edges {
        let [(a, b, first), (c, d, second)] = *uses.as_slice() else {
            panic!("nonmanifold edge {edge:?}: {uses:?}");
        };
        assert!(a == d && b == c, "incoherent winding at {edge:?}");
        neighbors[first].push((second, edge));
        neighbors[second].push((first, edge));
    }
    for (vertex, faces) in incident.iter().enumerate() {
        let Some(&first) = faces.first() else {
            continue;
        };
        let mut fan = BTreeSet::new();
        let mut pending = vec![first];
        while let Some(face) = pending.pop() {
            if !fan.insert(face) {
                continue;
            }
            pending.extend(
                neighbors[face]
                    .iter()
                    .filter(|(_, edge)| edge.contains(&vertex))
                    .map(|&(id, _)| id),
            );
        }
        assert_eq!(
            fan.len(),
            faces.len(),
            "disconnected vertex fan at {:?}",
            points[vertex]
        );
    }
    let mut seen = vec![false; faces.len()];
    for first in 0..faces.len() {
        if seen[first] {
            continue;
        }
        let origin = points[faces[first][0]];
        let mut volume = 0.0;
        let mut pending = vec![first];
        while let Some(face) = pending.pop() {
            if seen[face] {
                continue;
            }
            seen[face] = true;
            let [a, b, c] = faces[face].map(|i| std::array::from_fn(|k| points[i][k] - origin[k]));
            volume += super::dot(a, super::cross(b, c)) / 6.0;
            pending.extend(neighbors[face].iter().map(|&(id, _)| id));
        }
        assert!(
            volume.is_finite() && volume > 0.0,
            "shell material has nonpositive volume {volume} m^3"
        );
    }
}

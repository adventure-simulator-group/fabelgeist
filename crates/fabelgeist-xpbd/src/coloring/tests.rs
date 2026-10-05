use super::*;
use crate::{ConstraintArity, ConstraintEdges, ConstraintOccupancy};

/// No constraints are lost and distinct constraints in a color share no particle.
fn check(constraints: &[&[ParticleIndex]], coloring: &Coloring) {
    assert_eq!(
        coloring.constraint_count(),
        ConstraintCount::from(constraints.len()),
        "lost or gained constraints"
    );
    let mut seen = vec![false; constraints.len()];
    for &index in coloring.order() {
        assert!(
            !seen[usize::from(index)],
            "constraint {index} appears twice"
        );
        seen[usize::from(index)] = true;
    }
    for seen in seen {
        assert!(seen, "order is not a permutation");
    }
    for color in coloring.colors() {
        let mut used = std::collections::HashSet::new();
        for slot in color.slots() {
            let constraint = coloring.order()[usize::from(slot)];
            for &particle in constraints[usize::from(constraint)] {
                assert!(
                    used.insert(particle),
                    "colour {} uses particle {particle} twice",
                    color.color()
                );
            }
        }
    }
}

#[test]
fn colors_a_chain() {
    let mut records = Vec::new();
    for i in 0..100u32 {
        records.push([i, i + 1]);
    }
    let edges = ConstraintEdges::from(records.as_slice());
    let slices: Vec<&[ParticleIndex]> = edges
        .pairs()
        .iter()
        .map(<[ParticleIndex; 2]>::as_slice)
        .collect();
    let coloring = Coloring::for_constraints(&slices);
    check(&slices, &coloring);
    assert_eq!(
        coloring.color_count(),
        ColorCount::from(2),
        "a chain needs exactly two colours"
    );
}

#[test]
fn colors_a_star() {
    let mut records = Vec::new();
    for i in 1..20u32 {
        records.push([0, i]);
    }
    let edges = ConstraintEdges::from(records.as_slice());
    let slices: Vec<&[ParticleIndex]> = edges
        .pairs()
        .iter()
        .map(<[ParticleIndex; 2]>::as_slice)
        .collect();
    let coloring = Coloring::for_constraints(&slices);
    check(&slices, &coloring);
    assert_eq!(coloring.color_count(), ColorCount::from(19));
}

#[test]
fn colors_a_grid() {
    let (width, height) = (32u32, 32u32);
    let mut records = Vec::new();
    for y in 0..height {
        for x in 0..width {
            if x + 1 < width {
                records.push([y * width + x, y * width + x + 1]);
            }
            if y + 1 < height {
                records.push([y * width + x, (y + 1) * width + x]);
            }
        }
    }
    let edges = ConstraintEdges::from(records.as_slice());
    let slices: Vec<&[ParticleIndex]> = edges
        .pairs()
        .iter()
        .map(<[ParticleIndex; 2]>::as_slice)
        .collect();
    let coloring = Coloring::for_constraints(&slices);
    check(&slices, &coloring);
    assert!(
        (ColorCount::from(4)..=ColorCount::from(6)).contains(&coloring.color_count()),
        "expected 4-6 colours, got {}",
        coloring.color_count()
    );
}

#[test]
fn colors_mixed_arity() {
    let records = vec![
        vec![0u32, 1],
        vec![1, 2, 3, 4],
        vec![4, 5],
        vec![0, 5],
        vec![2, 6, 7],
    ];
    let mut constraints = Vec::new();
    for record in records {
        constraints.push(
            record
                .into_iter()
                .map(ParticleIndex::from)
                .collect::<Vec<_>>(),
        );
    }
    let slices: Vec<&[ParticleIndex]> = constraints.iter().map(Vec::as_slice).collect();
    let coloring = Coloring::for_constraints(&slices);
    check(&slices, &coloring);
}

#[test]
fn colors_nothing() {
    let coloring = Coloring::for_constraints(&[]);
    assert_eq!(
        coloring.constraint_count().occupancy(),
        ConstraintOccupancy::Empty
    );
    assert_eq!(coloring.color_count(), ColorCount::from(0));
}

#[test]
fn independent_constraints_share_one_color() {
    let mut records = Vec::new();
    for i in 0..50u32 {
        records.push([i * 2, i * 2 + 1]);
    }
    let edges = ConstraintEdges::from(records.as_slice());
    let slices: Vec<&[ParticleIndex]> = edges
        .pairs()
        .iter()
        .map(<[ParticleIndex; 2]>::as_slice)
        .collect();
    let coloring = Coloring::for_constraints(&slices);
    check(&slices, &coloring);
    assert_eq!(coloring.color_count(), ColorCount::from(1));
}

#[test]
fn fixed_arity_matches_the_general_form() {
    let flat = vec![0u32, 1, 1, 2, 2, 3, 3, 0];
    let incidence =
        ConstraintIncidence::from_native_flat(&flat, ConstraintArity::try_from(2).unwrap())
            .unwrap();
    let chunks: Vec<&[ParticleIndex]> = incidence.records().collect();
    assert_eq!(
        Coloring::for_incidence(&incidence).order(),
        Coloring::for_constraints(&chunks).order()
    );
}

#[test]
fn preserves_frozen_original_order_and_ranges_for_309_graphs() {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("../../tests/fixtures/coloring.json")).unwrap();
    assert_eq!(cases.as_array().unwrap().len(), 309);
    for case in cases.as_array().unwrap() {
        let mut constraints = Vec::new();
        for record in case["constraints"].as_array().unwrap() {
            let mut particles = Vec::new();
            for word in record.as_array().unwrap() {
                particles.push(ParticleIndex::from(word.as_u64().unwrap() as u32));
            }
            constraints.push(particles);
        }
        let refs: Vec<&[ParticleIndex]> = constraints.iter().map(Vec::as_slice).collect();
        let coloring = Coloring::for_constraints(&refs);
        let mut order = Vec::new();
        for index in case["order"].as_array().unwrap() {
            order.push(ConstraintIndex::from(index.as_u64().unwrap() as u32));
        }
        let mut ranges = Vec::new();
        for slot in case["ranges"].as_array().unwrap() {
            ranges.push(ConstraintSlot(slot.as_u64().unwrap() as u32));
        }
        assert_eq!(coloring.order, order, "graph {constraints:?}");
        assert_eq!(coloring.ranges, ranges, "graph {constraints:?}");
    }
}

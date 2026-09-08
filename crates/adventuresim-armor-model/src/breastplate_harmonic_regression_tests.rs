//! Frozen pre-optimization solver and bounded real-chart comparison harness.
//! The attempted buffer/row-sum optimization was reverted after no measured gain.
use super::*;

fn old_jacobi<const D: usize>(
    mut values: Vec<[f32; D]>,
    fixed: &[bool],
    neighbors: &[Vec<(usize, f32)>],
) -> Vec<[f32; D]> {
    for _ in 0..1200 {
        let old = values.clone();
        let mut delta = 0.0_f32;
        for vertex in 0..values.len() {
            if fixed[vertex] {
                continue;
            }
            let total = neighbors[vertex]
                .iter()
                .map(|neighbor| neighbor.1)
                .sum::<f32>();
            for axis in 0..D {
                values[vertex][axis] = neighbors[vertex]
                    .iter()
                    .map(|(neighbor, weight)| old[*neighbor][axis] * weight)
                    .sum::<f32>()
                    / total.max(1e-8);
                delta = delta.max((values[vertex][axis] - old[vertex][axis]).abs());
            }
        }
        if delta < 1e-7 {
            break;
        }
    }
    values
}

fn old_fixed_harmonic<const D: usize>(
    targets: &[Option<[f32; D]>],
    neighbors: &[Vec<(usize, f32)>],
    reference: &[[f32; D]],
) -> Vec<[f32; D]> {
    let fixed = targets
        .iter()
        .zip(reference)
        .map(|(target, origin)| {
            target.map(|point| std::array::from_fn(|axis| point[axis] - origin[axis]))
        })
        .collect::<Vec<_>>();
    let fixed_values = fixed.iter().flatten().copied().collect::<Vec<_>>();
    let center = fixed_values.iter().fold([0.0; D], |mut result, point| {
        for axis in 0..D {
            result[axis] += point[axis] / fixed_values.len() as f32;
        }
        result
    });
    let values = fixed
        .iter()
        .map(|point| point.unwrap_or(center))
        .collect::<Vec<_>>();
    old_jacobi(
        values,
        &fixed.iter().map(Option::is_some).collect::<Vec<_>>(),
        neighbors,
    )
    .into_iter()
    .zip(reference)
    .zip(targets)
    .map(|((delta, origin), target)| {
        target.unwrap_or_else(|| std::array::from_fn(|axis| origin[axis] + delta[axis]))
    })
    .collect()
}

fn assert_float_bits_equal<const D: usize>(left: &[[f32; D]], right: &[[f32; D]]) {
    assert_eq!(
        left.iter()
            .flatten()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        right
            .iter()
            .flatten()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    );
}

#[test]
fn harmonic_solver_matches_frozen_reference_for_affine_nonaffine_and_invalid_data() {
    let (boundary, _) = canonical_boundary(Default::default());
    let topology = CanonicalBreastplateTopology::from_metric_chart_with_candidates(
        &boundary,
        &boundary,
        &[],
        0.5,
        false,
        |point| point,
    );
    let reference = topology
        .canonical_positions
        .iter()
        .map(|p| [p[0], p[1], 0.2 * p[0] + p[1]])
        .collect::<Vec<_>>();
    let count = topology.boundary_vertices.len();
    for variant in 0..4 {
        let transform = |p: [f32; 3]| match variant {
            0 => p,
            1 => [1.02 * p[0] + 0.004, 0.98 * p[1] - 0.01, p[2] + 0.005],
            2 => [
                p[0] + 0.015 * (7.0 * p[1]).sin(),
                p[1] + 0.006 * (11.0 * p[0]).cos(),
                p[2] + 0.01 * p[0] * p[1],
            ],
            _ => [0.0; 3], // Solver equivalence even for a collapsed target; no acceptance exemption.
        };
        let fixed = reference
            .iter()
            .enumerate()
            .map(|(index, &point)| {
                (index < count * 2 || topology.is_structured_spoke_sample(index))
                    .then_some(transform(point))
            })
            .collect::<Vec<_>>();
        assert_float_bits_equal(
            &old_fixed_harmonic(&fixed, &topology.harmonic_neighbors, &reference),
            &harmonic_extension_with_fixed(&fixed, &topology.harmonic_neighbors, &reference),
        );
        let target_boundary = reference[..count]
            .iter()
            .copied()
            .map(transform)
            .collect::<Vec<_>>();
        let mut values = vec![[0.0; 3]; reference.len()];
        values[..count].copy_from_slice(&target_boundary);
        let center = target_boundary.iter().fold([0.0; 3], |mut sum, p| {
            for axis in 0..3 {
                sum[axis] += p[axis] / count as f32;
            }
            sum
        });
        values[count..].fill(center);
        let expected = old_jacobi(
            values,
            &(0..reference.len())
                .map(|index| index < count)
                .collect::<Vec<_>>(),
            &topology.harmonic_neighbors,
        );
        assert_float_bits_equal(
            &expected,
            &harmonic_extension(&target_boundary, &topology.harmonic_neighbors),
        );
    }
}

#[test]
#[ignore = "requires BREASTPLATE_HARMONIC_FIXTURE pointing at a real profile.mesh.json"]
fn real_chart_harmonic_bit_equivalence_and_bounded_timing() {
    let path = std::env::var("BREASTPLATE_HARMONIC_FIXTURE").expect("fixture path");
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let reference: Vec<[f32; 2]> =
        serde_json::from_value(json["reference_domain"].clone()).unwrap();
    let indices: Vec<u32> = serde_json::from_value(json["indices"].clone()).unwrap();
    let boundary_count = json["boundary_count"].as_u64().unwrap() as usize;
    let structured: Vec<usize> =
        serde_json::from_value(json["structured_samples"].clone()).unwrap();
    let neighbors = harmonic_neighbors(&reference, indices.as_chunks::<3>().0);
    let first_structured = *structured.iter().min().unwrap();
    let pinned = |index: usize| {
        index < boundary_count * 2
            || (index >= boundary_count * 2 + semantic_cap_vertex_count()
                && index < first_structured)
            || structured.contains(&index)
    };
    let mut old_time = std::time::Duration::ZERO;
    let mut new_time = std::time::Duration::ZERO;
    for variant in 0..3 {
        let fixed = reference
            .iter()
            .enumerate()
            .map(|(index, &point)| {
                pinned(index).then_some(match variant {
                    0 => point,
                    1 => [point[0] * 1.1 + 0.003, point[1] * 0.99 - 0.002],
                    _ => [
                        point[0] + 0.008 * (8.0 * point[1]).sin(),
                        point[1] + 0.004 * (12.0 * point[0]).cos(),
                    ],
                })
            })
            .collect::<Vec<_>>();
        for repeat in 0..4 {
            let mut run_old = || {
                let start = std::time::Instant::now();
                let result = old_fixed_harmonic(&fixed, &neighbors, &reference);
                old_time += start.elapsed();
                result
            };
            let mut run_new = || {
                let start = std::time::Instant::now();
                let result = harmonic_extension_with_fixed(&fixed, &neighbors, &reference);
                new_time += start.elapsed();
                result
            };
            let (old, new) = if repeat % 2 == 0 {
                (run_old(), run_new())
            } else {
                let new = run_new();
                (run_old(), new)
            };
            assert_float_bits_equal(&old, &new);
        }
    }
    eprintln!(
        "real chart {} sites:12 solves old={old_time:?} current={new_time:?} reference_over_current={:.3}",
        reference.len(),
        old_time.as_secs_f64() / new_time.as_secs_f64()
    );
}

//! Directional triangle sections shared by collar and bib fitting.

pub(super) const PROJECTION: &str = r#"
const MIN_PROJECTED_DOUBLE_AREA_M2: f32 = 1e-10;
// A determinant uses four coordinate differences, two products and a sum.
// For normal finite f32 operands, 16 spacings near one conservatively bounds
// their accumulated error relative to the sum of absolute product terms.
// This budget rejects unresolved projections; it never widens triangle edges.
const DETERMINANT_ROUNDOFF: f32 = 16.0 * 1.1920929e-7;
const SECTION_HEIGHT_TOLERANCE_M: f32 = 1e-6;

fn projected_edge(a: vec2<f32>, b: vec2<f32>, p: vec2<f32>) -> f32 {
    // Adjacent triangles must evaluate their shared edge from the same anchor.
    // Reversing traversal then only negates the result, so rounding cannot
    // leave a crack where both triangles reject the same near-edge query.
    let reversed = a.x > b.x || (a.x == b.x && a.y > b.y);
    let origin = select(a, b, reversed);
    let end = select(b, a, reversed);
    let edge = end - origin;
    let query = p - origin;
    let area = edge.x * query.y - edge.y * query.x;
    return select(area, -area, reversed);
}

// Where a depth ray crosses a triangle rotated into the ray's directional
// section; `y` is one when it does.
fn ray_depth(p: vec2<f32>, a: vec3<f32>, b: vec3<f32>, c: vec3<f32>) -> vec2<f32> {
    if (any(p < min(a.xy, min(b.xy, c.xy))) ||
        any(p > max(a.xy, max(b.xy, c.xy)))) {
        return vec2<f32>(0.0, 0.0);
    }
    let terms = vec2<f32>((b.y-c.y)*(a.x-c.x), (c.x-b.x)*(a.y-c.y));
    let determinant = terms.x + terms.y;
    let unresolved_area = DETERMINANT_ROUNDOFF * (abs(terms.x) + abs(terms.y));
    if (abs(determinant) < MIN_PROJECTED_DOUBLE_AREA_M2 ||
        abs(determinant) <= unresolved_area) {
        return vec2<f32>(0.0, 0.0);
    }
    // Test each edge directly. Subtracting two rounded coordinates from one
    // to obtain the third could conceal an exterior query at that edge.
    let weights = sign(determinant) * vec3<f32>(
        projected_edge(b.xy, c.xy, p),
        projected_edge(c.xy, a.xy, p),
        projected_edge(a.xy, b.xy, p),
    );
    let total = weights.x + weights.y + weights.z;
    if (any(weights < vec3<f32>(0.0)) || !(total > 0.0)) {
        return vec2<f32>(0.0, 0.0);
    }
    // Nonnegative normalized weights interpolate depth inside the triangle's
    // depth range. Clamp the final rounding so fitting never extrapolates it.
    let depths = vec3<f32>(a.z, b.z, c.z);
    let depth = clamp(dot(weights / total, depths), min(a.z, min(b.z, c.z)),
        max(a.z, max(b.z, c.z)));
    return vec2<f32>(depth, 1.0);
}

fn rotated(b: vec3<f32>, direction: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        b.x,
        b.y * direction.z - b.z * direction.y,
        b.y * direction.y + b.z * direction.z,
    );
}

"#;

#[cfg(test)]
mod tests {
    use super::*;
    use fabelgeist_armor::ArmorGpu;
    use fabelgeist_gpu::prelude::{BufferUpload, PassParameters};

    #[test]
    fn near_edge_on_projections_reject_exterior_queries_without_depth_extrapolation() {
        let mut queries = Vec::<[[f32; 4]; 4]>::new();
        let mut expected = Vec::new();
        for scale in [0.01, 1.0, 100.0] {
            for reverse in [false, true] {
                let a = [0.0, 0.0, 2.0, 0.0];
                let mut b = [0.1 * scale, 0.1 * scale, 4.0, 0.0];
                let mut c = [0.2 * scale, 0.2000001 * scale, 6.0, 0.0];
                if reverse {
                    std::mem::swap(&mut b, &mut c);
                }
                for p in [[-0.1, -0.1], [0.1, 0.10000009], [0.1, 0.100000025]] {
                    queries.push([[p[0] * scale, p[1] * scale, 0.0, 0.0], a, b, c]);
                    expected.push(None);
                }
                let mut b = [0.1 * scale, 0.0, 4.0, 0.0];
                let mut c = [0.0, 0.1 * scale, 6.0, 0.0];
                if reverse {
                    std::mem::swap(&mut b, &mut c);
                }
                for (p, depth) in [
                    ([0.025, 0.025], Some(3.5)),
                    ([0.0, 0.0], Some(2.0)),
                    ([0.05, 0.0], Some(3.0)),
                    ([0.075, 0.075], None),
                ] {
                    queries.push([[p[0] * scale, p[1] * scale, 0.0, 0.0], a, b, c]);
                    expected.push(depth);
                }
            }
        }
        let shared_edge_start = queries.len();
        for scale in [0.01, 1.0, 100.0] {
            // These adjacent triangles used to round their common edge in
            // opposite ways and both reject P, despite healthy determinants.
            let [a, b, c, d, p] = [
                [-0.14273149, 0.019695438],
                [0.055748522, -0.18255578],
                [0.057634126, 0.017809834],
                [-0.1446171, -0.18067017],
                [0.00835561, -0.13426238],
            ]
            .map(|xy| [xy[0] * scale, xy[1] * scale, 0.1, 0.0]);
            queries.extend([[p, a, b, c], [p, b, a, d], [p, a, c, b], [p, b, d, a]]);
        }
        let gpu = ArmorGpu::open().unwrap();
        let input = gpu.upload(BufferUpload::from_elements(&queries)).unwrap();
        let output = gpu
            .scratch(
                (queries.len() * size_of::<[f32; 4]>()) as u64,
                "projection depths",
            )
            .unwrap();
        let source = format!(
            "{PROJECTION}\n{}",
            r#"
            struct Query { p: vec4<f32>, a: vec4<f32>, b: vec4<f32>, c: vec4<f32> }
            @group(0) @binding(0) var<storage, read> queries: array<Query>;
            @group(0) @binding(1) var<storage, read_write> results: array<vec4<f32>>;
            @compute @workgroup_size(64)
            fn main(@builtin(global_invocation_id) id: vec3<u32>) {
                if (id.x >= arrayLength(&queries)) { return; }
                let q = queries[id.x];
                results[id.x] = vec4<f32>(ray_depth(q.p.xy, q.a.xyz, q.b.xyz, q.c.xyz), 0.0, 0.0);
            }
        "#
        );
        let kernel = gpu.cache().get(gpu.context(), &source).unwrap();
        let mut parameters = PassParameters::new();
        parameters.insert("queries", input);
        parameters.insert("results", output.clone());
        let mut batch = gpu.batch("projection containment");
        batch
            .dispatch_items(&kernel, &parameters, queries.len() as u32)
            .unwrap();
        batch.submit();
        let results = gpu.read::<[f32; 4]>(&output).unwrap();
        for (index, (actual, expected)) in results.iter().copied().zip(expected).enumerate() {
            if let Some(depth) = expected {
                assert_eq!(actual[1], 1.0, "query {index}: {actual:?}");
                // Seven determinant operations, normalization and a three-term
                // dot product round this interpolated depth a bounded number of times.
                let depth_roundoff = 16.0 * (6.0_f32.next_up() - 6.0);
                assert!(
                    (actual[0] - depth).abs() <= depth_roundoff,
                    "query {index}: {actual:?}, expected {depth}"
                );
                assert!((2.0..=6.0).contains(&actual[0]));
            } else {
                assert_eq!(actual, [0.0; 4], "query {index}");
            }
        }
        for pair in results[shared_edge_start..].as_chunks::<2>().0 {
            assert!(
                pair.iter().any(|result| result[1] == 1.0),
                "shared edge: {pair:?}"
            );
            for result in pair.iter().filter(|result| result[1] == 1.0) {
                assert_eq!(result[0], 0.1, "shared edge: {pair:?}");
            }
        }
    }
}

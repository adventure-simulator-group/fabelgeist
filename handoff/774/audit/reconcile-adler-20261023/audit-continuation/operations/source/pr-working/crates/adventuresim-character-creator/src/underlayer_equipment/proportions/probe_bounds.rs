//! Limit hypothetical skeletal probes before they collapse or reverse a fold.
//!
//! Each moving face retains a quarter of its area projected onto its original
//! plane throughout the translation path. This is a local orientation reserve,
//! not a nonadjacent collision guarantee or an exported editor-range contract.

const RETAINED_ORIENTED_AREA: f64 = 0.25;
const FRACTION_SEARCH_STEPS: usize = 24;

struct AreaPath {
    linear: f64,
    quadratic: f64,
}

impl AreaPath {
    fn new(base: [[f32; 3]; 3], sample: [[f32; 3]; 3]) -> Option<Self> {
        let edge = |points: [[f32; 3]; 3], corner: usize| {
            std::array::from_fn(|axis| f64::from(points[corner][axis]) - f64::from(points[0][axis]))
        };
        let [u, v] = [edge(base, 1), edge(base, 2)];
        let du = std::array::from_fn(|axis| edge(sample, 1)[axis] - u[axis]);
        let dv = std::array::from_fn(|axis| edge(sample, 2)[axis] - v[axis]);
        let normal = cross(u, v);
        let denominator = dot(normal, normal);
        if denominator <= 0.0 || !denominator.is_finite() {
            return None;
        }
        let linear = (dot(cross(du, v), normal) + dot(cross(u, dv), normal)) / denominator;
        let quadratic = dot(cross(du, dv), normal) / denominator;
        (linear.is_finite() && quadratic.is_finite()).then_some(Self { linear, quadratic })
    }

    fn preserves_orientation_through(&self, end: f64) -> bool {
        let at = |t: f64| 1.0 + self.linear * t + self.quadratic * t * t;
        let mut minimum = 1.0_f64.min(at(end));
        if self.quadratic > 0.0 {
            let stationary = (-self.linear / (2.0 * self.quadratic)).clamp(0.0, end);
            minimum = minimum.min(at(stationary));
        }
        minimum >= RETAINED_ORIENTED_AREA
    }
}

/// Scale one probe uniformly, retaining its skeletal translation correspondence.
pub(super) fn oriented_area_fraction(
    base: &[[f32; 3]],
    sample: &[[f32; 3]],
    faces: &[[u32; 3]],
) -> f32 {
    let paths = faces
        .iter()
        .map(|face| {
            AreaPath::new(
                face.map(|v| base[v as usize]),
                face.map(|v| sample[v as usize]),
            )
        })
        .collect::<Option<Vec<_>>>();
    // An invalid selected surface cannot justify extrapolation. Preserve it
    // for the ordinary body/garment validation rather than fabricating a probe.
    let Some(paths) = paths else {
        return 0.0;
    };
    let valid = |end| {
        paths
            .iter()
            .all(|path| path.preserves_orientation_through(end))
    };
    if valid(1.0) {
        return 1.0;
    }
    let (mut low, mut high) = (0.0, 1.0);
    for _ in 0..FRACTION_SEARCH_STEPS {
        let middle = (low + high) * 0.5;
        if valid(middle) {
            low = middle;
        } else {
            high = middle;
        }
    }
    // Casting must not round beyond the last accepted interval boundary.
    (low as f32).next_down().max(0.0)
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    (0..3).map(|i| a[i] * b[i]).sum()
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_specific_fold_stops_before_its_face_reverses() {
        let base = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]];
        let crossed = [[0., 0., 0.], [-0.25, 0., 0.], [0., 1., 0.]];
        let fraction = oriented_area_fraction(&base, &crossed, &[[0, 1, 2]]);
        assert!(fraction > 0.59 && fraction <= 0.6);
        let retained = 1.0 - 1.25 * f64::from(fraction);
        assert!(retained >= RETAINED_ORIENTED_AREA);
    }

    #[test]
    fn matching_endpoint_normals_do_not_hide_an_intermediate_collapse() {
        let base = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]];
        let reversed_edges = [[0., 0., 0.], [-1., 0., 0.], [0., -1., 0.]];
        let fraction = oriented_area_fraction(&base, &reversed_edges, &[[0, 1, 2]]);
        assert!(fraction > 0.24 && fraction <= 0.25);
    }

    #[test]
    fn translation_and_outward_growth_preserve_the_requested_probe() {
        let base = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]];
        let moved = base.map(|point| [point[0] * 2.0 + 3.0, point[1] + 4.0, point[2] - 2.0]);
        assert_eq!(oriented_area_fraction(&base, &moved, &[[0, 1, 2]]), 1.0);
    }
}

//! Decorative shapes that can be cut into a panel edge.
//!
//! Ports `assets.garment_programs.shapes`.

use super::prelude::*;
use crate::curve::Curve;

/// Sample `n_points` points along a curve at a fixed arc-length stride.
fn sample_arc(curve: &Curve, length: f64, stride: f64, n_points: usize, shift: f64) -> Vec<V2> {
    (0..n_points)
        .map(|i| {
            let t = (shift + i as f64 * stride) / length;
            curve.point(t)
        })
        .collect()
}

/// A sun-like mark: a ring of triangular rays between two arcs.
pub fn sun(width: f64, depth: f64, n_rays: usize, d_rays: f64) -> EdgeSequence {
    let out_arc = circle_from_three_points([0.0, 0.0], [width, 0.0], [width / 2.0, depth], false);
    let in_arc = circle_from_three_points(
        [d_rays, 0.0],
        [width - d_rays, 0.0],
        [width / 2.0, depth - d_rays],
        false,
    );

    let out_curve = out_arc.borrow().as_curve();
    let in_curve = in_arc.borrow().as_curve();
    let out_len = out_arc.borrow().length();
    let in_len = in_arc.borrow().length();

    let out_stride = out_len / n_rays as f64;
    let in_stride = in_len / n_rays as f64;

    let out_verts = sample_arc(&out_curve, out_len, out_stride, n_rays, out_stride / 2.0);
    let in_verts = sample_arc(&in_curve, in_len, in_stride, n_rays + 1, 0.0);

    // Interleave: inner, outer, inner, outer, ...
    let mut verts = out_verts;
    for (i, v) in in_verts.iter().enumerate() {
        verts.insert(i * 2, *v);
    }

    from_verts(&verts, false)
}

/// A decorative cut: either one loop, or several that keep their relative
/// spacing when projected onto an edge.
#[derive(Debug, Clone)]
pub enum SideCut {
    Single(EdgeSequence),
    Multi(Vec<EdgeSequence>),
}

/// Build a named decorative shape, returning its left and right halves.
///
/// `Sun` is generated procedurally and is the same on both sides; the other
/// styles are loaded from an SVG file and split down the middle.
pub fn build(
    name: &str,
    width: f64,
    depth: f64,
    n_rays: usize,
    d_rays: f64,
    filename: Option<String>,
) -> (SideCut, SideCut) {
    match name {
        "Sun" => {
            let shape = sun(width, depth, n_rays, d_rays);
            (SideCut::Single(shape.copy()), SideCut::Single(shape))
        }
        "SIGGRAPH_logo" => {
            let path = filename
                .unwrap_or_else(|| "assets/img/siggraph_logo_thick_connection.svg".to_string());
            halves_from_svg(&path, width)
        }
        "SVGFile" => {
            let path = filename
                .expect("shapes::SVGFile::ERROR::a filename is required for the SVGFile style");
            halves_from_svg(&path, width)
        }
        other => panic!("shapes::ERROR::unknown side-cut style '{other}'"),
    }
}

/// Load a shape from an SVG file and split it vertically into two halves.
///
/// The shape is scaled so its height matches `target_height`. Each path in the
/// file must form a closed loop crossing the vertical centre line exactly
/// twice, and the paths must not nest or intersect.
pub fn halves_from_svg(path: &str, target_height: f64) -> (SideCut, SideCut) {
    let (mut left, mut right) =
        crate::curve::svg_parse::halves_from_file_multi(path, target_height)
            .unwrap_or_else(|e| panic!("shapes::ERROR::loading '{path}': {e}"));

    // Orient the halves so their shortcuts point along +Y, which keeps their
    // relative placement correct downstream.
    for seq in left.iter_mut().chain(right.iter_mut()) {
        let sc = seq.shortcut();
        if sc[1][1] - sc[0][1] < 0.0 {
            seq.reverse();
        }
    }

    (SideCut::Multi(left), SideCut::Multi(right))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sun_has_two_edges_per_ray() {
        let n_rays = 6;
        let shape = sun(10.0, 6.0, n_rays, 1.0);
        // 2*n_rays + 1 vertices -> 2*n_rays edges.
        assert_eq!(shape.len(), 2 * n_rays);
        assert!(shape.is_chained());
    }

    /// Oracle: `assets.garment_programs.shapes.Sun(width=10, depth=6,
    /// n_rays=6, d_rays=1)` from the reference implementation.
    ///
    /// The rays are *not* a simple alternation -- the inner and outer arcs are
    /// sampled at different strides, so near the ends an "outer" point can sit
    /// lower than its inner neighbour. Pinning the reference's own vertices is
    /// the only check that would catch a stride or phase mistake.
    #[test]
    fn sun_matches_the_reference_shape() {
        let shape = sun(10.0, 6.0, 6, 1.0);
        let verts: Vec<V2> = shape.verts().iter().map(vget).collect();

        let expected: [V2; 13] = [
            [1.000000000, 0.000000000],
            [-0.052218537, 1.478240910],
            [1.186506411, 2.405744548],
            [1.094875162, 4.170937365],
            [2.693872580, 4.289952260],
            [3.536576090, 5.784794463],
            [5.000000000, 5.000000000],
            [6.463423910, 5.784794463],
            [7.306127420, 4.289952260],
            [8.905124838, 4.170937365],
            [8.813493589, 2.405744548],
            [10.052218537, 1.478240910],
            [9.000000000, 0.000000000],
        ];

        assert_eq!(verts.len(), expected.len());
        for (i, (got, want)) in verts.iter().zip(&expected).enumerate() {
            assert!(
                dist2(*got, *want) < 1e-8,
                "vertex {i}: {got:?} != {want:?} (reference)"
            );
        }
    }
}

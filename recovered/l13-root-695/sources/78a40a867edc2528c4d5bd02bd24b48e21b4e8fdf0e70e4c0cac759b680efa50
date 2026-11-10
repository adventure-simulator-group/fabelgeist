//! Operations on whole patterns: panel ordering, edge-loop normalisation and
//! serialisation.
//!
//! Ports the behavioural half of `pygarment.pattern.core.BasicPattern`.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::curve::Curve;
use crate::math::*;

use super::spec::{Curvature, EdgeSpec, PanelSpec, PatternSpec};

impl PanelSpec {
    /// A rotation-independent 3D anchor for the panel: the world position of
    /// the midpoint of the (3D-)highest side of its 2D bounding box.
    ///
    /// In most designs this lands on a body landmark -- neck, mid-arm, waist --
    /// which makes it stable across garments, and it does not depend on the
    /// panel's choice of local origin.
    pub fn universal_translation(&self) -> (V3, V2) {
        let verts = &self.vertices;
        let top_right = [
            verts.iter().map(|v| v[0]).fold(f64::NEG_INFINITY, f64::max),
            verts.iter().map(|v| v[1]).fold(f64::NEG_INFINITY, f64::max),
        ];
        let low_left = [
            verts.iter().map(|v| v[0]).fold(f64::INFINITY, f64::min),
            verts.iter().map(|v| v[1]).fold(f64::INFINITY, f64::min),
        ];
        let mid_x = (top_right[0] + low_left[0]) / 2.0;
        let mid_y = (top_right[1] + low_left[1]) / 2.0;

        let mid_points_2d = [
            [mid_x, top_right[1]],
            [mid_x, low_left[1]],
            [top_right[0], mid_y],
            [low_left[0], mid_y],
        ];

        let rot = euler_xyz_to_r(self.rotation);
        let mid_points_3d: Vec<V3> = mid_points_2d
            .iter()
            .map(|p| point_in_3d(*p, rot, self.translation))
            .collect();

        // NOTE: first maximum, not last -- `np.argmax` breaks ties towards the
        // lowest index, and a symmetric panel can have all four midpoints at
        // the same height.
        let best = argmax_first(&mid_points_3d.iter().map(|p| p[1]).collect::<Vec<_>>());

        (mid_points_3d[best], mid_points_2d[best])
    }

    /// The edge as a drawable curve, in the panel's own frame.
    pub fn edge_as_curve(&self, edge: &EdgeSpec) -> Curve {
        edge_as_curve(&self.vertices, edge, false)
    }
}

/// Build the curve for an edge from an explicit vertex list.
///
/// `flip_y` selects image space: the renderer mirrors panel vertices (SVG has Y
/// pointing down) and then needs curvature mirrored to match -- Bezier control
/// points get their local Y negated and arcs swap their sweep flag.
pub fn edge_as_curve(vertices: &[V2], edge: &EdgeSpec, flip_y: bool) -> Curve {
    let start = vertices[edge.endpoints[0]];
    let end = vertices[edge.endpoints[1]];
    let sign = if flip_y { -1.0 } else { 1.0 };

    match &edge.curvature {
        None => Curve::line(start, end),
        Some(Curvature::Quadratic { params }) => {
            let c = rel_to_abs_2d(start, end, [params[0][0], sign * params[0][1]]);
            Curve::quad(start, c, end)
        }
        Some(Curvature::Cubic { params }) => {
            let c1 = rel_to_abs_2d(start, end, [params[0][0], sign * params[0][1]]);
            let c2 = rel_to_abs_2d(start, end, [params[1][0], sign * params[1][1]]);
            Curve::cubic(start, c1, c2, end)
        }
        Some(Curvature::Circle {
            radius,
            large_arc,
            right,
        }) => {
            let sweep = if flip_y { !*right } else { *right };
            Curve::Arc(crate::curve::Arc::new(
                start,
                [*radius, *radius],
                0.0,
                *large_arc,
                sweep,
                end,
            ))
        }
    }
}

impl PatternSpec {
    /// The agreed-upon panel order, computing it if it is not set yet.
    pub fn panel_order(&mut self, force_update: bool) -> Vec<String> {
        if self.panel_order.is_none() || force_update {
            self.panel_order = Some(self.define_panel_order());
        }
        self.panel_order.clone().unwrap()
    }

    /// Order panels by their 3D anchor: left-to-right, then bottom-to-top,
    /// then back-to-front, treating coordinates within `tolerance` cm as equal.
    pub fn define_panel_order(&self) -> Vec<String> {
        let names = self.panel_names();
        if names.is_empty() {
            return Vec::new();
        }
        let locations: Vec<(String, V3)> = names
            .iter()
            .map(|n| {
                let (loc, _) = self.panel(n).unwrap().universal_translation();
                (n.clone(), loc)
            })
            .collect();

        fuzzy_sort(&names, &locations, 0, 10.0)
    }

    /// Write the pattern specification JSON.
    ///
    /// With `to_subfolder`, the file lands in `<path>/<name><tag>/`; the
    /// directory actually used is returned.
    pub fn serialize(
        &mut self,
        path: impl AsRef<Path>,
        to_subfolder: bool,
        tag: &str,
        empty_ok: bool,
    ) -> Result<PathBuf> {
        if !empty_ok && self.panel_order(false).is_empty() {
            anyhow::bail!("PatternSpec::ERROR::Asked to save an empty pattern");
        }

        let log_dir = if to_subfolder {
            path.as_ref().join(format!("{}{}", self.name, tag))
        } else {
            path.as_ref().to_path_buf()
        };
        std::fs::create_dir_all(&log_dir)
            .with_context(|| format!("creating {}", log_dir.display()))?;

        let spec_file = log_dir.join(format!("{}{}_specification.json", self.name, tag));
        std::fs::write(&spec_file, self.to_json())
            .with_context(|| format!("writing {}", spec_file.display()))?;

        Ok(log_dir)
    }

    /// Re-order and re-orient every panel's edge loop:
    ///
    /// * start the loop at the low-left vertex, and
    /// * traverse counter-clockwise, mirroring the panel if it is not.
    ///
    /// Stitch references are updated to the new edge ids. This is what the
    /// reference applies when *loading* a pattern whose
    /// `normalized_edge_loops` property is unset; freshly generated patterns
    /// are already marked normalized, so it is not part of generation.
    pub fn normalize_edge_loops(&mut self) {
        let names = self.panel_names();
        for name in names {
            let (rotated_edge_ids, _) = self.normalize_panel_edge_loop(&name);

            for stitch in self.stitches.iter_mut() {
                for side in stitch.sides.iter_mut() {
                    if side.panel == name {
                        side.edge = rotated_edge_ids[side.edge];
                    }
                }
            }
        }
        self.properties.normalized_edge_loops = true;
    }

    fn normalize_panel_edge_loop(&mut self, panel_name: &str) -> (Vec<usize>, bool) {
        let panel = self.panel_mut(panel_name).expect("unknown panel");

        let loop_origin_id = vert_at_left_corner(&panel.vertices);
        let ids: Vec<usize> = (0..panel.edges.len()).collect();
        let (rotated_edges, mut rotated_edge_ids) =
            rotate_edges(&panel.edges, &ids, loop_origin_id);
        panel.edges = rotated_edges;

        // Because the origin sits at a corner, the cross product of the first
        // and last edge reliably indicates the loop's orientation.
        let first_edge = edge_vector(&panel.vertices, &panel.edges[0]);
        let last_edge = edge_vector(&panel.vertices, panel.edges.last().unwrap());

        let mut flipped = false;
        if cross2(first_edge, last_edge) > 0.0 {
            // Should be negative -- counter-clockwise.
            flipped = true;

            for v in panel.vertices.iter_mut() {
                v[0] = -v[0];
            }

            let loop_origin_id = vert_at_left_corner(&panel.vertices);
            let (re, rids) = rotate_edges(&panel.edges, &rotated_edge_ids, loop_origin_id);
            panel.edges = re;
            rotated_edge_ids = rids;

            // Left/right symmetry changed in 3D, so curvature follows.
            for e in panel.edges.iter_mut() {
                if let Some(c) = &mut e.curvature {
                    c.flip_y();
                }
            }

            panel.translation[0] -= 2.0 * panel.translation[0];

            let panel_r = euler_xyz_to_r(panel.rotation);
            let mut flip_r = [[0.0; 3]; 3];
            flip_r[0][0] = -1.0;
            flip_r[1][1] = 1.0;
            flip_r[2][2] = -1.0;
            panel.rotation = r_to_euler(matmul3(panel_r, flip_r));
        }

        (rotated_edge_ids, flipped)
    }
}

/// Index of the largest value, breaking ties towards the lowest index -- the
/// behaviour of `numpy.argmax`, which several placement heuristics rely on.
pub fn argmax_first(values: &[f64]) -> usize {
    let mut best = 0usize;
    for (i, v) in values.iter().enumerate() {
        if *v > values[best] {
            best = i;
        }
    }
    best
}

fn edge_vector(vertices: &[V2], edge: &EdgeSpec) -> V2 {
    sub2(vertices[edge.endpoints[1]], vertices[edge.endpoints[0]])
}

/// Index of the vertex nearest the bounding box's low-left corner.
fn vert_at_left_corner(vertices: &[V2]) -> usize {
    let left_corner = [
        vertices.iter().map(|v| v[0]).fold(f64::INFINITY, f64::min),
        vertices.iter().map(|v| v[1]).fold(f64::INFINITY, f64::min),
    ];
    vertices
        .iter()
        .map(|v| norm2(sub2(*v, left_corner)))
        .enumerate()
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(i, _)| i)
        .unwrap()
}

/// Rotate the edge list so it starts from `new_origin_id`, remapping ids.
fn rotate_edges(
    edges: &[EdgeSpec],
    edge_ids: &[usize],
    new_origin_id: usize,
) -> (Vec<EdgeSpec>, Vec<usize>) {
    let first = edges
        .iter()
        .position(|e| e.endpoints[0] == new_origin_id)
        .expect("no edge starts at the requested loop origin");

    let mut rotated: Vec<EdgeSpec> = edges[first..].to_vec();
    rotated.extend_from_slice(&edges[..first]);

    let split = rotated.len() - first;
    let mut rotated_ids: Vec<usize> = edge_ids[split..].to_vec();
    rotated_ids.extend_from_slice(&edge_ids[..split]);

    (rotated, rotated_ids)
}

/// Sort names by `dim`, recursing into runs whose values are within
/// `tolerance`.
fn fuzzy_sort(
    names: &[String],
    locations: &[(String, V3)],
    dim: usize,
    tolerance: f64,
) -> Vec<String> {
    if names.is_empty() {
        return Vec::new();
    }

    let value_of = |n: &str| -> f64 {
        locations
            .iter()
            .find(|(name, _)| name == n)
            .map(|(_, v)| v[dim])
            .unwrap()
    };

    // Python sorts `zip(values, names)` tuples, so equal values fall back to
    // the panel name.
    let mut pairs: Vec<(f64, String)> = names.iter().map(|n| (value_of(n), n.clone())).collect();
    pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap().then_with(|| a.1.cmp(&b.1)));

    let sorted_reference: Vec<f64> = pairs.iter().map(|p| p.0).collect();
    let mut sorted_names: Vec<String> = pairs.into_iter().map(|p| p.1).collect();

    if dim + 1 < 3 {
        let (mut fuzzy_start, mut fuzzy_end) = (0usize, 0usize);
        for end in 1..sorted_reference.len() {
            fuzzy_end = end;
            if sorted_reference[end] - sorted_reference[fuzzy_start] >= tolerance {
                if end - fuzzy_start > 1 {
                    let sub = fuzzy_sort(
                        &sorted_names[fuzzy_start..end],
                        locations,
                        dim + 1,
                        tolerance,
                    );
                    sorted_names[fuzzy_start..end].clone_from_slice(&sub);
                }
                fuzzy_start = end;
            }
        }
        if fuzzy_start != fuzzy_end {
            let sub = fuzzy_sort(&sorted_names[fuzzy_start..], locations, dim + 1, tolerance);
            let n = sorted_names.len();
            sorted_names[fuzzy_start..n].clone_from_slice(&sub);
        }
    }

    sorted_names
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::spec::PanelSpec;

    fn panel_at(t: V3) -> PanelSpec {
        PanelSpec {
            translation: t,
            rotation: [0.0; 3],
            vertices: vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]],
            edges: vec![
                EdgeSpec {
                    endpoints: [0, 1],
                    label: None,
                    curvature: None,
                },
                EdgeSpec {
                    endpoints: [1, 2],
                    label: None,
                    curvature: None,
                },
                EdgeSpec {
                    endpoints: [2, 3],
                    label: None,
                    curvature: None,
                },
                EdgeSpec {
                    endpoints: [3, 0],
                    label: None,
                    curvature: None,
                },
            ],
            label: None,
        }
    }

    #[test]
    fn panel_order_sorts_left_to_right_then_up() {
        let mut spec = PatternSpec::empty();
        spec.name = "t".into();
        spec.panels = vec![
            ("right".into(), panel_at([50.0, 0.0, 0.0])),
            ("left_low".into(), panel_at([-50.0, 0.0, 0.0])),
            ("left_high".into(), panel_at([-50.0, 40.0, 0.0])),
        ];

        let order = spec.panel_order(false);
        assert_eq!(order, vec!["left_low", "left_high", "right"]);
    }

    /// Panels at the same spot fall back to name order, as Python's tuple sort
    /// does.
    #[test]
    fn ties_break_on_name() {
        let mut spec = PatternSpec::empty();
        spec.name = "t".into();
        spec.panels = vec![
            ("b".into(), panel_at([0.0, 0.0, 0.0])),
            ("a".into(), panel_at([0.0, 0.0, 0.0])),
        ];
        assert_eq!(spec.panel_order(false), vec!["a", "b"]);
    }

    /// `np.argmax` breaks ties towards the lowest index; Rust's `max_by` would
    /// return the last. A symmetric panel has all four bounding-box midpoints
    /// at the same height, so the difference decides its anchor -- and with it
    /// the whole pattern's panel order.
    #[test]
    fn argmax_breaks_ties_towards_the_first() {
        assert_eq!(argmax_first(&[1.0, 1.0, 1.0, 1.0]), 0);
        assert_eq!(argmax_first(&[0.0, 2.0, 2.0]), 1);
        assert_eq!(argmax_first(&[3.0, 1.0]), 0);
    }

    /// A panel whose midpoints are all level must anchor on the first of them.
    #[test]
    fn flat_panel_anchors_on_the_first_midpoint() {
        let mut p = panel_at([0.0, 50.0, 0.0]);
        // A shape with no height: every bounding-box midpoint is at y = 0.
        p.vertices = vec![[0.0, 0.0], [40.0, 0.0], [20.0, 0.0]];
        let (anchor, local) = p.universal_translation();
        assert_eq!(local, [20.0, 0.0], "should take the first midpoint");
        assert!((anchor[0] - 20.0).abs() < 1e-12, "{anchor:?}");
    }

    #[test]
    fn left_corner_vertex() {
        let verts = vec![[5.0, 5.0], [0.0, 0.0], [5.0, 0.0], [0.0, 5.0]];
        assert_eq!(vert_at_left_corner(&verts), 1);
    }

    #[test]
    fn rotate_edges_remaps_ids() {
        let panel = panel_at([0.0; 3]);
        let ids: Vec<usize> = (0..4).collect();
        let (rotated, new_ids) = rotate_edges(&panel.edges, &ids, 2);

        assert_eq!(rotated[0].endpoints, [2, 3]);
        // Edge 2 became edge 0, so its id maps accordingly.
        assert_eq!(new_ids[2], 0);
        assert_eq!(new_ids[3], 1);
        assert_eq!(new_ids[0], 2);
    }
}

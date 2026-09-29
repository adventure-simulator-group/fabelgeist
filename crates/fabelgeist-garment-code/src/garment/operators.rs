//! Shortcuts for common operations on panels and components.
//!
//! Ports `pygarment.garmentcode.operators`.

use crate::curve::{Curve, ILENGTH_S_TOL};
use crate::math::*;
use crate::optimize::{self, Bound};

use super::clone::deep_copy;
use super::component::Element;
use super::connector::StitchingRule;
use super::edge::{Edge, EdgeRef, EdgeSequence, Vert, esame, subdivide_param, vget, vsame, vset};
use super::interface::{Interface, InterfaceRef};

// ----- Edge sequence modifiers -----

/// Objective for [`cut_corner`]: find points on two curves whose difference
/// matches the target shape's shortcut.
fn fit_location_corner(l: &[f64], diff_target: V2, curve1: &Curve, curve2: &Curve) -> f64 {
    let point1 = curve1.point(l[0]);
    let point2 = curve2.point(l[1]);
    let diff_curr = sub2(point2, point1);
    (diff_curr[0] - diff_target[0]).powi(2) + (diff_curr[1] - diff_target[1]).powi(2)
}

/// Cut the corner formed by the two edges of `target_interface`, following
/// `target_shape`.
///
/// The shape may be scaled along its main direction to fit the corner. The
/// panel geometry and any interfaces that mentioned the original edges are
/// updated in place.
///
/// Returns the newly inserted edges and an interface over them.
pub fn cut_corner(
    target_shape: &EdgeSequence,
    target_interface: &InterfaceRef,
) -> (EdgeSequence, InterfaceRef) {
    let mut corner_shape = target_shape.copy();
    let panel = target_interface.borrow().panel[0].clone();
    let mut target_edges = target_interface.borrow().edges.clone();

    // Work on vertices rather than directions: the edges may have been
    // reversed during normalisation.
    if vsame(
        &target_edges.first().borrow().start,
        &target_edges.last().borrow().end,
    ) {
        target_edges.reverse_order();
    }
    if vsame(
        &corner_shape.first().borrow().start,
        &corner_shape.last().borrow().end,
    ) {
        corner_shape.reverse_order();
    }
    if corner_shape.first().borrow().start_p()[1] > corner_shape.last().borrow().end_p()[1] {
        // Orient the corner shape the same way as the vertices.
        corner_shape.reverse();
        corner_shape.snap_to([0.0, 0.0]);
    }

    let shortcut = corner_shape.shortcut();

    let mut curve1 = target_edges[0].borrow().as_curve();
    let mut curve2 = target_edges[1].borrow().as_curve();

    // Align the curve order with the projecting shape, so that curve1 is the
    // lower one.
    let swapped =
        target_edges.first().borrow().start_p()[1] > target_edges.last().borrow().end_p()[1];
    if swapped {
        std::mem::swap(&mut curve1, &mut curve2);
    }

    let out = optimize::minimize_bounded(
        |l| fit_location_corner(l, sub2(shortcut[1], shortcut[0]), &curve1, &curve2),
        &[0.5, 0.5],
        &[(Some(0.0), Some(1.0)), (Some(0.0), Some(1.0))],
    );

    let mut loc = [out.x[0], out.x[1]];
    let point1 = curve1.point(loc[0]);
    corner_shape.snap_to(point1);

    if swapped {
        // The edges run v2 -> corner -> v1.
        corner_shape.reverse();
        loc.swap(0, 1);
    }

    // Insert the new shape, connecting it to what is left of the two edges.
    let cut_edge1 = subdivide_param(&target_edges[0], &[loc[0], 1.0 - loc[0]], true)[0].clone();
    let sub2_seq = subdivide_param(&target_edges[1], &[loc[1], 1.0 - loc[1]], true);
    let cut_edge2 = sub2_seq.last().clone();

    cut_edge1.borrow_mut().end = corner_shape.first().borrow().start.clone();
    cut_edge2.borrow_mut().start = corner_shape.last().borrow().end.clone();

    corner_shape.insert_one(0, cut_edge1);
    corner_shape.push(cut_edge2);

    // Substitute the edges in the panel definition.
    panel.borrow_mut().edges.pop_edge(&target_edges[0]);
    panel
        .borrow_mut()
        .edges
        .substitute(&target_edges[1], &corner_shape);

    // Update interfaces that referenced the original edges.
    let first_new = corner_shape.first().clone();
    let last_new = corner_shape.last().clone();
    let interfaces: Vec<InterfaceRef> = panel.borrow().interfaces.values().cloned().collect();
    for intr in interfaces {
        let mut intr = intr.borrow_mut();
        if intr.edges.contains(&target_edges[0]) {
            intr.edges
                .substitute_one(&target_edges[0], first_new.clone());
        }
        if intr.edges.contains(&target_edges[1]) {
            intr.edges
                .substitute_one(&target_edges[1], last_new.clone());
        }
    }

    // Register an interface for the cut itself.
    let inserted = corner_shape.slice(1, corner_shape.len() - 1);
    let new_int = Interface::plain(&panel, inserted.clone());
    let key = format!("int_{}", panel.borrow().interfaces.len());
    panel.borrow_mut().interfaces.set(&key, new_int.clone());

    (inserted, new_int)
}

fn dist(a: V2, b: V2) -> f64 {
    norm2(sub2(b, a))
}

/// Objective for [`cut_into_edge_single`]: place a chord of the given width
/// symmetrically around `location` on the curve.
fn fit_location_edge(shift: &[f64], location: f64, width_target: f64, curve: &Curve) -> f64 {
    let pointc = curve.point(location);
    let point1 = curve.point(location + shift[0]);
    let point2 = curve.point(location - shift[1]);

    // Keep the two points equidistant from the centre.
    let reg_symmetry = (dist(point1, pointc) - dist(point2, pointc)).powi(2);
    (dist(point1, point2) - width_target).powi(2) + reg_symmetry
}

/// The result of cutting a shape into an edge.
pub struct CutResult {
    /// The edges that now replace the base edge.
    pub new_edges: EdgeSequence,
    /// Just the inserted shape.
    pub inserted: EdgeSequence,
    /// The parts of the original edge that survived.
    pub leftovers: EdgeSequence,
}

/// Insert `target_shape` into `base_edge`, centred at `offset` along it.
///
/// The shape is rotated so that its start->end vector aligns with the edge.
/// `right` picks which side of the base edge the cut opens towards.
pub fn cut_into_edge_single(
    target_shape: &EdgeSequence,
    base_edge: &EdgeRef,
    offset: f64,
    right: bool,
    tol: f64,
) -> CutResult {
    let mut new_edges = target_shape.copy();
    new_edges.snap_to([0.0, 0.0]);

    let shortcut = new_edges.shortcut();
    let target_shape_w = norm2(sub2(shortcut[1], shortcut[0]));
    let edge_len = base_edge.borrow().length();

    assert!(
        offset >= target_shape_w / 2.0 - tol && offset <= edge_len - target_shape_w / 2.0 + tol,
        "Operators-CuttingIntoEdge::ERROR::offset value ({offset}) is not within \
         the base_edge length ({edge_len})"
    );

    let curve = base_edge.borrow().as_curve();
    let rel_offset = curve.ilength(offset, ILENGTH_S_TOL);

    let out = optimize::minimize_bounded(
        |shift| fit_location_edge(shift, rel_offset, target_shape_w, &curve),
        &[0.1, 0.1],
        &[(Some(0.0), Some(1.0))],
    );
    let shift = [out.x[0], out.x[1]];

    assert!(
        close_to_zero_tol(out.fun, 0.01),
        "Cut_edge::ERROR::projection on the base edge finished with fun={}",
        out.fun
    );
    assert!(
        rel_offset + shift[0] <= 1.0 + tol && rel_offset - shift[1] >= -tol,
        "Cut_edge::ERROR::projection is out of edge bounds: \
         [{}, {}]. Check the offset value",
        rel_offset - shift[1],
        rel_offset + shift[0]
    );

    // Snap the ends of the cut to the base edge's own vertices when they land
    // right at its extremities.
    let t_start = rel_offset - shift[1];
    let t_end = rel_offset + shift[0];
    let at_edge_start = t_start <= tol;
    let at_edge_end = t_end >= 1.0 - tol;

    let ins_point = if at_edge_start {
        base_edge.borrow().start_p()
    } else {
        curve.point(t_start)
    };
    let fin_point = if at_edge_end {
        base_edge.borrow().end_p()
    } else {
        curve.point(t_end)
    };

    // Align the shape with the edge.
    let insert_vector = sub2(fin_point, ins_point);
    let angle = vector_angle(sub2(shortcut[1], shortcut[0]), insert_vector);
    new_edges.rotate(angle);
    new_edges.snap_to(ins_point);

    // Check which side the cut opens to, and flip if it is the wrong one.
    let verts: Vec<V2> = new_edges.verts().iter().map(vget).collect();
    let n = verts.len() as f64;
    let avg_vertex = [
        verts.iter().map(|v| v[0]).sum::<f64>() / n,
        verts.iter().map(|v| v[1]).sum::<f64>() / n,
    ];
    let first_start = new_edges.first().borrow().start_p();
    let right_position = cross2(insert_vector, sub2(avg_vertex, first_start)).signum() == -1.0;
    if right != right_position {
        let a = new_edges.first().borrow().start_p();
        let b = new_edges.last().borrow().end_p();
        new_edges.reflect(a, b);
    }

    // Splice into the base edge. No extra edges are needed when the shape sits
    // exactly at one end.
    let mut leftovers = EdgeSequence::new();
    let mut start_id = 0usize;
    let mut end_id = new_edges.len();

    if at_edge_start {
        let bs = base_edge.borrow().start.clone();
        new_edges.first().borrow_mut().start = bs;
    } else {
        let start_part = subdivide_param(base_edge, &[t_start, 1.0 - t_start], true)[0].clone();
        start_part.borrow_mut().end = new_edges.first().borrow().start.clone();
        new_edges.insert_one(0, start_part);
        leftovers.push(new_edges.first().clone());
        start_id = 1;
        end_id += 1;
    }

    if at_edge_end {
        let be = base_edge.borrow().end.clone();
        new_edges.last().borrow_mut().end = be;
    } else {
        let parts = subdivide_param(base_edge, &[t_end, 1.0 - t_end], true);
        let end_part = parts.last().clone();
        end_part.borrow_mut().start = new_edges.last().borrow().end.clone();
        new_edges.push(end_part);
        leftovers.push(new_edges.last().clone());
        end_id = new_edges.len() - 1;
    }

    let inserted = new_edges.slice(start_id, end_id);
    CutResult {
        new_edges,
        inserted,
        leftovers,
    }
}

fn close_to_zero_tol(v: f64, tol: f64) -> bool {
    v.abs() < tol
}

/// Insert several shapes into one edge in a single pass, preserving their
/// relative spacing.
///
/// Each shape's opening is assumed to be aligned with OY, and all of them are
/// expressed in the same coordinate system.
pub fn cut_into_edge_multi(
    target_shapes: &[EdgeSequence],
    base_edge: &EdgeRef,
    offset: f64,
    right: bool,
    flip_target: bool,
    tol: f64,
) -> CutResult {
    let shortcut_of = |s: &EdgeSequence| s.shortcut();

    let mut shapes: Vec<EdgeSequence> = target_shapes.to_vec();

    let ys: Vec<f64> = shapes
        .iter()
        .flat_map(|s| {
            let sc = shortcut_of(s);
            [sc[0][1], sc[1][1]]
        })
        .collect();
    let median_y = (ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
        + ys.iter().cloned().fold(f64::INFINITY, f64::min))
        / 2.0;

    if flip_target {
        shapes = shapes
            .iter()
            .map(|s| {
                let c = s.copy();
                c.reflect([0.0, median_y], [1.0, median_y]);
                c
            })
            .collect();
        // Flip the order too, to reflect the orientation change.
        for s in shapes.iter_mut() {
            s.reverse();
        }
    }

    // Offsets that place the whole group at the requested offset.
    let mut placed: Vec<(f64, EdgeSequence)> = shapes
        .into_iter()
        .map(|s| {
            let sc = shortcut_of(&s);
            let rel = (sc[0][1] + sc[1][1]) / 2.0 - median_y;
            (offset + rel, s)
        })
        .collect();

    // Project from farthest to closest.
    placed.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    let mut proj_edge = base_edge.clone();
    let mut int_edges = EdgeSequence::one(base_edge.clone());
    let mut new_in_edges = EdgeSequence::new();
    let mut all_new_edges = EdgeSequence::one(base_edge.clone());

    for (off, shape) in placed {
        let res = cut_into_edge_single(&shape, &proj_edge, off, right, tol);

        all_new_edges.substitute(&proj_edge, &res.new_edges);
        int_edges.substitute(&proj_edge, &res.leftovers);
        new_in_edges.extend(&res.inserted);
        proj_edge = res.new_edges.first().clone();
    }

    CutResult {
        new_edges: all_new_edges,
        inserted: new_in_edges,
        leftovers: int_edges,
    }
}

// ----- Panel operations -----

/// Copies of an element distributed around the Y axis.
pub fn distribute_y(
    component: &Element,
    n_copies: usize,
    odd_copy_shift: f64,
    name_tag: &str,
) -> Vec<Element> {
    component.set_name(&format!("{name_tag}_0"));
    let mut copies = vec![component.clone()];

    let delta_rotation = Rotation::from_euler_xyz([0.0, 360.0 / n_copies as f64, 0.0], true);

    for i in 0..n_copies.saturating_sub(1) {
        let new_component = deep_copy(copies.last().unwrap());
        new_component.set_name(&format!("{}_{}", name_tag, i + 1));
        new_component.rotate_by(delta_rotation);

        let t = match &new_component {
            Element::Panel(p) => p.borrow().translation,
            Element::Comp(_) => new_component.pivot_3d(),
        };
        new_component.translate_to(delta_rotation.apply(t));

        copies.push(new_component);
    }

    // Nudge alternating copies outwards to reduce collisions.
    if odd_copy_shift != 0.0 {
        for (i, c) in copies.iter().enumerate() {
            if i % 2 == 0
                && let Element::Panel(p) = c
            {
                let n = p.borrow().norm();
                c.translate_by(scale3(n, odd_copy_shift));
            }
        }
    }

    copies
}

/// Copies of a panel spread along a horizontal line perpendicular to its
/// normal.
pub fn distribute_horisontally(
    component: &Element,
    n_copies: usize,
    stride: f64,
    name_tag: &str,
) -> Vec<Element> {
    component.set_name(&format!("{name_tag}_0"));
    let mut copies = vec![component.clone()];

    let delta_translation = match component {
        Element::Panel(p) => {
            // Horizontally along the panel, then perpendicular to Y.
            let along = p.borrow().rotation.apply([0.0, 0.0, 1.0]);
            let dir = cross3(along, [0.0, 1.0, 0.0]);
            scale3(scale3(dir, 1.0 / norm3(dir)), stride)
        }
        Element::Comp(_) => [stride, 0.0, 0.0],
    };

    for i in 0..n_copies.saturating_sub(1) {
        let new_component = deep_copy(copies.last().unwrap());
        new_component.set_name(&format!("{}_{}", name_tag, i + 1));
        new_component.translate_by(delta_translation);
        copies.push(new_component);
    }

    copies
}

// ----- Sleeve support -----

/// Rearrange front and back sleeve openings so their vertical projections
/// match, letting two symmetric sleeve panels be built from them.
///
/// Assumes the front opening is the longer of the two.
pub fn even_armhole_openings(
    front_opening: &mut EdgeSequence,
    back_opening: &mut EdgeSequence,
    tol: f64,
) {
    // Build the sleeve panel shape from the inverted openings.
    let cfront = front_opening.copy();
    let cback = back_opening.copy();
    cback.reflect([0.0, 0.0], [1.0, 0.0]);
    let mut cback = cback;
    cback.reverse();
    cback.snap_to(cfront.last().borrow().end_p());

    let slope = [
        cfront.first().borrow().start_p(),
        cback.last().borrow().end_p(),
    ];
    let slope_vec = sub2(slope[1], slope[0]);
    let slope_perp = [-slope_vec[1], slope_vec[0]];
    let slope_midpoint = scale2(add2(slope[0], slope[1]), 0.5);

    // Where the sleeve line crosses the opening itself.
    let inter_segment = Curve::line(
        sub2(slope_midpoint, scale2(slope_perp, 20.0)),
        add2(slope_midpoint, scale2(slope_perp, 20.0)),
    );
    let target_segment = cfront.last().borrow().as_curve();
    let hits = target_segment.intersect(&inter_segment);

    if let Some((t, _)) = hits.first() {
        let intersect_t = *t;
        if !(close_enough(intersect_t, 0.0, tol) || close_enough(intersect_t, 1.0, tol)) {
            // The current split is unsatisfactory -- move the tail across.
            let last = front_opening.last().clone();
            let subdiv = subdivide_param(&last, &[intersect_t, 1.0 - intersect_t], true);
            let keep = subdiv[0].clone();
            let moved = subdiv[1].clone();
            let n = front_opening.len();
            front_opening.substitute_at(n - 1, &EdgeSequence::one(keep));

            // Detach the moved part's vertices before re-attaching them.
            {
                let mut m = moved.borrow_mut();
                let (s, e) = (vget(&m.start), vget(&m.end));
                m.start = super::edge::vert(s);
                m.end = super::edge::vert(e);
            }
            let moved_seq = EdgeSequence::one(moved.clone());
            moved_seq.reflect([0.0, 0.0], [1.0, 0.0]);
            let mut moved_seq = moved_seq;
            moved_seq.reverse();
            moved_seq.snap_to(back_opening.last().borrow().end_p());
            moved.borrow_mut().start = back_opening.last().borrow().end.clone();

            back_opening.push(moved);
        }
    }

    // Align the slope with OY, for a correctly sized sleeve panel.
    let slope_angle = (-slope_vec[0] / slope_vec[1]).atan();
    front_opening.rotate(-slope_angle);
    back_opening.rotate(slope_angle);
}

// ----- Curve tools -----

fn max_curvature(curve: &Curve, points_estimates: usize) -> f64 {
    (0..points_estimates)
        .map(|i| curve.curvature(i as f64 / (points_estimates - 1) as f64))
        .fold(f64::NEG_INFINITY, f64::max)
}

/// Objective for [`curve_match_tangents`]: preserve length and endpoint
/// tangents while keeping the curve as flat as possible.
#[allow(clippy::too_many_arguments)]
fn bend_extend_2_tangent(
    shift: &[f64],
    cp: &[V2; 4],
    target_len: f64,
    direction: V2,
    target_tan_start: V2,
    target_tan_end: V2,
    point_estimates: usize,
) -> f64 {
    let control = [
        cp[0],
        [cp[1][0] + shift[0], cp[1][1] + shift[1]],
        [cp[2][0] + shift[2], cp[2][1] + shift[3]],
        add2(cp[3], scale2(direction, shift[4])),
    ];
    let curve_inverse = Curve::cubic(control[0], control[1], control[2], control[3]);

    let length_diff = (curve_inverse.length() - target_len).powi(2);
    let tan_0_diff = dist2(curve_inverse.unit_tangent(0.0), target_tan_start).powi(2);
    let tan_1_diff = dist2(curve_inverse.unit_tangent(1.0), target_tan_end).powi(2);
    let curvature_reg = max_curvature(&curve_inverse, point_estimates).powi(2);
    let end_expansion_reg = 0.001 * shift[4] * shift[4];

    length_diff + tan_0_diff + tan_1_diff + curvature_reg + end_expansion_reg
}

/// Bend a cubic Bezier to hit the requested endpoint tangents while preserving
/// its length (or `target_len`) and overall direction.
///
/// Expects a starting curve that already approximates the desired solution.
pub fn curve_match_tangents(
    curve: &Curve,
    target_tan0: V2,
    target_tan1: V2,
    target_len: Option<f64>,
) -> EdgeRef {
    let Curve::Cubic { start, c1, c2, end } = curve else {
        panic!("Curve_match_tangents::ERROR::Only Cubic Bezier curves are supported");
    };
    let cps = [*start, *c1, *c2, *end];

    let direction = normalize2(sub2(cps[3], cps[0]));
    let target_tan0 = normalize2(target_tan0);
    let target_tan1 = normalize2(target_tan1);
    let target_len = target_len.unwrap_or_else(|| curve.length());

    let bounds: Vec<Bound> = vec![(None, None); 5];
    let out = optimize::minimize_bounded(
        |shift| {
            bend_extend_2_tangent(
                shift,
                &cps,
                target_len,
                direction,
                target_tan0,
                target_tan1,
                // NOTE: low values make the optimisation unstable.
                70,
            )
        },
        &[0.0; 5],
        &bounds,
    );
    let shift = out.x;

    let fin = [
        cps[0],
        [cps[1][0] + shift[0], cps[1][1] + shift[1]],
        [cps[2][0] + shift[2], cps[2][1] + shift[3]],
        add2(cps[3], scale2(direction, shift[4])),
    ];

    Edge::curve(fin[0], fin[3], vec![fin[1], fin[2]], false)
}

// ----- helpers shared with panels -----

/// Add a dart to a panel: cut `dart_shape` into `edge` and stitch the dart
/// sides together.
///
/// Returns the edges that replace `edge`, and the part of them that is still
/// on the panel border (i.e. excluding the dart itself).
///
/// When `edge_seq` / `int_edge_seq` are given, the cut is spliced into them and
/// they are returned instead -- this is how several darts get chained onto the
/// same waistline.
pub fn add_dart(
    panel: &super::panel::PanelRef,
    dart_shape: &EdgeSequence,
    edge: &EdgeRef,
    offset: f64,
    right: bool,
    edge_seq: Option<&mut EdgeSequence>,
    int_edge_seq: Option<&mut EdgeSequence>,
) -> (EdgeSequence, EdgeSequence) {
    let res = cut_into_edge_single(dart_shape, edge, offset, right, 1e-2);

    let dart_a = Interface::one(panel, res.inserted[0].clone());
    let dart_b = Interface::one(panel, res.inserted[1].clone());
    // Build the rule before borrowing the panel: matching a stitch can
    // subdivide the panel's own edges, which needs its own borrow.
    let rule = StitchingRule::new(dart_a, dart_b);
    panel.borrow_mut().stitching_rules.rules.push(rule);

    let new_edges = match edge_seq {
        Some(seq) => {
            seq.substitute(edge, &res.new_edges);
            seq.clone()
        }
        None => res.new_edges,
    };
    let int_new = match int_edge_seq {
        Some(seq) => {
            seq.substitute(edge, &res.leftovers);
            seq.clone()
        }
        None => res.leftovers,
    };

    (new_edges, int_new)
}

/// Replace `edge` in a sequence with the result of a cut.
pub fn splice(seq: &mut EdgeSequence, edge: &EdgeRef, replacement: &EdgeSequence) {
    if seq.contains(edge) {
        seq.substitute(edge, replacement);
    }
}

/// True when the two edges are the same object.
pub fn same_edge(a: &EdgeRef, b: &EdgeRef) -> bool {
    esame(a, b)
}

/// Move a vertex to an absolute location.
pub fn move_vert(v: &Vert, p: V2) {
    vset(v, p);
}

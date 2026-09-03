//! Stitching rules -- how two interfaces get connected.
//!
//! Ports `pygarment.garmentcode.connector`.

use crate::math::close_enough;
use crate::pattern::spec::{Stitch, StitchSide};

use super::edge::subdivide_len;
use super::interface::InterfaceRef;

/// A high-level instruction connecting two interfaces.
///
/// When the two sides do not break down the same way, the longer edges are
/// subdivided until they do: the serialized format has no native T-stitch, so
/// both sides must present matching segment counts and relative lengths.
#[derive(Debug, Clone)]
pub struct StitchingRule {
    pub int1: InterfaceRef,
    pub int2: InterfaceRef,
}

impl StitchingRule {
    pub fn new(int1: InterfaceRef, int2: InterfaceRef) -> Self {
        let rule = StitchingRule { int1, int2 };
        if !rule.is_matching(0.05) {
            rule.match_interfaces();
        }
        rule
    }

    /// Do both sides have the same segment count and relative partitioning?
    pub fn is_matching(&self, tol: f64) -> bool {
        let frac1 = self.int1.borrow().projecting_fractions();
        let frac2 = self.int2.borrow().projecting_fractions();

        frac1.len() == frac2.len()
            && frac1
                .iter()
                .zip(&frac2)
                // np.allclose semantics: atol + rtol * |b|, rtol defaulting to 1e-5.
                .all(|(a, b)| (a - b).abs() <= tol + 1e-5 * b.abs())
    }

    /// Subdivide both sides until they match.
    pub fn match_interfaces(&self) {
        let frac1 = self.int1.borrow().projecting_fractions();
        let frac2 = self.int2.borrow().projecting_fractions();

        let min_frac = frac1
            .iter()
            .chain(&frac2)
            .cloned()
            .fold(f64::INFINITY, f64::min);
        // The projection tolerance must stay below the smallest segment.
        let tol = (1e-2_f64).min(min_frac / 2.0);

        match_to_fractions(&self.int1, &frac2, tol);
        match_to_fractions(&self.int2, &frac1, tol);
    }

    /// Produce the stitches that connect the two interfaces.
    pub fn assembly(&self) -> Vec<Stitch> {
        let int1 = self.int1.borrow();
        let int2 = self.int2.borrow();

        let n = int1.edges.len().min(int2.edges.len());
        (0..n)
            .map(|i| Stitch {
                sides: [
                    StitchSide {
                        panel: int1.panel[i].borrow().name.clone(),
                        edge: int1.edges[i].borrow().geometric_id,
                    },
                    StitchSide {
                        panel: int2.panel[i].borrow().name.clone(),
                        edge: int2.edges[i].borrow().geometric_id,
                    },
                ],
                // Marked when *either* side asks for it: the same rule can be
                // reused by components with different fabric-side preferences.
                right_wrong: int1.right_wrong[i] || int2.right_wrong[i],
            })
            .collect()
    }
}

/// Insert vertices into `inter`'s edges so that its segment fractions include
/// those in `to_add`.
///
/// `tol` is how close two vertices must be to count as the same one; it has to
/// stay shorter than the smallest expected edge.
fn match_to_fractions(inter: &InterfaceRef, to_add: &[f64], tol: f64) {
    let (mut add_id, mut in_id) = (0usize, 0usize);
    let (mut covered_init, mut covered_added) = (0.0_f64, 0.0_f64);
    let mut curr_fractions = inter.borrow().projecting_fractions();

    // The sequences may be disconnected (they can even span panels), so the
    // subdivision is done edge by edge.
    while in_id < inter.borrow().edges.len() && add_id < to_add.len() {
        // Accumulated error can overshoot slightly, hence the clamp.
        let next_init = (covered_init + curr_fractions[in_id]).min(1.0);
        let next_added = (covered_added + to_add[add_id]).min(1.0);

        if close_enough(next_init, next_added, tol) {
            // The vertex is already there.
            in_id += 1;
            add_id += 1;
            covered_init = next_init;
            covered_added = next_added;
        } else if next_init < next_added {
            in_id += 1;
            covered_init = next_init;
        } else {
            // Add a vertex at the projected location.
            let in_frac = curr_fractions[in_id];
            let new_v_loc = in_frac - (next_init - next_added);
            let mut split_frac = new_v_loc / in_frac;

            let (base_edge, base_panel, flip) = {
                let i = inter.borrow();
                (
                    i.edges[in_id].clone(),
                    i.panel[in_id].clone(),
                    i.needs_flipping(in_id),
                )
            };

            if flip {
                split_frac = 1.0 - split_frac;
            }

            let mut subdiv = subdivide_len(&base_edge, &[split_frac, 1.0 - split_frac], true);

            // Update the panel, which always follows its own edge order.
            base_panel
                .borrow_mut()
                .edges
                .substitute(&base_edge, &subdiv);

            // Keep the interface sequence oriented.
            if flip {
                subdiv.reverse_order();
            }

            let panels = vec![base_panel.clone(); subdiv.len()];
            inter.borrow_mut().substitute(&base_edge, &subdiv, &panels);

            curr_fractions = inter.borrow().projecting_fractions();
            covered_init += curr_fractions[in_id];
            covered_added = next_added;
            in_id += 1;
            add_id += 1;
        }
    }

    assert!(
        add_id == to_add.len(),
        "StitchingRule::ERROR::Projection on {:?} failed",
        inter.borrow().panel_names()
    );
}

/// A collection of stitching rules.
#[derive(Debug, Clone, Default)]
pub struct Stitches {
    pub rules: Vec<StitchingRule>,
}

impl Stitches {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build from pairs of interfaces.
    pub fn from_pairs(pairs: Vec<(InterfaceRef, InterfaceRef)>) -> Self {
        Self {
            rules: pairs
                .into_iter()
                .map(|(a, b)| StitchingRule::new(a, b))
                .collect(),
        }
    }

    pub fn append(&mut self, int1: InterfaceRef, int2: InterfaceRef) {
        self.rules.push(StitchingRule::new(int1, int2));
    }

    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    pub fn assembly(&self) -> Vec<Stitch> {
        self.rules.iter().flat_map(|r| r.assembly()).collect()
    }
}

//! Proxies for the hierarchy the renderer draws the forest from.
//!
//! Plants are the leaves of a bounding-volume hierarchy ([`fabelgeist_hlod`]);
//! every node above stands in for its subtree with a proxy made of the same
//! instances the plants are drawn with, only fewer:
//!
//! * **branches**: its children's, each axis simplified by merging segments
//!   end to end -- a trunk stays whole, in fewer, longer pieces -- and then
//!   the thinnest twigs dropped;
//! * **leaves**: its children's clusters merged on a grid, one per cell and
//!   species, each merged card as large as the ones it stands for together,
//!   so a crown keeps its silhouette and its density.
//!
//! A node keeps at most half of what its children hold, and as little as a
//! quarter where that costs little error, so the proxies of the whole
//! hierarchy come to at most as much again as the plants.
//! A proxy's error is the largest detail it leaves out: the thickest branch
//! dropped, or how far a leaf cluster moved to its merged card -- half a
//! cell.

use std::collections::HashMap;

use fabelgeist_hlod::{Cluster, Hierarchy, Tree};
use rayon::prelude::*;

use crate::viewer::sim::{BranchInstance, LeafInstance, PlantInstance};

/// A node keeps at most half of its children's instances -- which bounds
/// all the proxies together to as many again as the plants -- and goes on
/// to a quarter while its error stays within `BUDGET` of its radius.
const HALF: usize = 2;
const QUARTER: usize = 4;
const BUDGET: f32 = 0.02;
/// ... but never fewer than this, where its children have as many.
const KEEP_AT_LEAST: usize = 8;

/// Everything the renderer draws: the plants' instances, then the proxies',
/// and the hierarchy saying which are whose.
pub struct Forest {
    pub branches: Vec<BranchInstance>,
    pub leaves: Vec<LeafInstance>,
    pub plants: Vec<PlantInstance>,
    pub hierarchy: Hierarchy,
}

/// A node's instances, for its parent to reduce.
enum Held {
    /// A plant's own: ranges into the plant instances.
    Plant(u32),
    /// A proxy's.
    Proxy(Proxy),
}

struct Proxy {
    branches: Vec<BranchInstance>,
    leaves: Vec<LeafInstance>,
    error: f32,
    /// The grid its leaves were merged on: where its parent's starts.
    cell: f32,
}

/// Gather the plants' instances, build the hierarchy over them and, with
/// `proxies`, a proxy for every node above a plant. Without, a node above
/// the plants is never drawn -- its error is unbounded -- and every plant in
/// view is, at its own level of detail.
pub fn build(
    per_plant: Vec<(Vec<BranchInstance>, Vec<LeafInstance>, PlantInstance)>,
    proxies: bool,
) -> Forest {
    let mut branches = Vec::with_capacity(per_plant.iter().map(|p| p.0.len()).sum());
    let mut leaves = Vec::with_capacity(per_plant.iter().map(|p| p.1.len()).sum());
    let mut plants = Vec::with_capacity(per_plant.len());
    let mut branch_ranges = Vec::with_capacity(per_plant.len());
    let mut leaf_ranges = Vec::with_capacity(per_plant.len());
    let mut axes = 0;
    for (b, l, p) in per_plant {
        let most = b.iter().map(|branch| branch.axis + 1).max().unwrap_or(0);
        branch_ranges.push([branches.len() as u32, b.len() as u32]);
        leaf_ranges.push([leaves.len() as u32, l.len() as u32]);
        branches.extend(b.into_iter().map(|mut branch| {
            branch.axis += axes;
            branch
        }));
        axes += most;
        leaves.extend(l);
        plants.push(p);
    }

    let spheres: Vec<[f32; 4]> = plants.iter().map(|p| p.sphere).collect();
    let tree = Tree::build(&spheres);
    let count = tree.nodes.len();

    // Bottom-up, a height at a time, the nodes of one in parallel.
    let mut height = vec![0u32; count];
    for (i, node) in tree.nodes.iter().enumerate() {
        if let Some([a, b]) = node.children {
            height[i] = 1 + height[a as usize].max(height[b as usize]);
        }
    }
    let tallest = if proxies {
        height.iter().copied().max().unwrap_or(0)
    } else {
        0
    };
    let mut held: Vec<Option<Held>> = tree
        .nodes
        .iter()
        .map(|n| n.object.map(Held::Plant))
        .collect();
    for h in 1..=tallest {
        let level: Vec<usize> = (0..count).filter(|&i| height[i] == h).collect();
        let built: Vec<(usize, Proxy)> = level
            .par_iter()
            .map(|&i| {
                let [a, b] = tree.nodes[i].children.expect("an internal node");
                let (mut stems, mut foliage) = (Vec::new(), Vec::new());
                // The grid starts where the children's left off: a plant's
                // at a little more than its clusters, a proxy's at its own.
                let mut cell = 0.0f32;
                for child in [a, b] {
                    match held[child as usize].as_ref().expect("children come first") {
                        Held::Plant(p) => {
                            let [bf, bc] = branch_ranges[*p as usize];
                            let [lf, lc] = leaf_ranges[*p as usize];
                            let own = &leaves[lf as usize..(lf + lc) as usize];
                            stems.extend_from_slice(&branches[bf as usize..(bf + bc) as usize]);
                            foliage.extend_from_slice(own);
                            let largest = own.iter().map(|l| l.size).fold(0.0, f32::max);
                            cell = cell.max(1.5 * largest);
                        }
                        Held::Proxy(proxy) => {
                            stems.extend_from_slice(&proxy.branches);
                            foliage.extend_from_slice(&proxy.leaves);
                            cell = cell.max(proxy.cell);
                        }
                    }
                }
                (i, reduce(stems, foliage, tree.nodes[i].sphere, cell))
            })
            .collect();
        for (i, proxy) in built {
            held[i] = Some(Held::Proxy(proxy));
        }
    }

    // The proxies after the plants' instances, node by node.
    let mut clusters = Vec::with_capacity(count);
    for held in held.iter_mut() {
        let Some(held) = held.take() else {
            clusters.push(Cluster {
                error: f32::INFINITY,
                finest_level: 2,
                ranges: vec![[0, 0], [0, 0]],
            });
            continue;
        };
        match held {
            Held::Plant(p) => clusters.push(Cluster {
                error: 0.0,
                finest_level: 0,
                ranges: vec![branch_ranges[p as usize], leaf_ranges[p as usize]],
            }),
            Held::Proxy(proxy) => {
                clusters.push(Cluster {
                    error: proxy.error,
                    // A proxy is already coarse: its tubes at the coarsest.
                    finest_level: 2,
                    ranges: vec![
                        [branches.len() as u32, proxy.branches.len() as u32],
                        [leaves.len() as u32, proxy.leaves.len() as u32],
                    ],
                });
                branches.extend(proxy.branches);
                leaves.extend(proxy.leaves);
            }
        }
    }
    let hierarchy = Hierarchy::new(&tree, &clusters, 2);
    Forest {
        branches,
        leaves,
        plants,
        hierarchy,
    }
}

/// How many of `n` a node keeps at most, reducing by `ratio`.
fn kept(n: usize, ratio: usize) -> usize {
    n.div_ceil(ratio).max(KEEP_AT_LEAST.min(n))
}

/// Segments merged end to end along their axes, pairwise: the first's
/// start to the second's end. Returns the merged segments and how far a
/// joint merged away lies from the segment replacing it -- the error.
fn simplify(stems: Vec<BranchInstance>) -> (Vec<BranchInstance>, f32) {
    // Each segment's continuation: the one of its axis starting where it
    // ends -- the same ring, to the bit.
    let key = |axis: u32, p: [f32; 4]| (axis, p[0].to_bits(), p[1].to_bits(), p[2].to_bits());
    let starts: HashMap<_, usize> = stems
        .iter()
        .enumerate()
        .map(|(i, s)| (key(s.axis, s.a), i))
        .collect();
    let next: Vec<Option<usize>> = stems
        .iter()
        .map(|s| starts.get(&key(s.axis, s.b)).copied())
        .collect();
    let mut continues = vec![false; stems.len()];
    for n in next.iter().flatten() {
        continues[*n] = true;
    }
    let mut out = Vec::with_capacity(stems.len() / 2 + 1);
    let mut error = 0.0f32;
    let mut done = vec![false; stems.len()];
    // Along each chain from its first segment, a pair at a time.
    for first in 0..stems.len() {
        if continues[first] || done[first] {
            continue;
        }
        let mut at = Some(first);
        while let Some(i) = at {
            done[i] = true;
            match next[i].filter(|&j| !done[j]) {
                Some(j) => {
                    done[j] = true;
                    let (s, t) = (&stems[i], &stems[j]);
                    out.push(BranchInstance {
                        b: t.b,
                        tangents: [s.tangents[0], t.tangents[1]],
                        light: t.light,
                        v1: t.v1,
                        ..*s
                    });
                    error = error.max(off_line(s.b, s.a, t.b));
                    at = next[j].filter(|&k| !done[k]);
                }
                None => {
                    out.push(stems[i]);
                    at = None;
                }
            }
        }
    }
    // A segment on no chain's walk -- none, in a tree -- stays as it is.
    for (i, s) in stems.iter().enumerate() {
        if !done[i] {
            out.push(*s);
        }
    }
    (out, error)
}

/// How far `p` lies from the segment `a`-`b`.
fn off_line(p: [f32; 4], a: [f32; 4], b: [f32; 4]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ap = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
    let l2 = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
    let t = if l2 > 0.0 {
        ((ap[0] * ab[0] + ap[1] * ab[1] + ap[2] * ab[2]) / l2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let d = [ap[0] - ab[0] * t, ap[1] - ab[1] * t, ap[2] - ab[2] * t];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// A proxy from its children's instances, its leaves merged on grids from
/// `cell` up.
fn reduce(
    stems: Vec<BranchInstance>,
    foliage: Vec<LeafInstance>,
    sphere: [f32; 4],
    cell: f32,
) -> Proxy {
    let budget = BUDGET * sphere[3];

    // Branches: axes simplified end to end, then the thinnest dropped --
    // down to half at least, and on towards a quarter while what is dropped
    // is small against the node.
    let (half, quarter) = (kept(stems.len(), HALF), kept(stems.len(), QUARTER));
    let (mut stems, mut error) = if quarter < stems.len() {
        simplify(stems)
    } else {
        (stems, 0.0)
    };
    if half < stems.len() || quarter < stems.len() {
        stems.sort_unstable_by(|x, y| y.a[3].total_cmp(&x.a[3]));
        let mut keep = stems.len();
        while keep > quarter {
            let thickness = 2.0 * stems[keep - 1].a[3].max(stems[keep - 1].b[3]);
            if keep <= half && thickness.max(error) > budget {
                break;
            }
            error = error.max(thickness);
            keep -= 1;
        }
        stems.truncate(keep);
    }

    // Leaves: merged on ever coarser grids -- to half at least, and on
    // towards a quarter while the cell stays within the budget. A cluster
    // moves at most half a cell to its card's centre: that is the error.
    let (half, quarter) = (kept(foliage.len(), HALF), kept(foliage.len(), QUARTER));
    let mut cell = cell.max(sphere[3] / 256.0).max(1e-3);
    let mut merged = merge(&foliage, cell);
    while cell < sphere[3] * 4.0
        && (merged.len() > half || (merged.len() > quarter && 0.75 * cell <= budget))
    {
        cell *= 1.5;
        merged = merge(&foliage, cell);
    }
    if merged.len() < foliage.len() {
        error = error.max(0.5 * cell);
    }
    Proxy {
        branches: stems,
        leaves: merged,
        error,
        cell,
    }
}

/// A merged card at most covers its cell: a coarse card's two blades span
/// about 1.7 times its size (`scene.wgsl`), so this is the size whose
/// blades just span one.
const CARD_PER_CELL: f32 = 0.6;

/// Leaf clusters merged per cell of edge `cell` and species: one card each,
/// at the clusters' centre weighted by area, as large as what they cover
/// together. Clusters lie in a cell as if at random, overlapping, so of its
/// area `A` they cover `A (1 - e^(-total / A))`: all of their area while
/// sparse, less once dense -- and no more than one card can.
fn merge(foliage: &[LeafInstance], cell: f32) -> Vec<LeafInstance> {
    let inverse = 1.0 / cell;
    // Per species and cell: the card, its clusters' area and their axes
    // weighted by it.
    type Gathered = (LeafInstance, f32, [f32; 3]);
    let mut cells: HashMap<(u32, [i32; 3]), Gathered> = HashMap::new();
    for leaf in foliage {
        let key = (
            leaf.species,
            leaf.center.map(|x| (x * inverse).floor() as i32),
        );
        let area = leaf.size * leaf.size;
        let entry = cells.entry(key).or_insert((
            LeafInstance {
                size: 0.0,
                center: [0.0; 3],
                axis: [0.0; 3],
                light: 0.0,
                ..*leaf
            },
            0.0,
            [0.0; 3],
        ));
        let (merged, total, axis) = entry;
        for (k, (c, a)) in merged.center.iter_mut().zip(axis.iter_mut()).enumerate() {
            *c += leaf.center[k] * area;
            *a += leaf.axis[k] * area;
        }
        merged.light += leaf.light * area;
        *total += area;
    }
    let mut out: Vec<LeafInstance> = cells
        .into_values()
        .filter(|(_, total, _)| *total > 0.0)
        .map(|(mut merged, total, axis)| {
            for k in 0..3 {
                merged.center[k] /= total;
            }
            merged.light /= total;
            let length = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
            merged.axis = if length > 1e-9 {
                axis.map(|x| x / length)
            } else {
                [0.0, 1.0, 0.0]
            };
            let area = cell * cell;
            let covered = area * -(-total / area).exp_m1();
            merged.size = covered.sqrt().min(CARD_PER_CELL * cell);
            merged
        })
        .collect();
    // The same order from one build to the next.
    out.sort_by(|a, b| {
        a.seed
            .total_cmp(&b.seed)
            .then(a.center[0].total_cmp(&b.center[0]))
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leaf(center: [f32; 3], size: f32) -> LeafInstance {
        LeafInstance {
            center,
            size,
            axis: [0.0, 1.0, 0.0],
            light: 1.0,
            ..LeafInstance::default()
        }
    }

    #[test]
    fn merging_keeps_what_the_leaves_cover() {
        let foliage: Vec<LeafInstance> = (0..100)
            .map(|i| leaf([i as f32 * 0.01, 0.0, 0.0], 0.3))
            .collect();
        let merged = merge(&foliage, 10.0);
        assert_eq!(merged.len(), 1);
        let before: f32 = foliage.iter().map(|l| l.size * l.size).sum();
        let area = merged[0].size * merged[0].size;
        assert!(area <= before && area > 0.95 * before, "{area} of {before}");
        // Crowded into a small cell, no larger than covers it.
        let merged = merge(&foliage, 1.0);
        assert!((merged[0].size - CARD_PER_CELL).abs() < 1e-3);
        assert!((merged[0].center[0] - 0.495).abs() < 1e-4);
    }

    /// A straight axis of `n` segments up from zero, on axis `axis`.
    fn axis(axis: u32, n: usize, x: f32, radius: f32) -> Vec<BranchInstance> {
        (0..n)
            .map(|i| BranchInstance {
                a: [x, i as f32, 0.0, radius],
                b: [x, i as f32 + 1.0, 0.0, radius],
                axis,
                ..BranchInstance::default()
            })
            .collect()
    }

    #[test]
    fn a_trunk_is_merged_whole_and_twigs_dropped() {
        // A trunk of 40 and twenty twigs of 3: a quarter of 100 is 25.
        let mut stems = axis(0, 40, 0.0, 0.3);
        for t in 0..20 {
            stems.extend(axis(1 + t, 3, 5.0 + t as f32, 0.005));
        }
        let proxy = reduce(stems, Vec::new(), [0.0, 20.0, 0.0, 30.0], 0.0);
        assert!(proxy.branches.len() <= 25, "{}", proxy.branches.len());
        // The trunk, in 20 pieces, still runs from 0 to 40 unbroken.
        let mut trunk: Vec<_> = proxy.branches.iter().filter(|b| b.axis == 0).collect();
        trunk.sort_by(|p, q| p.a[1].total_cmp(&q.a[1]));
        assert_eq!(trunk.len(), 20);
        for pair in trunk.windows(2) {
            assert_eq!(pair[0].b, pair[1].a);
        }
        assert_eq!((trunk[0].a[1], trunk[19].b[1]), (0.0, 40.0));
        // Straight, so merging cost nothing; the error is the twigs dropped.
        assert!((proxy.error - 0.01).abs() < 1e-6, "{}", proxy.error);
    }
}

//! The hierarchy as the GPU reads it, and the cut through it.

use crate::tree::Tree;

/// A node as the selection reads it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Node {
    /// Everything below it: centre and radius.
    pub sphere: [f32; 4],
    /// Its parent's sphere; for the root, its own.
    pub parent_sphere: [f32; 4],
    /// The largest detail its proxy leaves out, world units; zero for a leaf.
    pub error: f32,
    /// Its parent's error; infinite for the root, which is always coarse
    /// enough to stand in for.
    pub parent_error: f32,
    /// The finest level of detail it may be drawn at: a proxy's own detail
    /// is already reduced.
    pub finest_level: u32,
    pub pad: u32,
}

/// How the cut is taken: the view, and the lists it writes.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct View {
    /// The view frustum, inside where `dot(plane.xyz, p) + plane.w >= 0`.
    pub planes: [[f32; 4]; 6],
    /// The eye, and the radians a pixel subtends. For an orthographic view
    /// (a shadow map) the eye is unused.
    pub eye: [f32; 4],
    /// The error a drawn node's proxy may show, pixels.
    pub tolerance: f32,
    /// For an orthographic view, a pixel's size in world units; zero for a
    /// perspective one.
    pub texel: f32,
    /// A node whose radius spans at least this many pixels is drawn at level
    /// zero, at least `levels_px[1]` at level one, else at level two.
    pub levels_px: [f32; 2],
    /// Nodes in the hierarchy.
    pub count: u32,
    /// Instance streams: ranges per node.
    pub streams: u32,
    /// Levels this cut draws into, from `level_offset`.
    pub levels: u32,
    pub level_offset: u32,
    /// Levels in all, per stream: the draw lists' layout.
    pub total_levels: u32,
    pub pad: [u32; 3],
    /// Where each stream's lists begin in the list buffer.
    pub stream_base: [u32; 4],
    /// Each stream's list length: room for every instance it has.
    pub stream_stride: [u32; 4],
}

impl Default for View {
    fn default() -> Self {
        Self {
            planes: [[0.0, 0.0, 0.0, 1.0]; 6],
            eye: [0.0; 4],
            tolerance: 1.0,
            texel: 0.0,
            levels_px: [0.0; 2],
            count: 0,
            streams: 0,
            levels: 1,
            level_offset: 0,
            total_levels: 1,
            pad: [0; 3],
            stream_base: [0; 4],
            stream_stride: [0; 4],
        }
    }
}

/// The hierarchy the GPU selects from: nodes, and each node's instance
/// range per stream.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Hierarchy {
    pub nodes: Vec<Node>,
    /// `ranges[node * streams + stream]`: the first instance and how many.
    pub ranges: Vec<[u32; 2]>,
    pub streams: u32,
}

/// What a node is given: its proxy's error, the finest level it may be drawn
/// at, and its ranges, one per stream.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Cluster {
    pub error: f32,
    pub finest_level: u32,
    pub ranges: Vec<[u32; 2]>,
}

impl Hierarchy {
    /// The hierarchy of `tree`, its nodes given `clusters` (one per tree
    /// node, in its order). Errors are made to grow up the tree -- a node's
    /// is at least its children's -- which is what lets every node choose
    /// alone.
    pub fn new(tree: &Tree, clusters: &[Cluster], streams: u32) -> Self {
        assert_eq!(tree.nodes.len(), clusters.len());
        let count = tree.nodes.len();
        // Bottom-up: children come first.
        let mut error = vec![0.0f32; count];
        for (i, node) in tree.nodes.iter().enumerate() {
            let mut e = clusters[i].error.max(0.0);
            if let Some([a, b]) = node.children {
                e = e.max(error[a as usize]).max(error[b as usize]);
            }
            error[i] = e;
        }
        let mut nodes = Vec::with_capacity(count);
        let mut ranges = Vec::with_capacity(count * streams as usize);
        for (i, node) in tree.nodes.iter().enumerate() {
            let (parent_sphere, parent_error) = if node.parent == u32::MAX {
                (node.sphere, f32::INFINITY)
            } else {
                let p = node.parent as usize;
                (tree.nodes[p].sphere, error[p])
            };
            nodes.push(Node {
                sphere: node.sphere,
                parent_sphere,
                error: error[i],
                parent_error,
                finest_level: clusters[i].finest_level,
                pad: 0,
            });
            for s in 0..streams as usize {
                ranges.push(clusters[i].ranges.get(s).copied().unwrap_or([0, 0]));
            }
        }
        Self {
            nodes,
            ranges,
            streams,
        }
    }

    /// The cut `view` takes, on the host: each drawn node and its level.
    /// [`crate::gpu`]'s kernel is the same test, node for node.
    pub fn select(&self, view: &View) -> Vec<(u32, u32)> {
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(i, node)| decide(view, node).map(|level| (i as u32, level)))
            .collect()
    }
}

/// An error at a sphere, in pixels: over the distance to the sphere's
/// nearest point, so that a sphere holding another sees at least as much.
fn projected(view: &View, error: f32, sphere: [f32; 4]) -> f32 {
    if error.is_infinite() {
        return f32::INFINITY;
    }
    if view.texel > 0.0 {
        return error / view.texel;
    }
    let d = [
        sphere[0] - view.eye[0],
        sphere[1] - view.eye[1],
        sphere[2] - view.eye[2],
    ];
    let distance = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() - sphere[3];
    error / (distance.max(1e-4) * view.eye[3])
}

/// Whether a node is drawn, and at which level.
pub(crate) fn decide(view: &View, node: &Node) -> Option<u32> {
    for plane in &view.planes {
        let s = node.sphere;
        if plane[0] * s[0] + plane[1] * s[1] + plane[2] * s[2] + plane[3] < -s[3] {
            return None;
        }
    }
    if projected(view, node.error, node.sphere) > view.tolerance
        || projected(view, node.parent_error, node.parent_sphere) <= view.tolerance
    {
        return None;
    }
    let size = projected(view, node.sphere[3], node.sphere);
    let level = if view.texel > 0.0 || size >= view.levels_px[0] {
        0
    } else if size >= view.levels_px[1] {
        1
    } else {
        2
    };
    Some(level.max(node.finest_level).min(view.levels.max(1) - 1))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A grove of objects, every internal node's error growing with its
    /// size, as a proxy's does.
    pub(crate) fn grove(count: usize) -> (Tree, Hierarchy) {
        let spheres: Vec<[f32; 4]> = (0..count)
            .map(|i| {
                let f = i as f32;
                [
                    (f * 37.1) % 300.0,
                    (f * 0.7) % 10.0,
                    (f * 91.3) % 300.0,
                    2.0 + (f % 5.0),
                ]
            })
            .collect();
        let tree = Tree::build(&spheres);
        let clusters: Vec<Cluster> = tree
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| Cluster {
                error: if n.object.is_some() {
                    0.0
                } else {
                    0.02 * n.sphere[3]
                },
                finest_level: if n.object.is_some() { 0 } else { 2 },
                ranges: vec![[i as u32 * 10, 10], [i as u32 * 3, 3]],
            })
            .collect();
        let hierarchy = Hierarchy::new(&tree, &clusters, 2);
        (tree, hierarchy)
    }

    pub(crate) fn view_from(eye: [f32; 3], pixel: f32) -> View {
        View {
            // A view seeing everything, to test the cut alone.
            eye: [eye[0], eye[1], eye[2], pixel],
            tolerance: 1.0,
            levels_px: [200.0, 40.0],
            levels: 3,
            total_levels: 4,
            ..View::default()
        }
    }

    #[test]
    fn the_cut_covers_every_object_exactly_once() {
        let (tree, hierarchy) = grove(500);
        let mut drew_a_proxy = false;
        for (k, eye) in [
            [0.0, 2.0, 0.0],
            [150.0, 30.0, 150.0],
            [-400.0, 50.0, 900.0],
            [150.0, 3000.0, 150.0],
        ]
        .into_iter()
        .enumerate()
        {
            let cut = hierarchy.select(&view_from(eye, 0.001));
            let mut covered = vec![0u32; 500];
            for &(node, _) in &cut {
                for o in tree.objects_below(node) {
                    covered[o as usize] += 1;
                }
                drew_a_proxy |= tree.nodes[node as usize].object.is_none();
            }
            assert!(
                covered.iter().all(|&c| c == 1),
                "view {k}: {:?}",
                covered.iter().filter(|&&c| c != 1).count()
            );
        }
        assert!(drew_a_proxy, "no view drew a proxy");
    }

    #[test]
    fn a_node_outside_the_view_is_culled_with_all_below() {
        let (_, hierarchy) = grove(64);
        let mut view = view_from([150.0, 30.0, 150.0], 0.001);
        // Only x <= -20 is inside, well clear of every object: nothing is.
        view.planes[0] = [-1.0, 0.0, 0.0, -20.0];
        assert!(hierarchy.select(&view).is_empty());
    }
}

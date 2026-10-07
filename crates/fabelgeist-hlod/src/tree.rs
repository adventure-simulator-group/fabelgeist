//! A binary bounding-volume hierarchy over objects' bounding spheres.

/// A node of a [`Tree`].
#[derive(Clone, Debug, PartialEq)]
pub struct TreeNode {
    /// A sphere holding everything below: centre and radius.
    pub sphere: [f32; 4],
    /// Its two children, for an internal node.
    pub children: Option<[u32; 2]>,
    /// The object it is, for a leaf.
    pub object: Option<u32>,
    /// Its parent; `u32::MAX` for the root.
    pub parent: u32,
}

/// A binary bounding-volume hierarchy, one leaf per object. Nodes are stored
/// children before parents, the root last -- so a forward pass is bottom-up,
/// as building proxies is.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Tree {
    pub nodes: Vec<TreeNode>,
}

/// The smallest sphere found holding two spheres.
fn enclose(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let distance = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    if distance + b[3] <= a[3] {
        return a;
    }
    if distance + a[3] <= b[3] {
        return b;
    }
    let radius = 0.5 * (distance + a[3] + b[3]);
    let t = if distance > 1e-12 {
        (radius - a[3]) / distance
    } else {
        0.0
    };
    [
        a[0] + d[0] * t,
        a[1] + d[1] * t,
        a[2] + d[2] * t,
        // A hair more, so that rounding never leaves a child poking out.
        radius * (1.0 + 1e-6) + 1e-6,
    ]
}

impl Tree {
    /// A hierarchy over objects with `spheres` (centre, radius), split at
    /// the median of the centres along the widest axis, down to one object a
    /// leaf.
    pub fn build(spheres: &[[f32; 4]]) -> Self {
        let mut tree = Self::default();
        if spheres.is_empty() {
            return tree;
        }
        let mut order: Vec<u32> = (0..spheres.len() as u32).collect();
        tree.nodes.reserve(spheres.len() * 2);
        tree.split(spheres, &mut order);
        let root = tree.nodes.len() - 1;
        tree.nodes[root].parent = u32::MAX;
        tree
    }

    fn split(&mut self, spheres: &[[f32; 4]], objects: &mut [u32]) -> u32 {
        if objects.len() == 1 {
            let object = objects[0];
            self.nodes.push(TreeNode {
                sphere: spheres[object as usize],
                children: None,
                object: Some(object),
                parent: u32::MAX,
            });
            return self.nodes.len() as u32 - 1;
        }
        let mut lo = [f32::INFINITY; 3];
        let mut hi = [f32::NEG_INFINITY; 3];
        for &o in objects.iter() {
            for k in 0..3 {
                lo[k] = lo[k].min(spheres[o as usize][k]);
                hi[k] = hi[k].max(spheres[o as usize][k]);
            }
        }
        let axis = (0..3)
            .max_by(|&a, &b| (hi[a] - lo[a]).total_cmp(&(hi[b] - lo[b])))
            .unwrap_or(0);
        let middle = objects.len() / 2;
        objects.select_nth_unstable_by(middle, |&a, &b| {
            spheres[a as usize][axis]
                .total_cmp(&spheres[b as usize][axis])
                .then(a.cmp(&b))
        });
        let (left, right) = objects.split_at_mut(middle);
        let a = self.split(spheres, left);
        let b = self.split(spheres, right);
        let sphere = enclose(self.nodes[a as usize].sphere, self.nodes[b as usize].sphere);
        self.nodes.push(TreeNode {
            sphere,
            children: Some([a, b]),
            object: None,
            parent: u32::MAX,
        });
        let index = self.nodes.len() as u32 - 1;
        self.nodes[a as usize].parent = index;
        self.nodes[b as usize].parent = index;
        index
    }

    /// The objects below `node`.
    pub fn objects_below(&self, node: u32) -> Vec<u32> {
        let mut out = Vec::new();
        let mut stack = vec![node];
        while let Some(n) = stack.pop() {
            let n = &self.nodes[n as usize];
            match (n.object, n.children) {
                (Some(o), _) => out.push(o),
                (None, Some([a, b])) => {
                    stack.push(a);
                    stack.push(b);
                }
                _ => {}
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_object_is_a_leaf_inside_every_ancestor() {
        let spheres: Vec<[f32; 4]> = (0..257)
            .map(|i| {
                let f = i as f32;
                [
                    (f * 7.3) % 50.0,
                    (f * 1.7) % 5.0,
                    (f * 3.1) % 40.0,
                    0.2 + (f % 3.0),
                ]
            })
            .collect();
        let tree = Tree::build(&spheres);
        assert_eq!(tree.nodes.len(), 2 * spheres.len() - 1);
        let mut leaves: Vec<u32> = tree.nodes.iter().filter_map(|n| n.object).collect();
        leaves.sort();
        assert_eq!(leaves, (0..spheres.len() as u32).collect::<Vec<_>>());
        for (i, node) in tree.nodes.iter().enumerate() {
            if node.parent == u32::MAX {
                assert_eq!(i, tree.nodes.len() - 1, "the root is last");
                continue;
            }
            assert!((node.parent as usize) > i, "children before parents");
            let p = tree.nodes[node.parent as usize].sphere;
            let s = node.sphere;
            let d = ((p[0] - s[0]).powi(2) + (p[1] - s[1]).powi(2) + (p[2] - s[2]).powi(2)).sqrt();
            assert!(
                d + s[3] <= p[3] * (1.0 + 1e-5) + 1e-5,
                "node {i} pokes out of its parent"
            );
        }
    }
}

//! Branch module prototypes (Sec. 5.1) and the morphospace they sit in
//! (Sec. 5.2.2).
//!
//! A prototype is the *topology* of an architectural unit: a small rooted tree
//! of branch segments, given as unit directions in the module's own frame
//! (local `+Y` is the module's growth axis), relative lengths, and the
//! physiological age at which each segment starts to grow. Everything else --
//! actual lengths, diameters, orientation, tropism -- is decided per instance,
//! so one prototype serves every plant of every species in a scene.
//!
//! The library is nine prototypes after the architectural models of Hallé et
//! al. (1978), placed on a 3x3 grid in the morphospace spanned by apical
//! control `lambda` and determinacy `D`. A new module takes the prototype
//! whose position is nearest -- the Voronoi partition of the paper. A tenth,
//! Corner's model -- one unbranched axis, a palm's -- stands outside the
//! grid, for species that never branch ([`PlantType::unbranched`]).
//!
//! [`PlantType::unbranched`]: crate::PlantType::unbranched

use crate::identity::PrototypeId;
use crate::units::PhysiologicalAge;
use fabelgeist_math::{Vec3, Vec4};

/// No parent / no child.
pub const NONE: u32 = u32::MAX;

/// The physiological age at which a module is fully developed.
pub const A_MATURE: f32 = 1.0;

/// Most terminals any prototype has. Keeps a module's child table inline.
pub const MAX_TERMINALS: usize = 4;

#[derive(Clone, Debug, PartialEq)]
pub struct ProtoNode {
    /// Parent node within the prototype; [`NONE`] for the root.
    pub parent: u32,
    /// Unit direction of the segment ending at this node, module-local.
    pub direction: Vec3,
    /// Length relative to the species' segment length.
    pub length: f32,
    /// Physiological age at which this segment starts to grow.
    pub emergence: PhysiologicalAge,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Prototype {
    pub name: &'static str,
    /// Position in the `(lambda, D)` morphospace.
    pub morphospace: [f32; 2],
    /// Node 0 is the root; parents precede children.
    pub nodes: Vec<ProtoNode>,
    /// Node indices of the terminals, where child modules attach.
    pub terminals: Vec<u32>,
    /// Which terminal continues the axis (index into `terminals`).
    pub apical: usize,
    /// Child nodes per node.
    pub children: Vec<Vec<u32>>,
    /// Terminal slot per node, [`NONE`] for non-terminals.
    pub terminal_slot: Vec<u32>,
    /// Unit direction from the root to the apical terminal of a mature
    /// module: the axis tropism acts on.
    pub axis: Vec3,
}

/// A direction `theta` radians off the local `+Y` axis, at azimuth `phi`.
fn polar(theta: f32, phi: f32) -> Vec3 {
    Vec3::new(
        theta.sin() * phi.cos(),
        theta.cos(),
        theta.sin() * phi.sin(),
    )
}

struct Builder {
    nodes: Vec<ProtoNode>,
}

impl Builder {
    fn new() -> Self {
        Self {
            nodes: vec![ProtoNode {
                parent: NONE,
                direction: Vec3::new(0.0, 1.0, 0.0),
                length: 0.0,
                emergence: PhysiologicalAge::from_stage(0.0),
            }],
        }
    }

    /// Add a segment from `parent`, returning the new node.
    fn seg(&mut self, parent: u32, theta: f32, phi: f32, length: f32, emergence: f32) -> u32 {
        self.nodes.push(ProtoNode {
            parent,
            direction: polar(theta, phi),
            length,
            emergence: PhysiologicalAge::from_stage(emergence),
        });
        self.nodes.len() as u32 - 1
    }

    /// A straight chain of `count` internodes.
    fn chain(
        &mut self,
        from: u32,
        count: usize,
        theta: f32,
        phi: f32,
        length: f32,
        start: f32,
    ) -> u32 {
        let mut node = from;
        for i in 0..count {
            node = self.seg(node, theta, phi, length, start + 0.18 * i as f32);
        }
        node
    }

    fn build(self, name: &'static str, morphospace: [f32; 2], apical_node: u32) -> Prototype {
        let nodes = self.nodes;
        let mut children = vec![Vec::new(); nodes.len()];
        for (i, node) in nodes.iter().enumerate().skip(1) {
            children[node.parent as usize].push(i as u32);
        }
        let terminals: Vec<u32> = (1..nodes.len() as u32)
            .filter(|&i| children[i as usize].is_empty())
            .collect();
        assert!(
            terminals.len() <= MAX_TERMINALS,
            "{name}: too many terminals"
        );
        let mut terminal_slot = vec![NONE; nodes.len()];
        for (slot, &node) in terminals.iter().enumerate() {
            terminal_slot[node as usize] = slot as u32;
        }
        // The authored builder supplies an apical terminal. Its slot was
        // recorded above; assert the catalog contract before indexing it.
        let apical = terminal_slot[apical_node as usize] as usize;
        assert!(
            apical < terminals.len(),
            "{name}: apical node is not a terminal"
        );

        // Mature, untropised position of the apical terminal.
        let mut positions = vec![Vec3::new(0.0, 0.0, 0.0); nodes.len()];
        for i in 1..nodes.len() {
            let n = &nodes[i];
            positions[i] = positions[n.parent as usize] + n.direction * n.length;
        }
        let axis = positions[apical_node as usize].normalize();

        Prototype {
            name,
            morphospace,
            nodes,
            terminals,
            apical,
            children,
            terminal_slot,
            axis,
        }
    }
}

const GOLDEN: f32 = 2.399_963; // 137.5 degrees

impl Prototype {
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// The rotation a child attached at terminal `slot` starts from, relative
    /// to this module's frame: local `+Y` turned onto that terminal's segment.
    pub fn terminal_frame(&self, slot: usize) -> Vec4 {
        let direction = self.nodes[self.terminals[slot] as usize].direction;
        align_y(direction)
    }
}

/// The shortest rotation taking `+Y` onto unit `to`.
pub fn align_y(to: Vec3) -> Vec4 {
    let from = Vec3::new(0.0, 1.0, 0.0);
    let d = from.dot(to);
    if d > 1.0 - 1e-6 {
        return Vec4::quat_identity();
    }
    if d < -1.0 + 1e-6 {
        return Vec4::from_axis_angle(Vec3::new(1.0, 0.0, 0.0), std::f32::consts::PI);
    }
    let axis = from.cross(to);
    Vec4::new(axis.x, axis.y, axis.z, 1.0 + d).normalize()
}

/// The nine-prototype morphospace, and Corner's model beside it.
#[derive(Clone, Debug)]
pub struct PrototypeLibrary {
    pub prototypes: Vec<Prototype>,
    /// Corner's model: one axis that never branches. Outside the
    /// morphospace; [`PrototypeLibrary::select`] never picks it.
    pub unbranched: PrototypeId,
}

impl Default for PrototypeLibrary {
    fn default() -> Self {
        Self::halle()
    }
}

impl PrototypeLibrary {
    /// A library of `prototypes`, and Corner's model after them.
    pub fn new(mut prototypes: Vec<Prototype>) -> Self {
        let unbranched = PrototypeId::from_word(prototypes.len() as u16);
        // Corner: a single axis, every leaf from its tip.
        let mut b = Builder::new();
        let apex = b.chain(0, 3, 0.0, 0.0, 1.0, 0.0);
        prototypes.push(b.build("corner", [f32::NAN, f32::NAN], apex));
        Self {
            prototypes,
            unbranched,
        }
    }

    /// Nine prototypes on a 3x3 grid of `(lambda, D)`.
    ///
    /// Rows by determinacy: high `D` gives monopodial units (a continuing
    /// axis carrying laterals -- Rauh, Massart, Roux), middle `D` a leader
    /// with a subordinate fork (Troll, Scarrone), low `D` sympodial units
    /// ending in a fork (Leeuwenberg, Koriba). Columns by apical control:
    /// high `lambda` keeps the laterals short and steep relative to the axis,
    /// low `lambda` makes them long and even.
    pub fn halle() -> Self {
        use std::f32::consts::FRAC_PI_2 as HALF;
        let deg = |d: f32| d.to_radians();
        let mut prototypes = Vec::new();

        // --- D high: monopodial --------------------------------------------
        {
            // Whorled (Massart): axis of three, a whorl of three laterals.
            let mut b = Builder::new();
            let n1 = b.chain(0, 1, 0.0, 0.0, 1.0, 0.0);
            let apex = b.chain(n1, 2, deg(3.0), 0.0, 0.9, 0.18);
            for k in 0..3 {
                b.seg(n1, deg(72.0), k as f32 * deg(120.0), 0.7, 0.25);
            }
            prototypes.push(b.build("whorled", [0.85, 0.85], apex));
        }
        {
            // Alternate (Rauh): laterals spiralling up the axis.
            let mut b = Builder::new();
            let n1 = b.chain(0, 1, 0.0, 0.0, 1.0, 0.0);
            let n2 = b.chain(n1, 1, deg(4.0), 0.0, 0.9, 0.18);
            let apex = b.chain(n2, 1, deg(4.0), GOLDEN, 0.8, 0.36);
            b.seg(n1, deg(55.0), 0.0, 0.8, 0.25);
            b.seg(n2, deg(55.0), GOLDEN, 0.8, 0.43);
            prototypes.push(b.build("alternate", [0.5, 0.85], apex));
        }
        {
            // Opposite, long laterals (Roux with weak control).
            let mut b = Builder::new();
            let n1 = b.chain(0, 1, 0.0, 0.0, 1.0, 0.0);
            let apex = b.chain(n1, 1, deg(5.0), 0.0, 0.9, 0.18);
            let l1 = b.seg(n1, deg(48.0), 0.0, 0.8, 0.22);
            let _ = l1;
            b.seg(n1, deg(48.0), HALF * 2.0, 0.8, 0.22);
            prototypes.push(b.build("opposite", [0.15, 0.85], apex));
        }

        // --- D middle: leader with a subordinate fork ----------------------
        {
            let mut b = Builder::new();
            let n1 = b.chain(0, 1, 0.0, 0.0, 1.0, 0.0);
            let n2 = b.chain(n1, 1, deg(4.0), 0.0, 0.9, 0.18);
            let apex = b.seg(n2, deg(8.0), GOLDEN, 0.8, 0.36);
            b.seg(n2, deg(50.0), GOLDEN + HALF * 2.0, 0.7, 0.4);
            b.seg(n1, deg(62.0), 0.0, 0.6, 0.25);
            prototypes.push(b.build("leader", [0.85, 0.5], apex));
        }
        {
            let mut b = Builder::new();
            let n1 = b.chain(0, 1, 0.0, 0.0, 1.0, 0.0);
            let n2 = b.chain(n1, 1, deg(6.0), 0.0, 0.8, 0.18);
            let apex = b.seg(n2, deg(20.0), 0.0, 0.8, 0.36);
            b.seg(n2, deg(42.0), HALF * 2.0, 0.8, 0.36);
            b.seg(n1, deg(55.0), HALF, 0.6, 0.25);
            prototypes.push(b.build("forked leader", [0.5, 0.5], apex));
        }
        {
            let mut b = Builder::new();
            let n1 = b.chain(0, 1, 0.0, 0.0, 1.0, 0.0);
            let apex = b.seg(n1, deg(30.0), 0.0, 0.9, 0.18);
            b.seg(n1, deg(34.0), deg(120.0), 0.9, 0.18);
            b.seg(n1, deg(34.0), deg(240.0), 0.9, 0.18);
            prototypes.push(b.build("trifurcation", [0.15, 0.5], apex));
        }

        // --- D low: sympodial ---------------------------------------------
        {
            // Koriba: a dominant relay with one weaker fork.
            let mut b = Builder::new();
            let n1 = b.chain(0, 2, 0.0, 0.0, 0.9, 0.0);
            let apex = b.seg(n1, deg(15.0), 0.0, 0.8, 0.36);
            b.seg(n1, deg(40.0), HALF * 2.0, 0.7, 0.36);
            prototypes.push(b.build("relay", [0.85, 0.15], apex));
        }
        {
            // Leeuwenberg: an axis ending in three equal branches.
            let mut b = Builder::new();
            let n1 = b.chain(0, 1, 0.0, 0.0, 1.0, 0.0);
            let apex = b.seg(n1, deg(30.0), 0.0, 0.9, 0.18);
            b.seg(n1, deg(30.0), deg(120.0), 0.9, 0.18);
            b.seg(n1, deg(30.0), deg(240.0), 0.9, 0.18);
            prototypes.push(b.build("leeuwenberg", [0.5, 0.15], apex));
        }
        {
            // A wide dichotomy.
            let mut b = Builder::new();
            let n1 = b.chain(0, 1, 0.0, 0.0, 1.0, 0.0);
            let apex = b.seg(n1, deg(38.0), 0.0, 1.0, 0.18);
            b.seg(n1, deg(38.0), HALF * 2.0, 1.0, 0.18);
            prototypes.push(b.build("dichotomy", [0.15, 0.15], apex));
        }

        Self::new(prototypes)
    }

    pub fn len(&self) -> usize {
        self.prototypes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.prototypes.is_empty()
    }

    /// The prototype whose morphospace cell contains `(lambda, d)`.
    pub fn select(&self, lambda: f32, d: f32) -> PrototypeId {
        let mut best = 0;
        let mut best_distance = f32::INFINITY;
        for (i, p) in self.prototypes.iter().enumerate() {
            if i == self.unbranched.index() {
                continue;
            }
            let dl = p.morphospace[0] - lambda;
            let dd = p.morphospace[1] - d;
            let distance = dl * dl + dd * dd;
            if distance < best_distance {
                best_distance = distance;
                best = i;
            }
        }
        PrototypeId::from_word(best as u16)
    }

    pub fn get(&self, index: PrototypeId) -> &Prototype {
        &self.prototypes[index.index()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prototypes_are_well_formed() {
        let library = PrototypeLibrary::halle();
        assert_eq!(library.len(), 10);
        for p in &library.prototypes {
            assert!(!p.terminals.is_empty(), "{}", p.name);
            for (i, n) in p.nodes.iter().enumerate().skip(1) {
                assert!(
                    (n.parent as usize) < i,
                    "{}: parents precede children",
                    p.name
                );
                assert!((n.direction.length() - 1.0).abs() < 1e-5);
                assert!(n.emergence.stage() >= 0.0 && n.emergence.stage() < A_MATURE);
            }
            assert!(p.axis.y > 0.5, "{} grows upwards", p.name);
        }
    }

    #[test]
    fn selection_is_the_nearest_prototype() {
        let library = PrototypeLibrary::halle();
        assert_eq!(library.get(library.select(0.9, 0.9)).name, "whorled");
        assert_eq!(library.get(library.select(0.1, 0.1)).name, "dichotomy");
        assert_eq!(library.get(library.select(0.5, 0.45)).name, "forked leader");
        assert_eq!(library.get(library.unbranched).name, "corner");
        for (lambda, d) in [(1.0, 1.0), (0.0, 0.0), (1.0, 0.0), (0.0, 1.0)] {
            assert_ne!(library.select(lambda, d), library.unbranched);
        }
    }

    #[test]
    fn align_y_turns_up_onto_the_target() {
        for to in [
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, -1.0, 0.0),
            polar(0.7, 2.0),
        ] {
            let q = align_y(to);
            let r = q.rotate_vec3(Vec3::new(0.0, 1.0, 0.0));
            assert!((r - to).length() < 1e-5, "{r:?} vs {to:?}");
        }
    }
}

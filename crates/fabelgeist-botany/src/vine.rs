//! Climbing plants (not in the paper): vines that search for something to
//! climb, hold on to walls and bark, and hang where they find nothing.
//!
//! A vine is a tree of short internodes laid down by its shoot tips, after
//! the climbing-plant model of Hädrich et al. (*Interactive Modeling and
//! Authoring of Climbing Plants*, 2017) and the searcher shoots Rowe and
//! Speck measured:
//!
//! * **Searching**: a free tip grows on the way it was going, upwards
//!   (negative gravitropism), and towards the nearest support it can climb
//!   within its sensing range -- ivy's shoots grow towards the dark of a
//!   wall or a trunk (skototropism), a twiner's circumnutation sweeps for a
//!   stem. It never grows into a solid or into bark: it slides along it.
//! * **Holding**: a node laid within its adhesion reach of a support it can
//!   climb holds on -- ivy's adventitious roots to masonry, bark and the
//!   ground; a twiner only to stems thin enough to wind round. A held shoot
//!   crawls over the surface, its heading in the surface's tangent plane, as
//!   far up as it can, a twiner spiralling round its stem. On bark a node
//!   keeps its place on the branch -- which segment, how far along it, which
//!   way round -- so it rides on the tree as the branch grows, thickens and
//!   sags, and falls when the branch is shed.
//! * **Hanging**: what is not held is a cantilever from its last hold, and
//!   bends under its own weight and its leaves' by beam theory, as the
//!   trees' limbs do: a searcher stands out a few tens of centimetres and
//!   droops; a shoot grown over the top of a wall hangs down it.
//! * **Light**: each node's leaves are a sphere in the environment passes,
//!   so a vine is shaded by the canopy it climbs into, and shades it back. A
//!   shoot lengthens at its species' rate, slower in shade, as a tree's
//!   shoots do, and not at all past its hydraulic ceiling; a shoot left in
//!   the dark stops, and new shoots break from the stem where it is lit.
//! * **Wood**: stems thicken by the pipe model over the leaves they feed,
//!   and ring by ring, as the trees' do.

use std::collections::HashMap;

use fabelgeist_math::Vec3;
use fabelgeist_spatial_hash::HashGrid;

use crate::obstacles::Obstacles;
use crate::plant::{Plant, hydraulic, light_response};
use crate::prototype::{NONE, PrototypeLibrary};
use crate::rng::Rng;
use crate::terrain::Terrain;

const G: f32 = 9.81;

/// How a vine holds on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Climbing {
    /// Adventitious roots or adhesive pads (ivy, Virginia creeper): any
    /// surface it touches -- masonry, bark, the ground -- however curved.
    Roots,
    /// Twining stems or tendrils (honeysuckle, hop, clematis): only stems it
    /// can wind round, up to `max_support` across. Twiners rarely climb
    /// supports much over ten centimetres (Putz 1984).
    Twining { max_support: f32 },
}

/// Everything that distinguishes one climber from another. Lengths are
/// metres, times years.
#[derive(Clone, Debug, PartialEq)]
pub struct VineType {
    pub name: String,
    pub climbing: Climbing,
    /// How fast a shoot lengthens in full light, m a year.
    pub extension: f32,
    /// Length of an internode: the stem between two leaf nodes.
    pub internode: f32,
    /// Diameter of a new shoot.
    pub shoot_diameter: f32,
    /// Share of the sapwood joining the heartwood each year
    /// ([`crate::PlantType::heartwood_rate`]).
    pub heartwood_rate: f32,
    /// Young's modulus of the green stem, Pa.
    pub stiffness: f32,
    /// Fresh stem density, kg/m^3.
    pub density: f32,
    /// Mass of the leaves on a metre of leafy shoot, kg.
    pub leaf_mass: f32,
    /// How far a searching tip finds a support to grow towards.
    pub sensing: f32,
    /// How far from a support a node takes hold of it.
    pub adhesion: f32,
    /// New shoots a metre of growth puts out, in full light.
    pub branching: f32,
    /// Most shoots growing at once.
    pub max_shoots: u32,
    /// Most nodes a vine lays.
    pub max_nodes: usize,
    /// How long a shoot grows on without taking hold before it gives up,
    /// hanging.
    pub max_free: f32,
    /// A twiner's turn round its support against its climb: zero climbs
    /// straight, one at forty-five degrees.
    pub twist: f32,
    /// Weights of a tip's heading: on as it was going, upwards, towards a
    /// support it senses, and at random (circumnutation).
    pub persistence: f32,
    pub gravitropism: f32,
    pub attraction: f32,
    pub wander: f32,
    /// The highest above its root a shoot still lengthens
    /// ([`crate::PlantType::max_height`]).
    pub max_height: f32,
    pub shade_tolerance: f32,
    /// The light a shoot tip needs to keep growing, as a share of open sky.
    pub light_compensation: f32,
    /// Edge of a leaf cluster.
    pub leaf_size: f32,
    /// Years a node keeps its leaves, or `None` for as long as it is lit.
    pub leaf_years: Option<f32>,
    pub leaf_color: [f32; 3],
    pub bark_color: [f32; 3],
    pub max_age: f32,
}

impl VineType {
    /// English ivy, *Hedera helix*: an evergreen root climber, shade
    /// tolerant, on walls and trunks alike. Stem stiffness after Melzer et
    /// al. (2010); a juvenile shoot a few millimetres across.
    pub fn ivy() -> Self {
        Self {
            name: "ivy".into(),
            climbing: Climbing::Roots,
            extension: 1.2,
            internode: 0.18,
            shoot_diameter: 0.004,
            heartwood_rate: 0.03,
            stiffness: 1.5e9,
            density: 950.0,
            leaf_mass: 0.04,
            sensing: 1.5,
            adhesion: 0.04,
            branching: 0.7,
            max_shoots: 36,
            max_nodes: 6000,
            max_free: 1.6,
            twist: 0.0,
            persistence: 1.0,
            gravitropism: 0.8,
            attraction: 1.6,
            wander: 0.45,
            max_height: 25.0,
            shade_tolerance: 0.85,
            light_compensation: 0.05,
            leaf_size: 0.16,
            leaf_years: None,
            leaf_color: [0.10, 0.26, 0.09],
            bark_color: [0.42, 0.36, 0.28],
            max_age: 400.0,
        }
    }

    /// Common honeysuckle, *Lonicera periclymenum*: a deciduous twiner of
    /// hedges and woodland edges, climbing shrubs and saplings, not walls.
    pub fn honeysuckle() -> Self {
        Self {
            name: "honeysuckle".into(),
            climbing: Climbing::Twining { max_support: 0.1 },
            extension: 2.0,
            internode: 0.2,
            shoot_diameter: 0.005,
            heartwood_rate: 0.02,
            stiffness: 2.0e9,
            density: 900.0,
            leaf_mass: 0.03,
            sensing: 0.8,
            adhesion: 0.05,
            branching: 0.5,
            max_shoots: 20,
            max_nodes: 3000,
            max_free: 1.2,
            twist: 1.0,
            persistence: 0.8,
            gravitropism: 1.0,
            attraction: 2.0,
            wander: 0.5,
            max_height: 8.0,
            shade_tolerance: 0.5,
            light_compensation: 0.12,
            leaf_size: 0.16,
            leaf_years: Some(1.0),
            leaf_color: [0.30, 0.46, 0.20],
            bark_color: [0.50, 0.38, 0.30],
            max_age: 60.0,
        }
    }

    fn stem_radius(&self) -> f32 {
        self.shoot_diameter * 0.5
    }

    /// Radius of a node's leaves as a sphere in the environment passes.
    pub fn leaf_radius(&self) -> f32 {
        self.leaf_size * 0.75
    }

    /// Whether it can take hold of what `contact` touches.
    pub fn holds(&self, contact: &Contact) -> bool {
        match (self.climbing, contact.kind) {
            (Climbing::Roots, _) => true,
            (Climbing::Twining { max_support }, Support::Bark { .. }) => {
                contact.diameter <= max_support
            }
            (Climbing::Twining { .. }, _) => false,
        }
    }
}

/// What holds a node.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Hold {
    /// Nothing: it hangs from the nearest hold below it.
    Free,
    /// Masonry or the ground: fixed where it took hold.
    Fixed,
    /// A branch, which may move.
    Bark(BarkHold),
}

/// A place on a branch: which segment, how far along it, which way round.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BarkHold {
    /// [`Plant::id`] and [`crate::Module::id`]: lasting.
    pub plant: u64,
    pub module: u32,
    /// The segment ending at this node of the module.
    pub node: u16,
    /// Along the segment, `0..=1`.
    pub t: f32,
    /// Round it, from [`basis`]'s first axis.
    pub angle: f32,
    /// Where the plant and the module are now in the ecosystem: refreshed
    /// whenever plants are shed or reordered.
    pub plant_index: u32,
    pub module_index: u32,
}

/// One node of a vine: the end of the internode from its parent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VineNode {
    pub position: Vec3,
    pub parent: u32,
    /// The way its internode ran when it was laid, and how long it is.
    pub direction: Vec3,
    pub length: f32,
    /// Its stem: sapwood round heartwood, as a plant's.
    pub diameter: f32,
    pub sapwood: f32,
    pub heartwood: f32,
    /// The vine's age when it was laid.
    pub born: f32,
    pub hold: Hold,
    /// The outward normal of what it holds; zero when free.
    pub normal: Vec3,
    /// `Q_eff`, `Q Q_G` and the light it is drawn in, as a module's.
    pub light: f32,
    pub exposure: f32,
    pub lit: f32,
    /// How far it hangs below the way it was laid: the slope its load has
    /// bent into the stem from its hold to here, less what it was already
    /// bent when laid.
    pub sag: f32,
    pub birth_sag: f32,
    /// Starts a shoot of its own rather than carrying its parent's on.
    pub lateral: bool,
    /// Years its leaves have stood below the compensation point without a
    /// break, and how far they have fallen since, zero to one. Leaves fall
    /// after a year in the dark, over half a year, and old wood does not
    /// leaf again: a step's light going up and down about the compensation
    /// point leaves them be.
    pub dark: f32,
    pub bare: f32,
}

/// A growing shoot tip.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shoot {
    /// The node it is laying down: its internode lengthens.
    pub node: u32,
    pub heading: Vec3,
    /// How fast it lengthens now, m a year.
    pub rate: f32,
    pub active: bool,
}

#[derive(Clone, Debug)]
pub struct Vine {
    pub id: u64,
    pub species: u16,
    /// Where it is rooted.
    pub position: Vec3,
    pub age: f32,
    pub nodes: Vec<VineNode>,
    pub shoots: Vec<Shoot>,
    pub alive: bool,
    /// Is kept rather than dying ([`Plant::immortal`]).
    pub immortal: bool,
    /// Kept as it was where it would have died ([`Plant::preserved`]): it
    /// grows no more, but holds on to what it climbed.
    pub preserved: bool,
    rng: Rng,
}

/// What a contact touches.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Support {
    Ground,
    Masonry,
    /// A branch segment: its index in the [`BarkIndex`].
    Bark {
        segment: u32,
    },
}

/// The nearest point of a support.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Contact {
    pub point: Vec3,
    pub normal: Vec3,
    /// From the query point to the surface, negative inside.
    pub distance: f32,
    pub kind: Support,
    /// A branch's diameter there, and its axis; infinite and zero for the
    /// ground and masonry.
    pub diameter: f32,
    pub axis: Vec3,
}

/// A branch segment of a plant: the one ending at `node` of a module.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BarkSegment {
    pub plant: u32,
    pub module: u32,
    pub node: u16,
}

/// The branch segments near vines, hashed by their middles.
#[derive(Default)]
pub struct BarkIndex {
    pub segments: Vec<BarkSegment>,
    grid: HashGrid,
    /// The furthest any segment's surface reaches from its middle.
    reach: f32,
}

/// Cell of the bark index, metres.
const BARK_CELL: f32 = 1.0;

impl BarkIndex {
    /// The segments of every plant `wanted` picks.
    pub fn build(
        plants: &[Plant],
        prototypes: &PrototypeLibrary,
        wanted: impl Fn(&Plant) -> bool,
    ) -> Self {
        let mut segments = Vec::new();
        let mut middles = Vec::new();
        let mut reach = 0.0f32;
        for (p, plant) in plants.iter().enumerate() {
            if !plant.alive || !wanted(plant) {
                continue;
            }
            for (m, module) in plant.modules.iter().enumerate() {
                let prototype = prototypes.get(module.prototype);
                if module.node_offset == NONE {
                    continue;
                }
                let offset = module.node_offset as usize;
                for (n, node) in prototype.nodes.iter().enumerate().skip(1) {
                    let (Some(a), Some(b)) = (
                        plant.nodes.get(offset + node.parent as usize),
                        plant.nodes.get(offset + n),
                    ) else {
                        continue;
                    };
                    if b.diameter <= 0.0 {
                        continue;
                    }
                    let middle = (a.position + b.position) * 0.5;
                    reach = reach.max((b.position - a.position).length() * 0.5 + b.diameter * 0.5);
                    segments.push(BarkSegment {
                        plant: p as u32,
                        module: m as u32,
                        node: n as u16,
                    });
                    middles.push(middle.to_array());
                }
            }
        }
        Self {
            grid: HashGrid::new(middles.into_iter(), BARK_CELL),
            segments,
            reach,
        }
    }
}

/// Everything a vine can touch, as it is now.
pub struct Supports<'a> {
    pub terrain: &'a Terrain,
    pub obstacles: Option<&'a Obstacles>,
    pub plants: &'a [Plant],
    pub prototypes: &'a PrototypeLibrary,
    pub bark: &'a BarkIndex,
}

/// An orthonormal pair round unit `n` that turns smoothly with it, singular
/// only straight down, where no branch grows: Frisvad's construction with
/// its axes cycled, as the viewer's tubes use it.
pub fn basis(n: Vec3) -> (Vec3, Vec3) {
    let m = Vec3::new(n.z, n.x, n.y);
    let a = 1.0 / (1.0 + m.z).max(1e-6);
    let b = -m.x * m.y * a;
    let f1 = Vec3::new(1.0 - m.x * m.x * a, b, -m.x);
    let f2 = Vec3::new(b, 1.0 - m.y * m.y * a, -m.y);
    (Vec3::new(f1.y, f1.z, f1.x), Vec3::new(f2.y, f2.z, f2.x))
}

/// `v` turned towards straight down by `angle`, but never past it.
fn droop(v: Vec3, angle: f32) -> Vec3 {
    let down = Vec3::new(0.0, -1.0, 0.0);
    let axis = v.cross(down);
    let s = axis.length();
    if angle <= 0.0 || s < 1e-6 {
        return v;
    }
    let k = axis / s;
    let a = angle.min(v.dot(down).clamp(-1.0, 1.0).acos());
    v * a.cos() + k.cross(v) * a.sin()
}

impl Supports<'_> {
    /// A bark segment as the plant has it now: its ends and their radii.
    pub fn segment(&self, s: &BarkSegment) -> Option<(Vec3, Vec3, f32, f32)> {
        let plant = self.plants.get(s.plant as usize)?;
        let module = plant.modules.get(s.module as usize)?;
        let prototype = self.prototypes.get(module.prototype);
        let n = s.node as usize;
        if n == 0 || n >= prototype.node_count() || module.node_offset == NONE {
            return None;
        }
        let offset = module.node_offset as usize;
        let b = plant.nodes.get(offset + n)?;
        let a = plant
            .nodes
            .get(offset + prototype.nodes[n].parent as usize)?
            .position;
        let end = if b.end_diameter > 0.0 {
            b.end_diameter.min(b.diameter)
        } else {
            b.diameter * 0.5
        };
        Some((a, b.position, b.diameter * 0.5, end * 0.5))
    }

    fn ground(&self, p: Vec3) -> Contact {
        let h = self.terrain.height_at(p.x, p.z);
        let e = 0.25;
        let dx = self.terrain.height_at(p.x + e, p.z) - self.terrain.height_at(p.x - e, p.z);
        let dz = self.terrain.height_at(p.x, p.z + e) - self.terrain.height_at(p.x, p.z - e);
        let normal = Vec3::new(-dx / (2.0 * e), 1.0, -dz / (2.0 * e)).normalize();
        let distance = (p.y - h) * normal.y;
        Contact {
            point: p - normal * distance,
            normal,
            distance,
            kind: Support::Ground,
            diameter: f32::INFINITY,
            axis: Vec3::new(0.0, 0.0, 0.0),
        }
    }

    /// The nearest surface within `range` of `p` that `accept` takes.
    pub fn nearest(
        &self,
        p: Vec3,
        range: f32,
        accept: impl Fn(&Contact) -> bool,
    ) -> Option<Contact> {
        let mut best: Option<Contact> = None;
        let mut offer = |c: Contact| {
            if c.distance < range && best.is_none_or(|b| c.distance < b.distance) && accept(&c) {
                best = Some(c);
            }
        };
        offer(self.ground(p));
        if let Some(obstacles) = self.obstacles {
            let distance = obstacles.distance(p);
            if distance < range
                && let Some(normal) = obstacles.normal(p)
            {
                offer(Contact {
                    point: p - normal * distance,
                    normal,
                    distance,
                    kind: Support::Masonry,
                    diameter: f32::INFINITY,
                    axis: Vec3::new(0.0, 0.0, 0.0),
                });
            }
        }
        let cells = ((range + self.bark.reach) / BARK_CELL).ceil() as i32;
        self.bark.grid.for_each_near(p.to_array(), cells, |i| {
            let Some((a, b, ra, rb)) = self.segment(&self.bark.segments[i as usize]) else {
                return;
            };
            let ab = b - a;
            let length2 = ab.dot(ab).max(1e-12);
            let t = ((p - a).dot(ab) / length2).clamp(0.0, 1.0);
            let c = a + ab * t;
            let r = ra + (rb - ra) * t;
            let v = p - c;
            let d = v.length();
            if d < 1e-6 {
                return;
            }
            let normal = v / d;
            offer(Contact {
                point: c + normal * r,
                normal,
                distance: d - r,
                kind: Support::Bark { segment: i },
                diameter: 2.0 * r,
                axis: ab / length2.sqrt(),
            });
        });
        best
    }

    /// Where a hold on bark is now, `lift` out from the surface, and the
    /// surface's normal there; `None` if the branch is gone.
    pub fn bark_point(&self, hold: &BarkHold, lift: f32) -> Option<(Vec3, Vec3)> {
        let plant = self
            .plants
            .get(hold.plant_index as usize)
            .filter(|p| p.id == hold.plant)?;
        plant
            .modules
            .get(hold.module_index as usize)
            .filter(|m| m.id == hold.module)?;
        let segment = BarkSegment {
            plant: hold.plant_index,
            module: hold.module_index,
            node: hold.node,
        };
        let (a, b, ra, rb) = self.segment(&segment)?;
        let axis = (b - a).normalize();
        let (f1, f2) = basis(axis);
        let out = f1 * hold.angle.cos() + f2 * hold.angle.sin();
        let r = ra + (rb - ra) * hold.t;
        Some((a + (b - a) * hold.t + out * (r + lift), out))
    }
}

fn random_unit(rng: &mut Rng) -> Vec3 {
    Vec3::new(rng.gaussian(), rng.gaussian(), rng.gaussian()).normalize()
}

impl Vine {
    /// A seedling rooted at `position`, its first shoot leaning a little.
    pub fn new(
        id: u64,
        species_index: u16,
        species: &VineType,
        position: Vec3,
        mut rng: Rng,
    ) -> Self {
        let lean = random_unit(&mut rng) * 0.3;
        let heading = (Vec3::new(0.0, 1.0, 0.0) + lean).normalize();
        let root = VineNode {
            position,
            parent: NONE,
            direction: Vec3::new(0.0, 1.0, 0.0),
            length: 0.0,
            diameter: species.shoot_diameter,
            sapwood: species.shoot_diameter,
            heartwood: 0.0,
            born: 0.0,
            hold: Hold::Fixed,
            normal: Vec3::new(0.0, 1.0, 0.0),
            light: 1.0,
            exposure: 1.0,
            lit: 1.0,
            sag: 0.0,
            birth_sag: 0.0,
            lateral: false,
            dark: 0.0,
            bare: 0.0,
        };
        let tip = VineNode {
            parent: 0,
            direction: heading,
            hold: Hold::Free,
            normal: Vec3::new(0.0, 0.0, 0.0),
            ..root
        };
        Self {
            id,
            species: species_index,
            position,
            age: 0.0,
            nodes: vec![root, tip],
            shoots: vec![Shoot {
                node: 1,
                heading,
                rate: species.extension * 0.5,
                active: true,
            }],
            alive: true,
            immortal: false,
            preserved: false,
            rng,
        }
    }

    /// How leafy a node is, zero to one: its leaves open over its first
    /// months, are kept for as long as the species keeps them, and fall
    /// after a year in the dark ([`VineNode::bare`]).
    pub fn leafiness(&self, node: &VineNode, species: &VineType) -> f32 {
        let years = self.age - node.born;
        let opening = (years / 0.3).clamp(0.0, 1.0);
        let kept = species
            .leaf_years
            .map_or(1.0, |limit| (1.0 - (years - limit) / 0.5).clamp(0.0, 1.0));
        opening * kept * (1.0 - node.bare)
    }

    /// The sphere each node's leaves fill, and its parent's index among
    /// them -- for the environment passes. A held node's leaves stand out
    /// from what it holds.
    pub fn spheres(&self, species: &VineType) -> impl Iterator<Item = ([f32; 4], u32)> + '_ {
        let leaf = species.leaf_radius();
        let stem = species.stem_radius();
        let leafy: Vec<f32> = self
            .nodes
            .iter()
            .map(|n| self.leafiness(n, species))
            .collect();
        self.nodes.iter().zip(leafy).map(move |(node, leafy)| {
            let radius = (leaf * leafy).max(stem * 4.0);
            let c = node.position + node.normal * radius;
            ([c.x, c.y, c.z, radius], node.parent)
        })
    }

    /// Take the light the environment measured, `[Q, Q_G]` per node.
    pub fn set_light(&mut self, light: &[[f32; 2]], species: &VineType, shadowing: bool) {
        for (node, &[q, qg]) in self.nodes.iter_mut().zip(light) {
            node.exposure = if shadowing { q * qg } else { q };
            node.light = if shadowing {
                q * (species.shade_tolerance + (1.0 - species.shade_tolerance) * qg)
            } else {
                q
            };
        }
    }

    /// Grow for `years` at the rates the last [`Vine::regulate`] set: tips
    /// lay internodes, holds follow the branches they are on, and what hangs
    /// settles under its weight.
    pub fn develop(&mut self, species: &VineType, supports: &Supports, years: f32) {
        if !self.alive || years <= 0.0 {
            return;
        }
        if self.preserved {
            // Still on the branches it holds, wherever they grow.
            self.settle(species, supports);
            return;
        }
        self.age += years;
        let ease = 1.0 - (-years / 0.3).exp();
        for node in &mut self.nodes {
            node.lit += (node.light - node.lit) * ease;
            if node.bare > 0.0 {
                node.bare = (node.bare + years / 0.5).min(1.0);
            }
        }
        for s in 0..self.shoots.len() {
            let mut left = self.shoots[s].rate * years;
            while left > 0.0 && self.shoots[s].active {
                let tip = self.shoots[s].node as usize;
                let room = species.internode - self.nodes[tip].length;
                if left < room {
                    self.nodes[tip].length += left;
                    break;
                }
                self.nodes[tip].length = species.internode;
                left -= room;
                self.place(tip, species, supports);
                self.lay(s, species, supports);
            }
        }
        self.settle(species, supports);
    }

    /// A free node's place: from its parent's, the way it was laid, bent
    /// down by the sag it has taken on since -- resting on whatever it hangs
    /// against rather than passing into it.
    fn place(&mut self, i: usize, species: &VineType, supports: &Supports) {
        let node = self.nodes[i];
        if node.hold != Hold::Free || node.parent == NONE {
            return;
        }
        let from = self.nodes[node.parent as usize].position;
        let mut p = from + droop(node.direction, node.sag - node.birth_sag) * node.length;
        let r = species.stem_radius();
        if let Some(c) = supports.nearest(p, r, |_| true) {
            p = c.point + c.normal * r;
        }
        self.nodes[i].position = p;
    }

    /// The shoot's tip internode is grown: fix where it ends -- held if it
    /// has come to something it can hold -- and start the next.
    fn lay(&mut self, s: usize, species: &VineType, supports: &Supports) {
        let tip = self.shoots[s].node as usize;
        let r = species.stem_radius();
        let mut p = self.nodes[tip].position;
        // Out of whatever it grew into.
        if let Some(c) = supports.nearest(p, r, |_| true) {
            p = c.point + c.normal * r;
        }
        let mut hold = Hold::Free;
        let mut normal = Vec3::new(0.0, 0.0, 0.0);
        let mut contact = None;
        // It takes hold where the growing internode first touched something
        // it can climb -- before the ground it may be crawling on -- not
        // only where it ends, or it would step past a thin stem.
        let reach = r + species.adhesion;
        let hold_at = |q: Vec3| {
            supports
                .nearest(q, reach, |c| c.kind != Support::Ground && species.holds(c))
                .or_else(|| supports.nearest(q, reach, |c| species.holds(c)))
        };
        let parent = self.nodes[self.nodes[tip].parent as usize];
        let start = parent.position;
        let found = if parent.hold != Hold::Free {
            // Held, it has crept over the surface rather than out from it.
            p = self.creep(
                start,
                self.shoots[s].heading,
                self.nodes[tip].length,
                species,
                supports,
            );
            hold_at(p)
        } else {
            (1..=4).find_map(|k| hold_at(start + (p - start) * (k as f32 / 4.0)))
        };
        if let Some(c) = found {
            p = c.point + c.normal * r;
            normal = c.normal;
            hold = self.hold_on(&c, p, supports);
            contact = Some(c);
        }
        let parent = self.nodes[tip].parent as usize;
        let from = self.nodes[parent].position;
        let node = &mut self.nodes[tip];
        let run = p - from;
        node.length = run.length();
        if node.length > 1e-6 {
            node.direction = run / node.length;
        }
        node.position = p;
        node.hold = hold;
        node.normal = normal;
        node.birth_sag = node.sag;

        let heading = self.heading(s, species, supports, p, contact);
        // A shoot that has grown this far without holding anything gives up.
        if hold == Hold::Free && self.free_run(tip) > species.max_free {
            self.shoots[s].active = false;
            return;
        }
        let laid = self.nodes[tip];
        self.nodes.push(VineNode {
            parent: tip as u32,
            direction: heading,
            length: 0.0,
            diameter: species.shoot_diameter,
            sapwood: species.shoot_diameter,
            heartwood: 0.0,
            born: self.age,
            hold: Hold::Free,
            normal: Vec3::new(0.0, 0.0, 0.0),
            birth_sag: laid.sag,
            lateral: false,
            dark: 0.0,
            bare: 0.0,
            ..laid
        });
        self.shoots[s].node = (self.nodes.len() - 1) as u32;
        self.shoots[s].heading = heading;
        if self.nodes.len() >= species.max_nodes {
            for shoot in &mut self.shoots {
                shoot.active = false;
            }
        }
    }

    /// Where a shoot held at `start` and heading along `heading` gets to
    /// after `length` of growth over the surfaces it climbs: small steps,
    /// each brought back onto the surface and its heading into the surface's
    /// tangent plane -- a geodesic, near enough, round a trunk, over the
    /// edge of a wall. Off the end of what it holds it carries straight on.
    fn creep(
        &self,
        start: Vec3,
        heading: Vec3,
        length: f32,
        species: &VineType,
        supports: &Supports,
    ) -> Vec3 {
        const STEPS: usize = 8;
        let r = species.stem_radius();
        let step = length / STEPS as f32;
        let (mut p, mut h) = (start, heading);
        for k in 0..STEPS {
            let q = p + h * step;
            match supports.nearest(q, 2.0 * step + r, |c| species.holds(c)) {
                Some(c) => {
                    p = c.point + c.normal * r;
                    let along = h - c.normal * h.dot(c.normal);
                    if along.length() > 1e-6 {
                        h = along.normalize();
                    }
                }
                None => return q + h * (step * (STEPS - k - 1) as f32),
            }
        }
        p
    }

    /// The hold a node at `p` takes on what `contact` touches.
    fn hold_on(&self, contact: &Contact, p: Vec3, supports: &Supports) -> Hold {
        let Support::Bark { segment } = contact.kind else {
            return Hold::Fixed;
        };
        let s = supports.bark.segments[segment as usize];
        let Some((a, b, _, _)) = supports.segment(&s) else {
            return Hold::Free;
        };
        let plant = &supports.plants[s.plant as usize];
        let ab = b - a;
        let t = ((p - a).dot(ab) / ab.dot(ab).max(1e-12)).clamp(0.0, 1.0);
        let (f1, f2) = basis(ab.normalize());
        let v = p - (a + ab * t);
        Hold::Bark(BarkHold {
            plant: plant.id,
            module: plant.modules[s.module as usize].id,
            node: s.node,
            t,
            angle: v.dot(f2).atan2(v.dot(f1)),
            plant_index: s.plant,
            module_index: s.module,
        })
    }

    /// Length of stem from node `i` back to the nearest held node.
    fn free_run(&self, mut i: usize) -> f32 {
        let mut run = 0.0;
        while self.nodes[i].hold == Hold::Free && self.nodes[i].parent != NONE {
            run += self.nodes[i].length;
            i = self.nodes[i].parent as usize;
        }
        run
    }

    /// The way shoot `s` grows on from `p`: Hädrich et al.'s weighted sum of
    /// its own heading, up, a support it senses, and circumnutation -- in the
    /// surface's tangent plane where it holds one, twisting round a stem it
    /// twines -- slid along anything it would grow into.
    fn heading(
        &mut self,
        s: usize,
        species: &VineType,
        supports: &Supports,
        p: Vec3,
        contact: Option<Contact>,
    ) -> Vec3 {
        let up = Vec3::new(0.0, 1.0, 0.0);
        let previous = self.shoots[s].heading;
        let mut heading = previous * species.persistence
            + up * species.gravitropism
            + random_unit(&mut self.rng) * species.wander;
        // Not yet climbing -- free, or held on level ground or the top of a
        // wall -- it grows towards the nearest thing it could climb.
        let climbing = contact.is_some_and(|c| c.normal.y < 0.7);
        if !climbing
            && let Some(target) = supports.nearest(p, species.sensing, |c| {
                c.kind != Support::Ground && c.normal.y < 0.7 && species.holds(c)
            })
        {
            let toward = (target.point - p).normalize();
            if target.distance < species.internode {
                // Within reach: straight for it.
                heading = toward;
            } else {
                let pull = (1.0 - target.distance / species.sensing).max(0.0);
                heading += toward * (species.attraction * pull);
            }
        }
        if let Some(c) = contact {
            if species.twist > 0.0 && matches!(c.kind, Support::Bark { .. }) {
                let axis = if c.axis.y < 0.0 { -c.axis } else { c.axis };
                heading += axis.cross(c.normal).normalize() * species.twist;
            }
            heading -= c.normal * heading.dot(c.normal);
        }
        // Slide along whatever the next internode would run into.
        let r = species.stem_radius();
        let ahead = p + heading.normalize() * species.internode;
        if let Some(c) = supports.nearest(ahead, r + 0.01, |_| true) {
            heading -= c.normal * heading.dot(c.normal).min(0.0);
        }
        if heading.length() < 1e-4 {
            previous
        } else {
            heading.normalize()
        }
    }

    /// Holds follow the branches they are on, what hangs settles under its
    /// weight, and every free node takes its place.
    fn settle(&mut self, species: &VineType, supports: &Supports) {
        let lift = species.stem_radius();
        for i in 1..self.nodes.len() {
            if let Hold::Bark(hold) = self.nodes[i].hold {
                match supports.bark_point(&hold, lift) {
                    Some((p, n)) => {
                        self.nodes[i].position = p;
                        self.nodes[i].normal = n;
                    }
                    None => self.let_go(i),
                }
            }
        }
        // Positions and sag in turn: the load's lever arms are the drooped
        // shape's, so a couple of passes settle it.
        for _ in 0..3 {
            self.sag(species);
            for i in 1..self.nodes.len() {
                self.place(i, species, supports);
            }
        }
    }

    /// A node whose branch is gone hangs from its parent.
    fn let_go(&mut self, i: usize) {
        let from = self.nodes[self.nodes[i].parent as usize].position;
        let node = &mut self.nodes[i];
        let run = node.position - from;
        node.length = run.length();
        if node.length > 1e-6 {
            node.direction = run / node.length;
        }
        node.hold = Hold::Free;
        node.normal = Vec3::new(0.0, 0.0, 0.0);
        node.birth_sag = node.sag;
    }

    /// The slope every hanging internode's load bends into it, by beam
    /// theory as the trees' limbs: `d(theta)/ds = M / (E I)` with `I = pi d^4
    /// / 64`, integrated over the internode from the moment at its base --
    /// of everything hanging from it -- to the moment at its end, and summed
    /// from the hold out. Eased half-way to the new value each pass, so it
    /// settles rather than swings.
    fn sag(&mut self, species: &VineType) {
        let n = self.nodes.len();
        // A node hangs if nothing at or beyond it is held.
        let mut anchored = vec![false; n];
        for i in (0..n).rev() {
            anchored[i] |= self.nodes[i].hold != Hold::Free;
            let parent = self.nodes[i].parent;
            if parent != NONE && anchored[i] {
                anchored[parent as usize] = true;
            }
        }
        // What hangs from each node's end (its children's), then with its
        // own internode: mass and first moment.
        let mut beyond = vec![(0.0f32, Vec3::new(0.0, 0.0, 0.0)); n];
        let mut mass = vec![0.0f32; n];
        let mut moment = vec![Vec3::new(0.0, 0.0, 0.0); n];
        for i in (1..n).rev() {
            if anchored[i] {
                continue;
            }
            let node = &self.nodes[i];
            let parent = node.parent as usize;
            let r = node.diameter * 0.5;
            let leaves = species.leaf_mass * self.leafiness(node, species);
            let m = (species.density * std::f32::consts::PI * r * r + leaves) * node.length;
            let middle = (node.position + self.nodes[parent].position) * 0.5;
            beyond[i] = (mass[i], moment[i]);
            mass[i] += m;
            moment[i] += middle * m;
            if !anchored[parent] {
                mass[parent] += mass[i];
                let carried = moment[i];
                moment[parent] += carried;
            }
        }
        // The bending moment about `at` of a load of `mass` centred where
        // `first / mass` is.
        let bending = |mass: f32, first: Vec3, at: Vec3| -> f32 {
            if mass <= 0.0 {
                return 0.0;
            }
            let lever = first / mass - at;
            mass * G * Vec3::new(lever.x, 0.0, lever.z).length()
        };
        for i in 1..n {
            let parent = self.nodes[i].parent as usize;
            let target = if anchored[i] || mass[i] <= 0.0 {
                0.0
            } else {
                let at_base = bending(mass[i], moment[i], self.nodes[parent].position);
                let at_end = bending(beyond[i].0, beyond[i].1, self.nodes[i].position);
                let d = self.nodes[i].diameter.max(1e-4);
                let inertia = std::f32::consts::PI * d.powi(4) / 64.0;
                let slope = ((at_base + at_end) * 0.5 * self.nodes[i].length
                    / (species.stiffness * inertia))
                    .min(0.6);
                let above = if anchored[parent] {
                    0.0
                } else {
                    self.nodes[parent].sag
                };
                above + slope
            };
            let node = &mut self.nodes[i];
            node.sag += (target - node.sag) * 0.5;
        }
    }

    /// What happens once a step of `dt` years, after the environment has set
    /// each node's light: shoots are given their rates, those in the dark
    /// stop, new shoots break where it is lit, and stems thicken.
    pub fn regulate(&mut self, species: &VineType, dt: f32) {
        if !self.alive || self.preserved {
            return;
        }
        if self.age > species.max_age {
            self.end();
            return;
        }
        // Leaves left in the dark a year begin to fall.
        for node in &mut self.nodes {
            if node.exposure < species.light_compensation {
                node.dark += dt;
            } else {
                node.dark = 0.0;
            }
            if node.dark >= 1.0 && node.bare == 0.0 {
                node.bare = 1e-3;
            }
        }
        // Rates: light as a tree's shoots, and the hydraulic ceiling.
        for shoot in &mut self.shoots {
            if !shoot.active {
                continue;
            }
            let tip = &self.nodes[shoot.node as usize];
            if tip.exposure < species.light_compensation {
                shoot.active = false;
                continue;
            }
            let height = (tip.position.y - self.position.y).max(0.0);
            shoot.rate = species.extension
                * light_response(tip.light)
                * hydraulic(height, species.max_height);
        }

        // New shoots, from the nodes a little behind growing tips, as many as
        // a metre of growth puts out in this light.
        let active = self.shoots.iter().filter(|s| s.active).count() as u32;
        let mut room = species.max_shoots.saturating_sub(active);
        let growing: Vec<Shoot> = self.shoots.iter().filter(|s| s.active).copied().collect();
        for shoot in growing {
            let expected = species.branching * shoot.rate * dt;
            for _ in 0..self.rng.stochastic_round(expected) {
                if room == 0 || self.nodes.len() >= species.max_nodes {
                    break;
                }
                let mut at = shoot.node as usize;
                for _ in 0..2 + (self.rng.next_f32() * 5.0) as usize {
                    let parent = self.nodes[at].parent;
                    if parent == NONE || parent == 0 {
                        break;
                    }
                    at = parent as usize;
                }
                self.sprout(at, species);
                room -= 1;
            }
        }
        // Nothing growing: a new shoot from the best-lit stem.
        if self.shoots.iter().all(|s| !s.active) && self.nodes.len() < species.max_nodes {
            let best = (1..self.nodes.len())
                .filter(|&i| self.nodes[i].hold != Hold::Free)
                .max_by(|&a, &b| self.nodes[a].exposure.total_cmp(&self.nodes[b].exposure));
            if let Some(best) = best
                && self.nodes[best].exposure > 2.0 * species.light_compensation
                && self.rng.next_f32() < 0.5 * dt
            {
                self.sprout(best, species);
            }
        }
        self.shoots.retain(|s| s.active);

        // Stems, basipetally: the pipe model over the leaves, and rings.
        let n = self.nodes.len();
        let mut sum = vec![0.0f32; n];
        for i in (0..n).rev() {
            let leafy = self.leafiness(&self.nodes[i], species);
            let own = species.shoot_diameter * species.shoot_diameter * leafy;
            let sapwood = (sum[i] + own).sqrt().max(species.shoot_diameter);
            let node = &mut self.nodes[i];
            node.sapwood = sapwood;
            node.heartwood += sapwood * sapwood * species.heartwood_rate * dt;
            node.diameter = node
                .diameter
                .max((sapwood * sapwood + node.heartwood).sqrt());
            let parent = node.parent;
            if parent != NONE {
                sum[parent as usize] += sapwood * sapwood;
            }
        }
        if self.shoots.is_empty() && self.nodes.iter().all(|n| self.leafiness(n, species) < 0.05) {
            self.end();
        }
    }

    /// Where the vine would die: it dies, or, immortal, is preserved as it is.
    fn end(&mut self) {
        if self.immortal {
            self.preserved = true;
            for shoot in &mut self.shoots {
                shoot.active = false;
            }
        } else {
            self.alive = false;
        }
    }

    /// A new shoot from node `at`: across the surface it holds, or out and
    /// up from a free stem.
    fn sprout(&mut self, at: usize, species: &VineType) {
        let node = self.nodes[at];
        let mut heading = random_unit(&mut self.rng) + Vec3::new(0.0, 0.5, 0.0);
        if node.hold != Hold::Free {
            heading -= node.normal * heading.dot(node.normal);
        }
        let heading = if heading.length() < 1e-4 {
            node.direction
        } else {
            heading.normalize()
        };
        self.nodes.push(VineNode {
            parent: at as u32,
            direction: heading,
            length: 0.0,
            diameter: species.shoot_diameter,
            sapwood: species.shoot_diameter,
            heartwood: 0.0,
            born: self.age,
            hold: Hold::Free,
            normal: Vec3::new(0.0, 0.0, 0.0),
            birth_sag: node.sag,
            lateral: true,
            dark: 0.0,
            bare: 0.0,
            ..node
        });
        self.shoots.push(Shoot {
            node: (self.nodes.len() - 1) as u32,
            heading,
            rate: species.extension * light_response(node.light) * 0.5,
            active: true,
        });
    }

    /// Point bark holds at where their plants and modules are now in
    /// `plants`; a hold whose branch is gone lets go.
    pub fn refresh_holds(
        &mut self,
        plants: &HashMap<u64, u32>,
        modules: &mut dyn FnMut(u32, u32) -> Option<u32>,
    ) {
        for i in 0..self.nodes.len() {
            let Hold::Bark(mut hold) = self.nodes[i].hold else {
                continue;
            };
            let found = plants
                .get(&hold.plant)
                .and_then(|&p| modules(p, hold.module).map(|m| (p, m)));
            match found {
                Some((p, m)) => {
                    hold.plant_index = p;
                    hold.module_index = m;
                    self.nodes[i].hold = Hold::Bark(hold);
                }
                None => self.let_go(i),
            }
        }
    }

    /// Height of the highest node above the root.
    pub fn height(&self) -> f32 {
        self.nodes
            .iter()
            .map(|n| n.position.y - self.position.y)
            .fold(0.0, f32::max)
    }

    /// Length of stem laid, all told.
    pub fn stem_length(&self) -> f32 {
        self.nodes.iter().map(|n| n.length).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn leaves_ride_out_a_flickering_light_and_fall_after_a_year_in_the_dark() {
        let species = VineType::ivy();
        let terrain = Terrain::flat(40.0, 40.0);
        let prototypes = PrototypeLibrary::halle();
        let bark = BarkIndex::default();
        let supports = Supports {
            terrain: &terrain,
            obstacles: None,
            plants: &[],
            prototypes: &prototypes,
            bark: &bark,
        };
        let mut vine = Vine::new(1, 0, &species, Vec3::new(0.0, 10.0, 0.0), Rng::new(1));
        vine.shoots.clear();
        // Its leaves open.
        vine.develop(&species, &supports, 0.5);
        let c = species.light_compensation;
        let step = |vine: &mut Vine, exposure: f32| {
            for node in &mut vine.nodes {
                node.exposure = exposure;
                node.light = exposure;
            }
            vine.regulate(&species, 0.5);
            vine.develop(&species, &supports, 0.5);
            vine.leafiness(&vine.nodes[1], &species)
        };
        // A step's light either side of the compensation point: the leaves
        // stay, every step.
        for k in 0..20 {
            let leafy = step(&mut vine, if k % 2 == 0 { 0.8 * c } else { 1.2 * c });
            if k > 0 {
                assert!(leafy > 0.99, "step {k}: leaves at {leafy}");
            }
        }
        // A year and a half in the dark: they fall, and lit again, stay bare.
        for _ in 0..3 {
            step(&mut vine, 0.5 * c);
        }
        for _ in 0..4 {
            assert_eq!(step(&mut vine, 1.0), 0.0, "old wood leafed again");
        }
    }

    #[test]
    fn a_shoot_held_out_level_bends_as_a_cantilever_under_its_weight() {
        let species = VineType::ivy();
        let terrain = Terrain::flat(40.0, 40.0);
        let prototypes = PrototypeLibrary::halle();
        let bark = BarkIndex::default();
        let supports = Supports {
            terrain: &terrain,
            obstacles: None,
            plants: &[],
            prototypes: &prototypes,
            bark: &bark,
        };
        // A straight leafy shoot held out level from a hold, ten metres up,
        // short enough to bend only a little.
        let (segments, length) = (40, 0.25f32);
        let mut vine = Vine::new(1, 0, &species, Vec3::new(0.0, 10.0, 0.0), Rng::new(1));
        vine.age = 10.0;
        vine.shoots.clear();
        vine.nodes.truncate(1);
        let l = length / segments as f32;
        for i in 0..segments {
            let root = vine.nodes[0];
            vine.nodes.push(VineNode {
                position: Vec3::new((i + 1) as f32 * l, 10.0, 0.0),
                parent: i as u32,
                direction: Vec3::new(1.0, 0.0, 0.0),
                length: l,
                hold: Hold::Free,
                normal: Vec3::new(0.0, 0.0, 0.0),
                ..root
            });
        }
        for _ in 0..40 {
            vine.settle(&species, &supports);
        }
        // A cantilever under a uniform load w: its tip turns w L^3 / (6 E I)
        // and drops w L^4 / (8 E I).
        let r = species.shoot_diameter * 0.5;
        let w = (species.density * PI * r * r + species.leaf_mass) * G;
        let ei = species.stiffness * PI * species.shoot_diameter.powi(4) / 64.0;
        let tip = vine.nodes[segments].position;
        let before = vine.nodes[segments - 1].position;
        let slope = (before.y - tip.y).atan2(tip.x - before.x);
        let drop = 10.0 - tip.y;
        let (slope_theory, drop_theory) = (
            w * length.powi(3) / (6.0 * ei),
            w * length.powi(4) / (8.0 * ei),
        );
        assert!(
            (slope / slope_theory - 1.0).abs() < 0.05,
            "tip slope {slope} against {slope_theory}"
        );
        assert!(
            (drop / drop_theory - 1.0).abs() < 0.05,
            "tip drop {drop} against {drop_theory}"
        );
    }
}

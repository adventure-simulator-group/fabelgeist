//! A plant: an ordered tree of branch modules (Sec. 5.2) developing over
//! time (Sec. 5.3).
//!
//! Modules are stored parents-first, so a forward walk is acropetal (base to
//! tip) and a backward walk basipetal (tip to base). Shedding compacts the
//! array and keeps that order.

use fabelgeist_math::{Vec3, Vec4};

use crate::prototype::{A_MATURE, MAX_TERMINALS, NONE, Prototype, PrototypeLibrary};
use crate::rng::Rng;
use crate::species::PlantType;

/// One instance of a prototype, attached to a plant.
#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    /// Stable for the module's life, unlike its index, which shedding
    /// shifts: what anything drawn for it is keyed on.
    pub id: u32,
    pub prototype: u16,
    /// Parent module, or [`NONE`] for the root module.
    pub parent: u32,
    /// Terminal of the parent this module hangs from.
    pub parent_slot: u8,
    /// Branch order: zero along the trunk, one more at every lateral.
    pub order: u8,
    /// Child module per terminal, [`NONE`] where empty.
    pub children: [u32; MAX_TERMINALS],
    /// World orientation; local `+Y` is the module's growth axis.
    pub orientation: Vec4,
    /// `a_u`: physiological age, `0..=A_MATURE`.
    pub age: f32,
    /// `da_u / dt`: how fast `age` advances, physiological age per year, as
    /// the last [`Plant::regulate`] set it from vigor, light and limits.
    pub rate: f32,
    /// `Q(u)`: light exposure, as the environment last measured it.
    pub light: f32,
    /// `light`, eased towards each new measurement over a few months: what
    /// the module is drawn in, so its shading does not jump at every step.
    pub lit: f32,
    /// `Q` summed over this module and everything above it.
    pub light_total: f32,
    /// `v(u)`: the vigor flowing through this module.
    pub vigor: f32,
    /// First of this module's nodes in [`Plant::nodes`]; [`NONE`] until the
    /// module is first laid out.
    pub node_offset: u32,
    /// Bounding sphere, centre and radius.
    pub sphere: [f32; 4],
    /// Created this step; its orientation has not been optimised yet.
    pub fresh: bool,
    /// The plant's age, in years, when this module was created.
    pub born: f32,
    /// The light actually reaching the module, `Q Q_G`, before shade
    /// tolerance: what [`PlantType::light_compensation`] is compared with.
    pub exposure: f32,
    /// How far the module's load bends it down, as a world rotation about
    /// its own base. Applied on top of its parent's.
    pub sag: Vec4,
    /// The rotation the module is drawn with: its own sag and however much
    /// its parent has bent since this module grew.
    pub bend: Vec4,
    /// The parent's `bend` when this module was created. A shoot grows in
    /// the direction it is oriented towards at the time; only sag that
    /// comes afterwards carries it along.
    pub birth_bend: Vec4,
    /// Bending stress at the module's base over the wood's strength; at one
    /// or more the branch breaks off.
    pub stress: f32,
}

/// A node of a module instance, in world space.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Node {
    pub position: Vec3,
    /// Diameter of the segment ending here: its sapwood around its
    /// heartwood.
    pub diameter: f32,
    /// `d_b`, the pipe-model diameter: the wood still conducting, as much
    /// as the shoots above it draw on. For a module's root node, what it
    /// demands of its parent's terminal.
    pub sapwood: f32,
    /// The core that no longer conducts, as a squared diameter (m^2): rings
    /// retired from the sapwood, and sapwood the shed branches drew on. It
    /// only ever grows.
    pub heartwood: f32,
    /// Diameter at the far end of the segment: the thickest continuation.
    pub end_diameter: f32,
    /// `a_b`: physiological age of the segment ending here.
    pub age: f32,
}

/// What one growth step produced.
#[derive(Clone, Debug, Default)]
pub struct GrowthOutcome {
    /// Seeds to disperse this step.
    pub seeds: u32,
    /// Modules shed.
    pub shed: u32,
    /// Modules created.
    pub spawned: u32,
}

#[derive(Clone, Debug)]
pub struct Plant {
    pub id: u64,
    pub species: u16,
    /// Where the root module stands.
    pub position: Vec3,
    /// Where the clump this stem sprouted from is rooted: its own position
    /// for a plant grown from seed ([`PlantType::sprouts_per_year`]).
    pub clump: Vec3,
    /// Age in years.
    pub age: f32,
    pub modules: Vec<Module>,
    pub nodes: Vec<Node>,
    /// Eq. 11's `o`, fixed where the plant stands.
    pub adaptation: f32,
    /// How fast this individual grows next to others of its species -- its
    /// genes, its seed, the patch of soil it landed on -- around one.
    pub potential: f32,
    /// How tall this individual can grow next to others of its species on
    /// the same site, around one.
    pub stature: f32,
    /// `v_root` this step.
    pub root_vigor: f32,
    pub flowering: bool,
    pub alive: bool,
    /// Is kept rather than dying: where it would die -- starved, too dark,
    /// or come to its age -- it is [`preserved`](Plant::preserved) as it is.
    pub immortal: bool,
    /// Came to where it would have died, and stays as it was then: it grows,
    /// sheds and seeds no more, but stands, shading and crowding what grows
    /// round it.
    pub preserved: bool,
    /// Leaves an unbranched plant's tip has put out, all told: fractional,
    /// growing continuously. A palm's fronds are the last few of them.
    pub crown_leaves: f32,
    next_module: u32,
    seed_clock: f32,
    rng: Rng,
}

/// Eq. 5's smooth step `S(x) = 3x^2 - 2x^3`.
pub(crate) fn smooth(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// How much a module may still lengthen and branch under `load`, the
/// highest stress over strength between it and the ground: fully up to half
/// the safe stress `1 / safety_factor`, not at all from the safe stress on.
fn headroom(load: f32, species: &PlantType) -> f32 {
    smooth(2.0 * (1.0 - load * species.safety_factor))
}

/// How much a shoot at hydraulic `distance` may still lengthen and branch:
/// fully up to 60% of the plant's ceiling ([`Plant::max_height`]), not at
/// all at it. Water is pulled up against gravity and the wood's resistance,
/// so the further a shoot is from the roots, the lower the turgor its cells
/// expand with; at the ceiling there is too little left to grow (Koch et al.
/// 2004).
pub(crate) fn hydraulic(distance: f32, ceiling: f32) -> f32 {
    smooth((1.0 - distance / ceiling.max(1e-3)) / 0.2)
}

/// How fast a shoot lengthens in light `q` (`Q_eff`) against in full sun.
/// Photosynthesis saturates well below full sun, so a lightly shaded shoot
/// grows nearly as fast as a lit one and a deeply shaded one slowly.
pub(crate) fn light_response(q: f32) -> f32 {
    const K: f32 = 3.0;
    (1.0 - (-K * q.max(0.0)).exp()) / (1.0 - (-K).exp())
}

/// Mature, untropised extent of a prototype for a species: the bounding
/// sphere's centre in module space and its radius. What a new module will
/// occupy, and so what its orientation is optimised for.
pub fn mature_extent(prototype: &Prototype, species: &PlantType) -> (Vec3, f32) {
    let mut positions = Vec::with_capacity(prototype.nodes.len());
    positions.push(Vec3::new(0.0, 0.0, 0.0));
    for node in &prototype.nodes[1..] {
        let length = segment_length(species, A_MATURE - node.emergence) * node.length;
        positions.push(positions[node.parent as usize] + node.direction * length);
    }
    sphere_of(&positions, species.tip_diameter)
}

fn sphere_of(positions: &[Vec3], thickness: f32) -> (Vec3, f32) {
    let mut center = Vec3::new(0.0, 0.0, 0.0);
    for p in positions {
        center += *p;
    }
    center = center / positions.len().max(1) as f32;
    let radius = positions
        .iter()
        .map(|p| (*p - center).length())
        .fold(0.0f32, f32::max);
    (center, (radius + thickness * 0.5).max(1e-3))
}

/// Eq. 9: `l_b = min(l_max, beta * a_b)`.
fn segment_length(species: &PlantType, age: f32) -> f32 {
    (species.length_scale * age.max(0.0)).min(species.segment_length_max)
}

impl Plant {
    /// A seedling: one freshly created root module standing at `position`.
    pub fn new(
        id: u64,
        species_index: u16,
        species: &PlantType,
        prototypes: &PrototypeLibrary,
        position: Vec3,
        adaptation: f32,
        mut rng: Rng,
    ) -> Self {
        let prototype = if species.unbranched {
            prototypes.unbranched
        } else {
            prototypes.select(species.apical_control, species.determinacy)
        };
        // Individuals differ: a lognormal spread around the species.
        let potential = (0.15 * rng.gaussian()).exp().clamp(0.6, 1.6);
        let stature = (0.08 * rng.gaussian()).exp().clamp(0.8, 1.25);
        let roll = Vec4::from_axis_angle(
            Vec3::new(0.0, 1.0, 0.0),
            rng.range(0.0, std::f32::consts::TAU),
        );
        let mut plant = Self {
            id,
            species: species_index,
            position,
            clump: position,
            age: 0.0,
            modules: vec![Module {
                id: 0,
                prototype,
                parent: NONE,
                parent_slot: 0,
                order: 0,
                children: [NONE; MAX_TERMINALS],
                orientation: roll,
                age: 0.0,
                rate: 0.0,
                light: 1.0,
                lit: 1.0,
                light_total: 1.0,
                vigor: species.vigor_max,
                node_offset: NONE,
                sphere: [position.x, position.y, position.z, 1e-3],
                fresh: false,
                born: 0.0,
                exposure: 1.0,
                sag: Vec4::quat_identity(),
                bend: Vec4::quat_identity(),
                birth_bend: Vec4::quat_identity(),
                stress: 0.0,
            }],
            nodes: Vec::new(),
            adaptation,
            potential,
            stature,
            root_vigor: 1.0,
            flowering: false,
            alive: true,
            immortal: false,
            preserved: false,
            crown_leaves: 0.0,
            next_module: 1,
            seed_clock: 0.0,
            rng,
        };
        plant.update_geometry(species, prototypes);
        plant
    }

    /// The tallest this plant can grow: its species' ceiling, lower on a
    /// site it is poorly adapted to -- a species' height on its best soils
    /// is half again what it reaches on its poorest -- and varied by its
    /// stature.
    pub fn max_height(&self, species: &PlantType) -> f32 {
        species.max_height * (0.6 + 0.4 * self.adaptation) * self.stature
    }

    /// `v_root_max`, scaled by adaptation and, past `p_max`, falling linearly
    /// to zero -- senescence.
    pub fn root_vigor_cap(&self, species: &PlantType) -> f32 {
        let senescence = if self.age > species.max_age {
            (1.0 - (self.age - species.max_age) / species.senescence_years.max(1e-3)).max(0.0)
        } else {
            1.0
        };
        species.root_vigor_max * self.adaptation * senescence
    }

    /// Apical control and determinacy, which change once the plant flowers.
    pub fn architecture(&self, species: &PlantType) -> (f32, f32) {
        if self.flowering {
            (species.apical_control_mature, species.determinacy_mature)
        } else {
            (species.apical_control, species.determinacy)
        }
    }

    /// One growth step of `dt` years, after the environment has set each
    /// module's `light`: [`Plant::regulate`], then [`Plant::develop`] over
    /// the step.
    pub fn grow(
        &mut self,
        species: &PlantType,
        prototypes: &PrototypeLibrary,
        dt: f32,
    ) -> GrowthOutcome {
        let outcome = self.regulate(species, prototypes, dt);
        self.develop(species, dt);
        outcome
    }

    /// What happens once a step of `dt` years, after the environment has set
    /// each module's `light`: vigor is shared out, starved, dark, broken and
    /// cleared branches are shed, buds break into new modules, and every
    /// module is given the rate it develops at until the next step. Nothing
    /// grows here; [`Plant::develop`] does, continuously.
    pub fn regulate(
        &mut self,
        species: &PlantType,
        prototypes: &PrototypeLibrary,
        dt: f32,
    ) -> GrowthOutcome {
        let mut outcome = GrowthOutcome::default();
        if !self.alive || self.preserved {
            return outcome;
        }
        // Its age come, it senesces -- or, kept, stays as it is.
        if self.immortal && self.age >= species.max_age {
            self.end();
            return outcome;
        }
        let (lambda, determinacy) = self.architecture(species);

        // Basipetal: accumulate light towards the root.
        for u in (0..self.modules.len()).rev() {
            let mut total = self.modules[u].light;
            for &c in &self.modules[u].children {
                if c != NONE {
                    total += self.modules[c as usize].light_total;
                }
            }
            self.modules[u].light_total = total;
        }

        // Vigor at the root, capped by v_root_max.
        let cap = self.root_vigor_cap(species);
        self.root_vigor = (species.vigor_scale * self.modules[0].light_total).min(cap);
        self.modules[0].vigor = self.root_vigor;
        if self.root_vigor < species.vigor_min {
            self.end();
            return outcome;
        }

        // Flowering: vigorous plants mature sooner, F_eff = F_age v_max / v_root.
        // The ratio is capped at two, or a plant that never reaches its
        // potential -- every shrub in a forest's shade -- would never flower.
        if !self.flowering {
            let ratio = (species.root_vigor_max / self.root_vigor.max(1e-3)).min(2.0);
            let flowering_age = species.flowering_age * ratio;
            if self.age >= flowering_age {
                self.flowering = true;
            }
        }

        let limits = self.growth_limits(species, prototypes);
        // Below this, the trunk keeps no branches ([`PlantType::clear_bole`]).
        let bole = species.clear_bole * self.height();

        // Acropetal: Eq. 2, generalised to a terminal per slot. An empty
        // terminal of a mature module is a sink too, weighted by its share
        // of the module's own light, and what it receives decides whether a
        // new module attaches there.
        let mut spawn: Vec<(u32, u8, f32)> = Vec::new();
        // By index: children's vigor is written as parents are read.
        #[allow(clippy::needless_range_loop)]
        for u in 0..self.modules.len() {
            let module = &self.modules[u];
            if module.age < A_MATURE {
                continue;
            }
            let prototype = prototypes.get(module.prototype);
            let slots = prototype.terminals.len();
            let own = module.light / slots as f32;
            let mut weights = [0.0f32; MAX_TERMINALS];
            let mut sum = 0.0;
            for (slot, weight) in weights.iter_mut().enumerate().take(slots) {
                let control = if slot == prototype.apical {
                    lambda
                } else {
                    1.0 - lambda
                };
                let child = module.children[slot];
                let q = if child != NONE {
                    self.modules[child as usize].light_total
                } else {
                    own
                };
                *weight = control * q;
                sum += *weight;
            }
            if sum <= 0.0 {
                continue;
            }
            let vigor = module.vigor;
            let children = module.children;
            for slot in 0..slots {
                let share = vigor * weights[slot] / sum;
                let child = children[slot];
                if child != NONE {
                    self.modules[child as usize].vigor = share;
                } else if share * limits[u] > species.vigor_min
                    && !self.on_bole(prototypes, u, slot, bole)
                {
                    // A bud breaks only where the branch can bear more,
                    // water still reaches it, and the trunk keeps branches.
                    spawn.push((u as u32, slot as u8, share));
                }
            }
        }

        // Shedding, with everything a shed module carries:
        //
        // * starved: vigor below v_min (the paper's rule);
        // * self-pruned: a mature branch whose subtree averages less light
        //   than the compensation point -- branch autonomy, not in the paper;
        // * broken: bent past the wood's strength by its own weight -- not
        //   in the paper either;
        // * cleared: a branch off the trunk that the rising crown has left
        //   below it (a species' clear bole) -- nor this.
        let count = self.modules.len();
        let mut exposure = vec![0.0f32; count];
        let mut carried = vec![0u32; count];
        for u in (0..count).rev() {
            exposure[u] += self.modules[u].exposure;
            carried[u] += 1;
            let parent = self.modules[u].parent;
            if parent != NONE {
                exposure[parent as usize] += exposure[u];
                carried[parent as usize] += carried[u];
            }
        }
        // A plant that as a whole cannot reach its compensation point dies.
        if self.modules[0].age >= A_MATURE
            && exposure[0] / (carried[0] as f32) < species.light_compensation
        {
            self.end();
            return outcome;
        }
        let mut keep = vec![true; count];
        for u in 1..count {
            let module = &self.modules[u];
            let dark = module.age >= A_MATURE
                && exposure[u] / (carried[u] as f32) < species.light_compensation;
            let parent = module.parent as usize;
            let cleared = bole > 0.0
                && module.order > 0
                && self.modules[parent].order == 0
                && self.attachment(prototypes, u).y - self.position.y < bole;
            if !keep[parent]
                || module.vigor < species.vigor_min
                || dark
                || module.stress >= 1.0
                || cleared
            {
                keep[u] = false;
            }
        }
        spawn.retain(|&(u, _, _)| keep[u as usize]);
        outcome.shed = keep.iter().filter(|k| !**k).count() as u32;
        if outcome.shed > 0 {
            self.compact(&keep, &mut spawn);
        }

        // Development rates, Eqs. 5 and 6, slowed on a limb that bears all it
        // safely can and near the plant's ceiling.
        let limits = self.growth_limits(species, prototypes);
        for (module, limit) in self.modules.iter_mut().zip(limits) {
            let v = module.vigor.min(species.vigor_max);
            module.rate = smooth((v - species.vigor_min) / (species.vigor_max - species.vigor_min))
                * species.growth_rate
                * self.potential
                * light_response(module.light)
                * species.lateral_extension.powi(module.order.min(1) as i32)
                * limit;
        }

        // New modules at the vigorous empty terminals. Vigorous parents pick
        // more determinate prototypes: D' = v(parent) D / v_max.
        for (parent, slot, share) in spawn {
            let parent_module = &self.modules[parent as usize];
            let d = parent_module.vigor.min(species.vigor_max) / species.vigor_max * determinacy;
            let prototype = if species.unbranched {
                prototypes.unbranched
            } else {
                prototypes.select(lambda, d)
            };
            let frame = prototypes
                .get(parent_module.prototype)
                .terminal_frame(slot as usize);
            let roll = Vec4::from_axis_angle(
                Vec3::new(0.0, 1.0, 0.0),
                self.rng.range(0.0, std::f32::consts::TAU),
            );
            // From the parent as it is drawn, bent: a bud sits on the branch
            // as it hangs now.
            let orientation = parent_module
                .bend
                .mul_quat(parent_module.orientation)
                .mul_quat(frame)
                .mul_quat(roll)
                .normalize();
            let birth_bend = parent_module.bend;
            let apical = prototypes.get(parent_module.prototype).apical == slot as usize;
            let order = parent_module.order.saturating_add(u8::from(!apical));
            let index = self.modules.len() as u32;
            self.modules[parent as usize].children[slot as usize] = index;
            let id = self.next_module;
            self.next_module += 1;
            self.modules.push(Module {
                id,
                prototype,
                parent,
                parent_slot: slot,
                order,
                children: [NONE; MAX_TERMINALS],
                orientation,
                age: 0.0,
                // It starts to develop at the next step, as its light is
                // first measured.
                rate: 0.0,
                light: 1.0,
                lit: 1.0,
                light_total: 1.0,
                vigor: share,
                node_offset: NONE,
                sphere: [0.0; 4],
                fresh: true,
                born: self.age,
                exposure: 1.0,
                sag: Vec4::quat_identity(),
                bend: Vec4::quat_identity(),
                birth_bend,
                stress: 0.0,
            });
            outcome.spawned += 1;
        }

        // Seeding, once flowering, a period a year.
        if self.flowering {
            self.seed_clock += dt;
            while self.seed_clock >= 1.0 {
                self.seed_clock -= 1.0;
                let expected = species.seeding_frequency * self.adaptation;
                outcome.seeds += self.rng.stochastic_round(expected);
            }
        }

        outcome
    }

    /// Where the plant would die: it dies, or, [`immortal`](Plant::immortal),
    /// is preserved as it is.
    fn end(&mut self) {
        if self.immortal {
            self.preserved = true;
            for module in &mut self.modules {
                module.rate = 0.0;
            }
        } else {
            self.alive = false;
        }
    }

    /// Grow for `dt` years -- any span, a step or a frame of one -- at the
    /// rates the last [`Plant::regulate`] set: modules lengthen (Eqs. 5-10
    /// through their age), stems add wood, and a palm's tip puts out fronds.
    /// Call [`Plant::update_geometry`] after to see it.
    pub fn develop(&mut self, species: &PlantType, dt: f32) {
        if !self.alive || self.preserved || dt <= 0.0 {
            return;
        }
        self.age += dt;
        // Light as drawn eases towards light as measured over a few months.
        let ease = 1.0 - (-dt / 0.3).exp();
        for module in &mut self.modules {
            module.age = (module.age + module.rate * dt).min(A_MATURE);
            module.lit += (module.light - module.lit) * ease;
        }
        // Secondary growth: each year the innermost sapwood stops conducting
        // and joins the heartwood, while the pipe model keeps the sapwood in
        // step with the shoots it feeds -- so every year adds a ring.
        let retired = species.heartwood_rate * dt;
        for node in &mut self.nodes {
            node.heartwood += node.sapwood * node.sapwood * retired;
        }
        // A palm puts out leaves one after another, more a year as it grows,
        // up to the couple of dozen its crown holds at once.
        if species.unbranched
            && let Some(years) = species.leaf_years
        {
            self.crown_leaves += self.crown_size() / years.max(1e-3) * dt;
        }
    }

    /// How many leaves an unbranched plant's crown holds at its height.
    pub fn crown_size(&self) -> f32 {
        (6.0 + 0.8 * self.height()).min(26.0)
    }

    /// Whether the empty terminal `slot` of module `u` is a lateral bud on
    /// the trunk below `bole` metres, where the trunk keeps no branches.
    fn on_bole(&self, prototypes: &PrototypeLibrary, u: usize, slot: usize, bole: f32) -> bool {
        let module = &self.modules[u];
        let prototype = prototypes.get(module.prototype);
        if bole <= 0.0 || module.order > 0 || slot == prototype.apical {
            return false;
        }
        let terminal = (module.node_offset + prototype.terminals[slot]) as usize;
        self.nodes
            .get(terminal)
            .is_some_and(|n| n.position.y - self.position.y < bole)
    }

    /// How much each module may still grow, from zero to one, as the last
    /// geometry update left it: limited by the load it bears -- the highest
    /// stress over strength between it and the ground -- and by its
    /// hydraulic distance from the roots: its height, and the friction of
    /// the branch between it and the trunk ([`PlantType::branch_resistance`]).
    fn growth_limits(&self, species: &PlantType, prototypes: &PrototypeLibrary) -> Vec<f32> {
        let ceiling = self.max_height(species);
        let mut load = vec![0.0f32; self.modules.len()];
        // Length of branch from the trunk to each module's tip, and when
        // the branch it is on grew out of the trunk.
        let mut branch = vec![0.0f32; self.modules.len()];
        let mut sprouted = vec![self.age; self.modules.len()];
        let mut limits = Vec::with_capacity(self.modules.len());
        for (u, module) in self.modules.iter().enumerate() {
            let (below, along) = if module.parent == NONE {
                (0.0, 0.0)
            } else {
                let p = module.parent as usize;
                sprouted[u] = if self.modules[p].order == 0 {
                    module.born
                } else {
                    sprouted[p]
                };
                (load[p], branch[p])
            };
            load[u] = module.stress.max(below);
            let offset = module.node_offset as usize;
            let count = prototypes.get(module.prototype).node_count();
            // A module spawned since the last geometry update has no nodes
            // yet; it stands where it attaches, below its parent's top.
            let nodes = self.nodes.get(offset..offset + count).unwrap_or_default();
            let top = nodes
                .iter()
                .map(|n| n.position.y - self.position.y)
                .fold(0.0, f32::max);
            if module.order > 0 {
                let root = nodes
                    .first()
                    .map_or(Vec3::new(0.0, 0.0, 0.0), |n| n.position);
                let span = nodes
                    .iter()
                    .map(|n| (n.position - root).length())
                    .fold(0.0, f32::max);
                branch[u] = along + span;
            }
            // At the ceiling, gravity and the wood's friction each take about
            // half of the water's pull; a branch adds only friction.
            let distance = top + 0.5 * species.branch_resistance * branch[u];
            let aging = (-(self.age - sprouted[u]) / species.branch_half_life).exp2();
            limits.push(headroom(load[u], species) * hydraulic(distance, ceiling) * aging);
        }
        limits
    }

    /// Drop the modules `keep` rejects -- which must carry nothing -- and
    /// empty the terminals they hung from.
    pub fn discard(&mut self, keep: &[bool]) {
        self.compact(keep, &mut []);
    }

    /// Drop the modules `keep` rejects, remapping every index.
    fn compact(&mut self, keep: &[bool], spawn: &mut [(u32, u8, f32)]) {
        let mut remap = vec![NONE; self.modules.len()];
        let mut next = 0u32;
        for (u, &k) in keep.iter().enumerate() {
            if k {
                remap[u] = next;
                next += 1;
            }
        }
        let modules = std::mem::take(&mut self.modules);
        self.modules = modules
            .into_iter()
            .zip(keep)
            .filter(|(_, k)| **k)
            .map(|(mut module, _)| {
                if module.parent != NONE {
                    module.parent = remap[module.parent as usize];
                }
                for child in &mut module.children {
                    if *child != NONE {
                        *child = remap[*child as usize];
                    }
                }
                module
            })
            .collect();
        for entry in spawn {
            entry.0 = remap[entry.0 as usize];
        }
    }

    /// Where a module attaches: the plant's base, or its parent's terminal.
    pub fn attachment(&self, prototypes: &PrototypeLibrary, module: usize) -> Vec3 {
        let m = &self.modules[module];
        if m.parent == NONE {
            return self.position;
        }
        let parent = &self.modules[m.parent as usize];
        let terminal = prototypes.get(parent.prototype).terminals[m.parent_slot as usize];
        self.nodes[(parent.node_offset + terminal) as usize].position
    }

    /// Recompute every node, diameter and bounding sphere from the modules'
    /// ages and orientations (Eqs. 7-10).
    pub fn update_geometry(&mut self, species: &PlantType, prototypes: &PrototypeLibrary) {
        // Kept as it was: not even its sag settles further.
        if self.preserved {
            return;
        }
        let up = Vec3::new(0.0, 1.0, 0.0);
        let g1 = species.tropism_decay.max(1e-3);
        let g2 = species.tropism_strength;
        // Each module's bend: its own sag, on top of however much its parent
        // has bent since the module grew. A shoot that grew out of an
        // already-sagging branch grew towards the light from where it was,
        // so the sag it was born into does not tip it over.
        for u in 0..self.modules.len() {
            let module = &self.modules[u];
            let inherited = if module.parent == NONE {
                Vec4::quat_identity()
            } else {
                self.modules[module.parent as usize]
                    .bend
                    .mul_quat(module.birth_bend.conjugate())
            };
            let bend = module.sag.mul_quat(inherited).normalize();
            self.modules[u].bend = bend;
        }
        let total: usize = self
            .modules
            .iter()
            .map(|m| prototypes.get(m.prototype).node_count())
            .sum();
        // The old nodes: each module's wood carries over to its new ones.
        let old = std::mem::replace(&mut self.nodes, Vec::with_capacity(total));

        // Positions, acropetally. By index: each module reads the nodes its
        // parent wrote this same pass.
        #[allow(clippy::needless_range_loop)]
        for u in 0..self.modules.len() {
            let root = self.attachment(prototypes, u);
            let offset = self.nodes.len();
            let before = self.modules[u].node_offset as usize;
            self.modules[u].node_offset = offset as u32;
            let module = &self.modules[u];
            let prototype = prototypes.get(module.prototype);
            let (orientation, age) = (module.bend.mul_quat(module.orientation), module.age);
            self.nodes.push(Node {
                position: root,
                ..Node::default()
            });
            for node in &prototype.nodes[1..] {
                // Eq. 7: a segment ages from the moment it emerges.
                let segment_age = (age - node.emergence).max(0.0);
                let length = segment_length(species, segment_age) * node.length;
                // Eq. 10: young segments bend most; the offset fades as
                // g1 / (a_b + g1).
                let offset_dir = up * (g2 * g1 / (segment_age + g1));
                let direction = (orientation.rotate_vec3(node.direction) + offset_dir).normalize();
                let base = self.nodes[offset + node.parent as usize].position;
                let mut position = base + direction * length;
                // A drooping branch rests on the ground rather than passing
                // through it.
                position.y = position.y.max(self.position.y);
                self.nodes.push(Node {
                    position,
                    age: segment_age,
                    ..Node::default()
                });
            }
            // Until the diameters are worked out below, `diameter` holds the
            // one the node had: wood already laid down.
            if let Some(laid) = old.get(before..before + prototype.node_count()) {
                for (node, laid) in self.nodes[offset..].iter_mut().zip(laid) {
                    node.diameter = laid.diameter;
                    node.heartwood = laid.heartwood;
                }
            }
        }

        // Diameters, basipetally: Eq. 8, the pipe model with n = 2, for the
        // sapwood. Wood is never taken away: sapwood that a shed branch drew
        // on stops conducting but stays, as heartwood.
        for u in (0..self.modules.len()).rev() {
            let module = &self.modules[u];
            let prototype = prototypes.get(module.prototype);
            let offset = module.node_offset as usize;
            for n in (0..prototype.node_count()).rev() {
                let mut sum = 0.0f32;
                let mut thickest = 0.0f32;
                let mut continuation = |node: &Node| {
                    sum += node.sapwood * node.sapwood;
                    thickest = thickest.max(node.diameter);
                };
                for &c in &prototype.children[n] {
                    continuation(&self.nodes[offset + c as usize]);
                }
                let slot = prototype.terminal_slot[n];
                if slot != NONE {
                    let child = module.children[slot as usize];
                    if child != NONE {
                        continuation(
                            &self.nodes[self.modules[child as usize].node_offset as usize],
                        );
                    }
                }
                let node = &mut self.nodes[offset + n];
                // A new shoot starts thin and reaches its tip's diameter over
                // its first stretch of growth, so that what it adds below it
                // comes on gradually too; and a grown segment is never thinner
                // than a tip, or a shoot budding from its end, starting thin,
                // would thin everything below it.
                let tip = if n > 0 && node.age > 0.0 {
                    species.tip_diameter * (node.age / 0.2).min(1.0)
                } else {
                    0.0
                };
                let (sapwood, end) = if sum.sqrt() > tip {
                    (sum.sqrt(), thickest)
                } else {
                    (tip, tip * 0.5)
                };
                let laid = node.diameter;
                node.sapwood = sapwood;
                node.heartwood = node.heartwood.max(laid * laid - sapwood * sapwood);
                node.diameter = (sapwood * sapwood + node.heartwood).sqrt();
                node.end_diameter = end;
            }
        }

        self.update_mechanics(species, prototypes);

        // Bounding spheres.
        let mut scratch = Vec::new();
        for module in &mut self.modules {
            let count = prototypes.get(module.prototype).node_count();
            let offset = module.node_offset as usize;
            scratch.clear();
            scratch.extend(
                self.nodes[offset..offset + count]
                    .iter()
                    .map(|n| n.position),
            );
            let thickness = self.nodes[offset..offset + count]
                .iter()
                .map(|n| n.diameter)
                .fold(0.0, f32::max);
            let (center, radius) = sphere_of(&scratch, thickness);
            module.sphere = [center.x, center.y, center.z, radius];
        }
    }

    /// Load each module with everything it carries -- wood and leaves -- and
    /// from the bending moment at its base work out how far it sags and how
    /// close it is to breaking. Beam theory for a circular section: stress
    /// `32 M / (pi d^3)`, and over a module of length `l` a change of slope
    /// of about `M l / (2 E I)` with `I = pi d^4 / 64` -- the moment falls
    /// from `M` at the base towards the tip. Each module bends only by its
    /// own share; the slopes add up along the branch, as a beam's do. The
    /// trunk keeps upright, as reaction wood holds it. The sag is eased
    /// towards its new value, so a branch settles rather than oscillates
    /// between steps.
    fn update_mechanics(&mut self, species: &PlantType, prototypes: &PrototypeLibrary) {
        const G: f32 = 9.81;
        /// The most one module bends in a step's equilibrium, radians.
        const MAX_SLOPE: f32 = 0.35;
        let count = self.modules.len();
        let mut mass = vec![0.0f32; count];
        let mut moment = vec![Vec3::new(0.0, 0.0, 0.0); count];
        let mut span = vec![0.0f32; count];
        let leaves = species.leaf_size > 0.0;
        for u in (0..count).rev() {
            let module = &self.modules[u];
            let prototype = prototypes.get(module.prototype);
            let offset = module.node_offset as usize;
            let root = self.nodes[offset].position;
            let leafy = leaves && self.has_leaves(module, species);
            for (n, node) in prototype.nodes.iter().enumerate().skip(1) {
                let a = self.nodes[offset + node.parent as usize].position;
                let b = self.nodes[offset + n];
                let r = b.diameter * 0.5;
                let m =
                    species.wood_density * std::f32::consts::PI * r * r * (b.position - a).length();
                mass[u] += m;
                moment[u] += (a + b.position) * (0.5 * m);
                span[u] = span[u].max((b.position - root).length());
                let slot = prototype.terminal_slot[n];
                let bare_tip = prototype.children[n].is_empty()
                    && (slot == NONE || module.children[slot as usize] == NONE);
                if leafy && bare_tip && b.age > 0.0 {
                    mass[u] += species.leaf_mass;
                    moment[u] += b.position * species.leaf_mass;
                }
            }
            for &c in &module.children {
                if c != NONE {
                    let (m, carried) = (mass[c as usize], moment[c as usize]);
                    mass[u] += m;
                    moment[u] += carried;
                }
            }
        }

        let up = Vec3::new(0.0, 1.0, 0.0);
        for u in 0..count {
            let base = self.nodes[self.modules[u].node_offset as usize];
            let d = base.diameter;
            if mass[u] <= 0.0 || d <= 0.0 {
                continue;
            }
            let lever = moment[u] / mass[u] - base.position;
            let horizontal = Vec3::new(lever.x, 0.0, lever.z);
            let arm = horizontal.length();
            let bending = mass[u] * G * arm;
            let inertia = std::f32::consts::PI * d.powi(4) / 64.0;
            let stress = bending * (d * 0.5) / inertia;
            let module = &mut self.modules[u];
            module.stress = stress / species.wood_strength;
            if module.order == 0 {
                continue;
            }
            let slope =
                (bending * span[u] / (2.0 * species.wood_stiffness * inertia)).min(MAX_SLOPE);
            let target = if arm > 1e-4 {
                Vec4::from_axis_angle(up.cross(horizontal), slope)
            } else {
                Vec4::quat_identity()
            };
            // Ease half-way towards the new bend.
            let q = module.sag;
            let t = if q.dot(target) < 0.0 {
                Vec4::new(-target.x, -target.y, -target.z, -target.w)
            } else {
                target
            };
            module.sag = Vec4::new(q.x + t.x, q.y + t.y, q.z + t.z, q.w + t.w).normalize();
        }
    }

    /// Height of the highest node above the base.
    pub fn height(&self) -> f32 {
        self.nodes
            .iter()
            .map(|n| n.position.y - self.position.y)
            .fold(0.0, f32::max)
    }

    /// Diameter at the base of the trunk.
    pub fn trunk_diameter(&self) -> f32 {
        self.nodes.first().map_or(0.0, |n| n.diameter)
    }

    /// Wood volume: the sum over segments of `pi r^2 l`.
    pub fn wood_volume(&self, prototypes: &PrototypeLibrary) -> f32 {
        let mut volume = 0.0;
        for module in &self.modules {
            let prototype = prototypes.get(module.prototype);
            let offset = module.node_offset as usize;
            for (n, node) in prototype.nodes.iter().enumerate().skip(1) {
                let a = self.nodes[offset + node.parent as usize].position;
                let b = &self.nodes[offset + n];
                let r = b.diameter * 0.5;
                volume += std::f32::consts::PI * r * r * (b.position - a).length();
            }
        }
        volume
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seedling() -> (PlantType, PrototypeLibrary, Plant) {
        let species = PlantType::default();
        let prototypes = PrototypeLibrary::halle();
        let plant = Plant::new(
            1,
            0,
            &species,
            &prototypes,
            Vec3::new(0.0, 0.0, 0.0),
            1.0,
            Rng::new(7),
        );
        (species, prototypes, plant)
    }

    #[test]
    fn a_lit_seedling_develops_and_branches() {
        let (species, prototypes, mut plant) = seedling();
        for _ in 0..40 {
            for m in &mut plant.modules {
                m.light = 1.0;
                m.fresh = false;
            }
            plant.grow(&species, &prototypes, 0.5);
            plant.update_geometry(&species, &prototypes);
        }
        assert!(plant.alive);
        assert!(plant.modules.len() > 10, "{} modules", plant.modules.len());
        assert!(plant.height() > 2.0, "height {}", plant.height());
        // Parents always precede children.
        for (u, m) in plant.modules.iter().enumerate() {
            assert!(m.parent == NONE || (m.parent as usize) < u);
        }
        // The pipe model thickens towards the base.
        let base = plant.trunk_diameter();
        assert!(base > species.tip_diameter * 2.0, "base {base}");
    }

    #[test]
    fn vigor_is_conserved_through_the_split() {
        let (species, prototypes, mut plant) = seedling();
        for _ in 0..30 {
            for m in &mut plant.modules {
                m.light = 0.8;
            }
            plant.grow(&species, &prototypes, 0.5);
            plant.update_geometry(&species, &prototypes);
        }
        // Children of a module never receive more than flows through it.
        for m in &plant.modules {
            let to_children: f32 = m
                .children
                .iter()
                .filter(|&&c| c != NONE)
                .map(|&c| plant.modules[c as usize].vigor)
                .sum();
            assert!(to_children <= m.vigor * 1.0001 + 1e-6);
        }
    }

    #[test]
    fn limbs_stop_reaching_out_before_they_break() {
        let (species, prototypes, mut plant) = seedling();
        let mut broken = 0;
        for _ in 0..160 {
            for m in &mut plant.modules {
                m.light = 1.0;
                m.fresh = false;
            }
            broken += plant.modules.iter().filter(|m| m.stress >= 1.0).count();
            plant.grow(&species, &prototypes, 0.5);
            plant.update_geometry(&species, &prototypes);
        }
        assert!(plant.alive);
        assert_eq!(broken, 0, "limbs grew until they snapped");
        let worst = plant.modules.iter().map(|m| m.stress).fold(0.0, f32::max);
        assert!(
            worst < 1.5 / species.safety_factor,
            "stress {worst} well past the safe {}",
            1.0 / species.safety_factor
        );
        // The trunk holds itself upright, and nothing lies along the ground.
        for m in plant.modules.iter().filter(|m| m.order == 0) {
            assert_eq!(m.sag, Vec4::quat_identity());
        }
        let grounded = plant.nodes[1..]
            .iter()
            .filter(|n| n.position.y < plant.position.y + 0.05)
            .count();
        assert!(grounded <= 2, "{grounded} nodes on the ground");
    }

    #[test]
    fn a_plant_levels_off_below_its_height() {
        let species = PlantType {
            max_height: 5.0,
            ..PlantType::default()
        };
        let prototypes = PrototypeLibrary::halle();
        let mut plant = Plant::new(
            1,
            0,
            &species,
            &prototypes,
            Vec3::new(0.0, 0.0, 0.0),
            1.0,
            Rng::new(7),
        );
        for _ in 0..200 {
            for m in &mut plant.modules {
                m.light = 1.0;
                m.fresh = false;
            }
            plant.grow(&species, &prototypes, 0.5);
            plant.update_geometry(&species, &prototypes);
        }
        assert!(plant.alive);
        let height = plant.height();
        assert!(
            (3.5..=plant.max_height(&species) + 0.5).contains(&height),
            "height {height}"
        );
    }

    #[test]
    fn the_trunk_thickens_every_year_and_never_thins() {
        // A low ceiling, so the crown stops growing early and the pipe model
        // alone would leave the trunk as it is.
        let species = PlantType {
            max_height: 5.0,
            ..PlantType::default()
        };
        let prototypes = PrototypeLibrary::halle();
        let mut plant = Plant::new(
            1,
            0,
            &species,
            &prototypes,
            Vec3::new(0.0, 0.0, 0.0),
            1.0,
            Rng::new(7),
        );
        let mut diameters = Vec::new();
        for _ in 0..240 {
            for m in &mut plant.modules {
                m.light = 1.0;
                m.fresh = false;
            }
            plant.grow(&species, &prototypes, 0.5);
            plant.update_geometry(&species, &prototypes);
            diameters.push(plant.trunk_diameter());
        }
        assert!(plant.alive);
        for pair in diameters.windows(2) {
            assert!(pair[1] >= pair[0], "the trunk thinned: {pair:?}");
        }
        let (middle, end) = (diameters[119], diameters[239]);
        assert!(end > middle * 1.1, "{middle} -> {end} over sixty years");
    }

    #[test]
    fn individuals_of_a_species_grow_apart() {
        let species = PlantType::default();
        let prototypes = PrototypeLibrary::halle();
        let heights: Vec<f32> = (0..6)
            .map(|seed| {
                let mut plant = Plant::new(
                    1,
                    0,
                    &species,
                    &prototypes,
                    Vec3::new(0.0, 0.0, 0.0),
                    1.0,
                    Rng::new(seed),
                );
                for _ in 0..60 {
                    for m in &mut plant.modules {
                        m.light = 1.0;
                        m.fresh = false;
                    }
                    plant.grow(&species, &prototypes, 0.5);
                    plant.update_geometry(&species, &prototypes);
                }
                plant.height()
            })
            .collect();
        let (low, high) = heights
            .iter()
            .fold((f32::MAX, 0.0f32), |(l, h), &x| (l.min(x), h.max(x)));
        assert!(high > low * 1.1, "heights {heights:?}");
    }

    #[test]
    fn shade_slows_growth() {
        let (species, prototypes, plant) = seedling();
        let grown = |light: f32| {
            let mut plant = plant.clone();
            for _ in 0..30 {
                for m in &mut plant.modules {
                    m.light = light;
                    m.fresh = false;
                }
                plant.grow(&species, &prototypes, 0.5);
                plant.update_geometry(&species, &prototypes);
            }
            plant.height()
        };
        let (sun, shade) = (grown(1.0), grown(0.3));
        assert!(shade < sun * 0.9, "sun {sun}, shade {shade}");
    }

    #[test]
    fn a_clear_bole_keeps_the_lower_trunk_bare() {
        let species = PlantType {
            clear_bole: 0.5,
            ..PlantType::default()
        };
        let prototypes = PrototypeLibrary::halle();
        let mut plant = Plant::new(
            1,
            0,
            &species,
            &prototypes,
            Vec3::new(0.0, 0.0, 0.0),
            1.0,
            Rng::new(7),
        );
        for _ in 0..120 {
            for m in &mut plant.modules {
                m.light = 1.0;
                m.fresh = false;
            }
            plant.grow(&species, &prototypes, 0.5);
            plant.update_geometry(&species, &prototypes);
        }
        assert!(plant.alive);
        // The crown rises a little between steps; allow for the last one.
        let bole = 0.45 * plant.height();
        let mut branches = 0;
        for (u, m) in plant.modules.iter().enumerate() {
            if m.order > 0 && plant.modules[m.parent as usize].order == 0 {
                branches += 1;
                let at = plant.attachment(&prototypes, u).y;
                assert!(at >= bole, "a branch at {at} m below the bole at {bole} m");
            }
        }
        assert!(branches > 0, "no crown at all");
    }

    #[test]
    fn a_palm_never_branches_and_its_stem_stays_as_thick() {
        let species = PlantType::palm();
        let prototypes = PrototypeLibrary::halle();
        let mut plant = Plant::new(
            1,
            0,
            &species,
            &prototypes,
            Vec3::new(0.0, 0.0, 0.0),
            1.0,
            Rng::new(7),
        );
        for _ in 0..80 {
            for m in &mut plant.modules {
                m.light = 1.0;
                m.fresh = false;
            }
            plant.grow(&species, &prototypes, 0.5);
            plant.update_geometry(&species, &prototypes);
        }
        assert!(plant.alive);
        assert!(plant.height() > 5.0, "height {}", plant.height());
        for m in &plant.modules {
            assert_eq!(m.order, 0);
            assert_eq!(m.prototype, prototypes.unbranched);
        }
        let d = plant.trunk_diameter();
        assert!((d - species.tip_diameter).abs() < 1e-4, "stem {d}");
    }

    #[test]
    fn darkness_kills() {
        let (species, prototypes, mut plant) = seedling();
        for _ in 0..3 {
            for m in &mut plant.modules {
                m.light = 0.01;
            }
            plant.grow(&species, &prototypes, 1.0);
        }
        assert!(!plant.alive);
    }

    /// Grow `plant` in `light` for `years`, half a year at a time.
    fn grown(
        mut plant: Plant,
        species: &PlantType,
        prototypes: &PrototypeLibrary,
        light: f32,
        years: f32,
    ) -> Plant {
        for _ in 0..(years / 0.5) as usize {
            for m in &mut plant.modules {
                m.light = light;
                m.fresh = false;
            }
            plant.grow(species, prototypes, 0.5);
            plant.update_geometry(species, prototypes);
        }
        plant
    }

    #[test]
    fn kept_from_dying_a_plant_stays_as_it_was() {
        // Starved in the dark: kept as the seedling it was, even once lit.
        let (species, prototypes, mut plant) = seedling();
        plant.immortal = true;
        let plant = grown(plant, &species, &prototypes, 0.01, 3.0);
        assert!(plant.alive && plant.preserved);
        let (height, modules) = (plant.height(), plant.modules.len());
        let plant = grown(plant, &species, &prototypes, 1.0, 10.0);
        assert_eq!(
            (plant.height(), plant.modules.len()),
            (height, modules),
            "it grew on"
        );

        // Come to its age: kept as it was then, its crown whole.
        let old = PlantType {
            max_age: 12.0,
            senescence_years: 3.0,
            ..PlantType::default()
        };
        let mut plant = Plant::new(
            1,
            0,
            &old,
            &prototypes,
            Vec3::new(0.0, 0.0, 0.0),
            1.0,
            Rng::new(3),
        );
        plant.immortal = true;
        let plant = grown(plant, &old, &prototypes, 1.0, 13.0);
        assert!(plant.alive && plant.preserved, "age {}", plant.age);
        let (age, height, modules) = (plant.age, plant.height(), plant.modules.len());
        assert!((12.0..12.6).contains(&age), "kept at {age}, past its age");
        let plant = grown(plant, &old, &prototypes, 1.0, 30.0);
        assert_eq!(
            (plant.age, plant.height(), plant.modules.len()),
            (age, height, modules)
        );
        assert!(modules > 5, "kept without its crown");
    }

    #[test]
    fn senescence_ends_a_plant() {
        let species = PlantType {
            max_age: 5.0,
            senescence_years: 3.0,
            ..PlantType::default()
        };
        let prototypes = PrototypeLibrary::halle();
        let mut plant = Plant::new(
            1,
            0,
            &species,
            &prototypes,
            Vec3::new(0.0, 0.0, 0.0),
            1.0,
            Rng::new(3),
        );
        let mut years = 0.0;
        while plant.alive && years < 30.0 {
            for m in &mut plant.modules {
                m.light = 1.0;
            }
            plant.grow(&species, &prototypes, 0.5);
            plant.update_geometry(&species, &prototypes);
            years += 0.5;
        }
        assert!(!plant.alive, "still alive after {years} years");
        assert!(years <= 9.0);
    }
}

//! Hydraulic limits, load, and physiological growth rates.
//! Native arithmetic follows the paper in world-space metres and simulation years.
use super::*;

impl Plant {
    /// Whether the empty terminal `slot` of module `u` is a lateral bud on
    /// the trunk below `bole` metres, where the trunk keeps no branches.
    pub(super) fn on_bole(
        &self,
        prototypes: &PrototypeLibrary,
        u: usize,
        slot: usize,
        bole: f32,
    ) -> bool {
        let module = &self.modules[u];
        let prototype = prototypes.get(module.prototype);
        if bole <= 0.0 || module.order > 0 || slot == prototype.apical {
            return false;
        }
        let Some(offset) = module.node_offset else {
            return false;
        };
        let terminal = offset.index() + prototype.terminals[slot] as usize;
        self.nodes
            .get(terminal)
            .is_some_and(|n| n.position.y - self.position.y < bole)
    }

    /// How much each module may still grow, from zero to one, as the last
    /// geometry update left it: limited by the load it bears -- the highest
    /// stress over strength between it and the ground -- and by its
    /// hydraulic distance from the roots: its height, and the friction of
    /// the branch between it and the trunk ([`PlantType::branch_resistance`]).
    pub(super) fn growth_limits(
        &self,
        species: &PlantType,
        prototypes: &PrototypeLibrary,
    ) -> Vec<f32> {
        let ceiling = self.max_height(species);
        let mut load = vec![0.0f32; self.modules.len()];
        // Length of branch from the trunk to each module's tip, and when
        // the branch it is on grew out of the trunk.
        let mut branch = vec![0.0f32; self.modules.len()];
        let mut sprouted = vec![self.age; self.modules.len()];
        let mut limits = Vec::with_capacity(self.modules.len());
        for (u, module) in self.modules.iter().enumerate() {
            let (below, along) = if let Some(parent) = module.parent {
                let p = parent.index();
                sprouted[u] = if self.modules[p].order == 0 {
                    module.born
                } else {
                    sprouted[p]
                };
                (load[p], branch[p])
            } else {
                (0.0, 0.0)
            };
            load[u] = module.stress.max(below);
            let count = prototypes.get(module.prototype).node_count();
            // A module spawned since the last geometry update has no nodes
            // yet; it stands where it attaches, below its parent's top.
            let nodes = module
                .node_offset
                .and_then(|offset| self.nodes.get(offset.index()..offset.index() + count))
                .unwrap_or_default();
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
            let aging =
                (-(self.age.years() - sprouted[u].years()) / species.branch_half_life).exp2();
            limits.push(headroom(load[u], species) * hydraulic(distance, ceiling) * aging);
        }
        limits
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
    pub(super) fn update_mechanics(&mut self, species: &PlantType, prototypes: &PrototypeLibrary) {
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
            let Some(offset) = module.node_offset else {
                continue;
            };
            let offset = offset.index();
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
                    && (slot == NONE || module.children[slot as usize].is_none());
                if leafy && bare_tip && b.age.stage() > 0.0 {
                    mass[u] += species.leaf_mass;
                    moment[u] += b.position * species.leaf_mass;
                }
            }
            for &c in &module.children {
                if let Some(c) = c {
                    let (m, carried) = (mass[c.index()], moment[c.index()]);
                    mass[u] += m;
                    moment[u] += carried;
                }
            }
        }

        let up = Vec3::new(0.0, 1.0, 0.0);
        for u in 0..count {
            let Some(offset) = self.modules[u].node_offset else {
                continue;
            };
            let base = self.nodes[offset.index()];
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

    pub(super) fn set_development_rates(
        &mut self,
        species: &PlantType,
        prototypes: &PrototypeLibrary,
    ) {
        // Development rates, Eqs. 5 and 6, slowed on a limb that bears all it
        // safely can and near the plant's ceiling.
        let limits = self.growth_limits(species, prototypes);
        for (module, limit) in self.modules.iter_mut().zip(limits) {
            let v = module.vigor.min(species.vigor_max);
            module.rate = AgeRate::from_stages_per_year(
                smooth((v - species.vigor_min) / (species.vigor_max - species.vigor_min))
                    * species.growth_rate
                    * self.potential
                    * light_response(module.light)
                    * species.lateral_extension.powi(module.order.min(1) as i32)
                    * limit,
            );
        }
    }
}

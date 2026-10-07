//! The ecosystem scale (Sec. 6): many plants on a terrain, in a climate,
//! competing for light and spreading by seed.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use fabelgeist_math::{Vec3, Vec4};
use fabelgeist_spatial_hash::HashGrid;
use rayon::prelude::*;
use web_time::Instant;

use crate::climate::Climate;
use crate::environment::{Environment, EnvironmentSettings, OrientRequest, ShadowGrid, SphereSet};
use crate::obstacles::Obstacles;
use crate::plant::{Plant, mature_extent};
use crate::prototype::{NONE, PrototypeLibrary};
use crate::rng::Rng;
use crate::species::PlantType;
use crate::terrain::Terrain;
use crate::vine::{BarkIndex, Supports, Vine, VineType};

#[derive(Clone, Debug, PartialEq)]
pub struct EcosystemSettings {
    /// Years per step.
    pub dt: f32,
    pub environment: EnvironmentSettings,
    /// Sec. 6.2. Off, modules see only their local light.
    pub global_shadowing: bool,
    /// Edge of a shadow cell, metres. Grown if the grid would exceed
    /// `max_shadow_cells`.
    pub shadow_cell: f32,
    pub max_shadow_cells: usize,
    /// How far above the highest ground the shadow grid reaches, metres.
    pub shadow_height: f32,
    pub shadow_strength: f32,
    pub shadow_decay: f32,
    /// Plants beyond this are not seeded.
    pub max_plants: usize,
    /// A seed landing closer than this to a standing plant's base does not
    /// take root.
    pub seed_spacing: f32,
    /// A new module whose best orientation still leaves more than this
    /// share of it inside other modules does not grow: the bud stays
    /// dormant until the space frees up (crown shyness; not in the paper,
    /// which only steers modules away from each other).
    pub bud_crowding_limit: f32,
    pub seed: u64,
}

impl Default for EcosystemSettings {
    fn default() -> Self {
        Self {
            dt: 0.5,
            environment: EnvironmentSettings::default(),
            global_shadowing: true,
            shadow_cell: 1.0,
            max_shadow_cells: 8 << 20,
            shadow_height: 48.0,
            shadow_strength: 0.35,
            shadow_decay: 0.97,
            max_plants: 500_000,
            seed_spacing: 1.0,
            bud_crowding_limit: 0.5,
            seed: 1,
        }
    }
}

/// What a step did and how long it took.
#[derive(Clone, Debug, Default)]
pub struct StepReport {
    pub time: f32,
    pub plants: usize,
    pub modules: usize,
    pub spawned: u32,
    pub shed: u32,
    pub germinated: u32,
    pub died: u32,
    /// Sum of pairwise sphere overlap over total sphere volume (Fig. 15a).
    pub overlap_ratio: f32,
    pub light: Duration,
    pub grow: Duration,
    pub orient: Duration,
    pub geometry: Duration,
    pub seeding: Duration,
    pub total: Duration,
}

/// Aggregate measurements of the ecosystem.
#[derive(Clone, Debug, Default)]
pub struct Census {
    pub plants: usize,
    pub modules: usize,
    pub nodes: usize,
    /// Plants per species.
    pub per_species: Vec<usize>,
    /// Wood volume, m^3 (Fig. 15b's biomass at unit density).
    pub biomass: f32,
    pub max_height: f32,
    pub vines: usize,
}

pub struct Ecosystem {
    pub settings: EcosystemSettings,
    pub climate: Climate,
    pub terrain: Terrain,
    pub species: Vec<PlantType>,
    pub prototypes: PrototypeLibrary,
    pub plants: Vec<Plant>,
    /// Buildings: space nothing grows into.
    pub obstacles: Option<Arc<Obstacles>>,
    pub vine_species: Vec<VineType>,
    pub vines: Vec<Vine>,
    /// Years simulated.
    pub time: f32,
    /// When the next step's regulation is due ([`Ecosystem::advance`]).
    next_tick: f32,
    environment: Environment,
    /// Mature module extents per species per prototype.
    extents: Vec<Vec<(Vec3, f32)>>,
    /// The branches near vines, as the last step left them.
    bark: BarkIndex,
    /// What the obstacles deposit in the shadow grid, and the grid it was
    /// worked out for.
    occluders: Option<(ShadowGrid, Arc<Vec<u32>>)>,
    rng: Rng,
    next_id: u64,
}

impl Ecosystem {
    pub fn new(terrain: Terrain, climate: Climate, species: Vec<PlantType>) -> Self {
        let settings = EcosystemSettings::default();
        let mut ecosystem = Self {
            rng: Rng::new(settings.seed),
            settings,
            climate,
            terrain,
            species,
            prototypes: PrototypeLibrary::halle(),
            plants: Vec::new(),
            obstacles: None,
            vine_species: Vec::new(),
            vines: Vec::new(),
            time: 0.0,
            next_tick: 0.0,
            environment: Environment::default(),
            extents: Vec::new(),
            bark: BarkIndex::default(),
            occluders: None,
            next_id: 0,
        };
        ecosystem.refresh_species();
        ecosystem
    }

    pub fn with_settings(mut self, settings: EcosystemSettings) -> Self {
        self.rng = Rng::new(settings.seed);
        self.settings = settings;
        self
    }

    /// Buildings in the scene: solid space no module may take, which shades
    /// what grows below and beside it. Block their footprints in the
    /// terrain's soil too, or seeds will root inside them.
    pub fn with_obstacles(mut self, structure: fabelgeist_buildings::Structure) -> Self {
        self.obstacles = Some(Obstacles::new(structure));
        self.occluders = None;
        self
    }

    /// Climbers that can be planted ([`Ecosystem::plant_vine`]).
    pub fn with_vines(mut self, species: Vec<VineType>) -> Self {
        self.vine_species = species;
        self
    }

    /// Run the environment passes on the GPU.
    #[cfg(feature = "gpu")]
    pub fn with_gpu(mut self, context: fabelgeist_gpu::globals::WgpuContext) -> Result<Self> {
        self.environment = Environment::Gpu(Box::new(
            crate::environment::gpu::GpuEnvironment::new(context)?,
        ));
        Ok(self)
    }

    pub fn is_gpu(&self) -> bool {
        self.environment.is_gpu()
    }

    /// Call after editing `species` or `prototypes`: recomputes the cached
    /// mature extents and every plant's adaptation.
    pub fn refresh_species(&mut self) {
        self.extents = self
            .species
            .iter()
            .map(|s| {
                self.prototypes
                    .prototypes
                    .iter()
                    .map(|p| mature_extent(p, s))
                    .collect()
            })
            .collect();
        for plant in &mut self.plants {
            plant.adaptation = self
                .climate
                .adaptation(&self.species[plant.species as usize], plant.position.y);
        }
    }

    /// Plant a seedling of `species` at `(x, z)`, on the ground. `None` if the
    /// soil there is blocked.
    pub fn plant(&mut self, species: usize, x: f32, z: f32) -> Option<usize> {
        if !self.terrain.is_open(x, z) || species >= self.species.len() {
            return None;
        }
        let y = self.terrain.height_at(x, z);
        let adaptation = self.climate.adaptation(&self.species[species], y);
        let rng = self.rng.fork();
        self.next_id += 1;
        self.plants.push(Plant::new(
            self.next_id,
            species as u16,
            &self.species[species],
            &self.prototypes,
            Vec3::new(x, y, z),
            adaptation,
            rng,
        ));
        Some(self.plants.len() - 1)
    }

    /// Plant a climber of `species` at `(x, z)`. `None` if the soil there is
    /// blocked.
    pub fn plant_vine(&mut self, species: usize, x: f32, z: f32) -> Option<usize> {
        if !self.terrain.is_open(x, z) || species >= self.vine_species.len() {
            return None;
        }
        let y = self.terrain.height_at(x, z);
        let rng = self.rng.fork();
        self.next_id += 1;
        self.vines.push(Vine::new(
            self.next_id,
            species as u16,
            &self.vine_species[species],
            Vec3::new(x, y, z),
            rng,
        ));
        self.rebuild_bark();
        Some(self.vines.len() - 1)
    }

    /// Everything a vine can touch now.
    pub fn supports(&self) -> Supports<'_> {
        Supports {
            terrain: &self.terrain,
            obstacles: self.obstacles.as_deref(),
            plants: &self.plants,
            prototypes: &self.prototypes,
            bark: &self.bark,
        }
    }

    /// Point the vines' holds on bark at where their plants and modules are
    /// now, letting go of branches that are gone, and index the bark near
    /// them again.
    fn refresh_vines(&mut self) {
        if self.vines.is_empty() {
            return;
        }
        let ids: HashMap<u64, u32> = self
            .plants
            .iter()
            .enumerate()
            .map(|(i, p)| (p.id, i as u32))
            .collect();
        let plants = &self.plants;
        let mut maps: HashMap<u32, HashMap<u32, u32>> = HashMap::new();
        let mut modules = |p: u32, id: u32| -> Option<u32> {
            maps.entry(p)
                .or_insert_with(|| {
                    plants[p as usize]
                        .modules
                        .iter()
                        .enumerate()
                        .map(|(i, m)| (m.id, i as u32))
                        .collect()
                })
                .get(&id)
                .copied()
        };
        for vine in &mut self.vines {
            vine.refresh_holds(&ids, &mut modules);
        }
        self.vines.retain(|v| v.alive);
        self.rebuild_bark();
    }

    /// Index the branches of every plant whose crown may reach a vine.
    fn rebuild_bark(&mut self) {
        let reach = self
            .vine_species
            .iter()
            .map(|s| s.sensing + s.internode)
            .fold(1.0f32, f32::max);
        let boxes: Vec<[[f32; 2]; 2]> = self
            .vines
            .iter()
            .map(|vine| {
                let mut lo = [f32::INFINITY; 2];
                let mut hi = [f32::NEG_INFINITY; 2];
                for node in &vine.nodes {
                    lo = [lo[0].min(node.position.x), lo[1].min(node.position.z)];
                    hi = [hi[0].max(node.position.x), hi[1].max(node.position.z)];
                }
                [
                    [lo[0] - reach, lo[1] - reach],
                    [hi[0] + reach, hi[1] + reach],
                ]
            })
            .collect();
        self.bark = BarkIndex::build(&self.plants, &self.prototypes, |plant| {
            // A crown reaches out about as far as the plant is tall.
            let r = plant.height() + 1.0;
            let [x, z] = [plant.position.x, plant.position.z];
            boxes
                .iter()
                .any(|[lo, hi]| x > lo[0] - r && x < hi[0] + r && z > lo[1] - r && z < hi[1] + r)
        });
    }

    /// Work out what the obstacles deposit in the shadow grid, if the grid
    /// is new.
    fn refresh_occluders(&mut self) {
        let Some(obstacles) = &self.obstacles else {
            return;
        };
        let grid = self.shadow_grid();
        if grid.occluders.is_none() {
            let cells = Arc::new(obstacles.occupancy(&grid));
            self.occluders = Some((grid, cells));
        }
    }

    /// Scatter `count` seedlings over open ground, each species drawn with
    /// probability proportional to its adaptation at the spot (Sec. 6.1).
    pub fn scatter(&mut self, count: usize) -> usize {
        let mut planted = 0;
        let mut attempts = 0;
        while planted < count && attempts < count * 20 {
            attempts += 1;
            let x = self.terrain.origin[0] + self.rng.next_f32() * self.terrain.size[0];
            let z = self.terrain.origin[1] + self.rng.next_f32() * self.terrain.size[1];
            if !self.terrain.is_open(x, z) {
                continue;
            }
            let y = self.terrain.height_at(x, z);
            let weights: Vec<f32> = self
                .species
                .iter()
                .map(|s| self.climate.adaptation(s, y))
                .collect();
            let total: f32 = weights.iter().sum();
            if total <= 1e-6 {
                continue;
            }
            let mut pick = self.rng.next_f32() * total;
            let mut species = 0;
            for (i, w) in weights.iter().enumerate() {
                species = i;
                if pick < *w {
                    break;
                }
                pick -= w;
            }
            if self.plant(species, x, z).is_some() {
                planted += 1;
            }
        }
        planted
    }

    /// Remove every plant standing within `radius` of `(x, z)`: a gap in the
    /// canopy (Fig. 10).
    pub fn clear(&mut self, x: f32, z: f32, radius: f32) -> usize {
        let before = self.plants.len();
        self.plants.retain(|p| {
            let dx = p.position.x - x;
            let dz = p.position.z - z;
            dx * dx + dz * dz > radius * radius
        });
        self.refresh_vines();
        before - self.plants.len()
    }

    /// The shadow grid spanning the terrain.
    pub fn shadow_grid(&self) -> ShadowGrid {
        let (low, high) = self.terrain.height_range();
        let extent = [
            self.terrain.size[0],
            high - low + self.settings.shadow_height,
            self.terrain.size[1],
        ];
        let mut cell = self.settings.shadow_cell.max(0.05);
        let dims = |cell: f32| extent.map(|e| ((e / cell).ceil() as u32).max(1));
        while dims(cell).iter().map(|&d| d as usize).product::<usize>()
            > self.settings.max_shadow_cells
        {
            cell *= 1.25;
        }
        let mut grid = ShadowGrid {
            origin: [self.terrain.origin[0], low - cell, self.terrain.origin[1]],
            cell,
            dims: dims(cell),
            strength: self.settings.shadow_strength,
            decay: self.settings.shadow_decay,
            occluders: None,
        };
        if let Some((made_for, cells)) = &self.occluders
            && *made_for == grid
        {
            grid.occluders = Some(cells.clone());
        }
        grid
    }

    /// Module offsets per plant into the flattened sphere set.
    fn offsets(&self) -> (Vec<usize>, usize) {
        let mut offsets = Vec::with_capacity(self.plants.len());
        let mut total = 0;
        for plant in &self.plants {
            offsets.push(total);
            total += plant.modules.len();
        }
        (offsets, total)
    }

    /// Every module's current sphere, then every vine node's leaves, and the
    /// solids. Modules still awaiting orientation stand in with their mature
    /// sphere, so that they are seen at the size they will grow to.
    fn sphere_set(&self, offsets: &[usize], total: usize) -> SphereSet {
        let mut set = SphereSet {
            spheres: vec![[0.0; 4]; total],
            parents: vec![NONE; total],
            obstacles: self.obstacles.clone(),
        };
        let mut sphere_slices = split_by(&mut set.spheres, offsets, total);
        let mut parent_slices = split_by(&mut set.parents, offsets, total);
        let prototypes = &self.prototypes;
        let extents = &self.extents;
        self.plants
            .par_iter()
            .zip(offsets.par_iter())
            .zip(
                sphere_slices
                    .par_iter_mut()
                    .zip(parent_slices.par_iter_mut()),
            )
            .for_each(|((plant, &offset), (spheres, parents))| {
                for (u, module) in plant.modules.iter().enumerate() {
                    spheres[u] = if module.fresh {
                        let (center, radius) =
                            extents[plant.species as usize][module.prototype as usize];
                        let c = plant.attachment(prototypes, u)
                            + module.orientation.rotate_vec3(center);
                        [c.x, c.y, c.z, radius]
                    } else {
                        module.sphere
                    };
                    parents[u] = if module.parent == NONE {
                        NONE
                    } else {
                        (offset + module.parent as usize) as u32
                    };
                }
            });
        for vine in &self.vines {
            let base = set.spheres.len() as u32;
            for (sphere, parent) in vine.spheres(&self.vine_species[vine.species as usize]) {
                set.spheres.push(sphere);
                set.parents
                    .push(if parent == NONE { NONE } else { base + parent });
            }
        }
        set
    }

    /// Advance the simulation by `settings.dt` years: a step's regulation,
    /// then its growth.
    pub async fn step(&mut self) -> Result<StepReport> {
        let mut report = self.tick().await?;
        let clock = Instant::now();
        self.develop(self.settings.dt);
        self.next_tick = self.time;
        report.geometry += clock.elapsed();
        report.total += clock.elapsed();
        report.time = self.time;
        Ok(report)
    }

    /// Advance the simulation by `years` -- as little as a frame's worth --
    /// continuously: plants grow smoothly, and every `settings.dt` years a
    /// step's regulation runs (light, vigor, modules budding and shed,
    /// orientation, death and seeding). Returns the report of the last step
    /// that ran, if one did.
    pub async fn advance(&mut self, years: f32) -> Result<Option<StepReport>> {
        let mut last = None;
        let mut left = years.max(0.0);
        // A step due at the very end of the span is the next call's.
        while left > 1e-6 {
            if self.time >= self.next_tick - 1e-5 {
                last = Some(self.tick().await?);
                self.next_tick = self.time + self.settings.dt;
            }
            let span = left.min(self.next_tick - self.time);
            self.develop(span);
            left -= span;
        }
        Ok(last)
    }

    /// Grow every plant for `years` at the rates its last step set, and lay
    /// it out anew.
    pub fn develop(&mut self, years: f32) {
        let species = &self.species;
        let prototypes = &self.prototypes;
        self.plants
            .par_iter_mut()
            .filter(|p| p.alive)
            .for_each(|plant| {
                let s = &species[plant.species as usize];
                plant.develop(s, years);
                plant.update_geometry(s, prototypes);
            });
        if !self.vines.is_empty() {
            let supports = Supports {
                terrain: &self.terrain,
                obstacles: self.obstacles.as_deref(),
                plants: &self.plants,
                prototypes: &self.prototypes,
                bark: &self.bark,
            };
            let vine_species = &self.vine_species;
            self.vines.par_iter_mut().for_each(|vine| {
                vine.develop(&vine_species[vine.species as usize], &supports, years)
            });
        }
        self.time += years;
    }

    /// A step's regulation, at `self.time`, over the `settings.dt` years to
    /// the next.
    async fn tick(&mut self) -> Result<StepReport> {
        let start = Instant::now();
        let dt = self.settings.dt;
        let mut report = StepReport::default();
        self.refresh_occluders();

        // 1. Light: local (Eq. 1) and global (Sec. 6.2).
        let clock = Instant::now();
        let (offsets, total) = self.offsets();
        let set = self.sphere_set(&offsets, total);
        let shadowing = self.settings.global_shadowing;
        let shadow = shadowing.then(|| self.shadow_grid());
        let light = self
            .environment
            .light(&set, shadow.as_ref(), &self.settings.environment)
            .await?;
        report.overlap_ratio =
            overlap_ratio(&set, &light, self.settings.environment.light_sensitivity);
        // The vines' nodes follow the plants' modules in the set.
        let mut at = total;
        for vine in &mut self.vines {
            let count = vine.nodes.len();
            let species = &self.vine_species[vine.species as usize];
            vine.set_light(&light[at..at + count], species, shadowing);
            at += count;
        }
        report.light = clock.elapsed();

        // 2. Vigor, shedding, development and new modules, plant by plant.
        let clock = Instant::now();
        let species = &self.species;
        let prototypes = &self.prototypes;
        let outcomes: Vec<_> = self
            .plants
            .par_iter_mut()
            .zip(offsets.par_iter())
            .map(|(plant, &offset)| {
                let s = &species[plant.species as usize];
                for (u, module) in plant.modules.iter_mut().enumerate() {
                    let [q, qg] = light[offset + u];
                    // The paper takes Q_eff = lerp(s_tol, 1, Q Q_G), which
                    // floors *all* light at s_tol -- a module buried among
                    // others then keeps s_tol however crowded it is, and
                    // crowns never stop filling in. Shade tolerance is a
                    // tolerance of shade, so it is applied to the shadow
                    // alone: Q_eff = Q lerp(s_tol, 1, Q_G). Crowding still
                    // starves a module; shade starves only the intolerant.
                    module.exposure = if shadowing { q * qg } else { q };
                    module.light = if shadowing {
                        q * (s.shade_tolerance + (1.0 - s.shade_tolerance) * qg)
                    } else {
                        q
                    };
                }
                plant.regulate(s, prototypes, dt)
            })
            .collect();
        for outcome in &outcomes {
            report.spawned += outcome.spawned;
            report.shed += outcome.shed;
        }
        report.grow = clock.elapsed();

        // 3. Orient the new modules (Sec. 5.2.3).
        let clock = Instant::now();
        if report.spawned > 0 {
            let (offsets, total) = self.offsets();
            let set = self.sphere_set(&offsets, total);
            let mut requests = Vec::with_capacity(report.spawned as usize);
            for (plant, &offset) in self.plants.iter().zip(&offsets) {
                let s = &self.species[plant.species as usize];
                for (u, module) in plant.modules.iter().enumerate() {
                    if !module.fresh {
                        continue;
                    }
                    let (center, radius) =
                        self.extents[plant.species as usize][module.prototype as usize];
                    let root = plant.attachment(&self.prototypes, u);
                    let q = module.orientation;
                    // Laterals aim at the species' set-point angle (Digby
                    // and Firn's gravitropic set-point, which the paper
                    // cites); the leader -- the trunk's own continuation --
                    // aims straight up.
                    let cos_target = if module.order == 0 {
                        1.0
                    } else {
                        s.tropism_angle.cos()
                    };
                    // Whether, turned any way, it could reach a solid.
                    let near = self
                        .obstacles
                        .as_ref()
                        .is_some_and(|o| o.distance(root) < center.length() + radius + 0.5);
                    requests.push(OrientRequest {
                        root: root.to_array(),
                        radius,
                        local_center: center.to_array(),
                        index: (offset + u) as u32,
                        rotation: [q.x, q.y, q.z, q.w],
                        axis: self.prototypes.get(module.prototype).axis.to_array(),
                        cos_target,
                        collision_weight: s.collision_weight,
                        tropism_weight: s.tropism_weight,
                        parent: set.parents[offset + u],
                        obstacles: u32::from(near),
                    });
                }
            }
            let oriented = self
                .environment
                .orient(&set, &requests, &self.settings.environment)
                .await?;
            let limit = self.settings.bud_crowding_limit;
            let mut next = oriented.into_iter();
            for plant in &mut self.plants {
                let mut keep = vec![true; plant.modules.len()];
                let mut dormant = false;
                for (u, module) in plant.modules.iter_mut().enumerate() {
                    if module.fresh {
                        let result = next.next().expect("one rotation per request");
                        module.orientation = result.rotation;
                        module.fresh = false;
                        if result.crowding > limit {
                            keep[u] = false;
                            dormant = true;
                        }
                    }
                }
                if dormant {
                    report.spawned -= keep.iter().filter(|k| !**k).count() as u32;
                    plant.discard(&keep);
                }
            }
        }
        report.orient = clock.elapsed();

        // 4. Geometry.
        let clock = Instant::now();
        let species = &self.species;
        let prototypes = &self.prototypes;
        self.plants
            .par_iter_mut()
            .filter(|p| p.alive)
            .for_each(|plant| plant.update_geometry(&species[plant.species as usize], prototypes));
        report.geometry = clock.elapsed();

        // 5. Death and seeding.
        let clock = Instant::now();
        let before = self.plants.len();
        let seeds: Vec<(u16, Vec3, f32, u32)> = self
            .plants
            .iter()
            .zip(&outcomes)
            .filter(|(p, o)| p.alive && o.seeds > 0)
            .map(|(p, o)| {
                (
                    p.species,
                    p.position,
                    self.species[p.species as usize].seeding_radius,
                    o.seeds,
                )
            })
            .collect();
        self.plants.retain(|p| p.alive && !p.modules.is_empty());
        report.died = (before - self.plants.len()) as u32;
        report.germinated = self.disperse(&seeds) + self.sprout(dt);
        report.seeding = clock.elapsed();

        // 6. Vines: rates, new shoots and wood, and their holds on whatever
        // plants are left.
        let vine_species = &self.vine_species;
        self.vines
            .par_iter_mut()
            .for_each(|vine| vine.regulate(&vine_species[vine.species as usize], dt));
        self.refresh_vines();

        report.time = self.time;
        report.plants = self.plants.len();
        report.modules = self.plants.iter().map(|p| p.modules.len()).sum();
        report.total = start.elapsed();
        Ok(report)
    }

    /// Place seeds around their parents (a Gaussian of the seeding radius),
    /// where the soil is open and no plant already stands.
    fn disperse(&mut self, seeds: &[(u16, Vec3, f32, u32)]) -> u32 {
        if seeds.is_empty() {
            return 0;
        }
        let spacing = self.settings.seed_spacing.max(1e-3);
        let mut bases: Vec<[f32; 3]> = self
            .plants
            .iter()
            .map(|p| [p.position.x, 0.0, p.position.z])
            .collect();
        let grid = HashGrid::new(bases.iter().copied(), spacing);
        let mut germinated = 0;
        for &(species, origin, radius, count) in seeds {
            for _ in 0..count {
                if self.plants.len() >= self.settings.max_plants {
                    return germinated;
                }
                let x = origin.x + self.rng.gaussian() * radius;
                let z = origin.z + self.rng.gaussian() * radius;
                let mut crowded = false;
                grid.for_each_near([x, 0.0, z], 1, |j| {
                    let b = bases[j as usize];
                    crowded |= (b[0] - x).powi(2) + (b[2] - z).powi(2) < spacing * spacing;
                });
                // Seedlings placed this step are checked by brute force; there
                // are few of them.
                let start = grid.len();
                crowded |= bases[start..]
                    .iter()
                    .any(|b| (b[0] - x).powi(2) + (b[2] - z).powi(2) < spacing * spacing);
                if crowded {
                    continue;
                }
                if self.plant(species as usize, x, z).is_some() {
                    bases.push([x, 0.0, z]);
                    germinated += 1;
                }
            }
        }
        germinated
    }

    /// New stems from around the bases of plants that sprout
    /// ([`PlantType::sprouts_per_year`]), while their clump has room. Each
    /// leans out of the clump; its leader turns back up as it grows, so a
    /// clump opens into a vase.
    fn sprout(&mut self, dt: f32) -> u32 {
        /// Radius of a clump, metres.
        const CLUMP: f32 = 0.6;
        /// How far a new stem leans out, radians.
        const LEAN: f32 = 0.35;
        if self.species.iter().all(|s| s.sprouts_per_year <= 0.0) {
            return 0;
        }
        let mut stems: Vec<(u16, [f32; 3])> = self
            .plants
            .iter()
            .map(|p| (p.species, [p.position.x, 0.0, p.position.z]))
            .collect();
        let grid = HashGrid::new(stems.iter().map(|s| s.1), CLUMP);
        let mut sprouted = 0;
        for i in 0..self.plants.len() {
            // Stems sprout from the clump's root crown, not their own: a
            // clump holds its ground rather than creeping away.
            // Sprouting draws on the roots' reserves: a stem in the open
            // sprouts freely, a shaded one seldom.
            let (species, origin, age, reserves) = {
                let p = &self.plants[i];
                let s = &self.species[p.species as usize];
                let reserves = (p.root_vigor / p.root_vigor_cap(s).max(1e-6)).clamp(0.0, 1.0);
                (p.species, p.clump, p.age, reserves)
            };
            let (rate, most) = {
                let s = &self.species[species as usize];
                (
                    s.sprouts_per_year * reserves * reserves,
                    s.clump_stems as usize,
                )
            };
            if rate <= 0.0 || age < 2.0 {
                continue;
            }
            for _ in 0..self.rng.stochastic_round(rate * dt) {
                if self.plants.len() >= self.settings.max_plants {
                    return sprouted;
                }
                let near = |b: [f32; 3]| {
                    (b[0] - origin.x).powi(2) + (b[2] - origin.z).powi(2) < CLUMP * CLUMP
                };
                let mut clump = 0;
                grid.for_each_near([origin.x, 0.0, origin.z], 1, |j| {
                    let (other, b) = stems[j as usize];
                    clump += usize::from(other == species && near(b));
                });
                clump += stems[grid.len()..]
                    .iter()
                    .filter(|(other, b)| *other == species && near(*b))
                    .count();
                if clump >= most {
                    break;
                }
                let angle = self.rng.range(0.0, std::f32::consts::TAU);
                let out = Vec3::new(angle.cos(), 0.0, angle.sin());
                let at = origin + out * (CLUMP * 0.5 * self.rng.next_f32().sqrt());
                let Some(new) = self.plant(species as usize, at.x, at.z) else {
                    continue;
                };
                let lean = Vec4::from_axis_angle(Vec3::new(0.0, 1.0, 0.0).cross(out), LEAN);
                let s = &self.species[species as usize];
                let plant = &mut self.plants[new];
                plant.clump = origin;
                plant.modules[0].orientation = lean.mul_quat(plant.modules[0].orientation);
                plant.update_geometry(s, &self.prototypes);
                stems.push((species, [at.x, 0.0, at.z]));
                sprouted += 1;
            }
        }
        sprouted
    }

    /// Aggregate measurements.
    pub fn census(&self) -> Census {
        let mut census = Census {
            plants: self.plants.len(),
            per_species: vec![0; self.species.len()],
            ..Census::default()
        };
        for plant in &self.plants {
            census.modules += plant.modules.len();
            census.nodes += plant.nodes.len();
            census.per_species[plant.species as usize] += 1;
            census.max_height = census.max_height.max(plant.height());
        }
        census.vines = self.vines.len();
        census.biomass = self
            .plants
            .par_iter()
            .map(|p| p.wood_volume(&self.prototypes))
            .sum();
        census
    }
}

/// Fig. 15a's measure, recovered from the light values: `f_i` is
/// `-ln(Q_i) / sensitivity`, the overlap as a fraction of sphere `i`.
fn overlap_ratio(set: &SphereSet, light: &[[f32; 2]], sensitivity: f32) -> f32 {
    let mut overlap = 0.0f64;
    let mut volume = 0.0f64;
    for (s, l) in set.spheres.iter().zip(light) {
        let v = fabelgeist_spatial_hash::sphere_volume(s[3]) as f64;
        let f = -(l[0].max(1e-30)).ln() / sensitivity.max(1e-6);
        overlap += f as f64 * v;
        volume += v;
    }
    if volume > 0.0 {
        (overlap / volume) as f32
    } else {
        0.0
    }
}

/// Split `data` into the per-plant slices `offsets` describe.
fn split_by<'a, T>(mut data: &'a mut [T], offsets: &[usize], total: usize) -> Vec<&'a mut [T]> {
    let mut slices = Vec::with_capacity(offsets.len());
    for (i, &offset) in offsets.iter().enumerate() {
        let end = offsets.get(i + 1).copied().unwrap_or(total);
        let (head, tail) = data.split_at_mut(end - offset);
        slices.push(head);
        data = tail;
    }
    slices
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alone(species: PlantType) -> Ecosystem {
        let species = PlantType {
            seeding_frequency: 0.0,
            ..species
        };
        let climate = Climate::new(species.temperature_optimum, species.precipitation_optimum);
        let mut ecosystem = Ecosystem::new(Terrain::flat(40.0, 40.0), climate, vec![species]);
        ecosystem.plant(0, 0.0, 0.0);
        ecosystem
    }

    #[test]
    fn growing_frame_by_frame_is_smooth_and_matches_stepping() {
        let mut stepped = alone(PlantType::deciduous());
        let mut smooth = alone(PlantType::deciduous());
        while stepped.time < 20.0 {
            pollster::block_on(stepped.step()).unwrap();
        }
        let mut last = smooth.plants[0].height();
        let mut largest = 0.0f32;
        while smooth.time < 20.0 - 1e-3 {
            pollster::block_on(smooth.advance(1.0 / 60.0)).unwrap();
            let height = smooth.plants[0].height();
            largest = largest.max((height - last).abs());
            last = height;
        }
        // A frame's growth is a sliver, even across a step.
        assert!(largest < 0.02, "a {largest} m leap in a frame");
        let (a, b) = (&stepped.plants[0], &smooth.plants[0]);
        assert_eq!(a.modules.len(), b.modules.len());
        assert!((a.height() - b.height()).abs() < 0.01 * a.height());
    }

    #[test]
    fn a_shrub_holds_a_clump_of_stems() {
        let species = PlantType {
            seeding_frequency: 0.0,
            ..PlantType::shrub()
        };
        let most = species.clump_stems as usize;
        let climate = Climate::new(species.temperature_optimum, species.precipitation_optimum);
        let mut ecosystem = Ecosystem::new(Terrain::flat(20.0, 20.0), climate, vec![species]);
        ecosystem.plant(0, 0.0, 0.0);
        while ecosystem.time < 60.0 {
            pollster::block_on(ecosystem.step()).unwrap();
        }
        // The first stem is long dead, the clump lives on where it stood.
        let stems = ecosystem.plants.len();
        assert!((2..=most).contains(&stems), "{stems} stems");
        for plant in &ecosystem.plants {
            let from = (plant.position.x.powi(2) + plant.position.z.powi(2)).sqrt();
            assert!(from < 0.6, "a stem {from} m from the clump");
        }
    }
}

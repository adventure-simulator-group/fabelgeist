//! The ecosystem scale (Sec. 6): many plants on a terrain, in a climate,
//! competing for light and spreading by seed.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::error::{BotanyError, Result};
use crate::policy::{GlobalShadowing, Mortality, Seeding};
use crate::units::{GroundPoint, StepDuration, Years};
use fabelgeist_determinism::Seed;
use fabelgeist_math::{Vec3, Vec4};
use fabelgeist_spatial_hash::HashGrid;
use rayon::prelude::*;
use web_time::Instant;

use crate::climate::Climate;
use crate::environment::{Environment, EnvironmentSettings, OrientRequest, ShadowGrid, SphereSet};
use crate::identity::{
    ModuleId, ModuleIndex, PlantId, PlantIndex, SpeciesId, VineId, VineIndex, VineSpeciesId,
};
use crate::obstacles::Obstacles;
use crate::plant::{Plant, mature_extent};
use crate::prototype::{NONE, PrototypeLibrary};
use crate::rng::Rng;
use crate::species::PlantType;
use crate::terrain::Terrain;
use crate::vine::{BarkIndex, Supports, Vine, VineType};
use fabelgeist_math::units::Metres;

mod orientation;
mod seeding;

#[derive(Clone, Debug, PartialEq)]
pub struct EcosystemSettings {
    /// Years per step.
    pub dt: StepDuration,
    pub environment: EnvironmentSettings,
    /// Sec. 6.2. Off, modules see only their local light.
    pub global_shadowing: GlobalShadowing,
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
    /// Plants spread by seed. Off, only what is planted grows -- a shrub
    /// still renews its clump from the root, where it stands.
    pub seeding: Seeding,
    /// Plants and vines die: starved, in the dark, or of age. Off, each is
    /// kept instead as it is where it would have died -- growing no more,
    /// still shading and crowding what grows round it
    /// ([`Plant::preserved`]); turned back on, what was kept dies.
    pub mortality: Mortality,
    /// A seed landing closer than this to a standing plant's base does not
    /// take root.
    pub seed_spacing: f32,
    /// A new module whose best orientation still leaves more than this
    /// share of it inside other modules does not grow: the bud stays
    /// dormant until the space frees up (crown shyness; not in the paper,
    /// which only steers modules away from each other).
    pub bud_crowding_limit: f32,
    pub seed: Seed,
}

impl Default for EcosystemSettings {
    fn default() -> Self {
        Self {
            dt: StepDuration::HALF_YEAR,
            environment: EnvironmentSettings::default(),
            global_shadowing: GlobalShadowing::Enabled,
            shadow_cell: 1.0,
            max_shadow_cells: 8 << 20,
            shadow_height: 48.0,
            shadow_strength: 0.35,
            shadow_decay: 0.97,
            max_plants: 500_000,
            seed_spacing: 1.0,
            seeding: Seeding::Enabled,
            mortality: Mortality::Enabled,
            bud_crowding_limit: 0.5,
            seed: Seed::from_u64(1),
        }
    }
}

/// What a step did and how long it took.
#[derive(Clone, Debug, Default)]
pub struct StepReport {
    pub time: Years,
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
    pub time: Years,
    /// When the next step's regulation is due ([`Ecosystem::advance`]).
    next_tick: Years,
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
            time: Years::from_years(0.0),
            next_tick: Years::from_years(0.0),
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
                .adaptation(
                    &self.species[plant.species.index()],
                    Metres::from_metres(plant.position.y),
                )
                .ratio();
        }
    }

    /// Plant a seedling of `species` at `(x, z)`, on the ground. `None` if the
    /// soil there is blocked.
    pub fn plant(&mut self, species: SpeciesId, ground: GroundPoint) -> Result<PlantIndex> {
        // Height-field sampling is native metre arithmetic.
        let (x, z) = (ground.x.metres(), ground.z.metres());
        if species.index() >= self.species.len() {
            return Err(BotanyError::UnknownSpecies { species });
        }
        if !self.terrain.is_open(x, z) {
            return Err(BotanyError::BlockedGround);
        }
        let y = self.terrain.height_at(x, z);
        let adaptation = self
            .climate
            .adaptation(&self.species[species.index()], Metres::from_metres(y))
            .ratio();
        let rng = self.rng.fork();
        self.next_id += 1;
        self.plants.push(Plant::new(
            PlantId::from_word(self.next_id),
            species,
            &self.species[species.index()],
            &self.prototypes,
            Vec3::new(x, y, z),
            adaptation,
            rng,
        ));
        Ok(PlantIndex::from_word(self.plants.len() - 1))
    }

    /// Plant a climber of `species` at `(x, z)`. `None` if the soil there is
    /// blocked.
    pub fn plant_vine(&mut self, species: VineSpeciesId, ground: GroundPoint) -> Result<VineIndex> {
        // Height-field sampling is native metre arithmetic.
        let (x, z) = (ground.x.metres(), ground.z.metres());
        if species.index() >= self.vine_species.len() {
            return Err(BotanyError::UnknownVineSpecies { species });
        }
        if !self.terrain.is_open(x, z) {
            return Err(BotanyError::BlockedGround);
        }
        let y = self.terrain.height_at(x, z);
        let rng = self.rng.fork();
        self.next_id += 1;
        self.vines.push(Vine::new(
            VineId::from_word(self.next_id),
            species,
            &self.vine_species[species.index()],
            Vec3::new(x, y, z),
            rng,
        ));
        self.rebuild_bark();
        Ok(VineIndex::from_word(self.vines.len() - 1))
    }

    /// Where a seed thrown from `origin` along `direction` comes to rest, as
    /// `(x, z)`: the first ground it meets -- or, if it meets a building
    /// first, the open soil at the building's foot on the side it struck.
    /// `None` if it comes down nowhere a plant may root.
    pub fn landing(&self, origin: Vec3, direction: Vec3) -> Option<GroundPoint> {
        let d = direction.normalize();
        let (low, _) = self.terrain.height_range();
        let mut t = 0.0f32;
        for _ in 0..4096 {
            let p = origin + d * t;
            if p.y < low - 1.0 && d.y <= 0.0 {
                return None;
            }
            let above = p.y - self.terrain.height_at(p.x, p.z);
            if above <= 0.01 {
                return self.terrain.is_open(p.x, p.z).then_some(GroundPoint::new(
                    Metres::from_metres(p.x),
                    Metres::from_metres(p.z),
                ));
            }
            let solid = self
                .obstacles
                .as_ref()
                .map_or(f32::INFINITY, |o| o.distance(p));
            if solid <= 0.01 {
                // Struck a wall: it falls to the wall's foot, on this side.
                let normal = self.obstacles.as_ref().and_then(|o| o.normal(p))?;
                let out = Vec3::new(normal.x, 0.0, normal.z);
                let out = if out.length() > 1e-3 {
                    out.normalize()
                } else {
                    Vec3::new(-d.x, 0.0, -d.z).normalize()
                };
                return (0..=16)
                    .map(|k| p + out * (0.25 * k as f32))
                    .find(|q| self.terrain.is_open(q.x, q.z))
                    .map(|q| GroundPoint::new(Metres::from_metres(q.x), Metres::from_metres(q.z)));
            }
            // The height field is no distance: step a fraction of the
            // height above it.
            t += (above * 0.5).min(solid).max(0.02);
        }
        None
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
        let ids: HashMap<PlantId, PlantIndex> = self
            .plants
            .iter()
            .enumerate()
            .map(|(i, p)| (p.id, PlantIndex::from_word(i)))
            .collect();
        let plants = &self.plants;
        let mut maps: HashMap<PlantIndex, HashMap<ModuleId, ModuleIndex>> = HashMap::new();
        let mut modules = |p: PlantIndex, id: ModuleId| -> Option<ModuleIndex> {
            maps.entry(p)
                .or_insert_with(|| {
                    plants[p.index()]
                        .modules
                        .iter()
                        .enumerate()
                        .map(|(i, m)| (m.id, ModuleIndex::from_word(i as u32)))
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
                .map(|s| self.climate.adaptation(s, Metres::from_metres(y)).ratio())
                .collect();
            let total: f32 = weights.iter().sum();
            if total <= 1e-6 {
                continue;
            }
            let mut pick = self.rng.next_f32() * total;
            let mut species = SpeciesId::from_word(0);
            for (i, w) in weights.iter().enumerate() {
                species = SpeciesId::from_word(i as u16);
                if pick < *w {
                    break;
                }
                pick -= w;
            }
            if self
                .plant(
                    species,
                    GroundPoint::new(Metres::from_metres(x), Metres::from_metres(z)),
                )
                .is_ok()
            {
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
                            extents[plant.species.index()][module.prototype.index()];
                        let Some(root) =
                            plant.attachment(prototypes, ModuleIndex::from_word(u as u32))
                        else {
                            continue;
                        };
                        let c = root + module.orientation.rotate_vec3(center);
                        [c.x, c.y, c.z, radius]
                    } else {
                        module.sphere
                    };
                    // Environment kernels use a global native index and sentinel.
                    parents[u] = module
                        .parent
                        .map_or(NONE, |parent| (offset + parent.index()) as u32);
                }
            });
        for vine in &self.vines {
            let base = set.spheres.len() as u32;
            for (sphere, parent) in vine.spheres(&self.vine_species[vine.species.index()]) {
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
        self.develop(self.settings.dt.duration());
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
    pub async fn advance(&mut self, years: Years) -> Result<Option<StepReport>> {
        let mut last = None;
        let mut left = years.years().max(0.0);
        // A step due at the very end of the span is the next call's.
        while left > 1e-6 {
            if self.time.years() >= self.next_tick.years() - 1e-5 {
                last = Some(self.tick().await?);
                self.next_tick =
                    Years::from_years(self.time.years() + self.settings.dt.duration().years());
            }
            let span = left.min(self.next_tick.years() - self.time.years());
            self.develop(Years::from_years(span));
            left -= span;
        }
        Ok(last)
    }

    /// Grow every plant for `years` at the rates its last step set, and lay
    /// it out anew.
    pub fn develop(&mut self, years: Years) {
        let species = &self.species;
        let prototypes = &self.prototypes;
        self.plants
            .par_iter_mut()
            .filter(|p| p.alive)
            .for_each(|plant| {
                let s = &species[plant.species.index()];
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
                vine.develop(&vine_species[vine.species.index()], &supports, years)
            });
        }
        self.time = Years::from_years(self.time.years() + years.years());
    }

    /// A step's regulation, at `self.time`, over the `settings.dt` years to
    /// the next.
    async fn tick(&mut self) -> Result<StepReport> {
        let start = Instant::now();
        let dt = self.settings.dt.duration();
        let mut report = StepReport::default();
        self.refresh_occluders();

        // 1. Light: local (Eq. 1) and global (Sec. 6.2).
        let clock = Instant::now();
        let (offsets, total) = self.offsets();
        let set = self.sphere_set(&offsets, total);
        let shadowing = self.settings.global_shadowing.enabled();
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
            let species = &self.vine_species[vine.species.index()];
            vine.set_light(&light[at..at + count], species, shadowing);
            at += count;
        }
        report.light = clock.elapsed();

        // 2. Vigor, shedding, development and new modules, plant by plant.
        let clock = Instant::now();
        let species = &self.species;
        let prototypes = &self.prototypes;
        let mortality = self.settings.mortality.enabled();
        let outcomes: Vec<_> = self
            .plants
            .par_iter_mut()
            .zip(offsets.par_iter())
            .map(|(plant, &offset)| {
                let s = &species[plant.species.index()];
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
                plant.immortal = !mortality;
                if mortality && plant.preserved {
                    plant.alive = false;
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
        self.orient_new_modules(&mut report).await?;
        report.orient = clock.elapsed();

        // 4. Geometry.
        let clock = Instant::now();
        let species = &self.species;
        let prototypes = &self.prototypes;
        self.plants
            .par_iter_mut()
            .filter(|p| p.alive && !p.preserved)
            .for_each(|plant| plant.update_geometry(&species[plant.species.index()], prototypes));
        report.geometry = clock.elapsed();

        // 5. Death and seeding.
        let clock = Instant::now();
        let before = self.plants.len();
        let seeds: Vec<(SpeciesId, Vec3, f32, u32)> = self
            .plants
            .iter()
            .zip(&outcomes)
            .filter(|(p, o)| p.alive && o.seeds > 0)
            .map(|(p, o)| {
                (
                    p.species,
                    p.position,
                    self.species[p.species.index()].seeding_radius,
                    o.seeds,
                )
            })
            .collect();
        self.plants.retain(|p| p.alive && !p.modules.is_empty());
        report.died = (before - self.plants.len()) as u32;
        let sown = if self.settings.seeding.enabled() {
            self.disperse(&seeds)
        } else {
            0
        };
        report.germinated = sown + self.sprout(dt.years());
        report.seeding = clock.elapsed();

        // 6. Vines: rates, new shoots and wood, and their holds on whatever
        // plants are left.
        let vine_species = &self.vine_species;
        self.vines.par_iter_mut().for_each(|vine| {
            vine.immortal = !mortality;
            if mortality && vine.preserved {
                vine.alive = false;
            }
            vine.regulate(&vine_species[vine.species.index()], dt)
        });
        self.refresh_vines();

        report.time = self.time;
        report.plants = self.plants.len();
        report.modules = self.plants.iter().map(|p| p.modules.len()).sum();
        report.total = start.elapsed();
        Ok(report)
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
            census.per_species[plant.species.index()] += 1;
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
        ecosystem
            .plant(
                SpeciesId::from_word(0),
                GroundPoint::new(Metres::from_metres(0.0), Metres::from_metres(0.0)),
            )
            .unwrap();
        ecosystem
    }

    #[test]
    fn growing_frame_by_frame_is_smooth_and_matches_stepping() {
        let mut stepped = alone(PlantType::deciduous());
        let mut smooth = alone(PlantType::deciduous());
        while stepped.time < Years::from_years(20.0) {
            pollster::block_on(stepped.step()).unwrap();
        }
        let mut last = smooth.plants[0].height();
        let mut largest = 0.0f32;
        while smooth.time < Years::from_years(20.0 - 1e-3) {
            pollster::block_on(smooth.advance(Years::from_years(1.0 / 60.0))).unwrap();
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
    fn without_seeding_only_what_was_planted_grows() {
        let species = PlantType::default();
        let climate = Climate::new(species.temperature_optimum, species.precipitation_optimum);
        let grown = |seeding: bool| {
            let mut ecosystem =
                Ecosystem::new(Terrain::flat(40.0, 40.0), climate, vec![species.clone()])
                    .with_settings(EcosystemSettings {
                        seeding: Seeding::from_enabled(seeding),
                        ..EcosystemSettings::default()
                    });
            ecosystem
                .plant(
                    SpeciesId::from_word(0),
                    GroundPoint::new(Metres::from_metres(0.0), Metres::from_metres(0.0)),
                )
                .unwrap();
            // Past its flowering age, so it seeds if it may.
            while ecosystem.time < Years::from_years(100.0) {
                pollster::block_on(ecosystem.step()).unwrap();
            }
            ecosystem.plants.len()
        };
        assert!(grown(true) > 1, "a flowering tree should seed");
        assert_eq!(grown(false), 1);
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
        ecosystem
            .plant(
                SpeciesId::from_word(0),
                GroundPoint::new(Metres::from_metres(0.0), Metres::from_metres(0.0)),
            )
            .unwrap();
        while ecosystem.time < Years::from_years(60.0) {
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

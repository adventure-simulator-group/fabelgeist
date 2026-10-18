//! The simulation, on a thread of its own.
//!
//! A step of a large ecosystem takes longer than a frame, so the simulation
//! does not run on the UI thread: it runs here, on its own GPU context, and
//! publishes [`Snapshot`]s -- compact per-segment and per-leaf instances the
//! renderer draws directly. The window keeps its frame rate however slow the
//! simulation is, and only the latest snapshot is ever kept.

use crate::species::{Foliage, LitterContribution};
use crate::units::Years;
use crate::units::{Celsius, Millimetres};
use fabelgeist_determinism::Seed;
use fabelgeist_math::units::Metres;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::prototype::NONE;
use crate::scenes::Scene;
use crate::vine::Hold;
use crate::{Biome, Ecosystem, Plant, PlantType, PrototypeLibrary, Vine, VineType};
use fabelgeist_gpu::globals::WgpuContext;
use fabelgeist_hlod::Hierarchy;
use fabelgeist_math::Vec3;
use fabelgeist_surface_field::{Shape, SurfaceIndex};
use rayon::prelude::*;

use crate::viewer::profile::Profile;

mod commands;
mod foliage;
mod measurements;

/// One branch segment, as the branch shader reads it: a tube between two
/// rings, curving from one's tangent to the other's.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BranchInstance {
    /// Start ring: centre and radius.
    pub a: [f32; 4],
    /// End ring: centre and radius.
    pub b: [f32; 4],
    /// Each ring's tangent, octahedral-encoded. Where an axis runs on, the
    /// segments either side share the ring there exactly.
    pub tangents: [u32; 2],
    /// The surface's `v` at the start ring: length along the axis in
    /// circumferences, so that a unit along is as long as a turn around,
    /// running on across joints. Bark is drawn in `(u, v)`, `u` a turn
    /// around.
    pub v0: f32,
    /// The module's light as drawn, at the end ring: dims branches deep in
    /// a crown.
    pub light: f32,
    /// The plant it belongs to, in [`Snapshot::plant_bounds`].
    pub plant: u32,
    /// Bit 0: starts a lateral.
    pub flags: u32,
    /// The light at the start ring: the end light of the segment it grows
    /// out of, so shading runs on across a joint too.
    pub light_start: f32,
    /// The surface's `v` at the end ring: the next segment's `v0`.
    pub v1: f32,
    /// The axis it is part of: every segment of one branch, from where it
    /// leaves its parent to its tip. Its surface blends with other axes'
    /// ([`Snapshot::blend`]), never its own.
    pub axis: u32,
    /// For a lateral, how far along it -- as a share of its length -- it
    /// leaves its parent's surface: where its fillet grows, and so where
    /// its rings are gathered.
    pub exit: f32,
    pub pad: [u32; 2],
}

/// A plant as a whole, for culling and choosing its detail.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PlantInstance {
    /// Bounding sphere: centre and radius.
    pub sphere: [f32; 4],
    pub species: u32,
    pub pad: [u32; 3],
}

/// One leaf cluster at a branch tip.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LeafInstance {
    pub center: [f32; 3],
    pub size: f32,
    pub axis: [f32; 3],
    pub light: f32,
    /// The middle of the plant's foliage and its radius: how deep in the
    /// crown the cluster sits, which shades it from the sky.
    pub crown: [f32; 4],
    pub species: u32,
    pub seed: f32,
    /// The plant it belongs to, in [`Snapshot::plant_bounds`].
    pub plant: u32,
    pub pad: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TerrainVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 3],
}

pub struct TerrainGeometry {
    pub vertices: Vec<TerrainVertex>,
    pub indices: Vec<u32>,
}

/// A vertex of a building: its colour and what it is made of
/// ([`fabelgeist_buildings::Material`]), for the shader to lay masonry, tiles
/// and timber over.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct StructureVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 3],
    pub material: f32,
}

/// The scene's buildings, if it has any.
#[derive(Default)]
pub struct StructureGeometry {
    pub vertices: Vec<StructureVertex>,
    pub indices: Vec<u32>,
}

#[derive(Clone, Debug)]
pub struct SpeciesInfo {
    pub name: String,
    pub bark: [f32; 3],
    pub leaf: [f32; 3],
    pub foliage: Foliage,
    pub plants: usize,
}

/// The litter on the ground: how much foliage stands over each cell of a
/// grid over the terrain -- per unit of ground, needles and broad leaves
/// apart -- which is what falls onto it.
#[derive(Clone, Debug, Default)]
pub struct Litter {
    /// The grid's corner, (x, z).
    pub origin: [f32; 2],
    pub cell: f32,
    pub columns: u32,
    /// Row by row: needles, broad leaves, and two unused.
    pub cells: Vec<[f32; 4]>,
}

/// What the renderer and the panel need of the ecosystem at one moment.
pub struct Snapshot {
    /// Changes when the scene is reset: the terrain and camera follow it.
    pub scene_generation: u64,
    pub time: f32,
    pub plants: usize,
    pub modules: usize,
    pub step_ms: f32,
    pub species: Vec<SpeciesInfo>,
    /// Each plant's bounds, which its instances refer to by index.
    pub plant_bounds: Vec<PlantInstance>,
    pub branches: Vec<BranchInstance>,
    pub leaves: Vec<LeafInstance>,
    /// The branches' shapes, for surfaces to find each other and grow into
    /// one where they meet: one round cone per segment, its payload the
    /// segment's index in `branches`. The renderer indexes them on the GPU.
    /// Empty when the profile draws plain junctions.
    pub blend: BlendShapes,
    /// Whether branches grow fillets where they meet, and how far one
    /// reaches, in radii of the branch it grows from.
    pub fillets: bool,
    pub fillet_size: f32,
    pub litter: Litter,
    /// The hierarchy over the plants: each plant's instances, and above them
    /// the proxies standing in for groves (`hlod`).
    pub hierarchy: Hierarchy,
    /// Present on the first snapshot of a scene, and the buildings with it.
    pub terrain: Option<Arc<TerrainGeometry>>,
    pub structures: Option<Arc<StructureGeometry>>,
    /// Lower and upper corner of the terrain and everything on it.
    pub bounds: [[f32; 3]; 2],
    pub biome: Biome,
}

pub enum Command {
    Reset(Scene),
    Play(bool),
    Step,
    /// Simulated years per second of wall time; `f32::INFINITY` for as fast
    /// as it goes.
    Speed(f32),
    Climate {
        temperature: Celsius,
        precipitation: Millimetres,
    },
    GlobalShadowing(bool),
    /// Whether plants spread by seed.
    Seeding(bool),
    /// Whether plants and vines die.
    Mortality(bool),
    Profile(Profile),
    /// Clear the plants within `radius` of `(x, z)`.
    Gap {
        x: f32,
        z: f32,
        radius: f32,
    },
    /// Sow a seed of `species` -- the snapshot's species, the climbers
    /// after the plants -- where one thrown from `origin` along `direction`
    /// lands.
    Sow {
        species: usize,
        origin: [f32; 3],
        direction: [f32; 3],
    },
}

pub struct Simulation {
    commands: Sender<Command>,
    latest: Arc<Mutex<Option<Snapshot>>>,
    _thread: JoinHandle<()>,
}

impl Simulation {
    pub fn start(scene: Scene, profile: Profile) -> crate::error::Result<Self> {
        let (commands, receiver) = channel();
        let latest = Arc::new(Mutex::new(None));
        let slot = latest.clone();
        let thread = std::thread::Builder::new()
            .name("botany simulation".into())
            .spawn(move || {
                if let Err(error) = run(scene, profile, receiver, slot) {
                    eprintln!("simulation stopped: {error:#}");
                }
            })
            .map_err(crate::error::BotanyError::ThreadSpawn)?;
        Ok(Self {
            commands,
            latest,
            _thread: thread,
        })
    }

    pub fn send(&self, command: Command) {
        let _ = self.commands.send(command);
    }

    /// The newest snapshot since the last call, if there is one.
    pub fn take(&self) -> Option<Snapshot> {
        self.latest.lock().ok()?.take()
    }
}

struct State {
    ecosystem: Ecosystem,
    generation: u64,
    terrain_sent: bool,
    playing: bool,
    years_per_second: f32,
    last_step_ms: f32,
    profile: Profile,
}

fn run(
    scene: Scene,
    profile: Profile,
    commands: Receiver<Command>,
    latest: Arc<Mutex<Option<Snapshot>>>,
) -> anyhow::Result<()> {
    let context = pollster::block_on(WgpuContext::new())?;
    let build = |scene: Scene| -> crate::error::Result<Ecosystem> {
        scene.build(Seed::from_u64(1)).with_gpu(context.clone())
    };
    let build = |scene: Scene, profile: &Profile| -> anyhow::Result<Ecosystem> {
        let mut ecosystem = build(scene)?;
        ecosystem.settings.dt = profile.step_years;
        Ok(ecosystem)
    };
    let mut state = State {
        ecosystem: build(scene, &profile)?,
        generation: 1,
        terrain_sent: false,
        playing: true,
        years_per_second: f32::INFINITY,
        last_step_ms: 0.0,
        profile,
    };
    let mut dirty = true;
    let mut last_publish = Instant::now() - Duration::from_secs(1);
    let mut publish_cost = Duration::ZERO;
    let mut last_advance = Instant::now();
    let mut next_step = Instant::now();

    loop {
        // Commands first. While paused with nothing to publish, wait for one
        // rather than spin.
        let wait = if state.playing || dirty {
            Duration::ZERO
        } else {
            Duration::from_millis(100)
        };
        let mut first = match commands.recv_timeout(wait) {
            Ok(command) => Some(command),
            Err(RecvTimeoutError::Timeout) => None,
            Err(RecvTimeoutError::Disconnected) => return Ok(()),
        };
        let mut step_once = false;
        while let Some(command) = first.take().or_else(|| commands.try_recv().ok()) {
            state.apply_command(command, &build, &mut step_once)?;
            dirty = true;
        }

        let now = Instant::now();
        let dt = state.ecosystem.settings.dt.duration().years();
        if !state.profile.continuous {
            // A step at a time, as often as the speed asks and no oftener.
            if step_once || (state.playing && now >= next_step) {
                let report = pollster::block_on(state.ecosystem.step())?;
                state.last_step_ms = report.total.as_secs_f32() * 1e3;
                let interval = dt / state.years_per_second.max(1e-3);
                next_step = now + Duration::from_secs_f32(interval.min(10.0));
                dirty = true;
            } else if state.playing {
                std::thread::sleep((next_step - now).min(Duration::from_millis(5)));
            }
            last_advance = now;
        }
        // Growth is continuous: each pass advances by the wall time since
        // the last at the chosen speed, and a step's regulation runs
        // whenever one falls due within it.
        let years = if !state.profile.continuous {
            None
        } else if step_once {
            Some(dt)
        } else if state.playing {
            let elapsed = now.duration_since(last_advance).as_secs_f32();
            if elapsed < 1.0 / 120.0 {
                std::thread::sleep(Duration::from_secs_f32(1.0 / 120.0 - elapsed));
                None
            } else if state.years_per_second.is_finite() {
                // A slow machine falls behind rather than leaping.
                Some((elapsed * state.years_per_second).min(dt * 4.0))
            } else {
                Some(dt)
            }
        } else {
            None
        };
        if !state.playing {
            last_advance = now;
        }
        if let Some(years) = years {
            last_advance = now;
            if let Some(report) =
                pollster::block_on(state.ecosystem.advance(Years::from_years(years)))?
            {
                state.last_step_ms = report.total.as_secs_f32() * 1e3;
            }
            dirty = true;
        }

        // Publish at most as often as it costs, so a large scene spends its
        // time stepping rather than packing snapshots nobody draws -- and no
        // oftener than the profile asks.
        let every = Duration::from_secs_f32(state.profile.refresh_ms.max(1.0) / 1e3);
        let due = last_publish.elapsed() >= (publish_cost * 2).max(every);
        if dirty && (due || !state.playing) {
            let started = Instant::now();
            let snapshot = capture(
                &state.ecosystem,
                &state.profile,
                state.generation,
                !state.terrain_sent,
                state.last_step_ms,
            );
            state.terrain_sent = true;
            publish_cost = started.elapsed();
            last_publish = Instant::now();
            if let Ok(mut slot) = latest.lock() {
                // The terrain rides on the first snapshot of a scene; do not
                // lose it to a newer one the window has not taken yet.
                let mut snapshot = snapshot;
                if let Some(old) = slot.take()
                    && snapshot.terrain.is_none()
                    && old.scene_generation == snapshot.scene_generation
                {
                    snapshot.terrain = old.terrain;
                    snapshot.structures = old.structures;
                }
                *slot = Some(snapshot);
            }
            dirty = false;
        }
    }
}

/// Everything the window needs of `ecosystem` now, as `profile` draws it;
/// the terrain too when `with_terrain`.
pub fn capture(
    ecosystem: &Ecosystem,
    profile: &Profile,
    generation: u64,
    with_terrain: bool,
    step_ms: f32,
) -> Snapshot {
    let terrain = with_terrain.then(|| Arc::new(terrain_geometry(ecosystem)));
    let structures = with_terrain.then(|| Arc::new(structure_geometry(ecosystem)));

    let mut per_plant: Vec<(Vec<BranchInstance>, Vec<LeafInstance>, PlantInstance)> = ecosystem
        .plants
        .par_iter()
        .enumerate()
        .map(|(i, plant)| {
            let species = &ecosystem.species[plant.species.index()];
            instances(plant, i as u32, species, &ecosystem.prototypes)
        })
        .collect();
    let litter = if profile.litter {
        litter(ecosystem, &per_plant)
    } else {
        Litter::default()
    };
    // The vines after the plants, each as a plant of its own, their species
    // after the plants' species.
    let tree_branches: usize = per_plant.iter().map(|p| p.0.len()).sum();
    let first = per_plant.len() as u32;
    let vines: Vec<_> = ecosystem
        .vines
        .par_iter()
        .enumerate()
        .map(|(i, vine)| {
            let species = &ecosystem.vine_species[vine.species.index()];
            let kind = (ecosystem.species.len() + vine.species.index()) as u32;
            vine_instances(vine, first + i as u32, kind, species)
        })
        .collect();
    per_plant.extend(vines);
    let forest = crate::viewer::hlod::build(per_plant, profile.proxies);
    // The trees' own branches: proxies are copies, and never blend, and a
    // vine lies on the bark rather than growing into it.
    let blend = if profile.fillets {
        blend_index(
            &forest.branches[..tree_branches],
            profile.fillet_size,
            profile.fillet_thinnest,
        )
    } else {
        BlendShapes::default()
    };
    let crate::viewer::hlod::Forest {
        branches,
        leaves,
        plants: plant_bounds,
        hierarchy,
        ..
    } = forest;

    let measurements::SceneMeasurements {
        species,
        bounds,
        biome,
    } = measurements::SceneMeasurements::new(ecosystem);

    Snapshot {
        scene_generation: generation,
        time: ecosystem.time.years(),
        plants: ecosystem.plants.len(),
        modules: ecosystem.plants.iter().map(|p| p.modules.len()).sum(),
        step_ms,
        species,
        plant_bounds,
        branches,
        leaves,
        hierarchy,
        blend,
        fillets: profile.fillets,
        fillet_size: profile.fillet_size,
        litter,
        terrain,
        structures,
        bounds,
        biome,
    }
}

/// The litter grid's cells are at least this large, metres, and at most
/// this many a side.
const LITTER_CELL: f32 = 0.5;
const LITTER_COLUMNS: f32 = 256.0;

/// Each plant's foliage, spread over the ground under its crown.
fn litter(
    ecosystem: &Ecosystem,
    per_plant: &[(Vec<BranchInstance>, Vec<LeafInstance>, PlantInstance)],
) -> Litter {
    let t = &ecosystem.terrain;
    let cell = (t.size[0].max(t.size[1]) / LITTER_COLUMNS).max(LITTER_CELL);
    let columns = (t.size[0] / cell).ceil().max(1.0) as usize;
    let rows = (t.size[1] / cell).ceil().max(1.0) as usize;
    let mut cells = vec![[0.0f32; 4]; columns * rows];
    for (plant, (_, leaves, _)) in ecosystem.plants.iter().zip(per_plant) {
        let species = &ecosystem.species[plant.species.index()];
        // Grass is the ground cover itself.
        if leaves.is_empty() || species.litter == LitterContribution::GroundCover {
            continue;
        }
        let [x, z] = [plant.position.x, plant.position.z];
        let mut area = 0.0;
        let mut reach = 0.0f32;
        for leaf in leaves {
            area += leaf.size * leaf.size;
            reach = reach.max((leaf.center[0] - x).hypot(leaf.center[2] - z));
        }
        let reach = reach.max(0.5);
        // Spread as 1 - (d / r)^2 over the crown's disc, whose integral is
        // half the disc: the density at its centre is twice the average.
        let peak = 2.0 * area / (std::f32::consts::PI * reach * reach);
        let channel = usize::from(species.foliage != Foliage::Needles);
        let lo = |v: f32, o: f32| (((v - reach - o) / cell).floor().max(0.0)) as usize;
        let hi =
            |v: f32, o: f32, n: usize| (((v + reach - o) / cell).ceil().max(0.0) as usize).min(n);
        for row in lo(z, t.origin[1])..hi(z, t.origin[1], rows) {
            for column in lo(x, t.origin[0])..hi(x, t.origin[0], columns) {
                let cx = t.origin[0] + (column as f32 + 0.5) * cell;
                let cz = t.origin[1] + (row as f32 + 0.5) * cell;
                let d2 = ((cx - x).powi(2) + (cz - z).powi(2)) / (reach * reach);
                if d2 < 1.0 {
                    cells[row * columns + column][channel] += peak * (1.0 - d2);
                }
            }
        }
    }
    Litter {
        origin: t.origin,
        cell,
        columns: columns as u32,
        cells,
    }
}

/// Shapes for the surface field, and what their index needs: the cell edge,
/// and how many entries it will hold.
#[derive(Clone, Debug, Default)]
pub struct BlendShapes {
    pub shapes: Vec<Shape>,
    pub cell: f32,
    pub entries: u32,
}

/// Every segment at least `thinnest` in radius, as a round cone reaching
/// `size` of its radius out: as far as its fillets grow.
fn blend_index(branches: &[BranchInstance], size: f32, thinnest: f32) -> BlendShapes {
    let shapes: Vec<Shape> = branches
        .par_iter()
        .enumerate()
        .filter(|(_, b)| b.a[3].max(b.b[3]) >= thinnest)
        .map(|(i, b)| {
            Shape::round_cone(
                [b.a[0], b.a[1], b.a[2]],
                b.a[3],
                [b.b[0], b.b[1], b.b[2]],
                b.b[3],
            )
            .with_object(b.axis)
            .with_reach(size * b.a[3])
            .with_payload(i as u32)
        })
        .collect();
    let cell = SurfaceIndex::cell_for(&shapes);
    let entries = SurfaceIndex::entry_count(&shapes, cell) as u32;
    BlendShapes {
        shapes,
        cell,
        entries,
    }
}

/// Years from a module's birth that it carries leaves behind the tips: the
/// years it takes to develop, and a few more; a conifer keeps its needles
/// longer, further back along its branches.
const FOLIAGE_YEARS: Years = Years::from_years(8.0);
const NEEDLE_YEARS: Years = Years::from_years(14.0);
/// Twigs up to this many tip diameters thick carry leaves too, however old
/// -- spur shoots -- and a conifer's further back.
const TWIG_DIAMETERS: f32 = 2.5;
const NEEDLE_TWIG_DIAMETERS: f32 = 4.0;

/// A segment as the axes are traced: where it runs, and how thick.
struct Segment {
    module: usize,
    node: usize,
    a: Vec3,
    b: Vec3,
    direction: Vec3,
    length: f32,
    diameter: f32,
    end_diameter: f32,
}

/// Octahedral encoding of a unit vector into two snorm16s, as WGSL's
/// `unpack2x16snorm` reads them.
fn octahedral(v: Vec3) -> u32 {
    let n = v / (v.x.abs() + v.y.abs() + v.z.abs()).max(1e-12);
    let sign = |x: f32| if x >= 0.0 { 1.0 } else { -1.0 };
    let (x, y) = if n.z >= 0.0 {
        (n.x, n.y)
    } else {
        ((1.0 - n.y.abs()) * sign(n.x), (1.0 - n.x.abs()) * sign(n.y))
    };
    let snorm = |x: f32| (x.clamp(-1.0, 1.0) * 32767.0).round() as i16 as u16 as u32;
    snorm(x) | (snorm(y) << 16)
}

/// A plant's segments, leaf clusters and bounding sphere; `index` is its
/// place in the snapshot.
///
/// Segments are traced into axes: where a segment carries on into the
/// thickest of what grows from its end, the two share a ring there --
/// the same centre, radius and tangent (the bisector of their directions)
/// -- so the tube runs on without a seam. A segment that starts a lateral
/// begins inside its parent, and is flagged to flare into a collar.
fn instances(
    plant: &Plant,
    index: u32,
    species: &PlantType,
    prototypes: &PrototypeLibrary,
) -> (Vec<BranchInstance>, Vec<LeafInstance>, PlantInstance) {
    // Every grown segment, and which ends at which node.
    let mut segments: Vec<Segment> = Vec::new();
    let mut ending = vec![u32::MAX; plant.nodes.len()];
    for (u, module) in plant.modules.iter().enumerate() {
        let prototype = prototypes.get(module.prototype);
        let Some(offset) = module.node_offset else {
            continue;
        };
        let offset = offset.index();
        if offset + prototype.node_count() > plant.nodes.len() {
            continue;
        }
        for (n, node) in prototype.nodes.iter().enumerate().skip(1) {
            let a = plant.nodes[offset + node.parent as usize].position;
            let tip = plant.nodes[offset + n];
            let axis = tip.position - a;
            let length = axis.length();
            if length < 1e-4 || tip.diameter <= 0.0 {
                continue;
            }
            ending[offset + n] = segments.len() as u32;
            segments.push(Segment {
                module: u,
                node: n,
                a,
                b: tip.position,
                direction: axis / length,
                length,
                diameter: tip.diameter,
                end_diameter: tip.end_diameter,
            });
        }
    }

    // What each segment grows out of: the segment ending where it starts,
    // across module boundaries -- a module's root node is its parent's
    // terminal.
    let start_of = |s: &Segment| -> Option<usize> {
        let module = &plant.modules[s.module];
        let prototype = prototypes.get(module.prototype);
        let parent = prototype.nodes[s.node].parent as usize;
        if parent != 0 {
            return Some(module.node_offset?.index() + parent);
        }
        let up = &plant.modules[module.parent?.index()];
        let terminal = prototypes.get(up.prototype).terminals[module.parent_slot as usize];
        Some(up.node_offset?.index() + terminal as usize)
    };
    let count = segments.len();
    let mut parent = vec![u32::MAX; count];
    let mut next = vec![u32::MAX; count];
    for s in 0..count {
        let Some(node) = start_of(&segments[s]) else {
            continue;
        };
        let p = ending[node];
        if p == u32::MAX {
            continue;
        }
        parent[s] = p;
        // The axis carries on into the thickest of what grows from its end.
        let current = next[p as usize];
        if current == u32::MAX || segments[s].diameter > segments[current as usize].diameter {
            next[p as usize] = s as u32;
        }
    }

    // End diameters: where an axis runs on, the next segment's start.
    let end_diameter = |s: usize| {
        let segment = &segments[s];
        let n = next[s];
        if n != u32::MAX {
            segments[n as usize].diameter
        } else if segment.end_diameter > 0.0 {
            segment.end_diameter.min(segment.diameter)
        } else {
            segment.diameter * 0.5
        }
    };
    // Tangents, the surface's `v` along each axis, and which axis, parents
    // first: each segment's length over its mean circumference, added on.
    let mut t0 = vec![0u32; count];
    let mut v = vec![[0.0f32; 2]; count];
    let mut axis = vec![0u32; count];
    let mut axes = 0;
    for s in 0..count {
        let p = parent[s];
        let continues = p != u32::MAX && next[p as usize] == s as u32;
        if continues {
            axis[s] = axis[p as usize];
        } else {
            axis[s] = axes;
            axes += 1;
        }
        let direction = if continues {
            let p = p as usize;
            v[s][0] = v[p][1];
            (segments[p].direction + segments[s].direction).normalize()
        } else {
            segments[s].direction
        };
        t0[s] = octahedral(direction);
        let girth = std::f32::consts::PI * 0.5 * (segments[s].diameter + end_diameter(s));
        v[s][1] = v[s][0] + segments[s].length / girth.max(1e-4);
    }
    let mut branches = Vec::with_capacity(count);
    for s in 0..count {
        let segment = &segments[s];
        let module = &plant.modules[segment.module];
        let n = next[s];
        let t1 = if n != u32::MAX {
            t0[n as usize]
        } else {
            octahedral(segment.direction)
        };
        let end = end_diameter(s);
        let p = parent[s];
        let lateral = p != u32::MAX && next[p as usize] != s as u32;
        // Where a lateral leaves its parent's surface, found along its axis
        // by bisection against the parent's round cone.
        let exit = if lateral {
            let up = &segments[p as usize];
            let cone = Shape::round_cone(
                up.a.to_array(),
                up.diameter * 0.5,
                up.b.to_array(),
                end_diameter(p as usize) * 0.5,
            );
            let at = |t: f32| (segment.a + (segment.b - segment.a) * t).to_array();
            if cone.distance(at(1.0)) <= 0.0 {
                1.0
            } else {
                let (mut lo, mut hi) = (0.0f32, 1.0f32);
                for _ in 0..16 {
                    let mid = 0.5 * (lo + hi);
                    if cone.distance(at(mid)) < 0.0 {
                        lo = mid;
                    } else {
                        hi = mid;
                    }
                }
                hi
            }
        } else {
            0.0
        };
        let light = module.lit.clamp(0.0, 1.0);
        let light_start = if p == u32::MAX {
            light
        } else {
            plant.modules[segments[p as usize].module]
                .lit
                .clamp(0.0, 1.0)
        };
        branches.push(BranchInstance {
            a: [
                segment.a.x,
                segment.a.y,
                segment.a.z,
                segment.diameter * 0.5,
            ],
            b: [segment.b.x, segment.b.y, segment.b.z, end * 0.5],
            tangents: [t0[s], t1],
            v0: v[s][0],
            light,
            plant: index,
            flags: u32::from(lateral),
            light_start,
            v1: v[s][1],
            // Within the plant for now; `capture` offsets it.
            axis: axis[s],
            exit,
            pad: [0; 2],
        });
    }

    // Leaves.
    let mut leaves = Vec::new();
    let crowned = species.foliage == Foliage::Fronds;
    if species.leaf_size > 0.0 && !crowned {
        let (foliage_years, twig) = if species.foliage == Foliage::Needles {
            (NEEDLE_YEARS, NEEDLE_TWIG_DIAMETERS)
        } else {
            (FOLIAGE_YEARS, TWIG_DIAMETERS)
        };
        let twig = twig * species.tip_diameter;
        for segment in &segments {
            let module = &plant.modules[segment.module];
            let prototype = prototypes.get(module.prototype);
            let n = segment.node;
            let Some(offset) = module.node_offset else {
                continue;
            };
            let tip = plant.nodes[offset.index() + n];
            let opacity = foliage::leaf_fade(
                plant,
                module,
                prototype,
                n,
                &tip,
                foliage::LeafRetention {
                    years: foliage_years,
                    twig_diameter: Metres::from_metres(twig),
                },
                species,
            );
            // Pack semantic opacity into native shader-instance factors.
            let (tipness, fade) = (opacity.tip.ratio(), opacity.total.ratio());
            if fade < 0.02 {
                continue;
            }
            // Keyed on the module's lasting id, not its index, so a leaf
            // keeps its look when shedding shifts the modules around it.
            let hash = (plant.id.word() as u32)
                .wrapping_mul(2_654_435_761)
                .wrapping_add(module.id.word().wrapping_mul(40_503))
                .wrapping_add(n as u32 * 97);
            leaves.push(LeafInstance {
                center: tip.position.to_array(),
                // Leaves open with their shoot, from nothing: with the age
                // of the segment they grow on, which may emerge well after
                // its module began. Behind the tip they are a little smaller.
                size: species.leaf_size
                    * smoothstep(0.0, 0.3, tip.age.stage())
                    * (0.5 + 0.5 * tip.age.stage().min(1.0))
                    * (0.8 + 0.2 * tipness)
                    * fade,
                axis: segment.direction.to_array(),
                light: module.lit.clamp(0.0, 1.0),
                crown: [0.0; 4],
                species: plant.species.word() as u32,
                seed: (hash % 10_000) as f32 / 10_000.0,
                plant: index,
                pad: 0,
            });
        }
    }
    if crowned && species.leaf_size > 0.0 {
        palm_crown(plant, index, species, prototypes, &mut leaves);
    }

    // The crown: the clusters' centroid, and a radius that holds most of
    // them.
    if !leaves.is_empty() {
        let n = leaves.len() as f32;
        let mut middle = [0.0f32; 3];
        for leaf in &leaves {
            for (m, c) in middle.iter_mut().zip(leaf.center) {
                *m += c / n;
            }
        }
        let spread = leaves
            .iter()
            .map(|l| {
                (0..3)
                    .map(|k| (l.center[k] - middle[k]).powi(2))
                    .sum::<f32>()
            })
            .sum::<f32>()
            / n;
        let radius = (spread.sqrt() * 1.3).max(species.leaf_size);
        for leaf in &mut leaves {
            leaf.crown = [middle[0], middle[1], middle[2], radius];
        }
    }

    // Bounds: everything drawn, leaves' reach included.
    let bounds = measurements::plant_bounds(plant, species);
    (branches, leaves, bounds)
}

/// A vine's internodes as branch segments -- each shoot an axis, running on
/// without a seam where it carries on -- its leaves as clusters, standing
/// out from what it holds, and its bounds. `kind` is its species' place in
/// the snapshot's species.
fn vine_instances(
    vine: &Vine,
    index: u32,
    kind: u32,
    species: &VineType,
) -> (Vec<BranchInstance>, Vec<LeafInstance>, PlantInstance) {
    let nodes = &vine.nodes;
    let n = nodes.len();
    // Each node's carrying-on child: the one not starting a shoot.
    let mut next = vec![u32::MAX; n];
    for (i, node) in nodes.iter().enumerate().skip(1) {
        if !node.lateral && node.parent != NONE {
            next[node.parent as usize] = i as u32;
        }
    }
    let drawn = |i: usize| i > 0 && nodes[i].parent != NONE && nodes[i].length > 1e-4;
    let direction = |i: usize| {
        let d = nodes[i].position - nodes[nodes[i].parent as usize].position;
        d / d.length().max(1e-9)
    };
    let continues = |i: usize| !nodes[i].lateral && drawn(nodes[i].parent as usize);
    let tangent_in = |i: usize| {
        if continues(i) {
            (direction(nodes[i].parent as usize) + direction(i)).normalize()
        } else {
            direction(i)
        }
    };
    let mut v = vec![[0.0f32; 2]; n];
    let mut axis = vec![0u32; n];
    let mut axes = 0;
    let mut branches = Vec::with_capacity(n);
    for i in 0..n {
        if !drawn(i) {
            continue;
        }
        let node = &nodes[i];
        let parent = node.parent as usize;
        if continues(i) {
            axis[i] = axis[parent];
            v[i][0] = v[parent][1];
        } else {
            axis[i] = axes;
            axes += 1;
        }
        let carries_on = next[i] != u32::MAX && drawn(next[i] as usize);
        let end = if carries_on {
            nodes[next[i] as usize].diameter
        } else {
            node.diameter * 0.7
        };
        let girth = std::f32::consts::PI * 0.5 * (node.diameter + end);
        v[i][1] = v[i][0] + node.length / girth.max(1e-4);
        let t1 = if carries_on {
            tangent_in(next[i] as usize)
        } else {
            direction(i)
        };
        let a = nodes[parent].position;
        let b = node.position;
        branches.push(BranchInstance {
            a: [a.x, a.y, a.z, node.diameter * 0.5],
            b: [b.x, b.y, b.z, end * 0.5],
            tangents: [octahedral(tangent_in(i)), octahedral(t1)],
            v0: v[i][0],
            light: node.lit.clamp(0.0, 1.0),
            plant: index,
            flags: u32::from(node.lateral && drawn(parent)),
            light_start: nodes[parent].lit.clamp(0.0, 1.0),
            v1: v[i][1],
            axis: axis[i],
            exit: 0.0,
            pad: [0; 2],
        });
    }

    let up = Vec3::new(0.0, 1.0, 0.0);
    let mut leaves = Vec::new();
    for (i, node) in nodes.iter().enumerate().skip(1) {
        let leafy = vine.leafiness(node, species);
        if leafy < 0.02 {
            continue;
        }
        let size = species.leaf_size * leafy;
        // Held, the leaves stand out from the surface and turn up to the
        // sky; hanging, they follow the stem.
        let axis = if node.hold != Hold::Free {
            (node.normal + up * 0.6).normalize()
        } else {
            direction(i)
        };
        let center = node.position + node.normal * (size * 0.3);
        let behind = center - axis * size;
        let hash = (vine.id.word() as u32)
            .wrapping_mul(2_654_435_761)
            .wrapping_add((i as u32).wrapping_mul(40_503));
        leaves.push(LeafInstance {
            center: center.to_array(),
            size,
            axis: axis.to_array(),
            light: node.lit.clamp(0.0, 1.0),
            // A crown no deeper than the cluster: a vine's leaves are all on
            // its surface.
            crown: [behind.x, behind.y, behind.z, size],
            species: kind,
            seed: (hash % 10_000) as f32 / 10_000.0,
            plant: index,
            pad: 0,
        });
    }

    let mut lo = vine.position;
    let mut hi = vine.position;
    for node in nodes {
        lo = Vec3::new(
            lo.x.min(node.position.x),
            lo.y.min(node.position.y),
            lo.z.min(node.position.z),
        );
        hi = Vec3::new(
            hi.x.max(node.position.x),
            hi.y.max(node.position.y),
            hi.z.max(node.position.z),
        );
    }
    let center = (lo + hi) * 0.5;
    let bounds = PlantInstance {
        sphere: [
            center.x,
            center.y,
            center.z,
            (hi - lo).length() * 0.5 + species.leaf_size * 1.5,
        ],
        species: kind,
        pad: [0; 3],
    };
    (branches, leaves, bounds)
}

/// The buildings, as the renderer draws them.
fn structure_geometry(ecosystem: &Ecosystem) -> StructureGeometry {
    let Some(obstacles) = &ecosystem.obstacles else {
        return StructureGeometry::default();
    };
    let mesh = &obstacles.structure.mesh;
    StructureGeometry {
        vertices: (0..mesh.positions.len())
            .map(|i| StructureVertex {
                position: mesh.positions[i],
                normal: mesh.normals[i],
                color: mesh.colors[i],
                material: mesh.materials[i] as f32,
            })
            .collect(),
        indices: mesh.indices.clone(),
    }
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// A palm's fronds: the last [`Plant::crown_size`] of the leaves its tip
/// has put out ([`Plant::crown_leaves`]), each keeping its place in the
/// spiral as it ages -- unfolding upright out of the middle, sinking lower
/// down the crown shaft and drooping below the horizontal, and falling as
/// the next one opens.
fn palm_crown(
    plant: &Plant,
    index: u32,
    species: &PlantType,
    prototypes: &PrototypeLibrary,
    leaves: &mut Vec<LeafInstance>,
) {
    const GOLDEN: f32 = 2.399_963;
    // The top of the stem, the way it grows there, and its light.
    let mut apex: Option<(Vec3, Vec3, f32)> = None;
    for module in plant.modules.iter().filter(|m| m.order == 0) {
        let prototype = prototypes.get(module.prototype);
        let Some(offset) = module.node_offset else {
            continue;
        };
        let offset = offset.index();
        if offset + prototype.node_count() > plant.nodes.len() {
            continue;
        }
        for (n, node) in prototype.nodes.iter().enumerate().skip(1) {
            let a = plant.nodes[offset + node.parent as usize].position;
            let b = plant.nodes[offset + n].position;
            if (b - a).length() > 1e-4 && apex.is_none_or(|(top, _, _)| b.y > top.y) {
                apex = Some((b, (b - a).normalize(), module.lit.clamp(0.0, 1.0)));
            }
        }
    }
    let Some((top, stem, light)) = apex else {
        return;
    };
    let height = (top.y - plant.position.y).max(0.0);
    let crown = plant.crown_size();
    let made = plant.crown_leaves;
    let size = species.leaf_size * (0.5 + 0.5 * (height / 5.0).min(1.0));
    let turn = ((plant.id.word() as u32).wrapping_mul(2_654_435_761) % 10_000) as f32;
    let first = (made - crown).floor().max(0.0) as u32;
    for leaf in first..=(made.floor() as u32) {
        // How far through its life the frond is, zero to one.
        let age = (made - leaf as f32) / crown;
        if !(0.0..=1.0).contains(&age) {
            continue;
        }
        let unfolded = smoothstep(0.0, 0.15, age) * (1.0 - smoothstep(0.85, 1.0, age));
        if unfolded < 0.02 {
            continue;
        }
        let azimuth = turn + leaf as f32 * GOLDEN;
        let out = Vec3::new(azimuth.cos(), 0.0, azimuth.sin());
        let elevation = (72.0 - 110.0 * age).to_radians();
        let axis = out * elevation.cos() + Vec3::new(0.0, elevation.sin(), 0.0);
        let at = top - stem * (0.9 * age);
        leaves.push(LeafInstance {
            center: at.to_array(),
            size: size * unfolded,
            axis: axis.to_array(),
            light,
            crown: [0.0; 4],
            species: plant.species.word() as u32,
            seed: (leaf.wrapping_mul(7_919) % 10_000) as f32 / 10_000.0,
            plant: index,
            pad: 0,
        });
    }
}

fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// The ground, coloured by what grows there: dry or lush by precipitation,
/// paler where it is cold, rock on steep slopes, and -- where nothing may
/// grow -- water low down, rock or snow high up.
fn terrain_geometry(ecosystem: &Ecosystem) -> TerrainGeometry {
    let terrain = &ecosystem.terrain;
    // A level terrain comes as two triangles; resample it so the vertex
    // colours and the fog have something to work with.
    let source = if terrain.resolution[0] * terrain.resolution[1] < 64 * 64 {
        crate::Terrain::from_fn(terrain.origin, terrain.size, [64, 64], |x, z| {
            terrain.height_at(x, z)
        })
    } else {
        terrain.clone()
    };
    let mesh = source.mesh();
    let (low, high) = terrain.height_range();
    let climate = &ecosystem.climate;
    let wetness = (climate.precipitation.millimetres() / 1500.0).clamp(0.0, 1.0);

    let dry = [0.55, 0.47, 0.32];
    let lush = [0.27, 0.36, 0.15];
    // Where it is cold the ground is moss, peat and lichen, dark and
    // olive-brown -- the boreal floor, and the tundra's.
    let tundra = [0.30, 0.27, 0.18];
    let rock = [0.42, 0.40, 0.38];
    let snow = [0.92, 0.94, 0.98];
    let water = [0.12, 0.26, 0.33];
    let trodden = [0.40, 0.35, 0.27];

    let vertices =
        mesh.positions
            .iter()
            .zip(&mesh.normals)
            .map(|(p, n)| {
                let h = p[1];
                let temperature = climate.temperature_at(Metres::from_metres(h)).degrees();
                // Round a building's foot the ground is trodden bare.
                let built = ecosystem.obstacles.as_ref().is_some_and(|o| {
                    o.distance(fabelgeist_math::Vec3::new(p[0], h + 0.3, p[2])) < 1.2
                });
                let color = if built {
                    trodden
                } else if !terrain.is_open(p[0], p[2]) {
                    if h < low + 0.35 * (high - low).max(1.0) {
                        water
                    } else if temperature < 3.0 {
                        snow
                    } else {
                        rock
                    }
                } else {
                    let ground = mix(dry, lush, wetness);
                    let ground = mix(ground, tundra, (8.0 - temperature) / 10.0);
                    mix(ground, rock, (0.82 - n[1]) / 0.2)
                };
                TerrainVertex {
                    position: *p,
                    normal: *n,
                    color,
                }
            })
            .collect();
    let mut vertices: Vec<TerrainVertex> = vertices;
    let mut indices = mesh.indices.unwrap_or_default();

    // An apron of plain ground to the horizon, a little below the terrain,
    // so a scene sits in a landscape instead of floating in the haze.
    let apron = mix(
        mix(dry, lush, wetness),
        tundra,
        (8.0 - climate.temperature_at(Metres::from_metres(low)).degrees()) / 10.0,
    );
    let [cx, cz] = [
        terrain.origin[0] + terrain.size[0] * 0.5,
        terrain.origin[1] + terrain.size[1] * 0.5,
    ];
    let reach = terrain.size[0].max(terrain.size[1]) * 20.0;
    let y = low - 0.05;
    let base = vertices.len() as u32;
    for (dx, dz) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
        vertices.push(TerrainVertex {
            position: [cx + dx * reach, y, cz + dz * reach],
            normal: [0.0, 1.0, 0.0],
            color: apron,
        });
    }
    indices.extend_from_slice(&[base, base + 3, base + 1, base + 1, base + 3, base + 2]);

    TerrainGeometry { vertices, indices }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::SpeciesId;
    use crate::units::GroundPoint;
    use crate::{Climate, Terrain};

    /// Leaves between one frame and the next: how large the ones that
    /// appeared were, and how large the ones that went, as fractions of
    /// the species' leaf size -- matched by look and place.
    fn popping(species: PlantType, years: f32) -> (f32, f32) {
        let size = species.leaf_size;
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
        let leaves = |e: &Ecosystem| capture(e, &Profile::DETAILED, 1, false, 0.0).leaves;
        let mut before = leaves(&ecosystem);
        let (mut appeared, mut went) = (0.0f32, 0.0f32);
        let near = |a: &LeafInstance, b: &LeafInstance| {
            a.seed == b.seed
                && (0..3)
                    .map(|k| (a.center[k] - b.center[k]).powi(2))
                    .sum::<f32>()
                    < 0.05 * 0.05
        };
        while ecosystem.time < Years::from_years(years) {
            let shed = pollster::block_on(ecosystem.advance(Years::from_years(1.0 / 30.0)))
                .unwrap()
                .is_some_and(|r| r.shed > 0);
            let after = leaves(&ecosystem);
            // A branch shed takes its leaves with it, all at once, and the
            // twig it hung from, feeding fewer shoots, may leaf again.
            if !shed {
                for leaf in &after {
                    if !before.iter().any(|b| near(b, leaf)) {
                        appeared = appeared.max(leaf.size / size);
                    }
                }
                for leaf in &before {
                    if !after.iter().any(|a| near(a, leaf)) {
                        went = went.max(leaf.size / size);
                    }
                }
            }
            before = after;
        }
        (appeared, went)
    }

    #[test]
    fn an_axis_runs_on_without_a_seam() {
        // Where a segment carries on into the next, the two share the ring
        // between them exactly -- centre, radius, tangent, and the bark's
        // `v` -- so neither the tube nor its pattern breaks there.
        let species = PlantType {
            seeding_frequency: 0.0,
            ..PlantType::baobab()
        };
        let climate = Climate::new(species.temperature_optimum, species.precipitation_optimum);
        let mut ecosystem = Ecosystem::new(Terrain::flat(40.0, 40.0), climate, vec![species]);
        ecosystem
            .plant(
                SpeciesId::from_word(0),
                GroundPoint::new(Metres::from_metres(0.0), Metres::from_metres(0.0)),
            )
            .unwrap();
        while ecosystem.time < Years::from_years(100.0) {
            pollster::block_on(ecosystem.step()).unwrap();
        }
        let branches = capture(&ecosystem, &Profile::DETAILED, 1, false, 0.0).branches;
        let mut joints = 0;
        for x in &branches {
            for y in branches.iter().filter(|y| y.flags & 1 == 0) {
                if x.b[..3] == y.a[..3] {
                    assert_eq!(x.b[3], y.a[3], "radius");
                    assert_eq!(x.tangents[1], y.tangents[0], "tangent");
                    assert_eq!(x.v1, y.v0, "bark");
                    joints += 1;
                }
            }
        }
        assert!(joints > 100, "{joints} joints");
    }

    #[test]
    fn the_fast_profile_packs_the_plants_alone() {
        // A grove, so the hierarchy has nodes above the plants.
        let species = PlantType {
            seeding_frequency: 0.0,
            ..PlantType::conifer()
        };
        let climate = Climate::new(species.temperature_optimum, species.precipitation_optimum);
        let mut ecosystem = Ecosystem::new(Terrain::flat(40.0, 40.0), climate, vec![species]);
        for i in 0..4 {
            ecosystem
                .plant(
                    SpeciesId::from_word(0),
                    GroundPoint::new(
                        Metres::from_metres(i as f32 * 6.0 - 9.0),
                        Metres::from_metres(0.0),
                    ),
                )
                .unwrap();
        }
        while ecosystem.time < Years::from_years(30.0) {
            pollster::block_on(ecosystem.step()).unwrap();
        }
        let fast = capture(&ecosystem, &Profile::FAST, 1, false, 0.0);
        let detailed = capture(&ecosystem, &Profile::DETAILED, 1, false, 0.0);
        // No proxies, no fillets: the plants' instances, and nothing to
        // blend.
        assert!(!fast.fillets && fast.blend.shapes.is_empty());
        assert!(fast.branches.len() < detailed.branches.len());
        let plants: Vec<_> = fast
            .hierarchy
            .nodes
            .iter()
            .filter(|n| n.error == 0.0)
            .collect();
        assert_eq!(plants.len(), 4);
        assert!(
            fast.hierarchy
                .nodes
                .iter()
                .all(|n| n.error == 0.0 || n.error == f32::INFINITY)
        );
        // Either way, the same litter under the same crowns.
        assert_eq!(fast.litter.cells, detailed.litter.cells);
        let fallen: f32 = fast.litter.cells.iter().map(|c| c[0]).sum();
        let cell = fast.litter.cell * fast.litter.cell;
        let leaves: f32 = fast.leaves.iter().map(|l| l.size * l.size).sum();
        // Spread over the ground, the needles' area is the needles' area --
        // to the grid's resolution.
        assert!(
            (fallen * cell / leaves - 1.0).abs() < 0.2,
            "{} of {leaves}",
            fallen * cell
        );
    }

    #[test]
    fn leaves_come_and_go_gradually() {
        for species in [PlantType::deciduous(), PlantType::palm()] {
            let name = species.name.clone();
            let (appeared, went) = popping(species, 20.0);
            assert!(
                appeared < 0.1,
                "{name}: a leaf appeared at {appeared} of full size"
            );
            assert!(went < 0.1, "{name}: a leaf vanished at {went} of full size");
        }
    }
}

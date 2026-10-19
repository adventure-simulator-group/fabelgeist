//! Offline growth and fresh prepared-asset loading are separate arrival paths.

use std::{path::PathBuf, sync::Arc, time::Instant};

use adventuresim_environment::{
    prepared::{
        self, ForestFrame, PreparedUnique, SharedForest, TreeBank, TreeFocus, TreeRoot,
        VariantBudget,
    },
    render::{ForestDetail, ForestPart},
};
use anyhow::{Context, Result, ensure};
use fabelgeist_botany::{
    Ecosystem,
    scenes::Scene,
    viewer::{profile::Profile, sim},
};
use fabelgeist_gpu::globals::WgpuContext;
use fabelgeist_math::units::Metres;

use crate::benchmark::Preparation;
use adventuresim_environment::render::benchmark::ForestInstances;

pub struct Growth {
    pub scene: Scene,
    pub ecosystem: Option<Ecosystem>,
    pub years: f32,
    pub profile: Profile,
}

pub enum Source {
    Growth(Box<Growth>),
    Prepared {
        directory: PathBuf,
        representation: Representation,
    },
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum Representation {
    Unique,
    Shared,
}

pub enum FrozenForest {
    Unique(Arc<sim::Snapshot>),
    Shared(Arc<SharedForest>),
}

pub struct Loaded {
    pub forest: FrozenForest,
    pub frame: ForestFrame,
    pub preparation: Preparation,
}

impl FrozenForest {
    pub fn expected_instances(&self) -> ForestInstances {
        match self {
            Self::Unique(snapshot) => ForestInstances {
                branches: snapshot.branches.len(),
                leaves: snapshot.leaves.len(),
            },
            Self::Shared(forest) => forest.expected_instances(),
        }
    }
}

pub fn load(source: Source) -> Result<Loaded> {
    match source {
        Source::Growth(growth) => {
            let (unique, preparation) = grow(*growth)?;
            Ok(Loaded {
                frame: ForestFrame::from_unique(&unique),
                forest: FrozenForest::Unique(Arc::new(unique.snapshot)),
                preparation,
            })
        }
        Source::Prepared {
            directory,
            representation,
        } => load_prepared(directory, representation),
    }
}

pub fn prepare(growth: Growth, directory: PathBuf, budget: VariantBudget) -> Result<()> {
    ensure!(
        !directory.exists(),
        "prepared output already exists: {}",
        directory.display()
    );
    let (unique, _) = grow(growth)?;
    let library = unique.bake(budget)?;
    let unique_bytes = prepared::encode_unique(&unique)?;
    let (library_bytes, id) = prepared::encode_library(&library)?;
    let population = unique.population(&library, id)?;
    let population_bytes = prepared::encode_population(&population, &library, id)?;
    std::fs::create_dir_all(&directory)?;
    std::fs::write(directory.join("unique.forest"), &unique_bytes)?;
    std::fs::write(directory.join("library.forest"), &library_bytes)?;
    std::fs::write(directory.join("population.forest"), &population_bytes)?;
    println!(
        "prepared {} models / {} placements; unique {} bytes, library {} bytes ({} mesh bytes), population {} bytes",
        library.models.len(),
        population.placements.len(),
        unique_bytes.len(),
        library_bytes.len(),
        library.mesh_bytes(),
        population_bytes.len()
    );
    Ok(())
}

fn load_prepared(directory: PathBuf, representation: Representation) -> Result<Loaded> {
    let started = Instant::now();
    let names: &[&str] = match representation {
        Representation::Unique => &["unique.forest"],
        Representation::Shared => &["library.forest", "population.forest"],
    };
    let bytes = names
        .iter()
        .map(|name| {
            let path = directory.join(name);
            std::fs::read(&path).with_context(|| format!("read prepared asset {}", path.display()))
        })
        .collect::<Result<Vec<_>>>()?;
    let asset_load = started.elapsed();
    let mut preparation = Preparation {
        asset_load,
        asset_bytes: bytes.iter().map(Vec::len).sum(),
        ..Default::default()
    };
    let started = Instant::now();
    let (forest, frame) = match representation {
        Representation::Unique => {
            let unique = prepared::decode_unique(&bytes[0])?;
            let frame = ForestFrame::from_unique(&unique);
            (FrozenForest::Unique(Arc::new(unique.snapshot)), frame)
        }
        Representation::Shared => {
            let bank = Arc::new(TreeBank::from_bytes(&bytes[0])?);
            let forest = SharedForest::from_population_bytes(bank, &bytes[1])?;
            preparation.shared_counts(&forest)?;
            let frame = forest.population().frame.clone();
            (FrozenForest::Shared(Arc::new(forest)), frame)
        }
    };
    preparation.asset_decode = started.elapsed();
    Ok(Loaded {
        forest,
        frame,
        preparation,
    })
}

/// Read a fresh population while retaining the admitted CPU bank in both GPU policies.
pub fn load_population(bank: Arc<TreeBank>, path: &std::path::Path) -> Result<Loaded> {
    let started = Instant::now();
    let bytes =
        std::fs::read(path).with_context(|| format!("read population {}", path.display()))?;
    let asset_load = started.elapsed();
    let started = Instant::now();
    let forest = SharedForest::from_population_bytes(bank, &bytes)?;
    let mut preparation = Preparation {
        asset_load,
        asset_bytes: bytes.len(),
        ..Default::default()
    };
    preparation.shared_counts(&forest)?;
    preparation.asset_decode = started.elapsed();
    Ok(Loaded {
        frame: forest.population().frame.clone(),
        forest: FrozenForest::Shared(Arc::new(forest)),
        preparation,
    })
}

impl Preparation {
    fn shared_counts(&mut self, forest: &SharedForest) -> Result<()> {
        let library = forest.bank().library();
        self.mesh_bytes = library.mesh_bytes();
        self.library_models = library.models.len();
        self.grid_classes = library.grids.len();
        self.library_branches = library.branches.len();
        self.library_leaves = library.leaves.len();
        for placement in &forest.population().placements {
            let model = &library.models[placement.variant.index()];
            self.placed_branches += model.branches.count as usize;
            self.placed_leaves += model.leaves.count as usize;
            self.branch_slots +=
                prepared::MeshClass::for_records(ForestPart::Branches, model.branches.count)?
                    .map_or(0, |class| class.capacity.records() as usize);
            self.leaf_slots +=
                prepared::MeshClass::for_records(ForestPart::Leaves, model.leaves.count)?
                    .map_or(0, |class| class.capacity.records() as usize);
        }
        Ok(())
    }
}

fn grow(mut growth: Growth) -> Result<(PreparedUnique, Preparation)> {
    ensure!(
        growth.years.is_finite() && growth.years >= 0.0,
        "growth years must be finite and nonnegative"
    );
    let scene_started = Instant::now();
    let ecosystem = growth.ecosystem.take().unwrap_or_else(|| {
        growth
            .scene
            .build(fabelgeist_determinism::Seed::from_u64(1))
    });
    let scene = scene_started.elapsed();
    let started = Instant::now();
    let context = pollster::block_on(WgpuContext::new())?;
    let mut ecosystem = ecosystem.with_gpu(context)?;
    let gpu_setup = started.elapsed();
    ecosystem.settings.dt = growth.profile.step_years;
    let steps = (growth.years / ecosystem.settings.dt.duration().years()).ceil() as usize;
    let started = Instant::now();
    for _ in 0..steps {
        pollster::block_on(ecosystem.step())?;
    }
    let elapsed = started.elapsed();
    let started = Instant::now();
    let snapshot = sim::capture(&ecosystem, &growth.profile, 1, true, 0.0);
    let roots = ecosystem
        .plants
        .iter()
        .map(|p| TreeRoot {
            id: p.id,
            position: [p.position.x, p.position.y, p.position.z].map(Metres::from_metres),
        })
        .collect();
    let focus = ecosystem
        .plants
        .iter()
        .max_by(|a, b| a.height().total_cmp(&b.height()))
        .map(|tallest| TreeFocus {
            root: [tallest.position.x, tallest.position.y, tallest.position.z]
                .map(Metres::from_metres),
            height: Metres::from_metres(tallest.height()),
        });
    let p = growth.profile;
    let detail = ForestDetail {
        tube_sides: p.tube_sides,
        tube_spans: p.tube_spans,
        leaf_blades: p.leaf_blades,
        full_pixels: p.full_pixels,
        middle_pixels: p.middle_pixels,
        ..Default::default()
    };
    Ok((
        PreparedUnique {
            snapshot,
            roots,
            detail,
            label: growth.scene.name().into(),
            focus,
        },
        Preparation {
            scene,
            gpu_setup,
            growth: elapsed,
            packing: started.elapsed(),
            ..Default::default()
        },
    ))
}

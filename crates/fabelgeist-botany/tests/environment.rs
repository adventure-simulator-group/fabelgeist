//! The GPU environment passes against the host ones, and whole ecosystems.

use fabelgeist_botany::environment::gpu::GpuEnvironment;
use fabelgeist_botany::environment::host::HostEnvironment;
use fabelgeist_botany::environment::{OrientRequest, SphereSet};
use fabelgeist_botany::rng::Rng;
use fabelgeist_botany::{
    Climate, Ecosystem, EcosystemSettings, EnvironmentSettings, PlantType, Terrain,
};
use fabelgeist_gpu::globals::WgpuContext;

const NONE: u32 = u32::MAX;

fn scattered_spheres(count: usize, seed: u64) -> SphereSet {
    let mut rng = Rng::new(seed);
    let mut set = SphereSet::default();
    for i in 0..count {
        set.spheres.push([
            rng.range(-10.0, 10.0),
            rng.range(0.0, 12.0),
            rng.range(-10.0, 10.0),
            rng.range(0.05, 1.2),
        ]);
        // A chain every few spheres, so parent exclusion is exercised.
        set.parents
            .push(if i % 4 == 0 { NONE } else { i as u32 - 1 });
    }
    set
}

#[tokio::test]
async fn gpu_light_matches_host_light() -> anyhow::Result<()> {
    let context = WgpuContext::new().await?;
    let mut gpu = GpuEnvironment::new(context)?;
    let mut host = HostEnvironment::default();
    let settings = EnvironmentSettings::default();

    let set = scattered_spheres(3000, 5);
    let ecosystem = Ecosystem::new(Terrain::flat(24.0, 24.0), Climate::default(), vec![]);
    let shadow = ecosystem.shadow_grid();

    let a = gpu.light(&set, Some(&shadow), &settings).await?;
    let b = host.light(&set, Some(&shadow), &settings);
    assert_eq!(a.len(), b.len());
    let mut shaded = 0;
    for (i, (x, y)) in a.iter().zip(&b).enumerate() {
        assert!(
            (x[0] - y[0]).abs() < 1e-3,
            "Q[{i}]: gpu {} host {}",
            x[0],
            y[0]
        );
        assert!(
            (x[1] - y[1]).abs() < 1e-3,
            "Q_G[{i}]: gpu {} host {}",
            x[1],
            y[1]
        );
        shaded += usize::from(y[0] < 0.99);
    }
    assert!(shaded > 100, "the scene should have overlaps ({shaded})");
    Ok(())
}

/// Layers too large for the one-dispatch propagation go a dispatch per
/// layer; that path must agree too.
#[tokio::test]
async fn gpu_shadow_on_a_large_grid_matches_host() -> anyhow::Result<()> {
    let context = WgpuContext::new().await?;
    let mut gpu = GpuEnvironment::new(context)?;
    let mut host = HostEnvironment::default();
    let settings = EnvironmentSettings::default();

    let set = scattered_spheres(2000, 11);
    let ecosystem = Ecosystem::new(Terrain::flat(300.0, 300.0), Climate::default(), vec![]);
    let shadow = ecosystem.shadow_grid();
    assert!(shadow.dims[0] * shadow.dims[2] > 1 << 16);

    let a = gpu.light(&set, Some(&shadow), &settings).await?;
    let b = host.light(&set, Some(&shadow), &settings);
    for (i, (x, y)) in a.iter().zip(&b).enumerate() {
        assert!(
            (x[1] - y[1]).abs() < 1e-3,
            "Q_G[{i}]: gpu {} host {}",
            x[1],
            y[1]
        );
    }
    assert!(b.iter().any(|l| l[1] < 0.95), "something should be shaded");
    Ok(())
}

#[tokio::test]
async fn gpu_orientation_matches_host_orientation() -> anyhow::Result<()> {
    orientation_parity(scattered_spheres(2000, 9), false).await
}

/// Solids among the modules: every shape, turned, crowding the requests
/// that come near them.
#[tokio::test]
async fn gpu_orientation_against_solids_matches_host() -> anyhow::Result<()> {
    use fabelgeist_buildings::{Material, Structure};
    let mut structure = Structure::new();
    structure.block(
        [-4.0, 0.0, -3.0],
        [3.0, 1.0],
        9.0,
        0.4,
        Material::Stone,
        1.0,
    );
    structure.drum([5.0, 0.0, 4.0], 2.5, 10.0, Material::Stone, 1.0);
    structure.cone([5.0, 10.0, 4.0], 2.8, 3.0, Material::Slate, 1.0);
    structure.gable(
        [-4.0, 9.0, -3.0],
        3.3,
        1.3,
        1.5,
        0.4,
        Material::Tile,
        Material::Plaster,
        1.0,
    );
    structure.pyramid([3.0, 0.0, -6.0], 2.0, 6.0, 1.0, Material::Slate, 1.0);
    let mut set = scattered_spheres(2000, 9);
    set.obstacles = Some(fabelgeist_botany::Obstacles::new(structure));
    orientation_parity(set, true).await
}

async fn orientation_parity(set: SphereSet, obstacles: bool) -> anyhow::Result<()> {
    let context = WgpuContext::new().await?;
    let mut gpu = GpuEnvironment::new(context)?;
    let mut host = HostEnvironment::default();
    let settings = EnvironmentSettings::default();

    let mut rng = Rng::new(3);
    let requests: Vec<OrientRequest> = (0..200u32)
        .map(|k| {
            let index = k * 7;
            let s = set.spheres[index as usize];
            let q = fabelgeist_math::Vec4::from_axis_angle(
                fabelgeist_math::Vec3::new(rng.range(-1.0, 1.0), 1.0, rng.range(-1.0, 1.0)),
                rng.range(0.0, 1.0),
            );
            OrientRequest {
                root: [s[0], s[1] - 0.5, s[2]],
                radius: s[3],
                local_center: [0.0, 0.5, 0.0],
                index,
                rotation: [q.x, q.y, q.z, q.w],
                axis: [0.0, 1.0, 0.0],
                cos_target: 0.5f32.cos(),
                collision_weight: 1.0,
                tropism_weight: 0.5,
                parent: set.parents[index as usize],
                obstacles: u32::from(obstacles),
            }
        })
        .collect();

    let a = gpu.orient(&set, &requests, &settings).await?;
    let b = host.orient(&set, &requests, &settings);
    let mut moved = 0;
    for (i, ((x, y), r)) in a.iter().zip(&b).zip(&requests).enumerate() {
        // Same pattern search: the same rotation up to float noise, or at
        // worst a tie broken the other way.
        assert!(
            (x.crowding - y.crowding).abs() < 1e-3,
            "request {i}: crowding gpu {} host {}",
            x.crowding,
            y.crowding
        );
        let (x, y) = (x.rotation, y.rotation);
        let dot = (x.x * y.x + x.y * y.y + x.z * y.z + x.w * y.w).abs();
        assert!(dot > 0.999, "request {i}: gpu {x:?} host {y:?}");
        let start = r.rotation;
        let d0 = (y.x * start[0] + y.y * start[1] + y.z * start[2] + y.w * start[3]).abs();
        moved += usize::from(d0 < 0.9999);
    }
    assert!(moved > 50, "the search should move most modules ({moved})");
    if obstacles {
        let solids = set.obstacles.as_ref().expect("solids");
        let near = requests
            .iter()
            .filter(|r| solids.distance(fabelgeist_math::Vec3::from_array(r.root)) < r.radius + 0.5)
            .count();
        assert!(near > 5, "only {near} requests came near a solid");
    }
    Ok(())
}

fn forest(settings: EcosystemSettings) -> Ecosystem {
    let mut ecosystem = Ecosystem::new(
        Terrain::flat(30.0, 30.0),
        Climate::new(12.0, 672.0),
        vec![
            PlantType::conifer(),
            PlantType::deciduous(),
            PlantType::shrub(),
        ],
    )
    .with_settings(settings);
    ecosystem.scatter(12);
    ecosystem
}

#[tokio::test]
async fn a_forest_grows_on_the_host() -> anyhow::Result<()> {
    let mut ecosystem = forest(EcosystemSettings::default());
    for _ in 0..60 {
        ecosystem.step().await?;
    }
    let census = ecosystem.census();
    assert!(census.plants > 0);
    assert!(census.modules > 100, "{census:?}");
    assert!(census.max_height > 3.0, "{census:?}");
    let meshes = ecosystem.branch_meshes(5);
    for mesh in &meshes {
        mesh.validate()?;
    }
    Ok(())
}

#[tokio::test]
async fn a_forest_grows_on_the_gpu() -> anyhow::Result<()> {
    let context = WgpuContext::new().await?;
    let mut ecosystem = forest(EcosystemSettings::default()).with_gpu(context.clone())?;
    let mut last = None;
    for _ in 0..60 {
        last = Some(ecosystem.step().await?);
    }
    let report = last.unwrap();
    let census = ecosystem.census();
    assert!(census.modules > 100, "{census:?}");
    // Overlap stays bounded in a young stand.
    assert!(report.overlap_ratio < 0.5, "{report:?}");

    let mut mesher = fabelgeist_botany::render::BranchMesher::new(6);
    let species = mesher.mesh(&context, &ecosystem)?;
    let segments: usize = ecosystem.branch_segments().iter().map(Vec::len).sum();
    let swept: u32 = species
        .iter()
        .flat_map(|s| &s.meshes)
        .map(|m| fabelgeist_botany::render::segment_count(m, 6))
        .sum();
    assert_eq!(swept as usize, segments);
    Ok(())
}

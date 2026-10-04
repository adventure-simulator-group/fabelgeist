//! The pattern-to-cloth adapter, checked against real patterns from the
//! bundled presets rather than against a hand-built toy.

use fabelgeist_garment_code::assets;
use fabelgeist_garment_code::programs::MetaGarment;
use fabelgeist_garment_code::{Body, Design};

use super::*;

/// Build the first bundled design on the first bundled body -- a real garment,
/// with curved edges, many panels and dozens of stitches.
pub(super) fn preset_pattern() -> PatternSpec {
    let body = Body::from_yaml_str(assets::BODIES[0].yaml).expect("the bundled body parses");
    let design = Design::from_yaml_str(assets::DESIGNS[0].yaml).expect("the bundled design parses");
    let garment = MetaGarment::new("test", &body, &design);
    garment.assembly()
}

#[test]
fn samples_every_panel_outline() {
    let spec = preset_pattern();
    assert!(!spec.panels.is_empty(), "the preset produced no panels");

    let segments = stitched_segments(&spec);
    for (index, (name, panel)) in spec.panels.iter().enumerate() {
        let (outline, edge_starts) = sample_outline(panel, &segments[index]);
        assert_eq!(
            edge_starts.len(),
            panel.edges.len(),
            "panel {name}: an edge produced no entry"
        );
        assert!(
            outline.len() >= panel.edges.len(),
            "panel {name}: fewer outline points than edges"
        );
        assert!(
            outline.iter().all(|p| p.is_finite()),
            "panel {name}: a non-finite outline point"
        );

        // Starts must be strictly increasing, or two pattern edges would claim
        // the same outline edges and the seams would overlap.
        for pair in edge_starts.windows(2) {
            assert!(
                pair[1] > pair[0],
                "panel {name}: edge starts {edge_starts:?} are not increasing"
            );
        }

        // A curved edge has to gain interior samples; a straight one must not,
        // unless what it is sewn to is finer than it -- see
        // `stitched_segments`, which is what raises it.
        let own = curve_segments(panel);
        for (edge_index, edge) in panel.edges.iter().enumerate() {
            let span = outline_edges(&edge_starts, outline.len(), edge_index).len();
            assert_eq!(
                span, segments[index][edge_index],
                "panel {name} edge {edge_index}: sampled {span} runs, not the {} asked for",
                segments[index][edge_index]
            );
            if edge.curvature.is_none() && span != 1 {
                assert!(
                    span > own[edge_index],
                    "panel {name} edge {edge_index}: a straight edge split for no reason"
                );
            }
        }
    }
}

/// The two sides of a stitch must be cut into the same number of runs.
///
/// A seam joins one outline edge to one outline edge along the whole of both.
/// If the runs differ in length the shorter side's edges are re-used, and the
/// whole of each is sewn to several pieces of the other -- which gathers it
/// into a bunch rather than sewing it flat. A circle skirt is where it shows:
/// its waist arc samples into twenty-two runs, and the straight waistband edge
/// it is sewn to used to stay at one.
#[test]
fn both_sides_of_a_stitch_are_cut_the_same_way() {
    for bottom in [
        "SkirtCircle",
        "AsymmSkirtCircle",
        "GodetSkirt",
        "Pants",
        "Skirt2",
        "SkirtManyPanels",
        "PencilSkirt",
        "SkirtLevels",
    ] {
        let spec = outfit(bottom);
        let segments = stitched_segments(&spec);
        let index: std::collections::HashMap<&str, usize> = spec
            .panels
            .iter()
            .enumerate()
            .map(|(i, (name, _))| (name.as_str(), i))
            .collect();

        let mut checked = 0;
        for stitch in &spec.stitches {
            let (Some(&a), Some(&b)) = (
                index.get(stitch.sides[0].panel.as_str()),
                index.get(stitch.sides[1].panel.as_str()),
            ) else {
                continue;
            };
            let (a_edge, b_edge) = (stitch.sides[0].edge, stitch.sides[1].edge);
            if a_edge >= segments[a].len() || b_edge >= segments[b].len() {
                continue;
            }
            assert_eq!(
                segments[a][a_edge],
                segments[b][b_edge],
                "{bottom}: {}[{a_edge}] is cut into {} runs but {}[{b_edge}] into {}",
                stitch.sides[0].panel,
                segments[a][a_edge],
                stitch.sides[1].panel,
                segments[b][b_edge],
            );
            checked += 1;
        }
        assert!(checked > 0, "{bottom}: no stitch was checked");
    }
}

/// A shirt, a waistband and the named bottom, on the first bundled body.
fn outfit(bottom: &str) -> PatternSpec {
    use fabelgeist_garment_code::design::Value;

    let body = Body::from_yaml_str(assets::BODIES[0].yaml).expect("the bundled body parses");
    let design = Design::from_yaml_str(assets::DESIGNS[0].yaml).expect("the bundled design parses");
    design.set_v("meta.upper", Value::Str("Shirt".into()));
    design.set_v("meta.wb", Value::Str("StraightWB".into()));
    design.set_v("meta.bottom", Value::Str(bottom.into()));
    MetaGarment::new("outfit", &body, &design).assembly()
}

/// The whole point of the run-length rule: a stitched edge must not be
/// gathered onto itself.
///
/// Measured on the panel that broke -- a circle skirt's waistband. Every
/// particle of one stitched edge is matched to a particle of the other, and
/// the spread of where they land tells whether the edge was sewn along its
/// partner or bundled onto a fraction of it.
#[test]
fn a_stitched_edge_is_not_gathered_onto_itself() {
    let spec = outfit("SkirtCircle");
    let build = build_garment(&spec, &FitSettings::default(), &Fabric::COTTON).unwrap();

    // How many distinct partners each particle is sewn to. Gathering shows up
    // as one particle answering for a whole edge.
    let mut partners: std::collections::HashMap<u32, std::collections::HashSet<u32>> =
        std::collections::HashMap::new();
    for &[a, b] in &build.mesh.seams {
        partners.entry(a).or_default().insert(b);
        partners.entry(b).or_default().insert(a);
    }
    let worst = partners.values().map(|set| set.len()).max().unwrap_or(0);
    assert!(
        worst <= 4,
        "one particle is sewn to {worst} others -- that edge is gathered onto a point, not sewn along its partner"
    );
}

#[test]
fn builds_a_garment_from_a_preset() {
    let spec = preset_pattern();
    let settings = FitSettings::default();
    let build = build_garment(&spec, &settings, &Fabric::COTTON).expect("the preset builds");

    assert!(
        build.mesh.particle_count() > 500,
        "only {} particles for a whole garment",
        build.mesh.particle_count()
    );
    assert!(!build.mesh.triangles.is_empty());
    assert!(!build.mesh.seams.is_empty(), "nothing was sewn");
    assert!(
        build.skipped.is_empty(),
        "panels were dropped: {:?}",
        build.skipped
    );

    // Every particle somewhere sensible: a person is under two metres, and the
    // pattern is placed around one.
    for (index, position) in build.mesh.positions.iter().enumerate() {
        assert!(position.is_finite(), "particle {index} is not finite");
        assert!(
            position.length() < 5.0,
            "particle {index} is {position} -- the centimetre-to-metre scaling is wrong"
        );
    }

    // And the masses are a garment's, not a sack of sand's.
    let total: fabelgeist_shell::ParticleMass = build.mesh.masses.iter().sum();
    assert!(
        (fabelgeist_shell::ParticleMass::from(0.05)..fabelgeist_shell::ParticleMass::from(5.0))
            .contains(&total),
        "the garment weighs {total} kg"
    );
}

/// The seams are the part most likely to be silently wrong: a mis-paired edge
/// still produces a garment, just one with a twist in it.
///
/// What cannot be checked here is absolute distance. GarmentCode lays panels
/// out *around* the body rather than already touching -- the stitched edges of
/// a preset are about 30 cm apart before anything is simulated, and closing
/// that gap is exactly what the seam constraints are for. So the check is
/// relative: the pairing must join different panels, and it must do no worse
/// than pairing the pattern's own edge endpoints, which is the naive answer.
#[test]
fn seams_join_different_panels_no_worse_than_the_pattern_does() {
    let spec = preset_pattern();
    let build = build_garment(&spec, &FitSettings::default(), &Fabric::COTTON).unwrap();

    let cross_panel = build
        .mesh
        .seams
        .iter()
        .filter(|&&[a, b]| build.mesh.panel_of(a) != build.mesh.panel_of(b))
        .count();
    assert!(
        cross_panel > build.mesh.seams.len() / 2,
        "only {cross_panel} of {} seams join two panels",
        build.mesh.seams.len()
    );

    let mut spans: Vec<f32> = build
        .mesh
        .seams
        .iter()
        .map(|&[a, b]| {
            (build.mesh.positions[a as usize] - build.mesh.positions[b as usize]).length()
        })
        .collect();
    spans.sort_by(f32::total_cmp);
    let median = spans[spans.len() / 2];

    // The pattern's own answer: the mean gap between the two stitched edges'
    // endpoints, whichever way round is shorter. In metres, to compare.
    let mut naive: Vec<f32> = Vec::new();
    for stitch in &spec.stitches {
        let (Some(a), Some(b)) = (
            spec.panel(&stitch.sides[0].panel),
            spec.panel(&stitch.sides[1].panel),
        ) else {
            continue;
        };
        let place = |panel: &fabelgeist_garment_code::pattern::spec::PanelSpec, vertex: usize| {
            let point = panel.vertices[vertex];
            fabelgeist_cloth::garment::rotate_xyz(
                Vec3::new(point[0] as f32, point[1] as f32, 0.0),
                Vec3::new(
                    panel.rotation[0] as f32,
                    panel.rotation[1] as f32,
                    panel.rotation[2] as f32,
                ),
            ) + Vec3::new(
                panel.translation[0] as f32,
                panel.translation[1] as f32,
                panel.translation[2] as f32,
            )
        };
        let a_edge = &a.edges[stitch.sides[0].edge];
        let b_edge = &b.edges[stitch.sides[1].edge];
        let a0 = place(a, a_edge.endpoints[0]);
        let a1 = place(a, a_edge.endpoints[1]);
        let b0 = place(b, b_edge.endpoints[0]);
        let b1 = place(b, b_edge.endpoints[1]);
        let aligned = (a0 - b0).length() + (a1 - b1).length();
        let crossed = (a0 - b1).length() + (a1 - b0).length();
        naive.push(aligned.min(crossed) * 0.5 * CM_TO_M);
    }
    naive.sort_by(f32::total_cmp);
    let naive_median = naive[naive.len() / 2];

    assert!(
        median <= naive_median * 1.1,
        "the pairing spans {median:.3} m where the pattern's own endpoints span          {naive_median:.3} m -- it is choosing worse partners than the trivial answer"
    );
}

/// The direction test has to actually flip when the geometry says so, or every
/// seam would be sewn the same way round and half of them would be twisted.
#[test]
fn seam_direction_follows_the_geometry() {
    let outline = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(0.1, 0.0),
        Vec2::new(0.1, 0.3),
        Vec2::new(0.0, 0.3),
    ];
    let panels = vec![
        Panel::new("a", outline.clone()),
        // The same panel, turned upside down: its edge 1 now runs the other
        // way, so the seam between the two must be reversed.
        Panel::new("b", outline).placed(Placement {
            translation: Vec3::new(0.0, 0.3, 0.05),
            rotation: Vec3::new(0.0, 0.0, 180.0),
        }),
    ];

    let upright = should_reverse(&panels[..1], 0, &(1..2), 0, &(1..2));
    assert!(!upright, "a panel sewn to itself is not reversed");

    let flipped = should_reverse(&panels, 0, &(1..2), 1, &(1..2));
    assert!(flipped, "a panel turned upside down must sew reversed");
}

#[test]
fn resolution_controls_the_particle_count() {
    let spec = preset_pattern();
    let coarse = build_garment(
        &spec,
        &FitSettings {
            resolution_cm: 4.0,
            ..Default::default()
        },
        &Fabric::COTTON,
    )
    .unwrap();
    let fine = build_garment(
        &spec,
        &FitSettings {
            resolution_cm: 2.0,
            ..Default::default()
        },
        &Fabric::COTTON,
    )
    .unwrap();

    assert!(
        fine.mesh.particle_count() > coarse.mesh.particle_count() * 2,
        "halving the resolution went from {} to {} particles",
        coarse.mesh.particle_count(),
        fine.mesh.particle_count()
    );
}

#[test]
fn rejects_an_empty_pattern() {
    let spec = PatternSpec::empty();
    assert!(build_garment(&spec, &FitSettings::default(), &Fabric::COTTON).is_err());
}

#[test]
fn normalizes_a_body_to_a_height() {
    // A crude body: two metres tall in some other unit, off-centre.
    let raw: Vec<[f32; 3]> = vec![
        [100.0, 50.0, 10.0],
        [140.0, 220.0, 30.0],
        [120.0, 135.0, 20.0],
    ];
    let normalized = normalize_body(&raw, 1.7);

    let lowest = normalized.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
    let highest = normalized
        .iter()
        .map(|p| p.y)
        .fold(f32::NEG_INFINITY, f32::max);
    assert!(lowest.abs() < 1e-5, "the feet are at {lowest}, not zero");
    assert!(
        (highest - 1.7).abs() < 1e-5,
        "the head is at {highest}, not 1.7"
    );

    // Centred in x and z, so the garment's own centre line lands on the body.
    let center_x = (normalized.iter().map(|p| p.x).fold(f32::INFINITY, f32::min)
        + normalized
            .iter()
            .map(|p| p.x)
            .fold(f32::NEG_INFINITY, f32::max))
        * 0.5;
    assert!(center_x.abs() < 1e-5, "not centred in x: {center_x}");

    assert!(normalize_body(&[], 1.7).is_empty());
}

/// The whole pipeline, on a real device: a bundled preset becomes cloth, the
/// cloth is simulated, and the seams close.
///
/// This is the test that would catch a mis-wired seam, a wrong unit or an
/// unstable solve -- none of which the host-side checks above can see, because
/// they all look at the layout before anything has moved.
#[tokio::test]
async fn a_preset_garment_sews_itself_together() -> anyhow::Result<()> {
    let context = WgpuContext::new().await?;
    let spec = preset_pattern();
    let fabric = Fabric::COTTON;
    let settings = FitSettings {
        // Coarse, so the test is quick; the pipeline is the same.
        resolution_cm: 3.0,
        ..Default::default()
    };
    let build = build_garment(&spec, &settings, &fabric)?;

    let before = mean_seam_span(&build);
    let mut fit = Fit::new(context, &build, fabric, &settings)?;
    for _ in 0..240 {
        fit.step((1.0 / 60.0).into()).await?;
    }

    let positions = fit.positions().await?;
    assert_eq!(positions.len(), build.mesh.particle_count());
    assert!(
        positions.iter().all(|p| p.iter().all(|c| c.is_finite())),
        "the solve blew up"
    );

    let after = mean_seam_span_at(&build, &positions);
    assert!(
        after < before * 0.25,
        "the seams closed from {before:.3} m to {after:.3} m, which is not sewn together"
    );

    // And the garment has not stretched itself apart doing it.
    let worst = build
        .mesh
        .edges
        .iter()
        .zip(&build.mesh.rest_lengths)
        .map(|(edge, &rest)| {
            let a = Vec3::from_array(positions[edge[0] as usize]);
            let b = Vec3::from_array(positions[edge[1] as usize]);
            ((a - b).length() - rest).abs() / rest.max(1e-6)
        })
        .fold(0.0f32, f32::max);
    assert!(
        worst < 0.5,
        "an edge is {:.0}% off its rest length",
        worst * 100.0
    );
    Ok(())
}

/// A garment dropped onto a sphere has to stay outside it. Stands in for the
/// body here so the test needs no MHR assets.
#[tokio::test]
async fn a_garment_stays_outside_a_body() -> anyhow::Result<()> {
    let context = WgpuContext::new().await?;
    let spec = preset_pattern();
    let fabric = Fabric::COTTON;
    let settings = FitSettings {
        resolution_cm: 3.0,
        ..Default::default()
    };
    let build = build_garment(&spec, &settings, &fabric)?;
    let mut fit = Fit::new(context, &build, fabric, &settings)?;

    // A torso-sized sphere in the middle of where the panels are laid out.
    let center = build
        .mesh
        .positions
        .iter()
        .fold(Vec3::default(), |acc, p| acc + *p)
        * (1.0 / build.mesh.particle_count() as f32);
    let radius = 0.12f32;
    fit.collisions.set_colliders(
        &fit.context,
        vec![
            fabelgeist_physics::Collider::sphere(center, radius)
                .with_friction(0.4)
                .with_thickness(0.005),
            fabelgeist_physics::Collider::ground(-1.0),
        ],
    )?;

    for _ in 0..240 {
        fit.step((1.0 / 60.0).into()).await?;
    }

    let positions = fit.positions().await?;
    let mut inside = 0;
    for position in &positions {
        let point = Vec3::from_array(*position);
        if (point - center).length() < radius - 0.01 {
            inside += 1;
        }
    }
    assert_eq!(inside, 0, "{inside} particles ended up inside the body");
    Ok(())
}

fn mean_seam_span(build: &GarmentBuild) -> f32 {
    let positions: Vec<[f32; 3]> = build
        .mesh
        .positions
        .iter()
        .map(|p| [p.x, p.y, p.z])
        .collect();
    mean_seam_span_at(build, &positions)
}

fn mean_seam_span_at(build: &GarmentBuild, positions: &[[f32; 3]]) -> f32 {
    if build.mesh.seams.is_empty() {
        return 0.0;
    }
    build
        .mesh
        .seams
        .iter()
        .map(|&[a, b]| {
            (Vec3::from_array(positions[a as usize]) - Vec3::from_array(positions[b as usize]))
                .length()
        })
        .sum::<f32>()
        / build.mesh.seams.len() as f32
}

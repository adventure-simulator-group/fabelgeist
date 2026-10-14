use super::*;

fn square(size: f32) -> Vec<Vec2> {
    vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(size, 0.0),
        Vec2::new(size, size),
        Vec2::new(0.0, size),
    ]
}

/// Two panels facing each other, to be sewn down both sides -- a tube, which
/// is what a sleeve or a bodice front and back is.
fn tube() -> (Vec<Panel>, Vec<Seam>) {
    let panels = vec![
        Panel::new("front", square(0.3)).placed(Placement {
            translation: Vec3::new(-0.15, 0.0, 0.1),
            rotation: Vec3::default(),
        }),
        Panel::new("back", square(0.3)).placed(Placement {
            translation: Vec3::new(-0.15, 0.0, -0.1),
            rotation: Vec3::default(),
        }),
    ];
    let seams = vec![
        // Right edge of the front to the right edge of the back.
        Seam::new(
            SeamSide { panel: 0, edge: 1 },
            SeamSide { panel: 1, edge: 1 },
        ),
        // And the left edges.
        Seam::new(
            SeamSide { panel: 0, edge: 3 },
            SeamSide { panel: 1, edge: 3 },
        ),
    ];
    (panels, seams)
}

#[test]
fn places_panels_in_space() {
    let placement = Placement {
        translation: Vec3::new(1.0, 2.0, 3.0),
        rotation: Vec3::default(),
    };
    let placed = placement.apply(Vec2::new(0.5, -0.5));
    assert!((placed - Vec3::new(1.5, 1.5, 3.0)).length() < 1e-6);

    // A quarter turn about y sends +x to -z.
    let turned = Placement {
        translation: Vec3::default(),
        rotation: Vec3::new(0.0, 90.0, 0.0),
    };
    let result = turned.apply(Vec2::new(1.0, 0.0));
    assert!(
        (result - Vec3::new(0.0, 0.0, -1.0)).length() < 1e-5,
        "expected (0, 0, -1), got {result}"
    );

    // And about x, +y goes to +z.
    let pitched = Placement {
        translation: Vec3::default(),
        rotation: Vec3::new(90.0, 0.0, 0.0),
    };
    let pitched_result = pitched.apply(Vec2::new(0.0, 1.0));
    assert!(
        (pitched_result - Vec3::new(0.0, 0.0, 1.0)).length() < 1e-5,
        "expected (0, 0, 1), got {pitched_result}"
    );
}

#[test]
fn rotation_preserves_length() {
    let point = Vec3::new(0.3, -0.7, 1.1);
    for angles in [
        Vec3::new(37.0, 0.0, 0.0),
        Vec3::new(0.0, 121.0, 0.0),
        Vec3::new(0.0, 0.0, -64.0),
        Vec3::new(23.0, -47.0, 88.0),
    ] {
        let rotated = rotate_xyz(point, angles);
        assert!(
            (rotated.length() - point.length()).abs() < 1e-5,
            "rotating by {angles} changed the length"
        );
    }
}

#[test]
fn builds_a_single_panel() {
    let panels = vec![Panel::new("only", square(0.4))];
    let mesh = build(&panels, &[], 0.05, 0.2.into()).unwrap();

    assert!(mesh.particle_count() > 50, "too coarse to be a cloth");
    assert!(!mesh.triangles.is_empty());
    assert_eq!(mesh.panel_count(), 1);
    assert_eq!(mesh.panel_offsets, vec![0, mesh.particle_count() as u32]);
    assert!(mesh.seams.is_empty(), "one panel has nothing to sew to");

    // Rest lengths are the flat layout's own lengths, so the panel starts
    // exactly at rest.
    assert_eq!(mesh.rest_lengths.len(), mesh.edges.len());
    for (edge, &rest) in mesh.edges.iter().zip(&mesh.rest_lengths) {
        let actual = (mesh.positions[edge[0] as usize] - mesh.positions[edge[1] as usize]).length();
        assert!((actual - rest).abs() < 1e-6);
    }

    // A flat panel starts flat, so every hinge's bending measure rests at
    // zero, and its weights sum to zero so the measure is translation-free.
    assert_eq!(mesh.bend_weights.len(), mesh.bends.len());
    assert!(!mesh.bends.is_empty(), "a meshed panel has interior hinges");
    for record in &mesh.bend_weights {
        let weights: &[f32] = bytemuck::cast_slice(std::slice::from_ref(record));
        let magnitude = weights[..4].iter().map(|w| w.abs()).sum::<f32>().max(1e-9);
        assert!(
            weights[..4].iter().sum::<f32>().abs() < magnitude * 1e-3,
            "bending weights {weights:?} do not sum to zero"
        );
        assert!(
            weights[4] < 1e-4,
            "a flat panel's hinge rests at {}, not zero",
            weights[4]
        );
    }

    // The total mass is the area times the density.
    let total: ParticleMass = mesh.masses.iter().sum();
    assert!(
        (total - ParticleMass::from(0.16 * 0.2)).absolute() < ParticleMass::from(0.16 * 0.2 * 0.05),
        "a 0.4 m square at 0.2 kg/m^2 weighs {total}"
    );
}

#[test]
fn sews_two_panels_together() {
    let (panels, seams) = tube();
    let mesh = build(&panels, &seams, 0.03, 0.2.into()).unwrap();

    assert_eq!(mesh.panel_count(), 2);
    assert!(!mesh.seams.is_empty(), "the seams produced no constraints");

    // Every seam constraint must join the two panels, never a panel to itself.
    for &[a, b] in &mesh.seams {
        let panel_a = mesh.panel_of(a);
        let panel_b = mesh.panel_of(b);
        assert_ne!(
            panel_a, panel_b,
            "seam {a}-{b} joins panel {panel_a} to itself"
        );
    }

    // And every vertex along both sewn edges should be caught up in one.
    let sewn: std::collections::HashSet<u32> =
        mesh.seams.iter().flat_map(|&[a, b]| [a, b]).collect();
    assert!(
        sewn.len() > 10,
        "only {} vertices are sewn; the seam is too sparse to close",
        sewn.len()
    );
}

/// A seam pairs a panel edge with a panel edge, so the constraints have to run
/// along the two edges rather than jumping about. Checked by distance: the
/// panels are 0.2 apart, so a correct pairing is roughly that far and a
/// scrambled one is much further.
#[test]
fn seam_pairs_run_along_the_edges() {
    let (panels, seams) = tube();
    let mesh = build(&panels, &seams, 0.03, 0.2.into()).unwrap();

    for &[a, b] in &mesh.seams {
        let distance = (mesh.positions[a as usize] - mesh.positions[b as usize]).length();
        assert!(
            distance < 0.25,
            "seam {a}-{b} spans {distance}; the panels are only 0.2 apart, so this pair is wrong"
        );
    }
}

/// Reversing a seam should pair each vertex with the *other* end of the
/// partner edge. With the panels offset along the edge, that is easy to see.
#[test]
fn reversing_a_seam_flips_the_pairing() {
    let panels = vec![
        Panel::new("a", square(0.3)),
        Panel::new("b", square(0.3)).placed(Placement {
            translation: Vec3::new(0.0, 0.0, 0.2),
            rotation: Vec3::default(),
        }),
    ];
    let side_a = SeamSide { panel: 0, edge: 1 };
    let side_b = SeamSide { panel: 1, edge: 1 };

    let forward = build(&panels, &[Seam::new(side_a, side_b)], 0.03, 0.2.into()).unwrap();
    let reversed = build(
        &panels,
        &[Seam::new(side_a, side_b).reversed(true)],
        0.03,
        0.2.into(),
    )
    .unwrap();

    // Forward: partners are at the same height. Reversed: mirrored about the
    // edge's middle.
    let height_gap = |mesh: &GarmentMesh| {
        mesh.seams
            .iter()
            .map(|&[a, b]| (mesh.positions[a as usize].y - mesh.positions[b as usize].y).abs())
            .fold(0.0f32, f32::max)
    };
    assert!(
        height_gap(&forward) < 0.02,
        "a forward seam should pair like with like"
    );
    assert!(
        height_gap(&reversed) > 0.2,
        "a reversed seam should pair top with bottom"
    );
}

/// A seam pair that is already a mesh edge would be two constraints fighting
/// over one distance -- the stretch one holding it open, the seam one closing
/// it. Sewing an edge to itself is the way to produce that.
#[test]
fn drops_seams_that_duplicate_a_mesh_edge() {
    let panels = vec![Panel::new("only", square(0.3))];
    let side = SeamSide { panel: 0, edge: 0 };
    let mesh = build(&panels, &[Seam::new(side, side)], 0.05, 0.2.into()).unwrap();

    let edges: std::collections::HashSet<[u32; 2]> = mesh.edges.iter().copied().collect();
    for &[a, b] in &mesh.seams {
        assert_ne!(a, b, "a particle sewn to itself");
        assert!(
            !edges.contains(&[a.min(b), a.max(b)]),
            "seam {a}-{b} duplicates a stretch constraint"
        );
    }
}

#[test]
fn reports_a_bad_seam() {
    let panels = vec![Panel::new("only", square(0.3))];
    let bad_panel = Seam::new(
        SeamSide { panel: 0, edge: 0 },
        SeamSide { panel: 9, edge: 0 },
    );
    assert!(build(&panels, &[bad_panel], 0.05, 0.2.into()).is_err());

    let bad_edge = Seam::new(
        SeamSide { panel: 0, edge: 0 },
        SeamSide { panel: 0, edge: 99 },
    );
    assert!(build(&panels, &[bad_edge], 0.05, 0.2.into()).is_err());
}

#[test]
fn handles_no_panels() {
    let mesh = build(&[], &[], 0.05, 0.2.into()).unwrap();
    assert_eq!(mesh.particle_count(), 0);
    assert_eq!(mesh.panel_count(), 0);
}

#[test]
fn rejects_a_nonsense_resolution() {
    let panels = vec![Panel::new("only", square(0.3))];
    assert!(build(&panels, &[], 0.0, 0.2.into()).is_err());
    assert!(build(&panels, &[], -1.0, 0.2.into()).is_err());
}

#[test]
fn panel_lookup_covers_every_particle() {
    let (panels, seams) = tube();
    let mesh = build(&panels, &seams, 0.04, 0.2.into()).unwrap();
    for particle in 0..mesh.particle_count() as u32 {
        let panel = mesh.panel_of(particle);
        assert!(panel < mesh.panel_count());
        assert!(particle >= mesh.panel_offsets[panel]);
        assert!(particle < mesh.panel_offsets[panel + 1]);
    }
}

/// The adjacency the self-collision pass skips has to include the seams, or
/// the repulsion fights the seam constraint and the garment never closes.
#[test]
fn adjacency_covers_edges_and_seams() {
    let (panels, seams) = tube();
    let mesh = build(&panels, &seams, 0.04, 0.2.into()).unwrap();
    let adjacency = mesh.adjacency();

    assert_eq!(adjacency.len(), mesh.particle_count());
    for &[a, b] in &mesh.edges {
        assert!(
            adjacency[a as usize].contains(&b),
            "edge {a}-{b} is missing"
        );
        assert!(adjacency[b as usize].contains(&a));
    }
    for &[a, b] in &mesh.seams {
        assert!(
            adjacency[a as usize].contains(&b),
            "seam {a}-{b} is missing"
        );
        assert!(adjacency[b as usize].contains(&a));
    }
    for (index, list) in adjacency.iter().enumerate() {
        let mut sorted = list.clone();
        sorted.dedup();
        assert_eq!(sorted.len(), list.len(), "particle {index} has duplicates");
    }
}

/// The material positions are the flat shape the rest lengths were measured
/// from, so a placement must move `positions` and leave `material` alone --
/// otherwise the pattern drawn flat is the pattern already half-draped.
#[test]
fn material_positions_are_the_flat_rest_shape() {
    let placement = Placement {
        translation: Vec3::new(0.4, 1.2, -0.3),
        rotation: Vec3::new(0.0, 90.0, 0.0),
    };
    let panels = vec![Panel::new("front", square(0.3)).placed(placement)];
    let mesh = build(&panels, &[], 0.05, 0.2.into()).unwrap();

    assert_eq!(mesh.material.len(), mesh.positions.len());

    // Every material point is inside the panel it was cut from ...
    for point in &mesh.material {
        assert!(
            (-1e-5..=0.3 + 1e-5).contains(&point.x) && (-1e-5..=0.3 + 1e-5).contains(&point.y),
            "material point {point} is outside the 0.3 m square it was cut from"
        );
    }

    // ... and placing it gives back the world position, unchanged.
    for (flat, world) in mesh.material.iter().zip(&mesh.positions) {
        assert!(
            (placement.apply(*flat) - *world).length() < 1e-5,
            "placing {flat} gave a different point from the one built"
        );
    }

    // The rest lengths were measured flat, so they are the material distances.
    for (&[a, b], &rest) in mesh.edges.iter().zip(&mesh.rest_lengths) {
        let flat = (mesh.material[a as usize] - mesh.material[b as usize]).length();
        assert!(
            (flat - rest).abs() < 1e-5,
            "edge {a}-{b}: rest {rest}, flat {flat}"
        );
    }
}

/// A panel's particles are contiguous, and the ranges tile the whole mesh.
#[test]
fn panel_ranges_tile_the_mesh() {
    let (panels, seams) = tube();
    let mesh = build(&panels, &seams, 0.05, 0.2.into()).unwrap();

    let mut next = 0;
    for panel in 0..mesh.panel_count() {
        let range = mesh.panel_range(panel);
        assert_eq!(range.start, next, "panel {panel} does not follow the last");
        assert!(!range.is_empty(), "panel {panel} has no particles");
        for particle in range.clone() {
            assert_eq!(mesh.panel_of(particle as u32), panel);
        }
        next = range.end;
    }
    assert_eq!(next, mesh.particle_count());
}

#[test]
fn different_length_sewn_edges_have_one_partner_per_vertex() {
    let rectangle = |name: &str, width: f32| {
        Panel::new(
            name,
            vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(width, 0.0),
                Vec2::new(width, 1.0),
                Vec2::new(0.0, 1.0),
            ],
        )
    };
    let panels = [rectangle("a", 1.0), rectangle("b", 0.61)];
    let mesh = build(
        &panels,
        &[Seam::new(
            SeamSide { panel: 0, edge: 0 },
            SeamSide { panel: 1, edge: 0 },
        )],
        0.2,
        0.2.into(),
    )
    .unwrap();
    let mut partners = std::collections::HashMap::new();
    for &[a, b] in &mesh.seams {
        assert!(partners.insert(a, b).is_none(), "gathered vertex {a}");
        assert!(partners.insert(b, a).is_none(), "gathered vertex {b}");
    }
    assert_eq!(mesh.seams.len(), 6);
}

#[test]
fn sewn_edges_have_bending_continuity_in_material_space() {
    let panel = |name: &str| {
        Panel::new(
            name,
            vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(0.0, 1.0),
            ],
        )
    };
    let mesh = build(
        &[
            panel("a"),
            panel("b").placed(Placement {
                translation: Vec3::new(0.0, 0.0, 1.0),
                ..Default::default()
            }),
        ],
        &[Seam::new(
            SeamSide { panel: 0, edge: 0 },
            SeamSide { panel: 1, edge: 0 },
        )],
        0.2,
        0.2.into(),
    )
    .unwrap();
    let hinges: Vec<_> = mesh
        .bends
        .iter()
        .zip(&mesh.bend_weights)
        .filter(|(b, _)| mesh.panel_of(b.wings[0]) != mesh.panel_of(b.wings[1]))
        .collect();
    assert_eq!(hinges.len(), 5);
    for (_, record) in hinges {
        let weights: &[f32] = bytemuck::cast_slice(std::slice::from_ref(record));
        assert!(weights[..4].iter().sum::<f32>().abs() < 1e-6);
        assert_eq!(
            weights[4], 0.0,
            "initial panel separation must not become a permanent crease"
        );
    }
}

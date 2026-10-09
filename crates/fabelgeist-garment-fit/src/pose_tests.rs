//! Placing a garment, checked on a real pattern rather than a hand-built toy.
//!
//! The property that matters most is the one the whole design rests on: a
//! pose is *rigid*, so the mesh it moves stays a valid mesh. Every rest
//! length, bending weight and mass in a `GarmentMesh` was measured from its
//! placed positions, and none of them is recomputed when the garment is put
//! somewhere else. If a pose could change a distance, the fabric would
//! silently change size with it.

use fabelgeist_cloth::Fabric;
use fabelgeist_math::Vec3;

use super::pose::{Axis, GarmentPose, Move, Pivots};
use super::tests::preset_pattern;
use super::{FitSettings, build_garment};

fn preset() -> (Vec<Vec3>, Pivots, fabelgeist_cloth::GarmentMesh) {
    let spec = preset_pattern();
    let build =
        build_garment(&spec, &FitSettings::default(), &Fabric::COTTON).expect("the preset builds");
    let pivots = Pivots::of(&build.mesh);
    (build.mesh.positions.clone(), pivots, build.mesh)
}

/// A pose that exercises every axis at once, whole garment and one panel.
fn busy_pose(panels: usize) -> GarmentPose {
    let mut pose = GarmentPose {
        garment: Move {
            offset: Vec3::new(0.03, -0.07, 0.11),
            yaw: 37.0,
            pitch: -12.0,
            roll: 8.0,
        },
        panels: vec![Move::IDENTITY; panels],
    };
    *pose.panel_mut(panels / 2) = Move {
        offset: Vec3::new(-0.02, 0.05, 0.01),
        yaw: -20.0,
        pitch: 9.0,
        roll: -4.0,
    };
    pose
}

#[test]
fn an_untouched_pose_moves_nothing() {
    let (base, pivots, mesh) = preset();
    let placed = GarmentPose::for_mesh(&mesh).place(&base, &pivots);
    assert_eq!(placed, base, "an identity pose is not the identity");
}

/// The load-bearing property: placing preserves the fabric's own distances,
/// so the rest lengths the constraints were built with still describe it.
///
/// Every stretch constraint lies inside one panel -- they come from the
/// triangles, and no triangle spans two panels -- so this holds even for the
/// per-panel nudges, which are rigid within a panel but not between panels.
#[test]
fn placing_never_stretches_the_fabric() {
    let (base, pivots, mesh) = preset();
    let placed = busy_pose(mesh.panel_count()).place(&base, &pivots);

    assert_eq!(placed.len(), base.len());

    let mut worst = 0.0f32;
    for (&[a, b], &rest) in mesh.edges.iter().zip(&mesh.rest_lengths) {
        let moved = (placed[a as usize] - placed[b as usize]).length();
        worst = worst.max((moved - rest).abs());
    }
    // A tenth of a millimetre over a whole garment: single-precision rotation
    // of positions a metre from the origin, and nothing more.
    assert!(
        worst < 1e-4,
        "placing stretched a constraint by {worst} m -- the fabric changed size"
    );
}

/// Moving the whole garment moves the seams with it: the gap each seam has to
/// close is the same wherever the garment is put.
///
/// A per-panel nudge is the one thing that *does* change those gaps, and on
/// purpose -- pulling a sleeve out of an arm moves it away from the bodice it
/// is sewn to. So this is the whole-garment move on its own, and the panel
/// nudge is checked below for what it must not disturb.
#[test]
fn a_whole_garment_move_leaves_every_seam_gap_alone() {
    let (base, pivots, mesh) = preset();
    let pose = GarmentPose {
        garment: busy_pose(mesh.panel_count()).garment,
        panels: vec![Move::IDENTITY; mesh.panel_count()],
    };
    let placed = pose.place(&base, &pivots);

    for &[a, b] in &mesh.seams {
        let before = (base[a as usize] - base[b as usize]).length();
        let after = (placed[a as usize] - placed[b as usize]).length();
        assert!(
            (after - before).abs() < 1e-4,
            "a seam's two sides moved {} m apart",
            (after - before).abs()
        );
    }
}

/// A panel nudge changes only the seams that panel is part of.
#[test]
fn a_panel_nudge_disturbs_only_its_own_seams() {
    let (base, pivots, mesh) = preset();
    let moved_panel = mesh.panel_count() / 2;
    let mut pose = GarmentPose::for_mesh(&mesh);
    *pose.panel_mut(moved_panel) = Move {
        offset: Vec3::new(0.0, 0.09, 0.0),
        ..Move::IDENTITY
    };
    let placed = pose.place(&base, &pivots);

    let mut disturbed = 0usize;
    for &[a, b] in &mesh.seams {
        let touches = mesh.panel_of(a) == moved_panel || mesh.panel_of(b) == moved_panel;
        let before = (base[a as usize] - base[b as usize]).length();
        let after = (placed[a as usize] - placed[b as usize]).length();
        let changed = (after - before).abs();
        if touches {
            disturbed += usize::from(changed > 1e-4);
        } else {
            assert!(
                changed < 1e-4,
                "a seam between two panels that did not move changed by {changed} m"
            );
        }
    }
    assert!(
        disturbed > 0,
        "nudging panel {moved_panel} did not move any of its seams"
    );
}

/// A shift moves every particle by the same vector and turns nothing.
#[test]
fn a_shift_translates_the_whole_garment() {
    let (base, pivots, mesh) = preset();
    let offset = Vec3::new(0.04, -0.11, 0.06);
    let pose = GarmentPose {
        garment: Move {
            offset,
            ..Move::IDENTITY
        },
        panels: vec![Move::IDENTITY; mesh.panel_count()],
    };
    let placed = pose.place(&base, &pivots);

    for (index, (before, after)) in base.iter().zip(&placed).enumerate() {
        assert!(
            (*after - (*before + offset)).length() < 1e-6,
            "particle {index} shifted to {after}, not {}",
            *before + offset
        );
    }
}

/// Turn is the control that fixes a garment facing the wrong way, which is a
/// half-turn about the vertical axis and nothing else.
#[test]
fn a_half_turn_swaps_front_for_back() {
    let (base, pivots, mesh) = preset();
    let pose = GarmentPose {
        garment: Move {
            yaw: 180.0,
            ..Move::IDENTITY
        },
        panels: vec![Move::IDENTITY; mesh.panel_count()],
    };
    let placed = pose.place(&base, &pivots);

    let pivot = pivots.garment;
    for (index, (before, after)) in base.iter().zip(&placed).enumerate() {
        let expected = Vec3::new(
            pivot.x - (before.x - pivot.x),
            before.y,
            pivot.z - (before.z - pivot.z),
        );
        assert!(
            (*after - expected).length() < 1e-5,
            "particle {index}: a half-turn put it at {after}, not {expected}"
        );
    }
}

#[test]
fn a_panel_move_leaves_the_other_panels_alone() {
    let (base, pivots, mesh) = preset();
    assert!(mesh.panel_count() > 1, "the preset has only one panel");

    let moved_panel = mesh.panel_count() / 2;
    let mut pose = GarmentPose::for_mesh(&mesh);
    *pose.panel_mut(moved_panel) = Move {
        offset: Vec3::new(0.0, 0.13, 0.0),
        yaw: 25.0,
        ..Move::IDENTITY
    };
    let placed = pose.place(&base, &pivots);

    for panel in 0..mesh.panel_count() {
        let range = mesh.panel_range(panel);
        let same = base[range.clone()] == placed[range.clone()];
        if panel == moved_panel {
            assert!(!same, "the panel that was moved did not move");
        } else {
            assert!(
                same,
                "panel {} ({}) moved when panel {moved_panel} was nudged",
                panel, mesh.panel_names[panel]
            );
        }
    }
}

/// A pose describes a place, not a journey: it is always composed against the
/// pattern's own positions, so putting a garment back is exact rather than
/// approximately where it started.
#[test]
fn moving_a_garment_back_is_exact() {
    let (base, pivots, mesh) = preset();
    let pose = busy_pose(mesh.panel_count());
    let there = pose.place(&base, &pivots);
    assert_ne!(there, base);

    let back = GarmentPose::for_mesh(&mesh).place(&base, &pivots);
    assert_eq!(back, base, "returning to an untouched pose drifted");
}

/// The pivots are taken from the outline, so they do not move when the mesh
/// is made finer -- which is what lets a pose survive the resolution slider.
#[test]
fn pivots_do_not_follow_the_resolution() {
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

    let coarse_pivots = Pivots::of(&coarse.mesh);
    let fine_pivots = Pivots::of(&fine.mesh);

    assert!(
        (coarse_pivots.garment - fine_pivots.garment).length() < 5e-3,
        "the garment's pivot moved {} m when the mesh was refined",
        (coarse_pivots.garment - fine_pivots.garment).length()
    );
    assert_eq!(coarse_pivots.panels.len(), fine_pivots.panels.len());
    for (panel, ((_, coarse), (_, fine))) in coarse_pivots
        .panels
        .iter()
        .zip(&fine_pivots.panels)
        .enumerate()
    {
        assert!(
            (*coarse - *fine).length() < 5e-3,
            "panel {panel}'s pivot moved {} m when the mesh was refined",
            (*coarse - *fine).length()
        );
    }
}

/// The controls read and write centimetres; the solver keeps metres.
#[test]
fn the_controls_speak_centimetres() {
    let mut moved = Move::IDENTITY;
    Axis::Vertical.set(&mut moved, -12.5);
    assert!((moved.offset.y - -0.125).abs() < 1e-9, "{moved:?}");
    assert!((Axis::Vertical.get(&moved) - -12.5).abs() < 1e-4);

    for axis in Axis::ALL {
        let mut moved = Move::IDENTITY;
        let value = axis.limit() * 0.5;
        axis.set(&mut moved, value);
        assert!(!moved.is_identity(), "{axis:?} wrote nothing");
        assert!(
            (axis.get(&moved) - value).abs() < 1e-3,
            "{axis:?} read back {} instead of {value}",
            axis.get(&moved)
        );

        // And each axis is its own: no two of them share a field.
        for other in Axis::ALL {
            if other != axis {
                assert_eq!(other.get(&moved), 0.0, "{axis:?} also wrote {other:?}");
            }
        }
    }
}

/// A pose from a pattern with fewer panels, applied to one with more, must not
/// panic or move panels it says nothing about.
#[test]
fn a_short_pose_leaves_the_rest_where_they_were() {
    let (base, pivots, mesh) = preset();
    let pose = GarmentPose {
        garment: Move::IDENTITY,
        panels: vec![Move {
            offset: Vec3::new(0.0, 0.2, 0.0),
            ..Move::IDENTITY
        }],
    };
    let placed = pose.place(&base, &pivots);

    let first = mesh.panel_range(0);
    assert_ne!(base[first.clone()], placed[first.clone()]);
    assert_eq!(base[first.end..], placed[first.end..]);
}

#[test]
fn asking_for_a_panel_beyond_the_end_grows_the_pose() {
    let mut pose = GarmentPose::default();
    assert!(pose.is_identity());
    pose.panel_mut(4).yaw = 90.0;
    assert_eq!(pose.panels.len(), 5);
    assert!(!pose.is_identity());
    assert!(pose.panel(3).is_identity());
    assert_eq!(pose.panel(4).yaw, 90.0);
    // Past the end reads as untouched rather than panicking.
    assert!(pose.panel(99).is_identity());
}

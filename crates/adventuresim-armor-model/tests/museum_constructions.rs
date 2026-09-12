//! Construction contracts for the museum-derived plate families.
use adventuresim_armor_model::*;
use std::collections::{BTreeMap, BTreeSet};

fn frame(half_extents: [f32; 3]) -> PartFrame {
    PartFrame {
        origin: [0.0; 3],
        axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        half_extents,
    }
}

fn helmet(design: HelmetDesign) -> PartMesh {
    generate_helmet(&design, &frame([0.085, 0.115, 0.105])).unwrap()
}

fn component(mesh: &PartMesh, role: ArmorComponentRole) -> PartMesh {
    let part = mesh.components.iter().find(|p| p.role == role).unwrap();
    let mut result = PartMesh::new();
    result.positions = mesh.positions[part.vertices.clone()].to_vec();
    result.indices = mesh.indices[part.indices.clone()]
        .iter()
        .map(|i| i - part.vertices.start as u32)
        .collect();
    result
}

fn key(point: [f32; 3]) -> [i64; 3] {
    point.map(|x| (f64::from(x) * 1e7).round() as i64)
}

/// Returns Euler characteristic after welding shading aliases. A sealed slit
/// creates a handle; a painted slit cannot change this observable quantity.
fn closed(mesh: &PartMesh) -> i64 {
    mesh.normals()
        .expect("finite nondegenerate surface and valid components");
    let mut welded = BTreeMap::new();
    let mapping = mesh
        .positions
        .iter()
        .map(|p| {
            let next = welded.len() as u32;
            *welded.entry(key(*p)).or_insert(next)
        })
        .collect::<Vec<_>>();
    let mut edges = BTreeMap::<_, Vec<_>>::new();
    let mut used = BTreeSet::new();
    let mut volumes = Vec::new();
    for face in mesh.indices.as_chunks::<3>().0 {
        let [a, b, c] = face.map(|i| mapping[i as usize]);
        assert!(a != b && b != c && c != a, "collapsed welded triangle");
        used.extend([a, b, c]);
        for (a, b) in [(a, b), (b, c), (c, a)] {
            edges.entry((a.min(b), a.max(b))).or_default().push((a, b));
        }
        let [a, b, c] = face.map(|i| mesh.positions[i as usize].map(f64::from));
        let volume = a[0] * (b[1] * c[2] - b[2] * c[1])
            + a[1] * (b[2] * c[0] - b[0] * c[2])
            + a[2] * (b[0] * c[1] - b[1] * c[0]);
        volumes.push((mapping[face[0] as usize], volume));
    }
    for uses in edges.values() {
        assert_eq!(uses.len(), 2, "open seam or nonmanifold physical edge");
        assert_eq!(uses[0], (uses[1].1, uses[1].0), "inconsistent winding");
    }
    let mut adjacency = vec![Vec::new(); welded.len()];
    for &(a, b) in edges.keys() {
        adjacency[a as usize].push(b);
        adjacency[b as usize].push(a);
    }
    let mut components = vec![None; welded.len()];
    let mut shell_volumes = Vec::new();
    for &start in &used {
        if components[start as usize].is_some() {
            continue;
        }
        let component = shell_volumes.len();
        shell_volumes.push(0.0);
        let mut pending = vec![start];
        while let Some(vertex) = pending.pop() {
            if components[vertex as usize].is_some() {
                continue;
            }
            components[vertex as usize] = Some(component);
            pending.extend(&adjacency[vertex as usize]);
        }
    }
    for (vertex, volume) in volumes {
        shell_volumes[components[vertex as usize].unwrap()] += volume;
    }
    assert!(
        shell_volumes.iter().all(|volume| *volume > 0.0),
        "every disconnected shell must be wound outward: {shell_volumes:?}"
    );
    used.len() as i64 - edges.len() as i64 + (mesh.indices.len() / 3) as i64
}

fn bounds(mesh: &PartMesh, axis: usize) -> (f32, f32) {
    mesh.positions
        .iter()
        .map(|p| p[axis])
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(a, b), x| {
            (a.min(x), b.max(x))
        })
}

#[test]
fn radial_flutes_cover_the_complete_disc_and_preserve_its_boss() {
    let plain = BesagewDesign::default();
    let radial = RadialFluting {
        count: FluteCount(12),
        ..Default::default()
    };
    let d = BesagewDesign {
        fluting: Some(radial),
        ..plain.clone()
    };
    let mesh = generate_besagew(&d, PlateGauge::default()).unwrap();
    assert_eq!(
        closed(&mesh),
        2,
        "disc must have no wedge opening or handle"
    );
    assert_eq!(mesh.components[0].role, ArmorComponentRole::Besagew);
    let smooth = generate_besagew(&plain, PlateGauge::default()).unwrap();
    assert_eq!(bounds(&mesh, 2).1, bounds(&smooth, 2).1, "boss changed");
    let mut bins = vec![Vec::new(); 12];
    for p in &mesh.positions {
        let radius = p[0].hypot(p[1]);
        if (radius - d.radius.metres() * 0.625).abs() < 1e-5 {
            let angle = p[1].atan2(p[0]).rem_euclid(std::f32::consts::TAU);
            let sector = ((angle / std::f32::consts::TAU * 12.0 + 1e-4).floor() as usize) % 12;
            bins[sector].push(p[2]);
        }
    }
    for bin in &bins {
        let lo = bin.iter().copied().fold(f32::INFINITY, f32::min);
        let hi = bin.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        assert!(
            hi - lo > radial.depth.metres(),
            "a circular sector lost its relief"
        );
        assert!(
            (hi - bins[0].iter().copied().fold(f32::NEG_INFINITY, f32::max)).abs() < 1e-6,
            "relief does not repeat through the circular seam"
        );
    }
}

#[test]
fn adding_and_moving_besagew_preserves_every_shoulder_plate() {
    let fit = frame([0.09, 0.12, 0.09]);
    let mut design = SpaulderDesign::default();
    let base = generate_limb_armor(&LimbArmorDesign::Spaulder(design.clone()), &fit).unwrap();
    design.besagew = Some(BesagewDesign::default());
    let attached = generate_limb_armor(&LimbArmorDesign::Spaulder(design.clone()), &fit).unwrap();
    let plate = component(&attached, ArmorComponentRole::Plate);
    assert_eq!(plate.positions, base.positions);
    assert_eq!(plate.indices, base.indices);
    closed(&attached);
    let first = component(&attached, ArmorComponentRole::Besagew);
    design.besagew.as_mut().unwrap().shoulder_drop.0 += 15;
    let moved = generate_limb_armor(&LimbArmorDesign::Spaulder(design), &fit).unwrap();
    let second = component(&moved, ArmorComponentRole::Besagew);
    assert_eq!(first.indices, second.indices);
    for (a, b) in first.positions.iter().zip(&second.positions) {
        assert!((b[1] - a[1] + 0.015).abs() < 1e-6);
        assert_eq!([a[0], a[2]], [b[0], b[2]]);
    }
}

#[test]
fn buffe_is_a_separate_closed_guard_without_deleting_the_burgonet() {
    let mut design = BurgonetDesign::default();
    let base = helmet(HelmetDesign::Burgonet(design));
    design.buffe = Some(BuffeDesign::default());
    let guarded = helmet(HelmetDesign::Burgonet(design));
    let skull = component(&guarded, ArmorComponentRole::Skull);
    assert_eq!(skull.positions, base.positions);
    assert_eq!(skull.indices, base.indices);
    let guard = component(&guarded, ArmorComponentRole::Buffe);
    assert_eq!(closed(&guard), 2);
    design.buffe.as_mut().unwrap().face_projection.0 += 10;
    let changed = helmet(HelmetDesign::Burgonet(design));
    let projected = component(&changed, ArmorComponentRole::Buffe);
    assert_eq!(guard.indices, projected.indices);
    assert!(bounds(&projected, 2).1 > bounds(&guard, 2).1 + 0.004);
    assert_eq!(
        component(&changed, ArmorComponentRole::Skull).positions,
        skull.positions
    );
}

#[test]
fn shoulder_coverage_trims_the_crown_without_squeezing_it_or_moving_lames() {
    let fit = frame([0.09, 0.12, 0.09]);
    let base = SpaulderDesign::default();
    let build = |coverage, fit: &PartFrame| {
        generate_limb_armor(
            &LimbArmorDesign::Spaulder(SpaulderDesign {
                crown_coverage: Permille(coverage),
                ..base.clone()
            }),
            fit,
        )
        .unwrap()
    };
    let full = build(1000, &fit);
    let full_carriers = retained_carriers(&full);
    for coverage in [200, 500, 999, 1000] {
        let mesh = build(coverage, &fit);
        assert_eq!(closed(&mesh), 2 * i64::from(base.lame_count + 1));
        let carriers = retained_carriers(&mesh);
        assert_eq!(
            &carriers[..usize::from(base.lame_count)],
            &full_carriers[..usize::from(base.lame_count)],
            "crown trim moved arm lames"
        );
        let scaled = build(coverage, &frame([0.075, 0.10, 0.08]));
        assert_eq!(
            mesh.indices, scaled.indices,
            "wearer size changed trim topology"
        );
        if coverage <= 500 {
            assert!(bounds(&mesh, 1).1 < bounds(&full, 1).1);
        }
    }
    let half = build(500, &fit);
    let partial = retained_carriers(&half);
    // Half of the complete crown has coincident samples on the same formed
    // surface. A vertically compressed dome would move every interior sample.
    let full_crown = full_carriers.last().unwrap();
    let matches = partial
        .last()
        .unwrap()
        .iter()
        .filter(|p| {
            full_crown
                .iter()
                .any(|q| (0..3).all(|axis| (p[axis] - q[axis]).abs() < 1e-6))
        })
        .count();
    assert!(
        matches > partial.last().unwrap().len() / 3,
        "coverage deformed the crown instead of retaining its surface"
    );
    assert_eq!(half.indices, build(200, &fit).indices);
    for coverage in [199, 1001] {
        assert!(
            LimbArmorDesign::Spaulder(SpaulderDesign {
                crown_coverage: Permille(coverage),
                ..base.clone()
            })
            .validate()
            .is_err()
        );
    }
}

#[test]
fn buffe_ridge_and_chin_controls_preserve_other_plates_and_surface_correspondence() {
    let mut d = BurgonetDesign {
        cheek_depth: Permille(800),
        chin_tab: Millimeters(0),
        buffe: Some(BuffeDesign {
            neck_drop: Millimeters(20),
            chin_point: Millimeters(25),
            ..Default::default()
        }),
        ..Default::default()
    };
    let rounded = helmet(HelmetDesign::Burgonet(d));
    d.buffe.as_mut().unwrap().ridge_sharpness = Permille(1000);
    let sharp = helmet(HelmetDesign::Burgonet(d));
    assert_eq!(rounded.indices, sharp.indices);
    assert_eq!(
        component(&rounded, ArmorComponentRole::Skull).positions,
        component(&sharp, ArmorComponentRole::Skull).positions
    );
    let a = retained_carriers(&rounded).pop().unwrap();
    let b = retained_carriers(&sharp).pop().unwrap();
    assert!(a.iter().zip(&b).any(|(a, b)| (a[2] - b[2]).abs() > 0.001));
    for (a, b) in a.iter().zip(&b) {
        assert_eq!(
            [a[0], a[1]],
            [b[0], b[1]],
            "ridge control changed jaw width or height"
        );
        if a[0].abs() < 1e-7 {
            assert!((a[2] - b[2]).abs() < 1e-6, "ridge apex moved");
        }
    }
    closed(&component(&sharp, ArmorComponentRole::Buffe));
    d.buffe.as_mut().unwrap().chin_width = Permille(500);
    let narrow = helmet(HelmetDesign::Burgonet(d));
    assert_eq!(narrow.indices, sharp.indices);
    closed(&component(&narrow, ArmorComponentRole::Buffe));
    let bottom = bounds(&component(&sharp, ArmorComponentRole::Buffe), 1).0 + 0.025;
    let span = |points: &[[f32; 3]]| {
        points
            .iter()
            .filter(|p| p[1] < bottom)
            .map(|p| p[0].abs())
            .fold(0.0, f32::max)
    };
    let narrow_carrier = retained_carriers(&narrow).pop().unwrap();
    let narrowed = b
        .iter()
        .zip(&narrow_carrier)
        .map(|(a, b)| a[0].abs() - b[0].abs())
        .fold(0.0, f32::max);
    assert!(
        narrowed > 0.001,
        "chin control narrowed at most{narrowed}m; lower spans{}→{}",
        span(&b),
        span(&narrow_carrier)
    );
    for (chin, sharpness) in [(499, 0), (1101, 0), (500, 1001)] {
        let mut bad = d;
        let guard = bad.buffe.as_mut().unwrap();
        guard.chin_width = Permille(chin);
        guard.ridge_sharpness = Permille(sharpness);
        assert!(HelmetDesign::Burgonet(bad).validate().is_err());
    }
}

#[test]
fn raised_burgonet_peak_retains_a_separate_following_buffe() {
    let mut d = BurgonetDesign {
        buffe: Some(BuffeDesign::default()),
        ..Default::default()
    };
    let flat = helmet(HelmetDesign::Burgonet(d));
    d.peak_rise = Millimeters(20);
    let raised = helmet(HelmetDesign::Burgonet(d));
    assert_eq!(flat.indices, raised.indices);
    let a = component(&flat, ArmorComponentRole::Buffe);
    let b = component(&raised, ArmorComponentRole::Buffe);
    let original = retained_carriers(&flat).pop().unwrap();
    let elevated = retained_carriers(&raised).pop().unwrap();
    let rise = original
        .iter()
        .zip(&elevated)
        .map(|(a, b)| b[1] - a[1])
        .fold(0.0, f32::max);
    assert!(
        rise > 0.019,
        "peak did not raise corresponding buffe rim samples: {rise}m"
    );
    assert!((bounds(&b, 1).0 - bounds(&a, 1).0).abs() < 0.001);
    closed(&raised);
    d.peak_rise = Millimeters(21);
    assert!(HelmetDesign::Burgonet(d).validate().is_err());
}

#[test]
fn wrapped_gap_trims_both_medial_edges_without_changing_the_rear_returns() {
    let build = |gap, side, sign: f32| {
        generate_wrapped_tasset(
            &tassets(WrappedTassetDesign {
                inner_gap: Millimeters(gap),
                ..Default::default()
            }),
            side,
            TassetSpan::new(0.0, 0.5).unwrap(),
            |angle, _height| [sign * 0.02 + 0.10 * angle.sin(), 0.10 * angle.cos()],
        )
        .unwrap()
    };
    for (side, sign) in [(TassetSide::Left, 1.0), (TassetSide::Right, -1.0)] {
        let base = build(0, side, sign);
        for gap in [30, 80] {
            let mesh = build(gap, side, sign);
            assert_eq!(base.indices, mesh.indices);
            assert_eq!(closed(&mesh), 18);
            let a = retained_carriers(&base);
            let b = retained_carriers(&mesh);
            for (a, b) in a.iter().zip(&b) {
                assert!(
                    b.iter()
                        .all(|p| sign * p[0] >= f32::from(gap) * 0.001 - 1e-6)
                );
                let rear = |points: &[[f32; 3]]| {
                    let rear_z = points.iter().map(|p| p[2]).fold(f32::INFINITY, f32::min);
                    points
                        .iter()
                        .filter(|p| (p[2] - rear_z).abs() < 1e-6)
                        .map(|p| key(*p))
                        .collect::<BTreeSet<_>>()
                };
                assert_eq!(rear(a), rear(b), "medial trim moved terminal rear return");
            }
        }
    }
    assert!(
        tassets(WrappedTassetDesign {
            inner_gap: Millimeters(81),
            ..Default::default()
        })
        .validate()
        .is_err()
    );
}

#[test]
fn impossible_wrapped_gap_rejects_the_carrier_instead_of_violating_the_cut() {
    let d = tassets(WrappedTassetDesign {
        inner_gap: Millimeters(80),
        ..Default::default()
    });
    let mesh = generate_wrapped_tasset(
        &d,
        TassetSide::Left,
        TassetSpan::new(0.0, 0.5).unwrap(),
        |angle, _height| [0.04 * angle.sin(), 0.04 * angle.cos()],
    );
    assert!(
        mesh.is_err(),
        "no part of this carrier reaches the required80mm medial edge"
    );
}

#[test]
fn wrapped_tasset_shaped_edges_keep_clearance_on_the_final_conical_section() {
    let radius = |height: f32| 0.060 + 0.4 * (height - 0.600);
    for (rounding, slope, cutaway, split) in [(0, 0, 0, 0), (35, 0, 0, 0), (35, 400, 30, 4)] {
        let mut design = tassets(WrappedTassetDesign {
            inner_gap: Millimeters(0),
            upper_edge_slope: Permille(slope),
            hem_rounding: Millimeters(rounding),
            inner_cutaway: Millimeters(cutaway),
            section_break: split,
            section_gap: Millimeters(8),
            ..Default::default()
        });
        design.lame_count = 8;
        design.fluting = None;
        design.clearance = Millimeters(1);
        design.wall_thickness = Millimeters(1);
        design.flare = Permille(0);
        let mesh = generate_wrapped_tasset(
            &design,
            TassetSide::Left,
            TassetSpan::new(0.6, 0.9).unwrap(),
            |angle, height| {
                let radius = radius(height) + 0.006;
                [radius * angle.sin(), radius * angle.cos()]
            },
        )
        .unwrap();
        assert_eq!(closed(&mesh), 16);
        for point in &mesh.positions {
            let clearance = point[0].hypot(point[2]) - radius(point[1]);
            assert!(
                clearance >= 0.001 - 1e-6,
                "final shell lost clearance: {point:?}, {clearance}"
            );
        }
        for face in mesh.indices.as_chunks::<3>().0 {
            let triangle = face.map(|i| mesh.positions[i as usize]);
            for weights in [
                [0.5, 0.5, 0.0],
                [0.5, 0.0, 0.5],
                [0.0, 0.5, 0.5],
                [1.0 / 3.0; 3],
            ] {
                let point: [f32; 3] = std::array::from_fn(|axis| {
                    (0..3).map(|i| weights[i] * triangle[i][axis]).sum()
                });
                assert!(
                    point[0].hypot(point[2]) - radius(point[1]) >= 0.001 - 1e-6,
                    "finished triangle sample entered its final-height cone: {point:?}"
                );
            }
        }
        if rounding == 35 && split == 0 {
            assert!(
                mesh.positions.iter().any(|p| (p[1] - 0.635).abs() < 0.002),
                "the radius repair must preserve the raised hem"
            );
        }
    }
}

#[test]
fn bellows_change_only_the_separate_visor_and_four_rows_are_real_holes() {
    let mut design = CloseHelmetDesign::default();
    design.breaths.rows = 4;
    design.breaths.count_per_row = 2;
    design.breaths.height = Permille(600);
    design.breaths.row_spacing = Millimeters(14);
    design.breaths.center_offset = Millimeters(30);
    design.breaths.span = Millimeters(20);
    design.breaths.width = Millimeters(2);
    let smooth = helmet(HelmetDesign::CloseHelmet(design));
    design.bellows = Some(VisorBellows::default());
    let folded = helmet(HelmetDesign::CloseHelmet(design));
    for role in [ArmorComponentRole::Skull, ArmorComponentRole::Bevor] {
        let a = component(&smooth, role);
        let b = component(&folded, role);
        assert_eq!(
            a.positions, b.positions,
            "bellows altered a fixed helmet plate"
        );
        assert_eq!(a.indices, b.indices);
    }
    let visor = component(&folded, ArmorComponentRole::Visor);
    assert_ne!(
        component(&smooth, ArmorComponentRole::Visor).positions,
        visor.positions
    );
    // Sixteen breath slots and two bridged sight openings form eighteen handles.
    assert_eq!(closed(&visor), 2 - 2 * 18);
    assert!(
        folded
            .components
            .iter()
            .find(|p| p.role == ArmorComponentRole::Visor)
            .unwrap()
            .hinge
            .is_some()
    );
    let scaled = generate_helmet(
        &HelmetDesign::CloseHelmet(design),
        &frame([0.095, 0.13, 0.12]),
    )
    .unwrap();
    assert_eq!(
        scaled.indices, folded.indices,
        "fit changed pierced visor topology"
    );
    for (a, b) in scaled.components.iter().zip(&folded.components) {
        assert_eq!(
            (a.role, &a.vertices, &a.indices),
            (b.role, &b.vertices, &b.indices)
        );
    }
}

fn tassets(shape: WrappedTassetDesign) -> GarmentArmorDesign {
    GarmentArmorDesign {
        plate_shape: GarmentPlateShape::WrappedTassets(shape),
        lame_count: 9,
        flare: Permille(0),
        ..GarmentArmorDesign::new(GarmentArmorKind::Tassets)
    }
}

fn retained_carriers(mesh: &PartMesh) -> Vec<Vec<[f32; 3]>> {
    let mut carriers = Vec::new();
    mesh.refit_surfaces(|surface, _| carriers.push(surface.to_vec()))
        .unwrap();
    carriers
}

#[test]
fn buffe_breaths_make_closed_openings_and_keep_body_independent_connectivity() {
    let mut design = BurgonetDesign {
        buffe: Some(BuffeDesign::default()),
        ..Default::default()
    };
    let solid = helmet(HelmetDesign::Burgonet(design));
    let solid_skull = component(&solid, ArmorComponentRole::Skull);
    for count in [1, 4] {
        let breaths = VisorBreaths {
            count_per_row: count,
            ..VisorBreaths::buffe()
        };
        design.buffe.as_mut().unwrap().breaths = Some(breaths);
        let pierced = helmet(HelmetDesign::Burgonet(design));
        let guard = component(&pierced, ArmorComponentRole::Buffe);
        assert_eq!(
            closed(&guard),
            2 - 4 * i64::from(count),
            "each slot must be a real through-hole with closed walls"
        );
        assert_eq!(
            component(&pierced, ArmorComponentRole::Skull).positions,
            solid_skull.positions
        );
        let larger =
            generate_helmet(&HelmetDesign::Burgonet(design), &frame([0.105, 0.14, 0.12])).unwrap();
        assert_eq!(larger.indices, pierced.indices);
        assert_eq!(larger.positions.len(), pierced.positions.len());
        assert_eq!(larger.components, pierced.components);
        assert_eq!(
            closed(&component(&larger, ArmorComponentRole::Buffe)),
            2 - 4 * i64::from(count)
        );
        assert_ne!(larger.positions, pierced.positions);
    }
    let breaths = design.buffe.as_mut().unwrap().breaths.as_mut().unwrap();
    breaths.count_per_row = 2;
    breaths.rows = 2;
    breaths.height = Permille(300);
    breaths.width = Millimeters(4);
    breaths.length = Millimeters(10);
    breaths.rounding = Permille(0);
    let square = helmet(HelmetDesign::Burgonet(design));
    assert_eq!(closed(&component(&square, ArmorComponentRole::Buffe)), -14);
    design
        .buffe
        .as_mut()
        .unwrap()
        .breaths
        .as_mut()
        .unwrap()
        .count_per_row = 0;
    let disabled = helmet(HelmetDesign::Burgonet(design));
    assert_eq!(disabled.indices, solid.indices);
    assert_eq!(disabled.positions, solid.positions);
}

#[test]
fn buffe_breaths_reject_colliding_patterns_and_open_edge_cuts() {
    let mut design = BurgonetDesign {
        buffe: Some(BuffeDesign {
            breaths: Some(VisorBreaths::buffe()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let breaths = design.buffe.as_mut().unwrap().breaths.as_mut().unwrap();
    breaths.width = Millimeters(6);
    breaths.count_per_row = 8;
    assert!(HelmetDesign::Burgonet(design).validate().is_err());
    let breaths = design.buffe.as_mut().unwrap().breaths.as_mut().unwrap();
    *breaths = VisorBreaths {
        height: Permille(50),
        length: Millimeters(20),
        ..VisorBreaths::buffe()
    };
    assert!(
        generate_helmet(
            &HelmetDesign::Burgonet(design),
            &frame([0.085, 0.115, 0.105])
        )
        .is_err(),
        "a breath must leave metal between its upper wall and sight edge"
    );
}

#[test]
fn pauldron_wing_depth_survives_short_rounded_returns() {
    for side in [-1.0, 1.0] {
        let fit = PartFrame {
            axes: [[0.0, 0.0, side], [side, 0.0, 0.0], [0.0, 1.0, 0.0]],
            ..frame([0.09, 0.12, 0.09])
        };
        let mut design = PauldronDesign::default();
        design.outline.front_return = Milliradians(2200);
        design.outline.rear_return = Milliradians(2200);
        design.outline.corner_rounding = Permille(320);
        design.outline.wing_start = Milliradians(1000);
        let build = |d: &PauldronDesign| PauldronCarrier::new(d, &fit).unwrap().mesh().unwrap();
        let original = build(&design);
        design.outline.front_extension = Millimeters(80);
        let extended = build(&design);
        assert_eq!(original.indices, extended.indices);
        assert_eq!(closed(&extended), closed(&original));
        let before = retained_carriers(&original);
        let after = retained_carriers(&extended);
        let maximum_drop = before[0]
            .iter()
            .zip(&after[0])
            .map(|(a, b)| a[1] - b[1])
            .fold(0.0_f32, f32::max);
        assert!((0.0795..=0.0801).contains(&maximum_drop));
        let columns = before[0].len() / 25;
        assert_eq!(
            &before[0][..columns],
            &after[0][..columns],
            "hanging wing changed the arm attachment edge"
        );
        assert_eq!(
            &before[1 + usize::from(design.upper_lames)..],
            &after[1 + usize::from(design.upper_lames)..],
            "wing depth changed arm courses"
        );
    }
}

#[test]
fn deep_pauldron_wings_do_not_fold_when_fitted_to_flat_chest_planes() {
    for side in [-1.0, 1.0] {
        let diagonal = std::f32::consts::FRAC_1_SQRT_2;
        let fit = PartFrame {
            axes: [
                [0.0, 0.0, side],
                [side * diagonal, diagonal, 0.0],
                [-side * diagonal, diagonal, 0.0],
            ],
            ..frame([0.085, 0.14, 0.061])
        };
        let mut design = PauldronDesign::default();
        design.outline.front_return = Milliradians(2200);
        design.outline.rear_return = Milliradians(2200);
        design.outline.front_extension = Millimeters(80);
        design.outline.rear_extension = Millimeters(70);
        design.outline.corner_rounding = Permille(320);
        design.outline.front_wing_rounding = Permille(150);
        let mut carrier = PauldronCarrier::new(&design, &fit).unwrap();
        carrier
            .fit(|mut point| {
                if point[0] * side > 0.0 {
                    point[2] = point[2].signum() * point[2].abs().max(0.20);
                }
                point
            })
            .unwrap();
        let mesh = carrier.mesh().unwrap();
        assert_eq!(
            closed(&mesh),
            2 * i64::from(1 + design.upper_lames + design.lower_lames)
        );
        let surfaces = retained_carriers(&mesh);
        let main = &surfaces[0];
        let columns = main.len() / 25;
        let mut signs = [None, None];
        for row in 0..24 {
            for column in 0..columns - 1 {
                let a = main[row * columns + column];
                let b = main[row * columns + column + 1];
                let c = main[(row + 1) * columns + column + 1];
                // Only the returned front/back wings lie on the test planes.
                if a[0] * side <= 0.0
                    || b[0] * side <= 0.0
                    || c[0] * side <= 0.0
                    || a[2].abs() < 0.19
                    || a[2] * b[2] <= 0.0
                    || a[2] * c[2] <= 0.0
                {
                    continue;
                }
                let area = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
                assert!(area.abs() > 1e-10, "collapsed returned wing triangle");
                let half = usize::from(a[2] > 0.0);
                let sign = signs[half].get_or_insert(area.signum());
                assert_eq!(
                    *sign,
                    area.signum(),
                    "returned wing folded over its fitted chest plane: {side} {row} {column}"
                );
            }
        }
        assert!(signs.into_iter().all(|sign| sign.is_some()));
    }
}

#[test]
fn pauldron_returns_edit_one_wing_boundary_and_preserve_the_arm_courses() {
    for side in [-1.0, 1.0] {
        let fit = PartFrame {
            axes: [[0.0, 0.0, side], [side, 0.0, 0.0], [0.0, 1.0, 0.0]],
            ..frame([0.09, 0.12, 0.09])
        };
        let design = PauldronDesign::default();
        let build = |d: &PauldronDesign| PauldronCarrier::new(d, &fit).unwrap().mesh().unwrap();
        let base = build(&design);
        let original = retained_carriers(&base);
        for front in [false, true] {
            for angle in [1800, 2800] {
                let mut changed = design.clone();
                if front {
                    changed.outline.front_return = Milliradians(angle);
                } else {
                    changed.outline.rear_return = Milliradians(angle);
                }
                let mesh = build(&changed);
                assert_eq!(mesh.indices, base.indices);
                assert_eq!(
                    closed(&mesh),
                    2 * i64::from(1 + design.upper_lames + design.lower_lames)
                );
                let carriers = retained_carriers(&mesh);
                assert_eq!(
                    &carriers[1 + usize::from(design.upper_lames)..],
                    &original[1 + usize::from(design.upper_lames)..],
                    "arm articulation changed"
                );
                // Main plate carrier rows follow the angular chart. Its two
                // side borders are identifiable from their world front/back
                // extrema within each row, independently of chart handedness.
                let rows = 25;
                let columns = original[0].len() / rows;
                let mut moved = 0;
                for (a, b) in original[0]
                    .chunks_exact(columns)
                    .zip(carriers[0].chunks_exact(columns))
                {
                    let borders = [0, columns - 1];
                    let front_index = *borders
                        .iter()
                        .max_by(|&&i, &&j| a[i][2].total_cmp(&a[j][2]))
                        .unwrap();
                    let rear_index = columns - 1 - front_index;
                    let (fixed, edited) = if front {
                        (rear_index, front_index)
                    } else {
                        (front_index, rear_index)
                    };
                    assert!(
                        a[fixed]
                            .iter()
                            .zip(b[fixed])
                            .all(|(x, y)| (x - y).abs() < 1e-6),
                        "opposite wing boundary moved"
                    );
                    moved += usize::from(key(a[edited]) != key(b[edited]));
                }
                assert!(moved > rows / 2, "edited wing boundary did not respond");
            }
        }
    }
    for angle in [1799, 2801] {
        for front in [false, true] {
            let mut design = PauldronDesign::default();
            if front {
                design.outline.front_return = Milliradians(angle);
            } else {
                design.outline.rear_return = Milliradians(angle);
            }
            assert!(LimbArmorDesign::Pauldron(design).validate().is_err());
        }
    }
}

#[test]
fn pauldron_hanging_roundness_changes_breadth_without_moving_angular_returns() {
    let fit = PartFrame {
        axes: [[0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        ..frame([0.09, 0.12, 0.09])
    };
    let mut design = PauldronDesign {
        front_reach: Millimeters(150),
        ..Default::default()
    };
    design.outline.corner_rounding = Permille(650);
    design.outline.front_wing_rounding = Permille(700);
    design.outline.front_extension = Millimeters(80);
    design.outline.rear_extension = Millimeters(70);
    let build = |d: &PauldronDesign| PauldronCarrier::new(d, &fit).unwrap().mesh().unwrap();
    let narrow = build(&design);
    design.outline.front_wing_rounding = Permille(150);
    let broad = build(&design);
    assert_eq!(narrow.indices, broad.indices);
    assert_eq!(closed(&narrow), closed(&broad));
    let before = retained_carriers(&narrow);
    let after = retained_carriers(&broad);
    let broadened = before[0]
        .iter()
        .zip(&after[0])
        .filter(|(a, b)| a[1] - b[1] > 0.02)
        .count();
    assert!(broadened > 30, "hanging shield did not broaden");
    for (a, b) in before[0].iter().zip(&after[0]) {
        assert_eq!(a[0], b[0], "rounding changed medial reach");
        assert_eq!(a[2], b[2], "rounding changed angular return");
    }
    let arm_start = 1 + usize::from(design.upper_lames);
    assert!(
        before[arm_start..] == after[arm_start..],
        "rounding changed arm courses"
    );
    design.outline.front_wing_rounding = Permille(1001);
    assert!(
        LimbArmorDesign::Pauldron(design.clone())
            .validate()
            .is_err()
    );
    design.outline.front_wing_rounding = Permille(600);
    design.outline.corner_rounding = Permille(651);
    assert!(LimbArmorDesign::Pauldron(design).validate().is_err());
}

#[test]
fn pauldron_low_regions_move_independently_without_changing_coverage() {
    let fit = PartFrame {
        axes: [[0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        ..frame([0.09, 0.12, 0.09])
    };
    for front in [false, true] {
        let mut design = PauldronDesign::default();
        design.outline.front_extension = Millimeters(80);
        design.outline.rear_extension = Millimeters(80);
        design.outline.front_wing_rounding = Permille(800);
        design.outline.rear_wing_rounding = Permille(800);
        let build = |d: &PauldronDesign| PauldronCarrier::new(d, &fit).unwrap().mesh().unwrap();
        let position = |d: &mut PauldronDesign, value| {
            if front {
                d.outline.front_wing_position = Permille(value);
            } else {
                d.outline.rear_wing_position = Permille(value);
            }
        };
        position(&mut design, 350);
        let outer = build(&design);
        position(&mut design, 700);
        let medial = build(&design);
        assert_eq!(outer.indices, medial.indices);
        assert_eq!(closed(&outer), closed(&medial));
        let before = retained_carriers(&outer);
        let after = retained_carriers(&medial);
        let arm_start = 1 + usize::from(design.upper_lames);
        assert_eq!(
            before[arm_start..],
            after[arm_start..],
            "position moved arm lames"
        );
        let mut changed = 0;
        for (a, b) in before[0].iter().zip(&after[0]) {
            assert_eq!(a[0], b[0], "position changed medial coverage");
            assert_eq!(a[2], b[2], "position changed circumferential coverage");
            if (a[2] > 0.0) != front {
                assert_eq!(a, b, "position changed the opposite wing");
            } else if (a[1] - b[1]).abs() > 0.01 {
                changed += 1;
            }
        }
        assert!(changed > 30, "hanging low region did not move");
        design.outline.front_extension = Millimeters(0);
        design.outline.rear_extension = Millimeters(0);
        let plain = retained_carriers(&build(&design));
        let center = |points: &[[f32; 3]]| {
            let lowest = points
                .iter()
                .zip(&plain[0])
                .filter(|(p, base)| (p[2] > 0.0) == front && base[1] - p[1] > 0.075)
                .map(|(p, _)| p[0])
                .collect::<Vec<_>>();
            assert!(!lowest.is_empty(), "authored wing depth was attenuated");
            lowest.iter().sum::<f32>() / lowest.len() as f32
        };
        assert!(
            center(&after[0]) - center(&before[0]) > 0.015,
            "low region failed to move toward the neck"
        );
        for invalid in [249, 751] {
            position(&mut design, invalid);
            assert!(
                LimbArmorDesign::Pauldron(design.clone())
                    .validate()
                    .is_err()
            );
        }
    }
}

#[test]
fn joint_flute_direction_preserves_plain_carriers_and_morph_correspondence() {
    for construction in [
        JointCupConstruction::Wrapped,
        JointCupConstruction::RaisedCop,
    ] {
        let mut design = JointCupDesign {
            construction,
            ..JointCupDesign::couter()
        };
        let fit = frame([0.065, 0.055, 0.05]);
        let build = |d: &JointCupDesign, fit: &PartFrame| {
            generate_limb_armor(&LimbArmorDesign::Couter(d.clone()), fit).unwrap()
        };
        let plain = build(&design, &fit);
        design.flute_orientation = JointFluteOrientation::Transverse;
        assert_eq!(build(&design, &fit).positions, plain.positions);
        design.fluting = Some(PlateFluting::default());
        let transverse = build(&design, &fit);
        assert_eq!(closed(&transverse), 2);
        let other_body = build(&design, &frame([0.08, 0.06, 0.07]));
        assert_eq!(other_body.indices, transverse.indices);
        assert_eq!(closed(&other_body), 2);
        design.flute_orientation = JointFluteOrientation::Longitudinal;
        let longitudinal = build(&design, &fit);
        assert_eq!(closed(&longitudinal), 2);
        assert_ne!(longitudinal.positions, transverse.positions);
        let encoded = serde_json::to_value(&design).unwrap();
        assert_eq!(
            serde_json::from_value::<JointCupDesign>(encoded).unwrap(),
            design
        );
    }
}

#[test]
fn raised_cop_returns_preserve_the_opposite_side_and_closed_outline() {
    let fit = frame([0.065, 0.055, 0.05]);
    let design = JointCupDesign {
        construction: JointCupConstruction::RaisedCop,
        ..JointCupDesign::couter()
    };
    let build = |d: &JointCupDesign| {
        generate_limb_armor(&LimbArmorDesign::Couter(d.clone()), &fit).unwrap()
    };
    let original = build(&design);
    let before = retained_carriers(&original).remove(0);
    for medial in [false, true] {
        let mut changed = design.clone();
        if medial {
            changed.medial_wrap = Permille(400);
        } else {
            changed.lateral_wrap = Permille(650);
        }
        let mesh = build(&changed);
        assert_eq!(mesh.indices, original.indices);
        assert_eq!(closed(&mesh), 2);
        let after = retained_carriers(&mesh).remove(0);
        let mut moved = 0;
        for (a, b) in before.iter().zip(&after) {
            if (a[0] < 0.0) != medial {
                assert_eq!(a, b, "opposite return moved");
            } else if key(*a) != key(*b) {
                moved += 1;
            }
        }
        assert!(moved > 100, "selected return did not change");
    }
    let mut invalid = design;
    invalid.lateral_wrap = Permille(951);
    assert!(LimbArmorDesign::Couter(invalid).validate().is_err());
}

#[test]
fn distal_taper_retains_the_cup_attachment_and_closed_corresponding_courses() {
    for count in [1, 4] {
        let fit = frame([0.065, 0.06, 0.075]);
        let mut design = JointCupDesign {
            distal_extension: Some(JointExtension {
                lame_count: count,
                ..Default::default()
            }),
            ..Default::default()
        };
        let build = |d: &JointCupDesign| {
            generate_limb_armor(&LimbArmorDesign::Poleyn(d.clone()), &fit).unwrap()
        };
        let base = build(&design);
        let original = retained_carriers(&base);
        for taper in [700, 1100] {
            design.distal_extension.as_mut().unwrap().distal_taper = Permille(taper);
            let mesh = build(&design);
            assert_eq!(mesh.indices, base.indices);
            assert_eq!(closed(&mesh), 2 * i64::from(count + 1));
            assert_eq!(
                component(&mesh, ArmorComponentRole::Plate).positions,
                component(&base, ArmorComponentRole::Plate).positions
            );
            let carriers = retained_carriers(&mesh);
            // The proximal rim of the first lower plate remains registered to
            // the cop. Taper changes girth, never longitudinal reach.
            let first = &original[1];
            let changed = &carriers[1];
            let retained = first
                .iter()
                .zip(changed)
                .filter(|(a, b)| key(**a) == key(**b))
                .count();
            assert!(retained >= 20, "proximal attachment rim was moved");
            let mut moved = 0;
            for (a, b) in original
                .iter()
                .skip(1)
                .flatten()
                .zip(carriers.iter().skip(1).flatten())
            {
                assert_eq!(a[1], b[1], "girth altered longitudinal reach");
                if key(*a) != key(*b) {
                    let radius_a = a[0].hypot(a[2]);
                    let radius_b = b[0].hypot(b[2]);
                    assert_eq!(radius_b > radius_a, taper > 1000);
                    moved += 1;
                }
            }
            assert!(moved > first.len() / 2);
        }
        for taper in [699, 1101] {
            design.distal_extension.as_mut().unwrap().distal_taper = Permille(taper);
            assert!(LimbArmorDesign::Poleyn(design.clone()).validate().is_err());
        }
    }
}

/// On an unfluted constant-radius strip, each X/Z column must run proximally.
/// A rounded hem that doubles back reverses these samples even if its complete
/// shell has a positive total volume and consistent edge incidence.
fn assert_proximal_columns(carrier: &[[f32; 3]]) {
    let mut previous = BTreeMap::new();
    let mut comparisons = 0;
    for point in carrier {
        let column = key([point[0], 0.0, point[2]]);
        if let Some(y) = previous.insert(column, point[1]) {
            assert!(point[1] > y, "longitudinal section turns back at {point:?}");
            comparisons += 1;
        }
    }
    assert!(
        comparisons > carrier.len() / 2,
        "carrier columns were not exercised"
    );
}

#[test]
fn distal_joint_plates_are_independent_closed_shells_with_stable_fit_topology() {
    for cup in [JointCupDesign::poleyn(), JointCupDesign::couter()] {
        let fit = frame([0.065, 0.06, 0.075]);
        let base = generate_limb_armor(&LimbArmorDesign::Poleyn(cup.clone()), &fit).unwrap();
        let mut extended = cup.clone();
        extended.distal_extension = Some(JointExtension::default());
        let mesh = generate_limb_armor(&LimbArmorDesign::Poleyn(extended.clone()), &fit).unwrap();
        assert_eq!(mesh.components.len(), 2);
        let plate = component(&mesh, ArmorComponentRole::Plate);
        assert_eq!(plate.positions, base.positions);
        assert_eq!(plate.indices, base.indices);
        assert_eq!(
            closed(&component(&mesh, ArmorComponentRole::JointExtension)),
            6
        );
        for carrier in retained_carriers(&mesh).iter().skip(1) {
            assert_proximal_columns(carrier);
        }
        let larger = generate_limb_armor(
            &LimbArmorDesign::Poleyn(extended.clone()),
            &frame([0.095, 0.085, 0.105]),
        )
        .unwrap();
        assert_eq!(larger.indices, mesh.indices);
        assert_eq!(larger.positions.len(), mesh.positions.len());
        assert_ne!(larger.positions, mesh.positions);
        assert_eq!(closed(&larger), 8);
        for (a, b) in larger.components.iter().zip(&mesh.components) {
            assert_eq!(
                (a.role, &a.vertices, &a.indices),
                (b.role, &b.vertices, &b.indices)
            );
        }
        extended.distal_extension.as_mut().unwrap().lame_count = 4;
        let four = generate_limb_armor(&LimbArmorDesign::Poleyn(extended), &fit).unwrap();
        assert_eq!(
            closed(&component(&four, ArmorComponentRole::JointExtension)),
            8
        );
    }
}

#[test]
fn raised_cop_projection_changes_the_dish_without_moving_its_rim() {
    let mut design = JointCupDesign {
        construction: JointCupConstruction::RaisedCop,
        wing_roundness: Permille(1000),
        ..JointCupDesign::couter()
    };
    let build = |d: &JointCupDesign, fit: PartFrame| {
        generate_limb_armor(&LimbArmorDesign::Couter(d.clone()), &fit).unwrap()
    };
    let fit = frame([0.065, 0.055, 0.050]);
    let original = build(&design, fit);
    assert_eq!(closed(&original), 2);
    design.dome = Permille(1100);
    let raised = build(&design, fit);
    assert_eq!(original.indices, raised.indices);
    let original_surface = retained_carriers(&original).remove(0);
    let raised_surface = retained_carriers(&raised).remove(0);
    let columns = original_surface.len() / 25;
    let mut maximum_rise = 0.0_f32;
    for (i, (before, after)) in original_surface.iter().zip(&raised_surface).enumerate() {
        if i < columns
            || i >= original_surface.len() - columns
            || i % columns == 0
            || i % columns == columns - 1
        {
            assert!(
                before.iter().zip(after).all(|(a, b)| (a - b).abs() < 1e-6),
                "cop projection moved its perimeter"
            );
        }
        maximum_rise = maximum_rise
            .max(((after[0] - before[0]).powi(2) + (after[2] - before[2]).powi(2)).sqrt());
    }
    assert!(
        maximum_rise > 0.009,
        "cop projection did not raise the central dish"
    );
    assert!(
        original_surface.iter().any(|p| p[2] < -fit.half_extents[2]),
        "the returned fan must cover the front of the elbow"
    );
    for fit in [frame([0.052, 0.045, 0.043]), frame([0.08, 0.065, 0.06])] {
        let changed = build(&design, fit);
        assert_eq!(changed.indices, raised.indices);
        assert_eq!(closed(&changed), 2);
        assert_ne!(changed.positions, raised.positions);
    }
}

#[test]
fn raised_cop_schema_preserves_construction_and_rejects_axial_extensions() {
    let design = JointCupDesign {
        construction: JointCupConstruction::RaisedCop,
        ..JointCupDesign::couter()
    };
    let encoded = serde_json::to_value(&design).unwrap();
    let restored: JointCupDesign = serde_json::from_value(encoded.clone()).unwrap();
    assert_eq!(restored, design);
    let mut missing = encoded.clone();
    missing.as_object_mut().unwrap().remove("construction");
    assert!(serde_json::from_value::<JointCupDesign>(missing).is_err());
    let mut unknown = encoded.clone();
    unknown["unrecognized_shape"] = serde_json::json!(1);
    assert!(serde_json::from_value::<JointCupDesign>(unknown).is_err());
    let mut invalid = encoded;
    invalid["construction"] = serde_json::json!("UnrecognizedCop");
    assert!(serde_json::from_value::<JointCupDesign>(invalid).is_err());
    let mut extended = design;
    extended.distal_extension = Some(JointExtension::default());
    assert!(LimbArmorDesign::Couter(extended).validate().is_err());
}

#[test]
fn raised_cop_fan_spread_does_not_push_its_front_edge_away_from_the_arm() {
    let mut design = JointCupDesign {
        construction: JointCupConstruction::RaisedCop,
        wing: Permille(300),
        ..JointCupDesign::couter()
    };
    let fit = frame([0.065, 0.055, 0.050]);
    let build = |d: &JointCupDesign| {
        generate_limb_armor(&LimbArmorDesign::Couter(d.clone()), &fit).unwrap()
    };
    let narrow = build(&design);
    design.wing = Permille(2000);
    let broad = build(&design);
    assert_eq!(narrow.indices, broad.indices);
    assert_eq!(closed(&broad), 2);
    let narrow_surface = retained_carriers(&narrow).remove(0);
    let broad_surface = retained_carriers(&broad).remove(0);
    let mut spread = 0.0_f32;
    for (before, after) in narrow_surface.iter().zip(&broad_surface) {
        assert!(
            (before[2] - after[2]).abs() < 1e-6,
            "fan width changed its distance in front of the arm"
        );
        spread = spread.max(after[0] - before[0]);
    }
    assert!(spread > 0.1, "fan did not spread beyond the enclosing cup");
    let (_, top) = bounds(&broad, 1);
    let middle_reach = broad_surface
        .iter()
        .filter(|p| p[1].abs() < top * 0.1)
        .map(|p| p[0])
        .fold(f32::NEG_INFINITY, f32::max);
    let upper_reach = broad_surface
        .iter()
        .filter(|p| p[1] > top * 0.8)
        .map(|p| p[0])
        .fold(f32::NEG_INFINITY, f32::max);
    assert!(middle_reach > upper_reach + 0.04, "fan has no median point");
    design.construction = JointCupConstruction::Wrapped;
    assert!(LimbArmorDesign::Couter(design.clone()).validate().is_err());
    design.construction = JointCupConstruction::RaisedCop;
    design.wing = Permille(2001);
    assert!(LimbArmorDesign::Couter(design).validate().is_err());
}

#[test]
fn terminal_share_moves_course_boundaries_without_changing_total_joint_reach() {
    let fit = frame([0.065, 0.06, 0.075]);
    let mut design = JointCupDesign {
        distal_extension: Some(JointExtension::default()),
        ..Default::default()
    };
    let build = |d: &JointCupDesign| {
        generate_limb_armor(&LimbArmorDesign::Poleyn(d.clone()), &fit).unwrap()
    };
    let base = build(&design);
    design.distal_extension.as_mut().unwrap().terminal_share = Permille(800);
    let changed = build(&design);
    let first = component(&base, ArmorComponentRole::JointExtension);
    let second = component(&changed, ArmorComponentRole::JointExtension);
    assert_eq!(first.indices, second.indices);
    assert_ne!(first.positions, second.positions);
    assert!((bounds(&first, 1).0 - bounds(&second, 1).0).abs() < 1e-6);
    assert_eq!(
        component(&changed, ArmorComponentRole::Plate).positions,
        component(&base, ArmorComponentRole::Plate).positions
    );
    design.distal_extension.as_mut().unwrap().length = Millimeters(170);
    let longer = component(&build(&design), ArmorComponentRole::JointExtension);
    assert_eq!(longer.indices, second.indices);
    assert!(bounds(&longer, 1).0 < bounds(&second, 1).0 - 0.039);
}

#[test]
fn joint_extension_rejects_reversed_hems_and_keeps_near_limit_sections_monotone() {
    let mut design = JointCupDesign {
        distal_extension: Some(JointExtension {
            length: Millimeters(40),
            lame_count: 4,
            terminal_share: Permille(350),
            hem_rounding: Millimeters(25),
            ..Default::default()
        }),
        ..Default::default()
    };
    let fit = frame([0.065, 0.06, 0.075]);
    assert!(
        generate_limb_armor(&LimbArmorDesign::Poleyn(design.clone()), &fit).is_err(),
        "a hem longer than its terminal plate's monotone domain must be rejected"
    );
    design.distal_extension.as_mut().unwrap().hem_rounding = Millimeters(3);
    for count in [1, 4] {
        design.distal_extension.as_mut().unwrap().lame_count = count;
        for gauge in [1, 6] {
            design.gauge.thickness = Millimeters(gauge);
            let mesh = generate_limb_armor(&LimbArmorDesign::Poleyn(design.clone()), &fit).unwrap();
            assert_eq!(
                closed(&component(&mesh, ArmorComponentRole::JointExtension)),
                2 * i64::from(count)
            );
            for carrier in retained_carriers(&mesh).iter().skip(1) {
                assert_proximal_columns(carrier);
            }
        }
    }
    for count in [0, 5] {
        design.distal_extension.as_mut().unwrap().lame_count = count;
        assert!(LimbArmorDesign::Poleyn(design.clone()).validate().is_err());
    }
    design.distal_extension.as_mut().unwrap().lame_count = 3;
    for gauge in [0, 7] {
        design.gauge.thickness = Millimeters(gauge);
        assert!(generate_limb_armor(&LimbArmorDesign::Poleyn(design.clone()), &fit).is_err());
    }
}

fn arched_fauld(width: u16, height: u16) -> GarmentArmorDesign {
    GarmentArmorDesign {
        plate_shape: GarmentPlateShape::Fauld {
            waist_rise: Millimeters(50),
            front_arch: Permille(height),
            front_arch_width: Permille(width),
            chevron_slope: Permille(0),
        },
        flare: Permille(0),
        lame_count: 1,
        ..GarmentArmorDesign::new(GarmentArmorKind::Fauld)
    }
}

#[test]
fn fauld_arch_width_changes_only_the_front_opening_and_keeps_sections_upright() {
    let fit = frame([0.20, 0.08, 0.12]);
    let narrow = generate_garment_armor(&arched_fauld(250, 950), &fit).unwrap();
    let wide = generate_garment_armor(&arched_fauld(800, 950), &fit).unwrap();
    assert_eq!(narrow.indices, wide.indices);
    assert_eq!(closed(&narrow), 0, "a closed annular course is a torus");
    assert_eq!(closed(&wide), 0);
    let a = retained_carriers(&narrow).remove(0);
    let b = retained_carriers(&wide).remove(0);
    assert_proximal_columns(&a);
    assert_proximal_columns(&b);
    let mut broadened = 0;
    for (narrow, wide) in a.iter().zip(&b) {
        assert_eq!([narrow[0], narrow[2]], [wide[0], wide[2]]);
        if narrow[2] < 0.0
            || narrow[0].abs() < 1e-6
            || (narrow[1] - fit.half_extents[1]).abs() < 1e-6
        {
            assert!(
                (narrow[1] - wide[1]).abs() < 1e-6,
                "arch width moved the rear, center height or waist rim"
            );
        } else {
            assert!(wide[1] >= narrow[1] - 1e-6);
            broadened += usize::from(wide[1] > narrow[1] + 0.005);
        }
    }
    assert!(
        broadened > 20,
        "wider opening did not raise its flanking edge"
    );
    let fitted =
        generate_garment_armor(&arched_fauld(800, 950), &frame([0.26, 0.10, 0.16])).unwrap();
    assert_eq!(wide.indices, fitted.indices);
    closed(&fitted);
    assert_proximal_columns(&retained_carriers(&fitted)[0]);
}

#[test]
fn fauld_arch_and_plate_gauge_limits_reject_invalid_material_geometry() {
    let fit = frame([0.20, 0.08, 0.12]);
    for (width, height) in [(249, 500), (801, 500), (500, 1000)] {
        assert!(generate_garment_armor(&arched_fauld(width, height), &fit).is_err());
    }
    for thickness in [0, 17] {
        let mut design = arched_fauld(500, 500);
        design.wall_thickness = Millimeters(thickness);
        assert!(generate_garment_armor(&design, &fit).is_err());
    }
    for count in [0, 9] {
        let mut design = arched_fauld(500, 500);
        design.lame_count = count;
        assert!(generate_garment_armor(&design, &fit).is_err());
    }
    for (clearance, thickness) in [(1, 2), (26, 2), (10, 0), (10, 7)] {
        let gauge = PlateGauge {
            clearance: Millimeters(clearance),
            thickness: Millimeters(thickness),
        };
        assert!(
            generate_besagew(&BesagewDesign::default(), gauge).is_err(),
            "public besagew generation bypassed the plate gauge domain"
        );
    }
}

#[test]
fn wrapped_tassets_keep_closed_courses_and_mirrored_return_orientation() {
    let design = tassets(WrappedTassetDesign::default());
    let build = |side, sign: f32| {
        generate_wrapped_tasset(
            &design,
            side,
            TassetSpan::new(0.0, 0.55).unwrap(),
            |angle, _height| [sign * 0.12 + 0.085 * angle.sin(), 0.09 * angle.cos()],
        )
        .unwrap()
    };
    let left = build(TassetSide::Left, 1.0);
    let right = build(TassetSide::Right, -1.0);
    assert_eq!(closed(&left), 18);
    assert_eq!(closed(&right), 18);
    // Compare the retained carriers, before the area-weighted normal offsets.
    // Reversing angular order reflects the quad corners but changes its chosen
    // diagonal. Nonplanar quads therefore have slightly different normal-based
    // extrusion positions; that is not a reflected carrier-shape discrepancy.
    let carriers = |mesh: &PartMesh| {
        let mut positions = Vec::new();
        mesh.refit_surfaces(|surface, _| positions.extend_from_slice(surface))
            .unwrap();
        positions
    };
    let left_carrier = carriers(&left);
    let right_carrier = carriers(&right);
    assert_eq!(left_carrier.len(), right_carrier.len());
    let maximum = left_carrier
        .iter()
        .map(|p| {
            let reflected = [-p[0], p[1], p[2]];
            right_carrier
                .iter()
                .map(|q| (0..3).map(|i| (q[i] - reflected[i]).powi(2)).sum::<f32>())
                .fold(f32::INFINITY, f32::min)
                .sqrt()
        })
        .fold(0.0, f32::max);
    assert!(
        maximum < 1e-5,
        "maximum reflected tasset discrepancy is {maximum} metres"
    );
}

#[test]
fn nominal_wrapped_controls_change_shape_without_changing_correspondence() {
    let shape = WrappedTassetDesign::default();
    let base = tassets(shape);
    let fit = frame([0.22, 0.3, 0.10]);
    let reference = generate_garment_armor(&base, &fit).unwrap();
    let mut variants = Vec::new();
    let mut changed = shape;
    changed.inner_gap = Millimeters(55);
    variants.push(tassets(changed));
    let mut changed = shape;
    changed.knee_reach = Permille(1050);
    variants.push(tassets(changed));
    let mut changed = shape;
    changed.upper_edge_slope = Permille(250);
    variants.push(tassets(changed));
    let mut changed = shape;
    changed.inner_cutaway = Millimeters(35);
    variants.push(tassets(changed));
    let mut changed = shape;
    changed.hem_rounding = Millimeters(30);
    variants.push(tassets(changed));
    let mut changed = shape;
    changed.section_gap = Millimeters(7);
    variants.push(tassets(changed));
    let mut changed = base.clone();
    changed.flare = Permille(200);
    variants.push(changed);
    for variant in variants {
        let mesh = generate_garment_armor(&variant, &fit).unwrap();
        assert_eq!(mesh.indices, reference.indices);
        assert_eq!(mesh.positions.len(), reference.positions.len());
        let maximum = mesh
            .positions
            .iter()
            .zip(&reference.positions)
            .flat_map(|(a, b)| (0..3).map(move |i| (a[i] - b[i]).abs()))
            .fold(0.0, f32::max);
        assert!(
            maximum > 0.001,
            "nominal recipe control was ignored: {variant:?}"
        );
        closed(&mesh);
    }
}

#[test]
fn invalid_constructions_are_rejected_before_building_geometry() {
    let disc = BesagewDesign {
        fluting: Some(RadialFluting {
            count: FluteCount(0),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert!(generate_besagew(&disc, PlateGauge::default()).is_err());
    let burgonet = BurgonetDesign {
        buffe: Some(BuffeDesign {
            sight_gap: Millimeters(0),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert!(
        generate_helmet(
            &HelmetDesign::Burgonet(burgonet),
            &frame([0.085, 0.115, 0.105])
        )
        .is_err()
    );
    let close = CloseHelmetDesign {
        bellows: Some(VisorBellows {
            count: 0,
            ..Default::default()
        }),
        ..Default::default()
    };
    assert!(HelmetDesign::CloseHelmet(close).validate().is_err());
    let invalid = tassets(WrappedTassetDesign {
        section_break: 9,
        ..Default::default()
    });
    assert!(generate_garment_armor(&invalid, &frame([0.22, 0.3, 0.10])).is_err());
    let breast = BreastplateDesign {
        construction: BreastplateConstruction::Anime(AnimeDesign {
            lap_lift: Millimeters(4),
            ..Default::default()
        }),
        wall_thickness: Millimeters(3),
        ..Default::default()
    };
    assert!(
        validate_breastplate(&breast).is_err(),
        "overlapping courses lack gauge allowance"
    );
}

// Local copy of the established torso fixture, without its unrelated tests.
mod torso_fixture {
    use adventuresim_armor_model::{
        SurfaceMorph, TORSO_SHOULDER_ENVELOPE_SAMPLES, TorsoClearanceMesh, TorsoClearancePose,
        TorsoCoronalAnchor, TorsoShoulderSample, TorsoSurface, TorsoUpperRigAnchors, TorsoVertex,
    };

    fn patch_normals(positions: &[[f32; 3]], faces: &[[u32; 3]]) -> Vec<[f32; 3]> {
        let mut normals = vec![[0.0; 3]; positions.len()];
        for face in faces {
            let [a, b, c] = face.map(|i| positions[i as usize]);
            let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let normal = [
                u[1] * v[2] - u[2] * v[1],
                u[2] * v[0] - u[0] * v[2],
                u[0] * v[1] - u[1] * v[0],
            ];
            for index in face {
                for axis in 0..3 {
                    normals[*index as usize][axis] += normal[axis];
                }
            }
        }
        for normal in &mut normals {
            let length = normal.iter().map(|n| n * n).sum::<f32>().sqrt();
            assert!(length > 0.0);
            *normal = normal.map(|n| n / length);
        }
        normals
    }

    pub(super) fn torso() -> TorsoSurface {
        let columns = 13;
        let torso_rows = 13;
        // Keep the original torso stations, then extend actual clearance support
        // above the neck/shoulder anchors. The old fixture ended at y=0.576m,
        // below its own upper garment boundary (~0.613m), so exact body rays
        // correctly rejected it instead of extrapolating an imaginary shoulder.
        let rows = torso_rows + 2;
        let vertical_span = (rows - 1) as f32 / (torso_rows - 1) as f32 * 1.5;
        let mut vertices = Vec::new();
        for row in 0..rows {
            let vertical = row as f32 / (torso_rows - 1) as f32 * 1.5 - 0.3;
            for column in 0..columns {
                let lateral = column as f32 / (columns - 1) as f32 * 2.4 - 1.2;
                let breast_band = (-((vertical - 0.68) / 0.16).powi(2)).exp();
                let paired_breasts = (-((lateral.abs() - 0.42) / 0.18).powi(2)).exp();
                let section = (1.0 - (lateral / 1.30).powi(2)).max(0.0).sqrt();
                let chest_depth = 0.015 + section * (0.100 + breast_band * paired_breasts * 0.010);
                // Provide an actual superior-facing shoulder roof. A front-only
                // plane with fabricated envelope normals cannot exercise the
                // anatomical crest exit used by the angular surface fitter.
                let y = 0.98 + (vertical + 0.30) / 1.75 * 0.50;
                let roof_t = ((y - 1.38) / 0.08).clamp(0.0, 1.0);
                let roof_blend = roof_t * roof_t * (3.0 - 2.0 * roof_t);
                let roof_depth = 0.065 - 0.35 * (y - 1.42) - lateral * lateral * 0.004;
                let depth = chest_depth + roof_blend * (roof_depth - chest_depth);
                vertices.push(TorsoVertex {
                    uv: [(lateral + 1.2) / 2.4, (vertical + 0.3) / vertical_span],
                    lateral,
                    vertical,
                    position: [lateral * 0.155, y, depth],
                    normal: [0.0, 0.0, 1.0],
                    joint_indices: [0; 8],
                    joint_weights: [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                });
            }
        }
        let mut faces = Vec::new();
        for row in 0..rows - 1 {
            for column in 0..columns - 1 {
                let a = (row * columns + column) as u32;
                let b = a + 1;
                let d = ((row + 1) * columns + column) as u32;
                let c = d + 1;
                faces.extend([[a, b, c], [a, c, d]]);
            }
        }
        let base_positions = vertices.iter().map(|v| v.position).collect::<Vec<_>>();
        for (vertex, normal) in vertices
            .iter_mut()
            .zip(patch_normals(&base_positions, &faces))
        {
            vertex.normal = normal;
        }
        let mut morph = SurfaceMorph {
            name: "chest_depth".into(),
            positions: vertices
                .iter()
                .map(|vertex| {
                    let center = 1.0 - vertex.lateral.abs().min(1.0);
                    [
                        vertex.position[0] * 1.08,
                        1.425 + (vertex.position[1] - 1.425) * 1.03,
                        0.015 + (vertex.position[2] - 0.015) * 1.08 + center * 0.010,
                    ]
                })
                .collect(),
            normals: vertices.iter().map(|vertex| vertex.normal).collect(),
        };
        morph.normals = patch_normals(&morph.positions, &faces);
        let shoulder_envelope = (0..TORSO_SHOULDER_ENVELOPE_SAMPLES)
            .map(|sample| {
                let signed =
                    sample as f32 / (TORSO_SHOULDER_ENVELOPE_SAMPLES - 1) as f32 * 2.0 - 1.0;
                let absolute = signed.abs();
                TorsoShoulderSample {
                    position: [
                        signed * 0.10,
                        1.445 + absolute * 0.045 - absolute * absolute * 0.025,
                        0.045 - absolute * 0.020,
                    ],
                    normal: [0.0, 0.82, 0.57],
                }
            })
            .collect::<Vec<_>>();
        let morph_shoulders = shoulder_envelope
            .iter()
            .map(|sample| TorsoShoulderSample {
                position: [
                    sample.position[0] * 1.08,
                    1.425 + (sample.position[1] - 1.425) * 1.03,
                    0.015 + (sample.position[2] - 0.015) * 1.08,
                ],
                normal: sample.normal,
            })
            .collect::<Vec<_>>();
        let clearance = vertices
            .iter()
            .map(|vertex| TorsoShoulderSample {
                position: vertex.position,
                normal: vertex.normal,
            })
            .collect::<Vec<_>>();
        let morph_clearance: Vec<TorsoShoulderSample> = morph
            .positions
            .iter()
            .zip(&morph.normals)
            .map(|(position, normal)| TorsoShoulderSample {
                position: *position,
                normal: *normal,
            })
            .collect();
        let ring_columns = 32_usize;
        let mut enclosure_vertices = Vec::new();
        let mut morph_enclosure_vertices = Vec::new();
        let mut enclosure_texcoords = Vec::new();
        let mut enclosure_joint_indices = Vec::new();
        let mut enclosure_joint_weights = Vec::new();
        for row in 0..rows {
            let y = 0.98 + row as f32 / (rows - 1) as f32 * 0.50;
            let taper = 1.0 - 0.18 * ((y - 1.38) / 0.10).clamp(0.0, 1.0);
            for column in 0..ring_columns {
                let angle = column as f32 / ring_columns as f32 * std::f32::consts::TAU;
                let normal = [angle.sin(), 0.0, angle.cos()];
                let position = [0.090 * taper * angle.sin(), y, 0.015 + 0.050 * angle.cos()];
                enclosure_vertices.push(TorsoShoulderSample { position, normal });
                morph_enclosure_vertices.push(TorsoShoulderSample {
                    position: [
                        position[0] * 1.08,
                        1.425 + (y - 1.425) * 1.03,
                        0.015 + (position[2] - 0.015) * 1.10,
                    ],
                    normal,
                });
                let rear = angle.cos() < 0.0;
                enclosure_texcoords.push([
                    column as f32 / ring_columns as f32 + if rear { 2.0 } else { 0.0 },
                    row as f32 / (rows - 1) as f32,
                ]);
                enclosure_joint_indices.push(if rear {
                    [1, 0, 0, 0, 0, 0, 0, 0]
                } else {
                    [0; 8]
                });
                enclosure_joint_weights.push([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
            }
        }
        let mut enclosure_faces = Vec::new();
        for row in 0..rows - 1 {
            for column in 0..ring_columns {
                let next = (column + 1) % ring_columns;
                let a = (row * ring_columns + column) as u32;
                let b = (row * ring_columns + next) as u32;
                let d = ((row + 1) * ring_columns + column) as u32;
                let c = ((row + 1) * ring_columns + next) as u32;
                enclosure_faces.extend([[a, b, c], [a, c, d]]);
            }
        }
        let bottom_center = enclosure_vertices.len() as u32;
        let top_center = bottom_center + 1;
        for (y, normal) in [(0.98, [0.0, -1.0, 0.0]), (1.48, [0.0, 1.0, 0.0])] {
            enclosure_vertices.push(TorsoShoulderSample {
                position: [0.0, y, 0.015],
                normal,
            });
            morph_enclosure_vertices.push(TorsoShoulderSample {
                position: [0.0, 1.425 + (y - 1.425) * 1.03, 0.015],
                normal,
            });
            enclosure_texcoords.push([0.5, if y < 1.0 { 0.0 } else { 1.0 }]);
            enclosure_joint_indices.push([0; 8]);
            enclosure_joint_weights.push([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        }
        for column in 0..ring_columns {
            let next = (column + 1) % ring_columns;
            enclosure_faces.push([bottom_center, next as u32, column as u32]);
            let top = ((rows - 1) * ring_columns) as u32;
            enclosure_faces.push([top_center, top + column as u32, top + next as u32]);
        }
        let enclosure_texcoord_faces = enclosure_faces.clone();
        let morph_semantic_coordinates = vec![
            vertices
                .iter()
                .map(|vertex| [vertex.lateral, vertex.vertical])
                .collect(),
        ];
        TorsoSurface {
            domain: "test_body_v2".into(),
            front: [0.0, 0.0, 1.0],
            morph_fronts: vec![[0.0, 0.0, 1.0]],
            upper_rig_anchors: TorsoUpperRigAnchors {
                neck_base: [0.0, 1.425, 0.055],
                clavicles: [[-0.05, 1.450, 0.045], [0.05, 1.450, 0.045]],
                shoulders: [[-0.08, 1.465, 0.020], [0.08, 1.465, 0.020]],
            },
            morph_upper_rig_anchors: vec![TorsoUpperRigAnchors {
                neck_base: [0.0, 1.425, 0.065],
                clavicles: [[-0.054, 1.45075, 0.0574], [0.054, 1.45075, 0.0574]],
                shoulders: [[-0.0864, 1.4662, 0.0204], [0.0864, 1.4662, 0.0204]],
            }],
            morph_semantic_coordinates,
            vertices: vertices.clone(),
            faces: faces.clone(),
            coronal_anchors: vec![
                TorsoCoronalAnchor {
                    vertical: 0.20,
                    depth: 0.015,
                },
                TorsoCoronalAnchor {
                    vertical: 0.45,
                    depth: 0.015,
                },
                TorsoCoronalAnchor {
                    vertical: 0.70,
                    depth: 0.015,
                },
                TorsoCoronalAnchor {
                    vertical: 0.92,
                    depth: 0.015,
                },
            ],
            morph_coronal_depths: vec![vec![0.015; 4]],
            shoulder_envelope,
            morph_shoulder_envelopes: vec![morph_shoulders],
            clearance_mesh: TorsoClearanceMesh {
                base: TorsoClearancePose {
                    vertices: clearance.clone(),
                    enclosure_vertices,
                },
                faces: faces.clone(),
                enclosure_faces: enclosure_faces.clone(),
                enclosure_torso_faces: enclosure_faces,
                enclosure_texcoords,
                enclosure_texcoord_faces,
                enclosure_joint_indices,
                enclosure_joint_weights,
                morphs: vec![TorsoClearancePose {
                    vertices: morph_clearance.clone(),
                    enclosure_vertices: morph_enclosure_vertices,
                }],
            },
            morphs: vec![morph],
        }
    }
}

#[test]
fn anime_courses_remain_closed_and_keep_all_morph_channels_registered() {
    let surface = torso_fixture::torso();
    let mut design = BreastplateDesign {
        wall_thickness: Millimeters(2),
        construction: BreastplateConstruction::Anime(AnimeDesign {
            articulated_height: Permille(950),
            ..Default::default()
        }),
        ..Default::default()
    };
    let armor = generate_breastplate(&design, &surface).unwrap();
    let mut mesh = PartMesh::new();
    mesh.positions = armor.positions.clone();
    mesh.indices = armor.indices.clone();
    assert_eq!(
        closed(&mesh),
        2 * 2 * 7,
        "each front/rear course is a closed shell"
    );
    assert_eq!(armor.positions.len(), armor.texcoords.len());
    assert_eq!(armor.positions.len(), armor.joint_indices.len());
    assert_eq!(armor.positions.len(), armor.joint_weights.len());
    assert!(!armor.morphs.is_empty());
    for target in &armor.morphs {
        assert_eq!(target.direct_positions.len(), armor.positions.len());
        assert_eq!(target.position_deltas.len(), armor.positions.len());
        assert_eq!(target.normal_deltas.len(), armor.positions.len());
        for ((base, delta), direct) in armor
            .positions
            .iter()
            .zip(&target.position_deltas)
            .zip(&target.direct_positions)
        {
            for axis in 0..3 {
                assert!((base[axis] + delta[axis] - direct[axis]).abs() < 1e-6);
            }
        }
        mesh.positions = target.direct_positions.clone();
        assert_eq!(closed(&mesh), 28);
    }
    if let BreastplateConstruction::Anime(anime) = &mut design.construction {
        anime.chevron_slope = Permille(450);
        anime.rear_chevron_slope = Permille(300);
    }
    let shaped = generate_breastplate(&design, &surface).unwrap();
    assert_eq!(armor.indices, shaped.indices);
    assert_ne!(
        armor.positions, shaped.positions,
        "chevron controls were ignored"
    );
    if let BreastplateConstruction::Anime(anime) = &mut design.construction {
        anime.lame_count = 4;
    }
    let fewer = generate_breastplate(&design, &surface).unwrap();
    mesh.positions = fewer.positions;
    mesh.indices = fewer.indices;
    assert_eq!(closed(&mesh), 20);
    if let BreastplateConstruction::Anime(anime) = &mut design.construction {
        anime.articulated_height = Permille(951);
    }
    assert!(validate_breastplate(&design).is_err());
}

#[test]
fn buffe_courses_are_separate_closed_plates_with_independent_boundaries() {
    let mut design = BurgonetDesign {
        buffe: Some(BuffeDesign {
            courses: Some(BuffeCourses::default()),
            ridge_sharpness: Permille(1000),
            ..Default::default()
        }),
        ..Default::default()
    };
    let original = helmet(HelmetDesign::Burgonet(design));
    assert_eq!(closed(&component(&original, ArmorComponentRole::Buffe)), 6);
    let carriers = retained_carriers(&original);
    let count = carriers.len();
    design
        .buffe
        .as_mut()
        .unwrap()
        .courses
        .as_mut()
        .unwrap()
        .upper_boundary = Permille(800);
    let raised = helmet(HelmetDesign::Burgonet(design));
    let raised_carriers = retained_carriers(&raised);
    assert_eq!(raised_carriers[count - 3], carriers[count - 3]);
    assert_ne!(raised_carriers[count - 2], carriers[count - 2]);
    assert_ne!(raised_carriers[count - 1], carriers[count - 1]);
    design
        .buffe
        .as_mut()
        .unwrap()
        .courses
        .as_mut()
        .unwrap()
        .upper_boundary = Permille(700);
    design
        .buffe
        .as_mut()
        .unwrap()
        .courses
        .as_mut()
        .unwrap()
        .lower_boundary = Permille(450);
    let raised_lower = helmet(HelmetDesign::Burgonet(design));
    let raised_lower_carriers = retained_carriers(&raised_lower);
    assert_ne!(raised_lower_carriers[count - 3], carriers[count - 3]);
    assert_ne!(raised_lower_carriers[count - 2], carriers[count - 2]);
    assert_eq!(raised_lower_carriers[count - 1], carriers[count - 1]);
    for plates in [2, 3] {
        design
            .buffe
            .as_mut()
            .unwrap()
            .courses
            .as_mut()
            .unwrap()
            .plate_count = plates;
        let fitted = helmet(HelmetDesign::Burgonet(design));
        assert_eq!(
            closed(&component(&fitted, ArmorComponentRole::Buffe)),
            2 * i64::from(plates)
        );
        assert_eq!(
            component(&fitted, ArmorComponentRole::Skull).positions,
            component(&original, ArmorComponentRole::Skull).positions
        );
        let larger =
            generate_helmet(&HelmetDesign::Burgonet(design), &frame([0.105, 0.14, 0.12])).unwrap();
        assert_eq!(larger.indices, fitted.indices);
        assert_eq!(larger.components, fitted.components);
        assert_ne!(larger.positions, fitted.positions);
    }
}

#[test]
fn buffe_course_laps_change_the_covered_edge_and_keep_the_exposed_top() {
    let mut design = BurgonetDesign {
        buffe: Some(BuffeDesign {
            courses: Some(BuffeCourses::default()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let original = helmet(HelmetDesign::Burgonet(design));
    let carriers = retained_carriers(&original);
    design
        .buffe
        .as_mut()
        .unwrap()
        .courses
        .as_mut()
        .unwrap()
        .overlap = Millimeters(10);
    let wider = helmet(HelmetDesign::Burgonet(design));
    let wider_carriers = retained_carriers(&wider);
    let first = carriers.len() - 3;
    assert_eq!(carriers[first], wider_carriers[first]);
    let mut single_design = design;
    single_design.buffe.as_mut().unwrap().courses = None;
    let single_carriers = retained_carriers(&helmet(HelmetDesign::Burgonet(single_design)));
    let single = single_carriers.last().unwrap();
    let top = carriers.last().unwrap();
    assert_eq!(
        &top[top.len() - 41..],
        &single[single.len() - 41..],
        "courses must not widen or project the original sight edge"
    );
    for index in first + 1..carriers.len() {
        let a = &carriers[index];
        let b = &wider_carriers[index];
        // The central median lies at the middle of each transverse row.
        let edge_vertices = 41;
        assert_eq!(&a[a.len() - edge_vertices..], &b[b.len() - edge_vertices..]);
        for (a, b) in a[..edge_vertices].iter().zip(&b[..edge_vertices]) {
            assert!((a[1] - b[1] - 0.005).abs() < 1e-6);
        }
    }
    assert_eq!(closed(&component(&wider, ArmorComponentRole::Buffe)), 6);
}

#[test]
fn buffe_courses_pierce_only_the_upper_plate_and_reject_cuts_across_its_edge() {
    let mut design = BurgonetDesign {
        buffe: Some(BuffeDesign {
            courses: Some(BuffeCourses::default()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let solid = helmet(HelmetDesign::Burgonet(design));
    let carriers = retained_carriers(&solid);
    design.buffe.as_mut().unwrap().breaths = Some(VisorBreaths::buffe());
    let pierced = helmet(HelmetDesign::Burgonet(design));
    assert_eq!(
        closed(&component(&pierced, ArmorComponentRole::Buffe)),
        6 - 2 * 8
    );
    let pierced_carriers = retained_carriers(&pierced);
    for index in carriers.len() - 3..carriers.len() - 1 {
        assert_eq!(carriers[index], pierced_carriers[index]);
    }
    let larger =
        generate_helmet(&HelmetDesign::Burgonet(design), &frame([0.105, 0.14, 0.12])).unwrap();
    assert_eq!(larger.indices, pierced.indices);
    assert_eq!(larger.components, pierced.components);
    design
        .buffe
        .as_mut()
        .unwrap()
        .breaths
        .as_mut()
        .unwrap()
        .height = Permille(600);
    assert!(
        generate_helmet(
            &HelmetDesign::Burgonet(design),
            &frame([0.085, 0.115, 0.105])
        )
        .is_err()
    );
    design.buffe.as_mut().unwrap().breaths = None;
    design
        .buffe
        .as_mut()
        .unwrap()
        .courses
        .as_mut()
        .unwrap()
        .upper_boundary = Permille(450);
    assert!(HelmetDesign::Burgonet(design).validate().is_err());
}

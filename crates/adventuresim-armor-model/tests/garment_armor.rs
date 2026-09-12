use std::collections::BTreeMap;

use adventuresim_armor_model::{
    GarmentArmorDesign, GarmentArmorKind, Millimeters, PartFrame, PartMesh, Permille,
    generate_garment_armor,
};

const KINDS: [GarmentArmorKind; 13] = [
    GarmentArmorKind::ArmingDoublet,
    GarmentArmorKind::Brigandine,
    GarmentArmorKind::JackOfPlates,
    GarmentArmorKind::Fauld,
    GarmentArmorKind::MailChausses,
    GarmentArmorKind::MailShirt,
    GarmentArmorKind::MailSkirt,
    GarmentArmorKind::MailSleeve,
    GarmentArmorKind::PaddedChausses,
    GarmentArmorKind::PaddedSkirt,
    GarmentArmorKind::QuiltedSleeve,
    GarmentArmorKind::Tassets,
    GarmentArmorKind::Gorget,
];

#[test]
fn standalone_fauld_exposes_its_whole_attachment_component() {
    for count in [1, 6, 8] {
        let mut design = GarmentArmorDesign::new(GarmentArmorKind::Fauld);
        design.lame_count = count;
        let mesh = generate_garment_armor(&design, &frame(design.kind)).unwrap();
        assert_eq!(mesh.components.len(), 1);
        let component = &mesh.components[0];
        assert_eq!(
            component.role,
            adventuresim_armor_model::ArmorComponentRole::Fauld
        );
        assert_eq!(component.vertices, 0..mesh.positions.len());
        assert_eq!(component.indices, 0..mesh.indices.len());
    }
}

fn frame(kind: GarmentArmorKind) -> PartFrame {
    let half_extents = match kind {
        GarmentArmorKind::Gorget => [0.065, 0.055, 0.060],
        GarmentArmorKind::MailSleeve | GarmentArmorKind::QuiltedSleeve => [0.060, 0.280, 0.060],
        GarmentArmorKind::MailChausses | GarmentArmorKind::PaddedChausses => [0.100, 0.430, 0.110],
        GarmentArmorKind::Fauld
        | GarmentArmorKind::Tassets
        | GarmentArmorKind::MailSkirt
        | GarmentArmorKind::PaddedSkirt => [0.180, 0.120, 0.140],
        _ => [0.210, 0.240, 0.125],
    };
    PartFrame {
        origin: [0.0; 3],
        axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        half_extents,
    }
}

fn assert_solid(mesh: &PartMesh) {
    mesh.normals().expect("finite nondegenerate geometry");
    let mut edges = BTreeMap::<(u32, u32), Vec<(u32, u32)>>::new();
    let mut volume = 0.0;
    for triangle in mesh.indices.as_chunks::<3>().0 {
        let [a, b, c] = [triangle[0], triangle[1], triangle[2]];
        for (start, end) in [(a, b), (b, c), (c, a)] {
            edges
                .entry((start.min(end), start.max(end)))
                .or_default()
                .push((start, end));
        }
        let [p, q, r] = [a, b, c].map(|index| mesh.positions[index as usize]);
        volume += f64::from(
            p[0] * (q[1] * r[2] - q[2] * r[1])
                + p[1] * (q[2] * r[0] - q[0] * r[2])
                + p[2] * (q[0] * r[1] - q[1] * r[0]),
        ) / 6.0;
    }
    assert!(
        volume > 0.0,
        "shells must enclose positive physical volume: {volume}"
    );
    for incidences in edges.values() {
        assert_eq!(incidences.len(), 2, "each edge has exactly two faces");
        assert_eq!(
            incidences[0],
            (incidences[1].1, incidences[1].0),
            "neighboring triangles must have opposite edge winding"
        );
    }
}

#[test]
fn dense_fauld_flutes_keep_resolved_crests_and_closed_plates() {
    use adventuresim_armor_model::{FluteCount, PlateFluting};
    let mut design = GarmentArmorDesign::new(GarmentArmorKind::Fauld);
    for count in [48, 64] {
        let pattern = PlateFluting {
            count: FluteCount(count),
            ..Default::default()
        };
        let columns = pattern.columns(64);
        let crests = columns
            .windows(3)
            .filter(|samples| {
                let values = [0, 1, 2].map(|i| pattern.relief(samples[i], 0.5));
                values[1] > values[0] && values[1] > values[2]
            })
            .count();
        assert_eq!(crests, usize::from(count));
        design.fluting = Some(pattern);
        let mut mesh = generate_garment_armor(&design, &frame(GarmentArmorKind::Fauld)).unwrap();
        mesh.normals()
            .expect("finite fluted geometry before seam welding");
        let mut vertices = BTreeMap::new();
        let mut positions = Vec::new();
        let welded = mesh
            .positions
            .iter()
            .map(|p| {
                *vertices
                    .entry(p.map(|x| if x == 0.0 { 0 } else { x.to_bits() }))
                    .or_insert_with(|| {
                        positions.push(*p);
                        (positions.len() - 1) as u32
                    })
            })
            .collect::<Vec<_>>();
        mesh.positions = positions;
        for index in &mut mesh.indices {
            *index = welded[*index as usize];
        }
        for component in &mut mesh.components {
            let vertices = &welded[component.vertices.clone()];
            component.vertices = *vertices.iter().min().unwrap() as usize
                ..*vertices.iter().max().unwrap() as usize + 1;
        }
        assert_solid(&mesh);
    }
}

#[test]
fn every_garment_has_closed_consistently_wound_thickness() {
    for kind in KINDS {
        let design = GarmentArmorDesign::new(kind);
        let mesh = generate_garment_armor(&design, &frame(kind))
            .unwrap_or_else(|error| panic!("{kind:?}: {error}"));
        assert_solid(&mesh);
    }
}

#[test]
fn supported_extreme_parameters_keep_solid_topology_on_small_and_large_frames() {
    for kind in KINDS {
        for large in [false, true] {
            let mut design = GarmentArmorDesign::new(kind);
            design.clearance = Millimeters(if large { 40 } else { 1 });
            design.wall_thickness = Millimeters(if large { 16 } else { 1 });
            design.length = Permille(if large { 1_300 } else { 500 });
            design.flare = Permille(if large { 500 } else { 0 });
            design.waist = Permille(if large { 1_100 } else { 800 });
            design.lame_count = if large { 8 } else { 1 };
            let mut fit = frame(kind);
            fit.half_extents = fit.half_extents.map(|v| v * if large { 1.3 } else { 0.7 });
            assert_solid(
                &generate_garment_armor(&design, &fit)
                    .unwrap_or_else(|error| panic!("{kind:?}, large={large}: {error}")),
            );
        }
    }
}

#[test]
fn reflected_anatomical_frames_keep_outward_winding() {
    for kind in KINDS {
        let mut fit = frame(kind);
        fit.axes[0][0] = -1.0;
        fit.origin = [0.3, 1.1, -0.2];
        assert_solid(&generate_garment_armor(&GarmentArmorDesign::new(kind), &fit).unwrap());
    }
}

#[test]
fn garment_edits_retain_vertex_correspondence_unless_plate_count_changes() {
    for kind in KINDS {
        let mut design = GarmentArmorDesign::new(kind);
        let first = generate_garment_armor(&design, &frame(kind)).unwrap();
        design.length = Permille(800);
        design.flare = Permille(350);
        design.waist = Permille(1_050);
        let second = generate_garment_armor(&design, &frame(kind)).unwrap();
        assert_eq!(first.indices, second.indices);
        assert_eq!(first.positions.len(), second.positions.len());
        assert_ne!(first.positions, second.positions);
    }
}

#[test]
fn invalid_parameters_and_frames_are_rejected() {
    let mut design = GarmentArmorDesign::new(GarmentArmorKind::Fauld);
    design.lame_count = 0;
    assert!(generate_garment_armor(&design, &frame(design.kind)).is_err());
    design.lame_count = 4;
    let mut invalid = frame(design.kind);
    invalid.axes[0] = invalid.axes[1];
    assert!(generate_garment_armor(&design, &invalid).is_err());
}

fn connected_parts(mesh: &PartMesh) -> Vec<Vec<usize>> {
    let mut neighbors = vec![Vec::new(); mesh.positions.len()];
    for triangle in mesh.indices.as_chunks::<3>().0 {
        for (a, b) in [
            (triangle[0], triangle[1]),
            (triangle[1], triangle[2]),
            (triangle[2], triangle[0]),
        ] {
            neighbors[a as usize].push(b as usize);
            neighbors[b as usize].push(a as usize);
        }
    }
    let mut seen = vec![false; mesh.positions.len()];
    let mut parts = Vec::new();
    for start in 0..seen.len() {
        if seen[start] {
            continue;
        }
        let mut pending = vec![start];
        let mut part = Vec::new();
        seen[start] = true;
        while let Some(index) = pending.pop() {
            part.push(index);
            for &next in &neighbors[index] {
                if !seen[next] {
                    seen[next] = true;
                    pending.push(next);
                }
            }
        }
        parts.push(part);
    }
    parts
}

fn assert_welded_solid(mesh: &PartMesh) {
    // Match the review checker's micrometre weld so duplicated caps meeting
    // at the same physical edge cannot masquerade as two independent solids.
    let mut vertices = BTreeMap::<[i64; 3], u32>::new();
    let mapping = mesh
        .positions
        .iter()
        .map(|point| {
            let key = point.map(|v| (v * 1_000_000.0).round() as i64);
            let next = vertices.len() as u32;
            *vertices.entry(key).or_insert(next)
        })
        .collect::<Vec<_>>();
    let mut edges = BTreeMap::<(u32, u32), Vec<(u32, u32)>>::new();
    for triangle in mesh.indices.as_chunks::<3>().0 {
        let [a, b, c] = [triangle[0], triangle[1], triangle[2]].map(|i| mapping[i as usize]);
        assert!(
            a != b && b != c && a != c,
            "collapsed triangle after physical weld"
        );
        for (start, end) in [(a, b), (b, c), (c, a)] {
            edges
                .entry((start.min(end), start.max(end)))
                .or_default()
                .push((start, end));
        }
    }
    for uses in edges.values() {
        assert_eq!(
            uses.len(),
            2,
            "physical edge must have exactly two incident faces"
        );
        assert_eq!(
            uses[0],
            (uses[1].1, uses[1].0),
            "physical winding must agree"
        );
    }
}

fn angular_height_bounds(mesh: &PartMesh, part: &[usize], angle: f32) -> [f32; 2] {
    let mut bounds = [f32::INFINITY, f32::NEG_INFINITY];
    let members = part
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    for face in mesh.indices.as_chunks::<3>().0 {
        if !members.contains(&(face[0] as usize)) {
            continue;
        }
        for (a, b) in [(0, 1), (1, 2), (2, 0)] {
            let a = mesh.positions[face[a] as usize];
            let b = mesh.positions[face[b] as usize];
            let da = a[0] * angle.cos() - a[2] * angle.sin();
            let db = b[0] * angle.cos() - b[2] * angle.sin();
            if (da > 0.0) == (db > 0.0) || da == db {
                continue;
            }
            let t = da / (da - db);
            let point: [f32; 3] = std::array::from_fn(|axis| a[axis] + (b[axis] - a[axis]) * t);
            if point[0] * angle.sin() + point[2] * angle.cos() > 0.0 {
                bounds[0] = bounds[0].min(point[1]);
                bounds[1] = bounds[1].max(point[1]);
            }
        }
    }
    assert!(bounds.iter().all(|v| v.is_finite()));
    bounds
}

#[test]
fn gorget_lowest_neck_band_and_bib_are_one_physically_closed_sheet() {
    for count in [1, 3, 8] {
        let mut design = GarmentArmorDesign::new(GarmentArmorKind::Gorget);
        design.lame_count = count;
        let mesh = adventuresim_armor_model::generate_gorget_plates(
            &design,
            |t, angle| {
                [
                    0.08 * angle.sin(),
                    (0.05 + 0.015 * angle.cos()) * (1.0 - t),
                    0.08 * angle.cos(),
                ]
            },
            |t, angle| {
                [
                    (0.08 + 0.08 * t) * angle.sin(),
                    -0.04 * t,
                    (0.08 + 0.08 * t) * angle.cos(),
                ]
            },
        )
        .unwrap();
        assert_solid(&mesh);
        assert_welded_solid(&mesh);
        let parts = connected_parts(&mesh);
        assert_eq!(parts.len(), usize::from(count));
        let bib = parts
            .iter()
            .find(|part| {
                part.iter().any(|&i| {
                    let p = mesh.positions[i];
                    p[0].hypot(p[2]) > 0.10
                })
            })
            .unwrap();
        assert!(bib.iter().any(|&i| mesh.positions[i][1] > 0.001));
        assert!(bib.iter().any(|&i| mesh.positions[i][1] < -0.001));
        // Each transition point belongs to the same connected sheet as both
        // the raised neck band and descending bib, without a separate seam cap.
        for station in 0..16 {
            let angle = station as f32 * std::f32::consts::TAU / 16.0;
            let rim = [0.08 * angle.sin(), 0.0, 0.08 * angle.cos()];
            let distance = bib
                .iter()
                .map(|&index| {
                    let point = mesh.positions[index];
                    (0..3)
                        .map(|axis| (point[axis] - rim[axis]).powi(2))
                        .sum::<f32>()
                        .sqrt()
                })
                .fold(f32::INFINITY, f32::min);
            assert!(distance < 1e-6, "missing continuous seam at angle {angle}");
        }
    }
}

#[test]
fn standalone_gorget_count_creates_separate_overlapping_neck_plates() {
    for count in [1, 3, 8] {
        let mut design = GarmentArmorDesign::new(GarmentArmorKind::Gorget);
        design.lame_count = count;
        let mesh = generate_garment_armor(&design, &frame(design.kind)).unwrap();
        assert_welded_solid(&mesh);
        let parts = connected_parts(&mesh);
        assert_eq!(parts.len(), usize::from(count));
        for angle in [0.0, std::f32::consts::FRAC_PI_2, std::f32::consts::PI] {
            let mut heights = parts
                .iter()
                .map(|p| angular_height_bounds(&mesh, p, angle))
                .collect::<Vec<_>>();
            heights.sort_by(|a, b| b[1].total_cmp(&a[1]));
            for pair in heights.windows(2) {
                assert!(
                    pair[0][0] < pair[1][1] - 1e-5,
                    "adjacent collar plates must overlap: count={count}, angle={angle}, bounds={pair:?}"
                );
            }
        }
    }
}

#[test]
fn tasset_inner_cutaways_and_rounded_hems_do_not_reverse_narrow_lames() {
    use adventuresim_armor_model::GarmentPlateShape;
    for count in [3, 8] {
        for cutaway in [0, 400] {
            let mut design = GarmentArmorDesign::new(GarmentArmorKind::Tassets);
            design.lame_count = count;
            if let GarmentPlateShape::Tassets {
                inner_cutaway,
                hem_roundness,
                hem_point,
                ..
            } = &mut design.plate_shape
            {
                *inner_cutaway = Permille(cutaway);
                *hem_roundness = Permille(300);
                *hem_point = Permille(200);
            }
            let mesh = generate_garment_armor(&design, &frame(design.kind)).unwrap();
            mesh.refit_surfaces(|points, _| {
                let stride = points.len() / 9;
                for row in 1..9 {
                    for col in 0..stride {
                        assert!(
                            points[row * stride + col][1] > points[(row - 1) * stride + col][1],
                            "lame folded back along its length"
                        );
                    }
                }
            })
            .unwrap();
        }
    }
}

#[test]
fn increasing_gorget_slope_lowers_the_front_rim() {
    use adventuresim_armor_model::GarmentPlateShape;
    let mut design = GarmentArmorDesign::new(GarmentArmorKind::Gorget);
    let mut front_heights = Vec::new();
    for value in [0, 1000] {
        let GarmentPlateShape::Gorget { collar_slope, .. } = &mut design.plate_shape else {
            unreachable!()
        };
        *collar_slope = Permille(value);
        let mesh = generate_garment_armor(&design, &frame(GarmentArmorKind::Gorget)).unwrap();
        assert_solid(&mesh);
        front_heights.push(
            mesh.positions
                .iter()
                .filter(|p| p[0].abs() < 0.01 && p[2] > 0.06)
                .map(|p| p[1])
                .fold(f32::NEG_INFINITY, f32::max),
        );
    }
    assert!(front_heights[0] - front_heights[1] > 0.005);
}
#[test]
fn flared_short_gorget_collars_keep_compact_returns_at_physical_gauge() {
    for height in [0.002, 0.008, 0.030] {
        let mut design = GarmentArmorDesign::new(GarmentArmorKind::Gorget);
        design.lame_count = 3;
        design.wall_thickness = Millimeters(1);
        let mesh = adventuresim_armor_model::generate_gorget_plates(
            &design,
            |t, angle| {
                let radius = 0.075 + 0.02 * t;
                [
                    radius * angle.sin(),
                    height * (1.0 - t),
                    radius * angle.cos(),
                ]
            },
            |t, angle| {
                let radius = 0.095 + 0.04 * t;
                [radius * angle.sin(), -0.04 * t, radius * angle.cos()]
            },
        )
        .unwrap();
        assert_welded_solid(&mesh);
        let parts = connected_parts(&mesh);
        assert_eq!(parts.len(), 3);
        for part in &parts[1..] {
            let start = *part.iter().min().unwrap();
            let end = *part.iter().max().unwrap() + 1;
            let surface_vertices = (end - start) / 2;
            for index in start..start + surface_vertices {
                let outside = mesh.positions[index];
                let inside = mesh.positions[index + surface_vertices];
                let distance = (0..3)
                    .map(|axis| (outside[axis] - inside[axis]).powi(2))
                    .sum::<f32>()
                    .sqrt();
                assert!(
                    (distance - design.wall_thickness.metres()).abs() < 1e-7,
                    "short collars must not turn their gauge into long lateral blades"
                );
            }
        }
    }
}

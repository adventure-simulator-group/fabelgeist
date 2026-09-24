use super::*;

const BODY_RADIUS: f32 = 0.3;
const CLOTH_RADIUS: f32 = 0.34;

/// A closed sphere with outward triangles, standing in for the wearer.
fn sphere(radius: f32, center: Vec3) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    const RINGS: u32 = 16;
    const SEGMENTS: u32 = 24;
    let mut positions = vec![array(center + Vec3::new(0.0, radius, 0.0))];
    for ring in 1..RINGS {
        let polar = std::f32::consts::PI * ring as f32 / RINGS as f32;
        for segment in 0..SEGMENTS {
            let azimuth = std::f32::consts::TAU * segment as f32 / SEGMENTS as f32;
            let direction = Vec3::new(
                polar.sin() * azimuth.cos(),
                polar.cos(),
                polar.sin() * azimuth.sin(),
            );
            positions.push(array(center + direction * radius));
        }
    }
    let bottom = positions.len() as u32;
    positions.push(array(center - Vec3::new(0.0, radius, 0.0)));
    let at = |ring: u32, segment: u32| 1 + (ring - 1) * SEGMENTS + segment % SEGMENTS;
    let mut faces = Vec::new();
    for segment in 0..SEGMENTS {
        faces.push([0, at(1, segment + 1), at(1, segment)]);
        faces.push([bottom, at(RINGS - 1, segment), at(RINGS - 1, segment + 1)]);
        for ring in 1..RINGS - 1 {
            let (a, b) = (at(ring, segment), at(ring, segment + 1));
            let (c, d) = (at(ring + 1, segment), at(ring + 1, segment + 1));
            faces.push([a, b, d]);
            faces.push([a, d, c]);
        }
    }
    (positions, faces)
}

fn wearer(positions: Vec<[f32; 3]>, faces: Vec<[u32; 3]>) -> DrapeInput {
    let count = positions.len();
    DrapeInput {
        under_plate: None,
        selection: GarmentSelection::default(),
        settled: None,
        obstacles: vec![],
        positions,
        faces,
        names: vec![],
        joints: vec![],
        indices: vec![[0; 8]; count],
        weights: vec![[1., 0., 0., 0., 0., 0., 0., 0.]; count],
    }
}

/// A settled cap of cloth over the top of the sphere at the origin.
fn cap(radius: f32) -> DrapedGarment {
    const SIDE: usize = 7;
    const SPAN: f32 = 0.24;
    let mut positions = Vec::new();
    let mut texcoords = Vec::new();
    for row in 0..SIDE {
        for column in 0..SIDE {
            let u = column as f32 / (SIDE - 1) as f32 - 0.5;
            let v = row as f32 / (SIDE - 1) as f32 - 0.5;
            let direction = Vec3::new(u * SPAN, 0.2, v * SPAN);
            positions.push(array(direction / direction.length() * radius));
            texcoords.push([u, v]);
        }
    }
    let mut faces = Vec::new();
    for row in 0..SIDE - 1 {
        for column in 0..SIDE - 1 {
            let i = (row * SIDE + column) as u32;
            let side = SIDE as u32;
            faces.push([i, i + side, i + 1]);
            faces.push([i + 1, i + side, i + side + 1]);
        }
    }
    DrapedGarment {
        form: GarmentForm::Upper,
        name: "Cap".into(),
        fabric: FabricPreset::Cotton,
        normals: vec![],
        texcoords,
        positions,
        faces,
        indices: vec![],
        weights: vec![],
        stage: DrapeStage::Settling { step: 1, of: 1 },
    }
}

fn distance_from(center: Vec3, p: [f32; 3]) -> f32 {
    (vector(p) - center).length()
}

#[test]
fn a_drape_worn_on_its_own_body_keeps_its_shape() {
    let (positions, faces) = sphere(BODY_RADIUS, Vec3::default());
    let input = wearer(positions, faces);
    let cloth = cap(CLOTH_RADIUS);
    let saved = SettledDrape::capture(&input, &cloth).unwrap();
    let worn = saved.wear(&input).result.unwrap();
    assert_eq!(worn.stage, DrapeStage::Worn);
    assert_eq!(worn.faces, cloth.faces);
    assert_eq!(worn.texcoords, cloth.texcoords);
    for (worn, settled) in worn.positions.iter().zip(&cloth.positions) {
        assert!((vector(*worn) - vector(*settled)).length() < 1e-5);
    }
    assert_eq!(worn.indices.len(), worn.positions.len());
}

#[test]
fn a_drape_follows_a_larger_moved_body_and_keeps_its_ease() {
    let (positions, faces) = sphere(BODY_RADIUS, Vec3::default());
    let saved = SettledDrape::capture(&wearer(positions, faces), &cap(CLOTH_RADIUS)).unwrap();
    let center = Vec3::new(0.1, 0.05, -0.2);
    let larger = BODY_RADIUS * 1.2;
    let (positions, faces) = sphere(larger, center);
    let worn = saved.wear(&wearer(positions, faces)).result.unwrap();
    let ease = CLOTH_RADIUS - BODY_RADIUS;
    for p in &worn.positions {
        let height = distance_from(center, *p) - larger;
        // The faceted sphere's flat triangles sit up to a few millimetres
        // inside its corners, which the ease is measured from.
        assert!((height - ease).abs() < 0.006, "cloth sits {height} m out");
    }
}

#[test]
fn cloth_that_would_sink_into_a_body_is_pushed_clear() {
    let (positions, faces) = sphere(BODY_RADIUS, Vec3::default());
    let saved =
        SettledDrape::capture(&wearer(positions, faces), &cap(BODY_RADIUS + 0.002)).unwrap();
    let (positions, faces) = sphere(BODY_RADIUS, Vec3::default());
    let input = wearer(positions.clone(), faces.clone());
    let clearance = super::super::drape::body_clearance(&input);
    let body = fabelgeist_bvh::TriangleBvh::new(positions.into_iter().map(vector).collect(), faces);
    let worn = saved.wear(&input).result.unwrap();
    for p in &worn.positions {
        let (_, _, height) = body.closest_point(vector(*p), f32::MAX).unwrap();
        assert!(height > clearance * 0.99, "cloth sits {height} m out");
    }
}

#[test]
fn a_drape_is_not_worn_on_another_body_mesh() {
    let (positions, faces) = sphere(BODY_RADIUS, Vec3::default());
    let saved = SettledDrape::capture(&wearer(positions, faces), &cap(CLOTH_RADIUS)).unwrap();
    let (mut positions, mut faces) = sphere(BODY_RADIUS, Vec3::default());
    positions.push([0.0; 3]);
    faces.push([0, 1, 2]);
    assert!(saved.wear(&wearer(positions, faces)).result.is_err());
}

#[test]
fn an_unsettled_drape_is_not_saved() {
    let (positions, faces) = sphere(BODY_RADIUS, Vec3::default());
    let mut cloth = cap(CLOTH_RADIUS);
    cloth.stage = DrapeStage::Settling { step: 3, of: 180 };
    assert!(SettledDrape::capture(&wearer(positions, faces), &cloth).is_err());
}

#[test]
fn a_saved_garment_reloads_from_compact_json() {
    let (positions, faces) = sphere(BODY_RADIUS, Vec3::default());
    let input = wearer(positions, faces);
    let saved = SettledGarment::capture(&input, &cap(CLOTH_RADIUS)).unwrap();
    let json = serde_json::to_string_pretty(&saved).unwrap();
    // Per-vertex arrays are packed into single strings, not one line per number.
    assert!(json.lines().count() < 200, "{} lines", json.lines().count());
    let loaded: SettledGarment = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded, saved);
    loaded.validate().unwrap();
}

#[test]
fn a_truncated_packed_array_is_rejected() {
    let (positions, faces) = sphere(BODY_RADIUS, Vec3::default());
    let saved = SettledDrape::capture(&wearer(positions, faces), &cap(CLOTH_RADIUS)).unwrap();
    let mut json: serde_json::Value = serde_json::to_value(&saved).unwrap();
    json["bindings"] = serde_json::Value::String(STANDARD_TRUNCATED.into());
    assert!(serde_json::from_value::<SettledDrape>(json).is_err());
}

/// Five bytes: not a whole number of binding records.
const STANDARD_TRUNCATED: &str = "AAAAAAA=";

/// Two spheres side by side, standing in for the thighs.
fn thighs(gap: f32) -> DrapeInput {
    let offset = BODY_RADIUS + gap * 0.5;
    let (mut positions, mut faces) = sphere(BODY_RADIUS, Vec3::new(-offset, 0.0, 0.0));
    let (right, right_faces) = sphere(BODY_RADIUS, Vec3::new(offset, 0.0, 0.0));
    let first = positions.len() as u32;
    positions.extend(right);
    faces.extend(right_faces.into_iter().map(|face| face.map(|i| i + first)));
    wearer(positions, faces)
}

#[test]
fn cloth_bound_to_parts_that_move_apart_holds_together() {
    const SEGMENTS: usize = 24;
    const HEIGHT: f32 = BODY_RADIUS + 0.05;
    // A strip hanging over both spheres, one row of quads long.
    let span = 2.0 * BODY_RADIUS + 0.02;
    let mut positions = Vec::new();
    for column in 0..=SEGMENTS {
        let x = -span * 0.5 + span * column as f32 / SEGMENTS as f32;
        positions.push([x, HEIGHT, -0.02]);
        positions.push([x, HEIGHT, 0.02]);
    }
    let faces: Vec<[u32; 3]> = (0..SEGMENTS as u32)
        .flat_map(|c| {
            [
                [2 * c, 2 * c + 1, 2 * c + 2],
                [2 * c + 1, 2 * c + 3, 2 * c + 2],
            ]
        })
        .collect();
    let settled = DrapedGarment {
        texcoords: positions.iter().map(|p| [p[0], p[2]]).collect(),
        positions,
        faces,
        ..cap(CLOTH_RADIUS)
    };
    let saved = SettledDrape::capture(&thighs(0.02), &settled).unwrap();
    let worn = saved.wear(&thighs(0.12)).result.unwrap();
    let length = |positions: &[[f32; 3]], [a, b]: [u32; 2]| {
        (vector(positions[b as usize]) - vector(positions[a as usize])).length()
    };
    let stretch = settled
        .faces
        .iter()
        .flat_map(|&[a, b, c]| [[a, b], [b, c], [c, a]])
        .map(|edge| length(&worn.positions, edge) / length(&settled.positions, edge))
        .fold(0.0, f32::max);
    // Unheld, the edge across the widened gap would stretch about fivefold.
    assert!(stretch < 1.5, "an edge stretched {stretch} times");
}

//! Made-to-measure garments on the creator's metre-based MHR mesh.
use anyhow::{Context, Result, bail};
use fabelgeist_cloth::Fabric;
use fabelgeist_garment_code::{
    Body, Design, assets,
    design::Value,
    measure::{BodyMesh, Landmarks, measure},
    programs::MetaGarment,
};
use fabelgeist_garment_fit::{Fit, FitSettings, build_garment};
use fabelgeist_math::Vec3;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

mod conform;
mod dressing;
mod export;
mod fitted;
mod lining;
pub mod pattern;
mod placement;
mod selection;
mod settled;
mod shading;
mod stages;
mod symmetrize;
mod validation;
pub use lining::{PlateLining, UnderPlate};
pub use selection::{ClothLayer, Construction, FabricPreset, GarmentForm, GarmentSelection};
pub use settled::{BodyTopology, SettledDrape, SettledGarment};
pub use stages::{DrapeCheckpoints, DrapeSettings, StageSettings};

#[derive(Clone)]
pub struct DrapeInput {
    pub selection: GarmentSelection,
    /// The worn plate the garment lies under, when it is worn under plate.
    pub under_plate: Option<UnderPlate>,
    /// A drape saved once settled, fitted to this wearer instead of simulated.
    pub settled: Option<std::sync::Arc<SettledDrape>>,
    pub obstacles: Vec<DrapedGarment>,
    pub positions: Vec<[f32; 3]>,
    pub faces: Vec<[u32; 3]>,
    pub names: Vec<String>,
    pub joints: Vec<[f32; 8]>,
    pub indices: Vec<[u32; 8]>,
    pub weights: Vec<[f32; 8]>,
}
#[derive(Clone)]
pub struct DrapedGarment {
    pub form: GarmentForm,
    pub name: String,
    pub fabric: FabricPreset,
    pub texcoords: Vec<[f32; 2]>,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub faces: Vec<[u32; 3]>,
    pub indices: Vec<[u32; 8]>,
    pub weights: Vec<[f32; 8]>,
    pub stage: DrapeStage,
}

/// How far a garment has progressed through draping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrapeStage {
    /// Flat panels placed around the wearer, before any simulation.
    Placed,
    /// Seams closing without gravity; panels are still separate.
    Sewing { step: u32, of: u32 },
    /// Sewn garment settling under gravity.
    Settling { step: u32, of: u32 },
    /// Fitted from a saved drape; nothing was simulated on this wearer.
    Worn,
}

impl std::fmt::Display for DrapeStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Placed => f.write_str("panels placed"),
            Self::Sewing { step, of } => write!(f, "sewing step {step}/{of}"),
            Self::Settling { step, of } => write!(f, "settling step {step}/{of}"),
            Self::Worn => f.write_str("fitted from its saved drape"),
        }
    }
}
fn vector(p: [f32; 3]) -> Vec3 {
    Vec3::new(p[0], p[1], p[2])
}
fn array(p: Vec3) -> [f32; 3] {
    [p.x, p.y, p.z]
}

pub fn measured_body(input: &DrapeInput) -> Result<Body> {
    let joint = |name: &str| -> Result<[f32; 3]> {
        let i = input
            .names
            .iter()
            .position(|n| n == name)
            .with_context(|| format!("missing measurement joint {name}"))?;
        let j = input.joints.get(i).context("missing joint transform")?;
        Ok([j[0] * 100.0, j[1] * 100.0, j[2] * 100.0])
    };
    let marks = Landmarks {
        shoulders: [joint("l_uparm")?, joint("r_uparm")?],
        elbows: [joint("l_lowarm")?, joint("r_lowarm")?],
        wrists: [joint("l_wrist")?, joint("r_wrist")?],
        hips: [joint("l_upleg")?, joint("r_upleg")?],
        knees: [joint("l_lowleg")?, joint("r_lowleg")?],
        neck: joint("c_neck")?,
    };
    let vertices: Vec<_> = input
        .positions
        .iter()
        .map(|p| p.map(|v| v * 100.0))
        .collect();
    measure(
        BodyMesh {
            vertices: &vertices,
            faces: &input.faces,
        },
        &marks,
    )
}

mod drape;
mod outfit;
pub use drape::drape;
pub use outfit::{OutfitOutcome, drape_outfit};

/// One garment's drape result, and the stages it completed either way.
pub struct DrapeOutcome {
    pub result: Result<DrapedGarment>,
    /// Fit problems in a draped garment. The garment is still usable.
    pub warnings: Vec<String>,
    pub checkpoints: DrapeCheckpoints,
}

fn clear_panels(
    positions: &mut [Vec3],
    mesh: &fabelgeist_cloth::GarmentMesh,
    body: &fabelgeist_bvh::TriangleBvh,
    margin: f32,
) {
    let reach = body.bvh.bounds().extent().length() + 1.0;
    for panel in 0..mesh.panel_count() {
        let range = mesh.panel_range(panel);
        let Some(face) = mesh
            .triangles
            .iter()
            .find(|face| range.contains(&(face[0] as usize)))
        else {
            continue;
        };
        let [a, b, c] = face.map(|i| positions[i as usize]);
        let raw_normal = (b - a).cross(c - a);
        if raw_normal.length() < 1e-10 {
            continue;
        }
        let normal = raw_normal / raw_normal.length();
        let mut samples = positions[range.clone()].to_vec();
        for face in mesh
            .triangles
            .iter()
            .filter(|face| range.contains(&(face[0] as usize)))
        {
            samples.push(
                face.iter()
                    .fold(Vec3::default(), |sum, &i| sum + positions[i as usize])
                    / 3.0,
            );
        }
        let shift_along = |direction: Vec3| {
            let mut shift = 0.0f32;
            for &point in &samples {
                let ray = fabelgeist_bvh::Ray::new(point + direction * reach, direction * -1.0);
                if let Some((_, distance)) = body.raycast(&ray, reach * 2.0) {
                    shift = shift.max(reach - distance + margin);
                }
            }
            shift
        };
        let forward = shift_along(normal);
        let backward = shift_along(normal * -1.0);
        let (direction, shift) = if forward <= backward {
            (normal, forward)
        } else {
            (normal * -1.0, backward)
        };
        for point in &mut positions[range] {
            *point = *point + direction * shift;
        }
    }
}

struct SewnSurface {
    groups: Vec<Vec<usize>>,
    faces: Vec<[u32; 3]>,
    render_faces: Vec<usize>,
}
impl SewnSurface {
    fn from_positions(positions: &[[f32; 3]], faces: &[[u32; 3]]) -> Self {
        let mut first = std::collections::BTreeMap::new();
        let seams: Vec<_> = positions
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                first
                    .insert(p.map(f32::to_bits), i as u32)
                    .map(|other| [other, i as u32])
            })
            .collect();
        Self::new(positions.len(), &seams, faces)
    }
    fn expand(&self, sewn: &[[f32; 3]]) -> Vec<[f32; 3]> {
        let mut positions = vec![[0.0; 3]; self.groups.iter().map(Vec::len).sum()];
        for (group, position) in self.groups.iter().zip(sewn) {
            for &vertex in group {
                positions[vertex] = *position;
            }
        }
        positions
    }
    fn new(count: usize, seams: &[[u32; 2]], triangles: &[[u32; 3]]) -> Self {
        let mut parents: Vec<_> = (0..count).collect();
        fn root(parents: &[usize], mut vertex: usize) -> usize {
            while parents[vertex] != vertex {
                vertex = parents[vertex];
            }
            vertex
        }
        for &[a, b] in seams {
            let a = root(&parents, a as usize);
            let b = root(&parents, b as usize);
            parents[b] = a;
        }
        let mut ids = std::collections::BTreeMap::new();
        let mut groups = Vec::<Vec<usize>>::new();
        let mut mapping = vec![0u32; count];
        for vertex in 0..count {
            let root = root(&parents, vertex);
            let next = groups.len();
            let id = *ids.entry(root).or_insert_with(|| {
                groups.push(Vec::new());
                next
            });
            groups[id].push(vertex);
            mapping[vertex] = id as u32;
        }
        let render_faces: Vec<_> = triangles
            .iter()
            .enumerate()
            .filter_map(|(index, face)| {
                let f = face.map(|i| mapping[i as usize]);
                (f[0] != f[1] && f[1] != f[2] && f[2] != f[0]).then_some(index)
            })
            .collect();
        let faces = render_faces
            .iter()
            .map(|&i| triangles[i].map(|v| mapping[v as usize]))
            .collect();
        Self {
            groups,
            faces,
            render_faces,
        }
    }
    fn positions(&self, source: &[[f32; 3]]) -> Vec<[f32; 3]> {
        self.groups
            .iter()
            .map(|group| {
                let average = group
                    .iter()
                    .fold(Vec3::default(), |sum, &vertex| sum + vector(source[vertex]))
                    / group.len() as f32;
                array(average)
            })
            .collect()
    }
}
pub fn normals(positions: &[[f32; 3]], faces: &[[u32; 3]]) -> Vec<[f32; 3]> {
    let mut normals = vec![Vec3::default(); positions.len()];
    for f in faces {
        let [a, b, c] = f.map(|i| vector(positions[i as usize]));
        let n = (b - a).cross(c - a);
        for i in f {
            normals[*i as usize] = normals[*i as usize] + n;
        }
    }
    normals
        .into_iter()
        .map(|n| {
            let length = n.length();
            if length > 1e-10 {
                array(n / length)
            } else {
                [0.0, 1.0, 0.0]
            }
        })
        .collect()
}

// Resolve winding over shared stitched edges before accumulating vertex normals.
// Choose the outward sign per connected component, not independently per face.
fn orient_faces(
    positions: &[[f32; 3]],
    faces: &mut [[u32; 3]],
    body: &fabelgeist_bvh::TriangleBvh,
) {
    let mut edges = std::collections::BTreeMap::<(u32, u32), Vec<(usize, bool)>>::new();
    for (i, face) in faces.iter().enumerate() {
        for k in 0..3 {
            let (a, b) = (face[k], face[(k + 1) % 3]);
            edges
                .entry((a.min(b), a.max(b)))
                .or_default()
                .push((i, a < b));
        }
    }
    let mut adjacent = vec![Vec::new(); faces.len()];
    for uses in edges.values() {
        if uses.len() == 2 {
            let (a, b) = (uses[0], uses[1]);
            adjacent[a.0].push((b.0, a.1 == b.1));
            adjacent[b.0].push((a.0, a.1 == b.1));
        }
    }
    let mut flips = vec![None; faces.len()];
    for start in 0..faces.len() {
        if flips[start].is_some() {
            continue;
        }
        flips[start] = Some(false);
        let mut component = vec![start];
        let mut cursor = 0;
        while cursor < component.len() {
            let a = component[cursor];
            for &(b, opposite) in &adjacent[a] {
                if flips[b].is_none() {
                    flips[b] = Some(flips[a].unwrap() ^ opposite);
                    component.push(b);
                }
            }
            cursor += 1;
        }
        let mut outward = 0.0;
        for &i in &component {
            if flips[i].unwrap() {
                faces[i].swap(1, 2);
            }
            let [a, b, c] = faces[i].map(|v| vector(positions[v as usize]));
            let center = (a + b + c) / 3.0;
            if let Some((_, closest, _)) = body.closest_point(center, f32::MAX) {
                outward += (b - a).cross(c - a).dot(center - closest);
            }
        }
        if outward < 0.0 {
            for i in component {
                faces[i].swap(1, 2);
            }
        }
    }
}

// Sample each side outside the nearest-leg discontinuity, then interpolate
// across the hip span. Side vertices retain full leg motion; the centre blends.
fn transfer_skirt_skin(
    input: &DrapeInput,
    positions: &[[f32; 3]],
) -> Result<(Vec<[u32; 8]>, Vec<[f32; 8]>)> {
    let hip = |name: &str| -> Result<[f32; 8]> {
        let index = input
            .names
            .iter()
            .position(|n| n == name)
            .context("missing hip joint")?;
        input
            .joints
            .get(index)
            .copied()
            .context("missing hip transform")
    };
    let left = hip("l_upleg")?;
    let right = hip("r_upleg")?;
    let center = (left[0] + right[0]) * 0.5;
    let half_width = ((left[0] - right[0]).abs() * 0.5).max(0.025);
    let hip_y = (left[1] + right[1]) * 0.5;
    let mut direct = input.clone();
    direct.selection.construction = Construction::Sewn(pattern::Pattern::default());
    let (mut ids, mut weights) = transfer_skin(&direct, positions)?;
    let low: Vec<_> = positions
        .iter()
        .map(|p| [center - half_width, p[1], p[2]])
        .collect();
    let high: Vec<_> = positions
        .iter()
        .map(|p| [center + half_width, p[1], p[2]])
        .collect();
    let (low_ids, low_weights) = transfer_skin(&direct, &low)?;
    let (high_ids, high_weights) = transfer_skin(&direct, &high)?;
    let smooth = |t: f32| {
        let t = t.clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    };
    for (v, p) in positions.iter().enumerate() {
        if p[1] >= hip_y || (p[0] - center).abs() >= half_width {
            continue;
        }
        let horizontal = smooth((p[0] - center + half_width) / (2.0 * half_width));
        let vertical = smooth((hip_y - p[1]) / half_width);
        let mut blended = std::collections::BTreeMap::<u32, f32>::new();
        for (source_ids, source_weights, factor) in [
            (ids[v], weights[v], 1.0 - vertical),
            (low_ids[v], low_weights[v], vertical * (1.0 - horizontal)),
            (high_ids[v], high_weights[v], vertical * horizontal),
        ] {
            for (joint, w) in source_ids.into_iter().zip(source_weights) {
                *blended.entry(joint).or_default() += w * factor;
            }
        }
        let mut blended: Vec<_> = blended.into_iter().filter(|(_, w)| *w > 0.0).collect();
        blended.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        blended.truncate(8);
        let sum: f32 = blended.iter().map(|(_, w)| w).sum();
        ids[v] = [0; 8];
        weights[v] = [0.0; 8];
        for (slot, (joint, w)) in blended.into_iter().enumerate() {
            ids[v][slot] = joint;
            weights[v][slot] = w / sum;
        }
    }
    Ok((ids, weights))
}
pub fn transfer_skin(
    input: &DrapeInput,
    positions: &[[f32; 3]],
) -> Result<(Vec<[u32; 8]>, Vec<[f32; 8]>)> {
    if input.indices.len() != input.positions.len() || input.weights.len() != input.positions.len()
    {
        bail!("body skin arrays do not match its vertices");
    }
    if input.selection.form() == GarmentForm::Skirted {
        return transfer_skirt_skin(input, positions);
    }
    let tree = fabelgeist_bvh::TriangleBvh::new(
        input.positions.iter().copied().map(vector).collect(),
        input.faces.clone(),
    );
    let mut indices = Vec::new();
    let mut weights = Vec::new();
    for p in positions {
        let (triangle, point, _) = tree
            .closest_point(vector(*p), f32::MAX)
            .context("no body surface for garment skin transfer")?;
        let face = input.faces[triangle as usize];
        let [a, b, c] = face.map(|i| vector(input.positions[i as usize]));
        let (v0, v1, v2) = (b - a, c - a, point - a);
        let (d00, d01, d11, d20, d21) =
            (v0.dot(v0), v0.dot(v1), v1.dot(v1), v2.dot(v0), v2.dot(v1));
        let denominator = d00 * d11 - d01 * d01;
        let bary = if denominator.abs() > 1e-15 {
            let v = (d11 * d20 - d01 * d21) / denominator;
            let w = (d00 * d21 - d01 * d20) / denominator;
            [1.0 - v - w, v, w]
        } else {
            [1.0, 0.0, 0.0]
        };
        let mut influences = std::collections::BTreeMap::<u32, f32>::new();
        for (vertex, factor) in face.into_iter().zip(bary) {
            for (joint, weight) in input.indices[vertex as usize]
                .into_iter()
                .zip(input.weights[vertex as usize])
            {
                *influences.entry(joint).or_default() += factor.max(0.0) * weight;
            }
        }
        let mut influences: Vec<_> = influences.into_iter().filter(|(_, w)| *w > 0.0).collect();
        influences.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        influences.truncate(8);
        let total: f32 = influences.iter().map(|(_, w)| w).sum();
        if !total.is_finite() || total <= 0.0 {
            bail!("garment vertex has no skin weights");
        }
        let mut ids = [0; 8];
        let mut ws = [0.0; 8];
        for (i, (id, w)) in influences.into_iter().enumerate() {
            ids[i] = id;
            ws[i] = w / total;
        }
        indices.push(ids);
        weights.push(ws);
    }
    Ok((indices, weights))
}

#[cfg(test)]
mod tests {
    use super::pattern::{Pattern, Upper, shapes};
    use super::*;
    #[test]
    fn shapes_build_sewn_patterns() {
        let body = Body::from_yaml_str(assets::BODIES[0].yaml).unwrap();
        for shape in shapes::SHAPES {
            let design = shape.pattern.design().unwrap();
            let pattern = MetaGarment::new("test", &body, &design).assembly();
            let build = build_garment(&pattern, &FitSettings::default(), &Fabric::COTTON).unwrap();
            assert!(
                build.skipped.is_empty(),
                "{}: {:?}",
                shape.name,
                build.skipped
            );
            assert!(!build.mesh.triangles.is_empty());
        }
    }
    /// Lowest and highest pattern point, in metres.
    fn vertical_extent(length: f64) -> (f32, f32) {
        let body = Body::from_yaml_str(assets::BODIES[0].yaml).unwrap();
        let selection = GarmentSelection {
            construction: Construction::Sewn(Pattern {
                upper: Some(Upper::Straight {
                    length,
                    width: 1.05,
                    flare: 1.0,
                }),
                ..Pattern::default()
            }),
            ..GarmentSelection::chainmail()
        };
        selection.validate().unwrap();
        let pattern = MetaGarment::new("extent", &body, &selection.design().unwrap()).assembly();
        let build = build_garment(&pattern, &FitSettings::default(), &Fabric::CHAINMAIL).unwrap();
        assert!(build.skipped.is_empty(), "{:?}", build.skipped);
        let heights = build.mesh.positions.iter().map(|p| p.y);
        (
            heights.clone().fold(f32::INFINITY, f32::min),
            heights.fold(f32::NEG_INFINITY, f32::max),
        )
    }

    #[test]
    fn shirt_length_lowers_the_hem() {
        let (waist, _) = vertical_extent(1.0);
        let (thigh, _) = vertical_extent(2.0);
        assert!(thigh < waist - 0.1, "hem moved from {waist} m to {thigh} m");
    }

    #[test]
    fn flat_mail_panels_do_not_inflate_from_self_contact() {
        let body = Body::from_yaml_str(assets::BODIES[0].yaml).unwrap();
        let design = shapes::FITTED_SHIRT.pattern.design().unwrap();
        let pattern = MetaGarment::new("rest contact", &body, &design).assembly();
        let build = build_garment(&pattern, &FitSettings::default(), &Fabric::CHAINMAIL).unwrap();
        let start = build.mesh.positions.clone();
        let mut points = start.clone();
        let masses = vec![1.0; start.len()];
        let contacts = fabelgeist_cloth::surface_contact::SurfaceContacts::new(
            start.len(),
            build.mesh.triangles.clone(),
        )
        .with_seams(&build.mesh.seams);
        let count = contacts.solve(&mut points, &start, &masses, Fabric::CHAINMAIL.thickness, 1);
        let movement = points
            .iter()
            .zip(&start)
            .map(|(a, b)| (*a - *b).length())
            .fold(0.0, f32::max);
        assert!(
            movement < 1e-6,
            "{count} contacts moved flat mail by {} mm",
            movement * 1000.0
        );
    }
    #[test]
    fn garment_skin_interpolates_and_normalizes_body_influences() {
        let input = DrapeInput {
            under_plate: None,
            selection: GarmentSelection::default(),
            settled: None,
            obstacles: vec![],
            positions: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
            faces: vec![[0, 1, 2]],
            names: vec![],
            joints: vec![],
            indices: vec![[0; 8], [1; 8], [2; 8]],
            weights: vec![[1., 0., 0., 0., 0., 0., 0., 0.]; 3],
        };
        let (ids, weights) = transfer_skin(&input, &[[0.25, 0.25, 0.1]]).unwrap();
        assert_eq!(&ids[0][..3], &[0, 1, 2]);
        assert_eq!(&weights[0][..3], &[0.5, 0.25, 0.25]);
    }
}

#[cfg(test)]
mod seam_and_leg_regression {
    use super::*;
    #[test]
    fn opposite_panel_winding_does_not_cancel_seam_normals() {
        let positions = vec![[0., 0., 1.], [1., 0., 1.], [0., 1., 1.], [1., 1., 1.]];
        let mut faces = vec![[0, 1, 2], [1, 2, 3]];
        let body = fabelgeist_bvh::TriangleBvh::new(
            vec![
                Vec3::new(0., 0., 0.),
                Vec3::new(2., 0., 0.),
                Vec3::new(0., 2., 0.),
            ],
            vec![[0, 1, 2]],
        );
        orient_faces(&positions, &mut faces, &body);
        for normal in normals(&positions, &faces) {
            assert!(normal[2] > 0.999);
        }
    }
    #[test]
    fn dress_center_is_continuous_and_sides_retain_opposing_leg_motion() {
        let input = DrapeInput {
            under_plate: None,
            selection: GarmentSelection::from_shape(&pattern::shapes::DRESS),
            settled: None,
            obstacles: vec![],
            positions: vec![
                [-0.1, 0., 0.],
                [-0.1, 1., 0.],
                [-0.1, 0., 1.],
                [0.1, 0., 0.],
                [0.1, 1., 0.],
                [0.1, 0., 1.],
            ],
            faces: vec![[0, 1, 2], [3, 4, 5]],
            names: vec!["l_upleg".into(), "r_upleg".into()],
            joints: vec![
                [-0.1, 1., 0., 0., 0., 0., 1., 1.],
                [0.1, 1., 0., 0., 0., 0., 1., 1.],
            ],
            indices: vec![[0; 8], [0; 8], [0; 8], [1; 8], [1; 8], [1; 8]],
            weights: vec![[1., 0., 0., 0., 0., 0., 0., 0.]; 6],
        };
        let points = vec![
            [-0.1, 0.5, 0.1],
            [-0.00001, 0.5, 0.1],
            [0.00001, 0.5, 0.1],
            [0.1, 0.5, 0.1],
        ];
        let (ids, weights) = transfer_skin(&input, &points).unwrap();
        let displacement = |v: usize| {
            ids[v]
                .iter()
                .zip(weights[v])
                .map(|(&id, w)| if id == 0 { w } else { -w })
                .sum::<f32>()
        };
        assert!(displacement(0) > 0.999);
        assert!(displacement(3) < -0.999);
        assert!((displacement(1) - displacement(2)).abs() < 0.001);
        for weights in weights {
            assert!((weights.iter().sum::<f32>() - 1.0).abs() < 1e-6);
        }
    }
}

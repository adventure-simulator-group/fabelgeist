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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GarmentPreset {
    Shirt,
    Trousers,
    Skirt,
    Dress,
}
impl GarmentPreset {
    pub const ALL: [Self; 4] = [Self::Shirt, Self::Trousers, Self::Skirt, Self::Dress];
    pub fn label(self) -> &'static str {
        match self {
            Self::Shirt => "Shirt",
            Self::Trousers => "Trousers",
            Self::Skirt => "Skirt",
            Self::Dress => "Dress",
        }
    }
    pub fn design(self) -> Result<Design> {
        let design = Design::from_yaml_str(assets::DESIGNS[0].yaml)?;
        let upper = matches!(self, Self::Shirt | Self::Dress);
        design.set_v(
            "meta.upper",
            if upper {
                Value::Str("Shirt".into())
            } else {
                Value::Null
            },
        );
        design.set_v(
            "meta.bottom",
            match self {
                Self::Shirt => Value::Null,
                Self::Trousers => Value::Str("Pants".into()),
                _ => Value::Str("Skirt2".into()),
            },
        );
        design.set_v(
            "meta.wb",
            if upper {
                Value::Null
            } else {
                Value::Str("FittedWB".into())
            },
        );
        // Author each selectable style explicitly: the T-shirt asset's dormant
        // lower-body parameters otherwise describe shorts and a broad circle skirt.
        design.set_f("pants.length", 0.9);
        design.set_f("skirt.length", 0.45);
        design.set_f("skirt.ruffle", 1.0);
        design.set_f("skirt.flare", 1.0);
        design.set_f("waistband.waist", 1.03);
        if matches!(self, Self::Dress) {
            design.set_f("shirt.length", 1.0);
        }
        Ok(design)
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FabricPreset {
    Cotton,
    Silk,
    Denim,
    Wool,
    Jersey,
}
impl FabricPreset {
    pub const ALL: [Self; 5] = [
        Self::Cotton,
        Self::Silk,
        Self::Denim,
        Self::Wool,
        Self::Jersey,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Cotton => "Cotton",
            Self::Silk => "Silk",
            Self::Denim => "Denim",
            Self::Wool => "Wool",
            Self::Jersey => "Jersey",
        }
    }
    pub fn fabric(self) -> Fabric {
        let mut fabric = match self {
            Self::Cotton => Fabric::COTTON,
            Self::Silk => Fabric::SILK,
            Self::Denim => Fabric::DENIM,
            Self::Wool => Fabric::WOOL,
            Self::Jersey => Fabric::JERSEY,
        };
        fabric.bend_compliance *= 100.0;
        fabric
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GarmentSelection {
    pub preset: GarmentPreset,
    pub fabric: FabricPreset,
    pub resolution_cm: f32,
    pub steps: u32,
}
impl Default for GarmentSelection {
    fn default() -> Self {
        Self {
            preset: GarmentPreset::Shirt,
            fabric: FabricPreset::Cotton,
            resolution_cm: 3.5,
            steps: 180,
        }
    }
}
impl GarmentSelection {
    pub fn validate(&self) -> Result<()> {
        if !self.resolution_cm.is_finite() || !(2.5..=6.0).contains(&self.resolution_cm) {
            bail!("cloth resolution must be between 2.5 and 6 cm");
        }
        if !(30..=600).contains(&self.steps) {
            bail!("draping requires between 30 and 600 steps");
        }
        Ok(())
    }
}
#[derive(Clone)]
pub struct DrapeInput {
    pub selection: GarmentSelection,
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
    pub name: String,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub faces: Vec<[u32; 3]>,
    pub indices: Vec<[u32; 8]>,
    pub weights: Vec<[f32; 8]>,
    pub frame: u32,
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

pub fn drape(
    input: DrapeInput,
    cancel: &AtomicBool,
    mut preview: impl FnMut(DrapedGarment),
) -> Result<DrapedGarment> {
    input.selection.validate()?;
    let cancelled = || -> Result<()> {
        if cancel.load(Ordering::Relaxed) {
            bail!("Draping cancelled");
        }
        Ok(())
    };
    cancelled()?;
    let body = measured_body(&input).context("measuring the character")?;
    cancelled()?;
    let design = input.selection.preset.design()?;
    let pattern = MetaGarment::new(input.selection.preset.label(), &body, &design).assembly();
    let fabric = input.selection.fabric.fabric();
    let settings = FitSettings {
        resolution_cm: input.selection.resolution_cm,
        body_height_cm: body.get("height") as f32,
        ..Default::default()
    };
    let build = build_garment(&pattern, &settings, &fabric)?;
    if !build.skipped.is_empty() || build.mesh.triangles.is_empty() {
        bail!(
            "garment meshing failed; skipped panels: {:?}",
            build.skipped
        );
    }
    cancelled()?;
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async {
            let context = fabelgeist_gpu::prelude::WgpuContext::new().await?;
            let mut fit = Fit::new(context, &build, fabric, &settings)?;
            // Pattern placement is measured from the floor and the pelvis centre.
            // Keep the actual body size: translate the garment, never scale the body.
            let floor = input
                .positions
                .iter()
                .map(|p| p[1])
                .fold(f32::INFINITY, f32::min);
            let hip =
                |name: &str| input.joints[input.names.iter().position(|n| n == name).unwrap()];
            let (left, right) = (hip("l_upleg"), hip("r_upleg"));
            let mut pose = fabelgeist_garment_fit::GarmentPose::for_mesh(&build.mesh);
            pose.garment.offset = Vec3::new(
                (left[0] + right[0]) * 0.5,
                floor,
                (left[2] + right[2]) * 0.5,
            );
            fit.set_pose(pose)?;
            let vertices = input
                .positions
                .iter()
                .copied()
                .map(vector)
                .collect::<Vec<_>>();
            let mut collision_vertices = vertices.clone();
            let mut collision_faces = input.faces.clone();
            for garment in &input.obstacles {
                let offset = collision_vertices.len() as u32;
                collision_vertices.extend(garment.positions.iter().copied().map(vector));
                collision_faces.extend(garment.faces.iter().map(|face| face.map(|i| i + offset)));
            }
            let collision = fabelgeist_bvh::TriangleBvh::new(
                collision_vertices.clone(),
                collision_faces.clone(),
            );
            let mut placed = fit.placed_positions();
            clear_panels(
                &mut placed,
                &build.mesh,
                &collision,
                settings.body_offset_cm * 0.01 + fabric.particle_radius(),
            );
            fit.cloth.particles.write_positions(&fit.context, &placed)?;
            fit.set_body(&vertices, &input.faces, &settings, &fabric)
                .await?;
            if !input.obstacles.is_empty() {
                fit.set_collision_mesh(&collision_vertices, &collision_faces, &settings, &fabric)?;
            }
            let surface = SewnSurface::new(
                build.mesh.positions.len(),
                &build.mesh.seams,
                &build.mesh.triangles,
            );
            // Close the seams before gravity can pull the still-separated panels
            // below their supporting shoulders or waistband.
            fit.solver.settings.gravity = Vec3::default();
            fit.solver.settings.damping = 8.0;
            for _ in 0..60 {
                cancelled()?;
                fit.step_and_wait(1.0 / 60.0).await?;
            }
            let sewn = fit.cloth.read_positions(&fit.context).await?;
            fit.cloth.particles.write_positions(&fit.context, &sewn)?;
            fit.solver.settings.gravity = Vec3::new(0.0, -9.81, 0.0);
            fit.solver.settings.damping = fabric.damping;
            let mut output = DrapedGarment {
                name: format!(
                    "{} · {}",
                    input.selection.preset.label(),
                    input.selection.fabric.label()
                ),
                positions: Vec::new(),
                normals: Vec::new(),
                faces: surface.faces.clone(),
                indices: Vec::new(),
                weights: Vec::new(),
                frame: 0,
            };
            for frame in 0..=input.selection.steps {
                cancelled()?;
                if frame > 0 {
                    fit.step_and_wait(1.0 / 60.0).await?;
                }
                if frame % 6 == 0 || frame == input.selection.steps {
                    let sewn = surface.positions(&fit.positions().await?);
                    output.positions = sewn;
                    clear_surface(
                        &mut output.positions,
                        &collision,
                        settings.body_offset_cm * 0.01 + fabric.particle_radius(),
                    );
                    if output.positions.iter().flatten().any(|x| !x.is_finite()) {
                        bail!("cloth simulation produced non-finite positions");
                    }
                    orient_faces(&output.positions, &mut output.faces, &collision);
                    output.normals = normals(&output.positions, &output.faces);
                    output.frame = frame;
                    preview(output.clone());
                }
            }
            cancelled()?;
            (output.indices, output.weights) = transfer_skin(&input, &output.positions)?;
            Ok(output)
        })
}

pub fn drape_outfit(
    inputs: Vec<DrapeInput>,
    cancel: &AtomicBool,
    mut preview: impl FnMut(Vec<DrapedGarment>),
) -> Result<Vec<DrapedGarment>> {
    let mut finished = Vec::new();
    for mut input in inputs {
        input.obstacles = finished.clone();
        let garment = drape(input, cancel, |current| {
            let mut snapshot = finished.clone();
            snapshot.push(current);
            preview(snapshot);
        })?;
        finished.push(garment);
    }
    Ok(finished)
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

fn clear_surface(positions: &mut [[f32; 3]], body: &fabelgeist_bvh::TriangleBvh, margin: f32) {
    for position in positions {
        let point = vector(*position);
        let Some((triangle, closest, _)) = body.closest_point(point, f32::MAX) else {
            continue;
        };
        let [a, b, c] = body.triangles[triangle as usize].map(|i| body.positions[i as usize]);
        let raw_normal = (b - a).cross(c - a);
        if raw_normal.length() <= 1e-10 {
            continue;
        }
        let normal = raw_normal / raw_normal.length();
        let signed = (point - closest).dot(normal);
        if signed < margin {
            *position = array(point + normal * (margin - signed));
        }
    }
}

struct SewnSurface {
    groups: Vec<Vec<usize>>,
    faces: Vec<[u32; 3]>,
}
impl SewnSurface {
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
        let faces = triangles
            .iter()
            .map(|face| face.map(|i| mapping[i as usize]))
            .filter(|face| face[0] != face[1] && face[1] != face[2] && face[2] != face[0])
            .collect();
        Self { groups, faces }
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
    direct.selection.preset = GarmentPreset::Shirt;
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
    if matches!(
        input.selection.preset,
        GarmentPreset::Dress | GarmentPreset::Skirt
    ) {
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
    use super::*;
    #[test]
    fn presets_build_sewn_patterns() {
        let body = Body::from_yaml_str(assets::BODIES[0].yaml).unwrap();
        for preset in GarmentPreset::ALL {
            let pattern = MetaGarment::new("test", &body, &preset.design().unwrap()).assembly();
            let build = build_garment(&pattern, &FitSettings::default(), &Fabric::COTTON).unwrap();
            assert!(
                build.skipped.is_empty(),
                "{}: {:?}",
                preset.label(),
                build.skipped
            );
            assert!(!build.mesh.triangles.is_empty());
        }
    }
    #[test]
    fn garment_skin_interpolates_and_normalizes_body_influences() {
        let input = DrapeInput {
            selection: GarmentSelection::default(),
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
            selection: GarmentSelection {
                preset: GarmentPreset::Dress,
                ..Default::default()
            },
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

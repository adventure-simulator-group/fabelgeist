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
        match self {
            Self::Cotton => Fabric::COTTON,
            Self::Silk => Fabric::SILK,
            Self::Denim => Fabric::DENIM,
            Self::Wool => Fabric::WOOL,
            Self::Jersey => Fabric::JERSEY,
        }
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
            fit.set_body(&vertices, &input.faces, &settings, &fabric)
                .await?;
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
                faces: build.mesh.triangles.clone(),
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
                    output.positions = fit.positions().await?;
                    if output.positions.iter().flatten().any(|x| !x.is_finite()) {
                        bail!("cloth simulation produced non-finite positions");
                    }
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

pub fn transfer_skin(
    input: &DrapeInput,
    positions: &[[f32; 3]],
) -> Result<(Vec<[u32; 8]>, Vec<[f32; 8]>)> {
    if input.indices.len() != input.positions.len() || input.weights.len() != input.positions.len()
    {
        bail!("body skin arrays do not match its vertices");
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

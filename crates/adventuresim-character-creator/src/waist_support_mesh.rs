//! Closed folded band sheets follow the same frozen waist sources on all bodies.
use crate::device_fit::DevicePiece;
use crate::waist_support_plan::{WaistBandStation, WaistSupportPlan};
use anyhow::{Context, Result, ensure};
use fabelgeist_armor::gpu::body::Skin;
use fabelgeist_armor::{
    ArmorComponent, ArmorComponentRole, BuiltPart, GownWaistSupport, Millimeters, PlateFace,
};
use fabelgeist_math::Vec3;
struct WaistBandMesh {
    part: BuiltPart,
    owners: Vec<usize>,
}
impl WaistSupportPlan {
    pub fn attach(
        &self,
        piece: &mut DevicePiece,
        join: &crate::skirt_join::SkirtJoinPlan,
    ) -> Result<()> {
        let first = join.patch.carrier_point_count();
        for (samples, height, role) in [
            (
                &self.main,
                self.design.main.width,
                ArmorComponentRole::WaistSupportBand,
            ),
            (
                &self.front,
                self.design.front_height,
                ArmorComponentRole::WaistFrontBand,
            ),
        ] {
            let samples = samples
                .iter()
                .map(|p| WaistBandStation {
                    source: p.source.shifted(first),
                    progress: p.progress,
                })
                .collect::<Vec<_>>();
            let fit = |part: &BuiltPart| self.mesh(part, &samples, height, role);
            let mesh = fit(&piece.base)?;
            let endpoints = piece
                .endpoints
                .iter()
                .map(|part| fit(part).map(|m| m.part))
                .collect::<Result<Vec<_>>>()?;
            let mut skin = Skin {
                joint_indices: fabelgeist_rig::PackedRigSkin::empty(
                    piece.skin.joint_indices.joint_count(),
                ),
                ..Skin::default()
            };
            for source in mesh.owners {
                skin.texcoords.push(
                    *piece
                        .skin
                        .texcoords
                        .get(source)
                        .context("waist support has no cloth chart owner")?,
                );
                anyhow::ensure!(
                    source < piece.skin.joint_indices.len(),
                    "waist support has no cloth joint owner"
                );
                skin.joint_indices
                    .push(piece.skin.joint_indices.vertex(source))?;
                skin.joint_weights.push(
                    *piece
                        .skin
                        .joint_weights
                        .get(source)
                        .context("waist support has no cloth skin owner")?,
                );
            }
            let skin_aliases = (0..mesh.part.positions.len()).collect();
            piece.append(DevicePiece {
                base: mesh.part,
                skin,
                endpoints,
                skin_aliases,
            })?;
        }
        self.attach_fasteners(piece, join)?;
        Ok(())
    }
    fn mesh(
        &self,
        part: &BuiltPart,
        samples: &[WaistBandStation],
        height: Millimeters,
        role: ArmorComponentRole,
    ) -> Result<WaistBandMesh> {
        let points = samples
            .iter()
            .map(|p| p.source.position(part))
            .collect::<Result<Vec<_>>>()?;
        ensure!(points.len() >= 2, "waist support has no fitted span");
        let columns = points.len();
        let count = columns * 2;
        let mut mesh = BuiltPart {
            positions: Vec::new(),
            normals: Vec::new(),
            texcoords: Some(Vec::new()),
            indices: Vec::new(),
            faces: Vec::new(),
            components: Vec::new(),
            grids: Vec::new(),
        };
        let mut owners = Vec::new();
        for layer in 0..2 {
            for row in 0..2 {
                for (i, &point) in points.iter().enumerate() {
                    let previous = i.saturating_sub(1);
                    let next = (i + 1).min(columns - 1);
                    let normal = (points[next] - points[previous])
                        .cross(Vec3::up())
                        .normalize();
                    let thickness = self.design.main.thickness.metres()
                        * f32::from(GownWaistSupport::FOLDED_PLY_COUNT);
                    let position = point
                        - normal * (self.inset.metres() + layer as f32 * thickness)
                        - Vec3::up() * height.metres() * row as f32;
                    mesh.positions.push([position.x, position.y, position.z]);
                    let normal = normal * if layer == 0 { 1. } else { -1. };
                    mesh.normals.push([normal.x, normal.y, normal.z]);
                    if let Some(uv) = &mut mesh.texcoords {
                        uv.push([samples[i].progress, row as f32]);
                    }
                    owners.push(samples[i].source.skin_vertex());
                }
            }
        }
        for i in 0..columns - 1 {
            let a = i as u32;
            let b = a + 1;
            let c = a + columns as u32;
            let d = c + 1;
            let n = count as u32;
            mesh.indices
                .extend([a, d, b, a, c, d, a + n, b + n, d + n, a + n, d + n, c + n]);
        }
        let mut boundary = (0..columns).collect::<Vec<_>>();
        boundary.extend((columns..count).rev());
        for i in 0..boundary.len() {
            let a = boundary[i] as u32;
            let b = boundary[(i + 1) % boundary.len()] as u32;
            let n = count as u32;
            mesh.indices.extend([a, b, b + n, a, b + n, a + n]);
        }
        mesh.faces.resize(mesh.indices.len() / 3, PlateFace::Outer);
        mesh.components.push(ArmorComponent {
            role,
            vertices: 0..mesh.positions.len(),
            indices: 0..mesh.indices.len(),
            hinge: None,
            mount: None,
            material: Some(self.design.main.fabric.material()),
            textile: Some(self.design.main.fabric.clone()),
        });
        Ok(WaistBandMesh { part: mesh, owners })
    }
}

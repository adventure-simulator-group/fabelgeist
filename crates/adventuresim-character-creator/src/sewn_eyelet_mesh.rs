//! Thread follows each actual aperture contour, using its frozen cloth owners.
use crate::cloth_fastener_mesh::{
    ClosureFrame, ClosureMaterial, CordEnds, CordPath, FastenerFrame, FastenerMesh,
};
use crate::cloth_opening::ClosureSide;
use crate::sewn_eyelet_plan::SewnEyeletPair;
use anyhow::{Context, Result};
use fabelgeist_armor::{BuiltPart, ClothLacing, LacingEyelet};
use fabelgeist_math::Vec3;
impl FastenerMesh {
    pub(crate) fn sewn_eyelets(
        design: &ClothLacing,
        pair: &SewnEyeletPair,
        first: usize,
        part: &BuiltPart,
    ) -> Result<Self> {
        let LacingEyelet::Sewn { fabric, .. } = &design.eyelet else {
            anyhow::bail!("sewn aperture thread requires cloth eyelets");
        };
        let material = ClosureMaterial {
            surface: fabric.material(),
            fabric: Some(fabric.clone()),
        };
        // Identity numerical handoff: contour points already are canonical fitted metres.
        let frame = ClosureFrame::new(FastenerFrame {
            left: Vec3::new(0., 0., 0.),
            right: Vec3::right(),
            normal: Vec3::forward(),
        })?;
        let radius = design.eyelet.gauge().metres() * 0.5;
        let mut mesh = Self::empty();
        for (eyelet, side) in [
            (&pair.left, ClosureSide::Left),
            (&pair.right, ClosureSide::Right),
        ] {
            let center = eyelet.center.shifted(first).position(part)?;
            let normal = eyelet.center.shifted(first).normal(part)?;
            let points = eyelet
                .boundary
                .iter()
                .map(|&v| {
                    let [x, y, z] = *part
                        .positions
                        .get(v + first)
                        .context("sewn eyelet contour has no fitted cloth source")?;
                    let point = Vec3::new(x, y, z);
                    let radial = point - center;
                    let radial = (radial - normal * radial.dot(normal)).normalize();
                    Ok(point + radial * radius + normal * radius)
                })
                .collect::<Result<Vec<_>>>()?;
            let first = mesh.part.positions.len();
            let first_index = mesh.part.indices.len();
            mesh.cord(
                &frame,
                &CordPath {
                    points,
                    normal,
                    ends: CordEnds::Joined,
                },
                radius,
                side,
                &material,
            );
            mesh.close_thread_chart(first, first_index)?;
        }
        Ok(mesh)
    }
}

impl FastenerMesh {
    /// Only shading copies: existing triangles retain identical positions,
    /// normals and owners; the joined contour material endpoint reaches V=1.
    /// No thread detail, faces, silhouette or physical cloth edges are added.
    fn close_thread_chart(&mut self, first: usize, first_index: usize) -> Result<()> {
        let chart = self
            .part
            .texcoords
            .as_mut()
            .context("sewn thread has no cloth chart")?;
        let sides = chart[first..].iter().take_while(|uv| uv[1] == 0.).count();
        anyhow::ensure!(
            sides >= 3,
            "sewn thread chart lacks its first cross section"
        );
        let end = self.part.positions.len();
        let last = end - sides;
        for i in 0..sides {
            self.part.positions.push(self.part.positions[first + i]);
            self.part.normals.push(self.part.normals[first + i]);
            chart.push([chart[first + i][0], 1.]);
            self.owners.push(self.owners[first + i]);
        }
        for tri in self.part.indices[first_index..].as_chunks_mut::<3>().0 {
            if tri.iter().any(|&v| (last..end).contains(&(v as usize))) {
                for v in tri {
                    if (first..first + sides).contains(&(*v as usize)) {
                        *v = (end + *v as usize - first) as u32;
                    }
                }
            }
        }
        let component = self
            .part
            .components
            .last_mut()
            .context("sewn thread has no material component")?;
        component.vertices.end = self.part.positions.len();
        Ok(())
    }
}

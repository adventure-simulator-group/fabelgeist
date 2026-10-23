//! Shared actual Boolean edge ownership for laces and separate cloth panels.
use crate::armor_frames::Wearer;
use crate::cloth_opening::{ClothEdgeStation, ClothOpening};
use crate::cloth_torso_frame::ClothTorsoFrame;
use crate::surface_cut::{CUT_VERTEX_SNAP_METRES, SurfaceCut, dot, interpolate};
use anyhow::{Context, Result, ensure};
use fabelgeist_armor::{GownDesign, PartFrame};

pub(crate) struct BodiceOpeningEdges {
    pub opening: ClothOpening,
    pub positions: Vec<[f32; 3]>,
}
impl BodiceOpeningEdges {
    pub fn new(
        cut: &SurfaceCut,
        body: &Wearer<'_>,
        head: &PartFrame,
        gown: &GownDesign,
    ) -> Result<Self> {
        let frame = ClothTorsoFrame::new(body, head)?;
        let opening = frame
            .opening_cut(&gown.bodice, gown.waist_band.metres())?
            .context("cloth attachment requires a physical bodice opening")?;
        let positions = cut
            .points
            .iter()
            .map(|p| {
                interpolate(
                    body.faces[p.triangle].map(|v| body.positions[v as usize]),
                    p.weights,
                )
            })
            .collect::<Vec<_>>();
        let borders = cut.boundary_loops(&positions)?;
        let edge = |sign: f32| -> Result<Vec<ClothEdgeStation>> {
            let plane = opening.edge_plane(&frame, sign);
            let eligible = |v: u32| {
                let p = positions[v as usize];
                let residual = dot(p, plane.normal) - plane.offset;
                let depth = dot(p, frame.anterior) - dot(frame.neck, frame.anterior);
                residual.abs() <= CUT_VERTEX_SNAP_METRES
                    && p[1] >= opening.hem - CUT_VERTEX_SNAP_METRES
                    && p[1] <= opening.top + CUT_VERTEX_SNAP_METRES
                    && depth * opening.side > 0.
            };
            let mut runs = Vec::new();
            for border in &borders {
                for start in 0..border.len() {
                    if !eligible(border[start])
                        || eligible(border[(start + border.len() - 1) % border.len()])
                    {
                        continue;
                    }
                    runs.push(
                        (0..border.len())
                            .map(|i| border[(start + i) % border.len()])
                            .take_while(|&v| eligible(v))
                            .collect::<Vec<_>>(),
                    );
                }
            }
            ensure!(
                runs.len() == 1,
                "cloth attachment needs one complete physical edge on each side"
            );
            let mut vertices = runs.pop().context("missing lacing edge")?;
            ensure!(vertices.len() >= 2, "lacing edge is too short");
            if positions[vertices[0] as usize][1]
                > positions[vertices[vertices.len() - 1] as usize][1]
            {
                vertices.reverse();
            }
            let low = positions[vertices[0] as usize][1];
            let high = positions[vertices[vertices.len() - 1] as usize][1];
            ensure!(
                high > low
                    && vertices
                        .windows(2)
                        .all(|v| positions[v[0] as usize][1] < positions[v[1] as usize][1]),
                "lacing edge must advance continuously along the opening"
            );
            Ok(vertices
                .into_iter()
                .map(|v| ClothEdgeStation {
                    vertex: v as usize,
                    progress: (positions[v as usize][1] - low) / (high - low),
                })
                .collect())
        };
        Ok(Self {
            opening: ClothOpening {
                left: edge(-1.)?,
                right: edge(1.)?,
            },
            positions,
        })
    }
}

//! Freeze fasteners on actual inner-band triangles and the outer bodice source.
use crate::{
    cloth_opening::{ClosureAnchors, ClothSurfaceSample},
    device_fit::DevicePiece,
    skirt_join::SkirtJoinPlan,
    waist_support_plan::{WaistClosingSide, WaistSupportPlan},
};
use anyhow::{Context, Result, ensure};
use fabelgeist_armor::{ArmorComponentRole, BuiltPart, ClothFastener, PlateFace};
use fabelgeist_math::Vec3;
const RING_FRAME_SPAN_METRES: f32 = 0.001;
pub(crate) struct WaistFastenerSources {
    pub parallel: Vec<ClosureAnchors>,
    pub return_sources: Option<WaistReturnSources>,
}
pub(crate) struct WaistReturnSources {
    pub ring: ClosureAnchors,
    pub hook: ClothSurfaceSample,
}
impl WaistFastenerSources {
    pub fn new(plan: &WaistSupportPlan, piece: &DevicePiece, join: &SkirtJoinPlan) -> Result<Self> {
        let design = plan
            .design
            .fasteners
            .as_ref()
            .context("waist fasteners have no design")?;
        let side = plan
            .closing_side
            .context("waist fasteners require a skirt dressing opening")?;
        let front_span = plan.design.front_width.metres();
        let height = plan.design.front_height.metres();
        let overlap = join
            .patch
            .dressing_opening
            .as_ref()
            .context("waist fasteners require a frozen slit")?
            .overlap_metres();
        let ClothFastener::HookEye { width, wire, .. } = design.closures.fastener else {
            anyhow::bail!("waist band requires hook and eye closures");
        };
        let inset = design.inset.metres();
        let eye_footprint = crate::cloth_fastener_mesh::FastenerMesh::eye_sewing_footprint(
            &design.closures.fastener,
        )?;
        let footprint =
            crate::cloth_fastener_mesh::FastenerMesh::sewing_footprint(&design.closures.fastener);
        ensure!(
            inset > eye_footprint.forward && overlap - inset > eye_footprint.back,
            "eye sewing footprint crosses a band closing edge"
        );
        let front_distance = inset + footprint.setback - wire.metres() * 0.5;
        ensure!(
            front_distance + footprint.setback < front_span,
            "hook sewn tail crosses the front band"
        );
        let mut parallel = Vec::new();
        for at in &design.closures.positions {
            let down = height * at.unit();
            let margin = width.metres() * 0.5 + wire.metres() * 0.5;
            ensure!(
                down > margin
                    && height - down > margin
                    && plan.design.main.width.metres() - down > margin,
                "waist fastener crosses a finished band edge"
            );
            let front = front_coordinate(side, front_distance / front_span);
            let main = main_coordinate(side, (overlap - inset) / plan.main_span_metres);
            parallel.push(ClosureAnchors {
                left: band_sample(
                    &piece.base,
                    ArmorComponentRole::WaistFrontBand,
                    [front, at.unit()],
                )?,
                right: band_sample(
                    &piece.base,
                    ArmorComponentRole::WaistSupportBand,
                    [main, down / plan.design.main.width.metres()],
                )?,
            });
        }
        let return_sources = design
            .return_hook
            .as_ref()
            .map(|_| Self::return_sources(plan, piece, join, side))
            .transpose()?;
        Ok(Self {
            parallel,
            return_sources,
        })
    }
    fn return_sources(
        plan: &WaistSupportPlan,
        piece: &DevicePiece,
        join: &SkirtJoinPlan,
        side: WaistClosingSide,
    ) -> Result<WaistReturnSources> {
        let design = plan
            .design
            .fasteners
            .as_ref()
            .context("missing waist closure")?;
        let lacing = plan
            .lacing
            .as_ref()
            .context("return hook has no metal bodice ring design")?;
        let closing = front_coordinate(side, 0.);
        let top = band_sample(
            &piece.base,
            ArmorComponentRole::WaistFrontBand,
            [closing, 0.],
        )?;
        let inner = band_sample(
            &piece.base,
            ArmorComponentRole::WaistFrontBand,
            [
                front_coordinate(
                    side,
                    RING_FRAME_SPAN_METRES / plan.design.front_width.metres(),
                ),
                0.,
            ],
        )?;
        let origin = top.position(&piece.base)?;
        let mut across = inner.position(&piece.base)? - origin;
        across.y = 0.;
        let across = across.normalize();
        let outward = -top.normal(&piece.base)?;
        let chart = BodiceSewingChart::new(origin, across, outward, piece, join, plan.body_points);
        let radius = lacing.ring_radius.metres() + lacing.eyelet.gauge().metres() * 0.5;
        // Anchor to the actual generated lower bodice boundary, independently
        // of sparse support-band chords and their nominal inward seating.
        let ring_across = -lacing.inset.metres();
        let bottom = [ring_across - radius, ring_across, ring_across + radius]
            .into_iter()
            .map(|x| {
                chart.hem_at(x).with_context(|| {
                    format!("return ring origin={origin:?} across={across:?} outward={outward:?}")
                })
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .fold(f32::NEG_INFINITY, f32::max);
        let query = [0., bottom + radius];
        let ring = ClosureAnchors {
            left: chart.sample(query)?,
            right: chart.sample([RING_FRAME_SPAN_METRES, query[1]])?,
        };
        for [x, y] in [[-radius, 0.], [radius, 0.], [0., -radius], [0., radius]] {
            chart.sample([ring_across + x, query[1] + y])?;
        }
        let hook_design = design
            .return_hook
            .as_ref()
            .context("return hook has no design")?;
        let footprint = crate::cloth_fastener_mesh::FastenerMesh::sewing_footprint(hook_design);
        // The return points toward the bodice, so its tail can turn upward or
        // sideways. Reserve the complete rotational footprint at both edges.
        let margin = footprint.setback.hypot(footprint.half_width);
        let hook = band_sample(
            &piece.base,
            ArmorComponentRole::WaistFrontBand,
            [
                front_coordinate(
                    side,
                    footprint.half_width / plan.design.front_width.metres(),
                ),
                design.inset.metres().max(margin) / plan.design.front_height.metres(),
            ],
        )?;
        Ok(WaistReturnSources { ring, hook })
    }
}
/// Native chart boundary: horizontal cloth tangent and canonical upright metres.
struct BodiceSewingChart {
    triangles: Vec<SewingTriangle>,
}
struct SewingTriangle {
    vertices: [usize; 3],
    points: [[f64; 2]; 3],
}
impl BodiceSewingChart {
    fn new(
        origin: Vec3,
        across: Vec3,
        outward: Vec3,
        piece: &DevicePiece,
        join: &SkirtJoinPlan,
        body_points: usize,
    ) -> Self {
        let first = join.patch.carrier_point_count();
        let body = first..first + body_points;
        let mut triangles = Vec::new();
        for (ordinal, triangle) in piece.base.indices.as_chunks::<3>().0.iter().enumerate() {
            if piece.base.faces[ordinal] != PlateFace::Outer
                || !triangle.iter().all(|&v| {
                    let raw = piece.skin_aliases[v as usize];
                    body.contains(
                        &join
                            .patch
                            .dressing_opening
                            .as_ref()
                            .map_or(raw, |o| o.source_vertex(raw)),
                    )
                })
            {
                continue;
            }
            let normals = triangle.map(|v| {
                let [x, y, z] = piece.base.normals[v as usize];
                Vec3::new(x, y, z)
            });
            if normals.iter().map(|n| n.dot(outward)).sum::<f32>() <= 0. {
                continue;
            }
            let points = triangle.map(|v| {
                let [x, y, z] = piece.base.positions[v as usize];
                let p = Vec3::new(x, y, z) - origin;
                [f64::from(p.dot(across)), f64::from(p.y)]
            });
            triangles.push(SewingTriangle {
                vertices: triangle.map(|v| v as usize),
                points,
            });
        }
        Self { triangles }
    }
    fn sample(&self, query: [f32; 2]) -> Result<ClothSurfaceSample> {
        for triangle in &self.triangles {
            if let Some(weights) =
                crate::cloth_chart_triangle::weights(triangle.points, query.map(f64::from))
            {
                return Ok(ClothSurfaceSample {
                    vertices: triangle.vertices,
                    weights,
                });
            }
        }
        anyhow::bail!(
            "waist ring sewing footprint has no positive front bodice source at chart metres {query:?}"
        )
    }
    fn hem_at(&self, across: f32) -> Result<f32> {
        let across = f64::from(across);
        let mut lowest = f64::INFINITY;
        for triangle in &self.triangles {
            for edge in 0..3 {
                let a = triangle.points[edge];
                let b = triangle.points[(edge + 1) % 3];
                if across < a[0].min(b[0]) || across > a[0].max(b[0]) {
                    continue;
                }
                if a[0] == b[0] {
                    lowest = lowest.min(a[1].min(b[1]));
                    continue;
                }
                let t = (across - a[0]) / (b[0] - a[0]);
                lowest = lowest.min(a[1] + (b[1] - a[1]) * t);
            }
        }
        ensure!(
            lowest.is_finite(),
            "waist ring has no actual lower bodice edge: across={across}m triangles={} chart_x={:?}",
            self.triangles.len(),
            self.triangles
                .iter()
                .flat_map(|t| t.points)
                .map(|p| p[0])
                .fold([f64::INFINITY, f64::NEG_INFINITY], |[lo, hi], x| [
                    lo.min(x),
                    hi.max(x)
                ])
        );
        Ok(lowest as f32)
    }
}
fn band_sample(
    part: &BuiltPart,
    role: ArmorComponentRole,
    query: [f32; 2],
) -> Result<ClothSurfaceSample> {
    let component = part
        .components
        .iter()
        .find(|c| c.role == role)
        .context("waist band is missing")?;
    let half = component.vertices.len() / 2;
    let inner = component.vertices.start + half..component.vertices.end;
    let uv = part
        .texcoords
        .as_ref()
        .context("waist band has no cloth chart")?;
    for triangle in part.indices[component.indices.clone()].as_chunks::<3>().0 {
        if !triangle.iter().all(|&v| inner.contains(&(v as usize))) {
            continue;
        }
        let points = triangle.map(|v| uv[v as usize].map(f64::from));
        if let Some(weights) = crate::cloth_chart_triangle::weights(points, query.map(f64::from)) {
            return Ok(ClothSurfaceSample {
                vertices: triangle.map(|v| v as usize),
                weights,
            });
        }
    }
    anyhow::bail!("waist closure has no positive inner band source")
}
fn front_coordinate(side: WaistClosingSide, distance: f32) -> f32 {
    match side {
        WaistClosingSide::FrontStart => distance,
        WaistClosingSide::FrontEnd => 1. - distance,
    }
}
fn main_coordinate(side: WaistClosingSide, distance: f32) -> f32 {
    match side {
        WaistClosingSide::FrontStart => 1. - distance,
        WaistClosingSide::FrontEnd => distance,
    }
}

/// Check the sewn tail in its actual nominal frame, projected onto the front
/// band's cloth chart. This admits no nearest source and does not solve contact.
pub(crate) fn validate_return_tail(
    part: &BuiltPart,
    design: &ClothFastener,
    frame: crate::cloth_fastener_mesh::FastenerFrame,
) -> Result<()> {
    let ClothFastener::HookEye { wire, .. } = design else {
        anyhow::bail!("waist return must be a hook");
    };
    let footprint = crate::cloth_fastener_mesh::FastenerMesh::sewing_footprint(design);
    let chart_across = frame.normal.cross(Vec3::up()).normalize();
    let local = crate::cloth_fastener_mesh::ClosureFrame::new(frame)?;
    let component = part
        .components
        .iter()
        .find(|c| c.role == ArmorComponentRole::WaistFrontBand)
        .context("return hook has no front band")?;
    let inner = component.vertices.start + component.vertices.len() / 2..component.vertices.end;
    for side in [-1., 1.] {
        let [x, y, z] = local.point(Vec3::new(
            -footprint.setback,
            side * footprint.half_width,
            wire.metres(),
        ));
        let point = Vec3::new(x, y, z) - frame.left;
        let query = [f64::from(point.dot(chart_across)), f64::from(point.y)];
        let supported = part.indices[component.indices.clone()]
            .as_chunks::<3>()
            .0
            .iter()
            .filter(|t| t.iter().all(|&v| inner.contains(&(v as usize))))
            .any(|triangle| {
                let points = triangle.map(|v| {
                    let [x, y, z] = part.positions[v as usize];
                    let point = Vec3::new(x, y, z) - frame.left;
                    [f64::from(point.dot(chart_across)), f64::from(point.y)]
                });
                crate::cloth_chart_triangle::weights(points, query).is_some()
            });
        ensure!(
            supported,
            "return hook sewn tail crosses the front band's finished edge: query={query:?} frame_left={:?} frame_right={:?} normal={:?}",
            frame.left,
            frame.right,
            frame.normal
        );
    }
    Ok(())
}

//! A raised neckline band sampled from frozen sources on the gown's face cloth.
//! Planning uses the canonical unposed reference in metres, Y up, front +Z.
use crate::armor_frames::Wearer;
use crate::cloth_opening::ClothSurfaceSample;
use crate::device_fit::DevicePiece;
use crate::surface_cut::SurfaceCut;
use anyhow::{Context, Result, ensure};
use bevy::math::Vec3;
use cloth::ReferenceCloth;
use fabelgeist_armor::gpu::body::Skin;
use fabelgeist_armor::{
    ArmorComponent, ArmorComponentRole, BuiltPart, GownNecklineTrim, PlateFace,
};
use outline::TrimOutline;

#[path = "neckline_trim_cloth.rs"]
mod cloth;
#[path = "neckline_trim_outline.rs"]
mod outline;

/// Fewest intervals across the band; its edge rows sit low and the rest
/// carry its crown.
const MINIMUM_TRIM_ROWS: usize = 3;
/// Reference distance between stations along and across the band, in
/// metres. A wider band needs more rows to follow a curved chest.
const TRIM_STATION_SPACING_M: f32 = 0.02;
/// Downward share of a column's direction at which the band reaches its
/// front width. Cloth over a bust tilts the direction before it is vertical.
const FULL_HANG: f32 = 0.8;
/// Share of the crown relief kept at the band's two edges.
const EDGE_RELIEF_SHARE: f32 = 0.35;

/// One station of the band: a frozen cloth source and its chart coordinate.
struct TrimStation {
    source: ClothSurfaceSample,
    chart: [f32; 2],
}

pub(crate) struct NecklineTrimPlan {
    design: GownNecklineTrim,
    /// Row-major stations: `rows + 1` per column, edge first.
    stations: Vec<TrimStation>,
    /// Intervals across the band.
    rows: usize,
    /// Whether each column is joined to the next by a strip of the band.
    joins: Vec<bool>,
}

impl NecklineTrimPlan {
    pub fn new(design: &GownNecklineTrim, cut: &SurfaceCut, body: &Wearer<'_>) -> Result<Self> {
        let cloth = ReferenceCloth::new(cut, body);
        let positions = cloth
            .points
            .iter()
            .map(|point| point.to_array())
            .collect::<Vec<_>>();
        let neckline = cut
            .boundary_loops(&positions)?
            .into_iter()
            .filter(|boundary| {
                // Armholes lie to one side; the neck and waist cross the midline.
                let side = |sign: f32| {
                    boundary
                        .iter()
                        .any(|&v| cloth.points[v as usize].x * sign > 0.)
                };
                side(-1.) && side(1.)
            })
            .max_by(|a, b| cloth.height(a).total_cmp(&cloth.height(b)))
            .context("gown has no neckline boundary for its trim")?;
        ensure!(neckline.len() >= 3, "gown neckline is degenerate");
        let depth = cloth.centre(&neckline).z;
        let outline = TrimOutline::new(&cloth, &neckline, TRIM_STATION_SPACING_M)?;
        let front_width = design.front_width.metres();
        let shoulder_width = design.shoulder_width.metres();
        let rows = ((front_width / TRIM_STATION_SPACING_M).ceil() as usize).max(MINIMUM_TRIM_ROWS);
        let mut stations = Vec::with_capacity(outline.columns.len() * (rows + 1));
        for column in &outline.columns {
            // The band deepens where it hangs below the front of the opening
            // and stays slim where it runs up the sides and over the shoulders.
            let hang = (-column.reach.normalize_or_zero().y / FULL_HANG).clamp(0., 1.);
            let front = if column.point.z > depth {
                hang * hang * (3. - 2. * hang)
            } else {
                0.
            };
            let width = shoulder_width + (front_width - shoulder_width) * front;
            let sources = cloth.rows(column, width, rows)?;
            for (row, source) in sources.into_iter().enumerate() {
                let inset = width * row as f32 / rows as f32;
                stations.push(TrimStation {
                    source,
                    chart: [
                        (column.arc + inset * column.shear) / outline.length,
                        inset / front_width,
                    ],
                });
            }
        }
        Ok(Self {
            design: design.clone(),
            stations,
            rows,
            joins: outline.columns.iter().map(|c| c.joins_next).collect(),
        })
    }

    /// `first` converts frozen face-cloth ordinals to final carrier vertices.
    pub fn attach(&self, piece: &mut DevicePiece, first: usize) -> Result<()> {
        let base = self.mesh(&piece.base, first)?;
        let endpoints = piece
            .endpoints
            .iter()
            .map(|part| self.mesh(part, first))
            .collect::<Result<Vec<_>>>()?;
        let mut skin = Skin {
            joint_indices: fabelgeist_rig::PackedRigSkin::empty(
                piece.skin.joint_indices.joint_count(),
            ),
            ..Skin::default()
        };
        // Edge and crown vertices of one station share its cloth owner.
        for station in self.stations.iter().chain(&self.stations) {
            let owner = station.source.skin_vertex() + first;
            ensure!(
                owner < piece.skin.joint_indices.len(),
                "neckline trim has no cloth joint owner"
            );
            skin.texcoords.push(station.chart);
            skin.joint_indices
                .push(piece.skin.joint_indices.vertex(owner))?;
            skin.joint_weights.push(
                *piece
                    .skin
                    .joint_weights
                    .get(owner)
                    .context("neckline trim has no cloth skin owner")?,
            );
        }
        let skin_aliases = (0..base.positions.len()).collect();
        piece.append(DevicePiece {
            base,
            skin,
            endpoints,
            skin_aliases,
        })
    }

    /// The raised sheet first, then the same stations seated on the cloth to
    /// close both of its edges.
    fn mesh(&self, part: &BuiltPart, first: usize) -> Result<BuiltPart> {
        let relief = self.design.relief.metres();
        let stride = self.rows + 1;
        let count = self.stations.len();
        let mut mesh = BuiltPart {
            positions: Vec::with_capacity(count * 2),
            normals: Vec::with_capacity(count * 2),
            texcoords: Some(Vec::with_capacity(count * 2)),
            indices: Vec::new(),
            faces: Vec::new(),
            components: Vec::new(),
            grids: Vec::new(),
        };
        for raised in [true, false] {
            for (index, station) in self.stations.iter().enumerate() {
                let source = station.source.shifted(first);
                let normal = Vec3::from_array(source.normal(part)?.to_array());
                let row = index % stride;
                let share = if row == 0 || row == self.rows {
                    EDGE_RELIEF_SHARE
                } else {
                    1.
                };
                let lift = if raised { relief * share } else { 0. };
                let position = Vec3::from_array(source.position(part)?.to_array()) + normal * lift;
                mesh.positions.push(position.to_array());
                mesh.normals.push(normal.to_array());
                if let Some(chart) = &mut mesh.texcoords {
                    chart.push(station.chart);
                }
            }
        }
        let at = |column: usize, row: usize| (column * stride + row) as u32;
        // Orient the sheet by the cloth normal of its first quad.
        let [a, b, c] =
            [at(0, 0), at(1, 0), at(0, 1)].map(|v| Vec3::from_array(mesh.positions[v as usize]));
        let outward = (b - a).cross(c - a).dot(Vec3::from_array(mesh.normals[0])) > 0.;
        let mut quad = |corners: [u32; 4]| {
            let [a, b, c, d] = corners;
            if outward {
                mesh.indices.extend([a, b, d, a, d, c]);
            } else {
                mesh.indices.extend([a, d, b, a, c, d]);
            }
        };
        let seated = count as u32;
        let joined = self.joins.iter().enumerate().filter(|(_, joins)| **joins);
        for (column, _) in joined {
            for row in 0..self.rows {
                quad([
                    at(column, row),
                    at(column + 1, row),
                    at(column, row + 1),
                    at(column + 1, row + 1),
                ]);
            }
            // Walls drop from each raised edge to the cloth it lies on.
            let (edge, far) = (at(column, 0), at(column + 1, 0));
            quad([edge + seated, far + seated, edge, far]);
            let (edge, far) = (at(column, self.rows), at(column + 1, self.rows));
            quad([edge, far, edge + seated, far + seated]);
        }
        mesh.faces.resize(mesh.indices.len() / 3, PlateFace::Outer);
        mesh.components.push(ArmorComponent {
            role: ArmorComponentRole::EdgeBinding,
            vertices: 0..mesh.positions.len(),
            indices: 0..mesh.indices.len(),
            hinge: None,
            mount: None,
            material: Some(self.design.fabric.material()),
            textile: Some(self.design.fabric.clone()),
        });
        Ok(mesh)
    }
}

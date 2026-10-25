//! Small closed solids appended after the garment's GPU fitting and skinning.
use crate::cloth_hook_mesh::{HOOK_TAIL_SETBACK_SHARE, HookDimensions};
use crate::cloth_opening::ClosureSide;
use anyhow::{Result, ensure};
use fabelgeist_armor::{
    ArmorComponent, SurfaceMaterial, ArmorComponentRole, BuiltPart, ClothFastener,
    PlateFace, TextileAppearance,
};
use fabelgeist_math::Vec3;
use std::f32::consts::{PI, TAU};

// Submillimetre cords need a closed silhouette; thread relief belongs in shading.
const CORD_SECTION_SIDES: usize = 3;
const WIRE_SECTION_SIDES: usize = 3;
const CLOSURE_CURVE_SEGMENTS: usize = 8;

/// Canonical template sewing footprint, in local cloth-plane metres. These
/// extents describe the sewn tail/button, excluding the deliberately free hook.
pub(crate) struct SewingFootprint {
    pub setback: f32,
    pub half_width: f32,
}

/// Canonical fitted metre positions and a surface normal at an opening.
#[derive(Clone, Copy)]
pub(crate) struct FastenerFrame {
    pub left: Vec3,
    pub right: Vec3,
    pub normal: Vec3,
}
pub(crate) struct FastenerMesh {
    pub part: BuiltPart,
    pub owners: Vec<ClosureSide>,
}
pub(crate) struct ClosureFrame {
    origin: Vec3,
    x: Vec3,
    y: Vec3,
    z: Vec3,
    pub gap: f32,
}
pub(crate) struct CordPath {
    pub points: Vec<Vec3>,
    pub normal: Vec3,
    pub ends: CordEnds,
}
pub(crate) enum CordEnds {
    Joined,
    Capped,
}

impl FastenerMesh {
    pub(crate) fn sewing_footprint(design: &ClothFastener) -> SewingFootprint {
        match design {
            ClothFastener::HookEye { length, wire, .. } => SewingFootprint {
                setback: length.metres() * HOOK_TAIL_SETBACK_SHARE + wire.metres() * 0.5,
                half_width: wire.metres() * 0.5,
            },
            ClothFastener::ButtonLoop { diameter, .. } => SewingFootprint {
                setback: diameter.metres(),
                half_width: diameter.metres() * 0.5,
            },
        }
    }

    pub(crate) fn empty() -> Self {
        Self {
            part: BuiltPart {
                positions: Vec::new(),
                normals: Vec::new(),
                texcoords: Some(Vec::new()),
                indices: Vec::new(),
                faces: Vec::new(),
                components: Vec::new(),
                grids: Vec::new(),
            },
            owners: Vec::new(),
        }
    }
    pub fn new(design: &ClothFastener, frame: FastenerFrame) -> Result<Self> {
        let frame = ClosureFrame::new(frame)?;
        let mut mesh = Self::empty();
        match design {
            ClothFastener::ButtonLoop {
                diameter,
                cord,
                fabric,
            } => mesh.button_loop(&frame, diameter.metres(), cord.metres(), fabric),
            ClothFastener::HookEye {
                length,
                width,
                wire,
                color,
                roughness,
            } => {
                let material = fabelgeist_armor::ComponentAppearance::Surface(SurfaceMaterial {
                        base_color: color.material().base_color,
                        metallic: fabelgeist_math::units::Ratio::from_ratio(1.0),
                        roughness: fabelgeist_math::units::Ratio::from_ratio(roughness.unit()),
                    });
                mesh.hook_eye(
                    &frame,
                    HookDimensions {
                        reach: length.metres(),
                        width: width.metres(),
                        gauge: wire.metres(),
                    },
                    &material,
                )?;
            }
        }
        Ok(mesh)
    }
    /// Native solid dimensions are metres in the local fastener frame.
    fn button_loop(
        &mut self,
        frame: &ClosureFrame,
        diameter: f32,
        cord: f32,
        fabric: &TextileAppearance,
    ) {
        let radius = diameter * 0.5;
        let gauge = cord;
        let material = fabelgeist_armor::ComponentAppearance::Textile(fabric.clone());
        let shank_height = gauge * 2.;
        self.bead(frame, radius, shank_height, &material);
        self.cord(
            frame,
            &CordPath {
                points: vec![
                    Vec3::new(-radius, 0., 0.),
                    Vec3::new(-radius, 0., shank_height + radius * 0.2),
                ],
                normal: Vec3::up(),
                ends: CordEnds::Capped,
            },
            gauge * 0.5,
            ClosureSide::Left,
            &material,
        );
        let turn_radius = (radius * 0.8).max(gauge * 2.);
        // Two semicircular returns joined by straight sides keep the shank
        // clearance independent of the opening's length.
        let half = CLOSURE_CURVE_SEGMENTS / 2;
        let path = (0..2)
            .flat_map(|end| {
                (0..=half).map(move |i| {
                    let angle = PI * (i as f32 / half as f32 + end as f32 - 0.5);
                    let center = if end == 0 { frame.gap } else { -radius };
                    Vec3::new(
                        center + turn_radius * angle.cos(),
                        turn_radius * angle.sin(),
                        gauge,
                    )
                })
            })
            .collect::<Vec<_>>();
        self.cord(
            frame,
            &CordPath {
                points: path,
                normal: Vec3::forward(),
                ends: CordEnds::Joined,
            },
            gauge * 0.5,
            ClosureSide::Right,
            &material,
        );
    }
    pub(crate) fn component(
        &mut self,
        vertices: usize,
        indices: usize,
        material: &fabelgeist_armor::ComponentAppearance,
    ) {
        self.part.components.push(ArmorComponent {
            role: ArmorComponentRole::ClothFasteners,
            vertices: vertices..self.part.positions.len(),
            indices: indices..self.part.indices.len(),
            hinge: None,
            mount: None,
            appearance: material.clone(),
        });
        self.part
            .faces
            .resize(self.part.indices.len() / 3, PlateFace::Outer);
    }
    pub(crate) fn point(
        &mut self,
        frame: &ClosureFrame,
        position: Vec3,
        normal: Vec3,
        uv: [f32; 2],
        side: ClosureSide,
    ) {
        self.part.positions.push(frame.point(position));
        self.part.normals.push(frame.vector(normal));
        if let Some(chart) = &mut self.part.texcoords {
            chart.push(uv);
        }
        self.owners.push(side);
    }
    pub(crate) fn cord(
        &mut self,
        frame: &ClosureFrame,
        curve: &CordPath,
        radius: f32,
        side: ClosureSide,
        material: &fabelgeist_armor::ComponentAppearance,
    ) {
        let sides = if material.textile().is_some() {
            CORD_SECTION_SIDES
        } else {
            WIRE_SECTION_SIDES
        };
        let path = &curve.points;
        let closed = matches!(curve.ends, CordEnds::Joined);
        let first = self.part.positions.len();
        let index = self.part.indices.len();
        for (station, &center) in path.iter().enumerate() {
            let previous = if station > 0 {
                path[station - 1]
            } else if closed {
                path[path.len() - 1]
            } else {
                center
            };
            let next = if station + 1 < path.len() {
                path[station + 1]
            } else if closed {
                path[0]
            } else {
                center
            };
            let tangent = (next - previous).normalize();
            let across = tangent.cross(curve.normal).normalize();
            let normal = tangent.cross(across);
            for column in 0..sides {
                let angle = TAU * column as f32 / sides as f32;
                let outward = across * angle.cos() + normal * angle.sin();
                self.point(
                    frame,
                    center + outward * radius,
                    outward,
                    [
                        column as f32 / sides as f32,
                        station as f32 / path.len() as f32,
                    ],
                    side,
                );
            }
        }
        let spans = path.len() - usize::from(!closed);
        for station in 0..spans {
            for column in 0..sides {
                let a = (first + station * sides + column) as u32;
                let b = (first + station * sides + (column + 1) % sides) as u32;
                let c = (first + ((station + 1) % path.len()) * sides + column) as u32;
                let d =
                    (first + ((station + 1) % path.len()) * sides + (column + 1) % sides) as u32;
                self.part.indices.extend([a, b, c, b, d, c]);
            }
        }
        if !closed {
            for column in 1..sides - 1 {
                self.part.indices.extend([
                    first as u32,
                    (first + column + 1) as u32,
                    (first + column) as u32,
                ]);
                let last = first + (path.len() - 1) * sides;
                self.part.indices.extend([
                    last as u32,
                    (last + column) as u32,
                    (last + column + 1) as u32,
                ]);
            }
        }
        self.component(first, index, material);
    }
    fn bead(&mut self, frame: &ClosureFrame, radius: f32, rise: f32, material: &fabelgeist_armor::ComponentAppearance) {
        // Standard unit icosahedron, in the fastener frame; bead size is metres.
        let golden = (1. + 5_f32.sqrt()) * 0.5;
        let points = [
            [-1., golden, 0.],
            [1., golden, 0.],
            [-1., -golden, 0.],
            [1., -golden, 0.],
            [0., -1., golden],
            [0., 1., golden],
            [0., -1., -golden],
            [0., 1., -golden],
            [golden, 0., -1.],
            [golden, 0., 1.],
            [-golden, 0., -1.],
            [-golden, 0., 1.],
        ];
        let first = self.part.positions.len();
        let index = self.part.indices.len();
        for [x, y, z] in points {
            let normal = Vec3::new(x, y, z).normalize();
            self.point(
                frame,
                Vec3::new(-radius, 0., radius + rise) + normal * radius,
                normal,
                [normal.x * 0.5 + 0.5, normal.y * 0.5 + 0.5],
                ClosureSide::Left,
            );
        }
        let faces = [
            [0, 11, 5],
            [0, 5, 1],
            [0, 1, 7],
            [0, 7, 10],
            [0, 10, 11],
            [1, 5, 9],
            [5, 11, 4],
            [11, 10, 2],
            [10, 7, 6],
            [7, 1, 8],
            [3, 9, 4],
            [3, 4, 2],
            [3, 2, 6],
            [3, 6, 8],
            [3, 8, 9],
            [4, 9, 5],
            [2, 4, 11],
            [6, 2, 10],
            [8, 6, 7],
            [9, 8, 1],
        ];
        self.part
            .indices
            .extend(faces.into_iter().flatten().map(|i| i + first as u32));
        self.component(first, index, material);
    }
}
impl ClosureFrame {
    pub(crate) fn new(frame: FastenerFrame) -> Result<Self> {
        let delta = frame.right - frame.left;
        let gap = delta.x.hypot(delta.y).hypot(delta.z);
        ensure!(gap > 0. && gap.is_finite(), "cloth closure edges coincide");
        let x = delta / gap;
        let y = frame.normal.cross(x).normalize();
        let z = x.cross(y).normalize();
        ensure!(
            y.length_squared() > 0.5 && z.length_squared() > 0.5,
            "degenerate cloth closure frame"
        );
        Ok(Self {
            origin: frame.left,
            x,
            y,
            z,
            gap,
        })
    }
    pub(crate) fn vector(&self, v: Vec3) -> [f32; 3] {
        let p = self.x * v.x + self.y * v.y + self.z * v.z;
        [p.x, p.y, p.z]
    }
    pub(crate) fn point(&self, p: Vec3) -> [f32; 3] {
        let v = self.vector(p);
        [
            self.origin.x + v[0],
            self.origin.y + v[1],
            self.origin.z + v[2],
        ]
    }
}

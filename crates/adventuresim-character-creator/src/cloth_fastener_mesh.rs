//! Small closed solids appended after the garment's GPU fitting and skinning.
use crate::cloth_opening::ClosureSide;
use anyhow::{Result, ensure};
use fabelgeist_armor::{
    ArmorComponent, ArmorComponentMaterial, ArmorComponentRole, BuiltPart, ClothFastener,
    PlateFace, TextileAppearance,
};
use fabelgeist_math::Vec3;
use std::f32::consts::{PI, TAU};

const CORD_SECTION_SIDES: usize = 4;
const CLOSURE_CURVE_SEGMENTS: usize = 8;

/// Canonical fitted metre positions and a surface normal at an opening.
pub(crate) struct FastenerFrame {
    pub left: Vec3,
    pub right: Vec3,
    pub normal: Vec3,
}
pub(crate) struct FastenerMesh {
    pub part: BuiltPart,
    pub owners: Vec<ClosureSide>,
}
struct ClosureFrame {
    origin: Vec3,
    x: Vec3,
    y: Vec3,
    z: Vec3,
    gap: f32,
}
/// Native hook template dimensions in metres.
struct HookDimensions {
    reach: f32,
    width: f32,
    gauge: f32,
}
struct ClosureMaterial {
    surface: ArmorComponentMaterial,
    fabric: Option<TextileAppearance>,
}
struct CordPath {
    points: Vec<Vec3>,
    normal: Vec3,
    ends: CordEnds,
}
enum CordEnds {
    Joined,
    Capped,
}

impl FastenerMesh {
    pub fn new(design: &ClothFastener, frame: FastenerFrame) -> Result<Self> {
        let frame = ClosureFrame::new(frame)?;
        let mut mesh = Self {
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
        };
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
                let material = ClosureMaterial {
                    surface: ArmorComponentMaterial {
                        base_color: color.material().base_color,
                        metallic: 1.,
                        roughness: roughness.unit(),
                    },
                    fabric: None,
                };
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
        let material = ClosureMaterial {
            surface: fabric.material(),
            fabric: Some(fabric.clone()),
        };
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
    fn hook_eye(
        &mut self,
        frame: &ClosureFrame,
        dimensions: HookDimensions,
        material: &ClosureMaterial,
    ) -> Result<()> {
        let reach = dimensions.reach;
        let width = dimensions.width;
        let gauge = dimensions.gauge;
        ensure!(frame.gap < reach, "hook cannot bridge this cloth opening");
        let eye = (0..CLOSURE_CURVE_SEGMENTS)
            .map(|i| {
                let t = TAU * i as f32 / CLOSURE_CURVE_SEGMENTS as f32;
                Vec3::new(
                    frame.gap + reach * 0.35 + reach * 0.5 * t.cos(),
                    width * 0.5 * t.sin(),
                    gauge,
                )
            })
            .collect::<Vec<_>>();
        self.cord(
            frame,
            &CordPath {
                points: eye,
                normal: Vec3::forward(),
                ends: CordEnds::Joined,
            },
            gauge * 0.5,
            ClosureSide::Right,
            material,
        );
        let mut hook = vec![Vec3::new(-reach * 0.6, 0., gauge)];
        hook.extend((0..CLOSURE_CURVE_SEGMENTS).map(|i| {
            let t = PI + PI * 1.5 * i as f32 / (CLOSURE_CURVE_SEGMENTS - 1) as f32;
            Vec3::new(
                frame.gap + width * 0.2 + width * 0.4 * t.cos(),
                0.,
                gauge + width * 0.4 * (1. + t.sin()),
            )
        }));
        self.cord(
            frame,
            &CordPath {
                points: hook,
                normal: Vec3::up(),
                ends: CordEnds::Capped,
            },
            gauge * 0.5,
            ClosureSide::Left,
            material,
        );
        Ok(())
    }
    fn component(&mut self, vertices: usize, indices: usize, material: &ClosureMaterial) {
        self.part.components.push(ArmorComponent {
            role: ArmorComponentRole::ClothFasteners,
            vertices: vertices..self.part.positions.len(),
            indices: indices..self.part.indices.len(),
            hinge: None,
            mount: None,
            material: Some(material.surface),
            textile: material.fabric.clone(),
        });
        self.part
            .faces
            .resize(self.part.indices.len() / 3, PlateFace::Outer);
    }
    fn point(
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
    fn cord(
        &mut self,
        frame: &ClosureFrame,
        curve: &CordPath,
        radius: f32,
        side: ClosureSide,
        material: &ClosureMaterial,
    ) {
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
            for column in 0..CORD_SECTION_SIDES {
                let angle = TAU * column as f32 / CORD_SECTION_SIDES as f32;
                let outward = across * angle.cos() + normal * angle.sin();
                self.point(
                    frame,
                    center + outward * radius,
                    outward,
                    [
                        column as f32 / CORD_SECTION_SIDES as f32,
                        station as f32 / path.len() as f32,
                    ],
                    side,
                );
            }
        }
        let spans = path.len() - usize::from(!closed);
        for station in 0..spans {
            for column in 0..CORD_SECTION_SIDES {
                let a = (first + station * CORD_SECTION_SIDES + column) as u32;
                let b = (first + station * CORD_SECTION_SIDES + (column + 1) % CORD_SECTION_SIDES)
                    as u32;
                let c = (first + ((station + 1) % path.len()) * CORD_SECTION_SIDES + column) as u32;
                let d = (first
                    + ((station + 1) % path.len()) * CORD_SECTION_SIDES
                    + (column + 1) % CORD_SECTION_SIDES) as u32;
                self.part.indices.extend([a, b, c, b, d, c]);
            }
        }
        if !closed {
            for column in 1..CORD_SECTION_SIDES - 1 {
                self.part.indices.extend([
                    first as u32,
                    (first + column + 1) as u32,
                    (first + column) as u32,
                ]);
                let last = first + (path.len() - 1) * CORD_SECTION_SIDES;
                self.part.indices.extend([
                    last as u32,
                    (last + column) as u32,
                    (last + column + 1) as u32,
                ]);
            }
        }
        self.component(first, index, material);
    }
    fn bead(&mut self, frame: &ClosureFrame, radius: f32, rise: f32, material: &ClosureMaterial) {
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
    fn new(frame: FastenerFrame) -> Result<Self> {
        let delta = frame.right - frame.left;
        let gap = delta.length();
        ensure!(
            gap > f32::EPSILON && gap.is_finite(),
            "cloth closure edges coincide"
        );
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
    fn vector(&self, v: Vec3) -> [f32; 3] {
        let p = self.x * v.x + self.y * v.y + self.z * v.z;
        [p.x, p.y, p.z]
    }
    fn point(&self, p: Vec3) -> [f32; 3] {
        let v = self.vector(p);
        [
            self.origin.x + v[0],
            self.origin.y + v[1],
            self.origin.z + v[2],
        ]
    }
}

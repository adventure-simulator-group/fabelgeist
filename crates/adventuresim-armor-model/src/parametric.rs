//! Construction-independent geometry and anatomical placement for armor families.

use std::collections::BTreeMap;
#[path = "parametric_normals.rs"]
mod normals;
#[path = "plate_edges.rs"]
mod plate_edges;

use crate::{ArmorComponent, ArmorComponentRole, ArmorHinge, GenerateError, SurfaceRelief};

/// Anatomical placement, separate from a recipe's artistic controls.
/// Local coordinates are metres. Reflected frames are supported explicitly.
#[derive(Clone, Copy, Debug)]
pub struct PartFrame {
    pub origin: [f32; 3],
    pub axes: [[f32; 3]; 3],
    pub half_extents: [f32; 3],
}

impl PartFrame {
    pub fn point(&self, local: [f32; 3]) -> [f32; 3] {
        std::array::from_fn(|axis| {
            self.origin[axis] + (0..3).map(|i| self.axes[i][axis] * local[i]).sum::<f32>()
        })
    }

    pub fn validate(&self) -> Result<(), GenerateError> {
        let finite = self
            .origin
            .iter()
            .chain(self.axes.iter().flatten())
            .chain(self.half_extents.iter())
            .all(|v| v.is_finite());
        let orthogonal = (0..3).all(|i| {
            (0..3).all(|j| {
                (dot(self.axes[i], self.axes[j]) - if i == j { 1.0 } else { 0.0 }).abs() < 0.001
            })
        });
        if !finite || !orthogonal || self.half_extents.iter().any(|v| *v <= 0.0) {
            return Err(GenerateError::InvalidSurface);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct PartMesh {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub components: Vec<ArmorComponent>,
    shells: Vec<ShellLayout>,
}

#[derive(Clone, Debug)]
struct ShellLayout {
    first_vertex: usize,
    vertex_count: usize,
    complete_vertex_count: usize,
    first_index: usize,
    index_count: usize,
    thickness: f32,
    boundary_normals: BoundaryNormals,
    extrusion: ShellExtrusion,
    relief: Option<ReliefCarrier>,
}

#[derive(Clone, Debug)]
struct ReliefCarrier {
    carrier: Vec<[f32; 3]>,
    field: SurfaceRelief,
}

#[derive(Clone, Copy, Debug)]
pub enum ShellExtrusion {
    Normal,
    /// Angular contributions preserve narrow creases under unequal tessellation.
    AngleWeightedNormal,
    /// Rounded terminal plates return toward their cap's center.
    CappedAxis {
        origin: [f32; 3],
        axis: [f32; 3],
    },
    /// A graph-like plate returns along one direction without moving its trim boundary.
    Along {
        direction: [f32; 3],
    },
    /// Cut returns towards an axis while retaining normal plate gauge. Visor
    /// returns stay horizontal and do not shear a narrow bridge sideways when
    /// neighboring triangles have different slopes around an ocular ledge.
    Radial {
        origin: [f32; 3],
        axis: [f32; 3],
    },
}

impl ShellExtrusion {
    pub(crate) fn offset(
        self,
        point: [f32; 3],
        normal_vector: [f32; 3],
    ) -> Result<[f32; 3], GenerateError> {
        let offset = match self {
            ShellExtrusion::Normal | ShellExtrusion::AngleWeightedNormal => normal_vector,
            ShellExtrusion::CappedAxis { origin, axis } => {
                let delta = subtract(point, origin);
                let along = dot(delta, axis).max(0.0);
                let radial = subtract(delta, axis.map(|v| v * along));
                let projection = dot(radial, normal_vector);
                if !origin.iter().chain(&axis).all(|v| v.is_finite())
                    || (dot(axis, axis) - 1.0).abs() > 0.001
                    || projection <= 1e-6
                {
                    return Err(GenerateError::InvalidSurface);
                }
                radial.map(|v| v / projection)
            }
            ShellExtrusion::Along { direction } => {
                let projection = dot(direction, normal_vector);
                if !direction.iter().all(|v| v.is_finite())
                    || (dot(direction, direction) - 1.0).abs() > 0.001
                    || projection <= 1e-6
                {
                    return Err(GenerateError::InvalidSurface);
                }
                direction.map(|v| v / projection)
            }
            ShellExtrusion::Radial { origin, axis } => {
                if !axis.iter().chain(&origin).all(|v| v.is_finite())
                    || (dot(axis, axis) - 1.0).abs() > 0.001
                {
                    return Err(GenerateError::InvalidSurface);
                }
                let radial = subtract(point, origin);
                let projected = subtract(radial, axis.map(|v| v * dot(radial, axis)));
                let normal_projection = dot(projected, normal_vector);
                const MINIMUM_RADIAL_PROJECTION_M: f32 = 1e-6;
                if normal_projection < MINIMUM_RADIAL_PROJECTION_M {
                    return Err(GenerateError::InvalidSurface);
                }
                projected.map(|v| v / normal_projection)
            }
        };
        Ok(offset)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum BoundaryNormals {
    Smooth,
    /// Duplicate return-wall vertices so plate surfaces keep their own normals.
    Separate,
}

impl PartMesh {
    /// Physical sheet ranges include outer, inner and hard-normal return aliases.
    /// Distinct sheets remain distinct even where their surfaces touch exactly.
    pub fn shell_vertex_ranges(&self) -> impl Iterator<Item = std::ops::Range<usize>> + '_ {
        self.shells
            .iter()
            .map(|shell| shell.first_vertex..shell.first_vertex + shell.complete_vertex_count)
    }
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_component(mut self, role: ArmorComponentRole, hinge: Option<ArmorHinge>) -> Self {
        self.components = vec![ArmorComponent {
            role,
            vertices: 0..self.positions.len(),
            indices: 0..self.indices.len(),
            hinge,
            material: None,
        }];
        self
    }

    pub fn append(&mut self, other: Self) {
        let offset = self.positions.len() as u32;
        let index_offset = self.indices.len();
        self.components
            .extend(other.components.into_iter().map(|mut part| {
                part.vertices =
                    part.vertices.start + offset as usize..part.vertices.end + offset as usize;
                part.indices = part.indices.start + index_offset..part.indices.end + index_offset;
                part
            }));
        self.shells
            .extend(other.shells.into_iter().map(|mut shell| {
                shell.first_vertex += offset as usize;
                shell.first_index += index_offset;
                shell
            }));
        self.positions.extend(other.positions);
        self.indices
            .extend(other.indices.into_iter().map(|i| i + offset));
    }

    /// Refit authored carriers, then rebuild their inner walls at physical gauge.
    /// Carrier vertex correspondence and connectivity remain unchanged.
    pub fn refit_surfaces(
        &self,
        mut fit: impl FnMut(&mut [[f32; 3]], &[u32]),
    ) -> Result<Self, GenerateError> {
        let mut result = Self::new();
        for shell in &self.shells {
            let mut positions = shell.relief.as_ref().map_or_else(
                || {
                    self.positions[shell.first_vertex..shell.first_vertex + shell.vertex_count]
                        .to_vec()
                },
                |relief| relief.carrier.clone(),
            );
            let indices: Vec<_> = self.indices
                [shell.first_index..shell.first_index + shell.index_count]
                .iter()
                .map(|i| i - shell.first_vertex as u32)
                .collect();
            fit(&mut positions, &indices);
            result.append(Self::from_relief_surface(
                positions,
                indices,
                shell.thickness,
                shell.boundary_normals,
                shell.extrusion,
                shell.relief.as_ref().map(|relief| relief.field.clone()),
            )?);
        }
        result.components = self.components.clone();
        Ok(result)
    }

    pub fn transformed(mut self, frame: &PartFrame) -> Self {
        self.positions.iter_mut().for_each(|p| *p = frame.point(*p));
        for shell in &mut self.shells {
            if let Some(relief) = &mut shell.relief {
                relief.carrier.iter_mut().for_each(|p| *p = frame.point(*p));
                relief.field.transform(frame);
            }
            if let ShellExtrusion::Along { direction } = &mut shell.extrusion {
                *direction = std::array::from_fn(|coordinate| {
                    (0..3)
                        .map(|i| frame.axes[i][coordinate] * direction[i])
                        .sum()
                });
            }
            if let ShellExtrusion::CappedAxis { origin, axis } = &mut shell.extrusion {
                *origin = frame.point(*origin);
                *axis = std::array::from_fn(|coordinate| {
                    (0..3).map(|i| frame.axes[i][coordinate] * axis[i]).sum()
                });
            }
            if let ShellExtrusion::Radial { origin, axis } = &mut shell.extrusion {
                *origin = frame.point(*origin);
                *axis = std::array::from_fn(|coordinate| {
                    (0..3).map(|i| frame.axes[i][coordinate] * axis[i]).sum()
                });
            }
        }
        for part in &mut self.components {
            if let Some(hinge) = &mut part.hinge {
                hinge.origin = frame.point(hinge.origin);
                hinge.axis = std::array::from_fn(|axis| {
                    (0..3).map(|i| frame.axes[i][axis] * hinge.axis[i]).sum()
                });
            }
        }
        if dot(cross(frame.axes[0], frame.axes[1]), frame.axes[2]) < 0.0 {
            for triangle in self.indices.as_chunks_mut::<3>().0 {
                triangle.swap(1, 2);
            }
        }
        self
    }

    /// Area-weighted vertex normals, with finite, index and triangle checks.
    pub fn normals(&self) -> Result<Vec<[f32; 3]>, GenerateError> {
        self.validate_components()?;
        if self.positions.is_empty()
            || self.indices.is_empty()
            || !self.indices.len().is_multiple_of(3)
            || self.positions.iter().flatten().any(|v| !v.is_finite())
            || self
                .indices
                .iter()
                .any(|i| *i as usize >= self.positions.len())
        {
            return Err(GenerateError::InvalidSurface);
        }
        let mut normals = vec![[0.0; 3]; self.positions.len()];
        for triangle in self.indices.as_chunks::<3>().0 {
            let [a, b, c] =
                [triangle[0], triangle[1], triangle[2]].map(|i| self.positions[i as usize]);
            let normal = cross(subtract(b, a), subtract(c, a));
            if dot(normal, normal) <= 1e-18 {
                return Err(GenerateError::Degenerate);
            }
            for i in triangle {
                normals[*i as usize] = add(normals[*i as usize], normal);
            }
        }
        normals
            .into_iter()
            .map(|n| {
                let magnitude = dot(n, n).sqrt();
                if magnitude <= 1e-12 {
                    Err(GenerateError::Degenerate)
                } else {
                    Ok(n.map(|v| v / magnitude))
                }
            })
            .collect()
    }

    fn validate_components(&self) -> Result<(), GenerateError> {
        if self.components.is_empty() {
            return Ok(());
        }
        let (mut vertices, mut indices) = (0, 0);
        for (i, part) in self.components.iter().enumerate() {
            let valid_hinge = part.hinge.is_none_or(|hinge| {
                hinge
                    .origin
                    .iter()
                    .chain(&hinge.axis)
                    .all(|x| x.is_finite())
                    && (dot(hinge.axis, hinge.axis) - 1.0).abs() < 0.0001
            });
            if part.vertices.start != vertices
                || part.indices.start != indices
                || part.vertices.is_empty()
                || part.indices.is_empty()
                || part.vertices.end > self.positions.len()
                || part.indices.end > self.indices.len()
                || !part.indices.end.is_multiple_of(3)
                || !valid_hinge
                || self.components[..i]
                    .iter()
                    .any(|other| other.role == part.role)
                || self.indices[part.indices.clone()]
                    .iter()
                    .any(|v| !part.vertices.contains(&(*v as usize)))
            {
                return Err(GenerateError::InvalidSurface);
            }
            vertices = part.vertices.end;
            indices = part.indices.end;
        }
        if vertices != self.positions.len() || indices != self.indices.len() {
            return Err(GenerateError::InvalidSurface);
        }
        Ok(())
    }

    /// Thicken an outward-wound carrier and close every boundary edge.
    /// The input must have shared indices along its intended seams.
    pub fn from_surface(
        positions: Vec<[f32; 3]>,
        indices: Vec<u32>,
        thickness: f32,
        boundary_normals: BoundaryNormals,
        extrusion: ShellExtrusion,
    ) -> Result<Self, GenerateError> {
        Self::from_relief_surface(
            positions,
            indices,
            thickness,
            boundary_normals,
            extrusion,
            None,
        )
    }

    /// Add authored relief on both walls independently of physical gauge. Fitting uses
    /// the retained carrier and reapplies relief, so neither fit nor gauge is
    /// derived from the tight curvature of the flute troughs.
    pub fn from_relief_surface(
        positions: Vec<[f32; 3]>,
        indices: Vec<u32>,
        thickness: f32,
        boundary_normals: BoundaryNormals,
        extrusion: ShellExtrusion,
        relief: Option<SurfaceRelief>,
    ) -> Result<Self, GenerateError> {
        if !thickness.is_finite() || thickness <= 0.0 {
            return Err(GenerateError::InvalidSurface);
        }
        if let Some(field) = &relief {
            field.validate(positions.len())?;
        }
        let layout = ShellLayout {
            first_vertex: 0,
            vertex_count: positions.len(),
            complete_vertex_count: 0,
            first_index: 0,
            index_count: indices.len(),
            thickness,
            boundary_normals,
            extrusion,
            relief: relief.map(|field| ReliefCarrier {
                carrier: positions.clone(),
                field,
            }),
        };
        let mut mesh = Self {
            positions,
            indices,
            components: Vec::new(),
            shells: vec![layout],
        };
        let normals = match extrusion {
            ShellExtrusion::AngleWeightedNormal => mesh.angle_weighted_normals()?,
            _ => mesh.normals()?,
        };
        let count = mesh.positions.len() as u32;
        let mut edges = BTreeMap::<(u32, u32), Vec<(u32, u32)>>::new();
        for triangle in mesh.indices.as_chunks::<3>().0 {
            for (a, b) in [
                (triangle[0], triangle[1]),
                (triangle[1], triangle[2]),
                (triangle[2], triangle[0]),
            ] {
                edges.entry((a.min(b), a.max(b))).or_default().push((a, b));
            }
        }
        if edges
            .values()
            .any(|e| e.len() > 2 || (e.len() == 2 && e[0] != (e[1].1, e[1].0)))
        {
            return Err(GenerateError::InvalidSurface);
        }
        if let Some(relief) = &mesh.shells[0].relief {
            for (index, (point, normal)) in mesh.positions.iter_mut().zip(&normals).enumerate() {
                *point = add(
                    *point,
                    relief.field.offset(index, extrusion, *point, *normal)?,
                );
            }
        }
        let inner = mesh
            .positions
            .iter()
            .zip(normals)
            .map(|(p, n)| {
                let offset = extrusion.offset(*p, n)?;
                Ok(std::array::from_fn(|axis| {
                    p[axis] - thickness * offset[axis]
                }))
            })
            .collect::<Result<Vec<_>, GenerateError>>()?;
        let inner_indices = mesh
            .indices
            .as_chunks::<3>()
            .0
            .iter()
            .flat_map(|t| [t[0] + count, t[2] + count, t[1] + count])
            .collect::<Vec<_>>();
        mesh.positions.extend(inner);
        mesh.indices.extend(inner_indices);
        for edge in edges.values().filter(|e| e.len() == 1) {
            let (a, b) = edge[0];
            mesh.append_return([a, b, a + count, b + count], boundary_normals);
        }
        mesh.shells[0].complete_vertex_count = mesh.positions.len();
        mesh.normals()?;
        Ok(mesh)
    }

    fn append_return(&mut self, [a, b, c, d]: [u32; 4], shading: BoundaryNormals) {
        // Keep alias ordering independent of winding, including reflected fits.
        let reversed = a > b;
        let [a, b, c, d] = if reversed { [b, a, d, c] } else { [a, b, c, d] };
        let [a, b, c, d] = match shading {
            BoundaryNormals::Smooth => [a, b, c, d],
            BoundaryNormals::Separate => {
                let offset = self.positions.len() as u32;
                let points = [a, b, c, d].map(|i| self.positions[i as usize]);
                self.positions.extend(points);
                [offset, offset + 1, offset + 2, offset + 3]
            }
        };
        self.indices.extend(if reversed {
            [b, c, a, b, d, c]
        } else {
            [b, a, c, b, c, d]
        });
    }
}

pub(crate) fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    (0..3).map(|i| a[i] * b[i]).sum()
}
pub(crate) fn subtract(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
pub(crate) fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| a[i] + b[i])
}
pub(crate) fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

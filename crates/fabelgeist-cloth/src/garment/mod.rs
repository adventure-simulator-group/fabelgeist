//! Panels, seams, and the build that turns them into a simulable garment.
//!
//! This is the shape a sewing pattern actually has: flat pieces, each placed
//! somewhere around the body, and a list of which edge is sewn to which. The
//! build meshes each panel, moves it to its place, and turns the seam list
//! into constraints that pull the panels together.
//!
//! Nothing here knows about any particular pattern library. A caller converts
//! its own pattern format into [`Panel`]s and [`Seam`]s, which is a small
//! adapter rather than a dependency in either direction.

use anyhow::anyhow;
use fabelgeist_math::{Vec2, Vec3};

use crate::topology::{self, BendQuad};
use crate::triangulate::{PanelMesh, triangulate};

/// Where a flat panel sits in space.
///
/// Rotation is XYZ Euler angles in degrees, applied X then Y then Z, which is
/// the convention sewing-pattern formats tend to use. The panel's own plane is
/// XY, so an unrotated panel faces +Z.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    pub translation: Vec3,
    pub rotation: Vec3,
}

impl Default for Placement {
    fn default() -> Self {
        Self {
            translation: Vec3::default(),
            rotation: Vec3::default(),
        }
    }
}

impl Placement {
    /// Lift a point from the panel's plane into world space.
    pub fn apply(&self, point: Vec2) -> Vec3 {
        let local = Vec3::new(point.x, point.y, 0.0);
        rotate_xyz(local, self.rotation) + self.translation
    }
}

/// XYZ Euler rotation, in degrees.
pub fn rotate_xyz(point: Vec3, degrees: Vec3) -> Vec3 {
    let (x, y, z) = (
        degrees.x.to_radians(),
        degrees.y.to_radians(),
        degrees.z.to_radians(),
    );

    let (sx, cx) = x.sin_cos();
    let after_x = Vec3::new(
        point.x,
        point.y * cx - point.z * sx,
        point.y * sx + point.z * cx,
    );

    let (sy, cy) = y.sin_cos();
    let after_y = Vec3::new(
        after_x.x * cy + after_x.z * sy,
        after_x.y,
        -after_x.x * sy + after_x.z * cy,
    );

    let (sz, cz) = z.sin_cos();
    Vec3::new(
        after_y.x * cz - after_y.y * sz,
        after_y.x * sz + after_y.y * cz,
        after_y.z,
    )
}

/// One flat piece of a pattern.
#[derive(Clone, Debug)]
pub struct Panel {
    pub name: String,
    /// A simple closed polygon, in the panel's own plane. No repeated final
    /// point.
    pub outline: Vec<Vec2>,
    pub placement: Placement,
}

impl Panel {
    pub fn new(name: impl Into<String>, outline: Vec<Vec2>) -> Self {
        Self {
            name: name.into(),
            outline,
            placement: Placement::default(),
        }
    }

    pub fn placed(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }
}

/// One side of a seam: an edge of a panel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SeamSide {
    pub panel: usize,
    /// Index into that panel's outline edges: edge `i` runs from vertex `i` to
    /// vertex `i + 1`.
    pub edge: usize,
}

/// Two edges sewn together.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Seam {
    pub a: SeamSide,
    pub b: SeamSide,
    /// Sew the edges head-to-tail rather than head-to-head. Which one is right
    /// depends on how the two panels are wound and which way they face; a
    /// pattern format usually says.
    pub reversed: bool,
}

impl Seam {
    pub fn new(a: SeamSide, b: SeamSide) -> Self {
        Self {
            a,
            b,
            reversed: false,
        }
    }

    pub fn reversed(mut self, reversed: bool) -> Self {
        self.reversed = reversed;
        self
    }
}

/// A meshed garment: everything the solver needs, still on the host.
#[derive(Clone, Debug, Default)]
pub struct GarmentMesh {
    /// Initial particle positions, in world space.
    pub positions: Vec<Vec3>,
    /// The same particles in material space: where each one sits in the flat
    /// plane of the panel it was cut from, before that panel was placed.
    ///
    /// This is the fabric's rest shape, which is what every stretch and
    /// bending rest value below was measured from -- and it is the only
    /// record of it, because [`positions`](Self::positions) has already been
    /// rotated and translated into place. Kept so that a caller can draw the
    /// garment flat, the way it would be cut.
    ///
    /// Each panel is in its *own* plane, so panels overlap here; laying them
    /// out side by side is the caller's business, and its bounds are what a
    /// layout needs.
    pub material: Vec<Vec2>,
    /// Triangles, indexing `positions`. Also the render mesh.
    pub triangles: Vec<[u32; 3]>,
    /// Stretch constraints, with the rest length each was built at.
    pub edges: Vec<[u32; 2]>,
    pub rest_lengths: Vec<f32>,
    /// Bending constraints, and eight floats each: the four affine weights,
    /// the rest value, and three of padding so the kernel can index by a
    /// power of two.
    pub bends: Vec<BendQuad>,
    pub bend_weights: Vec<[f32; 8]>,
    /// Seam constraints: particle pairs to be pulled together.
    pub seams: Vec<[u32; 2]>,
    /// Per-particle mass, from the area it carries and the fabric density.
    pub masses: Vec<f32>,
    /// Where each panel's vertices start, plus a final total.
    pub panel_offsets: Vec<u32>,
    pub panel_names: Vec<String>,
}

impl GarmentMesh {
    pub fn particle_count(&self) -> usize {
        self.positions.len()
    }

    pub fn panel_count(&self) -> usize {
        self.panel_names.len()
    }

    /// The particles belonging to one panel, as a range into the vertex
    /// arrays. Panels are contiguous, which is what makes this a range rather
    /// than a list.
    pub fn panel_range(&self, panel: usize) -> std::ops::Range<usize> {
        let start = self.panel_offsets[panel] as usize;
        let end = self.panel_offsets[panel + 1] as usize;
        start..end
    }

    /// Which panel a particle belongs to.
    pub fn panel_of(&self, particle: u32) -> usize {
        match self.panel_offsets.binary_search(&particle) {
            Ok(index) => index,
            Err(index) => index - 1,
        }
    }

    pub fn inverse_masses(&self) -> Vec<f32> {
        topology::inverse_masses(&self.masses)
    }

    /// The mesh's edge adjacency, for the self-collision pass to skip.
    pub fn adjacency(&self) -> Vec<Vec<u32>> {
        let mut adjacency = crate::SelfCollision::adjacency(self.positions.len(), &self.edges);
        // Seam partners are held together by a constraint too, and a
        // self-collision pass fighting a seam is what keeps a garment from
        // ever closing.
        for &[a, b] in &self.seams {
            adjacency[a as usize].push(b);
            adjacency[b as usize].push(a);
        }
        for list in &mut adjacency {
            list.sort_unstable();
            list.dedup();
        }
        adjacency
    }
}

/// Mesh a set of panels and their seams.
///
/// `target_edge` sets the resolution: it is the edge length the mesher aims
/// for, and so it decides both the particle count and the finest fold the
/// fabric can make.
pub fn build(
    panels: &[Panel],
    seams: &[Seam],
    target_edge: f32,
    density: f32,
) -> anyhow::Result<GarmentMesh> {
    if panels.is_empty() {
        return Ok(GarmentMesh::default());
    }
    if !(target_edge > 0.0) {
        return Err(anyhow!("target edge must be positive, got {target_edge}"));
    }

    let mut mesh = GarmentMesh::default();
    let mut meshes: Vec<PanelMesh> = Vec::with_capacity(panels.len());

    for panel in panels {
        let offset = mesh.positions.len() as u32;
        mesh.panel_offsets.push(offset);
        mesh.panel_names.push(panel.name.clone());

        let panel_mesh = triangulate(&panel.outline, target_edge);
        for vertex in &panel_mesh.vertices {
            mesh.positions.push(panel.placement.apply(*vertex));
            mesh.material.push(*vertex);
        }
        for triangle in &panel_mesh.triangles {
            mesh.triangles.push([
                triangle[0] + offset,
                triangle[1] + offset,
                triangle[2] + offset,
            ]);
        }
        meshes.push(panel_mesh);
    }
    mesh.panel_offsets.push(mesh.positions.len() as u32);

    // Stretch and bending come straight from the triangles, and their rest
    // states from the flat layout -- which is the point of a sewing pattern:
    // the panel's own shape is the fabric's rest shape.
    let derived = topology::build(&mesh.triangles);
    mesh.rest_lengths = derived
        .edges
        .iter()
        .map(|&[a, b]| (mesh.positions[a as usize] - mesh.positions[b as usize]).length())
        .collect();
    mesh.edges = derived.edges;

    // A hinge across a seam does not exist -- seams are distance constraints,
    // not shared triangles -- so a garment creases at its seams, which is what
    // a real one does.
    //
    // The weights come from the flat layout, which is the fabric's rest shape,
    // so the rest value is zero for every hinge in a flat panel. A hinge whose
    // quad is degenerate has no unique weights and is dropped: leaving it in
    // with arbitrary ones would apply a force out of nothing.
    for bend in derived.bends {
        let [i0, i1, i2, i3] = bend.particles();
        let points = [
            mesh.positions[i0 as usize],
            mesh.positions[i1 as usize],
            mesh.positions[i2 as usize],
            mesh.positions[i3 as usize],
        ];
        let Some(weights) = topology::bending_weights(points) else {
            continue;
        };
        let rest = points
            .iter()
            .zip(&weights)
            .fold(Vec3::default(), |acc, (p, &k)| acc + *p * k)
            .length();
        mesh.bends.push(bend);
        mesh.bend_weights.push([
            weights[0], weights[1], weights[2], weights[3], rest, 0.0, 0.0, 0.0,
        ]);
    }

    for (index, seam) in seams.iter().enumerate() {
        let pairs = seam_pairs(seam, &meshes, &mesh.panel_offsets)
            .map_err(|error| anyhow!("seam {index} ({:?} to {:?}): {error}", seam.a, seam.b))?;
        mesh.seams.extend(pairs);
    }
    mesh.seams.sort_unstable();
    mesh.seams.dedup();
    // A pair that is already a mesh edge would be two constraints on the same
    // two particles, fighting over the rest length.
    let existing: std::collections::HashSet<[u32; 2]> = mesh.edges.iter().copied().collect();
    mesh.seams
        .retain(|&[a, b]| a != b && !existing.contains(&[a.min(b), a.max(b)]));

    mesh.masses = topology::vertex_masses(&mesh.positions, &mesh.triangles, density);
    Ok(mesh)
}

/// Pair up the particles along two sewn edges.
///
/// The two edges rarely have the same number of vertices -- they are different
/// lengths, and each was resampled independently -- so both are walked by
/// fraction of their length rather than by index. The finer of the two sets
/// the number of stitches, so no vertex on either edge is left unsewn.
fn seam_pairs(seam: &Seam, meshes: &[PanelMesh], offsets: &[u32]) -> anyhow::Result<Vec<[u32; 2]>> {
    let chain_a = chain(seam.a, meshes, offsets)?;
    let mut chain_b = chain(seam.b, meshes, offsets)?;
    if seam.reversed {
        chain_b.reverse();
    }
    if chain_a.len() < 2 || chain_b.len() < 2 {
        return Err(anyhow!("an edge has fewer than two vertices"));
    }

    let stitches = chain_a.len().max(chain_b.len());
    let mut pairs = Vec::with_capacity(stitches);
    for stitch in 0..stitches {
        let t = stitch as f32 / (stitches - 1) as f32;
        let a = chain_a[((t * (chain_a.len() - 1) as f32).round() as usize).min(chain_a.len() - 1)];
        let b = chain_b[((t * (chain_b.len() - 1) as f32).round() as usize).min(chain_b.len() - 1)];
        pairs.push([a.min(b), a.max(b)]);
    }
    Ok(pairs)
}

fn chain(side: SeamSide, meshes: &[PanelMesh], offsets: &[u32]) -> anyhow::Result<Vec<u32>> {
    let mesh = meshes
        .get(side.panel)
        .ok_or_else(|| anyhow!("no panel {}", side.panel))?;
    let chain = mesh
        .edge_chains
        .get(side.edge)
        .ok_or_else(|| anyhow!("panel {} has no edge {}", side.panel, side.edge))?;
    let offset = offsets[side.panel];
    Ok(chain.iter().map(|&v| v + offset).collect())
}

#[cfg(test)]
mod tests;

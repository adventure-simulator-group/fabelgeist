//! Admission checks before serialized records can become GPU addresses.

use bevy::{
    mesh::{Indices, VertexAttributeValues},
    prelude::Mesh,
};
use std::collections::HashSet;

use fabelgeist_botany::viewer::sim::{BranchInstance, LeafInstance, PlantInstance, SpeciesInfo};
use fabelgeist_math::units::Metres;

use super::{
    AddressMesh, ForestFrame, LibraryId, PreparedError, PreparedUnique, RecordSpan, Result,
    TreeBounds, TreeLibrary, TreePopulation, TreePose,
};
use crate::render::{ForestDetail, levels::Level, shapes};

pub(super) fn unique(unique: &PreparedUnique) -> Result<()> {
    let snapshot = &unique.snapshot;
    frame(&ForestFrame::from_unique(unique))?;
    require(
        !snapshot.fillets && snapshot.blend.shapes.is_empty(),
        "prepared renderer requires plain junctions",
    )?;
    require(
        snapshot.plant_bounds.len() == unique.roots.len() && snapshot.plants == unique.roots.len(),
        "woody plant roots/count mismatch; vines/proxies are unsupported",
    )?;
    let mut identities = HashSet::new();
    for root in &unique.roots {
        require(
            identities.insert(root.id) && root.position.iter().all(|v| v.metres().is_finite()),
            "duplicate plant identity or nonfinite root",
        )?;
    }
    records(
        &snapshot.species,
        &snapshot.plant_bounds,
        &snapshot.branches,
        &snapshot.leaves,
    )
}

pub(super) fn library(library: &TreeLibrary) -> Result<()> {
    detail(&library.detail)?;
    records(
        &library.species,
        &library.plants,
        &library.branches,
        &library.leaves,
    )?;
    require(
        library.plants.len() == library.models.len() && !library.models.is_empty(),
        "model/plant count",
    )?;
    let mut branch_start = 0;
    let mut leaf_start = 0;
    for (index, model) in library.models.iter().enumerate() {
        bounds(model.bounds)?;
        require(
            model.species.index() < library.species.len(),
            "model species reference",
        )?;
        require(
            model.branches.start as usize == branch_start
                && model.leaves.start as usize == leaf_start,
            "noncontiguous model records",
        )?;
        branch_start = end(model.branches, library.branches.len())?;
        leaf_start = end(model.leaves, library.leaves.len())?;
        require(
            library.plants[index].species as usize == model.species.index(),
            "model species mismatch",
        )?;
        require(
            library.branches[model.branches.start as usize..branch_start]
                .iter()
                .all(|b| b.plant as usize == index),
            "branch model reference",
        )?;
        require(
            library.leaves[model.leaves.start as usize..leaf_start]
                .iter()
                .all(|l| l.plant as usize == index && l.species as usize == model.species.index()),
            "leaf model reference",
        )?;
        let mut required = TreeBounds {
            low: [Metres::from_metres(0.0); 3],
            high: [Metres::from_metres(0.0); 3],
        };
        for branch in &library.branches[model.branches.start as usize..branch_start] {
            required.include_branch(branch);
        }
        for leaf in &library.leaves[model.leaves.start as usize..leaf_start] {
            required.include_leaf(
                leaf,
                library.species[model.species.index()].foliage,
                &library.detail,
            );
        }
        require(
            (0..3).all(|axis| {
                model.bounds.low[axis] <= required.low[axis]
                    && model.bounds.high[axis] >= required.high[axis]
            }),
            "model bounds do not cover shader geometry",
        )?;
        for (level, meshes) in Level::ALL.into_iter().zip(&model.meshes) {
            let shape = level.shape(&library.detail);
            address_mesh(
                &meshes.branches,
                model.branches,
                shapes::branch_grid(shape.sides, shape.spans),
                1,
            )?;
            address_mesh(
                &meshes.leaves,
                model.leaves,
                shapes::leaf_blades(shape.blades),
                shape.blades,
            )?;
        }
    }
    require(
        branch_start == library.branches.len() && leaf_start == library.leaves.len(),
        "unowned library records",
    )
}

pub(super) fn population(
    population: &TreePopulation,
    library: &TreeLibrary,
    id: LibraryId,
) -> Result<()> {
    require(
        population.library == id,
        "population/library content digest mismatch",
    )?;
    frame(&population.frame)?;
    require(
        population.frame.detail == library.detail,
        "population/library detail mismatch",
    )?;
    require(
        u32::try_from(population.placements.len()).is_ok(),
        "GPU placement index overflow",
    )?;
    require(
        population.placements.len() == population.frame.source.plants,
        "population/source count mismatch",
    )?;
    let mut identities = HashSet::new();
    for placement in &population.placements {
        require(
            identities.insert(placement.id),
            "duplicate placement identity",
        )?;
        let model = library
            .models
            .get(placement.variant.index())
            .ok_or(PreparedError::Invalid("placement variant reference"))?;
        placement.pose.validate()?;
        bounds(model.bounds.transformed(placement.pose))?;
    }
    Ok(())
}

impl TreePose {
    pub(super) fn validate(self) -> Result<()> {
        require(
            self.origin().iter().all(|v| v.metres().is_finite())
                && self.horizontal().ratio().is_finite()
                && self.horizontal().ratio() > 0.0
                && self.vertical().ratio().is_finite()
                && self.vertical().ratio() > 0.0
                && self.yaw().radians().is_finite(),
            "nonfinite or nonpositive tree transform",
        )
    }
}

fn frame(frame: &ForestFrame) -> Result<()> {
    detail(&frame.detail)?;
    bounds(TreeBounds {
        low: frame.bounds[0].map(Metres::from_metres),
        high: frame.bounds[1].map(Metres::from_metres),
    })?;
    require(
        frame.time.is_finite()
            && frame.time >= 0.0
            && frame.focus.is_none_or(|focus| {
                focus.root.iter().all(|v| v.metres().is_finite())
                    && focus.height.metres().is_finite()
                    && focus.height.metres() >= 0.0
            }),
        "source age or camera framing",
    )?;
    if let Some(terrain) = &frame.terrain {
        surface(&terrain.indices, terrain.vertices.len())?;
        require(
            terrain.vertices.iter().all(|v| {
                v.position
                    .into_iter()
                    .chain(v.normal)
                    .chain(v.color)
                    .all(f32::is_finite)
            }),
            "nonfinite terrain vertex",
        )?;
    }
    if let Some(structures) = &frame.structures {
        surface(&structures.indices, structures.vertices.len())?;
        require(
            structures.vertices.iter().all(|v| {
                v.position
                    .into_iter()
                    .chain(v.normal)
                    .chain(v.color)
                    .chain([v.material])
                    .all(f32::is_finite)
            }),
            "nonfinite structure vertex",
        )?;
    }
    Ok(())
}

fn detail(detail: &ForestDetail) -> Result<()> {
    // Fixed prototype resource bounds prevent overflow and unreasonable grids.
    const MAX_GRID: u32 = 64;
    require(
        (3..=MAX_GRID).contains(&detail.tube_sides)
            && (1..=MAX_GRID).contains(&detail.tube_spans)
            && (1..=MAX_GRID).contains(&detail.leaf_blades)
            && detail.full_pixels.is_finite()
            && detail.middle_pixels.is_finite()
            && detail.full_pixels > detail.middle_pixels
            && detail.middle_pixels > 0.0,
        "invalid forest detail",
    )
}

fn records(
    species: &[SpeciesInfo],
    plants: &[PlantInstance],
    branches: &[BranchInstance],
    leaves: &[LeafInstance],
) -> Result<()> {
    require(
        !species.is_empty()
            && species
                .iter()
                .all(|s| s.bark.into_iter().chain(s.leaf).all(f32::is_finite)),
        "species colors",
    )?;
    for plant in plants {
        require(
            (plant.species as usize) < species.len()
                && plant.sphere.iter().all(|v| v.is_finite())
                && plant.sphere[3] >= 0.0,
            "plant record",
        )?;
    }
    for branch in branches {
        require(
            (branch.plant as usize) < plants.len()
                && branch
                    .a
                    .into_iter()
                    .chain(branch.b)
                    .chain([
                        branch.v0,
                        branch.v1,
                        branch.light,
                        branch.light_start,
                        branch.exit,
                    ])
                    .all(f32::is_finite)
                && branch.a[3] >= 0.0
                && branch.b[3] >= 0.0,
            "branch record",
        )?;
    }
    for leaf in leaves {
        require(
            (leaf.plant as usize) < plants.len()
                && (leaf.species as usize) < species.len()
                && leaf
                    .center
                    .into_iter()
                    .chain(leaf.axis)
                    .chain(leaf.crown)
                    .chain([leaf.size, leaf.light, leaf.seed])
                    .all(f32::is_finite)
                && leaf.size >= 0.0
                && leaf.crown[3] >= 0.0
                && leaf.axis.iter().map(|x| x * x).sum::<f32>() > 0.0,
            "leaf record",
        )?;
    }
    Ok(())
}

fn address_mesh(mesh: &AddressMesh, span: RecordSpan, grid: Mesh, stride: u32) -> Result<()> {
    let Some(VertexAttributeValues::Float32x3(vertices)) = grid.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        return Err(PreparedError::Invalid("canonical grid positions"));
    };
    let Some(Indices::U32(indices)) = grid.indices() else {
        return Err(PreparedError::Invalid("canonical grid indices"));
    };
    require(
        mesh.vertices.len() as u64 == u64::from(span.count) * vertices.len() as u64
            && mesh.indices.len() as u64 == u64::from(span.count) * indices.len() as u64,
        "address mesh cardinality",
    )?;
    surface(&mesh.indices, mesh.vertices.len())?;
    let high = (u64::from(span.start) + u64::from(span.count)) * u64::from(stride);
    require(
        high <= super::meshes::EXACT_ADDRESS,
        "inexact GPU record address",
    )?;
    for (index, vertex) in mesh.vertices.iter().enumerate() {
        let [x, y, z] = vertices[index % vertices.len()];
        let record = span.start as usize + index / vertices.len();
        require(
            *vertex == [x, y, z + (record as u64 * u64::from(stride)) as f32],
            "noncanonical GPU vertex address",
        )?;
    }
    for (index, &triangle) in mesh.indices.iter().enumerate() {
        let record = index / indices.len();
        require(
            u64::from(triangle)
                == u64::from(indices[index % indices.len()])
                    + record as u64 * vertices.len() as u64,
            "noncanonical address mesh triangles",
        )?;
    }
    Ok(())
}

fn surface(indices: &[u32], vertices: usize) -> Result<()> {
    require(
        indices.len().is_multiple_of(3) && indices.iter().all(|&index| (index as usize) < vertices),
        "triangle index bounds",
    )
}
fn bounds(bounds: TreeBounds) -> Result<()> {
    require(
        (0..3).all(|axis| {
            bounds.low[axis].metres().is_finite()
                && bounds.high[axis].metres().is_finite()
                && bounds.low[axis] <= bounds.high[axis]
        }),
        "nonfinite or inverted bounds",
    )
}
fn end(span: RecordSpan, length: usize) -> Result<usize> {
    let end = (span.start as usize)
        .checked_add(span.count as usize)
        .ok_or(PreparedError::Invalid("record range overflow"))?;
    require(end <= length, "record range bounds")?;
    Ok(end)
}
fn require(condition: bool, message: &'static str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(PreparedError::Invalid(message))
    }
}

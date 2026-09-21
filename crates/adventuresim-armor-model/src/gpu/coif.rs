//! The mail coif on the device, placed by the head frame, from the drape
//! measured on the wearer.
//!
//! The hood's rings, the neck tube and the two flaps' rows are decided by the
//! design alone; each vertex is evaluated on the device from the fit buffer:
//! the head frame's [`FIT_PROFILE_WORD`](super::FIT_PROFILE_WORD) words, then the wearer's
//! [`COIF_DRAPE_WORDS`](super::COIF_DRAPE_WORDS) drape floats.

use std::f32::consts::TAU;

use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::Buffer;

use super::close_helmet_dome::{AROUND, DOME, full_dome};
use super::coif_shape::SHAPE;
use super::coord::{CoordExtrusion, CoordKernel, CoordShell};
use super::coord_topology::CoordTopology;
use super::recipe::PartRecipe;
use super::{ArmorGpu, DevicePart};
use crate::{BoundaryNormals, CoifDesign, GenerateError, HelmetDesign};

const HOOD_ROWS: usize = 8;
const NECK_ROWS: usize = 8;
const FLAP_ROWS: usize = 10;
const FACE_EDGE: usize = AROUND / 8;
const FLAP_EDGE: usize = AROUND / 12;
/// Vertex kinds past the dome's, in a coordinate's fourth float.
const KIND_HOOD: f32 = 4.0;
const KIND_NECK: f32 = 5.0;
const KIND_FRONT_FLAP: f32 = 6.0;
const KIND_BACK_FLAP: f32 = 7.0;

/// A ring vertex's angle around the head, as its sine and cosine.
fn around(i: usize) -> [f32; 2] {
    let angle = i as f32 / AROUND as f32 * TAU;
    [angle.sin(), angle.cos()]
}

/// The coif's carrier topology: the dome, the hood rows open at the face,
/// the closed neck tube, then the front and back flaps hung from its last
/// ring.
fn carrier() -> CoordTopology {
    let mut surface = CoordTopology::default();
    let rim = full_dome(&mut surface);
    let mut previous = rim[FACE_EDGE..=AROUND - FACE_EDGE].to_vec();
    let mut neck = Vec::new();
    for row in 1..=HOOD_ROWS {
        let t = row as f32 / HOOD_ROWS as f32;
        let full = row == HOOD_ROWS;
        let range = if full {
            0..AROUND
        } else {
            FACE_EDGE..AROUND - FACE_EDGE + 1
        };
        let ring = range
            .map(|i| {
                let [s, c] = around(i);
                surface.vertex([s, c, t, KIND_HOOD])
            })
            .collect::<Vec<_>>();
        let strip = if full {
            &ring[FACE_EDGE..=AROUND - FACE_EDGE]
        } else {
            &ring
        };
        surface.connect(&previous, strip, false);
        if full {
            neck = ring;
        } else {
            previous = ring;
        }
    }
    let mut previous = neck;
    for row in 1..=NECK_ROWS {
        let t = row as f32 / NECK_ROWS as f32;
        let ring = (0..AROUND)
            .map(|i| {
                let [s, c] = around(i);
                surface.vertex([s, c, t, KIND_NECK])
            })
            .collect::<Vec<_>>();
        surface.connect(&previous, &ring, true);
        previous = ring;
    }
    let neck = previous;
    for (columns, kind) in [
        (front_columns(), KIND_FRONT_FLAP),
        (back_columns(), KIND_BACK_FLAP),
    ] {
        let mut previous = columns.iter().map(|i| neck[*i]).collect::<Vec<_>>();
        for row in 1..=FLAP_ROWS {
            let t = row as f32 / FLAP_ROWS as f32;
            let ring = columns
                .iter()
                .map(|i| {
                    let [s, c] = around(*i);
                    surface.vertex([s, c, t, kind])
                })
                .collect::<Vec<_>>();
            surface.connect(&previous, &ring, false);
            previous = ring;
        }
    }
    surface
}

/// The neck columns the breast flap hangs from, across the front.
fn front_columns() -> Vec<usize> {
    (AROUND - FLAP_EDGE..AROUND).chain(0..=FLAP_EDGE).collect()
}

fn back_columns() -> Vec<usize> {
    (AROUND / 2 - FLAP_EDGE..=AROUND / 2 + FLAP_EDGE).collect()
}

/// Record a mail coif into a new device part.
///
/// The coif is evaluated in its own frame and placed by `frame`, the head
/// frame at the start of that buffer. `fit` is the coif's own frame --
/// identity axes at the origin, with the head frame's half extents -- in
/// [`FIT_PROFILE_WORD`](super::FIT_PROFILE_WORD) floats, then the wearer's
/// [`COIF_DRAPE_WORDS`](super::COIF_DRAPE_WORDS) drape floats.
pub fn record_coif(
    gpu: &ArmorGpu,
    batch: &mut KernelBatch,
    design: &CoifDesign,
    fit: &Buffer,
    frame: &Buffer,
) -> Result<DevicePart, GenerateError> {
    HelmetDesign::MailCoif(*design).validate()?;
    let surface = carrier();
    let mut recipe = PartRecipe::new();
    recipe.push_coord(
        CoordShell {
            coords: surface.coords,
            indices: surface.indices,
            boundary_normals: BoundaryNormals::Smooth,
            thickness: design.fit.wall_thickness.metres(),
            extrusion: CoordExtrusion::Normal,
            // Each flap's width follows its first attachment column.
            values: [
                around(front_columns()[0])[0],
                around(back_columns()[0])[0],
                0.0,
                0.0,
            ],
            mirrored: false,
            frame: 0,
            passes: 1,
            hinge: None,
        },
        CoordKernel::new(gpu, &format!("{DOME}{SHAPE}"))?,
    )?;
    let words = [
        design.fit.clearance.metres() + design.fit.wall_thickness.metres(),
        design.fit.crown_height.unit(),
        design.front_flap_length.metres(),
        design.back_flap_length.metres(),
        design.flap_width.unit(),
    ];
    let mut part = recipe.record(gpu, batch, &words, &[fit])?;
    part.place_by(frame);
    Ok(part)
}

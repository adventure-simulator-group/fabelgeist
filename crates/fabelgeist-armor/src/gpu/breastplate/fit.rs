//! Recording the wearer's measurement and the plates' section fit.

use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::Buffer;

use super::TorsoBody;
use super::carrier_wgsl::{self, COARSE_WORDS};
use super::fit_wgsl::{self, CENTER_WORDS};
use super::kernels::{Params, dispatch};
use super::plates::{Plate, Plates};
use super::shape_wgsl;
use super::topology::{SKIRT_SAMPLES, V_SAMPLES};
use super::wearer_wgsl;
use crate::gpu::ArmorGpu;
use crate::{BreastplateDesign, GenerateError};

/// The ordered encodings of positive and negative infinity: a bound's
/// starting value before an atomic reduction.
const ORDERED_POSITIVE_INFINITY: u32 = 0xff80_0000;
const ORDERED_NEGATIVE_INFINITY: u32 = 0x007f_ffff;
/// Seating may contract an oversized carrier. Enclosure after construction
/// preserves existing radial reach, including the rear side lap.
#[derive(Clone, Copy, Default)]
#[repr(u32)]
pub(super) enum RadialFit {
    #[default]
    Seat,
    Enclose,
}

/// The wearer as the plates were fitted to it: the design and wearer words
/// every shape kernel reads, and the body in the wearer's frame.
pub(super) struct Fitted {
    pub plate: Buffer,
    pub body_local: Buffer,
}

/// Measure the wearer, evaluate both carriers, fit them to the torso and
/// lap the rear plate onto the front, up to the front's refinement.
pub(super) fn record_fitted(
    gpu: &ArmorGpu,
    batch: &mut KernelBatch,
    design: &BreastplateDesign,
    torso: &TorsoBody,
    plates: &Plates,
    status: &Buffer,
) -> Result<Fitted, GenerateError> {
    let plate = gpu.upload(&shape_wgsl::design_words(design))?;
    let torso_faces = gpu.upload(torso.torso_faces)?;
    let body_local = gpu.scratch(torso.body.vertex_count as u64 * 12, "body in wearer frame")?;
    record_wearer(gpu, batch, &plate, torso, &torso_faces, &body_local, status)?;
    let fitter = Fitter {
        gpu,
        plate: &plate,
        torso_faces: &torso_faces,
        torso_count: torso.torso_faces.len() as u32,
        body_local: &body_local,
        status,
    };
    let (coarse, back) = (&plates.coarse, &plates.back);
    let coarse_centers = fitter.record_carrier(batch, coarse, false)?;
    let back_centers = fitter.record_carrier(batch, back, true)?;
    fitter.record_fit(batch, coarse, &coarse_centers, false, RadialFit::Seat)?;
    fitter.record_fit(batch, back, &back_centers, true, RadialFit::Seat)?;
    fitter.record_lap(batch, back, coarse)?;
    // Restore local enclosure after seating the lap.
    fitter.record_fit(batch, back, &back_centers, true, RadialFit::Enclose)?;
    for (plate, centers, rear) in [
        (coarse, &coarse_centers, false),
        (back, &back_centers, true),
    ] {
        dispatch(
            gpu,
            batch,
            fit_wgsl::TRIM_CARRIER,
            &[("positions", true)],
            Params {
                width: plate.width(),
                ..Params::default()
            },
            &[("positions", &plate.positions)],
            plate.width(),
        )?;
        let trim = if rear { &plates.back } else { &plates.front };
        trim.record_arm_trim(gpu, batch, fitter.plate, rear)?;
        fitter.record_trim_validation(batch, plate, centers, rear, &trim.arm_distances)?;
    }
    Ok(Fitted { plate, body_local })
}

/// Record the wearer's frame, anchors and scales into `plate`, and every
/// body vertex in that frame into `body_local`.
fn record_wearer(
    gpu: &ArmorGpu,
    batch: &mut KernelBatch,
    plate: &Buffer,
    torso: &TorsoBody,
    torso_faces: &Buffer,
    body_local: &Buffer,
    status: &Buffer,
) -> Result<(), GenerateError> {
    dispatch(
        gpu,
        batch,
        wearer_wgsl::WEARER,
        &[("positions", false)],
        Params::default(),
        &[
            ("plate", plate),
            ("rig", torso.rig),
            ("surface", &torso.surface.words),
            ("semantic", torso.semantic),
            ("positions", &torso.body.positions),
            ("status", status),
        ],
        1,
    )?;
    let vertices = torso.body.vertex_count;
    dispatch(
        gpu,
        batch,
        wearer_wgsl::BODY_LOCAL,
        &[("positions", false), ("body_local", true)],
        Params::counted(vertices),
        &[
            ("plate", plate),
            ("positions", &torso.body.positions),
            ("body_local", body_local),
        ],
        vertices,
    )?;
    let corners = torso.torso_faces.len() as u32 * 3;
    let bounds = gpu.upload(&[
        ORDERED_POSITIVE_INFINITY,
        ORDERED_POSITIVE_INFINITY,
        ORDERED_POSITIVE_INFINITY,
        ORDERED_NEGATIVE_INFINITY,
        ORDERED_NEGATIVE_INFINITY,
        ORDERED_NEGATIVE_INFINITY,
    ])?;
    dispatch(
        gpu,
        batch,
        wearer_wgsl::TORSO_BOUNDS,
        &[("body_local", false)],
        Params::counted(corners),
        &[
            ("plate", plate),
            ("torso_faces", torso_faces),
            ("body_local", body_local),
            ("bounds", &bounds),
        ],
        corners,
    )?;
    dispatch(
        gpu,
        batch,
        wearer_wgsl::SCALES,
        &[],
        Params::default(),
        &[("plate", plate), ("bounds", &bounds)],
        1,
    )
}

/// What every fitting pass reads.
struct Fitter<'a> {
    gpu: &'a ArmorGpu,
    plate: &'a Buffer,
    torso_faces: &'a Buffer,
    torso_count: u32,
    body_local: &'a Buffer,
    status: &'a Buffer,
}

impl Fitter<'_> {
    fn record_trim_validation(
        &self,
        batch: &mut KernelBatch,
        plate: &Plate,
        centers: &Buffer,
        rear: bool,
        arm_distances: &Buffer,
    ) -> Result<(), GenerateError> {
        let params = Params {
            count: plate.width(),
            width: plate.width(),
            rear,
            torso_count: self.torso_count,
            extra: (V_SAMPLES as u32 - 1) * plate.width(),
            ..Params::default()
        };
        let support = self
            .gpu
            .scratch(plate.count() as u64 * 4, "trim body support")?;
        dispatch(
            self.gpu,
            batch,
            fit_wgsl::MEASURE,
            &[("positions", false), ("body_local", false)],
            params,
            &[
                ("plate", self.plate),
                ("positions", &plate.positions),
                ("centers", centers),
                ("torso_faces", self.torso_faces),
                ("body_local", self.body_local),
                ("support_radii", &support),
                ("columns", &self.gpu.upload(&plate.topology.columns)?),
                ("status", self.status),
            ],
            plate.width(),
        )?;
        dispatch(
            self.gpu,
            batch,
            fit_wgsl::CHECK_SUPPORT,
            &[("positions", false)],
            params,
            &[
                ("plate", self.plate),
                ("positions", &plate.positions),
                ("support_radii", &support),
                ("status", self.status),
                ("arm_distances", arm_distances),
            ],
            plate.width(),
        )
    }

    /// Evaluate an unfluted plate's carrier, and the torso section centre at
    /// each of its vertices' heights, which fitting never changes.
    fn record_carrier(
        &self,
        batch: &mut KernelBatch,
        plate: &Plate,
        rear: bool,
    ) -> Result<Buffer, GenerateError> {
        let gpu = self.gpu;
        let count = plate.count();
        let params = Params {
            count,
            width: plate.width(),
            rear,
            torso_count: self.torso_count,
            ..Params::default()
        };
        let samples = plate.width() * V_SAMPLES as u32;
        let coarse = gpu.scratch(samples as u64 * COARSE_WORDS as u64 * 4, "regular chart")?;
        dispatch(
            gpu,
            batch,
            carrier_wgsl::COARSE_ENTRY,
            &[],
            params,
            &[
                ("plate", self.plate),
                ("coarse", &coarse),
                ("columns", &self.gpu.upload(&plate.topology.columns)?),
                ("status", self.status),
            ],
            samples,
        )?;
        dispatch(
            gpu,
            batch,
            carrier_wgsl::GRID_ENTRY,
            &[("positions", true)],
            params,
            &[
                ("plate", self.plate),
                ("columns", &gpu.upload(&plate.topology.columns)?),
                ("coarse", &coarse),
                ("positions", &plate.positions),
            ],
            count,
        )?;
        let centers = gpu.scratch(
            count as u64 * CENTER_WORDS as u64 * 4,
            "torso section centres",
        )?;
        dispatch(
            gpu,
            batch,
            wearer_wgsl::CENTERS,
            &[("positions", false), ("body_local", false)],
            params,
            &[
                ("plate", self.plate),
                ("positions", &plate.positions),
                ("torso_faces", self.torso_faces),
                ("body_local", self.body_local),
                ("centers", &centers),
            ],
            count,
        )?;
        Ok(centers)
    }

    /// Close the rear plate's width onto the front's edge: each side in
    /// turn, one invocation per row.
    fn record_lap(
        &self,
        batch: &mut KernelBatch,
        back: &Plate,
        front: &Plate,
    ) -> Result<(), GenerateError> {
        for side in 0..2 {
            dispatch(
                self.gpu,
                batch,
                fit_wgsl::LAP,
                &[("positions", true)],
                Params {
                    width: back.width(),
                    front_count: front.width(),
                    side,
                    ..Params::default()
                },
                &[
                    ("plate", self.plate),
                    ("front", &front.positions),
                    ("positions", &back.positions),
                ],
                (V_SAMPLES + SKIRT_SAMPLES - 1) as u32,
            )?;
        }
        Ok(())
    }

    /// Measure each carrier point against its own torso ray, then smooth a
    /// conservative local support envelope. A flank correction cannot push
    /// the whole anterior chest away from its wearer.
    fn record_fit(
        &self,
        batch: &mut KernelBatch,
        plate: &Plate,
        centers: &Buffer,
        rear: bool,
        radial_fit: RadialFit,
    ) -> Result<(), GenerateError> {
        let positions = &plate.positions;
        let count = plate.count();
        let support_radii = self.gpu.scratch(count as u64 * 4, "local torso support")?;
        let envelope = self
            .gpu
            .scratch(count as u64 * 4, "torso support envelope")?;
        let params = Params {
            count,
            rear,
            radial_fit,
            width: plate.width(),
            torso_count: self.torso_count,
            ..Params::default()
        };
        dispatch(
            self.gpu,
            batch,
            fit_wgsl::MEASURE,
            &[("positions", false), ("body_local", false)],
            params,
            &[
                ("plate", self.plate),
                ("positions", positions),
                ("centers", centers),
                ("torso_faces", self.torso_faces),
                ("body_local", self.body_local),
                ("support_radii", &support_radii),
                ("columns", &self.gpu.upload(&plate.topology.columns)?),
                ("status", self.status),
            ],
            count,
        )?;
        dispatch(
            self.gpu,
            batch,
            fit_wgsl::ENVELOPE,
            &[],
            params,
            &[("support_radii", &support_radii), ("envelope", &envelope)],
            count,
        )?;
        dispatch(
            self.gpu,
            batch,
            fit_wgsl::APPLY,
            &[("positions", true)],
            params,
            &[
                ("positions", positions),
                ("plate", self.plate),
                ("centers", centers),
                ("envelope", &envelope),
                ("support_radii", &support_radii),
                ("status", self.status),
            ],
            count,
        )
    }
}

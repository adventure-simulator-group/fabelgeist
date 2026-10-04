//! Recording the wearer's measurement and the plates' section fit.

use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::Buffer;
use fabelgeist_gpu::prelude::BufferUpload;

use super::TorsoBody;
use super::carrier_wgsl::{self, COARSE_WORDS};
use super::fit_wgsl::{self, CENTER_WORDS, PREPARED_WORDS, RESIDUAL_WORDS};
use super::kernels::{Params, dispatch};
use super::plates::{Plate, Plates};
use super::shape_wgsl;
use super::topology::{SKIRT_SAMPLES, U_SAMPLES, V_SAMPLES};
use super::wearer_wgsl;
use crate::gpu::ArmorGpu;
use crate::{BreastplateDesign, GenerateError};

/// The section fit stops after this many measurements.
pub(super) const ITERATIONS: u32 = 25;

/// The ordered encodings of positive and negative infinity: a bound's
/// starting value before an atomic reduction.
const ORDERED_POSITIVE_INFINITY: u32 = 0xff80_0000;
const ORDERED_NEGATIVE_INFINITY: u32 = 0x007f_ffff;
/// A fit's control words before its first measurement: no residual, no
/// witness, not finished.
const CONTROL_START: [u32; 4] = [0, u32::MAX, 0, 0];

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
    let plate = gpu.upload(BufferUpload::from_elements(&shape_wgsl::design_words(
        design,
    )))?;
    let torso_faces = gpu.upload(BufferUpload::from_elements(torso.torso_faces))?;
    let body_local = gpu.scratch(
        (torso.body.vertex_count as u64 * 12).into(),
        ("body in wearer frame").into(),
    )?;
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
    fitter.record_fit(
        batch,
        &coarse.positions,
        &coarse_centers,
        coarse.count(),
        false,
    )?;
    fitter.record_fit(batch, &back.positions, &back_centers, back.count(), true)?;
    fitter.record_lap(batch, back, coarse)?;
    // Restore the enclosure with a smooth profile after seating the lap.
    fitter.record_fit(batch, &back.positions, &back_centers, back.count(), true)?;
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
        (1u32).into(),
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
        (vertices).into(),
    )?;
    let corners = torso.torso_faces.len() as u32 * 3;
    let bounds = gpu.upload(BufferUpload::from_elements(&[
        ORDERED_POSITIVE_INFINITY,
        ORDERED_POSITIVE_INFINITY,
        ORDERED_POSITIVE_INFINITY,
        ORDERED_NEGATIVE_INFINITY,
        ORDERED_NEGATIVE_INFINITY,
        ORDERED_NEGATIVE_INFINITY,
    ]))?;
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
        (corners).into(),
    )?;
    dispatch(
        gpu,
        batch,
        wearer_wgsl::SCALES,
        &[],
        Params::default(),
        &[("plate", plate), ("bounds", &bounds)],
        (1u32).into(),
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
    /// Evaluate an unfluted plate's carrier, and the torso section centre at
    /// each of its vertices' heights, which fitting never changes.
    fn record_carrier(
        &self,
        batch: &mut KernelBatch,
        plate: &Plate,
        rear: bool,
    ) -> Result<Buffer, GenerateError> {
        let gpu = self.gpu;
        // The skirt hangs from the regular chart's own columns.
        if plate.width() != U_SAMPLES as u32 {
            return Err(GenerateError::InvalidSurface);
        }
        let count = plate.count();
        let params = Params {
            count,
            width: plate.width(),
            rear,
            torso_count: self.torso_count,
            ..Params::default()
        };
        let samples = U_SAMPLES as u32 * V_SAMPLES as u32;
        let coarse = gpu.scratch(
            (samples as u64 * COARSE_WORDS as u64 * 4).into(),
            ("regular chart").into(),
        )?;
        dispatch(
            gpu,
            batch,
            carrier_wgsl::COARSE_ENTRY,
            &[],
            params,
            &[
                ("plate", self.plate),
                ("coarse", &coarse),
                ("status", self.status),
            ],
            (samples).into(),
        )?;
        dispatch(
            gpu,
            batch,
            carrier_wgsl::GRID_ENTRY,
            &[("positions", true)],
            params,
            &[
                ("plate", self.plate),
                (
                    "columns",
                    &gpu.upload(BufferUpload::from_elements(&plate.topology.columns))?,
                ),
                ("coarse", &coarse),
                ("positions", &plate.positions),
            ],
            (count).into(),
        )?;
        let centers = gpu.scratch(
            (count as u64 * CENTER_WORDS as u64 * 4).into(),
            ("torso section centres").into(),
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
            (count).into(),
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
                    side,
                    ..Params::default()
                },
                &[
                    ("plate", self.plate),
                    ("front", &front.positions),
                    ("positions", &back.positions),
                ],
                ((V_SAMPLES + SKIRT_SAMPLES - 1) as u32).into(),
            )?;
        }
        Ok(())
    }

    /// Fit a carrier to clear the torso's sections: every iteration
    /// recorded, the ones after convergence doing nothing.
    fn record_fit(
        &self,
        batch: &mut KernelBatch,
        positions: &Buffer,
        centers: &Buffer,
        count: u32,
        rear: bool,
    ) -> Result<(), GenerateError> {
        let gpu = self.gpu;
        let fit = Fit {
            fitter: self,
            positions,
            centers,
            prepared: gpu.scratch(
                (count as u64 * PREPARED_WORDS as u64 * 4).into(),
                ("fit origins").into(),
            )?,
            residuals: gpu.scratch(
                (count as u64 * RESIDUAL_WORDS as u64 * 4).into(),
                ("fit residuals").into(),
            )?,
            profile: gpu.scratch((8u64 * 4).into(), ("fit profile").into())?,
            control: gpu.upload(BufferUpload::from_elements(&CONTROL_START))?,
            params: Params {
                count,
                rear,
                torso_count: self.torso_count,
                ..Params::default()
            },
        };
        dispatch(
            gpu,
            batch,
            fit_wgsl::PREPARE,
            &[("positions", false)],
            fit.params,
            &[
                ("plate", self.plate),
                ("positions", positions),
                ("centers", centers),
                ("prepared", &fit.prepared),
            ],
            (count).into(),
        )?;
        for iteration in 0..ITERATIONS {
            fit.apply(batch)?;
            fit.measure(batch, iteration)?;
        }
        fit.apply(batch)?;
        dispatch(
            gpu,
            batch,
            fit_wgsl::FINISH,
            &[],
            fit.params,
            &[
                ("plate", self.plate),
                ("profile", &fit.profile),
                ("status", self.status),
            ],
            (1u32).into(),
        )
    }
}

/// One plate's section fit in progress.
struct Fit<'a> {
    fitter: &'a Fitter<'a>,
    positions: &'a Buffer,
    centers: &'a Buffer,
    prepared: Buffer,
    residuals: Buffer,
    profile: Buffer,
    control: Buffer,
    params: Params,
}

impl Fit<'_> {
    /// Move every carrier vertex by the current profile.
    fn apply(&self, batch: &mut KernelBatch) -> Result<(), GenerateError> {
        dispatch(
            self.fitter.gpu,
            batch,
            fit_wgsl::APPLY,
            &[("positions", true)],
            self.params,
            &[
                ("plate", self.fitter.plate),
                ("prepared", &self.prepared),
                ("profile", &self.profile),
                ("positions", self.positions),
            ],
            (self.params.count).into(),
        )
    }

    /// Find the vertex furthest inside its clearance, and grow the profile
    /// to clear it; or finish the fit when none is.
    fn measure(&self, batch: &mut KernelBatch, iteration: u32) -> Result<(), GenerateError> {
        let fitter = self.fitter;
        let count = self.params.count;
        dispatch(
            fitter.gpu,
            batch,
            fit_wgsl::RESIDUAL,
            &[("positions", false), ("body_local", false)],
            self.params,
            &[
                ("plate", fitter.plate),
                ("positions", self.positions),
                ("centers", self.centers),
                ("torso_faces", fitter.torso_faces),
                ("body_local", fitter.body_local),
                ("residuals", &self.residuals),
                ("control", &self.control),
            ],
            (count).into(),
        )?;
        dispatch(
            fitter.gpu,
            batch,
            fit_wgsl::WITNESS,
            &[],
            self.params,
            &[("residuals", &self.residuals), ("control", &self.control)],
            (count).into(),
        )?;
        dispatch(
            fitter.gpu,
            batch,
            fit_wgsl::UPDATE,
            &[],
            Params {
                iteration,
                ..self.params
            },
            &[
                ("prepared", &self.prepared),
                ("residuals", &self.residuals),
                ("profile", &self.profile),
                ("control", &self.control),
                ("status", fitter.status),
            ],
            (1u32).into(),
        )
    }
}

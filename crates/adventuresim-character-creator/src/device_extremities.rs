//! Hand and foot armor fitted on the device: the mitten, the sabaton and the
//! leather boot.

use adventuresim_armor_model::{
    BootDesign, DevicePart, GarmentArmorKind, GauntletDesign, LimbArmorDesign,
    record_extremity_armor,
};
use anyhow::{Result, bail};
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::Buffer;

use crate::armor_frames::{FitRegion, Side};
use crate::device_clearance::ClearanceFit;
use crate::device_clearance::PlateFit;
use crate::device_foot_sections::{axial_bounds, record_axial_bounds, record_sections};
use crate::device_footwear_fit::{SectionFit, record_ankle_fairing, record_section_fit};
use crate::device_frames::{DeviceFrame, DeviceWearer};
use crate::device_piece::{DeviceCheck, DeviceRecording};

/// Room a footwear section keeps beyond its gauge.
const PROFILE_CLEARANCE_MARGIN_M: f32 = 0.002;
/// Leg garments a boot shaft encloses.
const BOOT_LAYERS: [GarmentArmorKind; 2] = [
    GarmentArmorKind::MailChausses,
    GarmentArmorKind::PaddedChausses,
];

impl DeviceWearer<'_> {
    /// Record a hand or foot design's shells and fit, short of thickening.
    pub fn record_fitted_extremity(
        &self,
        batch: &mut KernelBatch,
        design: &LimbArmorDesign,
        region: FitRegion,
    ) -> Result<DeviceRecording> {
        match (design, region) {
            (LimbArmorDesign::MittenGauntlet(d), FitRegion::Hand(side)) => {
                self.record_mitten(batch, design, d, side)
            }
            (LimbArmorDesign::Sabaton(d), FitRegion::Foot(side)) => {
                let frame = self.record_foot_frame(batch, side)?;
                let part = record_extremity_armor(self.gpu, batch, design, &[&frame.frame])?;
                let check = self.record_sabaton_fit(batch, d, region, &frame, &part)?;
                Ok(DeviceRecording {
                    part,
                    frames: vec![(frame, region)],
                    checks: vec![check],
                })
            }
            (LimbArmorDesign::LeatherBoot(d), FitRegion::Foot(side)) => {
                self.record_boot(batch, design, d, side)
            }
            _ => bail!("{design:?} does not fit {region:?} on the device"),
        }
    }

    /// The mitten is cleared over the hand, its cuff over a vambrace; the
    /// thumb plate follows its own frame unfitted.
    fn record_mitten(
        &self,
        batch: &mut KernelBatch,
        design: &LimbArmorDesign,
        d: &GauntletDesign,
        side: Side,
    ) -> Result<DeviceRecording> {
        let region = FitRegion::Hand(side);
        let frame = self.record_frame(batch, region)?;
        let thumb = self.record_thumb_frame(batch, side)?;
        let part = record_extremity_armor(self.gpu, batch, design, &[&frame.frame, &thumb.frame])?;
        let thumb_shell = part.shell_count() - 1;
        let fit = ClearanceFit {
            region,
            clearance: d.gauge.clearance.metres(),
            thickness: d.gauge.thickness.metres(),
            style: PlateFit::Mitten {
                cuff_length: d.cuff_length.unit(),
                cuff_clearance: d.cuff_clearance,
            },
            count: part.shell_carriers(thumb_shell).start,
            cuff: part.shell_carriers(0).end,
        };
        self.record_clearance_fit(batch, &frame, &fit, part.carriers(), part.status())?;
        Ok(DeviceRecording {
            part,
            frames: vec![(frame, region), (thumb, region)],
            checks: Vec::new(),
        })
    }

    /// The boot is fitted to the foot and lower leg, then its shaft opened
    /// over both chausses a wearer may put beneath it.
    fn record_boot(
        &self,
        batch: &mut KernelBatch,
        design: &LimbArmorDesign,
        d: &BootDesign,
        side: Side,
    ) -> Result<DeviceRecording> {
        let gpu = self.gpu;
        let region = FitRegion::Foot(side);
        let frame = self.record_frame(batch, region)?;
        let mut part = record_extremity_armor(gpu, batch, design, &[&frame.frame])?;
        let carriers = part.carrier_count();
        let clearance =
            d.gauge.clearance.metres() + d.gauge.thickness.metres() + PROFILE_CLEARANCE_MARGIN_M;
        // Each stage is measured between the thickened boot's extremes.
        let record_bounds = |part: &mut adventuresim_armor_model::DevicePart,
                             batch: &mut KernelBatch|
         -> Result<_> {
            part.record_shells(gpu, batch)?;
            let bounds = axial_bounds(gpu, 1)?;
            record_axial_bounds(
                gpu,
                batch,
                &frame.frame,
                part.positions(),
                part.vertex_count(),
                &bounds,
                0,
            )?;
            Ok(bounds)
        };

        let bounds = record_bounds(&mut part, batch)?;
        let support =
            self.record_any_region_support(batch, &[region, FitRegion::LowerLeg(side)])?;
        let skin = self.record_body_points(batch, &frame.frame, &support)?;
        let envelope = record_sections(
            gpu,
            batch,
            &frame.frame,
            &bounds,
            &skin,
            self.body.vertex_count,
        )?;
        record_section_fit(
            gpu,
            batch,
            &frame.frame,
            &bounds,
            &envelope,
            clearance,
            SectionFit::Envelope,
            part.carriers(),
            carriers,
        )?;

        let bounds = record_bounds(&mut part, batch)?;
        let BootLayers {
            leg,
            garments,
            hems,
        } = self.record_boot_layers(batch, side, &frame.frame)?;
        let slices = self.record_garment_slices(batch, &frame.frame, &bounds, &hems, &garments)?;
        let dressed = record_sections(
            gpu,
            batch,
            &frame.frame,
            &bounds,
            &slices.points,
            slices.count,
        )?;
        record_section_fit(
            gpu,
            batch,
            &frame.frame,
            &bounds,
            &dressed,
            clearance,
            SectionFit::Layer { hems: &hems },
            part.carriers(),
            carriers,
        )?;
        record_ankle_fairing(
            gpu,
            batch,
            &frame.frame,
            &dressed,
            clearance,
            &hems,
            part.carriers(),
            carriers,
        )?;
        Ok(DeviceRecording {
            part,
            frames: vec![(frame, region), (leg, FitRegion::WholeLeg(side))],
            checks: layer_checks(garments),
        })
    }

    /// Record both chausses on the `side` leg, and each one's axial bounds in
    /// the foot `frame`: the garment layers a boot's shaft must clear.
    fn record_boot_layers(
        &self,
        batch: &mut KernelBatch,
        side: Side,
        frame: &Buffer,
    ) -> Result<BootLayers> {
        let gpu = self.gpu;
        let (leg, leg_support) = self.record_leg_cage(batch, side)?;
        let hems = axial_bounds(gpu, BOOT_LAYERS.len())?;
        let mut garments = Vec::with_capacity(BOOT_LAYERS.len());
        for (index, kind) in BOOT_LAYERS.into_iter().enumerate() {
            let garment = self.record_fitted_chausses(batch, kind, side, &leg, &leg_support)?;
            record_axial_bounds(
                gpu,
                batch,
                frame,
                garment.positions(),
                garment.vertex_count(),
                &hems,
                index as u32,
            )?;
            garments.push(garment);
        }
        Ok(BootLayers {
            leg,
            garments,
            hems,
        })
    }
}

/// The garment layers a boot is fitted over: the whole-leg frame they were
/// fitted in, the fitted chausses, and each one's axial bounds.
struct BootLayers {
    leg: DeviceFrame,
    garments: Vec<DevicePart>,
    hems: Buffer,
}

/// Checks that raise a garment layer's failure once the batch has run.
fn layer_checks(garments: Vec<DevicePart>) -> Vec<DeviceCheck> {
    garments
        .into_iter()
        .map(|garment| -> DeviceCheck {
            Box::new(move |gpu| {
                anyhow::ensure!(
                    gpu.read::<u32>(garment.status())?[0] == 0,
                    adventuresim_armor_model::GenerateError::InvalidSurface
                );
                Ok(())
            })
        })
        .collect()
}

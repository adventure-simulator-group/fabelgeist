//! Flexible limb clothing fitted to convex triangle-band sections on the GPU.
//! The fitting frame is the canonical, unposed wearer; section topology never
//! becomes a morph target or an asset downloaded from the server.
use anyhow::Result;
use fabelgeist_armor::{
    PuffAndSlashDesign, PuffAndSlashKind, PuffEdge, gpu::record_puff_and_slash,
};
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::BufferUpload;

use crate::armor_frames::FitRegion;
use crate::armor_layer::ArmorLayerSurface;
use crate::device_frames::DeviceWearer;
use crate::device_garment_kernel::{Grid, Word, atomic, dispatch, read, read_u32, write};
use crate::device_piece::DeviceRecording;
use crate::device_textile_support::TextileSupport;

const SECTION_COUNT: u32 = 25;
/// One further station encloses the hand at a cuff's free edge.
const SECTION_STATIONS: u32 = SECTION_COUNT + 1;
const SECTION_RADII: u32 = 64;
const SECTION_WORDS: u32 = 2 + SECTION_RADII;
const SECTION_HALF_WIDTH_M: f32 = 0.014;
/// Length of a sleeve head, from its seam, that is drawn onto the cloth
/// beneath it: all the way at the seam, not at all at this distance.
const SEAM_TAPER_M: f32 = 0.05;
/// Each triangle contributes its three corners and up to six band crossings.
const SAMPLES_PER_TRIANGLE: u32 = 9;

impl DeviceWearer<'_> {
    pub(crate) fn record_fitted_puff(
        &self,
        batch: &mut KernelBatch,
        design: &PuffAndSlashDesign,
        region: FitRegion,
        layers: &[ArmorLayerSurface<'_>],
    ) -> Result<DeviceRecording> {
        let frame = self.record_frame(batch, region)?;
        // A cuff must clear the hand that its free edge surrounds.
        let hand = match (&design.cuff, region) {
            (Some(_), FitRegion::WholeArm(side)) => Some(FitRegion::Hand(side)),
            _ => None,
        };
        let regions = [Some(region), hand];
        let regions = regions.iter().flatten().copied().collect::<Vec<_>>();
        let support = TextileSupport::new(self.host, &regions, layers)?;
        let faces = &support.faces;
        anyhow::ensure!(
            !faces.is_empty(),
            "puff-and-slash garment has no limb support"
        );
        let gpu = self.gpu;
        let part = record_puff_and_slash(gpu, batch, design, &frame.frame)?;
        let faces_buffer = gpu.upload(BufferUpload::from_elements(faces))?;
        let positions = gpu.upload(BufferUpload::from_elements(&support.positions))?;
        let capacity = faces.len() as u32 * SAMPLES_PER_TRIANGLE + 1;
        let samples = gpu.scratch(
            u64::from(capacity) * u64::from(SECTION_STATIONS) * 8,
            "limb section samples",
        )?;
        let hulls = gpu.scratch(
            u64::from(capacity) * u64::from(SECTION_STATIONS) * 8,
            "limb section hulls",
        )?;
        let sections = gpu.scratch(
            u64::from(SECTION_WORDS * SECTION_STATIONS) * 4,
            "limb sections",
        )?;
        let source = format!(
            "const SECTIONS: u32 = {SECTION_COUNT}u;\nconst RADII: u32 = {SECTION_RADII}u;\nconst SECTION_WORDS: u32 = {SECTION_WORDS}u;\nconst HALF_WIDTH: f32 = {SECTION_HALF_WIDTH_M};\n{}\n{}",
            include_str!("device_section_hull.wgsl"),
            include_str!("device_puff.wgsl")
        );
        let words = fit_parameters(design, capacity, &support, part.carrier_count());
        let buffers = [
            read("fit", &frame.frame),
            read("positions", &positions),
            read_u32("faces", &faces_buffer),
            write("samples", &samples),
            write("hulls", &hulls),
            write("sections", &sections),
            write("carriers", part.carriers()),
            atomic("status", part.status()),
        ];
        for (entry, grid) in [
            (
                include_str!("device_puff_sections.wgsl"),
                Grid::Singles(SECTION_STATIONS),
            ),
            (
                include_str!("device_puff_fit.wgsl"),
                Grid::Items(part.carrier_count()),
            ),
        ] {
            dispatch(
                self,
                batch,
                &format!("{source}{entry}"),
                &buffers,
                &words,
                grid,
            )?;
        }
        Ok(DeviceRecording {
            part,
            frames: vec![(frame, region)],
            checks: Vec::new(),
        })
    }
}

fn fit_parameters(
    design: &PuffAndSlashDesign,
    capacity: u32,
    support: &TextileSupport,
    carriers: u32,
) -> [Word; 11] {
    let sleeve = design.kind == PuffAndSlashKind::Sleeve;
    [
        Word::U("capacity", capacity),
        Word::U("face_count", support.faces.len() as u32),
        Word::U("body_face_count", support.body_faces),
        Word::U("count", carriers),
        Word::F(
            "inset",
            if sleeve {
                design.proximal_inset.unit()
            } else {
                0.0
            },
        ),
        Word::F(
            "cuff",
            design.cuff.as_ref().map_or(0., |cuff| cuff.length.metres()),
        ),
        Word::F("length", design.length.unit()),
        Word::F("proximal", design.proximal_position.unit()),
        Word::F(
            "underarm",
            if sleeve {
                design.underarm_drop.unit()
            } else {
                0.0
            },
        ),
        Word::F("head", design.head_start()),
        Word::F(
            "seam",
            if sleeve && design.pattern.proximal_edge == PuffEdge::Seam {
                SEAM_TAPER_M
            } else {
                0.0
            },
        ),
    ]
}

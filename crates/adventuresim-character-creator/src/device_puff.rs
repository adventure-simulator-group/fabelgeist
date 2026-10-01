//! Flexible limb clothing fitted to convex triangle-band sections on the GPU.
//! The fitting frame is the canonical, unposed wearer; section topology never
//! becomes a morph target or an asset downloaded from the server.
use anyhow::Result;
use fabelgeist_armor::{PuffAndSlashDesign, PuffAndSlashKind, gpu::record_puff_and_slash};
use fabelgeist_compute::KernelBatch;

use crate::armor_frames::{FitRegion, Wearer};
use crate::armor_layer::ArmorLayerSurface;
use crate::device_frames::DeviceWearer;
use crate::device_garment_kernel::{Grid, Word, atomic, dispatch, read, read_u32, write};
use crate::device_piece::DeviceRecording;

const SECTION_COUNT: u32 = 25;
const SECTION_RADII: u32 = 64;
const SECTION_WORDS: u32 = 2 + SECTION_RADII;
const SECTION_HALF_WIDTH_M: f32 = 0.014;
const SLEEVE_PROXIMAL_INSET_RADII: f32 = 1.5;
const FULLNESS_TAPER_COURSES: f32 = 0.8;
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
        let support = PuffSupport::new(self.host, region, layers)?;
        let faces = &support.faces;
        anyhow::ensure!(
            !faces.is_empty(),
            "puff-and-slash garment has no limb support"
        );
        let gpu = self.gpu;
        let part = record_puff_and_slash(gpu, batch, design, &frame.frame)?;
        let faces_buffer = gpu.upload(&faces)?;
        let positions = gpu.upload(&support.positions)?;
        let capacity = faces.len() as u32 * SAMPLES_PER_TRIANGLE + 1;
        let samples = gpu.scratch(
            u64::from(capacity) * u64::from(SECTION_COUNT) * 8,
            "limb section samples",
        )?;
        let hulls = gpu.scratch(
            u64::from(capacity) * u64::from(SECTION_COUNT) * 8,
            "limb section hulls",
        )?;
        let sections = gpu.scratch(
            u64::from(SECTION_WORDS * SECTION_COUNT) * 4,
            "limb sections",
        )?;
        let source = format!(
            "const SECTIONS: u32 = {SECTION_COUNT}u;\nconst RADII: u32 = {SECTION_RADII}u;\nconst SECTION_WORDS: u32 = {SECTION_WORDS}u;\nconst HALF_WIDTH: f32 = {SECTION_HALF_WIDTH_M};\nconst TAPER_COURSES: f32 = {FULLNESS_TAPER_COURSES};\n{}",
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
                Grid::Singles(SECTION_COUNT),
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
    support: &PuffSupport,
    carriers: u32,
) -> [Word; 10] {
    let sleeve = design.kind == PuffAndSlashKind::Sleeve;
    [
        Word::U("capacity", capacity),
        Word::U("face_count", support.faces.len() as u32),
        Word::U("body_face_count", support.body_faces),
        Word::U("count", carriers),
        Word::U(
            "taper",
            u32::from(sleeve && design.proximal_position.0 == 1000),
        ),
        Word::F(
            "inset",
            if sleeve {
                SLEEVE_PROXIMAL_INSET_RADII
            } else {
                0.0
            },
        ),
        Word::F("length", design.length.unit()),
        Word::F("proximal", design.proximal_position.unit()),
        Word::F("puffs", f32::from(design.puff_count)),
        Word::F(
            "allowance",
            design.clearance.metres() + 2.0 * design.thickness.metres(),
        ),
    ]
}

struct PuffSupport {
    positions: Vec<[f32; 3]>,
    faces: Vec<[u32; 3]>,
    body_faces: u32,
}

impl PuffSupport {
    fn new(
        wearer: &Wearer<'_>,
        region: FitRegion,
        layers: &[ArmorLayerSurface<'_>],
    ) -> Result<Self> {
        let mut owned = vec![false; wearer.positions.len()];
        for index in wearer.support_indices(region)? {
            owned[index] = true;
        }
        let mut faces = wearer
            .faces
            .iter()
            .copied()
            .filter(|face| face.iter().any(|&i| owned[i as usize]))
            .collect::<Vec<_>>();
        let body_faces = faces.len() as u32;
        let mut positions = wearer.positions.to_vec();
        for layer in layers {
            let mut owned = vec![false; layer.positions.len()];
            for index in
                wearer.support_indices_for(region, layer.joint_indices, layer.joint_weights)?
            {
                owned[index] = true;
            }
            for face in layer
                .faces
                .iter()
                .filter(|face| face.iter().all(|&i| owned[i as usize]))
            {
                let first = positions.len() as u32;
                positions.extend(face.map(|i| layer.positions[i as usize]));
                faces.push([first, first + 1, first + 2]);
            }
        }
        Ok(Self {
            positions,
            faces,
            body_faces,
        })
    }
}

//! The device armor is generated on.
//!
//! [`ArmorGpu`] shares compiled kernels through its cache. Resource mapping
//! and mutation follow each operation's ownership contract.

use crate::GenerateError;
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::Buffer;

pub use body_cloth::{
    BODY_CLOTH_FIRST_POINT, BODY_CLOTH_POINT_WORDS, BodyClothLayer, record_body_cloth,
};
pub use close_helmet::{CLOSE_HELMET_PROFILE_WORDS, FIT_PROFILE_WORD, record_close_helmet};
pub use close_helmet_profile::CloseHelmetProfile;
pub use coif::record_coif;
pub use coif_drape::{
    COIF_DRAPE_SECTIONS, COIF_DRAPE_WORDS, CoifDrapeProfile, CoifDrapeSection, CoifFlapDrape,
    CoifNeckDrape,
};
pub use extremities::record_extremity_armor;
pub use facade::{ArmorGpu, device_error};
pub use garment::{record_fauld, record_garment_tube, record_tassets};
pub use garment_torso::record_garment_torso;
pub use gorget::record_gorget_plates;
pub use helmet::{generate_helmet_on, record_helmet};
pub use limb_build::{generate_limb_armor_on, record_limb_armor};
pub use part::{BuiltPart, PartSlots};
pub use puff_and_slash::record_puff_and_slash;
pub use recipe::frame_words;
pub use skirt::{SKIRT_FIT_WORDS, SKIRT_PROFILE_FIRST_WORD, SKIRT_SECTION_WORDS, record_skirt};
pub use skirt_attachment::{
    AttachedClothLayer, ClothRingVertex, SKIRT_ATTACHMENT_POINT_WORDS, SkirtAttachment,
    SkirtSeamLayout,
};
pub use skirt_opening::{
    OpeningChartPoint, OpeningEdgeSource, SkirtDressingOpening, SkirtOpeningError,
    SkirtOpeningSource,
};
pub use skirt_sampling::SkirtHangingRows;
pub use staging::{Staged, StagedResults, Staging};

pub mod anatomy;
pub mod body;
mod body_cloth;
pub mod bracer;
mod bracer_contour_wgsl;
mod bracer_wgsl;
pub mod breastplate;
pub(crate) mod chart;
pub(crate) mod close_helmet;
pub(crate) mod close_helmet_dome;
pub(crate) mod close_helmet_profile;
pub(crate) mod close_helmet_shape;
pub(crate) mod coif;
pub(crate) mod coif_drape;
pub(crate) mod coif_shape;
pub(crate) mod coord;
pub(crate) mod coord_topology;
pub(crate) mod extremities;
mod facade;
pub(crate) mod footwear;
pub(crate) mod garment;
pub(crate) mod garment_torso;
pub(crate) mod gorget;
pub(crate) mod helmet;
pub(crate) mod limb;
mod limb_build;
pub mod metal;
pub(crate) mod mitten;
pub(crate) mod part;
pub mod pauldron;
pub(crate) mod placement;
mod puff_and_slash;
pub(crate) mod recipe;
pub(crate) mod shell_plan;
mod skirt;
mod skirt_attachment;
mod skirt_attachment_layers;
mod skirt_opening;
mod skirt_opening_cut;
mod skirt_opening_path;
mod skirt_opening_split;
mod skirt_sampling;
mod skirt_structure;
pub mod staging;
mod textile_chart;
mod textile_pattern;
pub mod wgsl;
pub mod wrapped_tassets;

/// A part under construction on the device: its carriers, until
/// [`DevicePart::record_shells`] thickens them, then its final mesh.
pub struct DevicePart(part::PartBuild);

impl DevicePart {
    /// Carrier points of every shell, packed `f32` triples in shell order.
    pub fn carriers(&self) -> &Buffer {
        &self.0.carriers
    }

    /// Nonzero once any stage has found the part invalid.
    pub fn status(&self) -> &Buffer {
        &self.0.status
    }

    /// Final mesh indices of the outer material point shared by each wall copy.
    pub fn outer_aliases(&self) -> Vec<usize> {
        self.0.layout().outer_aliases()
    }

    pub fn shell_count(&self) -> usize {
        self.0.layout().shell_count()
    }

    /// The carriers of one shell, as indices into [`DevicePart::carriers`].
    pub fn shell_carriers(&self, shell: usize) -> std::ops::Range<u32> {
        let layout = self.0.layout();
        let first = layout.first_carrier(shell);
        first..first + layout.carrier_count(shell)
    }

    /// The carrier triangles of one shell as authored, before any
    /// reflection rewinds them, indexed within the shell.
    pub fn shell_carrier_triangles(&self, shell: usize) -> &[u32] {
        self.0.layout().carrier_triangles(shell)
    }

    pub fn carrier_count(&self) -> u32 {
        self.0.layout().total_carriers()
    }

    /// Place the part by the frame at the start of `frame` once its shells
    /// are thickened: its shapes then evaluate it in its own frame.
    pub fn place_by(&mut self, frame: &Buffer) {
        self.0.placement = Some(frame.clone());
    }

    /// Thicken every shell and compute the final normals.
    pub fn record_shells(
        &mut self,
        gpu: &ArmorGpu,
        batch: &mut KernelBatch,
    ) -> Result<(), GenerateError> {
        self.0.record_shells(gpu, batch)
    }

    /// Final positions, once the shells are recorded.
    pub fn positions(&self) -> &Buffer {
        self.0.positions()
    }

    pub fn vertex_count(&self) -> u32 {
        self.0.final_count()
    }

    /// Final triangles, once the shells are recorded.
    pub fn indices(&self) -> &Buffer {
        self.0.final_indices()
    }

    pub fn triangle_count(&self) -> u32 {
        self.0.final_triangle_count()
    }

    /// Read the finished part back, after its batch has been submitted.
    pub fn read(&self, gpu: &ArmorGpu) -> Result<BuiltPart, GenerateError> {
        let mut staging = Staging::new();
        let slots = self.stage(&mut staging);
        self.finish(&gpu.read_staged(staging)?, slots)
    }

    /// Stage the finished part for a shared readback.
    pub fn stage<'a>(&'a self, staging: &mut Staging<'a>) -> part::PartSlots {
        self.0.stage(staging)
    }

    /// The finished part, from a shared readback.
    pub fn finish(
        &self,
        results: &StagedResults,
        slots: part::PartSlots,
    ) -> Result<BuiltPart, GenerateError> {
        self.0.finish(results, slots)
    }
}

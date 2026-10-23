//! Hair carried from the bust onto a character, and draped on its body, on
//! the GPU.
//!
//! The bust is only stretched to the character's head
//! ([`HeadPlacement`]), so wherever the character's skull is fuller its
//! strands would lie under the skin. Each strand is carried, then moved
//! whole along its root's normal -- the bust scalp's, carried, and so
//! smooth -- until its root sits just outside the body: pushing each root
//! to the nearest point of the body instead would move neighbouring roots
//! apart along the normals of different facets wherever the body is coarse,
//! and part the hair along its edges, at the nape most. Where the normal
//! runs too nearly along the skin to reach it, the root is pushed straight
//! out instead.
//!
//! A hairstyle is draped on the mannequin bust; carried onto a character
//! whose shoulders stand higher or wider, long hair would hang through
//! them. What a body does to a strand is how its equilibrium under gravity
//! against the body differs from its equilibrium against nothing. That is
//! found for guides spread over the head, on the bust as groomed and on the
//! character as carried, and the character's minus the bust's is added to
//! each strand: where the two bodies hold the hair alike, or a bun or a
//! quiff touches neither, it is nothing and the style keeps its shape;
//! where the character's shoulders stand in the way, it is how the hair
//! falls over them. A body acts on a tight coil -- a bun's, a braid's --
//! unsteadily, so the drape takes hold only below the head, fading in from
//! the ears down to the jaw. And the solve knows nothing of what ties a
//! lock: a ponytail's tail, resting against a neck a little thicker than
//! the bust's, would slide off it strand by strand, round both sides. So a
//! strand is draped only as far as it sinks deep into the character's body
//! as carried -- long hair through a shoulder fully, a tail resting against
//! the neck not at all, the shrinkwrap after pushing it out. Every strand
//! follows the one guide whose root is nearest its own: over a shoulder the
//! guides either side of a strand may fall in front and behind, and a blend
//! of them would run through the shoulder, where following one parts the
//! hair cleanly, as hair parts over a shoulder. Last, every strand is
//! shrinkwrapped around the body, keeping its length.

use std::sync::Arc;

use anyhow::Result;
use fabelgeist_compute::{BufferCopy, Kernel, KernelBatch};
use fabelgeist_gpu::prelude::{Buffer, BufferByteOffset};
use fabelgeist_hair::Head;
use fabelgeist_hair::gpu::wgsl::common::{HEAD, MATH};
use fabelgeist_hair::gpu::{GpuGroom, GpuGroomTarget, head_shape};
use fabelgeist_hair::gpu::error::HairGpuError;
use fabelgeist_hair::parameters::StrandCount;
use fabelgeist_hair::ops::{Op, STANDARD_GRAVITY, Settle, Shrinkwrap};
use fabelgeist_math::Vec3;

use super::{BodyField, HairGpu, KernelSource, params, storage, upload};
use wgsl::{CARRY, CARRY_BODY, FOLDED, FOLLOW, GATHER, MOVES, framed};

mod wgsl;
use crate::hair::HeadPlacement;

/// How far off the character's skin a strand's root sits, metres: as far as
/// the hair crate keeps a card's root off its bust.
const ROOT_CLEARANCE: f32 = 0.0005;
/// Steps taken to set a root on the skin along its normal, how close it
/// must come, metres, and how far it may move, metres, before it is pushed
/// straight out instead.
const ROOT_STEPS: u32 = 12;
const ROOT_TOLERANCE: f32 = 0.0001;
const MAX_ROOT_SHIFT: f32 = 0.03;
/// The flattest a root's normal may meet the skin and still be followed:
/// the cosine of the angle between them.
const MIN_ROOT_INCIDENCE: f32 = 0.3;
/// Strands draped as guides for the rest.
const GUIDES: u32 = 2048;
/// How deep inside the character's body, metres, a strand starts to be
/// draped, and how deep it is draped fully. A tail resting a couple of
/// centimetres into a thicker neck is left to the shrinkwrap, which pushes
/// it straight out; long hair through a shoulder runs several centimetres
/// deep.
const SINK_START: f32 = 0.03;
const SINK_FULL: f32 = 0.05;
/// How far below the cranium's centre the drape starts to take hold, and
/// where it takes hold fully, in the cranium's vertical radii.
const DRAPE_START: f32 = 0.5;
const DRAPE_FULL: f32 = 1.0;

/// The placing kernels.
pub(super) struct PlaceKernels {
    carry: Arc<Kernel>,
    gather: Arc<Kernel>,
    moves: Arc<Kernel>,
    follow: Arc<Kernel>,
}

impl PlaceKernels {
    pub(super) fn new(source: &KernelSource) -> Result<Self> {
        Ok(Self {
            carry: source.compile(&format!(
                "{}{MATH}{HEAD}{FOLDED}{CARRY_BODY}",
                framed(CARRY)
            ))?,
            gather: source.compile(&format!("{GATHER}{FOLDED}"))?,
            moves: source.compile(&format!("{}{FOLDED}", framed(MOVES)))?,
            follow: source.compile(&format!("{FOLLOW}{MATH}{HEAD}{FOLDED}"))?,
        })
    }
}

/// Where hair grown on a bust is draped: the bust, how it is placed on the
/// character, and whether the hair hangs free there.
pub(crate) struct Draping<'a> {
    pub(crate) bust: &'a Head,
    pub(crate) placement: &'a HeadPlacement,
    /// Whether the hair hangs free, as the hair crate drapes it on its
    /// bust: tied hair is held at its tie and styled hair by its product,
    /// which the drape knows nothing of, so they are only shrinkwrapped.
    pub(crate) drapes: bool,
}

impl Draping<'_> {
    /// `groomed` carried onto the character, its roots lifted onto the
    /// skin, draped on the body `field` samples if it hangs free, and
    /// shrinkwrapped around it: a groom of its own on the GPU.
    pub(crate) fn place(
        &self,
        gpu: &HairGpu,
        groomed: &GpuGroom,
        field: &BodyField,
    ) -> Result<GpuGroom> {
        let carried = self.carry(gpu, groomed, field)?;
        if self.drapes && groomed.layout().native_strands() != 0 {
            self.drape(gpu, groomed, &carried, field)?;
        }
        let mut placed = carried.into_groom(self.root_bounds(groomed.root_bounds()));
        let shrinkwrap = Op::Shrinkwrap(Shrinkwrap {
            preserve_length: true,
            ..Shrinkwrap::default()
        });
        gpu.groomer().apply(&mut placed, &[shrinkwrap], &field.buffer)?;
        Ok(placed)
    }

    /// `groomed` carried onto the character, each strand moved whole so its
    /// root sits just outside the body.
    fn carry(&self, gpu: &HairGpu, groomed: &GpuGroom, field: &BodyField) -> Result<GpuGroomTarget> {
        let context = gpu.context();
        if groomed.device() != context.device.as_ref() {
            return Err(HairGpuError::DeviceMismatch.into());
        }
        let layout = groomed.layout();
        let (count, points) = (layout.native_strands(), layout.native_points());
        let carried = gpu.groomer().allocate(layout)?;
        let mut carry = self.frame().params();
        carry.insert("src", groomed.positions().clone());
        carry.insert("dst", carried.positions().clone());
        carry.insert("strands_in", groomed.attributes().clone());
        carry.insert("strands_out", carried.attributes().clone());
        carry.insert("roots_in", groomed.roots().clone());
        carry.insert("roots_out", carried.roots().clone());
        carry.insert("head_shape", field.buffer.clone());
        carry.insert("count", count);
        carry.insert("points", points);
        carry.insert("clearance", ROOT_CLEARANCE);
        carry.insert("tolerance", ROOT_TOLERANCE);
        carry.insert("max_shift", MAX_ROOT_SHIFT);
        carry.insert("min_incidence", MIN_ROOT_INCIDENCE);
        carry.insert("steps", ROOT_STEPS);
        carry.insert("pad0", 0u32);
        carry.insert("pad1", 0u32);
        let mut batch = KernelBatch::labelled(context, "hair carry");
        batch.dispatch_folded(&gpu.place.carry, &carry, layout.strand_invocations())?;
        batch.submit()?;
        Ok(carried)
    }

    /// `carried`, the strands of `groomed` as carried, moved as the
    /// character's body moves the guides more than the bust does.
    fn drape(
        &self,
        gpu: &HairGpu,
        groomed: &GpuGroom,
        carried: &GpuGroomTarget,
        field: &BodyField,
    ) -> Result<()> {
        let context = gpu.context();
        let kernels = &gpu.place;
        let layout = groomed.layout();
        let (count, points) = (layout.native_strands(), layout.native_points());
        // Guides spread evenly over the head: the groom's strands are in no
        // order of place, so every few makes an even sample.
        let step = (count / GUIDES).max(1);
        let guides = count.div_ceil(step);
        // The native gather chooses a prefix-sized population at this stride.
        let guide_layout = layout.prefix(StrandCount::from_count(guides as usize))?;
        let guide_bytes = u64::from(guide_layout.positions_bytes());
        let placed_guides = storage(context, guide_bytes, "hair guides as placed")?;
        let groomed_guides = storage(context, guide_bytes, "hair guides as groomed")?;
        let mut batch = KernelBatch::labelled(context, "hair guides");
        batch.dispatch_folded(
            &kernels.gather,
            &params! {
                "placed" => carried.positions().clone(),
                "groomed" => groomed.positions().clone(),
                "placed_guides" => placed_guides.clone(),
                "groomed_guides" => groomed_guides.clone(),
                "guides" => guides,
                "points" => points,
                "step" => step,
                "pad0" => 0u32,
            },
            guide_layout.sample_invocations(),
        )?;
        batch.submit()?;

        // Each guide's equilibrium against its body and against nothing.
        let settle = Settle::default();
        let load = Vec3::new(0.0, -STANDARD_GRAVITY, 0.0);
        let nowhere = BodyField::nowhere(context)?;
        let bust = upload(context, &head_shape(self.bust), "hair bust shape")?;
        let [by_body, placed_free, by_bust, groomed_free] = {
            let mut groomer = gpu.groomer();
            let mut solve = |guides_at: &Buffer, body| {
                groomer.equilibrium(guides_at, guides, points, &settle, load, body)
            };
            [
                solve(&placed_guides, &field.buffer)?,
                solve(&placed_guides, &nowhere.buffer)?,
                solve(&groomed_guides, &bust)?,
                solve(&groomed_guides, &nowhere.buffer)?,
            ]
        };

        let moves = storage(context, guide_bytes, "hair guide moves")?;
        let draped = storage(
            context,
            u64::from(layout.positions_bytes()),
            "hair draped",
        )?;
        let mut batch = KernelBatch::labelled(context, "hair drape");
        let mut guide_moves = self.frame().params();
        guide_moves.insert("by_body", by_body);
        guide_moves.insert("placed_free", placed_free);
        guide_moves.insert("by_bust", by_bust);
        guide_moves.insert("groomed_free", groomed_free);
        guide_moves.insert("moves", moves.clone());
        guide_moves.insert("count", guide_layout.native_samples());
        guide_moves.insert("pad0", 0u32);
        guide_moves.insert("pad1", 0u32);
        guide_moves.insert("pad2", 0u32);
        guide_moves.insert("pad3", 0u32);
        batch.dispatch_folded(&kernels.moves, &guide_moves, guide_layout.sample_invocations())?;
        let [hold_start, hold_full] = [DRAPE_START, DRAPE_FULL].map(|radii| {
            let cranium = &self.bust.cranium;
            let below = cranium.center - Vec3::new(0.0, cranium.radii.y * radii, 0.0);
            self.placement.point(below.to_array())[1]
        });
        batch.dispatch_folded(
            &kernels.follow,
            &params! {
                "src" => carried.positions().clone(),
                "dst" => draped.clone(),
                "moves" => moves,
                "guides_at" => placed_guides,
                "head_shape" => field.buffer.clone(),
                "count" => count,
                "points" => points,
                "guides" => guides,
                "pad0" => 0u32,
                "sink_start" => SINK_START,
                "sink_full" => SINK_FULL,
                "hold_start" => hold_start,
                "hold_full" => hold_full,
            },
            layout.strand_invocations(),
        )?;
        batch.copy_buffer(
            &draped,
            carried.positions(),
            BufferCopy {
                source_offset: BufferByteOffset::START,
                destination_offset: BufferByteOffset::START,
                bytes: layout.positions_bytes(),
            },
        )?;
        batch.submit()?;
        Ok(())
    }

    /// The box the carried roots are in, from the box `bust` the groomed
    /// ones are in, and as far again as a root may be lifted.
    fn root_bounds(&self, [lo, hi]: [Vec3; 2]) -> [Vec3; 2] {
        let corners = (0..8).map(|corner| {
            let pick =
                |bit: u32, low: f32, high: f32| if corner >> bit & 1 == 0 { low } else { high };
            Vec3::from_array(self.placement.point([
                pick(0, lo.x, hi.x),
                pick(1, lo.y, hi.y),
                pick(2, lo.z, hi.z),
            ]))
        });
        let margin = Vec3::splat(MAX_ROOT_SHIFT);
        let (low, high) = corners.fold(
            (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
            |(low, high), corner| (low.min(corner), high.max(corner)),
        );
        [low - margin, high + margin]
    }

    /// The placement as the kernels take it.
    fn frame(&self) -> Frame {
        let origin = self.placement.point([0.0; 3]);
        let axes = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]].map(|unit| {
            let carried = self.placement.point(unit);
            std::array::from_fn(|k| carried[k] - origin[k])
        });
        Frame { origin, axes }
    }
}

/// The placement's origin and axes, the bust's +X, +Y and +Z scaled.
struct Frame {
    origin: [f32; 3],
    axes: [[f32; 3]; 3],
}

impl Frame {
    fn params(&self) -> super::Params {
        let [x, y, z] = self.axes;
        // A normal carries by the inverse transpose: each axis over its
        // squared length.
        let inverse = self
            .axes
            .map(|axis| 1.0 / axis.iter().map(|c| c * c).sum::<f32>().max(1e-12));
        params! {
            "origin_x" => self.origin[0],
            "origin_y" => self.origin[1],
            "origin_z" => self.origin[2],
            "x_x" => x[0],
            "x_y" => x[1],
            "x_z" => x[2],
            "y_x" => y[0],
            "y_y" => y[1],
            "y_z" => y[2],
            "z_x" => z[0],
            "z_y" => z[1],
            "z_z" => z[2],
            "inverse_x" => inverse[0],
            "inverse_y" => inverse[1],
            "inverse_z" => inverse[2],
        }
    }
}

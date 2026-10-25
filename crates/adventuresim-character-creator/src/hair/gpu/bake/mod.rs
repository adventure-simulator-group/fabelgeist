//! The scalp's textures: the hair lying on it, baked from the groom on the
//! GPU.
//!
//! Painted one flat colour, the scalp shows between cards and strands as a
//! smooth, dead patch. Over a real scalp the eye meets the hair nearest it --
//! strands lying along the skin in the groom's own direction, parted where
//! the groom parts and spiralling out of its whorl. Each strand's first few
//! millimetres off the skin are drawn into the scalp's texture
//! ([`ScalpMap`]) on the bust the hair grew on, one selected sample per
//! texel showing: lighter the further it lies off the skin, darker where it
//! meets it, and running the way it runs, so the scalp is shaded as the
//! strands are. Root-based noise supplies lock tones, with a smaller
//! variation among individual strands.
//!
//! Where no strand lies, the scalp is either in the shadow of hair standing
//! further off it -- between an afro's coils, below the coils further out --
//! or bare: at a parting, at a whorl's eye, along a thinning hairline. How
//! bare the scalp is round each texel, over a couple of millimetres and
//! weighted to the nearest, makes the texel clear in the texture's alpha, so
//! the skin beneath the cap shows there, lit as the rest of the body: hair
//! thins into a parting over those millimetres, as it does at a real one,
//! its last strands lying over the skin, while a texel or two between coils
//! shows no skin at all. Shadowed scalp is the hair's colour in shade.
//!
//! Every strand draws itself, an invocation each. An atomic maximum selects
//! each texel's largest packed sample: clamped normalized height quantized to
//! twelve bits first, then direction and shade to break ties. Each texel finds
//! whether it is bare, the bare share is blurred, the textures are written,
//! and each smaller mip level is averaged from the one before.

use super::{HairGpu, KernelSource, params, storage};
use crate::hair::AtlasLevel;
use crate::hair::scalp_map::ScalpMap;
use anyhow::Result;
use fabelgeist_compute::{Kernel, KernelBatch};
use fabelgeist_gpu::prelude::{Buffer, BufferByteLength, BufferDefinition, InvocationCount};
use fabelgeist_hair::Head;
use fabelgeist_hair::gpu::GpuGroom;
use fabelgeist_hair::gpu::wgsl::common::{HEAD, MATH};
use fabelgeist_math::units::{Metres, Ratio};
use grid::Grid;
pub use layout::{ScalpMipLevel, ScalpTextureLayout};
pub(in crate::hair) use layout::ScalpTexelSpan;
use std::sync::Arc;
use wgsl::{BARE, BLUR, DRAW, FINISH, HALVE, LONGEST, PACKING};

mod grid;
mod layout;
mod readback;
mod wgsl;

/// The longest step along a strand mapped as straight, metres: the map
/// curves, and a strand drawn straight between points a centimetre apart
/// would cut its corners.
const STEP: Metres = Metres::from_metres(0.002);
/// Near-skin threshold for drawing sampled segments, in metres.
/// Crossing segments are drawn whole; height is clamped for shading.
const REACH: Metres = Metres::from_metres(0.008);
/// Base shade shares at the skin and at [`REACH`], before lock and strand
/// tone variation.
const AT_SKIN: Ratio = Ratio::from_ratio(0.25);
const AT_REACH: Ratio = Ratio::from_ratio(0.9);
/// How much darker a lock may be than the lightest, and a strand than the
/// rest of its lock.
const LOCK_SPREAD: Ratio = Ratio::from_ratio(0.5);
const STRAND_SPREAD: Ratio = Ratio::from_ratio(0.15);
/// About how wide a lock is, metres: the cell of the noise its tone is
/// drawn from, by where its strands are rooted.
const LOCK_WIDTH: Metres = Metres::from_metres(0.004);
/// How far off the skin hair still shadows it, metres: hair this near
/// over a texel no strand lies on keeps it from showing bare.
const SHADOW_REACH: Metres = Metres::from_metres(0.025);
/// The side of the blocks shadowing is judged over, metres: one strand
/// passing over a block shadows all of it, as hair further off the skin
/// shadows more than the line of its own width.
const SHADOW_BLOCK: Metres = Metres::from_metres(0.0015);
/// How far round each texel the bare scalp is measured, metres, twice over
/// so the nearest counts most: hair thins into a parting over about twice
/// this.
const BARE_REACH: Metres = Metres::from_metres(0.0008);
/// The share of bare scalp round a texel at which skin starts to show
/// there, and shows fully: most of a parting is bare, the gaps within and
/// between an afro's coils single texels among strands.
const BARE_START: Ratio = Ratio::from_ratio(0.1);
const BARE_FULL: Ratio = Ratio::from_ratio(0.75);
/// How much of a strand still shows where the scalp round it is bare: at
/// a parting the hair thins strand by strand, the last strands' ends lying
/// over the skin, rather than fading as a whole.
const STRAND_OVER_SKIN: Ratio = Ratio::from_ratio(0.5);
/// The scalp's shade where no strand lies on it but hair lies over it.
const SHADOWED: Ratio = Ratio::from_ratio(0.35);

/// The hair lying on the scalp, each a mip chain, largest first, over the
/// cap's texture coordinates.
pub struct ScalpTexture {
    /// The scalp's shade as grey, as a share of the hair's colour, and
    /// whether hair covers it as alpha.
    pub shade: Vec<AtlasLevel>,
    /// The selected strand sample's direction across the texture, along `u`
    /// and `v`, each from 0 backwards to 1 forwards (a half each where no
    /// strand lies), and the shade again.
    pub strands: Vec<AtlasLevel>,
}

/// The scalp's textures on the GPU: every mip level, largest first, one
/// after another with four bytes per texel.
pub struct GpuScalpTexture {
    shade: ScalpTextureChain,
    strands: ScalpTextureChain,
}

/// One padded chain, its admitted layout, and its actual creating device.
#[derive(Clone)]
pub struct ScalpTextureChain {
    buffer: Buffer,
    layout: Arc<ScalpTextureLayout>,
    device: wgpu::Device,
}

/// The bake's kernels.
pub(super) struct BakeKernels {
    draw: Arc<Kernel>,
    bare: Arc<Kernel>,
    blur: Arc<Kernel>,
    finish: Arc<Kernel>,
    halve: Arc<Kernel>,
    longest: Arc<Kernel>,
}

impl BakeKernels {
    pub(super) fn new(source: &KernelSource) -> Result<Self> {
        Ok(Self {
            draw: source.compile(&format!("{DRAW}{MATH}{HEAD}{PACKING}"))?,
            bare: source.compile(BARE)?,
            blur: source.compile(BLUR)?,
            finish: source.compile(&format!("{FINISH}{PACKING}"))?,
            halve: source.compile(HALVE)?,
            longest: source.compile(LONGEST)?,
        })
    }
}

impl GpuScalpTexture {
    /// `groom`, grown on `bust`, baked into textures mapped by `map`.
    pub(in crate::hair) fn bake(
        gpu: &HairGpu,
        groom: &GpuGroom,
        bust: &Head,
        map: &ScalpMap,
    ) -> Result<Self> {
        let context = gpu.context();
        let layout = Arc::new(ScalpTextureLayout::new()?);
        let grid = Grid::new(&layout, map);
        let allocate = |bytes: BufferByteLength, label: &str| {
            Buffer::new(
                context,
                bytes.max(BufferByteLength::from(size_of::<[u32; 4]>())),
                BufferDefinition::storage().with_label(label.into()),
            )
        };
        let texels = layout.element_bytes(layout.base().elements());
        let topmost = allocate(texels, "hair scalp topmost")?;
        let shadowed = allocate(
            layout.element_bytes(grid.shadow_elements()),
            "hair scalp shadowed",
        )?;
        let bare = [
            allocate(texels, "hair scalp bare")?,
            allocate(texels, "hair scalp bare, blurred")?,
        ];
        let chain = |label: &str| -> Result<ScalpTextureChain> {
            Ok(ScalpTextureChain {
                buffer: allocate(layout.bytes(), label)?,
                layout: layout.clone(),
                device: context.device.as_ref().clone(),
            })
        };
        let baked = Self {
            shade: chain("hair scalp shade")?,
            strands: chain("hair scalp strands")?,
        };
        let mut batch = KernelBatch::labelled(context, "hair scalp bake");
        batch.encoder().clear_buffer(&topmost.buffer, 0, None);
        batch.encoder().clear_buffer(&shadowed.buffer, 0, None);
        grid.draw(gpu, &mut batch, groom, bust, map, [&topmost, &shadowed])?;
        grid.bare(gpu, &mut batch, [&topmost, &shadowed], &bare)?;
        baked.finish(gpu, &mut batch, &topmost, &bare[0])?;
        batch.submit()?;
        Ok(baked)
    }

    /// The shade texture's chain: shade as grey, coverage as alpha.
    pub fn shade(&self) -> &ScalpTextureChain {
        &self.shade
    }

    /// The strands texture's chain: the way each runs, and its shade.
    pub fn strands(&self) -> &ScalpTextureChain {
        &self.strands
    }

    /// The textures back on the host, on the device that created both chains.
    pub fn read(&self, gpu: &HairGpu) -> Result<ScalpTexture> {
        Ok(ScalpTexture {
            shade: self.shade.read(gpu)?,
            strands: self.strands.read(gpu)?,
        })
    }

    /// Record the textures written from `topmost` and the bare `share`, and
    /// every smaller mip level.
    fn finish(
        &self,
        gpu: &HairGpu,
        batch: &mut KernelBatch<'_>,
        topmost: &Buffer,
        share: &Buffer,
    ) -> Result<()> {
        let kernels = &gpu.bake;
        let layout = self.shade.layout();
        let texel_count = layout.base().elements();
        batch.dispatch_folded(
            &kernels.finish,
            &params! {
                "topmost" => topmost.clone(),
                "share" => share.clone(),
                "shade" => self.shade.buffer().clone(),
                "strands" => self.strands.buffer().clone(),
                "size" => layout.base().side(),
                "shadowed_shade" => SHADOWED.ratio(),
                "bare_start" => BARE_START.ratio(),
                "bare_full" => BARE_FULL.ratio(),
                "strand_over_skin" => STRAND_OVER_SKIN.ratio(),
                "pad0" => 0u32,
                "pad1" => 0u32,
                "pad2" => 0u32,
            },
            InvocationCount::from(texel_count),
        )?;
        for pair in layout.levels().windows(2) {
            let [from, to] = [pair[0], pair[1]];
            for texture in [&self.shade, &self.strands] {
                batch.dispatch_folded(
                    &kernels.halve,
                    &params! {
                        "chain" => texture.buffer().clone(),
                        "from_side" => from.side(),
                        "from_stride" => from.row_elements(),
                        "from_offset" => layout.native_word_offset(&from),
                        "to_side" => to.side(),
                        "to_stride" => to.row_elements(),
                        "to_offset" => layout.native_word_offset(&to),
                        "pad0" => 0u32,
                        "pad1" => 0u32,
                    },
                    InvocationCount::from(to.elements()),
                )?;
            }
        }
        Ok(())
    }
}

impl ScalpTextureChain {
    /// The native allocation; external aliases follow the buffer's access contract.
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    /// The exact padded layout admitted with this allocation.
    pub fn layout(&self) -> &ScalpTextureLayout {
        &self.layout
    }
}

/// The longest of `groom`'s strands, metres.
pub(crate) fn longest(gpu: &HairGpu, groom: &GpuGroom) -> Result<Metres> {
    let context = gpu.context();
    let out = storage(context, 16, "hair longest strand")?;
    let mut batch = KernelBatch::labelled(context, "hair longest strand");
    batch.encoder().clear_buffer(&out.buffer, 0, None);
    batch.dispatch_folded(
        &gpu.bake.longest,
        &params! {
            "positions" => groom.positions().clone(),
            "longest" => out.clone(),
            "count" => groom.layout().native_strands(),
            "points" => groom.layout().native_points(),
            "pad0" => 0u32,
            "pad1" => 0u32,
        },
        groom.layout().strand_invocations(),
    )?;
    batch.submit()?;
    let words: Vec<u32> = pollster::block_on(out.read(context))?;
    Ok(Metres::from_metres(f32::from_bits(words[0])))
}

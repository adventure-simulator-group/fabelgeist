//! The scalp's textures: the hair lying on it, baked from the groom on the
//! GPU.
//!
//! Painted one flat colour, the scalp shows between cards and strands as a
//! smooth, dead patch. Over a real scalp the eye meets the hair nearest it --
//! strands lying along the skin in the groom's own direction, parted where
//! the groom parts and spiralling out of its whorl. Each strand's first few
//! millimetres off the skin are drawn into the scalp's texture
//! ([`ScalpMap`]) on the bust the hair grew on, the topmost strand over each
//! texel showing: lighter the further it lies off the skin, darker where it
//! meets it, and running the way it runs, so the scalp is shaded as the
//! strands are. Single strands are far finer than a pixel at any distance
//! the head is seen from, so what shows is locks: strands rooted together
//! share a tone, lighter or darker than the next lock, and differ a little
//! among themselves.
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
//! Every strand draws itself, an invocation each, keeping the topmost strand
//! over each texel with an atomic maximum: its height off the skin first in
//! the packed value, then its direction and shade. Then each texel finds
//! whether it is bare, the bare share is blurred, the textures are written,
//! and each smaller mip level is averaged from the one before.

use std::sync::Arc;

use anyhow::Result;
use fabelgeist_compute::{Kernel, KernelBatch};
use fabelgeist_gpu::prelude::Buffer;
use fabelgeist_hair::Head;
use fabelgeist_hair::gpu::wgsl::common::{HEAD, MATH};
use fabelgeist_hair::gpu::{GpuGroom, head_shape};

use super::{HairGpu, KernelSource, params, storage, upload};
use wgsl::{BARE, BLUR, DRAW, FINISH, HALVE, LONGEST, PACKING};

mod wgsl;
use crate::hair::AtlasLevel;
use crate::hair::scalp_map::ScalpMap;

/// The texture's side, texels: about an eighth of a millimetre a texel,
/// near a hair's own width, so lying strands read as lines and a parting's
/// edge as strands' ends.
const SIZE: u32 = 2048;
/// The longest step along a strand mapped as straight, metres: the map
/// curves, and a strand drawn straight between points a centimetre apart
/// would cut its corners.
const STEP: f32 = 0.002;
/// How far off the skin a strand is drawn, metres: the layer of hair the
/// eye meets between cards.
const REACH: f32 = 0.008;
/// A strand's shade where it meets the skin, and at [`REACH`] off it, as a
/// share of the hair's colour.
const AT_SKIN: f32 = 0.25;
const AT_REACH: f32 = 0.9;
/// How much darker a lock may be than the lightest, and a strand than the
/// rest of its lock.
const LOCK_SPREAD: f32 = 0.5;
const STRAND_SPREAD: f32 = 0.15;
/// About how wide a lock is, metres: the cell of the noise its tone is
/// drawn from, by where its strands are rooted.
const LOCK_WIDTH: f32 = 0.004;
/// How far off the skin hair still shadows it, metres: hair this near
/// over a texel no strand lies on keeps it from showing bare.
const SHADOW_REACH: f32 = 0.025;
/// The side of the blocks shadowing is judged over, metres: one strand
/// passing over a block shadows all of it, as hair further off the skin
/// shadows more than the line of its own width.
const SHADOW_BLOCK: f32 = 0.0015;
/// How far round each texel the bare scalp is measured, metres, twice over
/// so the nearest counts most: hair thins into a parting over about twice
/// this.
const BARE_REACH: f32 = 0.0008;
/// The share of bare scalp round a texel at which skin starts to show
/// there, and shows fully: most of a parting is bare, the gaps within and
/// between an afro's coils single texels among strands.
const BARE_START: f32 = 0.1;
const BARE_FULL: f32 = 0.75;
/// How much of a strand still shows where the scalp round it is bare: at
/// a parting the hair thins strand by strand, the last strands' ends lying
/// over the skin, rather than fading as a whole.
const STRAND_OVER_SKIN: f32 = 0.5;
/// The scalp's shade where no strand lies on it but hair lies over it.
const SHADOWED: f32 = 0.35;
/// Bytes of a texel.
const TEXEL_BYTES: u64 = 4;

/// The hair lying on the scalp, each a mip chain, largest first, over the
/// cap's texture coordinates.
pub struct ScalpTexture {
    /// The scalp's shade as grey, as a share of the hair's colour, and
    /// whether hair covers it as alpha.
    pub shade: Vec<AtlasLevel>,
    /// The direction the topmost strand runs across the texture, along `u`
    /// and `v`, each from 0 backwards to 1 forwards (a half each where no
    /// strand lies), and the shade again.
    pub strands: Vec<AtlasLevel>,
}

/// The scalp's textures on the GPU: every mip level, largest first, one
/// after another ([`levels`]), four bytes a texel.
pub struct GpuScalpTexture {
    shade: Buffer,
    strands: Buffer,
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

/// One mip level's place in the chain: its side, texels; how far apart its
/// rows start, texels, padded so a row is a whole number of the 256 bytes a
/// copy into a texture moves rows in; and where it starts, texels in.
#[derive(Clone, Copy, Debug)]
pub struct Level {
    pub side: u32,
    pub stride: u32,
    pub offset: u32,
}

/// Texels in a row a copy into a texture moves rows in multiples of.
const ROW_TEXELS: u32 = 64;

/// Every mip level of the texture, largest first.
pub fn levels() -> Vec<Level> {
    let mut levels = Vec::new();
    let (mut side, mut offset) = (SIZE, 0);
    loop {
        let stride = side.max(ROW_TEXELS);
        levels.push(Level {
            side,
            stride,
            offset,
        });
        if side == 1 {
            return levels;
        }
        offset += stride * side;
        side /= 2;
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
        let texel = map.texel(SIZE as usize);
        let block_side = ((SHADOW_BLOCK / texel).round() as u32).max(1);
        let blocks = SIZE.div_ceil(block_side);
        let bare_reach = ((BARE_REACH / texel).round() as u32).max(1);
        let texels = u64::from(SIZE) * u64::from(SIZE);
        let chain: u64 = levels()
            .iter()
            .map(|level| u64::from(level.stride) * u64::from(level.side))
            .sum();
        let topmost = storage(context, texels * TEXEL_BYTES, "hair scalp topmost")?;
        let shadowed = storage(
            context,
            u64::from(blocks * blocks) * TEXEL_BYTES,
            "hair scalp shadowed",
        )?;
        let bare = [
            storage(context, texels * TEXEL_BYTES, "hair scalp bare")?,
            storage(context, texels * TEXEL_BYTES, "hair scalp bare, blurred")?,
        ];
        let baked = Self {
            shade: storage(context, chain * TEXEL_BYTES, "hair scalp shade")?,
            strands: storage(context, chain * TEXEL_BYTES, "hair scalp strands")?,
        };
        let mut batch = KernelBatch::labelled(context, "hair scalp bake");
        batch.encoder().clear_buffer(&topmost.buffer, 0, None);
        batch.encoder().clear_buffer(&shadowed.buffer, 0, None);
        let grid = Grid {
            block_side,
            blocks,
            bare_reach,
        };
        grid.draw(gpu, &mut batch, groom, bust, map, [&topmost, &shadowed])?;
        grid.bare(gpu, &mut batch, [&topmost, &shadowed], &bare)?;
        baked.finish(gpu, &mut batch, &topmost, &bare[0])?;
        batch.submit()?;
        Ok(baked)
    }

    /// The shade texture's chain: shade as grey, coverage as alpha.
    pub fn shade(&self) -> &Buffer {
        &self.shade
    }

    /// The strands texture's chain: the way each runs, and its shade.
    pub fn strands(&self) -> &Buffer {
        &self.strands
    }

    /// The textures back on the host.
    pub fn read(&self, gpu: &HairGpu) -> Result<ScalpTexture> {
        let context = gpu.context();
        let chain = |buffer: &Buffer| -> Result<Vec<AtlasLevel>> {
            let texels: Vec<u32> = pollster::block_on(buffer.read(context))?;
            let bytes: &[u8] = bytemuck::cast_slice(&texels);
            Ok(levels()
                .into_iter()
                .map(|level| {
                    let texel = TEXEL_BYTES as usize;
                    let rgba = (0..level.side as usize)
                        .flat_map(|row| {
                            let start =
                                (level.offset as usize + row * level.stride as usize) * texel;
                            bytes[start..start + level.side as usize * texel]
                                .iter()
                                .copied()
                        })
                        .collect();
                    AtlasLevel {
                        width: level.side,
                        height: level.side,
                        rgba,
                    }
                })
                .collect())
        };
        Ok(ScalpTexture {
            shade: chain(&self.shade)?,
            strands: chain(&self.strands)?,
        })
    }
}

/// How the bake's grid is judged: the side of the blocks shadowing is
/// judged over, how many a row, and how far bare scalp is measured, texels.
struct Grid {
    block_side: u32,
    blocks: u32,
    bare_reach: u32,
}

impl Grid {
    /// Record every strand of `groom`, grown on `bust`, drawn into
    /// `topmost`, or marking the blocks it shadows.
    fn draw(
        &self,
        gpu: &HairGpu,
        batch: &mut KernelBatch<'_>,
        groom: &GpuGroom,
        bust: &Head,
        map: &ScalpMap,
        [topmost, shadowed]: [&Buffer; 2],
    ) -> Result<()> {
        let context = gpu.context();
        let kernels = &gpu.bake;
        let [center, axis, forward, side] = [map.center, map.axis, map.forward, map.side];
        let (block_side, blocks) = (self.block_side, self.blocks);
        batch.dispatch_folded(
            &kernels.draw,
            &params! {
                "positions" => groom.positions().clone(),
                "strands" => groom.attributes().clone(),
                "head_shape" => upload(context, &head_shape(bust), "hair bust shape")?,
                "topmost" => topmost.clone(),
                "shadowed" => shadowed.clone(),
                "center_x" => center.x,
                "center_y" => center.y,
                "center_z" => center.z,
                "rim" => map.rim,
                "axis_x" => axis.x,
                "axis_y" => axis.y,
                "axis_z" => axis.z,
                "size" => SIZE,
                "forward_x" => forward.x,
                "forward_y" => forward.y,
                "forward_z" => forward.z,
                "count" => groom.layout().native_strands(),
                "side_x" => side.x,
                "side_y" => side.y,
                "side_z" => side.z,
                "points" => groom.layout().native_points(),
                "step" => STEP,
                "reach" => REACH,
                "shadow_reach" => SHADOW_REACH,
                "at_skin" => AT_SKIN,
                "at_reach" => AT_REACH,
                "lock_spread" => LOCK_SPREAD,
                "strand_spread" => STRAND_SPREAD,
                "lock_width" => LOCK_WIDTH,
                "block_side" => block_side,
                "blocks" => blocks,
                "pad0" => 0u32,
                "pad1" => 0u32,
            },
            groom.layout().strand_invocations(),
        )?;
        Ok(())
    }

    /// Record which texels are bare, into `bare[0]`, and their share round
    /// each texel, blurred twice along rows and columns, back into it.
    fn bare(
        &self,
        gpu: &HairGpu,
        batch: &mut KernelBatch<'_>,
        [topmost, shadowed]: [&Buffer; 2],
        bare: &[Buffer; 2],
    ) -> Result<()> {
        let kernels = &gpu.bake;
        let (block_side, blocks, bare_reach) = (self.block_side, self.blocks, self.bare_reach);
        let texel_count = SIZE * SIZE;
        batch.dispatch_folded(
            &kernels.bare,
            &params! {
                "topmost" => topmost.clone(),
                "shadowed" => shadowed.clone(),
                "bare" => bare[0].clone(),
                "size" => SIZE,
                "block_side" => block_side,
                "blocks" => blocks,
                "pad0" => 0u32,
            },
            texel_count,
        )?;
        // Twice over, along rows then columns each time, so the nearest
        // scalp counts most and the share falls smoothly.
        for pass in 0..4u32 {
            let (from, to) = (&bare[(pass % 2) as usize], &bare[((pass + 1) % 2) as usize]);
            batch.dispatch_folded(
                &kernels.blur,
                &params! {
                    "blur_in" => from.clone(),
                    "blur_out" => to.clone(),
                    "size" => SIZE,
                    "stride" => if pass % 2 == 0 { 1 } else { SIZE },
                    "reach" => bare_reach,
                    "pad0" => 0u32,
                },
                texel_count,
            )?;
        }
        Ok(())
    }
}

impl GpuScalpTexture {
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
        let texel_count = SIZE * SIZE;
        batch.dispatch_folded(
            &kernels.finish,
            &params! {
                "topmost" => topmost.clone(),
                "share" => share.clone(),
                "shade" => self.shade.clone(),
                "strands" => self.strands.clone(),
                "size" => SIZE,
                "shadowed_shade" => SHADOWED,
                "bare_start" => BARE_START,
                "bare_full" => BARE_FULL,
                "strand_over_skin" => STRAND_OVER_SKIN,
                "pad0" => 0u32,
                "pad1" => 0u32,
                "pad2" => 0u32,
            },
            texel_count,
        )?;
        for pair in levels().windows(2) {
            let [from, to] = [pair[0], pair[1]];
            for texture in [&self.shade, &self.strands] {
                batch.dispatch_folded(
                    &kernels.halve,
                    &params! {
                        "chain" => texture.clone(),
                        "from_side" => from.side,
                        "from_stride" => from.stride,
                        "from_offset" => from.offset,
                        "to_side" => to.side,
                        "to_stride" => to.stride,
                        "to_offset" => to.offset,
                        "pad0" => 0u32,
                        "pad1" => 0u32,
                    },
                    to.side * to.side,
                )?;
            }
        }
        Ok(())
    }
}

/// The longest of `groom`'s strands, metres.
pub(crate) fn longest(gpu: &HairGpu, groom: &GpuGroom) -> Result<f32> {
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
    Ok(f32::from_bits(words.first().copied().unwrap_or(0)))
}

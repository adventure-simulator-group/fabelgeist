//! Running a metal bake through the device.

use fabelgeist_compute::{KernelBatch, host_float};
use fabelgeist_gpu::prelude::BufferUpload;
use fabelgeist_gpu::prelude::{Buffer, PassParameters, Result, WorkgroupGrid};

use super::MetalGpu;
use super::finish::FINISH_WORDS;
use super::textures::{DRAWS_PER_SCRATCH, STAMP_GROUP};
use crate::engraving::{Cut, Engraving, EngravingError, Relief, ReliefImage, ReliefPixels};
use crate::material::{DepthMap, Metal, MetalTextureSize, MetalTextures};
use crate::ornament::Ornament;

pub(super) fn textures(gpu: &MetalGpu, metal: &Metal, size: MetalTextureSize) -> Result<MetalTextures> {
    metal.validate()?;
    let cut = metal
        .engraving
        .as_ref()
        .map(|engraving| engraving.cut().map(|cut| (engraving, cut)))
        .transpose()?;
    bake(gpu, metal, size, cut.as_ref().map(|(e, cut)| (*e, cut)))
}

/// Bake the maps with the engraving's cut already resolved.
pub(super) fn bake(
    gpu: &MetalGpu,
    metal: &Metal,
    size: MetalTextureSize,
    engraving: Option<(&Engraving, &Cut<'_>)>,
) -> Result<MetalTextures> {
    let texels = size.texels();
    let slope_words = size.words(2)?;
    let finish_words = size.words(FINISH_WORDS)?;
    let height = gpu.scratch(texels, "metal scratch height".into())?;
    let recess = gpu.scratch(texels, "metal recess".into())?;
    let slopes = gpu.scratch(slope_words, "metal slopes".into())?;
    let normal = gpu.scratch(texels, "metal normal".into())?;
    let metal_roughness = gpu.scratch(texels, "metal roughness".into())?;
    let depth = gpu.scratch(texels, "metal depth".into())?;
    let finish = gpu.scratch(finish_words, "metal finish".into())?;
    let mut batch = gpu.batch("metal textures");
    record_scratches(gpu, &mut batch, metal, size, &height)?;
    record_finish(gpu, &mut batch, metal, size, &finish)?;
    let depth_uv = match engraving {
        Some((engraving, cut)) => {
            match cut {
                Cut::Image(image) => {
                    record_image(gpu, &mut batch, engraving, image, size, &recess, &slopes)?
                }
                Cut::Ornament(ornament) => {
                    record_ornament(gpu, &mut batch, engraving, ornament, size, &recess)?
                }
            }
            record_cut_slopes(gpu, &mut batch, engraving, size, &recess, &slopes)?
        }
        None => None,
    };
    let mut bake = PassParameters::new();
    bake.insert("size", size);
    bake.insert("seed", metal.seed);
    bake.insert("scratch_draws", metal.scratch_density * DRAWS_PER_SCRATCH);
    bake.insert("engraved", u32::from(engraving.is_some()));
    bake.insert("roughness", metal.roughness);
    bake.insert(
        "recess_roughness",
        engraving.map_or(0.0, |(engraving, _)| engraving.recess_roughness),
    );
    bake.insert(host_float::ZERO_FIELD, 0u32);
    bake.insert("pad0", 0u32);
    bake.insert("height", height);
    bake.insert("slopes", slopes);
    bake.insert("recess", recess);
    bake.insert("normal", normal.clone());
    bake.insert("metal_roughness", metal_roughness.clone());
    bake.insert("depth", depth.clone());
    bake.insert("finish", finish);
    gpu.dispatch(&mut batch, &gpu.bake, &bake, texels.into())?;
    batch.submit()?;
    let bytes = |buffer: &Buffer| -> Result<Vec<u8>> {
        let words: Vec<u32> = gpu.read(buffer)?;
        Ok(bytemuck::cast_slice(&words[..usize::from(texels)]).to_vec())
    };
    Ok(MetalTextures {
        size,
        normal: bytes(&normal)?,
        metal_roughness: bytes(&metal_roughness)?,
        depth: depth_uv
            .map(|uv_scale| {
                Ok::<_, fabelgeist_gpu::prelude::Error>(DepthMap {
                    pixels: bytes(&depth)?,
                    uv_scale,
                })
            })
            .transpose()?,
    })
}

/// Evaluate the planishing, polish grain and smudges into `finish`.
fn record_finish(
    gpu: &MetalGpu,
    batch: &mut KernelBatch,
    metal: &Metal,
    size: MetalTextureSize,
    finish: &Buffer,
) -> Result<()> {
    let mut parameters = PassParameters::new();
    parameters.insert("size", size);
    parameters.insert("seed", metal.seed);
    parameters.insert(host_float::ZERO_FIELD, 0u32);
    parameters.insert("pad0", 0u32);
    parameters.insert("waviness", metal.waviness);
    parameters.insert("grain", metal.grain);
    parameters.insert("smudge", metal.smudge);
    parameters.insert("pad1", 0.0f32);
    parameters.insert("finish", finish.clone());
    gpu.dispatch(batch, &gpu.finish, &parameters, size.texels().into())
}

/// Stamp the metal's scratches into `height`.
fn record_scratches(
    gpu: &MetalGpu,
    batch: &mut KernelBatch,
    metal: &Metal,
    size: MetalTextureSize,
    height: &Buffer,
) -> Result<()> {
    if metal.scratch_density == 0 {
        return Ok(());
    }
    // No scratch takes more steps than its longest possible length allows;
    // the margin covers rounding, and any unused invocation returns early.
    let most_steps = (metal.scratch_length * f32::from(size) * 2.0).ceil() as u32 + 2;
    let mut stamp = PassParameters::new();
    stamp.insert("size", size);
    stamp.insert("seed", metal.seed);
    stamp.insert(host_float::ZERO_FIELD, 0u32);
    stamp.insert("pad1", 0u32);
    stamp.insert("scratch_length", metal.scratch_length);
    stamp.insert("scratch_width", metal.scratch_width);
    stamp.insert("scratch_depth", metal.scratch_depth);
    stamp.insert("scratch_angle", metal.scratch_angle);
    stamp.insert("scratch_spread", metal.scratch_spread);
    for pad in ["pad2", "pad3", "pad4"] {
        stamp.insert(pad, 0.0f32);
    }
    stamp.insert("height", height.clone());
    batch
        .dispatch(
            &gpu.scratches,
            &stamp,
            WorkgroupGrid::from([
                (most_steps + 1).div_ceil(STAMP_GROUP),
                metal.scratch_density,
                1,
            ]),
        )
        .map(|_| ())
        .map_err(super::device_error)
}

/// Draw the ornament into `recess`.
fn record_ornament(
    gpu: &MetalGpu,
    batch: &mut KernelBatch,
    engraving: &Engraving,
    ornament: &Ornament,
    size: MetalTextureSize,
    recess: &Buffer,
) -> Result<()> {
    let (sin, cos) = engraving.rotation.sin_cos();
    let (motif, first, second, strands) = ornament.motif.words();
    let mut draw = PassParameters::new();
    draw.insert("size", size);
    draw.insert("motif", motif);
    draw.insert("repeats", ornament.repeats);
    draw.insert("strands", strands);
    draw.insert("tiles", engraving.tiles);
    draw.insert("sin_rotation", sin);
    draw.insert("cos_rotation", cos);
    draw.insert("line", ornament.line);
    draw.insert("first", first);
    draw.insert("second", second);
    draw.insert("fillets", u32::from(ornament.fillets));
    draw.insert(host_float::ZERO_FIELD, 0u32);
    draw.insert("recess", recess.clone());
    gpu.dispatch(batch, &gpu.ornament, &draw, size.texels().into())
}

/// Resample the engraving's image onto the tile.
fn record_image(
    gpu: &MetalGpu,
    batch: &mut KernelBatch,
    engraving: &Engraving,
    image: &ReliefImage,
    size: MetalTextureSize,
    recess: &Buffer,
    slopes: &Buffer,
) -> Result<()> {
    let texels = size.texels();
    let (sin, cos) = engraving.rotation.sin_cos();
    let (pixels, normal_map, strength) = match (&image.pixels, engraving.relief) {
        (ReliefPixels::Height(heights), Relief::Height { .. }) => {
            (gpu.upload(BufferUpload::from_elements(heights))?, 0u32, 0.0)
        }
        (ReliefPixels::Slopes(slopes), Relief::Normal { strength }) => (
            gpu.upload(BufferUpload::from_elements(slopes))?,
            1,
            strength,
        ),
        _ => return Err(EngravingError::ReliefMismatch.into()),
    };
    let mut sample = PassParameters::new();
    sample.insert("size", size);
    sample.insert("image_width", image.width as u32);
    sample.insert("image_height", image.height as u32);
    sample.insert("normal_map", normal_map);
    sample.insert("tiles", engraving.tiles);
    sample.insert("sin_rotation", sin);
    sample.insert("cos_rotation", cos);
    sample.insert("strength", strength);
    sample.insert(host_float::ZERO_FIELD, 0u32);
    for pad in ["pad0", "pad1", "pad2"] {
        sample.insert(pad, 0u32);
    }
    sample.insert("image", pixels);
    sample.insert("recess", recess.clone());
    sample.insert("slopes", slopes.clone());
    gpu.dispatch(batch, &gpu.engraving, &sample, texels.into())
}

/// The slopes of a height map's cut; returns its parallax depth.
fn record_cut_slopes(
    gpu: &MetalGpu,
    batch: &mut KernelBatch,
    engraving: &Engraving,
    size: MetalTextureSize,
    recess: &Buffer,
    slopes: &Buffer,
) -> Result<Option<f32>> {
    let texels = size.texels();
    let Relief::Height { depth } = engraving.relief else {
        return Ok(None);
    };
    let mut cut = PassParameters::new();
    cut.insert("size", size);
    cut.insert("depth", depth);
    cut.insert(host_float::ZERO_FIELD, 0u32);
    for pad in ["pad1", "pad2"] {
        cut.insert(pad, 0u32);
    }
    for pad in ["pad3", "pad4", "pad5"] {
        cut.insert(pad, 0.0f32);
    }
    cut.insert("recess", recess.clone());
    cut.insert("slopes", slopes.clone());
    gpu.dispatch(batch, &gpu.cut_slopes, &cut, texels.into())?;
    Ok(Some(depth * Metal::TILES_PER_METRE))
}

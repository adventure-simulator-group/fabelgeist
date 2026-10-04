//! Running a metal bake through the device.

use fabelgeist_compute::{KernelBatch, host_float};
use fabelgeist_gpu::prelude::BufferUpload;
use fabelgeist_gpu::prelude::PassParameterName;
use fabelgeist_gpu::prelude::{Buffer, PassParameters};

use super::MetalGpu;
use super::finish::FINISH_WORDS;
use super::textures::{DRAWS_PER_SCRATCH, STAMP_GROUP};
use crate::engraving::{Cut, Engraving, Relief, ReliefImage, ReliefPixels};
use crate::material::{DepthMap, Metal, MetalError, MetalTextures};
use crate::ornament::Ornament;

/// The texture sizes a bake supports.
const SIZES: std::ops::RangeInclusive<u32> = 32..=1024;

pub(super) fn textures(
    gpu: &MetalGpu,
    metal: &Metal,
    size: u32,
) -> Result<MetalTextures, MetalError> {
    metal.validate()?;
    if !SIZES.contains(&size) {
        return Err(MetalError::TextureSize { requested: size });
    }
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
    size: u32,
    engraving: Option<(&Engraving, &Cut<'_>)>,
) -> Result<MetalTextures, MetalError> {
    let texels = size * size;
    let height = gpu.scratch(texels.into(), ("metal scratch height").into())?;
    let recess = gpu.scratch(texels.into(), ("metal recess").into())?;
    let slopes = gpu.scratch(u64::from(texels) * 2, ("metal slopes").into())?;
    let normal = gpu.scratch(texels.into(), ("metal normal").into())?;
    let metal_roughness = gpu.scratch(texels.into(), ("metal roughness").into())?;
    let depth = gpu.scratch(texels.into(), ("metal depth").into())?;
    let finish = gpu.scratch(
        u64::from(texels) * u64::from(FINISH_WORDS),
        ("metal finish").into(),
    )?;
    let mut batch = gpu.batch(("metal textures").into());
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
    bake.insert("size".into(), (size).into());
    bake.insert("seed".into(), (metal.seed).into());
    bake.insert(
        "scratch_draws".into(),
        (metal.scratch_density * DRAWS_PER_SCRATCH).into(),
    );
    bake.insert("engraved".into(), (u32::from(engraving.is_some())).into());
    bake.insert("roughness".into(), (metal.roughness).into());
    bake.insert(
        "recess_roughness".into(),
        (engraving.map_or(0.0, |(engraving, _)| engraving.recess_roughness)).into(),
    );
    bake.insert(
        PassParameterName::from(host_float::ZERO_FIELD),
        (0u32).into(),
    );
    bake.insert("pad0".into(), (0u32).into());
    bake.insert("height".into(), (height).into());
    bake.insert("slopes".into(), (slopes).into());
    bake.insert("recess".into(), (recess).into());
    bake.insert("normal".into(), (normal.clone()).into());
    bake.insert("metal_roughness".into(), (metal_roughness.clone()).into());
    bake.insert("depth".into(), (depth.clone()).into());
    bake.insert("finish".into(), (finish).into());
    gpu.dispatch(&mut batch, &gpu.bake, &bake, (texels).into())?;
    batch.submit();
    let bytes = |buffer: &Buffer| -> Result<Vec<u8>, MetalError> {
        let words: Vec<u32> = gpu.read(buffer)?;
        Ok(bytemuck::cast_slice(&words[..texels as usize]).to_vec())
    };
    Ok(MetalTextures {
        size,
        normal: bytes(&normal)?,
        metal_roughness: bytes(&metal_roughness)?,
        depth: depth_uv
            .map(|uv_scale| {
                Ok::<_, MetalError>(DepthMap {
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
    size: u32,
    finish: &Buffer,
) -> Result<(), MetalError> {
    let mut parameters = PassParameters::new();
    parameters.insert("size".into(), (size).into());
    parameters.insert("seed".into(), (metal.seed).into());
    parameters.insert(
        PassParameterName::from(host_float::ZERO_FIELD),
        (0u32).into(),
    );
    parameters.insert("pad0".into(), (0u32).into());
    parameters.insert("waviness".into(), (metal.waviness).into());
    parameters.insert("grain".into(), (metal.grain).into());
    parameters.insert("smudge".into(), (metal.smudge).into());
    parameters.insert("pad1".into(), (0.0f32).into());
    parameters.insert("finish".into(), (finish.clone()).into());
    gpu.dispatch(batch, &gpu.finish, &parameters, (size * size).into())
}

/// Stamp the metal's scratches into `height`.
fn record_scratches(
    gpu: &MetalGpu,
    batch: &mut KernelBatch,
    metal: &Metal,
    size: u32,
    height: &Buffer,
) -> Result<(), MetalError> {
    if metal.scratch_density == 0 {
        return Ok(());
    }
    // No scratch takes more steps than its longest possible length allows;
    // the margin covers rounding, and any unused invocation returns early.
    let most_steps = (metal.scratch_length * size as f32 * 2.0).ceil() as u32 + 2;
    let mut stamp = PassParameters::new();
    stamp.insert("size".into(), (size).into());
    stamp.insert("seed".into(), (metal.seed).into());
    stamp.insert(
        PassParameterName::from(host_float::ZERO_FIELD),
        (0u32).into(),
    );
    stamp.insert("pad1".into(), (0u32).into());
    stamp.insert("scratch_length".into(), (metal.scratch_length).into());
    stamp.insert("scratch_width".into(), (metal.scratch_width).into());
    stamp.insert("scratch_depth".into(), (metal.scratch_depth).into());
    stamp.insert("scratch_angle".into(), (metal.scratch_angle).into());
    stamp.insert("scratch_spread".into(), (metal.scratch_spread).into());
    for pad in ["pad2", "pad3", "pad4"] {
        stamp.insert(pad.into(), (0.0f32).into());
    }
    stamp.insert("height".into(), (height.clone()).into());
    batch
        .dispatch(
            &gpu.scratches,
            &stamp,
            ([
                (most_steps + 1).div_ceil(STAMP_GROUP),
                metal.scratch_density,
                1,
            ])
            .into(),
        )
        .map(|_| ())
        .map_err(MetalError::Dispatch)
}

/// Draw the ornament into `recess`.
fn record_ornament(
    gpu: &MetalGpu,
    batch: &mut KernelBatch,
    engraving: &Engraving,
    ornament: &Ornament,
    size: u32,
    recess: &Buffer,
) -> Result<(), MetalError> {
    let (sin, cos) = engraving.rotation.sin_cos();
    let (motif, first, second, strands) = ornament.motif.words();
    let mut draw = PassParameters::new();
    draw.insert("size".into(), (size).into());
    draw.insert("motif".into(), (motif).into());
    draw.insert("repeats".into(), (ornament.repeats).into());
    draw.insert("strands".into(), (strands).into());
    draw.insert("tiles".into(), (engraving.tiles).into());
    draw.insert("sin_rotation".into(), (sin).into());
    draw.insert("cos_rotation".into(), (cos).into());
    draw.insert("line".into(), (ornament.line).into());
    draw.insert("first".into(), (first).into());
    draw.insert("second".into(), (second).into());
    draw.insert("fillets".into(), (u32::from(ornament.fillets)).into());
    draw.insert(
        PassParameterName::from(host_float::ZERO_FIELD),
        (0u32).into(),
    );
    draw.insert("recess".into(), (recess.clone()).into());
    gpu.dispatch(batch, &gpu.ornament, &draw, (size * size).into())
}

/// Resample the engraving's image onto the tile.
fn record_image(
    gpu: &MetalGpu,
    batch: &mut KernelBatch,
    engraving: &Engraving,
    image: &ReliefImage,
    size: u32,
    recess: &Buffer,
    slopes: &Buffer,
) -> Result<(), MetalError> {
    let texels = size * size;
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
        _ => return Err(MetalError::ReliefMismatch),
    };
    let mut sample = PassParameters::new();
    sample.insert("size".into(), (size).into());
    sample.insert("image_width".into(), (image.width as u32).into());
    sample.insert("image_height".into(), (image.height as u32).into());
    sample.insert("normal_map".into(), (normal_map).into());
    sample.insert("tiles".into(), (engraving.tiles).into());
    sample.insert("sin_rotation".into(), (sin).into());
    sample.insert("cos_rotation".into(), (cos).into());
    sample.insert("strength".into(), (strength).into());
    sample.insert(
        PassParameterName::from(host_float::ZERO_FIELD),
        (0u32).into(),
    );
    for pad in ["pad0", "pad1", "pad2"] {
        sample.insert(pad.into(), (0u32).into());
    }
    sample.insert("image".into(), (pixels).into());
    sample.insert("recess".into(), (recess.clone()).into());
    sample.insert("slopes".into(), (slopes.clone()).into());
    gpu.dispatch(batch, &gpu.engraving, &sample, (texels).into())
}

/// The slopes of a height map's cut; returns its parallax depth.
fn record_cut_slopes(
    gpu: &MetalGpu,
    batch: &mut KernelBatch,
    engraving: &Engraving,
    size: u32,
    recess: &Buffer,
    slopes: &Buffer,
) -> Result<Option<f32>, MetalError> {
    let texels = size * size;
    let Relief::Height { depth } = engraving.relief else {
        return Ok(None);
    };
    let mut cut = PassParameters::new();
    cut.insert("size".into(), (size).into());
    cut.insert("depth".into(), (depth).into());
    cut.insert(
        PassParameterName::from(host_float::ZERO_FIELD),
        (0u32).into(),
    );
    for pad in ["pad1", "pad2"] {
        cut.insert(pad.into(), (0u32).into());
    }
    for pad in ["pad3", "pad4", "pad5"] {
        cut.insert(pad.into(), (0.0f32).into());
    }
    cut.insert("recess".into(), (recess.clone()).into());
    cut.insert("slopes".into(), (slopes.clone()).into());
    gpu.dispatch(batch, &gpu.cut_slopes, &cut, (texels).into())?;
    Ok(Some(depth * Metal::TILES_PER_METRE))
}

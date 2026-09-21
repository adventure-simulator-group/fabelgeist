//! Running a metal bake through the device.

use fabelgeist_compute::{KernelBatch, host_float};
use fabelgeist_gpu::prelude::{Buffer, PassParameters};

use super::PlateGpu;
use super::textures::{DRAWS_PER_SCRATCH, STAMP_GROUP};
use crate::engraving::{Engraving, Relief, ReliefImage, ReliefPixels};
use crate::material::{DepthMap, Metal, MetalTextures};

/// The texture sizes a bake supports.
const SIZES: std::ops::RangeInclusive<u32> = 32..=1024;

pub(super) fn textures(gpu: &PlateGpu, metal: &Metal, size: u32) -> Result<MetalTextures, String> {
    metal.validate()?;
    if !SIZES.contains(&size) {
        return Err("Texture size must be 32–1024".into());
    }
    let image = metal
        .engraving
        .as_ref()
        .map(|engraving| engraving.load().map(|image| (engraving, image)))
        .transpose()?;
    bake(
        gpu,
        metal,
        size,
        image.as_ref().map(|(engraving, image)| (*engraving, image)),
    )
}

/// Bake the maps with an already decoded engraving image.
pub(super) fn bake(
    gpu: &PlateGpu,
    metal: &Metal,
    size: u32,
    engraving: Option<(&Engraving, &ReliefImage)>,
) -> Result<MetalTextures, String> {
    let texels = size * size;
    let height = gpu.scratch(texels.into(), "metal scratch height")?;
    let recess = gpu.scratch(texels.into(), "metal recess")?;
    let slopes = gpu.scratch(u64::from(texels) * 2, "metal slopes")?;
    let normal = gpu.scratch(texels.into(), "metal normal")?;
    let metal_roughness = gpu.scratch(texels.into(), "metal roughness")?;
    let depth = gpu.scratch(texels.into(), "metal depth")?;
    let mut batch = gpu.batch("metal textures");
    record_scratches(gpu, &mut batch, metal, size, &height)?;
    let depth_uv = match engraving {
        Some((engraving, image)) => {
            record_engraving(gpu, &mut batch, engraving, image, size, &recess, &slopes)?
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
    gpu.dispatch(&mut batch, &gpu.bake, &bake, texels)?;
    batch.submit();
    let bytes = |buffer: &Buffer| -> Result<Vec<u8>, String> {
        let words: Vec<u32> = gpu.read(buffer)?;
        Ok(bytemuck::cast_slice(&words[..texels as usize]).to_vec())
    };
    Ok(MetalTextures {
        size,
        normal: bytes(&normal)?,
        metal_roughness: bytes(&metal_roughness)?,
        depth: depth_uv
            .map(|uv_scale| {
                Ok::<_, String>(DepthMap {
                    pixels: bytes(&depth)?,
                    uv_scale,
                })
            })
            .transpose()?,
    })
}

/// Stamp the metal's scratches into `height`.
fn record_scratches(
    gpu: &PlateGpu,
    batch: &mut KernelBatch,
    metal: &Metal,
    size: u32,
    height: &Buffer,
) -> Result<(), String> {
    if metal.scratch_density == 0 {
        return Ok(());
    }
    // No scratch takes more steps than its longest possible length allows;
    // the margin covers rounding, and any unused invocation returns early.
    let most_steps = (metal.scratch_length * size as f32 * 2.0).ceil() as u32 + 2;
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
            [
                (most_steps + 1).div_ceil(STAMP_GROUP),
                metal.scratch_density,
                1,
            ],
        )
        .map(|_| ())
        .map_err(super::device_error)
}

/// Resample the engraving onto the tile; returns the parallax depth of a
/// height map.
fn record_engraving(
    gpu: &PlateGpu,
    batch: &mut KernelBatch,
    engraving: &Engraving,
    image: &ReliefImage,
    size: u32,
    recess: &Buffer,
    slopes: &Buffer,
) -> Result<Option<f32>, String> {
    let texels = size * size;
    let (sin, cos) = engraving.rotation.sin_cos();
    let (pixels, normal_map, strength) = match (&image.pixels, engraving.relief) {
        (ReliefPixels::Height(heights), Relief::Height { .. }) => (gpu.upload(heights)?, 0u32, 0.0),
        (ReliefPixels::Slopes(slopes), Relief::Normal { strength }) => {
            (gpu.upload(slopes)?, 1, strength)
        }
        _ => return Err("the relief image does not match the engraving's relief".into()),
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
    gpu.dispatch(batch, &gpu.engraving, &sample, texels)?;
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
    gpu.dispatch(batch, &gpu.cut_slopes, &cut, texels)?;
    Ok(Some(depth * Metal::TILES_PER_METRE))
}

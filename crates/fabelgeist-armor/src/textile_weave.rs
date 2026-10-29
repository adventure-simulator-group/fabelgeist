//! Pixel-integrated plain weave retains fine yarn coverage at bounded resolution.
use crate::{Permille, TextileColor, TextileThreadCount};

/// Native raster boundary: normalized garment UV and its pixel footprint.
pub(crate) fn weave_coverage(
    chart: [f32; 2],
    footprint: [f32; 2],
    warp: TextileThreadCount,
    weft: TextileThreadCount,
    width: Permille,
) -> f32 {
    let counts = [warp, weft];
    let yarn: [f32; 2] = std::array::from_fn(|axis| {
        let count = f32::from(counts[axis].0);
        let center = chart[axis] * count;
        let span = footprint[axis] * count;
        let integral = |position: f32| {
            position.floor() * width.unit()
                + (position.rem_euclid(1.) - (1. - width.unit()) * 0.5).clamp(0., width.unit())
        };
        ((integral(center + span * 0.5) - integral(center - span * 0.5)) / span).clamp(0., 1.)
    });
    1. - (1. - yarn[0]) * (1. - yarn[1])
}

/// Straight-alpha sRGB image adapter; fractional coverage composites in linear light.
pub(crate) fn composite_yarn(base: [u8; 4], color: TextileColor, coverage: f32) -> [u8; 4] {
    if coverage <= 0. {
        return base;
    }
    if coverage >= 1. {
        return [color.0[0], color.0[1], color.0[2], 255];
    }
    let retained = f32::from(base[3]) / 255. * (1. - coverage);
    let alpha = coverage + retained;
    let mut pixel = [0; 4];
    for channel in 0..3 {
        let linear =
            (decode(color.0[channel]) * coverage + decode(base[channel]) * retained) / alpha;
        let encoded = if linear <= 0.0031308 {
            linear * 12.92
        } else {
            1.055 * linear.powf(1. / 2.4) - 0.055
        };
        pixel[channel] = (encoded * 255.).round() as u8;
    }
    pixel[3] = (alpha * 255.).round() as u8;
    pixel
}

fn decode(channel: u8) -> f32 {
    let encoded = f32::from(channel) / 255.;
    if encoded <= 0.04045 {
        encoded / 12.92
    } else {
        ((encoded + 0.055) / 1.055).powf(2.4)
    }
}

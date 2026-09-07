//! Intrinsic surface colors and coverage filtering in linear light.

use bevy::{
    color::{LinearRgba, Srgba},
    image::Image,
    render::render_resource::TextureFormat,
};

const CHANNEL_MAX: f32 = u8::MAX as f32;

/// An opaque, display-encoded RGB palette color.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SrgbColor(pub [u8; 3]);

impl SrgbColor {
    fn linear(self) -> LinearRgba {
        Srgba::rgb(
            self.0[0] as f32 / CHANNEL_MAX,
            self.0[1] as f32 / CHANNEL_MAX,
            self.0[2] as f32 / CHANNEL_MAX,
        )
        .into()
    }

    fn from_linear(color: LinearRgba) -> Self {
        let color: Srgba = color.into();
        Self(
            [color.red, color.green, color.blue]
                .map(|channel| (channel * CHANNEL_MAX).round().clamp(0.0, CHANNEL_MAX) as u8),
        )
    }

    /// Blend palette endpoints only according to pixel coverage, not surface shading.
    pub(crate) fn covered_by(self, foreground: Self, coverage: f32) -> Self {
        if coverage <= 0.0 {
            return self;
        }
        if coverage >= 1.0 {
            return foreground;
        }
        let a = self.linear();
        let b = foreground.linear();
        Self::from_linear(LinearRgba::rgb(
            a.red + (b.red - a.red) * coverage,
            a.green + (b.green - a.green) * coverage,
            a.blue + (b.blue - a.blue) * coverage,
        ))
    }
}

/// Colors of masonry units and their independently selected joint material.
/// Recipe signatures fix N to their nonempty authored palette size.
#[derive(Clone, Copy, Debug)]
pub struct MasonryColors<const N: usize> {
    pub units: [SrgbColor; N],
    pub mortar: SrgbColor,
}

/// Filter color boundaries in linear light; other recipes and map types retain
/// their existing filtering. Intermediate mip colors represent area coverage.
pub(crate) fn albedo_image(data: Vec<u8>, size: u32) -> Image {
    let mut image = super::image_rgba_mipped(data, size, true);
    image.texture_descriptor.format = TextureFormat::Rgba8UnormSrgb;
    let bytes = image.data.as_mut().expect("generated image has CPU data");
    let mut parent_start = 0;
    let mut width = size as usize;
    while width > 1 {
        let child_start = parent_start + width * width * 4;
        let child_width = width / 2;
        for y in 0..child_width {
            for x in 0..child_width {
                let mut sum = [0.0; 3];
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    let index = parent_start + ((y * 2 + dy) * width + x * 2 + dx) * 4;
                    let c = SrgbColor([bytes[index], bytes[index + 1], bytes[index + 2]]).linear();
                    sum[0] += c.red;
                    sum[1] += c.green;
                    sum[2] += c.blue;
                }
                let c = SrgbColor::from_linear(LinearRgba::rgb(
                    sum[0] / 4.0,
                    sum[1] / 4.0,
                    sum[2] / 4.0,
                ));
                let index = child_start + (y * child_width + x) * 4;
                bytes[index..index + 3].copy_from_slice(&c.0);
            }
        }
        parent_start = child_start;
        width = child_width;
    }
    image
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn antialiasing_preserves_endpoints_and_averages_light_not_encoded_bytes() {
        let black = SrgbColor([0; 3]);
        let white = SrgbColor([255; 3]);
        assert_eq!(black.covered_by(white, 0.0), black);
        assert_eq!(black.covered_by(white, 1.0), white);
        assert_eq!(black.covered_by(white, 0.5), SrgbColor([188; 3]));
        let image = albedo_image(
            vec![
                0, 0, 0, 255, 255, 255, 255, 255, 0, 0, 0, 255, 255, 255, 255, 255,
            ],
            2,
        );
        assert_eq!(&image.data.unwrap()[16..], &[188, 188, 188, 255]);
    }
}

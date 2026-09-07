//! Explicit palette overrides for material review exports.

use adventuresim_procedural_textures::{
    DRESSED_STONE_COLORS, HANDMADE_BRICK_COLORS, HEWN_OAK_COLORS, HewnOakColors, MasonryColors,
    SrgbColor, SurfaceTextureSet, TextureRecipeId, generate_dressed_stone_textures,
    generate_handmade_brick_textures, generate_hewn_oak_textures,
};
use bevy::{asset::Assets, image::Image};

#[derive(Default, clap::Args)]
pub(super) struct Options {
    /// Light wood region, in #RRGGBB.
    #[arg(long, value_parser = parse_color)]
    wood_light: Option<SrgbColor>,
    /// Dark wood region, in #RRGGBB.
    #[arg(long, value_parser = parse_color)]
    wood_dark: Option<SrgbColor>,
    /// Brick/stone color. Supply one color, or repeat for every palette entry.
    #[arg(long, value_parser = parse_color)]
    unit_color: Vec<SrgbColor>,
    /// Joint material color, independently of brick or stone, in #RRGGBB.
    #[arg(long, value_parser = parse_color)]
    mortar_color: Option<SrgbColor>,
}

fn parse_color(value: &str) -> Result<SrgbColor, String> {
    let hex = value.strip_prefix('#').ok_or("color must be #RRGGBB")?;
    if hex.len() != 6 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("color must contain exactly six hexadecimal digits".to_owned());
    }
    let mut channels = [0; 3];
    for (index, channel) in channels.iter_mut().enumerate() {
        *channel = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16)
            .map_err(|error| error.to_string())?;
    }
    Ok(SrgbColor(channels))
}

impl Options {
    fn masonry<const N: usize>(
        &self,
        mut colors: MasonryColors<N>,
    ) -> Result<MasonryColors<N>, String> {
        if self.wood_light.is_some() || self.wood_dark.is_some() {
            return Err("wood color options require hewn-oak".to_owned());
        }
        match self.unit_color.as_slice() {
            [] => {}
            [single] => colors.units.fill(*single),
            palette if palette.len() == N => colors.units.copy_from_slice(palette),
            _ => {
                return Err(format!(
                    "supply one unit color or exactly {N} palette entries"
                ));
            }
        }
        if let Some(mortar) = self.mortar_color {
            colors.mortar = mortar;
        }
        Ok(colors)
    }

    pub(super) fn generate(
        &self,
        recipe: TextureRecipeId,
        images: &mut Assets<Image>,
    ) -> Result<Option<SurfaceTextureSet>, String> {
        Ok(Some(match recipe {
            TextureRecipeId::HewnOak => {
                if !self.unit_color.is_empty() || self.mortar_color.is_some() {
                    return Err("unit and mortar colors require a masonry recipe".to_owned());
                }
                generate_hewn_oak_textures(
                    images,
                    &HewnOakColors {
                        light: self.wood_light.unwrap_or(HEWN_OAK_COLORS.light),
                        dark: self.wood_dark.unwrap_or(HEWN_OAK_COLORS.dark),
                    },
                )
            }
            TextureRecipeId::HandmadeBrick => {
                generate_handmade_brick_textures(images, &self.masonry(HANDMADE_BRICK_COLORS)?)
            }
            TextureRecipeId::DressedStone => {
                generate_dressed_stone_textures(images, &self.masonry(DRESSED_STONE_COLORS)?)
            }
            _ => {
                if self.wood_light.is_some()
                    || self.wood_dark.is_some()
                    || !self.unit_color.is_empty()
                    || self.mortar_color.is_some()
                {
                    return Err(
                        "palette overrides support hewn-oak, handmade-brick and dressed-stone"
                            .to_owned(),
                    );
                }
                return Ok(None);
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colors_and_palette_cardinality_are_validated() {
        assert_eq!(parse_color("#aB12fF").unwrap(), SrgbColor([171, 18, 255]));
        for invalid in ["ffffff", "#fff", "#12345z", "#é1234", "#1234567"] {
            assert!(parse_color(invalid).is_err());
        }
        let options = Options {
            unit_color: vec![SrgbColor([0; 3]); 2],
            ..Default::default()
        };
        assert!(options.masonry(HANDMADE_BRICK_COLORS).is_err());
        let mortar = SrgbColor([12, 34, 56]);
        let options = Options {
            mortar_color: Some(mortar),
            ..Default::default()
        };
        let selected = options.masonry(HANDMADE_BRICK_COLORS).unwrap();
        assert_eq!(selected.units, HANDMADE_BRICK_COLORS.units);
        assert_eq!(selected.mortar, mortar);
    }
}

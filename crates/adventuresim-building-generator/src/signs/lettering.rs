use super::*;
use ab_glyph::{Font, FontRef, Glyph, ScaleFont, point};

pub(super) const TEXTURE_WIDTH: u32 = 1024;
const TEXTURE_PADDING: f32 = 58.0;
const MAX_LETTER_HEIGHT: f32 = 160.0;
const LINE_GAP_EM: f32 = 0.15;
const BORDER_INSET: u32 = 23;
const BORDER_STROKE: u32 = 4;

pub struct SignTexture {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub lines: Vec<String>,
}

impl SignFont {
    fn face(self) -> FontRef<'static> {
        let bytes: &[u8] = match self {
            Self::GrenzeGotisch => include_bytes!("../../assets/fonts/GrenzeGotisch-Bold.ttf"),
            Self::UnifrakturCook => include_bytes!("../../assets/fonts/UnifrakturCook-Bold.ttf"),
        };
        FontRef::try_from_slice(bytes).expect("bundled sign font is valid")
    }
}

impl SignFinish {
    pub(super) const fn colors(self) -> ([u8; 3], [u8; 3]) {
        match self {
            Self::PalePaint => ([223, 204, 165], [38, 25, 18]),
            Self::DarkWood => ([44, 32, 28], [237, 224, 193]),
        }
    }
}

impl SignTexture {
    pub fn paint(sign: &ShopSign, size: Vec2) -> Self {
        let width = TEXTURE_WIDTH;
        let height = (width as f32 * size.y / size.x).round() as u32;
        let (paint, ink) = sign.finish.colors();
        let mut rgba = vec![0; (width * height * 4) as usize];
        for y in 0..height {
            for x in 0..width {
                let edge = x.min(width - 1 - x).min(y.min(height - 1 - y));
                let border = (BORDER_INSET..BORDER_INSET + BORDER_STROKE).contains(&edge);
                let color = if border { ink } else { paint };
                let index = ((y * width + x) * 4) as usize;
                rgba[index..index + 3].copy_from_slice(&color);
                rgba[index + 3] = 255;
            }
        }
        let face = sign.font.face();
        let full = sign.name.text();
        let usable_width = width as f32 - TEXTURE_PADDING * 2.0;
        let usable_height = height as f32 - TEXTURE_PADDING * 2.0;
        let single_px = MAX_LETTER_HEIGHT.min(usable_height);
        let lines = if line_width(&face, &full, single_px) <= usable_width {
            vec![full]
        } else {
            vec![sign.name.proprietor.clone(), sign.name.trade.clone()]
        };
        let reference_bounds = lines
            .iter()
            .map(|line| ink_bounds(&face, &layout(&face, line, MAX_LETTER_HEIGHT)))
            .collect::<Vec<_>>();
        let widest = reference_bounds
            .iter()
            .map(|(min, max)| max.x - min.x)
            .fold(0.0_f32, f32::max);
        let reference_height = reference_bounds
            .iter()
            .map(|(min, max)| max.y - min.y)
            .sum::<f32>()
            + MAX_LETTER_HEIGHT * LINE_GAP_EM * (lines.len() - 1) as f32;
        let px = MAX_LETTER_HEIGHT
            * (usable_width / widest.max(1.0))
                .min(usable_height / reference_height.max(1.0))
                .min(1.0);
        let shaped = lines
            .iter()
            .map(|line| {
                let glyphs = layout(&face, line, px);
                let bounds = ink_bounds(&face, &glyphs);
                (glyphs, bounds)
            })
            .collect::<Vec<_>>();
        let line_gap = px * LINE_GAP_EM;
        let total_height = shaped
            .iter()
            .map(|(_, (min, max))| max.y - min.y)
            .sum::<f32>()
            + line_gap * (lines.len() - 1) as f32;
        let mut top = (height as f32 - total_height) * 0.5;
        for (glyphs, (min, max)) in shaped {
            let offset = point((width as f32 - (max.x - min.x)) * 0.5 - min.x, top - min.y);
            top += max.y - min.y + line_gap;
            for mut glyph in glyphs {
                glyph.position += offset;
                if let Some(outline) = face.outline_glyph(glyph) {
                    let bounds = outline.px_bounds();
                    outline.draw(|x, y, coverage| {
                        let x = x as i32 + bounds.min.x as i32;
                        let y = y as i32 + bounds.min.y as i32;
                        if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
                            return;
                        }
                        let offset = ((y as u32 * width + x as u32) * 4) as usize;
                        for channel in 0..3 {
                            rgba[offset + channel] = (rgba[offset + channel] as f32
                                * (1.0 - coverage)
                                + ink[channel] as f32 * coverage)
                                .round() as u8;
                        }
                    });
                }
            }
        }
        Self {
            width,
            height,
            rgba,
            lines,
        }
    }
}

fn layout(face: &FontRef<'_>, text: &str, px: f32) -> Vec<Glyph> {
    let scaled = face.as_scaled(px);
    let mut x = 0.0;
    let mut previous = None;
    text.chars()
        .map(|character| {
            let id = scaled.glyph_id(character);
            assert_ne!(id.0, 0, "sign font lacks {character:?}");
            if let Some(previous) = previous {
                x += scaled.kern(previous, id);
            }
            let glyph = id.with_scale_and_position(px, point(x, scaled.ascent()));
            x += scaled.h_advance(id);
            previous = Some(id);
            glyph
        })
        .collect()
}

fn ink_bounds(face: &FontRef<'_>, glyphs: &[Glyph]) -> (ab_glyph::Point, ab_glyph::Point) {
    let mut min = point(f32::MAX, f32::MAX);
    let mut max = point(f32::MIN, f32::MIN);
    for glyph in glyphs {
        if let Some(outline) = face.outline_glyph(glyph.clone()) {
            let b = outline.px_bounds();
            min.x = min.x.min(b.min.x);
            min.y = min.y.min(b.min.y);
            max.x = max.x.max(b.max.x);
            max.y = max.y.max(b.max.y);
        }
    }
    (min, max)
}

fn line_width(face: &FontRef<'_>, text: &str, px: f32) -> f32 {
    let (min, max) = ink_bounds(face, &layout(face, text, px));
    max.x - min.x
}

//! Colorimetry over the vendored measured curves. See MEASURED_PAINT.md.
use super::{PaintStock, StockMix, WHITE_SCALE};
use serde::Deserialize;
use std::sync::OnceLock;

pub const CALIBRATION_JSON: &str =
    include_str!("../../../references/data/measured-paint-data.json");
pub fn calibration_id() -> &'static str {
    static ID: OnceLock<String> = OnceLock::new();
    ID.get_or_init(|| {
        blake3::hash(CALIBRATION_JSON.as_bytes())
            .to_hex()
            .to_string()
    })
}
pub fn provenance() -> serde_json::Value {
    let data: serde_json::Value =
        serde_json::from_str(CALIBRATION_JSON).expect("vendored calibration");
    serde_json::json!({"id": calibration_id(), "source": data["source"], "model": "Piecewise spectral interpolation; D65/2 degree, 400–780 nm at 5 nm. Gum Arabic on parchment; shield transfer and roughness are estimates."})
}
#[derive(Deserialize)]
struct Data {
    illuminant_d65: Vec<f64>,
    observer_xyz: Vec<[f64; 3]>,
    stocks: Vec<StockData>,
}
#[derive(Deserialize)]
struct StockData {
    id: PaintStock,
    knots: Vec<Knot>,
}
#[derive(Deserialize)]
struct Knot {
    white_permille: u16,
    reflectance: Vec<f64>,
}
fn data() -> &'static Data {
    static DATA: OnceLock<Data> = OnceLock::new();
    DATA.get_or_init(|| serde_json::from_str(CALIBRATION_JSON).expect("vendored calibration"))
}
pub(super) fn reflectance(mix: StockMix) -> Vec<f64> {
    let stock = data()
        .stocks
        .iter()
        .find(|s| s.id == mix.stock())
        .expect("measured stock");
    if stock.knots.len() == 1 {
        return stock.knots[0].reflectance.clone();
    }
    let span = if mix.white_permille() <= WHITE_SCALE / 2 {
        0
    } else {
        1
    };
    let [a, b] = [&stock.knots[span], &stock.knots[span + 1]];
    let weight = f64::from(mix.white_permille() - a.white_permille)
        / f64::from(b.white_permille - a.white_permille);
    a.reflectance
        .iter()
        .zip(&b.reflectance)
        .map(|(a, b)| a + (b - a) * weight)
        .collect()
}
/// Cache the finite palette once; material baking never integrates per pixel.
pub(super) fn color(mix: StockMix) -> [u8; 3] {
    static COLORS: OnceLock<Vec<Vec<[u8; 3]>>> = OnceLock::new();
    COLORS.get_or_init(|| {
        PaintStock::ALL
            .into_iter()
            .map(|stock| {
                let end = if stock.supports_tints() {
                    WHITE_SCALE
                } else {
                    0
                };
                (0..=end)
                    .map(|white| integrate(StockMix::new(stock, white).expect("bounded mix")))
                    .collect()
            })
            .collect()
    })[mix.stock().index()][usize::from(mix.white_permille())]
}
fn integrate(mix: StockMix) -> [u8; 3] {
    let data = data();
    let normalization: f64 = data
        .illuminant_d65
        .iter()
        .zip(&data.observer_xyz)
        .map(|(e, c)| e * c[1])
        .sum();
    let mut xyz = [0.0; 3];
    for ((r, e), observer) in reflectance(mix)
        .into_iter()
        .zip(&data.illuminant_d65)
        .zip(&data.observer_xyz)
    {
        for c in 0..3 {
            xyz[c] += r * e * observer[c] / normalization;
        }
    }
    // IEC sRGB D65 XYZ matrix; clipped only at the display boundary.
    let [x, y, z] = xyz;
    [
        3.2406 * x - 1.5372 * y - 0.4986 * z,
        -0.9689 * x + 1.8758 * y + 0.0415 * z,
        0.0557 * x - 0.2040 * y + 1.0570 * z,
    ]
    .map(|v| {
        let encoded = if v <= 0.0031308 {
            12.92 * v
        } else {
            1.055 * v.powf(1.0 / 2.4) - 0.055
        };
        (encoded.clamp(0.0, 1.0) * 255.0).round() as u8
    })
}
/// CIELAB of the quantized D65 preview; error is explicitly a screen-color error.
pub(super) fn lab(rgb: [u8; 3]) -> [f64; 3] {
    let [r, g, b] = rgb.map(|c| {
        let c = f64::from(c) / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    });
    let xyz = [
        (0.4124 * r + 0.3576 * g + 0.1805 * b) / 0.95047,
        0.2126 * r + 0.7152 * g + 0.0722 * b,
        (0.0193 * r + 0.1192 * g + 0.9505 * b) / 1.08883,
    ];
    let [x, y, z] = xyz.map(|v| {
        if v > (6.0_f64 / 29.0).powi(3) {
            v.cbrt()
        } else {
            v / (3.0 * (6.0_f64 / 29.0).powi(2)) + 4.0 / 29.0
        }
    });
    [116.0 * y - 16.0, 500.0 * (x - y), 200.0 * (y - z)]
}

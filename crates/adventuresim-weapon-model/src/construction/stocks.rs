//! Section lofts for tillers and firearm stocks with explicit lock cavities.
use super::*;

#[derive(Clone, Copy, Debug)]
pub(crate) struct StockStation {
    pub(crate) y: f64,
    pub(crate) width: f64,
    pub(crate) bottom: f64,
    pub(crate) top: f64,
}
impl StockStation {
    pub(crate) fn new(y: f64, width: f64, bottom: f64, top: f64) -> Self {
        Self {
            y,
            width,
            bottom,
            top,
        }
    }
    pub(crate) fn at(stations: &[Self], y: f64) -> Self {
        let index = stations
            .windows(2)
            .position(|p| y <= p[1].y)
            .unwrap_or(stations.len() - 2);
        let a = stations[index];
        let b = stations[index + 1];
        let t = (y - a.y) / (b.y - a.y);
        Self {
            y,
            width: a.width + (b.width - a.width) * t,
            bottom: a.bottom + (b.bottom - a.bottom) * t,
            top: a.top + (b.top - a.top) * t,
        }
    }
}
impl Solid {
    pub(crate) fn stock(stations: &[StockStation]) -> Result<Self, String> {
        if stations.len() < 2
            || stations.iter().any(|s| s.width <= 0.0 || s.top <= s.bottom)
            || stations.windows(2).any(|s| s[1].y <= s[0].y)
        {
            return Err(
                "stock stations need positive sections and increasing axial positions".into(),
            );
        }
        let rings: Vec<_> = stations
            .iter()
            .map(|s| {
                [
                    [-s.width / 2.0, s.y, s.bottom],
                    [s.width / 2.0, s.y, s.bottom],
                    [s.width / 2.0, s.y, s.top],
                    [-s.width / 2.0, s.y, s.top],
                ]
            })
            .collect();
        let mut solid = Self::default();
        for pair in rings.windows(2) {
            for side in 0..4 {
                let next = (side + 1) % 4;
                solid.triangle(pair[0][side], pair[1][next], pair[0][next], 1);
                solid.triangle(pair[0][side], pair[1][side], pair[1][next], 1);
            }
        }
        let first = rings[0];
        let last = *rings.last().unwrap();
        solid.quad(first[0], first[1], first[2], first[3], 2);
        solid.quad(last[0], last[3], last[2], last[1], 2);
        Ok(solid.positive())
    }
}

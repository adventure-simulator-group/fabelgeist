//! Piecewise rational ridge fields shared by contoured blanks.
use super::contoured_plate::PlateField;
use super::*;

pub(super) struct RidgeField<'a> {
    pub length: f64,
    pub stations: &'a [PlateThicknessStation],
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum PlateBand {
    Flat,
    LeftSlope,
    RightSlope,
    LeftOuterSlope,
    RightOuterSlope,
    LeftEdge,
    RightEdge,
}

impl PlateField for RidgeField<'_> {
    type Cell = (usize, PlateBand);
    fn cuts(&self) -> Vec<PlanarCut> {
        let mut cuts = Vec::new();
        for station in self.stations {
            cuts.push(PlanarCut::Axial(station.at.get() * self.length));
        }
        for pair in self.stations.windows(2) {
            if pair.iter().all(|s| s.edge == s.ridge) {
                continue;
            }
            for side in [-1.0, 1.0] {
                let mut fractions = vec![1.0, 0.0];
                if pair.iter().any(|s| s.hollow_depth() > 0.0) {
                    fractions.push(0.5);
                }
                for fraction in fractions {
                    let point = |s: &PlateThicknessStation| {
                        [
                            side * if fraction == 1.0 {
                                s.ridge_half_width.get()
                            } else if fraction == 0.0 {
                                s.flat_half_width.get()
                            } else {
                                (s.flat_half_width.get() + s.ridge_half_width.get()) / 2.0
                            },
                            s.at.get() * self.length,
                        ]
                    };
                    cuts.push(PlanarCut::Transverse {
                        start: point(&pair[0]),
                        end: point(&pair[1]),
                    });
                }
            }
        }
        cuts
    }

    fn cell(&self, [x, y]: PlanarPoint) -> (usize, PlateBand) {
        let t = (y / self.length).clamp(0.0, 1.0);
        let index = self
            .stations
            .windows(2)
            .position(|pair| t <= pair[1].at.get())
            .unwrap();
        let a = &self.stations[index];
        let b = &self.stations[index + 1];
        if a.edge == a.ridge && b.edge == b.ridge {
            return (index, PlateBand::Flat);
        }
        let u = (t - a.at.get()) / (b.at.get() - a.at.get());
        let half =
            a.ridge_half_width.get() + (b.ridge_half_width.get() - a.ridge_half_width.get()) * u;
        let flat =
            a.flat_half_width.get() + (b.flat_half_width.get() - a.flat_half_width.get()) * u;
        let band = if x.abs() <= flat {
            PlateBand::Flat
        } else if x.abs() <= half {
            let outer =
                (a.hollow_depth() > 0.0 || b.hollow_depth() > 0.0) && x.abs() > (flat + half) / 2.0;
            if outer && x < 0.0 {
                PlateBand::LeftOuterSlope
            } else if outer {
                PlateBand::RightOuterSlope
            } else if x < 0.0 {
                PlateBand::LeftSlope
            } else {
                PlateBand::RightSlope
            }
        } else if x < 0.0 {
            PlateBand::LeftEdge
        } else {
            PlateBand::RightEdge
        };
        (index, band)
    }

    fn thickness(&self, x: f64, y: f64) -> f64 {
        let t = (y / self.length).clamp(0.0, 1.0);
        let stations = self
            .stations
            .windows(2)
            .find(|s| t <= s[1].at.get())
            .unwrap();
        let u = (t - stations[0].at.get()) / (stations[1].at.get() - stations[0].at.get());
        let edge = stations[0].edge.get() + (stations[1].edge.get() - stations[0].edge.get()) * u;
        let ridge =
            stations[0].ridge.get() + (stations[1].ridge.get() - stations[0].ridge.get()) * u;
        if edge == 0.0 && ridge == 0.0 {
            return 0.0;
        }
        let half = stations[0].ridge_half_width.get()
            + (stations[1].ridge_half_width.get() - stations[0].ridge_half_width.get()) * u;
        let flat = stations[0].flat_half_width.get()
            + (stations[1].flat_half_width.get() - stations[0].flat_half_width.get()) * u;
        let q = ((half - x.abs()) / (half - flat)).clamp(0.0, 1.0);
        let hollow = stations[0].hollow_depth()
            + (stations[1].hollow_depth() - stations[0].hollow_depth()) * u;
        edge + (ridge - edge) * q - 4.0 * hollow * q.min(1.0 - q)
    }
}

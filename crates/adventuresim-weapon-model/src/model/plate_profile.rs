//! Ordered moving section landmarks define continuous ridges and relieved faces.
use super::contoured_plate::PlateField;
use super::*;

pub(super) struct ProfileField<'a> {
    pub length: f64,
    pub width: f64,
    pub stations: &'a [PlateSectionStation],
}

struct SectionSlice<'a> {
    first: &'a PlateSectionStation,
    second: &'a PlateSectionStation,
    width: f64,
    progress: f64,
}

impl SectionSlice<'_> {
    fn landmark(&self, index: usize) -> (f64, f64) {
        let a = &self.first.profile[index];
        let b = &self.second.profile[index];
        (
            (a.across.get() + (b.across.get() - a.across.get()) * self.progress) * self.width,
            a.thickness.get() + (b.thickness.get() - a.thickness.get()) * self.progress,
        )
    }

    fn band(&self, x: f64) -> usize {
        (0..self.first.profile.len() - 1)
            .find(|&i| x <= self.landmark(i + 1).0)
            .unwrap_or(self.first.profile.len() - 2)
    }
}

impl ProfileField<'_> {
    fn slice(&self, y: f64) -> (usize, SectionSlice<'_>) {
        let at = (y / self.length).clamp(0.0, 1.0);
        let index = self
            .stations
            .windows(2)
            .position(|p| at <= p[1].at.get())
            .unwrap();
        let first = &self.stations[index];
        let second = &self.stations[index + 1];
        let progress = (at - first.at.get()) / (second.at.get() - first.at.get());
        (
            index,
            SectionSlice {
                first,
                second,
                width: self.width,
                progress,
            },
        )
    }
}

impl PlateField for ProfileField<'_> {
    type Cell = (usize, usize);

    fn cuts(&self) -> Vec<PlanarCut> {
        let mut cuts: Vec<_> = self
            .stations
            .iter()
            .map(|s| PlanarCut::Axial(s.at.get() * self.length))
            .collect();
        for pair in self.stations.windows(2) {
            for i in 1..pair[0].profile.len() - 1 {
                let point = |s: &PlateSectionStation| {
                    [
                        s.profile[i].across.get() * self.width,
                        s.at.get() * self.length,
                    ]
                };
                cuts.push(PlanarCut::Transverse {
                    start: point(&pair[0]),
                    end: point(&pair[1]),
                });
            }
        }
        cuts
    }

    fn cell(&self, [x, y]: PlanarPoint) -> Self::Cell {
        let (index, slice) = self.slice(y);
        (index, slice.band(x))
    }

    fn thickness(&self, x: f64, y: f64) -> f64 {
        let (_, slice) = self.slice(y);
        let index = slice.band(x);
        let (left, a) = slice.landmark(index);
        let (right, b) = slice.landmark(index + 1);
        let progress = ((x - left) / (right - left)).clamp(0.0, 1.0);
        a + (b - a) * progress
    }
}

//! Ordered face identities shared by ridge and authored-profile fields.

/// A station interval and its section band identify one continuous plate face.
///
/// Each field uses one band family. Station-first ordering preserves contour
/// grouping's original tuple ordering within either family.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::model) struct PlateCell {
    station: usize,
    band: CellBand,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum CellBand {
    Ridge(PlateBand),
    Profile(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::model) enum PlateBand {
    Flat,
    LeftSlope,
    RightSlope,
    LeftOuterSlope,
    RightOuterSlope,
    LeftEdge,
    RightEdge,
}

impl PlateCell {
    pub(in crate::model) fn ridge(station: usize, band: PlateBand) -> Self {
        Self {
            station,
            band: CellBand::Ridge(band),
        }
    }

    pub(in crate::model) fn profile(station: usize, band: usize) -> Self {
        Self {
            station,
            band: CellBand::Profile(band),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_face_sorting_preserves_station_then_band_order() {
        let mut cells = [
            PlateCell::profile(2, 0),
            PlateCell::profile(0, 3),
            PlateCell::profile(1, 0),
            PlateCell::profile(0, 1),
            PlateCell::profile(0, 3),
        ];
        cells.sort();
        assert_eq!(
            cells,
            [
                PlateCell::profile(0, 1),
                PlateCell::profile(0, 3),
                PlateCell::profile(0, 3),
                PlateCell::profile(1, 0),
                PlateCell::profile(2, 0),
            ]
        );
    }

    #[test]
    fn ridge_face_sorting_preserves_station_then_band_order() {
        let mut cells = [
            PlateCell::ridge(1, PlateBand::Flat),
            PlateCell::ridge(0, PlateBand::RightEdge),
            PlateCell::ridge(0, PlateBand::LeftSlope),
            PlateCell::ridge(0, PlateBand::Flat),
            PlateCell::ridge(0, PlateBand::LeftSlope),
        ];
        cells.sort();
        assert_eq!(
            cells,
            [
                PlateCell::ridge(0, PlateBand::Flat),
                PlateCell::ridge(0, PlateBand::LeftSlope),
                PlateCell::ridge(0, PlateBand::LeftSlope),
                PlateCell::ridge(0, PlateBand::RightEdge),
                PlateCell::ridge(1, PlateBand::Flat),
            ]
        );
    }
}

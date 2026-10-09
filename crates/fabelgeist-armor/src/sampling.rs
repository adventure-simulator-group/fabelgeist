//! Parametric sampling for shipped armor and its separate bake source.
//!
//! Detail levels change where a recipe is evaluated. They never collapse edges
//! or weld vertices from an already triangulated armor mesh.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArmorLod {
    Lod4,
    Lod5,
    Lod6,
}

impl TryFrom<u8> for ArmorLod {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            4 => Ok(Self::Lod4),
            5 => Ok(Self::Lod5),
            6 => Ok(Self::Lod6),
            _ => Err("armor LOD must be 4, 5 or 6"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArmorDetail {
    Runtime(ArmorLod),
    BakeSource,
}

impl ArmorDetail {
    /// Uniform intervals between structural boundaries. Callers retain explicit
    /// openings, plate overlaps, tips and other authored feature coordinates.
    pub fn segments(self, source: usize, minimum: usize) -> usize {
        let divisor = match self {
            Self::BakeSource => 1,
            Self::Runtime(ArmorLod::Lod4) => 12,
            Self::Runtime(ArmorLod::Lod5) => 16,
            Self::Runtime(ArmorLod::Lod6) => 24,
        };
        source.div_ceil(divisor).max(minimum).min(source)
    }

    pub(crate) fn fluting(
        self,
        pattern: Option<&crate::PlateFluting>,
    ) -> Option<&crate::PlateFluting> {
        match self {
            Self::BakeSource => pattern,
            Self::Runtime(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_sampling_retains_end_intervals_and_drops_relief() {
        let pattern = crate::PlateFluting::default();
        for lod in [ArmorLod::Lod4, ArmorLod::Lod5, ArmorLod::Lod6] {
            let detail = ArmorDetail::Runtime(lod);
            assert!(detail.segments(40, 6) < 40);
            assert_eq!(detail.segments(2, 2), 2);
            assert!(detail.fluting(Some(&pattern)).is_none());
        }
        assert!(ArmorDetail::BakeSource.fluting(Some(&pattern)).is_some());
    }
}

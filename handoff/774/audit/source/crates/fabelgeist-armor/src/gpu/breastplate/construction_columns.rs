//! Correspondence between retained construction rails and the carrier chart.
use crate::{GenerateError, SurfaceColumn};

#[derive(Clone, Debug)]
pub(crate) struct ConstructionColumns {
    first: usize,
    end: usize,
}

impl ConstructionColumns {
    pub fn full(width: usize) -> Self {
        Self {
            first: 0,
            end: width,
        }
    }

    pub fn retained(rails: &[Vec<usize>]) -> Result<Self, GenerateError> {
        let first = rails
            .iter()
            .position(|rail| !rail.is_empty())
            .ok_or(GenerateError::Degenerate)?;
        let end = rails.iter().rposition(|rail| !rail.is_empty()).unwrap() + 1;
        if end - first < 2 || rails[first..end].iter().any(Vec::is_empty) {
            return Err(GenerateError::InvalidSurface);
        }
        Ok(Self { first, end })
    }

    pub fn range(&self) -> std::ops::Range<usize> {
        self.first..self.end
    }

    pub fn width(&self) -> usize {
        self.end - self.first
    }

    /// Fractional cut vertices outside the retained vertical rails use the
    /// endpoint rail. They remain actual rendered samples, never invented metal.
    pub fn column(&self, source: SurfaceColumn, source_width: usize) -> SurfaceColumn {
        let (a, b, blend) = source
            .bracket(source_width as u32)
            .expect("carrier column must belong to its source chart");
        let local = |column: u32| {
            SurfaceColumn::at(
                (column as usize).clamp(self.first, self.end - 1) as u32 - self.first as u32,
            )
        };
        SurfaceColumn::between(local(a), local(b), blend)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retained_rails_preserve_fractional_correspondence_and_cut_endpoints() {
        let columns =
            ConstructionColumns::retained(&[vec![], vec![1], vec![2], vec![3], vec![]]).unwrap();
        assert_eq!(columns.width(), 3);
        for (source, expected) in [(0, 0), (1, 0), (2, 1), (3, 2), (4, 2)] {
            assert_eq!(
                columns.column(SurfaceColumn::at(source), 5),
                SurfaceColumn::at(expected)
            );
        }
        let source = SurfaceColumn::between(SurfaceColumn::at(1), SurfaceColumn::at(2), 0.25);
        assert_eq!(columns.column(source, 5).bracket(3), Some((0, 1, 0.25)));
        assert!(ConstructionColumns::retained(&[vec![0], vec![], vec![2]]).is_err());
    }
}

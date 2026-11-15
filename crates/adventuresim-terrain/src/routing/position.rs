//! Grid-local search identity. Coordinates count cells from the grid origin;
//! numeric access is for the `RoutingGrid` boundary and the distance kernel.

use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct GridRow(u32);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct GridColumn(u32);

// Field order preserves the previous row-major tie order without flattening.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) struct GridPosition {
    row: GridRow,
    column: GridColumn,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct GridDirection {
    row: Ordering,
    column: Ordering,
}

impl GridPosition {
    pub(super) fn row(self) -> u32 {
        self.row.0
    }

    pub(super) fn column(self) -> u32 {
        self.column.0
    }

    pub(super) fn direction_to(self, next: Self) -> GridDirection {
        GridDirection {
            row: next.row.cmp(&self.row),
            column: next.column.cmp(&self.column),
        }
    }

    // Offsets are grid-cell steps supplied by the bounded neighbor kernel.
    pub(super) fn offset(self, column_offset: i32, row_offset: i32) -> Option<Self> {
        Some(Self {
            row: GridRow(self.row.0.checked_add_signed(row_offset)?),
            column: GridColumn(self.column.0.checked_add_signed(column_offset)?),
        })
    }
}

impl From<(u32, u32)> for GridPosition {
    fn from((column, row): (u32, u32)) -> Self {
        Self {
            row: GridRow(row),
            column: GridColumn(column),
        }
    }
}

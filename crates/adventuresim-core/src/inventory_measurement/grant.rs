//! Requested issuance counts and the allocation of fungible or separate rows.

use super::ItemQuantity;

/// Zero requests are explicit no-ops; an issued row always has a nonzero count.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InventoryGrantQuantity {
    Empty,
    Items(ItemQuantity),
}

/// Stacks retain the requested count, while separate rows each contain one unit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InventoryRowAllocation {
    Stacked,
    Individual,
}

impl InventoryGrantQuantity {
    pub fn rows(
        self,
        allocation: InventoryRowAllocation,
    ) -> impl ExactSizeIterator<Item = ItemQuantity> {
        let (quantity, rows) = match (self, allocation) {
            (Self::Empty, _) => (ItemQuantity::ONE, 0),
            (Self::Items(quantity), InventoryRowAllocation::Stacked) => (quantity, 1),
            (Self::Items(quantity), InventoryRowAllocation::Individual) => (
                ItemQuantity::ONE,
                usize::try_from(quantity.get())
                    .expect("native inventory count fits the row iterator"),
            ),
        };
        std::iter::repeat_n(quantity, rows)
    }
}

impl From<u32> for InventoryGrantQuantity {
    fn from(quantity: u32) -> Self {
        match ItemQuantity::new(quantity) {
            Some(quantity) => Self::Items(quantity),
            None => Self::Empty,
        }
    }
}

impl From<ItemQuantity> for InventoryGrantQuantity {
    fn from(quantity: ItemQuantity) -> Self {
        Self::Items(quantity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_requests_never_issue_a_row() {
        let empty = InventoryGrantQuantity::from(0);
        assert_eq!(empty, InventoryGrantQuantity::Empty);
        for allocation in [
            InventoryRowAllocation::Stacked,
            InventoryRowAllocation::Individual,
        ] {
            assert_eq!(empty.rows(allocation).next(), None);
        }
    }

    #[test]
    fn allocation_conserves_units_without_empty_rows() {
        for raw in 1..=256 {
            let quantity = ItemQuantity::new(raw).unwrap();
            let request = InventoryGrantQuantity::from(quantity);
            assert_eq!(
                request
                    .rows(InventoryRowAllocation::Stacked)
                    .collect::<Vec<_>>(),
                [quantity]
            );
            let mut total = 0;
            for unit in request.rows(InventoryRowAllocation::Individual) {
                assert_eq!(unit, ItemQuantity::ONE);
                total += unit.get();
            }
            assert_eq!(total, raw);
        }
    }

    #[test]
    fn maximum_native_quantity_is_not_truncated_or_eagerly_allocated() {
        let request = InventoryGrantQuantity::from(u32::MAX);
        let mut individual = request.rows(InventoryRowAllocation::Individual);
        assert_eq!(individual.len(), usize::try_from(u32::MAX).unwrap());
        assert_eq!(individual.next(), Some(ItemQuantity::ONE));
        assert_eq!(individual.len(), usize::try_from(u32::MAX - 1).unwrap());
        assert_eq!(
            request.rows(InventoryRowAllocation::Stacked).next(),
            ItemQuantity::new(u32::MAX)
        );
    }
}

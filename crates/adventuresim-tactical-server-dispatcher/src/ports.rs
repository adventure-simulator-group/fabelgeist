//! Bounded mission listener allocation. Exhaustion must never wrap into other services.

pub(super) struct TacticalPorts {
    next: Option<u16>,
    last: u16,
}

impl TacticalPorts {
    pub(super) fn new(first: u16, last: u16) -> Result<Self, &'static str> {
        if first == 0 || first > last {
            return Err("tactical ports require a nonzero, ordered range");
        }
        Ok(Self {
            next: Some(first),
            last,
        })
    }

    pub(super) fn allocate(&mut self) -> Option<u16> {
        let port = self.next?;
        self.next = port.checked_add(1).filter(|next| *next <= self.last);
        Some(port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocation_stops_without_wrapping_or_entering_other_services() {
        for (first, last) in [(6001, 6002), (u16::MAX, u16::MAX)] {
            let mut ports = TacticalPorts::new(first, last).unwrap();
            for expected in first..=last {
                assert_eq!(ports.allocate(), Some(expected));
            }
            assert_eq!(ports.allocate(), None);
            assert_eq!(ports.allocate(), None);
        }
        assert!(TacticalPorts::new(0, 1).is_err());
        assert!(TacticalPorts::new(2, 1).is_err());
    }
}

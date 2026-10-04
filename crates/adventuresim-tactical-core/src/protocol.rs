//! Distinct serial-number families for transient tactical protocols.
//!
//! Serial ordering uses the forward half of the wrapping `u32` space. These
//! values deliberately do not implement `Ord`: numeric ordering is incorrect
//! across rollover. They are not elapsed durations or animation frame indexes.
//!
//! ```compile_fail
//! use adventuresim_tactical_core::protocol::{InputTick, EquipmentRevision};
//! let tick = InputTick::new(1);
//! let revision: EquipmentRevision = tick;
//! ```

use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};

macro_rules! counter {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Reflect, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(u32);

        impl $name {
            pub const fn new(value: u32) -> Self {
                Self(value)
            }
            pub const fn get(self) -> u32 {
                self.0
            }
            pub const fn next(self) -> Self {
                Self(self.0.wrapping_add(1))
            }
        }

        impl From<u32> for $name {
            fn from(value: u32) -> Self {
                Self(value)
            }
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

macro_rules! ordered_serial {
    ($name:ident) => {
        counter!($name);
        impl $name {
            pub const fn is_newer_than(self, previous: Self) -> bool {
                let distance = self.0.wrapping_sub(previous.0);
                distance != 0 && distance <= u32::MAX / 2
            }
        }
    };
}

ordered_serial!(InputTick);
ordered_serial!(EquipmentSequence);
ordered_serial!(JumpSequence);
ordered_serial!(PostureSequence);
counter!(EquipmentRevision);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serial_wire_values_remain_numeric_across_rollover() {
        let original = serde_json::to_string(&InputTick::new(u32::MAX)).unwrap();
        assert_eq!(original, u32::MAX.to_string());
        let last: InputTick = serde_json::from_str(&original).unwrap();
        let next: InputTick = serde_json::from_str("0").unwrap();
        assert!(next.is_newer_than(last));
        assert!(serde_json::from_str::<EquipmentSequence>("-1").is_err());
        assert!(serde_json::from_str::<EquipmentRevision>("4294967296").is_err());
    }

    #[test]
    fn serial_order_rejects_replays_and_the_old_half_after_rollover() {
        macro_rules! verify {
            ($name:ident) => {
                let last = $name::new(u32::MAX);
                let next = last.next();
                assert_eq!(next.get(), 0);
                assert!(next.is_newer_than(last));
                assert!(!last.is_newer_than(next));
                assert!(!next.is_newer_than(next));
                assert!(!$name::new(1 << 31).is_newer_than(next));
                assert!($name::new(u32::MAX / 2).is_newer_than(next));
            };
        }
        verify!(InputTick);
        verify!(EquipmentSequence);
        verify!(JumpSequence);
        verify!(PostureSequence);
        assert_eq!(EquipmentRevision::new(u32::MAX).next().get(), 0);
    }
}

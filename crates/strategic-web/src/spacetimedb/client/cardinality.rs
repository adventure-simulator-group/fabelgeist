//! Cardinalities and addresses in the SQL/SATS protocol.
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct QueryRowCount(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum QueryCardinality {
    Empty,
    Singleton,
    Multiple,
}

impl QueryRowCount {
    pub(super) const fn cardinality(self) -> QueryCardinality {
        match self.0 {
            0 => QueryCardinality::Empty,
            1 => QueryCardinality::Singleton,
            _ => QueryCardinality::Multiple,
        }
    }
}

impl From<usize> for QueryRowCount {
    fn from(count: usize) -> Self {
        Self(count)
    }
}

impl fmt::Display for QueryRowCount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct QueryColumnCount(usize);

impl From<usize> for QueryColumnCount {
    fn from(count: usize) -> Self {
        Self(count)
    }
}

impl fmt::Display for QueryColumnCount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ProductFieldCount(usize);

impl From<usize> for ProductFieldCount {
    fn from(count: usize) -> Self {
        Self(count)
    }
}

impl fmt::Display for ProductFieldCount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SumValueCount(usize);

impl SumValueCount {
    pub(super) const TAG_AND_PAYLOAD: Self = Self(2);
}

impl From<usize> for SumValueCount {
    fn from(count: usize) -> Self {
        Self(count)
    }
}

impl fmt::Display for SumValueCount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SatsSumTag(u64);

impl SatsSumTag {
    pub(super) fn variant(self, variants: &[serde_json::Value]) -> Option<&serde_json::Value> {
        match usize::try_from(self.0) {
            Ok(index) => variants.get(index),
            Err(_) => None,
        }
    }
}

impl From<u64> for SatsSumTag {
    fn from(tag: u64) -> Self {
        Self(tag)
    }
}

impl fmt::Display for SatsSumTag {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct QueryRowIndex(usize);

impl From<usize> for QueryRowIndex {
    fn from(index: usize) -> Self {
        Self(index)
    }
}

impl fmt::Display for QueryRowIndex {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct QueryColumnIndex(usize);

impl From<usize> for QueryColumnIndex {
    fn from(index: usize) -> Self {
        Self(index)
    }
}

impl fmt::Display for QueryColumnIndex {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

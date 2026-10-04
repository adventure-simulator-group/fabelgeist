//! Structural SQL/SATS failures and generated-row admission causes.
use super::super::cardinality::{
    ProductFieldCount, QueryColumnCount, QueryColumnIndex, QueryRowIndex, SatsSumTag, SumValueCount,
};

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub(crate) enum SatsValueError {
    #[error("expected hexadecimal bytes")]
    ExpectedHexBytes,
    #[error("invalid hexadecimal bytes")]
    InvalidHexBytes,
    #[error("expected an array")]
    ExpectedArray,
    #[error("expected a tagged sum array")]
    ExpectedTaggedSum,
    #[error("sum expected a tag and payload but received {received} values")]
    SumCardinality { received: SumValueCount },
    #[error("sum tag was not an unsigned integer")]
    ExpectedUnsignedSumTag,
    #[error("unknown sum tag {tag}")]
    UnknownSumTag { tag: SatsSumTag },
    #[error("option variant had no name")]
    UnnamedOptionVariant,
    #[error("expected a product array")]
    ExpectedProductArray,
    #[error("product expected {expected} fields but received {received}")]
    ProductCardinality {
        expected: ProductFieldCount,
        received: ProductFieldCount,
    },
    #[error("identity was not a hexadecimal string")]
    ExpectedHexIdentity,
    #[error("identity contained invalid hexadecimal digits")]
    InvalidHexIdentity,
    #[error("SpacetimeDB returned a non-array SQL row")]
    ExpectedSqlRowArray,
    #[error("row expected {expected} columns but received {received}")]
    RowCardinality {
        expected: QueryColumnCount,
        received: QueryColumnCount,
    },
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum SatsQueryDecodeError {
    #[error("generated-row query returned an unnamed column at index {column}")]
    UnnamedColumn { column: QueryColumnIndex },
    #[error("malformed SpacetimeDB SQL value in row {row}: {source}")]
    MalformedRow {
        row: QueryRowIndex,
        #[source]
        source: SatsValueError,
    },
    #[error("cannot decode generated SpacetimeDB row {row}: {source}")]
    GeneratedRow {
        row: QueryRowIndex,
        #[source]
        source: serde_json::Error,
    },
}

//! Encode generated requests for the database's raw HTTP reducer endpoint.

use spacetimedb_sats::{
    algebraic_value::ser::value_serialize, ser::Serialize, serde::SerdeWrapper,
};

/// Use SATS product positions and sum tags from the generated request itself.
/// Generated Rust variant spelling differs from the module's HTTP schema names;
/// positional encoding carries the same schema without rewriting payload keys.
pub fn json_reducer_argument<T: Serialize + ?Sized>(
    request: &T,
) -> serde_json::Result<serde_json::Value> {
    serde_json::to_value(SerdeWrapper::from_ref(&value_serialize(request)))
}

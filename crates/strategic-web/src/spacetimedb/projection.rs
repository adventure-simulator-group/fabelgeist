//! Convert generated SATS fields into their owning presentation/domain schema.
use adventuresim_core::strategic_place::PlaceIdentityError;
use serde::de::DeserializeOwned;
use serde_json::Value;
use spacetimedb_sats::{ser::Serialize as SatsSerialize, serde::SerdeWrapper};
use std::{convert::Infallible, fmt};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewField {
    SettlementLanguages,
    SettlementIndustries,
    SettlementEconomy,
    TravelRoute,
    TravelTerrain,
}

impl fmt::Display for ViewField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::SettlementLanguages => "settlement languages",
            Self::SettlementIndustries => "settlement industries",
            Self::SettlementEconomy => "settlement economy",
            Self::TravelRoute => "travel route",
            Self::TravelTerrain => "travel terrain",
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ViewProjectionError {
    #[error("cannot encode generated {field}: {source}")]
    Encode {
        field: ViewField,
        #[source]
        source: serde_json::Error,
    },
    #[error("cannot admit {field} into its domain schema: {source}")]
    Decode {
        field: ViewField,
        #[source]
        source: serde_json::Error,
    },
    #[error("invalid generated case-site identity: {0}")]
    CaseSiteIdentity(#[from] PlaceIdentityError),
}

impl From<Infallible> for ViewProjectionError {
    fn from(impossible: Infallible) -> Self {
        match impossible {}
    }
}

pub(super) fn sats_to_serde<T, U>(
    value: &T,
    field: ViewField,
) -> std::result::Result<U, ViewProjectionError>
where
    T: SatsSerialize + ?Sized,
    U: DeserializeOwned,
{
    let encoded = match serde_json::to_value(SerdeWrapper::from_ref(value)) {
        Ok(encoded) => encoded,
        Err(source) => return Err(ViewProjectionError::Encode { field, source }),
    };
    match serde_json::from_value(normalize_sats_serde_value(encoded)) {
        Ok(projected) => Ok(projected),
        Err(source) => Err(ViewProjectionError::Decode { field, source }),
    }
}

fn normalize_sats_serde_value(value: Value) -> Value {
    match value {
        Value::Array(values) => {
            Value::Array(values.into_iter().map(normalize_sats_serde_value).collect())
        }
        Value::Object(object) if object.len() == 1 => {
            let (name, payload) = object.into_iter().next().expect("one field");
            if name.eq_ignore_ascii_case("none") && payload.as_array().is_some_and(Vec::is_empty) {
                return Value::Null;
            }
            if name.eq_ignore_ascii_case("some") {
                return normalize_sats_serde_value(payload);
            }
            if payload.as_array().is_some_and(Vec::is_empty) {
                return Value::String(name);
            }
            Value::Object(
                [(name, normalize_sats_serde_value(payload))]
                    .into_iter()
                    .collect(),
            )
        }
        Value::Object(object) => Value::Object(
            object
                .into_iter()
                .map(|(name, value)| (name, normalize_sats_serde_value(value)))
                .collect(),
        ),
        value => value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spacetimedb_sats::ser::{Error as _, Serializer};
    use std::error::Error as _;

    struct UnencodableSatsField;

    impl SatsSerialize for UnencodableSatsField {
        fn serialize<S: Serializer>(&self, _: S) -> std::result::Result<S::Ok, S::Error> {
            Err(S::Error::custom("synthetic SDK encoder failure"))
        }
    }

    #[test]
    fn failed_sats_encoding_retains_the_field_stage_and_native_cause() {
        let error = sats_to_serde::<_, adventuresim_world_schema::SettlementLanguageProfile>(
            &UnencodableSatsField,
            ViewField::SettlementLanguages,
        )
        .unwrap_err();
        assert!(matches!(
            error,
            ViewProjectionError::Encode {
                field: ViewField::SettlementLanguages,
                ..
            }
        ));
        let source = error
            .source()
            .unwrap()
            .downcast_ref::<serde_json::Error>()
            .unwrap();
        assert!(source.is_data());
        assert_eq!(source.to_string(), "synthetic SDK encoder failure");
    }

    #[test]
    fn duplicate_industry_evidence_is_rejected_by_the_domain_schema_with_context() {
        let generated = adventuresim_stdb_client::InferredIndustryProfile {
            outputs: vec![
                adventuresim_stdb_client::IndustryEvidence::Fallback(
                    adventuresim_stdb_client::FallbackIndustry::WoodlandFuelwood,
                );
                2
            ],
        };
        let error = sats_to_serde::<_, adventuresim_world_schema::InferredIndustryProfile>(
            &generated,
            ViewField::SettlementIndustries,
        )
        .unwrap_err();
        assert!(matches!(
            error,
            ViewProjectionError::Decode {
                field: ViewField::SettlementIndustries,
                ..
            }
        ));
        assert!(
            error
                .source()
                .unwrap()
                .downcast_ref::<serde_json::Error>()
                .unwrap()
                .is_data()
        );
    }
}

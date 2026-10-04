//! Provision forecasts retain database, interval, and custody admission causes.

#[derive(Debug, thiserror::Error)]
pub(crate) enum TravelProvisionError {
    #[error("{0}")]
    Query(#[from] crate::spacetimedb::SpacetimeError),
    #[error("{0}")]
    AlcoholInterval(#[from] adventuresim_core::alcohol::AlcoholIntervalError),
    #[error("{0}")]
    Custody(#[from] adventuresim_core::physical_object::CustodyIdentityError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn custody_admission_keeps_its_class_cause_and_existing_diagnostic() {
        let cause = adventuresim_core::physical_object::OperationalCustody::character((0).into())
            .unwrap_err();
        let expected = cause.to_string();
        let error = TravelProvisionError::from(cause);
        assert!(matches!(error, TravelProvisionError::Custody(_)));
        assert!(
            error
                .source()
                .unwrap()
                .is::<adventuresim_core::physical_object::CustodyIdentityError>()
        );
        assert_eq!(error.to_string(), expected);
    }
}

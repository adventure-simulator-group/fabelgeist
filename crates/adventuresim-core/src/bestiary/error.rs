//! Failed admission of a canonical, authored threat identity.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnknownThreatId;

impl std::fmt::Display for UnknownThreatId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("unknown threat ID")
    }
}

impl std::error::Error for UnknownThreatId {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bestiary::ThreatId;
    use std::error::Error;

    #[test]
    fn invalid_authored_vocabulary_exposes_the_same_standard_and_serde_failure() {
        for key in ["Bandit", "not_an_authored_threat"] {
            let error = key.parse::<ThreatId>().unwrap_err();
            let cause: &dyn Error = &error;
            assert_eq!(
                cause.downcast_ref::<UnknownThreatId>(),
                Some(&UnknownThreatId)
            );
            assert!(cause.source().is_none());
            assert_eq!(cause.to_string(), "unknown threat ID");
            let wire = serde_json::to_string(key).unwrap();
            assert_eq!(
                serde_json::from_str::<ThreatId>(&wire)
                    .unwrap_err()
                    .to_string(),
                "unknown threat ID"
            );
        }
        assert_eq!("bandit".parse::<ThreatId>().unwrap(), ThreatId::Bandit);
    }
}

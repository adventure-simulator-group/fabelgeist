//! Canonical identities for investigation records.
use super::ValidationError;
use serde::{Deserialize, Serialize};

macro_rules! stable_id {
    ($name:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        pub struct $name(String);
        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
                let value = value.into();
                if value.is_empty()
                    || value.len() > 256
                    || !value.bytes().all(|b| {
                        b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b':' | b'.')
                    })
                {
                    return Err(ValidationError::InvalidId);
                }
                Ok(Self(value))
            }
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
            }
        }
    };
}

stable_id!(CaseId);
stable_id!(EventId);
stable_id!(PropositionId);
stable_id!(ObservationId);
stable_id!(RecollectionId);
stable_id!(ClaimId);
stable_id!(EvidenceId);
stable_id!(BeliefId);
stable_id!(LeadId);
stable_id!(RevisionId);
stable_id!(SharingReceiptId);
stable_id!(EvidenceKnowledgeSourceId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoded_record_ids_obey_the_same_canonical_contract() {
        for invalid in ["", "space separated", "control\n", &"a".repeat(257)] {
            let json = serde_json::to_string(invalid).unwrap();
            assert!(serde_json::from_str::<CaseId>(&json).is_err());
            assert!(serde_json::from_str::<EvidenceId>(&json).is_err());
        }
        let id: CaseId = serde_json::from_str("\"case:one\"").unwrap();
        assert_eq!(id.as_str(), "case:one");
        assert_eq!(serde_json::to_string(&id).unwrap(), "\"case:one\"");
    }
}

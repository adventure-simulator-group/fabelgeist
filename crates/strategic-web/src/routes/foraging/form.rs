//! Bounded browser submissions; plan eligibility remains reducer authority.

use crate::routes::return_url::LocalReturnUrl;
use adventuresim_world_schema::calendar::{MINUTES_PER_HOUR, StrategicDuration};
use serde::Serialize;
mod error;
use error::{ForageFormError, ForageScalarField};

pub(super) const FORAGE_FORM_MAX_BYTES: usize = 1_024;
const FORAGE_FORM_MAX_PAIRS: usize = 8;
const FORAGE_FORM_MAX_SOURCES: usize = 5;
const FORAGE_FORM_MAX_SOURCE_LEN: usize = 32;
const FORAGE_FORM_MAX_RETURN_TO_LEN: usize = 512;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct ForageForm {
    source: ForageSubmittedSources,
    hours: ForageSubmittedHours,
    return_to: ForageReturnHint,
}

impl ForageForm {
    pub(super) fn sources(&self) -> &ForageSubmittedSources {
        &self.source
    }
    pub(super) fn duration(&self) -> StrategicDuration {
        self.hours.duration()
    }
    pub(super) fn return_destination(&self) -> LocalReturnUrl<'_> {
        self.return_to.resolve()
    }
}

impl TryFrom<&[u8]> for ForageForm {
    type Error = ForageFormError;
    fn try_from(body: &[u8]) -> std::result::Result<Self, Self::Error> {
        if body.len() > FORAGE_FORM_MAX_BYTES {
            return Err(ForageFormError::BodyTooLarge);
        }
        let mut source = ForageSubmittedSources::default();
        let mut hours = None;
        let mut return_to = None;
        for (pair_index, (key, value)) in form_urlencoded::parse(body).enumerate() {
            if pair_index >= FORAGE_FORM_MAX_PAIRS {
                return Err(ForageFormError::TooManyPairs);
            }
            match ForageFormField::from_http_key(key.as_ref()) {
                Some(ForageFormField::Source) => {
                    source.check_vacancy()?;
                    source.try_push(ForageSourceToken::try_from(value.as_ref())?)?;
                }
                Some(ForageFormField::Scalar(ForageScalarField::Hours)) => {
                    if hours.is_some() {
                        return Err(ForageFormError::DuplicateScalar(ForageScalarField::Hours));
                    }
                    hours = Some(ForageSubmittedHours::try_from(value.as_ref())?);
                }
                Some(ForageFormField::Scalar(ForageScalarField::ReturnTo)) => {
                    if return_to.is_some() {
                        return Err(ForageFormError::DuplicateScalar(
                            ForageScalarField::ReturnTo,
                        ));
                    }
                    return_to = Some(ForageReturnHint::try_from(value.as_ref())?);
                }
                None => {}
            }
        }
        Ok(Self {
            source,
            hours: hours.ok_or(ForageFormError::MissingScalar(ForageScalarField::Hours))?,
            return_to: return_to
                .ok_or(ForageFormError::MissingScalar(ForageScalarField::ReturnTo))?,
        })
    }
}

enum ForageFormField {
    Source,
    Scalar(ForageScalarField),
}
impl ForageFormField {
    fn from_http_key(key: &str) -> Option<Self> {
        match key {
            "source" => Some(Self::Source),
            "hours" => Some(Self::Scalar(ForageScalarField::Hours)),
            "return_to" => Some(Self::Scalar(ForageScalarField::ReturnTo)),
            _ => None,
        }
    }
}

/// Complete ordered submission, including duplicates and unknown source tokens.
#[derive(Debug, Default, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub(super) struct ForageSubmittedSources(Vec<ForageSourceToken>);
impl ForageSubmittedSources {
    fn check_vacancy(&self) -> std::result::Result<(), ForageFormError> {
        if self.0.len() >= FORAGE_FORM_MAX_SOURCES {
            Err(ForageFormError::TooManySources)
        } else {
            Ok(())
        }
    }
    fn try_push(&mut self, source: ForageSourceToken) -> std::result::Result<(), ForageFormError> {
        self.check_vacancy()?;
        self.0.push(source);
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
struct ForageSourceToken(String);
impl TryFrom<&str> for ForageSourceToken {
    type Error = ForageFormError;
    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        if value.len() > FORAGE_FORM_MAX_SOURCE_LEN {
            return Err(ForageFormError::SourceTooLong);
        }
        Ok(Self(value.to_owned()))
    }
}

/// Eight-bit submitted hours; legal whole-hour plans are checked by the reducer.
#[derive(Debug, PartialEq, Eq)]
struct ForageSubmittedHours(u8);
impl TryFrom<&str> for ForageSubmittedHours {
    type Error = ForageFormError;
    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        value.parse().map(Self).map_err(ForageFormError::Hours)
    }
}
impl ForageSubmittedHours {
    fn duration(&self) -> StrategicDuration {
        StrategicDuration::new(u64::from(self.0) * MINUTES_PER_HOUR)
    }
}

/// Bounded decoded browser hint, admitted as a local URL before navigation.
#[derive(Debug, PartialEq, Eq)]
struct ForageReturnHint(String);
impl TryFrom<&str> for ForageReturnHint {
    type Error = ForageFormError;
    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        if value.len() > FORAGE_FORM_MAX_RETURN_TO_LEN {
            return Err(ForageFormError::ReturnHintTooLong);
        }
        Ok(Self(value.to_owned()))
    }
}
impl ForageReturnHint {
    fn resolve(&self) -> LocalReturnUrl<'_> {
        LocalReturnUrl::try_from(self.0.as_str()).unwrap_or(LocalReturnUrl::ROOT)
    }
}

#[cfg(test)]
mod tests;

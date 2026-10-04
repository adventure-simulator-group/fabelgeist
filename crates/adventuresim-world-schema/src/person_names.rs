//! Historically sourced personal-name identities, generation, and rendering.

use crate::{Culture, OfficialReligion, Sex, calendar::CalendarYear};
use fabelgeist_determinism::StreamId;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

mod error;
mod identity_validation;
pub use error::NameCatalogError;

mod name_catalog_schema {
    include!("name_catalog_schema.rs");
}
use name_catalog_schema::NameCatalogDocument;
pub use name_catalog_schema::{
    HistoricalRecordUnit, NameRegister, NameReligiousTradition, SurnameClass,
};

include!(concat!(env!("OUT_DIR"), "/name_catalog.rs"));

pub const GERMAN_WESTERN_CHRISTIAN_REPERTOIRE_ID: &str = "german_western_christian_1475_1639";

const FAMILY_STREAM: StreamId = StreamId::new("person-name.family");
const FORM_SELECTOR_STREAM: StreamId = StreamId::new("person-name.form-selector");
const FORM_STREAM: StreamId = StreamId::new("person-name.form");
const SURNAME_STREAM: StreamId = StreamId::new("person-name.surname");
const SURNAME_FORM_STREAM: StreamId = StreamId::new("person-name.surname-form");

macro_rules! name_id {
    ($name:ident) => {
        #[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

name_id!(NameFamilyId);
name_id!(NameFormId);
name_id!(SurnameId);

/// Stable selector used when choosing a culture/register-specific name form.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct NameFormSelectionSeed(u64);

impl NameFormSelectionSeed {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Stable deterministic input to name-family and form selection.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NameStableSeed(u64);

impl NameStableSeed {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

impl From<u64> for NameStableSeed {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

/// A validated personal-name projection suitable for direct display.
///
/// This type proves only that the rendered value is bounded, nonempty, and
/// free of control characters. It does not parse authored names or imply a
/// particular social role.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct RenderedPersonalName(String);

impl RenderedPersonalName {
    pub const MAX_CHARS: usize = 160;

    pub fn new(value: impl Into<String>) -> Result<Self, NameCatalogError> {
        let value = value.into();
        if value.is_empty()
            || value.trim() != value
            || value.chars().count() > Self::MAX_CHARS
            || value.chars().any(char::is_control)
        {
            return Err(NameCatalogError::InvalidRenderedName);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl std::fmt::Display for RenderedPersonalName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl AsRef<str> for RenderedPersonalName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl TryFrom<String> for RenderedPersonalName {
    type Error = NameCatalogError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for RenderedPersonalName {
    type Error = NameCatalogError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl<'de> Deserialize<'de> for RenderedPersonalName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum GivenNameResolution {
    Resolved {
        family_id: NameFamilyId,
        native_form_id: NameFormId,
    },
    UnresolvedHistorical {
        possible_family_ids: Vec<NameFamilyId>,
        recorded_form: String,
    },
    Authored {
        full_name: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PersonalNameIdentity {
    pub given: GivenNameResolution,
    pub native_culture: Culture,
    pub form_selector: NameFormSelectionSeed,
    pub surname_id: Option<SurnameId>,
}

impl PersonalNameIdentity {
    pub fn authored(full_name: impl Into<String>, native_culture: Culture) -> Self {
        Self {
            given: GivenNameResolution::Authored {
                full_name: full_name.into(),
            },
            native_culture,
            form_selector: NameFormSelectionSeed::new(0),
            surname_id: None,
        }
    }

    pub fn inherited_surname(&self) -> Option<&SurnameId> {
        self.surname_id.as_ref()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NameSocialClass {
    Commoner,
    Noble,
    Clergy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NameEducation {
    Unlettered,
    Literate,
    Scholarly,
}

/// Geographic evidence context, distinct from a character's culture.
///
/// Regions can support frequency modifiers once comparable evidence exists. They
/// do not create cultures or select a separate baseline repertoire by
/// themselves.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NameGeographicRegion {
    Unspecified,
    Mvp,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NameGenerationContext {
    pub sex: Sex,
    pub culture: Culture,
    pub religion: OfficialReligion,
    pub geographic_region: NameGeographicRegion,
    pub birth_year: CalendarYear,
    pub social_class: NameSocialClass,
    pub education: NameEducation,
}

impl NameGenerationContext {
    pub fn german_lutheran(sex: Sex, birth_year: CalendarYear) -> Self {
        Self {
            sex,
            culture: Culture::German,
            religion: OfficialReligion::Lutheran,
            geographic_region: NameGeographicRegion::Mvp,
            birth_year,
            social_class: NameSocialClass::Commoner,
            education: NameEducation::Unlettered,
        }
    }
}

impl NameReligiousTradition {
    const fn for_religion(religion: OfficialReligion) -> Self {
        match religion {
            OfficialReligion::RomanCatholic
            | OfficialReligion::Lutheran
            | OfficialReligion::Reformed
            | OfficialReligion::Anglican => Self::WesternChristian,
            OfficialReligion::EasternOrthodox => Self::EasternChristian,
            OfficialReligion::Islamic => Self::Islamic,
            OfficialReligion::Judaism => Self::Jewish,
        }
    }
}

pub fn catalog_digest() -> &'static str {
    NAME_CATALOG_DIGEST
}

pub fn generate_personal_name(
    context: NameGenerationContext,
    stable_seed: NameStableSeed,
    inherited_surname: Option<SurnameId>,
) -> Result<PersonalNameIdentity, NameCatalogError> {
    generate_personal_name_from_catalog(catalog(), context, stable_seed, inherited_surname)
}

fn generate_personal_name_from_catalog(
    catalog: &NameCatalogDocument,
    context: NameGenerationContext,
    stable_seed: NameStableSeed,
    inherited_surname: Option<SurnameId>,
) -> Result<PersonalNameIdentity, NameCatalogError> {
    let repertoire = matching_repertoire(catalog, context)?;
    let family_weights = repertoire
        .families
        .get(&context.sex)
        .ok_or(NameCatalogError::EmptyEligibleNames)?;
    let family_index = FAMILY_STREAM
        .rng(stable_seed.get(), &[])
        .weighted_index(
            &family_weights
                .iter()
                .map(|entry| entry.frequency)
                .collect::<Vec<_>>(),
        )
        .map_err(NameCatalogError::Sampling)?;
    let family_id = family_weights[family_index].family_id.as_str();
    let eligible_forms: Vec<_> = repertoire
        .everyday_forms
        .iter()
        .filter(|entry| entry.family_id == family_id)
        .collect();
    if eligible_forms.is_empty() {
        return Err(NameCatalogError::EmptyEligibleNames);
    }
    let form_selector =
        NameFormSelectionSeed::new(FORM_SELECTOR_STREAM.rng(stable_seed.get(), &[]).next_u64());
    let form_index = FORM_STREAM
        .rng(form_selector.get(), &[])
        .weighted_index(
            &eligible_forms
                .iter()
                .map(|entry| entry.frequency)
                .collect::<Vec<_>>(),
        )
        .map_err(NameCatalogError::Sampling)?;
    let surname_id = match inherited_surname {
        Some(id) => Some(id),
        None => {
            let index = SURNAME_STREAM
                .rng(stable_seed.get(), &[])
                .weighted_index(
                    &repertoire
                        .surnames
                        .iter()
                        .map(|entry| entry.frequency)
                        .collect::<Vec<_>>(),
                )
                .map_err(NameCatalogError::Sampling)?;
            Some(SurnameId::new(
                repertoire.surnames[index].surname_id.clone(),
            ))
        }
    };
    Ok(PersonalNameIdentity {
        given: GivenNameResolution::Resolved {
            family_id: NameFamilyId::new(family_id),
            native_form_id: NameFormId::new(eligible_forms[form_index].form_id.clone()),
        },
        native_culture: context.culture,
        form_selector,
        surname_id,
    })
}

pub fn render_personal_name(
    identity: &PersonalNameIdentity,
    viewer_culture: Culture,
    register: NameRegister,
    sex: Sex,
) -> Result<RenderedPersonalName, NameCatalogError> {
    identity.validate()?;
    let catalog = catalog();
    let given = match &identity.given {
        GivenNameResolution::Authored { full_name } => {
            return RenderedPersonalName::new(full_name.clone());
        }
        GivenNameResolution::UnresolvedHistorical { recorded_form, .. } => recorded_form.clone(),
        GivenNameResolution::Resolved {
            family_id,
            native_form_id,
        } => {
            if register == NameRegister::Everyday && viewer_culture == identity.native_culture {
                form_text(catalog, native_form_id.as_str())?.to_owned()
            } else {
                render_family_form(
                    catalog,
                    family_id.as_str(),
                    viewer_culture,
                    register,
                    identity.form_selector,
                )?
            }
        }
    };
    let Some(surname_id) = &identity.surname_id else {
        return RenderedPersonalName::new(given);
    };
    let surname = render_surname(
        catalog,
        surname_id.as_str(),
        viewer_culture,
        sex,
        identity.form_selector,
    )?;
    RenderedPersonalName::new(format!("{given} {surname}"))
}

fn catalog() -> &'static NameCatalogDocument {
    static CATALOG: OnceLock<NameCatalogDocument> = OnceLock::new();
    CATALOG.get_or_init(|| {
        serde_json::from_str(NAME_CATALOG_JSON).expect("build-validated name catalog parses")
    })
}

fn matching_repertoire(
    catalog: &NameCatalogDocument,
    context: NameGenerationContext,
) -> Result<&name_catalog_schema::NameRepertoireDefinition, NameCatalogError> {
    catalog
        .repertoires
        .iter()
        .find(|repertoire| {
            repertoire.culture == context.culture
                && repertoire.religious_tradition
                    == NameReligiousTradition::for_religion(context.religion)
                && (repertoire.start_year..=repertoire.end_year).contains(&context.birth_year)
        })
        .ok_or(NameCatalogError::NoMatchingRepertoire)
}

fn form_text<'a>(
    catalog: &'a NameCatalogDocument,
    form_id: &str,
) -> Result<&'a str, NameCatalogError> {
    catalog
        .given_forms
        .iter()
        .find(|form| form.id == form_id)
        .map(|form| form.text.as_str())
        .ok_or_else(|| NameCatalogError::MissingCatalogEntry(form_id.to_owned()))
}

fn render_family_form(
    catalog: &NameCatalogDocument,
    family_id: &str,
    culture: Culture,
    register: NameRegister,
    selector: NameFormSelectionSeed,
) -> Result<String, NameCatalogError> {
    let forms: Vec<_> = catalog
        .given_forms
        .iter()
        .filter(|form| {
            form.culture == culture
                && form.register == register
                && form
                    .family_ids
                    .iter()
                    .any(|candidate| candidate == family_id)
        })
        .collect();
    if forms.is_empty() {
        return Err(NameCatalogError::EmptyEligibleNames);
    }
    Ok(
        forms[FORM_STREAM.rng(selector.get(), &[]).index(forms.len())]
            .text
            .clone(),
    )
}

fn render_surname(
    catalog: &NameCatalogDocument,
    surname_id: &str,
    culture: Culture,
    sex: Sex,
    selector: NameFormSelectionSeed,
) -> Result<String, NameCatalogError> {
    let sex_specific: Vec<_> = catalog
        .surname_forms
        .iter()
        .filter(|form| {
            form.surname_id == surname_id && form.culture == culture && form.sex == Some(sex)
        })
        .collect();
    let forms = if sex_specific.is_empty() {
        catalog
            .surname_forms
            .iter()
            .filter(|form| {
                form.surname_id == surname_id && form.culture == culture && form.sex.is_none()
            })
            .collect::<Vec<_>>()
    } else {
        sex_specific
    };
    if forms.is_empty() {
        return Err(NameCatalogError::EmptyEligibleNames);
    }
    Ok(forms[SURNAME_FORM_STREAM
        .rng(selector.get(), &[])
        .index(forms.len())]
    .text
    .clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod validation {
        include!("name_catalog_validation.rs");
    }

    #[test]
    fn compiled_catalog_is_valid_and_source_backed() {
        validation::validate(catalog()).unwrap();
        assert!(!catalog_digest().is_empty());
    }

    #[test]
    fn repertoire_fragments_extend_the_matching_sex_group() {
        let repertoire = catalog()
            .repertoires
            .iter()
            .find(|entry| entry.id == GERMAN_WESTERN_CHRISTIAN_REPERTOIRE_ID)
            .unwrap();
        let female = &repertoire.families[&Sex::Female];
        let male = &repertoire.families[&Sex::Male];
        assert_eq!(female[0].family_id, "anna");
        assert_eq!(male[0].family_id, "johannes");
        assert!(female.iter().any(|entry| entry.family_id == "aulia"));
        assert!(male.iter().any(|entry| entry.family_id == "gallus"));
        assert!(!female.iter().any(|entry| entry.family_id == "gallus"));
        assert!(!male.iter().any(|entry| entry.family_id == "aulia"));
    }

    #[test]
    fn generated_name_has_resolved_family_and_stable_rendering() {
        let context =
            NameGenerationContext::german_lutheran(Sex::Male, CalendarYear::new(1522).unwrap());
        let identity = generate_personal_name(context, NameStableSeed::new(42), None).unwrap();
        let first = render_personal_name(
            &identity,
            Culture::German,
            NameRegister::Everyday,
            Sex::Male,
        )
        .unwrap();
        assert_eq!(
            first,
            render_personal_name(
                &identity,
                Culture::German,
                NameRegister::Everyday,
                Sex::Male,
            )
            .unwrap()
        );
        assert!(first.as_str().contains(' '));
    }

    #[test]
    fn hans_and_johannes_share_one_family_while_hans_dominates_everyday_use() {
        let hans = catalog()
            .given_forms
            .iter()
            .find(|form| form.id == "johannes_hans_de")
            .unwrap();
        let johannes = catalog()
            .given_forms
            .iter()
            .find(|form| form.id == "johannes_full_de")
            .unwrap();
        assert_eq!(hans.family_ids, johannes.family_ids);
        let repertoire = catalog()
            .repertoires
            .iter()
            .find(|entry| entry.id == GERMAN_WESTERN_CHRISTIAN_REPERTOIRE_ID)
            .unwrap();
        let hans_frequency = repertoire
            .everyday_forms
            .iter()
            .find(|entry| entry.form_id == hans.id)
            .unwrap()
            .frequency;
        let other_johannes_frequencies: u64 = repertoire
            .everyday_forms
            .iter()
            .filter(|entry| entry.family_id == "johannes" && entry.form_id != hans.id)
            .map(|entry| entry.frequency)
            .sum();
        assert!(hans_frequency > other_johannes_frequencies);
    }

    #[test]
    fn henne_can_remain_historically_unresolved() {
        let henne = catalog()
            .given_forms
            .iter()
            .find(|form| form.id == "henne_de")
            .unwrap();
        assert_eq!(henne.family_ids, ["johannes", "heinrich"]);
        let identity = PersonalNameIdentity {
            given: GivenNameResolution::UnresolvedHistorical {
                possible_family_ids: henne
                    .family_ids
                    .iter()
                    .cloned()
                    .map(NameFamilyId::new)
                    .collect(),
                recorded_form: henne.text.clone(),
            },
            native_culture: Culture::German,
            form_selector: NameFormSelectionSeed::new(7),
            surname_id: None,
        };
        assert_eq!(
            render_personal_name(
                &identity,
                Culture::German,
                NameRegister::Everyday,
                Sex::Male,
            )
            .unwrap()
            .as_str(),
            "Henne"
        );
    }

    #[test]
    fn authored_names_are_not_parsed() {
        let identity = PersonalNameIdentity::authored("Theophrastus Bombast", Culture::German);
        assert_eq!(
            render_personal_name(
                &identity,
                Culture::German,
                NameRegister::Everyday,
                Sex::Male,
            )
            .unwrap()
            .as_str(),
            "Theophrastus Bombast"
        );
    }

    #[test]
    fn rendered_names_reject_unvalidated_wire_and_authored_text() {
        for invalid in ["", " Hans", "Hans ", "Hans\nBecker"] {
            assert!(RenderedPersonalName::new(invalid).is_err());
            assert!(
                serde_json::from_str::<RenderedPersonalName>(
                    &serde_json::to_string(invalid).unwrap()
                )
                .is_err()
            );
        }
        let authored = PersonalNameIdentity::authored("Hans\nBecker", Culture::German);
        assert!(
            render_personal_name(
                &authored,
                Culture::German,
                NameRegister::Everyday,
                Sex::Male,
            )
            .is_err()
        );
    }

    #[test]
    fn culture_and_period_select_the_baseline_without_invented_modifiers() {
        let lutheran =
            NameGenerationContext::german_lutheran(Sex::Male, CalendarYear::new(1520).unwrap());
        let mut catholic_elsewhere = lutheran;
        catholic_elsewhere.religion = OfficialReligion::RomanCatholic;
        catholic_elsewhere.geographic_region = NameGeographicRegion::Unspecified;

        assert_eq!(
            generate_personal_name(lutheran, NameStableSeed::new(91), None).unwrap(),
            generate_personal_name(catholic_elsewhere, NameStableSeed::new(91), None).unwrap()
        );

        let mut jewish = lutheran;
        jewish.religion = OfficialReligion::Judaism;
        assert_eq!(
            generate_personal_name(jewish, NameStableSeed::new(91), None),
            Err(NameCatalogError::NoMatchingRepertoire)
        );
    }

    #[test]
    fn register_rendering_does_not_mutate_the_native_everyday_projection() {
        let identity = PersonalNameIdentity {
            given: GivenNameResolution::Resolved {
                family_id: NameFamilyId::new("johannes"),
                native_form_id: NameFormId::new("johannes_hans_de"),
            },
            native_culture: Culture::German,
            form_selector: NameFormSelectionSeed::new(31),
            surname_id: Some(SurnameId::new("becker")),
        };
        let everyday = render_personal_name(
            &identity,
            Culture::German,
            NameRegister::Everyday,
            Sex::Male,
        )
        .unwrap();
        assert_eq!(everyday.as_str(), "Hans Becker");
        assert_eq!(
            render_personal_name(
                &identity,
                Culture::German,
                NameRegister::Documentary,
                Sex::Male,
            )
            .unwrap()
            .as_str(),
            "Johannes Becker"
        );
        assert_eq!(
            render_personal_name(
                &identity,
                Culture::German,
                NameRegister::Everyday,
                Sex::Male,
            )
            .unwrap(),
            everyday
        );
    }

    #[test]
    fn adding_a_form_does_not_change_family_prevalence() {
        let mut expanded = catalog().clone();
        let mut extra_form = expanded
            .given_forms
            .iter()
            .find(|form| form.id == "johannes_johann_de")
            .unwrap()
            .clone();
        extra_form.id = "johannes_test_variant_de".into();
        extra_form.text = "Testvariant".into();
        expanded.given_forms.push(extra_form);
        let repertoire = expanded
            .repertoires
            .iter_mut()
            .find(|entry| entry.id == GERMAN_WESTERN_CHRISTIAN_REPERTOIRE_ID)
            .unwrap();
        let mut extra_frequency = repertoire
            .everyday_forms
            .iter()
            .find(|entry| entry.form_id == "johannes_johann_de")
            .unwrap()
            .clone();
        extra_frequency.form_id = "johannes_test_variant_de".into();
        repertoire.everyday_forms.push(extra_frequency);

        let context =
            NameGenerationContext::german_lutheran(Sex::Male, CalendarYear::new(1520).unwrap());
        for seed in 0..256 {
            let original = generate_personal_name_from_catalog(
                catalog(),
                context,
                NameStableSeed::new(seed),
                None,
            )
            .unwrap();
            let variant = generate_personal_name_from_catalog(
                &expanded,
                context,
                NameStableSeed::new(seed),
                None,
            )
            .unwrap();
            let family = |identity: PersonalNameIdentity| match identity.given {
                GivenNameResolution::Resolved { family_id, .. } => family_id,
                _ => panic!("generated identities must resolve one family"),
            };
            assert_eq!(family(original), family(variant));
        }
    }

    #[test]
    fn register_specific_forms_render_from_one_family() {
        let synthetic: NameCatalogDocument = serde_json::from_value(serde_json::json!({
            "given_forms": [
                {
                    "id": "heinrich_de",
                    "text": "Heinrich",
                    "culture": "german",
                    "register": "everyday",
                    "family_ids": ["heinrich"]
                },
                {
                    "id": "heinricus_doc",
                    "text": "Heinricus",
                    "culture": "german",
                    "register": "documentary",
                    "family_ids": ["heinrich"]
                }
            ]
        }))
        .unwrap();
        assert_eq!(
            render_family_form(
                &synthetic,
                "heinrich",
                Culture::German,
                NameRegister::Everyday,
                NameFormSelectionSeed::new(11),
            )
            .unwrap(),
            "Heinrich"
        );
        assert_eq!(
            render_family_form(
                &synthetic,
                "heinrich",
                Culture::German,
                NameRegister::Documentary,
                NameFormSelectionSeed::new(11),
            )
            .unwrap(),
            "Heinricus"
        );
    }

    #[test]
    fn attested_feminine_surname_form_is_selected_without_a_suffix_rule() {
        let identity = PersonalNameIdentity {
            given: GivenNameResolution::Resolved {
                family_id: NameFamilyId::new("anna"),
                native_form_id: NameFormId::new("anna_de"),
            },
            native_culture: Culture::German,
            form_selector: NameFormSelectionSeed::new(17),
            surname_id: Some(SurnameId::new("pfeiffer")),
        };
        assert!(
            render_personal_name(
                &identity,
                Culture::German,
                NameRegister::Everyday,
                Sex::Female,
            )
            .unwrap()
            .as_str()
            .ends_with(" Pfeifferin")
        );
        assert!(
            render_personal_name(
                &identity,
                Culture::German,
                NameRegister::Everyday,
                Sex::Male,
            )
            .unwrap()
            .as_str()
            .ends_with(" Pfeiffer")
        );
    }
}

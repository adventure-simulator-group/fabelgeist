// Serialized schema for the externally authored personal-name catalog.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NameCatalogDocument {
    #[serde(default)]
    pub sources: Vec<NameSourceDefinition>,
    #[serde(default)]
    pub given_families: Vec<GivenNameFamilyDefinition>,
    #[serde(default)]
    pub given_forms: Vec<GivenNameFormDefinition>,
    #[serde(default)]
    pub surnames: Vec<SurnameDefinition>,
    #[serde(default)]
    pub surname_forms: Vec<SurnameFormDefinition>,
    #[serde(default)]
    pub observations: Vec<NameObservationDefinition>,
    #[serde(default)]
    pub derivations: Vec<NameDerivationDefinition>,
    #[serde(default)]
    pub repertoires: Vec<NameRepertoireDefinition>,
    #[serde(default)]
    pub repertoire_fragments: Vec<NameRepertoireFragmentDefinition>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HistoricalRecordUnit {
    People,
    Mentions,
    Baptisms,
    CitizenAdmissions,
    Inscriptions,
    TaxEntries,
    PropertyEntries,
    ParishEntries,
    ScholarlyInterpretation,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NameSourceDefinition {
    pub id: String,
    pub title: String,
    pub url: String,
    pub date_range: String,
    pub geography: String,
    pub record_unit: HistoricalRecordUnit,
    pub sample_size: Option<u32>,
    pub quantitative: bool,
    pub population_bias: String,
    pub notes: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NameSex {
    Female,
    Male,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NameCulture {
    German,
    English,
    Italian,
    Elven,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NameReligiousTradition {
    WesternChristian,
    EasternChristian,
    Islamic,
    Jewish,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NameRegister {
    Everyday,
    Documentary,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SurnameClass {
    HereditaryCommoner,
    NobleDynasty,
    FarmOrHousehold,
    UnresolvedByname,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NameObservationDefinition {
    pub id: String,
    pub source_id: String,
    pub locator: String,
    pub recorded_spelling: String,
    pub observed_count: Option<u32>,
    pub derivation: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NameDerivationDefinition {
    pub id: String,
    pub repertoire_id: String,
    #[serde(default)]
    pub family_id: Option<String>,
    #[serde(default)]
    pub form_id: Option<String>,
    #[serde(default)]
    pub surname_id: Option<String>,
    pub frequency: u64,
    pub observation_ids: Vec<String>,
    pub operation: String,
    pub rationale: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GivenNameFamilyDefinition {
    pub id: String,
    pub sex: NameSex,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GivenNameFormDefinition {
    pub id: String,
    pub text: String,
    pub culture: NameCulture,
    pub register: NameRegister,
    pub family_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SurnameDefinition {
    pub id: String,
    pub class: SurnameClass,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SurnameFormDefinition {
    pub id: String,
    pub surname_id: String,
    pub text: String,
    pub culture: NameCulture,
    pub sex: Option<NameSex>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FrequencyFamilyDefinition {
    pub family_id: String,
    /// Relative sampling score, deliberately separate from source counts.
    pub frequency: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FrequencyFormDefinition {
    pub family_id: String,
    pub form_id: String,
    /// Relative within-family sampling score, not an observed count.
    pub frequency: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FrequencySurnameDefinition {
    pub surname_id: String,
    /// Relative sampling score, not a pooled census frequency.
    pub frequency: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NameRepertoireDefinition {
    pub id: String,
    pub culture: NameCulture,
    pub religious_tradition: NameReligiousTradition,
    pub start_year: i32,
    pub end_year: i32,
    pub female_families: Vec<FrequencyFamilyDefinition>,
    pub male_families: Vec<FrequencyFamilyDefinition>,
    pub everyday_forms: Vec<FrequencyFormDefinition>,
    pub surnames: Vec<FrequencySurnameDefinition>,
}

/// Evidence files may contribute independently reviewed slices to one
/// repertoire. The build script resolves these into the target before runtime
/// embedding, so selection still sees one unambiguous production repertoire.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NameRepertoireFragmentDefinition {
    pub repertoire_id: String,
    #[serde(default)]
    pub male_families: Vec<FrequencyFamilyDefinition>,
    #[serde(default)]
    pub female_families: Vec<FrequencyFamilyDefinition>,
    #[serde(default)]
    pub everyday_forms: Vec<FrequencyFormDefinition>,
    #[serde(default)]
    pub surnames: Vec<FrequencySurnameDefinition>,
}

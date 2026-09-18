// Build-time validation for the externally authored name catalog.

use super::name_catalog_schema::{
    GivenNameFamilyDefinition, GivenNameFormDefinition, NameCatalogDocument, NameCulture,
    NameObservationDefinition, NameRegister, NameReligiousTradition, NameRepertoireDefinition,
    NameSex, NameSourceDefinition, SurnameClass, SurnameDefinition, SurnameFormDefinition,
};
use std::collections::{BTreeMap, BTreeSet};

#[path = "name_catalog_validation/derivations.rs"]
mod derivations;

const SECURE_WEB_SCHEME: &str = "https://";

type FamilyIndex<'a> = BTreeMap<&'a str, NameSex>;
type FormIndex<'a> = BTreeMap<&'a str, &'a GivenNameFormDefinition>;
type SurnameIndex<'a> = BTreeMap<&'a str, SurnameClass>;
type SurnameFormIndex<'a> = BTreeMap<&'a str, Vec<&'a SurnameFormDefinition>>;

pub fn validate(catalog: &NameCatalogDocument) -> Result<(), String> {
    if !catalog.repertoire_fragments.is_empty() {
        return Err("repertoire fragments must be resolved before validation".into());
    }
    if catalog.sources.is_empty()
        || catalog.given_families.is_empty()
        || catalog.given_forms.is_empty()
        || catalog.surnames.is_empty()
        || catalog.repertoires.is_empty()
    {
        return Err("sources, given names, surnames, and repertoires must be nonempty".into());
    }

    let source_ids = validate_sources(&catalog.sources)?;
    let observation_ids = validate_observations(&catalog.observations, &source_ids)?;
    let families = validate_families(&catalog.given_families)?;
    let forms = validate_forms(&catalog.given_forms, &families)?;
    let (surnames, surname_forms) = validate_surnames(&catalog.surnames, &catalog.surname_forms)?;
    validate_repertoires(
        &catalog.repertoires,
        &families,
        &forms,
        &surnames,
        &surname_forms,
    )?;
    derivations::validate(
        &catalog.derivations,
        &catalog.repertoires,
        &observation_ids,
        &families,
        &forms,
        &surnames,
    )
}

fn validate_sources(sources: &[NameSourceDefinition]) -> Result<BTreeSet<&str>, String> {
    let mut source_ids = BTreeSet::new();
    for source in sources {
        validate_id("source", &source.id)?;
        if !source_ids.insert(source.id.as_str()) {
            return Err(format!("duplicate source ID {}", source.id));
        }
        if source.title.trim().is_empty()
            || source.date_range.trim().is_empty()
            || source.geography.trim().is_empty()
            || source.population_bias.trim().is_empty()
            || source.notes.trim().is_empty()
            || !source.url.starts_with(SECURE_WEB_SCHEME)
        {
            return Err(format!("source {} has incomplete provenance", source.id));
        }
        if source.quantitative && source.sample_size == Some(0) {
            return Err(format!("source {} has a zero sample size", source.id));
        }
    }
    Ok(source_ids)
}

fn validate_families<'a>(
    definitions: &'a [GivenNameFamilyDefinition],
) -> Result<FamilyIndex<'a>, String> {
    let mut families = BTreeMap::new();
    for family in definitions {
        validate_id("given-name family", &family.id)?;
        if families.insert(family.id.as_str(), family.sex).is_some() {
            return Err(format!("duplicate given-name family ID {}", family.id));
        }
    }
    Ok(families)
}

fn validate_forms<'a>(
    definitions: &'a [GivenNameFormDefinition],
    families: &FamilyIndex<'_>,
) -> Result<FormIndex<'a>, String> {
    let mut forms = BTreeMap::new();
    for form in definitions {
        validate_id("given-name form", &form.id)?;
        if form.text.trim().is_empty() || form.family_ids.is_empty() {
            return Err(format!("given-name form {} is incomplete", form.id));
        }
        let mut linked_sex = None;
        let mut linked_ids = BTreeSet::new();
        for family_id in &form.family_ids {
            if !linked_ids.insert(family_id) {
                return Err(format!(
                    "given-name form {} repeats family {}",
                    form.id, family_id
                ));
            }
            let sex = families.get(family_id.as_str()).ok_or_else(|| {
                format!(
                    "given-name form {} references unknown family {}",
                    form.id, family_id
                )
            })?;
            if linked_sex.replace(*sex).is_some_and(|known| known != *sex) {
                return Err(format!(
                    "given-name form {} links families with incompatible sexes",
                    form.id
                ));
            }
        }
        if forms.insert(form.id.as_str(), form).is_some() {
            return Err(format!("duplicate given-name form ID {}", form.id));
        }
    }
    Ok(forms)
}

fn validate_surnames<'a>(
    definitions: &'a [SurnameDefinition],
    form_definitions: &'a [SurnameFormDefinition],
) -> Result<(SurnameIndex<'a>, SurnameFormIndex<'a>), String> {
    let mut surnames = BTreeMap::new();
    for surname in definitions {
        validate_id("surname", &surname.id)?;
        if surnames
            .insert(surname.id.as_str(), surname.class)
            .is_some()
        {
            return Err(format!("duplicate surname ID {}", surname.id));
        }
    }
    let mut surname_forms = BTreeSet::new();
    let mut surname_forms_by_identity = BTreeMap::<&str, Vec<_>>::new();
    let mut surnames_with_forms = BTreeSet::new();
    for form in form_definitions {
        validate_id("surname form", &form.id)?;
        if form.text.trim().is_empty() || !surnames.contains_key(form.surname_id.as_str()) {
            return Err(format!(
                "surname form {} is incomplete or dangling",
                form.id
            ));
        }
        if !surname_forms.insert(form.id.as_str()) {
            return Err(format!("duplicate surname form ID {}", form.id));
        }
        surnames_with_forms.insert(form.surname_id.as_str());
        surname_forms_by_identity
            .entry(form.surname_id.as_str())
            .or_default()
            .push(form);
    }
    for surname_id in surnames.keys() {
        if !surnames_with_forms.contains(surname_id) {
            return Err(format!("surname {surname_id} has no display form"));
        }
    }
    Ok((surnames, surname_forms_by_identity))
}

fn validate_repertoires(
    repertoires: &[NameRepertoireDefinition],
    families: &FamilyIndex<'_>,
    forms: &FormIndex<'_>,
    surnames: &SurnameIndex<'_>,
    surname_forms: &SurnameFormIndex<'_>,
) -> Result<(), String> {
    let mut repertoire_ids = BTreeSet::new();
    let mut selectors = Vec::new();
    for repertoire in repertoires {
        validate_id("repertoire", &repertoire.id)?;
        if !repertoire_ids.insert(repertoire.id.as_str()) {
            return Err(format!("duplicate repertoire ID {}", repertoire.id));
        }
        if repertoire.start_year > repertoire.end_year {
            return Err(format!(
                "repertoire {} has an invalid selector",
                repertoire.id
            ));
        }
        if selectors.iter().any(|selector| {
            repertoire_selectors_overlap(
                *selector,
                (
                    repertoire.culture,
                    repertoire.religious_tradition,
                    repertoire.start_year,
                    repertoire.end_year,
                ),
            )
        }) {
            return Err(format!(
                "ambiguous repertoire selector for {}",
                repertoire.id
            ));
        }
        selectors.push((
            repertoire.culture,
            repertoire.religious_tradition,
            repertoire.start_year,
            repertoire.end_year,
        ));
        validate_repertoire(repertoire, families, forms, surnames, surname_forms)?;
    }
    Ok(())
}

fn repertoire_selectors_overlap(
    left: (NameCulture, NameReligiousTradition, i32, i32),
    right: (NameCulture, NameReligiousTradition, i32, i32),
) -> bool {
    left.0 == right.0 && left.1 == right.1 && left.2 <= right.3 && right.2 <= left.3
}

fn validate_repertoire(
    repertoire: &NameRepertoireDefinition,
    families: &FamilyIndex<'_>,
    forms: &FormIndex<'_>,
    surnames: &SurnameIndex<'_>,
    surname_forms: &SurnameFormIndex<'_>,
) -> Result<(), String> {
    let eligible_families = validate_family_frequencies(repertoire, families)?;
    validate_form_frequencies(repertoire, forms, &eligible_families)?;
    validate_surname_frequencies(repertoire, surnames, surname_forms)
}

fn validate_family_frequencies<'a>(
    repertoire: &'a NameRepertoireDefinition,
    families: &FamilyIndex<'_>,
) -> Result<BTreeSet<&'a str>, String> {
    let mut eligible_families = BTreeSet::new();
    for (sex, frequencies) in [
        (NameSex::Female, &repertoire.female_families),
        (NameSex::Male, &repertoire.male_families),
    ] {
        if frequencies.is_empty() {
            return Err(format!(
                "repertoire {} has no {:?} families",
                repertoire.id, sex
            ));
        }
        for frequency in frequencies {
            if frequency.frequency == 0 || !eligible_families.insert(frequency.family_id.as_str()) {
                return Err(format!(
                    "repertoire {} has a zero or duplicate family frequency for {}",
                    repertoire.id, frequency.family_id
                ));
            }
            if families.get(frequency.family_id.as_str()) != Some(&sex) {
                return Err(format!(
                    "repertoire {} frequencies {} under the wrong sex",
                    repertoire.id, frequency.family_id
                ));
            }
        }
    }
    Ok(eligible_families)
}

fn validate_form_frequencies(
    repertoire: &NameRepertoireDefinition,
    forms: &FormIndex<'_>,
    eligible_families: &BTreeSet<&str>,
) -> Result<(), String> {
    let mut eligible_forms = BTreeSet::new();
    let mut families_with_forms = BTreeSet::new();
    let mut form_totals = BTreeMap::<&str, u64>::new();
    for frequency in &repertoire.everyday_forms {
        if frequency.frequency == 0
            || !eligible_forms.insert((frequency.family_id.as_str(), frequency.form_id.as_str()))
        {
            return Err(format!(
                "repertoire {} has a zero or duplicate form frequency for {}/{}",
                repertoire.id, frequency.family_id, frequency.form_id
            ));
        }
        if !eligible_families.contains(frequency.family_id.as_str()) {
            return Err(format!(
                "repertoire {} lists a form frequency for ineligible family {}",
                repertoire.id, frequency.family_id
            ));
        }
        let form = forms.get(frequency.form_id.as_str()).ok_or_else(|| {
            format!(
                "repertoire {} references unknown form {}",
                repertoire.id, frequency.form_id
            )
        })?;
        if form.register != NameRegister::Everyday
            || form.culture != repertoire.culture
            || !form.family_ids.iter().any(|id| id == &frequency.family_id)
        {
            return Err(format!(
                "repertoire {} form {} is incompatible with family, culture, or register",
                repertoire.id, frequency.form_id
            ));
        }
        families_with_forms.insert(frequency.family_id.as_str());
        let total = form_totals.entry(frequency.family_id.as_str()).or_default();
        *total = total
            .checked_add(frequency.frequency)
            .ok_or_else(|| format!("form frequencies overflow for {}", frequency.family_id))?;
    }
    for family_id in eligible_families.iter().copied() {
        if !families_with_forms.contains(family_id) {
            return Err(format!(
                "repertoire {} family {} has no everyday form",
                repertoire.id, family_id
            ));
        }
    }
    Ok(())
}

fn validate_surname_frequencies(
    repertoire: &NameRepertoireDefinition,
    surnames: &SurnameIndex<'_>,
    surname_forms: &SurnameFormIndex<'_>,
) -> Result<(), String> {
    if repertoire.surnames.is_empty() {
        return Err(format!(
            "repertoire {} has no surname frequency",
            repertoire.id
        ));
    }
    let mut eligible_surnames = BTreeSet::new();
    for frequency in &repertoire.surnames {
        if frequency.frequency == 0 || !eligible_surnames.insert(frequency.surname_id.as_str()) {
            return Err(format!(
                "repertoire {} has a zero or duplicate surname frequency for {}",
                repertoire.id, frequency.surname_id
            ));
        }
        if surnames.get(frequency.surname_id.as_str()) != Some(&SurnameClass::HereditaryCommoner) {
            return Err(format!(
                "repertoire {} may generate only hereditary commoner surnames: {}",
                repertoire.id, frequency.surname_id
            ));
        }
        let compatible_forms = surname_forms
            .get(frequency.surname_id.as_str())
            .into_iter()
            .flatten()
            .filter(|form| form.culture == repertoire.culture)
            .collect::<Vec<_>>();
        for sex in [NameSex::Female, NameSex::Male] {
            if !compatible_forms
                .iter()
                .any(|form| form.sex.is_none() || form.sex == Some(sex))
            {
                return Err(format!(
                    "repertoire {} surname {} has no compatible {:?} form",
                    repertoire.id, frequency.surname_id, sex
                ));
            }
        }
    }
    Ok(())
}

fn validate_observations<'a>(
    observations: &'a [NameObservationDefinition],
    sources: &BTreeSet<&'a str>,
) -> Result<BTreeSet<&'a str>, String> {
    let mut observation_ids = BTreeSet::new();
    for record in observations {
        validate_id("observation", &record.id)?;
        if !observation_ids.insert(record.id.as_str()) {
            return Err(format!("duplicate observation ID {}", record.id));
        }
        if !sources.contains(record.source_id.as_str())
            || record.locator.trim().is_empty()
            || record.derivation.trim().is_empty()
            || record.recorded_spelling.trim().is_empty()
        {
            return Err(format!(
                "observation {} is incomplete or dangling",
                record.id
            ));
        }
    }
    Ok(observation_ids)
}

fn validate_id(kind: &str, id: &str) -> Result<(), String> {
    if id.is_empty()
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(format!("{kind} ID {id:?} must be lowercase snake_case"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::name_catalog_schema::NameDerivationDefinition;
    use super::*;

    #[test]
    fn stable_ids_reject_human_facing_spellings() {
        assert!(validate_id("fixture", "johannes_family").is_ok());
        assert!(validate_id("fixture", "Johannes").is_err());
        assert!(validate_id("fixture", "johannes-family").is_err());
    }

    #[test]
    fn overlapping_selectors_are_ambiguous_but_adjacent_periods_are_not() {
        let first = (
            NameCulture::German,
            NameReligiousTradition::WesternChristian,
            1500,
            1550,
        );
        assert!(repertoire_selectors_overlap(
            first,
            (
                NameCulture::German,
                NameReligiousTradition::WesternChristian,
                1550,
                1600
            )
        ));
        assert!(!repertoire_selectors_overlap(
            first,
            (
                NameCulture::German,
                NameReligiousTradition::WesternChristian,
                1551,
                1600
            )
        ));
        assert!(!repertoire_selectors_overlap(
            first,
            (
                NameCulture::English,
                NameReligiousTradition::WesternChristian,
                1500,
                1550
            )
        ));
        assert!(!repertoire_selectors_overlap(
            first,
            (
                NameCulture::German,
                NameReligiousTradition::Jewish,
                1500,
                1550
            )
        ));
    }

    #[test]
    fn derivation_frequency_must_match_its_repertoire_entry() {
        let repertoire = NameRepertoireDefinition {
            id: "fixture_repertoire".into(),
            culture: NameCulture::German,
            religious_tradition: NameReligiousTradition::WesternChristian,
            start_year: 1500,
            end_year: 1600,
            female_families: vec![],
            male_families: vec![
                super::super::name_catalog_schema::FrequencyFamilyDefinition {
                    family_id: "fixture_family".into(),
                    frequency: 4,
                },
            ],
            everyday_forms: vec![],
            surnames: vec![],
        };
        let mut families = BTreeMap::new();
        families.insert("fixture_family", NameSex::Male);
        let derivation = NameDerivationDefinition {
            id: "fixture_derivation".into(),
            repertoire_id: "fixture_repertoire".into(),
            family_id: Some("fixture_family".into()),
            form_id: None,
            surname_id: None,
            frequency: 3,
            observation_ids: vec!["fixture_observation".into()],
            operation: "fixture".into(),
            rationale: "fixture".into(),
        };
        let observations = BTreeSet::from(["fixture_observation"]);
        let forms = BTreeMap::new();
        let surnames = BTreeMap::new();
        let error = derivations::validate(
            &[derivation],
            &[repertoire],
            &observations,
            &families,
            &forms,
            &surnames,
        )
        .unwrap_err();
        assert!(error.contains("frequency 3 disagrees"));
    }
}

use super::super::name_catalog_schema::{NameDerivationDefinition, NameRepertoireDefinition};
use super::{FamilyIndex, FormIndex, SurnameIndex};
use std::collections::{BTreeMap, BTreeSet};

type TargetKey = (String, Option<String>, Option<String>, Option<String>);

pub(super) fn validate(
    derivations: &[NameDerivationDefinition],
    repertoires: &[NameRepertoireDefinition],
    observation_ids: &BTreeSet<&str>,
    families: &FamilyIndex<'_>,
    forms: &FormIndex<'_>,
    surnames: &SurnameIndex<'_>,
) -> Result<(), String> {
    let targets = repertoire_targets(repertoires);
    let repertoire_ids = repertoires
        .iter()
        .map(|repertoire| repertoire.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut derivation_ids = BTreeSet::<String>::new();
    let mut covered_targets = BTreeSet::new();
    for derivation in derivations {
        validate_entry(
            derivation,
            &repertoire_ids,
            observation_ids,
            families,
            forms,
            surnames,
            &targets,
            &mut derivation_ids,
            &mut covered_targets,
        )?;
    }
    if covered_targets.len() != targets.len() {
        return Err(format!(
            "{} eligible repertoire entries lack provenance derivations",
            targets.len() - covered_targets.len()
        ));
    }
    Ok(())
}

fn repertoire_targets(repertoires: &[NameRepertoireDefinition]) -> BTreeMap<TargetKey, u64> {
    let mut targets = BTreeMap::new();
    for repertoire in repertoires {
        for entry in repertoire
            .female_families
            .iter()
            .chain(repertoire.male_families.iter())
        {
            targets.insert(
                (
                    repertoire.id.clone(),
                    Some(entry.family_id.clone()),
                    None,
                    None,
                ),
                entry.frequency,
            );
        }
        for entry in &repertoire.everyday_forms {
            targets.insert(
                (
                    repertoire.id.clone(),
                    Some(entry.family_id.clone()),
                    Some(entry.form_id.clone()),
                    None,
                ),
                entry.frequency,
            );
        }
        for entry in &repertoire.surnames {
            targets.insert(
                (
                    repertoire.id.clone(),
                    None,
                    None,
                    Some(entry.surname_id.clone()),
                ),
                entry.frequency,
            );
        }
    }
    targets
}

#[expect(
    clippy::too_many_arguments,
    reason = "validation keeps each catalog index explicit"
)]
fn validate_entry(
    derivation: &NameDerivationDefinition,
    repertoire_ids: &BTreeSet<&str>,
    observation_ids: &BTreeSet<&str>,
    families: &FamilyIndex<'_>,
    forms: &FormIndex<'_>,
    surnames: &SurnameIndex<'_>,
    targets: &BTreeMap<TargetKey, u64>,
    derivation_ids: &mut BTreeSet<String>,
    covered_targets: &mut BTreeSet<TargetKey>,
) -> Result<(), String> {
    super::validate_id("derivation", &derivation.id)?;
    if !derivation_ids.insert(derivation.id.clone()) {
        return Err(format!("duplicate derivation ID {}", derivation.id));
    }
    if derivation.operation.trim().is_empty()
        || derivation.rationale.trim().is_empty()
        || derivation.observation_ids.is_empty()
        || derivation.frequency == 0
    {
        return Err(format!("derivation {} is incomplete", derivation.id));
    }
    if !repertoire_ids.contains(derivation.repertoire_id.as_str()) {
        return Err(format!(
            "derivation {} references unknown repertoire {}",
            derivation.id, derivation.repertoire_id
        ));
    }
    validate_target_links(derivation, families, forms, surnames)?;
    let target = (
        derivation.repertoire_id.clone(),
        derivation.family_id.clone(),
        derivation.form_id.clone(),
        derivation.surname_id.clone(),
    );
    if targets.get(&target) != Some(&derivation.frequency) {
        return Err(format!(
            "derivation {} frequency {} disagrees with its repertoire target",
            derivation.id, derivation.frequency
        ));
    }
    if !covered_targets.insert(target) {
        return Err(format!("duplicate derivation target in {}", derivation.id));
    }
    validate_observation_links(derivation, observation_ids)
}

fn validate_target_links(
    derivation: &NameDerivationDefinition,
    families: &FamilyIndex<'_>,
    forms: &FormIndex<'_>,
    surnames: &SurnameIndex<'_>,
) -> Result<(), String> {
    let target_count = derivation.family_id.is_some() as u8
        + derivation.form_id.is_some() as u8
        + derivation.surname_id.is_some() as u8;
    let valid_shape = (derivation.family_id.is_some()
        && derivation.form_id.is_none()
        && derivation.surname_id.is_none())
        || (derivation.family_id.is_some()
            && derivation.form_id.is_some()
            && derivation.surname_id.is_none())
        || (derivation.family_id.is_none()
            && derivation.form_id.is_none()
            && derivation.surname_id.is_some());
    if target_count == 0 || !valid_shape {
        return Err(format!(
            "derivation {} has an invalid target",
            derivation.id
        ));
    }
    if let Some(family_id) = &derivation.family_id
        && !families.contains_key(family_id.as_str())
    {
        return Err(format!(
            "derivation {} references unknown family {}",
            derivation.id, family_id
        ));
    }
    if let Some(form_id) = &derivation.form_id {
        let form = forms.get(form_id.as_str()).ok_or_else(|| {
            format!(
                "derivation {} references unknown form {}",
                derivation.id, form_id
            )
        })?;
        if !form
            .family_ids
            .iter()
            .any(|family_id| Some(family_id) == derivation.family_id.as_ref())
        {
            return Err(format!(
                "derivation {} targets a form outside its family",
                derivation.id
            ));
        }
    }
    if let Some(surname_id) = &derivation.surname_id
        && !surnames.contains_key(surname_id.as_str())
    {
        return Err(format!(
            "derivation {} references unknown surname {}",
            derivation.id, surname_id
        ));
    }
    Ok(())
}

fn validate_observation_links(
    derivation: &NameDerivationDefinition,
    observation_ids: &BTreeSet<&str>,
) -> Result<(), String> {
    let mut references = BTreeSet::new();
    for observation_id in &derivation.observation_ids {
        if !references.insert(observation_id.as_str())
            || !observation_ids.contains(observation_id.as_str())
        {
            return Err(format!(
                "derivation {} has a duplicate or dangling observation ID {}",
                derivation.id, observation_id
            ));
        }
    }
    Ok(())
}

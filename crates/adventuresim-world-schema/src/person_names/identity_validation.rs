//! Catalog integrity of semantic identities, independent of display register.

use super::*;

impl PersonalNameIdentity {
    /// Validate every stored reference, including metadata an authored display
    /// does not render. A native form captures a spelling within its semantic
    /// family; neither reference may independently select a different identity.
    pub fn validate(&self) -> Result<(), NameCatalogError> {
        let catalog = catalog();
        let family_exists = |id: &NameFamilyId| {
            catalog
                .given_families
                .iter()
                .any(|family| family.id == id.as_str())
                .then_some(())
                .ok_or_else(|| NameCatalogError::MissingCatalogEntry(id.as_str().to_owned()))
        };
        match &self.given {
            GivenNameResolution::Resolved {
                family_id,
                native_form_id,
            } => {
                family_exists(family_id)?;
                let form = catalog
                    .given_forms
                    .iter()
                    .find(|form| form.id == native_form_id.as_str())
                    .ok_or_else(|| {
                        NameCatalogError::MissingCatalogEntry(native_form_id.as_str().to_owned())
                    })?;
                if !form.family_ids.iter().any(|id| id == family_id.as_str())
                    || form.culture != self.native_culture
                    || form.register != NameRegister::Everyday
                {
                    return Err(NameCatalogError::InconsistentNativeForm);
                }
            }
            GivenNameResolution::UnresolvedHistorical {
                possible_family_ids,
                recorded_form,
            } => {
                RenderedPersonalName::new(recorded_form.clone())?;
                for id in possible_family_ids {
                    family_exists(id)?;
                }
            }
            GivenNameResolution::Authored { full_name } => {
                RenderedPersonalName::new(full_name.clone())?;
            }
        }
        if let Some(id) = &self.surname_id
            && !catalog
                .surnames
                .iter()
                .any(|surname| surname.id == id.as_str())
        {
            return Err(NameCatalogError::MissingCatalogEntry(
                id.as_str().to_owned(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolved() -> PersonalNameIdentity {
        let form = catalog()
            .given_forms
            .iter()
            .find(|form| form.culture == Culture::German && form.register == NameRegister::Everyday)
            .unwrap();
        PersonalNameIdentity {
            given: GivenNameResolution::Resolved {
                family_id: NameFamilyId::new(form.family_ids[0].clone()),
                native_form_id: NameFormId::new(form.id.clone()),
            },
            native_culture: Culture::German,
            form_selector: NameFormSelectionSeed::new(fabelgeist_determinism::Seed::from_u64(0)),
            surname_id: None,
        }
    }

    fn native_render(
        identity: &PersonalNameIdentity,
    ) -> Result<RenderedPersonalName, NameCatalogError> {
        render_personal_name(
            identity,
            identity.native_culture,
            NameRegister::Everyday,
            Sex::Male,
        )
    }

    #[test]
    fn native_render_rejects_missing_family_and_mismatched_valid_references() {
        let mut identity = resolved();
        let GivenNameResolution::Resolved {
            family_id,
            native_form_id,
        } = &mut identity.given
        else {
            unreachable!();
        };
        let form = catalog()
            .given_forms
            .iter()
            .find(|form| form.id == native_form_id.as_str())
            .unwrap();
        *family_id = NameFamilyId::new("missing-family");
        assert!(matches!(
            native_render(&identity),
            Err(NameCatalogError::MissingCatalogEntry(_))
        ));
        let other = catalog()
            .given_families
            .iter()
            .find(|family| !form.family_ids.contains(&family.id))
            .unwrap();
        let GivenNameResolution::Resolved { family_id, .. } = &mut identity.given else {
            unreachable!()
        };
        *family_id = NameFamilyId::new(other.id.clone());
        assert_eq!(
            native_render(&identity),
            Err(NameCatalogError::InconsistentNativeForm)
        );
    }

    #[test]
    fn authored_display_still_validates_hereditary_metadata() {
        let mut identity = PersonalNameIdentity::authored("Authored Name", Culture::German);
        identity.surname_id = Some(SurnameId::new("missing-surname"));
        assert!(matches!(
            native_render(&identity),
            Err(NameCatalogError::MissingCatalogEntry(_))
        ));
        identity.surname_id = Some(SurnameId::new(catalog().surnames[0].id.clone()));
        assert_eq!(native_render(&identity).unwrap().as_str(), "Authored Name");
    }

    #[test]
    fn unresolved_names_preserve_evidence_but_reject_missing_candidates() {
        let mut identity = resolved();
        identity.given = GivenNameResolution::UnresolvedHistorical {
            possible_family_ids: vec![],
            recorded_form: "Recorded Spelling".into(),
        };
        assert_eq!(
            native_render(&identity).unwrap().as_str(),
            "Recorded Spelling"
        );
        let GivenNameResolution::UnresolvedHistorical {
            possible_family_ids,
            ..
        } = &mut identity.given
        else {
            unreachable!()
        };
        possible_family_ids.push(NameFamilyId::new("missing-family"));
        assert!(matches!(
            native_render(&identity),
            Err(NameCatalogError::MissingCatalogEntry(_))
        ));
    }
}

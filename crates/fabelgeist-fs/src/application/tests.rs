use super::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn identity_admission_rejects_namespace_separators_nul_and_unnamed_applications() {
    for (input, field, violation) in [
        (
            ("com", "studio", ""),
            ApplicationIdentityField::Application,
            ApplicationIdentityViolation::EmptyApplication,
        ),
        (
            ("com/other", "studio", "game"),
            ApplicationIdentityField::Qualifier,
            ApplicationIdentityViolation::PathSeparator,
        ),
        (
            ("com", "studio\\other", "game"),
            ApplicationIdentityField::Organization,
            ApplicationIdentityViolation::PathSeparator,
        ),
        (
            ("com", "studio", "game\0suffix"),
            ApplicationIdentityField::Application,
            ApplicationIdentityViolation::InteriorNul,
        ),
    ] {
        let error = ApplicationIdentity::try_from(input).unwrap_err();
        assert_eq!(error.field(), field);
        assert_eq!(error.violation(), violation);
    }
    let identity = ApplicationIdentity::try_from(("", "", "Game Ω")).unwrap();
    assert_eq!(identity.application, "Game Ω");
    assert_eq!(
        ApplicationIdentity::try_from(("com", "adventure-simulator-group", "fabelgeist")).unwrap(),
        ApplicationIdentity::fabelgeist()
    );
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test::wasm_bindgen_test]
async fn absent_browser_window_is_a_classifiable_application_directory_failure() {
    let failure = ApplicationIdentity::fabelgeist()
        .directory()
        .await
        .unwrap_err();
    assert!(matches!(failure, ApplicationDirectoryError::MissingWindow));
}

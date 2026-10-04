use super::*;

#[test]
fn browser_checkbox_form_accepts_one_or_many_sources_without_javascript() {
    assert_eq!(
        ForageForm::try_from(b"return_to=%2Flocations%2Fcamp&source=plants&hours=1".as_slice()),
        Ok(ForageForm {
            source: ForageSubmittedSources(vec![ForageSourceToken("plants".into())]),
            hours: ForageSubmittedHours(1),
            return_to: ForageReturnHint("/locations/camp".into()),
        })
    );
    assert_eq!(
        ForageForm::try_from(
            b"return_to=%2Flocations%2Fcamp&source=high_game&source=plants&hours=24".as_slice()
        ),
        Ok(ForageForm {
            source: ForageSubmittedSources(vec![
                ForageSourceToken("high_game".into()),
                ForageSourceToken("plants".into())
            ]),
            hours: ForageSubmittedHours(24),
            return_to: ForageReturnHint("/locations/camp".into()),
        })
    );
}

#[test]
fn browser_checkbox_form_preserves_none_and_duplicates_for_authoritative_validation() {
    assert_eq!(
        ForageForm::try_from(b"return_to=%2Flocations%2Fcamp&hours=1".as_slice()),
        Ok(ForageForm {
            source: ForageSubmittedSources::default(),
            hours: ForageSubmittedHours(1),
            return_to: ForageReturnHint("/locations/camp".into()),
        })
    );
    assert_eq!(
        ForageForm::try_from(
            b"return_to=%2Flocations%2Fcamp&source=plants&source=plants&hours=1".as_slice()
        )
        .unwrap()
        .source,
        ForageSubmittedSources(vec![
            ForageSourceToken("plants".into()),
            ForageSourceToken("plants".into())
        ])
    );
}

#[test]
fn browser_checkbox_form_rejects_ambiguous_scalar_fields() {
    assert!(
        ForageForm::try_from(
            b"return_to=%2Flocations%2Fcamp&return_to=%2Fother&source=plants&hours=1".as_slice()
        )
        .is_err()
    );
    assert!(
        ForageForm::try_from(
            b"return_to=%2Flocations%2Fcamp&source=plants&hours=1&hours=2".as_slice()
        )
        .is_err()
    );
}

#[test]
fn browser_checkbox_form_is_explicitly_bounded_before_authentication() {
    assert!(ForageForm::try_from(vec![b'x'; FORAGE_FORM_MAX_BYTES + 1].as_slice()).is_err());

    let too_many_pairs = format!(
        "return_to=%2Flocations%2Fcamp&hours=1{}",
        "&ignored=x".repeat(FORAGE_FORM_MAX_PAIRS - 1)
    );
    assert!(ForageForm::try_from(too_many_pairs.as_bytes()).is_err());

    let too_many_sources = format!(
        "return_to=%2Flocations%2Fcamp&hours=1{}",
        "&source=plants".repeat(FORAGE_FORM_MAX_SOURCES + 1)
    );
    assert!(ForageForm::try_from(too_many_sources.as_bytes()).is_err());

    let long_source = "x".repeat(FORAGE_FORM_MAX_SOURCE_LEN + 1);
    assert!(
        ForageForm::try_from(
            format!("return_to=%2Flocations%2Fcamp&hours=1&source={long_source}").as_bytes()
        )
        .is_err()
    );

    let long_return = "x".repeat(FORAGE_FORM_MAX_RETURN_TO_LEN + 1);
    assert!(
        ForageForm::try_from(format!("return_to=%2F{long_return}&hours=1").as_bytes()).is_err()
    );
}

#[test]
fn admission_retains_native_numeric_causes_and_scalar_roles() {
    use std::error::Error;
    let missing = ForageForm::try_from(b"return_to=%2F".as_slice()).unwrap_err();
    assert_eq!(
        missing,
        ForageFormError::MissingScalar(ForageScalarField::Hours)
    );
    let duplicate =
        ForageForm::try_from(b"return_to=%2F&hours=1&hours=nope".as_slice()).unwrap_err();
    assert_eq!(
        duplicate,
        ForageFormError::DuplicateScalar(ForageScalarField::Hours)
    );
    let overflow = ForageForm::try_from(b"return_to=%2F&hours=256".as_slice()).unwrap_err();
    assert!(matches!(overflow, ForageFormError::Hours(_)));
    let cause = overflow
        .source()
        .unwrap()
        .downcast_ref::<std::num::ParseIntError>()
        .unwrap();
    assert_eq!(*cause.kind(), std::num::IntErrorKind::PosOverflow);
}

#[test]
fn submitted_sources_and_hours_keep_reducer_admission_authority() {
    let form = ForageForm::try_from(
        b"source=pl%C3%A4nts&source=plants&source=plants&source=&return_to=%2F&hours=25".as_slice(),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(form.sources()).unwrap(),
        serde_json::json!(["plänts", "plants", "plants", ""])
    );
    for (input, minutes) in [
        ("0", 0),
        ("1", 60),
        ("24", 1440),
        ("25", 1500),
        ("255", 15300),
    ] {
        let body = format!("return_to=%2F&hours={input}");
        let form = ForageForm::try_from(body.as_bytes()).unwrap();
        assert_eq!(form.duration(), StrategicDuration::new(minutes));
        assert_eq!(
            serde_json::to_value(form.duration()).unwrap(),
            serde_json::json!(minutes)
        );
    }
}

#[test]
fn decoded_limits_are_utf8_bytes_and_capacity_precedes_token_admission() {
    let accepted = "é".repeat(FORAGE_FORM_MAX_SOURCE_LEN / 2);
    assert!(ForageSourceToken::try_from(accepted.as_str()).is_ok());
    assert_eq!(
        ForageSourceToken::try_from(format!("{accepted}x").as_str()),
        Err(ForageFormError::SourceTooLong)
    );
    let body = format!(
        "return_to=%2F&hours=1{}&source={}",
        "&source=plants".repeat(5),
        "x".repeat(33)
    );
    assert_eq!(
        ForageForm::try_from(body.as_bytes()),
        Err(ForageFormError::TooManySources)
    );
    let mut sources = ForageSubmittedSources::default();
    for _ in 0..FORAGE_FORM_MAX_SOURCES {
        sources
            .try_push(ForageSourceToken::try_from("plants").unwrap())
            .unwrap();
    }
    assert_eq!(
        sources.try_push(ForageSourceToken::try_from("plants").unwrap()),
        Err(ForageFormError::TooManySources)
    );
    assert_eq!(
        serde_json::to_value(sources).unwrap(),
        serde_json::json!(["plants", "plants", "plants", "plants", "plants"])
    );
}

#[test]
fn decoded_return_hints_resolve_before_typed_navigation() {
    let external =
        ForageForm::try_from(b"return_to=https%3A%2F%2Fexample.com&hours=1".as_slice()).unwrap();
    assert_eq!(external.return_destination(), LocalReturnUrl::ROOT);
    let local = ForageForm::try_from(
        b"return_to=%2Flocations%2Fcamp%3Ftab%3Dfood%23forage&hours=1".as_slice(),
    )
    .unwrap();
    assert_eq!(
        local.return_destination().to_string(),
        "/locations/camp?tab=food#forage"
    );
}

use super::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn suffix_admission_preserves_case_compounds_and_the_leading_dot() {
    for spelling in ["png", ".png", "PNG", ".tar.gz", "c++", "abcdefghijklmno"] {
        let suffix = FileSuffix::try_from(spelling).unwrap();
        let body = spelling.strip_prefix('.').unwrap_or(spelling);
        assert_eq!(suffix.0, format!(".{body}"));
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn malformed_suffixes_retain_their_original_spelling_and_violation() {
    for (spelling, violation) in [
        ("", FileSuffixViolation::Empty),
        (".", FileSuffixViolation::Empty),
        ("png.", FileSuffixViolation::TrailingDot),
        ("*.png", FileSuffixViolation::InvalidCharacter),
        ("dir/png", FileSuffixViolation::InvalidCharacter),
        ("dir\\png", FileSuffixViolation::InvalidCharacter),
        ("p ng", FileSuffixViolation::InvalidCharacter),
        ("p\0ng", FileSuffixViolation::InvalidCharacter),
        ("é", FileSuffixViolation::InvalidCharacter),
        ("abcdefghijklmnop", FileSuffixViolation::TooLong),
        (".abcdefghijklmnop", FileSuffixViolation::TooLong),
    ] {
        let error = FileSuffix::try_from(spelling).unwrap_err();
        assert_eq!(error.violation(), violation);
        assert_eq!(error.value, spelling);
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn a_type_filter_requires_suffixes_and_preserves_the_authored_group() {
    assert_eq!(
        FileSuffixes::try_from(Vec::new()).unwrap_err(),
        FileSuffixesError
    );
    let first = FileSuffix::try_from("PNG").unwrap();
    let second = FileSuffix::try_from("png").unwrap();
    let values = vec![first.clone(), second, first];
    let group = FileSuffixes::try_from(values.clone()).unwrap();
    assert_eq!(group.0, values);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn native_provider_extensions_are_dotless_and_keep_compounds_and_case() {
    let suffixes = FileSuffixes::try_from(vec![
        FileSuffix::try_from(".PNG").unwrap(),
        FileSuffix::try_from("tar.gz").unwrap(),
        FileSuffix::try_from("c++").unwrap(),
    ])
    .unwrap();
    assert_eq!(suffixes.native_extensions(), ["PNG", "tar.gz", "c++"]);
}

use super::*;
use crate::FileText;
use std::error::Error;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn locator_admission_and_mime_classification_preserve_address_identity() {
    for (spelling, violation) in [
        ("", ResourceLocatorViolation::Empty),
        ("bad\0path", ResourceLocatorViolation::InteriorNul),
        (
            "ftp://host/file",
            ResourceLocatorViolation::UnsupportedScheme,
        ),
    ] {
        assert_eq!(
            ResourceLocator::try_from(spelling).unwrap_err().violation(),
            violation
        );
    }
    let resource = ResourceLocator::try_from("prism://project/résumé.PNG").unwrap();
    assert_eq!(resource.as_ref(), "prism://project/résumé.PNG");
    assert_eq!(resource.mime(), ResourceMime::Png);
    assert_eq!(
        ResourceLocator::try_from("model.glb").unwrap().mime(),
        ResourceMime::Binary
    );
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
async fn inline_payloads_preserve_bytes_and_classify_failures() {
    let encoded = ResourceLocator::try_from("data:application/octet-stream;base64,AP8=").unwrap();
    assert_eq!(encoded.read().await.unwrap().as_ref(), &[0, 255]);
    assert_eq!(encoded.inline_data().await.unwrap(), encoded);
    let escaped = ResourceLocator::try_from("data:text/plain,%00%FFhttp://host").unwrap();
    assert_eq!(escaped.read().await.unwrap().as_ref(), b"\0\xffhttp://host");
    let invalid = ResourceLocator::try_from("data:application/octet-stream;base64,???").unwrap();
    let failure = invalid.read().await.unwrap_err();
    assert_eq!(failure.resource, invalid);
    assert_eq!(failure.operation, ResourceOperation::Read);
    assert!(matches!(failure.cause, ResourceFailure::Base64(_)));
    assert!(
        failure
            .source()
            .unwrap()
            .source()
            .unwrap()
            .is::<base64::DecodeError>()
    );
    let missing = ResourceLocator::try_from("data:text/plain")
        .unwrap()
        .read()
        .await
        .unwrap_err();
    assert!(matches!(
        missing.cause,
        ResourceFailure::MissingDataSeparator
    ));
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
async fn immutable_resource_routes_reject_writes_without_losing_the_address() {
    let contents = FileContents::from(vec![0, 255]);
    for spelling in [
        "data:text/plain,value",
        "https://host/file",
        "blob:opaque-address",
    ] {
        let resource = ResourceLocator::try_from(spelling).unwrap();
        let failure = resource.write(&contents).await.unwrap_err();
        assert_eq!(failure.resource, resource);
        assert_eq!(failure.operation, ResourceOperation::Write);
        assert!(matches!(failure.cause, ResourceFailure::ReadOnly));
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn utf8_admission_retains_spelling_and_rejected_bytes() {
    let text = FileText::try_from(FileContents::from("résumé\r\n".as_bytes().to_vec())).unwrap();
    assert_eq!(text.as_ref(), "résumé\r\n");
    let failure = FileText::try_from(FileContents::from(vec![0, 255])).unwrap_err();
    assert_eq!(
        failure
            .source()
            .unwrap()
            .downcast_ref::<std::string::FromUtf8Error>()
            .unwrap()
            .as_bytes(),
        &[0, 255]
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn native_resource_round_trip_and_error_context_are_exact() {
    let storage = tempfile::tempdir().unwrap();
    let path = storage.path().join("payload.bin");
    let resource = ResourceLocator::try_from(path.to_str().unwrap()).unwrap();
    let contents = FileContents::from(vec![0, 1, 255, 128]);
    resource.write(&contents).await.unwrap();
    assert_eq!(resource.read().await.unwrap(), contents);
    let inline = resource.inline_data().await.unwrap();
    assert_eq!(inline.read().await.unwrap(), contents);
    let missing =
        ResourceLocator::try_from(storage.path().join("missing").to_str().unwrap()).unwrap();
    let failure = missing.read().await.unwrap_err();
    assert_eq!(failure.resource, missing);
    assert_eq!(failure.operation, ResourceOperation::Read);
    assert_eq!(
        failure
            .source()
            .unwrap()
            .source()
            .unwrap()
            .downcast_ref::<std::io::Error>()
            .unwrap()
            .kind(),
        std::io::ErrorKind::NotFound
    );
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
async fn project_resources_require_their_namespace_authority() {
    assert!(crate::ProjectRoot::current().is_none());
    let resource = ResourceLocator::try_from("prism://project/missing-root-file").unwrap();
    assert!(matches!(
        resource.read().await.unwrap_err().cause,
        ResourceFailure::MissingProjectRoot
    ));
    assert!(matches!(
        resource
            .write(&FileContents::default())
            .await
            .unwrap_err()
            .cause,
        ResourceFailure::MissingProjectRoot
    ));
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn admitted_project_segments_reach_the_native_provider_without_raw_names() {
    let storage = tempfile::tempdir().unwrap();
    let root = crate::ProjectRoot::from(crate::NativeDirectory::from(storage.path().to_path_buf()));
    let resource = ResourceLocator::try_from("prism://project/nested/payload.bin").unwrap();
    let path = resource.project_path(&root).unwrap().unwrap();
    let file = path
        .resolve(root.clone(), crate::EntryLookupIntent::CreateIfMissing)
        .await
        .unwrap()
        .unwrap();
    let contents = FileContents::from(vec![0, 255]);
    file.write(&contents).await.unwrap();
    assert_eq!(file.read().await.unwrap(), contents);
    let invalid = ResourceLocator::try_from("prism://project/nested/../escape").unwrap();
    let failure = match invalid.project_path(&root) {
        Err(failure) => failure,
        Ok(_) => panic!("traversal was admitted"),
    };
    assert!(matches!(failure, ResourceFailure::ChildName(_)));
    let invalid_encoding = ResourceLocator::try_from("prism://project/%FF.bin").unwrap();
    let failure = match invalid_encoding.project_path(&root) {
        Err(failure) => failure,
        Ok(_) => panic!("invalid UTF-8 path was admitted"),
    };
    assert!(matches!(failure, ResourceFailure::PathEncoding(_)));
    assert!(failure.source().unwrap().is::<std::str::Utf8Error>());
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn native_resource_addresses_match_namespace_components_and_retain_the_absolute_root() {
    let storage = tempfile::tempdir().unwrap();
    let directory = storage.path().join("root");
    std::fs::create_dir(&directory).unwrap();
    let root = crate::ProjectRoot::from(crate::NativeDirectory::from(directory.clone()));
    let resource = ResourceLocator::try_from(
        format!("file://{}/nested/payload.bin", directory.display()).as_str(),
    )
    .unwrap();
    let file = resource
        .project_path(&root)
        .unwrap()
        .unwrap()
        .resolve(root.clone(), crate::EntryLookupIntent::CreateIfMissing)
        .await
        .unwrap()
        .unwrap();
    let contents = FileContents::from(vec![0, 255, 1]);
    file.write(&contents).await.unwrap();
    assert_eq!(
        std::fs::read(directory.join("nested/payload.bin")).unwrap(),
        contents.as_ref()
    );
    let sibling = ResourceLocator::try_from(
        format!("file://{}-sibling/payload.bin", directory.display()).as_str(),
    )
    .unwrap();
    assert!(sibling.project_path(&root).unwrap().is_none());
    let traversal =
        ResourceLocator::try_from(format!("file://{}/../escape", directory.display()).as_str())
            .unwrap();
    assert!(matches!(
        traversal.project_path(&root),
        Err(ResourceFailure::ChildName(_))
    ));
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn native_file_failures_preserve_the_file_operation_and_io_cause() {
    use crate::FileEntry;
    let storage = tempfile::tempdir().unwrap();
    let file = crate::NativeFile::from(storage.path().join("missing-parent").join("payload"));
    for (failure, expected) in [
        (file.read().await.unwrap_err(), crate::FileOperation::Read),
        (
            file.write(&FileContents::default()).await.unwrap_err(),
            crate::FileOperation::Write,
        ),
    ] {
        assert!(
            matches!(&failure, crate::FileContentError::Native { operation, .. } if *operation == expected)
        );
        assert_eq!(
            failure
                .source()
                .unwrap()
                .downcast_ref::<std::io::Error>()
                .unwrap()
                .kind(),
            std::io::ErrorKind::NotFound
        );
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn malformed_http_requests_retain_the_provider_error() {
    let resource = ResourceLocator::try_from("http://[").unwrap();
    let failure = resource.read().await.unwrap_err();
    assert_eq!(failure.resource, resource);
    assert!(matches!(failure.cause, ResourceFailure::HttpRequest(_)));
    assert!(
        failure
            .source()
            .unwrap()
            .source()
            .unwrap()
            .downcast_ref::<reqwest::Error>()
            .unwrap()
            .is_builder()
    );
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test::wasm_bindgen_test]
async fn cached_blob_failures_retain_the_first_cause_and_distinguish_reuse() {
    use super::error::FetchFailureOrigin;
    let resource = ResourceLocator::try_from("blob:content-io-retention-test").unwrap();
    let first = resource.read().await.unwrap_err();
    let second = resource.read().await.unwrap_err();
    match (first.cause, second.cause) {
        (
            ResourceFailure::RetainedFetch {
                source: first,
                origin: FetchFailureOrigin::Attempt,
            },
            ResourceFailure::RetainedFetch {
                source: second,
                origin: FetchFailureOrigin::Cache,
            },
        ) => {
            assert!(std::sync::Arc::ptr_eq(&first, &second));
            assert!(matches!(first.as_ref(), ResourceFailure::MissingWindow));
        }
        _ => panic!("blob failure was not retained"),
    }
}

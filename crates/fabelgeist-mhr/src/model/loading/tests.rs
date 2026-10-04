use super::*;
use crate::character::fixture::{RigCase, RigFixture};
use crate::{CharacterDecodeError, ModelDefinitionError, RigBlendShapeCount};
use crate::{MhrAsset, MhrAssetReadError};
use fabelgeist_fs::{EntryName, ResourceFailure};

#[derive(Clone, Copy)]
enum AssetFixtureCase {
    InvalidRigAndDefinition,
    BadCorrectiveArchives,
    MissingCorrectiveArchives,
}
struct AssetFiles(tempfile::TempDir);
impl AssetFiles {
    fn from_case(case: AssetFixtureCase) -> Self {
        let storage = tempfile::tempdir().unwrap();
        let (rig, definition) = match case {
            AssetFixtureCase::InvalidRigAndDefinition => {
                (FileContents::default(), vec![0xff, 0xfe])
            }
            _ => (
                RigFixture::from_case(RigCase::Valid).bytes(),
                b"Momentum Model Definition V1.0\n[ParameterTransform]\n".to_vec(),
            ),
        };
        std::fs::write(storage.path().join("lod4.fbx"), rig.as_ref()).unwrap();
        std::fs::write(storage.path().join("compact_v6_1.model"), definition).unwrap();
        if matches!(case, AssetFixtureCase::BadCorrectiveArchives) {
            std::fs::write(storage.path().join("corrective_blendshapes_lod4.npz"), []).unwrap();
            std::fs::write(storage.path().join("corrective_activation.npz"), []).unwrap();
        }
        Self(storage)
    }
    fn native(&self) -> MhrAssetDirectory {
        MhrAssetDirectory::from(self.0.path().to_path_buf())
    }
    fn resource(&self) -> ResourceLocator {
        ResourceLocator::try_from(self.0.path().to_str().unwrap()).unwrap()
    }
}

#[test]
fn native_loading_keeps_rig_before_definition_admission_and_decoder_causes() {
    let files = AssetFiles::from_case(AssetFixtureCase::InvalidRigAndDefinition);
    let error = Mhr::from_files(&files.native(), MhrConfig::default(), &Device::default())
        .err()
        .unwrap();
    assert!(matches!(
        &error,
        MhrLoadError::Rig(CharacterDecodeError::Fbx(_))
    ));
    let character_error = std::error::Error::source(&error).unwrap();
    assert!(character_error.is::<CharacterDecodeError>());
    assert!(
        character_error
            .source()
            .unwrap()
            .is::<fabelgeist_fbx::FbxDecodeError>()
    );
}

#[test]
fn native_corrective_decoding_keeps_basis_before_activation_and_zip_cause() {
    let files = AssetFiles::from_case(AssetFixtureCase::BadCorrectiveArchives);
    let error = Mhr::from_files(&files.native(), MhrConfig::default(), &Device::default())
        .err()
        .unwrap();
    assert!(matches!(
        &error,
        MhrLoadError::CorrectiveArchive {
            role: CorrectiveArchiveRole::Basis,
            ..
        }
    ));
    assert!(
        std::error::Error::source(&error)
            .unwrap()
            .is::<fabelgeist_numpy_storage::ZipReadError>()
    );
}

#[tokio::test]
async fn provider_lookup_orders_keep_their_distinct_corrective_roles() {
    let files = AssetFiles::from_case(AssetFixtureCase::MissingCorrectiveArchives);
    let native = Mhr::from_files(&files.native(), MhrConfig::default(), &Device::default())
        .err()
        .unwrap();
    assert!(matches!(
        native,
        MhrLoadError::Asset(MhrAssetReadError::Native {
            asset: MhrAsset::CorrectiveBasis(_),
            ..
        })
    ));
    let resource = Mhr::from_uri(&files.resource(), MhrConfig::default(), &Device::default())
        .await
        .err()
        .unwrap();
    assert!(matches!(
        resource,
        MhrLoadError::Asset(MhrAssetReadError::Lookup {
            asset: MhrAsset::CorrectiveActivation,
            ..
        })
    ));
}

#[test]
fn byte_loading_keeps_missing_and_invalid_correctives_before_definition_parsing() {
    let rig = RigFixture::from_case(RigCase::Valid).bytes();
    let definition = FileText::from("invalid definition".to_owned());
    let missing = Mhr::from_asset_bytes(
        &rig,
        &definition,
        None,
        MhrConfig::default(),
        &Device::default(),
    )
    .err()
    .unwrap();
    assert!(matches!(
        missing,
        MhrLoadError::Asset(MhrAssetReadError::MissingCorrectiveArchives)
    ));
    let invalid = Mhr::from_asset_bytes(
        &rig,
        &definition,
        Some((FileContents::default(), FileContents::default())),
        MhrConfig::default(),
        &Device::default(),
    )
    .err()
    .unwrap();
    assert!(matches!(
        &invalid,
        MhrLoadError::CorrectiveNetwork(crate::CorrectiveDecodeError::Archive {
            role: CorrectiveArchiveRole::Activation,
            ..
        })
    ));
    assert!(
        std::error::Error::source(&invalid)
            .unwrap()
            .source()
            .unwrap()
            .is::<fabelgeist_numpy_storage::ZipReadError>()
    );
}

#[test]
fn disabled_correctives_preserve_definition_errors_and_rig_layout_admission() {
    let rig = RigFixture::from_case(RigCase::Valid).bytes();
    let config = MhrConfig {
        pose_correctives: PoseCorrectivePolicy::Disabled,
        ..MhrConfig::default()
    };
    let definition_error = Mhr::from_asset_bytes(
        &rig,
        &FileText::from("invalid definition".to_owned()),
        Some((FileContents::default(), FileContents::default())),
        config,
        &Device::default(),
    )
    .err()
    .unwrap();
    assert!(matches!(
        &definition_error,
        MhrLoadError::Definition(ModelDefinitionError::InvalidHeader(_))
    ));
    assert!(
        std::error::Error::source(&definition_error)
            .unwrap()
            .is::<ModelDefinitionError>()
    );
    let layout_error = Mhr::from_asset_bytes(
        &rig,
        &FileText::from("Momentum Model Definition V1.0\n[ParameterTransform]\n".to_owned()),
        None,
        config,
        &Device::default(),
    )
    .err()
    .unwrap();
    match layout_error {
        MhrLoadError::ModelLayout(source) => {
            assert_eq!(source.shapes, RigBlendShapeCount::from(0));
            assert_eq!(
                source.to_string(),
                "rig has 0 blend shapes, expected 45 identity plus 72 expression"
            );
        }
        other => panic!("wrong error: {other:?}"),
    }
}

#[tokio::test]
async fn resource_admission_preserves_invalid_definition_bytes_before_decoding_the_rig() {
    let storage = tempfile::tempdir().unwrap();
    std::fs::write(storage.path().join("lod4.fbx"), []).unwrap();
    let payload = vec![0xff, 0xfe, 0x00];
    std::fs::write(storage.path().join("compact_v6_1.model"), &payload).unwrap();
    let directory = ResourceLocator::try_from(storage.path().to_str().unwrap()).unwrap();
    let failure = match Mhr::from_uri(
        &directory,
        MhrConfig {
            lod: crate::CharacterLod::Detailed,
            pose_correctives: PoseCorrectivePolicy::Disabled,
        },
        &Device::default(),
    )
    .await
    {
        Err(failure) => failure,
        Ok(_) => panic!("invalid model-definition text was accepted"),
    };
    match failure {
        MhrLoadError::Asset(MhrAssetReadError::ModelDefinitionText(source)) => {
            let encoding = std::error::Error::source(&source)
                .unwrap()
                .downcast_ref::<std::string::FromUtf8Error>()
                .unwrap();
            assert_eq!(encoding.as_bytes(), payload);
        }
        _ => panic!("expected model-definition admission before FBX decoding"),
    }
}

#[tokio::test]
async fn asset_lookup_retains_both_attempts_and_their_original_causes() {
    let storage = tempfile::tempdir().unwrap();
    let directory = ResourceLocator::try_from(storage.path().to_str().unwrap()).unwrap();
    let failure = match Mhr::from_uri(&directory, MhrConfig::default(), &Device::default()).await {
        Err(failure) => failure,
        Ok(_) => panic!("missing assets were accepted"),
    };
    match failure {
        MhrLoadError::Asset(MhrAssetReadError::Lookup {
            asset,
            direct,
            nested,
        }) => {
            assert_eq!(asset, MhrAsset::Rig(crate::CharacterLod::Detailed));
            assert_eq!(asset.file_name(), EntryName::try_from("lod4.fbx").unwrap());
            assert_eq!(
                direct.resource,
                ResourceLocator::try_from(storage.path().join("lod4.fbx").to_str().unwrap())
                    .unwrap()
            );
            assert_eq!(
                nested.resource,
                ResourceLocator::try_from(storage.path().join("assets/lod4.fbx").to_str().unwrap())
                    .unwrap()
            );
            for failure in [direct, nested] {
                assert!(
                    matches!(&failure.cause, ResourceFailure::Native(source) if source.kind() == std::io::ErrorKind::NotFound)
                );
            }
        }
        _ => panic!("expected both asset resource lookup failures"),
    }
}

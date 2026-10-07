//! Whole-texture format selection through the actual public GPU API.
#![cfg(not(target_arch = "wasm32"))]

use std::{error::Error, sync::Arc};

use fabelgeist_gpu::{globals::WgpuContext, prelude::*};
use serde::Deserialize;

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Method {
    Plane,
    Volume,
    Cube,
}

#[derive(Deserialize)]
struct NativeCase {
    method: Method,
    native_prefix: String,
}

#[derive(Deserialize)]
struct Fixtures {
    stored: TextureFormat,
    requested: TextureFormat,
    uninitialized_request: TextureFormat,
    cases: [NativeCase; 3],
}

impl Fixtures {
    fn from_json() -> Self {
        serde_json::from_str(include_str!("fixtures/texture_views.json")).unwrap()
    }
}

enum TextureFixture {
    Plane(Texture2d),
    Volume(Texture3d),
    Cube(TextureCube),
}

impl TextureFixture {
    fn empty(method: Method) -> Self {
        match method {
            Method::Plane => Self::Plane(Texture2d::default()),
            Method::Volume => Self::Volume(Texture3d::default()),
            Method::Cube => Self::Cube(TextureCube::default()),
        }
    }

    fn new(context: &WgpuContext, method: Method, format: TextureFormat) -> Self {
        // Authored tiny pixel extents enter the existing public constructor ports.
        match method {
            Method::Plane => {
                Self::Plane(Texture2d::create(context, Vec2::new(2.0, 2.0), format).unwrap())
            }
            Method::Volume => {
                Self::Volume(Texture3d::new(context, Vec3::new(2.0, 2.0, 2.0), format).unwrap())
            }
            Method::Cube => Self::Cube(TextureCube::new(context, 2.0, format).unwrap()),
        }
    }

    fn format(&self) -> TextureFormat {
        match self {
            Self::Plane(texture) => texture.format,
            Self::Volume(texture) => texture.format,
            Self::Cube(texture) => texture.format,
        }
    }

    fn view(
        &self,
        context: &WgpuContext,
        requested: TextureFormat,
    ) -> TextureViewResult<Arc<wgpu::TextureView>> {
        match self {
            Self::Plane(texture) => texture.view_with_format(context, requested),
            Self::Volume(texture) => texture.view_with_format(context, requested),
            Self::Cube(texture) => texture.view_with_format(context, requested),
        }
    }

    fn clear_view(&mut self) {
        match self {
            Self::Plane(texture) => texture.view = None,
            Self::Volume(texture) => texture.view = None,
            Self::Cube(texture) => texture.view = None,
        }
    }

    fn clear_native_texture(&mut self) {
        match self {
            Self::Plane(texture) => texture.texture = None,
            Self::Volume(texture) => texture.texture = None,
            Self::Cube(texture) => texture.texture = None,
        }
    }
}

#[tokio::test]
async fn missing_cache_returns_errors_before_native_texture_admission() {
    let context = WgpuContext::new_compute().await.unwrap();
    let fixtures = Fixtures::from_json();
    for case in fixtures.cases {
        let empty = TextureFixture::empty(case.method);
        let error = empty.view(&context, empty.format()).unwrap_err();
        assert!(matches!(error, TextureViewError::MissingView));
        assert_eq!(error.to_string(), "Texture has no view");
        assert!(error.source().is_none());

        let mut created = TextureFixture::new(&context, case.method, fixtures.stored);
        let cached = created.view(&context, fixtures.stored).unwrap();
        created.clear_native_texture();
        let still_cached = created.view(&context, fixtures.stored).unwrap();
        assert!(Arc::ptr_eq(&cached, &still_cached));

        let mut cleared = TextureFixture::new(&context, case.method, fixtures.stored);
        cleared.clear_view();
        let error = cleared.view(&context, fixtures.stored).unwrap_err();
        assert!(matches!(error, TextureViewError::MissingView));
        assert!(error.source().is_none());
    }
}

#[tokio::test]
async fn different_format_requires_the_native_texture() {
    let context = WgpuContext::new_compute().await.unwrap();
    let fixtures = Fixtures::from_json();
    for case in fixtures.cases {
        let empty = TextureFixture::empty(case.method);
        assert_ne!(empty.format(), fixtures.uninitialized_request);
        let error = empty
            .view(&context, fixtures.uninitialized_request)
            .unwrap_err();
        assert!(matches!(error, TextureViewError::Uninitialized));
        assert_eq!(error.to_string(), "Texture is not initialized");
        assert!(error.source().is_none());
    }
}

#[tokio::test]
async fn native_rejections_retain_the_request_and_real_cause_through_context() {
    fn assert_error_contract<E: Error + Send + Sync + 'static>() {}
    assert_error_contract::<TextureViewError>();
    let context = WgpuContext::new_compute().await.unwrap();
    let fixtures = Fixtures::from_json();
    for case in fixtures.cases {
        let texture = TextureFixture::new(&context, case.method, fixtures.stored);
        let outer_scope = context
            .device
            .push_error_scope(wgpu::ErrorFilter::Validation);
        let error = texture.view(&context, fixtures.requested).unwrap_err();
        let cause = match (&case.method, &error) {
            (Method::Plane, TextureViewError::Texture2d { requested, cause })
            | (Method::Volume, TextureViewError::Texture3d { requested, cause })
            | (Method::Cube, TextureViewError::TextureCube { requested, cause }) => {
                assert_eq!(*requested, fixtures.requested);
                cause.as_ref()
            }
            _ => panic!("wrong public method classification: {error:?}"),
        };
        let source = error.source().unwrap();
        let native_source = source.downcast_ref::<wgpu::Error>().unwrap();
        assert!(std::ptr::eq(native_source, cause));
        assert!(source.is::<wgpu::Error>());
        assert!(error.to_string().starts_with(&case.native_prefix));
        assert!(error.to_string().ends_with(&cause.to_string()));
        assert!(cause.to_string().contains("view of format Rgba8Unorm"));
        assert!(outer_scope.pop().await.is_none());

        let generic = anyhow::Error::new(error).context("Texture caller");
        assert!(generic.downcast_ref::<TextureViewError>().is_some());
        // The outer static context retains its own native message identity.
        assert_eq!(generic.downcast_ref::<&str>(), Some(&"Texture caller"));
        assert!(generic.downcast_ref::<String>().is_none());
        assert!(generic.chain().any(|cause| cause.is::<wgpu::Error>()));
    }
}

#[tokio::test]
async fn cached_and_alternate_views_preserve_storage_policy_and_sharing() {
    let context = WgpuContext::new_compute().await.unwrap();
    let fixtures = Fixtures::from_json();
    for case in fixtures.cases {
        let texture = TextureFixture::new(&context, case.method, TextureFormat::Rgba8Unorm);
        let cached = texture.view(&context, TextureFormat::Rgba8Unorm).unwrap();
        let shared = texture.view(&context, TextureFormat::Rgba8Unorm).unwrap();
        let alternate = texture.view(&context, fixtures.requested).unwrap();
        assert!(Arc::ptr_eq(&cached, &shared));
        assert!(!Arc::ptr_eq(&cached, &alternate));
        assert_eq!(cached.texture(), alternate.texture());
        assert!(
            alternate
                .texture()
                .usage()
                .contains(wgpu::TextureUsages::STORAGE_BINDING)
        );
    }
}

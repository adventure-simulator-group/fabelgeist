use super::{ApplicationDirectoryError, ApplicationIdentity, BrowserApplicationStage};
use crate::{DirectoryEntry, WebDirectory};
use std::sync::Arc;
use wasm_bindgen::{JsCast, JsValue};

impl ApplicationIdentity {
    pub(super) async fn origin_directory(
        &self,
    ) -> Result<Arc<dyn DirectoryEntry>, ApplicationDirectoryError> {
        let window = web_sys::window().ok_or(ApplicationDirectoryError::MissingWindow)?;
        let mut receiver: JsValue = window.into();
        for (property, stage) in [
            ("navigator", BrowserApplicationStage::Navigator),
            ("storage", BrowserApplicationStage::Storage),
        ] {
            receiver = js_sys::Reflect::get(&receiver, &property.into()).map_err(
                |source: JsValue| -> ApplicationDirectoryError {
                    ApplicationDirectoryError::from_browser(stage, source)
                },
            )?;
        }
        let method = js_sys::Reflect::get(&receiver, &"getDirectory".into()).map_err(
            |source: JsValue| -> ApplicationDirectoryError {
                ApplicationDirectoryError::from_browser(BrowserApplicationStage::ReadMethod, source)
            },
        )?;
        let method = method.dyn_into::<js_sys::Function>().map_err(
            |source: JsValue| -> ApplicationDirectoryError {
                ApplicationDirectoryError::from_browser(
                    BrowserApplicationStage::DecodeMethod,
                    source,
                )
            },
        )?;
        let promise =
            method
                .call0(&receiver)
                .map_err(|source: JsValue| -> ApplicationDirectoryError {
                    ApplicationDirectoryError::from_browser(
                        BrowserApplicationStage::InvokeMethod,
                        source,
                    )
                })?;
        let promise = promise.dyn_into::<js_sys::Promise>().map_err(
            |source: JsValue| -> ApplicationDirectoryError {
                ApplicationDirectoryError::from_browser(
                    BrowserApplicationStage::DecodePromise,
                    source,
                )
            },
        )?;
        let handle = wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .map_err(|source: JsValue| -> ApplicationDirectoryError {
                ApplicationDirectoryError::from_browser(
                    BrowserApplicationStage::AwaitDirectory,
                    source,
                )
            })?;
        let handle = handle
            .dyn_into::<web_sys::FileSystemDirectoryHandle>()
            .map_err(|source: JsValue| -> ApplicationDirectoryError {
                ApplicationDirectoryError::from_browser(
                    BrowserApplicationStage::DecodeDirectory,
                    source,
                )
            })?;
        Ok(Arc::new(WebDirectory::from(handle)))
    }
}
impl ApplicationDirectoryError {
    fn from_browser(stage: BrowserApplicationStage, source: JsValue) -> Self {
        Self::Browser {
            stage,
            source: crate::BrowserIoCause::from(source),
        }
    }
}
